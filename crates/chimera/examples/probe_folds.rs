//! Probe the mechanical source alone, without a tract or any audio samples.
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::sources::two_mass::TwoMassFolds;

fn main() -> Result<(), String> {
    for tension in [1.0, 2.0, 3.0] {
        for pressure in [0.0, 8000.0, 16000.0, 24000.0] {
            let mut source = TwoMassFolds::new(96000.0);
            source
                .configure(tension, 1.0, 0.05, 1.0)
                .map_err(|e| format!("configure: {e:?}"))?;
            let mut x = Vec::new();
            for i in 0..96000 {
                let area = source
                    .step(pressure, 0.0)
                    .map_err(|e| format!("Q={tension}, P={pressure}, sample {i}: {e:?}"))?;
                if i >= 48000 && i % 2 == 0 {
                    x.push(area);
                }
            }
            let mean = x.iter().sum::<f64>() / x.len() as f64;
            let rms = (x.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / x.len() as f64).sqrt();
            let f = yin_pitch(
                &x[..4096],
                48000.0,
                YinOpts {
                    min_f0: Some(50.0),
                    max_f0: Some(1500.0),
                    ..Default::default()
                },
            );
            println!(
                "Q={tension}, P={pressure}: area-rms {rms:.6}, pitch {:.1}, aperiodicity {:.3}, speed {:.1}",
                f.f0,
                f.aperiodicity,
                source.speed()
            );
        }
    }
    Ok(())
}
