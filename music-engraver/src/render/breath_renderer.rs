use crate::font::FontError;
use crate::font::MusicFont;
use crate::layout::breath::BreathMarkLayout;
use crate::render::SvgWriter;

/// Draw a breath mark glyph (comma, tick, or caesura) at the position
/// computed by `layout_breath_mark`.
pub fn draw_breath_mark(
    writer: &mut SvgWriter,
    font: &MusicFont,
    layout: &BreathMarkLayout,
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
    use crate::layout::breath::{layout_breath_mark, BreathMark};
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
    fn draw_comma_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_breath_mark(BreathMark::Comma, 500.0, &staff);
        let mut writer = test_writer();
        draw_breath_mark(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
        assert!(
            svg.contains("translate("),
            "should have a translate transform"
        );
    }

    #[test]
    fn draw_tick_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_breath_mark(BreathMark::Tick, 500.0, &staff);
        let mut writer = test_writer();
        draw_breath_mark(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
    }

    #[test]
    fn draw_caesura_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_breath_mark(BreathMark::Caesura, 500.0, &staff);
        let mut writer = test_writer();
        draw_breath_mark(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
    }

    #[test]
    fn comma_and_tick_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let comma = layout_breath_mark(BreathMark::Comma, 100.0, &staff);
        let tick = layout_breath_mark(BreathMark::Tick, 100.0, &staff);

        let mut w1 = test_writer();
        draw_breath_mark(&mut w1, &font, &comma).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_breath_mark(&mut w2, &font, &tick).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(svg1, svg2, "comma and tick should produce different paths");
    }

    #[test]
    fn tick_and_caesura_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let tick = layout_breath_mark(BreathMark::Tick, 100.0, &staff);
        let caesura = layout_breath_mark(BreathMark::Caesura, 100.0, &staff);

        let mut w1 = test_writer();
        draw_breath_mark(&mut w1, &font, &tick).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_breath_mark(&mut w2, &font, &caesura).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(
            svg1, svg2,
            "tick and caesura should produce different paths"
        );
    }

    #[test]
    fn all_marks_render_without_error() {
        let font = test_font();
        let staff = test_staff();
        for mark in BreathMark::all() {
            let layout = layout_breath_mark(*mark, 100.0, &staff);
            let mut writer = test_writer();
            let result = draw_breath_mark(&mut writer, &font, &layout);
            assert!(result.is_ok(), "{mark:?} should render without error");
            let svg = writer.to_svg();
            assert!(svg.contains("<path"), "{mark:?} should produce a path");
        }
    }

    #[test]
    fn draw_embeds_translate_with_layout_x() {
        let font = test_font();
        let staff = test_staff();
        let note_right_x = 789.0;
        let layout = layout_breath_mark(BreathMark::Comma, note_right_x, &staff);
        let mut writer = test_writer();
        draw_breath_mark(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        // The layout.x includes padding offset from note_right_x
        let expected_x_prefix = format!("{}", (layout.x as i32));
        assert!(
            svg.contains(&expected_x_prefix),
            "SVG should contain the breath mark x-coordinate starting with {expected_x_prefix}"
        );
    }
}
