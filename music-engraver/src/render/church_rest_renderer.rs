//! Render a church-rest cluster (combinations of whole + breve rest glyphs
//! used as old-style multi-measure rests for small counts).

use crate::font::{FontError, MusicFont};
use crate::layout::multi_measure_rest::ChurchRestLayout;
use crate::render::svg_writer::TextStyle;
use crate::render::SvgWriter;

/// Draw a church-rest cluster onto an SVG writer.
///
/// Renders:
/// - One `<path>` per rest glyph in the cluster (typically 1 or 2).
/// - One bold centered count number above the staff (matching the H-bar
///   convention so a reader can scan the count without distinguishing styles).
///
/// Returns `Ok(())` on success. The only failure mode is a missing rest
/// glyph in the music font — which would be a corrupt bundled Bravura,
/// caught by unit tests.
pub fn draw_church_rest(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &ChurchRestLayout,
) -> Result<(), FontError> {
    for placement in &layout.glyphs {
        let outline = font.glyph_outline(placement.glyph)?;
        let transform = format!("translate({}, {})", placement.x, placement.y);
        svg.add_path(&outline.path_data, "black", Some(&transform));
    }

    // Skip the count text entirely when the cluster is empty — the caller
    // should have fallen back to the H-bar, but if they didn't, drawing
    // just a stray number would be misleading.
    if !layout.glyphs.is_empty() {
        let style = TextStyle {
            anchor: "middle",
            ..TextStyle::bold(layout.count_font_size)
        };
        svg.add_text(
            layout.count_x,
            layout.count_y,
            &layout.count.to_string(),
            &style,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::multi_measure_rest::layout_church_rest;
    use crate::layout::staff::StaffLayout;

    fn setup() -> (MusicFont<'static>, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, staff)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 400.0, -200.0, -400.0, 6000.0, 2000.0)
    }

    fn render(count: u32) -> String {
        let (font, staff) = setup();
        let layout = layout_church_rest(0.0, 5000.0, count, &staff);
        let mut svg = make_svg();
        draw_church_rest(&mut svg, &font, &layout).unwrap();
        svg.to_svg()
    }

    #[test]
    fn count_1_draws_one_path() {
        let output = render(1);
        assert_eq!(
            output.matches("<path ").count(),
            1,
            "count=1 should draw exactly 1 rest glyph"
        );
    }

    #[test]
    fn count_2_draws_one_path() {
        let output = render(2);
        assert_eq!(
            output.matches("<path ").count(),
            1,
            "count=2 should draw exactly 1 breve rest glyph"
        );
    }

    #[test]
    fn count_3_draws_two_paths() {
        let output = render(3);
        assert_eq!(
            output.matches("<path ").count(),
            2,
            "count=3 should draw 2 rest glyphs (breve + whole)"
        );
    }

    #[test]
    fn count_4_draws_two_paths() {
        let output = render(4);
        assert_eq!(
            output.matches("<path ").count(),
            2,
            "count=4 should draw 2 breve rest glyphs"
        );
    }

    #[test]
    fn count_text_is_present() {
        for n in 1..=4u32 {
            let output = render(n);
            let expected = format!(">{n}</text>");
            assert!(
                output.contains(&expected),
                "count={n} output must contain {expected}; got:\n{output}"
            );
        }
    }

    #[test]
    fn count_text_is_bold_and_centered() {
        let output = render(3);
        assert!(
            output.contains("font-weight=\"bold\""),
            "count text should be bold; got:\n{output}"
        );
        assert!(
            output.contains("text-anchor=\"middle\""),
            "count text should be centered; got:\n{output}"
        );
    }

    #[test]
    fn unsupported_count_draws_nothing() {
        for n in &[0u32, 5, 7, 100] {
            let (font, staff) = setup();
            let layout = layout_church_rest(0.0, 5000.0, *n, &staff);
            let mut svg = make_svg();
            draw_church_rest(&mut svg, &font, &layout).unwrap();
            let output = svg.to_svg();
            assert_eq!(
                output.matches("<path ").count(),
                0,
                "unsupported count {n} should draw no paths"
            );
            assert!(
                !output.contains("</text>"),
                "unsupported count {n} should also suppress the count number"
            );
        }
    }

    #[test]
    fn count_3_and_4_paths_horizontally_distinct() {
        // Both render 2 paths, but the contents/transforms differ because
        // count=3 uses breve+whole and count=4 uses breve+breve.
        let out3 = render(3);
        let out4 = render(4);
        assert_ne!(out3, out4, "count=3 and count=4 must render different SVG");
    }

    #[test]
    fn count_1_and_2_produce_different_glyphs() {
        // Whole rest vs breve rest have different path_data — the SVG strings
        // must differ even though both contain exactly 1 path.
        let out1 = render(1);
        let out2 = render(2);
        let path1 = out1.split("<path ").nth(1).unwrap();
        let path2 = out2.split("<path ").nth(1).unwrap();
        // path_data starts after `d="...`. We extract a substring of each
        // and assert they differ; this guarantees the two glyphs are not
        // the same outline accidentally.
        assert_ne!(
            path1.split("/>").next(),
            path2.split("/>").next(),
            "count=1 (whole) and count=2 (breve) must use different glyph outlines"
        );
    }

    #[test]
    fn count_1_glyph_anchored_at_position_6() {
        // Whole rest hangs from the 4th line (staff position 6).
        let (font, staff) = setup();
        let layout = layout_church_rest(0.0, 5000.0, 1, &staff);
        let mut svg = make_svg();
        draw_church_rest(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        let expected_y = staff.y_of(6);
        let needle = format!(", {expected_y})");
        assert!(
            output.contains(&needle),
            "transform should reference y={expected_y} (position 6); SVG:\n{output}"
        );
    }

    #[test]
    fn count_2_glyph_anchored_at_position_4() {
        // Breve sits on the middle (3rd) line, staff position 4.
        let (font, staff) = setup();
        let layout = layout_church_rest(0.0, 5000.0, 2, &staff);
        let mut svg = make_svg();
        draw_church_rest(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        let expected_y = staff.y_of(4);
        let needle = format!(", {expected_y})");
        assert!(
            output.contains(&needle),
            "transform should reference y={expected_y} (position 4); SVG:\n{output}"
        );
    }
}
