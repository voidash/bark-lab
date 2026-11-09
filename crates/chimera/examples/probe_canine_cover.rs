//! Standalone pressure/tissue causality probe; no recordings or acoustic waveform input.
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::sources::canine_cover::CanineCover;
use serde_json::json;

fn main() -> Result<(), String> {
    let mut rows = Vec::new();
    for hz in [170.0, 330.0, 600.0] {
        for p in [0.0, 12000.0, 18000.0, 28000.0] {
            let mut f = CanineCover::new(96000.0);
            f.configure(hz, 1.0, 0.015, 0.7)
                .map_err(|e| format!("config {e:?}"))?;
            let mut a = Vec::new();
            for i in 0..96000 {
                let v = f.step(p, 0.0).map_err(|e| format!("{hz}/{p}/{i}: {e:?}"))?;
                if i >= 48000 && i % 2 == 0 {
                    a.push(v);
                }
            }
            let mean = a.iter().sum::<f64>() / a.len() as f64;
            let rms = (a.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / a.len() as f64).sqrt();
            let pitch = yin_pitch(
                &a[..4096],
                48000.0,
                YinOpts {
                    min_f0: Some(50.0),
                    max_f0: Some(1600.0),
                    ..Default::default()
                },
            );
            rows.push(json!({"nominalHz":hz,"pressureDynCm2":p,"areaAcRms":rms,"f0":pitch.f0,"aperiodicity":pitch.aperiodicity,"speed":f.speed()}));
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())?
    );
    Ok(())
}
