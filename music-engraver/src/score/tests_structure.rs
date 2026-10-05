//! Bar-structure tests (RM-MN-004, RM-MN-010): mid-score clef and meter
//! changes, measure metadata (numbers, nominal/actual lengths, pickups,
//! hidden meters, cadenzas), empty bars, and bar-number placement. Each
//! test goes through the public `ScoreBuilder` and checks the converted
//! measures, the laid-out page, or the rendered SVG.

use super::*;
use crate::font::bravura_font;
use crate::layout::clef::{ClefLayout, ClefSize};
use crate::layout::measure::{MeasureElement, MeasureLayout, NoteEvent};
use crate::layout::page::PageLayout;
use crate::layout::staff::StaffLayout;
use music::notation::rhythm::duration::DurationKind;
use music::note::note::Note;
use smufl::Glyph;

/// Bravura staff space in font design units.
const SS: f64 = 250.0;

fn c5() -> Pitch {
    Pitch::new(Note::C, 5)
}

fn c3() -> Pitch {
    Pitch::new(Note::C, 3)
}

/// The findings' RM-MN-004 reproduction: treble 3/4 C5 | bass 2/4 C3.
fn rm_mn_004() -> ScoreBuilder {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(3, 4)
        .note(c5(), Duration::QTR)
        .barline()
        .clef(Clef::Bass)
        .time_signature(2, 4)
        .note(c3(), Duration::QTR)
        .end_barline()
}

/// Lay out `builder` exactly as `try_render_svg` does.
fn page_of(mut builder: ScoreBuilder) -> PageLayout {
    builder.flush_pending();
    let contents = builder.build_measure_contents().unwrap();
    let mut page_config = PageLayoutConfig::new(SS, builder.effective_system_width(SS));
    page_config.measure_numbering = builder.measure_numbering;
    layout_page(
        &builder.build_prefix(),
        &contents,
        &MeasureLayoutConfig::from_staff_space(SS),
        &page_config,
        &SystemBreaking::Fixed(builder.effective_measures_per_system()),
    )
}

fn metas(builder: &ScoreBuilder) -> Vec<MeasureMeta> {
    let mut builder = builder.clone();
    builder.flush_pending();
    builder
        .build_measure_contents()
        .unwrap()
        .into_iter()
        .map(|content| content.meta)
        .collect()
}

/// Short name of each laid-out element, for order assertions.
fn kinds(layout: &MeasureLayout) -> Vec<String> {
    layout
        .elements
        .iter()
        .map(|element| match &element.element {
            MeasureElement::Clef(clef) => format!("clef:{:?}", clef.glyph),
            MeasureElement::KeySignature(_) => "key".into(),
            MeasureElement::TimeSignature(kind) => format!("time:{kind:?}"),
            MeasureElement::Note(note) => format!("note:{}", note.staff_position),
            MeasureElement::Rest(_) => "rest".into(),
            MeasureElement::Spacer(_) => "spacer".into(),
            MeasureElement::Barline(_) => "barline".into(),
            other => format!("{other:?}").chars().take(12).collect(),
        })
        .collect()
}

fn element_x(layout: &MeasureLayout, kind: &str) -> f64 {
    let index = kinds(layout)
        .iter()
        .position(|k| k == kind)
        .unwrap_or_else(|| panic!("{kind} not in {:?}", kinds(layout)));
    layout.elements[index].x
}

fn path_data(glyph: Glyph) -> String {
    bravura_font().glyph_outline(glyph).unwrap().path_data
}

/// `(x, y)` translate of every `<path>` whose outline is exactly `glyph`.
fn glyph_origins(svg: &str, glyph: Glyph) -> Vec<(f64, f64)> {
    let needle = format!("<path d=\"{}\"", path_data(glyph));
    svg.match_indices(&needle)
        .map(|(i, _)| {
            let rest = &svg[i + needle.len()..];
            let t = rest.split("translate(").nth(1).expect("glyph transform");
            let inner = t.split(')').next().unwrap();
            let mut parts = inner.split(',').map(|v| v.trim().parse::<f64>().unwrap());
            (parts.next().unwrap(), parts.next().unwrap())
        })
        .collect()
}

/// `(x, y, text)` of every `<text>` element.
fn texts(svg: &str) -> Vec<(f64, f64, String)> {
    svg.split("<text ")
        .skip(1)
        .map(|t| {
            let attr = |name: &str| -> f64 {
                let key = format!(" {name}=\"");
                let v = format!(" {t}");
                let v = v.split(&key).nth(1).unwrap().to_owned();
                v[..v.find('"').unwrap()].parse().unwrap()
            };
            let body = t[t.find('>').unwrap() + 1..t.find("</text>").unwrap()].to_owned();
            (attr("x"), attr("y"), body)
        })
        .collect()
}

fn view_box(svg: &str) -> (f64, f64, f64, f64) {
    let v = svg.split("viewBox=\"").nth(1).unwrap();
    let mut parts = v[..v.find('"').unwrap()]
        .split(' ')
        .map(|n| n.parse::<f64>().unwrap());
    (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    )
}

// --- RM-MN-004: clef and meter changes ------------------------------------

#[test]
fn rm_mn_004_places_each_note_in_the_clef_active_at_its_onset() {
    let contents = rm_mn_004().build_measure_contents().unwrap();
    let MeasureEvent::Note(first) = &contents[0].events[0] else {
        panic!("first event must be a note")
    };
    // Treble: E4 is the bottom line (0), so C5 is position 5.
    assert_eq!(first.staff_position, 5);
    // Bass: G2 is the bottom line (0), so C3 is position 3.
    assert!(matches!(
        contents[1].events[..],
        [
            MeasureEvent::ClefChange(ClefChange {
                clef: ClefKind::Bass,
                placement: ClefChangePlacement::BeforeBarline,
            }),
            MeasureEvent::TimeSignature(TimeSignatureKind::Numeric {
                numerator: 2,
                denominator: 4,
            }),
            MeasureEvent::Note(NoteEvent {
                staff_position: 3,
                ..
            }),
        ]
    ));
}

#[test]
fn rm_mn_004_draws_change_clef_before_barline_and_meter_after_it() {
    let page = page_of(rm_mn_004());
    let system = &page.systems[0].system;
    let (first, second) = (&system.measures[0], &system.measures[1]);
    assert_eq!(
        kinds(&first.layout),
        [
            "clef:GClef",
            "time:Numeric { numerator: 3, denominator: 4 }",
            "note:5",
            "clef:FClefChange",
            "barline"
        ]
    );
    assert_eq!(
        kinds(&second.layout),
        [
            "time:Numeric { numerator: 2, denominator: 4 }",
            "note:3",
            "barline"
        ]
    );

    let config = MeasureLayoutConfig::from_staff_space(SS);
    // The change clef keeps its margin after the note's rod and its padding
    // before the barline.
    let note = &first.layout.elements[2];
    let clef = &first.layout.elements[3];
    let barline_x = element_x(&first.layout, "barline");
    let clef_ink = ClefLayout::change(&Clef::Bass).ink_box();
    assert!(clef.x >= note.x + note.width + config.clef_change_margin - 1e-6);
    assert!(
        (barline_x - (clef.x + clef_ink.x_right * SS + config.clef_padding)).abs() < 1e-6,
        "barline {barline_x} must follow the clef ink plus padding"
    );
    // The meter change opens the next measure after the barline's margin.
    assert!(
        (second.layout.elements[0].x - config.time_sig_change_margin).abs() < 1e-6,
        "meter change x {}",
        second.layout.elements[0].x
    );
    assert!(
        second.x_offset + second.layout.elements[0].x
            > first.x_offset + barline_x + config.barline_width
    );
}

#[test]
fn rm_mn_004_svg_has_change_clef_glyph_and_both_meters() {
    let svg = rm_mn_004().render_svg();
    let staff = StaffLayout::new(0.0, 0.0, 1.0, SS);
    let change = glyph_origins(&svg, Glyph::FClefChange);
    assert_eq!(change.len(), 1, "one bass change clef");
    assert_eq!(change[0].1, staff.y_of(6), "F clef on the fourth line");
    assert_eq!(glyph_origins(&svg, Glyph::GClef).len(), 1);
    assert!(
        glyph_origins(&svg, Glyph::FClef).is_empty(),
        "no full-size F clef"
    );

    let threes = glyph_origins(&svg, Glyph::TimeSig3);
    let twos = glyph_origins(&svg, Glyph::TimeSig2);
    assert_eq!((threes.len(), twos.len()), (1, 1));
    // 3/4 in the prefix, 2/4 after the change clef (which sits before the
    // barline).
    assert!(threes[0].0 < change[0].0 && change[0].0 < twos[0].0);
}

#[test]
fn change_at_a_system_break_ends_with_courtesy_and_opens_with_new_prefix() {
    let page = page_of(rm_mn_004().measures_per_system(1));
    let (first, second) = (&page.systems[0].system, &page.systems[1].system);
    assert_eq!(
        kinds(&first.measures[0].layout),
        [
            "clef:GClef",
            "time:Numeric { numerator: 3, denominator: 4 }",
            "note:5",
            "clef:FClefChange",
            "barline",
            "time:Numeric { numerator: 2, denominator: 4 }"
        ],
        "courtesy clef before, courtesy meter after the final barline"
    );
    assert_eq!(
        kinds(&second.measures[0].layout),
        [
            "clef:FClef",
            "time:Numeric { numerator: 2, denominator: 4 }",
            "note:3",
            "barline"
        ],
        "next system's prefix shows the new clef and meter"
    );
    assert_eq!(second.clef_kind, ClefKind::Bass);
    // The courtesy meter hangs past the barline, inside the staff.
    let layout = &first.measures[0].layout;
    assert!(layout.closing_barline_end() < layout.total_width);
    assert!(first.measures[0].x_offset + layout.total_width <= first.staff_width + 1e-6);
}

#[test]
fn continuation_prefix_uses_clef_in_force_without_repeating_the_meter() {
    // A mid-measure change in system 1 governs system 2's prefix.
    let page = page_of(
        ScoreBuilder::new()
            .time_signature(2, 4)
            .measures_per_system(1)
            .note(c5(), Duration::QTR)
            .clef_change(Clef::Alto)
            .note(Pitch::new(Note::C, 4), Duration::QTR)
            .barline()
            .note(Pitch::new(Note::C, 4), Duration::HALF)
            .end_barline(),
    );
    let first = &page.systems[0].system.measures[0].layout;
    assert_eq!(
        kinds(first),
        [
            "clef:GClef",
            "time:Numeric { numerator: 2, denominator: 4 }",
            "note:5",
            "clef:CClefChange",
            "note:4",
            "barline"
        ]
    );
    assert_eq!(
        kinds(&page.systems[1].system.measures[0].layout),
        ["clef:CClef", "note:4", "barline"]
    );
}

#[test]
fn mid_measure_clef_change_applies_from_its_onset_in_every_voice() {
    // The change is entered in voice 1 at beat 2; voice 0's beat-2 note is
    // already in bass clef, its beat-1 note still in treble.
    let contents = ScoreBuilder::new()
        .note(c5(), Duration::QTR)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .voice(1)
        .rest(Duration::QTR)
        .clef_change(Clef::Bass)
        .note(c3(), Duration::QTR)
        .end_barline()
        .build_measure_contents()
        .unwrap();
    let positions: Vec<_> = contents[0]
        .events
        .iter()
        .map(|event| match event {
            MeasureEvent::Note(note) => format!("note:{}", note.staff_position),
            MeasureEvent::ClefChange(change) => format!("clef:{:?}", change.clef),
            other => format!("{other:?}"),
        })
        .collect();
    // C4 in bass clef sits a line above the staff: position 10.
    assert_eq!(positions, ["note:5", "clef:Bass", "note:10"]);
    let MeasureEvent::Note(voice1_note) = &contents[0].additional_voices[0][1] else {
        panic!("voice 1 ends with a note")
    };
    assert_eq!(voice1_note.staff_position, 3);
}

#[test]
fn clef_after_barline_placement_opens_the_measure() {
    let page = page_of(
        ScoreBuilder::new()
            .clef(Clef::Bass)
            .note(c3(), Duration::WHOLE)
            .barline()
            .clef_change_after_barline(Clef::Alto)
            .note(Pitch::new(Note::C, 4), Duration::WHOLE)
            .end_barline(),
    );
    let measures = &page.systems[0].system.measures;
    assert_eq!(
        kinds(&measures[0].layout),
        ["clef:FClef", "note:3", "barline"]
    );
    assert_eq!(
        kinds(&measures[1].layout),
        ["clef:CClefChange", "note:4", "barline"]
    );
    let config = MeasureLayoutConfig::from_staff_space(SS);
    assert!((measures[1].layout.elements[0].x - config.clef_change_margin).abs() < 1e-6);
}

#[test]
fn meter_changes_switch_between_symbols_and_numbers() {
    // mn-c03-m008: cut C → 3/2 → cut C → C.
    let builder = ScoreBuilder::new()
        .cut_time()
        .note(c5(), Duration::WHOLE)
        .barline()
        .time_signature_change(3, 2)
        .note(c5(), Duration::WHOLE)
        .barline()
        .cut_time_change()
        .note(c5(), Duration::WHOLE)
        .barline()
        .common_time_change()
        .note(c5(), Duration::WHOLE)
        .end_barline();
    let page = page_of(builder.clone());
    let meters: Vec<_> = page.systems[0]
        .system
        .measures
        .iter()
        .flat_map(|measure| &measure.layout.elements)
        .filter_map(|element| match &element.element {
            MeasureElement::TimeSignature(kind) => Some(kind.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        meters,
        [
            TimeSignatureKind::CutCommon,
            TimeSignatureKind::Numeric {
                numerator: 3,
                denominator: 2
            },
            TimeSignatureKind::CutCommon,
            TimeSignatureKind::Common,
        ]
    );
    let svg = builder.clone().render_svg();
    assert_eq!(glyph_origins(&svg, Glyph::TimeSigCutCommon).len(), 2);
    assert_eq!(glyph_origins(&svg, Glyph::TimeSigCommon).len(), 1);
    let lengths: Vec<_> = metas(&builder)
        .iter()
        .map(|meta| meta.nominal_length)
        .collect();
    assert_eq!(
        lengths,
        [
            Some(MeasureLength::new(1, 1)),
            Some(MeasureLength::new(3, 2)),
            Some(MeasureLength::new(1, 1)),
            Some(MeasureLength::new(1, 1)),
        ]
    );
}

#[test]
fn hidden_meter_change_draws_nothing_but_sets_nominal_length() {
    // mn-c11-r027: unprinted 24/4 phrases between printed 3/4 bars.
    let builder = ScoreBuilder::new()
        .time_signature(3, 4)
        .note(c5(), Duration::new(DurationKind::Half, 1))
        .barline()
        .hidden_time_signature_change(24, 4)
        .note(c5(), Duration::WHOLE)
        .barline()
        .time_signature_change(3, 4)
        .note(c5(), Duration::new(DurationKind::Half, 1))
        .end_barline();
    let contents = builder.build_measure_contents().unwrap();
    assert!(
        matches!(contents[1].events[..], [MeasureEvent::Note(_)]),
        "hidden change leaves no event"
    );
    let meta = &contents[1].meta;
    assert_eq!(
        meta.meter,
        Some(TimeSignatureKind::Numeric {
            numerator: 24,
            denominator: 4
        })
    );
    assert!(!meta.meter_visible);
    assert_eq!(meta.nominal_length, Some(MeasureLength::new(6, 1)));
    assert_eq!(meta.actual_length, MeasureLength::new(1, 1));
    assert!(meta.is_incomplete());
    assert!(contents[2].meta.meter_visible);
    let svg = builder.render_svg();
    // 3/4 printed twice, 24/4 never.
    assert_eq!(glyph_origins(&svg, Glyph::TimeSig3).len(), 2);
    assert!(glyph_origins(&svg, Glyph::TimeSig2).is_empty());
}

#[test]
fn hidden_initial_meter_keeps_its_length() {
    let meta = &metas(
        &ScoreBuilder::new()
            .hidden_time_signature(1, 4)
            .note(c5(), Duration::QTR),
    )[0];
    assert_eq!(meta.nominal_length, Some(MeasureLength::new(1, 4)));
    assert!(!meta.meter_visible);
    let svg = ScoreBuilder::new()
        .hidden_time_signature(1, 4)
        .note(c5(), Duration::QTR)
        .render_svg();
    assert!(glyph_origins(&svg, Glyph::TimeSig1).is_empty());
}

#[test]
fn time_signature_change_after_the_first_event_is_an_error() {
    let result = ScoreBuilder::new()
        .time_signature(4, 4)
        .note(c5(), Duration::QTR)
        .barline()
        .note(c5(), Duration::QTR)
        .time_signature_change(3, 4)
        .end_barline()
        .try_render_svg();
    assert!(matches!(
        result,
        Err(crate::error::EngraverError::Structure(
            ScoreStructureError::MidMeasureTimeSignatureChange { measure: 1 }
        ))
    ));
}

// --- RM-MN-010: measure metadata --------------------------------------------

#[test]
fn pickup_is_bar_zero_and_does_not_advance_the_count() {
    let numbers: Vec<_> = metas(
        &ScoreBuilder::new()
            .time_signature(3, 4)
            .partial(Duration::QTR)
            .note(c5(), Duration::QTR)
            .barline()
            .note(c5(), Duration::new(DurationKind::Half, 1))
            .barline()
            .note(c5(), Duration::new(DurationKind::Half, 1))
            .end_barline(),
    )
    .iter()
    .map(|meta| (meta.number, meta.anacrusis, meta.nominal_length))
    .collect();
    assert_eq!(
        numbers,
        [
            (0, true, Some(MeasureLength::new(1, 4))),
            (1, false, Some(MeasureLength::new(3, 4))),
            (2, false, Some(MeasureLength::new(3, 4))),
        ]
    );
}

#[test]
fn start_number_and_mid_score_pickup() {
    // A mid-score pickup continues the incomplete bar before it (LilyPond
    // numbers both as one bar).
    let numbers: Vec<_> = metas(
        &ScoreBuilder::new()
            .time_signature(3, 4)
            .first_measure_number(5)
            .note(c5(), Duration::new(DurationKind::Half, 1))
            .barline()
            .note(c5(), Duration::HALF)
            .barline_style(BarlineStyle::Double)
            .partial(Duration::QTR)
            .note(c5(), Duration::QTR)
            .barline()
            .note(c5(), Duration::new(DurationKind::Half, 1))
            .end_barline(),
    )
    .iter()
    .map(|meta| (meta.number, meta.is_incomplete()))
    .collect();
    assert_eq!(numbers, [(5, false), (6, true), (6, false), (7, false)]);
}

#[test]
fn measure_length_override_holds_until_a_meter_change_or_reset() {
    // mn-c06-m004: a 9/8 bar under 4/4; mn-c12-r002: last bar 3/2.
    let lengths: Vec<_> = metas(
        &ScoreBuilder::new()
            .time_signature(4, 4)
            .note(c5(), Duration::WHOLE)
            .barline()
            .measure_length(9, 8)
            .note(c5(), Duration::WHOLE)
            .note(c5(), Duration::EIGHTH)
            .barline()
            .note(c5(), Duration::WHOLE)
            .barline()
            .reset_measure_length()
            .note(c5(), Duration::WHOLE)
            .barline()
            .measure_length(3, 2)
            .note(c5(), Duration::WHOLE)
            .barline()
            .time_signature_change(3, 4)
            .note(c5(), Duration::new(DurationKind::Half, 1))
            .end_barline(),
    )
    .iter()
    .map(|meta| (meta.nominal_length.unwrap(), meta.meter.clone().unwrap()))
    .collect();
    let four_four = TimeSignatureKind::Numeric {
        numerator: 4,
        denominator: 4,
    };
    assert_eq!(
        lengths,
        [
            (MeasureLength::new(1, 1), four_four.clone()),
            (MeasureLength::new(9, 8), four_four.clone()),
            (MeasureLength::new(9, 8), four_four.clone()),
            (MeasureLength::new(1, 1), four_four.clone()),
            (MeasureLength::new(3, 2), four_four),
            (
                MeasureLength::new(3, 4),
                TimeSignatureKind::Numeric {
                    numerator: 3,
                    denominator: 4
                }
            ),
        ]
    );
}

#[test]
fn cadenza_measures_have_no_nominal_length() {
    let metas = metas(
        &ScoreBuilder::new()
            .time_signature(4, 4)
            .cadenza_on()
            .note(c5(), Duration::QTR)
            .note(c5(), Duration::QTR)
            .note(c5(), Duration::QTR)
            .note(c5(), Duration::QTR)
            .note(c5(), Duration::QTR)
            .cadenza_off()
            .barline()
            .note(c5(), Duration::WHOLE)
            .end_barline(),
    );
    assert_eq!(metas[0].nominal_length, None);
    assert_eq!(metas[0].actual_length, MeasureLength::new(5, 4));
    assert!(!metas[0].is_incomplete());
    assert_eq!(metas[1].nominal_length, Some(MeasureLength::new(1, 1)));
}

#[test]
fn actual_length_is_the_longest_voice_with_tuplets_at_their_ratio() {
    let meta = &metas(
        &ScoreBuilder::new()
            .time_signature(4, 4)
            .tuplet_ratio(
                3,
                2,
                vec![
                    (c5(), Duration::EIGHTH),
                    (c5(), Duration::EIGHTH),
                    (c5(), Duration::EIGHTH),
                ],
            )
            .note(c5(), Duration::HALF)
            .voice(1)
            .note(c3(), Duration::new(DurationKind::Half, 1))
            .end_barline(),
    )[0];
    assert_eq!(meta.actual_length, MeasureLength::new(3, 4));
    assert!(meta.is_incomplete());
}

// --- Empty bars (G18) ---------------------------------------------------------

#[test]
fn spacer_bar_is_empty_and_at_least_the_minimum_width() {
    let builder = ScoreBuilder::new()
        .time_signature(3, 4)
        .note(c5(), Duration::new(DurationKind::Half, 1))
        .barline()
        .spacer(Duration::new(DurationKind::Half, 1))
        .end_barline();
    assert_eq!(metas(&builder)[1].actual_length, MeasureLength::new(3, 4));

    let config = MeasureLayoutConfig::from_staff_space(SS);
    let elements = [
        MeasureElement::Spacer(crate::layout::measure::SpacerEvent {
            duration_log2: Some(1),
            dots: 1,
            annotations: Default::default(),
        }),
        MeasureElement::Barline(BarlineStyle::Single),
    ];
    let natural = crate::layout::measure::layout_measure(&elements, &config);
    assert!(
        natural.closing_barline_x() >= config.empty_measure_min_width - 1e-9,
        "empty bar natural width {} < {}",
        natural.closing_barline_x(),
        config.empty_measure_min_width
    );

    // Nothing but staff lines, the clef, the meter, the note, and barlines.
    let svg = builder.render_svg();
    let empty = ScoreBuilder::new()
        .time_signature(3, 4)
        .note(c5(), Duration::new(DurationKind::Half, 1))
        .end_barline()
        .render_svg();
    assert_eq!(
        svg.matches("<path ").count(),
        empty.matches("<path ").count()
    );
}

// --- Bar numbers (G22) --------------------------------------------------------

#[test]
fn every_bar_numbers_skip_the_pickup_and_sit_on_barlines() {
    let builder = ScoreBuilder::new()
        .time_signature(3, 4)
        .measure_numbering(MeasureNumbering::EveryBar)
        .partial(Duration::QTR)
        .note(c5(), Duration::QTR)
        .barline()
        .note(c5(), Duration::new(DurationKind::Half, 1))
        .barline()
        .note(c5(), Duration::new(DurationKind::Half, 1))
        .end_barline();
    let page = page_of(builder.clone());
    let system = &page.systems[0];
    let measures = &system.system.measures;
    let barline_x = |index: usize| {
        system.x + measures[index].x_offset + measures[index].layout.closing_barline_x()
    };

    let numbers = texts(&builder.render_svg());
    let labels: Vec<_> = numbers.iter().map(|(_, _, text)| text.as_str()).collect();
    assert_eq!(labels, ["1", "2"], "no number on the pickup");
    assert!((numbers[0].0 - barline_x(0)).abs() < 1e-6);
    assert!((numbers[1].0 - barline_x(1)).abs() < 1e-6);
    let above = crate::layout::bar_number::BAR_NUMBER_ABOVE_STAFF_SS * SS;
    assert!((numbers[0].1 - (system.y - above)).abs() < 1e-6);
}

#[test]
fn system_start_number_sits_after_the_prefix() {
    let builder = ScoreBuilder::new()
        .key_signature(crate::layout::key_signature::KeySignature::Sharps(2))
        .time_signature(4, 4)
        .measures_per_system(1)
        .measure_numbering(MeasureNumbering::SystemStart)
        .note(c5(), Duration::WHOLE)
        .barline()
        .note(c5(), Duration::WHOLE)
        .end_barline();
    let page = page_of(builder.clone());
    let first_note_x = |system: usize| {
        let measure = &page.systems[system].system.measures[0];
        page.systems[system].x + measure.x_offset + element_x(&measure.layout, "note:5")
    };
    let numbers = texts(&builder.render_svg());
    assert_eq!(numbers.len(), 2);
    assert_eq!((numbers[0].2.as_str(), numbers[1].2.as_str()), ("1", "2"));
    assert!(
        (numbers[0].0 - first_note_x(0)).abs() < 1e-6,
        "not over the clef"
    );
    assert!((numbers[1].0 - first_note_x(1)).abs() < 1e-6);
}

// --- Clef metrics and page bounds -------------------------------------------

#[test]
fn clef_ink_never_reaches_the_first_key_signature_accidental() {
    let config = MeasureLayoutConfig::from_staff_space(SS);
    for clef in [Clef::Treble, Clef::Bass, Clef::Alto, Clef::Tenor] {
        for key in [
            crate::layout::key_signature::KeySignature::Sharps(7),
            crate::layout::key_signature::KeySignature::Flats(7),
        ] {
            let svg = ScoreBuilder::new()
                .clef(clef.clone_kind())
                .key_signature(key.clone())
                .note(Pitch::new(Note::C, 4), Duration::WHOLE)
                .render_svg();
            let layout = ClefLayout::from_clef_ref(&clef);
            let (clef_x, _) = glyph_origins(&svg, layout.glyph)[0];
            let accidental = match key {
                crate::layout::key_signature::KeySignature::Sharps(_) => Glyph::AccidentalSharp,
                _ => Glyph::AccidentalFlat,
            };
            let first_accidental_x = glyph_origins(&svg, accidental)
                .into_iter()
                .map(|(x, _)| x)
                .fold(f64::INFINITY, f64::min);
            let gap = first_accidental_x - (clef_x + layout.ink_box().x_right * SS);
            assert!(
                gap >= config.clef_padding - 1e-6,
                "{clef:?} {key:?}: clef ink to first accidental gap {gap} < padding"
            );
        }
    }
}

#[test]
fn view_box_contains_every_clef_glyph() {
    for clef in [
        Clef::Treble,
        Clef::Treble8va,
        Clef::Treble8ba,
        Clef::Bass,
        Clef::Alto,
        Clef::Tenor,
    ] {
        let svg = ScoreBuilder::new()
            .clef(clef.clone_kind())
            .note(Pitch::new(Note::C, 4), Duration::WHOLE)
            .render_svg();
        let layout = ClefLayout::from_clef_ref(&clef);
        assert_eq!(layout.size, ClefSize::Full);
        let (x, y) = glyph_origins(&svg, layout.glyph)[0];
        let ink = layout.ink_box();
        let (vb_x, vb_y, vb_w, vb_h) = view_box(&svg);
        assert!(vb_y <= y + ink.y_top * SS, "{clef:?} top clipped");
        assert!(
            vb_y + vb_h >= y + ink.y_bottom * SS,
            "{clef:?} bottom clipped"
        );
        assert!(vb_x <= x + ink.x_left * SS && vb_x + vb_w >= x + ink.x_right * SS);
    }
}

#[test]
fn view_box_contains_high_key_signature_accidentals() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Tenor)
        .key_signature(crate::layout::key_signature::KeySignature::Flats(7))
        .note(Pitch::new(Note::C, 4), Duration::WHOLE)
        .render_svg();
    let top = glyph_origins(&svg, Glyph::AccidentalFlat)
        .into_iter()
        .map(|(_, y)| y)
        .fold(f64::INFINITY, f64::min);
    let flat_top = crate::layout::glyph_metrics::glyph_box(Glyph::AccidentalFlat)
        .unwrap()
        .y_top;
    assert!(view_box(&svg).1 <= top + flat_top * SS);
}

/// `music::Clef` has no `Clone`; rebuild the same variant.
trait CloneKind {
    fn clone_kind(&self) -> Clef;
}

impl CloneKind for Clef {
    fn clone_kind(&self) -> Clef {
        ClefKind::from_clef(self).to_clef()
    }
}
