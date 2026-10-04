//! End-to-end group-span behavior at the public score builder and SVG surface.

use crate::font::bravura_font;
use crate::layout::group::{BeamSpec, GroupMark, TupletBracketVisibility, TupletNumberDisplay, TupletSpec};
use crate::layout::lyric::LyricSyllable;
use crate::layout::stem::StemDirection;
use crate::layout::system::MeasureEvent;
use crate::layout::tuplet::TupletPlacement;
use crate::score::ScoreBuilder;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

fn p(note: Note) -> Pitch { Pitch::new(note, 4) }
fn beams(svg: &str) -> usize { svg.matches("<polygon ").count() }

#[test]
fn beamed_rest_and_chord_keep_kinds_and_stems() {
    let builder = ScoreBuilder::new()
        .begin_beam_with(BeamSpec::new().stem_direction(StemDirection::Down))
        .chord(vec![p(Note::C), p(Note::E)], Duration::EIGHTH)
        .rest(Duration::EIGHTH)
        .note(p(Note::G), Duration::EIGHTH)
        .end_beam().end_barline();
    let events = &builder.build_measure_contents()[0].events;
    assert!(matches!(events[1], MeasureEvent::Chord(_)));
    assert!(matches!(events[2], MeasureEvent::Rest(_)));
    assert!(matches!(events[3], MeasureEvent::Note(_)));
    if let MeasureEvent::Chord(chord) = &events[1] {
        assert_eq!(chord.staff_positions.len(), 2);
        assert_eq!(chord.stem_direction, Some(StemDirection::Down));
    }
    let svg = builder.render_svg();
    assert!(beams(&svg) > 0, "a rest must not interrupt its enclosing beam");
    assert!(svg.contains(&bravura_font().glyph_outline(smufl::Glyph::Rest8th).unwrap().path_data));
}

#[test]
fn unbeamed_mixed_triplets_and_partial_inner_beam() {
    let triplet = TupletSpec::new(3, 2).bracket(TupletBracketVisibility::Always);
    let rest_triplet = ScoreBuilder::new().begin_tuplet(triplet)
        .rest(Duration::EIGHTH).note(p(Note::B), Duration::EIGHTH)
        .note(p(Note::A), Duration::EIGHTH).end_tuplet().end_barline();
    let svg = rest_triplet.render_svg();
    assert_eq!(beams(&svg), 0, "tuplets must not imply beams");
    assert!(svg.contains(&bravura_font().glyph_outline(smufl::Glyph::Tuplet3).unwrap().path_data));
    let mixed = ScoreBuilder::new().begin_tuplet(triplet)
        .note(p(Note::D), Duration::QTR).note(p(Note::E), Duration::EIGHTH)
        .end_tuplet().end_barline();
    assert_eq!(beams(&mixed.render_svg()), 0);
    let quarter_triplet = ScoreBuilder::new().begin_tuplet(triplet)
        .note(p(Note::C), Duration::QTR).note(p(Note::D), Duration::QTR)
        .begin_beam().note(p(Note::E), Duration::EIGHTH)
        .note(p(Note::F), Duration::EIGHTH).end_beam().end_tuplet().end_barline();
    let svg = quarter_triplet.render_svg();
    assert!(beams(&svg) > 0, "only the eighths receive the inner beam");
    assert!(svg.contains(&bravura_font().glyph_outline(smufl::Glyph::Tuplet3).unwrap().path_data));
}

#[test]
fn chord_triplet_number_ratio_placement_and_visibility() {
    let spec = TupletSpec::new(3, 2).number_display(TupletNumberDisplay::Ratio)
        .bracket(TupletBracketVisibility::Always).placement(TupletPlacement::Below);
    let chord = || vec![p(Note::C), p(Note::E)];
    let builder = ScoreBuilder::new().begin_tuplet(spec)
        .chord(chord(), Duration::EIGHTH).chord(chord(), Duration::EIGHTH)
        .chord(chord(), Duration::EIGHTH).end_tuplet().end_barline();
    let events = builder.build_measure_contents();
    assert_eq!(events[0].events.iter().filter(|event| matches!(event, MeasureEvent::Chord(_))).count(), 3);
    let svg = builder.render_svg();
    assert!(svg.contains(&bravura_font().glyph_outline(smufl::Glyph::TupletColon).unwrap().path_data));
    assert_eq!(beams(&svg), 0);
    let hidden = ScoreBuilder::new().begin_tuplet(spec.number_display(TupletNumberDisplay::Hidden)
        .bracket(TupletBracketVisibility::Never))
        .note(p(Note::C), Duration::QTR).note(p(Note::D), Duration::QTR)
        .note(p(Note::E), Duration::QTR).end_tuplet().end_barline().render_svg();
    assert!(!hidden.contains(&bravura_font().glyph_outline(smufl::Glyph::TupletColon).unwrap().path_data));
}

#[test]
fn beam_crosses_barline_and_subdivides_secondary_beams() {
    let cross = ScoreBuilder::new().begin_beam().note(p(Note::F), Duration::EIGHTH)
        .barline().note(p(Note::G), Duration::EIGHTH).end_beam().end_barline();
    let contents = cross.build_measure_contents();
    assert!(matches!(contents[0].events.last(), Some(MeasureEvent::GroupMark(GroupMark::BeamEnd { continues: true }))));
    assert!(matches!(contents[1].events.first(), Some(MeasureEvent::GroupMark(GroupMark::BeamStart { continued: true, .. }))));
    assert!(beams(&cross.render_svg()) > 0);
    let notes = || ScoreBuilder::new().begin_beam_with(BeamSpec::new().subdivide(3))
        .note(p(Note::C), Duration::SIXTEENTH).note(p(Note::D), Duration::SIXTEENTH)
        .note(p(Note::E), Duration::SIXTEENTH).note(p(Note::F), Duration::SIXTEENTH)
        .end_beam().end_barline();
    let subdivided = notes().render_svg();
    assert!(beams(&subdivided) > 0);
    let plain = ScoreBuilder::new().begin_beam()
        .note(p(Note::C), Duration::SIXTEENTH).note(p(Note::D), Duration::SIXTEENTH)
        .note(p(Note::E), Duration::SIXTEENTH).note(p(Note::F), Duration::SIXTEENTH)
        .end_beam().end_barline().render_svg();
    assert_ne!(subdivided, plain, "subdividing eighth beats changes secondary beam geometry");
}

#[test]
fn annotations_on_tuplets_and_beam_members_have_visible_endpoints() {
    let builder = ScoreBuilder::new().begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam().note(p(Note::C), Duration::EIGHTH)
        .slur_start().tie().lyric(LyricSyllable::with_hyphen("hap"))
        .note(p(Note::C), Duration::EIGHTH).slur_end()
        .lyric(LyricSyllable::word("py"))
        .note(p(Note::D), Duration::EIGHTH).end_beam().end_tuplet().end_barline();
    let svg = builder.clone().render_svg();
    assert!(svg.contains(">hap<") && svg.contains(">py<"));
    assert!(svg.contains("<polygon"), "tie or beam filled geometry");
    let events = builder.build_measure_contents();
    let notes: Vec<_> = events[0].events.iter().filter_map(|event| match event {
        MeasureEvent::Note(n) => Some(n), _ => None
    }).collect();
    assert!(notes[0].annotations.slur_start && notes[0].annotations.tie_forward);
    assert!(notes[1].annotations.slur_end);
}
