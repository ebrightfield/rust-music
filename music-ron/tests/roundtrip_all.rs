// REQ-O38 (R31): parse → convert → structural assert, one test per Document variant.
// Uses structural assertions (not render substring) because renderer entry-points
// vary and may not exist for all variants.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::DurationKind;
use music::note::pitch_class::Pc;
use music_ron::ast::common::OwnedTuning;
use music_ron::{ast::Document, convert, parse};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("fixtures/happy/{name}.ron")).unwrap()
}

#[test]
fn roundtrip_snippet() {
    let doc = parse(&fixture("snippet")).unwrap();
    let Document::Snippet(ref s) = doc else {
        panic!("expected Snippet")
    };
    let resolved = convert::convert_snippet(s).unwrap();
    assert_eq!(resolved.clef, Clef::Treble);
    // 3 notes + 1 rest
    assert_eq!(resolved.events.len(), 4);
    // Quarter-note duration = 32 ticks
    assert_eq!(resolved.events[0].duration(), 32);
    assert!(!resolved.events[0].tied);
}

#[test]
fn roundtrip_tab() {
    let doc = parse(&fixture("tab")).unwrap();
    let Document::Tab(ref t) = doc else {
        panic!("expected Tab")
    };
    let fretboard = match &t.tuning {
        OwnedTuning::Named(name) => music_ron::tuning_registry::resolve(name, "tuning").unwrap(),
        OwnedTuning::Inline { .. } => panic!("fixture uses named tuning"),
    };
    let events = convert::convert_tab(t, fretboard).unwrap();
    // 2 tab events
    assert_eq!(events.len(), 2);
}

#[test]
fn roundtrip_fretboard_shape() {
    let doc = parse(&fixture("fretboard_shape")).unwrap();
    let Document::FretboardShape(ref fs) = doc else {
        panic!("expected FretboardShape")
    };
    let resolved = convert::convert_fretboard_shape(fs).unwrap();
    // 6 strings for standard guitar
    assert_eq!(resolved.frets.len(), 6);
    // First string is open (fret 0)
    assert_eq!(resolved.frets[0], convert::FretValue::Open);
    // Third string is fret 2
    assert_eq!(resolved.frets[2], convert::FretValue::Fingered(2));
}

#[test]
fn roundtrip_pitch_circle() {
    let doc = parse(&fixture("pitch_circle")).unwrap();
    let Document::PitchCircle(ref pc) = doc else {
        panic!("expected PitchCircle")
    };
    let resolved = convert::convert_pitch_circle(pc).unwrap();
    assert!(matches!(
        resolved.identity,
        convert::PitchCircleIdentity::Pcs(ref v)
            if *v == vec![Pc::from(0u8), Pc::from(4u8), Pc::from(7u8)]
    ));
}

#[test]
fn roundtrip_chord_progression() {
    let doc = parse(&fixture("chord_progression")).unwrap();
    let Document::ChordProgression(ref cp) = doc else {
        panic!("expected ChordProgression")
    };
    let resolved = convert::convert_chord_progression(cp).unwrap();
    assert_eq!(resolved.chords.len(), 4);
    // First chord has no explicit duration
    assert!(resolved.chords[0].duration.is_none());
    // Second chord has half-note duration (kind, dots)
    assert_eq!(resolved.chords[1].duration, Some((DurationKind::Half, 0)));
}

#[test]
fn roundtrip_scale_diagram() {
    let doc = parse(&fixture("scale_diagram")).unwrap();
    let Document::ScaleDiagram(ref sd) = doc else {
        panic!("expected ScaleDiagram")
    };
    let resolved = convert::convert_scale_diagram(sd).unwrap();
    assert!(matches!(
        resolved.identity,
        convert::ScaleDiagramIdentity::Scale(ref s) if s == "major"
    ));
    assert_eq!(resolved.orientation, convert::Orientation::Vertical);
    assert_eq!(resolved.start_fret, 0);
    assert_eq!(resolved.num_frets, 5);
}

#[test]
fn roundtrip_interval_matrix() {
    let doc = parse(&fixture("interval_matrix")).unwrap();
    let Document::IntervalMatrix(ref im) = doc else {
        panic!("expected IntervalMatrix")
    };
    let resolved = convert::convert_interval_matrix(im).unwrap();
    assert_eq!(resolved.style, convert::IntervalStyle::Matrix);
    assert!(matches!(
        resolved.identity,
        convert::IntervalMatrixIdentity::Pcs(ref v)
            if *v == vec![Pc::from(0u8), Pc::from(3u8), Pc::from(7u8)]
    ));
}
