//! A playable canine-like source/filter prototype, not a calibrated anatomical dog model.
//!
//! Moore (Interspeech 2016, doi:10.21437/Interspeech.2016-841) motivates separating
//! breath, phonation and mouth articulation. The 17 cm supraglottal tract is an initial
//! design hypothesis near his medium-mammal scaling, not a measured breed profile.
//! Areas and call envelopes are tuning parameters; perceptual validation is still required.

use super::build::{CMH2O, fill_areas};
use super::mammal::{MammalAirway, MammalSpec, mammal_airway, mammal_noise, step_glottis};
use super::types::{
    CallDef, CallKey, CreatureDef, CreatureGroup, CreatureSim, Curve, GateTimes, ParamDef,
};
use crate::airway::network::{AirwayNetwork, Tube};
use crate::math::{PI, js_min, pow, sin};

const PITCH: usize = 0;
const MOUTH: usize = 1;
const ROUGH: usize = 2;
const TONE: usize = 3;
const PRESSURE: usize = 4;
const BRIGHT: usize = 5;
const NASAL: usize = 6;

struct Dog {
    a: MammalAirway,
    tubes: Vec<(&'static str, Tube)>,
}

pub(super) fn airway(fs: f64) -> MammalAirway {
    mammal_airway(
        fs,
        MammalSpec {
            trachea: (18.0, 1.7),
            pharynx: (6.0, 1.8),
            oral: (11.0, 2.6),
            nasal: Some((13.0, 0.9)),
            wall: None,
            wall_radiation: Some(0.15),
            coarse: None,
            nasal_radiates: None,
        },
        |_, _| {},
    )
}

fn create(fs: f64) -> Box<dyn CreatureSim> {
    let mut a = airway(fs);
    a.glottis_src.params.drift = 0.0;
    a.glottis_src.params.vibrato_depth = 0.0;
    let mut tubes = vec![
        ("trachea", a.trachea),
        ("pharynx", a.pharynx),
        ("oral", a.oral),
    ];
    if let Some(nasal) = a.nasal {
        tubes.push(("nasal", nasal));
    }
    Box::new(Dog { a, tubes })
}

impl CreatureSim for Dog {
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
        6.0e-8
    }

    fn control(&mut self, p: &[f64], gate: f64, _dt: f64) {
        let a = &mut self.a;
        let tone = p[TONE];
        let rough = p[ROUGH];
        let g = &mut a.glottis_src.params;
        g.f0 = p[PITCH];
        g.amplitude = 0.18;
        g.open_quotient = 0.42 + 0.22 * tone - 0.12 * p[BRIGHT];
        g.skew = 0.68;
        g.closure_sharpness = 0.55 + 0.45 * p[BRIGHT];
        g.effort = 0.7 + 0.3 * (1.0 - tone);
        g.chink = 0.003 + 0.012 * (1.0 - tone);
        g.jitter = 0.001 + 0.035 * rough;
        g.shimmer = 0.005 + 0.12 * rough;
        g.chaos = rough;
        a.net
            .set_fixed_pressure(a.lungs, p[PRESSURE] * CMH2O * gate);
        if let Some(velum) = a.velum {
            a.net.set_port_target(velum, p[NASAL] * 0.5);
        }
        let ph = fill_areas(a.pharynx, |u| 1.2 + 1.0 * sin(PI * u), &mut a.phar_area);
        a.net.set_tube_targets(a.pharynx, ph, None, None);
        // A wide cavity feeding a narrower mouth: opening changes the filter independently
        // of note pitch, rather than shifting all formants with playback rate.
        let mouth = 0.12 + 3.5 * p[MOUTH];
        let oral = fill_areas(
            a.oral,
            |u| {
                if u < 0.65 {
                    2.6 + 0.6 * sin(PI * u / 0.65)
                } else {
                    2.6 + (mouth - 2.6) * (u - 0.65) / 0.35
                }
            },
            &mut a.oral_area,
        );
        a.net.set_tube_targets(a.oral, oral, None, None);
        mammal_noise(a, 1.0e-6 + 5.0e-6 * (1.0 - tone), None);
    }

    fn sample(&mut self) -> f64 {
        step_glottis(&mut self.a);
        self.a.net.step()
    }

    fn source_state(&self, out: &mut Vec<(&'static str, f64)>) {
        out.clear();
        out.push(("glottis", js_min(1.0, self.a.glottis_src.area / 0.3)));
    }
}

/// Fixed public-domain melody fragment: Twinkle's first phrase, C C G G A A G.
/// Authored once as call automation, so playback stays on the creature's audio clock.
fn tune(howl: bool) -> CallDef {
    let mut keys = Vec::new();
    let beat = 0.5;
    let notes = [
        (0.0, 1.0),
        (0.0, 1.0),
        (7.0, 1.0),
        (7.0, 1.0),
        (9.0, 1.0),
        (9.0, 1.0),
        (7.0, 2.0),
    ];
    let mut t = 0.0;
    for (semitones, beats) in notes {
        let pitch = 261.625565 * pow(2.0, semitones / 12.0);
        if howl {
            keys.push(
                CallKey::new(
                    t,
                    &[
                        ("gate", 1.0),
                        ("pitch", pitch),
                        ("tone", 1.0),
                        ("mouth", 0.13),
                        ("rough", 0.015),
                    ],
                )
                .curve(Curve::Step),
            );
        } else {
            keys.extend(bark_keys(t, pitch));
        }
        t += beats * beat;
    }
    keys.push(CallKey::new(t, &[("gate", 0.0)]).curve(Curve::Step));
    CallDef::new(
        if howl { "howlTune" } else { "barkTune" },
        if howl { "Howl a tune" } else { "Bark a tune" },
        "Twinkle's first phrase at 120 BPM; keys transpose the whole phrase",
        Some(261.625565),
        keys,
    )
}

fn bark_keys(t: f64, pitch: f64) -> Vec<CallKey> {
    vec![
        CallKey::new(
            t,
            &[
                ("gate", 0.0),
                ("pitch", pitch * 1.15),
                ("mouth", 0.18),
                ("tone", 0.0),
                ("rough", 0.72),
            ],
        )
        .curve(Curve::Step),
        CallKey::new(t + 0.012, &[("gate", 1.15), ("mouth", 0.85)]),
        CallKey::new(
            t + 0.065,
            &[("gate", 0.95), ("pitch", pitch), ("mouth", 0.65)],
        )
        .curve(Curve::Exp),
        CallKey::new(
            t + 0.17,
            &[("gate", 0.25), ("pitch", pitch * 0.85), ("mouth", 0.22)],
        ),
        CallKey::new(t + 0.25, &[("gate", 0.0), ("mouth", 0.12)]),
    ]
}

pub fn def() -> CreatureDef {
    let k = CallKey::new;
    CreatureDef {
        id: "dog-model", name: "Dog model", latin: "Experimental physical model", group: CreatureGroup::Mammal,
        blurb: "A canine-like prototype: short rough barks, sustained howls and stylized yodels. Try both tune buttons, or choose what the keys play.",
        role: "Pitched barks and howling melodies · experimental timbre",
        oversample: 2,
        gate_times: Some(GateTimes { attack: 0.004, release: 0.018 }),
        params: vec![
            ParamDef::new("pitch", "Pitch", 100.0, 1200.0, 330.0).unit("Hz").log().smooth(0.008).pitch(),
            ParamDef::new("mouth", "Mouth opening", 0.0, 1.0, 0.13).smooth(0.012),
            ParamDef::new("rough", "Roughness", 0.0, 0.85, 0.015).smooth(0.008),
            ParamDef::new("tone", "Howl character", 0.0, 1.0, 1.0).smooth(0.008),
            ParamDef::new("pressure", "Breath pressure", 3.0, 20.0, 9.0).unit("cmH₂O").smooth(0.01),
            ParamDef::new("bright", "Brightness", 0.0, 1.0, 0.4).smooth(0.015),
            ParamDef::new("nasal", "Nasal coupling", 0.0, 1.0, 0.08).smooth(0.02),
        ],
        calls: vec![
            CallDef::new("bark", "Bark", "A short rough bark with a falling pitch and closing mouth", Some(330.0), bark_keys(0.0, 330.0)),
            CallDef::new("howl", "Howl", "A sustained tonal howl with a gentle rise and fall", Some(330.0), vec![
                k(0.0, &[("gate", 0.0), ("pitch", 270.0), ("tone", 1.0), ("mouth", 0.1), ("rough", 0.015)]),
                k(0.12, &[("gate", 0.8)]), k(0.4, &[("pitch", 330.0), ("mouth", 0.16)]).curve(Curve::Exp),
                k(1.8, &[("gate", 0.9), ("pitch", 340.0)]), k(2.3, &[("pitch", 290.0), ("mouth", 0.08)]).curve(Curve::Exp),
                k(2.5, &[("gate", 0.0)]),
            ]),
            CallDef::new("yodel", "Yodel", "Stylized pitch and timbre alternation, not a validated biological register model", Some(330.0), vec![
                k(0.0, &[("gate", 1.0), ("pitch", 330.0), ("tone", 0.45), ("mouth", 0.4), ("rough", 0.16)]),
                k(0.28, &[("pitch", 660.0), ("tone", 1.0), ("mouth", 0.12), ("rough", 0.015)]).curve(Curve::Step),
                k(0.5, &[("pitch", 330.0), ("tone", 0.45), ("mouth", 0.4), ("rough", 0.16)]).curve(Curve::Step),
                k(0.78, &[("pitch", 660.0), ("tone", 1.0), ("mouth", 0.12), ("rough", 0.015)]).curve(Curve::Step),
                k(1.0, &[("pitch", 330.0), ("tone", 0.45), ("mouth", 0.4), ("rough", 0.16)]).curve(Curve::Step),
                k(1.28, &[("pitch", 660.0), ("tone", 1.0), ("mouth", 0.12), ("rough", 0.015)]).curve(Curve::Step),
                k(1.55, &[("gate", 0.0)]),
            ]),
            tune(false), tune(true),
        ], events: vec![], create,
    }.resolved()
}
