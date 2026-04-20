use super::*;
use crate::font::bravura_font;
use crate::layout::barline::BarlineStyle;
use crate::layout::key_signature::KeySignature;
use crate::layout::lyric::LyricSyllable;
use crate::layout::measure::{MeasureLayoutConfig, NoteAnnotations, NoteEvent, RestEvent};
use crate::layout::system::{
    layout_system, MeasureContent, MeasureEvent, SystemPrefix,
};
use crate::layout::time_signature::TimeSignatureKind;
use music::notation::clef::Clef;

fn setup() -> (MusicFont<'static>, EngravingConfig, MeasureLayoutConfig) {
    let font = bravura_font();
    let config = font.engraving_config();
    let measure_cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
    (font, config, measure_cfg)
}

fn make_svg() -> SvgWriter {
    SvgWriter::new(1200.0, 200.0, -200.0, -200.0, 12000.0, 2000.0)
}

fn treble_prefix() -> SystemPrefix {
    SystemPrefix::new(
        &Clef::Treble,
        KeySignature::Open,
        Some(TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        }),
    )
}

fn quarter_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
    annotations: NoteAnnotations::default(),
    })
}

#[test]
fn system_renders_staff_lines() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<line ").count(), 5 + 1 + 1,
        "5 staff lines + 1 stem + 1 barline = 7");
}

#[test]
fn two_measure_system() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![quarter_note(4), quarter_note(6)],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![quarter_note(2), quarter_note(8)],
            barline: BarlineStyle::Final,
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // 4 noteheads + 1 clef + 2 time sig digits = 7 paths
    assert_eq!(output.matches("<path ").count(), 7);
    // 5 staff + 4 stems + 1 single barline + 2 final barline strokes = 12 lines
    // (final barline = thin + thick, drawn as 2 lines)
    let line_count = output.matches("<line ").count();
    assert!(line_count >= 10, "expected >= 10 lines, got {}", line_count);
}

#[test]
fn system_with_x_y_offset() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg1 = make_svg();
    draw_system(&mut svg1, &font, &config, &system, 0.0, 0.0).unwrap();

    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system, 500.0, 300.0).unwrap();

    assert_ne!(svg1.to_svg(), svg2.to_svg(), "offset should change SVG");
}

#[test]
fn system_with_key_signature() {
    let (font, config, mcfg) = setup();
    let prefix = SystemPrefix::new(
        &Clef::Treble,
        KeySignature::Flats(3),
        Some(TimeSignatureKind::Numeric {
            numerator: 3,
            denominator: 4,
        }),
    );
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&prefix, &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // clef(1) + 3 flats + 2 time digits + 1 notehead = 7 paths
    assert_eq!(output.matches("<path ").count(), 7);
}

#[test]
fn system_with_rests() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            quarter_note(4),
            MeasureEvent::Rest(RestEvent {
                duration_log2: 2,
                dots: 0,
            }),
        ],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // clef(1) + 2 time digits + notehead(1) + rest(1) = 5 paths
    assert_eq!(output.matches("<path ").count(), 5);
}

#[test]
fn target_width_produces_wider_staff() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Single,
    }];

    let natural = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let wide = layout_system(&treble_prefix(), &measures, &mcfg, Some(10000.0));

    // The wide system staff lines should span a wider x-range
    let mut svg_n = make_svg();
    draw_system(&mut svg_n, &font, &config, &natural, 0.0, 0.0).unwrap();
    let mut svg_w = make_svg();
    draw_system(&mut svg_w, &font, &config, &wide, 0.0, 0.0).unwrap();

    // Wide system SVG should contain x2="10000" (or close) for staff lines
    let output_w = svg_w.to_svg();
    assert!(output_w.contains("x2=\"10000\""), "staff lines should span target width");
}

#[test]
fn empty_system_renders_nothing() {
    let (font, config, mcfg) = setup();
    let system = layout_system(&treble_prefix(), &[], &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    assert_eq!(output.matches("<path ").count(), 0);
    // No staff lines either since width is 0
    assert_eq!(output.matches("<line ").count(), 0);
}

#[test]
fn three_measure_system_has_correct_element_count() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Final,
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // clef(1) + 2 time digits + 3 noteheads = 6 paths
    assert_eq!(output.matches("<path ").count(), 6);
}

// --- tie rendering ---

fn tied_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { tie_forward: true, ..Default::default() },})
}

#[test]
fn tie_within_measure_draws_filled_path() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![tied_note(4), quarter_note(4)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_tied = make_svg();
    draw_system(&mut svg_tied, &font, &config, &system, 0.0, 0.0).unwrap();
    let output_tied = svg_tied.to_svg();

    // Same without tie
    let untied_measures = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(4)],
        barline: BarlineStyle::Single,
    }];
    let untied_system = layout_system(&treble_prefix(), &untied_measures, &mcfg, None);
    let mut svg_untied = make_svg();
    draw_system(&mut svg_untied, &font, &config, &untied_system, 0.0, 0.0).unwrap();
    let output_untied = svg_untied.to_svg();

    // Tied version should have one extra filled path (the tie curve)
    let tied_paths = output_tied.matches(r#"stroke="none""#).count();
    let untied_paths = output_untied.matches(r#"stroke="none""#).count();
    assert_eq!(
        tied_paths,
        untied_paths + 1,
        "tied version should have 1 more filled path (the tie)"
    );
}

#[test]
fn tie_across_barline_draws_filled_path() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![tied_note(4)],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Should contain a tie path (filled, with Bézier curves)
    let tie_paths = output.matches(r#"stroke="none""#).count();
    assert!(tie_paths >= 1, "expected at least 1 tie path, got {tie_paths}");

    // The tie path should contain Bézier curve commands
    assert!(output.contains(" C"), "tie should contain cubic Bézier curves");
}

#[test]
fn no_tie_when_tie_forward_is_false() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(4)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // No filled paths with stroke="none" (no ties)
    assert_eq!(
        output.matches(r#"stroke="none""#).count(),
        0,
        "no ties should be drawn"
    );
}

#[test]
fn tie_not_drawn_when_no_matching_target() {
    let (font, config, mcfg) = setup();
    // Tied note at pos 4, but next note is at pos 6 (different position)
    let measures = vec![MeasureContent {
        events: vec![tied_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // No tie should be drawn since the next note is at a different position
    assert_eq!(
        output.matches(r#"stroke="none""#).count(),
        0,
        "no tie when target pitch differs"
    );
}

#[test]
fn multiple_ties_in_system() {
    let (font, config, mcfg) = setup();
    // Two tied pairs: pos 4→4 and pos 6→6
    let measures = vec![MeasureContent {
        events: vec![
            tied_note(4),
            tied_note(6),
            quarter_note(4),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 2, "expected 2 ties, got {tie_count}");
}

#[test]
fn collect_note_positions_skips_non_notes() {
    let (_, _, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            quarter_note(2),
            MeasureEvent::Rest(RestEvent { duration_log2: 2, dots: 0 }),
            quarter_note(4),
        ],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let positions = collect_note_positions(&system);
    assert_eq!(positions.len(), 2, "only 2 notes, rest is skipped");
}

// --- chord tie rendering ---

use crate::layout::measure::ChordEvent;

fn tied_chord(positions: Vec<i8>) -> MeasureEvent {
    MeasureEvent::Chord(ChordEvent {
        staff_positions: positions,
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None, None],
        stem_direction: None,
        annotations: NoteAnnotations { tie_forward: true, ..Default::default() },})
}

fn untied_chord(positions: Vec<i8>) -> MeasureEvent {
    let acc_count = positions.len();
    MeasureEvent::Chord(ChordEvent {
        staff_positions: positions,
        duration_log2: 2,
        dots: 0,
        accidentals: vec![None; acc_count],
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })
}

#[test]
fn collect_note_positions_includes_chord_notes() {
    let (_, _, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            untied_chord(vec![0, 4]),
            quarter_note(2),
        ],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let positions = collect_note_positions(&system);
    // Chord expands to 2 entries + 1 single note = 3
    assert_eq!(positions.len(), 3, "chord (2 notes) + single note = 3 entries");
}

#[test]
fn chord_tie_draws_ties_for_all_notes() {
    let (font, config, mcfg) = setup();
    // Tied chord at pos [0, 4] followed by another chord at [0, 4]
    let measures = vec![MeasureContent {
        events: vec![tied_chord(vec![0, 4]), untied_chord(vec![0, 4])],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Two ties: one for pos 0 and one for pos 4
    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 2, "expected 2 ties (one per chord note), got {tie_count}");
}

#[test]
fn chord_tie_to_single_note_at_matching_position() {
    let (font, config, mcfg) = setup();
    // Tied chord [0, 4], followed by single note at pos 4
    // Only pos 4 has a target, so only 1 tie should be drawn
    let measures = vec![MeasureContent {
        events: vec![tied_chord(vec![0, 4]), quarter_note(4)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Only 1 tie: pos 4 matches, pos 0 has no target
    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 1, "expected 1 tie (only pos 4 matches), got {tie_count}");
}

#[test]
fn untied_chord_draws_no_ties() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![untied_chord(vec![0, 4]), untied_chord(vec![0, 4])],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 0, "no ties when tie_forward is false");
}

#[test]
fn chord_tie_across_barline() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![tied_chord(vec![2, 6])],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![untied_chord(vec![2, 6])],
            barline: BarlineStyle::Final,
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let tie_count = output.matches(r#"stroke="none""#).count();
    assert_eq!(tie_count, 2, "2 ties across barline (one per chord note)");
}

// --- slur rendering ---

fn slur_start_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { slur_start: true, ..Default::default() },})
}

fn slur_end_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { slur_end: true, ..Default::default() },})
}

#[test]
fn slur_within_measure_draws_filled_path() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![slur_start_note(2), quarter_note(4), slur_end_note(6)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Should have 1 filled path (the slur curve)
    let filled = output.matches(r#"stroke="none""#).count();
    assert_eq!(filled, 1, "expected 1 slur curve, got {filled}");
    // Slur should contain Bézier curves
    assert!(output.contains(" C"), "slur should contain cubic Bézier commands");
}

#[test]
fn slur_across_barline() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![slur_start_note(4)],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![slur_end_note(6)],
            barline: BarlineStyle::Final,
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let filled = output.matches(r#"stroke="none""#).count();
    assert_eq!(filled, 1, "slur across barline should draw 1 curve");
}

#[test]
fn no_slur_without_flags() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(2), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let filled = output.matches(r#"stroke="none""#).count();
    assert_eq!(filled, 0, "no slur when no slur_start/slur_end flags");
}

#[test]
fn slur_differs_from_no_slur() {
    let (font, config, mcfg) = setup();
    let with_slur = vec![MeasureContent {
        events: vec![slur_start_note(2), slur_end_note(6)],
        barline: BarlineStyle::Single,
    }];
    let without_slur = vec![MeasureContent {
        events: vec![quarter_note(2), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];

    let sys_s = layout_system(&treble_prefix(), &with_slur, &mcfg, None);
    let sys_n = layout_system(&treble_prefix(), &without_slur, &mcfg, None);

    let mut svg_s = make_svg();
    draw_system(&mut svg_s, &font, &config, &sys_s, 0.0, 0.0).unwrap();
    let mut svg_n = make_svg();
    draw_system(&mut svg_n, &font, &config, &sys_n, 0.0, 0.0).unwrap();

    assert_ne!(svg_s.to_svg(), svg_n.to_svg(), "slurred vs un-slurred should differ");
}

#[test]
fn slur_start_without_end_draws_nothing() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![slur_start_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let filled = output.matches(r#"stroke="none""#).count();
    assert_eq!(filled, 0, "no slur without matching slur_end");
}

// --- hairpin rendering ---

use crate::layout::hairpin::HairpinType;

fn cresc_start_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { hairpin_start: Some(HairpinType::Crescendo), ..Default::default() },})
}

fn hairpin_end_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations { hairpin_end: true, ..Default::default() },})
}

#[test]
fn hairpin_within_measure_draws_two_lines() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![cresc_start_note(4), quarter_note(6), hairpin_end_note(8)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_hp = make_svg();
    draw_system(&mut svg_hp, &font, &config, &system, 0.0, 0.0).unwrap();
    let output_hp = svg_hp.to_svg();

    // Without hairpin for comparison
    let measures_no = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
    }];
    let system_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &system_no, 0.0, 0.0).unwrap();
    let output_no = svg_no.to_svg();

    // Hairpin adds exactly 2 lines (wedge)
    let hp_lines = output_hp.matches("<line ").count();
    let no_lines = output_no.matches("<line ").count();
    assert_eq!(hp_lines, no_lines + 2, "hairpin should add 2 lines");
}

#[test]
fn no_hairpin_without_flags() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg1 = make_svg();
    draw_system(&mut svg1, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg1.to_svg();

    // Count lines: 5 staff + 2 stems + 1 barline = 8 (no hairpin lines)
    let line_count = output.matches("<line ").count();
    assert_eq!(line_count, 8, "no extra hairpin lines without flags");
}

#[test]
fn hairpin_start_without_end_draws_nothing_extra() {
    let (font, config, mcfg) = setup();
    let with_start = vec![MeasureContent {
        events: vec![cresc_start_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let without = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let sys_s = layout_system(&treble_prefix(), &with_start, &mcfg, None);
    let sys_n = layout_system(&treble_prefix(), &without, &mcfg, None);

    let mut svg_s = make_svg();
    draw_system(&mut svg_s, &font, &config, &sys_s, 0.0, 0.0).unwrap();
    let mut svg_n = make_svg();
    draw_system(&mut svg_n, &font, &config, &sys_n, 0.0, 0.0).unwrap();

    // Same output — hairpin_start without hairpin_end produces nothing
    assert_eq!(svg_s.to_svg(), svg_n.to_svg());
}

#[test]
fn decrescendo_differs_from_crescendo() {
    let (font, config, mcfg) = setup();
    let cresc_measures = vec![MeasureContent {
        events: vec![cresc_start_note(4), hairpin_end_note(6)],
        barline: BarlineStyle::Single,
    }];
    let decresc_measures = vec![MeasureContent {
        events: vec![
            MeasureEvent::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations { hairpin_start: Some(HairpinType::Decrescendo), ..Default::default() },}),
            hairpin_end_note(6),
        ],
        barline: BarlineStyle::Single,
    }];

    let sys_c = layout_system(&treble_prefix(), &cresc_measures, &mcfg, None);
    let sys_d = layout_system(&treble_prefix(), &decresc_measures, &mcfg, None);

    let mut svg_c = make_svg();
    draw_system(&mut svg_c, &font, &config, &sys_c, 0.0, 0.0).unwrap();
    let mut svg_d = make_svg();
    draw_system(&mut svg_d, &font, &config, &sys_d, 0.0, 0.0).unwrap();

    assert_ne!(svg_c.to_svg(), svg_d.to_svg(), "cresc and decresc should differ");
}

#[test]
fn hairpin_across_barline() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![cresc_start_note(4)],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![hairpin_end_note(6)],
            barline: BarlineStyle::Final,
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Should have hairpin lines (the wedge crosses the barline)
    let without_measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
        },
    ];
    let sys_no = layout_system(&treble_prefix(), &without_measures, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &sys_no, 0.0, 0.0).unwrap();

    let hp_lines = output.matches("<line ").count();
    let no_lines = svg_no.to_svg().matches("<line ").count();
    assert_eq!(hp_lines, no_lines + 2, "cross-barline hairpin adds 2 lines");
}

// --- lyric extender rendering ---

fn note_with_lyric(pos: i8, syl: LyricSyllable) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            lyric: Some(syl),
            ..Default::default()
        },
    })
}

#[test]
fn lyric_extender_within_measure_draws_line() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            note_with_lyric(4, LyricSyllable::with_extender("love")),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Without the extender, we'd have only staff lines + stem lines + barline.
    // The extender adds exactly one more <line> element.
    let without = {
        let m = vec![MeasureContent {
            events: vec![
                note_with_lyric(4, LyricSyllable::word("love")),
                quarter_note(6),
            ],
            barline: BarlineStyle::Single,
        }];
        let sys = layout_system(&treble_prefix(), &m, &mcfg, None);
        let mut s = make_svg();
        draw_system(&mut s, &font, &config, &sys, 0.0, 0.0).unwrap();
        s.to_svg()
    };

    let with_count = output.matches("<line ").count();
    let without_count = without.matches("<line ").count();
    assert_eq!(
        with_count,
        without_count + 1,
        "extender should add exactly 1 line element"
    );
}

#[test]
fn no_extender_for_word_continuation() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            note_with_lyric(4, LyricSyllable::word("day")),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_word = make_svg();
    draw_system(&mut svg_word, &font, &config, &system, 0.0, 0.0).unwrap();

    // Same but with no lyric at all
    let measures_none = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let sys_none = layout_system(&treble_prefix(), &measures_none, &mcfg, None);
    let mut svg_none = make_svg();
    draw_system(&mut svg_none, &font, &config, &sys_none, 0.0, 0.0).unwrap();

    // Word continuation should not add an extra line (only the text differs)
    let word_lines = svg_word.to_svg().matches("<line ").count();
    let none_lines = svg_none.to_svg().matches("<line ").count();
    assert_eq!(
        word_lines, none_lines,
        "word lyric should not add an extender line"
    );
}

#[test]
fn no_extender_for_hyphen_continuation() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            note_with_lyric(4, LyricSyllable::with_hyphen("hap")),
            note_with_lyric(6, LyricSyllable::word("py")),
        ],
        barline: BarlineStyle::Single,
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_hyp = make_svg();
    draw_system(&mut svg_hyp, &font, &config, &system, 0.0, 0.0).unwrap();

    // Compare against same notes with no lyrics
    let measures_none = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
    }];
    let sys_none = layout_system(&treble_prefix(), &measures_none, &mcfg, None);
    let mut svg_none = make_svg();
    draw_system(&mut svg_none, &font, &config, &sys_none, 0.0, 0.0).unwrap();

    let hyp_lines = svg_hyp.to_svg().matches("<line ").count();
    let none_lines = svg_none.to_svg().matches("<line ").count();
    assert_eq!(
        hyp_lines, none_lines,
        "hyphen continuation should not produce an extender line"
    );
}

#[test]
fn lyric_extender_across_barline() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![note_with_lyric(4, LyricSyllable::with_extender("love"))],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![note_with_lyric(6, LyricSyllable::word("you"))],
            barline: BarlineStyle::Final,
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();

    // Compare against same content without extender
    let measures_no = vec![
        MeasureContent {
            events: vec![note_with_lyric(4, LyricSyllable::word("love"))],
            barline: BarlineStyle::Single,
        },
        MeasureContent {
            events: vec![note_with_lyric(6, LyricSyllable::word("you"))],
            barline: BarlineStyle::Final,
        },
    ];
    let sys_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &sys_no, 0.0, 0.0).unwrap();

    let with_lines = svg.to_svg().matches("<line ").count();
    let without_lines = svg_no.to_svg().matches("<line ").count();
    assert_eq!(
        with_lines,
        without_lines + 1,
        "cross-barline extender should add 1 line"
    );
}

#[test]
fn lyric_extender_differs_from_no_extender() {
    let (font, config, mcfg) = setup();
    let with = vec![MeasureContent {
        events: vec![
            note_with_lyric(4, LyricSyllable::with_extender("ah")),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
    }];
    let without = vec![MeasureContent {
        events: vec![
            note_with_lyric(4, LyricSyllable::word("ah")),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
    }];

    let sys_w = layout_system(&treble_prefix(), &with, &mcfg, None);
    let sys_n = layout_system(&treble_prefix(), &without, &mcfg, None);

    let mut svg_w = make_svg();
    draw_system(&mut svg_w, &font, &config, &sys_w, 0.0, 0.0).unwrap();
    let mut svg_n = make_svg();
    draw_system(&mut svg_n, &font, &config, &sys_n, 0.0, 0.0).unwrap();

    assert_ne!(
        svg_w.to_svg(),
        svg_n.to_svg(),
        "extender vs no-extender should differ"
    );
}
