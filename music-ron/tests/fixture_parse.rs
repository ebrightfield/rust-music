use music_ron::{parse, Document};

fn parse_fixture(name: &str) -> Document {
    let path = format!("fixtures/happy/{name}.ron");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"));
    parse(&content).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

#[test]
fn fixture_snippet() {
    assert!(matches!(parse_fixture("snippet"), Document::Snippet(_)));
}

#[test]
fn fixture_tab() {
    assert!(matches!(parse_fixture("tab"), Document::Tab(_)));
}

#[test]
fn fixture_fretboard_shape() {
    assert!(matches!(parse_fixture("fretboard_shape"), Document::FretboardShape(_)));
}

#[test]
fn fixture_pitch_circle() {
    assert!(matches!(parse_fixture("pitch_circle"), Document::PitchCircle(_)));
}

#[test]
fn fixture_chord_progression() {
    assert!(matches!(parse_fixture("chord_progression"), Document::ChordProgression(_)));
}

#[test]
fn fixture_scale_diagram() {
    assert!(matches!(parse_fixture("scale_diagram"), Document::ScaleDiagram(_)));
}

#[test]
fn fixture_interval_matrix() {
    assert!(matches!(parse_fixture("interval_matrix"), Document::IntervalMatrix(_)));
}
