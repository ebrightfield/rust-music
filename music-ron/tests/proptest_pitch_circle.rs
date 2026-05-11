// REQ-O39 (R32): proptest strategy for OwnedPitchCircle round-trip.
// Exercises the 3-way XOR identity (chord, pcs, scale) with optional root/theme.

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
    ]
}

/// Generate a pcs array with 1..=12 values in 0..=11.
fn arb_pcs_ron() -> impl Strategy<Value = String> {
    prop::collection::vec(0u8..=11u8, 1..=12)
        .prop_map(|pcs| format!("[{}]", pcs.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")))
}

/// Optional root note RON fragment.
fn arb_root_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(", root: \"C\"".to_string()),
        Just(", root: \"A\"".to_string()),
        Just(", root: \"Fis\"".to_string()),
    ]
}

/// Optional theme RON fragment.
fn arb_theme_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(", theme: \"dark\"".to_string()),
        Just(", theme: \"default\"".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn roundtrip_pitch_circle_chord(
        chord in arb_chord_symbol(),
        root in arb_root_ron(),
        theme in arb_theme_ron(),
    ) {
        let src = format!(
            "(kind: \"PitchCircle\", chord: \"{chord}\"{root}{theme})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_pitch_circle_pcs(
        pcs in arb_pcs_ron(),
        root in arb_root_ron(),
        theme in arb_theme_ron(),
    ) {
        let src = format!(
            "(kind: \"PitchCircle\", pcs: {pcs}{root}{theme})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_pitch_circle_scale(
        root in arb_root_ron(),
        theme in arb_theme_ron(),
    ) {
        // Scale names are pass-through strings (resolved downstream).
        let scales = ["major", "minor", "dorian", "mixolydian"];
        for scale in &scales {
            let src = format!(
                "(kind: \"PitchCircle\", scale: \"{scale}\"{root}{theme})"
            );
            let doc = music_ron::parse(&src).unwrap();
            let back_src = ron::ser::to_string(&doc).unwrap();
            let back = music_ron::parse(&back_src).unwrap();
            assert_eq!(format!("{doc:?}"), format!("{back:?}"));
        }
    }
}
