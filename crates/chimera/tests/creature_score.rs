//! Audio-clock local scores: validation, cancellation and actual source identity.
use chimera::creatures::score::CreatureScore;
use chimera::creatures::{CreatureVoice, creature_by_id};
use serde_json::json;

fn score() -> CreatureScore {
    serde_json::from_value(json!({"keys":[
        {"t":0.0,"values":{"gate":1.0,"pitch":330.0}},
        {"t":0.2,"values":{"pitch":500.0},"curve":"step"},
        {"t":0.4,"values":{"gate":0.0}}
    ]}))
    .unwrap()
}

#[test]
fn scores_render_identically_at_different_blocks_and_release_on_both_sources() {
    for id in ["dog", "dog-physics"] {
        let def = creature_by_id(id).unwrap();
        let render = |chunk| {
            let settings = def
                .param_index("sourceModel")
                .map(|i| vec![(i, 1.0)])
                .unwrap_or_default();
            let mut voice = CreatureVoice::new(def, 48000.0, &settings);
            let timing = voice.play_score(&score()).unwrap();
            assert_eq!(timing.0, 0.0);
            let mut audio = vec![0.0; 48000];
            for block in audio.chunks_mut(chunk) {
                voice.render(block);
            }
            assert_eq!(voice.active_call(), None);
            assert!(audio.iter().all(|v| v.is_finite()));
            assert!(audio.iter().any(|v| v.abs() > 0.005));
            assert!(audio[43200..].iter().all(|v| v.abs() < 1e-5));
            let mut t = voice.new_telemetry();
            voice.telemetry(&mut t);
            if id == "dog" {
                assert!(
                    t.tubes
                        .iter()
                        .all(|tube| tube.rms.iter().all(|v| *v == 0.0))
                );
            } else {
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
            audio
        };
        assert_eq!(render(128), render(77));
        assert_eq!(render(128), render(4096));
    }
}

#[test]
fn invalid_score_preserves_current_call_and_stop_returns_controls() {
    let def = creature_by_id("dog-physics").unwrap();
    let mut voice = CreatureVoice::new(def, 48000.0, &[]);
    voice.play_call_id("howl", None, 1.0);
    for value in [
        json!({"keys":[]}),
        json!({"keys":[{"t":0,"values":{"gate":1}},{"t":0.1,"values":{"gate":1}}]}),
        json!({"keys":[{"t":0,"values":{"unknown":1}},{"t":0.1,"values":{"gate":0}}]}),
        json!({"keys":[{"t":0,"values":{"pitch":99999}},{"t":0.1,"values":{"gate":0}}]}),
        json!({"keys":[{"t":0.2,"values":{"gate":1}},{"t":0.1,"values":{"gate":0}}]}),
        json!({"keys":[{"t":0,"values":{"!bark":1}},{"t":0.1,"values":{"gate":0}}]}),
    ] {
        let invalid: CreatureScore = serde_json::from_value(value).unwrap();
        assert!(voice.play_score(&invalid).is_err());
        assert_eq!(voice.active_call(), Some("howl"));
    }
    voice.play_score(&score()).unwrap();
    voice.render(&mut [0.0; 128]);
    assert_eq!(voice.active_call(), Some("score"));
    voice.stop_call();
    voice.render(&mut [0.0; 128]);
    assert_eq!(voice.active_call(), None);
    assert_eq!(voice.targets(), voice.performer());
}
