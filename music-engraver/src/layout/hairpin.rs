/// Hairpin type: crescendo (opening wedge) or decrescendo (closing wedge).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HairpinType {
    /// Opening wedge: starts narrow/closed, ends wide/open. Indicates increasing volume.
    Crescendo,
    /// Closing wedge: starts wide/open, ends narrow/closed. Indicates decreasing volume.
    Decrescendo,
}

/// Layout result for a hairpin (crescendo/decrescendo wedge).
///
/// All coordinates in font design units. The hairpin is drawn as two
/// converging or diverging lines from (x_start, y) to (x_end, y).
#[derive(Clone, Debug)]
pub struct HairpinLayout {
    /// Hairpin type.
    pub kind: HairpinType,
    /// X-coordinate of the left end.
    pub x_start: f64,
    /// X-coordinate of the right end.
    pub x_end: f64,
    /// Y-coordinate of the center line (midpoint between top and bottom lines).
    pub y_center: f64,
    /// Half the opening width at the wide end, in font design units.
    /// The wedge spans from (y_center - half_opening) to (y_center + half_opening).
    pub half_opening: f64,
    /// Stroke width for the hairpin lines.
    pub stroke_width: f64,
}

/// Default vertical distance from bottom staff line to hairpin center, in staff spaces.
/// Placed slightly below dynamics text (which sits at 2.5ss below staff).
pub const HAIRPIN_BELOW_STAFF_SS: f64 = 3.5;

/// Default half-opening of the hairpin at its widest, in staff spaces.
/// A full opening of ~1 staff space is standard for engraved hairpins.
pub const HAIRPIN_HALF_OPENING_SS: f64 = 0.5;

/// Compute the layout for a hairpin (crescendo/decrescendo wedge).
///
/// `x_start` and `x_end` are the horizontal extents of the hairpin.
/// `staff_bottom_y` is the y-coordinate of the bottom staff line.
/// `staff_space` is one staff space in font design units.
/// `stroke_width` is the line thickness (typically from `EngravingConfig::hairpin_thickness`
/// or a fallback like `staff_line_thickness`).
pub fn layout_hairpin(
    kind: HairpinType,
    x_start: f64,
    x_end: f64,
    staff_bottom_y: f64,
    staff_space: f64,
    stroke_width: f64,
) -> HairpinLayout {
    let y_center = staff_bottom_y + HAIRPIN_BELOW_STAFF_SS * staff_space;
    let half_opening = HAIRPIN_HALF_OPENING_SS * staff_space;

    HairpinLayout {
        kind,
        x_start,
        x_end,
        y_center,
        half_opening,
        stroke_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: f64 = 250.0;
    const BOTTOM_Y: f64 = 1000.0;
    const SW: f64 = 10.0;

    fn cresc() -> HairpinLayout {
        layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    fn decresc() -> HairpinLayout {
        layout_hairpin(HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    #[test]
    fn crescendo_kind() {
        assert_eq!(cresc().kind, HairpinType::Crescendo);
    }

    #[test]
    fn decrescendo_kind() {
        assert_eq!(decresc().kind, HairpinType::Decrescendo);
    }

    #[test]
    fn x_positions_preserved() {
        let h = cresc();
        assert!((h.x_start - 100.0).abs() < 1e-6);
        assert!((h.x_end - 600.0).abs() < 1e-6);
    }

    #[test]
    fn y_center_below_staff() {
        let h = cresc();
        let expected = BOTTOM_Y + HAIRPIN_BELOW_STAFF_SS * SS;
        assert!((h.y_center - expected).abs() < 1e-6);
        assert!(h.y_center > BOTTOM_Y);
    }

    #[test]
    fn half_opening_scales_with_staff_space() {
        let small = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 0.0, 200.0, SW);
        let large = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 0.0, 400.0, SW);
        assert!(large.half_opening > small.half_opening);
        assert!((small.half_opening - HAIRPIN_HALF_OPENING_SS * 200.0).abs() < 1e-6);
        assert!((large.half_opening - HAIRPIN_HALF_OPENING_SS * 400.0).abs() < 1e-6);
    }

    #[test]
    fn stroke_width_preserved() {
        let h = cresc();
        assert!((h.stroke_width - SW).abs() < 1e-6);
    }

    #[test]
    fn different_staff_bottom_shifts_y() {
        let h1 = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 500.0, SS, SW);
        let h2 = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 1500.0, SS, SW);
        assert!(h2.y_center > h1.y_center);
        let diff = h2.y_center - h1.y_center;
        assert!((diff - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn zero_width_hairpin() {
        let h = layout_hairpin(HairpinType::Crescendo, 300.0, 300.0, BOTTOM_Y, SS, SW);
        assert!((h.x_start - h.x_end).abs() < 1e-6);
    }

    #[test]
    fn crescendo_and_decrescendo_same_geometry() {
        let c = cresc();
        let d = decresc();
        // Same positions, only kind differs
        assert!((c.y_center - d.y_center).abs() < 1e-6);
        assert!((c.half_opening - d.half_opening).abs() < 1e-6);
        assert!((c.x_start - d.x_start).abs() < 1e-6);
        assert!((c.x_end - d.x_end).abs() < 1e-6);
    }

    #[test]
    fn opening_is_positive() {
        let h = cresc();
        assert!(h.half_opening > 0.0);
    }
}
