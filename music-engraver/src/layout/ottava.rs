/// Ottava bracket layout — 8va/8vb/15ma/15mb octave transposition lines.
///
/// Ottava lines indicate that a passage should be played one or two octaves
/// higher or lower than written. They appear as a dashed horizontal line
/// above the staff (8va, 15ma) or below the staff (8vb, 15mb) with a text
/// label at the start and an optional hook at the end.
use crate::layout::staff::StaffLayout;

/// Kind of ottava transposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OttavaKind {
    /// One octave up (8va — "ottava alta"). Line appears above the staff.
    Ottava8va,
    /// One octave down (8vb — "ottava bassa"). Line appears below the staff.
    Ottava8vb,
    /// Two octaves up (15ma — "quindicesima alta"). Line appears above the staff.
    Ottava15ma,
    /// Two octaves down (15mb — "quindicesima bassa"). Line appears below the staff.
    Ottava15mb,
}

impl OttavaKind {
    /// Whether this ottava line appears above the staff.
    pub fn is_above(&self) -> bool {
        matches!(self, OttavaKind::Ottava8va | OttavaKind::Ottava15ma)
    }

    /// Text label for the ottava bracket start.
    pub fn label(&self) -> &'static str {
        match self {
            OttavaKind::Ottava8va => "8va",
            OttavaKind::Ottava8vb => "8vb",
            OttavaKind::Ottava15ma => "15ma",
            OttavaKind::Ottava15mb => "15mb",
        }
    }
}

/// Result of laying out an ottava bracket.
#[derive(Debug, Clone)]
pub struct OttavaBracketLayout {
    /// Kind of ottava (determines above/below and label).
    pub kind: OttavaKind,
    /// X-coordinate of the label start (left edge).
    pub x_start: f64,
    /// X-coordinate of the bracket end (right edge).
    pub x_end: f64,
    /// Y-coordinate of the dashed line.
    pub y_line: f64,
    /// X-coordinate where the dashed line begins (after the text label).
    pub x_line_start: f64,
    /// Text label (e.g. "8va").
    pub label: String,
    /// X, Y position for the text label.
    pub label_x: f64,
    pub label_y: f64,
    /// Font size for the label in font design units.
    pub font_size: f64,
    /// Line thickness in font design units.
    pub line_thickness: f64,
    /// Dash length for the dashed line, in font design units.
    pub dash_length: f64,
    /// Gap between dashes, in font design units.
    pub dash_gap: f64,
    /// Hook height at the end (downward for above, upward for below).
    pub hook_height: f64,
    /// Whether to draw a hook at the end.
    pub has_end_hook: bool,
}

/// Distance above the top staff line for 8va/15ma lines, in staff spaces.
pub const OTTAVA_ABOVE_STAFF_SS: f64 = 3.5;

/// Distance below the bottom staff line for 8vb/15mb lines, in staff spaces.
pub const OTTAVA_BELOW_STAFF_SS: f64 = 3.5;

/// Font size for the ottava text label, in staff spaces.
const OTTAVA_FONT_SIZE_SS: f64 = 1.4;

/// Estimated width of the text label in staff spaces (approximate, varies
/// by font). Used to offset the start of the dashed line.
const OTTAVA_LABEL_WIDTH_SS: f64 = 2.5;

/// Dash length in staff spaces.
const OTTAVA_DASH_LENGTH_SS: f64 = 0.8;

/// Gap between dashes in staff spaces.
const OTTAVA_DASH_GAP_SS: f64 = 0.4;

/// Hook height at the bracket end, in staff spaces.
const OTTAVA_HOOK_HEIGHT_SS: f64 = 1.0;

/// Lay out an ottava bracket above or below the staff.
///
/// `x_start` is the horizontal position of the first affected note.
/// `x_end` is the horizontal position past the last affected note.
/// `has_end_hook` controls whether a vertical hook is drawn at the right end
/// (true for the final segment, false for continuation segments).
pub fn layout_ottava_bracket(
    kind: OttavaKind,
    x_start: f64,
    x_end: f64,
    staff: &StaffLayout,
    staff_space: f64,
    has_end_hook: bool,
) -> OttavaBracketLayout {
    let y_line = if kind.is_above() {
        // Above top staff line
        staff.y_of(8) - OTTAVA_ABOVE_STAFF_SS * staff_space
    } else {
        // Below bottom staff line
        staff.y_of(0) + OTTAVA_BELOW_STAFF_SS * staff_space
    };

    let font_size = OTTAVA_FONT_SIZE_SS * staff_space;
    let line_thickness = 0.12 * staff_space;
    let dash_length = OTTAVA_DASH_LENGTH_SS * staff_space;
    let dash_gap = OTTAVA_DASH_GAP_SS * staff_space;

    let label_x = x_start;
    // For above-staff: baseline sits on the line; for below-staff: text above the line
    let label_y = if kind.is_above() {
        y_line + font_size * 0.35
    } else {
        y_line - font_size * 0.15
    };

    let label = kind.label().to_string();
    let label_width_est = OTTAVA_LABEL_WIDTH_SS * staff_space;
    let x_line_start = x_start + label_width_est;

    let hook_height = OTTAVA_HOOK_HEIGHT_SS * staff_space;

    OttavaBracketLayout {
        kind,
        x_start,
        x_end,
        y_line,
        x_line_start,
        label,
        label_x,
        label_y,
        font_size,
        line_thickness,
        dash_length,
        dash_gap,
        hook_height,
        has_end_hook,
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
    fn ottava_8va_is_above() {
        assert!(OttavaKind::Ottava8va.is_above());
    }

    #[test]
    fn ottava_8vb_is_below() {
        assert!(!OttavaKind::Ottava8vb.is_above());
    }

    #[test]
    fn ottava_15ma_is_above() {
        assert!(OttavaKind::Ottava15ma.is_above());
    }

    #[test]
    fn ottava_15mb_is_below() {
        assert!(!OttavaKind::Ottava15mb.is_above());
    }

    #[test]
    fn label_text_matches_kind() {
        assert_eq!(OttavaKind::Ottava8va.label(), "8va");
        assert_eq!(OttavaKind::Ottava8vb.label(), "8vb");
        assert_eq!(OttavaKind::Ottava15ma.label(), "15ma");
        assert_eq!(OttavaKind::Ottava15mb.label(), "15mb");
    }

    #[test]
    fn above_bracket_y_is_above_staff_top() {
        let staff = test_staff();
        let layout =
            layout_ottava_bracket(OttavaKind::Ottava8va, 100.0, 800.0, &staff, 250.0, true);
        let staff_top = staff.y_of(8);
        assert!(
            layout.y_line < staff_top,
            "8va y_line {} should be above staff top {}",
            layout.y_line,
            staff_top
        );
    }

    #[test]
    fn below_bracket_y_is_below_staff_bottom() {
        let staff = test_staff();
        let layout =
            layout_ottava_bracket(OttavaKind::Ottava8vb, 100.0, 800.0, &staff, 250.0, true);
        let staff_bottom = staff.y_of(0);
        assert!(
            layout.y_line > staff_bottom,
            "8vb y_line {} should be below staff bottom {}",
            layout.y_line,
            staff_bottom
        );
    }

    #[test]
    fn x_coordinates_preserved() {
        let layout = layout_ottava_bracket(
            OttavaKind::Ottava8va,
            150.0,
            900.0,
            &test_staff(),
            250.0,
            true,
        );
        assert!((layout.x_start - 150.0).abs() < f64::EPSILON);
        assert!((layout.x_end - 900.0).abs() < f64::EPSILON);
    }

    #[test]
    fn line_start_is_after_label() {
        let layout = layout_ottava_bracket(
            OttavaKind::Ottava8va,
            100.0,
            800.0,
            &test_staff(),
            250.0,
            true,
        );
        assert!(
            layout.x_line_start > layout.x_start,
            "dashed line should start after label"
        );
    }

    #[test]
    fn has_end_hook_when_requested() {
        let layout = layout_ottava_bracket(
            OttavaKind::Ottava8va,
            100.0,
            800.0,
            &test_staff(),
            250.0,
            true,
        );
        assert!(layout.has_end_hook);
    }

    #[test]
    fn no_end_hook_when_not_requested() {
        let layout = layout_ottava_bracket(
            OttavaKind::Ottava8va,
            100.0,
            800.0,
            &test_staff(),
            250.0,
            false,
        );
        assert!(!layout.has_end_hook);
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let layout_250 = layout_ottava_bracket(
            OttavaKind::Ottava8va,
            0.0,
            500.0,
            &test_staff(),
            250.0,
            true,
        );
        let staff_500 = StaffLayout::new(0.0, 0.0, 2000.0, 500.0);
        let layout_500 =
            layout_ottava_bracket(OttavaKind::Ottava8va, 0.0, 500.0, &staff_500, 500.0, true);
        assert!((layout_500.font_size / layout_250.font_size - 2.0).abs() < 0.01);
    }

    #[test]
    fn dash_length_scales_with_staff_space() {
        let layout_250 = layout_ottava_bracket(
            OttavaKind::Ottava8va,
            0.0,
            500.0,
            &test_staff(),
            250.0,
            true,
        );
        let staff_500 = StaffLayout::new(0.0, 0.0, 2000.0, 500.0);
        let layout_500 =
            layout_ottava_bracket(OttavaKind::Ottava8va, 0.0, 500.0, &staff_500, 500.0, true);
        assert!((layout_500.dash_length / layout_250.dash_length - 2.0).abs() < 0.01);
    }

    #[test]
    fn above_and_below_produce_different_y() {
        let staff = test_staff();
        let above = layout_ottava_bracket(OttavaKind::Ottava8va, 100.0, 800.0, &staff, 250.0, true);
        let below = layout_ottava_bracket(OttavaKind::Ottava8vb, 100.0, 800.0, &staff, 250.0, true);
        assert!(
            (above.y_line - below.y_line).abs() > 100.0,
            "above and below y should differ substantially"
        );
    }

    #[test]
    fn hook_height_is_positive() {
        let layout = layout_ottava_bracket(
            OttavaKind::Ottava8va,
            100.0,
            800.0,
            &test_staff(),
            250.0,
            true,
        );
        assert!(layout.hook_height > 0.0);
    }
}
