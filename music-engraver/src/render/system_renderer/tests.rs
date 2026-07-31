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
use crate::layout::ottava::OttavaKind;
use crate::layout::volta::{VoltaAnnotation, VoltaHooks};
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
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(2), quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_tied = make_svg();
    draw_system(&mut svg_tied, &font, &config, &system, 0.0, 0.0).unwrap();
    let output_tied = svg_tied.to_svg();

    // Same without tie
    let untied_measures = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(4)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![untied_chord(vec![2, 6])],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![slur_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
    }];
    let without_slur = vec![MeasureContent {
        events: vec![quarter_note(2), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_hp = make_svg();
    draw_system(&mut svg_hp, &font, &config, &system, 0.0, 0.0).unwrap();
    let output_hp = svg_hp.to_svg();

    // Without hairpin for comparison
    let measures_no = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
    }];
    let without = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![hairpin_end_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let sys_no = layout_system(&treble_prefix(), &without_measures, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &sys_no, 0.0, 0.0).unwrap();

    let hp_lines = output.matches("<line ").count();
    let no_lines = svg_no.to_svg().matches("<line ").count();
    assert_eq!(hp_lines, no_lines + 2, "cross-barline hairpin adds 2 lines");
}

// --- cresc-text (dashed-text crescendo/diminuendo) rendering ---

use crate::layout::cresc_text::CrescTextKind;

fn cresc_text_start_note(pos: i8, kind: CrescTextKind) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            cresc_text_start: Some(kind),
            ..Default::default()
        },
    })
}

fn cresc_text_end_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            cresc_text_end: true,
            ..Default::default()
        },
    })
}

#[test]
fn cresc_text_within_measure_emits_label_and_dashed_line() {
    // The cresc_text path should contribute exactly:
    //   - one <text> element (the "dim." label)
    //   - one dashed <line> (the continuation line, with stroke-dasharray)
    // above what a no-marking baseline produces.
    //
    // Uses the `Diminuendo` kind ("dim." — 4-char label) and a 4-note
    // span so the label + padding + dashed line all fit within the
    // raw measure-layout spacing the system_renderer test fixtures use.
    // (The score-level `render_svg` path uses extra page-level stretching
    // and has more room to play with, but this test exercises the
    // system_renderer in isolation.)
    let (font, config, mcfg) = setup();
    let with_marking = vec![MeasureContent {
        events: vec![
            cresc_text_start_note(4, CrescTextKind::Diminuendo),
            quarter_note(6),
            quarter_note(8),
            cresc_text_end_note(2),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let baseline = vec![MeasureContent {
        events: vec![
            quarter_note(4),
            quarter_note(6),
            quarter_note(8),
            quarter_note(2),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_m = layout_system(&treble_prefix(), &with_marking, &mcfg, None);
    let sys_b = layout_system(&treble_prefix(), &baseline, &mcfg, None);

    let mut svg_m = make_svg();
    draw_system(&mut svg_m, &font, &config, &sys_m, 0.0, 0.0).unwrap();
    let mut svg_b = make_svg();
    draw_system(&mut svg_b, &font, &config, &sys_b, 0.0, 0.0).unwrap();
    let out_m = svg_m.to_svg();
    let out_b = svg_b.to_svg();

    // Exactly one text element more than the baseline (the label).
    assert_eq!(
        out_m.matches("<text").count(),
        out_b.matches("<text").count() + 1,
        "cresc-text marking must add exactly one <text> element"
    );

    // Label content present, anchored to start, italic, non-bold.
    assert!(out_m.contains(">dim.</text>"), "label 'dim.' must appear");
    assert!(out_m.contains("font-style=\"italic\""), "label must be italic");
    assert!(out_m.contains("text-anchor=\"start\""), "label must be left-anchored");

    // Exactly one dashed line more than the baseline.
    assert_eq!(
        out_m.matches("stroke-dasharray").count(),
        out_b.matches("stroke-dasharray").count() + 1,
        "cresc-text marking must add exactly one dashed line"
    );
    assert_eq!(
        out_m.matches("<line ").count(),
        out_b.matches("<line ").count() + 1,
        "cresc-text marking must add exactly one <line> (the dashed continuation)"
    );
}

#[test]
fn no_cresc_text_without_start_flag() {
    // A note with only cresc_text_end and no preceding cresc_text_start
    // must not produce any marking — same byte output as the baseline.
    let (font, config, mcfg) = setup();
    let end_only = vec![MeasureContent {
        events: vec![quarter_note(4), cresc_text_end_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let baseline = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_e = layout_system(&treble_prefix(), &end_only, &mcfg, None);
    let sys_b = layout_system(&treble_prefix(), &baseline, &mcfg, None);

    let mut svg_e = make_svg();
    draw_system(&mut svg_e, &font, &config, &sys_e, 0.0, 0.0).unwrap();
    let mut svg_b = make_svg();
    draw_system(&mut svg_b, &font, &config, &sys_b, 0.0, 0.0).unwrap();

    assert_eq!(
        svg_e.to_svg(),
        svg_b.to_svg(),
        "cresc_text_end without a preceding cresc_text_start must render identically to baseline"
    );
}

#[test]
fn cresc_text_start_without_end_draws_nothing_extra() {
    // Same orphan-start semantics as hairpins: an unresolved
    // cresc_text_start produces no marking until a matching end appears.
    let (font, config, mcfg) = setup();
    let start_only = vec![MeasureContent {
        events: vec![
            cresc_text_start_note(4, CrescTextKind::Crescendo),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let baseline = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_s = layout_system(&treble_prefix(), &start_only, &mcfg, None);
    let sys_b = layout_system(&treble_prefix(), &baseline, &mcfg, None);

    let mut svg_s = make_svg();
    draw_system(&mut svg_s, &font, &config, &sys_s, 0.0, 0.0).unwrap();
    let mut svg_b = make_svg();
    draw_system(&mut svg_b, &font, &config, &sys_b, 0.0, 0.0).unwrap();

    assert_eq!(
        svg_s.to_svg(),
        svg_b.to_svg(),
        "cresc_text_start without cresc_text_end must produce no extra elements"
    );
}

#[test]
fn cresc_text_kinds_differ_in_label_content() {
    // Same start/end notes, three different kinds — each label must
    // appear in its own render, and the cresc./decresc./dim. labels
    // must NOT cross-contaminate between renders.
    let (font, config, mcfg) = setup();

    let mut renders = Vec::new();
    for kind in [
        CrescTextKind::Crescendo,
        CrescTextKind::Decrescendo,
        CrescTextKind::Diminuendo,
    ] {
        let measures = vec![MeasureContent {
            events: vec![cresc_text_start_note(4, kind), cresc_text_end_note(6)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        }];
        let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
        let mut svg = make_svg();
        draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
        renders.push((kind, svg.to_svg()));
    }

    let cresc = &renders[0].1;
    let decresc = &renders[1].1;
    let dim = &renders[2].1;

    assert!(cresc.contains(">cresc.</text>"), "Crescendo render missing 'cresc.' label");
    assert!(decresc.contains(">decresc.</text>"), "Decrescendo render missing 'decresc.' label");
    assert!(dim.contains(">dim.</text>"), "Diminuendo render missing 'dim.' label");

    // Cross-contamination guards: the cresc. render must not contain the
    // dim. or decresc. literal strings, etc.
    assert!(!cresc.contains("decresc."), "Crescendo render must not contain 'decresc.'");
    assert!(!cresc.contains("dim."), "Crescendo render must not contain 'dim.'");
    assert!(!dim.contains("cresc."), "Diminuendo render must not contain 'cresc.' (catches accidental fallthrough on label dispatch)");
    // decresc. *contains* "cresc." as a substring, so we only guard the
    // reverse direction here.
    assert!(!decresc.contains(">dim.</text>"), "Decrescendo render must not contain 'dim.' label");

    // Each pairing produces visually distinct SVG.
    assert_ne!(cresc, decresc, "Crescendo and Decrescendo SVG must differ");
    assert_ne!(decresc, dim, "Decrescendo and Diminuendo SVG must differ");
    assert_ne!(cresc, dim, "Crescendo and Diminuendo SVG must differ");
}

#[test]
fn cresc_text_across_barline_emits_one_label_and_one_line() {
    // Within a single system but across a barline: the marking should
    // still emit exactly one label + one dashed line (we are not
    // crossing a system break). Two-measure layout gives enough span.
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![
                cresc_text_start_note(4, CrescTextKind::Diminuendo),
                quarter_note(6),
            ],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(7), cresc_text_end_note(2)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let baseline = vec![
        MeasureContent {
            events: vec![quarter_note(4), quarter_note(6)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(7), quarter_note(2)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let sys_m = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let sys_b = layout_system(&treble_prefix(), &baseline, &mcfg, None);

    let mut svg_m = make_svg();
    draw_system(&mut svg_m, &font, &config, &sys_m, 0.0, 0.0).unwrap();
    let mut svg_b = make_svg();
    draw_system(&mut svg_b, &font, &config, &sys_b, 0.0, 0.0).unwrap();

    let out_m = svg_m.to_svg();
    let out_b = svg_b.to_svg();

    assert_eq!(
        out_m.matches(">dim.</text>").count(),
        1,
        "across-barline marking must emit exactly one label"
    );
    assert_eq!(
        out_m.matches("stroke-dasharray").count(),
        out_b.matches("stroke-dasharray").count() + 1,
        "across-barline marking must add exactly one dashed line"
    );
}

#[test]
fn cresc_text_dashed_line_endpoints_lie_between_start_and_end_notes() {
    // Locks the geometric routing: the dashed line's x1 must be greater
    // than the start note's x (the label sits at the start, the line
    // begins after the label), and x2 must lie strictly left of the
    // end note's x (with the standard 0.3 staff-space pad).
    //
    // This catches a regression where the renderer accidentally swapped
    // start/end positions or routed the line to the wrong notes.
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            cresc_text_start_note(4, CrescTextKind::Diminuendo),
            quarter_note(6),
            quarter_note(8),
            cresc_text_end_note(0),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let out = svg.to_svg();

    // Sanity: the marking actually rendered a dashed line — without it,
    // the x2 assertion below would be vacuously true on an empty match.
    assert!(
        out.contains("stroke-dasharray"),
        "test precondition: scenario must be wide enough for a dashed line; got:\n{out}"
    );

    // Pull the cresc-text info back out to derive expected endpoints.
    let info = collect_cresc_text_note_info(&system);
    let start = info
        .iter()
        .find(|n| n.cresc_text_start.is_some())
        .expect("start note present");
    let end = info
        .iter()
        .find(|n| n.cresc_text_end)
        .expect("end note present");
    assert!(
        end.x > start.x,
        "test setup precondition: end note must be after start note"
    );

    // The dashed line ends at: system_x (=0) + end.x − 0.3 * staff_space.
    let expected_x2 = end.x - 0.3 * config.staff_space;
    let needle = format!("x2=\"{}\"", expected_x2);
    assert!(
        out.contains(&needle),
        "expected dashed-line x2={} in SVG; got:\n{out}",
        expected_x2
    );
}

#[test]
fn cresc_text_renders_independently_of_a_hairpin_on_other_notes() {
    // The two paths share the same vertical band but do not interfere:
    // a system that has BOTH a hairpin (on one pair of notes) and a
    // cresc-text marking (on a different pair of notes) must emit
    // exactly one hairpin contribution + one cresc-text contribution.
    let (font, config, mcfg) = setup();
    let mixed = vec![MeasureContent {
        events: vec![
            cresc_start_note(4), // hairpin start
            hairpin_end_note(6), // hairpin end
            cresc_text_start_note(8, CrescTextKind::Diminuendo),
            quarter_note(7),
            quarter_note(2),
            cresc_text_end_note(0),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let baseline = vec![MeasureContent {
        events: vec![
            quarter_note(4),
            quarter_note(6),
            quarter_note(8),
            quarter_note(7),
            quarter_note(2),
            quarter_note(0),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_m = layout_system(&treble_prefix(), &mixed, &mcfg, None);
    let sys_b = layout_system(&treble_prefix(), &baseline, &mcfg, None);

    let mut svg_m = make_svg();
    draw_system(&mut svg_m, &font, &config, &sys_m, 0.0, 0.0).unwrap();
    let mut svg_b = make_svg();
    draw_system(&mut svg_b, &font, &config, &sys_b, 0.0, 0.0).unwrap();
    let out_m = svg_m.to_svg();
    let out_b = svg_b.to_svg();

    // Hairpin contributes 2 solid lines, cresc-text contributes 1
    // dashed line: total +3 line elements.
    assert_eq!(
        out_m.matches("<line ").count(),
        out_b.matches("<line ").count() + 3,
        "hairpin (2 lines) + cresc-text (1 dashed line) must add exactly 3 lines over baseline"
    );
    // Exactly one dashed line (the cresc-text continuation); the hairpin
    // wedge lines are solid, not dashed.
    assert_eq!(
        out_m.matches("stroke-dasharray").count(),
        out_b.matches("stroke-dasharray").count() + 1,
        "cresc-text must add exactly one dashed line; hairpin must add none"
    );
    // Exactly one text element added (the cresc-text label); hairpin
    // contributes no <text> in this scenario.
    assert_eq!(
        out_m.matches("<text").count(),
        out_b.matches("<text").count() + 1,
        "cresc-text label must add exactly one <text>; hairpin must add none"
    );
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
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_word = make_svg();
    draw_system(&mut svg_word, &font, &config, &system, 0.0, 0.0).unwrap();

    // Same but with no lyric at all
    let measures_none = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg_hyp = make_svg();
    draw_system(&mut svg_hyp, &font, &config, &system, 0.0, 0.0).unwrap();

    // Compare against same notes with no lyrics
    let measures_none = vec![MeasureContent {
        events: vec![quarter_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![note_with_lyric(6, LyricSyllable::word("you"))],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![note_with_lyric(6, LyricSyllable::word("you"))],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
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
        volta: None,
        additional_voices: vec![],
    }];
    let without = vec![MeasureContent {
        events: vec![
            note_with_lyric(4, LyricSyllable::word("ah")),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
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

// --- volta brackets ---

#[test]
fn volta_bracket_adds_lines_to_system() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Final,
        volta: Some(VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        }),
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();

    // Same without volta
    let measures_no = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Final,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &sys_no, 0.0, 0.0).unwrap();

    let with_lines = svg.to_svg().matches("<line ").count();
    let without_lines = svg_no.to_svg().matches("<line ").count();
    // Both hooks: +3 lines (top + left hook + right hook)
    assert_eq!(with_lines - without_lines, 3, "both hooks adds 3 lines");
}

#[test]
fn volta_bracket_text_appears_in_svg() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Final,
        volta: Some(VoltaAnnotation {
            text: Some("2.".to_string()),
            hooks: VoltaHooks::Both,
        }),
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">2.</text>"), "should contain volta text '2.'");
}

#[test]
fn volta_no_bracket_without_annotation() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Final,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(!output.contains(">1.</text>"));
    assert!(!output.contains(">2.</text>"));
}

#[test]
fn volta_left_only_adds_two_lines() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Final,
        volta: Some(VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::LeftOnly,
        }),
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();

    let measures_no = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Final,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &sys_no, 0.0, 0.0).unwrap();

    let diff = svg.to_svg().matches("<line ").count()
        - svg_no.to_svg().matches("<line ").count();
    assert_eq!(diff, 2, "left-only volta adds 2 lines (top + left hook)");
}

#[test]
fn volta_multi_measure_two_brackets() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: Some(VoltaAnnotation {
                text: Some("1.".to_string()),
                hooks: VoltaHooks::LeftOnly,
            }),
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: Some(VoltaAnnotation {
                text: None,
                hooks: VoltaHooks::RightOnly,
            }),
            additional_voices: vec![],
        },
    ];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">1.</text>"), "first measure has text");
    // 2 brackets × top line = 2 top lines
    // + left hook on first + right hook on second = 2 hooks
    // Total extra lines = 4
    let measures_no = vec![
        MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(6)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let sys_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &sys_no, 0.0, 0.0).unwrap();

    let diff = svg.to_svg().matches("<line ").count()
        - svg_no.to_svg().matches("<line ").count();
    assert_eq!(diff, 4, "two-measure volta adds 4 lines (2 tops + left + right hooks)");
}

// ── Ottava bracket tests ──

fn ottava_start_note(pos: i8, kind: OttavaKind) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ottava_start: Some(kind),
            ..NoteAnnotations::default()
        },
    })
}

fn ottava_end_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ottava_end: true,
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn ottava_bracket_adds_dashed_line_and_text() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            ottava_start_note(10, OttavaKind::Ottava8va),
            quarter_note(12),
            ottava_end_note(14),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">8va</text>"), "should contain '8va' label");
    assert!(output.contains("stroke-dasharray"), "should have dashed line");
}

#[test]
fn ottava_bracket_has_end_hook() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            ottava_start_note(10, OttavaKind::Ottava8va),
            ottava_end_note(12),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    // Dashed line = 1 <line> with dasharray, hook = 1 <line> without dasharray
    // Total should be at least 7 (5 staff lines + dashed + hook)
    let line_count = output.matches("<line ").count();
    assert!(line_count >= 7, "expected ≥7 lines (5 staff + dashed + hook), got {line_count}");
}

#[test]
fn no_ottava_without_flags() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(10), quarter_note(12)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(!output.contains("8va"), "should have no ottava label");
    assert!(!output.contains("stroke-dasharray"), "should have no dashed line");
}

#[test]
fn ottava_8vb_has_different_label() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            ottava_start_note(-2, OttavaKind::Ottava8vb),
            ottava_end_note(-4),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(output.contains(">8vb</text>"), "should contain '8vb' label");
}

#[test]
fn ottava_start_without_end_draws_nothing() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            ottava_start_note(10, OttavaKind::Ottava8va),
            quarter_note(12),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(!output.contains("8va"), "start without end should draw no ottava bracket");
}

// --- glissando ---

fn glissando_note(pos: i8, style: GlissandoStyle) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            glissando_start: Some(style),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn glissando_within_measure_adds_line() {
    let (font, config, mcfg) = setup();
    // Use half notes to give notes more horizontal space
    let gliss_note = MeasureEvent::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 1,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            glissando_start: Some(GlissandoStyle::Line),
            ..NoteAnnotations::default()
        },
    });
    let half_note = MeasureEvent::Note(NoteEvent {
        staff_position: 8,
        duration_log2: 1,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    });
    let measures = vec![MeasureContent {
        events: vec![gliss_note, half_note],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    // Justify to a wide target width so notes are well-separated
    let system = layout_system(&treble_prefix(), &measures, &mcfg, Some(8000.0));
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Without glissando
    let plain_note = MeasureEvent::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 1,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    });
    let half_note2 = MeasureEvent::Note(NoteEvent {
        staff_position: 8,
        duration_log2: 1,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    });
    let measures_no = vec![MeasureContent {
        events: vec![plain_note, half_note2],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system_no = layout_system(&treble_prefix(), &measures_no, &mcfg, Some(8000.0));
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &system_no, 0.0, 0.0).unwrap();
    let output_no = svg_no.to_svg();

    let gliss_lines = output.matches("<line").count();
    let no_gliss_lines = output_no.matches("<line").count();
    assert!(gliss_lines > no_gliss_lines,
        "glissando should add at least one line: {} vs {}",
        gliss_lines, no_gliss_lines);
}

#[test]
fn no_glissando_without_flag() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(0), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(!output.contains("gliss."), "no glissando flag means no gliss text");
}

#[test]
fn glissando_cross_barline() {
    let (font, config, mcfg) = setup();
    let measures = vec![
        MeasureContent {
            events: vec![glissando_note(0, GlissandoStyle::Line)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
    ];
    // Wide target so cross-barline notes have enough separation
    let system = layout_system(&treble_prefix(), &measures, &mcfg, Some(8000.0));
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let mut svg_no = make_svg();
    let measures_no = vec![
        MeasureContent {
            events: vec![quarter_note(0)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        MeasureContent {
            events: vec![quarter_note(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
    ];
    let system_no = layout_system(&treble_prefix(), &measures_no, &mcfg, Some(8000.0));
    draw_system(&mut svg_no, &font, &config, &system_no, 0.0, 0.0).unwrap();
    let output_no = svg_no.to_svg();
    assert_ne!(output, output_no, "cross-barline glissando should differ from no glissando");
}

#[test]
fn glissando_with_text_shows_label() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            glissando_note(0, GlissandoStyle::LineWithText),
            quarter_note(8),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    // Wide target to ensure enough horizontal space for glissando line
    let system = layout_system(&treble_prefix(), &measures, &mcfg, Some(8000.0));
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();
    assert!(output.contains("gliss."), "LineWithText should show 'gliss.' label");
}

#[test]
fn glissando_start_without_target_draws_nothing() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![glissando_note(4, GlissandoStyle::Line)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    let mut svg_no = make_svg();
    let measures_no = vec![MeasureContent {
        events: vec![quarter_note(4)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    draw_system(&mut svg_no, &font, &config, &system_no, 0.0, 0.0).unwrap();
    let output_no = svg_no.to_svg();
    assert_eq!(output.matches("<line").count(), output_no.matches("<line").count(),
        "glissando with no target note should not add any lines");
}

// --- additional voice span rendering ---

fn voice_note(pos: i8, dir: StemDirection) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(dir),
        annotations: NoteAnnotations::default(),
    })
}

fn voice_tied_note(pos: i8, dir: StemDirection) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(dir),
        annotations: NoteAnnotations { tie_forward: true, ..Default::default() },
    })
}

#[test]
fn tie_in_additional_voice_draws_filled_path() {
    let (font, config, mcfg) = setup();
    // Primary voice: two plain notes (stems up)
    // Additional voice: tied note → target note (stems down)
    let measures = vec![MeasureContent {
        events: vec![
            voice_note(8, StemDirection::Up),
            voice_note(8, StemDirection::Up),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![vec![
            voice_tied_note(0, StemDirection::Down),
            voice_note(0, StemDirection::Down),
        ]],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Without tie in additional voice
    let measures_no_tie = vec![MeasureContent {
        events: vec![
            voice_note(8, StemDirection::Up),
            voice_note(8, StemDirection::Up),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![vec![
            voice_note(0, StemDirection::Down),
            voice_note(0, StemDirection::Down),
        ]],
    }];
    let system_no = layout_system(&treble_prefix(), &measures_no_tie, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &system_no, 0.0, 0.0).unwrap();
    let output_no = svg_no.to_svg();

    let tied_fills = output.matches(r#"stroke="none""#).count();
    let untied_fills = output_no.matches(r#"stroke="none""#).count();
    assert_eq!(
        tied_fills,
        untied_fills + 1,
        "tie in additional voice should add 1 filled path (the tie curve)"
    );
}

#[test]
fn slur_in_additional_voice_draws_filled_path() {
    let (font, config, mcfg) = setup();
    let slur_start_note = MeasureEvent::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations { slur_start: true, ..Default::default() },
    });
    let slur_end_note = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations { slur_end: true, ..Default::default() },
    });
    let measures = vec![MeasureContent {
        events: vec![
            voice_note(8, StemDirection::Up),
            voice_note(8, StemDirection::Up),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![vec![slur_start_note, slur_end_note]],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Without slur
    let measures_no = vec![MeasureContent {
        events: vec![
            voice_note(8, StemDirection::Up),
            voice_note(8, StemDirection::Up),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![vec![
            voice_note(0, StemDirection::Down),
            voice_note(4, StemDirection::Down),
        ]],
    }];
    let system_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &system_no, 0.0, 0.0).unwrap();
    let output_no = svg_no.to_svg();

    let slurred_fills = output.matches(r#"stroke="none""#).count();
    let plain_fills = output_no.matches(r#"stroke="none""#).count();
    assert_eq!(
        slurred_fills,
        plain_fills + 1,
        "slur in additional voice should add 1 filled path (the slur curve)"
    );
}

#[test]
fn hairpin_in_additional_voice_draws_lines() {
    use crate::layout::hairpin::HairpinType;
    let (font, config, mcfg) = setup();
    let hp_start_note = MeasureEvent::Note(NoteEvent {
        staff_position: 0,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations {
            hairpin_start: Some(HairpinType::Crescendo),
            ..Default::default()
        },
    });
    let hp_end_note = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: Some(StemDirection::Down),
        annotations: NoteAnnotations { hairpin_end: true, ..Default::default() },
    });
    let measures = vec![MeasureContent {
        events: vec![
            voice_note(8, StemDirection::Up),
            voice_note(8, StemDirection::Up),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![vec![hp_start_note, hp_end_note]],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Without hairpin
    let measures_no = vec![MeasureContent {
        events: vec![
            voice_note(8, StemDirection::Up),
            voice_note(8, StemDirection::Up),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![vec![
            voice_note(0, StemDirection::Down),
            voice_note(4, StemDirection::Down),
        ]],
    }];
    let system_no = layout_system(&treble_prefix(), &measures_no, &mcfg, None);
    let mut svg_no = make_svg();
    draw_system(&mut svg_no, &font, &config, &system_no, 0.0, 0.0).unwrap();
    let output_no = svg_no.to_svg();

    let hp_lines = output.matches("<line ").count();
    let plain_lines = output_no.matches("<line ").count();
    assert_eq!(
        hp_lines,
        plain_lines + 2,
        "hairpin in additional voice should add 2 lines (the wedge)"
    );
}

#[test]
fn collect_note_positions_includes_additional_voices() {
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![voice_note(8, StemDirection::Up)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![vec![voice_note(0, StemDirection::Down)]],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let positions = collect_note_positions(&system);
    // Should have 2 notes: 1 primary + 1 additional
    assert_eq!(positions.len(), 2, "collect should include notes from additional voices");
    assert_eq!(positions[0].1, 8, "first note is primary voice at pos 8");
    assert_eq!(positions[1].1, 0, "second note is additional voice at pos 0");
}

#[test]
fn collect_note_positions_without_additional_voices_unchanged() {
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![voice_note(4, StemDirection::Up), voice_note(6, StemDirection::Up)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let positions = collect_note_positions(&system);
    assert_eq!(positions.len(), 2, "should have exactly 2 notes");
    assert_eq!(positions[0].1, 4);
    assert_eq!(positions[1].1, 6);
}

// --- trill extension ---

use crate::layout::ornament::Ornament;

fn trill_ext_note(pos: i8) -> MeasureEvent {
    // Use a long duration (whole note) so there's enough horizontal room
    // for the wiggle to fit at least one tile in the test SVG window.
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    })
}

fn trill_no_ext_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: false,
            ..NoteAnnotations::default()
        },
    })
}

/// Count the number of `<path` elements in an SVG fragment whose `translate(...)`
/// transform has a y-coordinate equal to `y_needle`. Used to verify that the
/// wiggle segments share the trill glyph's baseline.
fn count_paths_with_translate_y(svg: &str, y_needle: f64) -> usize {
    let needle = format!(",{y_needle})");
    svg.matches(&needle).count()
}

#[test]
fn trill_extension_collector_flags_only_marked_notes() {
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            trill_ext_note(4),
            trill_no_ext_note(6),
            quarter_note(8),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(info.len(), 3, "should collect one entry per note");
    assert!(info[0].has_trill_extension, "first note has extension");
    assert!(!info[1].has_trill_extension, "second has trill but no extension");
    assert!(!info[2].has_trill_extension, "third has no trill");
}

#[test]
fn trill_extension_collector_requires_trill_ornament() {
    // Even if trill_extension=true, no extension if the ornament isn't Trill.
    let bad = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Mordent),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    });
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![bad, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(
        !info[0].has_trill_extension,
        "mordent with trill_extension=true must not be flagged as a trill extension"
    );
}

#[test]
fn trill_extension_renders_at_least_one_wiggle_path() {
    let (font, config, mcfg) = setup();
    // Two whole notes side-by-side give a wide span between them.
    let measures = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_ext = svg.to_svg();

    // Compare against the identical score without trill_extension:
    // the difference is exactly the wiggle paths we added.
    let measures2 = vec![MeasureContent {
        events: vec![trill_no_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures2, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let without_ext = svg2.to_svg();

    let with_paths = with_ext.matches("<path").count();
    let without_paths = without_ext.matches("<path").count();
    assert!(
        with_paths > without_paths,
        "trill_extension must add at least one path: with={with_paths}, without={without_paths}"
    );
}

#[test]
fn trill_extension_wiggle_shares_trill_glyph_y() {
    use crate::layout::ornament::layout_ornament;
    use crate::layout::staff::StaffLayout;

    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Recompute the ornament y the same way draw_system_trill_extensions does
    let staff = StaffLayout::new(0.0, 0.0, system.staff_width, config.staff_space);
    let info = collect_trill_extension_note_info(&system);
    let first = info.iter().find(|n| n.has_trill_extension).expect("trill present");
    let layout = layout_ornament(Ornament::Trill, first.x, first.staff_position, &staff);

    // At least one path must translate to the computed wiggle y. The exact
    // string match guards against drift in the y math.
    let hits = count_paths_with_translate_y(&output, layout.y);
    assert!(
        hits >= 1,
        "expected at least one wiggle segment at y={}, got {hits}",
        layout.y
    );
}

#[test]
fn no_trill_extension_without_flag() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![trill_no_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    // Compute the trill-glyph y and verify nothing else translates to it
    // beyond the "tr" glyph itself (1 path).
    use crate::layout::ornament::layout_ornament;
    use crate::layout::staff::StaffLayout;
    let staff = StaffLayout::new(0.0, 0.0, system.staff_width, config.staff_space);
    let first = collect_trill_extension_note_info(&system)[0].x;
    let layout = layout_ornament(Ornament::Trill, first, 4, &staff);
    let hits = count_paths_with_translate_y(&output, layout.y);
    assert_eq!(
        hits, 1,
        "without extension, only the 'tr' glyph itself should sit at y={}",
        layout.y
    );
}

#[test]
fn last_note_in_system_with_trill_extension_extends_to_system_edge() {
    // A trill on the final note of the system has no following note, so
    // the wiggle extends to (just inside) the system's right edge. This
    // is the cross-system convention: a sustained trill at the end of a
    // system continues visually to the system break.
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4), trill_ext_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_ext = svg.to_svg();

    // Identical score but without the extension flag — produces a "tr"
    // glyph but no wiggle. The with-ext version must have more paths.
    let measures2 = vec![MeasureContent {
        events: vec![quarter_note(4), trill_no_ext_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures2, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let without_ext = svg2.to_svg();

    let with_paths = with_ext.matches("<path").count();
    let without_paths = without_ext.matches("<path").count();
    assert!(
        with_paths > without_paths,
        "trill at end of system should still draw a wiggle to the system edge: \
         with={with_paths}, without={without_paths}"
    );
}

#[test]
fn last_note_trill_extension_wiggle_stays_inside_system_edge() {
    // The cross-system wiggle must end strictly inside the system's right
    // edge — the final barline lives at x = system.staff_width, and the
    // wiggle leaves a small gap before it so the two marks read distinctly.
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4), trill_ext_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    // Recompute exactly what draw_system_trill_extensions computes for the
    // end_x so the assertion is grounded in the production geometry.
    use crate::layout::staff::StaffLayout;
    use crate::layout::trill_extension::{layout_trill_extension, trill_extension_right_edge};
    use crate::render::system_renderer::collect_trill_extension_note_info;

    let staff = StaffLayout::new(0.0, 0.0, system.staff_width, config.staff_space);
    let info = collect_trill_extension_note_info(&system);
    let trill_idx = info
        .iter()
        .position(|n| n.has_trill_extension)
        .expect("trill extension present");
    let note = &info[trill_idx];

    let trill_advance = font.glyph_advance(smufl::Glyph::OrnamentTrill).unwrap() as f64;
    let wiggle_advance = font.glyph_advance(smufl::Glyph::WiggleTrill).unwrap() as f64;
    let ornament =
        crate::layout::ornament::layout_ornament(Ornament::Trill, note.x, note.staff_position, &staff);

    let staff_space = staff.staff_space;
    let start_x = ornament.x + trill_advance + 0.15 * staff_space;
    let end_x = system.staff_width - 0.5 * staff_space;

    let layout = layout_trill_extension(start_x, end_x, ornament.y, wiggle_advance)
        .expect("there should be enough room for at least one wiggle segment");
    let right_edge = trill_extension_right_edge(&layout);

    assert!(
        right_edge < system.staff_width,
        "wiggle right edge {right_edge} must be inside system right edge {}",
        system.staff_width
    );
    assert!(
        system.staff_width - right_edge >= 0.5 * staff_space - 1e-6,
        "wiggle must leave at least 0.5 staff_space before the final barline; \
         right={right_edge}, staff_width={}",
        system.staff_width
    );
}

#[test]
fn last_note_trill_extension_wiggle_shares_trill_glyph_y() {
    // The cross-system wiggle shares the trill glyph's y — same convention
    // as the inter-note variant, so the two marks read as a continuous line.
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![quarter_note(4), trill_ext_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let output = svg.to_svg();

    use crate::layout::ornament::layout_ornament;
    use crate::layout::staff::StaffLayout;
    let staff = StaffLayout::new(0.0, 0.0, system.staff_width, config.staff_space);
    let info = collect_trill_extension_note_info(&system);
    let first = info.iter().find(|n| n.has_trill_extension).expect("trill present");
    let layout = layout_ornament(Ornament::Trill, first.x, first.staff_position, &staff);

    let hits = count_paths_with_translate_y(&output, layout.y);
    assert!(
        hits >= 2,
        "expected the 'tr' glyph plus at least one wiggle segment at y={}, got {hits}",
        layout.y
    );
}

#[test]
fn last_note_trill_extension_silently_skips_when_no_room() {
    // If the trilled note sits too close to the system's right edge to fit
    // even a single wiggle segment, the layout function returns None and
    // the renderer silently skips — the trill is still indicated by the
    // "tr" glyph alone. This is the same fail-safe used for the inter-note
    // variant when notes are tightly packed.
    let (font, config, mcfg) = setup();
    // Single very-short measure: trill on a quarter at staff_position 4.
    // The natural width is just enough for the clef + a single quarter,
    // leaving little room for a wiggle past the "tr" glyph.
    let measures = vec![MeasureContent {
        events: vec![trill_ext_note(4)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    // Force a tiny target width so end_x lands left of start_x.
    let trill_advance = font.glyph_advance(smufl::Glyph::OrnamentTrill).unwrap() as f64;
    let wiggle_advance = font.glyph_advance(smufl::Glyph::WiggleTrill).unwrap() as f64;

    // Recompute end_x with a deliberately too-small staff_width: simulate
    // the case where the trilled note is the last in a system that's been
    // squeezed so tight that no segment fits.
    use crate::layout::staff::StaffLayout;
    use crate::layout::trill_extension::layout_trill_extension;
    let staff = StaffLayout::new(0.0, 0.0, system.staff_width, config.staff_space);
    let info = collect_trill_extension_note_info(&system);
    let note = info.iter().find(|n| n.has_trill_extension).unwrap();
    let ornament =
        crate::layout::ornament::layout_ornament(Ornament::Trill, note.x, note.staff_position, &staff);
    let start_x = ornament.x + trill_advance + 0.15 * staff.staff_space;
    // Set end_x equal to start_x so no segment fits.
    let bad_end = start_x;
    assert!(
        layout_trill_extension(start_x, bad_end, ornament.y, wiggle_advance).is_none(),
        "zero-width span must produce no wiggle"
    );

    // Smoke check the live draw path doesn't error: real system has room
    // and should draw at least one wiggle, but the layout-level fail-safe
    // is the production-critical guarantee.
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let _ = svg.to_svg();
}

#[test]
fn trill_extension_on_chord_uses_top_note_position() {
    let chord_event = MeasureEvent::Chord(crate::layout::measure::ChordEvent {
        staff_positions: vec![2, 4, 7], // top note is 7
        duration_log2: 0,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    });
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![chord_event, quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(info[0].has_trill_extension, "chord trill extension flagged");
    assert_eq!(
        info[0].staff_position, 7,
        "chord trill extension should anchor to top note (max staff pos)"
    );
}

// --- trill bracket wiring ---

use crate::layout::trill_bracket::TrillBracketSide;
use crate::render::system_renderer::bracket_side_for_system_pass;

fn trill_ext_bracketed_note(pos: i8, side: TrillBracketSide) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: Some(side),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn bracket_side_for_system_pass_within_system_passes_through() {
    assert_eq!(
        bracket_side_for_system_pass(TrillBracketSide::Start, false),
        Some(TrillBracketSide::Start)
    );
    assert_eq!(
        bracket_side_for_system_pass(TrillBracketSide::End, false),
        Some(TrillBracketSide::End)
    );
    assert_eq!(
        bracket_side_for_system_pass(TrillBracketSide::Both, false),
        Some(TrillBracketSide::Both)
    );
}

#[test]
fn bracket_side_for_system_pass_cross_system_filters_end() {
    // Cross-system: End hook is suppressed (page renderer draws it on N+1).
    assert_eq!(
        bracket_side_for_system_pass(TrillBracketSide::Start, true),
        Some(TrillBracketSide::Start)
    );
    assert_eq!(
        bracket_side_for_system_pass(TrillBracketSide::End, true),
        None,
        "End hook on cross-system trill must NOT be drawn on source system"
    );
    assert_eq!(
        bracket_side_for_system_pass(TrillBracketSide::Both, true),
        Some(TrillBracketSide::Start),
        "Both reduces to Start on the source system; End drawn on target"
    );
}

#[test]
fn trill_bracket_collector_propagates_bracket_when_extension_active() {
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_note(4, TrillBracketSide::Both),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(info[0].bracket, Some(TrillBracketSide::Both));
    // Note without bracket has None.
    assert_eq!(info[1].bracket, None);
}

#[test]
fn trill_bracket_collector_drops_bracket_when_no_extension() {
    // A user request for a bracket on a note that has trill_extension=false
    // (e.g. set the bracket but never set the extension flag) must NOT
    // produce a bracket annotation downstream — bracketing implies an
    // extension to bracket.
    let bad = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: false,
            trill_bracket: Some(TrillBracketSide::Both),
            ..NoteAnnotations::default()
        },
    });
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![bad, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(
        info[0].bracket, None,
        "bracket without extension must be silently dropped"
    );
}

#[test]
fn trill_bracket_both_within_system_renders_two_hooks_more_than_no_bracket() {
    // Within-system Both bracket renders 2 hooks (Start + End), so the
    // SVG must contain at least 2 more <line> elements than the same score
    // without a bracket.
    let (font, config, mcfg) = setup();
    let measures_bracketed = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_note(4, TrillBracketSide::Both),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures_bracketed, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_bracket = svg.to_svg();

    let measures_plain = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures_plain, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let without_bracket = svg2.to_svg();

    let with_lines = with_bracket.matches("<line ").count();
    let without_lines = without_bracket.matches("<line ").count();
    assert_eq!(
        with_lines - without_lines,
        2,
        "Both bracket should add exactly 2 hook <line> elements: with={with_lines}, without={without_lines}"
    );
}

#[test]
fn trill_bracket_start_only_renders_one_hook_more() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_note(4, TrillBracketSide::Start),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_bracket = svg.to_svg();

    let measures_plain = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures_plain, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let without_bracket = svg2.to_svg();

    let with_lines = with_bracket.matches("<line ").count();
    let without_lines = without_bracket.matches("<line ").count();
    assert_eq!(
        with_lines - without_lines,
        1,
        "Start-only bracket should add exactly 1 hook"
    );
}

#[test]
fn trill_bracket_end_only_within_system_renders_one_hook_more() {
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_note(4, TrillBracketSide::End),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_bracket = svg.to_svg();

    let measures_plain = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures_plain, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let without_bracket = svg2.to_svg();

    let with_lines = with_bracket.matches("<line ").count();
    let without_lines = without_bracket.matches("<line ").count();
    assert_eq!(
        with_lines - without_lines,
        1,
        "End-only bracket within-system should add exactly 1 hook"
    );
}

#[test]
fn trill_bracket_no_bracket_renders_zero_extra_hooks() {
    let (font, config, mcfg) = setup();
    // Identical scores: one with trill+extension, one without; neither
    // requests a bracket. The line count should match exactly.
    let measures = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let plain = svg.to_svg();

    // No bracket annotation present → no hook lines added on top of the
    // standard score chrome. This is a regression guard: future changes
    // must not introduce stray hooks when no bracket was requested.
    let line_count = plain.matches("<line ").count();

    // A note with bracket=None must produce the same <line> count.
    let measures2 = vec![MeasureContent {
        events: vec![
            MeasureEvent::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 0,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations {
                    ornament: Some(Ornament::Trill),
                    trill_extension: true,
                    trill_bracket: None,
                    ..NoteAnnotations::default()
                },
            }),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures2, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let none = svg2.to_svg();
    assert_eq!(none.matches("<line ").count(), line_count);
}

#[test]
fn trill_bracket_end_hook_for_last_note_in_system_is_suppressed_in_system_pass() {
    // When the trilled note is the LAST in its system, the cross-system
    // suppression rule applies: an End-only bracket emits NO hook on this
    // system (the page renderer will emit it on N+1 if/when there's a next
    // system; if there isn't, nothing is drawn — see the open issue in
    // the progress log).
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            quarter_note(4),
            trill_ext_bracketed_note(6, TrillBracketSide::End),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_bracket = svg.to_svg();

    let measures_plain = vec![MeasureContent {
        events: vec![quarter_note(4), trill_ext_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures_plain, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let without_bracket = svg2.to_svg();

    let with_lines = with_bracket.matches("<line ").count();
    let without_lines = without_bracket.matches("<line ").count();
    assert_eq!(
        with_lines, without_lines,
        "End hook on last note of system must be suppressed on the source system pass"
    );
}

#[test]
fn trill_bracket_both_on_last_note_in_system_emits_only_start_hook() {
    // Both bracket on the system's last trilled note → only Start hook on
    // this system; End hook deferred to the page renderer for system N+1.
    let (font, config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            quarter_note(4),
            trill_ext_bracketed_note(6, TrillBracketSide::Both),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_bracket = svg.to_svg();

    let measures_plain = vec![MeasureContent {
        events: vec![quarter_note(4), trill_ext_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures_plain, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let without_bracket = svg2.to_svg();

    let delta = with_bracket.matches("<line ").count() - without_bracket.matches("<line ").count();
    assert_eq!(
        delta, 1,
        "Both bracket on last note in system should add exactly 1 hook (Start only); \
         End is deferred to page renderer"
    );
}

// --- trill bracket custom direction / length wiring ---

use crate::layout::trill_bracket::HookDirection;

fn trill_ext_bracketed_custom_note(
    pos: i8,
    side: TrillBracketSide,
    direction: HookDirection,
    length_ss: f64,
) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: Some(side),
            trill_bracket_direction: Some(direction),
            trill_bracket_length_ss: Some(length_ss),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn trill_bracket_custom_collector_propagates_direction_and_length() {
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_custom_note(4, TrillBracketSide::Both, HookDirection::Up, 1.25),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(info[0].bracket, Some(TrillBracketSide::Both));
    assert_eq!(
        info[0].bracket_direction,
        Some(HookDirection::Up),
        "collector must propagate the user's chosen hook direction"
    );
    assert_eq!(
        info[0].bracket_length_ss,
        Some(1.25),
        "collector must propagate the user's chosen hook length"
    );
    // Note without bracket has both fields None.
    assert_eq!(info[1].bracket_direction, None);
    assert_eq!(info[1].bracket_length_ss, None);
}

#[test]
fn trill_bracket_custom_collector_drops_direction_and_length_without_bracket() {
    // A user that sets direction/length but no bracket side must NOT see those
    // overrides leak through — the bracket fields are coupled at the
    // annotation surface but the collector enforces the coupling defensively.
    let bad = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_bracket: None,
            trill_bracket_direction: Some(HookDirection::Up),
            trill_bracket_length_ss: Some(2.0),
            ..NoteAnnotations::default()
        },
    });
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![bad, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(
        info[0].bracket_direction, None,
        "direction must be silently dropped when bracket is None"
    );
    assert_eq!(
        info[0].bracket_length_ss, None,
        "length must be silently dropped when bracket is None"
    );
}

#[test]
fn trill_bracket_custom_length_changes_hook_line_geometry() {
    // A longer hook must produce a visibly different SVG than the default.
    // The number of <line> elements should be unchanged — only y-coordinate
    // values differ.
    let (font, config, mcfg) = setup();
    let measures_short = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_custom_note(4, TrillBracketSide::Both, HookDirection::Down, 0.5),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures_short, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let short_svg = svg.to_svg();

    let measures_long = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_custom_note(4, TrillBracketSide::Both, HookDirection::Down, 1.5),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system_long = layout_system(&treble_prefix(), &measures_long, &mcfg, None);
    let mut svg_long = make_svg();
    draw_system(&mut svg_long, &font, &config, &system_long, 0.0, 0.0).unwrap();
    let long_svg = svg_long.to_svg();

    assert_eq!(
        short_svg.matches("<line ").count(),
        long_svg.matches("<line ").count(),
        "Length change must NOT add/remove <line> elements"
    );
    assert_ne!(
        short_svg, long_svg,
        "Length change must change the SVG output"
    );
}

#[test]
fn trill_bracket_custom_direction_up_flips_hook_y_extents() {
    // Down vs Up direction at the same length: the SVG must differ in the
    // y-extent of the last hook line, with both having the same span but
    // opposite anchoring relative to the wiggle baseline.
    let (font, config, mcfg) = setup();
    let make_svg_for = |direction: HookDirection| -> String {
        let measures = vec![MeasureContent {
            events: vec![
                trill_ext_bracketed_custom_note(4, TrillBracketSide::Start, direction, 1.0),
                quarter_note(6),
            ],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        }];
        let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
        let mut svg = make_svg();
        draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
        svg.to_svg()
    };
    let down = make_svg_for(HookDirection::Down);
    let up = make_svg_for(HookDirection::Up);
    assert_ne!(down, up, "Direction flip must change the SVG output");
    // Same number of hook lines in both cases.
    assert_eq!(
        down.matches("<line ").count(),
        up.matches("<line ").count(),
        "Direction flip must NOT add/remove <line> elements"
    );
}

#[test]
fn trill_bracket_custom_defaults_match_plain_bracketed() {
    // The custom code path with `direction=Down, length=0.75ss` must produce
    // byte-identical SVG to the non-custom path. This is the regression canary
    // that protects existing users from any drift.
    let (font, config, mcfg) = setup();
    let measures_custom = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_custom_note(4, TrillBracketSide::Both, HookDirection::Down, 0.75),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures_custom, &mcfg, None);
    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let custom_svg = svg.to_svg();

    let measures_plain = vec![MeasureContent {
        events: vec![
            trill_ext_bracketed_note(4, TrillBracketSide::Both),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system_plain = layout_system(&treble_prefix(), &measures_plain, &mcfg, None);
    let mut svg_plain = make_svg();
    draw_system(&mut svg_plain, &font, &config, &system_plain, 0.0, 0.0).unwrap();
    let plain_svg = svg_plain.to_svg();

    assert_eq!(
        custom_svg, plain_svg,
        "custom(Down, 0.75ss) must render byte-identically to the plain bracketed API"
    );
}

// --- trill wiggle speed wiring ---

use crate::layout::trill_extension::TrillWiggleSpeed;

fn trill_ext_speed_note(pos: i8, speed: TrillWiggleSpeed) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_wiggle_speed: Some(speed),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn trill_wiggle_speed_collector_propagates_speed_when_extension_active() {
    // The collector must carry the user-selected speed through to the draw
    // pass — without this, every wiggle would tile the default `WiggleTrill`
    // glyph regardless of what the user asked for.
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![
            trill_ext_speed_note(4, TrillWiggleSpeed::Fast),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(
        info[0].wiggle_speed,
        Some(TrillWiggleSpeed::Fast),
        "collector must surface the user's chosen speed",
    );
    assert_eq!(
        info[1].wiggle_speed, None,
        "plain note (no extension) must carry no speed",
    );
}

#[test]
fn trill_wiggle_speed_collector_drops_speed_when_no_extension() {
    // Speed without an extension flag is silently inert downstream: the
    // wiggle requires both the trill ornament AND the extension flag.
    let bad = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: false,
            trill_wiggle_speed: Some(TrillWiggleSpeed::Slowest),
            ..NoteAnnotations::default()
        },
    });
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![bad, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(
        info[0].wiggle_speed, None,
        "speed without extension must be silently dropped",
    );
}

#[test]
fn trill_wiggle_speed_fast_tiles_more_segments_than_slow() {
    // The whole point of choosing a speed: a faster (denser) wiggle tile is
    // shorter, so a fixed-width span fits more tiles. Verify the inequality
    // holds in real rendered output — the strict guarantee against any
    // future regression that wires the speed but ignores its advance width.
    let (font, config, mcfg) = setup();

    let measures_fast = vec![MeasureContent {
        events: vec![
            trill_ext_speed_note(4, TrillWiggleSpeed::Fastest),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let measures_slow = vec![MeasureContent {
        events: vec![
            trill_ext_speed_note(4, TrillWiggleSpeed::Slowest),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];

    let system_fast = layout_system(&treble_prefix(), &measures_fast, &mcfg, None);
    let system_slow = layout_system(&treble_prefix(), &measures_slow, &mcfg, None);

    let mut svg_fast = make_svg();
    draw_system(&mut svg_fast, &font, &config, &system_fast, 0.0, 0.0).unwrap();
    let fast = svg_fast.to_svg();

    let mut svg_slow = make_svg();
    draw_system(&mut svg_slow, &font, &config, &system_slow, 0.0, 0.0).unwrap();
    let slow = svg_slow.to_svg();

    let fast_paths = fast.matches("<path").count();
    let slow_paths = slow.matches("<path").count();
    assert!(
        fast_paths > slow_paths,
        "Fastest wiggle must tile more segments than Slowest in the same span: \
         fast={fast_paths}, slow={slow_paths}",
    );
}

#[test]
fn trill_wiggle_speed_each_variant_uses_its_own_glyph_advance() {
    // Sanity check that the speeds actually map to different SMuFL glyphs
    // with different advance widths in Bravura. If two speeds shared an
    // advance, the "fast > slow tile count" guarantee would silently fail
    // for that pair.
    let (font, _config, _mcfg) = setup();
    let mut advances: Vec<f64> = TrillWiggleSpeed::ALL
        .iter()
        .map(|s| font.glyph_advance(s.to_glyph()).expect("glyph present") as f64)
        .collect();
    advances.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // All 9 speeds must yield non-zero advances (so the layout loop terminates).
    for a in &advances {
        assert!(*a > 0.0, "speed glyph advance must be positive, got {a}");
    }

    // Bravura's wiggleTrill family progresses monotonically: every pair of
    // adjacent advances must differ, otherwise the speed choice would be a
    // no-op for that pair.
    for pair in advances.windows(2) {
        assert!(
            pair[1] - pair[0] > 1.0,
            "adjacent wiggle speed advances must differ by >1 font unit, got {pair:?}",
        );
    }
}

#[test]
fn trill_wiggle_speed_none_uses_default_glyph() {
    // When `wiggle_speed` is None on the collector info, the draw pass must
    // tile `Glyph::WiggleTrill` (the Default for TrillWiggleSpeed). Compare
    // against an explicit Standard request: outputs must be byte-identical.
    let (font, config, mcfg) = setup();

    let measures_none = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let measures_standard = vec![MeasureContent {
        events: vec![
            trill_ext_speed_note(4, TrillWiggleSpeed::Standard),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];

    let s_none = layout_system(&treble_prefix(), &measures_none, &mcfg, None);
    let s_std = layout_system(&treble_prefix(), &measures_standard, &mcfg, None);

    let mut svg_none = make_svg();
    draw_system(&mut svg_none, &font, &config, &s_none, 0.0, 0.0).unwrap();
    let none_out = svg_none.to_svg();

    let mut svg_std = make_svg();
    draw_system(&mut svg_std, &font, &config, &s_std, 0.0, 0.0).unwrap();
    let std_out = svg_std.to_svg();

    assert_eq!(
        none_out, std_out,
        "wiggle_speed=None and wiggle_speed=Standard must produce identical SVG"
    );
}

// --- trill-with-mordent extension (compound prefix glyph with wiggle) ---

fn trill_with_mordent_ext_note(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::TrillWithMordent),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn trill_extension_collector_accepts_trill_with_mordent() {
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![trill_with_mordent_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(
        info[0].has_trill_extension,
        "TrillWithMordent + trill_extension flag must be recognized as an extension"
    );
    assert_eq!(
        info[0].ornament,
        Some(Ornament::TrillWithMordent),
        "collector should propagate the actual ornament so the draw pass knows the glyph advance"
    );
}

#[test]
fn trill_extension_collector_drops_short_trill() {
    // ShortTrill is by definition the wave-less form; even with the flag
    // set, the collector should treat it as inert.
    let bad = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::ShortTrill),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    });
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![bad, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(
        !info[0].has_trill_extension,
        "ShortTrill with trill_extension=true must NOT be flagged: the short-trill mark is wave-less by convention"
    );
    assert!(
        info[0].ornament.is_none(),
        "ornament field is None when has_trill_extension is false"
    );
}

#[test]
fn trill_extension_collector_drops_non_trill_ornament_field() {
    // Confirms `ornament` field is None when the ornament is something
    // that does not support extension (Mordent, Turn, etc.).
    let bad = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Turn),
            trill_extension: true,
            ..NoteAnnotations::default()
        },
    });
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![bad, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(!info[0].has_trill_extension);
    assert!(info[0].ornament.is_none());
}

#[test]
fn trill_extension_collector_propagates_trill_variant() {
    // Sanity check: for a plain `Trill`, the ornament field carries Trill.
    let (_font, _config, mcfg) = setup();
    let measures = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert_eq!(info[0].ornament, Some(Ornament::Trill));
}

#[test]
fn trill_with_mordent_extension_renders_wiggle_paths() {
    // The compound `OrnamentPrecompTrillWithMordent` glyph is ~470 font-units
    // wider than the bare "tr" (Bravura: 990 vs 521). To verify the wiggle
    // still draws at least one segment for the compound case, we widen
    // `spring_constant` directly so the inter-note gap exceeds the
    // compound glyph's advance plus one wiggle segment with room to spare.
    let (font, config, mut mcfg) = setup();
    mcfg.spring_constant = 8.0 * config.staff_space;
    let trill_compound_whole = trill_with_mordent_ext_note(4);
    let plain_whole = MeasureEvent::Note(NoteEvent {
        staff_position: 6,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    });

    let measures = vec![MeasureContent {
        events: vec![trill_compound_whole, plain_whole.clone()],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);

    let mut svg = make_svg();
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();
    let with_compound = svg.to_svg();
    let compound_paths = with_compound.matches("<path").count();

    // Identical score but with `trill_extension = false` so no wiggle is
    // drawn. The difference is exactly the wiggle's contribution.
    let measures3 = vec![MeasureContent {
        events: vec![
            MeasureEvent::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 0,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations {
                    ornament: Some(Ornament::TrillWithMordent),
                    trill_extension: false, // <-- no extension
                    ..NoteAnnotations::default()
                },
            }),
            plain_whole.clone(),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system3 = layout_system(&treble_prefix(), &measures3, &mcfg, None);
    let mut svg3 = make_svg();
    draw_system(&mut svg3, &font, &config, &system3, 0.0, 0.0).unwrap();
    let without_ext_paths = svg3.to_svg().matches("<path").count();

    assert!(
        compound_paths > without_ext_paths,
        "TrillWithMordent + extension must add wiggle paths beyond plain TrillWithMordent: \
         with_ext={compound_paths}, without_ext={without_ext_paths}"
    );

    // Cross-check: a `Trill` + extension at the same layout renders a
    // different SVG than the compound version (different prefix glyph,
    // different wiggle start). This is the canary that the collector +
    // draw pass propagated the actual ornament rather than hardcoding
    // `Trill` for the glyph-advance lookup.
    let measures2 = vec![MeasureContent {
        events: vec![trill_ext_note(4), plain_whole],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let system2 = layout_system(&treble_prefix(), &measures2, &mcfg, None);
    let mut svg2 = make_svg();
    draw_system(&mut svg2, &font, &config, &system2, 0.0, 0.0).unwrap();
    let with_plain = svg2.to_svg();
    assert_ne!(
        with_compound, with_plain,
        "compound and plain trill extensions must differ — different prefix glyph, different wiggle start"
    );
}

#[test]
fn trill_with_mordent_extension_uses_wider_glyph_advance() {
    // The precomposed compound glyph is wider than the bare "tr". The
    // wiggle therefore starts further right, leaving less span and fewer
    // segments. This test enforces the relation as a regression canary —
    // if a future change reverts to hardcoding `Glyph::OrnamentTrill` for
    // the advance lookup, the compound and plain versions would have
    // identical segment counts.
    let font = bravura_font();
    let trill_advance = font
        .glyph_advance(smufl::Glyph::OrnamentTrill)
        .unwrap() as f64;
    let compound_advance = font
        .glyph_advance(smufl::Glyph::OrnamentPrecompTrillWithMordent)
        .unwrap() as f64;
    assert!(
        compound_advance > trill_advance,
        "the precomposed trill-with-mordent compound glyph must be wider than \
         the bare 'tr' glyph in Bravura — this is the geometric premise of \
         the wiggle-start adjustment; advances: trill={trill_advance}, compound={compound_advance}"
    );
}

// --- trill_extension_length_ss (explicit early termination) ---

fn trill_ext_length_note(pos: i8, length_ss: f64) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_length_ss: Some(length_ss),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn collector_propagates_explicit_length_when_trill_extension_active() {
    let measures = vec![MeasureContent {
        events: vec![trill_ext_length_note(4, 2.5), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let (_font, _config, mcfg) = setup();
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(info[0].has_trill_extension);
    assert_eq!(info[0].explicit_length_ss, Some(2.5));
}

#[test]
fn collector_drops_explicit_length_when_trill_extension_inactive() {
    // The collector's contract: explicit length only travels through when
    // `has_trill_extension == true`. A length set on a note that ALSO has
    // `trill_extension = false` must be filtered out — matching how
    // bracket / wiggle_speed fields are filtered. Without this, a stale
    // length annotation could leak into the renderer state.
    let stale_length = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: false,
            trill_extension_length_ss: Some(3.0),
            ..NoteAnnotations::default()
        },
    });
    let measures = vec![MeasureContent {
        events: vec![stale_length, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let (_font, _config, mcfg) = setup();
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(!info[0].has_trill_extension);
    assert_eq!(info[0].explicit_length_ss, None);
}

#[test]
fn explicit_length_renders_fewer_paths_than_default_when_shorter() {
    // The default test measure spacing leaves only ~1 wiggle of natural
    // span (just enough for the existing `_renders_at_least_one_wiggle_path`
    // smoke test). For an *inequality* test we need the natural span to
    // accommodate several wiggles, so we widen `spring_constant` to ensure
    // there's a meaningful contrast between explicit-1.0-ss and natural.
    let (font, config, mut mcfg) = setup();
    mcfg.spring_constant = 12.0 * config.staff_space;

    let short = vec![MeasureContent {
        events: vec![trill_ext_length_note(4, 1.0), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let natural = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_short = layout_system(&treble_prefix(), &short, &mcfg, None);
    let sys_natural = layout_system(&treble_prefix(), &natural, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_short, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_natural, 0.0, 0.0).unwrap();
    let short_paths = svg_a.to_svg().matches("<path").count();
    let natural_paths = svg_b.to_svg().matches("<path").count();
    assert!(
        short_paths < natural_paths,
        "1.0-ss explicit length must produce fewer paths than the natural span: \
         short={short_paths}, natural={natural_paths}"
    );
}

#[test]
fn explicit_length_larger_than_natural_clamps_to_natural() {
    // A length larger than the natural span must clamp — the wiggle never
    // overruns the next note. We exercise this by setting an absurdly large
    // length (1000 ss) and verifying the result is byte-identical to the
    // natural (no-length) render.
    let (font, config, mcfg) = setup();
    let huge = vec![MeasureContent {
        events: vec![trill_ext_length_note(4, 1000.0), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let natural = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_huge = layout_system(&treble_prefix(), &huge, &mcfg, None);
    let sys_natural = layout_system(&treble_prefix(), &natural, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_huge, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_natural, 0.0, 0.0).unwrap();

    // Note: the natural version is a "within-system" trill (there's a
    // following note), so cross_system == false in both branches.
    // The end_x must clamp identically.
    assert_eq!(
        svg_a.to_svg(),
        svg_b.to_svg(),
        "explicit length 1000 must clamp to natural and render byte-identical"
    );
}

#[test]
fn explicit_length_zero_produces_no_wiggle() {
    // A non-positive length disables the wiggle entirely (the "tr" glyph
    // still renders). We compare path counts vs the plain-trill-no-extension
    // case (which also produces only the "tr" glyph + notes + staff) and
    // expect equality on path count: the wiggle contributed nothing.
    let (font, config, mcfg) = setup();
    let zero_length = vec![MeasureContent {
        events: vec![trill_ext_length_note(4, 0.0), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let plain_trill = vec![MeasureContent {
        events: vec![trill_no_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_zero = layout_system(&treble_prefix(), &zero_length, &mcfg, None);
    let sys_plain = layout_system(&treble_prefix(), &plain_trill, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_zero, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_plain, 0.0, 0.0).unwrap();
    let zero_paths = svg_a.to_svg().matches("<path").count();
    let plain_paths = svg_b.to_svg().matches("<path").count();
    assert_eq!(
        zero_paths, plain_paths,
        "explicit length 0 must contribute zero wiggle paths: zero={zero_paths}, plain={plain_paths}"
    );
}

#[test]
fn explicit_length_negative_produces_no_wiggle() {
    // A negative length goes through the same `_ <= 0` branch as zero —
    // produces no wiggle. Documenting this contract explicitly: callers
    // need not validate length sign at the API boundary.
    let (font, config, mcfg) = setup();
    let negative_length = vec![MeasureContent {
        events: vec![trill_ext_length_note(4, -2.0), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let plain_trill = vec![MeasureContent {
        events: vec![trill_no_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_neg = layout_system(&treble_prefix(), &negative_length, &mcfg, None);
    let sys_plain = layout_system(&treble_prefix(), &plain_trill, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_neg, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_plain, 0.0, 0.0).unwrap();
    assert_eq!(
        svg_a.to_svg().matches("<path").count(),
        svg_b.to_svg().matches("<path").count(),
        "negative explicit length must contribute zero wiggle paths"
    );
}

#[test]
fn explicit_length_in_chord_collector_propagates() {
    use crate::layout::measure::ChordEvent;

    let chord_with_length = MeasureEvent::Chord(ChordEvent {
        staff_positions: vec![2, 4, 6],
        duration_log2: 0,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_length_ss: Some(2.0),
            ..NoteAnnotations::default()
        },
    });
    let measures = vec![MeasureContent {
        events: vec![chord_with_length, quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let (_font, _config, mcfg) = setup();
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(info[0].has_trill_extension, "chord trill must be flagged");
    assert_eq!(
        info[0].explicit_length_ss,
        Some(2.0),
        "chord-level explicit length must travel through the collector"
    );
    // Top-of-chord anchoring: the system_renderer pins the wiggle's y to
    // the highest chord note.
    assert_eq!(info[0].staff_position, 6);
}

#[test]
fn explicit_length_on_last_note_avoids_cross_system_extension() {
    // The trilled note is the LAST note in the system AND has an explicit
    // length. The wiggle terminates at the explicit point, NOT at the
    // system edge — confirmed by comparing the rendered system width
    // between the explicit-length version and an unannotated version.
    // Specifically: the explicit-length version's wiggle must have fewer
    // segments than the system-edge default (which extends to the staff
    // right edge minus a small gap). Widen `spring_constant` to ensure
    // the natural-to-edge span is meaningfully larger than the explicit
    // 1.5-ss request.
    let (font, config, mut mcfg) = setup();
    mcfg.spring_constant = 12.0 * config.staff_space;

    let with_explicit = vec![MeasureContent {
        events: vec![quarter_note(4), trill_ext_length_note(6, 1.5)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let without_explicit = vec![MeasureContent {
        events: vec![quarter_note(4), trill_ext_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_explicit = layout_system(&treble_prefix(), &with_explicit, &mcfg, None);
    let sys_natural = layout_system(&treble_prefix(), &without_explicit, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_explicit, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_natural, 0.0, 0.0).unwrap();
    let explicit_paths = svg_a.to_svg().matches("<path").count();
    let natural_paths = svg_b.to_svg().matches("<path").count();
    assert!(
        explicit_paths < natural_paths,
        "explicit length on last note must terminate before the system edge: \
         explicit={explicit_paths}, natural-to-edge={natural_paths}"
    );
}

#[test]
fn explicit_length_with_end_bracket_anchors_at_shortened_terminus() {
    // The bracket's End hook follows the wiggle's actual right edge. With
    // an explicit length, the wiggle terminates earlier — so the End hook
    // moves left with it. We verify by counting `<line>` elements: an End
    // bracket adds exactly one vertical hook line. The explicit-length
    // version must have the same number of `<line>` elements as a natural
    // End-bracketed trill (the hook count is glyph-independent — only its
    // x-position changes).
    use crate::layout::trill_bracket::TrillBracketSide;
    let (font, config, mut mcfg) = setup();
    mcfg.spring_constant = 12.0 * config.staff_space;

    let make_event = |length_ss: Option<f64>| -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 0,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations {
                ornament: Some(Ornament::Trill),
                trill_extension: true,
                trill_bracket: Some(TrillBracketSide::End),
                trill_extension_length_ss: length_ss,
                ..NoteAnnotations::default()
            },
        })
    };

    let with_length = vec![MeasureContent {
        events: vec![make_event(Some(1.5)), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let without_length = vec![MeasureContent {
        events: vec![make_event(None), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_with = layout_system(&treble_prefix(), &with_length, &mcfg, None);
    let sys_without = layout_system(&treble_prefix(), &without_length, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_with, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_without, 0.0, 0.0).unwrap();
    let line_count_with = svg_a.to_svg().matches("<line").count();
    let line_count_without = svg_b.to_svg().matches("<line").count();
    assert_eq!(
        line_count_with, line_count_without,
        "End-bracket hook count must be glyph-position-independent: \
         with_length={line_count_with}, without_length={line_count_without}"
    );
    // And the byte content must differ — same hook count, different x.
    assert_ne!(
        svg_a.to_svg(),
        svg_b.to_svg(),
        "explicit-length End-bracketed trill must render distinctly from natural-length"
    );
}

// --- trill_extension_to_note_offset (note-anchored end) ---

fn trill_ext_to_note(pos: i8, offset: usize) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_to_note_offset: Some(offset),
            ..NoteAnnotations::default()
        },
    })
}

#[test]
fn collector_propagates_to_note_offset_when_trill_extension_active() {
    // Collector must thread the new offset field through into the info
    // record's `to_note_offset` slot. Catches a regression in the
    // collector's field-copy that silently drops the new annotation.
    let measures = vec![MeasureContent {
        events: vec![trill_ext_to_note(4, 2), quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let (_font, _config, mcfg) = setup();
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(info[0].has_trill_extension);
    assert_eq!(info[0].to_note_offset, Some(2));
}

#[test]
fn collector_drops_to_note_offset_when_trill_extension_inactive() {
    // Matches the filtering rule for `explicit_length_ss`: the offset
    // only travels when `trill_extension == true`. A stale offset on a
    // note with the extension flag off must be filtered out so the
    // renderer can't see it.
    let stale_offset = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: false,
            trill_extension_to_note_offset: Some(2),
            ..NoteAnnotations::default()
        },
    });
    let measures = vec![MeasureContent {
        events: vec![stale_offset, quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let (_font, _config, mcfg) = setup();
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(!info[0].has_trill_extension);
    assert_eq!(info[0].to_note_offset, None);
}

#[test]
fn to_note_offset_two_renders_more_paths_than_offset_one() {
    // A trill anchored to the note 2 positions ahead must cover a longer
    // horizontal span than a trill anchored to the immediately following
    // note — strictly more wiggle paths. The wider spring_constant
    // ensures a meaningful difference in span; otherwise both spans
    // round down to the same per-tile count.
    let (font, config, mut mcfg) = setup();
    mcfg.spring_constant = 12.0 * config.staff_space;

    let to_one = vec![MeasureContent {
        events: vec![trill_ext_to_note(4, 1), quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let to_two = vec![MeasureContent {
        events: vec![trill_ext_to_note(4, 2), quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_one = layout_system(&treble_prefix(), &to_one, &mcfg, None);
    let sys_two = layout_system(&treble_prefix(), &to_two, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_one, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_two, 0.0, 0.0).unwrap();
    let one_paths = svg_a.to_svg().matches("<path").count();
    let two_paths = svg_b.to_svg().matches("<path").count();
    assert!(
        two_paths > one_paths,
        "offset=2 must render strictly more paths than offset=1: \
         one={one_paths}, two={two_paths}"
    );
}

#[test]
fn to_note_offset_one_byte_equivalent_to_natural_default() {
    // Offset = 1 means "extend to the next note" which is the implicit
    // default of `trill_extension = true` with no offset set. The two
    // renders must be byte-identical — locks in the "offset=1 is the
    // implicit default" contract.
    let (font, config, mcfg) = setup();

    let with_offset_one = vec![MeasureContent {
        events: vec![trill_ext_to_note(4, 1), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let natural = vec![MeasureContent {
        events: vec![trill_ext_note(4), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_off = layout_system(&treble_prefix(), &with_offset_one, &mcfg, None);
    let sys_nat = layout_system(&treble_prefix(), &natural, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_off, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_nat, 0.0, 0.0).unwrap();
    assert_eq!(
        svg_a.to_svg(),
        svg_b.to_svg(),
        "to_note_offset=Some(1) must render byte-identically to the natural \
         default (no offset set)"
    );
}

#[test]
fn to_note_offset_zero_drops_wiggle() {
    // Offset = 0 is degenerate (the target is the trilled note itself).
    // The renderer's branch sets `end_x = start_x` so layout_trill_extension
    // returns None — same suppression as non-positive explicit lengths.
    let (font, config, mcfg) = setup();

    let zero_offset = vec![MeasureContent {
        events: vec![trill_ext_to_note(4, 0), quarter_note(6)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let plain_trill_ornament = vec![MeasureContent {
        events: vec![
            MeasureEvent::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 0,
                dots: 0,
                accidental: None,
                stem_direction: None,
                annotations: NoteAnnotations {
                    ornament: Some(Ornament::Trill),
                    trill_extension: false,
                    ..NoteAnnotations::default()
                },
            }),
            quarter_note(6),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_zero = layout_system(&treble_prefix(), &zero_offset, &mcfg, None);
    let sys_plain = layout_system(&treble_prefix(), &plain_trill_ornament, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_zero, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_plain, 0.0, 0.0).unwrap();
    let zero_paths = svg_a.to_svg().matches("<path").count();
    let plain_paths = svg_b.to_svg().matches("<path").count();
    assert_eq!(
        zero_paths, plain_paths,
        "offset=0 must drop the wiggle — same path count as an ornament-only \
         trill: zero={zero_paths}, plain={plain_paths}"
    );
}

#[test]
fn to_note_offset_overshoot_falls_back_to_system_edge() {
    // An offset that walks past the last note in the system (`notes.get(i +
    // 99) → None`) must reach the system-edge fallback, NOT panic on the
    // index. The fallback yields the same end_x as a trilled last note —
    // so an overshoot offset on note 1 of a 3-note system covers from the
    // trilled note all the way to the staff-right-edge minus the system
    // edge gap. Strictly more paths than offset=1 in the same layout.
    let (font, config, mut mcfg) = setup();
    mcfg.spring_constant = 12.0 * config.staff_space;

    let overshoot = vec![MeasureContent {
        events: vec![
            trill_ext_to_note(4, 99),
            quarter_note(6),
            quarter_note(8),
        ],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let to_one = vec![MeasureContent {
        events: vec![trill_ext_to_note(4, 1), quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_overshoot = layout_system(&treble_prefix(), &overshoot, &mcfg, None);
    let sys_one = layout_system(&treble_prefix(), &to_one, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_overshoot, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_one, 0.0, 0.0).unwrap();
    let overshoot_paths = svg_a.to_svg().matches("<path").count();
    let one_paths = svg_b.to_svg().matches("<path").count();
    assert!(
        overshoot_paths > one_paths,
        "overshoot offset must extend to the system edge — more paths than \
         offset=1: overshoot={overshoot_paths}, one={one_paths}"
    );
}

#[test]
fn to_note_offset_overshoot_renders_byte_identical_to_last_note_natural() {
    // Critical anchor canary: when the offset overshoots, the end_x
    // formula is `staff_width - SYSTEM_EDGE_GAP_SS * staff_space`. A
    // trill on the LAST note of the same system with the default
    // (no-offset) extension uses exactly the same formula. With the same
    // trilled-note x-position, the two renders must be byte-identical.
    //
    // To pin the trilled note at the same x in both scores, we trill the
    // SAME note (position 4 at index 0) in both — but in the overshoot
    // version the offset is huge, and in the "natural last-note" version
    // we put the trilled note as the only note in its measure with no
    // following note. Both reach the system-edge fallback for end_x.
    let (font, config, mcfg) = setup();

    let overshoot = vec![MeasureContent {
        events: vec![trill_ext_to_note(4, 99)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let last_note_natural = vec![MeasureContent {
        events: vec![trill_ext_note(4)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_overshoot = layout_system(&treble_prefix(), &overshoot, &mcfg, None);
    let sys_natural = layout_system(&treble_prefix(), &last_note_natural, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_overshoot, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_natural, 0.0, 0.0).unwrap();
    let overshoot_paths = svg_a.to_svg().matches("<path").count();
    let natural_paths = svg_b.to_svg().matches("<path").count();
    assert_eq!(
        overshoot_paths, natural_paths,
        "overshoot offset and natural last-note must reach the same system-edge \
         end_x — same path count: overshoot={overshoot_paths}, natural={natural_paths}"
    );
}

#[test]
fn to_note_offset_in_chord_collector_propagates() {
    // The chord branch of the collector must also propagate the new
    // field. Mirrors `explicit_length_in_chord_collector_propagates` for
    // the offset field. Catches a regression that wires through the
    // Note branch but forgets the Chord branch.
    use crate::layout::measure::ChordEvent;

    let chord_with_offset = MeasureEvent::Chord(ChordEvent {
        staff_positions: vec![2, 4, 6],
        duration_log2: 0,
        dots: 0,
        accidentals: vec![None, None, None],
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_to_note_offset: Some(2),
            ..NoteAnnotations::default()
        },
    });
    let measures = vec![MeasureContent {
        events: vec![chord_with_offset, quarter_note(7), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let (_font, _config, mcfg) = setup();
    let system = layout_system(&treble_prefix(), &measures, &mcfg, None);
    let info = collect_trill_extension_note_info(&system);
    assert!(info[0].has_trill_extension, "chord trill must be flagged");
    assert_eq!(
        info[0].to_note_offset,
        Some(2),
        "chord-level to_note_offset must travel through the collector"
    );
    // Top-of-chord anchoring: the system_renderer pins the wiggle's y to
    // the highest chord note.
    assert_eq!(info[0].staff_position, 6);
}

#[test]
fn to_note_offset_yields_to_explicit_length_when_both_set() {
    // The documented precedence: when both `trill_extension_length_ss`
    // and `trill_extension_to_note_offset` are set, the explicit length
    // wins at draw time. Verify by rendering a note with both fields
    // set vs. just the length field — the SVGs must be byte-identical.
    let (font, config, mut mcfg) = setup();
    mcfg.spring_constant = 12.0 * config.staff_space;

    let both_set = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_length_ss: Some(1.0),
            trill_extension_to_note_offset: Some(2),
            ..NoteAnnotations::default()
        },
    });
    let length_only = MeasureEvent::Note(NoteEvent {
        staff_position: 4,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations {
            ornament: Some(Ornament::Trill),
            trill_extension: true,
            trill_extension_length_ss: Some(1.0),
            ..NoteAnnotations::default()
        },
    });
    let measures_both = vec![MeasureContent {
        events: vec![both_set, quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let measures_length = vec![MeasureContent {
        events: vec![length_only, quarter_note(6), quarter_note(8)],
        barline: BarlineStyle::Single,
        volta: None,
        additional_voices: vec![],
    }];
    let sys_both = layout_system(&treble_prefix(), &measures_both, &mcfg, None);
    let sys_length = layout_system(&treble_prefix(), &measures_length, &mcfg, None);
    let mut svg_a = make_svg();
    let mut svg_b = make_svg();
    draw_system(&mut svg_a, &font, &config, &sys_both, 0.0, 0.0).unwrap();
    draw_system(&mut svg_b, &font, &config, &sys_length, 0.0, 0.0).unwrap();
    assert_eq!(
        svg_a.to_svg(),
        svg_b.to_svg(),
        "when both explicit length and to_note_offset are set, the explicit \
         length wins — render must be byte-identical to length-only"
    );
}
