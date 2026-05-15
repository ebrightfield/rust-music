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

/// Draw a bracket connector (orchestral section grouping).
///
/// Renders a thick vertical line spanning `y_top..y_bottom`, with SMuFL
/// `bracketTop` and `bracketBottom` scroll glyphs anchored at the line's
/// endpoints — the published-engraving convention (Gould, Behind Bars) for
/// section brackets. Each scroll glyph extends outward (above `y_top` or
/// below `y_bottom`) and joins the thick line at its origin point, so no
/// horizontal serif stroke is needed.
///
/// Returns an error only if the bundled font is missing one of the two
/// scroll glyphs — caught by unit tests against the Bravura bundle.
pub fn draw_bracket(
    svg: &mut SvgWriter,
    font: &MusicFont,
    bracket: &BracketLayout,
) -> Result<(), FontError> {
    // Thick vertical line: y_top..y_bottom inclusive. The scroll glyphs
    // attach at the line's endpoints (their origins sit at y_top / y_bottom),
    // so this segment together with the two glyph paths forms a continuous
    // bracket without a visible seam.
    svg.add_line(
        bracket.x + bracket.thickness / 2.0,
        bracket.y_top,
        bracket.x + bracket.thickness / 2.0,
        bracket.y_bottom,
        "black",
        bracket.thickness,
    );

    // Decorative top scroll. The glyph's origin (font convention: bBoxSW for
    // bracketTop) sits at the line's top edge; the outline extends upward
    // into the area above the top staff.
    let top_outline = font.glyph_outline(bracket.top_glyph)?;
    let top_transform = format!("translate({},{})", bracket.x, bracket.y_top);
    svg.add_path(&top_outline.path_data, "black", Some(&top_transform));

    // Decorative bottom scroll. The glyph's origin (font convention: bBoxNW
    // for bracketBottom — i.e. the bbox extends downward from the origin)
    // sits at the line's bottom edge; the outline extends below the bottom
    // staff.
    let bottom_outline = font.glyph_outline(bracket.bottom_glyph)?;
    let bottom_transform = format!("translate({},{})", bracket.x, bracket.y_bottom);
    svg.add_path(&bottom_outline.path_data, "black", Some(&bottom_transform));

    Ok(())
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
        draw_bracket(svg, font, bracket)?;
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
    fn draw_bracket_produces_one_line_and_two_glyph_paths() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();

        let output = svg.to_svg();
        let line_count = output.matches("<line").count();
        let path_count = output.matches("<path").count();
        // Exactly 1 line (the thick vertical) + 2 glyph paths (bracketTop +
        // bracketBottom). Catches a regression that reverts to serif lines
        // (which would emit 3 lines and 0 paths) or that draws either glyph
        // twice.
        assert_eq!(
            line_count, 1,
            "bracket should produce exactly 1 line (vertical), got {line_count}"
        );
        assert_eq!(
            path_count, 2,
            "bracket should produce exactly 2 glyph paths (top + bottom scroll), got {path_count}"
        );
    }

    #[test]
    fn draw_bracket_top_glyph_anchored_at_y_top() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        // The top scroll glyph must be translated to (bracket.x, bracket.y_top).
        // y_top is 100.0 (the y_start passed to layout_multi_staff).
        let needle = format!("translate({},{})", bracket.x, bracket.y_top);
        assert!(
            output.contains(&needle),
            "top scroll must be translated to (x={}, y={}); SVG:\n{output}",
            bracket.x,
            bracket.y_top
        );
    }

    #[test]
    fn draw_bracket_bottom_glyph_anchored_at_y_bottom() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        let needle = format!("translate({},{})", bracket.x, bracket.y_bottom);
        assert!(
            output.contains(&needle),
            "bottom scroll must be translated to (x={}, y={}); SVG:\n{output}",
            bracket.x,
            bracket.y_bottom
        );
    }

    #[test]
    fn draw_bracket_top_and_bottom_glyphs_render_distinct_outlines() {
        // The two scroll glyphs curl in opposite vertical directions, so their
        // path_data must differ. Catches a regression where both glyph paths
        // accidentally pick up the same outline (e.g. swapping `top_glyph`
        // into `bottom_glyph` slot, or reusing the same Glyph variant).
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        // Split on `<path ` and extract each path's `d="..."` attribute.
        let parts: Vec<&str> = output.split("<path ").collect();
        assert_eq!(parts.len(), 3, "expected 2 path elements (plus 1 head split)");
        let extract_d = |s: &str| -> String {
            let d_start = s.find("d=\"").expect("path missing d attribute") + 3;
            let d_rest = &s[d_start..];
            let d_end = d_rest.find('"').expect("unterminated d attribute");
            d_rest[..d_end].to_string()
        };
        let d_top = extract_d(parts[1]);
        let d_bottom = extract_d(parts[2]);
        assert_ne!(
            d_top, d_bottom,
            "bracketTop and bracketBottom outlines must differ; got identical path data"
        );
        // Sanity: both must be non-empty SVG path commands.
        assert!(d_top.starts_with('M'), "top path must start with moveto");
        assert!(d_bottom.starts_with('M'), "bottom path must start with moveto");
    }

    #[test]
    fn draw_bracket_vertical_line_uses_thickness() {
        // Stroke width on the single line element must be `bracket.thickness`.
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        let needle = format!("stroke-width=\"{}\"", bracket.thickness);
        assert!(
            output.contains(&needle),
            "vertical line must use stroke-width={}; SVG:\n{output}",
            bracket.thickness
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
        let path_count = output.matches("<path").count();
        assert_eq!(
            line_count, 1,
            "section bracket should produce exactly 1 line (thick vertical), got {line_count}"
        );
        assert_eq!(
            path_count, 2,
            "section bracket should produce exactly 2 scroll-glyph paths, got {path_count}"
        );
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
