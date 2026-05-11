// REQ-O39 (R32): proptest strategy for OwnedDuration round-trip.

use proptest::prelude::*;

fn arb_duration_shorthand() -> impl Strategy<Value = String> {
    let bases: Vec<&str> = vec!["1", "2", "4", "8", "16", "32", "64", "128"];
    let dots: Vec<&str> = vec!["", ".", ".."];

    (prop::sample::select(bases), prop::sample::select(dots))
        .prop_map(|(base, dot)| format!("{base}{dot}"))
}

proptest! {
    #[test]
    fn roundtrip_duration_shorthand(s in arb_duration_shorthand()) {
        // Embed a valid duration in a minimal Snippet, serialize via ron, re-parse.
        let src = format!(
            "(kind: \"Snippet\", clef: \"treble\", events: [Rest(duration: \"{s}\")])"
        );
        let doc = music_ron::parse(&src).unwrap();
        let back_src = ron::ser::to_string(&doc).unwrap();
        let back = music_ron::parse(&back_src).unwrap();
        prop_assert_eq!(format!("{doc:?}"), format!("{back:?}"));
    }
}
