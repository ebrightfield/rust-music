use crate::layout::tab::TabStaffLayout;

/// Layout result for a palm mute ("P.M.") text annotation above a tab fret event.
///
/// Palm muting is a guitar technique where the picking hand rests lightly
/// on the strings near the bridge, producing a muffled, percussive tone.
/// In tablature, it's notated with "P.M." above the affected notes, often
/// with a dashed line extending over the duration of the muted passage.
#[derive(Clone, Debug)]
pub struct TabPalmMuteLayout {
    /// X-coordinate (centered on the fret event).
    pub x: f64,
    /// Y-coordinate (above the staff).
    pub y: f64,
    /// Font size for the "P.M." text.
    pub font_size: f64,
}

/// Layout result for a dashed "P.M." continuation line between two events.
#[derive(Clone, Debug)]
pub struct TabPalmMuteDashLayout {
    /// X-coordinate of the line start (after "P.M." text or previous event).
    pub x_start: f64,
    /// X-coordinate of the line end (at the next event).
    pub x_end: f64,
    /// Y-coordinate of the dashed line.
    pub y: f64,
    /// Stroke width for the dashed line.
    pub stroke_width: f64,
}

/// Distance above the top staff line for palm-mute text, in staff spaces.
///
/// The TAB rhythm beam and tuplet lanes can reach 5.2 staff spaces above the
/// staff. Palm-mute spans sit above both so their dashed line cannot cut
/// through stems, beams, or tuplet brackets.
pub const PALM_MUTE_ABOVE_STAFF_SS: f64 = 6.2;

/// Font size for "P.M." text relative to staff space.
pub const PALM_MUTE_FONT_SIZE_RATIO: f64 = 1.0;

/// Horizontal offset from "P.M." text center to the start of the dashed line,
/// in staff spaces. Accounts for the text width.
pub const PALM_MUTE_DASH_OFFSET_SS: f64 = 1.2;

/// Compute layout for a palm mute text annotation on a tab staff.
///
/// `x` is the horizontal position of the fret event.
pub fn layout_tab_palm_mute(tab_staff: &TabStaffLayout, x: f64) -> TabPalmMuteLayout {
    let y = tab_staff.y_origin - tab_staff.staff_space * PALM_MUTE_ABOVE_STAFF_SS;
    let font_size = tab_staff.staff_space * PALM_MUTE_FONT_SIZE_RATIO;

    TabPalmMuteLayout { x, y, font_size }
}

/// Compute layout for a dashed continuation line between two palm-muted events.
///
/// `x_start` is the x of the first event (dashed line starts after the text).
/// `x_end` is the x of the last event in the muted passage.
/// Returns `None` if the span is too short to draw a visible dashed line.
pub fn layout_tab_palm_mute_dash(
    tab_staff: &TabStaffLayout,
    x_start: f64,
    x_end: f64,
    stroke_width: f64,
) -> Option<TabPalmMuteDashLayout> {
    let text_offset = tab_staff.staff_space * PALM_MUTE_DASH_OFFSET_SS;
    let dash_start = x_start + text_offset;

    // Only draw if there's enough room for a visible dashed line
    if x_end <= dash_start + tab_staff.staff_space * 0.3 {
        return None;
    }

    let y = tab_staff.y_origin - tab_staff.staff_space * PALM_MUTE_ABOVE_STAFF_SS;

    Some(TabPalmMuteDashLayout {
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
    fn palm_mute_x_matches_input() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_palm_mute(&staff, 1234.5);
        assert!((layout.x - 1234.5).abs() < f64::EPSILON);
    }

    #[test]
    fn palm_mute_y_is_above_staff() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_palm_mute(&staff, 500.0);
        assert!(
            layout.y < staff.y_origin,
            "P.M. text should be above the top staff line: y={} < y_origin={}",
            layout.y,
            staff.y_origin
        );
    }

    #[test]
    fn palm_mute_y_offset_scales_with_staff_space() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_palm_mute(&staff, 500.0);
        let expected_y = staff.y_origin - staff.staff_space * PALM_MUTE_ABOVE_STAFF_SS;
        assert!(
            (layout.y - expected_y).abs() < f64::EPSILON,
            "y should be {} but got {}",
            expected_y,
            layout.y
        );
    }

    #[test]
    fn palm_mute_font_size_scales_with_staff_space() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_palm_mute(&staff, 500.0);
        let expected_size = staff.staff_space * PALM_MUTE_FONT_SIZE_RATIO;
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
        let l1 = layout_tab_palm_mute(&staff, 100.0);
        let l2 = layout_tab_palm_mute(&staff, 200.0);
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
        let layout = layout_tab_palm_mute(&staff, 100.0);
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
        let result = layout_tab_palm_mute_dash(&staff, 500.0, 1500.0, stroke);
        assert!(result.is_some(), "wide span should produce a dash layout");
    }

    #[test]
    fn dash_layout_returns_none_for_narrow_span() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        // x_end barely past x_start + text offset
        let text_offset = staff.staff_space * PALM_MUTE_DASH_OFFSET_SS;
        let x_end = 500.0 + text_offset + staff.staff_space * 0.1;
        let result = layout_tab_palm_mute_dash(&staff, 500.0, x_end, stroke);
        assert!(result.is_none(), "narrow span should return None");
    }

    #[test]
    fn dash_x_start_is_offset_from_text_center() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_palm_mute_dash(&staff, 500.0, 1500.0, stroke).unwrap();
        let expected_start = 500.0 + staff.staff_space * PALM_MUTE_DASH_OFFSET_SS;
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
        let dash = layout_tab_palm_mute_dash(&staff, 500.0, 1500.0, stroke).unwrap();
        assert!(
            (dash.x_end - 1500.0).abs() < f64::EPSILON,
            "dash end should be 1500 but got {}",
            dash.x_end
        );
    }

    #[test]
    fn dash_y_matches_text_y() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let text_layout = layout_tab_palm_mute(&staff, 500.0);
        let dash = layout_tab_palm_mute_dash(&staff, 500.0, 1500.0, stroke).unwrap();
        assert!(
            (dash.y - text_layout.y).abs() < f64::EPSILON,
            "dashed line y should match text y: {} vs {}",
            dash.y,
            text_layout.y
        );
    }

    #[test]
    fn dash_stroke_width_preserved() {
        let (staff, config) = guitar_staff();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_palm_mute_dash(&staff, 500.0, 1500.0, stroke).unwrap();
        assert!(
            (dash.stroke_width - stroke).abs() < f64::EPSILON,
            "stroke_width should be preserved"
        );
    }
}
