//! C♭ and B♯ through the public `ScoreBuilder`: staff placement and
//! within-measure accidental state both follow the written octave
//! (`Pitch::octave`), so C♭5 (MIDI 71) shares accidental state with C5 — not
//! C4 — and B♯3 (MIDI 60) shares it with B3 — not B4.
//!
//! The C♭/B♯ pitches are built from MIDI numbers so the written octave is
//! derived by `music::Pitch`, the path that previously produced the octave of
//! the sounding C..B block instead.

use super::*;
use crate::font::bravura_font;
use music::note::note::Note;
use smufl::Glyph;

/// `(staff_position, accidental glyph)` for each single-note event of measure 0.
/// Every accidental here is automatic, hence never parenthesized.
fn placed_notes(builder: &ScoreBuilder) -> Vec<(i8, Option<Glyph>)> {
    builder.build_measure_contents().unwrap()[0]
        .events
        .iter()
        .map(|event| match event {
            MeasureEvent::Note(note) => (
                note.staff_position,
                note.accidental.map(|accidental| {
                    assert!(!accidental.parenthesized);
                    accidental.glyph
                }),
            ),
            other => panic!("expected a single note, got {other:?}"),
        })
        .collect()
}

/// `translate(x, y)` of every SVG path drawing `glyph`, in document order.
fn glyph_translations(svg: &str, glyph: Glyph) -> Vec<(f64, f64)> {
    let font = bravura_font();
    let outline = font.glyph_outline(glyph).expect("Bravura glyph");
    let needle = format!("<path d=\"{}\"", outline.path_data);
    svg.match_indices(&needle)
        .map(|(start, _)| {
            let element = &svg[start..start + svg[start..].find("/>").expect("closed path")];
            let args_start = element.find("translate(").expect("translated glyph") + 10;
            let args = &element[args_start..args_start + element[args_start..].find(')').unwrap()];
            let mut coords = args.split(',').map(|n| n.trim().parse::<f64>().unwrap());
            (coords.next().unwrap(), coords.next().unwrap())
        })
        .collect()
}

fn c_flat_5() -> Pitch {
    let pitch = Pitch::from_midi_spelled_as(71, &vec![Note::Ces]).unwrap();
    assert_eq!(
        (pitch.note, pitch.octave, pitch.midi_note),
        (Note::Ces, 5, 71)
    );
    pitch
}

fn b_sharp_3() -> Pitch {
    let pitch = Pitch::from_midi_spelled_as(60, &vec![Note::Bis]).unwrap();
    assert_eq!(
        (pitch.note, pitch.octave, pitch.midi_note),
        (Note::Bis, 3, 60)
    );
    pitch
}

#[test]
fn c_flat_5_shares_accidental_state_with_c5_not_c4() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(c_flat_5(), Duration::QTR)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .end_barline();
    assert_eq!(
        placed_notes(&builder),
        vec![
            (5, Some(Glyph::AccidentalFlat)),
            // The flat on C5 is cancelled for the following C5…
            (5, Some(Glyph::AccidentalNatural)),
            // …while C4 is a different written note and needs no sign.
            (-2, None),
        ]
    );
}

#[test]
fn b_sharp_3_shares_accidental_state_with_b3_not_b4() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(b_sharp_3(), Duration::QTR)
        .note(Pitch::new(Note::B, 3), Duration::QTR)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .end_barline();
    assert_eq!(
        placed_notes(&builder),
        vec![
            (-3, Some(Glyph::AccidentalSharp)),
            (-3, Some(Glyph::AccidentalNatural)),
            (4, None),
        ]
    );
}

#[test]
fn c_flat_and_b_sharp_place_on_written_letters_in_bass() {
    let c_flat_4 = Pitch::from_midi_spelled_as(59, &vec![Note::Ces]).unwrap();
    assert_eq!((c_flat_4.octave, c_flat_4.midi_note), (4, 59));
    let b_sharp_2 = Pitch::from_midi_spelled_as(48, &vec![Note::Bis]).unwrap();
    assert_eq!((b_sharp_2.octave, b_sharp_2.midi_note), (2, 48));
    let builder = ScoreBuilder::new()
        .clef(Clef::Bass)
        .note(c_flat_4, Duration::QTR)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .note(b_sharp_2, Duration::QTR)
        .note(Pitch::new(Note::B, 2), Duration::QTR)
        .end_barline();
    assert_eq!(
        placed_notes(&builder),
        vec![
            (10, Some(Glyph::AccidentalFlat)),
            (10, Some(Glyph::AccidentalNatural)),
            (2, Some(Glyph::AccidentalSharp)),
            (2, Some(Glyph::AccidentalNatural)),
        ]
    );
}

/// Rendered output: the C♭5 notehead and its flat sit at the C5 height of a
/// plain C5, the cancelling natural is drawn for the following C5, and C4
/// keeps its own height with no accidental.
#[test]
fn rendered_c_flat_5_draws_flat_and_cancelling_natural_at_c5_height() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(c_flat_5(), Duration::QTR)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let reference = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let heads = glyph_translations(&svg, Glyph::NoteheadBlack);
    let reference_heads = glyph_translations(&reference, Glyph::NoteheadBlack);
    assert_eq!(heads.len(), 3);
    assert_eq!(reference_heads.len(), 3);
    for (head, reference_head) in heads.iter().zip(&reference_heads) {
        assert_eq!(
            head.1, reference_head.1,
            "notehead heights must match C5 C5 C4"
        );
    }
    assert!(heads[2].1 > heads[0].1, "C4 must sit below C♭5");

    let flats = glyph_translations(&svg, Glyph::AccidentalFlat);
    let naturals = glyph_translations(&svg, Glyph::AccidentalNatural);
    assert_eq!(flats.len(), 1, "one flat, on C♭5");
    assert_eq!(naturals.len(), 1, "one natural, on the second C5");
    assert!(glyph_translations(&reference, Glyph::AccidentalNatural).is_empty());

    assert_eq!(flats[0].1, heads[0].1);
    assert!(flats[0].0 < heads[0].0);
    assert_eq!(naturals[0].1, heads[1].1);
    assert!(heads[0].0 < naturals[0].0 && naturals[0].0 < heads[1].0);
}
