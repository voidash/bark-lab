//! Pitch response of the two existing dog voices, with no source/recording substitution.
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureVoice, creature_by_id};
use serde_json::json;

fn main() -> Result<(), String> {
    let mut rows = Vec::new();
    for id in ["dog", "dog-physics"] {
        let def = creature_by_id(id).ok_or("dog voice missing")?;
        for (mouth, throat) in if id == "dog-physics" {
            vec![(0.13, 1.2), (0.4, 1.8), (0.65, 1.8)]
        } else {
            vec![(0.13, 1.2)]
        } {
            for nominal in [
                170.0, 210.0, 260.0, 300.0, 350.0, 400.0, 450.0, 500.0, 600.0,
            ] {
                if nominal < def.params[0].min {
                    continue;
                }
                let settings: &[(&str, f64)] = if id == "dog-physics" {
                    &[
                        ("sourceModel", 1.0),
                        ("tone", 1.0),
                        ("rough", 0.0),
                        ("pressure", 16.0),
                        ("damping", 1.0),
                        ("mouth", 0.13),
                        ("aspiration", 0.05),
                    ]
                } else {
                    &[("voice", 1.0), ("bright", 0.5), ("level", 1.0)]
                };
                let mut initial = settings
                    .iter()
                    .map(|(k, v)| {
                        def.param_index(k)
                            .map(|i| (i, *v))
                            .ok_or_else(|| format!("missing {id}/{k}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                initial.push((def.param_index("pitch").ok_or("pitch missing")?, nominal));
                if id == "dog-physics" {
                    initial.push((def.param_index("mouth").ok_or("mouth missing")?, mouth));
                    initial.push((def.param_index("throat").ok_or("throat missing")?, throat));
                }
                let mut voice = CreatureVoice::new(def, 48000.0, &initial);
                voice.set_gate(1.0);
                let mut audio = vec![0.0; 48000];
                voice.render(&mut audio);
                let frame: Vec<_> = audio[24000..28096].iter().map(|&v| f64::from(v)).collect();
                let p = yin_pitch(
                    &frame,
                    48000.0,
                    YinOpts {
                        min_f0: Some(70.0),
                        max_f0: Some(1400.0),
                        ..Default::default()
                    },
                );
                let rms = (frame.iter().map(|v| v * v).sum::<f64>() / frame.len() as f64).sqrt();
                let mut tele = voice.new_telemetry();
                voice.telemetry(&mut tele);
                if tele.source.iter().any(|(k, v)| *k == "fault" && *v != 0.0) {
                    return Err(format!("fault {id}/{nominal}"));
                }
                rows.push(json!({"voice":id,"mouth":mouth,"throat":throat,"nominalHz":nominal,"f0":p.f0,"aperiodicity":p.aperiodicity,"rms":rms}));
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?
    );
    Ok(())
}
