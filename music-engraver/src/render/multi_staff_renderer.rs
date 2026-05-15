//! Rendering for multi-staff connectors: braces and brackets.
//!
//! Draws the visual connectors that group staves at the left edge of a system.

use crate::font::{FontError, MusicFont};
use crate::layout::multi_staff::{BraceLayout, BracketLayout, MultiStaffLayout};
use crate::render::SvgWriter;

/// Draw a brace connector (curly brace for piano/keyboard).
///
/// The brace glyph is vertically scaled to span exactly from `brace.y_top`
/// (top of first staff) to `brace.y_bottom` (bottom of last staff). The
/// scale factor is derived from the font's actual brace bbox height (read
/// from SMuFL metadata) — this keeps brace rendering correct across
/// different SMuFL fonts whose brace glyph may have different design
/// heights. (Bravura's brace is ~3.988 staff-spaces tall; an earlier
/// implementation incorrectly assumed 1 staff-space, producing a brace ~4×
/// too large and offset upward off the staff system.)
///
/// Falls back to scaling against 1 staff space (the previous, font-specific
/// assumption) if the font's metadata lacks bbox data for the brace glyph —
/// in that case the rendering may be slightly off, but the brace will at
/// least be drawn.
pub fn draw_brace(
    svg: &mut SvgWriter,
    font: &MusicFont,
    brace: &BraceLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(brace.glyph)?;

    // Bravura's brace path runs from y=0 (bottom, at the glyph's origin) up
    // to y = -bbox.height() (top, after the path's y-flip into SVG space).
    // We want the brace's bottom edge at `y_bottom` and top edge at `y_top`.
    //
    // SVG `transform="translate(tx,ty) scale(1,sy)"` composes right-to-left:
    // a path point (x, py) becomes (x + tx, ty + sy*py). So a path-space
    // point at py=0 lands at SVG y = ty, and py = -h lands at ty - sy*h.
    //
    // Solving: ty = y_bottom and sy = (y_bottom - y_top) / h, where h is the
    // brace glyph's bbox height in design units.
    let span = brace.y_bottom - brace.y_top;
    let glyph_height = font
        .glyph_bbox_design_units(brace.glyph)
        .map(|bb| bb.height())
        // Fallback: the SMuFL recommended default has no brace-specific
        // value, so we use the long-standing (incorrect-for-Bravura)
        // assumption of 1 staff space. This branch is unreachable for the
        // bundled Bravura font; it exists for robustness against custom
        // metadata-stripped fonts.
        .filter(|h| *h > 0.0)
        .unwrap_or_else(|| font.engraving_config().staff_space);
    let scale_y = span / glyph_height;
    let ty = brace.y_bottom;

    let transform = format!("translate({},{}) scale(1,{})", brace.x, ty, scale_y);

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
        assert!(
            output.contains("scale(1,"),
            "brace transform should contain scale(1,<y>)"
        );
    }

    /// Locks in the corrected brace geometry: the brace glyph's bottom edge
    /// must land at `y_bottom` (bottom of staff system), and `scale_y` must
    /// be derived from the font's actual brace bbox height. A regression
    /// that reverts to "design_height = 1 staff_space" would set `scale_y`
    /// to the span-in-staff-spaces value (14 for a default grand staff),
    /// quadruple the correct ~3.51, and fire this test.
    #[test]
    fn brace_transform_scale_y_matches_font_bbox_height() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let brace = layout.brace.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 4000.0);
        draw_brace(&mut svg, &font, brace).unwrap();
        let output = svg.to_svg();

        // Compute expected scale_y from the actual font bbox so the test
        // stays font-agnostic. For Bravura: span = 14 ss × 250 = 3500;
        // brace height ≈ 997 design units → scale_y ≈ 3.5106.
        let bbox = font.glyph_bbox_design_units(brace.glyph).unwrap();
        let expected_scale = brace.span_height() / bbox.height();
        let expected_substr = format!("scale(1,{})", expected_scale);
        assert!(
            output.contains(&expected_substr),
            "brace scale: expected substring `{expected_substr}`; SVG:\n{output}"
        );

        // Confirm the correct value is comfortably distinct from the old
        // buggy value (14 for a default grand staff). A regression to the
        // pre-fix assumption "design_height = 1 staff space" would yield a
        // scale_y ≈ 14, which is roughly 4× the correct value.
        assert!(
            expected_scale > 3.0 && expected_scale < 4.0,
            "expected scale_y ≈ 3.51 for Bravura grand staff, got {expected_scale}"
        );
    }

    /// Locks in the corrected translate y: the brace glyph's origin (its
    /// bbox SW corner, the bottom in font space) is anchored to `y_bottom`.
    /// Combined with the correct scale_y this means the brace top lands at
    /// `y_top` exactly. The pre-fix code translated to `y_top` instead,
    /// which placed the brace bottom at the top of the staff system and
    /// made the rest of the brace extend off-screen above the music.
    #[test]
    fn brace_transform_translate_y_at_staff_system_bottom() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let brace = layout.brace.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 4000.0);
        draw_brace(&mut svg, &font, brace).unwrap();
        let output = svg.to_svg();

        // Look for the translate component: `translate(brace.x, brace.y_bottom)`.
        let needle = format!("translate({},{})", brace.x, brace.y_bottom);
        assert!(
            output.contains(&needle),
            "brace should translate to (x, y_bottom = {}); SVG:\n{output}",
            brace.y_bottom
        );
        // And NOT to y_top (the pre-fix behavior).
        let bad_needle = format!("translate({},{})", brace.x, brace.y_top);
        assert!(
            !output.contains(&bad_needle),
            "brace must not translate to y_top (= {}); that was the pre-fix bug",
            brace.y_top
        );
    }

    /// Empty-metadata fallback: if a font supplies no brace bbox, the
    /// renderer still emits a brace (with a fallback scale) rather than
    /// failing or panicking. The scale is the previous (incorrect)
    /// assumption — kept so a font missing only its bbox metadata still
    /// renders, just slightly off.
    #[test]
    fn brace_render_falls_back_when_metadata_missing_bbox() {
        use crate::font::{MusicFont, BRAVURA_OTF};
        let empty_metadata = br#"{"fontName":"Empty"}"#;
        let font = MusicFont::new(BRAVURA_OTF, empty_metadata).unwrap();

        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let brace = layout.brace.as_ref().unwrap();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, -50.0, 5500.0, 3500.0);
        // Must not panic; must emit a path.
        draw_brace(&mut svg, &font, brace).unwrap();
        let output = svg.to_svg();
        assert!(
            output.contains("<path"),
            "brace path should be emitted even without bbox metadata"
        );
        // Fallback uses staff_space as glyph_height, so scale_y == span/ss == 14.
        let span = brace.y_bottom - brace.y_top;
        let staff_space = font.engraving_config().staff_space;
        let expected_fallback_scale = span / staff_space;
        let needle = format!("scale(1,{})", expected_fallback_scale);
        assert!(
            output.contains(&needle),
            "fallback scale should be span/staff_space = {expected_fallback_scale}; SVG:\n{output}"
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
