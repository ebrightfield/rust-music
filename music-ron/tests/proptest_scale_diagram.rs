// REQ-O39 (R32): proptest strategy for OwnedScaleDiagram round-trip.
// Exercises the 2-way XOR identity (pcs, scale) with optional display options.

use proptest::prelude::*;

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

/// Orientation RON fragment.
fn arb_orientation_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("horizontal".to_string()),
        Just("vertical".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn roundtrip_scale_diagram_pcs(
        pcs in arb_pcs_ron(),
        root in arb_root_ron(),
        theme in arb_theme_ron(),
        orientation in arb_orientation_ron(),
        start_fret in 0u8..=12u8,
        num_frets in 1u8..=24u8,
    ) {
        let src = format!(
            "(kind: \"ScaleDiagram\", pcs: {pcs}, tuning: \"standard\", \
             start_fret: {start_fret}, num_frets: {num_frets}, \
             orientation: \"{orientation}\"{root}{theme})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_scale_diagram_scale(
        root in arb_root_ron(),
        theme in arb_theme_ron(),
        orientation in arb_orientation_ron(),
        start_fret in 0u8..=12u8,
        num_frets in 1u8..=24u8,
    ) {
        let scales = ["major", "minor", "dorian", "mixolydian", "pentatonic"];
        for scale in &scales {
            let src = format!(
                "(kind: \"ScaleDiagram\", scale: \"{scale}\", tuning: \"standard\", \
                 start_fret: {start_fret}, num_frets: {num_frets}, \
                 orientation: \"{orientation}\"{root}{theme})"
            );
            let doc = music_ron::parse(&src).unwrap();
            let back_src = ron::ser::to_string(&doc).unwrap();
            let back = music_ron::parse(&back_src).unwrap();
            assert_eq!(format!("{doc:?}"), format!("{back:?}"));
        }
    }

    #[test]
    fn roundtrip_scale_diagram_display_flags(
        show_degrees in proptest::bool::ANY,
        highlight_root in proptest::bool::ANY,
        orientation in arb_orientation_ron(),
    ) {
        let src = format!(
            "(kind: \"ScaleDiagram\", scale: \"major\", tuning: \"standard\", \
             start_fret: 0, num_frets: 5, orientation: \"{orientation}\", \
             show_degrees: {show_degrees}, highlight_root: {highlight_root})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }
}
