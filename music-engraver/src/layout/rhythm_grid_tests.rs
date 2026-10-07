use super::*;
use crate::layout::barline::BarlineStyle;
use crate::layout::group::TupletSpec;
use crate::layout::measure::{MeasureElement, MeasureLayoutConfig};
use crate::layout::system::{layout_staves_followed_by, layout_system, SystemLayout};
use crate::score::multi_staff::MultiStaffScore;
use crate::score::ScoreBuilder;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}
fn cfg() -> MeasureLayoutConfig {
    MeasureLayoutConfig::from_staff_space(250.0)
}

fn layout(builder: &ScoreBuilder, target: Option<f64>) -> SystemLayout {
    let contents = builder.build_measure_contents().unwrap();
    layout_system(&builder.build_prefix(), &contents, &cfg(), target)
}

fn voice_notes(layout: &MeasureLayout) -> Vec<f64> {
    layout
        .elements
        .iter()
        .filter_map(|element| match element.element {
            MeasureElement::Note(_) | MeasureElement::Chord(_) => Some(element.x),
            _ => None,
        })
        .collect()
}

fn polyphony(reverse: bool) -> ScoreBuilder {
    let mut score = ScoreBuilder::new().clef(Clef::Treble).time_signature(4, 4);
    if reverse {
        score = score
            .voice(1)
            .note(p(Note::G, 4), Duration::HALF)
            .note(p(Note::A, 4), Duration::QTR)
            .note(p(Note::B, 4), Duration::QTR);
    }
    score = score
        .voice(0)
        .note(p(Note::C, 5), Duration::QTR)
        .note(p(Note::D, 5), Duration::QTR)
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::F, 5), Duration::QTR);
    if !reverse {
        score = score
            .voice(1)
            .note(p(Note::G, 4), Duration::HALF)
            .note(p(Note::A, 4), Duration::QTR)
            .note(p(Note::B, 4), Duration::QTR);
    }
    score.end_barline()
}

#[test]
fn simultaneous_polyphonic_beats_are_identical_in_both_builder_orders() {
    for reverse in [false, true] {
        let system = layout(&polyphony(reverse), None);
        let measure = &system.measures[0];
        assert!(
            (measure.layout.total_spring - 4.0 * cfg().spring_constant).abs() < 1e-9,
            "a half spanning beat 2 cannot add its spring to four quarter columns"
        );
        assert_eq!(
            measure.layout.total_width,
            measure.layout.total_rod + measure.layout.total_spring
        );
        let upper = voice_notes(&measure.layout);
        let lower = voice_notes(&measure.additional_voice_layouts[0]);
        assert_eq!(upper[0], lower[0]);
        assert_eq!(upper[2], lower[1], "beat 3 in builder order {reverse}");
        assert_eq!(upper[3], lower[2], "beat 4 in builder order {reverse}");
        assert!(upper.windows(2).all(|p| p[1] > p[0]));
    }
}

#[test]
fn accidentals_dots_chords_and_mid_measure_structure_reserve_rods_once() {
    let score = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .voice(0)
        .note(p(Note::C, 5), Duration::QTR)
        .inline_barline(BarlineStyle::Dashed)
        .clef_change(Clef::Bass)
        .chord(
            vec![p(Note::Fis, 3), p(Note::A, 3)],
            Duration::new(DurationKind::Qtr, 1),
        )
        .note(p(Note::B, 3), Duration::EIGHTH)
        .note(p(Note::C, 4), Duration::QTR)
        .voice(1)
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::A, 4), Duration::QTR)
        .rest(Duration::EIGHTH)
        .note(p(Note::B, 4), Duration::EIGHTH)
        .note(p(Note::C, 5), Duration::QTR)
        .end_barline();
    let system = layout(&score, None);
    let measure = &system.measures[0];
    let upper = voice_notes(&measure.layout);
    let lower = voice_notes(&measure.additional_voice_layouts[0]);
    assert_eq!(upper[0], lower[0]);
    assert_eq!(upper[1], lower[1]);
    assert_eq!(upper[3], lower[3]);
    let chord = measure
        .layout
        .elements
        .iter()
        .find(|el| matches!(el.element, MeasureElement::Chord(_)))
        .unwrap();
    let before = measure
        .layout
        .elements
        .iter()
        .find(|el| matches!(el.element, MeasureElement::Barline(BarlineStyle::Dashed)))
        .unwrap();
    let font = &*crate::font::BUNDLED_BRAVURA;
    let sharp_left = chord.x
        - f64::from(font.glyph_advance(smufl::Glyph::AccidentalSharp).unwrap())
        - crate::layout::accidental::ACCIDENTAL_NOTEHEAD_PADDING_SS * cfg().staff_space
        + font
            .glyph_bbox_design_units(smufl::Glyph::AccidentalSharp)
            .unwrap()
            .x_left;
    assert!(
        sharp_left >= before.x + before.width,
        "the accidental and clef must not overprint the inline barline"
    );
    assert!(measure.layout.closing_barline_x() > upper[3]);
    let justified = layout(&score, Some(system.total_width * 1.4));
    let primary = voice_notes(&justified.measures[0].layout);
    let second = voice_notes(&justified.measures[0].additional_voice_layouts[0]);
    assert_eq!(primary[1], second[1]);
    assert_eq!(primary[3], second[3]);
    let natural_first = measure
        .layout
        .elements
        .iter()
        .find(|el| matches!(el.element, MeasureElement::Note(_)))
        .unwrap();
    let justified_first = justified.measures[0]
        .layout
        .elements
        .iter()
        .find(|el| matches!(el.element, MeasureElement::Note(_)))
        .unwrap();
    assert!(
        (chord.x
            - upper[0]
            - natural_first.width
            - (primary[1] - primary[0] - justified_first.width))
            .abs()
            < 1e-9,
        "the structural and accidental rods cannot be scaled by justification"
    );
}

#[test]
fn nested_and_simple_tuplets_share_exact_rational_columns() {
    let score = ScoreBuilder::new()
        .time_signature(4, 4)
        .voice(0)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::C, 5), Duration::QTR)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::D, 5), Duration::EIGHTH)
        .note(p(Note::E, 5), Duration::EIGHTH)
        .note(p(Note::F, 5), Duration::EIGHTH)
        .end_tuplet()
        .note(p(Note::G, 5), Duration::QTR)
        .end_tuplet()
        .note(p(Note::A, 5), Duration::HALF)
        .voice(1)
        .note(p(Note::C, 4), Duration::HALF)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::D, 4), Duration::QTR)
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::F, 4), Duration::QTR)
        .end_tuplet()
        .note(p(Note::G, 4), Duration::HALF)
        .end_barline();
    let system = layout(&score, None);
    let top = voice_notes(&system.measures[0].layout);
    let bottom = voice_notes(&system.measures[0].additional_voice_layouts[0]);
    assert_eq!(top[0], bottom[0]);
    assert_eq!(top[5], bottom[1]);
    assert!(top[1] < top[2] && top[2] < top[3]);
    assert_eq!(
        system.measures[0].layout.closing_barline_x(),
        system.measures[0].additional_voice_layouts[0].closing_barline_x()
    );
}

fn asymmetric_staves() -> (ScoreBuilder, ScoreBuilder) {
    let mut upper = ScoreBuilder::new().clef(Clef::Treble).time_signature(4, 4);
    for _ in 0..16 {
        upper = upper.note(p(Note::C, 5), Duration::SIXTEENTH);
    }
    upper = upper
        .barline()
        .rest(Duration::QTR)
        .note(p(Note::D, 5), Duration::QTR)
        .note(p(Note::E, 5), Duration::HALF)
        .end_barline();
    let mut lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .rest(Duration::QTR)
        .note(p(Note::D, 3), Duration::QTR)
        .note(p(Note::E, 3), Duration::HALF)
        .barline();
    for _ in 0..16 {
        lower = lower.note(p(Note::C, 3), Duration::SIXTEENTH);
    }
    (upper, lower.end_barline())
}

#[test]
fn asymmetric_staves_share_every_measure_boundary_and_beats() {
    let (upper, lower) = asymmetric_staves();
    let (u, l) = (
        upper.build_measure_contents().unwrap(),
        lower.build_measure_contents().unwrap(),
    );
    let prefixes = [upper.build_prefix(), lower.build_prefix()];
    let staves = [
        (&prefixes[0], u.as_slice(), None),
        (&prefixes[1], l.as_slice(), None),
    ];
    for width in [None, Some(15_000.0)] {
        let systems = layout_staves_followed_by(&staves, &cfg(), width);
        let (top, bottom) = (&systems[0], &systems[1]);
        assert_eq!(top.total_width, bottom.total_width);
        for index in 0..2 {
            assert_eq!(
                top.measures[index].x_offset,
                bottom.measures[index].x_offset
            );
            assert_eq!(
                top.measures[index].shared_closing_barline_x,
                bottom.measures[index].shared_closing_barline_x
            );
            assert_eq!(
                top.measures[index].layout.closing_barline_x(),
                bottom.measures[index].layout.closing_barline_x()
            );
        }
        assert_eq!(
            voice_notes(&top.measures[0].layout)[4],
            voice_notes(&bottom.measures[0].layout)[0]
        );
        assert_eq!(
            voice_notes(&top.measures[1].layout)[0],
            voice_notes(&bottom.measures[1].layout)[4]
        );
        assert!(
            top.measures[1].x_offset
                >= top.measures[0].x_offset + top.measures[0].layout.total_width
        );
    }
}

#[test]
fn auto_and_optimal_breaks_consult_both_staves() {
    let (upper, lower) = asymmetric_staves();
    for optimal in [false, true] {
        let score = MultiStaffScore::grand_staff(upper.clone(), lower.clone());
        let score = if optimal {
            score.optimal_line_breaks()
        } else {
            score.auto_line_breaks()
        };
        let (_, chunks) = score.staves_into_systems(&cfg(), 12_000.0, 4).unwrap();
        assert_eq!(chunks, vec![(0, 1), (1, 2)]);
    }
}

#[test]
fn forced_inline_break_preserves_shared_continuation_tick() {
    let upper = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p(Note::C, 5), Duration::QTR)
        .note(p(Note::D, 5), Duration::QTR)
        .inline_barline(BarlineStyle::Dashed)
        .system_break()
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::F, 5), Duration::QTR)
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .rest(Duration::HALF)
        .note(p(Note::C, 3), Duration::QTR)
        .note(p(Note::D, 3), Duration::QTR)
        .end_barline();
    let score = MultiStaffScore::grand_staff(upper, lower).explicit_line_breaks();
    let (staves, chunks) = score.staves_into_systems(&cfg(), 10_000.0, 4).unwrap();
    assert_eq!(chunks, vec![(0, 1), (1, 2)]);
    assert_eq!(staves[0].0[1].meta.visual_start, MeasureLength::new(1, 2));
    assert_eq!(staves[1].0[1].meta.visual_start, MeasureLength::new(1, 2));
    let prefixes = [staves[0].1.clone(), staves[1].1.clone()];
    let views = [
        (&prefixes[0], &staves[0].0[1..], None),
        (&prefixes[1], &staves[1].0[1..], None),
    ];
    let systems = layout_staves_followed_by(&views, &cfg(), None);
    assert_eq!(
        voice_notes(&systems[0].measures[0].layout)[0],
        voice_notes(&systems[1].measures[0].layout)[0]
    );
    assert_eq!(
        systems[0].measures[0].layout.closing_barline_x(),
        systems[1].measures[0].layout.closing_barline_x()
    );
}

/// Modus Novus mn-c11-r029, bar 7: the tiny secondary G-sixteenth,
/// G-sixteenth, G-eighth sits beneath the primary G-eighth pair.
/// Their attacks at 1/4 and 3/8 coincide, while the intervening
/// sixteenth is an independent column.
#[test]
fn modus_novus_c11_r029_secondary_voice_matches_primary_attacks() {
    let score = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .voice(0)
        .note(p(Note::F, 5), Duration::new(DurationKind::Eighth, 1))
        .note(p(Note::G, 4), Duration::SIXTEENTH)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .begin_tuplet(TupletSpec::new(3, 2))
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::EIGHTH)
        .note(p(Note::Cis, 5), Duration::EIGHTH)
        .end_tuplet()
        .voice(1)
        .spacer(Duration::QTR)
        .note(p(Note::G, 4), Duration::SIXTEENTH)
        .note(p(Note::G, 4), Duration::SIXTEENTH)
        .note(p(Note::G, 4), Duration::EIGHTH)
        .end_barline();
    let system = layout(&score, Some(11_000.0));
    let primary = voice_notes(&system.measures[0].layout);
    let secondary = voice_notes(&system.measures[0].additional_voice_layouts[0]);
    assert_eq!(secondary[0], primary[2], "the first G-eighth starts at 1/4");
    assert_eq!(
        secondary[2], primary[3],
        "the second G-eighth starts at 3/8"
    );
    assert!(secondary[1] > secondary[0] && secondary[1] < secondary[2]);
    assert!(
        primary[4] > primary[3],
        "the tuplet begins after the shared eighths"
    );
}

#[test]
fn crossing_break_note_retains_its_later_original_onset() {
    let upper = ScoreBuilder::new()
        .time_signature(4, 4)
        .note(p(Note::C, 5), Duration::QTR)
        .system_break()
        .note(p(Note::D, 5), Duration::QTR)
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::F, 5), Duration::QTR)
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .note(p(Note::C, 3), Duration::HALF)
        .note(p(Note::D, 3), Duration::QTR)
        .note(p(Note::E, 3), Duration::QTR)
        .end_barline();
    let score = MultiStaffScore::grand_staff(upper, lower).explicit_line_breaks();
    let (staves, chunks) = score.staves_into_systems(&cfg(), 12_000.0, 4).unwrap();
    assert_eq!(chunks, vec![(0, 1), (1, 2)]);
    assert_eq!(staves[0].0[1].meta.visual_start, MeasureLength::new(1, 4));
    assert_eq!(
        staves[1].0[1].meta.visual_voice_onsets[0],
        MeasureLength::new(1, 2)
    );
    let prefixes = [staves[0].1.clone(), staves[1].1.clone()];
    let inputs = [
        (&prefixes[0], &staves[0].0[1..], None),
        (&prefixes[1], &staves[1].0[1..], None),
    ];
    let systems = layout_staves_followed_by(&inputs, &cfg(), None);
    let upper_notes = voice_notes(&systems[0].measures[0].layout);
    let lower_notes = voice_notes(&systems[1].measures[0].layout);
    assert!(upper_notes[0] < lower_notes[0]);
    assert_eq!(upper_notes[1], lower_notes[0]);
    assert_eq!(upper_notes[2], lower_notes[1]);
}

#[test]
fn unentered_and_spacer_staves_inherit_sounding_boundaries() {
    let upper = ScoreBuilder::new()
        .time_signature(4, 4)
        .note(p(Note::C, 5), Duration::QTR)
        .note(p(Note::D, 5), Duration::QTR)
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::F, 5), Duration::QTR)
        .barline()
        .note(p(Note::C, 5), Duration::WHOLE)
        .end_barline();
    let blank = ScoreBuilder::new().clef(Clef::Bass);
    let (staves, chunks) = MultiStaffScore::grand_staff(upper.clone(), blank)
        .staves_into_systems(&cfg(), 11_000.0, 2)
        .unwrap();
    assert_eq!(chunks, vec![(0, 2)]);
    assert_eq!(staves[0].0.len(), staves[1].0.len());
    let prefixes = [staves[0].1.clone(), staves[1].1.clone()];
    let views = [
        (&prefixes[0], staves[0].0.as_slice(), None),
        (&prefixes[1], staves[1].0.as_slice(), None),
    ];
    let systems = layout_staves_followed_by(&views, &cfg(), None);
    for i in 0..2 {
        assert_eq!(
            systems[0].measures[i].layout.closing_barline_x(),
            systems[1].measures[i].layout.closing_barline_x()
        );
    }
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .time_signature(4, 4)
        .spacer(Duration::HALF)
        .spacer(Duration::HALF)
        .barline()
        .spacer(Duration::WHOLE)
        .end_barline();
    let contents = [
        upper.build_measure_contents().unwrap(),
        lower.build_measure_contents().unwrap(),
    ];
    let prefixes = [upper.build_prefix(), lower.build_prefix()];
    let views = [
        (&prefixes[0], contents[0].as_slice(), None),
        (&prefixes[1], contents[1].as_slice(), None),
    ];
    let systems = layout_staves_followed_by(&views, &cfg(), Some(14_000.0));
    let spacer = systems[1].measures[0]
        .layout
        .elements
        .iter()
        .find(|el| matches!(el.element, MeasureElement::Spacer(_)))
        .unwrap();
    let note = systems[0].measures[0]
        .layout
        .elements
        .iter()
        .find(|el| matches!(el.element, MeasureElement::Note(_)))
        .unwrap();
    assert_eq!(spacer.x, note.x);
    assert_eq!(
        systems[0].measures[1].x_offset,
        systems[1].measures[1].x_offset
    );
}

#[test]
fn simultaneous_structural_and_accidental_left_rods_use_max_not_sum() {
    use crate::layout::key_signature::KeySignature;
    let upper = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .note(p(Note::C, 5), Duration::QTR)
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .note(p(Note::Fis, 3), Duration::QTR)
        .end_barline();
    let upper_alone = voice_notes(&layout(&upper, None).measures[0].layout)[0];
    let lower_alone = voice_notes(&layout(&lower, None).measures[0].layout)[0];
    let contents = [
        upper.build_measure_contents().unwrap(),
        lower.build_measure_contents().unwrap(),
    ];
    let prefixes = [upper.build_prefix(), lower.build_prefix()];
    let views = [
        (&prefixes[0], contents[0].as_slice(), None),
        (&prefixes[1], contents[1].as_slice(), None),
    ];
    let systems = layout_staves_followed_by(&views, &cfg(), None);
    let upper_x = voice_notes(&systems[0].measures[0].layout)[0];
    let lower_x = voice_notes(&systems[1].measures[0].layout)[0];
    assert_eq!(upper_x, lower_x);
    assert_eq!(upper_x, upper_alone.max(lower_alone));
}

/// mn-c12-i001 begins A-flat5, E4, F-sharp3, A4 on alternating
/// treble/bass staves. Routing the *same voice* across staves and its
/// dashed glissandi is a separate feature; this exercises the onset
/// columns beneath that routing with whole-note spacer counterparts.
#[test]
fn modus_novus_c12_i001_alternating_staff_spacers_share_column() {
    let upper = ScoreBuilder::new()
        .clef(Clef::Treble)
        .note(p(Note::Aes, 5), Duration::WHOLE)
        .note(p(Note::E, 4), Duration::WHOLE)
        .spacer(Duration::WHOLE)
        .note(p(Note::A, 4), Duration::WHOLE)
        .end_barline();
    let lower = ScoreBuilder::new()
        .clef(Clef::Bass)
        .spacer(Duration::WHOLE)
        .spacer(Duration::WHOLE)
        .note(p(Note::Fis, 3), Duration::WHOLE)
        .spacer(Duration::WHOLE)
        .end_barline();
    let contents = [
        upper.build_measure_contents().unwrap(),
        lower.build_measure_contents().unwrap(),
    ];
    let prefixes = [upper.build_prefix(), lower.build_prefix()];
    let views = [
        (&prefixes[0], contents[0].as_slice(), None),
        (&prefixes[1], contents[1].as_slice(), None),
    ];
    let systems = layout_staves_followed_by(&views, &cfg(), None);
    let upper = &systems[0].measures[0].layout;
    let lower = &systems[1].measures[0].layout;
    let upper_ticks: Vec<_> = upper
        .elements
        .iter()
        .filter(|el| {
            matches!(
                el.element,
                MeasureElement::Note(_) | MeasureElement::Spacer(_)
            )
        })
        .map(|el| el.x)
        .collect();
    let lower_ticks: Vec<_> = lower
        .elements
        .iter()
        .filter(|el| {
            matches!(
                el.element,
                MeasureElement::Note(_) | MeasureElement::Spacer(_)
            )
        })
        .map(|el| el.x)
        .collect();
    assert_eq!(upper_ticks, lower_ticks);
    assert!(upper_ticks.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn breve_and_dotted_events_share_exact_closing_tick() {
    let breve = Duration::new(DurationKind::Breve, 0);
    let score = ScoreBuilder::new()
        .voice(0)
        .note(p(Note::C, 5), breve)
        .voice(1)
        .note(p(Note::G, 4), Duration::new(DurationKind::Whole, 1))
        .note(p(Note::A, 4), Duration::HALF)
        .end_barline();
    let system = layout(&score, None);
    let measure = &system.measures[0];
    assert_eq!(
        voice_notes(&measure.layout)[0],
        voice_notes(&measure.additional_voice_layouts[0])[0]
    );
    assert_eq!(
        measure.layout.closing_barline_x(),
        measure.additional_voice_layouts[0].closing_barline_x()
    );
    assert!(
        voice_notes(&measure.additional_voice_layouts[0])[1] < measure.layout.closing_barline_x()
    );
}
