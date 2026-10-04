// REQ-O39 (R32): proptest strategy for OwnedSnippet round-trip.
// Exercises clef variants, optional meta, and mixed event sequences.

use proptest::prelude::*;

fn arb_pitch_str() -> impl Strategy<Value = String> {
    let valid_combos: Vec<(&str, &str)> = vec![
        ("c", ""),
        ("c", "s"),
        ("c", "es"),
        ("d", ""),
        ("d", "s"),
        ("d", "es"),
        ("e", ""),
        ("e", "es"),
        ("f", ""),
        ("f", "s"),
        ("g", ""),
        ("g", "s"),
        ("g", "es"),
        ("a", ""),
        ("a", "s"),
        ("a", "es"),
        ("b", ""),
        ("b", "es"),
    ];
    let combo = prop::sample::select(valid_combos);
    let octave = 1u8..=7u8;
    (combo, octave).prop_map(|((l, a), o)| format!("{l}{a}{o}"))
}

fn arb_duration_str() -> impl Strategy<Value = String> {
    let bases: Vec<&str> = vec!["1", "2", "4", "8", "16", "32", "64", "128"];
    let dots: Vec<&str> = vec!["", ".", ".."];
    (prop::sample::select(bases), prop::sample::select(dots)).prop_map(|(b, d)| format!("{b}{d}"))
}

fn arb_clef() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("treble"),
        Just("treble8va"),
        Just("treble8ba"),
        Just("bass"),
        Just("alto"),
        Just("tenor"),
    ]
}

fn arb_event_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        (arb_pitch_str(), arb_duration_str())
            .prop_map(|(p, d)| format!("Note(pitch: \"{p}\", duration: \"{d}\")")),
        arb_duration_str().prop_map(|d| format!("Rest(duration: \"{d}\")")),
        (
            prop::collection::vec(arb_pitch_str(), 2..=3),
            arb_duration_str()
        )
            .prop_map(|(ps, d)| {
                let joined = ps
                    .iter()
                    .map(|p| format!("\"{p}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Chord(pitches: [{joined}], duration: \"{d}\")")
            }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn roundtrip_snippet_varied_clefs(
        clef in arb_clef(),
        events in prop::collection::vec(arb_event_ron(), 1..=6),
    ) {
        let evts = events.join(", ");
        let src = format!(
            "(kind: \"Snippet\", clef: \"{clef}\", events: [{evts}])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back = ron::ser::to_string(&doc).unwrap();
        let doc2 = music_ron::parse(&back).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{doc2:?}"));
    }

    #[test]
    fn roundtrip_snippet_with_meta(
        clef in arb_clef(),
        p in arb_pitch_str(),
        d in arb_duration_str(),
    ) {
        let src = format!(
            "(kind: \"Snippet\", meta: (title: Some(\"Test\")), clef: \"{clef}\", events: [Note(pitch: \"{p}\", duration: \"{d}\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back = ron::ser::to_string(&doc).unwrap();
        let doc2 = music_ron::parse(&back).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{doc2:?}"));
    }

    #[test]
    fn roundtrip_snippet_with_tie(
        p1 in arb_pitch_str(),
        p2 in arb_pitch_str(),
        d in arb_duration_str(),
    ) {
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [Note(pitch: \"{p1}\", duration: \"{d}\"), Tie, Note(pitch: \"{p2}\", duration: \"{d}\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back = ron::ser::to_string(&doc).unwrap();
        let doc2 = music_ron::parse(&back).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{doc2:?}"));
    }
}
