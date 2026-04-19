use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::measure::MeasureElement;
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{auto_stem_direction, StemDirection};
use crate::layout::system::SystemLayout;
use crate::layout::tie::{layout_tie, tie_direction_from_stem};
use crate::render::measure_renderer::draw_measure;
use crate::render::note_renderer::NoteheadKind;
use crate::render::staff_renderer::draw_staff_lines;
use crate::render::tie_renderer::draw_tie;
use crate::render::SvgWriter;

/// Collect notes from the system's positioned elements in order, yielding
/// (x_in_system, staff_position, duration_log2, tie_forward, stem_direction_override)
/// for each note event. Chord notes are expanded into individual entries so
/// each chord note can be tied independently. Skips clefs, rests, barlines, etc.
fn collect_note_positions(system: &SystemLayout) -> Vec<(f64, i8, u8, bool, Option<StemDirection>)> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for elem in &measure.layout.elements {
            let elem_x = measure.x_offset + elem.x;
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push((elem_x, n.staff_position, n.duration_log2, n.tie_forward, n.stem_direction));
                }
                MeasureElement::Chord(c) => {
                    // Each note in the chord gets its own entry so ties can
                    // match by staff position independently.
                    for &pos in &c.staff_positions {
                        notes.push((elem_x, pos, c.duration_log2, c.tie_forward, c.stem_direction));
                    }
                }
                _ => {}
            }
        }
    }
    notes
}

/// Draw a complete system (staff lines + all measures) onto an SVG writer.
///
/// Staff lines span the full `staff_width` of the system layout.
/// Each measure is drawn at its computed x-offset. After all measures are
/// rendered, tie curves are drawn between notes marked with `tie_forward`.
pub fn draw_system(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    x: f64,
    y: f64,
) -> Result<(), FontError> {
    if system.measures.is_empty() {
        return Ok(());
    }

    // Create staff spanning the full system width
    let staff = StaffLayout::new(x, y, system.staff_width, config.staff_space);

    // Draw continuous staff lines
    draw_staff_lines(svg, &staff, config);

    // Draw each measure at its offset
    let clef = system.clef_kind.to_clef();
    for sys_measure in &system.measures {
        draw_measure(
            svg,
            &staff,
            font,
            config,
            &sys_measure.layout,
            x + sys_measure.x_offset,
            &clef,
        )?;
    }

    // Draw ties between notes with tie_forward = true and their target notes
    draw_system_ties(svg, font, config, system, &staff, x)?;

    Ok(())
}

/// After all measures in a system are drawn, scan for tied notes and draw
/// tie curves between them.
///
/// For each note with `tie_forward = true`, finds the next note at the same
/// staff position and draws a tie from the right edge of the first notehead
/// to the left edge of the second.
fn draw_system_ties(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let note_positions = collect_note_positions(system);

    for (i, &(nx, pos, dur_log2, tie_forward, stem_dir)) in note_positions.iter().enumerate() {
        if !tie_forward {
            continue;
        }

        // Find the next note at the same staff position
        let target = note_positions[i + 1..]
            .iter()
            .find(|&&(_, target_pos, _, _, _)| target_pos == pos);

        let Some(&(target_x, _, _, _, _)) = target else {
            continue;
        };

        // Compute notehead advance width for tie endpoint positioning
        let notehead_kind = match dur_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        // Tie starts at right edge of first notehead, ends at left edge of second
        let tie_x_start = system_x + nx + advance;
        let tie_x_end = system_x + target_x;

        // Determine tie direction from stem direction
        let direction = stem_dir.unwrap_or_else(|| auto_stem_direction(pos));
        let tie_dir = tie_direction_from_stem(direction);

        // Y-coordinate of the notehead center
        let note_y = staff.y_of(pos);

        let tie_layout = layout_tie(tie_x_start, tie_x_end, note_y, tie_dir, config);
        draw_tie(svg, &tie_layout);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::barline::BarlineStyle;
    use crate::layout::key_signature::KeySignature;
    use crate::layout::measure::{MeasureLayoutConfig, NoteEvent, RestEvent};
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
        tie_forward: false,
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
            tie_forward: true,
        })
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
            tie_forward: true,
        })
    }

    fn untied_chord(positions: Vec<i8>) -> MeasureEvent {
        let acc_count = positions.len();
        MeasureEvent::Chord(ChordEvent {
            staff_positions: positions,
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None; acc_count],
            stem_direction: None,
            tie_forward: false,
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
}
