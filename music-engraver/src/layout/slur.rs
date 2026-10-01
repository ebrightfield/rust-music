use crate::font::EngravingConfig;
use crate::layout::stem::StemDirection;

/// Slur direction — whether the slur arcs above or below the notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlurDirection {
    Over,
    Under,
}

/// Convention: slur curves away from stems, same as ties.
pub fn slur_direction_from_stem(stem: StemDirection) -> SlurDirection {
    match stem {
        StemDirection::Up => SlurDirection::Under,
        StemDirection::Down => SlurDirection::Over,
    }
}

/// Vertical offset from the attachment point to the slur endpoint.
const SLUR_ENDPOINT_OFFSET_SS: f64 = 0.4;

/// The computed geometry for rendering a slur as two cubic Bézier curves
/// forming a filled crescent, similar to a tie but supporting asymmetric
/// start/end y-positions (different pitches).
#[derive(Debug, Clone, PartialEq)]
pub struct SlurLayout {
    pub x_start: f64,
    pub x_end: f64,
    /// Y of the slur at the start endpoint
    pub y_start: f64,
    /// Y of the slur at the end endpoint
    pub y_end: f64,
    /// Outer Bézier control points
    pub outer_cp1: (f64, f64),
    pub outer_cp2: (f64, f64),
    /// Inner Bézier control points
    pub inner_cp1: (f64, f64),
    pub inner_cp2: (f64, f64),
    /// Y at midpoint of outer curve (apex)
    pub y_outer_apex: f64,
    /// Y at midpoint of inner curve
    pub y_inner_apex: f64,
}

/// Compute slur geometry between two note positions.
///
/// Unlike ties, slurs connect different pitches, so `start_y` and `end_y`
/// may differ. The curve apex is computed relative to the midpoint y of
/// the two endpoints, with height scaling proportional to span.
///
/// `start_y` / `end_y` are the y-coordinates of the notehead or stem-tip
/// attachment points (in font design units). The slur offsets from these
/// based on direction.
pub fn layout_slur(
    x_start: f64,
    x_end: f64,
    start_y: f64,
    end_y: f64,
    direction: SlurDirection,
    config: &EngravingConfig,
) -> SlurLayout {
    let ss = config.staff_space;
    let span = (x_end - x_start).abs();

    let dir_sign = match direction {
        SlurDirection::Over => -1.0,
        SlurDirection::Under => 1.0,
    };

    // Offset endpoints away from the notes
    let offset = SLUR_ENDPOINT_OFFSET_SS * ss;
    let y_start = start_y + dir_sign * offset;
    let y_end = end_y + dir_sign * offset;

    // Reference y for the apex: use the extreme endpoint so the curve
    // always clears both notes. Over → use the higher (smaller y) endpoint;
    // Under → use the lower (larger y) endpoint.
    let y_ref = match direction {
        SlurDirection::Over => y_start.min(y_end),
        SlurDirection::Under => y_start.max(y_end),
    };

    // Slur height scales with span, clamped. Slurs are typically
    // taller than ties since they span phrases rather than just repeats.
    let height_ss = (span / ss * 0.12).clamp(0.5, 2.0);
    let height = height_ss * ss;

    // Crescent thickness from engraving config
    let endpoint_thick = config.to_font_units(config.slur_endpoint_thickness);
    let midpoint_thick = config.to_font_units(config.slur_midpoint_thickness);

    let y_outer_apex = y_ref + dir_sign * height;
    let y_inner_apex = y_ref + dir_sign * (height - (midpoint_thick - endpoint_thick));

    // Control point x-positions at 1/3 and 2/3 of span
    let cp_x1 = x_start + span / 3.0;
    let cp_x2 = x_start + span * 2.0 / 3.0;

    let outer_cp1 = (cp_x1, y_outer_apex);
    let outer_cp2 = (cp_x2, y_outer_apex);
    let inner_cp1 = (cp_x2, y_inner_apex);
    let inner_cp2 = (cp_x1, y_inner_apex);

    SlurLayout {
        x_start,
        x_end,
        y_start,
        y_end,
        outer_cp1,
        outer_cp2,
        inner_cp1,
        inner_cp2,
        y_outer_apex,
        y_inner_apex,
    }
}

/// Compute a half-slur trailing off the right edge of a system.
///
/// Used when a slur's end note is in the next system. The slur starts at the
/// source note and extends to `x_right_edge`. Since we don't know the target
/// note's pitch, both endpoints use the source note's y-position.
pub fn layout_half_slur_right(
    x_start: f64,
    x_right_edge: f64,
    note_y: f64,
    direction: SlurDirection,
    config: &EngravingConfig,
) -> SlurLayout {
    layout_slur(x_start, x_right_edge, note_y, note_y, direction, config)
}

/// Compute a half-slur leading from the left edge of a system to a note.
///
/// Used when a slur's start note is in the previous system. The slur starts
/// at `x_left_edge` and ends at the target note. Since we don't know the
/// source note's pitch, both endpoints use the target note's y-position.
pub fn layout_half_slur_left(
    x_left_edge: f64,
    x_end: f64,
    note_y: f64,
    direction: SlurDirection,
    config: &EngravingConfig,
) -> SlurLayout {
    layout_slur(x_left_edge, x_end, note_y, note_y, direction, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> EngravingConfig {
        let metadata: smufl::Metadata =
            serde_json::from_slice(crate::font::BRAVURA_METADATA).unwrap();
        EngravingConfig::from_smufl(&metadata.engraving_defaults, 1000)
    }

    #[test]
    fn direction_from_stem_up_is_under() {
        assert_eq!(
            slur_direction_from_stem(StemDirection::Up),
            SlurDirection::Under
        );
    }

    #[test]
    fn direction_from_stem_down_is_over() {
        assert_eq!(
            slur_direction_from_stem(StemDirection::Down),
            SlurDirection::Over
        );
    }

    #[test]
    fn over_slur_start_above_note() {
        let cfg = test_config();
        let layout = layout_slur(100.0, 500.0, 400.0, 400.0, SlurDirection::Over, &cfg);
        assert!(
            layout.y_start < 400.0,
            "Over slur start should be above note y"
        );
    }

    #[test]
    fn under_slur_start_below_note() {
        let cfg = test_config();
        let layout = layout_slur(100.0, 500.0, 400.0, 400.0, SlurDirection::Under, &cfg);
        assert!(
            layout.y_start > 400.0,
            "Under slur start should be below note y"
        );
    }

    #[test]
    fn over_slur_apex_above_both_endpoints() {
        let cfg = test_config();
        let layout = layout_slur(100.0, 500.0, 400.0, 400.0, SlurDirection::Over, &cfg);
        assert!(
            layout.y_outer_apex < layout.y_start,
            "Over apex should be above start"
        );
        assert!(
            layout.y_outer_apex < layout.y_end,
            "Over apex should be above end"
        );
        assert!(
            layout.y_inner_apex > layout.y_outer_apex,
            "Inner apex closer to notes than outer"
        );
    }

    #[test]
    fn under_slur_apex_below_both_endpoints() {
        let cfg = test_config();
        let layout = layout_slur(100.0, 500.0, 400.0, 400.0, SlurDirection::Under, &cfg);
        assert!(
            layout.y_outer_apex > layout.y_start,
            "Under apex should be below start"
        );
        assert!(
            layout.y_outer_apex > layout.y_end,
            "Under apex should be below end"
        );
        assert!(layout.y_inner_apex < layout.y_outer_apex);
    }

    #[test]
    fn asymmetric_endpoints_different_y() {
        let cfg = test_config();
        let layout = layout_slur(100.0, 500.0, 300.0, 500.0, SlurDirection::Under, &cfg);
        // Start and end y should differ (offset from different base ys)
        assert!(
            (layout.y_start - layout.y_end).abs() > 1.0,
            "Asymmetric note ys should produce different endpoint ys"
        );
    }

    #[test]
    fn x_positions_preserved() {
        let cfg = test_config();
        let layout = layout_slur(150.0, 700.0, 400.0, 400.0, SlurDirection::Over, &cfg);
        assert!((layout.x_start - 150.0).abs() < f64::EPSILON);
        assert!((layout.x_end - 700.0).abs() < f64::EPSILON);
    }

    #[test]
    fn control_points_at_third_positions() {
        let cfg = test_config();
        let layout = layout_slur(100.0, 400.0, 500.0, 500.0, SlurDirection::Over, &cfg);
        let expected_cp1_x = 100.0 + 300.0 / 3.0;
        let expected_cp2_x = 100.0 + 300.0 * 2.0 / 3.0;
        assert!((layout.outer_cp1.0 - expected_cp1_x).abs() < 1e-6);
        assert!((layout.outer_cp2.0 - expected_cp2_x).abs() < 1e-6);
    }

    #[test]
    fn short_slur_has_minimum_height() {
        let cfg = test_config();
        // Symmetric endpoints: y_ref = max(y_start, y_end) for Under
        let layout = layout_slur(100.0, 120.0, 500.0, 500.0, SlurDirection::Under, &cfg);
        let y_ref = layout.y_start.max(layout.y_end);
        let height = (layout.y_outer_apex - y_ref).abs();
        let min_height = 0.5 * cfg.staff_space;
        assert!(
            (height - min_height).abs() < 1e-6,
            "height {height} != min {min_height}"
        );
    }

    #[test]
    fn long_slur_has_larger_height_than_short() {
        let cfg = test_config();
        let short = layout_slur(100.0, 1000.0, 500.0, 500.0, SlurDirection::Over, &cfg);
        let long = layout_slur(100.0, 2500.0, 500.0, 500.0, SlurDirection::Over, &cfg);
        // For symmetric endpoints with Over, y_ref = min(y_start, y_end)
        let short_ref = short.y_start.min(short.y_end);
        let long_ref = long.y_start.min(long.y_end);
        let short_height = (short.y_outer_apex - short_ref).abs();
        let long_height = (long.y_outer_apex - long_ref).abs();
        assert!(long_height > short_height);
    }

    #[test]
    fn symmetric_endpoints_produce_symmetric_offsets() {
        let cfg = test_config();
        let over = layout_slur(100.0, 500.0, 400.0, 400.0, SlurDirection::Over, &cfg);
        let under = layout_slur(100.0, 500.0, 400.0, 400.0, SlurDirection::Under, &cfg);
        let over_offset = (400.0 - over.y_start).abs();
        let under_offset = (under.y_start - 400.0).abs();
        assert!((over_offset - under_offset).abs() < 1e-6);
    }

    #[test]
    fn crescent_thickness_from_config() {
        let cfg = test_config();
        let layout = layout_slur(100.0, 500.0, 400.0, 400.0, SlurDirection::Over, &cfg);
        let midpoint_fu = cfg.to_font_units(cfg.slur_midpoint_thickness);
        let endpoint_fu = cfg.to_font_units(cfg.slur_endpoint_thickness);
        let crescent_diff = (layout.y_outer_apex - layout.y_inner_apex).abs();
        let expected_diff = midpoint_fu - endpoint_fu;
        assert!(
            (crescent_diff - expected_diff).abs() < 1e-6,
            "crescent diff {crescent_diff} != expected {expected_diff}"
        );
    }

    #[test]
    fn ascending_slur_apex_between_endpoints() {
        let cfg = test_config();
        // Start note high (y=300), end note low (y=600) — ascending interval in
        // SVG coords (lower pitch = higher y)
        let layout = layout_slur(100.0, 500.0, 300.0, 600.0, SlurDirection::Over, &cfg);
        // Apex y should be above (less than) both endpoints for an Over slur
        assert!(layout.y_outer_apex < layout.y_start);
        assert!(layout.y_outer_apex < layout.y_end);
    }

    #[test]
    fn descending_slur_under_apex_below_both() {
        let cfg = test_config();
        // High note to low note in pitch, meaning low y to high y
        let layout = layout_slur(100.0, 500.0, 300.0, 600.0, SlurDirection::Under, &cfg);
        // Under apex should be below (greater y than) both endpoints
        assert!(layout.y_outer_apex > layout.y_start);
        assert!(layout.y_outer_apex > layout.y_end);
    }

    // --- half-slur tests ---

    #[test]
    fn half_slur_right_produces_valid_layout() {
        let cfg = test_config();
        let layout = layout_half_slur_right(100.0, 800.0, 500.0, SlurDirection::Under, &cfg);
        assert!((layout.x_start - 100.0).abs() < f64::EPSILON);
        assert!((layout.x_end - 800.0).abs() < f64::EPSILON);
        assert!(layout.outer_cp1.0 > 100.0);
        assert!(layout.outer_cp2.0 < 800.0);
        // Under: apex below endpoint
        assert!(layout.y_outer_apex > layout.y_start);
    }

    #[test]
    fn half_slur_left_produces_valid_layout() {
        let cfg = test_config();
        let layout = layout_half_slur_left(50.0, 400.0, 500.0, SlurDirection::Over, &cfg);
        assert!((layout.x_start - 50.0).abs() < f64::EPSILON);
        assert!((layout.x_end - 400.0).abs() < f64::EPSILON);
        assert!(layout.outer_cp1.0 > 50.0);
        assert!(layout.outer_cp2.0 < 400.0);
        // Over: apex above endpoint
        assert!(layout.y_outer_apex < layout.y_start);
    }

    #[test]
    fn half_slur_right_symmetric_endpoints() {
        let cfg = test_config();
        let layout = layout_half_slur_right(100.0, 600.0, 400.0, SlurDirection::Over, &cfg);
        // Both y_start and y_end derive from the same note_y, so they should be equal
        assert!(
            (layout.y_start - layout.y_end).abs() < 1e-6,
            "half-slur right should have symmetric y endpoints"
        );
    }

    #[test]
    fn half_slur_left_symmetric_endpoints() {
        let cfg = test_config();
        let layout = layout_half_slur_left(50.0, 400.0, 500.0, SlurDirection::Under, &cfg);
        assert!(
            (layout.y_start - layout.y_end).abs() < 1e-6,
            "half-slur left should have symmetric y endpoints"
        );
    }
}
