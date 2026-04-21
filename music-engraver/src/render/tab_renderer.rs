use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::tab::{layout_fret_number, tab_clef_glyph, FretNumberLayout, TabStaffLayout};
use crate::render::svg_writer::{RectStyle, TextStyle};
use crate::render::SvgWriter;

/// Draw all staff lines for a tab staff.
pub fn draw_tab_staff_lines(
    svg: &mut SvgWriter,
    tab_staff: &TabStaffLayout,
    config: &EngravingConfig,
) {
    let stroke_width = config.staff_line_thickness_fu();
    let x1 = tab_staff.x;
    let x2 = tab_staff.x + tab_staff.width;
    for y in tab_staff.line_ys() {
        svg.add_line(x1, y, x2, y, "black", stroke_width);
    }
}

/// Draw the TAB clef glyph centered vertically on the staff.
///
/// The glyph is translated so its origin sits at the vertical center
/// of the staff, offset one staff space from the left edge (matching
/// standard clef placement convention).
pub fn draw_tab_clef(
    svg: &mut SvgWriter,
    tab_staff: &TabStaffLayout,
    font: &MusicFont,
) -> Result<(), FontError> {
    let glyph = tab_clef_glyph(tab_staff.line_count);
    let outline = font.glyph_outline(glyph)?;
    let x = tab_staff.x + tab_staff.staff_space;
    let y = tab_staff.center_y();
    let transform = format!("translate({x}, {y})");
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

/// Draw a single fret number on the tab staff.
///
/// Renders a white background rectangle to mask the staff line, then
/// the fret number text centered on the string position.
pub fn draw_fret_number(svg: &mut SvgWriter, layout: &FretNumberLayout) {
    // White background rect to mask the staff line behind the number.
    let bg_style = RectStyle {
        fill: "white",
        stroke: "none",
        stroke_width: 0.0,
    };
    svg.add_styled_rect(
        layout.x - layout.bg_half_width,
        layout.y - layout.bg_half_height,
        layout.bg_half_width * 2.0,
        layout.bg_half_height * 2.0,
        &bg_style,
    );

    // Fret number text, centered on the string line.
    let text_style = TextStyle {
        font_family: "serif",
        font_size: layout.font_size,
        fill: "black",
        anchor: "middle",
        font_weight: "bold",
        font_style: "normal",
        dominant_baseline: "central",
    };
    svg.add_text(layout.x, layout.y, &layout.text, &text_style);
}

/// Convenience: layout + draw a fret number in one call.
pub fn draw_fret_number_at(
    svg: &mut SvgWriter,
    tab_staff: &TabStaffLayout,
    string: u8,
    fret: u8,
    x: f64,
) {
    let layout = layout_fret_number(tab_staff, string, fret, x);
    draw_fret_number(svg, &layout);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn setup() -> (MusicFont<'static>, EngravingConfig, TabStaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 400.0, -100.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn tab_staff_lines_produces_six_lines() {
        let (_, config, staff) = setup();
        let mut svg = make_svg();
        draw_tab_staff_lines(&mut svg, &staff, &config);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            6,
            "guitar tab staff should have 6 lines"
        );
    }

    #[test]
    fn four_string_tab_staff_produces_four_lines() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::four_string(0.0, 0.0, 5000.0, &config);
        let mut svg = make_svg();
        draw_tab_staff_lines(&mut svg, &staff, &config);
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 4);
    }

    #[test]
    fn tab_staff_lines_use_config_stroke_width() {
        let (_, config, staff) = setup();
        let mut svg = make_svg();
        draw_tab_staff_lines(&mut svg, &staff, &config);
        let output = svg.to_svg();
        let expected = format!("stroke-width=\"{}\"", config.staff_line_thickness_fu());
        assert!(output.contains(&expected));
    }

    #[test]
    fn tab_clef_adds_path_element() {
        let (font, _, staff) = setup();
        let mut svg = make_svg();
        draw_tab_clef(&mut svg, &staff, &font).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("<path "), "should contain a clef path");
        assert!(output.contains("fill=\"black\""));
    }

    #[test]
    fn tab_clef_positioned_at_staff_center() {
        let (font, _, staff) = setup();
        let mut svg = make_svg();
        draw_tab_clef(&mut svg, &staff, &font).unwrap();
        let output = svg.to_svg();
        let expected_y = staff.center_y();
        let expected_x = staff.x + staff.staff_space;
        let expected_transform = format!("translate({expected_x}, {expected_y})");
        assert!(
            output.contains(&expected_transform),
            "TAB clef should be centered on staff"
        );
    }

    #[test]
    fn fret_number_draws_rect_and_text() {
        let (_, _, staff) = setup();
        let mut svg = make_svg();
        draw_fret_number_at(&mut svg, &staff, 1, 5, 1000.0);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<rect ").count(),
            1,
            "should have 1 background rect"
        );
        assert_eq!(
            output.matches("<text ").count(),
            1,
            "should have 1 text element"
        );
        assert!(output.contains(">5</text>"), "should show fret number 5");
    }

    #[test]
    fn fret_number_open_string_shows_zero() {
        let (_, _, staff) = setup();
        let mut svg = make_svg();
        draw_fret_number_at(&mut svg, &staff, 3, 0, 500.0);
        let output = svg.to_svg();
        assert!(output.contains(">0</text>"), "open string should show 0");
    }

    #[test]
    fn fret_number_two_digits_shows_correctly() {
        let (_, _, staff) = setup();
        let mut svg = make_svg();
        draw_fret_number_at(&mut svg, &staff, 1, 12, 500.0);
        let output = svg.to_svg();
        assert!(output.contains(">12</text>"), "should show fret 12");
    }

    #[test]
    fn fret_number_text_is_bold_centered() {
        let (_, _, staff) = setup();
        let mut svg = make_svg();
        draw_fret_number_at(&mut svg, &staff, 1, 7, 500.0);
        let output = svg.to_svg();
        assert!(output.contains(r#"font-weight="bold""#));
        assert!(output.contains(r#"text-anchor="middle""#));
    }

    #[test]
    fn fret_number_text_has_central_baseline() {
        let (_, _, staff) = setup();
        let mut svg = make_svg();
        draw_fret_number_at(&mut svg, &staff, 1, 5, 500.0);
        let output = svg.to_svg();
        assert!(
            output.contains(r#"dominant-baseline="central""#),
            "fret numbers must be vertically centered on string lines"
        );
    }

    #[test]
    fn fret_number_bg_rect_is_white_no_stroke() {
        let (_, _, staff) = setup();
        let mut svg = make_svg();
        draw_fret_number_at(&mut svg, &staff, 1, 5, 500.0);
        let output = svg.to_svg();
        assert!(output.contains(r#"fill="white""#), "bg should be white");
        assert!(output.contains(r#"stroke="none""#), "bg should have no stroke");
    }

    #[test]
    fn multiple_fret_numbers_produce_multiple_elements() {
        let (_, _, staff) = setup();
        let mut svg = make_svg();
        draw_fret_number_at(&mut svg, &staff, 1, 0, 500.0);
        draw_fret_number_at(&mut svg, &staff, 2, 3, 500.0);
        draw_fret_number_at(&mut svg, &staff, 6, 7, 500.0);
        let output = svg.to_svg();
        assert_eq!(output.matches("<rect ").count(), 3);
        assert_eq!(output.matches("<text ").count(), 3);
        assert!(output.contains(">0</text>"));
        assert!(output.contains(">3</text>"));
        assert!(output.contains(">7</text>"));
    }

    #[test]
    fn different_strings_have_different_y_positions() {
        let (_, _, staff) = setup();
        let mut svg1 = make_svg();
        let mut svg2 = make_svg();
        draw_fret_number_at(&mut svg1, &staff, 1, 5, 500.0);
        draw_fret_number_at(&mut svg2, &staff, 6, 5, 500.0);
        assert_ne!(
            svg1.to_svg(),
            svg2.to_svg(),
            "same fret on different strings should produce different y positions"
        );
    }

    #[test]
    fn full_tab_staff_with_clef_and_notes() {
        let (font, config, staff) = setup();
        let mut svg = make_svg();
        draw_tab_staff_lines(&mut svg, &staff, &config);
        draw_tab_clef(&mut svg, &staff, &font).unwrap();
        draw_fret_number_at(&mut svg, &staff, 1, 0, 1500.0);
        draw_fret_number_at(&mut svg, &staff, 2, 1, 1500.0);
        draw_fret_number_at(&mut svg, &staff, 3, 0, 1500.0);
        draw_fret_number_at(&mut svg, &staff, 4, 2, 1500.0);
        draw_fret_number_at(&mut svg, &staff, 5, 3, 1500.0);
        let output = svg.to_svg();
        assert!(output.starts_with("<svg"));
        assert!(output.contains("</svg>"));
        assert_eq!(output.matches("<line ").count(), 6);
        assert_eq!(output.matches("<path ").count(), 1);
        assert_eq!(output.matches("<text ").count(), 5);
        assert_eq!(output.matches("<rect ").count(), 5);
    }
}
