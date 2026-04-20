use smufl::Glyph;

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
        (GraceNoteKind::Acciaccatura, StemDirection::Up) => {
            Glyph::GraceNoteAcciaccaturaStemUp
        }
        (GraceNoteKind::Acciaccatura, StemDirection::Down) => {
            Glyph::GraceNoteAcciaccaturaStemDown
        }
        (GraceNoteKind::Appoggiatura, StemDirection::Up) => {
            Glyph::GraceNoteAppoggiaturaStemUp
        }
        (GraceNoteKind::Appoggiatura, StemDirection::Down) => {
            Glyph::GraceNoteAppoggiaturaStemDown
        }
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
}
