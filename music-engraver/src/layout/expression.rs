/// Expression text layout — italic performance directions below the staff.
///
/// Standard engraving convention: expression text (e.g. "dolce", "espressivo",
/// "legato", "cantabile") appears below the staff in italic, centered on the
/// note it applies to, typically at the same vertical level as dynamics.

use crate::layout::staff::StaffLayout;

/// Result of laying out an expression text marking.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionLayout {
    /// The expression text content.
    pub text: String,
    /// Center x-position (aligned with the note it applies to).
    pub x_center: f64,
    /// Baseline y-position of the text (below the staff).
    pub y_baseline: f64,
    /// Font size in font design units.
    pub font_size: f64,
}

/// Distance below the bottom staff line for expression text placement, in staff spaces.
/// Slightly below dynamics (which sit at 2.5ss) to avoid collision.
const EXPRESSION_BELOW_STAFF_SS: f64 = 4.0;

/// Font size for expression text, in staff spaces.
const EXPRESSION_FONT_SIZE_SS: f64 = 1.4;

/// Lay out an expression text marking below the staff.
///
/// `text` is the expression content (e.g. "dolce", "legato").
/// `note_center_x` is the horizontal center of the note it applies to.
/// `staff` provides vertical reference for placement below the bottom staff line.
/// `staff_space` is the staff space size in font design units.
pub fn layout_expression(
    text: &str,
    note_center_x: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> ExpressionLayout {
    let font_size = EXPRESSION_FONT_SIZE_SS * staff_space;
    let below_offset = EXPRESSION_BELOW_STAFF_SS * staff_space;

    // Bottom staff line y
    let bottom_line_y = staff.y_of(0);
    let y_baseline = bottom_line_y + below_offset;

    ExpressionLayout {
        text: text.to_string(),
        x_center: note_center_x,
        y_baseline,
        font_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    #[test]
    fn text_content_preserved() {
        let layout = layout_expression("dolce", 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "dolce");
    }

    #[test]
    fn x_center_preserved() {
        let layout = layout_expression("legato", 1234.0, &test_staff(), 250.0);
        assert_eq!(layout.x_center, 1234.0);
    }

    #[test]
    fn baseline_below_bottom_staff_line() {
        let staff = test_staff();
        let bottom_y = staff.y_of(0);
        let layout = layout_expression("espressivo", 0.0, &staff, 250.0);
        assert!(
            layout.y_baseline > bottom_y,
            "baseline {} should be below bottom line {}",
            layout.y_baseline,
            bottom_y
        );
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = layout_expression("a", 0.0, &test_staff(), 125.0);
        let large = layout_expression("a", 0.0, &test_staff(), 250.0);
        assert!(
            (large.font_size - 2.0 * small.font_size).abs() < 0.01,
            "font size should scale linearly with staff_space"
        );
    }

    #[test]
    fn different_positions_produce_different_layouts() {
        let a = layout_expression("dolce", 100.0, &test_staff(), 250.0);
        let b = layout_expression("dolce", 900.0, &test_staff(), 250.0);
        assert!((a.x_center - b.x_center).abs() > 700.0);
    }

    #[test]
    fn font_size_is_positive() {
        let layout = layout_expression("x", 0.0, &test_staff(), 250.0);
        assert!(layout.font_size > 0.0);
    }

    #[test]
    fn below_staff_offset_is_correct() {
        let staff = test_staff();
        let bottom_y = staff.y_of(0);
        let layout = layout_expression("test", 0.0, &staff, 250.0);
        let expected = bottom_y + EXPRESSION_BELOW_STAFF_SS * 250.0;
        assert!(
            (layout.y_baseline - expected).abs() < 0.01,
            "expected y={expected}, got {}",
            layout.y_baseline
        );
    }
}
