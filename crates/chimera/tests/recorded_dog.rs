//! Preserve the actual canine source, natural bark ending, sustained howl and honest telemetry.
use chimera::creatures::recorded_dog::{BARK_PCM, SOURCE_RATE, pcm_sample};
use chimera::creatures::{CreatureVoice, creature_by_id};

fn rms(audio: &[f32]) -> f64 {
    (audio.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / audio.len() as f64).sqrt()
}

#[test]
fn bark_preserves_the_recorded_waveform_at_two_pitches() {
    let def = creature_by_id("dog").unwrap();
    for pitch in [330.0, 660.0] {
        let mut voice = CreatureVoice::new(
            def,
            48000.0,
            &[
                (def.param_index("voice").unwrap(), 0.0),
                (def.param_index("pitch").unwrap(), pitch),
            ],
        );
        voice.set_gate(1.0);
        let mut audio = vec![0.0; 12000];
        voice.render(&mut audio);
        // Allow the common decimator's group delay; compare the source's broadband waveform,
        // not merely its energy or an estimated fundamental (which a toy oscillator can match).
        let step = SOURCE_RATE / 48000.0 * pitch / 330.0;
        let start = 1000;
        let end = (4000.0 / step) as usize;
        let best = (0..64)
            .map(|delay| {
                let (mut xy, mut xx, mut yy) = (0.0, 0.0, 0.0);
                for i in start..end {
                    let pos = (i - delay) as f64 * step;
                    let k = pos as usize;
                    let f = pos - k as f64;
                    let x = pcm_sample(BARK_PCM, k) * (1.0 - f) + pcm_sample(BARK_PCM, k + 1) * f;
                    let y = f64::from(audio[i]);
                    xy += x * y;
                    xx += x * x;
                    yy += y * y;
                }
                xy / (xx * yy).sqrt()
            })
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            best > 0.95,
            "recorded bark lost at {pitch} Hz: correlation {best}"
        );
    }
}

#[test]
fn held_bark_ends_and_held_howl_sustains_without_fake_pressure() {
    let def = creature_by_id("dog").unwrap();
    for fs in [44100.0, 48000.0, 96000.0] {
        for kind in [0.0, 1.0] {
            let mut voice =
                CreatureVoice::new(def, fs, &[(def.param_index("voice").unwrap(), kind)]);
            voice.set_gate(1.0);
            let mut audio = vec![0.0; (fs * 6.0) as usize];
            voice.render(&mut audio);
            assert!(audio.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
            if kind == 0.0 {
                assert!(
                    rms(&audio[(fs * 0.7) as usize..]) < 1e-5,
                    "held bark must end naturally"
                );
            } else {
                for window in audio[(fs * 0.8) as usize..].chunks((fs * 0.1) as usize) {
                    assert!(rms(window) > 0.005, "sustained howl has a dropout at {fs}");
                }
            }
            let mut telemetry = voice.new_telemetry();
            voice.telemetry(&mut telemetry);
            assert!(telemetry.out_rms > 0.001);
            assert!(
                telemetry
                    .tubes
                    .iter()
                    .all(|t| t.rms.iter().all(|&x| x == 0.0)),
                "recordings must not fabricate simulated pressure"
            );
            voice.set_gate(0.0);
            let mut tail = vec![0.0; (fs * 0.8) as usize];
            voice.render(&mut tail);
            assert!(rms(&tail[tail.len() / 2..]) < 1e-5);
        }
    }
}

#[test]
fn every_recorded_call_releases_and_is_block_independent() {
    let def = creature_by_id("dog").unwrap();
    for call in &def.calls {
        let mut reference = Vec::new();
        for block in [128, 77, 4096] {
            let mut voice = CreatureVoice::new(def, 48000.0, &[]);
            voice.play_call_id(call.id, None, 1.0);
            let mut audio = vec![0.0; ((call.keys.last().unwrap().t + 0.8) * 48000.0) as usize];
            for part in audio.chunks_mut(block) {
                voice.render(part);
            }
            assert!(voice.active_call().is_none());
            assert!(rms(&audio) > 0.005, "{} is silent", call.id);
            assert!(
                rms(&audio[audio.len() - 4800..]) < 1e-5,
                "{} fails to release",
                call.id
            );
            if reference.is_empty() {
                reference = audio;
            } else {
                assert_eq!(audio, reference, "{} changes at block {block}", call.id);
            }
        }
    }
}
