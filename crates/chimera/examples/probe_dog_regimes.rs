//! A bounded interaction experiment: detuning × acoustic load × pressure, fixed note.
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureVoice, creature_by_id};
use serde_json::json;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let focus = args.iter().any(|a| a == "--focus");
    let canine = args.iter().any(|a| a == "--canine");
    if args.iter().any(|a| a != "--focus" && a != "--canine") || args.len() > 2 {
        return Err("usage: probe_dog_regimes [--focus] [--canine]".into());
    }
    let def = creature_by_id("dog-physics").ok_or("physical dog missing")?;
    let index = |key| {
        def.param_index(key)
            .ok_or_else(|| format!("missing control {key}"))
    };
    let source_model = index("sourceModel")?;
    let (rough, throat, pressure, tone, mouth, pitch, damping) = (
        index("rough")?,
        index("throat")?,
        index("pressure")?,
        index("tone")?,
        index("mouth")?,
        index("pitch")?,
        index("damping")?,
    );
    let mut results = Vec::new();
    let detunes: &[f64] = if focus {
        &[0.7, 0.85]
    } else {
        &[-0.7, 0.0, 0.3, 0.5, 0.7, 0.85]
    };
    let loads: &[f64] = if focus {
        &[0.3, 0.4, 0.6]
    } else {
        &[0.2, 0.4, 0.8, 1.2]
    };
    let pressures: &[f64] = if focus {
        &[24.0, 28.0, 32.0]
    } else {
        &[12.0, 20.0, 28.0]
    };
    let dampings: &[f64] = if focus { &[0.7, 1.0, 1.4] } else { &[1.0] };
    for &detune in detunes {
        for &load in loads {
            for &breath in pressures {
                for &damp in dampings {
                    let mut voice = CreatureVoice::new(
                        def,
                        48000.0,
                        &[
                            (source_model, if canine { 1.0 } else { 0.0 }),
                            (rough, detune),
                            (throat, load),
                            (pressure, breath),
                            (tone, 0.0),
                            (mouth, 0.65),
                            (pitch, 330.0),
                            (damping, damp),
                        ],
                    );
                    voice.set_gate(1.0);
                    let mut audio = vec![0.0f32; 36000];
                    voice.render(&mut audio);
                    let x: Vec<_> = audio[24000..].iter().map(|&x| f64::from(x)).collect();
                    let p = yin_pitch(
                        &x[..4096],
                        48000.0,
                        YinOpts {
                            min_f0: Some(65.0),
                            max_f0: Some(1600.0),
                            ..Default::default()
                        },
                    );
                    let rms = (x.iter().map(|x| x * x).sum::<f64>() / x.len() as f64).sqrt();
                    let mut telemetry = voice.new_telemetry();
                    voice.telemetry(&mut telemetry);
                    let fault = telemetry
                        .source
                        .iter()
                        .any(|(k, v)| *k == "fault" && *v != 0.0);
                    let mut periods = Vec::new();
                    for at in [18000, 24000, 30000] {
                        let frame: Vec<_> =
                            audio[at..at + 4096].iter().map(|&v| f64::from(v)).collect();
                        periods.push(
                            yin_pitch(
                                &frame,
                                48000.0,
                                YinOpts {
                                    min_f0: Some(65.0),
                                    max_f0: Some(1600.0),
                                    ..Default::default()
                                },
                            )
                            .aperiodicity,
                        );
                    }
                    periods.sort_by(f64::total_cmp);
                    results.push(json!({"sourceModel":if canine {1} else {0},"detuning":detune,"throat":load,"pressure":breath,"damping":damp,"rms":rms,"f0":p.f0,"aperiodicity":p.aperiodicity,"medianAperiodicity":periods[1],"fault":fault}));
                }
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&results).map_err(|e| format!("metrics: {e}"))?
    );
    Ok(())
}
