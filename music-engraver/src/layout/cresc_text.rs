/// Crescendo / decrescendo / diminuendo *text* layout with dashed
/// continuation line — the wedgeless ("cresc. - - -") alternative to a
/// hairpin, used for long crescendi or where a wedge would be too crowded.
///
/// Engraving convention (Gould, *Behind Bars*, ch. "Hairpins; cresc.,
/// dim. with dashed lines"):
/// - italic text label at the start ("cresc.", "decresc.", "dim.")
/// - dashed horizontal continuation line to the right, ending where the
///   dynamic change ends
/// - placed below the staff, at the same vertical band as hairpins/dynamics
/// - no end hook (unlike ottava brackets — the dashed line simply stops)
/// - the dashed line is omitted if there isn't room (i.e. the text alone
///   spans the duration). This module emits the line whenever the
///   geometry has space; downstream layers can choose to suppress it.
use crate::layout::staff::StaffLayout;

/// Which kind of dashed-text dynamic this is. Determines the label string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrescTextKind {
    /// `cresc.` (growing louder).
    Crescendo,
    /// `decresc.` (growing quieter — sometimes written `decr.` or `dim.`).
    Decrescendo,
    /// `dim.` (diminuendo — growing quieter, the more common spelling in
    /// modern editions). Conceptually a synonym of `Decrescendo` with a
    /// shorter label.
    Diminuendo,
}

impl CrescTextKind {
    /// The italic text label as it appears below the staff.
    pub fn label(&self) -> &'static str {
        match self {
            CrescTextKind::Crescendo => "cresc.",
            CrescTextKind::Decrescendo => "decresc.",
            CrescTextKind::Diminuendo => "dim.",
        }
    }
}

/// Result of laying out a dashed-text crescendo/diminuendo marking.
#[derive(Debug, Clone)]
pub struct CrescTextLayout {
    /// Which marking this is (drives the label string).
    pub kind: CrescTextKind,
    /// The text label content (a copy of `kind.label()` for convenience).
    pub label: String,
    /// X-coordinate of the start of the marking (where the label's left
    /// edge sits).
    pub x_start: f64,
    /// X-coordinate of the end of the marking (where the dashed line
    /// stops). If `x_end <= x_line_start`, no dashed line is drawn.
    pub x_end: f64,
    /// X-coordinate where the dashed continuation line begins (after the
    /// label and a small padding gap).
    pub x_line_start: f64,
    /// Baseline y-coordinate for the label (and the dashed line — both
    /// sit on the same horizontal baseline so they read as one unit).
    pub y_baseline: f64,
    /// X-anchor for the label text (left edge of the label).
    pub label_x: f64,
    /// Y-anchor for the label text (baseline).
    pub label_y: f64,
    /// Font size for the label, in font design units.
    pub font_size: f64,
    /// Stroke thickness of the dashed continuation line, in font design units.
    pub line_thickness: f64,
    /// Dash length in font design units (SVG `stroke-dasharray` first value).
    pub dash_length: f64,
    /// Gap between dashes in font design units (SVG `stroke-dasharray` second value).
    pub dash_gap: f64,
    /// Whether the renderer should emit the text label.
    ///
    /// `true` for a normal in-system marking laid out via
    /// [`layout_cresc_text`]. `false` for a cross-system continuation
    /// segment laid out via [`layout_cresc_text_continuation`] — the
    /// trailing half (right edge of previous system) and the incoming
    /// half (left edge of next system) both drop the label and render
    /// only the dashed continuation line, mirroring the cross-system
    /// hairpin and trill-extension conventions.
    pub has_label: bool,
}

/// Vertical placement of the dashed-text dynamic, in staff spaces below
/// the bottom staff line. Aligned with hairpins (`HAIRPIN_BELOW_STAFF_SS
/// = 3.5`) so that a section that mixes hairpins and dashed-text dynamics
/// reads as one continuous dynamic axis.
pub const CRESC_TEXT_BELOW_STAFF_SS: f64 = 3.5;

/// Font size for the italic label, in staff spaces. Slightly larger than
/// the engraver's body text (1.4 SS, matching ottava and dynamics
/// expression labels).
pub const CRESC_TEXT_FONT_SIZE_SS: f64 = 1.4;

/// Approximate label width per character, in staff spaces. Used to
/// compute where the dashed line begins after the label.
///
/// Italic serif text averages ~0.55 staff spaces per character at the
/// 1.4-SS body size; rounded to 0.6 to leave a touch of safety margin.
/// Font-agnostic (intentionally — the engraving constants module does
/// not have a measured-text API yet).
pub const CRESC_TEXT_LABEL_WIDTH_PER_CHAR_SS: f64 = 0.6;

/// Padding between the end of the label and the start of the dashed
/// continuation line, in staff spaces.
pub const CRESC_TEXT_LABEL_PADDING_SS: f64 = 0.25;

/// Dashed-line stroke thickness, in staff spaces. Matches `HAIRPIN_*` /
/// ottava conventions — same physical line weight as other "extension"
/// dashes in this engraver so the visual register is uniform.
pub const CRESC_TEXT_LINE_THICKNESS_SS: f64 = 0.12;

/// Dash length in staff spaces. Matches ottava (`OTTAVA_DASH_LENGTH_SS =
/// 0.8`) — longer than hairpin-internal dashes since this dash carries
/// the marking *itself*, not a decoration.
pub const CRESC_TEXT_DASH_LENGTH_SS: f64 = 0.8;

/// Gap between dashes in staff spaces. Matches ottava
/// (`OTTAVA_DASH_GAP_SS = 0.4`).
pub const CRESC_TEXT_DASH_GAP_SS: f64 = 0.4;

/// Lay out a dashed-text crescendo / decrescendo / diminuendo marking
/// below a staff.
///
/// - `x_start` is the x-position where the label's left edge sits
///   (typically aligned with the first affected note).
/// - `x_end` is the x-position where the dashed continuation line stops
///   (typically aligned with the last affected note or just past it).
///   If `x_end <= x_line_start`, the renderer will omit the dashed line.
/// - `staff` provides the vertical reference (the marking sits a fixed
///   distance below the bottom staff line).
/// - `staff_space` is the staff space in font design units.
pub fn layout_cresc_text(
    kind: CrescTextKind,
    x_start: f64,
    x_end: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> CrescTextLayout {
    let label = kind.label().to_string();
    let font_size = CRESC_TEXT_FONT_SIZE_SS * staff_space;
    let line_thickness = CRESC_TEXT_LINE_THICKNESS_SS * staff_space;
    let dash_length = CRESC_TEXT_DASH_LENGTH_SS * staff_space;
    let dash_gap = CRESC_TEXT_DASH_GAP_SS * staff_space;

    let bottom_line_y = staff.y_of(0);
    let y_baseline = bottom_line_y + CRESC_TEXT_BELOW_STAFF_SS * staff_space;

    let label_width_est =
        label.chars().count() as f64 * CRESC_TEXT_LABEL_WIDTH_PER_CHAR_SS * staff_space;
    let padding = CRESC_TEXT_LABEL_PADDING_SS * staff_space;
    let x_line_start = x_start + label_width_est + padding;

    CrescTextLayout {
        kind,
        label,
        x_start,
        x_end,
        x_line_start,
        y_baseline,
        label_x: x_start,
        label_y: y_baseline,
        font_size,
        line_thickness,
        dash_length,
        dash_gap,
        has_label: true,
    }
}

/// Lay out a cross-system *continuation* segment of a dashed-text
/// crescendo / decrescendo / diminuendo marking.
///
/// Two cases use this:
///   1. The trailing half on the previous system — from the original
///      start to the system's right edge — where the label has already
///      appeared earlier on that system. (In practice the label may
///      have appeared at a much earlier x; the cross-system splitter
///      is responsible for choosing the right `x_start`.)
///   2. The incoming half on the next system — from the system's left
///      edge to the dynamic's end — where the label is suppressed
///      because the marking is logically continuing, not starting.
///
/// Both halves drop the label (`has_label = false`) and the dashed
/// continuation line begins immediately at `x_start` (no label-width
/// offset), so the dashed line reads continuously across the break.
///
/// Vertical placement, dash geometry, and line thickness are
/// byte-identical to [`layout_cresc_text`] for the same staff and
/// staff space — a phrase that crosses a system boundary stays on the
/// same horizontal axis.
///
/// `kind` is preserved on the returned layout so callers can still
/// inspect *which* marking this continuation belongs to (useful for
/// debugging and golden tests); the field is just not rendered.
pub fn layout_cresc_text_continuation(
    kind: CrescTextKind,
    x_start: f64,
    x_end: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> CrescTextLayout {
    let label = kind.label().to_string();
    let font_size = CRESC_TEXT_FONT_SIZE_SS * staff_space;
    let line_thickness = CRESC_TEXT_LINE_THICKNESS_SS * staff_space;
    let dash_length = CRESC_TEXT_DASH_LENGTH_SS * staff_space;
    let dash_gap = CRESC_TEXT_DASH_GAP_SS * staff_space;

    let bottom_line_y = staff.y_of(0);
    let y_baseline = bottom_line_y + CRESC_TEXT_BELOW_STAFF_SS * staff_space;

    // No label offset — the dashed line starts immediately at x_start.
    let x_line_start = x_start;

    CrescTextLayout {
        kind,
        label,
        x_start,
        x_end,
        x_line_start,
        y_baseline,
        label_x: x_start,
        label_y: y_baseline,
        font_size,
        line_thickness,
        dash_length,
        dash_gap,
        has_label: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::staff::StaffLayout;

    const SS: f64 = 250.0;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, SS)
    }

    // ---- CrescTextKind::label --------------------------------------------

    #[test]
    fn label_crescendo_is_cresc_dot() {
        assert_eq!(CrescTextKind::Crescendo.label(), "cresc.");
    }

    #[test]
    fn label_decrescendo_is_decresc_dot() {
        assert_eq!(CrescTextKind::Decrescendo.label(), "decresc.");
    }

    #[test]
    fn label_diminuendo_is_dim_dot() {
        assert_eq!(CrescTextKind::Diminuendo.label(), "dim.");
    }

    #[test]
    fn all_three_kinds_have_distinct_labels() {
        let a = CrescTextKind::Crescendo.label();
        let b = CrescTextKind::Decrescendo.label();
        let c = CrescTextKind::Diminuendo.label();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    // ---- layout output fields --------------------------------------------

    #[test]
    fn label_field_matches_kind() {
        let l = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 800.0, &test_staff(), SS);
        assert_eq!(l.label, "cresc.");

        let l = layout_cresc_text(CrescTextKind::Decrescendo, 100.0, 800.0, &test_staff(), SS);
        assert_eq!(l.label, "decresc.");

        let l = layout_cresc_text(CrescTextKind::Diminuendo, 100.0, 800.0, &test_staff(), SS);
        assert_eq!(l.label, "dim.");
    }

    #[test]
    fn x_coordinates_preserved() {
        let l = layout_cresc_text(CrescTextKind::Crescendo, 150.0, 920.0, &test_staff(), SS);
        assert!((l.x_start - 150.0).abs() < f64::EPSILON);
        assert!((l.x_end - 920.0).abs() < f64::EPSILON);
        assert!((l.label_x - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn baseline_is_below_bottom_staff_line() {
        let staff = test_staff();
        let bottom = staff.y_of(0);
        let l = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 800.0, &staff, SS);
        assert!(
            l.y_baseline > bottom,
            "y_baseline {} should be below bottom staff line {}",
            l.y_baseline,
            bottom
        );
        // Locks the exact value to CRESC_TEXT_BELOW_STAFF_SS.
        let expected = bottom + CRESC_TEXT_BELOW_STAFF_SS * SS;
        assert!((l.y_baseline - expected).abs() < 1e-9);
    }

    #[test]
    fn label_and_dashed_line_share_baseline() {
        let l = layout_cresc_text(CrescTextKind::Crescendo, 0.0, 1000.0, &test_staff(), SS);
        assert!((l.label_y - l.y_baseline).abs() < f64::EPSILON);
    }

    #[test]
    fn dashed_line_starts_after_label() {
        let l = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 1500.0, &test_staff(), SS);
        // Label "cresc." has 6 characters.
        let expected_label_width = 6.0 * CRESC_TEXT_LABEL_WIDTH_PER_CHAR_SS * SS;
        let expected_padding = CRESC_TEXT_LABEL_PADDING_SS * SS;
        let expected = 100.0 + expected_label_width + expected_padding;
        assert!(
            (l.x_line_start - expected).abs() < 1e-9,
            "x_line_start={} expected={}",
            l.x_line_start,
            expected
        );
    }

    #[test]
    fn longer_label_pushes_line_start_further_right() {
        let staff = test_staff();
        let cresc = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 1500.0, &staff, SS);
        let decresc = layout_cresc_text(CrescTextKind::Decrescendo, 100.0, 1500.0, &staff, SS);
        let dim = layout_cresc_text(CrescTextKind::Diminuendo, 100.0, 1500.0, &staff, SS);
        // "dim." (4) < "cresc." (6) < "decresc." (8)
        assert!(
            dim.x_line_start < cresc.x_line_start,
            "dim ({}) should start before cresc ({})",
            dim.x_line_start,
            cresc.x_line_start
        );
        assert!(
            cresc.x_line_start < decresc.x_line_start,
            "cresc ({}) should start before decresc ({})",
            cresc.x_line_start,
            decresc.x_line_start
        );
        // The differences should equal exactly the per-char-width times
        // the character-count delta.
        let per_char = CRESC_TEXT_LABEL_WIDTH_PER_CHAR_SS * SS;
        assert!(((cresc.x_line_start - dim.x_line_start) - 2.0 * per_char).abs() < 1e-9);
        assert!(((decresc.x_line_start - cresc.x_line_start) - 2.0 * per_char).abs() < 1e-9);
    }

    // ---- font / line scaling ---------------------------------------------

    #[test]
    fn font_size_matches_const_times_staff_space() {
        let l = layout_cresc_text(CrescTextKind::Crescendo, 0.0, 500.0, &test_staff(), SS);
        let expected = CRESC_TEXT_FONT_SIZE_SS * SS;
        assert!((l.font_size - expected).abs() < 1e-9);
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = layout_cresc_text(
            CrescTextKind::Crescendo,
            0.0,
            500.0,
            &StaffLayout::new(0.0, 0.0, 2000.0, 125.0),
            125.0,
        );
        let large = layout_cresc_text(
            CrescTextKind::Crescendo,
            0.0,
            500.0,
            &StaffLayout::new(0.0, 0.0, 2000.0, 250.0),
            250.0,
        );
        assert!((large.font_size / small.font_size - 2.0).abs() < 1e-9);
    }

    #[test]
    fn dash_constants_match_consts() {
        let l = layout_cresc_text(CrescTextKind::Crescendo, 0.0, 500.0, &test_staff(), SS);
        assert!((l.dash_length - CRESC_TEXT_DASH_LENGTH_SS * SS).abs() < 1e-9);
        assert!((l.dash_gap - CRESC_TEXT_DASH_GAP_SS * SS).abs() < 1e-9);
        assert!((l.line_thickness - CRESC_TEXT_LINE_THICKNESS_SS * SS).abs() < 1e-9);
    }

    #[test]
    fn dash_constants_scale_with_staff_space() {
        let small = layout_cresc_text(
            CrescTextKind::Crescendo,
            0.0,
            500.0,
            &StaffLayout::new(0.0, 0.0, 2000.0, 125.0),
            125.0,
        );
        let large = layout_cresc_text(
            CrescTextKind::Crescendo,
            0.0,
            500.0,
            &StaffLayout::new(0.0, 0.0, 2000.0, 250.0),
            250.0,
        );
        assert!((large.dash_length / small.dash_length - 2.0).abs() < 1e-9);
        assert!((large.dash_gap / small.dash_gap - 2.0).abs() < 1e-9);
        assert!((large.line_thickness / small.line_thickness - 2.0).abs() < 1e-9);
    }

    // ---- baseline placement vs other markings ----------------------------

    #[test]
    fn baseline_aligns_with_hairpin_y_center() {
        // The whole point of using HAIRPIN_BELOW_STAFF_SS is that a
        // mixed-marking phrase (cresc. text then a hairpin) reads as one
        // continuous dynamic axis. Lock the alignment explicitly.
        use crate::layout::hairpin::HAIRPIN_BELOW_STAFF_SS;
        assert!(
            (CRESC_TEXT_BELOW_STAFF_SS - HAIRPIN_BELOW_STAFF_SS).abs() < 1e-12,
            "cresc-text below-staff offset ({}) must match hairpin ({})",
            CRESC_TEXT_BELOW_STAFF_SS,
            HAIRPIN_BELOW_STAFF_SS
        );
    }

    // ---- range / endpoint contract --------------------------------------

    #[test]
    fn empty_range_yields_x_line_start_past_x_end() {
        // If the caller supplies x_end == x_start (degenerate), the
        // dashed line should be a no-op: x_line_start > x_end.
        let l = layout_cresc_text(CrescTextKind::Crescendo, 500.0, 500.0, &test_staff(), SS);
        assert!(l.x_line_start > l.x_end);
    }

    #[test]
    fn negative_x_start_handled() {
        // The layout module is geometry-agnostic; negative coordinates
        // should pass through unchanged (the engraver renders with
        // arbitrary viewBox transforms).
        let l = layout_cresc_text(CrescTextKind::Crescendo, -100.0, 500.0, &test_staff(), SS);
        assert!((l.x_start + 100.0).abs() < f64::EPSILON);
        assert!((l.label_x + 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn x_end_preserved_independently_of_kind() {
        let staff = test_staff();
        let a = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 999.0, &staff, SS);
        let b = layout_cresc_text(CrescTextKind::Decrescendo, 100.0, 999.0, &staff, SS);
        let c = layout_cresc_text(CrescTextKind::Diminuendo, 100.0, 999.0, &staff, SS);
        assert!((a.x_end - 999.0).abs() < f64::EPSILON);
        assert!((b.x_end - 999.0).abs() < f64::EPSILON);
        assert!((c.x_end - 999.0).abs() < f64::EPSILON);
    }

    // ---- has_label field on the plain (non-continuation) layout ----------

    #[test]
    fn plain_layout_has_label_is_true() {
        let staff = test_staff();
        let l = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 1000.0, &staff, SS);
        assert!(
            l.has_label,
            "plain layout_cresc_text must set has_label=true so the renderer emits the label"
        );
    }

    #[test]
    fn plain_layout_has_label_is_true_for_all_kinds() {
        let staff = test_staff();
        for kind in [
            CrescTextKind::Crescendo,
            CrescTextKind::Decrescendo,
            CrescTextKind::Diminuendo,
        ] {
            let l = layout_cresc_text(kind, 100.0, 1000.0, &staff, SS);
            assert!(l.has_label, "has_label should be true for kind {:?}", kind);
        }
    }

    // ---- layout_cresc_text_continuation: cross-system continuation -------

    #[test]
    fn continuation_has_label_is_false() {
        let staff = test_staff();
        let l = layout_cresc_text_continuation(CrescTextKind::Crescendo, 100.0, 1000.0, &staff, SS);
        assert!(
            !l.has_label,
            "continuation layout must set has_label=false to suppress the label across system breaks"
        );
    }

    #[test]
    fn continuation_has_label_is_false_for_all_kinds() {
        let staff = test_staff();
        for kind in [
            CrescTextKind::Crescendo,
            CrescTextKind::Decrescendo,
            CrescTextKind::Diminuendo,
        ] {
            let l = layout_cresc_text_continuation(kind, 100.0, 1000.0, &staff, SS);
            assert!(
                !l.has_label,
                "has_label should be false for kind {:?}",
                kind
            );
        }
    }

    #[test]
    fn continuation_x_line_start_equals_x_start() {
        // Continuation segment: no label, so the dashed line begins
        // immediately at x_start — no label-width offset.
        let staff = test_staff();
        let l = layout_cresc_text_continuation(CrescTextKind::Crescendo, 250.0, 1500.0, &staff, SS);
        assert!(
            (l.x_line_start - l.x_start).abs() < f64::EPSILON,
            "continuation x_line_start={} must equal x_start={}",
            l.x_line_start,
            l.x_start
        );
    }

    #[test]
    fn continuation_x_line_start_strictly_left_of_plain() {
        // A continuation begins its dashed line at x_start; a plain
        // marking begins its dashed line *after* the label region.
        let staff = test_staff();
        let plain = layout_cresc_text(CrescTextKind::Crescendo, 250.0, 1500.0, &staff, SS);
        let cont =
            layout_cresc_text_continuation(CrescTextKind::Crescendo, 250.0, 1500.0, &staff, SS);
        assert!(
            cont.x_line_start < plain.x_line_start,
            "continuation x_line_start ({}) must be left of plain x_line_start ({}) — continuation skips the label region",
            cont.x_line_start,
            plain.x_line_start
        );
        // Exact delta: the label-width estimate + padding.
        let label_chars = CrescTextKind::Crescendo.label().chars().count() as f64;
        let expected_delta = label_chars * CRESC_TEXT_LABEL_WIDTH_PER_CHAR_SS * SS
            + CRESC_TEXT_LABEL_PADDING_SS * SS;
        let actual_delta = plain.x_line_start - cont.x_line_start;
        assert!(
            (actual_delta - expected_delta).abs() < 1e-9,
            "delta between plain and continuation x_line_start ({}) must equal label_width + padding ({})",
            actual_delta,
            expected_delta
        );
    }

    #[test]
    fn continuation_x_coordinates_preserved() {
        let staff = test_staff();
        let l =
            layout_cresc_text_continuation(CrescTextKind::Decrescendo, 175.5, 923.25, &staff, SS);
        assert!((l.x_start - 175.5).abs() < f64::EPSILON);
        assert!((l.x_end - 923.25).abs() < f64::EPSILON);
        assert!((l.label_x - 175.5).abs() < f64::EPSILON);
    }

    #[test]
    fn continuation_baseline_matches_plain_baseline() {
        // The whole point of a cross-system continuation is to keep
        // the marking on the same horizontal axis as the original
        // in-system segment. Lock the vertical equality.
        let staff = test_staff();
        let plain = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 1000.0, &staff, SS);
        let cont =
            layout_cresc_text_continuation(CrescTextKind::Crescendo, 100.0, 1000.0, &staff, SS);
        assert!(
            (plain.y_baseline - cont.y_baseline).abs() < f64::EPSILON,
            "continuation y_baseline ({}) must equal plain y_baseline ({})",
            cont.y_baseline,
            plain.y_baseline
        );
        assert!(
            (plain.label_y - cont.label_y).abs() < f64::EPSILON,
            "continuation label_y must equal plain label_y (same baseline)"
        );
    }

    #[test]
    fn continuation_baseline_is_below_bottom_staff_line() {
        let staff = test_staff();
        let bottom = staff.y_of(0);
        let l = layout_cresc_text_continuation(CrescTextKind::Crescendo, 100.0, 1000.0, &staff, SS);
        let expected = bottom + CRESC_TEXT_BELOW_STAFF_SS * SS;
        assert!(
            (l.y_baseline - expected).abs() < 1e-9,
            "continuation baseline must be exactly CRESC_TEXT_BELOW_STAFF_SS below the bottom line"
        );
    }

    #[test]
    fn continuation_dash_constants_match_plain() {
        // Geometry of the dashed line itself (dash_length, dash_gap,
        // line_thickness) must be identical between the in-system
        // segment and its cross-system continuation, otherwise the
        // reader would see a visual "step" at the system break.
        let staff = test_staff();
        let plain = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 1000.0, &staff, SS);
        let cont =
            layout_cresc_text_continuation(CrescTextKind::Crescendo, 100.0, 1000.0, &staff, SS);
        assert!((plain.dash_length - cont.dash_length).abs() < f64::EPSILON);
        assert!((plain.dash_gap - cont.dash_gap).abs() < f64::EPSILON);
        assert!((plain.line_thickness - cont.line_thickness).abs() < f64::EPSILON);
    }

    #[test]
    fn continuation_dash_constants_scale_with_staff_space() {
        let small = layout_cresc_text_continuation(
            CrescTextKind::Crescendo,
            0.0,
            500.0,
            &StaffLayout::new(0.0, 0.0, 2000.0, 125.0),
            125.0,
        );
        let large = layout_cresc_text_continuation(
            CrescTextKind::Crescendo,
            0.0,
            500.0,
            &StaffLayout::new(0.0, 0.0, 2000.0, 250.0),
            250.0,
        );
        assert!((large.dash_length / small.dash_length - 2.0).abs() < 1e-9);
        assert!((large.dash_gap / small.dash_gap - 2.0).abs() < 1e-9);
        assert!((large.line_thickness / small.line_thickness - 2.0).abs() < 1e-9);
    }

    #[test]
    fn continuation_preserves_kind_field() {
        // Even though the renderer suppresses the label, the kind is
        // still preserved on the layout for golden tests and
        // debugging.
        let staff = test_staff();
        let c = layout_cresc_text_continuation(CrescTextKind::Crescendo, 0.0, 500.0, &staff, SS);
        let d = layout_cresc_text_continuation(CrescTextKind::Decrescendo, 0.0, 500.0, &staff, SS);
        let m = layout_cresc_text_continuation(CrescTextKind::Diminuendo, 0.0, 500.0, &staff, SS);
        assert_eq!(c.kind, CrescTextKind::Crescendo);
        assert_eq!(d.kind, CrescTextKind::Decrescendo);
        assert_eq!(m.kind, CrescTextKind::Diminuendo);
    }

    #[test]
    fn continuation_preserves_label_string_for_debugging() {
        // The label string is kept on the layout (it just doesn't get
        // rendered when has_label == false). Useful for inspection.
        let staff = test_staff();
        let l = layout_cresc_text_continuation(CrescTextKind::Diminuendo, 0.0, 500.0, &staff, SS);
        assert_eq!(l.label, "dim.");
    }

    #[test]
    fn continuation_empty_range_yields_no_dashed_line_region() {
        // Degenerate: x_end == x_start. Since x_line_start == x_start
        // for a continuation, x_line_start == x_end too — the
        // renderer's "no dashed line" check (`x_line_start < x_end`)
        // will suppress the line.
        let staff = test_staff();
        let l = layout_cresc_text_continuation(CrescTextKind::Crescendo, 500.0, 500.0, &staff, SS);
        // Mirrors the renderer's `x_line_start < x_end` check verbatim
        // (negated), so the assertion tracks the real suppression rule.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        let suppressed = !(l.x_line_start < l.x_end);
        assert!(
            suppressed,
            "degenerate range: x_line_start ({}) must not be strictly less than x_end ({})",
            l.x_line_start, l.x_end
        );
    }

    #[test]
    fn continuation_x_end_independent_of_kind() {
        let staff = test_staff();
        let c = layout_cresc_text_continuation(CrescTextKind::Crescendo, 100.0, 999.0, &staff, SS);
        let d =
            layout_cresc_text_continuation(CrescTextKind::Decrescendo, 100.0, 999.0, &staff, SS);
        let m = layout_cresc_text_continuation(CrescTextKind::Diminuendo, 100.0, 999.0, &staff, SS);
        assert!((c.x_end - 999.0).abs() < f64::EPSILON);
        assert!((d.x_end - 999.0).abs() < f64::EPSILON);
        assert!((m.x_end - 999.0).abs() < f64::EPSILON);
    }

    #[test]
    fn continuation_x_line_start_does_not_depend_on_kind() {
        // For a continuation, x_line_start = x_start, which is
        // independent of the kind. A regression that introduced a
        // label-width offset would surface here.
        let staff = test_staff();
        let c = layout_cresc_text_continuation(CrescTextKind::Crescendo, 100.0, 999.0, &staff, SS);
        let d =
            layout_cresc_text_continuation(CrescTextKind::Decrescendo, 100.0, 999.0, &staff, SS);
        let m = layout_cresc_text_continuation(CrescTextKind::Diminuendo, 100.0, 999.0, &staff, SS);
        assert!((c.x_line_start - 100.0).abs() < f64::EPSILON);
        assert!((d.x_line_start - 100.0).abs() < f64::EPSILON);
        assert!((m.x_line_start - 100.0).abs() < f64::EPSILON);
    }
}
