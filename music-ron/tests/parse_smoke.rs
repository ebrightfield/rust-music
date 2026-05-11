use music_ron::{parse, Document, MusicRonError};

#[test]
fn smoke_snippet() {
    let src = r#"(kind: "Snippet", clef: "treble", events: [])"#;
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(matches!(doc, Document::Snippet(_)));
}

#[test]
fn smoke_tab() {
    let src = r#"(kind: "Tab", tuning: "standard", events: [])"#;
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(matches!(doc, Document::Tab(_)));
}

#[test]
fn smoke_fretboard_shape() {
    let src = r#"(kind: "FretboardShape", tuning: "standard", frets: [0, 0, 2, 2, 1, 0])"#;
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(matches!(doc, Document::FretboardShape(_)));
}

#[test]
fn smoke_pitch_circle() {
    let src = r#"(kind: "PitchCircle", pcs: [0, 4, 7])"#;
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(matches!(doc, Document::PitchCircle(_)));
}

#[test]
fn smoke_chord_progression() {
    let src = r#"(kind: "ChordProgression", chords: [])"#;
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(matches!(doc, Document::ChordProgression(_)));
}

#[test]
fn smoke_scale_diagram() {
    let src = r#"(kind: "ScaleDiagram", scale: "major", tuning: "standard", start_fret: 0, num_frets: 5, orientation: "vertical")"#;
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(matches!(doc, Document::ScaleDiagram(_)));
}

#[test]
fn smoke_interval_matrix() {
    let src = r#"(kind: "IntervalMatrix", pcs: [0, 3, 7], style: "matrix")"#;
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed: {e}"));
    assert!(matches!(doc, Document::IntervalMatrix(_)));
}

#[test]
fn version_1_accepted() {
    let src = r#"(kind: "Snippet", version: 1, clef: "treble", events: [])"#;
    let doc = parse(src).unwrap();
    assert!(matches!(doc, Document::Snippet(_)));
}

#[test]
fn version_none_accepted() {
    let src = r#"(kind: "Snippet", clef: "treble", events: [])"#;
    let doc = parse(src).unwrap();
    assert!(matches!(doc, Document::Snippet(_)));
}

#[test]
fn version_99_rejected() {
    let src = r#"(kind: "Snippet", version: 99, clef: "treble", events: [])"#;
    let err = parse(src).unwrap_err();
    assert!(matches!(err, MusicRonError::UnsupportedVersion { got: 99, max_supported: 1, .. }));
}

#[test]
fn oversized_input_rejected() {
    let huge = "x".repeat(1_048_577);
    let err = parse(&huge).unwrap_err();
    assert!(matches!(err, MusicRonError::SyntaxError { .. }));
}

#[test]
fn bom_prefixed_input_accepted_by_parse() {
    // REQ-G8 / QA finding W20: `parse` must transparently strip a leading
    // UTF-8 BOM for callers that load bytes themselves (e.g. from the network
    // or via `std::fs::read_to_string`) and hand the raw string to `parse`.
    let src = "\u{FEFF}(kind: \"Snippet\", clef: \"treble\", events: [])";
    let doc = parse(src).unwrap_or_else(|e| panic!("parse failed on BOM-prefixed input: {e}"));
    assert!(matches!(doc, Document::Snippet(_)));
}
