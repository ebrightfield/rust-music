//! Layout geometry for tablature hammer-on and pull-off arcs.
//!
//! A hammer-on (H) is an ascending legato technique where the left hand strikes
//! a higher fret without picking. A pull-off (P) is the descending counterpart.
//! Both are notated as a curved arc above the affected string with "H" or "P"
//! text centered at the apex.

use super::tab::TabStaffLayout;

/// Horizontal padding from fret number center to arc endpoint (staff spaces ratio).
/// Keeps the arc from overlapping the fret number text.
const ARC_H_PADDING_RATIO: f64 = 0.5;

/// Arc height above the string line (staff spaces ratio).
/// Controls how tall the arc rises above the notation.
const ARC_HEIGHT_RATIO: f64 = 0.8;

/// The type of legato technique.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegatoKind {
    /// Hammer-on: ascending to a higher fret. Labeled "H".
    HammerOn,
    /// Pull-off: descending to a lower fret. Labeled "P".
    PullOff,
}

impl LegatoKind {
    /// Single-character label for the technique.
    pub fn label(self) -> &'static str {
        match self {
            LegatoKind::HammerOn => "H",
            LegatoKind::PullOff => "P",
        }
    }
}

/// Layout result for a hammer-on or pull-off arc between two fret positions.
#[derive(Clone, Debug)]
pub struct TabLegatoLayout {
    /// The legato technique type.
    pub kind: LegatoKind,
    /// X-coordinate of the arc start (after the source fret number).
    pub x_start: f64,
    /// X-coordinate of the arc end (before the target fret number).
    pub x_end: f64,
    /// Y-coordinate of the string line (arc endpoints sit here).
    pub y_string: f64,
    /// Y-coordinate of the arc apex (above the string line, lower y value).
    pub y_apex: f64,
    /// X-coordinate of the text label center.
    pub x_text: f64,
    /// Y-coordinate of the text label center (at or near the apex).
    pub y_text: f64,
    /// Font size for the "H"/"P" label.
    pub font_size: f64,
    /// Stroke width for the arc curve.
    pub stroke_width: f64,
}

/// Compute hammer-on/pull-off arc geometry between two fret positions on the same string.
///
/// The arc curves above the string line between source and target x-positions,
/// with the text label centered at the apex.
///
/// Returns `None` if the two x-positions are too close for a visible arc
/// (less than 2× padding apart).
pub fn layout_tab_legato(
    tab_staff: &TabStaffLayout,
    string: u8,
    x_source: f64,
    x_target: f64,
    kind: LegatoKind,
    stroke_width: f64,
) -> Option<TabLegatoLayout> {
    let h_padding = tab_staff.staff_space * ARC_H_PADDING_RATIO;
    let x_start = x_source + h_padding;
    let x_end = x_target - h_padding;

    // Don't draw if endpoints would overlap
    if x_end <= x_start {
        return None;
    }

    let y_string = tab_staff.string_y(string);
    let arc_height = tab_staff.staff_space * ARC_HEIGHT_RATIO;
    // Arc goes upward (negative y direction in SVG)
    let y_apex = y_string - arc_height;
    let x_mid = (x_start + x_end) / 2.0;

    let font_size = tab_staff.staff_space * 0.55;

    Some(TabLegatoLayout {
        kind,
        x_start,
        x_end,
        y_string,
        y_apex,
        x_text: x_mid,
        y_text: y_apex - font_size * 0.15, // slightly above the arc apex
        font_size,
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
    fn legato_layout_returns_some_for_normal_spacing() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0);
        assert!(layout.is_some());
    }

    #[test]
    fn legato_returns_none_when_too_close() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 510.0, LegatoKind::HammerOn, 5.0);
        assert!(layout.is_none());
    }

    #[test]
    fn hammer_on_label_is_h() {
        assert_eq!(LegatoKind::HammerOn.label(), "H");
    }

    #[test]
    fn pull_off_label_is_p() {
        assert_eq!(LegatoKind::PullOff.label(), "P");
    }

    #[test]
    fn arc_x_start_after_source() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        assert!(layout.x_start > 500.0, "arc start should be after source fret x");
    }

    #[test]
    fn arc_x_end_before_target() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::PullOff, 5.0).unwrap();
        assert!(layout.x_end < 1000.0, "arc end should be before target fret x");
    }

    #[test]
    fn arc_apex_above_string_line() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 3, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        assert!(
            layout.y_apex < layout.y_string,
            "arc apex ({}) should be above string line ({}) in SVG coords",
            layout.y_apex, layout.y_string
        );
    }

    #[test]
    fn text_centered_horizontally() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 400.0, 1200.0, LegatoKind::HammerOn, 5.0).unwrap();
        let expected_mid = (layout.x_start + layout.x_end) / 2.0;
        assert!(
            (layout.x_text - expected_mid).abs() < 0.001,
            "text x ({}) should be at midpoint ({})",
            layout.x_text, expected_mid
        );
    }

    #[test]
    fn text_at_or_above_apex() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        assert!(
            layout.y_text <= layout.y_apex,
            "text y ({}) should be at or above arc apex ({})",
            layout.y_text, layout.y_apex
        );
    }

    #[test]
    fn different_strings_produce_different_y() {
        let staff = test_staff();
        let l1 = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let l5 = layout_tab_legato(&staff, 5, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        assert!(
            (l1.y_string - l5.y_string).abs() > 100.0,
            "string 1 and 5 should have different y"
        );
    }

    #[test]
    fn stroke_width_preserved() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 7.5).unwrap();
        assert!((layout.stroke_width - 7.5).abs() < 0.001);
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = TabStaffLayout::new(0.0, 0.0, 5000.0, 100.0, 6);
        let large = TabStaffLayout::new(0.0, 0.0, 5000.0, 400.0, 6);
        let l_small = layout_tab_legato(&small, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let l_large = layout_tab_legato(&large, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        assert!(
            l_large.font_size > l_small.font_size,
            "larger staff space should produce larger font size"
        );
    }

    #[test]
    fn kind_preserved_in_layout() {
        let staff = test_staff();
        let h = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let p = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::PullOff, 5.0).unwrap();
        assert_eq!(h.kind, LegatoKind::HammerOn);
        assert_eq!(p.kind, LegatoKind::PullOff);
    }
}
