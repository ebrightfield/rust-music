use smufl::Glyph;

use crate::font::EngravingConfig;
use crate::layout::slur::{layout_slur, slur_direction_from_stem, SlurLayout};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;

/// Grace note type.
///
/// Acciaccatura (slashed) is a very short ornamental note, notated with a
/// slash through the stem. Appoggiatura (unslashed) takes rhythmic value
/// from the principal note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GraceNoteKind {
    /// Slashed grace note — acciaccatura. Conventional very short ornament.
    Acciaccatura,
    /// Unslashed grace note — appoggiatura. Takes rhythmic value from principal.
    Appoggiatura,
}

/// Scale factor for grace note glyphs relative to normal noteheads.
///
/// SMuFL grace note glyphs are already scaled in the font, but when we
/// draw them with a scale transform we use this factor. The conventional
/// size is roughly 60–66% of normal. We use the font's own glyph unscaled
/// and only apply this when computing spacing.
pub const GRACE_NOTE_SCALE: f64 = 0.6;

/// Horizontal spacing from the grace note to the principal note, in staff spaces.
pub const GRACE_NOTE_SPACING_SS: f64 = 0.5;

/// Return the SMuFL glyph for a grace note of the given kind and stem direction.
///
/// SMuFL provides composite glyphs that include the notehead, stem, flag,
/// and (for acciaccatura) the slash — all in a single glyph.
pub fn grace_note_glyph(kind: GraceNoteKind, stem_dir: StemDirection) -> Glyph {
    match (kind, stem_dir) {
        (GraceNoteKind::Acciaccatura, StemDirection::Up) => Glyph::GraceNoteAcciaccaturaStemUp,
        (GraceNoteKind::Acciaccatura, StemDirection::Down) => Glyph::GraceNoteAcciaccaturaStemDown,
        (GraceNoteKind::Appoggiatura, StemDirection::Up) => Glyph::GraceNoteAppoggiaturaStemUp,
        (GraceNoteKind::Appoggiatura, StemDirection::Down) => Glyph::GraceNoteAppoggiaturaStemDown,
    }
}

/// Computed position and glyph for a grace note.
#[derive(Clone, Debug)]
pub struct GraceNoteLayout {
    /// X-coordinate for the grace note glyph origin.
    pub x: f64,
    /// Y-coordinate for the grace note glyph origin (staff position mapped to y).
    pub y: f64,
    /// The SMuFL composite glyph to render.
    pub glyph: Glyph,
    /// Scale factor to apply when rendering.
    pub scale: f64,
    /// Staff position (for ledger line computation, if needed).
    pub staff_position: i8,
}

/// Compute the layout for a grace note placed before a principal note.
///
/// `principal_x` is the x-coordinate of the principal note's notehead.
/// `grace_staff_position` is the staff position of the grace note.
/// `kind` is acciaccatura or appoggiatura.
/// `stem_dir` is the stem direction for the grace note.
/// `staff` provides coordinate mapping.
///
/// The grace note is placed to the left of the principal note with a gap
/// of `GRACE_NOTE_SPACING_SS` staff spaces.
pub fn layout_grace_note(
    principal_x: f64,
    grace_staff_position: i8,
    kind: GraceNoteKind,
    stem_dir: StemDirection,
    staff: &StaffLayout,
) -> GraceNoteLayout {
    let glyph = grace_note_glyph(kind, stem_dir);
    let y = staff.y_of(grace_staff_position);
    let spacing = GRACE_NOTE_SPACING_SS * staff.staff_space;

    // Grace note placed to the left of principal. Approximate glyph width
    // at the scaled size — Bravura's grace note glyphs are roughly 1 staff
    // space wide at full size; at GRACE_NOTE_SCALE they are ~0.6 ss.
    let approx_width = staff.staff_space * GRACE_NOTE_SCALE;
    let x = principal_x - spacing - approx_width;

    GraceNoteLayout {
        x,
        y,
        glyph,
        scale: GRACE_NOTE_SCALE,
        staff_position: grace_staff_position,
    }
}

/// Compute how much extra horizontal space a grace note consumes to the left
/// of the principal note, in font design units. This is used by measure layout
/// to shift the principal note rightward.
pub fn grace_note_x_reservation(staff: &StaffLayout) -> f64 {
    let approx_width = staff.staff_space * GRACE_NOTE_SCALE;
    let spacing = GRACE_NOTE_SPACING_SS * staff.staff_space;
    approx_width + spacing
}

/// Compute the connecting slur from a grace note to its principal note.
///
/// The slur direction follows the principal note's stem (slurs curve away
/// from stems — stem-up → under, stem-down → over). This matches the
/// standard "slur opposite stems" rule applied to a one-note span.
///
/// Endpoints attach near the noteheads:
/// - `x_start` is just to the right of the grace notehead.
/// - `x_end` is just to the left of the principal notehead.
/// - `y_start` is the grace notehead y; `y_end` is the principal notehead y.
///
/// `layout_slur` applies the conventional [`SLUR_ENDPOINT_OFFSET_SS`] offset
/// away from the noteheads on top of these values.
///
/// Returns `None` when the horizontal distance is degenerate (e.g. the grace
/// note has been placed at or past the principal x) — callers should skip
/// rendering rather than emit a zero-width slur.
pub fn layout_grace_note_slur(
    grace: &GraceNoteLayout,
    principal_x: f64,
    principal_staff_position: i8,
    principal_stem_dir: StemDirection,
    staff: &StaffLayout,
    config: &EngravingConfig,
) -> Option<SlurLayout> {
    // Right edge of the scaled grace notehead. A standard Bravura notehead
    // has advance ~1.18 staff spaces; half-width ~0.59 ss. The grace glyph is
    // drawn at GRACE_NOTE_SCALE, so the scaled half-width is ~0.35 ss.
    let grace_half = 0.59 * staff.staff_space * GRACE_NOTE_SCALE;
    let x_start = grace.x + grace_half;
    let x_end = principal_x - 0.05 * staff.staff_space;

    // Degenerate / inverted span — caller should skip.
    if x_end - x_start < 0.1 * staff.staff_space {
        return None;
    }

    let y_start = grace.y;
    let y_end = staff.y_of(principal_staff_position);
    let direction = slur_direction_from_stem(principal_stem_dir);

    Some(layout_slur(
        x_start, x_end, y_start, y_end, direction, config,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn test_staff() -> StaffLayout {
        let font = bravura_font();
        let config = font.engraving_config();
        StaffLayout::from_config(0.0, 0.0, 5000.0, &config)
    }

    #[test]
    fn acciaccatura_up_glyph() {
        assert_eq!(
            grace_note_glyph(GraceNoteKind::Acciaccatura, StemDirection::Up),
            Glyph::GraceNoteAcciaccaturaStemUp
        );
    }

    #[test]
    fn acciaccatura_down_glyph() {
        assert_eq!(
            grace_note_glyph(GraceNoteKind::Acciaccatura, StemDirection::Down),
            Glyph::GraceNoteAcciaccaturaStemDown
        );
    }

    #[test]
    fn appoggiatura_up_glyph() {
        assert_eq!(
            grace_note_glyph(GraceNoteKind::Appoggiatura, StemDirection::Up),
            Glyph::GraceNoteAppoggiaturaStemUp
        );
    }

    #[test]
    fn appoggiatura_down_glyph() {
        assert_eq!(
            grace_note_glyph(GraceNoteKind::Appoggiatura, StemDirection::Down),
            Glyph::GraceNoteAppoggiaturaStemDown
        );
    }

    #[test]
    fn all_four_glyphs_are_distinct() {
        let glyphs = [
            grace_note_glyph(GraceNoteKind::Acciaccatura, StemDirection::Up),
            grace_note_glyph(GraceNoteKind::Acciaccatura, StemDirection::Down),
            grace_note_glyph(GraceNoteKind::Appoggiatura, StemDirection::Up),
            grace_note_glyph(GraceNoteKind::Appoggiatura, StemDirection::Down),
        ];
        for i in 0..glyphs.len() {
            for j in (i + 1)..glyphs.len() {
                assert_ne!(glyphs[i], glyphs[j], "glyphs {i} and {j} should differ");
            }
        }
    }

    #[test]
    fn layout_grace_note_x_left_of_principal() {
        let staff = test_staff();
        let layout = layout_grace_note(
            1000.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        assert!(
            layout.x < 1000.0,
            "grace note should be left of principal: {} < 1000",
            layout.x
        );
    }

    #[test]
    fn layout_grace_note_y_matches_staff_position() {
        let staff = test_staff();
        let layout = layout_grace_note(
            1000.0,
            6,
            GraceNoteKind::Appoggiatura,
            StemDirection::Down,
            &staff,
        );
        let expected_y = staff.y_of(6);
        assert!(
            (layout.y - expected_y).abs() < 0.01,
            "y should match staff position 6: {} vs {}",
            layout.y,
            expected_y
        );
    }

    #[test]
    fn layout_grace_note_scale_is_grace_scale() {
        let staff = test_staff();
        let layout = layout_grace_note(
            1000.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        assert!(
            (layout.scale - GRACE_NOTE_SCALE).abs() < f64::EPSILON,
            "scale should be GRACE_NOTE_SCALE"
        );
    }

    #[test]
    fn layout_grace_note_preserves_staff_position() {
        let staff = test_staff();
        let layout = layout_grace_note(
            1000.0,
            -2,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(layout.staff_position, -2);
    }

    #[test]
    fn acciaccatura_and_appoggiatura_same_x_for_same_input() {
        let staff = test_staff();
        let acc = layout_grace_note(
            1000.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let app = layout_grace_note(
            1000.0,
            4,
            GraceNoteKind::Appoggiatura,
            StemDirection::Up,
            &staff,
        );
        assert!(
            (acc.x - app.x).abs() < 0.01,
            "both kinds should have same x positioning: {} vs {}",
            acc.x,
            app.x
        );
    }

    #[test]
    fn x_reservation_is_positive() {
        let staff = test_staff();
        let reservation = grace_note_x_reservation(&staff);
        assert!(
            reservation > 0.0,
            "reservation should be positive: {}",
            reservation
        );
    }

    #[test]
    fn x_reservation_matches_layout_offset() {
        let staff = test_staff();
        let reservation = grace_note_x_reservation(&staff);
        let layout = layout_grace_note(
            1000.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        // The principal is at 1000; the grace note x + reservation should
        // approximately equal 1000 (the grace note is at principal - reservation).
        let reconstructed = layout.x + reservation;
        assert!(
            (reconstructed - 1000.0).abs() < 1.0,
            "x + reservation should ≈ principal_x: {} + {} = {} vs 1000",
            layout.x,
            reservation,
            reconstructed
        );
    }

    #[test]
    fn ledger_line_position_preserved() {
        let staff = test_staff();
        // Grace note below staff (needs ledger lines)
        let layout = layout_grace_note(
            1000.0,
            -2,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(layout.staff_position, -2);
        assert!(
            layout.y > staff.y_of(0),
            "below-staff note y should be below bottom line"
        );
    }

    #[test]
    fn different_principal_x_shifts_grace_note() {
        let staff = test_staff();
        let layout1 = layout_grace_note(
            1000.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let layout2 = layout_grace_note(
            2000.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        assert!(
            (layout2.x - layout1.x - 1000.0).abs() < 0.01,
            "1000 shift in principal should shift grace by 1000: {} vs {}",
            layout1.x,
            layout2.x
        );
    }

    fn test_config() -> crate::font::EngravingConfig {
        let metadata: smufl::Metadata =
            serde_json::from_slice(crate::font::BRAVURA_METADATA).unwrap();
        crate::font::EngravingConfig::from_smufl(&metadata.engraving_defaults, 1000)
    }

    #[test]
    fn slur_spans_grace_to_principal_horizontally() {
        let staff = test_staff();
        let cfg = test_config();
        let principal_x = 1000.0;
        let grace = layout_grace_note(
            principal_x,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let slur = layout_grace_note_slur(&grace, principal_x, 4, StemDirection::Up, &staff, &cfg)
            .expect("non-degenerate slur");

        // Slur start should be to the right of the grace glyph origin.
        assert!(
            slur.x_start > grace.x,
            "slur x_start {} should be right of grace origin {}",
            slur.x_start,
            grace.x
        );
        // Slur end should be to the left of the principal x.
        assert!(
            slur.x_end < principal_x,
            "slur x_end {} should be left of principal {}",
            slur.x_end,
            principal_x
        );
        // Endpoints must form a positive span.
        assert!(slur.x_end > slur.x_start, "x_end must exceed x_start");
    }

    #[test]
    fn slur_under_for_stem_up_principal() {
        // Stem-up principal → slur arcs *under* the notes (away from stems).
        // y_outer_apex must be *below* the endpoint y in SVG coords (greater y).
        let staff = test_staff();
        let cfg = test_config();
        let principal_x = 1000.0;
        let grace_pos = 4;
        let principal_pos = 4;
        let grace = layout_grace_note(
            principal_x,
            grace_pos,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let slur = layout_grace_note_slur(
            &grace,
            principal_x,
            principal_pos,
            StemDirection::Up,
            &staff,
            &cfg,
        )
        .expect("non-degenerate slur");

        let endpoint_y = staff.y_of(principal_pos);
        assert!(
            slur.y_outer_apex > endpoint_y,
            "under-slur apex {} should be below (larger y than) endpoint {}",
            slur.y_outer_apex,
            endpoint_y
        );
    }

    #[test]
    fn slur_over_for_stem_down_principal() {
        // Stem-down principal → slur arcs over the notes.
        let staff = test_staff();
        let cfg = test_config();
        let principal_x = 1000.0;
        let grace_pos = 6;
        let principal_pos = 6;
        let grace = layout_grace_note(
            principal_x,
            grace_pos,
            GraceNoteKind::Appoggiatura,
            StemDirection::Down,
            &staff,
        );
        let slur = layout_grace_note_slur(
            &grace,
            principal_x,
            principal_pos,
            StemDirection::Down,
            &staff,
            &cfg,
        )
        .expect("non-degenerate slur");

        let endpoint_y = staff.y_of(principal_pos);
        assert!(
            slur.y_outer_apex < endpoint_y,
            "over-slur apex {} should be above (smaller y than) endpoint {}",
            slur.y_outer_apex,
            endpoint_y
        );
    }

    #[test]
    fn slur_endpoints_differ_for_different_pitches() {
        // Asymmetric pitches: grace below, principal above.
        let staff = test_staff();
        let cfg = test_config();
        let principal_x = 1000.0;
        let grace_pos = 2;
        let principal_pos = 6;
        let grace = layout_grace_note(
            principal_x,
            grace_pos,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let slur = layout_grace_note_slur(
            &grace,
            principal_x,
            principal_pos,
            StemDirection::Up,
            &staff,
            &cfg,
        )
        .expect("non-degenerate slur");

        // y_start corresponds to grace position (lower on staff → larger y in SVG)
        // y_end corresponds to principal position (higher on staff → smaller y).
        // After the SLUR_ENDPOINT_OFFSET_SS shift (under-slur shifts both endpoints
        // down equally), the relative order is preserved.
        assert!(
            slur.y_start > slur.y_end,
            "lower grace (pos {grace_pos}) should yield larger y than higher principal (pos {principal_pos}): {} vs {}",
            slur.y_start,
            slur.y_end
        );
    }

    #[test]
    fn slur_returns_none_for_degenerate_span() {
        // If the grace note is placed *past* the principal (defensive guard).
        let staff = test_staff();
        let cfg = test_config();

        // Manually construct a degenerate GraceNoteLayout where the grace
        // origin is essentially at the principal x.
        let degenerate = GraceNoteLayout {
            x: 1000.0,
            y: staff.y_of(4),
            glyph: Glyph::GraceNoteAcciaccaturaStemUp,
            scale: GRACE_NOTE_SCALE,
            staff_position: 4,
        };
        let slur = layout_grace_note_slur(&degenerate, 1000.0, 4, StemDirection::Up, &staff, &cfg);
        assert!(slur.is_none(), "degenerate span should return None");
    }

    #[test]
    fn slur_x_start_within_grace_glyph_bounds() {
        // The slur start should fall at or beyond the right half of the
        // scaled grace notehead — not at the glyph origin itself.
        let staff = test_staff();
        let cfg = test_config();
        let principal_x = 1500.0;
        let grace = layout_grace_note(
            principal_x,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let slur = layout_grace_note_slur(&grace, principal_x, 4, StemDirection::Up, &staff, &cfg)
            .expect("non-degenerate slur");

        // Expected start ≈ grace.x + 0.59 * staff_space * GRACE_NOTE_SCALE.
        let expected = grace.x + 0.59 * staff.staff_space * GRACE_NOTE_SCALE;
        assert!(
            (slur.x_start - expected).abs() < 0.01,
            "slur x_start {} should match expected {}",
            slur.x_start,
            expected
        );
    }

    #[test]
    fn slur_x_end_just_left_of_principal() {
        let staff = test_staff();
        let cfg = test_config();
        let principal_x = 1500.0;
        let grace = layout_grace_note(
            principal_x,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let slur = layout_grace_note_slur(&grace, principal_x, 4, StemDirection::Up, &staff, &cfg)
            .expect("non-degenerate slur");

        let expected_end = principal_x - 0.05 * staff.staff_space;
        assert!(
            (slur.x_end - expected_end).abs() < 0.01,
            "slur x_end {} should match expected {}",
            slur.x_end,
            expected_end
        );
    }
}
