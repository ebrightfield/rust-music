use crate::font::FontError;
use crate::font::MusicFont;
use crate::layout::navigation::NavigationSignLayout;
use crate::render::SvgWriter;

/// Draw a navigation sign glyph (segno, coda) at the position computed by
/// `layout_navigation_sign`.
pub fn draw_navigation_sign(
    writer: &mut SvgWriter,
    font: &MusicFont,
    layout: &NavigationSignLayout,
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
    use crate::layout::navigation::{layout_navigation_sign, NavigationSign};
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
    fn draw_segno_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_navigation_sign(NavigationSign::Segno, 100.0, &staff);
        let mut writer = test_writer();
        draw_navigation_sign(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
        assert!(svg.contains("translate("), "should have a translate transform");
    }

    #[test]
    fn draw_coda_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_navigation_sign(NavigationSign::Coda, 200.0, &staff);
        let mut writer = test_writer();
        draw_navigation_sign(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
    }

    #[test]
    fn segno_and_coda_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let segno = layout_navigation_sign(NavigationSign::Segno, 100.0, &staff);
        let coda = layout_navigation_sign(NavigationSign::Coda, 100.0, &staff);

        let mut w1 = test_writer();
        draw_navigation_sign(&mut w1, &font, &segno).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_navigation_sign(&mut w2, &font, &coda).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(svg1, svg2, "segno and coda should produce different paths");
    }

    #[test]
    fn coda_and_coda_square_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let coda = layout_navigation_sign(NavigationSign::Coda, 100.0, &staff);
        let sq = layout_navigation_sign(NavigationSign::CodaSquare, 100.0, &staff);

        let mut w1 = test_writer();
        draw_navigation_sign(&mut w1, &font, &coda).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_navigation_sign(&mut w2, &font, &sq).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(svg1, svg2, "coda and coda square should differ");
    }

    #[test]
    fn all_signs_render_without_error() {
        let font = test_font();
        let staff = test_staff();
        for sign in NavigationSign::all() {
            let layout = layout_navigation_sign(*sign, 100.0, &staff);
            let mut writer = test_writer();
            let result = draw_navigation_sign(&mut writer, &font, &layout);
            assert!(result.is_ok(), "{sign:?} should render without error");
            let svg = writer.to_svg();
            assert!(svg.contains("<path"), "{sign:?} should produce a path");
        }
    }

    #[test]
    fn draw_embeds_x_coordinate() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_navigation_sign(NavigationSign::Segno, 789.0, &staff);
        let mut writer = test_writer();
        draw_navigation_sign(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("789"), "SVG should contain the x-coordinate 789");
    }
}
