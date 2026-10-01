use crate::font::{FontError, MusicFont};
use crate::layout::tremolo::TremoloLayout;
use crate::render::SvgWriter;

/// Draw tremolo slashes on a note stem.
///
/// Renders the appropriate SMuFL tremolo glyph (1–3 slashes) at the
/// position computed by [`crate::layout::tremolo::layout_tremolo`].
/// The glyph is translated to (x, y) from the layout.
pub fn draw_tremolo(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &TremoloLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(layout.glyph)?;
    let transform = format!("translate({},{})", layout.x, layout.y);
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::staff::StaffLayout;
    use crate::layout::stem::StemDirection;
    use crate::layout::tremolo::{layout_tremolo, TremoloCount};

    fn test_font() -> MusicFont<'static> {
        bravura_font()
    }

    fn test_staff() -> StaffLayout {
        let font = test_font();
        let config = font.engraving_config();
        StaffLayout::from_config(0.0, 0.0, 5000.0, &config)
    }

    #[test]
    fn draw_single_tremolo_adds_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_tremolo(
            TremoloCount::Single,
            100.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Up,
        );
        let mut svg = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        draw_tremolo(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("<path"), "should contain a path element");
        assert!(
            output.contains("translate("),
            "should have translate transform"
        );
    }

    #[test]
    fn draw_double_tremolo_adds_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_tremolo(
            TremoloCount::Double,
            100.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Up,
        );
        let mut svg = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        draw_tremolo(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("<path"));
    }

    #[test]
    fn draw_triple_tremolo_adds_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_tremolo(
            TremoloCount::Triple,
            100.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Down,
        );
        let mut svg = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        draw_tremolo(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("<path"));
    }

    #[test]
    fn different_counts_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let mut svg1 = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        let l1 = layout_tremolo(
            TremoloCount::Single,
            100.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Up,
        );
        draw_tremolo(&mut svg1, &font, &l1).unwrap();

        let mut svg2 = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        let l2 = layout_tremolo(
            TremoloCount::Double,
            100.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Up,
        );
        draw_tremolo(&mut svg2, &font, &l2).unwrap();

        let mut svg3 = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        let l3 = layout_tremolo(
            TremoloCount::Triple,
            100.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Up,
        );
        draw_tremolo(&mut svg3, &font, &l3).unwrap();

        assert_ne!(svg1.to_svg(), svg2.to_svg());
        assert_ne!(svg2.to_svg(), svg3.to_svg());
        assert_ne!(svg1.to_svg(), svg3.to_svg());
    }

    #[test]
    fn translate_contains_layout_coordinates() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_tremolo(
            TremoloCount::Single,
            123.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Up,
        );
        let mut svg = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        draw_tremolo(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(
            output.contains("123"),
            "x-coordinate 123 should appear in translate"
        );
    }

    #[test]
    fn path_is_filled_black() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_tremolo(
            TremoloCount::Double,
            100.0,
            500.0,
            200.0,
            &staff,
            StemDirection::Up,
        );
        let mut svg = SvgWriter::new(1000.0, 400.0, 0.0, 0.0, 1000.0, 400.0);
        draw_tremolo(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("fill=\"black\""));
    }
}
