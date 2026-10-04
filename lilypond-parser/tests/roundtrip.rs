//! Round-trip tests: render with `music`'s LilyPond output, parse it back,
//! and verify the data matches.

use lilypond_parser::{parse, Event, Item};
use music::notation::lilypond::ToLilypondString;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::{Note, Pitch};

fn render_pitch(p: &Pitch, d: &Duration) -> String {
    format!("{}{}", p.to_lilypond_string(), d.to_lilypond_string())
}

#[test]
fn pitch_duration_roundtrip_across_octaves() {
    for note in [Note::C, Note::Cis, Note::Des, Note::E, Note::Fis, Note::B] {
        for octave in 1i8..=6i8 {
            let pitch = Pitch::new(note, octave);
            let dur = Duration::new(DurationKind::Qtr, 0);
            let src = render_pitch(&pitch, &dur);
            let items = parse(&src).expect(&format!("parse {:?}", src));
            match &items[0] {
                Item::Event(Event::Note(p, d)) => {
                    assert_eq!(p.note, note, "note mismatch for {}", src);
                    assert_eq!(p.octave, octave, "octave mismatch for {}", src);
                    assert_eq!(*d, dur);
                }
                other => panic!("expected note for {:?}, got {:?}", src, other),
            }
        }
    }
}

#[test]
fn ces_bis_roundtrip() {
    let mut cases = Vec::new();
    for oct in 0i8..=8i8 {
        cases.push((Note::Ces, oct));
        cases.push((Note::Bis, oct));
    }
    for (note, octave) in cases {
        let pitch = Pitch::new(note, octave);
        let dur = Duration::new(DurationKind::Eighth, 0);
        let src = render_pitch(&pitch, &dur);
        let items = parse(&src).unwrap_or_else(|e| panic!("parse {:?}: {:?}", src, e));
        match &items[0] {
            Item::Event(Event::Note(p, _)) => {
                assert_eq!(p.note, note, "note mismatch for {}", src);
                assert_eq!(p.octave, octave, "octave mismatch for {}", src);
                assert_eq!(p.midi_note, pitch.midi_note, "midi mismatch for {}", src);
            }
            other => panic!("{:?}", other),
        }
    }
}

/// LilyPond octave marks are written octaves, so `ces'` (C♭4) sounds B3 and
/// `bis` (B♯3) sounds C4 — through both output and input.
#[test]
fn ces_bis_marks_carry_written_octave_and_midi() {
    let dur = Duration::new(DurationKind::Qtr, 0);
    for (note, octave, midi, src) in [
        (Note::Ces, 4, 59, "ces'4"),
        (Note::Ces, 5, 71, "ces''4"),
        (Note::Bis, 3, 60, "bis4"),
        (Note::Bis, 4, 72, "bis'4"),
    ] {
        let pitch = Pitch::new(note, octave);
        assert_eq!(pitch.midi_note, midi, "{note:?}{octave}");
        assert_eq!(render_pitch(&pitch, &dur), src);
        match &parse(src).unwrap()[0] {
            Item::Event(Event::Note(p, _)) => assert_eq!(*p, pitch, "parsing {src}"),
            other => panic!("expected note for {src:?}, got {other:?}"),
        }
    }
}

#[test]
fn all_duration_kinds_roundtrip() {
    let kinds = [
        DurationKind::Breve,
        DurationKind::Whole,
        DurationKind::Half,
        DurationKind::Qtr,
        DurationKind::Eighth,
        DurationKind::Sixteenth,
        DurationKind::ThirtySecond,
        DurationKind::SixtyFourth,
        DurationKind::OneTwentyEighth,
    ];
    for k in kinds {
        for dots in 0..=2 {
            let p = Pitch::new(Note::C, 4);
            let d = Duration::new(k, dots);
            let src = render_pitch(&p, &d);
            let items = parse(&src).unwrap_or_else(|e| panic!("{:?} -> {:?}", src, e));
            match &items[0] {
                Item::Event(Event::Note(_, d2)) => assert_eq!(*d2, d, "for {}", src),
                _ => panic!(),
            }
        }
    }
}
