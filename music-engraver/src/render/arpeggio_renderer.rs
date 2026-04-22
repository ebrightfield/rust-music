use crate::font::{FontError, MusicFont};
use crate::layout::arpeggio::ArpeggioLayout;
use crate::render::SvgWriter;

/// Draw an arpeggio wavy line from a computed layout.
///
/// The arpeggio glyph is vertically scaled to span the chord's height and
/// positioned to the left of the noteheads.
pub fn draw_arpeggio(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &ArpeggioLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(layout.glyph)?;

    // Position the glyph at the top of the arpeggio line, scaled vertically
    // to span from y_top to y_bottom.
    let transform = format!(
        "translate({},{}) scale(1,{})",
        layout.x, layout.y_top, layout.scale_y
    );

    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::arpeggio::{layout_arpeggio, ArpeggioDirection};
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        let font = bravura_font();
        let config = font.engraving_config();
        StaffLayout::from_config(0.0, 0.0, 5000.0, &config)
    }

    #[test]
    fn draw_produces_path_element() {
        let staff = test_staff();
        let font = bravura_font();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 4, 8], 500.0, &staff).unwrap();
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, -200.0, 1000.0, 1200.0);
        draw_arpeggio(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("<path"), "should produce a path element");
    }

    #[test]
    fn draw_includes_transform() {
        let staff = test_staff();
        let font = bravura_font();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 500.0, &staff).unwrap();
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, -200.0, 1000.0, 1200.0);
        draw_arpeggio(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("translate("), "should have translate transform");
        assert!(output.contains("scale(1,"), "should have vertical scale");
    }

    #[test]
    fn up_and_down_produce_different_paths() {
        let staff = test_staff();
        let font = bravura_font();
        let up_layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 500.0, &staff).unwrap();
        let down_layout = layout_arpeggio(ArpeggioDirection::Down, &[0, 8], 500.0, &staff).unwrap();

        let mut svg_up = SvgWriter::new(200.0, 200.0, 0.0, -200.0, 1000.0, 1200.0);
        draw_arpeggio(&mut svg_up, &font, &up_layout).unwrap();
        let mut svg_down = SvgWriter::new(200.0, 200.0, 0.0, -200.0, 1000.0, 1200.0);
        draw_arpeggio(&mut svg_down, &font, &down_layout).unwrap();

        assert_ne!(
            svg_up.to_svg(),
            svg_down.to_svg(),
            "up and down arpeggios should produce different SVG"
        );
    }

    #[test]
    fn three_note_chord_renders() {
        let staff = test_staff();
        let font = bravura_font();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 4, 8], 500.0, &staff).unwrap();
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, -200.0, 1000.0, 1200.0);
        let result = draw_arpeggio(&mut svg, &font, &layout);
        assert!(result.is_ok());
        let output = svg.to_svg();
        assert_eq!(output.matches("<path").count(), 1, "one path for the arpeggio");
    }

    #[test]
    fn x_coordinate_appears_in_transform() {
        let staff = test_staff();
        let font = bravura_font();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 750.0, &staff).unwrap();
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, -200.0, 1000.0, 1200.0);
        draw_arpeggio(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        // The x coordinate (750 - padding) should appear in the transform
        let x_str = format!("{:.2}", layout.x);
        assert!(
            output.contains(&x_str) || output.contains(&format!("{}", layout.x as i64)),
            "transform should contain the x coordinate"
        );
    }
}
