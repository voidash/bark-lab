//! New body-cover family checks. Legacy-source guards remain unchanged.
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureVoice, creature_by_id};
use chimera::sources::canine_cover::CanineCover;
use chimera::sources::two_mass::FoldError;

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&v| f64::from(v).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

#[test]
fn tissue_requires_pressure_releases_and_prestress_changes_frequency() {
    let mut pitches = Vec::new();
    for hz in [170.0, 330.0] {
        let mut tissue = CanineCover::new(96000.0);
        tissue.configure(hz, 1.0, 0.015, 0.7).unwrap();
        let mut opening = Vec::new();
        for i in 0..96000 {
            let area = tissue.step(18000.0, 0.0).unwrap();
            if i >= 48000 && i % 2 == 0 {
                opening.push(area);
            }
        }
        assert!(tissue.speed() > 1.0);
        let p = yin_pitch(
            &opening[..4096],
            48000.0,
            YinOpts {
                min_f0: Some(70.0),
                max_f0: Some(1200.0),
                ..Default::default()
            },
        );
        assert!(p.aperiodicity < 0.1);
        pitches.push(p.f0);
        for _ in 0..96000 {
            tissue.step(0.0, 0.0).unwrap();
        }
        assert!(tissue.speed() < 1e-8, "unpowered tissue must dissipate");
        assert_eq!(tissue.step(f64::NAN, 0.0), Err(FoldError::InvalidInput));
    }
    assert!(
        pitches[1] > pitches[0] * 1.4,
        "prestress did not raise physical frequency: {pitches:?}"
    );
}

#[test]
fn six_mass_live_calls_have_body_motion_flow_correct_aperture_and_release() {
    let def = creature_by_id("dog-physics").unwrap();
    let source = def.param_index("sourceModel").unwrap();
    for call in ["bark", "howl", "yodel", "roughBark"] {
        let mut voice = CreatureVoice::new(def, 48000.0, &[(source, 1.0)]);
        voice.play_call_id(call, None, 1.0);
        let duration = def.calls[def.call_index(call).unwrap()]
            .keys
            .last()
            .unwrap()
            .t
            + 0.7;
        let mut audio = vec![0.0; (duration * 48000.0) as usize];
        let mut t = voice.new_telemetry();
        let mut body = false;
        let mut flow = false;
        for block in audio.chunks_mut(128) {
            voice.render(block);
            voice.telemetry(&mut t);
            let get = |key| t.source.iter().find(|(k, _)| *k == key).unwrap().1;
            assert_eq!(get("fault"), 0.0, "source fault in {call}");
            assert_eq!(get("sourceModel"), 1.0);
            assert_eq!(get("foldLength"), 1.25);
            let a = get("restArea") + 1.25 * (get("leftLower") + get("rightLower"));
            let b = get("restArea") + 1.25 * (get("leftUpper") + get("rightUpper"));
            assert!((get("glottalArea") - a.min(b).max(0.0)).abs() < 1e-12);
            body |= get("leftBody").abs() > 1e-4 || get("rightBody").abs() > 1e-4;
            flow |= get("glottalFlow").abs() > 1.0;
        }
        assert!(audio.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
        assert!(rms(&audio) > 0.001, "{call} silent");
        assert!(body && flow);
        assert!(
            rms(&audio[audio.len() - 4800..]) < 1e-4,
            "{call} did not release"
        );
    }
}

#[test]
fn body_cover_parameter_edges_remain_finite_at_supported_rates() {
    let def = creature_by_id("dog-physics").unwrap();
    let mode = def.param_index("sourceModel").unwrap();
    for fs in [44100.0, 48000.0, 96000.0] {
        for (i, p) in def.params.iter().enumerate().filter(|(i, _)| *i != mode) {
            for v in [p.min, p.max] {
                let mut voice = CreatureVoice::new(def, fs, &[(mode, 1.0), (i, v)]);
                voice.set_gate(1.0);
                let mut audio = vec![0.0; (fs * 0.15) as usize];
                voice.render(&mut audio);
                let mut t = voice.new_telemetry();
                voice.telemetry(&mut t);
                assert!(audio.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
                assert_eq!(
                    t.source.iter().find(|(k, _)| *k == "fault").unwrap().1,
                    0.0,
                    "{fs}/{}/{v}",
                    p.key
                );
            }
        }
    }
}

#[test]
fn body_cover_calls_do_not_depend_on_audio_block_size() {
    let def = creature_by_id("dog-physics").unwrap();
    let mode = def.param_index("sourceModel").unwrap();
    let render = |block_size| {
        let mut voice = CreatureVoice::new(def, 48000.0, &[(mode, 1.0)]);
        voice.play_call_id("bark", None, 1.0);
        let mut audio = vec![0.0; 38400];
        for block in audio.chunks_mut(block_size) {
            voice.render(block);
        }
        audio
    };
    let reference = render(128);
    assert_eq!(reference, render(77));
    assert_eq!(reference, render(4096));
}

#[test]
fn rough_breath_pulse_is_irregular_without_additive_turbulence() {
    let def = creature_by_id("dog-physics").unwrap();
    // The call itself explicitly selects the body-cover source, even when the
    // performer previously selected legacy. Noise is disabled at the network.
    let mut voice = CreatureVoice::new(def, 48000.0, &[]);
    voice.sim.net_mut().unwrap().noise_scale = 0.0;
    voice.play_call_id("roughBark", None, 1.0);
    let mut audio = vec![0.0; 12000];
    voice.render(&mut audio);
    assert!(rms(&audio[480..]) > 0.01);
    let frame: Vec<_> = audio[1920..6016].iter().map(|&v| f64::from(v)).collect();
    let pitch = yin_pitch(
        &frame,
        48000.0,
        YinOpts {
            min_f0: Some(65.0),
            max_f0: Some(1600.0),
            ..Default::default()
        },
    );
    assert!(
        pitch.aperiodicity > 0.25,
        "lost the mechanical irregular regime: {}",
        pitch.aperiodicity
    );
    let mut t = voice.new_telemetry();
    voice.telemetry(&mut t);
    assert_eq!(
        t.source
            .iter()
            .find(|(k, _)| *k == "sourceModel")
            .unwrap()
            .1,
        1.0
    );
    assert_eq!(t.source.iter().find(|(k, _)| *k == "fault").unwrap().1, 0.0);
}
