// REQ-O39 (R32): proptest strategy for OwnedFretboardShape round-trip.
// Exercises fret values (muted/open/fingered), optional barre, optional fingers.

use proptest::prelude::*;

/// Generate a single fret value RON fragment: "x" (muted), 0 (open), or 1..=24 (fingered).
fn arb_fret_ron() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("\"x\"".to_string()),
        Just("0".to_string()),
        (1u8..=24u8).prop_map(|f| f.to_string()),
    ]
}

/// Generate a frets array with exactly `num_strings` entries.
fn arb_frets_ron(num_strings: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(arb_fret_ron(), num_strings..=num_strings)
        .prop_map(|fs| format!("[{}]", fs.join(", ")))
}

/// Generate a valid barre RON fragment for the given string count.
/// barre.fret > 0, from < to, both within 1..=num_strings.
fn arb_barre_ron(num_strings: usize) -> impl Strategy<Value = String> {
    let ns = num_strings;
    (1u8..=24u8, 1..ns, 1..ns).prop_flat_map(move |(fret, a, b)| {
        let (from, to) = if a < b { (a, b) } else { (b, a) };
        // Ensure from < to (if equal, bump to by 1 if possible)
        let (from, to) = if from == to && to < ns {
            (from, to + 1)
        } else if from == to {
            (from - 1, to)
        } else {
            (from, to)
        };
        Just(format!(
            "(fret: {fret}, from_string: {from}, to_string: {to})"
        ))
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn roundtrip_fretboard_basic(frets in arb_frets_ron(6)) {
        let src = format!(
            "(kind: \"FretboardShape\", tuning: \"standard\", frets: {frets})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_fretboard_with_barre(
        frets in arb_frets_ron(6),
        barre in arb_barre_ron(6),
    ) {
        let src = format!(
            "(kind: \"FretboardShape\", tuning: \"standard\", frets: {frets}, barre: {barre})"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_fretboard_with_fingers(frets in arb_frets_ron(6)) {
        // 6 fingers, some None
        let src = format!(
            "(kind: \"FretboardShape\", tuning: \"standard\", frets: {frets}, fingers: [Some(1), None, Some(2), Some(3), Some(1), None])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }

    #[test]
    fn roundtrip_fretboard_varied_strings(
        num_strings in 3usize..=8usize,
    ) {
        // Inline tuning with num_strings pitches, all "c4"
        let pitches = (0..num_strings)
            .map(|_| "\"c4\"".to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let frets = (0..num_strings)
            .map(|i| (i % 5).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let src = format!(
            "(kind: \"FretboardShape\", tuning: (pitches: [{pitches}]), frets: [{frets}])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }
}
