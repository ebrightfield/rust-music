use crate::layout::staff::StaffLayout;

/// Distance from the middle staff line to the top/bottom edge of the H-bar,
/// in staff spaces. The bar spans from line 2 (space below middle) to
/// line 6 (space above middle), i.e. 1 staff space above and below the
/// middle line.
pub const HBAR_HALF_HEIGHT_SS: f64 = 1.0;

/// Thickness of the vertical serif strokes at each end of the H-bar,
/// in staff spaces.
pub const HBAR_SERIF_THICKNESS_SS: f64 = 0.16;

/// Horizontal padding from barline edges to H-bar ends, in staff spaces.
/// Prevents the bar from touching the barlines.
pub const HBAR_HORIZONTAL_PADDING_SS: f64 = 1.5;

/// Thickness of the horizontal bar (the crossbar of the H), in staff spaces.
pub const HBAR_BAR_THICKNESS_SS: f64 = 0.5;

/// Distance from the top of the H-bar to the baseline of the count number,
/// in staff spaces.
pub const COUNT_ABOVE_HBAR_SS: f64 = 0.8;

/// Font size for the measure count number, in staff spaces.
pub const COUNT_FONT_SIZE_SS: f64 = 1.8;

/// Computed geometry for a multi-measure rest H-bar with count number.
#[derive(Clone, Debug)]
pub struct MultiMeasureRestLayout {
    /// X coordinate of the left edge of the H-bar (left serif).
    pub x_left: f64,
    /// X coordinate of the right edge of the H-bar (right serif).
    pub x_right: f64,
    /// Y coordinate of the top of the vertical serifs.
    pub y_top: f64,
    /// Y coordinate of the bottom of the vertical serifs.
    pub y_bottom: f64,
    /// Y coordinate of the top edge of the horizontal bar.
    pub bar_y_top: f64,
    /// Y coordinate of the bottom edge of the horizontal bar.
    pub bar_y_bottom: f64,
    /// Thickness of the vertical serif strokes in font design units.
    pub serif_thickness: f64,
    /// X coordinate for the centered count number text.
    pub count_x: f64,
    /// Y coordinate for the count number text baseline.
    pub count_y: f64,
    /// Font size for the count number in font design units.
    pub count_font_size: f64,
    /// The number of measures of rest.
    pub count: u32,
}

/// Compute the layout for a multi-measure rest spanning a given width.
///
/// The H-bar (thick horizontal line with vertical serifs at each end) is
/// centered vertically on the middle staff line, spanning most of the
/// available width with padding from the edges. A count number is placed
/// above the bar, centered horizontally.
///
/// `x_left_edge` and `x_right_edge` define the available horizontal space
/// (typically the barline-to-barline span of the rest measure).
pub fn layout_multi_measure_rest(
    x_left_edge: f64,
    x_right_edge: f64,
    count: u32,
    staff: &StaffLayout,
) -> MultiMeasureRestLayout {
    let ss = staff.staff_space;

    let h_pad = HBAR_HORIZONTAL_PADDING_SS * ss;
    let x_left = x_left_edge + h_pad;
    let x_right = x_right_edge - h_pad;

    // Middle line is staff position 4
    let mid_y = staff.y_of(4);
    let half_height = HBAR_HALF_HEIGHT_SS * ss;
    let y_top = mid_y - half_height;
    let y_bottom = mid_y + half_height;

    let bar_half = HBAR_BAR_THICKNESS_SS * ss / 2.0;
    let bar_y_top = mid_y - bar_half;
    let bar_y_bottom = mid_y + bar_half;

    let serif_thickness = HBAR_SERIF_THICKNESS_SS * ss;

    let count_x = (x_left + x_right) / 2.0;
    let count_y = y_top - COUNT_ABOVE_HBAR_SS * ss;
    let count_font_size = COUNT_FONT_SIZE_SS * ss;

    MultiMeasureRestLayout {
        x_left,
        x_right,
        y_top,
        y_bottom,
        bar_y_top,
        bar_y_bottom,
        serif_thickness,
        count_x,
        count_y,
        count_font_size,
        count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 5000.0, 250.0)
    }

    #[test]
    fn hbar_centered_on_middle_line() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let mid_y = staff.y_of(4);
        let center = (layout.y_top + layout.y_bottom) / 2.0;
        assert!(
            (center - mid_y).abs() < 0.01,
            "H-bar should be centered on middle line: center={center}, mid_y={mid_y}"
        );
    }

    #[test]
    fn hbar_spans_one_space_each_side() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let height = layout.y_bottom - layout.y_top;
        let expected = 2.0 * HBAR_HALF_HEIGHT_SS * staff.staff_space;
        assert!(
            (height - expected).abs() < 0.01,
            "H-bar height should be {expected}, got {height}"
        );
    }

    #[test]
    fn hbar_padded_from_edges() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(100.0, 4900.0, 4, &staff);
        let pad = HBAR_HORIZONTAL_PADDING_SS * staff.staff_space;
        assert!(
            (layout.x_left - (100.0 + pad)).abs() < 0.01,
            "x_left should be padded from left edge"
        );
        assert!(
            (layout.x_right - (4900.0 - pad)).abs() < 0.01,
            "x_right should be padded from right edge"
        );
    }

    #[test]
    fn bar_thinner_than_serifs() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let bar_height = layout.bar_y_bottom - layout.bar_y_top;
        let serif_height = layout.y_bottom - layout.y_top;
        assert!(
            bar_height < serif_height,
            "horizontal bar ({bar_height}) should be thinner than serif span ({serif_height})"
        );
    }

    #[test]
    fn bar_centered_on_middle_line() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let mid_y = staff.y_of(4);
        let bar_center = (layout.bar_y_top + layout.bar_y_bottom) / 2.0;
        assert!(
            (bar_center - mid_y).abs() < 0.01,
            "bar center ({bar_center}) should match middle line ({mid_y})"
        );
    }

    #[test]
    fn count_above_hbar() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        assert!(
            layout.count_y < layout.y_top,
            "count y ({}) should be above (less than) H-bar top ({})",
            layout.count_y,
            layout.y_top
        );
    }

    #[test]
    fn count_centered_horizontally() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(100.0, 4900.0, 4, &staff);
        let expected_center = (layout.x_left + layout.x_right) / 2.0;
        assert!(
            (layout.count_x - expected_center).abs() < 0.01,
            "count x ({}) should be centered ({})",
            layout.count_x,
            expected_center
        );
    }

    #[test]
    fn count_preserved() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 12, &staff);
        assert_eq!(layout.count, 12);
    }

    #[test]
    fn serif_thickness_positive() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        assert!(layout.serif_thickness > 0.0);
    }

    #[test]
    fn font_size_positive() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        assert!(layout.count_font_size > 0.0);
    }

    #[test]
    fn different_counts_same_geometry() {
        let staff = test_staff();
        let l4 = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let l16 = layout_multi_measure_rest(0.0, 5000.0, 16, &staff);
        // Geometry identical, only count differs
        assert!((l4.x_left - l16.x_left).abs() < 0.01);
        assert!((l4.y_top - l16.y_top).abs() < 0.01);
        assert_ne!(l4.count, l16.count);
    }

    #[test]
    fn narrow_span_still_produces_layout() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 1000.0, 2, &staff);
        // Even if x_right < x_left (very narrow), we still get a layout
        // (renderer can decide whether to draw)
        assert_eq!(layout.count, 2);
    }

    #[test]
    fn nonzero_origin() {
        let staff = StaffLayout::new(500.0, 200.0, 5000.0, 250.0);
        let layout = layout_multi_measure_rest(500.0, 5500.0, 3, &staff);
        let mid_y = staff.y_of(4);
        let center = (layout.y_top + layout.y_bottom) / 2.0;
        assert!(
            (center - mid_y).abs() < 0.01,
            "should work with nonzero origin"
        );
    }
}
