// REQ-O39 (R32): proptest strategy for OwnedEvent round-trip.
// Exercises Note, Chord, Rest, Tie, and Tuplet variants.

use proptest::prelude::*;

/// Valid (letter, accidental-shorthand) pairs — excludes combos that yield
/// ExcessiveAccidental in the music crate.
fn arb_pitch_str() -> impl Strategy<Value = String> {
    let valid_combos: Vec<(&str, &str)> = vec![
        ("c", ""), ("c", "s"), ("c", "ss"), ("c", "es"),
        ("d", ""), ("d", "s"), ("d", "ss"), ("d", "es"), ("d", "eses"),
        ("e", ""), ("e", "s"), ("e", "es"), ("e", "eses"),
        ("f", ""), ("f", "s"), ("f", "ss"), ("f", "es"),
        ("g", ""), ("g", "s"), ("g", "ss"), ("g", "es"), ("g", "eses"),
        ("a", ""), ("a", "s"), ("a", "ss"), ("a", "es"), ("a", "eses"),
        ("b", ""), ("b", "s"), ("b", "es"), ("b", "eses"),
    ];
    let combo = prop::sample::select(valid_combos);
    let octave = 0u8..=8u8;
    (combo, octave).prop_map(|((l, a), o)| format!("{l}{a}{o}"))
}

fn arb_duration_str() -> impl Strategy<Value = String> {
    let bases: Vec<&str> = vec!["1", "2", "4", "8", "16", "32", "64", "128"];
    let dots: Vec<&str> = vec!["", ".", ".."];
    (prop::sample::select(bases), prop::sample::select(dots))
        .prop_map(|(b, d)| format!("{b}{d}"))
}

/// Generate a single event RON fragment (Note, Chord, Rest, or Tie).
fn arb_event_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        // Note
        (arb_pitch_str(), arb_duration_str())
            .prop_map(|(p, d)| format!("Note(pitch: \"{p}\", duration: \"{d}\")")),
        // Chord (2–3 pitches)
        (
            prop::collection::vec(arb_pitch_str(), 2..=3),
            arb_duration_str()
        )
            .prop_map(|(ps, d)| {
                let joined = ps.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
                format!("Chord(pitches: [{joined}], duration: \"{d}\")")
            }),
        // Rest
        arb_duration_str().prop_map(|d| format!("Rest(duration: \"{d}\")")),
        // Tie (unit variant)
        Just("Tie".to_string()),
    ]
}

proptest! {
    #[test]
    fn roundtrip_note_event(p in arb_pitch_str(), d in arb_duration_str()) {
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [Note(pitch: \"{p}\", duration: \"{d}\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_chord_event(
        ps in prop::collection::vec(arb_pitch_str(), 2..=4),
        d in arb_duration_str()
    ) {
        let joined = ps.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [Chord(pitches: [{joined}], duration: \"{d}\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_rest_event(d in arb_duration_str()) {
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [Rest(duration: \"{d}\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_mixed_events(events in prop::collection::vec(arb_event_ron(), 1..=5)) {
        let joined = events.join(", ");
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [{joined}])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_tuplet_event(
        p1 in arb_pitch_str(),
        p2 in arb_pitch_str(),
        p3 in arb_pitch_str(),
    ) {
        // 3:2 tuplet with three eighth-note children
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [Tuplet(numerator: 3, denominator: 2, base: Eighth, children: [Note(pitch: \"{p1}\", duration: \"8\"), Note(pitch: \"{p2}\", duration: \"8\"), Note(pitch: \"{p3}\", duration: \"8\")])])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }
}
