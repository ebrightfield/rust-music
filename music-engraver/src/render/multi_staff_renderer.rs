//! Rendering for multi-staff connectors: braces and brackets.
//!
//! Draws the visual connectors that group staves at the left edge of a system.

use crate::font::{FontError, MusicFont};
use crate::layout::multi_staff::{BraceLayout, BracketLayout, MultiStaffLayout};
use crate::render::SvgWriter;

/// Draw a brace connector (curly brace for piano/keyboard).
///
/// The brace glyph is vertically scaled to span the distance from the top
/// of the first staff to the bottom of the last staff. It is positioned to
/// the left of the staff system at the vertical midpoint.
pub fn draw_brace(
    svg: &mut SvgWriter,
    font: &MusicFont,
    brace: &BraceLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(brace.glyph)?;

    // Build a transform that scales the glyph vertically and positions it.
    // The brace glyph origin is at its top; we translate to center it.
    let transform = format!(
        "translate({},{}) scale(1,{})",
        brace.x,
        brace.y_center - brace.span_height / 2.0,
        brace.scale_y
    );

    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

/// Draw a bracket connector (thick line with serifs for orchestral sections).
///
/// Renders a thick vertical line from `y_top` to `y_bottom`, with short
/// horizontal serif lines at both ends extending to the right (toward the staff).
pub fn draw_bracket(svg: &mut SvgWriter, bracket: &BracketLayout) {
    // Thick vertical line
    svg.add_line(
        bracket.x + bracket.thickness / 2.0,
        bracket.y_top,
        bracket.x + bracket.thickness / 2.0,
        bracket.y_bottom,
        "black",
        bracket.thickness,
    );

    // Top serif (extends rightward)
    svg.add_line(
        bracket.x,
        bracket.y_top + bracket.serif_thickness / 2.0,
        bracket.x + bracket.serif_length,
        bracket.y_top + bracket.serif_thickness / 2.0,
        "black",
        bracket.serif_thickness,
    );

    // Bottom serif (extends rightward)
    svg.add_line(
        bracket.x,
        bracket.y_bottom - bracket.serif_thickness / 2.0,
        bracket.x + bracket.serif_length,
        bracket.y_bottom - bracket.serif_thickness / 2.0,
        "black",
        bracket.serif_thickness,
    );
}

/// Draw all connectors for a multi-staff layout.
///
/// Dispatches to [`draw_brace`] or [`draw_bracket`] based on the layout.
pub fn draw_multi_staff_connectors(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &MultiStaffLayout,
) -> Result<(), FontError> {
    if let Some(ref brace) = layout.brace {
        draw_brace(svg, font, brace)?;
    }
    if let Some(ref bracket) = layout.bracket {
        draw_bracket(svg, bracket);
    }
    Ok(())
}

/// Draw a joined barline through all staves in a multi-staff group.
///
/// Renders a single barline from the top of the first staff to the bottom
/// of the last staff, connecting them visually.
pub fn draw_joined_barline(
    svg: &mut SvgWriter,
    x: f64,
    y_top: f64,
    y_bottom: f64,
    thickness: f64,
) {
    svg.add_line(x, y_top, x, y_bottom, "black", thickness);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::multi_staff::{layout_multi_staff, StaffGroup};

    const SS: f64 = 250.0;

    #[test]
    fn draw_brace_produces_path_element() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let brace = layout.brace.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_brace(&mut svg, &font, brace).unwrap();

        let output = svg.to_svg();
        assert!(output.contains("<path"), "brace should produce a <path> element");
        assert!(output.contains("translate("), "brace should have a transform");
        assert!(output.contains("scale("), "brace should have vertical scaling");
    }

    #[test]
    fn draw_bracket_produces_three_lines() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, bracket);

        let output = svg.to_svg();
        let line_count = output.matches("<line").count();
        assert_eq!(
            line_count, 3,
            "bracket should produce 3 lines (1 vertical + 2 serifs), got {line_count}"
        );
    }

    #[test]
    fn draw_multi_staff_connectors_brace() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let font = bravura_font();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_multi_staff_connectors(&mut svg, &font, &layout).unwrap();

        let output = svg.to_svg();
        assert!(output.contains("<path"), "grand staff should have brace path");
    }

    #[test]
    fn draw_multi_staff_connectors_bracket() {
        let group = StaffGroup::section(3);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let font = bravura_font();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_multi_staff_connectors(&mut svg, &font, &layout).unwrap();

        let output = svg.to_svg();
        let line_count = output.matches("<line").count();
        assert_eq!(line_count, 3, "section bracket should produce 3 lines");
    }

    #[test]
    fn draw_multi_staff_connectors_none() {
        let group = StaffGroup::independent(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let font = bravura_font();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_multi_staff_connectors(&mut svg, &font, &layout).unwrap();

        let output = svg.to_svg();
        assert!(
            !output.contains("<path"),
            "independent group should have no path"
        );
        assert!(
            !output.contains("<line"),
            "independent group should have no lines"
        );
    }

    #[test]
    fn draw_joined_barline_produces_line() {
        let mut svg = SvgWriter::new(200.0, 600.0, 0.0, 0.0, 5500.0, 3500.0);
        draw_joined_barline(&mut svg, 100.0, 0.0, 2500.0, 4.0);

        let output = svg.to_svg();
        assert!(output.contains("<line"), "joined barline should produce a line");
    }

    #[test]
    fn brace_and_bracket_produce_different_output() {
        let font = bravura_font();

        let brace_group = StaffGroup::grand_staff();
        let brace_layout = layout_multi_staff(&brace_group, 0.0, SS, 5000.0);
        let mut svg1 = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_multi_staff_connectors(&mut svg1, &font, &brace_layout).unwrap();

        let bracket_group = StaffGroup::section(2);
        let bracket_layout = layout_multi_staff(&bracket_group, 0.0, SS, 5000.0);
        let mut svg2 = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_multi_staff_connectors(&mut svg2, &font, &bracket_layout).unwrap();

        assert_ne!(
            svg1.to_svg(),
            svg2.to_svg(),
            "brace and bracket should produce different SVG output"
        );
    }

    #[test]
    fn brace_transform_contains_scale_y() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let brace = layout.brace.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_brace(&mut svg, &font, brace).unwrap();

        let output = svg.to_svg();
        // The scale_y should be > 1 since we're scaling from 1 staff space
        // to the full span across 2 staves + gap
        assert!(
            output.contains("scale(1,"),
            "brace transform should contain scale(1,<y>)"
        );
    }

    #[test]
    fn bracket_vertical_line_spans_full_range() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        // The y-coordinates in the bracket should span from first staff top
        // to last staff bottom
        assert_eq!(bracket.y_top, 100.0);
        let expected_bottom = layout.staff_y_origins[1] + SS * 4.0;
        assert!(
            (bracket.y_bottom - expected_bottom).abs() < 0.01,
            "bracket should span to bottom of last staff"
        );
    }
}
