//! Inline barlines, explicit system breaks, and endings without a final
//! barline, through the score builders, page layout, and SVG.

use super::*;
use crate::layout::measure::{layout_measure, MeasureElement, MeasureLayout};
use crate::layout::system::measure_event_to_element;
use crate::score::guitar::{GuitarScore, GuitarScoreError};
use crate::score::multi_staff::MultiStaffScore;
use music::note::note::Note;
use smufl::Glyph;

const Q: Duration = Duration::QTR;
const W: Duration = Duration::WHOLE;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn staff_space() -> f64 {
    bravura_font().engraving_config().staff_space
}

fn page(mut builder: ScoreBuilder) -> PageLayout {
    builder
        .page_layout(staff_space())
        .expect("valid score")
        .expect("score has measures")
}

/// Number of measure contents on each system.
fn system_sizes(page: &PageLayout) -> Vec<usize> {
    page.systems
        .iter()
        .map(|system| system.system.measures.len())
        .collect()
}

/// `count` measures of four quarter notes, each closed by `barline()`.
fn quarters(mut builder: ScoreBuilder, count: usize) -> ScoreBuilder {
    for _ in 0..count {
        for note in [Note::C, Note::D, Note::E, Note::F] {
            builder = builder.note(p(note, 5), Q);
        }
        builder = builder.barline();
    }
    builder
}

/// The natural layout of one measure content (no prefix).
fn natural(content: &MeasureContent) -> MeasureLayout {
    let mut elements: Vec<MeasureElement> = content
        .events
        .iter()
        .map(measure_event_to_element)
        .collect();
    elements.push(MeasureElement::Barline(content.barline));
    layout_measure(
        &elements,
        &MeasureLayoutConfig::from_staff_space(staff_space()),
    )
}

fn note_accidentals(events: &[MeasureEvent]) -> Vec<Option<Glyph>> {
    events
        .iter()
        .filter_map(|event| match event {
            MeasureEvent::Note(note) => Some(note.accidental.map(|resolved| resolved.glyph)),
            _ => None,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Line {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    dashed: bool,
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

/// Every `<line>` of `svg`.
fn lines(svg: &str) -> Vec<Line> {
    svg.split("<line")
        .skip(1)
        .map(|tag| {
            let tag = &tag[..tag.find("/>").expect("line closes")];
            let number = |name| {
                attribute(tag, name)
                    .expect("line coordinate")
                    .parse::<f64>()
                    .expect("numeric coordinate")
            };
            Line {
                x1: number("x1"),
                y1: number("y1"),
                x2: number("x2"),
                y2: number("y2"),
                dashed: attribute(tag, "stroke-dasharray").is_some(),
            }
        })
        .collect()
}

/// `translate(x, y)` of every `<path>` drawing `glyph`.
fn glyph_origins(svg: &str, glyph: Glyph) -> Vec<(f64, f64)> {
    let data = bravura_font().glyph_outline(glyph).unwrap().path_data;
    svg.split("<path")
        .skip(1)
        .filter(|tag| attribute(tag, "d") == Some(data.as_str()))
        .map(|tag| {
            let transform = attribute(tag, "transform").expect("glyph is translated");
            let inner = &transform["translate(".len()..transform.len() - 1];
            let (x, y) = inner.split_once(", ").expect("two coordinates");
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

// --- inline barlines ---

#[test]
fn inline_barline_stays_inside_its_measure_and_keeps_accidental_state() {
    let builder = ScoreBuilder::new()
        .note(p(Note::Fis, 4), Q)
        .inline_barline(BarlineStyle::Dashed)
        .note(p(Note::Fis, 4), Q)
        .barline()
        .note(p(Note::Fis, 4), Q)
        .end_barline();

    let contents = builder.build_measure_contents().unwrap();
    assert_eq!(
        contents.len(),
        2,
        "the inline barline does not end a measure"
    );
    assert!(matches!(
        contents[0].events[1],
        MeasureEvent::Barline(BarlineStyle::Dashed)
    ));
    // F#4 | inline | F#4: the second sharp is suppressed; the real barline
    // resets the state, so the next measure's F#4 shows it again.
    assert_eq!(
        note_accidentals(&contents[0].events),
        vec![Some(Glyph::AccidentalSharp), None]
    );
    assert_eq!(
        note_accidentals(&contents[1].events),
        vec![Some(Glyph::AccidentalSharp)]
    );

    let page = page(builder);
    assert_eq!(system_sizes(&page), vec![2]);
    let elements = &page.systems[0].system.measures[0].layout.elements;
    let xs: Vec<(f64, f64, bool)> = elements
        .iter()
        .filter_map(|element| match element.element {
            MeasureElement::Note(_) => Some((element.x, element.width, false)),
            MeasureElement::Barline(BarlineStyle::Dashed) => Some((element.x, element.width, true)),
            _ => None,
        })
        .collect();
    let [(first_x, first_width, false), (barline_x, _, true), (second_x, _, false)] = xs[..] else {
        panic!("note, dashed barline, note in order: {xs:?}");
    };
    assert!(
        first_x + first_width <= barline_x && barline_x < second_x,
        "barline at {barline_x} must sit between the notes ({first_x}+{first_width}, {second_x})"
    );
}

#[test]
fn inline_barline_does_not_advance_the_measure_number() {
    let builder = ScoreBuilder::new()
        .measures_per_system(1)
        .measure_numbering(crate::layout::bar_number::MeasureNumbering::SystemStart)
        .note(p(Note::C, 5), Q)
        .inline_barline(BarlineStyle::Dashed)
        .note(p(Note::D, 5), Q)
        .barline()
        .note(p(Note::E, 5), Q)
        .end_barline();
    let page = page(builder);
    let numbers: Vec<i32> = page
        .systems
        .iter()
        .map(|system| system.system.measures[0].meta.number)
        .collect();
    assert_eq!(numbers, vec![1, 2]);
}

#[test]
fn invisible_inline_barline_takes_no_space_and_dashed_takes_a_barline_rod() {
    let with = |style| {
        ScoreBuilder::new()
            .note(p(Note::C, 5), Q)
            .inline_barline(style)
            .note(p(Note::D, 5), Q)
            .end_barline()
            .build_measure_contents()
            .unwrap()
    };
    let plain = natural(
        &ScoreBuilder::new()
            .note(p(Note::C, 5), Q)
            .note(p(Note::D, 5), Q)
            .end_barline()
            .build_measure_contents()
            .unwrap()[0],
    );
    let invisible = natural(&with(BarlineStyle::Invisible)[0]);
    let dashed = natural(&with(BarlineStyle::Dashed)[0]);

    let barline = &invisible.elements[1];
    assert!(matches!(
        barline.element,
        MeasureElement::Barline(BarlineStyle::Invisible)
    ));
    assert_eq!(
        (barline.rod, barline.spring, barline.width),
        (0.0, 0.0, 0.0)
    );
    assert_eq!(invisible.total_rod, plain.total_rod);
    assert_eq!(invisible.total_width, plain.total_width);
    assert_eq!(invisible.elements[2].x, plain.elements[1].x);

    let barline_width = MeasureLayoutConfig::from_staff_space(staff_space()).barline_width;
    assert!((dashed.total_rod - plain.total_rod - barline_width).abs() < 1e-9);
    assert!((dashed.elements[2].x - plain.elements[1].x - barline_width).abs() < 1e-9);
}

#[test]
fn dashed_and_tick_barlines_render_their_smufl_glyphs() {
    let svg = ScoreBuilder::new()
        .note(p(Note::C, 5), Q)
        .inline_barline(BarlineStyle::Tick)
        .note(p(Note::D, 5), Q)
        .barline_style(BarlineStyle::Dashed)
        .note(p(Note::E, 5), Q)
        .end_barline()
        .render_svg();
    assert_eq!(glyph_origins(&svg, Glyph::BarlineDashed).len(), 1);
    assert_eq!(glyph_origins(&svg, Glyph::BarlineTick).len(), 1);
}

// --- endings ---

/// Vertical lines exactly spanning the first system's staff (barline strokes).
fn staff_height_lines(svg: &str) -> usize {
    let staff = crate::layout::staff::StaffLayout::new(0.0, 0.0, 1.0, staff_space());
    let (top, bottom) = (staff.y_of(8), staff.y_of(0));
    lines(svg)
        .iter()
        .filter(|line| line.x1 == line.x2 && line.y1 == top && line.y2 == bottom)
        .count()
}

#[test]
fn ending_with_an_invisible_barline_draws_no_final_barline() {
    let incomplete = |builder: ScoreBuilder| builder.note(p(Note::A, 4), Q);
    let unmarked = incomplete(ScoreBuilder::new())
        .barline_style(BarlineStyle::Invisible)
        .render_svg();
    let final_bar = incomplete(ScoreBuilder::new()).end_barline().render_svg();
    let flushed = incomplete(ScoreBuilder::new()).render_svg();
    assert_eq!(staff_height_lines(&unmarked), 0);
    assert_eq!(staff_height_lines(&final_bar), 2, "thin + thick strokes");
    assert_eq!(
        flushed, final_bar,
        "an unclosed score still ends with Final"
    );
}

// --- system breaks ---

#[test]
fn system_break_forces_a_break_and_restarts_the_fixed_count() {
    let builder = quarters(ScoreBuilder::new().measures_per_system(4), 2).system_break();
    assert_eq!(system_sizes(&page(quarters(builder, 4))), vec![2, 4]);
}

#[test]
fn system_break_after_a_measures_last_event_breaks_at_its_barline() {
    let builder = quarters(ScoreBuilder::new().measures_per_system(4), 1)
        .note(p(Note::C, 5), W)
        .system_break()
        .barline();
    assert_eq!(system_sizes(&page(quarters(builder, 3))), vec![2, 3]);
}

#[test]
fn no_break_keeps_measures_on_one_system() {
    let builder = quarters(ScoreBuilder::new().measures_per_system(1), 1).no_break();
    assert_eq!(system_sizes(&page(quarters(builder, 2))), vec![2, 1]);
}

#[test]
fn explicit_line_breaks_break_nowhere_else_even_when_content_overflows() {
    let sixteenths = |mut builder: ScoreBuilder, measures: usize| {
        for _ in 0..measures {
            for _ in 0..16 {
                builder = builder.note(p(Note::G, 4), Duration::SIXTEENTH);
            }
            builder = builder.barline();
        }
        builder
    };
    let narrow = ScoreBuilder::new().system_width_fu(4000.0);
    let explicit = sixteenths(
        sixteenths(narrow.clone().explicit_line_breaks(), 4).system_break(),
        4,
    );
    assert_eq!(system_sizes(&page(explicit)), vec![4, 4]);

    let auto = sixteenths(sixteenths(narrow.auto_line_breaks(), 4).system_break(), 4);
    assert_eq!(
        system_sizes(&page(auto)),
        vec![1; 8],
        "width-based breaking would break every overflowing measure"
    );
}

#[test]
fn break_at_an_inline_invisible_barline_splits_one_logical_measure() {
    let builder = ScoreBuilder::new()
        .clef(Clef::Bass)
        .explicit_line_breaks()
        .note(p(Note::Fis, 2), Q)
        .note(p(Note::G, 2), Q)
        .inline_barline(BarlineStyle::Invisible)
        .system_break()
        .note(p(Note::Fis, 2), Q)
        .note(p(Note::A, 2), Q)
        .barline()
        .note(p(Note::Fis, 2), W)
        .end_barline();
    let page = page(builder);

    assert_eq!(system_sizes(&page), vec![1, 2]);
    let first = &page.systems[0].system.measures[0];
    let continued = &page.systems[1].system.measures[0];
    let next = &page.systems[1].system.measures[1];
    // The first piece closes with the invisible barline: no width, no glyph.
    let closing = first.layout.elements.last().unwrap();
    assert!(matches!(
        closing.element,
        MeasureElement::Barline(BarlineStyle::Invisible)
    ));
    assert_eq!(closing.width, 0.0);

    // One logical measure: shared accidental state and one measure number.
    let accidentals = |measure: &crate::layout::system::SystemMeasure| -> Vec<Option<Glyph>> {
        measure
            .layout
            .elements
            .iter()
            .filter_map(|element| match &element.element {
                MeasureElement::Note(note) => Some(note.accidental.map(|r| r.glyph)),
                _ => None,
            })
            .collect()
    };
    assert_eq!(accidentals(first), vec![Some(Glyph::AccidentalSharp), None]);
    assert_eq!(accidentals(continued), vec![None, None]);
    assert_eq!(accidentals(next), vec![Some(Glyph::AccidentalSharp)]);
}

#[test]
fn mid_measure_break_without_an_inline_barline_closes_the_piece_invisibly() {
    let mut builder = ScoreBuilder::new()
        .note(p(Note::C, 5), Q)
        .system_break()
        .note(p(Note::D, 5), Q)
        .end_barline();
    let logical = builder.build_measure_contents().unwrap();
    let pieces = builder.line_break_plan(&logical).apply(logical);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].barline, BarlineStyle::Invisible);
    assert_eq!(pieces[0].meta.line_break, LineBreak::Force);
    assert!(pieces[1].meta.continuation);
    assert_eq!(pieces[1].barline, BarlineStyle::Final);
    assert_eq!(
        system_sizes(&builder.page_layout(staff_space()).unwrap().unwrap()),
        vec![1, 1]
    );
}

// --- multi-staff ---

fn grand_staff_systems(
    upper: ScoreBuilder,
    lower: ScoreBuilder,
) -> (Vec<StaveDataOf>, Vec<(usize, usize)>) {
    let ss = staff_space();
    MultiStaffScore::grand_staff(upper, lower).staves_into_systems(
        &MeasureLayoutConfig::from_staff_space(ss),
        40.0 * ss,
        4,
    ).unwrap()
}

type StaveDataOf = crate::score::multi_staff::StaveData;

#[test]
fn every_stave_breaks_where_any_stave_requests_it() {
    let upper = quarters(quarters(ScoreBuilder::new(), 1).system_break(), 2);
    let lower = quarters(ScoreBuilder::new().clef(Clef::Bass), 3);
    let (staves, chunks) = grand_staff_systems(upper, lower);
    assert_eq!(chunks, vec![(0, 1), (1, 3)]);
    for (contents, _) in &staves {
        assert_eq!(contents.len(), 3);
        assert_eq!(contents[0].meta.line_break, LineBreak::Force);
    }
}

#[test]
fn mid_measure_break_on_one_stave_splits_every_stave_at_that_onset() {
    let upper = ScoreBuilder::new()
        .note(p(Note::C, 5), Q)
        .note(p(Note::D, 5), Q)
        .inline_barline(BarlineStyle::Invisible)
        .system_break()
        .note(p(Note::E, 5), Q)
        .note(p(Note::F, 5), Q)
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .note(p(Note::C, 3), Duration::HALF)
        .note(p(Note::G, 2), Duration::HALF)
        .end_barline();
    let (staves, chunks) = grand_staff_systems(upper, lower);
    assert_eq!(chunks, vec![(0, 1), (1, 2)]);
    let (lower_pieces, _) = &staves[1];
    assert_eq!(lower_pieces.len(), 2);
    assert_eq!(lower_pieces[0].events.len(), 1);
    assert_eq!(lower_pieces[0].barline, BarlineStyle::Invisible);
    assert!(lower_pieces[1].meta.continuation);
    assert_eq!(lower_pieces[1].barline, BarlineStyle::Final);
}

#[test]
fn joined_barlines_follow_the_closing_style_and_align_with_staff_barlines() {
    let stave = |clef: Clef, octave: i8| {
        ScoreBuilder::new()
            .clef(clef)
            .note(p(Note::C, octave), W)
            .barline_style(BarlineStyle::Dashed)
            .note(p(Note::D, octave), W)
            .barline_style(BarlineStyle::Tick)
            .note(p(Note::E, octave), W)
            .barline()
            .note(p(Note::F, octave), W)
            .barline_style(BarlineStyle::Invisible)
    };
    let svg = MultiStaffScore::grand_staff(stave(Clef::Treble, 5), stave(Clef::Bass, 3))
        .measures_per_system(4)
        .render_svg();
    let all = lines(&svg);
    let ss = staff_space();
    let vertical = |line: &&Line| line.x1 == line.x2;
    let mut staff_line_ys: Vec<f64> = all
        .iter()
        .filter(|line| line.y1 == line.y2)
        .map(|line| line.y1)
        .collect();
    staff_line_ys.sort_by(f64::total_cmp);
    staff_line_ys.dedup();
    let (upper_top, upper_bottom, lower_top, lower_bottom) = (
        staff_line_ys[0],
        staff_line_ys[4],
        staff_line_ys[5],
        staff_line_ys[9],
    );

    // Dashed: each staff draws the glyph; only the gap gets a dashed line,
    // centred on the glyphs.
    let thickness = bravura_font().engraving_config().dashed_barline_thickness * ss;
    let dashed_xs: Vec<f64> = glyph_origins(&svg, Glyph::BarlineDashed)
        .iter()
        .map(|(x, _)| x + thickness / 2.0)
        .collect();
    assert_eq!(dashed_xs.len(), 2);
    assert!((dashed_xs[0] - dashed_xs[1]).abs() < 1e-9);
    let dashed: Vec<&Line> = all.iter().filter(|line| line.dashed).collect();
    assert_eq!(dashed.len(), 1);
    assert!((dashed[0].x1 - dashed_xs[0]).abs() < 1e-9);
    assert_eq!((dashed[0].y1, dashed[0].y2), (upper_bottom, lower_top));

    // Joined solid lines: the system start and the single barline only (not
    // the tick or the invisible ending), each on a staff barline's x.
    let joined: Vec<&Line> = all
        .iter()
        .filter(vertical)
        .filter(|line| !line.dashed && line.y1 == upper_top && line.y2 == lower_bottom)
        .collect();
    assert_eq!(
        joined.len(),
        2,
        "system start + the single barline: {joined:?}"
    );
    let staff_barline_xs: Vec<f64> = all
        .iter()
        .filter(vertical)
        .filter(|line| line.y1 == upper_top && line.y2 == upper_bottom)
        .map(|line| line.x1)
        .collect();
    assert_eq!(
        staff_barline_xs.len(),
        1,
        "one single barline on the upper staff"
    );
    assert_eq!(joined[1].x1, staff_barline_xs[0]);
    let tick_x = glyph_origins(&svg, Glyph::BarlineTick)[0].0;
    assert!(joined
        .iter()
        .all(|line| line.x1 < tick_x || line.x1 > tick_x + ss));
}

// --- guitar ---

/// Three whole-note measures on the open-E string, optionally with a system
/// break after the first.
fn guitar_measures(break_after_first: bool) -> GuitarScore {
    let mut score = GuitarScore::standard();
    score.set_time_signature(4, 4);
    for (note, fret) in [(Note::E, 0), (Note::F, 1), (Note::G, 3)] {
        score.note(p(note, 4), W, 1, fret).unwrap();
        score.barline().unwrap();
        if break_after_first && fret == 0 {
            score.system_break().unwrap();
        }
    }
    score
}

#[test]
fn guitar_system_break_breaks_notation_and_tab_together() {
    let systems = |score| {
        let svg = MultiStaffScore::guitar(score)
            .measures_per_system(4)
            .render_svg();
        (
            glyph_origins(&svg, Glyph::GClef8Vb).len(),
            glyph_origins(&svg, Glyph::_6StringTabClef).len(),
        )
    };
    assert_eq!(systems(guitar_measures(false)), (1, 1));
    assert_eq!(systems(guitar_measures(true)), (2, 2));
}

#[test]
fn guitar_system_break_must_follow_a_completed_measure() {
    let mut score = GuitarScore::standard();
    assert_eq!(
        score.system_break().err(),
        Some(GuitarScoreError::LineBreakNotAtBarline)
    );
    score.note(p(Note::E, 4), Q, 1, 0).unwrap();
    assert_eq!(
        score.no_break().err(),
        Some(GuitarScoreError::LineBreakNotAtBarline)
    );
}

// --- TAB barlines ---

#[test]
fn tab_barlines_draw_dashed_tick_and_invisible_styles() {
    use crate::layout::tab::TabStaffLayout;
    let font = bravura_font();
    let config = font.engraving_config();
    let ss = config.staff_space;
    let tab = TabStaffLayout::new(0.0, 0.0, 10_000.0, ss, 6);
    let draw = |style| {
        let mut svg = crate::render::SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        super::tab::draw_measure_barline(&mut svg, &font, &config, &tab, 300.0, &style).unwrap();
        lines(&svg.to_svg())
    };
    let dashed = draw(BarlineStyle::Dashed);
    assert_eq!(dashed.len(), 1);
    assert!(dashed[0].dashed);
    assert_eq!(
        (dashed[0].x1, dashed[0].y1, dashed[0].y2),
        (300.0, tab.y_origin, tab.bottom_y())
    );
    assert_eq!(
        draw(BarlineStyle::Tick),
        vec![Line {
            x1: 300.0,
            y1: tab.y_origin - ss / 2.0,
            x2: 300.0,
            y2: tab.y_origin + ss / 2.0,
            dashed: false,
        }]
    );
    assert!(draw(BarlineStyle::Invisible).is_empty());
}
