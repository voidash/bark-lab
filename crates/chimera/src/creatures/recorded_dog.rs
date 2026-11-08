//! Recognizable canine voice from CC0 recordings. The airway is an illustrative pose only:
//! no simulated pressure is presented as the cause of this recorded-source audio.
//! Source URLs, licence verification and reproducible conversion: data/README.md.

use super::dog::airway;
use super::mammal::MammalAirway;
use super::types::{
    CallDef, CallKey, CreatureDef, CreatureGroup, CreatureSim, Curve, GateTimes, ParamDef,
};
use crate::airway::network::{AirwayNetwork, Tube};
use crate::math::{PI, TAU, cos, exp, pow};

pub const SOURCE_RATE: f64 = 24000.0;
pub const BARK_PCM: &[u8] = include_bytes!("data/dog-bark.pcm");
pub const HOWL_PCM: &[u8] = include_bytes!("data/dog-howl.pcm");
const PITCH: usize = 0;
const KIND: usize = 1;
const BRIGHT: usize = 2;
const LEVEL: usize = 3;
const ROOT: f64 = 330.0;

/// A nominal note centre for transposition. Natural recordings retain their pitch contours;
/// it is not a promise that a noisy bark has a fundamental of exactly ROOT Hz.
pub struct RecordedDog {
    a: MammalAirway,
    tubes: Vec<(&'static str, Tube)>,
    kind: usize,
    position: f64,
    step: f64,
    looping: bool,
    event_pending: bool,
    previous_gate: f64,
    level: f64,
    brightness: f64,
    low: f64,
    filter_alpha: f64,
    envelope: f64,
    envelope_decay: f64,
    fs: f64,
}

fn create(fs: f64) -> Box<dyn CreatureSim> {
    let a = airway(fs);
    let mut tubes = vec![
        ("trachea", a.trachea),
        ("pharynx", a.pharynx),
        ("oral", a.oral),
    ];
    if let Some(nasal) = a.nasal {
        tubes.push(("nasal", nasal));
    }
    Box::new(RecordedDog {
        a,
        tubes,
        kind: 1,
        position: 0.0,
        step: SOURCE_RATE / fs,
        looping: true,
        event_pending: false,
        previous_gate: 0.0,
        level: 0.0,
        brightness: 0.5,
        low: 0.0,
        filter_alpha: 1.0 - exp(-TAU * 1800.0 / fs),
        envelope: 0.0,
        envelope_decay: exp(-1.0 / (0.025 * fs)),
        fs,
    })
}

pub fn pcm_sample(pcm: &[u8], index: usize) -> f64 {
    let i = index * 2;
    f64::from(i16::from_le_bytes([pcm[i], pcm[i + 1]])) / 32768.0
}

fn read(pcm: &[u8], position: f64) -> f64 {
    let frames = pcm.len() / 2;
    if position < 0.0 || position >= (frames - 1) as f64 {
        return 0.0;
    }
    let i = position as usize;
    let fraction = position - i as f64;
    pcm_sample(pcm, i) * (1.0 - fraction) + pcm_sample(pcm, i + 1) * fraction
}

impl RecordedDog {
    fn restart(&mut self, kind: usize, looping: bool) {
        self.kind = kind;
        self.position = 0.0;
        self.looping = looping;
        self.low = 0.0;
    }
}

impl CreatureSim for RecordedDog {
    fn net(&self) -> Option<&AirwayNetwork> {
        Some(&self.a.net)
    }
    fn net_mut(&mut self) -> Option<&mut AirwayNetwork> {
        Some(&mut self.a.net)
    }
    fn tubes(&self) -> &[(&'static str, Tube)] {
        &self.tubes
    }
    fn gain(&self) -> f64 {
        0.6
    }

    fn control(&mut self, p: &[f64], gate: f64, _dt: f64) {
        let kind = usize::from(p[KIND] >= 0.5);
        // Restoring performer parameters at a call's end must not start a new voice
        // while the old gate is releasing.
        if !self.event_pending
            && ((gate > 0.01 && self.previous_gate <= 0.01)
                || (kind != self.kind && gate > 0.01 && gate >= self.previous_gate))
        {
            self.restart(kind, kind == 1);
        }
        self.event_pending = false;
        self.previous_gate = gate;
        self.step = SOURCE_RATE / self.fs * p[PITCH] / ROOT;
        self.level = gate * p[LEVEL];
        self.brightness = p[BRIGHT];
        // Schematic geometry only: never step this network or fabricate pressure telemetry.
        // Keep its areas initialized for the anatomical view, with zero acoustic pressure.
        self.a.net.snap_to_targets();
    }

    fn trigger(&mut self, event: usize, _amount: f64) {
        match event {
            0 => self.restart(0, false),
            1 => self.restart(1, false),
            2 => self.restart(1, true),
            _ => return,
        }
        self.event_pending = true;
    }

    fn sample(&mut self) -> f64 {
        self.envelope *= self.envelope_decay;
        if self.level < 1e-6 {
            return 0.0;
        }
        let pcm = if self.kind == 0 { BARK_PCM } else { HOWL_PCM };
        let mut value = read(pcm, self.position);
        if self.kind == 1 && self.looping {
            // Loop only the steady part of the howl, retaining its natural onset. A source-
            // time crossfade avoids a hard splice at every wrap, at every playback rate.
            let start = 0.65 * SOURCE_RATE;
            let end = 1.20 * SOURCE_RATE;
            let fade = 0.08 * SOURCE_RATE;
            if self.position >= end - fade {
                let u = (self.position - (end - fade)) / fade;
                let blend = 0.5 - 0.5 * cos(PI * u);
                value =
                    value * (1.0 - blend) + read(pcm, start + self.position - (end - fade)) * blend;
            }
            self.position += self.step;
            if self.position >= end {
                self.position = start + fade + self.position - end;
            }
        } else {
            self.position = (self.position + self.step).min(pcm.len() as f64 / 2.0);
        }
        self.low += self.filter_alpha * (value - self.low);
        // Neutral brightness preserves the recording. The control supplies gentle timbral
        // variation, independent of the illustrative tract, rather than a fictitious anatomy fit.
        value += (self.brightness - 0.5) * 0.8 * (value - self.low);
        value *= self.level;
        self.envelope = self.envelope.max(value.abs());
        value
    }

    fn source_state(&self, out: &mut Vec<(&'static str, f64)>) {
        out.clear();
        out.push(("recording", self.envelope.min(1.0)));
    }
}

fn tune(howl: bool) -> CallDef {
    let mut keys = Vec::new();
    let mut t = 0.0;
    for (i, (semitones, beats)) in [
        (0.0, 1.0),
        (0.0, 1.0),
        (7.0, 1.0),
        (7.0, 1.0),
        (9.0, 1.0),
        (9.0, 1.0),
        (7.0, 2.0),
    ]
    .iter()
    .enumerate()
    {
        let pitch = 261.625565 * pow(2.0, semitones / 12.0);
        let mut values = vec![
            ("gate", 1.0),
            ("pitch", pitch),
            ("voice", if howl { 1.0 } else { 0.0 }),
        ];
        if !howl {
            values.push(("!bark", 1.0));
        } else if i == 0 {
            values.push(("!howlLoop", 1.0));
        }
        keys.push(CallKey::new(t, &values).curve(Curve::Step));
        t += beats * 0.5;
    }
    keys.push(CallKey::new(t, &[("gate", 0.0)]).curve(Curve::Step));
    CallDef::new(
        if howl { "howlTune" } else { "barkTune" },
        if howl { "Howl a tune" } else { "Bark a tune" },
        "Real dog voice, transposed through Twinkle's first phrase",
        Some(261.625565),
        keys,
    )
}

pub fn def() -> CreatureDef {
    let k = CallKey::new;
    CreatureDef {
        id: "dog", name: "Dog reference", latin: "Canis familiaris", group: CreatureGroup::Mammal,
        blurb: "Real dog recordings: a clean bark and a husky howl, with pitch and phrasing controls. The tract drawing is schematic. The synthetic attempt is kept under Dog model.",
        role: "Recorded canine voice · CC0 sources · schematic vocal anatomy",
        oversample: 2, gate_times: Some(GateTimes { attack: 0.002, release: 0.018 }),
        params: vec![
            ParamDef::new("pitch", "Pitch centre", 165.0, 1000.0, ROOT).unit("Hz").log().smooth(0.008).pitch(),
            ParamDef::new("voice", "Howl (0 = bark)", 0.0, 1.0, 1.0).smooth(0.001).digits(0),
            ParamDef::new("bright", "Brightness", 0.0, 1.0, 0.5).smooth(0.02),
            ParamDef::new("level", "Intensity", 0.2, 1.5, 1.0).smooth(0.01),
        ],
        calls: vec![
            CallDef::new("bark", "Bark", "An actual recorded dog bark", Some(ROOT), vec![k(0.0, &[("gate", 1.0), ("pitch", ROOT), ("voice", 0.0), ("!bark", 1.0)]).curve(Curve::Step), k(0.55, &[("gate", 0.0)]).curve(Curve::Step)]),
            CallDef::new("howl", "Howl", "An actual husky howl with its natural onset and ending", Some(ROOT), vec![k(0.0, &[("gate", 1.0), ("pitch", ROOT), ("voice", 1.0), ("!howl", 1.0)]).curve(Curve::Step), k(2.25, &[("gate", 0.0)]).curve(Curve::Step)]),
            CallDef::new("yodel", "Yodel", "A husky howl with fast octave changes", Some(ROOT), vec![
                k(0.0, &[("gate", 1.0), ("pitch", ROOT), ("voice", 1.0), ("!howlLoop", 1.0)]).curve(Curve::Step),
                k(0.55, &[("pitch", 660.0)]).curve(Curve::Step), k(0.8, &[("pitch", 330.0)]).curve(Curve::Step),
                k(1.05, &[("pitch", 660.0)]).curve(Curve::Step), k(1.3, &[("pitch", 330.0)]).curve(Curve::Step),
                k(1.55, &[("pitch", 660.0)]).curve(Curve::Step), k(1.9, &[("gate", 0.0)]).curve(Curve::Step),
            ]), tune(false), tune(true),
        ], events: vec!["bark", "howl", "howlLoop"], create,
    }.resolved()
}
