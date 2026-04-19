use crate::font::EngravingConfig;
use crate::layout::stem::StemDirection;

/// Tie direction — whether the tie arcs above or below the note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TieDirection {
    Over,
    Under,
}

/// Determines tie direction from stem direction. Convention: tie curves
/// away from the stem — stem up → tie under, stem down → tie over.
pub fn tie_direction_from_stem(stem: StemDirection) -> TieDirection {
    match stem {
        StemDirection::Up => TieDirection::Under,
        StemDirection::Down => TieDirection::Over,
    }
}

/// Vertical offset from the note's y-center to the tie endpoint.
/// Positive = downward in SVG coordinates (y increases downward).
/// "Over" ties start slightly above the notehead; "Under" slightly below.
const TIE_ENDPOINT_OFFSET_SS: f64 = 0.35;

/// The computed geometry for rendering a tie as two cubic Bézier curves
/// (outer and inner) forming a filled crescent shape.
#[derive(Debug, Clone, PartialEq)]
pub struct TieLayout {
    /// Start point (right edge of first notehead)
    pub x_start: f64,
    /// End point (left edge of second notehead)
    pub x_end: f64,
    /// Y-coordinate of tie endpoints
    pub y_endpoint: f64,
    /// Outer Bézier control points: (cp1x, cp1y, cp2x, cp2y)
    pub outer_cp1: (f64, f64),
    pub outer_cp2: (f64, f64),
    /// Inner Bézier control points
    pub inner_cp1: (f64, f64),
    pub inner_cp2: (f64, f64),
    /// Y at midpoint of outer curve (for the crescent apex)
    pub y_outer_apex: f64,
    /// Y at midpoint of inner curve
    pub y_inner_apex: f64,
}

/// Compute tie layout between two note x-positions at a given y-center.
///
/// The tie is modeled as a filled crescent: outer Bézier arc at `midpoint_thickness`
/// distance from the center, inner arc at `endpoint_thickness` distance, creating
/// a shape that tapers from thin endpoints to a thicker middle.
///
/// `note_y` is the y-coordinate of the notehead center (in font design units).
/// `x_start` / `x_end` are the horizontal positions of the tie endpoints.
pub fn layout_tie(
    x_start: f64,
    x_end: f64,
    note_y: f64,
    direction: TieDirection,
    config: &EngravingConfig,
) -> TieLayout {
    let ss = config.staff_space;
    let span = (x_end - x_start).abs();

    // Vertical offset from note center to tie endpoint
    let endpoint_offset = TIE_ENDPOINT_OFFSET_SS * ss;
    let y_endpoint = match direction {
        TieDirection::Over => note_y - endpoint_offset,
        TieDirection::Under => note_y + endpoint_offset,
    };

    // Tie height (arc bulge) scales with span but is clamped.
    // Short ties: ~0.5 staff spaces; long ties: up to ~1.5 staff spaces.
    let height_ss = (span / ss * 0.15).clamp(0.4, 1.5);
    let height = height_ss * ss;

    // Endpoint and midpoint thicknesses from engraving config
    let endpoint_thick = config.to_font_units(config.tie_endpoint_thickness);
    let midpoint_thick = config.to_font_units(config.tie_midpoint_thickness);

    // Direction multiplier: Over = negative y (upward), Under = positive y (downward)
    let dir_sign = match direction {
        TieDirection::Over => -1.0,
        TieDirection::Under => 1.0,
    };

    // Outer curve apex y (the "outside" of the crescent)
    let y_outer_apex = y_endpoint + dir_sign * height;
    // Inner curve apex y (closer to the note)
    let y_inner_apex = y_endpoint + dir_sign * (height - (midpoint_thick - endpoint_thick));

    // Control point x-positions: 1/3 and 2/3 of the span
    let cp_x1 = x_start + span / 3.0;
    let cp_x2 = x_start + span * 2.0 / 3.0;

    // Outer Bézier control points push the curve to the outer apex
    let outer_cp1 = (cp_x1, y_outer_apex);
    let outer_cp2 = (cp_x2, y_outer_apex);

    // Inner Bézier control points are slightly less extreme
    let inner_cp1 = (cp_x2, y_inner_apex);
    let inner_cp2 = (cp_x1, y_inner_apex);

    TieLayout {
        x_start,
        x_end,
        y_endpoint,
        outer_cp1,
        outer_cp2,
        inner_cp1,
        inner_cp2,
        y_outer_apex,
        y_inner_apex,
    }
}

/// Compute a half-tie trailing off the right edge of a system.
///
/// Used when a tied note's target is in the next system. The tie starts at
/// the note's right edge and extends to `x_right_edge` (typically the
/// right end of the staff lines). The curve tapers to nothing at the edge.
pub fn layout_half_tie_right(
    x_start: f64,
    x_right_edge: f64,
    note_y: f64,
    direction: TieDirection,
    config: &EngravingConfig,
) -> TieLayout {
    // Use the full span for control-point spacing but reduce height since
    // the tie is "incomplete" — convention is a shorter, gentler arc.
    layout_tie(x_start, x_right_edge, note_y, direction, config)
}

/// Compute a half-tie leading from the left edge of a system to a note.
///
/// Used when a tied note's source is in the previous system. The tie starts
/// at `x_left_edge` (typically the left end of the note area) and ends at
/// the note's left edge. The curve tapers from nothing at the left edge.
pub fn layout_half_tie_left(
    x_left_edge: f64,
    x_end: f64,
    note_y: f64,
    direction: TieDirection,
    config: &EngravingConfig,
) -> TieLayout {
    layout_tie(x_left_edge, x_end, note_y, direction, config)
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
    fn tie_direction_from_stem_up_is_under() {
        assert_eq!(
            tie_direction_from_stem(StemDirection::Up),
            TieDirection::Under
        );
    }

    #[test]
    fn tie_direction_from_stem_down_is_over() {
        assert_eq!(
            tie_direction_from_stem(StemDirection::Down),
            TieDirection::Over
        );
    }

    #[test]
    fn over_tie_endpoint_above_note() {
        let cfg = test_config();
        let note_y = 500.0;
        let layout = layout_tie(100.0, 400.0, note_y, TieDirection::Over, &cfg);
        // Endpoint should be above (less than) note_y
        assert!(layout.y_endpoint < note_y);
    }

    #[test]
    fn under_tie_endpoint_below_note() {
        let cfg = test_config();
        let note_y = 500.0;
        let layout = layout_tie(100.0, 400.0, note_y, TieDirection::Under, &cfg);
        // Endpoint should be below (greater than) note_y
        assert!(layout.y_endpoint > note_y);
    }

    #[test]
    fn over_tie_apex_above_endpoint() {
        let cfg = test_config();
        let layout = layout_tie(100.0, 400.0, 500.0, TieDirection::Over, &cfg);
        // Outer apex should be above (less y) the endpoint
        assert!(layout.y_outer_apex < layout.y_endpoint);
        // Inner apex should be between endpoint and outer apex
        assert!(layout.y_inner_apex < layout.y_endpoint);
        assert!(layout.y_inner_apex > layout.y_outer_apex);
    }

    #[test]
    fn under_tie_apex_below_endpoint() {
        let cfg = test_config();
        let layout = layout_tie(100.0, 400.0, 500.0, TieDirection::Under, &cfg);
        // Outer apex should be below (greater y) the endpoint
        assert!(layout.y_outer_apex > layout.y_endpoint);
        // Inner apex between endpoint and outer apex
        assert!(layout.y_inner_apex > layout.y_endpoint);
        assert!(layout.y_inner_apex < layout.y_outer_apex);
    }

    #[test]
    fn x_positions_preserved() {
        let cfg = test_config();
        let layout = layout_tie(150.0, 600.0, 500.0, TieDirection::Over, &cfg);
        assert!((layout.x_start - 150.0).abs() < f64::EPSILON);
        assert!((layout.x_end - 600.0).abs() < f64::EPSILON);
    }

    #[test]
    fn control_points_between_endpoints() {
        let cfg = test_config();
        let layout = layout_tie(100.0, 400.0, 500.0, TieDirection::Over, &cfg);
        // Outer cp1 x should be at ~1/3 of span
        let expected_cp1_x = 100.0 + 300.0 / 3.0;
        assert!((layout.outer_cp1.0 - expected_cp1_x).abs() < 1e-6);
        // Outer cp2 x should be at ~2/3 of span
        let expected_cp2_x = 100.0 + 300.0 * 2.0 / 3.0;
        assert!((layout.outer_cp2.0 - expected_cp2_x).abs() < 1e-6);
    }

    #[test]
    fn short_tie_has_smaller_height_than_long() {
        let cfg = test_config();
        // Use spans large enough to exceed the 0.4ss minimum clamp
        // Short: span=900 → 900/250*0.15 = 0.54ss
        // Long: span=2400 → 2400/250*0.15 = 1.44ss
        let short = layout_tie(100.0, 1000.0, 500.0, TieDirection::Over, &cfg);
        let long = layout_tie(100.0, 2500.0, 500.0, TieDirection::Over, &cfg);
        let short_height = (short.y_endpoint - short.y_outer_apex).abs();
        let long_height = (long.y_endpoint - long.y_outer_apex).abs();
        assert!(
            long_height > short_height,
            "long {long_height} should be > short {short_height}"
        );
    }

    #[test]
    fn symmetric_over_under_same_magnitude() {
        let cfg = test_config();
        let note_y = 500.0;
        let over = layout_tie(100.0, 400.0, note_y, TieDirection::Over, &cfg);
        let under = layout_tie(100.0, 400.0, note_y, TieDirection::Under, &cfg);
        // Endpoint offsets should be equal magnitude, opposite direction
        let over_offset = (note_y - over.y_endpoint).abs();
        let under_offset = (under.y_endpoint - note_y).abs();
        assert!((over_offset - under_offset).abs() < 1e-6);
        // Apex heights should be equal magnitude
        let over_height = (over.y_endpoint - over.y_outer_apex).abs();
        let under_height = (under.y_outer_apex - under.y_endpoint).abs();
        assert!((over_height - under_height).abs() < 1e-6);
    }

    #[test]
    fn crescent_thickness_matches_config() {
        let cfg = test_config();
        let layout = layout_tie(100.0, 400.0, 500.0, TieDirection::Over, &cfg);
        // Outer - inner apex difference should relate to midpoint - endpoint thickness
        let midpoint_fu = cfg.to_font_units(cfg.tie_midpoint_thickness);
        let endpoint_fu = cfg.to_font_units(cfg.tie_endpoint_thickness);
        let crescent_diff = (layout.y_outer_apex - layout.y_inner_apex).abs();
        let expected_diff = midpoint_fu - endpoint_fu;
        assert!(
            (crescent_diff - expected_diff).abs() < 1e-6,
            "crescent diff {crescent_diff} != expected {expected_diff}"
        );
    }

    #[test]
    fn very_short_tie_has_minimum_height() {
        let cfg = test_config();
        // Very short span — height should be clamped at 0.4 staff spaces
        let layout = layout_tie(100.0, 120.0, 500.0, TieDirection::Under, &cfg);
        let height = (layout.y_outer_apex - layout.y_endpoint).abs();
        let min_height = 0.4 * cfg.staff_space;
        assert!(
            (height - min_height).abs() < 1e-6,
            "height {height} != min {min_height}"
        );
    }

    // --- half-tie tests ---

    #[test]
    fn half_tie_right_produces_valid_layout() {
        let cfg = test_config();
        let layout = layout_half_tie_right(100.0, 800.0, 500.0, TieDirection::Under, &cfg);
        assert!((layout.x_start - 100.0).abs() < f64::EPSILON);
        assert!((layout.x_end - 800.0).abs() < f64::EPSILON);
        // Should have control points between endpoints
        assert!(layout.outer_cp1.0 > 100.0);
        assert!(layout.outer_cp2.0 < 800.0);
    }

    #[test]
    fn half_tie_left_produces_valid_layout() {
        let cfg = test_config();
        let layout = layout_half_tie_left(50.0, 400.0, 500.0, TieDirection::Over, &cfg);
        assert!((layout.x_start - 50.0).abs() < f64::EPSILON);
        assert!((layout.x_end - 400.0).abs() < f64::EPSILON);
        assert!(layout.outer_cp1.0 > 50.0);
        assert!(layout.outer_cp2.0 < 400.0);
    }

    #[test]
    fn half_tie_right_over_apex_above_endpoint() {
        let cfg = test_config();
        let layout = layout_half_tie_right(100.0, 600.0, 500.0, TieDirection::Over, &cfg);
        assert!(layout.y_outer_apex < layout.y_endpoint);
    }

    #[test]
    fn half_tie_left_under_apex_below_endpoint() {
        let cfg = test_config();
        let layout = layout_half_tie_left(50.0, 400.0, 500.0, TieDirection::Under, &cfg);
        assert!(layout.y_outer_apex > layout.y_endpoint);
    }
}
