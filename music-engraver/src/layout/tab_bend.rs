//! Layout geometry for tablature bend notation.
//!
//! A bend is notated as an upward-curving arrow above the fret number on the
//! affected string, with text indicating the bend amount (e.g. "full", "1/2",
//! "1/4"). The arrow starts at the fret position and curves upward, with an
//! arrowhead at the top pointing up.

use super::tab::TabStaffLayout;

/// Horizontal half-width of the bend curve (staff spaces ratio).
const BEND_HALF_WIDTH_RATIO: f64 = 0.35;

/// Bend arrow height above the string line (staff spaces ratio).
const BEND_HEIGHT_RATIO: f64 = 1.6;

/// Arrowhead half-width (staff spaces ratio).
const ARROWHEAD_HALF_WIDTH_RATIO: f64 = 0.15;

/// Arrowhead height (staff spaces ratio).
const ARROWHEAD_HEIGHT_RATIO: f64 = 0.25;

/// The amount a string is bent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BendAmount {
    /// Quarter-step bend. Labeled "1/4".
    Quarter,
    /// Half-step bend. Labeled "1/2".
    Half,
    /// Whole-step bend. Labeled "full".
    Full,
    /// One-and-a-half-step bend. Labeled "1 1/2".
    OneAndHalf,
    /// Custom bend in semitones with a user-provided label.
    Custom(&'static str),
}

impl BendAmount {
    /// Text label for the bend amount.
    pub fn label(self) -> &'static str {
        match self {
            BendAmount::Quarter => "1/4",
            BendAmount::Half => "1/2",
            BendAmount::Full => "full",
            BendAmount::OneAndHalf => "1 1/2",
            BendAmount::Custom(s) => s,
        }
    }
}

/// Layout result for a bend arrow at a fret position.
#[derive(Clone, Debug)]
pub struct TabBendLayout {
    /// The bend amount (determines text label).
    pub amount: BendAmount,
    /// X-coordinate of the bend arrow origin (at the fret number).
    pub x: f64,
    /// Y-coordinate of the string line (arrow base).
    pub y_string: f64,
    /// Y-coordinate of the arrow tip (above the string, lower y in SVG).
    pub y_tip: f64,
    /// X-coordinate of the curve's control point (same as x for vertical bend).
    pub x_control: f64,
    /// Y-coordinate of the curve's control point (between string and tip).
    pub y_control: f64,
    /// Left x of arrowhead base.
    pub arrow_x_left: f64,
    /// Right x of arrowhead base.
    pub arrow_x_right: f64,
    /// Y-coordinate of arrowhead base (slightly below tip).
    pub arrow_y_base: f64,
    /// X-coordinate for the text label (centered above the arrow).
    pub x_text: f64,
    /// Y-coordinate for the text label (above the arrow tip).
    pub y_text: f64,
    /// Font size for the label text.
    pub font_size: f64,
    /// Stroke width for the arrow curve.
    pub stroke_width: f64,
}

/// Compute bend arrow geometry at a fret position on a string.
///
/// The arrow curves upward from just above the fret number to a tip,
/// with an arrowhead pointing up and text indicating the bend amount.
pub fn layout_tab_bend(
    tab_staff: &TabStaffLayout,
    string: u8,
    x: f64,
    amount: BendAmount,
    stroke_width: f64,
) -> TabBendLayout {
    let ss = tab_staff.staff_space;
    let y_string = tab_staff.string_y(string);

    // Arrow goes upward from below the string to above
    let bend_height = ss * BEND_HEIGHT_RATIO;
    // Start the arrow from slightly above the fret number text area
    let y_base = y_string - ss * 0.45;
    let y_tip = y_base - bend_height;

    // Curve control point: shifted right and partway up for a curved arrow
    let half_width = ss * BEND_HALF_WIDTH_RATIO;
    let x_control = x + half_width;
    let y_control = (y_base + y_tip) / 2.0;

    // Arrowhead at the tip
    let arrow_half_w = ss * ARROWHEAD_HALF_WIDTH_RATIO;
    let arrow_h = ss * ARROWHEAD_HEIGHT_RATIO;

    let font_size = ss * 0.55;

    TabBendLayout {
        amount,
        x,
        y_string,
        y_tip,
        x_control,
        y_control,
        arrow_x_left: x - arrow_half_w,
        arrow_x_right: x + arrow_half_w,
        arrow_y_base: y_tip + arrow_h,
        x_text: x,
        y_text: y_tip - font_size * 0.3,
        font_size,
        stroke_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staff() -> TabStaffLayout {
        TabStaffLayout::new(0.0, 0.0, 5000.0, 250.0, 6)
    }

    #[test]
    fn bend_layout_has_tip_above_string() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            layout.y_tip < layout.y_string,
            "tip ({}) should be above string ({}) in SVG coords",
            layout.y_tip, layout.y_string
        );
    }

    #[test]
    fn arrowhead_base_below_tip() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 3, 500.0, BendAmount::Half, 5.0);
        assert!(
            layout.arrow_y_base > layout.y_tip,
            "arrowhead base ({}) should be below tip ({}) in SVG coords",
            layout.arrow_y_base, layout.y_tip
        );
    }

    #[test]
    fn arrowhead_straddles_x() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(layout.arrow_x_left < 500.0, "left edge should be left of x");
        assert!(layout.arrow_x_right > 500.0, "right edge should be right of x");
    }

    #[test]
    fn text_above_tip() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            layout.y_text < layout.y_tip,
            "text ({}) should be above tip ({}) in SVG coords",
            layout.y_text, layout.y_tip
        );
    }

    #[test]
    fn different_strings_produce_different_y() {
        let staff = test_staff();
        let l1 = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let l5 = layout_tab_bend(&staff, 5, 500.0, BendAmount::Full, 5.0);
        assert!(
            (l1.y_string - l5.y_string).abs() > 100.0,
            "string 1 and 5 should have different y positions"
        );
    }

    #[test]
    fn x_preserved() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 750.0, BendAmount::Half, 5.0);
        assert!((layout.x - 750.0).abs() < 0.001);
        assert!((layout.x_text - 750.0).abs() < 0.001);
    }

    #[test]
    fn stroke_width_preserved() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 7.5);
        assert!((layout.stroke_width - 7.5).abs() < 0.001);
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = TabStaffLayout::new(0.0, 0.0, 5000.0, 100.0, 6);
        let large = TabStaffLayout::new(0.0, 0.0, 5000.0, 400.0, 6);
        let l_small = layout_tab_bend(&small, 1, 500.0, BendAmount::Full, 5.0);
        let l_large = layout_tab_bend(&large, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            l_large.font_size > l_small.font_size,
            "larger staff space should produce larger font size"
        );
    }

    #[test]
    fn amount_preserved() {
        let staff = test_staff();
        let l = layout_tab_bend(&staff, 1, 500.0, BendAmount::Quarter, 5.0);
        assert_eq!(l.amount, BendAmount::Quarter);
    }

    #[test]
    fn control_point_between_base_and_tip() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 3, 500.0, BendAmount::Full, 5.0);
        // y_control should be between y_tip and y_string (in SVG coords, tip < control < string)
        assert!(
            layout.y_control > layout.y_tip && layout.y_control < layout.y_string,
            "control y ({}) should be between tip ({}) and string ({})",
            layout.y_control, layout.y_tip, layout.y_string
        );
    }

    #[test]
    fn bend_labels_correct() {
        assert_eq!(BendAmount::Quarter.label(), "1/4");
        assert_eq!(BendAmount::Half.label(), "1/2");
        assert_eq!(BendAmount::Full.label(), "full");
        assert_eq!(BendAmount::OneAndHalf.label(), "1 1/2");
        assert_eq!(BendAmount::Custom("2").label(), "2");
    }

    #[test]
    fn bend_height_scales_with_staff_space() {
        let small = TabStaffLayout::new(0.0, 0.0, 5000.0, 100.0, 6);
        let large = TabStaffLayout::new(0.0, 0.0, 5000.0, 400.0, 6);
        let l_small = layout_tab_bend(&small, 1, 500.0, BendAmount::Full, 5.0);
        let l_large = layout_tab_bend(&large, 1, 500.0, BendAmount::Full, 5.0);
        let h_small = l_small.y_string - l_small.y_tip;
        let h_large = l_large.y_string - l_large.y_tip;
        assert!(
            h_large > h_small,
            "larger staff space should produce taller bend arrow"
        );
    }
}
