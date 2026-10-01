//! Layout geometry for tablature slide lines.
//!
//! A slide is a diagonal line between two fret numbers on the same string,
//! indicating the player should slide from one fret position to another.

use super::tab::TabStaffLayout;

/// Horizontal padding from fret number center to slide line endpoint (staff spaces ratio).
/// Keeps the line from overlapping the fret number text.
const SLIDE_H_PADDING_RATIO: f64 = 0.6;

/// Layout result for a single slide line between two fret positions.
#[derive(Clone, Debug)]
pub struct TabSlideLayout {
    /// X-coordinate of the slide line start (after the source fret number).
    pub x_start: f64,
    /// Y-coordinate of the slide line start (on the string line).
    pub y_start: f64,
    /// X-coordinate of the slide line end (before the target fret number).
    pub x_end: f64,
    /// Y-coordinate of the slide line end (on the string line).
    pub y_end: f64,
    /// Stroke width for the slide line.
    pub stroke_width: f64,
}

/// Compute slide line geometry between two fret positions on the same string.
///
/// The line starts after the source fret number and ends before the target
/// fret number, with horizontal padding to avoid overlapping the text.
/// The y-coordinates are on the string line for both endpoints (the line
/// is diagonal only if the two x-positions differ, which they always do
/// in practice since events are at different horizontal positions).
///
/// Returns `None` if the two x-positions are too close for a visible line
/// (less than 2× padding apart).
pub fn layout_tab_slide(
    tab_staff: &TabStaffLayout,
    string: u8,
    x_source: f64,
    x_target: f64,
    stroke_width: f64,
) -> Option<TabSlideLayout> {
    let h_padding = tab_staff.staff_space * SLIDE_H_PADDING_RATIO;
    let x_start = x_source + h_padding;
    let x_end = x_target - h_padding;

    // Don't draw if endpoints would overlap
    if x_end <= x_start {
        return None;
    }

    let y = tab_staff.string_y(string);

    Some(TabSlideLayout {
        x_start,
        y_start: y,
        x_end,
        y_end: y,
        stroke_width,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staff() -> TabStaffLayout {
        TabStaffLayout::new(0.0, 0.0, 5000.0, 250.0, 6)
    }

    #[test]
    fn slide_layout_returns_some_for_normal_spacing() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 500.0, 1000.0, 5.0);
        assert!(layout.is_some());
    }

    #[test]
    fn slide_x_start_is_after_source() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 500.0, 1000.0, 5.0).unwrap();
        assert!(
            layout.x_start > 500.0,
            "slide start should be after source fret x"
        );
    }

    #[test]
    fn slide_x_end_is_before_target() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 500.0, 1000.0, 5.0).unwrap();
        assert!(
            layout.x_end < 1000.0,
            "slide end should be before target fret x"
        );
    }

    #[test]
    fn slide_y_on_string_line() {
        let staff = test_staff();
        let expected_y = staff.string_y(3);
        let layout = layout_tab_slide(&staff, 3, 500.0, 1000.0, 5.0).unwrap();
        assert!((layout.y_start - expected_y).abs() < 0.001);
        assert!((layout.y_end - expected_y).abs() < 0.001);
    }

    #[test]
    fn slide_returns_none_when_too_close() {
        let staff = test_staff();
        // Two fret positions very close together — padding overlap
        let layout = layout_tab_slide(&staff, 1, 500.0, 510.0, 5.0);
        assert!(
            layout.is_none(),
            "should return None when x positions too close"
        );
    }

    #[test]
    fn slide_preserves_stroke_width() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 500.0, 1000.0, 7.5).unwrap();
        assert!((layout.stroke_width - 7.5).abs() < 0.001);
    }

    #[test]
    fn slide_different_strings_produce_different_y() {
        let staff = test_staff();
        let layout1 = layout_tab_slide(&staff, 1, 500.0, 1000.0, 5.0).unwrap();
        let layout6 = layout_tab_slide(&staff, 6, 500.0, 1000.0, 5.0).unwrap();
        assert!(
            (layout1.y_start - layout6.y_start).abs() > 100.0,
            "string 1 and string 6 should have very different y positions"
        );
    }

    #[test]
    fn slide_padding_scales_with_staff_space() {
        let small = TabStaffLayout::new(0.0, 0.0, 5000.0, 100.0, 6);
        let large = TabStaffLayout::new(0.0, 0.0, 5000.0, 400.0, 6);
        let layout_small = layout_tab_slide(&small, 1, 500.0, 1000.0, 5.0).unwrap();
        let layout_large = layout_tab_slide(&large, 1, 500.0, 1000.0, 5.0).unwrap();
        // Larger staff space → more padding → shorter slide line
        let len_small = layout_small.x_end - layout_small.x_start;
        let len_large = layout_large.x_end - layout_large.x_start;
        assert!(
            len_small > len_large,
            "larger staff space should produce shorter slide line (more padding)"
        );
    }

    #[test]
    fn slide_x_end_always_greater_than_x_start() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 100.0, 2000.0, 5.0).unwrap();
        assert!(layout.x_end > layout.x_start);
    }

    #[test]
    fn slide_with_four_string_staff() {
        let staff = TabStaffLayout::new(0.0, 0.0, 5000.0, 250.0, 4);
        let layout = layout_tab_slide(&staff, 2, 500.0, 1000.0, 5.0);
        assert!(layout.is_some());
        let l = layout.unwrap();
        let expected_y = staff.string_y(2);
        assert!((l.y_start - expected_y).abs() < 0.001);
    }
}
