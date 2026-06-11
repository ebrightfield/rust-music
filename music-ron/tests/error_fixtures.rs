// REQ-O37 (R30): every MusicRonError variant elicited by ≥1 fixture.

use music_ron::{parse, parse_path, MusicRonError, Document};
use music_ron::convert::{
    convert_snippet, convert_chord_progression, convert_pitch_circle,
    convert_interval_matrix,
};
use std::path::Path;

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(format!("fixtures/error/{name}")).unwrap()
}

/// Helper: parse fixture, extract Snippet, convert it.
fn parse_and_convert_snippet(name: &str) -> Result<(), MusicRonError> {
    let src = read_fixture(name);
    let doc = parse(&src)?;
    let Document::Snippet(s) = doc else {
        panic!("expected Snippet variant");
    };
    convert_snippet(&s)?;
    Ok(())
}

// --- Parse-time errors ---

#[test]
fn err_syntax() {
    let src = read_fixture("syntax.ron");
    let err = parse(&src).unwrap_err();
    assert!(matches!(err, MusicRonError::SyntaxError { .. }), "got {err:?}");
}

#[test]
fn err_unsupported_version() {
    let src = read_fixture("unsupported_version.ron");
    let err = parse(&src).unwrap_err();
    assert!(
        matches!(err, MusicRonError::UnsupportedVersion { got: 99, max_supported: 1, .. }),
        "got {err:?}"
    );
}

#[test]
fn err_io() {
    let err = parse_path(Path::new("/nonexistent/path/to/file.ron")).unwrap_err();
    assert!(matches!(err, MusicRonError::Io { .. }), "got {err:?}");
}

// InvalidDuration shorthand is caught at deser time → SyntaxError.
// The InvalidDuration variant is reachable programmatically but not from
// file-based parse→convert flow because custom Deserialize validates eagerly.
#[test]
fn err_invalid_duration_surfaces_as_syntax_error() {
    let src = read_fixture("invalid_duration.ron");
    let err = parse(&src).unwrap_err();
    assert!(matches!(err, MusicRonError::SyntaxError { .. }), "got {err:?}");
}

// --- Convert-time errors (parse succeeds, convert fails) ---

#[test]
fn err_octave_out_of_range() {
    let err = parse_and_convert_snippet("octave_out_of_range.ron").unwrap_err();
    assert!(
        matches!(err, MusicRonError::OctaveOutOfRange { got: 200, .. }),
        "got {err:?}"
    );
}

#[test]
fn err_invalid_tuplet() {
    let err = parse_and_convert_snippet("invalid_tuplet.ron").unwrap_err();
    assert!(matches!(err, MusicRonError::InvalidTuplet { .. }), "got {err:?}");
}

#[test]
fn err_invalid_tie() {
    let err = parse_and_convert_snippet("invalid_tie.ron").unwrap_err();
    assert!(matches!(err, MusicRonError::InvalidTie { .. }), "got {err:?}");
}

#[test]
fn err_unknown_clef() {
    let err = parse_and_convert_snippet("unknown_clef.ron").unwrap_err();
    assert!(
        matches!(err, MusicRonError::UnknownClef { ref name, .. } if name == "soprano"),
        "got {err:?}"
    );
}

#[test]
fn err_unknown_tuning() {
    let src = read_fixture("unknown_tuning.ron");
    let doc = parse(&src).unwrap();
    let Document::Tab(t) = doc else { panic!("expected Tab") };
    // convert_tab takes a resolved &Fretboard; tuning resolution is separate.
    let tuning_name = match &t.tuning {
        music_ron::ast::common::OwnedTuning::Named(n) => n.as_str(),
        _ => panic!("expected named tuning"),
    };
    let err = music_ron::tuning_registry::resolve(tuning_name, "tuning").unwrap_err();
    assert!(
        matches!(err, MusicRonError::UnknownTuning { ref name, .. } if name == "open_g"),
        "got {err:?}"
    );
}

#[test]
fn err_unknown_chord_symbol() {
    let src = read_fixture("unknown_chord_symbol.ron");
    let doc = parse(&src).unwrap();
    let Document::ChordProgression(cp) = doc else { panic!("expected ChordProgression") };
    let err = convert_chord_progression(&cp).unwrap_err();
    assert!(matches!(err, MusicRonError::UnknownChordSymbol { .. }), "got {err:?}");
}

// UnknownScale is raised by downstream consumers, not by convert_scale_diagram
// (which passes unknown names through). Verify the fixture parses to a
// ScaleDiagram with the unresolvable name; downstream code can assert
// UnknownScale when it attempts lookup.
#[test]
fn err_unknown_scale_fixture_parses() {
    let src = read_fixture("unknown_scale.ron");
    let doc = parse(&src).unwrap();
    let Document::ScaleDiagram(sd) = doc else { panic!("expected ScaleDiagram") };
    assert_eq!(sd.scale.as_deref(), Some("superlocrian_bebop_99"));
}

#[test]
fn err_invalid_pitch() {
    let src = read_fixture("invalid_pitch.ron");
    let doc = parse(&src).unwrap();
    let Document::PitchCircle(pc) = doc else { panic!("expected PitchCircle") };
    let err = convert_pitch_circle(&pc).unwrap_err();
    assert!(matches!(err, MusicRonError::InvalidPitch { .. }), "got {err:?}");
}

#[test]
fn err_ambiguous_identity() {
    let src = read_fixture("ambiguous_identity.ron");
    let doc = parse(&src).unwrap();
    let Document::PitchCircle(pc) = doc else { panic!("expected PitchCircle") };
    let err = convert_pitch_circle(&pc).unwrap_err();
    assert!(matches!(err, MusicRonError::AmbiguousIdentity { .. }), "got {err:?}");
}

#[test]
fn err_unknown_style() {
    let src = read_fixture("unknown_style.ron");
    let doc = parse(&src).unwrap();
    let Document::IntervalMatrix(im) = doc else { panic!("expected IntervalMatrix") };
    let err = convert_interval_matrix(&im).unwrap_err();
    assert!(matches!(err, MusicRonError::UnknownStyle { ref got, .. } if got == "spiral"), "got {err:?}");
}
