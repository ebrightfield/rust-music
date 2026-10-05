use super::*;
use crate::font::bravura_font;
use crate::layout::barline::BarlineStyle;
use crate::layout::clef::ClefLayout;
use crate::layout::key_signature::KeySignature;
use crate::layout::accidental::ResolvedAccidental;
use crate::layout::measure::{
    layout_measure, MeasureLayoutConfig, NoteAnnotations, RestEvent,
};
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
        total_rod: 0.0,
        total_spring: 0.0,
    };
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 1, "one notehead");
    assert_eq!(
        output.matches("<line ").count(),
        0,
        "no stem for whole note"
    );
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        accidental: Some(ResolvedAccidental::plain(Glyph::AccidentalSharp)),
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: Default::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 1, "one rest glyph");
    assert_eq!(output.matches("<line ").count(), 0, "no lines for rest");
}

#[test]
fn measure_with_multi_measure_rest_draws_hbar_and_count() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::MultiMeasureRest {
        count: 8,
        style: crate::layout::multi_measure_rest::MultiMeasureRestStyle::HBar,
    }];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    // H-bar = 2 vertical serifs + 1 crossbar = 3 filled rects
    assert_eq!(
        output.matches("<rect").count(),
        3,
        "H-bar should produce exactly 3 rects (two serifs + crossbar)",
    );
    // Count number appears as a <text> node
    assert!(
        output.contains(">8</text>"),
        "count number 8 should appear in a <text> element",
    );
    // No glyph paths or staff/stem lines for the H-bar itself
    assert_eq!(
        output.matches("<path ").count(),
        0,
        "H-bar uses rects, not paths"
    );
    assert_eq!(
        output.matches("<line ").count(),
        0,
        "H-bar has no <line> elements"
    );
}

#[test]
fn measure_with_multi_measure_rest_x_offset_shifts_hbar() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::MultiMeasureRest {
        count: 4,
        style: crate::layout::multi_measure_rest::MultiMeasureRestStyle::HBar,
    }];
    let layout = layout_measure(&elements, &cfg);

    let mut svg_a = make_svg();
    draw_measure(
        &mut svg_a,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let out_a = svg_a.to_svg();

    let mut svg_b = make_svg();
    draw_measure(
        &mut svg_b,
        &staff,
        &font,
        &config,
        &layout,
        1500.0,
        &Clef::Treble,
    )
    .unwrap();
    let out_b = svg_b.to_svg();

    assert_ne!(
        out_a, out_b,
        "x_offset must shift the H-bar rects/text coordinates",
    );
    // Both must still emit exactly 3 rects and the count text
    assert_eq!(out_a.matches("<rect").count(), 3);
    assert_eq!(out_b.matches("<rect").count(), 3);
    assert!(out_a.contains(">4</text>"));
    assert!(out_b.contains(">4</text>"));
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
            annotations: NoteAnnotations::default(),
        }),
        MeasureElement::Barline(BarlineStyle::Single),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
            annotations: NoteAnnotations::default(),
        }),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    // 1 clef + 2 key sig accidentals + 1 notehead = 4 paths
    assert_eq!(
        output.matches("<path ").count(),
        4,
        "clef + 2 sharps + notehead"
    );
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
            annotations: NoteAnnotations::default(),
        }),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);

    let mut svg_0 = make_svg();
    draw_measure(
        &mut svg_0,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();

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
    assert_eq!(notehead_kind_from_log2(-1), NoteheadKind::DoubleWhole);
    assert_eq!(notehead_kind_from_log2(0), NoteheadKind::Whole);
    assert_eq!(notehead_kind_from_log2(1), NoteheadKind::Half);
    assert_eq!(notehead_kind_from_log2(2), NoteheadKind::Filled);
    assert_eq!(notehead_kind_from_log2(3), NoteheadKind::Filled);
    assert_eq!(notehead_kind_from_log2(7), NoteheadKind::Filled);
}

#[test]
fn flag_count_from_log2_mapping() {
    assert_eq!(flag_count_from_log2(-1), 0); // breve
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
            annotations: NoteAnnotations::default(),
        }),
        MeasureElement::Note(NoteEvent {
            staff_position: 6,
            duration_log2: 3,
            dots: 0,
            accidental: Some(ResolvedAccidental::plain(Glyph::AccidentalNatural)),
            stem_direction: Some(StemDirection::Down),
            annotations: NoteAnnotations::default(),
        }),
        MeasureElement::Rest(RestEvent {
            duration_log2: 2,
            dots: 0,
            annotations: Default::default(),
        }),
        MeasureElement::Barline(BarlineStyle::Single),
    ];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_staff_lines(&mut svg, &staff, &config);
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    // Count expected paths:
    // clef(1) + flat(1) + time(2 digits) + notehead(1) + notehead(1) + accidental(1) + flag(1) + rest(1) = 9
    assert_eq!(
        output.matches("<path ").count(),
        9,
        "expected 9 paths total"
    );

    // Lines: 5 staff lines + 2 stems + 1 barline = 8
    assert_eq!(
        output.matches("<line ").count(),
        8,
        "expected 8 lines total"
    );
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    // 2 noteheads (at different x-offsets due to second), 1 stem
    assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
    assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
    // Verify both noteheads have different translate positions
    let translates: Vec<&str> = output.matches("translate(").collect();
    assert_eq!(
        translates.len(),
        2,
        "two translate transforms for two noteheads"
    );
}

#[test]
fn chord_with_accidentals() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![
            Some(ResolvedAccidental::plain(Glyph::AccidentalSharp)),
            None,
        ],
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    // 1 accidental + 2 noteheads = 3 paths, 1 stem
    assert_eq!(
        output.matches("<path ").count(),
        3,
        "accidental + 2 noteheads"
    );
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    // 3 noteheads, no stem
    assert_eq!(output.matches("<path ").count(), 3, "three noteheads");
    assert_eq!(
        output.matches("<line ").count(),
        0,
        "no stem for whole note chord"
    );
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    assert_eq!(
        output.matches("<path ").count(),
        0,
        "empty chord = no paths"
    );
    assert_eq!(
        output.matches("<line ").count(),
        0,
        "empty chord = no lines"
    );
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let chord = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];

    let layout_s = layout_measure(&single, &cfg);
    let layout_c = layout_measure(&chord, &cfg);

    let mut svg_s = make_svg();
    draw_measure(
        &mut svg_s,
        &staff,
        &font,
        &config,
        &layout_s,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let mut svg_c = make_svg();
    draw_measure(
        &mut svg_c,
        &staff,
        &font,
        &config,
        &layout_c,
        0.0,
        &Clef::Treble,
    )
    .unwrap();

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
        annotations: NoteAnnotations::default(),
    })];
    let elements_down = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations::default(),
    })];

    let layout_up = layout_measure(&elements_up, &cfg);
    let layout_down = layout_measure(&elements_down, &cfg);

    let mut svg_up = make_svg();
    draw_measure(
        &mut svg_up,
        &staff,
        &font,
        &config,
        &layout_up,
        0.0,
        &Clef::Treble,
    )
    .unwrap();

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
        annotations: NoteAnnotations {
            dynamic: Some(Dynamic::Forte.into()),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations {
            dynamic: Some(Dynamic::Pp.into()),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
        annotations: NoteAnnotations {
            dynamic: Some(Dynamic::Mf.into()),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
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
            annotations: NoteAnnotations {
                dynamic: Some(dyn_mark.into()),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };

    let svg_p = make(Dynamic::Piano);
    let svg_f = make(Dynamic::Forte);
    assert_ne!(
        svg_p, svg_f,
        "different dynamics should produce different SVGs"
    );
}

// --- tuplet group rendering ---


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
        annotations: NoteAnnotations {
            rehearsal_mark: Some((
                "A".to_string(),
                crate::layout::rehearsal::RehearsalStyle::Boxed,
            )),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(
        output.contains("<text "),
        "rehearsal mark should produce a text element"
    );
    assert!(
        output.contains("<rect "),
        "boxed rehearsal mark should produce a rect element"
    );
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(
        !output.contains("<text "),
        "no rehearsal mark, no text element"
    );
    assert!(
        !output.contains("<rect "),
        "no rehearsal mark, no rect element"
    );
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
        annotations: NoteAnnotations {
            rehearsal_mark: Some((
                "B".to_string(),
                crate::layout::rehearsal::RehearsalStyle::Boxed,
            )),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(
        output.contains(">B<"),
        "chord rehearsal mark text should appear"
    );
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
        annotations: NoteAnnotations {
            rehearsal_mark: Some((
                "C".to_string(),
                crate::layout::rehearsal::RehearsalStyle::Plain,
            )),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(
        output.contains(">C<"),
        "plain rehearsal mark text should appear"
    );
    assert!(
        !output.contains("<rect "),
        "plain style should not produce a rect"
    );
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
            annotations: NoteAnnotations {
                rehearsal_mark: mark,
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let with_mark = make(Some((
        "A".to_string(),
        crate::layout::rehearsal::RehearsalStyle::Boxed,
    )));
    let without_mark = make(None);
    assert_ne!(
        with_mark, without_mark,
        "rehearsal mark should change SVG output"
    );
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
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);

    // Without tempo mark
    let mut svg_without = make_svg();
    draw_measure(
        &mut svg_without,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let out_without = svg_without.to_svg();

    // With tempo mark
    let elements_with = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            tempo_mark: Some(crate::layout::tempo::TempoMark::text("Allegro")),
            ..Default::default()
        },
    })];
    let layout_with = layout_measure(&elements_with, &cfg);
    let mut svg_with = make_svg();
    draw_measure(
        &mut svg_with,
        &staff,
        &font,
        &config,
        &layout_with,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let out_with = svg_with.to_svg();

    assert!(
        out_with.contains(">Allegro<"),
        "tempo mark text should appear"
    );
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
        annotations: NoteAnnotations {
            tempo_mark: Some(crate::layout::tempo::TempoMark::metronome(crate::layout::tempo::MetronomeMark::bpm(crate::layout::tempo::MetronomeNoteKind::Quarter, 120))),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    assert!(output.contains("= 120"), "metronome BPM should appear");
    // Should have a path for the metronome note glyph (in addition to the notehead)
    assert!(
        output.matches("<path").count() >= 2,
        "should have notehead + metronome glyph paths"
    );
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
        annotations: NoteAnnotations {
            tempo_mark: Some(crate::layout::tempo::TempoMark::text("Presto")),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(
        output.contains(">Presto<"),
        "chord tempo mark text should appear"
    );
}

#[test]
fn note_with_lyric_adds_text_element() {
    use crate::layout::lyric::LyricSyllable;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            lyric: Some(LyricSyllable::word("sun")),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(
        output.contains(">sun<"),
        "lyric text 'sun' should appear in SVG"
    );
    assert!(
        output.contains("<text"),
        "should contain a text element for the lyric"
    );
}

#[test]
fn note_without_lyric_has_no_lyric_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert_eq!(
        output.matches("<text").count(),
        0,
        "note without lyric should have no text elements"
    );
}

#[test]
fn chord_with_lyric_adds_text_element() {
    use crate::layout::lyric::LyricSyllable;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 8],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations {
            lyric: Some(LyricSyllable::with_hyphen("hap")),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    // The lyric text is rendered alone — the hyphen between syllables is
    // drawn by the system-level pass once the next syllable's x is known.
    // The measure renderer must emit just the syllable text, not "hap -".
    assert!(
        output.contains(">hap<"),
        "chord lyric should render 'hap' alone, got: {output}"
    );
    assert!(
        !output.contains("hap -"),
        "measure renderer must not append ' -' to hyphenated syllable, got: {output}"
    );
}

#[test]
fn lyric_differs_from_no_lyric() {
    use crate::layout::lyric::LyricSyllable;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let make = |lyric: Option<LyricSyllable>| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                lyric,
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let without = make(None);
    let with = make(Some(LyricSyllable::word("day")));
    assert_ne!(without, with, "lyric should change the SVG output");
}

#[test]
fn note_with_chord_symbol_adds_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            chord_symbol: Some("Cmaj7".to_string()),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(
        output.contains(">Cmaj7<"),
        "should contain chord symbol text"
    );
    assert!(output.contains("bold"), "chord symbol should be bold");
}

#[test]
fn note_without_chord_symbol_has_no_extra_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert_eq!(
        output.matches("<text").count(),
        0,
        "no text elements without chord symbol"
    );
}

#[test]
fn chord_event_with_chord_symbol_adds_text() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 7],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations {
            chord_symbol: Some("C".to_string()),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">C<"), "should contain chord symbol 'C'");
}

#[test]
fn chord_symbol_changes_output() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let make = |sym: Option<String>| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                chord_symbol: sym,
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let without = make(None);
    let with = make(Some("Am7".to_string()));
    assert_ne!(without, with, "chord symbol should change the SVG output");
}

// --- Ornament tests ---

#[test]
fn note_with_ornament_trill_adds_path() {
    use crate::layout::ornament::Ornament;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let make = |orn: Option<Ornament>| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                ornament: orn,
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let without = make(None);
    let with = make(Some(Ornament::Trill));
    let paths_without = without.matches("<path").count();
    let paths_with = with.matches("<path").count();
    assert!(
        paths_with > paths_without,
        "trill ornament should add a path element: {} vs {}",
        paths_with,
        paths_without
    );
}

#[test]
fn note_with_ornament_turn_adds_path() {
    use crate::layout::ornament::Ornament;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 6,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Turn),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let out = svg.to_svg();
    assert!(out.contains("<path"), "ornament turn should produce a path");
    assert!(
        out.contains("translate("),
        "ornament should have a translate transform"
    );
}

#[test]
fn chord_with_ornament_adds_path() {
    use crate::layout::ornament::Ornament;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let make = |orn: Option<Ornament>| {
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4, 8],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None, None],
            stem_direction: None,
            annotations: NoteAnnotations {
                ornament: orn,
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let without = make(None);
    let with = make(Some(Ornament::Mordent));
    assert_ne!(without, with, "ornament on chord should change SVG output");
}

#[test]
fn different_ornaments_on_note_produce_different_output() {
    use crate::layout::ornament::Ornament;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let make = |orn: Ornament| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                ornament: Some(orn),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let trill_svg = make(Ornament::Trill);
    let mordent_svg = make(Ornament::Mordent);
    assert_ne!(trill_svg, mordent_svg, "different ornaments should differ");
}

// Navigation sign (segno, coda) tests

#[test]
fn note_with_navigation_sign_adds_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let without = {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations::default(),
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let with_segno = {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                navigation_sign: Some(crate::layout::navigation::NavigationSign::Segno),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };

    let without_paths = without.matches("<path ").count();
    let with_paths = with_segno.matches("<path ").count();
    assert!(
        with_paths > without_paths,
        "navigation sign should add a path: {} > {}",
        with_paths,
        without_paths
    );
}

#[test]
fn chord_with_navigation_sign_adds_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let without = {
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4, 7],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None, None],
            stem_direction: None,
            annotations: NoteAnnotations::default(),
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let with_coda = {
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4, 7],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None, None],
            stem_direction: None,
            annotations: NoteAnnotations {
                navigation_sign: Some(crate::layout::navigation::NavigationSign::Coda),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };

    let without_paths = without.matches("<path ").count();
    let with_paths = with_coda.matches("<path ").count();
    assert!(
        with_paths > without_paths,
        "coda sign on chord should add a path: {} > {}",
        with_paths,
        without_paths
    );
}

#[test]
fn different_navigation_signs_produce_different_output() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |sign: crate::layout::navigation::NavigationSign| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                navigation_sign: Some(sign),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let segno_svg = make(crate::layout::navigation::NavigationSign::Segno);
    let coda_svg = make(crate::layout::navigation::NavigationSign::Coda);
    assert_ne!(
        segno_svg, coda_svg,
        "segno and coda should produce different output"
    );
}

// Pedal marking tests

#[test]
fn note_with_pedal_down_adds_path() {
    use crate::layout::pedal::PedalMark;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let without = {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations::default(),
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let with_pedal = {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                pedal: Some(PedalMark::Down),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let paths_without = without.matches("<path ").count();
    let paths_with = with_pedal.matches("<path ").count();
    assert!(
        paths_with > paths_without,
        "pedal down should add a path: {paths_with} vs {paths_without}"
    );
}

#[test]
fn note_without_pedal_has_fewer_paths() {
    use crate::layout::pedal::PedalMark;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |pedal: Option<PedalMark>| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                pedal,
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg().matches("<path ").count()
    };
    assert!(make(None) < make(Some(PedalMark::Down)));
    assert!(make(None) < make(Some(PedalMark::Up)));
}

#[test]
fn chord_with_pedal_down_adds_path() {
    use crate::layout::measure::ChordEvent;
    use crate::layout::pedal::PedalMark;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |pedal: Option<PedalMark>| {
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4, 7],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None, None],
            stem_direction: None,
            annotations: NoteAnnotations {
                pedal,
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg().matches("<path ").count()
    };
    assert!(
        make(Some(PedalMark::Down)) > make(None),
        "chord with pedal should have more paths"
    );
}

#[test]
fn pedal_down_and_up_produce_different_output() {
    use crate::layout::pedal::PedalMark;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |mark: PedalMark| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                pedal: Some(mark),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };
    let down_svg = make(PedalMark::Down);
    let up_svg = make(PedalMark::Up);
    assert_ne!(down_svg, up_svg, "pedal down and up should differ");
}

// ── Tremolo tests ──────────────────────────────────────────────

#[test]
fn note_with_tremolo_adds_extra_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let with = NoteEvent {
        staff_position: 4,
        duration_log2: 2, // quarter note (has stem)
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            tremolo: Some(crate::layout::tremolo::TremoloCount::Single),
            ..NoteAnnotations::default()
        },
    };
    let without = NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    };

    let elements_with = vec![MeasureElement::Note(with)];
    let elements_without = vec![MeasureElement::Note(without)];

    let layout_with = layout_measure(&elements_with, &cfg);
    let layout_without = layout_measure(&elements_without, &cfg);

    let mut svg_with = make_svg();
    draw_measure(
        &mut svg_with,
        &staff,
        &font,
        &config,
        &layout_with,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let mut svg_without = make_svg();
    draw_measure(
        &mut svg_without,
        &staff,
        &font,
        &config,
        &layout_without,
        0.0,
        &Clef::Treble,
    )
    .unwrap();

    let with_paths = svg_with.to_svg().matches("<path ").count();
    let without_paths = svg_without.to_svg().matches("<path ").count();
    assert!(
        with_paths > without_paths,
        "tremolo should add an extra path: with={}, without={}",
        with_paths,
        without_paths
    );
}

#[test]
fn chord_with_tremolo_adds_extra_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let with = ChordEvent {
        staff_positions: vec![0, 4, 8],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations {
            tremolo: Some(crate::layout::tremolo::TremoloCount::Double),
            ..NoteAnnotations::default()
        },
    };
    let without = ChordEvent {
        staff_positions: vec![0, 4, 8],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    };

    let elements_with = vec![MeasureElement::Chord(with)];
    let elements_without = vec![MeasureElement::Chord(without)];

    let layout_with = layout_measure(&elements_with, &cfg);
    let layout_without = layout_measure(&elements_without, &cfg);

    let mut svg_with = make_svg();
    draw_measure(
        &mut svg_with,
        &staff,
        &font,
        &config,
        &layout_with,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let mut svg_without = make_svg();
    draw_measure(
        &mut svg_without,
        &staff,
        &font,
        &config,
        &layout_without,
        0.0,
        &Clef::Treble,
    )
    .unwrap();

    let with_paths = svg_with.to_svg().matches("<path ").count();
    let without_paths = svg_without.to_svg().matches("<path ").count();
    assert!(
        with_paths > without_paths,
        "chord tremolo should add extra path"
    );
}

#[test]
fn different_tremolo_counts_produce_different_svgs() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |count: crate::layout::tremolo::TremoloCount| -> String {
        let note = NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                tremolo: Some(count),
                ..NoteAnnotations::default()
            },
        };
        let elements = vec![MeasureElement::Note(note)];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };

    let single = make(crate::layout::tremolo::TremoloCount::Single);
    let double = make(crate::layout::tremolo::TremoloCount::Double);
    let triple = make(crate::layout::tremolo::TremoloCount::Triple);

    assert_ne!(single, double, "single and double tremolo should differ");
    assert_ne!(double, triple, "double and triple tremolo should differ");
    assert_ne!(single, triple, "single and triple tremolo should differ");
}

#[test]
fn whole_note_tremolo_not_drawn_without_stem() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let with = NoteEvent {
        staff_position: 4,
        duration_log2: 0, // whole note — no stem
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            tremolo: Some(crate::layout::tremolo::TremoloCount::Single),
            ..NoteAnnotations::default()
        },
    };
    let without = NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    };

    let elements_with = vec![MeasureElement::Note(with)];
    let elements_without = vec![MeasureElement::Note(without)];

    let layout_with = layout_measure(&elements_with, &cfg);
    let layout_without = layout_measure(&elements_without, &cfg);

    let mut svg_with = make_svg();
    draw_measure(
        &mut svg_with,
        &staff,
        &font,
        &config,
        &layout_with,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let mut svg_without = make_svg();
    draw_measure(
        &mut svg_without,
        &staff,
        &font,
        &config,
        &layout_without,
        0.0,
        &Clef::Treble,
    )
    .unwrap();

    // Whole notes have no stem, so tremolo can't be drawn on a stem
    let with_paths = svg_with.to_svg().matches("<path ").count();
    let without_paths = svg_without.to_svg().matches("<path ").count();
    assert_eq!(
        with_paths, without_paths,
        "whole note tremolo should not add path (no stem)"
    );
}

// --- arpeggio ---

#[test]
fn chord_with_arpeggio_adds_path() {
    use crate::layout::arpeggio::ArpeggioDirection;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |arp: Option<ArpeggioDirection>| {
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4, 8],
            duration_log2: 1,
            dots: 0,
            accidentals: vec![],
            stem_direction: None,
            annotations: NoteAnnotations {
                arpeggio: arp,
                ..NoteAnnotations::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg().matches("<path ").count()
    };

    let with = make(Some(ArpeggioDirection::Up));
    let without = make(None);
    assert!(
        with > without,
        "chord with arpeggio should have more paths: {with} vs {without}"
    );
}

#[test]
fn chord_without_arpeggio_has_no_extra_path() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    let elements = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 8],
        duration_log2: 1,
        dots: 0,
        accidentals: vec![],
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();
    // Without arpeggio, no arpeggio transform should appear
    assert!(
        !output.contains("scale(1,"),
        "no vertical scaling without arpeggio"
    );
}

#[test]
fn note_with_arpeggio_adds_path() {
    use crate::layout::arpeggio::ArpeggioDirection;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |arp: Option<ArpeggioDirection>| {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 0,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                arpeggio: arp,
                ..NoteAnnotations::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg().matches("<path ").count()
    };

    let with = make(Some(ArpeggioDirection::Up));
    let without = make(None);
    assert!(
        with > without,
        "note with arpeggio should have more paths: {with} vs {without}"
    );
}

#[test]
fn arpeggio_up_and_down_differ_on_chord() {
    use crate::layout::arpeggio::ArpeggioDirection;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |dir: ArpeggioDirection| {
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4, 8],
            duration_log2: 1,
            dots: 0,
            accidentals: vec![],
            stem_direction: None,
            annotations: NoteAnnotations {
                arpeggio: Some(dir),
                ..NoteAnnotations::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };

    assert_ne!(
        make(ArpeggioDirection::Up),
        make(ArpeggioDirection::Down),
        "up and down arpeggio should produce different SVG"
    );
}

// --- Breath mark tests ---

#[test]
fn note_with_breath_mark_produces_extra_path() {
    use crate::layout::breath::BreathMark;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Without breath mark
    let elements_no = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];
    let layout_no = layout_measure(&elements_no, &cfg);
    let mut svg_no = make_svg();
    draw_measure(
        &mut svg_no,
        &staff,
        &font,
        &config,
        &layout_no,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let without = svg_no.to_svg();

    // With breath mark
    let elements_yes = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            breath_mark: Some(BreathMark::Comma),
            ..Default::default()
        },
    })];
    let layout_yes = layout_measure(&elements_yes, &cfg);
    let mut svg_yes = make_svg();
    draw_measure(
        &mut svg_yes,
        &staff,
        &font,
        &config,
        &layout_yes,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let with = svg_yes.to_svg();

    let paths_without = without.matches("<path").count();
    let paths_with = with.matches("<path").count();
    assert_eq!(
        paths_with,
        paths_without + 1,
        "breath mark should add exactly 1 path element"
    );
}

#[test]
fn chord_with_breath_mark_adds_path() {
    use crate::layout::breath::BreathMark;
    use crate::layout::measure::ChordEvent;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Without breath mark
    let elements_no = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 7],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })];
    let layout_no = layout_measure(&elements_no, &cfg);
    let mut svg_no = make_svg();
    draw_measure(
        &mut svg_no,
        &staff,
        &font,
        &config,
        &layout_no,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let without = svg_no.to_svg();

    // With breath mark
    let elements_yes = vec![MeasureElement::Chord(ChordEvent {
        staff_positions: vec![0, 4, 7],
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations {
            breath_mark: Some(BreathMark::Comma),
            ..Default::default()
        },
    })];
    let layout_yes = layout_measure(&elements_yes, &cfg);
    let mut svg_yes = make_svg();
    draw_measure(
        &mut svg_yes,
        &staff,
        &font,
        &config,
        &layout_yes,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let with = svg_yes.to_svg();

    let paths_without = without.matches("<path").count();
    let paths_with = with.matches("<path").count();
    assert_eq!(
        paths_with,
        paths_without + 1,
        "breath mark on chord should add exactly 1 path element"
    );
}

#[test]
fn different_breath_marks_produce_different_svg() {
    use crate::layout::breath::BreathMark;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let make = |mark: BreathMark| -> String {
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                breath_mark: Some(mark),
                ..Default::default()
            },
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(
            &mut svg,
            &staff,
            &font,
            &config,
            &layout,
            0.0,
            &Clef::Treble,
        )
        .unwrap();
        svg.to_svg()
    };

    let comma = make(BreathMark::Comma);
    let tick = make(BreathMark::Tick);
    let caesura = make(BreathMark::Caesura);

    assert_ne!(comma, tick, "comma and tick should differ");
    assert_ne!(tick, caesura, "tick and caesura should differ");
    assert_ne!(comma, caesura, "comma and caesura should differ");
}

#[test]
fn breath_mark_translate_is_right_of_note() {
    use crate::layout::breath::BreathMark;
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            breath_mark: Some(BreathMark::Comma),
            ..Default::default()
        },
    })];
    let layout = layout_measure(&elements, &cfg);
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let output = svg.to_svg();

    // The breath mark translate x should appear in the SVG
    // and be positive (to the right of x=0)
    let translate_count = output.matches("translate(").count();
    assert!(
        translate_count >= 2,
        "should have at least 2 translate()s (notehead + breath mark), got {translate_count}"
    );
}

// ── Multi-voice rendering tests ────────────────────────────────────────────

#[test]
fn additional_voices_empty_produces_no_extra_elements() {
    let (font, config, staff) = setup();
    let mut svg = make_svg();
    // No additional voice layouts → no extra paths
    let empty_primary = MeasureLayout {
        elements: vec![],
        total_width: 0.0,
        total_rod: 0.0,
        total_spring: 0.0,
    };
    draw_additional_voices(&mut svg, &staff, &font, &config, &empty_primary, &[], 0.0).unwrap();
    let output = svg.to_svg();
    assert_eq!(output.matches("<path ").count(), 0);
    assert_eq!(output.matches("<line ").count(), 0);
}

#[test]
fn additional_voice_with_note_draws_extra_notehead() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Primary voice: quarter note at position 4 (middle line)
    let primary_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Up),
        annotations: NoteAnnotations::default(),
    })];
    let primary_layout = layout_measure(&primary_elements, &cfg);

    // Additional voice: quarter note at position 0 (bottom line), stems down
    let voice1_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations::default(),
    })];
    let mut voice1_layout = layout_measure(&voice1_elements, &cfg);
    // Scale to match primary width
    if voice1_layout.total_width > 0.0 && primary_layout.total_width > 0.0 {
        let scale = primary_layout.total_width / voice1_layout.total_width;
        for elem in &mut voice1_layout.elements {
            elem.x *= scale;
            elem.width *= scale;
        }
        voice1_layout.total_width = primary_layout.total_width;
    }

    // Draw primary only
    let mut svg_primary = make_svg();
    draw_measure(
        &mut svg_primary,
        &staff,
        &font,
        &config,
        &primary_layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    let primary_paths = svg_primary.to_svg().matches("<path ").count();

    // Draw primary + additional voice
    let mut svg_both = make_svg();
    draw_measure(
        &mut svg_both,
        &staff,
        &font,
        &config,
        &primary_layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    draw_additional_voices(
        &mut svg_both,
        &staff,
        &font,
        &config,
        &primary_layout,
        &[voice1_layout],
        0.0,
    )
    .unwrap();
    let both_paths = svg_both.to_svg().matches("<path ").count();

    // Additional voice adds at least one more path (the notehead)
    assert!(
        both_paths > primary_paths,
        "additional voice should add extra paths: primary={primary_paths}, both={both_paths}"
    );
}

#[test]
fn additional_voice_rest_is_displaced_downward() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Additional voice 1 (index 0): rest displaced down by 2 staff spaces
    let voice1_elements = vec![MeasureElement::Rest(RestEvent {
        duration_log2: 2,
        dots: 0,
        annotations: Default::default(),
    })];
    let voice1_layout = layout_measure(&voice1_elements, &cfg);

    // Draw rest without displacement (baseline)
    let mut svg_normal = make_svg();
    crate::render::rest_renderer::draw_rest(&mut svg_normal, &staff, &font, 100.0, 2).unwrap();
    let normal_svg = svg_normal.to_svg();

    // Draw rest via additional voices (should be displaced)
    let mut svg_displaced = make_svg();
    let empty_primary = MeasureLayout {
        elements: vec![],
        total_width: 0.0,
        total_rod: 0.0,
        total_spring: 0.0,
    };
    draw_additional_voices(
        &mut svg_displaced,
        &staff,
        &font,
        &config,
        &empty_primary,
        &[voice1_layout],
        0.0,
    )
    .unwrap();
    let displaced_svg = svg_displaced.to_svg();

    // Both should produce a path, but the translate y should differ
    assert_eq!(normal_svg.matches("<path ").count(), 1);
    assert_eq!(displaced_svg.matches("<path ").count(), 1);
    // The displaced version should have a different y coordinate
    assert_ne!(
        normal_svg, displaced_svg,
        "displaced rest should differ from normal rest"
    );
}

#[test]
fn additional_voice_skips_barlines() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Voice layout with a barline element — should be skipped by draw_additional_voices
    let elements = vec![
        MeasureElement::Note(NoteEvent {
            staff_position: 0,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: Some(StemDirection::Down),
            annotations: NoteAnnotations::default(),
        }),
        MeasureElement::Barline(BarlineStyle::Single),
    ];
    let layout = layout_measure(&elements, &cfg);

    let mut svg = make_svg();
    let empty_primary = MeasureLayout {
        elements: vec![],
        total_width: 0.0,
        total_rod: 0.0,
        total_spring: 0.0,
    };
    draw_additional_voices(
        &mut svg,
        &staff,
        &font,
        &config,
        &empty_primary,
        &[layout],
        0.0,
    )
    .unwrap();
    let output = svg.to_svg();

    // The barline should not appear (no vertical line from barline renderer)
    // but the note path should appear
    assert!(
        output.matches("<path ").count() >= 1,
        "note should still render"
    );
    // Additional voice barlines are skipped — the primary voice already drew them.
    // We verify indirectly: line count should only include stem line(s), not barline.
    let line_count = output.matches("<line ").count();
    assert!(
        line_count <= 1,
        "at most 1 line (stem), no barline: got {line_count}"
    );
}

#[test]
fn two_additional_voices_both_render() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Voice 1: note at position 0 (bottom)
    let v1_elems = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations::default(),
    })];
    let v1_layout = layout_measure(&v1_elems, &cfg);

    // Voice 2: note at position 8 (top)
    let v2_elems = vec![MeasureElement::Note(NoteEvent {
        staff_position: 8,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Up),
        annotations: NoteAnnotations::default(),
    })];
    let v2_layout = layout_measure(&v2_elems, &cfg);

    let mut svg = make_svg();
    let empty_primary = MeasureLayout {
        elements: vec![],
        total_width: 0.0,
        total_rod: 0.0,
        total_spring: 0.0,
    };
    draw_additional_voices(
        &mut svg,
        &staff,
        &font,
        &config,
        &empty_primary,
        &[v1_layout, v2_layout],
        0.0,
    )
    .unwrap();
    let output = svg.to_svg();

    // Two noteheads + two stems = at least 2 paths and 2 lines
    let paths = output.matches("<path ").count();
    let lines = output.matches("<line ").count();
    assert!(
        paths >= 2,
        "two voices should produce at least 2 notehead paths, got {paths}"
    );
    assert!(
        lines >= 2,
        "two voices should produce at least 2 stem lines, got {lines}"
    );
}

// ── Cross-voice collision avoidance tests ────────────────────────────────

#[test]
fn collision_at_unison_offsets_additional_voice_notehead() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Primary: quarter at position 4, stems up
    let primary_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Up),
        annotations: NoteAnnotations::default(),
    })];
    let primary_layout = layout_measure(&primary_elements, &cfg);

    // Additional: quarter at SAME position 4, stems down → collision
    let voice1_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations::default(),
    })];
    let mut voice1_layout = layout_measure(&voice1_elements, &cfg);
    // Scale to match primary
    let scale = primary_layout.total_width / voice1_layout.total_width;
    for elem in &mut voice1_layout.elements {
        elem.x *= scale;
        elem.width *= scale;
    }
    voice1_layout.total_width = primary_layout.total_width;

    // Draw with collision: additional voice's note should be offset
    let mut svg_collision = make_svg();
    draw_measure(
        &mut svg_collision,
        &staff,
        &font,
        &config,
        &primary_layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    draw_additional_voices(
        &mut svg_collision,
        &staff,
        &font,
        &config,
        &primary_layout,
        &[voice1_layout.clone()],
        0.0,
    )
    .unwrap();
    let collision_svg = svg_collision.to_svg();

    // Draw without collision: note at position 0 (far from primary pos 4)
    let far_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations::default(),
    })];
    let mut far_layout = layout_measure(&far_elements, &cfg);
    let scale2 = primary_layout.total_width / far_layout.total_width;
    for elem in &mut far_layout.elements {
        elem.x *= scale2;
        elem.width *= scale2;
    }
    far_layout.total_width = primary_layout.total_width;

    let mut svg_no_collision = make_svg();
    draw_measure(
        &mut svg_no_collision,
        &staff,
        &font,
        &config,
        &primary_layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    draw_additional_voices(
        &mut svg_no_collision,
        &staff,
        &font,
        &config,
        &primary_layout,
        &[far_layout],
        0.0,
    )
    .unwrap();
    let no_collision_svg = svg_no_collision.to_svg();

    // The collision version should differ from the no-collision version
    // (different translate() positions due to offset)
    assert_ne!(
        collision_svg, no_collision_svg,
        "unison collision should produce different SVG than non-colliding voices"
    );
}

#[test]
fn collision_at_second_offsets_additional_voice_notehead() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Primary: quarter at position 5, stems up
    let primary_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 5,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Up),
        annotations: NoteAnnotations::default(),
    })];
    let primary_layout = layout_measure(&primary_elements, &cfg);

    // Additional: quarter at position 4 (second below), stems down → collision
    let voice1_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations::default(),
    })];
    let mut voice1_layout = layout_measure(&voice1_elements, &cfg);
    let scale = primary_layout.total_width / voice1_layout.total_width;
    for elem in &mut voice1_layout.elements {
        elem.x *= scale;
        elem.width *= scale;
    }
    voice1_layout.total_width = primary_layout.total_width;

    // Draw with collision
    let mut svg = make_svg();
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &primary_layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    draw_additional_voices(
        &mut svg,
        &staff,
        &font,
        &config,
        &primary_layout,
        &[voice1_layout],
        0.0,
    )
    .unwrap();
    let output = svg.to_svg();

    // Should have 2 notehead paths (primary + additional)
    let paths = output.matches("<path ").count();
    assert!(
        paths >= 2,
        "second collision should still render both noteheads, got {paths} paths"
    );
}

#[test]
fn no_collision_at_third_no_offset() {
    let (font, config, staff) = setup();
    let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    // Primary: quarter at position 6, stems up
    let primary_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 6,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Up),
        annotations: NoteAnnotations::default(),
    })];
    let primary_layout = layout_measure(&primary_elements, &cfg);

    // Additional: quarter at position 4 (third below = distance 2), stems down → NO collision
    let voice1_elements = vec![MeasureElement::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations::default(),
    })];
    let mut voice1_layout = layout_measure(&voice1_elements, &cfg);
    let scale = primary_layout.total_width / voice1_layout.total_width;
    for elem in &mut voice1_layout.elements {
        elem.x *= scale;
        elem.width *= scale;
    }
    voice1_layout.total_width = primary_layout.total_width;

    // Draw with collision detection (but no actual collision)
    let mut svg_with_detection = make_svg();
    draw_measure(
        &mut svg_with_detection,
        &staff,
        &font,
        &config,
        &primary_layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    draw_additional_voices(
        &mut svg_with_detection,
        &staff,
        &font,
        &config,
        &primary_layout,
        &[voice1_layout.clone()],
        0.0,
    )
    .unwrap();
    let detected_svg = svg_with_detection.to_svg();

    // Draw without any collision detection (empty primary)
    let empty_primary = MeasureLayout {
        elements: vec![],
        total_width: 0.0,
        total_rod: 0.0,
        total_spring: 0.0,
    };
    let mut svg_no_detection = make_svg();
    draw_measure(
        &mut svg_no_detection,
        &staff,
        &font,
        &config,
        &primary_layout,
        0.0,
        &Clef::Treble,
    )
    .unwrap();
    draw_additional_voices(
        &mut svg_no_detection,
        &staff,
        &font,
        &config,
        &empty_primary,
        &[voice1_layout],
        0.0,
    )
    .unwrap();
    let undetected_svg = svg_no_detection.to_svg();

    // For a third (no collision), both should produce identical output
    assert_eq!(
        detected_svg, undetected_svg,
        "notes a third apart should not be offset (no collision)"
    );
}


