//! RM-MN-008: breves engrave as double-whole notes and rests through the
//! public score builders, never as whole notes and rests.

use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::font::bravura_font;
use music_engraver::score::guitar::GuitarScore;
use music_engraver::score::ScoreBuilder;
use smufl::Glyph;

fn breve() -> Duration {
    Duration::new(DurationKind::Breve, 0)
}

/// Number of times `glyph`'s exact outline is drawn in `svg`.
fn glyph_count(svg: &str, glyph: Glyph) -> usize {
    let path = bravura_font().glyph_outline(glyph).unwrap().path_data;
    svg.matches(&format!("<path d=\"{path}\" ")).count()
}

/// 4/2 score: a note, a rest, then a G-B-D triad, one value per measure.
fn note_rest_chord_score(value: Duration) -> String {
    ScoreBuilder::new()
        .time_signature(4, 2)
        .note(Pitch::new(Note::G, 4), value)
        .barline()
        .rest(value)
        .barline()
        .chord(
            vec![
                Pitch::new(Note::G, 4),
                Pitch::new(Note::B, 4),
                Pitch::new(Note::D, 5),
            ],
            value,
        )
        .end_barline()
        .render_svg()
}

#[test]
fn breve_note_rest_and_chord_render_double_whole_glyphs() {
    let svg = note_rest_chord_score(breve());
    assert_eq!(
        glyph_count(&svg, Glyph::NoteheadDoubleWhole),
        4,
        "one breve note plus three chord tones"
    );
    assert_eq!(glyph_count(&svg, Glyph::RestDoubleWhole), 1);
    assert_eq!(glyph_count(&svg, Glyph::NoteheadWhole), 0);
    assert_eq!(glyph_count(&svg, Glyph::RestWhole), 0);
}

#[test]
fn whole_note_rest_and_chord_keep_whole_glyphs() {
    let svg = note_rest_chord_score(Duration::WHOLE);
    assert_eq!(glyph_count(&svg, Glyph::NoteheadWhole), 4);
    assert_eq!(glyph_count(&svg, Glyph::RestWhole), 1);
    assert_eq!(glyph_count(&svg, Glyph::NoteheadDoubleWhole), 0);
    assert_eq!(glyph_count(&svg, Glyph::RestDoubleWhole), 0);
}

#[test]
fn breves_are_stemless_like_whole_notes() {
    // All pitches sit inside the staff, so every `<line>` is a staff line,
    // barline, or stem. Breves must add no stems beyond what the stemless
    // whole-note score draws, while half notes add one stem per note/chord.
    let lines = |svg: &str| svg.matches("<line ").count();
    let breve_svg = note_rest_chord_score(breve());
    let whole_svg = note_rest_chord_score(Duration::WHOLE);
    let half_svg = note_rest_chord_score(Duration::HALF);
    assert_eq!(lines(&breve_svg), lines(&whole_svg));
    assert_eq!(lines(&half_svg), lines(&whole_svg) + 2);
}

#[test]
fn guitar_breves_render_double_whole_glyphs_on_the_standard_staff() {
    let mut score = GuitarScore::standard();
    score.set_time_signature(4, 2);
    score.note(Pitch::new(Note::G, 3), breve(), 3, 0).unwrap();
    score.barline().unwrap();
    score.rest(breve());
    score.end_barline().unwrap();
    let svg = score.render_svg();
    assert_eq!(glyph_count(&svg, Glyph::NoteheadDoubleWhole), 1);
    assert_eq!(glyph_count(&svg, Glyph::RestDoubleWhole), 1);
    assert_eq!(glyph_count(&svg, Glyph::NoteheadWhole), 0);
    assert_eq!(glyph_count(&svg, Glyph::RestWhole), 0);
}
