use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::staff::StaffLayout;
use crate::layout::system::SystemLayout;
use crate::render::measure_renderer::draw_measure;
use crate::render::staff_renderer::draw_staff_lines;
use crate::render::SvgWriter;

/// Draw a complete system (staff lines + all measures) onto an SVG writer.
///
/// Staff lines span the full `staff_width` of the system layout.
/// Each measure is drawn at its computed x-offset.
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
}
