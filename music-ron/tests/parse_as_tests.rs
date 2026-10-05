use music_ron::ast::{OwnedPitchCircle, OwnedScore, OwnedSnippet, OwnedTab};
use music_ron::{parse_as, MusicRonError};

#[test]
fn parse_as_snippet_ok() {
    let src = r#"(kind: "Snippet", clef: "treble", events: [])"#;
    let s: OwnedSnippet = parse_as(src).unwrap();
    assert_eq!(s.clef, "treble");
}

#[test]
fn parse_as_tab_ok() {
    let src = r#"(kind: "Tab", tuning: "standard", events: [])"#;
    let t: OwnedTab = parse_as(src).unwrap();
    assert!(matches!(t.tuning, music_ron::ast::OwnedTuning::Named(ref s) if s == "standard"));
}

#[test]
fn parse_as_pitch_circle_ok() {
    let src = r#"(kind: "PitchCircle", pcs: [0, 4, 7])"#;
    let pc: OwnedPitchCircle = parse_as(src).unwrap();
    assert_eq!(pc.pcs, Some(vec![0, 4, 7]));
}

#[test]
fn parse_as_score_ok() {
    let src = r#"(kind: "Score", parts: [], measures: [])"#;
    let score: OwnedScore = parse_as(src).unwrap();
    assert!(score.parts.is_empty());
    assert!(score.measures.is_empty());
}

#[test]
fn parse_as_kind_mismatch_is_syntax_error() {
    let src = r#"(kind: "Snippet", clef: "treble", events: [])"#;
    let err = parse_as::<OwnedTab>(src).unwrap_err();
    match err {
        MusicRonError::SyntaxError { message, .. } => {
            assert!(message.contains("kind mismatch"), "got: {message}");
            assert!(
                message.contains("Tab"),
                "expected Tab in message, got: {message}"
            );
            assert!(
                message.contains("Snippet"),
                "expected Snippet in message, got: {message}"
            );
        }
        other => panic!("expected SyntaxError, got: {other:?}"),
    }
}

#[test]
fn parse_as_propagates_version_error() {
    let src = r#"(kind: "Snippet", version: 99, clef: "treble", events: [])"#;
    let err = parse_as::<OwnedSnippet>(src).unwrap_err();
    assert!(matches!(
        err,
        MusicRonError::UnsupportedVersion { got: 99, .. }
    ));
}
