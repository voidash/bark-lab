//! Acoustic and runtime checks for the canine prototype; these cannot judge dog-likeness.

use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureVoice, creature_by_id};

fn rms(audio: &[f32]) -> f64 {
    (audio.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / audio.len() as f64).sqrt()
}

fn call(id: &str, block: usize) -> Vec<f32> {
    let def = creature_by_id("dog-model").expect("dog is registered");
    let call = &def.calls[def.call_index(id).expect("dog call exists")];
    let mut voice = CreatureVoice::new(def, 48000.0, &[]);
    let mut audio = vec![0.0; ((call.keys.last().unwrap().t + 0.8) * 48000.0).ceil() as usize];
    voice.play_call_id(id, None, 1.0);
    for part in audio.chunks_mut(block) {
        voice.render(part);
    }
    assert!(voice.active_call().is_none(), "{id} must complete");
    audio
}

#[test]
fn dog_calls_are_audible_release_and_ignore_host_block_size() {
    let def = creature_by_id("dog-model").unwrap();
    for c in &def.calls {
        let audio = call(c.id, 128);
        assert!(audio.iter().all(|x| x.is_finite()));
        assert!(rms(&audio) > 0.001, "{} is silent", c.id);
        assert!(
            rms(&audio[audio.len() - 4800..]) < 1e-4,
            "{} rings after release",
            c.id
        );
        for block in [77, 4096] {
            assert_eq!(
                call(c.id, block),
                audio,
                "{} depends on block size {block}",
                c.id
            );
        }
    }
}

#[test]
fn clean_held_howls_follow_note_pitch() {
    let def = creature_by_id("dog-model").unwrap();
    for fs in [44100.0, 48000.0, 96000.0] {
        for target in [220.0, 330.0, 440.0] {
            let mut voice = CreatureVoice::new(
                def,
                fs,
                &[
                    (def.param_index("pitch").unwrap(), target),
                    (def.param_index("rough").unwrap(), 0.0),
                ],
            );
            voice.set_gate(1.0);
            let mut warm = vec![0.0; (fs * 0.3) as usize];
            voice.render(&mut warm);
            let mut audio = vec![0.0; 4096];
            voice.render(&mut audio);
            let frame: Vec<_> = audio.iter().map(|&x| f64::from(x)).collect();
            let estimate = yin_pitch(
                &frame,
                fs,
                YinOpts {
                    min_f0: Some(100.0),
                    max_f0: Some(1200.0),
                    ..Default::default()
                },
            );
            let cents = 1200.0 * (estimate.f0 / target).log2();
            assert!(
                cents.abs() < 25.0,
                "{fs} Hz: target {target}, measured {estimate:?}, {cents} cents"
            );
            assert!(rms(&audio) > 0.001);
        }
    }
}

#[test]
fn dog_parameter_extremes_keep_the_airway_finite() {
    let def = creature_by_id("dog-model").unwrap();
    let mut cases: Vec<Vec<(usize, f64)>> = vec![
        def.params
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.min))
            .collect(),
        def.params
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.max))
            .collect(),
    ];
    for (i, p) in def.params.iter().enumerate() {
        cases.push(vec![(i, p.min)]);
        cases.push(vec![(i, p.max)]);
    }
    for fs in [44100.0, 48000.0, 96000.0] {
        for params in &cases {
            let mut voice = CreatureVoice::new(def, fs, params);
            voice.set_gate(1.0);
            let mut audio = vec![0.0; (fs * 0.15) as usize];
            voice.render(&mut audio);
            assert!(audio.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
            // The generic output stage silences non-finite samples, so inspect the physical
            // state too: a numerically broken airway must not pass by being muted.
            let net = voice.sim.net().unwrap();
            assert!(
                net.p.iter().all(|x| x.is_finite()),
                "non-finite airway at {fs}: {params:?}"
            );
        }
    }
}

#[test]
fn howl_tune_keeps_the_requested_intervals() {
    let audio = call("howlTune", 128);
    for (i, target) in [
        261.625565, 261.625565, 391.995436, 391.995436, 440.0, 440.0, 391.995436,
    ]
    .iter()
    .enumerate()
    {
        let start = (i as f64 * 0.5 * 48000.0) as usize + 9600;
        let frame: Vec<_> = audio[start..start + 4096]
            .iter()
            .map(|&x| f64::from(x))
            .collect();
        let estimate = yin_pitch(
            &frame,
            48000.0,
            YinOpts {
                min_f0: Some(100.0),
                max_f0: Some(1200.0),
                ..Default::default()
            },
        );
        assert!(
            (1200.0 * (estimate.f0 / target).log2()).abs() < 25.0,
            "note {i}: {estimate:?}, expected {target}"
        );
    }
}
