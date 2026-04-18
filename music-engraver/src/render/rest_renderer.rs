use crate::font::{FontError, MusicFont};
use crate::layout::rest::{rest_glyph, rest_y};
use crate::layout::staff::StaffLayout;
use crate::render::SvgWriter;

/// Draw a rest glyph centered on the staff.
///
/// `log2_duration`: 0 = whole, 1 = half, 2 = quarter, 3 = eighth, etc.
/// `x`: horizontal position for the rest.
///
/// Returns `Ok(advance_width)` if drawn, or `Ok(0.0)` if `log2_duration`
/// is out of range. The advance width is in font design units.
pub fn draw_rest(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    log2_duration: u8,
) -> Result<f64, FontError> {
    let glyph = match rest_glyph(log2_duration) {
        Some(g) => g,
        None => return Ok(0.0),
    };

    let outline = font.glyph_outline(glyph)?;
    let y = rest_y(staff, log2_duration);
    let transform = format!("translate({x}, {y})");
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(outline.advance_width as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn setup() -> (MusicFont<'static>, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, staff)
    }

    #[test]
    fn draw_rest_whole_produces_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance = draw_rest(&mut svg, &staff, &font, 500.0, 0).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
        assert!(advance > 0.0, "whole rest should have positive advance");
        // Whole rest at position 6: y = (8-6)*125 = 250
        let expected_y = staff.y_of(6);
        assert!(output.contains(&format!("translate(500, {expected_y})")));
    }

    #[test]
    fn draw_rest_half_produces_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance = draw_rest(&mut svg, &staff, &font, 600.0, 1).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
        assert!(advance > 0.0);
        let expected_y = staff.y_of(4);
        assert!(output.contains(&format!("translate(600, {expected_y})")));
    }

    #[test]
    fn draw_rest_quarter_produces_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance = draw_rest(&mut svg, &staff, &font, 700.0, 2).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
        assert!(advance > 0.0);
    }

    #[test]
    fn draw_rest_eighth_produces_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance = draw_rest(&mut svg, &staff, &font, 800.0, 3).unwrap();
        assert!(advance > 0.0);
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn draw_rest_invalid_returns_zero_no_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance = draw_rest(&mut svg, &staff, &font, 500.0, 8).unwrap();
        assert_eq!(advance, 0.0);
        assert_eq!(svg.to_svg().matches("<path ").count(), 0);
    }

    #[test]
    fn each_rest_duration_produces_distinct_path() {
        let (font, staff) = setup();
        let mut svgs: Vec<String> = Vec::new();
        for d in 0..=7 {
            let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2500.0);
            draw_rest(&mut svg, &staff, &font, 0.0, d).unwrap();
            svgs.push(svg.to_svg());
        }
        for i in 0..svgs.len() {
            for j in (i + 1)..svgs.len() {
                assert_ne!(
                    svgs[i], svgs[j],
                    "rest durations {} and {} should produce different SVG",
                    i, j
                );
            }
        }
    }

    #[test]
    fn draw_rest_returns_reasonable_advance_widths() {
        let (font, staff) = setup();
        for d in 0..=7 {
            let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2500.0);
            let advance = draw_rest(&mut svg, &staff, &font, 0.0, d).unwrap();
            assert!(
                advance > 50.0 && advance < 1000.0,
                "rest log2={d} advance={advance} should be reasonable"
            );
        }
    }

    #[test]
    fn draw_rest_all_durations_produce_one_path_each() {
        let (font, staff) = setup();
        for d in 0..=7 {
            let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2500.0);
            draw_rest(&mut svg, &staff, &font, 500.0, d).unwrap();
            assert_eq!(
                svg.to_svg().matches("<path ").count(),
                1,
                "rest log2={d} should produce exactly one path"
            );
        }
    }
}
