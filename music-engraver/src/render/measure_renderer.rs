use music::notation::clef::Clef;

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::dot::dot_staff_position;
use crate::layout::measure::{MeasureElement, MeasureLayout, NoteEvent};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{auto_stem_direction, StemDirection};
use crate::render::barline_renderer::draw_barline;
use crate::render::dot_renderer::draw_dots;
use crate::render::flag_renderer::draw_flag;
use crate::render::key_sig_renderer::draw_key_signature;
use crate::render::note_renderer::{draw_ledger_lines, draw_notehead, NoteheadKind};
use crate::render::rest_renderer::draw_rest;
use crate::render::staff_renderer::draw_clef;
use crate::render::stem_renderer::{draw_stem, stem_endpoints, stem_x};
use crate::render::time_sig_renderer::draw_time_signature;
use crate::render::SvgWriter;

/// Draw a complete laid-out measure onto an SVG writer.
///
/// `x_offset` shifts the entire measure horizontally (for multi-measure rendering).
/// `clef_for_key_sig` determines accidental placement for key signatures.
///
/// Draws staff lines first, then iterates through positioned elements.
pub fn draw_measure(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    layout: &MeasureLayout,
    x_offset: f64,
    clef_for_key_sig: &Clef,
) -> Result<(), FontError> {
    for positioned in &layout.elements {
        let elem_x = x_offset + positioned.x;
        match &positioned.element {
            MeasureElement::Clef(clef_layout) => {
                draw_clef(svg, staff, clef_layout, font)?;
            }
            MeasureElement::KeySignature(key) => {
                draw_key_signature(svg, staff, font, elem_x, key, clef_for_key_sig)?;
            }
            MeasureElement::TimeSignature(kind) => {
                draw_time_signature(svg, staff, font, elem_x, kind)?;
            }
            MeasureElement::Note(note) => {
                draw_note_event(svg, staff, font, config, elem_x, note)?;
            }
            MeasureElement::Rest(rest) => {
                draw_rest(svg, staff, font, elem_x, rest.duration_log2)?;
            }
            MeasureElement::Barline(style) => {
                draw_barline(svg, staff, font, elem_x, *style)?;
            }
        }
    }

    Ok(())
}

/// Notehead kind from log2 duration: 0=whole, 1=half, 2+=filled.
fn notehead_kind_from_log2(duration_log2: u8) -> NoteheadKind {
    match duration_log2 {
        0 => NoteheadKind::Whole,
        1 => NoteheadKind::Half,
        _ => NoteheadKind::Filled,
    }
}

/// Number of flags from log2 duration: 0–2 have no flags, 3=one flag, 4=two, etc.
fn flag_count_from_log2(duration_log2: u8) -> u8 {
    if duration_log2 <= 2 {
        0
    } else {
        duration_log2 - 2
    }
}

/// Draw a complete note event: accidental + notehead + ledger lines + stem + flag + dots.
fn draw_note_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    note: &NoteEvent,
) -> Result<(), FontError> {
    let kind = notehead_kind_from_log2(note.duration_log2);
    let position = note.staff_position;

    // Draw accidental (pre-resolved glyph) to the left of notehead
    if let Some(acc_glyph) = note.accidental {
        let outline = font.glyph_outline(acc_glyph)?;
        let acc_advance = outline.advance_width as f64;
        let padding = 0.12 * staff.staff_space;
        let acc_x = x - acc_advance - padding;
        let acc_y = staff.y_of(position);
        let transform = format!("translate({acc_x}, {acc_y})");
        svg.add_path(&outline.path_data, "black", Some(&transform));
    }

    // Draw notehead
    let advance = draw_notehead(svg, staff, font, x, position, kind)?;

    // Draw ledger lines
    draw_ledger_lines(svg, staff, config, x, advance, position);

    // Determine stem direction
    let needs_stem = note.duration_log2 >= 1; // whole notes have no stem
    let direction = if needs_stem {
        Some(note.stem_direction.unwrap_or_else(|| auto_stem_direction(position)))
    } else {
        None
    };

    // Draw stem
    if let Some(dir) = direction {
        draw_stem(svg, staff, config, x, advance, position, dir);

        // Draw flag
        let flags = flag_count_from_log2(note.duration_log2);
        if flags > 0 {
            let sx = stem_x(x, advance, dir, config.stem_thickness_fu());
            let (y_top, y_bottom) = stem_endpoints(staff, position, dir);
            let tip_y = match dir {
                StemDirection::Up => y_top,
                StemDirection::Down => y_bottom,
            };
            draw_flag(svg, font, sx, tip_y, flags, dir)?;
        }
    }

    // Draw augmentation dots
    if note.dots > 0 {
        let dot_pos = dot_staff_position(position);
        draw_dots(svg, staff, font, x, advance, dot_pos, note.dots)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::barline::BarlineStyle;
    use crate::layout::clef::ClefLayout;
    use crate::layout::key_signature::KeySignature;
    use crate::layout::measure::{layout_measure, MeasureLayoutConfig, RestEvent};
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
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 3,
                dots: 0,
                accidental: Some(Glyph::AccidentalNatural),
                stem_direction: Some(StemDirection::Down),
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
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 notehead, 1 stem + 1 ledger line = 2 lines
        assert_eq!(output.matches("<path ").count(), 1);
        assert_eq!(output.matches("<line ").count(), 2, "stem + 1 ledger line");
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
        })];
        let elements_down = vec![MeasureElement::Note(NoteEvent {
            staff_position: 0,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: Some(StemDirection::Down),
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
}
