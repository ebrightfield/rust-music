// REQ-O39 (R32): proptest strategy for OwnedChordProgression round-trip.
// Exercises chord entries with optional durations, meter, and key.

use proptest::prelude::*;

/// Chord symbols known to parse successfully via parse_chord_name.
fn arb_chord_symbol() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("C".to_string()),
        Just("Am".to_string()),
        Just("G7".to_string()),
        Just("Dm7".to_string()),
        Just("Fmaj7".to_string()),
        Just("Bb".to_string()),
        Just("Em".to_string()),
        Just("D".to_string()),
    ]
}

/// Valid duration shorthand strings.
fn arb_duration_str() -> impl Strategy<Value = String> {
    let bases = ["1", "2", "4", "8", "16", "32", "64", "128"];
    let dots = ["", ".", ".."];
    let combos: Vec<String> = bases
        .iter()
        .flat_map(|b| dots.iter().map(move |d| format!("{b}{d}")))
        .collect();
    proptest::sample::select(combos)
}

/// A single chord entry RON fragment.
fn arb_chord_entry() -> impl Strategy<Value = String> {
    (arb_chord_symbol(), proptest::bool::ANY).prop_flat_map(|(sym, has_dur)| {
        if has_dur {
            arb_duration_str()
                .prop_map(move |dur| format!("(symbol: \"{sym}\", duration: \"{dur}\")"))
                .boxed()
        } else {
            Just(format!("(symbol: \"{sym}\")")).boxed()
        }
    })
}

/// Optional meter RON fragment.
fn arb_meter_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(", meter: (beats: 4, unit: Qtr)".to_string()),
        Just(", meter: (beats: 3, unit: Qtr)".to_string()),
        Just(", meter: (beats: 6, unit: Eighth)".to_string()),
    ]
}

/// Optional key RON fragment.
fn arb_key_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(", key: (tonic: C, mode: \"major\")".to_string()),
        Just(", key: (tonic: A, mode: \"minor\")".to_string()),
        Just(", key: (tonic: G, mode: \"mixolydian\")".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn roundtrip_chord_progression(
        entries in prop::collection::vec(arb_chord_entry(), 1..=8),
        meter in arb_meter_ron(),
        key in arb_key_ron(),
    ) {
        let chords_str = entries.join(", ");
        let src = format!(
            "(kind: \"ChordProgression\", chords: [{chords_str}]{meter}{key})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_chord_progression_single_entry(
        sym in arb_chord_symbol(),
    ) {
        let src = format!(
            "(kind: \"ChordProgression\", chords: [(symbol: \"{sym}\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_chord_progression_all_options(
        sym in arb_chord_symbol(),
        dur in arb_duration_str(),
    ) {
        let src = format!(
            "(kind: \"ChordProgression\", chords: [(symbol: \"{sym}\", duration: \"{dur}\")], meter: (beats: 4, unit: Qtr), key: (tonic: C, mode: \"major\"))"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }
}
