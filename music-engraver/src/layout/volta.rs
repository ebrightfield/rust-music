/// Volta bracket layout — 1st/2nd ending brackets above the staff.
///
/// Volta brackets (also called "ending brackets") indicate repeated sections
/// where different endings are played on each repetition. They appear above the
/// staff as horizontal lines with optional vertical hooks at start/end, and
/// text labels like "1.", "2.", or "1.–3." at the left side.
///
/// A volta bracket can span one or more measures. For multi-measure voltas:
/// - First measure: left hook + text + top line (no right hook — open end)
/// - Middle measures: top line only (no hooks, no text)
/// - Last measure: top line + right hook (closed end)
///
/// Single-measure voltas get left hook + text + top line + right hook.
use crate::layout::staff::StaffLayout;

/// Which edges of a volta bracket segment have vertical hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoltaHooks {
    /// Left hook + right hook (single-measure volta or last measure).
    Both,
    /// Left hook only (first measure of multi-measure volta).
    LeftOnly,
    /// Right hook only (last measure of multi-measure volta, no text).
    RightOnly,
    /// No hooks (middle measure of multi-measure volta).
    Neither,
}

/// Annotation on a measure indicating it is part of a volta bracket.
#[derive(Debug, Clone, PartialEq)]
pub struct VoltaAnnotation {
    /// Text label (e.g. "1.", "2.", "1.–3."). `None` for continuation measures.
    pub text: Option<String>,
    /// Which edges have vertical hooks.
    pub hooks: VoltaHooks,
}

/// Result of laying out a volta bracket segment above a measure.
#[derive(Debug, Clone)]
pub struct VoltaBracketLayout {
    /// Left x-coordinate of the bracket (measure start).
    pub x_left: f64,
    /// Right x-coordinate of the bracket (measure end).
    pub x_right: f64,
    /// Y-coordinate of the top horizontal line.
    pub y_top: f64,
    /// Y-coordinate of the bottom of vertical hooks (closer to staff).
    pub y_hook_bottom: f64,
    /// Whether to draw a left vertical hook.
    pub left_hook: bool,
    /// Whether to draw a right vertical hook.
    pub right_hook: bool,
    /// Text label position (x, y) and content. `None` if no text.
    pub text: Option<(f64, f64, String)>,
    /// Font size for the text label in font design units.
    pub font_size: f64,
    /// Line thickness in font design units.
    pub line_thickness: f64,
}

/// Distance above the top staff line for the volta bracket,
/// in staff spaces.
pub const VOLTA_ABOVE_STAFF_SS: f64 = 3.0;

/// Hook drop height in staff spaces (vertical extent of the hook
/// measured downward from the top line).
const VOLTA_HOOK_HEIGHT_SS: f64 = 1.0;

/// Font size for the volta text label, in staff spaces.
const VOLTA_FONT_SIZE_SS: f64 = 1.4;

/// Left padding before the text label, in staff spaces.
const VOLTA_TEXT_PAD_LEFT_SS: f64 = 0.4;

/// Lay out a volta bracket segment above a measure.
///
/// `x_left` and `x_right` define the horizontal span (typically the measure's
/// full width). `staff` provides vertical reference for "above the top staff line".
pub fn layout_volta_bracket(
    annotation: &VoltaAnnotation,
    x_left: f64,
    x_right: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> VoltaBracketLayout {
    let y_top = staff.y_of(8) - VOLTA_ABOVE_STAFF_SS * staff_space;
    let y_hook_bottom = y_top + VOLTA_HOOK_HEIGHT_SS * staff_space;
    let font_size = VOLTA_FONT_SIZE_SS * staff_space;
    // Thin barline thickness is a good match for volta bracket lines
    let line_thickness = 0.16 * staff_space;

    let (left_hook, right_hook) = match annotation.hooks {
        VoltaHooks::Both => (true, true),
        VoltaHooks::LeftOnly => (true, false),
        VoltaHooks::RightOnly => (false, true),
        VoltaHooks::Neither => (false, false),
    };

    let text = annotation.text.as_ref().map(|t| {
        let text_x = x_left + VOLTA_TEXT_PAD_LEFT_SS * staff_space;
        // Baseline just below the top line for visual centering
        let text_y = y_top + font_size * 0.8;
        (text_x, text_y, t.clone())
    });

    VoltaBracketLayout {
        x_left,
        x_right,
        y_top,
        y_hook_bottom,
        left_hook,
        right_hook,
        text,
        font_size,
        line_thickness,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 2000.0, 250.0)
    }

    #[test]
    fn single_measure_volta_has_both_hooks() {
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout = layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0);
        assert!(layout.left_hook);
        assert!(layout.right_hook);
    }

    #[test]
    fn left_only_has_no_right_hook() {
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::LeftOnly,
        };
        let layout = layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0);
        assert!(layout.left_hook);
        assert!(!layout.right_hook);
    }

    #[test]
    fn right_only_has_no_left_hook() {
        let ann = VoltaAnnotation {
            text: None,
            hooks: VoltaHooks::RightOnly,
        };
        let layout = layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0);
        assert!(!layout.left_hook);
        assert!(layout.right_hook);
    }

    #[test]
    fn neither_has_no_hooks() {
        let ann = VoltaAnnotation {
            text: None,
            hooks: VoltaHooks::Neither,
        };
        let layout = layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0);
        assert!(!layout.left_hook);
        assert!(!layout.right_hook);
    }

    #[test]
    fn text_is_present_when_annotation_has_text() {
        let ann = VoltaAnnotation {
            text: Some("2.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout = layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0);
        let (tx, _ty, ref content) = layout.text.as_ref().unwrap();
        assert_eq!(content, "2.");
        // Text x should be to the right of x_left
        assert!(*tx > 100.0);
    }

    #[test]
    fn text_is_none_when_annotation_has_no_text() {
        let ann = VoltaAnnotation {
            text: None,
            hooks: VoltaHooks::RightOnly,
        };
        let layout = layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0);
        assert!(layout.text.is_none());
    }

    #[test]
    fn bracket_y_top_is_above_staff() {
        let staff = test_staff();
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout = layout_volta_bracket(&ann, 0.0, 500.0, &staff, 250.0);
        // Top of staff is y_of(8) = 0 - 4*250 = -1000
        // y_top should be above that (more negative)
        let staff_top = staff.y_of(8);
        assert!(layout.y_top < staff_top, "y_top {} should be above staff top {}", layout.y_top, staff_top);
    }

    #[test]
    fn hook_bottom_is_below_top() {
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout = layout_volta_bracket(&ann, 0.0, 500.0, &test_staff(), 250.0);
        // In SVG, y increases downward, so hook_bottom > y_top
        assert!(layout.y_hook_bottom > layout.y_top);
    }

    #[test]
    fn x_coordinates_match_input() {
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout = layout_volta_bracket(&ann, 200.0, 800.0, &test_staff(), 250.0);
        assert!((layout.x_left - 200.0).abs() < f64::EPSILON);
        assert!((layout.x_right - 800.0).abs() < f64::EPSILON);
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout_250 = layout_volta_bracket(&ann, 0.0, 500.0, &test_staff(), 250.0);
        let staff_500 = StaffLayout::new(0.0, 0.0, 2000.0, 500.0);
        let layout_500 = layout_volta_bracket(&ann, 0.0, 500.0, &staff_500, 500.0);
        assert!((layout_500.font_size / layout_250.font_size - 2.0).abs() < 0.01);
    }

    #[test]
    fn line_thickness_scales_with_staff_space() {
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout_250 = layout_volta_bracket(&ann, 0.0, 500.0, &test_staff(), 250.0);
        let staff_500 = StaffLayout::new(0.0, 0.0, 2000.0, 500.0);
        let layout_500 = layout_volta_bracket(&ann, 0.0, 500.0, &staff_500, 500.0);
        assert!((layout_500.line_thickness / layout_250.line_thickness - 2.0).abs() < 0.01);
    }

    #[test]
    fn text_x_has_left_padding() {
        let ann = VoltaAnnotation {
            text: Some("1.".to_string()),
            hooks: VoltaHooks::Both,
        };
        let layout = layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0);
        let (tx, _, _) = layout.text.as_ref().unwrap();
        let expected_pad = VOLTA_TEXT_PAD_LEFT_SS * 250.0;
        assert!((*tx - 100.0 - expected_pad).abs() < 0.01);
    }
}
