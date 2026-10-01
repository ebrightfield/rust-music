// REQ-O39 (R32): proptest strategy for OwnedPitch round-trip.

use proptest::prelude::*;

/// Valid (letter, accidental-shorthand) pairs — excludes combos that yield
/// ExcessiveAccidental in the music crate (Ceses, Eisis, Feses, Bisis).
fn arb_pitch_shorthand() -> impl Strategy<Value = String> {
    let valid_combos: Vec<(&str, &str)> = vec![
        // C: natural, sharp, double-sharp (no double-flat)
        ("c", ""),
        ("c", "s"),
        ("c", "ss"),
        ("c", "es"),
        // D: all five accidentals valid
        ("d", ""),
        ("d", "s"),
        ("d", "ss"),
        ("d", "es"),
        ("d", "eses"),
        // E: natural, sharp, flat, double-flat (no double-sharp)
        ("e", ""),
        ("e", "s"),
        ("e", "es"),
        ("e", "eses"),
        // F: natural, sharp, double-sharp, flat (no double-flat)
        ("f", ""),
        ("f", "s"),
        ("f", "ss"),
        ("f", "es"),
        // G: all five accidentals valid
        ("g", ""),
        ("g", "s"),
        ("g", "ss"),
        ("g", "es"),
        ("g", "eses"),
        // A: all five accidentals valid
        ("a", ""),
        ("a", "s"),
        ("a", "ss"),
        ("a", "es"),
        ("a", "eses"),
        // B: natural, sharp, flat, double-flat (no double-sharp)
        ("b", ""),
        ("b", "s"),
        ("b", "es"),
        ("b", "eses"),
    ];
    let combo_strategy = prop::sample::select(valid_combos);
    let octave_strategy = 0u8..=8u8;

    (combo_strategy, octave_strategy).prop_map(|((letter, acc), oct)| format!("{letter}{acc}{oct}"))
}

proptest! {
    #[test]
    fn roundtrip_pitch_shorthand(s in arb_pitch_shorthand()) {
        // REQ-O39: ron-serialize → parse → structural eq.
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [Note(pitch: \"{s}\", duration: \"4\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        // Serialize and re-parse.
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }
}
