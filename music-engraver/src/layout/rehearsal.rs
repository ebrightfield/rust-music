/// Rehearsal mark layout — boxed letters or numbers above the staff.
///
/// Standard engraving convention: rehearsal marks appear above the staff
/// in a bold font, typically enclosed in a box or circle. We support
/// boxed text (the most common style).
use crate::layout::staff::StaffLayout;

/// Style of the rehearsal mark enclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RehearsalStyle {
    /// Enclosed in a rectangle (most common).
    Boxed,
    /// No enclosure, just bold text.
    Plain,
}

/// Result of laying out a rehearsal mark.
#[derive(Debug, Clone)]
pub struct RehearsalMarkLayout {
    /// Text content (e.g. "A", "B", "12").
    pub text: String,
    /// Center x-position (aligned with the note/barline it marks).
    pub x_center: f64,
    /// Baseline y-position of the text (above the staff).
    pub y_baseline: f64,
    /// Font size in font design units.
    pub font_size: f64,
    /// Enclosure style.
    pub style: RehearsalStyle,
    /// Box geometry (if Boxed): (x, y, width, height) of the enclosing rect.
    pub box_rect: Option<(f64, f64, f64, f64)>,
    /// Box stroke width (in font design units).
    pub box_stroke_width: f64,
}

/// Distance above the top staff line for rehearsal mark placement,
/// in staff spaces.
const REHEARSAL_ABOVE_STAFF_SS: f64 = 2.5;

/// Font size for rehearsal marks, in staff spaces.
const REHEARSAL_FONT_SIZE_SS: f64 = 1.8;

/// Horizontal padding inside the box, in staff spaces.
const REHEARSAL_BOX_PAD_X_SS: f64 = 0.4;

/// Vertical padding inside the box (above baseline), in staff spaces.
const REHEARSAL_BOX_PAD_TOP_SS: f64 = 0.3;

/// Vertical padding inside the box (below baseline), in staff spaces.
const REHEARSAL_BOX_PAD_BOTTOM_SS: f64 = 0.15;

/// Box stroke width, in staff spaces.
const REHEARSAL_BOX_STROKE_SS: f64 = 0.1;

/// Lay out a rehearsal mark above the staff.
///
/// `text` is the rehearsal mark content (e.g. "A", "1").
/// `x_center` is the horizontal position to center the mark on.
/// `staff` provides vertical reference for "above the top staff line".
/// `staff_space` is the staff space size in font design units.
pub fn layout_rehearsal_mark(
    text: &str,
    x_center: f64,
    staff: &StaffLayout,
    staff_space: f64,
    style: RehearsalStyle,
) -> RehearsalMarkLayout {
    let font_size = REHEARSAL_FONT_SIZE_SS * staff_space;
    let above_offset = REHEARSAL_ABOVE_STAFF_SS * staff_space;

    // Top staff line is at the minimum y (y increases downward in our coordinate system)
    let top_line_y = staff.y_of(8); // staff position 8 = top line

    // Baseline sits above the top staff line
    let y_baseline = top_line_y - above_offset;

    let box_rect = if style == RehearsalStyle::Boxed {
        let pad_x = REHEARSAL_BOX_PAD_X_SS * staff_space;
        let pad_top = REHEARSAL_BOX_PAD_TOP_SS * staff_space;
        let pad_bottom = REHEARSAL_BOX_PAD_BOTTOM_SS * staff_space;

        // Estimate text width: roughly 0.6 × font_size per character
        let estimated_char_width = font_size * 0.6;
        let text_width = estimated_char_width * text.len() as f64;

        let box_width = text_width + 2.0 * pad_x;
        let box_height = font_size + pad_top + pad_bottom;
        let box_x = x_center - box_width / 2.0;
        // Box top is above baseline by (font_size ascent + padding)
        let box_y = y_baseline - font_size - pad_top;

        Some((box_x, box_y, box_width, box_height))
    } else {
        None
    };

    let box_stroke_width = REHEARSAL_BOX_STROKE_SS * staff_space;

    RehearsalMarkLayout {
        text: text.to_string(),
        x_center,
        y_baseline,
        font_size,
        style,
        box_rect,
        box_stroke_width,
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
    fn boxed_rehearsal_has_box_rect() {
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        assert!(layout.box_rect.is_some());
        assert_eq!(layout.style, RehearsalStyle::Boxed);
    }

    #[test]
    fn plain_rehearsal_has_no_box_rect() {
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Plain);
        assert!(layout.box_rect.is_none());
        assert_eq!(layout.style, RehearsalStyle::Plain);
    }

    #[test]
    fn rehearsal_baseline_above_top_staff_line() {
        let staff = test_staff();
        let top_y = staff.y_of(8);
        let layout =
            layout_rehearsal_mark("A", 500.0, &staff, 250.0, RehearsalStyle::Boxed);
        assert!(
            layout.y_baseline < top_y,
            "baseline {} should be above top line {}",
            layout.y_baseline,
            top_y
        );
    }

    #[test]
    fn rehearsal_x_center_preserved() {
        let layout =
            layout_rehearsal_mark("B", 1234.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        assert_eq!(layout.x_center, 1234.0);
    }

    #[test]
    fn box_centered_on_x() {
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        let (bx, _, bw, _) = layout.box_rect.unwrap();
        let box_center = bx + bw / 2.0;
        assert!(
            (box_center - 500.0).abs() < 1.0,
            "box center {} should be near 500",
            box_center
        );
    }

    #[test]
    fn wider_text_produces_wider_box() {
        let single = layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        let multi = layout_rehearsal_mark("ABC", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        let (_, _, w1, _) = single.box_rect.unwrap();
        let (_, _, w3, _) = multi.box_rect.unwrap();
        assert!(w3 > w1, "3-char box width {} should exceed 1-char {}", w3, w1);
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = layout_rehearsal_mark("A", 0.0, &test_staff(), 125.0, RehearsalStyle::Boxed);
        let large = layout_rehearsal_mark("A", 0.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        assert!(
            (large.font_size - 2.0 * small.font_size).abs() < 0.01,
            "font size should scale linearly with staff_space"
        );
    }

    #[test]
    fn box_stroke_width_positive() {
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        assert!(layout.box_stroke_width > 0.0);
    }

    #[test]
    fn text_content_preserved() {
        let layout =
            layout_rehearsal_mark("42", 500.0, &test_staff(), 250.0, RehearsalStyle::Plain);
        assert_eq!(layout.text, "42");
    }

    #[test]
    fn box_y_above_baseline() {
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        let (_, by, _, _) = layout.box_rect.unwrap();
        assert!(
            by < layout.y_baseline,
            "box top {} should be above baseline {}",
            by,
            layout.y_baseline
        );
    }

    #[test]
    fn different_positions_produce_different_layouts() {
        let a = layout_rehearsal_mark("A", 100.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        let b = layout_rehearsal_mark("A", 900.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        assert!((a.x_center - b.x_center).abs() > 700.0);
        let (ax, _, _, _) = a.box_rect.unwrap();
        let (bx, _, _, _) = b.box_rect.unwrap();
        assert!((ax - bx).abs() > 700.0);
    }
}
