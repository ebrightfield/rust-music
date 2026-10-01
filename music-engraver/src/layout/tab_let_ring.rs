use crate::layout::tab::TabStaffLayout;

/// Layout result for a "let ring" text annotation above a tab fret event.
///
/// "Let ring" instructs the player to allow the string(s) to sustain freely
/// rather than muting them before the next note. In tablature, it's notated
/// with italic "let ring" above the affected notes, often with a dashed line
/// extending over the duration of the ringing passage.
#[derive(Clone, Debug)]
pub struct TabLetRingLayout {
    /// X-coordinate (centered on the fret event).
    pub x: f64,
    /// Y-coordinate (above the staff).
    pub y: f64,
    /// Font size for the "let ring" text.
    pub font_size: f64,
}

/// Layout result for a dashed "let ring" continuation line between two events.
#[derive(Clone, Debug)]
pub struct TabLetRingDashLayout {
    /// X-coordinate of the line start (after "let ring" text).
    pub x_start: f64,
    /// X-coordinate of the line end (at the last event).
    pub x_end: f64,
    /// Y-coordinate of the dashed line.
    pub y: f64,
    /// Stroke width for the dashed line.
    pub stroke_width: f64,
}

/// Distance above the top staff line for "let ring" text, in staff spaces.
///
/// This lane sits above palm mute and above the TAB rhythm/tuplet lanes, so
/// simultaneous spans remain distinct without crossing rhythmic geometry.
pub const LET_RING_ABOVE_STAFF_SS: f64 = 7.4;

/// Font size for "let ring" text relative to staff space.
pub const LET_RING_FONT_SIZE_RATIO: f64 = 1.0;

/// Horizontal offset from "let ring" text center to the start of the dashed
/// line, in staff spaces. Wider than palm mute because "let ring" text is
/// longer than "P.M.".
pub const LET_RING_DASH_OFFSET_SS: f64 = 2.0;

/// Compute layout for a "let ring" text annotation on a tab staff.
///
/// `x` is the horizontal position of the fret event.
pub fn layout_tab_let_ring(tab_staff: &TabStaffLayout, x: f64) -> TabLetRingLayout {
    let y = tab_staff.y_origin - tab_staff.staff_space * LET_RING_ABOVE_STAFF_SS;
    let font_size = tab_staff.staff_space * LET_RING_FONT_SIZE_RATIO;

    TabLetRingLayout { x, y, font_size }
}

/// Compute layout for a dashed continuation line between "let ring" events.
///
/// `x_start` is the x of the first event (dashed line starts after the text).
/// `x_end` is the x of the last event in the ringing passage.
/// Returns `None` if the span is too short to draw a visible dashed line.
pub fn layout_tab_let_ring_dash(
    tab_staff: &TabStaffLayout,
    x_start: f64,
    x_end: f64,
    stroke_width: f64,
) -> Option<TabLetRingDashLayout> {
    let text_offset = tab_staff.staff_space * LET_RING_DASH_OFFSET_SS;
    let dash_start = x_start + text_offset;

    // Only draw if there's enough room for a visible dashed line
    if x_end <= dash_start + tab_staff.staff_space * 0.3 {
        return None;
    }

    let y = tab_staff.y_origin - tab_staff.staff_space * LET_RING_ABOVE_STAFF_SS;

    Some(TabLetRingDashLayout {
        x_start: dash_start,
        x_end,
        y,
        stroke_width,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::font::EngravingConfig;

    fn guitar_staff() -> (TabStaffLayout, EngravingConfig) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 0.0, 5000.0, &config);
        (staff, config)
    }

    #[test]
    fn let_ring_x_matches_input() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_let_ring(&staff, 1234.5);
        assert!((layout.x - 1234.5).abs() < f64::EPSILON);
    }

    #[test]
    fn let_ring_y_is_above_staff() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_let_ring(&staff, 500.0);
        assert!(
            layout.y < staff.y_origin,
            "let ring text should be above the top staff line: y={} < y_origin={}",
            layout.y,
            staff.y_origin
        );
    }

    #[test]
    fn let_ring_y_offset_scales_with_staff_space() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_let_ring(&staff, 500.0);
        let expected_y = staff.y_origin - staff.staff_space * LET_RING_ABOVE_STAFF_SS;
        assert!(
            (layout.y - expected_y).abs() < f64::EPSILON,
            "y should be {} but got {}",
            expected_y,
            layout.y
        );
    }

    #[test]
    fn let_ring_is_above_palm_mute_position() {
        use crate::layout::tab_palm_mute::layout_tab_palm_mute;
        let (staff, _) = guitar_staff();
        let lr = layout_tab_let_ring(&staff, 500.0);
        let pm = layout_tab_palm_mute(&staff, 500.0);
        assert!(
            lr.y < pm.y,
            "let ring (y={}) should be above palm mute (y={}) since y decreases upward",
            lr.y,
            pm.y
        );
        // Compile-time guarantee: let ring sits above palm mute.
        const _: () = assert!(
            LET_RING_ABOVE_STAFF_SS > crate::layout::tab_palm_mute::PALM_MUTE_ABOVE_STAFF_SS
        );
    }

    #[test]
    fn let_ring_font_size_scales_with_staff_space() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_let_ring(&staff, 500.0);
        let expected_size = staff.staff_space * LET_RING_FONT_SIZE_RATIO;
        assert!(
            (layout.font_size - expected_size).abs() < f64::EPSILON,
            "font_size should be {} but got {}",
            expected_size,
            layout.font_size
        );
    }

    #[test]
    fn different_x_produces_different_layout() {
        let (staff, _) = guitar_staff();
        let l1 = layout_tab_let_ring(&staff, 100.0);
        let l2 = layout_tab_let_ring(&staff, 200.0);
        assert!(
            (l1.x - l2.x).abs() > f64::EPSILON,
            "different x should produce different layout"
        );
    }

    #[test]
    fn nonzero_origin_shifts_y() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 500.0, 5000.0, &config);
        let layout = layout_tab_let_ring(&staff, 100.0);
        assert!(
            layout.y < 500.0,
            "y should be above the shifted origin, got {}",
            layout.y
        );
    }

    #[test]
    fn dash_layout_returns_some_for_wide_span() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let result = layout_tab_let_ring_dash(&staff, 500.0, 2000.0, stroke);
        assert!(result.is_some(), "wide span should produce a dash layout");
    }

    #[test]
    fn dash_layout_returns_none_for_narrow_span() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let text_offset = staff.staff_space * LET_RING_DASH_OFFSET_SS;
        let x_end = 500.0 + text_offset + staff.staff_space * 0.1;
        let result = layout_tab_let_ring_dash(&staff, 500.0, x_end, stroke);
        assert!(result.is_none(), "narrow span should return None");
    }

    #[test]
    fn dash_x_start_is_offset_from_text_center() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_let_ring_dash(&staff, 500.0, 2000.0, stroke).unwrap();
        let expected_start = 500.0 + staff.staff_space * LET_RING_DASH_OFFSET_SS;
        assert!(
            (dash.x_start - expected_start).abs() < f64::EPSILON,
            "dash start should be {} but got {}",
            expected_start,
            dash.x_start
        );
    }

    #[test]
    fn dash_x_end_matches_input() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_let_ring_dash(&staff, 500.0, 2000.0, stroke).unwrap();
        assert!(
            (dash.x_end - 2000.0).abs() < f64::EPSILON,
            "dash end should be 2000 but got {}",
            dash.x_end
        );
    }

    #[test]
    fn dash_y_matches_text_y() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let text_layout = layout_tab_let_ring(&staff, 500.0);
        let dash = layout_tab_let_ring_dash(&staff, 500.0, 2000.0, stroke).unwrap();
        assert!(
            (dash.y - text_layout.y).abs() < f64::EPSILON,
            "dashed line y should match text y: {} vs {}",
            dash.y,
            text_layout.y
        );
    }
}
