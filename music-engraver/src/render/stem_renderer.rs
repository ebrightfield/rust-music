use crate::font::EngravingConfig;
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{stem_length_staff_spaces, StemDirection};
use crate::layout::StaffPosition;
use crate::render::SvgWriter;

/// Compute the x-coordinate of a stem given the notehead's x and advance width.
///
/// Stem-up: stem is on the right side of the notehead (x + advance_width).
/// Stem-down: stem is on the left side of the notehead (x).
///
/// The stem thickness is centered on the attachment point — we offset by
/// half the stem thickness inward so the stem visually aligns with the
/// notehead edge.
pub fn stem_x(
    notehead_x: f64,
    notehead_advance: f64,
    direction: StemDirection,
    stem_thickness: f64,
) -> f64 {
    match direction {
        // Right edge, inset by half stem width so the stem's right edge
        // aligns with the notehead's right edge
        StemDirection::Up => notehead_x + notehead_advance - stem_thickness / 2.0,
        // Left edge, outset by half stem width so the stem's left edge
        // aligns with the notehead's left edge
        StemDirection::Down => notehead_x + stem_thickness / 2.0,
    }
}

/// Compute the y-coordinates (top, bottom) of a stem for a note at the given
/// position drawn at `scale` (1.0 for normal size).
///
/// Returns `(y_top, y_bottom)` where `y_top < y_bottom` (SVG: y increases downward).
pub fn stem_endpoints(
    staff: &StaffLayout,
    position: StaffPosition,
    direction: StemDirection,
    scale: f64,
) -> (f64, f64) {
    let notehead_y = staff.y_of(position);
    let length_ss = stem_length_staff_spaces(position, direction, scale);
    let length_fu = length_ss * staff.staff_space;

    match direction {
        StemDirection::Up => {
            let y_top = notehead_y - length_fu;
            (y_top, notehead_y)
        }
        StemDirection::Down => {
            let y_bottom = notehead_y + length_fu;
            (notehead_y, y_bottom)
        }
    }
}

/// Draw a stem for a note drawn at `scale` (1.0 for normal size).
///
/// The stem is drawn as a vertical line from the notehead to the stem tip,
/// using `stem_thickness` from the engraving config.
#[allow(clippy::too_many_arguments)]
pub fn draw_stem(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    config: &EngravingConfig,
    notehead_x: f64,
    notehead_advance: f64,
    position: StaffPosition,
    direction: StemDirection,
    scale: f64,
) {
    let thickness = config.stem_thickness_fu() * scale;
    let x = stem_x(notehead_x, notehead_advance, direction, thickness);
    let (y_top, y_bottom) = stem_endpoints(staff, position, direction, scale);

    svg.add_line(x, y_top, x, y_bottom, "black", thickness);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::font::MusicFont;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    // --- stem_x ---

    #[test]
    fn stem_up_is_on_right_side() {
        let x = stem_x(500.0, 295.0, StemDirection::Up, 30.0);
        // Right edge of notehead minus half stem: 500 + 295 - 15 = 780
        assert!((x - 780.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stem_down_is_on_left_side() {
        let x = stem_x(500.0, 295.0, StemDirection::Down, 30.0);
        // Left edge of notehead plus half stem: 500 + 15 = 515
        assert!((x - 515.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stem_x_with_zero_thickness() {
        // Edge case: zero thickness shouldn't panic
        let x_up = stem_x(100.0, 200.0, StemDirection::Up, 0.0);
        assert!((x_up - 300.0).abs() < f64::EPSILON);
        let x_dn = stem_x(100.0, 200.0, StemDirection::Down, 0.0);
        assert!((x_dn - 100.0).abs() < f64::EPSILON);
    }

    // --- stem_endpoints ---

    #[test]
    fn stem_up_from_bottom_line() {
        let (_, _, staff) = setup();
        // Position 0 (bottom line), stem up, default length 3.5 ss
        let (y_top, y_bottom) = stem_endpoints(&staff, 0, StemDirection::Up, 1.0);
        let notehead_y = staff.y_of(0); // 1000.0
        assert!((y_bottom - notehead_y).abs() < f64::EPSILON);
        // Stem goes up (negative y direction): 3.5 * 250 = 875
        let expected_top = notehead_y - 875.0; // 125.0
        assert!((y_top - expected_top).abs() < f64::EPSILON);
    }

    #[test]
    fn stem_down_from_top_line() {
        let (_, _, staff) = setup();
        // Position 8 (top line), stem down, default length 3.5 ss
        let (y_top, y_bottom) = stem_endpoints(&staff, 8, StemDirection::Down, 1.0);
        let notehead_y = staff.y_of(8); // 0.0
        assert!((y_top - notehead_y).abs() < f64::EPSILON);
        let expected_bottom = notehead_y + 875.0;
        assert!((y_bottom - expected_bottom).abs() < f64::EPSILON);
    }

    #[test]
    fn stem_up_from_middle_line() {
        let (_, _, staff) = setup();
        // Position 4, stem up, default length 3.5 ss
        let (y_top, y_bottom) = stem_endpoints(&staff, 4, StemDirection::Up, 1.0);
        let notehead_y = staff.y_of(4); // 500.0
        assert!((y_bottom - notehead_y).abs() < f64::EPSILON);
        assert!((y_top - (500.0 - 875.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn stem_extends_for_far_ledger_note() {
        let (_, _, staff) = setup();
        // Position -4, stem up: length = (4-(-4))/2 = 4.0 ss
        let (y_top, y_bottom) = stem_endpoints(&staff, -4, StemDirection::Up, 1.0);
        let notehead_y = staff.y_of(-4); // (8-(-4)) * 125 = 1500
        assert!((y_bottom - notehead_y).abs() < f64::EPSILON);
        let expected_top = notehead_y - 4.0 * 250.0; // 1500 - 1000 = 500
        assert!((y_top - expected_top).abs() < f64::EPSILON);
        // Tip at y=500 = middle line — correct
        assert!((y_top - staff.y_of(4)).abs() < f64::EPSILON);
    }

    #[test]
    fn stem_top_always_less_than_bottom() {
        let (_, _, staff) = setup();
        for pos in -6..=14 {
            for dir in [StemDirection::Up, StemDirection::Down] {
                let (y_top, y_bottom) = stem_endpoints(&staff, pos, dir, 1.0);
                assert!(
                    y_top < y_bottom,
                    "pos={pos}, dir={dir:?}: y_top={y_top} should be < y_bottom={y_bottom}"
                );
            }
        }
    }

    // --- draw_stem ---

    #[test]
    fn draw_stem_produces_line_element() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 2000.0);
        draw_stem(
            &mut svg,
            &staff,
            &config,
            500.0,
            295.0,
            4,
            StemDirection::Up,
            1.0,
        );
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            1,
            "stem should produce exactly one <line> element"
        );
    }

    #[test]
    fn draw_stem_uses_config_thickness() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 2000.0);
        draw_stem(
            &mut svg,
            &staff,
            &config,
            500.0,
            295.0,
            4,
            StemDirection::Down,
            1.0,
        );
        let output = svg.to_svg();
        let expected_sw = format!("stroke-width=\"{}\"", config.stem_thickness_fu());
        assert!(
            output.contains(&expected_sw),
            "stem should use stem_thickness from config"
        );
    }

    #[test]
    fn draw_stem_up_x_on_right() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 2000.0);
        let notehead_x = 500.0;
        let advance = 295.0;
        draw_stem(
            &mut svg,
            &staff,
            &config,
            notehead_x,
            advance,
            2,
            StemDirection::Up,
            1.0,
        );
        let output = svg.to_svg();

        let thickness = config.stem_thickness_fu();
        let expected_x = notehead_x + advance - thickness / 2.0;
        let x_str = format!("x1=\"{expected_x}\"");
        assert!(
            output.contains(&x_str),
            "stem-up x should be on right side of notehead: expected {x_str}"
        );
    }

    #[test]
    fn draw_stem_down_x_on_left() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 2000.0);
        let notehead_x = 500.0;
        let advance = 295.0;
        draw_stem(
            &mut svg,
            &staff,
            &config,
            notehead_x,
            advance,
            6,
            StemDirection::Down,
            1.0,
        );
        let output = svg.to_svg();

        let thickness = config.stem_thickness_fu();
        let expected_x = notehead_x + thickness / 2.0;
        let x_str = format!("x1=\"{expected_x}\"");
        assert!(
            output.contains(&x_str),
            "stem-down x should be on left side of notehead"
        );
    }

    #[test]
    fn draw_stem_vertical_y_coordinates() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -500.0, 6000.0, 3000.0);
        // Position 0, stem up: y_bottom=1000, y_top=1000-875=125
        draw_stem(
            &mut svg,
            &staff,
            &config,
            500.0,
            295.0,
            0,
            StemDirection::Up,
            1.0,
        );
        let output = svg.to_svg();

        let (y_top, y_bottom) = stem_endpoints(&staff, 0, StemDirection::Up, 1.0);
        let y1_str = format!("y1=\"{y_top}\"");
        let y2_str = format!("y2=\"{y_bottom}\"");
        assert!(output.contains(&y1_str), "should contain y1 (top)");
        assert!(output.contains(&y2_str), "should contain y2 (bottom)");
    }
}
