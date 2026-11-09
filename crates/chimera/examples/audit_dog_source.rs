//! Causal source ablations. Diagnostic overrides never change the live model.
use chimera::airway::network::{AirwayNetwork, Tube};
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureSim, CreatureVoice, creature_by_id};
use serde_json::json;
use std::fs;
use std::path::Path;

struct FrozenFolds {
    inner: Box<dyn CreatureSim>,
    glottis: usize,
    rest_area: f64,
    source: Vec<(&'static str, f64)>,
}

impl CreatureSim for FrozenFolds {
    fn net(&self) -> Option<&AirwayNetwork> {
        self.inner.net()
    }
    fn net_mut(&mut self) -> Option<&mut AirwayNetwork> {
        self.inner.net_mut()
    }
    fn tubes(&self) -> &[(&'static str, Tube)] {
        self.inner.tubes()
    }
    fn gain(&self) -> f64 {
        self.inner.gain()
    }
    fn control(&mut self, p: &[f64], gate: f64, dt: f64) {
        self.inner.control(p, gate, dt);
        self.inner.source_state(&mut self.source);
        // Verified named state from PhysicalDog, not a guessed port/parameter index.
        self.rest_area = self
            .source
            .iter()
            .find(|(k, _)| *k == "restArea")
            .expect("dog rest area disappeared")
            .1;
    }
    fn sample(&mut self) -> f64 {
        let net = self
            .inner
            .net_mut()
            .expect("dog acoustic network disappeared");
        net.set_port_area(self.glottis, self.rest_area);
        net.step()
    }
    fn source_state(&self, out: &mut Vec<(&'static str, f64)>) {
        self.inner.source_state(out);
        out.retain(|(key, _)| *key == "fault");
        out.extend([("frozenDiagnostic", 1.0), ("glottalArea", self.rest_area)]);
    }
}

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

fn write_wav(path: &Path, audio: &[f32], gain: f64) -> Result<(), String> {
    let bytes = (audio.len() * 2) as u32;
    let mut data = Vec::with_capacity(audio.len() * 2 + 44);
    data.extend_from_slice(b"RIFF");
    data.extend_from_slice(&(bytes + 36).to_le_bytes());
    data.extend_from_slice(b"WAVEfmt ");
    data.extend_from_slice(&16u32.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&48000u32.to_le_bytes());
    data.extend_from_slice(&96000u32.to_le_bytes());
    data.extend_from_slice(&2u16.to_le_bytes());
    data.extend_from_slice(&16u16.to_le_bytes());
    data.extend_from_slice(b"data");
    data.extend_from_slice(&bytes.to_le_bytes());
    for &v in audio {
        let sample = ((f64::from(v) * gain).clamp(-1.0, 1.0) * 32767.0).round() as i16;
        data.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, data).map_err(|e| format!("{}: {e}", path.display()))
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 && !(args.len() == 2 && args[1] == "--canine") {
        return Err("usage: audit_dog_source OUTPUT [--canine]".into());
    }
    let root = Path::new(&args[0]);
    fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let def = creature_by_id("dog-physics").ok_or("physical dog missing")?;
    let mut reports = Vec::new();
    let calls: &[&str] = if args.len() == 2 {
        &["bark", "howl", "roughBark"]
    } else {
        &["bark", "howl"]
    };
    for &call in calls {
        let short = call != "howl";
        let mut cases = Vec::new();
        for (name, frozen, noise) in [
            ("full", false, true),
            ("no-noise", false, false),
            ("frozen", true, true),
            ("frozen-no-noise", true, false),
        ] {
            let mode = def
                .param_index("sourceModel")
                .ok_or("source version control missing")?;
            let mut voice = CreatureVoice::new(
                def,
                48000.0,
                &[(mode, if args.len() == 2 { 1.0 } else { 0.0 })],
            );
            if frozen {
                let glottis = voice
                    .sim
                    .net()
                    .ok_or("dog network missing")?
                    .noise_sources()
                    .first()
                    .ok_or("no registered glottal noise source")?
                    .src;
                let inner = std::mem::replace(&mut voice.sim, def.create(voice.fs));
                voice.sim = Box::new(FrozenFolds {
                    inner,
                    glottis,
                    rest_area: 0.0,
                    source: Vec::with_capacity(32),
                });
            }
            voice
                .sim
                .net_mut()
                .ok_or("dog network missing")?
                .noise_scale = if noise { 1.0 } else { 0.0 };
            voice.play_call_id(call, None, 1.0);
            let mut audio = vec![0.0; if short { 38400 } else { 153600 }];
            let mut source = Vec::with_capacity(32);
            let mut trace = Vec::new();
            let trace_start = if short { 2880 } else { 48000 };
            let (mut min_area, mut max_area, mut max_speed) = (f64::INFINITY, 0.0f64, 0.0f64);
            for (sample, block) in audio.chunks_mut(1).enumerate() {
                voice.render(block);
                voice.sim.source_state(&mut source);
                for &(key, value) in &source {
                    if !value.is_finite() {
                        return Err(format!("non-finite {key} in {call}/{name}"));
                    }
                    match key {
                        "fault" if value != 0.0 => {
                            return Err(format!("mechanical fault in {call}/{name}"));
                        }
                        "glottalArea" => {
                            min_area = min_area.min(value);
                            max_area = max_area.max(value);
                        }
                        "glottis" => {
                            max_speed = max_speed.max(value);
                        }
                        _ => {}
                    }
                }
                if name == "full"
                    && sample >= trace_start
                    && sample < trace_start + 1920
                    && sample % 4 == 0
                {
                    let value = |key| {
                        source
                            .iter()
                            .find(|(k, _)| *k == key)
                            .map(|(_, v)| *v)
                            .ok_or_else(|| format!("missing source state {key}"))
                    };
                    trace.push([
                        sample as f64 / 48000.0,
                        value("glottalArea")?,
                        value("glottalFlow")?,
                    ]);
                }
            }
            if !audio.iter().all(|v| v.is_finite()) {
                return Err(format!("non-finite audio in {call}/{name}"));
            }
            let at = if short { 1920 } else { 48000 };
            let frame: Vec<_> = audio[at..at + 4096].iter().map(|&v| f64::from(v)).collect();
            let pitch = yin_pitch(
                &frame,
                48000.0,
                YinOpts {
                    min_f0: Some(65.0),
                    max_f0: Some(1600.0),
                    ..Default::default()
                },
            );
            let window = if short {
                &audio[480..12000]
            } else {
                &audio[38400..86400]
            };
            let report = json!({"call":call,"case":name,"rms":rms(window),"f0Estimate":pitch.f0,"yinAperiodicity":pitch.aperiodicity,"areaMinCm2":min_area,"areaMaxCm2":max_area,"maxSourceSpeedGlyph":max_speed,"frozen":frozen,"noiseEnabled":noise,"traceColumns":["timeSeconds","glottalAreaCm2","glottalFlowCm3PerSecond"],"trace":trace});
            cases.push((name, audio, report));
        }
        let peak = cases
            .iter()
            .flat_map(|(_, a, _)| a.iter())
            .map(|v| f64::from(v.abs()))
            .fold(1e-12, f64::max);
        for (name, a, mut report) in cases {
            write_wav(&root.join(format!("{call}-{name}.wav")), &a, 0.85 / peak)?;
            report["commonGain"] = json!(0.85 / peak);
            reports.push(report);
        }
    }
    let report = json!({"sourceModel":if args.len() == 2 {"canine-body-cover"} else {"legacy-four-mass"},"method":"Exact live call plans. Crossed fold-motion/noise ablations, common gain within each call. Frozen aperture follows configured rest area without tissue dynamics; changing source changes flow/feedback, so RMS differences are not an additive noise fraction. YIN on near-silence is not evidence of a periodic source. These are diagnostics, not alternate canine voices.","cases":reports});
    fs::write(
        root.join("source-audit.json"),
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("source audit: {e}"))?;
    for case in report["cases"]
        .as_array()
        .ok_or("invalid report structure")?
    {
        println!(
            "{} / {}: rms {}, f0 {}, aperiodicity {}",
            case["call"], case["case"], case["rms"], case["f0Estimate"], case["yinAperiodicity"]
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
