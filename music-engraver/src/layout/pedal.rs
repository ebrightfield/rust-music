/// Piano pedal marking layout — "Ped." and "*" symbols below the staff.
///
/// Standard engraving places pedal markings well below the staff,
/// below dynamics and expression text. "Ped." (depress) and "*" (release)
/// are the traditional glyph-based notation; bracket-style pedal lines
/// are a more modern alternative (not yet implemented).
use smufl::Glyph;

use crate::layout::staff::StaffLayout;

/// A pedal marking event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PedalMark {
    /// Sustain pedal down — renders the "Ped." glyph (SMuFL keyboardPedalPed).
    Down,
    /// Sustain pedal up — renders the asterisk/star glyph (SMuFL keyboardPedalUp).
    Up,
    /// Half pedal — partial sustain pedal depression, used by late-romantic and
    /// modern composers to retain partial resonance while reducing blur.
    /// Renders the SMuFL keyboardPedalHalf glyph (a horizontal stroke through
    /// a vertical hash).
    Half,
    /// Sostenuto pedal down — the middle pedal on a grand piano, which
    /// sustains only the notes already held when it is depressed.
    /// Renders the SMuFL keyboardPedalSost glyph ("Sost.").
    Sost,
}

impl PedalMark {
    /// Return the SMuFL glyph for this pedal marking.
    pub fn glyph(self) -> Glyph {
        match self {
            PedalMark::Down => Glyph::KeyboardPedalPed,
            PedalMark::Up => Glyph::KeyboardPedalUp,
            PedalMark::Half => Glyph::KeyboardPedalHalf,
            PedalMark::Sost => Glyph::KeyboardPedalSost,
        }
    }
}

/// Vertical distance from the bottom staff line to the pedal marking baseline,
/// in staff spaces. Placed well below dynamics (2.5ss), expression text (4.0ss),
/// and lyrics (5.5ss) to avoid collisions.
pub const PEDAL_BELOW_STAFF_SS: f64 = 7.0;

/// Layout result for a pedal marking.
#[derive(Clone, Debug)]
pub struct PedalLayout {
    /// SMuFL glyph to render.
    pub glyph: Glyph,
    /// X-position in font design units (centered on the note it applies to).
    pub x: f64,
    /// Y-position in font design units (below the staff, baseline of the glyph).
    pub y: f64,
}

/// Compute the position for a pedal marking below the staff.
///
/// `note_center_x` is the horizontal center of the note the pedal event applies to.
/// `pedal_advance_width` is the advance width of the pedal glyph (from the font).
/// `staff` provides the bottom y-coordinate and staff space.
pub fn layout_pedal(
    mark: PedalMark,
    note_center_x: f64,
    pedal_advance_width: f64,
    staff: &StaffLayout,
) -> PedalLayout {
    let glyph = mark.glyph();
    let x = note_center_x - pedal_advance_width / 2.0;
    let y = staff.bottom_y() + PEDAL_BELOW_STAFF_SS * staff.staff_space;

    PedalLayout { glyph, x, y }
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
    fn down_maps_to_keyboard_pedal_ped() {
        assert_eq!(PedalMark::Down.glyph(), Glyph::KeyboardPedalPed);
    }

    #[test]
    fn up_maps_to_keyboard_pedal_up() {
        assert_eq!(PedalMark::Up.glyph(), Glyph::KeyboardPedalUp);
    }

    #[test]
    fn down_and_up_are_distinct_glyphs() {
        assert_ne!(PedalMark::Down.glyph(), PedalMark::Up.glyph());
    }

    #[test]
    fn half_maps_to_keyboard_pedal_half() {
        assert_eq!(PedalMark::Half.glyph(), Glyph::KeyboardPedalHalf);
    }

    #[test]
    fn sost_maps_to_keyboard_pedal_sost() {
        assert_eq!(PedalMark::Sost.glyph(), Glyph::KeyboardPedalSost);
    }

    #[test]
    fn all_four_variants_have_distinct_glyphs() {
        // Each variant must produce a unique glyph so callers can rely on
        // visual disambiguation between Down/Up/Half/Sost in published output.
        let glyphs = [
            PedalMark::Down.glyph(),
            PedalMark::Up.glyph(),
            PedalMark::Half.glyph(),
            PedalMark::Sost.glyph(),
        ];
        for i in 0..glyphs.len() {
            for j in (i + 1)..glyphs.len() {
                assert_ne!(
                    glyphs[i], glyphs[j],
                    "PedalMark variants at index {i} and {j} share a glyph"
                );
            }
        }
    }

    #[test]
    fn layout_preserves_glyph_for_half() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Half, 0.0, 100.0, &staff);
        assert_eq!(layout.glyph, Glyph::KeyboardPedalHalf);
    }

    #[test]
    fn layout_preserves_glyph_for_sost() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Sost, 0.0, 100.0, &staff);
        assert_eq!(layout.glyph, Glyph::KeyboardPedalSost);
    }

    #[test]
    fn half_layout_below_staff_matches_other_variants() {
        // All pedal variants share the same vertical band — the y coordinate
        // is the staff's bottom + PEDAL_BELOW_STAFF_SS regardless of glyph.
        // A Half marking is still a pedal, so it belongs on the pedal axis.
        let staff = test_staff();
        let half = layout_pedal(PedalMark::Half, 500.0, 100.0, &staff);
        let down = layout_pedal(PedalMark::Down, 500.0, 100.0, &staff);
        assert!(
            (half.y - down.y).abs() < 1e-6,
            "Half pedal y ({}) must match Down pedal y ({})",
            half.y,
            down.y
        );
    }

    #[test]
    fn sost_layout_below_staff_matches_other_variants() {
        let staff = test_staff();
        let sost = layout_pedal(PedalMark::Sost, 500.0, 100.0, &staff);
        let down = layout_pedal(PedalMark::Down, 500.0, 100.0, &staff);
        assert!(
            (sost.y - down.y).abs() < 1e-6,
            "Sost pedal y ({}) must match Down pedal y ({})",
            sost.y,
            down.y
        );
    }

    #[test]
    fn layout_centers_horizontally() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Down, 500.0, 200.0, &staff);
        let expected_x = 500.0 - 200.0 / 2.0;
        assert!((layout.x - expected_x).abs() < 1e-6, "got {}", layout.x);
    }

    #[test]
    fn layout_places_below_staff_at_correct_distance() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Down, 500.0, 200.0, &staff);
        let expected_y = staff.bottom_y() + PEDAL_BELOW_STAFF_SS * staff.staff_space;
        assert!(
            (layout.y - expected_y).abs() < 1e-6,
            "expected y={expected_y}, got {}",
            layout.y
        );
    }

    #[test]
    fn layout_y_is_below_staff_bottom() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Up, 500.0, 100.0, &staff);
        assert!(layout.y > staff.bottom_y(), "pedal should be below staff");
    }

    #[test]
    fn layout_preserves_glyph_for_down() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Down, 0.0, 100.0, &staff);
        assert_eq!(layout.glyph, Glyph::KeyboardPedalPed);
    }

    #[test]
    fn layout_preserves_glyph_for_up() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Up, 0.0, 100.0, &staff);
        assert_eq!(layout.glyph, Glyph::KeyboardPedalUp);
    }

    #[test]
    fn layout_with_zero_width_centers_at_note() {
        let staff = test_staff();
        let layout = layout_pedal(PedalMark::Down, 300.0, 0.0, &staff);
        assert!((layout.x - 300.0).abs() < 1e-6);
    }

    #[test]
    fn wider_glyph_shifts_x_left() {
        let staff = test_staff();
        let narrow = layout_pedal(PedalMark::Down, 500.0, 100.0, &staff);
        let wide = layout_pedal(PedalMark::Down, 500.0, 400.0, &staff);
        assert!(wide.x < narrow.x, "wider glyph should have lower x");
    }

    #[test]
    fn down_and_up_at_same_position_differ_only_in_glyph() {
        let staff = test_staff();
        let down = layout_pedal(PedalMark::Down, 500.0, 200.0, &staff);
        let up = layout_pedal(PedalMark::Up, 500.0, 200.0, &staff);
        // Same position since both are centered with the same width
        assert!((down.x - up.x).abs() < 1e-6);
        assert!((down.y - up.y).abs() < 1e-6);
        // Different glyphs
        assert_ne!(down.glyph, up.glyph);
    }
}
