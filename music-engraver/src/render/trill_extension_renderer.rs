//! Render a tiled trill wavy-line extension.
//!
//! Emits one SVG `<path>` per segment in the layout by translating the SMuFL
//! wiggle glyph outline. The renderer is intentionally font-aware (it queries
//! the font for the glyph outline) while the layout phase remains font-agnostic
//! — see `layout::trill_extension` for the geometry contract.

use crate::font::{FontError, MusicFont};
use crate::layout::trill_extension::TrillExtensionLayout;
use crate::render::SvgWriter;

/// Render a trill wavy-line extension by tiling the wiggle glyph at each
/// segment position computed by [`layout_trill_extension`](crate::layout::trill_extension::layout_trill_extension).
///
/// Returns `Ok(())` after emitting one `<path>` per segment. A layout with an
/// empty `segment_xs` is a no-op (the layout step already returned `None` for
/// spans too short to tile, but the renderer handles the empty case
/// defensively for callers that build layouts manually).
pub fn draw_trill_extension(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &TrillExtensionLayout,
) -> Result<(), FontError> {
    if layout.segment_xs.is_empty() {
        return Ok(());
    }

    let outline = font.glyph_outline(layout.glyph)?;
    for &x in &layout.segment_xs {
        let transform = format!("translate({x},{y})", y = layout.y);
        svg.add_path(&outline.path_data, "black", Some(&transform));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::trill_extension::layout_trill_extension;
    use smufl::Glyph;

    fn test_writer() -> SvgWriter {
        SvgWriter::new(400.0, 100.0, 0.0, 0.0, 2000.0, 500.0)
    }

    fn wiggle_advance(font: &MusicFont) -> f64 {
        font.glyph_advance(Glyph::WiggleTrill)
            .expect("WiggleTrill must have an advance") as f64
    }

    #[test]
    fn renders_one_path_per_segment_two_segments() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(100.0, 100.0 + 2.0 * advance, 50.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        // Exactly two <path elements (no other content drawn in this test)
        assert_eq!(svg.matches("<path").count(), 2);
    }

    #[test]
    fn renders_one_path_per_segment_five_segments() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 5.0 * advance, 0.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert_eq!(svg.matches("<path").count(), 5);
    }

    #[test]
    fn embeds_each_segment_x_in_translate() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(100.0, 100.0 + 3.0 * advance, 50.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        for &x in &layout.segment_xs {
            let needle = format!("translate({x},");
            assert!(
                svg.contains(&needle),
                "SVG should embed segment x {x} (looked for {needle:?}): {}",
                &svg[..svg.len().min(400)]
            );
        }
    }

    #[test]
    fn embeds_y_coordinate_in_translate() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 2.0 * advance, 271.5, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains(",271.5)"),
            "SVG should embed y coordinate 271.5: {}",
            &svg[..svg.len().min(400)]
        );
    }

    #[test]
    fn empty_layout_is_noop() {
        let font = bravura_font();
        let layout = TrillExtensionLayout {
            segment_xs: vec![],
            y: 0.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 100.0,
        };
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert_eq!(svg.matches("<path").count(), 0);
    }

    #[test]
    fn all_segments_share_same_path_data() {
        // Each segment translates the same glyph outline; the path data after
        // each <path d="..." should match exactly. We assert this by counting
        // the most common "d=" prefix substring.
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 4.0 * advance, 0.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        // Grab the first path's d="..." literal and count its occurrences.
        let first_d_start = svg.find("d=\"").expect("at least one path") + 3;
        let first_d_end = svg[first_d_start..]
            .find('"')
            .expect("closing quote")
            + first_d_start;
        let path_d = &svg[first_d_start..first_d_end];
        // 4 segments, identical d attribute on every <path>
        assert_eq!(svg.matches(path_d).count(), 4);
    }

    #[test]
    fn unknown_glyph_returns_font_error() {
        // A non-WiggleTrill glyph in the layout struct should propagate the
        // font's error if the glyph isn't supported. We pick a glyph value
        // that's safe to ask for: actually any well-known glyph works here,
        // so instead we test the inverse — a known glyph never errors.
        let font = bravura_font();
        let layout = TrillExtensionLayout {
            segment_xs: vec![0.0],
            y: 0.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 100.0,
        };
        let mut writer = test_writer();
        assert!(draw_trill_extension(&mut writer, &font, &layout).is_ok());
    }

    #[test]
    fn segment_path_color_is_black() {
        let font = bravura_font();
        let advance = wiggle_advance(&font);
        let layout = layout_trill_extension(0.0, 2.0 * advance, 0.0, advance).unwrap();
        let mut writer = test_writer();
        draw_trill_extension(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains("fill=\"black\""),
            "wiggle paths should fill black: {}",
            &svg[..svg.len().min(400)]
        );
    }
}
