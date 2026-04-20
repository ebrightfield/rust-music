use crate::font::FontError;
use crate::font::MusicFont;
use crate::layout::ornament::OrnamentLayout;
use crate::render::SvgWriter;

/// Draw an ornament glyph at the position computed by `layout_ornament`.
///
/// Returns `Ok(())` after rendering the glyph path.
pub fn draw_ornament(
    writer: &mut SvgWriter,
    font: &MusicFont,
    layout: &OrnamentLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(layout.glyph)?;
    let transform = format!("translate({},{})", layout.x, layout.y);
    writer.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::ornament::{layout_ornament, Ornament};
    use crate::layout::staff::StaffLayout;

    fn test_font() -> MusicFont<'static> {
        bravura_font()
    }

    fn test_staff() -> StaffLayout {
        let font = test_font();
        let config = font.engraving_config();
        StaffLayout::from_config(0.0, 0.0, 5000.0, &config)
    }

    fn test_writer() -> SvgWriter {
        SvgWriter::new(200.0, 100.0, 0.0, 0.0, 1000.0, 500.0)
    }

    #[test]
    fn draw_trill_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_ornament(Ornament::Trill, 100.0, 4, &staff);
        let mut writer = test_writer();
        draw_ornament(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
        assert!(svg.contains("translate("), "should have a translate transform");
    }

    #[test]
    fn draw_mordent_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_ornament(Ornament::Mordent, 200.0, 6, &staff);
        let mut writer = test_writer();
        draw_ornament(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
    }

    #[test]
    fn draw_turn_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_ornament(Ornament::Turn, 300.0, 4, &staff);
        let mut writer = test_writer();
        draw_ornament(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
    }

    #[test]
    fn trill_and_mordent_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let trill = layout_ornament(Ornament::Trill, 100.0, 4, &staff);
        let mordent = layout_ornament(Ornament::Mordent, 100.0, 4, &staff);

        let mut w1 = test_writer();
        draw_ornament(&mut w1, &font, &trill).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_ornament(&mut w2, &font, &mordent).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(svg1, svg2, "trill and mordent should produce different paths");
    }

    #[test]
    fn turn_and_inverted_turn_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let turn = layout_ornament(Ornament::Turn, 100.0, 4, &staff);
        let inv = layout_ornament(Ornament::InvertedTurn, 100.0, 4, &staff);

        let mut w1 = test_writer();
        draw_ornament(&mut w1, &font, &turn).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_ornament(&mut w2, &font, &inv).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(svg1, svg2, "turn and inverted turn should differ");
    }

    #[test]
    fn all_ornaments_render_without_error() {
        let font = test_font();
        let staff = test_staff();
        for ornament in Ornament::all() {
            let layout = layout_ornament(*ornament, 100.0, 4, &staff);
            let mut writer = test_writer();
            let result = draw_ornament(&mut writer, &font, &layout);
            assert!(result.is_ok(), "{ornament:?} should render without error");
            let svg = writer.to_svg();
            assert!(svg.contains("<path"), "{ornament:?} should produce a path");
        }
    }

    #[test]
    fn draw_ornament_embeds_x_coordinate() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_ornament(Ornament::Trill, 789.0, 4, &staff);
        let mut writer = test_writer();
        draw_ornament(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains("789"),
            "SVG should contain the x-coordinate 789"
        );
    }
}
