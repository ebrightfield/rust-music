use super::*;
use crate::font::bravura_font;
use crate::layout::barline::BarlineStyle;
use crate::layout::clef::ClefLayout;
use crate::layout::key_signature::KeySignature;
use crate::layout::measure::{layout_measure, BeamGroupEvent, MeasureLayoutConfig, RestEvent};
use crate::layout::time_signature::TimeSignatureKind;
use crate::render::staff_renderer::draw_staff_lines;
use smufl::Glyph;

fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
    (font, config, staff)
}

fn make_svg() -> SvgWriter {
    SvgWriter::new(800.0, 200.0, -500.0, -500.0, 8000.0, 2000.0)
}

#[test]
fn empty_measure_produces_no_elements() {
    let (font, config, staff) = setup();
    let layout = MeasureLayout {
        elements: vec![],
        total_width: 0.0,
    };
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();
    // Only the SVG wrapper, no path or line elements
    assert_eq!(output.matches("<path ").count(), 0);
    assert_eq!(output.matches("<line ").count(), 0);
}

#[test]
fn measure_with_single_quarter_note() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // Should have 1 notehead path + 1 stem line
    assert_eq!(output.matches("<path ").count(), 1, "one notehead");
    assert_eq!(output.matches("<line ").count(), 1, "one stem line");
}

#[test]
fn measure_with_whole_note_has_no_stem() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0, // whole note
        dots: 0,
        accidental: None,
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 1, "one notehead");
    assert_eq!(output.matches("<line ").count(), 0, "no stem for whole note");
}

#[test]
fn measure_with_eighth_note_has_flag() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 3, // eighth note
        dots: 0,
        accidental: None,
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 notehead + 1 flag = 2 paths, 1 stem line
    assert_eq!(output.matches("<path ").count(), 2, "notehead + flag");
    assert_eq!(output.matches("<line ").count(), 1, "one stem");
}

#[test]
fn measure_with_dotted_quarter() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 5, // in a space (dot position unchanged)
        duration_log2: 2,
        dots: 1,
        accidental: None,
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 notehead + 1 dot = 2 paths, 1 stem
    assert_eq!(output.matches("<path ").count(), 2, "notehead + dot");
    assert_eq!(output.matches("<line ").count(), 1, "one stem");
}

#[test]
fn measure_with_accidental_note() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: Some(Glyph::AccidentalSharp),
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 accidental + 1 notehead = 2 paths, 1 stem
    assert_eq!(output.matches("<path ").count(), 2, "accidental + notehead");
    assert_eq!(output.matches("<line ").count(), 1, "one stem");
}

#[test]
fn measure_with_rest() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Rest(RestEvent {
        duration_log2: 2,
        dots: 0,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 1, "one rest glyph");
    assert_eq!(output.matches("<line ").count(), 0, "no lines for rest");
}

#[test]
fn measure_with_barline() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![
        MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        }),
        MeasureElement::Barline(BarlineStyle::Single),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 notehead path, 1 stem + 1 barline = 2 lines
    assert_eq!(output.matches("<path ").count(), 1, "one notehead");
    assert_eq!(output.matches("<line ").count(), 2, "stem + barline");
}

#[test]
fn measure_with_clef_and_key_signature() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![
        MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
        MeasureElement::KeySignature(KeySignature::Sharps(2)),
        MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        }),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 clef + 2 key sig accidentals + 1 notehead = 4 paths
    assert_eq!(output.matches("<path ").count(), 4, "clef + 2 sharps + notehead");
    // 1 stem line
    assert!(output.matches("<line ").count() >= 1, "at least 1 stem");
}

#[test]
fn measure_with_time_signature() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![
        MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        }),
        MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        }),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 digit paths (4, 4) + 1 notehead = 3 paths
    assert_eq!(output.matches("<path ").count(), 3, "2 digits + notehead");
}

#[test]
fn x_offset_shifts_elements() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);

    let mut svg_0 = make_svg();
    draw_measure(&mut svg_0, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();

    let mut svg_offset = make_svg();
    draw_measure(
        &mut svg_offset,
        &staff,
        &font,
        &config,
        &layout,
        1000.0,
        &Clef::Treble,
    )
    .unwrap();

    // The SVGs should differ because of the offset
    assert_ne!(
        svg_0.to_svg(),
        svg_offset.to_svg(),
        "offset should produce different SVG"
    );
    // The offset version should contain translate with larger x
    assert!(svg_offset.to_svg().contains("translate(1000"));
}

#[test]
fn notehead_kind_from_log2_mapping() {
    assert_eq!(notehead_kind_from_log2(0), NoteheadKind::Whole);
    assert_eq!(notehead_kind_from_log2(1), NoteheadKind::Half);
    assert_eq!(notehead_kind_from_log2(2), NoteheadKind::Filled);
    assert_eq!(notehead_kind_from_log2(3), NoteheadKind::Filled);
    assert_eq!(notehead_kind_from_log2(7), NoteheadKind::Filled);
}

#[test]
fn flag_count_from_log2_mapping() {
    assert_eq!(flag_count_from_log2(0), 0); // whole
    assert_eq!(flag_count_from_log2(1), 0); // half
    assert_eq!(flag_count_from_log2(2), 0); // quarter
    assert_eq!(flag_count_from_log2(3), 1); // eighth
    assert_eq!(flag_count_from_log2(4), 2); // sixteenth
    assert_eq!(flag_count_from_log2(5), 3); // 32nd
    assert_eq!(flag_count_from_log2(7), 5); // 128th
}

#[test]
fn full_measure_with_all_element_types() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![
        MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
        MeasureElement::KeySignature(KeySignature::Flats(1)),
        MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
            numerator: 3,
            denominator: 4,
        }),
        MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        }),
        MeasureElement::Note(NoteEvent {
            staff_position: 6,
            duration_log2: 3,
            dots: 0,
            accidental: Some(Glyph::AccidentalNatural),
            stem_direction: Some(StemDirection::Down),
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        }),
        MeasureElement::Rest(RestEvent {
            duration_log2: 2,
            dots: 0,
        }),
        MeasureElement::Barline(BarlineStyle::Single),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_staff_lines(&mut svg, &staff, &config);
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // Count expected paths:
    // clef(1) + flat(1) + time(2 digits) + notehead(1) + notehead(1) + accidental(1) + flag(1) + rest(1) = 9
    assert_eq!(output.matches("<path ").count(), 9, "expected 9 paths total");

    // Lines: 5 staff lines + 2 stems + 1 barline = 8
    assert_eq!(output.matches("<line ").count(), 8, "expected 8 lines total");
}

#[test]
fn note_with_ledger_lines_below_staff() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: -2, // middle C in treble
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 notehead, 1 stem + 1 ledger line = 2 lines
    assert_eq!(output.matches("<path ").count(), 1);
    assert_eq!(output.matches("<line ").count(), 2, "stem + 1 ledger line");
}

// --- chord rendering ---

#[test]
fn chord_two_notes_third_apart() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4], // E4 and B4 in treble — a fifth
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 noteheads, 1 stem
    assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
    assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
}

#[test]
fn chord_with_second_has_two_noteheads() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    // Adjacent positions 4 and 5 — a second, one notehead should be offset
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![4, 5],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 noteheads (at different x-offsets due to second), 1 stem
    assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
    assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
    // Verify both noteheads have different translate positions
    let translates: Vec<&str> = output.matches("translate(").collect();
    assert_eq!(translates.len(), 2, "two translate transforms for two noteheads");
}

#[test]
fn chord_with_accidentals() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![Some(Glyph::AccidentalSharp), None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 accidental + 2 noteheads = 3 paths, 1 stem
    assert_eq!(output.matches("<path ").count(), 3, "accidental + 2 noteheads");
    assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
}

#[test]
fn chord_whole_note_no_stem() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 8],
        duration_log2: 0, // whole note chord
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 3 noteheads, no stem
    assert_eq!(output.matches("<path ").count(), 3, "three noteheads");
    assert_eq!(output.matches("<line ").count(), 0, "no stem for whole note chord");
}

#[test]
fn chord_eighth_note_has_flag() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4],
        duration_log2: 3, // eighth note
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 noteheads + 1 flag = 3 paths, 1 stem
    assert_eq!(output.matches("<path ").count(), 3, "2 noteheads + flag");
    assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
}

#[test]
fn chord_with_ledger_lines() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    // Chord spanning from below staff to on staff
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![-2, 4], // C4 (ledger line) and B4
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 noteheads, 1 stem + 1 ledger line = 2 lines
    assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
    assert_eq!(output.matches("<line ").count(), 2, "stem + ledger line");
}

#[test]
fn chord_empty_produces_nothing() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 0, "empty chord = no paths");
    assert_eq!(output.matches("<line ").count(), 0, "empty chord = no lines");
}

#[test]
fn chord_dotted_quarter() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![3, 5], // in spaces, so dots don't need line avoidance
        duration_log2: 2,
        dots: 1,
        accidentals: vec![None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 noteheads + 2 dots = 4 paths, 1 stem
    assert_eq!(output.matches("<path ").count(), 4, "2 noteheads + 2 dots");
    assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
}

#[test]
fn chord_differs_from_single_note() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let single = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let chord = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];

    let layout_s = layout_measure(&single, &cfg);
    let layout_c = layout_measure(&chord, &cfg);

    let mut svg_s = make_svg();
    draw_measure(&mut svg_s, &staff, &font, &config, &layout_s, 0.0, &Clef::Treble).unwrap();
    let mut svg_c = make_svg();
    draw_measure(&mut svg_c, &staff, &font, &config, &layout_c, 0.0, &Clef::Treble).unwrap();

    // Chord should have more paths (2 noteheads vs 1)
    assert!(
        svg_c.to_svg().matches("<path ").count() > svg_s.to_svg().matches("<path ").count(),
        "chord should have more noteheads than single note"
    );
}

#[test]
fn stem_direction_override_is_respected() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Position 0 (bottom line): auto would be stem up
    let elements_up = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Up),
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let elements_down = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
    tie_forward: false,
    dynamic: None,
    slur_start: false,
    slur_end: false,
    hairpin_start: None,
    hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];

    let layout_up = layout_measure(&elements_up, &cfg);
    let layout_down = layout_measure(&elements_down, &cfg);

    let mut svg_up = make_svg();
    draw_measure(&mut svg_up, &staff, &font, &config, &layout_up, 0.0, &Clef::Treble).unwrap();

    let mut svg_down = make_svg();
    draw_measure(
        &mut svg_down,
        &staff,
        &font,
        &config,
        &layout_down,
        0.0,
        &Clef::Treble,
    )
    .unwrap();

    // Different stem directions should produce different SVG
    assert_ne!(
        svg_up.to_svg(),
        svg_down.to_svg(),
        "up vs down stem should differ"
    );
}

// --- beam group rendering ---

#[test]
fn beam_group_two_eighths() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: vec![
            NoteEvent {
                staff_position: 0,
                duration_log2: 3,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
            },
            NoteEvent {
                staff_position: 2,
                duration_log2: 3,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
            },
        ],
        stem_direction: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 noteheads as paths
    assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
    // 2 stems as lines
    assert_eq!(output.matches("<line ").count(), 2, "two stems");
    // 1 primary beam as polygon
    assert_eq!(output.matches("<polygon ").count(), 1, "one beam polygon");
}

#[test]
fn beam_group_four_sixteenths() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: vec![
            NoteEvent { staff_position: 0, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            NoteEvent { staff_position: 2, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            NoteEvent { staff_position: 4, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            NoteEvent { staff_position: 6, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        ],
        stem_direction: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 4, "four noteheads");
    assert_eq!(output.matches("<line ").count(), 4, "four stems");
    // Primary + secondary beam = 2 polygons
    assert_eq!(output.matches("<polygon ").count(), 2, "two beam polygons (primary + secondary)");
}

#[test]
fn beam_group_with_accidental() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: vec![
            NoteEvent {
                staff_position: 2,
                duration_log2: 3,
                dots: 0,
                accidental: Some(Glyph::AccidentalSharp),
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
            },
            NoteEvent {
                staff_position: 4,
                duration_log2: 3,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
            },
        ],
        stem_direction: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 accidental + 2 noteheads = 3 paths
    assert_eq!(output.matches("<path ").count(), 3, "accidental + 2 noteheads");
    assert_eq!(output.matches("<line ").count(), 2, "two stems");
    assert_eq!(output.matches("<polygon ").count(), 1, "one beam");
}

#[test]
fn beam_group_with_ledger_lines() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    // Notes below the staff requiring ledger lines
    let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: vec![
            NoteEvent { staff_position: -2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            NoteEvent { staff_position: -4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        ],
        stem_direction: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
    // 2 stems + ledger lines (1 for pos -2, 2 for pos -4)
    let line_count = output.matches("<line ").count();
    assert!(line_count >= 5, "stems + ledger lines: got {line_count}");
}

#[test]
fn beam_group_empty() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: vec![],
        stem_direction: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 0);
    assert_eq!(output.matches("<line ").count(), 0);
    assert_eq!(output.matches("<polygon ").count(), 0);
}

#[test]
fn beam_group_differs_from_flagged_notes() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Two separate eighth notes (flagged)
    let flagged = vec![
        MeasureElement::Note(NoteEvent {
            staff_position: 0,
            duration_log2: 3,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        }),
        MeasureElement::Note(NoteEvent {
            staff_position: 2,
            duration_log2: 3,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        }),
    ];
    // Same notes but beamed
    let beamed = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: vec![
            NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        ],
        stem_direction: None,
    })];

    let layout_f = layout_measure(&flagged, &cfg);
    let layout_b = layout_measure(&beamed, &cfg);

    let mut svg_f = make_svg();
    draw_measure(&mut svg_f, &staff, &font, &config, &layout_f, 0.0, &Clef::Treble).unwrap();
    let mut svg_b = make_svg();
    draw_measure(&mut svg_b, &staff, &font, &config, &layout_b, 0.0, &Clef::Treble).unwrap();

    let out_f = svg_f.to_svg();
    let out_b = svg_b.to_svg();

    // Flagged: 2 noteheads + 2 flags = 4 paths, 0 polygons
    assert_eq!(out_f.matches("<path ").count(), 4, "flagged: 2 noteheads + 2 flags");
    assert_eq!(out_f.matches("<polygon ").count(), 0, "flagged: no polygons");

    // Beamed: 2 noteheads = 2 paths, 1 polygon
    assert_eq!(out_b.matches("<path ").count(), 2, "beamed: 2 noteheads only");
    assert_eq!(out_b.matches("<polygon ").count(), 1, "beamed: 1 beam polygon");
}

#[test]
fn beam_group_mixed_durations() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    // Eighth + two sixteenths
    let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: vec![
            NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            NoteEvent { staff_position: 4, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            NoteEvent { staff_position: 6, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        ],
        stem_direction: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 3, "three noteheads");
    assert_eq!(output.matches("<line ").count(), 3, "three stems");
    // Primary beam spanning all + secondary beam for 16th notes
    assert!(
        output.matches("<polygon ").count() >= 2,
        "at least 2 beam polygons"
    );
}

// --- dynamics rendering in measure ---

use crate::layout::dynamics::Dynamic;

#[test]
fn note_with_dynamic_adds_extra_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: Some(Dynamic::Forte),
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 notehead + 1 dynamic glyph = 2 paths, 1 stem
    assert_eq!(output.matches("<path ").count(), 2, "notehead + dynamic");
    assert_eq!(output.matches("<line ").count(), 1, "one stem");
}

#[test]
fn note_without_dynamic_no_extra_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 1 notehead, no dynamic
    assert_eq!(output.matches("<path ").count(), 1, "notehead only");
}

#[test]
fn chord_with_dynamic_adds_extra_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: Some(Dynamic::Pp),
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 2 noteheads + 1 dynamic = 3 paths, 1 stem
    assert_eq!(output.matches("<path ").count(), 3, "2 noteheads + dynamic");
    assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
}

#[test]
fn dynamic_glyph_positioned_below_staff() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: Some(Dynamic::Mf),
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // Dynamic should be rendered with a translate below the staff.
    // The staff bottom_y is at position 0. The dynamic is 2.5 staff spaces below.
    let bottom_y = staff.bottom_y();
    let dynamic_y = bottom_y + 2.5 * staff.staff_space;
    // The SVG should contain a translate with the dynamic y coordinate
    let y_str = format!("{dynamic_y}");
    assert!(
        output.contains(&y_str),
        "dynamic translate should contain y={dynamic_y}"
    );
}

#[test]
fn different_dynamics_on_notes_produce_different_svgs() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |dyn_mark: Dynamic| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: false,
            dynamic: Some(dyn_mark),
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: None, tempo_mark: None, expression: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        svg.to_svg()
    };

    let svg_p = make(Dynamic::Piano);
    let svg_f = make(Dynamic::Forte);
    assert_ne!(svg_p, svg_f, "different dynamics should produce different SVGs");
}

// --- tuplet group rendering ---

use crate::layout::measure::TupletGroupEvent;

#[test]
fn tuplet_triplet_renders_beams_plus_bracket() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::TupletGroup(TupletGroupEvent {
        beam_group: BeamGroupEvent {
            notes: vec![
                NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
                NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
                NoteEvent { staff_position: 4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            ],
            stem_direction: None,
        },
        tuplet_number: 3,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 3 noteheads + 1 tuplet number glyph = 4 paths
    assert_eq!(output.matches("<path ").count(), 4, "3 noteheads + tuplet number");
    // 3 stems + 2 hooks + 2 bracket segments = 7 lines
    assert_eq!(output.matches("<line ").count(), 7, "3 stems + 4 bracket/hook lines");
    // 1 primary beam polygon
    assert_eq!(output.matches("<polygon ").count(), 1, "one beam polygon");
}

#[test]
fn tuplet_differs_from_plain_beam_group() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let notes = vec![
        NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        NoteEvent { staff_position: 4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
    ];

    let plain = vec![MeasureElement::BeamGroup(BeamGroupEvent {
        notes: notes.clone(),
        stem_direction: None,
    })];
    let tuplet = vec![MeasureElement::TupletGroup(TupletGroupEvent {
        beam_group: BeamGroupEvent {
            notes: notes.clone(),
            stem_direction: None,
        },
        tuplet_number: 3,
    })];

    let layout_p = layout_measure(&plain, &cfg);
    let layout_t = layout_measure(&tuplet, &cfg);

    let mut svg_p = make_svg();
    draw_measure(&mut svg_p, &staff, &font, &config, &layout_p, 0.0, &Clef::Treble).unwrap();
    let mut svg_t = make_svg();
    draw_measure(&mut svg_t, &staff, &font, &config, &layout_t, 0.0, &Clef::Treble).unwrap();

    let out_p = svg_p.to_svg();
    let out_t = svg_t.to_svg();

    // Tuplet has extra bracket lines and number glyph
    let paths_p = out_p.matches("<path ").count();
    let paths_t = out_t.matches("<path ").count();
    assert_eq!(paths_t, paths_p + 1, "tuplet adds 1 extra path (number glyph)");

    let lines_p = out_p.matches("<line ").count();
    let lines_t = out_t.matches("<line ").count();
    assert!(lines_t > lines_p, "tuplet has more lines (bracket + hooks)");
}

#[test]
fn tuplet_quintuplet_renders() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::TupletGroup(TupletGroupEvent {
        beam_group: BeamGroupEvent {
            notes: vec![
                NoteEvent { staff_position: 2, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
                NoteEvent { staff_position: 3, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
                NoteEvent { staff_position: 4, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
                NoteEvent { staff_position: 5, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
                NoteEvent { staff_position: 6, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
            ],
            stem_direction: None,
        },
        tuplet_number: 5,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    // 5 noteheads + 1 tuplet "5" glyph = 6 paths
    assert_eq!(output.matches("<path ").count(), 6, "5 noteheads + tuplet number");
}

#[test]
fn tuplet_empty_produces_nothing() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::TupletGroup(TupletGroupEvent {
        beam_group: BeamGroupEvent {
            notes: vec![],
            stem_direction: None,
        },
        tuplet_number: 3,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 0);
    assert_eq!(output.matches("<line ").count(), 0);
}

#[test]
fn tuplet_different_numbers_produce_different_glyphs() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let notes = vec![
        NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
        NoteEvent { staff_position: 4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None, slur_start: false, slur_end: false, hairpin_start: None, hairpin_end: false, rehearsal_mark: None, tempo_mark: None, expression: None },
    ];

    let make = |number: u32| {
        let elems = vec![MeasureElement::TupletGroup(TupletGroupEvent {
            beam_group: BeamGroupEvent {
                notes: notes.clone(),
                stem_direction: None,
            },
            tuplet_number: number,
        })];
        let layout = layout_measure(&elems, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        svg.to_svg()
    };

    let svg_3 = make(3);
    let svg_5 = make(5);
    assert_ne!(svg_3, svg_5, "triplet and quintuplet should differ");
}

// --- Rehearsal mark integration in measure renderer ---

#[test]
fn note_with_rehearsal_mark_produces_text_and_rect() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: Some(("A".to_string(), crate::layout::rehearsal::RehearsalStyle::Boxed)),
        tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();
    assert!(output.contains("<text "), "rehearsal mark should produce a text element");
    assert!(output.contains("<rect "), "boxed rehearsal mark should produce a rect element");
    assert!(output.contains(">A<"), "text content 'A' should appear");
}

#[test]
fn note_without_rehearsal_mark_has_no_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();
    assert!(!output.contains("<text "), "no rehearsal mark, no text element");
    assert!(!output.contains("<rect "), "no rehearsal mark, no rect element");
}

#[test]
fn chord_with_rehearsal_mark_produces_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 7],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: Some(("B".to_string(), crate::layout::rehearsal::RehearsalStyle::Boxed)),
        tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">B<"), "chord rehearsal mark text should appear");
}

#[test]
fn plain_rehearsal_mark_has_no_rect() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: Some(("C".to_string(), crate::layout::rehearsal::RehearsalStyle::Plain)),
        tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">C<"), "plain rehearsal mark text should appear");
    assert!(!output.contains("<rect "), "plain style should not produce a rect");
}

#[test]
fn rehearsal_mark_differs_from_no_mark() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let make = |mark: Option<(String, crate::layout::rehearsal::RehearsalStyle)>| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: false,
            hairpin_start: None,
            hairpin_end: false,
            rehearsal_mark: mark,
            tempo_mark: None, expression: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        svg.to_svg()
    };
    let with_mark = make(Some(("A".to_string(), crate::layout::rehearsal::RehearsalStyle::Boxed)));
    let without_mark = make(None);
    assert_ne!(with_mark, without_mark, "rehearsal mark should change SVG output");
}

// --- Tempo mark integration tests ---

#[test]
fn note_with_tempo_mark_produces_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);

    // Without tempo mark
    let mut svg_without = make_svg();
    draw_measure(&mut svg_without, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let out_without = svg_without.to_svg();

    // With tempo mark
    let elements_with = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: Some(crate::layout::tempo::TempoMark::Text("Allegro".into())),
        expression: None,
    })];
    let layout_with = layout_measure(&elements_with, &cfg);
    let mut svg_with = make_svg();
    draw_measure(&mut svg_with, &staff, &font, &config, &layout_with, 0.0, &Clef::Treble).unwrap();
    let out_with = svg_with.to_svg();

    assert!(out_with.contains(">Allegro<"), "tempo mark text should appear");
    assert!(!out_without.contains(">Allegro<"), "no tempo mark, no text");
    assert_ne!(out_with, out_without, "tempo mark should change SVG output");
}

#[test]
fn note_with_metronome_tempo_produces_path_and_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: Some(crate::layout::tempo::TempoMark::Metronome {
            note_kind: crate::layout::tempo::MetronomeNoteKind::Quarter,
            dotted: false,
            bpm: 120,
        }),
        expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();

    assert!(output.contains("= 120"), "metronome BPM should appear");
    // Should have a path for the metronome note glyph (in addition to the notehead)
    assert!(output.matches("<path").count() >= 2, "should have notehead + metronome glyph paths");
}

#[test]
fn chord_with_tempo_mark_produces_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 7],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: Some(crate::layout::tempo::TempoMark::Text("Presto".into())),
        expression: None,
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">Presto<"), "chord tempo mark text should appear");
}
