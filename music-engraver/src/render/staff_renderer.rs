use crate::font::{EngravingConfig, MusicFont};
use crate::layout::clef::ClefLayout;
use crate::layout::staff::StaffLayout;
use crate::render::SvgWriter;

/// Draw the five staff lines onto the SVG writer.
pub fn draw_staff_lines(svg: &mut SvgWriter, staff: &StaffLayout, config: &EngravingConfig) {
    let stroke_width = config.staff_line_thickness_fu();
    let x1 = staff.x;
    let x2 = staff.x + staff.width;
    for y in staff.line_ys() {
        svg.add_line(x1, y, x2, y, "black", stroke_width);
    }
}

/// Draw a clef glyph on the staff.
///
/// The glyph is translated so its origin aligns with the left edge of the
/// staff (offset by one staff space as a conventional left margin) and the
/// correct staff line vertically.
pub fn draw_clef(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    clef: &ClefLayout,
    font: &MusicFont,
) -> Result<(), crate::font::FontError> {
    let outline = font.glyph_outline(clef.glyph)?;
    let y = staff.y_of(clef.staff_position);
    // Horizontal offset: one staff space from the left edge, conventional margin
    let x = staff.x + staff.staff_space;
    let transform = format!("translate({x}, {y})");
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use music::notation::clef::Clef;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    #[test]
    fn staff_lines_produces_five_line_elements() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_staff_lines(&mut svg, &staff, &config);
        let output = svg.to_svg();

        let line_count = output.matches("<line ").count();
        assert_eq!(line_count, 5, "should have exactly 5 staff lines");
    }

    #[test]
    fn staff_lines_have_correct_y_coordinates() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_staff_lines(&mut svg, &staff, &config);
        let output = svg.to_svg();

        // Staff space is 250 for Bravura (1000 UPM / 4)
        // Lines at y = 0, 250, 500, 750, 1000
        for y in [0.0, 250.0, 500.0, 750.0, 1000.0] {
            let needle = format!("y1=\"{y}\"");
            assert!(output.contains(&needle), "missing staff line at y={y}");
        }
    }

    #[test]
    fn staff_lines_use_config_stroke_width() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_staff_lines(&mut svg, &staff, &config);
        let output = svg.to_svg();

        // Bravura staff line thickness: 0.13 staff spaces = 32.5 font units
        let expected_sw = format!("stroke-width=\"{}\"", config.staff_line_thickness_fu());
        assert!(
            output.contains(&expected_sw),
            "staff lines should use thickness from EngravingConfig"
        );
    }

    #[test]
    fn draw_treble_clef_adds_path_element() {
        let (font, _, staff) = setup();
        let clef = ClefLayout::from_clef(Clef::Treble);
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_clef(&mut svg, &staff, &clef, &font).unwrap();
        let output = svg.to_svg();

        assert!(
            output.contains("<path "),
            "should contain a <path> element for the clef"
        );
        assert!(
            output.contains("fill=\"black\""),
            "clef path should be filled black"
        );
    }

    #[test]
    fn treble_clef_positioned_at_second_line() {
        let (font, _, staff) = setup();
        let clef = ClefLayout::from_clef(Clef::Treble);
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_clef(&mut svg, &staff, &clef, &font).unwrap();
        let output = svg.to_svg();

        // Second line from bottom = staff position 2
        // y = (8 - 2) * 125 = 750
        let expected_y = staff.y_of(2);
        assert!(
            (expected_y - 750.0).abs() < f64::EPSILON,
            "staff position 2 should map to y=750"
        );

        // Check the transform contains the correct y translation
        let expected_x = staff.x + staff.staff_space;
        let expected_transform = format!("translate({expected_x}, {expected_y})");
        assert!(
            output.contains(&expected_transform),
            "clef transform should position at ({expected_x}, {expected_y})"
        );
    }

    #[test]
    fn bass_clef_positioned_at_fourth_line() {
        let (font, _, staff) = setup();
        let clef = ClefLayout::from_clef(Clef::Bass);
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_clef(&mut svg, &staff, &clef, &font).unwrap();
        let output = svg.to_svg();

        // Fourth line from bottom = staff position 6
        // y = (8 - 6) * 125 = 250
        let expected_y = staff.y_of(6);
        assert!(
            (expected_y - 250.0).abs() < f64::EPSILON,
            "staff position 6 should map to y=250"
        );

        let expected_x = staff.x + staff.staff_space;
        let expected_transform = format!("translate({expected_x}, {expected_y})");
        assert!(
            output.contains(&expected_transform),
            "bass clef should be positioned at fourth line"
        );
    }

    #[test]
    fn full_staff_with_clef_contains_lines_and_path() {
        let (font, config, staff) = setup();
        let clef = ClefLayout::from_clef(Clef::Treble);
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_staff_lines(&mut svg, &staff, &config);
        draw_clef(&mut svg, &staff, &clef, &font).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<line ").count(), 5);
        assert!(output.matches("<path ").count() >= 1);
        assert!(output.starts_with("<svg"));
        assert!(output.contains("</svg>"));
    }
}
