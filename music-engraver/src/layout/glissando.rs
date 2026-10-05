use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;

/// Horizontal padding from notehead edge to glissando line endpoint,
/// in staff spaces. Prevents the line from starting/ending inside the
/// notehead glyph.
pub const GLISSANDO_H_PADDING_SS: f64 = 0.4;

/// Vertical offset from the notehead center toward the direction of the
/// glissando, in staff spaces. Moves the endpoint slightly away from center
/// so the line visually connects to the edge of the notehead rather than
/// its center.
pub const GLISSANDO_V_OFFSET_SS: f64 = 0.15;

/// Whether to render "gliss." text label centered along the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlissandoStyle {
    /// Plain diagonal line, no text.
    Line,
    /// Diagonal line with "gliss." text label.
    LineWithText,
    /// Broken diagonal line (LilyPond `dashed-line`).
    Dashed,
    /// Repeated zigzag segments along the diagonal (LilyPond `zigzag`).
    Wavy,
}

/// Computed geometry for a glissando line between two notes.
#[derive(Clone, Debug)]
pub struct GlissandoLayout {
    /// X coordinate of the start of the line (right edge of first notehead + padding).
    pub x_start: f64,
    /// Y coordinate of the start of the line.
    pub y_start: f64,
    /// X coordinate of the end of the line (left edge of second notehead - padding).
    pub x_end: f64,
    /// Y coordinate of the end of the line.
    pub y_end: f64,
    /// Stroke width in font design units.
    pub stroke_width: f64,
    /// Rendering style of the line; retained across system-break fragments.
    pub style: GlissandoStyle,
    /// Whether to show "gliss." text label.
    pub show_text: bool,
    /// Midpoint x for text label placement.
    pub text_x: f64,
    /// Midpoint y for text label placement.
    pub text_y: f64,
    /// Font size for "gliss." text in font design units.
    pub text_font_size: f64,
}

/// Compute a glissando line from one note position to another.
///
/// The line connects the right edge of the source notehead to the left
/// edge of the target notehead, with padding to avoid overlap. Vertical
/// endpoints are offset slightly in the direction of pitch movement so the
/// line appears to emerge from and arrive at the notehead edge.
///
/// Returns `None` if the horizontal span is too small for a visible line
/// (source and target overlap or are adjacent).
pub fn layout_glissando(
    x_source: f64,
    staff_pos_source: i8,
    x_target: f64,
    staff_pos_target: i8,
    staff: &StaffLayout,
    style: GlissandoStyle,
    _stem_direction: Option<StemDirection>,
) -> Option<GlissandoLayout> {
    let ss = staff.staff_space;
    let h_pad = GLISSANDO_H_PADDING_SS * ss;
    // Notehead advance width is approximately 1.18 staff spaces in Bravura.
    // The x_source position is at the notehead's left edge, so we advance
    // past the notehead plus a small padding.
    let notehead_width = ss * 1.18;

    let x_start = x_source + notehead_width + h_pad;
    let x_end = x_target - h_pad;

    // Too close — no room for a visible line
    if x_end - x_start < ss * 0.3 {
        return None;
    }

    let y_source = staff.y_of(staff_pos_source);
    let y_target = staff.y_of(staff_pos_target);
    let v_offset = GLISSANDO_V_OFFSET_SS * ss;

    // Nudge endpoints vertically toward the direction of travel
    let (y_start, y_end) = if y_target < y_source {
        // Ascending (target is higher = lower y in SVG)
        (y_source - v_offset, y_target + v_offset)
    } else if y_target > y_source {
        // Descending
        (y_source + v_offset, y_target - v_offset)
    } else {
        // Same pitch — horizontal glissando (rare but valid)
        (y_source, y_target)
    };

    let stroke_width = ss * 0.08;
    let show_text = style == GlissandoStyle::LineWithText;
    let text_x = (x_start + x_end) / 2.0;
    let text_y = (y_start + y_end) / 2.0;
    let text_font_size = ss * 0.9;

    Some(GlissandoLayout {
        style,
        x_start,
        y_start,
        x_end,
        y_end,
        stroke_width,
        show_text,
        text_x,
        text_y,
        text_font_size,
    })
}

/// Compute a trailing half-glissando from a note to the right edge of a system.
///
/// Used for cross-system glissandos: the note at the end of system N has
/// `glissando_start` but no target within the system. The half-line slopes
/// gently toward the staff center, hinting at a continuation.
pub fn layout_half_glissando_right(
    x_source: f64,
    staff_pos_source: i8,
    x_right_edge: f64,
    staff: &StaffLayout,
    style: GlissandoStyle,
) -> Option<GlissandoLayout> {
    let ss = staff.staff_space;
    let notehead_width = ss * 1.18;
    let h_pad = GLISSANDO_H_PADDING_SS * ss;

    let x_start = x_source + notehead_width + h_pad;
    let x_end = x_right_edge - h_pad * 0.5;

    if x_end - x_start < ss * 0.3 {
        return None;
    }

    let y_source = staff.y_of(staff_pos_source);
    // Slope toward the middle line (staff pos 4) to suggest continuation
    let mid_y = staff.y_of(4);
    let slope_ratio = 0.25;
    let y_end = y_source + (mid_y - y_source) * slope_ratio;

    let stroke_width = ss * 0.08;
    let show_text = style == GlissandoStyle::LineWithText;
    let text_x = (x_start + x_end) / 2.0;
    let text_y = (y_source + y_end) / 2.0;
    let text_font_size = ss * 0.9;

    Some(GlissandoLayout {
        x_start,
        y_start: y_source,
        x_end,
        y_end,
        stroke_width,
        style,
        show_text,
        text_x,
        text_y,
        text_font_size,
    })
}

/// Compute an incoming half-glissando from the left edge of a system to a note.
///
/// Used for cross-system glissandos: the first note in system N+1 receives
/// a continuation line from the left. The line slopes from the staff center
/// toward the target note.
pub fn layout_half_glissando_left(
    x_left_edge: f64,
    x_target: f64,
    staff_pos_target: i8,
    staff: &StaffLayout,
    style: GlissandoStyle,
) -> Option<GlissandoLayout> {
    let ss = staff.staff_space;
    let h_pad = GLISSANDO_H_PADDING_SS * ss;

    let x_start = x_left_edge + h_pad * 0.5;
    let x_end = x_target - h_pad;

    if x_end - x_start < ss * 0.3 {
        return None;
    }

    let y_target = staff.y_of(staff_pos_target);
    // Start from near the middle line, slope toward the target
    let mid_y = staff.y_of(4);
    let slope_ratio = 0.25;
    let y_start = y_target + (mid_y - y_target) * slope_ratio;

    let stroke_width = ss * 0.08;

    Some(GlissandoLayout {
        x_start,
        y_start,
        x_end,
        y_end: y_target,
        style,
        stroke_width,
        show_text: false, // no text label on incoming half
        text_x: 0.0,
        text_y: 0.0,
        text_font_size: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staff() -> StaffLayout {
        // Bravura: 250 font units per staff space (1000 UPM / 4 spaces)
        StaffLayout::new(0.0, 0.0, 5000.0, 250.0)
    }

    // With staff_space=250, notehead width ≈ 295, h_pad = 100.
    // x_start ≈ x_source + 395, so sources and targets must be >500 apart.
    const X1: f64 = 100.0;
    const X2: f64 = 1200.0;

    #[test]
    fn ascending_glissando_has_correct_direction() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 0, X2, 4, &staff, GlissandoStyle::Line, None)
            .expect("should produce layout");
        // Ascending: y_end < y_start (lower y = higher pitch in SVG)
        assert!(
            layout.y_end < layout.y_start,
            "ascending should have y_end < y_start"
        );
    }

    #[test]
    fn descending_glissando_has_correct_direction() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 8, X2, 0, &staff, GlissandoStyle::Line, None)
            .expect("should produce layout");
        // Descending: y_end > y_start
        assert!(
            layout.y_end > layout.y_start,
            "descending should have y_end > y_start"
        );
    }

    #[test]
    fn same_pitch_horizontal() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 4, X2, 4, &staff, GlissandoStyle::Line, None)
            .expect("should produce layout");
        let diff = (layout.y_end - layout.y_start).abs();
        assert!(diff < 0.01, "same pitch should be horizontal");
    }

    #[test]
    fn too_close_returns_none() {
        let staff = test_staff();
        let result = layout_glissando(100.0, 0, 110.0, 4, &staff, GlissandoStyle::Line, None);
        assert!(result.is_none(), "overlapping positions should return None");
    }

    #[test]
    fn x_start_right_of_source() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 0, X2, 4, &staff, GlissandoStyle::Line, None)
            .expect("should produce layout");
        assert!(layout.x_start > X1, "x_start should be right of source x");
    }

    #[test]
    fn x_end_left_of_target() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 0, X2, 4, &staff, GlissandoStyle::Line, None)
            .expect("should produce layout");
        assert!(layout.x_end < X2, "x_end should be left of target x");
    }

    #[test]
    fn line_style_no_text() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 0, X2, 4, &staff, GlissandoStyle::Line, None)
            .expect("should produce layout");
        assert!(!layout.show_text);
    }

    #[test]
    fn line_with_text_style() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 0, X2, 4, &staff, GlissandoStyle::LineWithText, None)
            .expect("should produce layout");
        assert!(layout.show_text);
    }

    #[test]
    fn text_at_midpoint() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 0, X2, 8, &staff, GlissandoStyle::LineWithText, None)
            .expect("should produce layout");
        let expected_mid_x = (layout.x_start + layout.x_end) / 2.0;
        assert!((layout.text_x - expected_mid_x).abs() < 0.01);
    }

    #[test]
    fn stroke_width_positive() {
        let staff = test_staff();
        let layout = layout_glissando(X1, 0, X2, 4, &staff, GlissandoStyle::Line, None)
            .expect("should produce layout");
        assert!(layout.stroke_width > 0.0);
    }

    #[test]
    fn ascending_vs_descending_y_directions_differ() {
        let staff = test_staff();
        let asc = layout_glissando(X1, 0, X2, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let desc = layout_glissando(X1, 8, X2, 0, &staff, GlissandoStyle::Line, None).unwrap();
        // y_start and y_end should be swapped
        assert!(
            (asc.y_start - desc.y_end).abs() < 1.0,
            "endpoint y values should approximately swap"
        );
    }

    // --- half-glissando right (trailing) ---

    #[test]
    fn half_right_produces_layout_for_valid_span() {
        let staff = test_staff();
        let layout = layout_half_glissando_right(100.0, 0, 5000.0, &staff, GlissandoStyle::Line);
        assert!(layout.is_some(), "valid span should produce layout");
    }

    #[test]
    fn half_right_x_start_past_notehead() {
        let staff = test_staff();
        let layout =
            layout_half_glissando_right(100.0, 0, 5000.0, &staff, GlissandoStyle::Line).unwrap();
        assert!(
            layout.x_start > 100.0,
            "x_start should be right of source x"
        );
    }

    #[test]
    fn half_right_x_end_near_staff_edge() {
        let staff = test_staff();
        let layout =
            layout_half_glissando_right(100.0, 0, 5000.0, &staff, GlissandoStyle::Line).unwrap();
        assert!(layout.x_end < 5000.0, "x_end should be left of right edge");
        assert!(layout.x_end > 4900.0, "x_end should be near the right edge");
    }

    #[test]
    fn half_right_slopes_toward_middle() {
        let staff = test_staff();
        let mid_y = staff.y_of(4);
        // Source at bottom line (pos 0), above middle → should slope toward middle
        let layout =
            layout_half_glissando_right(100.0, 0, 5000.0, &staff, GlissandoStyle::Line).unwrap();
        let source_y = staff.y_of(0);
        // y_end should be between source and mid_y
        assert!(
            (layout.y_end - source_y).abs() < (source_y - mid_y).abs(),
            "y_end should be between source and middle line"
        );
    }

    #[test]
    fn half_right_too_close_returns_none() {
        let staff = test_staff();
        // Source close to right edge
        let layout = layout_half_glissando_right(4900.0, 0, 5000.0, &staff, GlissandoStyle::Line);
        assert!(layout.is_none(), "too-close span should return None");
    }

    #[test]
    fn half_right_with_text_shows_text() {
        let staff = test_staff();
        let layout =
            layout_half_glissando_right(100.0, 0, 5000.0, &staff, GlissandoStyle::LineWithText)
                .unwrap();
        assert!(
            layout.show_text,
            "LineWithText should show text on trailing half"
        );
    }

    // --- half-glissando left (incoming) ---

    #[test]
    fn half_left_produces_layout_for_valid_span() {
        let staff = test_staff();
        let layout = layout_half_glissando_left(0.0, 1200.0, 4, &staff, GlissandoStyle::Line);
        assert!(layout.is_some(), "valid span should produce layout");
    }

    #[test]
    fn half_left_x_end_left_of_target() {
        let staff = test_staff();
        let layout =
            layout_half_glissando_left(0.0, 1200.0, 4, &staff, GlissandoStyle::Line).unwrap();
        assert!(layout.x_end < 1200.0, "x_end should be left of target x");
    }

    #[test]
    fn half_left_slopes_toward_target() {
        let staff = test_staff();
        let target_y = staff.y_of(8);
        let mid_y = staff.y_of(4);
        let layout =
            layout_half_glissando_left(0.0, 1200.0, 8, &staff, GlissandoStyle::Line).unwrap();
        // y_start should be between target and mid_y
        assert!(
            (layout.y_start - target_y).abs() < (target_y - mid_y).abs(),
            "y_start should be between target and middle line"
        );
        assert!(
            (layout.y_end - target_y).abs() < 0.01,
            "y_end should match target y"
        );
    }

    #[test]
    fn half_left_no_text() {
        let staff = test_staff();
        let layout =
            layout_half_glissando_left(0.0, 1200.0, 4, &staff, GlissandoStyle::Line).unwrap();
        assert!(!layout.show_text, "incoming half should never show text");
    }

    #[test]
    fn half_left_too_close_returns_none() {
        let staff = test_staff();
        let layout = layout_half_glissando_left(1100.0, 1200.0, 4, &staff, GlissandoStyle::Line);
        assert!(layout.is_none(), "too-close span should return None");
    }
}
