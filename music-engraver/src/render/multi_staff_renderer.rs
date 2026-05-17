//! Rendering for multi-staff connectors: braces and brackets.
//!
//! Draws the visual connectors that group staves at the left edge of a system.

use crate::font::{FontError, MusicFont};
use crate::layout::multi_staff::{BraceLayout, BracketLayout, MultiStaffLayout, SubBracketLayout};
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
/// section brackets.
///
/// **Metric-driven anchoring** (parallel to the brace bbox fix): rather than
/// assuming the scroll glyph's origin sits at a specific bbox corner
/// (Bravura's convention is bBoxSW for `bracketTop` and bBoxNW for
/// `bracketBottom`), the renderer queries each glyph's actual bbox from the
/// font's SMuFL metadata and computes a translate that places the glyph's
/// inner edge — the edge meeting the thick line — exactly at `y_top` /
/// `y_bottom`. The same applies horizontally: the glyph's left edge is
/// aligned to `bracket.x`. This keeps the seam clean for any conforming
/// SMuFL font, even one whose bracket-scroll glyph origin diverges from the
/// Bravura convention. For Bravura specifically (bracketTop bbox.x_left=0,
/// bbox.y_bottom=0; bracketBottom bbox.x_left=0, bbox.y_top=0) the formula
/// reduces to the previous `translate(bracket.x, bracket.y_top|y_bottom)`
/// pattern, so output is byte-identical.
///
/// Falls back to the corner-anchored translate (the pre-fix behavior) when
/// the font's metadata supplies no bbox for a scroll glyph — the bracket
/// still renders, just under the SMuFL-convention assumption.
///
/// Returns an error only if the bundled font is missing one of the two
/// scroll glyphs — caught by unit tests against the Bravura bundle.
pub fn draw_bracket(
    svg: &mut SvgWriter,
    font: &MusicFont,
    bracket: &BracketLayout,
) -> Result<(), FontError> {
    // Thick vertical line: y_top..y_bottom inclusive. The scroll glyphs
    // attach at the line's endpoints, so this segment together with the two
    // glyph paths forms a continuous bracket without a visible seam.
    svg.add_line(
        bracket.x + bracket.thickness / 2.0,
        bracket.y_top,
        bracket.x + bracket.thickness / 2.0,
        bracket.y_bottom,
        "black",
        bracket.thickness,
    );

    // Decorative top scroll. We want the glyph's *bottom* edge (where it
    // meets the thick line) to land at SVG y = bracket.y_top, and its left
    // edge to land at SVG x = bracket.x. With a `translate(tx, ty)` the
    // glyph's path point (px, py) lands at (tx + px, ty + py). The glyph's
    // bottom in SVG-flipped path space is at py = bbox.y_bottom (the largest
    // y, since SVG y grows downward); its left is at px = bbox.x_left. So:
    //   tx = bracket.x      - bbox.x_left
    //   ty = bracket.y_top  - bbox.y_bottom
    let top_outline = font.glyph_outline(bracket.top_glyph)?;
    let (top_tx, top_ty) = bracket_anchor(
        font,
        bracket.top_glyph,
        bracket.x,
        bracket.y_top,
        ScrollEnd::Top,
    );
    let top_transform = format!("translate({},{})", top_tx, top_ty);
    svg.add_path(&top_outline.path_data, "black", Some(&top_transform));

    // Decorative bottom scroll. Symmetric: the glyph's *top* edge meets the
    // line at SVG y = bracket.y_bottom; its left edge at SVG x = bracket.x.
    // The glyph's top in SVG-flipped path space is at py = bbox.y_top. So:
    //   tx = bracket.x        - bbox.x_left
    //   ty = bracket.y_bottom - bbox.y_top
    let bottom_outline = font.glyph_outline(bracket.bottom_glyph)?;
    let (bot_tx, bot_ty) = bracket_anchor(
        font,
        bracket.bottom_glyph,
        bracket.x,
        bracket.y_bottom,
        ScrollEnd::Bottom,
    );
    let bottom_transform = format!("translate({},{})", bot_tx, bot_ty);
    svg.add_path(&bottom_outline.path_data, "black", Some(&bottom_transform));

    Ok(())
}

/// Which end of the bracket the scroll glyph attaches to. Determines which
/// edge of the glyph's bbox is treated as the "inner" edge meeting the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrollEnd {
    /// Glyph sits above its anchor; its **bottom** edge meets the line.
    Top,
    /// Glyph sits below its anchor; its **top** edge meets the line.
    Bottom,
}

/// Compute the `(tx, ty)` translate values that align a bracket scroll
/// glyph's inner edge with `(line_x, line_y)`, using the glyph's bbox from
/// the font's SMuFL metadata.
///
/// For fonts whose metadata supplies no bbox for the glyph, falls back to
/// `(line_x, line_y)` — the corner-anchored translate that assumes the
/// SMuFL bracket-scroll convention (bBoxSW for `bracketTop`, bBoxNW for
/// `bracketBottom`). The fallback keeps rendering nonzero for
/// metadata-stripped custom fonts.
fn bracket_anchor(
    font: &MusicFont,
    glyph: smufl::Glyph,
    line_x: f64,
    line_y: f64,
    end: ScrollEnd,
) -> (f64, f64) {
    match font.glyph_bbox_design_units(glyph) {
        Some(bbox) => {
            let tx = line_x - bbox.x_left;
            let ty = match end {
                ScrollEnd::Top => line_y - bbox.y_bottom,
                ScrollEnd::Bottom => line_y - bbox.y_top,
            };
            (tx, ty)
        }
        None => (line_x, line_y),
    }
}

/// Draw a nested sub-bracket as a thin vertical line.
///
/// Sub-brackets indicate two-deep grouping within a section bracket (e.g.
/// Violin I + Violin II nested inside a string-section bracket). Per the
/// engraving convention adopted here (Behind Bars; Lilypond's `StaffGroup`
/// inside `StaffGroup` rendering), the inner bracket has **no scroll
/// decoration** — only a thinner vertical line sized by SMuFL's
/// `subBracketThickness` engraving default. This keeps the nesting reading
/// as visually subordinate to the main bracket's scrolled outline.
///
/// Stroke geometry mirrors the main bracket's: the line is drawn centred at
/// `sub.x + sub.thickness / 2.0`, so the stroked region extends from
/// `sub.x` (left edge) to `sub.x + sub.thickness` (right edge).
pub fn draw_sub_bracket(svg: &mut SvgWriter, sub: &SubBracketLayout) {
    svg.add_line(
        sub.x + sub.thickness / 2.0,
        sub.y_top,
        sub.x + sub.thickness / 2.0,
        sub.y_bottom,
        "black",
        sub.thickness,
    );
}

/// Draw all connectors for a multi-staff layout.
///
/// Dispatches to [`draw_brace`] / [`draw_bracket`] / [`draw_sub_bracket`]
/// based on the layout's optional fields and sub-bracket list.
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
    for sub in &layout.sub_brackets {
        draw_sub_bracket(svg, sub);
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
    use crate::layout::multi_staff::{
        layout_multi_staff, StaffGroup, SubBracket, SUB_BRACKET_THICKNESS_SS,
    };

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

        // For Bravura the metric-driven formula (translate to
        // `bracket.x - bbox.x_left, bracket.y_top - bbox.y_bottom`) reduces
        // to `translate(bracket.x, bracket.y_top)` since Bravura's
        // bracketTop bbox has x_left=0 and y_bottom=0. So this Bravura test
        // continues to assert against the fixed substring while
        // `bracket_top_anchor_uses_glyph_bbox_y_bottom` (below) proves the
        // formula actually consults the bbox by using a synthetic font.
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

        // Bravura's bracketBottom bbox has x_left=0 and y_top=0, so the
        // metric-driven formula (translate to `bracket.x - bbox.x_left,
        // bracket.y_bottom - bbox.y_top`) reduces to `translate(bracket.x,
        // bracket.y_bottom)`. See the synthetic-font test below for
        // formula-locking.
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

    /// Bravura is exactly the convention case (bbox.x_left=0, bbox.y_bottom=0
    /// for bracketTop), so the *value* of the translate hasn't changed —
    /// what changed is that the value is now derived from the bbox rather
    /// than written as `(bracket.x, bracket.y_top)` literally. This test
    /// proves the derivation is in effect by asserting the SVG translate
    /// matches the bbox-driven formula evaluated against the live font.
    #[test]
    fn bracket_top_translate_matches_metric_driven_formula() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        let bbox = font
            .glyph_bbox_design_units(bracket.top_glyph)
            .expect("Bravura supplies bracketTop bbox");
        let expected_tx = bracket.x - bbox.x_left;
        let expected_ty = bracket.y_top - bbox.y_bottom;
        let needle = format!("translate({},{})", expected_tx, expected_ty);
        assert!(
            output.contains(&needle),
            "bracketTop translate must be `{needle}`; SVG:\n{output}"
        );
    }

    #[test]
    fn bracket_bottom_translate_matches_metric_driven_formula() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let font = bravura_font();
        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        let bbox = font
            .glyph_bbox_design_units(bracket.bottom_glyph)
            .expect("Bravura supplies bracketBottom bbox");
        let expected_tx = bracket.x - bbox.x_left;
        let expected_ty = bracket.y_bottom - bbox.y_top;
        let needle = format!("translate({},{})", expected_tx, expected_ty);
        assert!(
            output.contains(&needle),
            "bracketBottom translate must be `{needle}`; SVG:\n{output}"
        );
    }

    /// A synthetic font whose bracketTop bbox has a non-zero `bBoxSW.y`
    /// proves the renderer actually consults the bbox: a regression that
    /// reverted to `translate(bracket.x, bracket.y_top)` would mis-anchor
    /// the glyph (ignoring the offset) and this test would fire.
    ///
    /// The synthetic font reuses Bravura's OTF (so `glyph_outline` still
    /// works) but supplies custom `glyphBBoxes` for `bracketTop` and
    /// `bracketBottom`. Bravura's defaults are bracketTop SW=(0,0)/NE=(1.876,
    /// 1.18) and bracketBottom SW=(0,-1.18)/NE=(1.876,0); we shift each
    /// inner-edge corner by 0.5 staff-spaces (= 125 design units) in the
    /// inner-edge direction to produce a measurable, non-zero offset.
    #[test]
    fn bracket_top_anchor_uses_glyph_bbox_y_bottom_when_nonzero() {
        use crate::font::{MusicFont, BRAVURA_OTF};
        // bracketTop with bBoxSW.y = -0.5 (instead of 0) → after y-flip the
        // bbox.y_bottom becomes +125 (instead of 0). The renderer should
        // translate to (bracket.x - 0, bracket.y_top - 125).
        let custom_metadata = br#"{
            "fontName": "Synthetic",
            "glyphBBoxes": {
                "bracketTop": {
                    "bBoxSW": [0.0, -0.5],
                    "bBoxNE": [1.876, 1.18]
                },
                "bracketBottom": {
                    "bBoxSW": [0.0, -1.18],
                    "bBoxNE": [1.876, 0.0]
                }
            }
        }"#;
        let font = MusicFont::new(BRAVURA_OTF, custom_metadata).unwrap();

        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        // Expected ty = bracket.y_top - 125. (UPM=1000, ss = UPM/4 = 250,
        // 0.5 * 250 = 125; with y-flip, sw.y = -0.5 → bbox.y_bottom = +125.)
        let bbox = font.glyph_bbox_design_units(bracket.top_glyph).unwrap();
        assert!(
            (bbox.y_bottom - 125.0).abs() < 0.01,
            "synthetic bbox.y_bottom should be 125, got {}",
            bbox.y_bottom
        );
        let expected_ty = bracket.y_top - 125.0;
        let needle = format!("translate({},{})", bracket.x, expected_ty);
        assert!(
            output.contains(&needle),
            "expected metric-driven translate `{needle}`; SVG:\n{output}"
        );
        // Anti-needle: the corner-anchored translate (the regression
        // pattern) must NOT appear in the SVG for this synthetic font.
        let bad_needle = format!("translate({},{})", bracket.x, bracket.y_top);
        assert!(
            !output.contains(&bad_needle),
            "regression: bracketTop must not anchor at raw y_top when bbox.y_bottom != 0"
        );
    }

    #[test]
    fn bracket_bottom_anchor_uses_glyph_bbox_y_top_when_nonzero() {
        use crate::font::{MusicFont, BRAVURA_OTF};
        // bracketBottom with bBoxNE.y = +0.5 (instead of 0) → after y-flip
        // the bbox.y_top becomes -125 (instead of 0). The renderer should
        // translate to (bracket.x, bracket.y_bottom - (-125)) = (x, y_bottom
        // + 125).
        let custom_metadata = br#"{
            "fontName": "Synthetic",
            "glyphBBoxes": {
                "bracketTop": {
                    "bBoxSW": [0.0, 0.0],
                    "bBoxNE": [1.876, 1.18]
                },
                "bracketBottom": {
                    "bBoxSW": [0.0, -1.18],
                    "bBoxNE": [1.876, 0.5]
                }
            }
        }"#;
        let font = MusicFont::new(BRAVURA_OTF, custom_metadata).unwrap();

        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        let bbox = font.glyph_bbox_design_units(bracket.bottom_glyph).unwrap();
        assert!(
            (bbox.y_top - (-125.0)).abs() < 0.01,
            "synthetic bbox.y_top should be -125, got {}",
            bbox.y_top
        );
        let expected_ty = bracket.y_bottom + 125.0; // y_bottom - (-125)
        let needle = format!("translate({},{})", bracket.x, expected_ty);
        assert!(
            output.contains(&needle),
            "expected metric-driven translate `{needle}`; SVG:\n{output}"
        );
        let bad_needle = format!("translate({},{})", bracket.x, bracket.y_bottom);
        assert!(
            !output.contains(&bad_needle),
            "regression: bracketBottom must not anchor at raw y_bottom when bbox.y_top != 0"
        );
    }

    /// The x-axis half of the metric-driven anchor: a synthetic font whose
    /// scroll glyphs have `bBoxSW.x != 0` shifts the translate's x by
    /// `-bbox.x_left` so the glyph's left edge still lands at `bracket.x`.
    #[test]
    fn bracket_anchor_uses_glyph_bbox_x_left_when_nonzero() {
        use crate::font::{MusicFont, BRAVURA_OTF};
        // bracketTop with bBoxSW.x = 0.4 (instead of 0) → bbox.x_left = 100
        // (0.4 sp × 250 design units/sp). Renderer should translate to
        // (bracket.x - 100, ...) so the glyph's left edge sits at bracket.x.
        let custom_metadata = br#"{
            "fontName": "Synthetic",
            "glyphBBoxes": {
                "bracketTop": {
                    "bBoxSW": [0.4, 0.0],
                    "bBoxNE": [1.876, 1.18]
                },
                "bracketBottom": {
                    "bBoxSW": [0.0, -1.18],
                    "bBoxNE": [1.876, 0.0]
                }
            }
        }"#;
        let font = MusicFont::new(BRAVURA_OTF, custom_metadata).unwrap();

        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        let bbox = font.glyph_bbox_design_units(bracket.top_glyph).unwrap();
        assert!(
            (bbox.x_left - 100.0).abs() < 0.01,
            "synthetic bbox.x_left should be 100, got {}",
            bbox.x_left
        );
        let expected_tx = bracket.x - 100.0;
        let needle = format!("translate({},{})", expected_tx, bracket.y_top);
        assert!(
            output.contains(&needle),
            "expected metric-driven x translate `{needle}`; SVG:\n{output}"
        );
    }

    /// Empty-metadata fallback: if a font's bbox lookup returns None for a
    /// scroll glyph, the renderer falls back to `translate(bracket.x,
    /// bracket.y_top|y_bottom)` — the corner-anchored placement that
    /// assumes the SMuFL-canonical bracket-scroll convention. The bracket
    /// still renders rather than panicking.
    #[test]
    fn bracket_render_falls_back_when_metadata_missing_bbox() {
        use crate::font::{MusicFont, BRAVURA_OTF};
        let empty_metadata = br#"{"fontName":"Empty"}"#;
        let font = MusicFont::new(BRAVURA_OTF, empty_metadata).unwrap();

        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        let mut svg = SvgWriter::new(200.0, 600.0, -200.0, 0.0, 5500.0, 3500.0);
        // Must not panic; must emit two paths and one line.
        draw_bracket(&mut svg, &font, bracket).unwrap();
        let output = svg.to_svg();

        let path_count = output.matches("<path").count();
        let line_count = output.matches("<line").count();
        assert_eq!(path_count, 2, "fallback should still emit 2 scroll paths");
        assert_eq!(line_count, 1, "fallback should still emit the thick line");

        // Fallback anchors at corner: bracket.x for x, raw y_top / y_bottom
        // for y. Both must appear in the SVG.
        let top_needle = format!("translate({},{})", bracket.x, bracket.y_top);
        let bot_needle = format!("translate({},{})", bracket.x, bracket.y_bottom);
        assert!(
            output.contains(&top_needle),
            "fallback top translate `{top_needle}` not found; SVG:\n{output}"
        );
        assert!(
            output.contains(&bot_needle),
            "fallback bottom translate `{bot_needle}` not found; SVG:\n{output}"
        );
    }

    // ---------- Sub-bracket rendering ----------

    #[test]
    fn draw_sub_bracket_produces_one_thin_line() {
        // A sub-bracket should render as a single thin vertical line — no
        // scroll glyphs, no serifs. Catches a regression that accidentally
        // attaches scroll paths or extra horizontal serifs to the inner
        // bracket (defeating the engraving convention that the inner bracket
        // reads as subordinate).
        let group = StaffGroup::section(3)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let sub = &layout.sub_brackets[0];

        let mut svg = SvgWriter::new(200.0, 600.0, -400.0, -50.0, 5500.0, 4000.0);
        draw_sub_bracket(&mut svg, sub);
        let output = svg.to_svg();

        let path_count = output.matches("<path").count();
        let line_count = output.matches("<line").count();
        assert_eq!(line_count, 1, "sub-bracket: exactly 1 line, got {line_count}");
        assert_eq!(path_count, 0, "sub-bracket: no paths (no scrolls), got {path_count}");
    }

    #[test]
    fn draw_sub_bracket_uses_sub_bracket_thickness() {
        // Stroke width must equal `sub.thickness` (≈ 0.16 ss). A regression
        // that hard-coded `bracket_thickness` (0.5 ss) for the inner bracket
        // would visually merge with the outer one.
        let group = StaffGroup::section(3)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let sub = &layout.sub_brackets[0];

        let mut svg = SvgWriter::new(200.0, 600.0, -400.0, -50.0, 5500.0, 4000.0);
        draw_sub_bracket(&mut svg, sub);
        let output = svg.to_svg();

        let needle = format!("stroke-width=\"{}\"", sub.thickness);
        assert!(
            output.contains(&needle),
            "sub-bracket must use stroke-width={}; SVG:\n{output}",
            sub.thickness
        );
        // Sanity: the thickness substring corresponds to the SMuFL
        // `subBracketThickness` default (0.16 ss × SS = 40 design units).
        let expected = SUB_BRACKET_THICKNESS_SS * SS;
        assert!(
            (sub.thickness - expected).abs() < 1e-6,
            "sub.thickness layout drift: expected {expected}, got {}",
            sub.thickness
        );
    }

    #[test]
    fn draw_sub_bracket_line_spans_y_range() {
        // The drawn line's `y1`/`y2` SVG attributes must match the layout's
        // `y_top`/`y_bottom`. A regression that swaps the endpoints (or only
        // draws half the line) is caught here. The x coordinate is the
        // stroke centre — sub.x + thickness/2.
        let group = StaffGroup::section(4)
            .with_sub_brackets(vec![SubBracket { start_index: 1, staff_count: 3 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let sub = &layout.sub_brackets[0];

        let mut svg = SvgWriter::new(200.0, 600.0, -400.0, -50.0, 5500.0, 6000.0);
        draw_sub_bracket(&mut svg, sub);
        let output = svg.to_svg();

        let x_centre = sub.x + sub.thickness / 2.0;
        // SvgWriter::add_line emits attributes in the order x1, y1, x2, y2.
        let needle_x1 = format!("x1=\"{}\"", x_centre);
        let needle_y1 = format!("y1=\"{}\"", sub.y_top);
        let needle_x2 = format!("x2=\"{}\"", x_centre);
        let needle_y2 = format!("y2=\"{}\"", sub.y_bottom);
        assert!(output.contains(&needle_x1), "missing {needle_x1}; SVG:\n{output}");
        assert!(output.contains(&needle_y1), "missing {needle_y1}; SVG:\n{output}");
        assert!(output.contains(&needle_x2), "missing {needle_x2}; SVG:\n{output}");
        assert!(output.contains(&needle_y2), "missing {needle_y2}; SVG:\n{output}");
        // Sanity: the y range matches the staves the sub-bracket spans
        // (staves 1..3, i.e. staves 1, 2, 3 inclusive).
        let staff_height = SS * 4.0;
        let expected_y_top = layout.staff_y_origins[1];
        let expected_y_bottom = layout.staff_y_origins[3] + staff_height;
        assert!(
            (sub.y_top - expected_y_top).abs() < 1e-6,
            "sub.y_top: expected {expected_y_top}, got {}", sub.y_top
        );
        assert!(
            (sub.y_bottom - expected_y_bottom).abs() < 1e-6,
            "sub.y_bottom: expected {expected_y_bottom}, got {}", sub.y_bottom
        );
    }

    #[test]
    fn draw_multi_staff_connectors_renders_main_and_sub_brackets() {
        // End-to-end: a bracket group with one sub-bracket should emit:
        //   * 1 main-bracket thick line + 2 scroll paths
        //   * 1 sub-bracket thin line
        // → total 2 lines + 2 paths. This locks the count delta added by
        // sub-brackets specifically (compare with the no-sub-brackets
        // baseline test `draw_multi_staff_connectors_bracket` which asserts
        // 1 line + 2 paths).
        let group = StaffGroup::section(3)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let font = bravura_font();

        let mut svg = SvgWriter::new(200.0, 600.0, -400.0, -50.0, 5500.0, 4000.0);
        draw_multi_staff_connectors(&mut svg, &font, &layout).unwrap();

        let output = svg.to_svg();
        let line_count = output.matches("<line").count();
        let path_count = output.matches("<path").count();
        assert_eq!(line_count, 2, "main thick + sub thin = 2 lines, got {line_count}");
        assert_eq!(path_count, 2, "2 scroll-glyph paths, got {path_count}");
    }

    #[test]
    fn draw_multi_staff_connectors_two_sub_brackets_emit_two_thin_lines() {
        // With N sub-brackets the connector pass must emit exactly N
        // additional thin lines over the main-bracket baseline. Locks the
        // 1:1 layout→render mapping for sub-brackets.
        let group = StaffGroup::section(6).with_sub_brackets(vec![
            SubBracket { start_index: 0, staff_count: 2 },
            SubBracket { start_index: 3, staff_count: 3 },
        ]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let font = bravura_font();

        let mut svg = SvgWriter::new(200.0, 600.0, -400.0, -50.0, 8000.0, 6000.0);
        draw_multi_staff_connectors(&mut svg, &font, &layout).unwrap();

        let output = svg.to_svg();
        let line_count = output.matches("<line").count();
        // 1 main thick line + 2 sub thin lines = 3.
        assert_eq!(line_count, 3, "expected 3 lines; SVG:\n{output}");
        // Each of the two sub-brackets must contribute its own y-range.
        let sub0_y1 = format!("y1=\"{}\"", layout.sub_brackets[0].y_top);
        let sub1_y1 = format!("y1=\"{}\"", layout.sub_brackets[1].y_top);
        assert!(output.contains(&sub0_y1), "missing sub-bracket 0 y1; SVG:\n{output}");
        assert!(output.contains(&sub1_y1), "missing sub-bracket 1 y1; SVG:\n{output}");
        // The two y1 substrings must be distinct strings (the two sub-brackets
        // start at different y-coordinates by construction).
        assert_ne!(sub0_y1, sub1_y1);
    }

    #[test]
    fn main_bracket_x_shifts_left_when_sub_bracket_present_end_to_end() {
        // Render the same section group twice — once without sub-brackets,
        // once with — and verify the SVGs differ in the main-bracket
        // vertical-line x position. The presence of a sub-bracket must
        // push the main bracket leftward in the rendered output (not just
        // in the layout).
        let font = bravura_font();
        let staff_count = 3;

        let plain = StaffGroup::section(staff_count);
        let plain_layout = layout_multi_staff(&plain, 0.0, SS, 5000.0);
        let mut svg_plain = SvgWriter::new(200.0, 600.0, -400.0, -50.0, 5500.0, 4000.0);
        draw_multi_staff_connectors(&mut svg_plain, &font, &plain_layout).unwrap();
        let out_plain = svg_plain.to_svg();

        let nested = StaffGroup::section(staff_count)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let nested_layout = layout_multi_staff(&nested, 0.0, SS, 5000.0);
        let mut svg_nested = SvgWriter::new(200.0, 600.0, -400.0, -50.0, 5500.0, 4000.0);
        draw_multi_staff_connectors(&mut svg_nested, &font, &nested_layout).unwrap();
        let out_nested = svg_nested.to_svg();

        // The plain SVG must contain the unshifted main-bracket stroke centre
        // (at -BRACKET_THICKNESS_SS/2 * SS = -62.5).
        let plain_centre = format!(
            "x1=\"{}\"",
            plain_layout.bracket.as_ref().unwrap().x
                + plain_layout.bracket.as_ref().unwrap().thickness / 2.0
        );
        assert!(out_plain.contains(&plain_centre), "plain centre missing");

        // The nested SVG must contain the shifted main-bracket stroke centre
        // — strictly further left than the plain centre — AND must NOT contain
        // the plain (unshifted) centre.
        let nested_main = nested_layout.bracket.as_ref().unwrap();
        let nested_centre = format!("x1=\"{}\"", nested_main.x + nested_main.thickness / 2.0);
        assert!(out_nested.contains(&nested_centre), "nested centre missing");
        assert!(
            !out_nested.contains(&plain_centre),
            "regression: nested-bracket SVG must NOT keep the plain centre x"
        );
        // Numeric verification of the leftward shift.
        assert!(
            nested_main.x < plain_layout.bracket.as_ref().unwrap().x,
            "expected nested main.x ({}) < plain main.x ({})",
            nested_main.x,
            plain_layout.bracket.as_ref().unwrap().x,
        );
    }
}
