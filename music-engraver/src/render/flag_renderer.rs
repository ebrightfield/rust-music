use crate::font::{FontError, MusicFont};
use crate::layout::flag::flag_glyph;
use crate::layout::stem::StemDirection;
use crate::render::SvgWriter;

/// Draw a flag at the stem tip.
///
/// The flag glyph is placed at `(stem_tip_x, stem_tip_y)`. SMuFL flag glyphs
/// are designed so their origin sits at the stem tip — no additional offset
/// is needed.
///
/// `flag_count`: 1 for eighth, 2 for sixteenth, 3 for 32nd, 4 for 64th, 5 for 128th.
/// Returns `Ok(true)` if a flag was drawn, `Ok(false)` if `flag_count` is 0.
pub fn draw_flag(
    svg: &mut SvgWriter,
    font: &MusicFont,
    stem_tip_x: f64,
    stem_tip_y: f64,
    flag_count: u8,
    direction: StemDirection,
) -> Result<bool, FontError> {
    let glyph = match flag_glyph(flag_count, direction) {
        Some(g) => g,
        None => return Ok(false),
    };

    let outline = font.glyph_outline(glyph)?;
    let transform = format!("translate({stem_tip_x}, {stem_tip_y})");
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{bravura_font, EngravingConfig};
    use crate::layout::staff::StaffLayout;
    use crate::layout::stem::StemDirection;
    use crate::render::stem_renderer::stem_endpoints;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    #[test]
    fn draw_flag_eighth_up_produces_path() {
        let (font, _, _) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        let result = draw_flag(&mut svg, &font, 780.0, 125.0, 1, StemDirection::Up).unwrap();
        assert!(result, "should return true for drawn flag");
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<path ").count(),
            1,
            "should produce exactly one path"
        );
        assert!(output.contains("translate(780, 125)"));
    }

    #[test]
    fn draw_flag_eighth_down_produces_path() {
        let (font, _, _) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        let result = draw_flag(&mut svg, &font, 515.0, 1875.0, 1, StemDirection::Down).unwrap();
        assert!(result);
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
        assert!(output.contains("translate(515, 1875)"));
    }

    #[test]
    fn draw_flag_zero_count_returns_false_no_svg() {
        let (font, _, _) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        let result = draw_flag(&mut svg, &font, 780.0, 125.0, 0, StemDirection::Up).unwrap();
        assert!(!result, "should return false for 0 flags");
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<path ").count(),
            0,
            "no path for 0 flags"
        );
    }

    #[test]
    fn draw_flag_sixteenth_produces_different_path_than_eighth() {
        let (font, _, _) = setup();
        let mut svg8 = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_flag(&mut svg8, &font, 780.0, 125.0, 1, StemDirection::Up).unwrap();

        let mut svg16 = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_flag(&mut svg16, &font, 780.0, 125.0, 2, StemDirection::Up).unwrap();

        assert_ne!(
            svg8.to_svg(),
            svg16.to_svg(),
            "eighth and sixteenth flags should produce different SVG"
        );
    }

    #[test]
    fn draw_flag_up_and_down_produce_different_paths() {
        let (font, _, _) = setup();
        let mut svg_up = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_flag(&mut svg_up, &font, 780.0, 125.0, 1, StemDirection::Up).unwrap();

        let mut svg_down = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_flag(&mut svg_down, &font, 780.0, 125.0, 1, StemDirection::Down).unwrap();

        assert_ne!(
            svg_up.to_svg(),
            svg_down.to_svg(),
            "up and down flags should differ"
        );
    }

    #[test]
    fn draw_flag_32nd_up_produces_path() {
        let (font, _, _) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        let result = draw_flag(&mut svg, &font, 780.0, 125.0, 3, StemDirection::Up).unwrap();
        assert!(result);
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn draw_flag_64th_down_produces_path() {
        let (font, _, _) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        let result = draw_flag(&mut svg, &font, 515.0, 1875.0, 4, StemDirection::Down).unwrap();
        assert!(result);
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn draw_flag_128th_up_produces_path() {
        let (font, _, _) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -1000.0, 6000.0, 3000.0);
        let result = draw_flag(&mut svg, &font, 780.0, -500.0, 5, StemDirection::Up).unwrap();
        assert!(result);
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn flag_at_realistic_stem_tip_position() {
        let (font, _, staff) = setup();
        // Position 0 (bottom line), stem up: tip is above
        let (y_top, _) = stem_endpoints(&staff, 0, StemDirection::Up);
        let stem_x = 780.0; // right side of notehead

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_flag(&mut svg, &font, stem_x, y_top, 1, StemDirection::Up).unwrap();
        let output = svg.to_svg();

        let expected_translate = format!("translate({stem_x}, {y_top})");
        assert!(
            output.contains(&expected_translate),
            "flag should be at stem tip: {expected_translate}"
        );
    }

    #[test]
    fn flag_count_6_returns_false() {
        let (font, _, _) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        let result = draw_flag(&mut svg, &font, 780.0, 125.0, 6, StemDirection::Up).unwrap();
        assert!(!result);
    }
}
