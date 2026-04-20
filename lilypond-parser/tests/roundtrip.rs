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
        for octave in 1u8..=6u8 {
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
    for oct in 0u8..=8u8 {
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
            }
            other => panic!("{:?}", other),
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
