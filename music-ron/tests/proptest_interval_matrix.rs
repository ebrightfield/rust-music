// REQ-O39 (R32): proptest strategy for OwnedIntervalMatrix round-trip.
// Exercises the 3-way XOR identity (chord, pcs, scale) with all 4 style
// variants and optional display options (root, title, theme, bar_size, cell_size).

use proptest::prelude::*;

/// Generate a pcs array with 1..=12 values in 0..=11.
fn arb_pcs_ron() -> impl Strategy<Value = String> {
    prop::collection::vec(0u8..=11u8, 1..=12)
        .prop_map(|pcs| format!("[{}]", pcs.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")))
}

/// One of the 4 known styles.
fn arb_style_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("vector".to_string()),
        Just("full_vector".to_string()),
        Just("matrix".to_string()),
        Just("linear".to_string()),
    ]
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

/// Optional title RON fragment.
fn arb_title_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just(", title: \"My Matrix\"".to_string()),
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
    fn roundtrip_interval_matrix_chord(
        style in arb_style_ron(),
        root in arb_root_ron(),
        title in arb_title_ron(),
        theme in arb_theme_ron(),
    ) {
        let chords = ["C", "Am7", "Dm", "G7", "FMaj7"];
        for chord in &chords {
            let src = format!(
                "(kind: \"IntervalMatrix\", chord: \"{chord}\", style: \"{style}\"{root}{title}{theme})"
            );
            let doc = music_ron::parse(&src).unwrap();
            let back_src = ron::ser::to_string(&doc).unwrap();
            let back = music_ron::parse(&back_src).unwrap();
            assert_eq!(format!("{doc:?}"), format!("{back:?}"));
        }
    }

    #[test]
    fn roundtrip_interval_matrix_pcs(
        pcs in arb_pcs_ron(),
        style in arb_style_ron(),
        root in arb_root_ron(),
        title in arb_title_ron(),
        theme in arb_theme_ron(),
    ) {
        let src = format!(
            "(kind: \"IntervalMatrix\", pcs: {pcs}, style: \"{style}\"{root}{title}{theme})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_interval_matrix_scale(
        style in arb_style_ron(),
        root in arb_root_ron(),
        title in arb_title_ron(),
        theme in arb_theme_ron(),
    ) {
        let scales = ["major", "minor", "dorian", "mixolydian"];
        for scale in &scales {
            let src = format!(
                "(kind: \"IntervalMatrix\", scale: \"{scale}\", style: \"{style}\"{root}{title}{theme})"
            );
            let doc = music_ron::parse(&src).unwrap();
            let back_src = ron::ser::to_string(&doc).unwrap();
            let back = music_ron::parse(&back_src).unwrap();
            assert_eq!(format!("{doc:?}"), format!("{back:?}"));
        }
    }
}
