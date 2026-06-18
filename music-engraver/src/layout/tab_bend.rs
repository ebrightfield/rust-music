//! Layout geometry for tablature bend notation.
//!
//! Three bend variants are supported:
//!
//! - **Bend** (`layout_tab_bend`): curved arrow upward from the fret number,
//!   with arrowhead and amount label. Indicates string is picked then bent.
//! - **Pre-bend** (`layout_tab_pre_bend`): straight vertical arrow from the
//!   fret number upward. String is bent *before* picking — the player holds
//!   the bend then strikes. Same arrowhead and label as a regular bend, but
//!   the shaft is a straight line, not a curve.
//! - **Release** (`layout_tab_release`): downward-curving arrow from the top
//!   of a prior bend back toward the string line. Indicates pitch returning
//!   to normal after a bend. Arrow points down.

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

/// Layout result for a pre-bend (straight vertical arrow) at a fret position.
#[derive(Clone, Debug)]
pub struct TabPreBendLayout {
    /// The bend amount (determines text label).
    pub amount: BendAmount,
    /// X-coordinate of the arrow (at the fret number).
    pub x: f64,
    /// Y-coordinate of the arrow base (just above the fret number).
    pub y_base: f64,
    /// Y-coordinate of the arrow tip (above the string, lower y in SVG).
    pub y_tip: f64,
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
    /// Stroke width for the arrow line.
    pub stroke_width: f64,
}

/// Compute pre-bend arrow geometry at a fret position on a string.
///
/// A pre-bend is a straight vertical line (not curved) with an upward
/// arrowhead, indicating the string is bent before being picked.
pub fn layout_tab_pre_bend(
    tab_staff: &TabStaffLayout,
    string: u8,
    x: f64,
    amount: BendAmount,
    stroke_width: f64,
) -> TabPreBendLayout {
    let ss = tab_staff.staff_space;
    let y_string = tab_staff.string_y(string);

    let bend_height = ss * BEND_HEIGHT_RATIO;
    let y_base = y_string - ss * 0.45;
    let y_tip = y_base - bend_height;

    let arrow_half_w = ss * ARROWHEAD_HALF_WIDTH_RATIO;
    let arrow_h = ss * ARROWHEAD_HEIGHT_RATIO;
    let font_size = ss * 0.55;

    TabPreBendLayout {
        amount,
        x,
        y_base,
        y_tip,
        arrow_x_left: x - arrow_half_w,
        arrow_x_right: x + arrow_half_w,
        arrow_y_base: y_tip + arrow_h,
        x_text: x,
        y_text: y_tip - font_size * 0.3,
        font_size,
        stroke_width,
    }
}

/// Layout result for a release bend (downward arrow from bent position).
#[derive(Clone, Debug)]
pub struct TabReleaseLayout {
    /// X-coordinate of the arrow start (at the fret number).
    pub x: f64,
    /// Y-coordinate of the arrow start (top — where the bend was held).
    pub y_top: f64,
    /// Y-coordinate of the arrow tip (bottom — released position near string).
    pub y_bottom: f64,
    /// X-coordinate of the curve's control point.
    pub x_control: f64,
    /// Y-coordinate of the curve's control point.
    pub y_control: f64,
    /// Left x of arrowhead base.
    pub arrow_x_left: f64,
    /// Right x of arrowhead base.
    pub arrow_x_right: f64,
    /// Y-coordinate of arrowhead base (slightly above tip, since arrow points down).
    pub arrow_y_base: f64,
    /// Stroke width for the arrow curve.
    pub stroke_width: f64,
}

/// Compute release bend geometry at a fret position on a string.
///
/// A release bend shows pitch returning down after a bend or pre-bend.
/// The arrow curves downward from above the string to near the string,
/// with an arrowhead pointing down.
pub fn layout_tab_release(
    tab_staff: &TabStaffLayout,
    string: u8,
    x: f64,
    stroke_width: f64,
) -> TabReleaseLayout {
    let ss = tab_staff.staff_space;
    let y_string = tab_staff.string_y(string);

    let bend_height = ss * BEND_HEIGHT_RATIO;
    let y_base = y_string - ss * 0.45;
    // Top starts where a bend/pre-bend arrow tip would be
    let y_top = y_base - bend_height;
    // Bottom ends near the string
    let y_bottom = y_base;

    let half_width = ss * BEND_HALF_WIDTH_RATIO;
    let x_control = x + half_width;
    let y_control = (y_top + y_bottom) / 2.0;

    let arrow_half_w = ss * ARROWHEAD_HALF_WIDTH_RATIO;
    let arrow_h = ss * ARROWHEAD_HEIGHT_RATIO;

    TabReleaseLayout {
        x,
        y_top,
        y_bottom,
        x_control,
        y_control,
        arrow_x_left: x - arrow_half_w,
        arrow_x_right: x + arrow_half_w,
        // Arrowhead base is above the tip (arrow points down)
        arrow_y_base: y_bottom - arrow_h,
        stroke_width,
    }
}

/// Layout result for a combined bend-and-release at a fret position.
///
/// A bend-release is the common gesture where the string is picked, bent up
/// to a target pitch, then released back to the original pitch — drawn as a
/// single inverted-V arc (up-curve to an apex, down-curve back to the string)
/// with one amount label at the apex, an upward arrowhead at the apex, and a
/// downward arrowhead where the release lands near the string.
#[derive(Clone, Debug)]
pub struct TabBendReleaseLayout {
    /// The bend amount (determines the apex text label).
    pub amount: BendAmount,
    /// X-coordinate of the gesture origin (at the fret number).
    pub x: f64,
    /// Y-coordinate of the string line (both the up-curve base and the
    /// down-curve landing reference).
    pub y_string: f64,
    /// Y-coordinate of the arc base, just above the fret number text.
    pub y_base: f64,
    /// Y-coordinate of the apex (highest point of the arc, lowest y in SVG).
    pub y_apex: f64,
    /// X-coordinate of the apex (shifted right of `x` so the two halves of the
    /// arc are visually distinct).
    pub x_apex: f64,
    /// X-coordinate of the up-curve control point.
    pub x_control_up: f64,
    /// Y-coordinate of the up-curve control point (between base and apex).
    pub y_control_up: f64,
    /// X-coordinate of the down-curve control point.
    pub x_control_down: f64,
    /// Y-coordinate of the down-curve control point (between apex and base).
    pub y_control_down: f64,
    /// Left x of the up-arrowhead base (at the apex).
    pub up_arrow_x_left: f64,
    /// Right x of the up-arrowhead base (at the apex).
    pub up_arrow_x_right: f64,
    /// Y-coordinate of the up-arrowhead base (below the apex tip).
    pub up_arrow_y_base: f64,
    /// Left x of the down-arrowhead base (at the release landing).
    pub down_arrow_x_left: f64,
    /// Right x of the down-arrowhead base (at the release landing).
    pub down_arrow_x_right: f64,
    /// Y-coordinate of the down-arrowhead base (above the landing tip).
    pub down_arrow_y_base: f64,
    /// X-coordinate for the apex text label.
    pub x_text: f64,
    /// Y-coordinate for the apex text label (above the apex).
    pub y_text: f64,
    /// Font size for the label text.
    pub font_size: f64,
    /// Stroke width for the arc.
    pub stroke_width: f64,
}

/// Compute combined bend-and-release geometry at a fret position on a string.
///
/// The arc rises from just above the fret number to an apex (an upward
/// arrowhead and the amount label sit at the apex), then descends back to the
/// string with a downward arrowhead. The apex shares the regular bend's height
/// so a bend-release reads at the same scale as a plain bend.
pub fn layout_tab_bend_release(
    tab_staff: &TabStaffLayout,
    string: u8,
    x: f64,
    amount: BendAmount,
    stroke_width: f64,
) -> TabBendReleaseLayout {
    let ss = tab_staff.staff_space;
    let y_string = tab_staff.string_y(string);

    // Same vertical extent as a regular bend so the two read at one scale.
    let bend_height = ss * BEND_HEIGHT_RATIO;
    let y_base = y_string - ss * 0.45;
    let y_apex = y_base - bend_height;

    // The apex sits to the right of the origin; the up-curve climbs to it and
    // the down-curve returns from it, giving the inverted-V its width.
    let half_width = ss * BEND_HALF_WIDTH_RATIO;
    let x_apex = x + half_width;

    // Control points bow each half of the arc outward from the apex.
    let x_control_up = x + half_width * 0.5;
    let y_control_up = (y_base + y_apex) / 2.0;
    let x_control_down = x_apex + half_width * 0.5;
    let y_control_down = (y_apex + y_base) / 2.0;

    let arrow_half_w = ss * ARROWHEAD_HALF_WIDTH_RATIO;
    let arrow_h = ss * ARROWHEAD_HEIGHT_RATIO;

    let font_size = ss * 0.55;

    TabBendReleaseLayout {
        amount,
        x,
        y_string,
        y_base,
        y_apex,
        x_apex,
        x_control_up,
        y_control_up,
        x_control_down,
        y_control_down,
        // Up-arrowhead at the apex, pointing up.
        up_arrow_x_left: x_apex - arrow_half_w,
        up_arrow_x_right: x_apex + arrow_half_w,
        up_arrow_y_base: y_apex + arrow_h,
        // Down-arrowhead where the release lands, pointing down.
        down_arrow_x_left: x - arrow_half_w,
        down_arrow_x_right: x + arrow_half_w,
        down_arrow_y_base: y_base - arrow_h,
        x_text: x_apex,
        y_text: y_apex - font_size * 0.3,
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

    // --- Pre-bend layout tests ---

    #[test]
    fn pre_bend_tip_above_base() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            layout.y_tip < layout.y_base,
            "tip ({}) should be above base ({}) in SVG coords",
            layout.y_tip, layout.y_base
        );
    }

    #[test]
    fn pre_bend_arrowhead_between_tip_and_base() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 3, 500.0, BendAmount::Half, 5.0);
        assert!(
            layout.arrow_y_base > layout.y_tip && layout.arrow_y_base < layout.y_base,
            "arrowhead base ({}) should be between tip ({}) and base ({})",
            layout.arrow_y_base, layout.y_tip, layout.y_base
        );
    }

    #[test]
    fn pre_bend_x_preserved() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 750.0, BendAmount::Full, 5.0);
        assert!((layout.x - 750.0).abs() < 0.001);
        assert!((layout.x_text - 750.0).abs() < 0.001);
    }

    #[test]
    fn pre_bend_text_above_tip() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            layout.y_text < layout.y_tip,
            "text ({}) should be above tip ({})",
            layout.y_text, layout.y_tip
        );
    }

    #[test]
    fn pre_bend_tip_matches_regular_bend_tip() {
        let staff = test_staff();
        let pb = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let b = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        // Both use the same BEND_HEIGHT_RATIO and y_base calculation,
        // so their tips should be at the same height
        assert!(
            (pb.y_tip - b.y_tip).abs() < 0.001,
            "tips should match: pre-bend={}, bend={}",
            pb.y_tip, b.y_tip
        );
    }

    #[test]
    fn pre_bend_amount_preserved() {
        let staff = test_staff();
        let l = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Quarter, 5.0);
        assert_eq!(l.amount, BendAmount::Quarter);
    }

    #[test]
    fn pre_bend_arrowhead_straddles_x() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(layout.arrow_x_left < 500.0);
        assert!(layout.arrow_x_right > 500.0);
    }

    // --- Release layout tests ---

    #[test]
    fn release_top_above_bottom() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 5.0);
        assert!(
            layout.y_top < layout.y_bottom,
            "top ({}) should be above bottom ({}) in SVG coords",
            layout.y_top, layout.y_bottom
        );
    }

    #[test]
    fn release_arrowhead_near_bottom() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 3, 500.0, 5.0);
        // Arrow points down, so arrowhead base is above the bottom (tip)
        assert!(
            layout.arrow_y_base < layout.y_bottom,
            "arrowhead base ({}) should be above bottom ({})",
            layout.arrow_y_base, layout.y_bottom
        );
        assert!(
            layout.arrow_y_base > layout.y_top,
            "arrowhead base ({}) should be below top ({})",
            layout.arrow_y_base, layout.y_top
        );
    }

    #[test]
    fn release_control_point_between_top_and_bottom() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 5.0);
        assert!(
            layout.y_control > layout.y_top && layout.y_control < layout.y_bottom,
            "control ({}) should be between top ({}) and bottom ({})",
            layout.y_control, layout.y_top, layout.y_bottom
        );
    }

    #[test]
    fn release_x_preserved() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 750.0, 5.0);
        assert!((layout.x - 750.0).abs() < 0.001);
    }

    #[test]
    fn release_stroke_width_preserved() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 7.5);
        assert!((layout.stroke_width - 7.5).abs() < 0.001);
    }

    #[test]
    fn release_arrowhead_straddles_x() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 5.0);
        assert!(layout.arrow_x_left < 500.0);
        assert!(layout.arrow_x_right > 500.0);
    }

    // --- Bend-release layout tests ---

    #[test]
    fn bend_release_apex_above_base() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            l.y_apex < l.y_base,
            "apex ({}) should be above base ({}) in SVG coords",
            l.y_apex, l.y_base
        );
    }

    #[test]
    fn bend_release_apex_matches_regular_bend_tip() {
        let staff = test_staff();
        let br = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let b = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            (br.y_apex - b.y_tip).abs() < 0.001,
            "bend-release apex ({}) should match bend tip ({}) for one scale",
            br.y_apex, b.y_tip
        );
    }

    #[test]
    fn bend_release_apex_right_of_origin() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            l.x_apex > l.x,
            "apex x ({}) should be right of origin x ({})",
            l.x_apex, l.x
        );
    }

    #[test]
    fn bend_release_up_arrowhead_below_apex() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 3, 500.0, BendAmount::Half, 5.0);
        assert!(
            l.up_arrow_y_base > l.y_apex,
            "up-arrowhead base ({}) should be below apex ({}) in SVG coords",
            l.up_arrow_y_base, l.y_apex
        );
    }

    #[test]
    fn bend_release_down_arrowhead_above_base() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 3, 500.0, BendAmount::Half, 5.0);
        assert!(
            l.down_arrow_y_base < l.y_base,
            "down-arrowhead base ({}) should be above landing base ({}) in SVG coords",
            l.down_arrow_y_base, l.y_base
        );
    }

    #[test]
    fn bend_release_text_above_apex() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            l.y_text < l.y_apex,
            "text ({}) should be above apex ({}) in SVG coords",
            l.y_text, l.y_apex
        );
    }

    #[test]
    fn bend_release_up_control_between_base_and_apex() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        assert!(
            l.y_control_up < l.y_base && l.y_control_up > l.y_apex,
            "up control ({}) should sit between base ({}) and apex ({})",
            l.y_control_up, l.y_base, l.y_apex
        );
    }

    #[test]
    fn bend_release_amount_preserved() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Quarter, 5.0);
        assert_eq!(l.amount, BendAmount::Quarter);
    }

    #[test]
    fn bend_release_stroke_width_preserved() {
        let staff = test_staff();
        let l = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 7.5);
        assert!((l.stroke_width - 7.5).abs() < 0.001);
    }
}
