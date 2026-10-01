// REQ-O39 (R32): proptest strategy for OwnedTab round-trip.
// Exercises named tuning, string conventions, and variable-length event
// sequences with shorthand durations.

use proptest::prelude::*;

/// Duration shorthand strings matching the 8 DurationKind bases x dot suffixes.
fn arb_duration_str() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("1"),
        Just("2"),
        Just("4"),
        Just("8"),
        Just("16"),
        Just("32"),
        Just("64"),
        Just("128"),
        Just("4."),
        Just("8."),
        Just("2.."),
    ]
}

/// Optional string_convention RON fragment.
fn arb_convention_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(", string_convention: \"ZeroIndexedFromLow\"".to_string()),
        Just(", string_convention: \"OneIndexedFromHigh\"".to_string()),
    ]
}

/// A single tab event RON string.
fn arb_tab_event_ron() -> impl Strategy<Value = String> {
    (1u8..=6u8, 0u8..=24u8, arb_duration_str()).prop_map(|(string, fret, dur)| {
        format!("(string: {string}, fret: {fret}, duration: \"{dur}\")")
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn roundtrip_tab_named_tuning(
        convention in arb_convention_ron(),
        events in prop::collection::vec(arb_tab_event_ron(), 1..=8),
    ) {
        let events_str = events.join(", ");
        let src = format!(
            "(kind: \"Tab\", tuning: \"standard\", events: [{events_str}]{convention})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_tab_drop_d_tuning(
        events in prop::collection::vec(arb_tab_event_ron(), 1..=4),
    ) {
        let events_str = events.join(", ");
        let src = format!(
            "(kind: \"Tab\", tuning: \"drop_d\", events: [{events_str}])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_tab_inline_tuning(
        events in prop::collection::vec(arb_tab_event_ron(), 1..=4),
        convention in arb_convention_ron(),
    ) {
        // 4-string inline tuning (e.g. ukulele-like)
        let src = format!(
            "(kind: \"Tab\", tuning: (pitches: [\"g4\", \"c4\", \"e4\", \"a4\"]), \
             events: [{events}]{convention})",
            events = events.join(", "),
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }
}
