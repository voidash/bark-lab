//! Mechanical causality and runtime checks. None of these is a dog-likeness judge.
use chimera::airway::frequency::{find_peaks, linspace, transfer_function};
use chimera::analysis::pitch::{YinOpts, yin_pitch};
use chimera::creatures::{CreatureVoice, creature_by_id};
use chimera::sources::two_mass::{FoldError, TwoMassFolds};

fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}

#[test]
fn mechanical_telemetry_matches_the_solved_aperture_and_does_not_change_audio() {
    let def = creature_by_id("dog-physics").unwrap();
    let mut observed = CreatureVoice::new(def, 48000.0, &[]);
    let mut control = CreatureVoice::new(def, 48000.0, &[]);
    observed.set_gate(1.0);
    control.set_gate(1.0);
    let mut telemetry = observed.new_telemetry();
    let mut moving = false;
    for _ in 0..150 {
        let mut a = [0.0; 128];
        let mut b = [0.0; 128];
        observed.render(&mut a);
        control.render(&mut b);
        assert_eq!(a, b, "inspection changed the simulated sound");
        observed.telemetry(&mut telemetry);
        let value = |key| telemetry.source.iter().find(|(k, _)| *k == key).unwrap().1;
        assert!(telemetry.source.iter().all(|(_, v)| v.is_finite()));
        let lower =
            value("restArea") + value("foldLength") * (value("leftLower") + value("rightLower"));
        let upper =
            value("restArea") + value("foldLength") * (value("leftUpper") + value("rightUpper"));
        assert!((value("glottalArea") - lower.min(upper).max(0.0)).abs() < 1e-12);
        assert!(value("glottalArea") >= 0.0);
        assert!((value("tracheaLength") - 18.0).abs() < 1e-10);
        assert!((value("pharynxLength") + value("oralLength") - 17.0).abs() < 1e-10);
        assert!((value("nasalLength") - 13.0).abs() < 1e-10);
        moving |= value("leftLower").abs() > 1e-4;
    }
    assert!(moving, "no tissue motion reached the inspector");
}

#[test]
fn length_control_changes_airway_resonances_without_changing_the_source_preset() {
    let def = creature_by_id("dog-physics").unwrap();
    let frequencies = linspace(100.0, 5000.0, 981);
    let mut peaks = Vec::new();
    for length in [17.0, 22.4] {
        let mut voice = CreatureVoice::new(
            def,
            48000.0,
            &[
                (def.param_index("tractLength").unwrap(), length),
                (def.param_index("mouth").unwrap(), 0.65),
                (def.param_index("nasal").unwrap(), 0.0),
            ],
        );
        // Settle geometry without phonation; test the actual linearized acoustic
        // network rather than a pitch/formant estimator applied to its source.
        voice.render(&mut vec![0.0; 9600]);
        let net = voice.sim.net().unwrap();
        assert!(net.check_cfl() <= 1.0);
        let pharynx = voice
            .sim
            .tubes()
            .iter()
            .find(|(name, _)| *name == "pharynx")
            .unwrap()
            .1;
        let elements = net.linear_elements();
        let cut: Vec<_> = (0..elements.n_branches)
            .filter(|&b| {
                elements.b[b] == pharynx.first_node() || elements.a[b] == pharynx.first_node()
            })
            .filter(|&b| {
                elements.a[b] != pharynx.first_node() + 1
                    && elements.b[b] != pharynx.first_node() + 1
            })
            .collect();
        let transfer = transfer_function(&elements, pharynx.first_node(), &frequencies, &cut);
        let resonances = find_peaks(&frequencies, &transfer.mag, 100.0);
        assert!(
            resonances.len() >= 3,
            "missing physical airway resonances: {resonances:?}"
        );
        peaks.push(
            resonances
                .iter()
                .take(3)
                .map(|p| p.freq)
                .collect::<Vec<_>>(),
        );
    }
    for i in 0..3 {
        assert!(
            peaks[1][i] < peaks[0][i] * 0.9,
            "lengthening should lower resonances, not transpose the source: {peaks:?}"
        );
    }
}

#[test]
fn tissue_needs_air_and_tension_changes_the_emergent_frequency() {
    let fs = 96000.0;
    let mut pitches = Vec::new();
    for q in [1.0, 2.0, 3.0] {
        let mut folds = TwoMassFolds::new(fs);
        folds.configure(q, 1.0, 0.05, 1.0).unwrap();
        let mut area = Vec::new();
        for i in 0..96000 {
            let value = folds.step(16000.0, 0.0).unwrap();
            if i >= 48000 && i % 2 == 0 {
                area.push(value);
            }
        }
        assert!(folds.speed() > 1.0, "pressure must sustain tissue motion");
        let pitch = yin_pitch(
            &area[..4096],
            48000.0,
            YinOpts {
                min_f0: Some(50.0),
                max_f0: Some(1500.0),
                ..Default::default()
            },
        );
        assert!(pitch.aperiodicity < 0.1);
        pitches.push(pitch.f0);
        // Removing air must dissipate motion. There is no running pulse oscillator under it.
        for _ in 0..96000 {
            folds.step(0.0, 0.0).unwrap();
        }
        assert!(folds.speed() < 1e-8, "unpowered tissue does not settle");
        assert!((folds.area - 0.05).abs() < 1e-10);
    }
    assert!(
        pitches[1] > pitches[0] * 1.4 && pitches[2] > pitches[1] * 1.2,
        "tension must raise physical frequency: {pitches:?}"
    );
}

#[test]
fn boundary_pressure_affects_the_tissue_and_invalid_inputs_are_errors() {
    let mut a = TwoMassFolds::new(96000.0);
    let mut b = a.clone();
    let mut difference = 0.0;
    for _ in 0..20000 {
        let x = a.step(16000.0, 0.0).unwrap();
        let y = b.step(16000.0, 4000.0).unwrap();
        difference += (x - y).powi(2);
    }
    // Both systems may be closed at the same instant. Compare their trajectories.
    assert!(
        (difference / 20000.0).sqrt() > 0.001,
        "tract pressure must feed back on the source"
    );
    assert_eq!(a.step(f64::NAN, 0.0), Err(FoldError::InvalidInput));
    assert_eq!(
        a.configure(0.0, 1.0, 0.05, 1.0),
        Err(FoldError::InvalidInput)
    );
}

#[test]
fn physical_calls_have_airway_energy_release_and_identical_blocks() {
    let def = creature_by_id("dog-physics").unwrap();
    for call in &def.calls {
        let mut reference = Vec::new();
        for block in [128, 77, 4096] {
            let mut voice = CreatureVoice::new(def, 48000.0, &[]);
            voice.play_call_id(call.id, None, 1.0);
            let mut audio = vec![0.0; ((call.keys.last().unwrap().t + 0.8) * 48000.0) as usize];
            for part in audio.chunks_mut(block) {
                voice.render(part);
            }
            assert!(audio.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
            assert!(rms(&audio) > 0.001, "{} is inaudible", call.id);
            assert!(
                rms(&audio[audio.len() - 4800..]) < 1e-4,
                "{} fails to release",
                call.id
            );
            assert!(voice.active_call().is_none());
            let mut telemetry = voice.new_telemetry();
            voice.telemetry(&mut telemetry);
            assert_eq!(
                telemetry
                    .source
                    .iter()
                    .find(|(name, _)| *name == "fault")
                    .unwrap()
                    .1,
                0.0,
                "{} mechanical source fault",
                call.id
            );
            assert!(
                telemetry
                    .tubes
                    .iter()
                    .any(|t| t.rms.iter().any(|&p| p > 0.0)),
                "audio has no physical airway energy"
            );
            if reference.is_empty() {
                reference = audio;
            } else {
                assert_eq!(audio, reference, "{} depends on block {block}", call.id);
            }
        }
    }
}

#[test]
fn physical_parameter_extremes_do_not_hide_mechanical_faults() {
    let def = creature_by_id("dog-physics").unwrap();
    let mut cases = vec![
        def.params
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.min))
            .collect::<Vec<_>>(),
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
            let mut audio = vec![0.0; (fs * 0.3) as usize];
            voice.render(&mut audio);
            assert!(audio.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
            assert!(voice.sim.net().unwrap().p.iter().all(|p| p.is_finite()));
            let mut telemetry = voice.new_telemetry();
            voice.telemetry(&mut telemetry);
            assert_eq!(
                telemetry
                    .source
                    .iter()
                    .find(|(name, _)| *name == "fault")
                    .unwrap()
                    .1,
                0.0,
                "mechanical fault at {fs}, {params:?}"
            );
        }
    }
}

#[test]
fn interacting_folds_and_load_can_sustain_irregular_phonation() {
    let def = creature_by_id("dog-physics").unwrap();
    let param = |key| def.param_index(key).unwrap();
    let mut measured = Vec::new();
    for (detuning, throat) in [(0.0, 1.2), (0.0, 0.4), (0.85, 1.2), (0.85, 0.4)] {
        let mut voice = CreatureVoice::new(
            def,
            48000.0,
            &[
                (param("rough"), detuning),
                (param("throat"), throat),
                (param("pressure"), 28.0),
                (param("tone"), 0.0),
                (param("mouth"), 0.65),
                (param("pitch"), 330.0),
                (param("damping"), 0.7),
            ],
        );
        voice.set_gate(1.0);
        let mut audio = vec![0.0; 48000];
        voice.render(&mut audio);
        let mut aperiodicities = Vec::new();
        for at in [18000, 24000, 30000] {
            let frame: Vec<_> = audio[at..at + 4096].iter().map(|&x| f64::from(x)).collect();
            aperiodicities.push(
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
        aperiodicities.sort_by(f64::total_cmp);
        measured.push(aperiodicities[1]);
        assert!(rms(&audio[24000..]) > 0.01);
        let mut telemetry = voice.new_telemetry();
        voice.telemetry(&mut telemetry);
        assert_eq!(
            telemetry
                .source
                .iter()
                .find(|(name, _)| *name == "fault")
                .unwrap()
                .1,
            0.0
        );
    }
    println!("median YIN aperiodicity, regular / narrow only / detuned only / both: {measured:?}");
    assert!(
        measured[0] < 0.1,
        "regular control unexpectedly irregular: {measured:?}"
    );
    assert!(
        measured[3] > 0.2,
        "the physical rough regime disappeared: {measured:?}"
    );
    // This statistical diagnostic guards the mechanism, not a claim of canine realism.
}
