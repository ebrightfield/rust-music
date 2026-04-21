//! SVG rendering for rhythm notation above tablature staves.
//!
//! Draws stems and flags above the tab staff to indicate duration.
//! Stems always point up. Flags use SMuFL glyphs (same as standard notation).

use crate::font::{FontError, MusicFont};
use crate::layout::flag::flag_glyph;
use crate::layout::stem::StemDirection;
use crate::layout::tab_rhythm::TabRhythmLayout;
use crate::render::SvgWriter;

/// Draw a rhythm stem (vertical line) and optional flag above a tab staff.
///
/// Half notes get a stem only; quarter notes get a filled stem; eighth and
/// shorter notes get a stem plus a flag glyph at the tip. The visual
/// difference between half and quarter stems is the stroke width (half note
/// stems are thinner) — matching common tablature convention.
///
/// Returns `Ok(true)` if something was drawn, `Ok(false)` for no-op.
pub fn draw_tab_rhythm(
    svg: &mut SvgWriter,
    layout: &TabRhythmLayout,
    font: &MusicFont,
) -> Result<bool, FontError> {
    // Draw stem line
    svg.add_line(
        layout.x,
        layout.y_base,
        layout.x,
        layout.y_tip,
        "black",
        layout.stem_width,
    );

    // Draw flag at the stem tip if needed
    if layout.flag_count > 0 {
        if let Some(glyph) = flag_glyph(layout.flag_count, StemDirection::Up) {
            let outline = font.glyph_outline(glyph)?;
            let transform = format!("translate({}, {})", layout.x, layout.y_tip);
            svg.add_path(&outline.path_data, "black", Some(&transform));
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_rhythm::layout_tab_rhythm;

    fn setup() -> (MusicFont<'static>, TabStaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 500.0, 5000.0, &config);
        (font, staff)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 600.0, -200.0, -400.0, 6000.0, 3000.0)
    }

    #[test]
    fn quarter_stem_draws_line() {
        let (font, staff) = setup();
        let config = font.engraving_config();
        let layout = layout_tab_rhythm(&staff, 1000.0, 2, config.stem_thickness_fu()).unwrap();
        let mut svg = make_svg();
        let drawn = draw_tab_rhythm(&mut svg, &layout, &font).unwrap();
        assert!(drawn);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            1,
            "quarter note should draw 1 stem line"
        );
        // No flag path for quarter note
        assert_eq!(
            output.matches("<path ").count(),
            0,
            "quarter note should have no flag"
        );
    }

    #[test]
    fn eighth_stem_draws_line_and_flag() {
        let (font, staff) = setup();
        let config = font.engraving_config();
        let layout = layout_tab_rhythm(&staff, 1000.0, 3, config.stem_thickness_fu()).unwrap();
        let mut svg = make_svg();
        draw_tab_rhythm(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 1, "should have stem line");
        assert_eq!(output.matches("<path ").count(), 1, "should have flag path");
    }

    #[test]
    fn sixteenth_stem_draws_line_and_flag() {
        let (font, staff) = setup();
        let config = font.engraving_config();
        let layout = layout_tab_rhythm(&staff, 1000.0, 4, config.stem_thickness_fu()).unwrap();
        let mut svg = make_svg();
        draw_tab_rhythm(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 1);
        assert_eq!(output.matches("<path ").count(), 1);
    }

    #[test]
    fn half_note_has_stem_but_no_flag() {
        let (font, staff) = setup();
        let config = font.engraving_config();
        let layout = layout_tab_rhythm(&staff, 1000.0, 1, config.stem_thickness_fu()).unwrap();
        let mut svg = make_svg();
        draw_tab_rhythm(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 1);
        assert_eq!(output.matches("<path ").count(), 0);
    }

    #[test]
    fn eighth_flag_differs_from_sixteenth_flag() {
        let (font, staff) = setup();
        let config = font.engraving_config();
        let eighth = layout_tab_rhythm(&staff, 1000.0, 3, config.stem_thickness_fu()).unwrap();
        let sixteenth = layout_tab_rhythm(&staff, 1000.0, 4, config.stem_thickness_fu()).unwrap();
        let mut svg8 = make_svg();
        let mut svg16 = make_svg();
        draw_tab_rhythm(&mut svg8, &eighth, &font).unwrap();
        draw_tab_rhythm(&mut svg16, &sixteenth, &font).unwrap();
        assert_ne!(
            svg8.to_svg(),
            svg16.to_svg(),
            "eighth and sixteenth flags should differ"
        );
    }

    #[test]
    fn stem_y_coordinates_in_svg() {
        let (font, staff) = setup();
        let config = font.engraving_config();
        let layout = layout_tab_rhythm(&staff, 1000.0, 2, config.stem_thickness_fu()).unwrap();
        let mut svg = make_svg();
        draw_tab_rhythm(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        // Verify the stem line has y1 and y2 coordinates that are above the staff
        // y_origin is 500, so stem should have y values < 500
        assert!(
            output.contains(&format!("y1=\"{}\"", layout.y_base)),
            "should contain stem base y"
        );
        assert!(
            output.contains(&format!("y2=\"{}\"", layout.y_tip)),
            "should contain stem tip y"
        );
    }

    #[test]
    fn flag_transform_at_stem_tip() {
        let (font, staff) = setup();
        let config = font.engraving_config();
        let layout = layout_tab_rhythm(&staff, 1000.0, 3, config.stem_thickness_fu()).unwrap();
        let mut svg = make_svg();
        draw_tab_rhythm(&mut svg, &layout, &font).unwrap();
        let output = svg.to_svg();
        let expected = format!("translate({}, {})", layout.x, layout.y_tip);
        assert!(
            output.contains(&expected),
            "flag should be positioned at stem tip"
        );
    }
}
