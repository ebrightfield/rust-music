use music::notation::clef::Clef;
use smufl::Glyph;

use super::staff::StaffPosition;

/// Clef placement information for layout.
///
/// Maps a `Clef` to the SMuFL glyph that represents it and the staff
/// position where the glyph's reference point sits.
#[derive(Clone, Debug)]
pub struct ClefLayout {
    /// SMuFL glyph for this clef.
    pub glyph: Glyph,
    /// Staff position of the clef's reference line. For a G-clef this is
    /// the line that defines G4 (staff position 2 = second line from bottom).
    /// For an F-clef this is the line that defines F3 (staff position 6 =
    /// fourth line from bottom).
    pub staff_position: StaffPosition,
}

impl ClefLayout {
    /// Resolve a `Clef` to its SMuFL glyph and staff position.
    pub fn from_clef(clef: Clef) -> Self {
        match clef {
            Clef::Treble => Self {
                glyph: Glyph::GClef,
                staff_position: 2,
            },
            Clef::Treble8va => Self {
                glyph: Glyph::GClef8Va,
                staff_position: 2,
            },
            Clef::Treble8ba => Self {
                glyph: Glyph::GClef8Vb,
                staff_position: 2,
            },
            Clef::Bass => Self {
                glyph: Glyph::FClef,
                staff_position: 6,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treble_clef_maps_to_gclef_on_second_line() {
        let cl = ClefLayout::from_clef(Clef::Treble);
        assert_eq!(cl.glyph, Glyph::GClef);
        assert_eq!(cl.staff_position, 2);
    }

    #[test]
    fn treble_8va_maps_to_gclef8va_on_second_line() {
        let cl = ClefLayout::from_clef(Clef::Treble8va);
        assert_eq!(cl.glyph, Glyph::GClef8Va);
        assert_eq!(cl.staff_position, 2);
    }

    #[test]
    fn treble_8ba_maps_to_gclef8vb_on_second_line() {
        let cl = ClefLayout::from_clef(Clef::Treble8ba);
        assert_eq!(cl.glyph, Glyph::GClef8Vb);
        assert_eq!(cl.staff_position, 2);
    }

    #[test]
    fn bass_clef_maps_to_fclef_on_fourth_line() {
        let cl = ClefLayout::from_clef(Clef::Bass);
        assert_eq!(cl.glyph, Glyph::FClef);
        // Fourth line from bottom = staff position 6
        assert_eq!(cl.staff_position, 6);
    }

    #[test]
    fn all_g_clef_variants_share_same_staff_position() {
        assert_eq!(ClefLayout::from_clef(Clef::Treble).staff_position, 2);
        assert_eq!(ClefLayout::from_clef(Clef::Treble8va).staff_position, 2);
        assert_eq!(ClefLayout::from_clef(Clef::Treble8ba).staff_position, 2);
    }

    #[test]
    fn different_clef_types_produce_different_glyphs() {
        let treble = ClefLayout::from_clef(Clef::Treble);
        let bass = ClefLayout::from_clef(Clef::Bass);
        assert_ne!(treble.glyph, bass.glyph);
        assert_ne!(treble.staff_position, bass.staff_position);
    }
}
