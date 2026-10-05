//! End-to-end group-span behavior at the public score builder and SVG surface.

use crate::font::bravura_font;
use crate::layout::group::{
    BeamSpec, GroupMark, TupletBracketVisibility, TupletNumberDisplay, TupletSpec,
};
use crate::layout::lyric::LyricSyllable;
use crate::layout::stem::StemDirection;
use crate::layout::system::MeasureEvent;
use crate::layout::tuplet::TupletPlacement;
use crate::score::ScoreBuilder;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

fn p(note: Note) -> Pitch {
    Pitch::new(note, 4)
}
fn beams(svg: &str) -> usize {
    svg.matches("<polygon ").count()
}

#[test]
fn beamed_rest_and_chord_keep_kinds_and_stems() {
    let builder = ScoreBuilder::new()
        .begin_beam_with(BeamSpec::new().stem_direction(StemDirection::Down))
        .chord(vec![p(Note::C), p(Note::E)], Duration::EIGHTH)
        .rest(Duration::EIGHTH)
        .note(p(Note::G), Duration::EIGHTH)
        .end_beam()
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    let events = &contents[0].events;
    assert!(matches!(events[1], MeasureEvent::Chord(_)));
    assert!(matches!(events[2], MeasureEvent::Rest(_)));
    assert!(matches!(events[3], MeasureEvent::Note(_)));
    if let MeasureEvent::Chord(chord) = &events[1] {
        assert_eq!(chord.staff_positions.len(), 2);
        assert_eq!(chord.stem_direction, Some(StemDirection::Down));
    }
    let svg = builder.render_svg();
    assert!(
        beams(&svg) > 0,
        "a rest must not interrupt its enclosing beam"
    );
    let rest_glyph = &bravura_font()
        .glyph_outline(smufl::Glyph::Rest8th)
        .unwrap()
        .path_data;
    let rest = svg
        .lines()
        .find(|line| line.contains(rest_glyph))
        .expect("the beamed rest retains its own eighth-rest glyph");
    let rest_x: f64 = rest
        .split("translate(")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let attr = |line: &str, name: &str| -> Option<f64> {
        line.split(&format!("{name}=\""))
            .nth(1)?
            .split('"')
            .next()?
            .parse()
            .ok()
    };
    assert!(
        !svg.lines().any(|line| line.starts_with("  <line ")
            && attr(line, "x1").is_some_and(|x| (x - rest_x).abs() < 1e-6)
            && attr(line, "x1") == attr(line, "x2")),
        "a rest cannot acquire the stem of its enclosing beam"
    );
}

#[test]
fn consecutive_beamed_chords_keep_both_tones_and_their_accidentals() {
    let score = ScoreBuilder::new()
        .begin_beam()
        .chord(vec![p(Note::C), p(Note::E)], Duration::EIGHTH)
        .chord(vec![p(Note::D), p(Note::Fis)], Duration::EIGHTH)
        .end_beam()
        .end_barline();
    let contents = score.build_measure_contents().unwrap();
    let chords: Vec<_> = contents[0]
        .events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Chord(chord) => Some(chord),
            _ => None,
        })
        .collect();
    assert_eq!(chords.len(), 2);
    assert!(chords.iter().all(|chord| chord.staff_positions.len() == 2));
    assert!(
        chords[1].accidentals[1].is_some(),
        "second chord retains its F-sharp"
    );
    let svg = score.render_svg();
    let head = bravura_font()
        .glyph_outline(smufl::Glyph::NoteheadBlack)
        .unwrap()
        .path_data;
    assert_eq!(svg.matches(&head).count(), 4);
    assert!(beams(&svg) > 0);
}

#[test]
fn unbeamed_mixed_triplets_and_partial_inner_beam() {
    let triplet = TupletSpec::new(3, 2).bracket(TupletBracketVisibility::Always);
    let rest_triplet = ScoreBuilder::new()
        .begin_tuplet(triplet)
        .rest(Duration::EIGHTH)
        .note(p(Note::B), Duration::EIGHTH)
        .note(p(Note::A), Duration::EIGHTH)
        .end_tuplet()
        .end_barline();
    let svg = rest_triplet.render_svg();
    assert_eq!(beams(&svg), 0, "tuplets must not imply beams");
    assert!(svg.contains(
        &bravura_font()
            .glyph_outline(smufl::Glyph::Tuplet3)
            .unwrap()
            .path_data
    ));
    let mixed = ScoreBuilder::new()
        .begin_tuplet(triplet)
        .note(p(Note::D), Duration::QTR)
        .note(p(Note::E), Duration::EIGHTH)
        .end_tuplet()
        .end_barline();
    assert_eq!(beams(&mixed.render_svg()), 0);
    let quarter_triplet = ScoreBuilder::new()
        .begin_tuplet(triplet)
        .note(p(Note::C), Duration::QTR)
        .note(p(Note::D), Duration::QTR)
        .begin_beam()
        .note(p(Note::E), Duration::EIGHTH)
        .note(p(Note::F), Duration::EIGHTH)
        .end_beam()
        .end_tuplet()
        .end_barline();
    let svg = quarter_triplet.render_svg();
    assert!(beams(&svg) > 0, "only the eighths receive the inner beam");
    assert!(svg.contains(
        &bravura_font()
            .glyph_outline(smufl::Glyph::Tuplet3)
            .unwrap()
            .path_data
    ));
}

#[test]
fn chord_triplet_number_ratio_placement_and_visibility() {
    let spec = TupletSpec::new(3, 2)
        .number_display(TupletNumberDisplay::Ratio)
        .bracket(TupletBracketVisibility::Always)
        .placement(TupletPlacement::Below);
    let chord = || vec![p(Note::C), p(Note::E)];
    let builder = ScoreBuilder::new()
        .begin_tuplet(spec)
        .chord(chord(), Duration::EIGHTH)
        .chord(chord(), Duration::EIGHTH)
        .chord(chord(), Duration::EIGHTH)
        .end_tuplet()
        .end_barline();
    let events = builder.build_measure_contents().unwrap();
    assert_eq!(
        events[0]
            .events
            .iter()
            .filter(|event| matches!(event, MeasureEvent::Chord(_)))
            .count(),
        3
    );
    let svg = builder.render_svg();
    assert!(svg.contains(
        &bravura_font()
            .glyph_outline(smufl::Glyph::TupletColon)
            .unwrap()
            .path_data
    ));
    assert_eq!(beams(&svg), 0);
    let hidden = ScoreBuilder::new()
        .begin_tuplet(
            spec.number_display(TupletNumberDisplay::Hidden)
                .bracket(TupletBracketVisibility::Never),
        )
        .note(p(Note::C), Duration::QTR)
        .note(p(Note::D), Duration::QTR)
        .note(p(Note::E), Duration::QTR)
        .end_tuplet()
        .end_barline()
        .render_svg();
    assert!(!hidden.contains(
        &bravura_font()
            .glyph_outline(smufl::Glyph::TupletColon)
            .unwrap()
            .path_data
    ));
}

#[test]
fn quarter_triplet_remains_unbeamed_and_forced_tuplet_side_moves_number() {
    let draw = |placement| {
        ScoreBuilder::new()
            .begin_tuplet(
                TupletSpec::new(3, 2)
                    .bracket(TupletBracketVisibility::Always)
                    .placement(placement),
            )
            .note(p(Note::C), Duration::QTR)
            .note(p(Note::D), Duration::QTR)
            .note(p(Note::E), Duration::QTR)
            .end_tuplet()
            .end_barline()
            .render_svg()
    };
    let above = draw(TupletPlacement::Above);
    let below = draw(TupletPlacement::Below);
    let numeral = bravura_font()
        .glyph_outline(smufl::Glyph::Tuplet3)
        .unwrap()
        .path_data;
    let number_y = |svg: &str| -> f64 {
        let glyph = svg
            .lines()
            .find(|line| line.contains(&numeral))
            .expect("quarter triplet prints a tuplet numeral");
        glyph
            .split("translate(")
            .nth(1)
            .unwrap()
            .split(',')
            .nth(1)
            .unwrap()
            .split(')')
            .next()
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    };
    assert_eq!(
        beams(&above),
        0,
        "quarter triplets may have brackets but not beams"
    );
    assert_eq!(beams(&below), 0);
    assert!(
        number_y(&above) < number_y(&below),
        "forced tuplet side must move the entire numbered bracket"
    );
}

#[test]
fn beam_continuing_across_a_system_break_draws_both_open_ends() {
    let svg = ScoreBuilder::new()
        .measures_per_system(1)
        .begin_beam()
        .note(p(Note::F), Duration::EIGHTH)
        .barline()
        .note(p(Note::G), Duration::EIGHTH)
        .end_beam()
        .end_barline()
        .render_svg();
    assert!(
        beams(&svg) >= 2,
        "the first and next system both need visible broken-beam ends"
    );
}

#[test]
fn beam_crosses_barline_and_subdivides_secondary_beams() {
    let cross = ScoreBuilder::new()
        .begin_beam()
        .note(p(Note::F), Duration::EIGHTH)
        .barline()
        .note(p(Note::G), Duration::EIGHTH)
        .end_beam()
        .end_barline();
    let contents = cross.build_measure_contents().unwrap();
    assert!(matches!(
        contents[0].events.last(),
        Some(MeasureEvent::GroupMark(GroupMark::BeamEnd {
            continues: true
        }))
    ));
    assert!(matches!(
        contents[1].events.first(),
        Some(MeasureEvent::GroupMark(GroupMark::BeamStart {
            continued: true,
            ..
        }))
    ));
    assert!(beams(&cross.render_svg()) > 0);
    let notes = || {
        ScoreBuilder::new()
            .begin_beam_with(BeamSpec::new().subdivide(3))
            .note(p(Note::C), Duration::SIXTEENTH)
            .note(p(Note::D), Duration::SIXTEENTH)
            .note(p(Note::E), Duration::SIXTEENTH)
            .note(p(Note::F), Duration::SIXTEENTH)
            .end_beam()
            .end_barline()
    };
    let subdivided = notes().render_svg();
    assert!(beams(&subdivided) > 0);
    let plain = ScoreBuilder::new()
        .begin_beam()
        .note(p(Note::C), Duration::SIXTEENTH)
        .note(p(Note::D), Duration::SIXTEENTH)
        .note(p(Note::E), Duration::SIXTEENTH)
        .note(p(Note::F), Duration::SIXTEENTH)
        .end_beam()
        .end_barline()
        .render_svg();
    assert_ne!(
        subdivided, plain,
        "subdividing eighth beats changes secondary beam geometry"
    );
}

#[test]
fn annotations_on_tuplets_and_beam_members_have_visible_endpoints() {
    let builder = ScoreBuilder::new()
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::C), Duration::EIGHTH)
        .slur_start()
        .tie()
        .lyric(LyricSyllable::with_hyphen("hap"))
        .note(p(Note::C), Duration::EIGHTH)
        .slur_end()
        .lyric(LyricSyllable::word("py"))
        .note(p(Note::D), Duration::EIGHTH)
        .end_beam()
        .end_tuplet()
        .end_barline();
    let svg = builder.clone().render_svg();
    assert!(svg.contains(">hap<") && svg.contains(">py<"));
    assert!(
        svg.lines()
            .filter(|line| line.starts_with("  <path ") && !line.contains("transform="))
            .count()
            >= 2,
        "both the slur and tie curves must reach beamed tuplet members"
    );
    let events = builder.build_measure_contents().unwrap();
    let notes: Vec<_> = events[0]
        .events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(n) => Some(n),
            _ => None,
        })
        .collect();
    assert!(notes[0].annotations.slur_start && notes[0].annotations.tie_forward);
    assert!(notes[1].annotations.slur_end);
}

#[test]
fn malformed_group_spans_return_specific_render_errors() {
    use crate::error::EngraverError;
    use crate::score::GroupSpanError;

    let nested = ScoreBuilder::new()
        .begin_beam()
        .note(p(Note::C), Duration::EIGHTH)
        .begin_beam()
        .note(p(Note::D), Duration::EIGHTH)
        .end_beam()
        .end_barline();
    assert!(matches!(
        nested.try_render_svg(),
        Err(EngraverError::Group(GroupSpanError::NestedBeam { .. }))
    ));

    let unbeamable = ScoreBuilder::new()
        .begin_beam()
        .note(p(Note::C), Duration::QTR)
        .end_beam()
        .end_barline();
    assert!(matches!(
        unbeamable.try_render_svg(),
        Err(EngraverError::Group(GroupSpanError::UnbeamableNote { .. }))
    ));

    let open = ScoreBuilder::new()
        .begin_tuplet(TupletSpec::new(3, 2))
        .rest(Duration::EIGHTH)
        .end_barline();
    assert!(matches!(
        open.try_render_svg(),
        Err(EngraverError::Group(GroupSpanError::UnclosedTuplet { .. }))
    ));
}

#[test]
fn nested_tuplet_onsets_apply_both_ratios_to_mixed_members() {
    use crate::layout::group::scan_groups;
    use crate::layout::measure::MeasureLayoutConfig;
    use crate::layout::system::layout_system;
    let builder = ScoreBuilder::new()
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::C), Duration::QTR)
        .begin_tuplet(TupletSpec::new(5, 4))
        .rest(Duration::EIGHTH)
        .chord(vec![p(Note::D), p(Note::Fis)], Duration::EIGHTH)
        .end_tuplet()
        .end_tuplet()
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    use crate::layout::measure_meta::MeasureLength;
    assert_eq!(contents[0].meta.actual_length, MeasureLength::new(3, 10));
    let cfg = MeasureLayoutConfig::from_staff_space(bravura_font().engraving_config().staff_space);
    let system = layout_system(&builder.build_prefix(), &contents, &cfg, None);
    let elements = &system.measures[0].layout.elements;
    let scan = scan_groups(elements.iter().map(|item| &item.element));
    let notes: Vec<_> = elements
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            matches!(
                item.element,
                crate::layout::measure::MeasureElement::Note(_)
                    | crate::layout::measure::MeasureElement::Chord(_)
                    | crate::layout::measure::MeasureElement::Rest(_)
            )
        })
        .map(|(index, item)| (scan.onsets[index], &item.element))
        .collect();
    assert_eq!(notes.len(), 3);
    assert!(
        (notes[1].0 - 1.0 / 6.0).abs() < 1e-9,
        "nested rest begins after outer quarter"
    );
    assert!((notes[2].0 - (1.0 / 6.0 + 1.0 / 8.0 * 2.0 / 3.0 * 4.0 / 5.0)).abs() < 1e-9);
}

#[test]
fn glissando_and_dynamic_attach_to_group_member_noteheads() {
    use crate::layout::dynamics::Dynamic;
    use crate::layout::glissando::{GlissandoStyle, GLISSANDO_H_PADDING_SS};
    use crate::layout::hairpin::HairpinType;
    let builder = ScoreBuilder::new()
        .begin_beam()
        .note(p(Note::C), Duration::EIGHTH)
        .glissando(GlissandoStyle::LineWithText)
        .dynamic(Dynamic::Piano)
        .hairpin_start(HairpinType::Crescendo)
        .note(p(Note::G), Duration::EIGHTH)
        .note(p(Note::A), Duration::EIGHTH)
        .hairpin_end()
        .end_beam()
        .end_barline();
    let svg = builder.render_svg();
    let head = bravura_font()
        .glyph_outline(smufl::Glyph::NoteheadBlack)
        .unwrap()
        .path_data;
    let heads: Vec<f64> = svg
        .lines()
        .filter(|line| line.contains(&head))
        .filter_map(|line| line.split("translate(").nth(1))
        .map(|xy| xy.split(',').next().unwrap().parse::<f64>().unwrap())
        .collect();
    assert_eq!(heads.len(), 3);
    let gliss = svg
        .lines()
        .find(|line| line.starts_with("  <line ") && line.contains("stroke-width=\"20\""))
        .expect("slanted glissando must render");
    let value = |key: &str| -> f64 {
        gliss
            .split(&format!("{key}=\""))
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap()
    };
    let ss = bravura_font().engraving_config().staff_space;
    assert!((value("x1") - (heads[0] + (1.18 + GLISSANDO_H_PADDING_SS) * ss)).abs() < 0.01);
    assert!((value("x2") - (heads[1] - GLISSANDO_H_PADDING_SS * ss)).abs() < 0.01);
    assert!(
        svg.contains("gliss."),
        "the start member's labeled glissando stays visible"
    );
    let piano = bravura_font()
        .glyph_outline(smufl::Glyph::DynamicPiano)
        .unwrap()
        .path_data;
    assert!(
        svg.contains(&piano),
        "the start member's dynamic stays visible"
    );
    let ends: Vec<f64> = svg
        .lines()
        .filter(|line| line.starts_with("  <line "))
        .filter_map(|line| {
            let attr = |key: &str| -> Option<f64> {
                line.split(&format!("{key}=\""))
                    .nth(1)?
                    .split('"')
                    .next()?
                    .parse()
                    .ok()
            };
            (attr("y1")? > 1000.0
                && attr("y2")? > 1000.0
                && (attr("x2")? - (heads[2] - 0.3 * ss)).abs() < 1.0)
                .then(|| attr("x2").unwrap())
        })
        .collect();
    assert_eq!(
        ends.len(),
        2,
        "both hairpin edges end beside the final beamed note"
    );
}

#[test]
fn post_span_notehead_marks_still_target_the_last_rhythmic_member() {
    use crate::layout::articulation::{Articulation, ArticulationMark};
    use crate::layout::breath::BreathMark;
    use crate::layout::measure::NoteSize;
    let score = ScoreBuilder::new()
        .begin_beam()
        .note(p(Note::C), Duration::EIGHTH)
        .note(p(Note::D), Duration::EIGHTH)
        .end_beam()
        .articulation_mark(ArticulationMark::from(Articulation::Staccato))
        .parenthesized_breath_mark(BreathMark::Comma)
        .parenthesize()
        .note_size(NoteSize::Cue)
        .end_barline();
    let contents = score.build_measure_contents().unwrap();
    let MeasureEvent::Note(note) = &contents[0].events[2] else {
        panic!("the last beam member is an ordinary note");
    };
    assert_eq!(note.annotations.articulations.len(), 1);
    assert!(note.annotations.breath_mark_parenthesized);
    assert_eq!(note.annotations.parenthesized_noteheads, vec![true]);
    assert_eq!(note.annotations.size, NoteSize::Cue);
    let svg = score.render_svg();
    assert!(svg.contains(
        &bravura_font()
            .glyph_outline(smufl::Glyph::NoteheadParenthesisLeft)
            .unwrap()
            .path_data
    ));
}

#[test]
fn rest_member_marks_render_at_the_rest_not_the_adjacent_note() {
    use crate::layout::dynamics::Dynamic;
    use crate::layout::hairpin::HairpinType;
    use crate::layout::tempo::TempoMark;
    use crate::layout::text_script::TextScript;
    use crate::svg_probe::{glyph, text};

    let score = ScoreBuilder::new()
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::C), Duration::EIGHTH)
        .rest(Duration::EIGHTH)
        .dynamic(Dynamic::Piano)
        .hairpin_start(HairpinType::Crescendo)
        .text_script(TextScript::above("rest mark"))
        .tempo(TempoMark::text("Andante"))
        .note(p(Note::G), Duration::EIGHTH)
        .hairpin_end()
        .end_beam()
        .end_tuplet()
        .end_barline();
    let contents = score.build_measure_contents().unwrap();
    let MeasureEvent::Rest(rest) = &contents[0].events[3] else {
        panic!("the marked tuplet member must remain a rest");
    };
    assert!(rest.annotations.dynamic.is_some());
    assert!(rest.annotations.hairpin_start.is_some());
    assert_eq!(rest.annotations.text_scripts.len(), 1);
    assert!(rest.annotations.tempo_mark.is_some());

    let svg = score.render_svg();
    let font = bravura_font();
    let rest = glyph(&svg, &font, smufl::Glyph::Rest8th);
    let dynamic = glyph(&svg, &font, smufl::Glyph::DynamicPiano);
    let rest_center = rest.x + f64::from(font.glyph_advance(smufl::Glyph::Rest8th).unwrap()) / 2.0;
    let dynamic_center =
        dynamic.x + f64::from(font.glyph_advance(smufl::Glyph::DynamicPiano).unwrap()) / 2.0;
    assert!((rest_center - dynamic_center).abs() < 0.01);
    assert!((text(&svg, "rest mark").x - rest.x).abs() < 0.01);
    assert!((text(&svg, "Andante").x - rest.x).abs() < 0.01);
    assert!(beams(&svg) > 0);
}

#[test]
fn inline_barline_break_splits_an_open_tuplet_and_beam_on_the_performed_onset() {
    use crate::layout::barline::BarlineStyle;
    let score = ScoreBuilder::new()
        .time_signature(4, 4)
        .begin_tuplet(TupletSpec::new(3, 2))
        .begin_beam()
        .note(p(Note::C), Duration::EIGHTH)
        .system_break()
        .inline_barline(BarlineStyle::Dashed)
        .note(p(Note::D), Duration::EIGHTH)
        .end_beam()
        .rest(Duration::EIGHTH)
        .end_tuplet()
        .end_barline();
    let mut for_layout = score.clone();
    let page = for_layout
        .page_layout(bravura_font().engraving_config().staff_space)
        .unwrap()
        .unwrap();
    assert_eq!(page.systems.len(), 2);
    assert_eq!(page.systems[0].system.measures.len(), 1);
    assert_eq!(page.systems[1].system.measures.len(), 1);
    let svg = score.render_svg();
    assert!(beams(&svg) >= 2, "each system must retain its partial beam");
    let numeral = bravura_font()
        .glyph_outline(smufl::Glyph::Tuplet3)
        .unwrap()
        .path_data;
    assert!(
        svg.contains(&numeral),
        "the divided tuplet must keep its number"
    );
}
