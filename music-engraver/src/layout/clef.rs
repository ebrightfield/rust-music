use music::notation::clef::Clef;
use smufl::Glyph;

use super::glyph_metrics::{glyph_box, GlyphBox};
use super::staff::StaffPosition;

/// Whether a clef is drawn at full size (system start) or as a smaller
/// mid-score change clef.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClefSize {
    /// Full-size clef, as in a system prefix.
    #[default]
    Full,
    /// Change-size clef for a clef change inside a system or as a courtesy
    /// clef at a system end. Uses the SMuFL `*ClefChange` glyph where one
    /// exists, otherwise the full glyph scaled by [`ClefLayout::scale`].
    Change,
}

/// Clef placement information for layout.
///
/// Maps a `Clef` to the SMuFL glyph that represents it and the staff
/// position where the glyph's reference point sits.
#[derive(Clone, Debug, PartialEq)]
pub struct ClefLayout {
    /// SMuFL glyph for this clef.
    pub glyph: Glyph,
    /// Staff position of the clef's reference line. For a G-clef this is
    /// the line that defines G4 (staff position 2 = second line from bottom).
    /// For an F-clef this is the line that defines F3 (staff position 6 =
    /// fourth line from bottom). For a C-clef this is the line that defines
    /// C4, on which the glyph is vertically centered (alto: staff position 4 =
    /// middle line; tenor: staff position 6 = fourth line from bottom).
    pub staff_position: StaffPosition,
    /// Full-size or change-size.
    pub size: ClefSize,
}

impl ClefLayout {
    /// Resolve a `&Clef` to its full-size SMuFL glyph and staff position.
    pub fn from_clef_ref(clef: &Clef) -> Self {
        Self::sized(clef, ClefSize::Full)
    }

    /// Resolve a `Clef` to its full-size SMuFL glyph and staff position
    /// (consumes value).
    pub fn from_clef(clef: Clef) -> Self {
        Self::sized(&clef, ClefSize::Full)
    }

    /// The change-size clef for a mid-score clef change: `gClefChange`,
    /// `fClefChange` or `cClefChange`; the octave-transposing treble clefs,
    /// which have no SMuFL change glyph, use their full glyph scaled down.
    pub fn change(clef: &Clef) -> Self {
        Self::sized(clef, ClefSize::Change)
    }

    fn sized(clef: &Clef, size: ClefSize) -> Self {
        let change = size == ClefSize::Change;
        let (glyph, staff_position) = match clef {
            Clef::Treble if change => (Glyph::GClefChange, 2),
            Clef::Treble => (Glyph::GClef, 2),
            Clef::Treble8va => (Glyph::GClef8Va, 2),
            Clef::Treble8ba => (Glyph::GClef8Vb, 2),
            Clef::Bass if change => (Glyph::FClefChange, 6),
            Clef::Bass => (Glyph::FClef, 6),
            Clef::Alto if change => (Glyph::CClefChange, 4),
            Clef::Alto => (Glyph::CClef, 4),
            Clef::Tenor if change => (Glyph::CClefChange, 6),
            Clef::Tenor => (Glyph::CClef, 6),
        };
        Self {
            glyph,
            staff_position,
            size,
        }
    }

    /// Uniform scale applied to [`Self::glyph`] when drawing: 1 except for a
    /// change-size clef drawn from a full-size glyph, which shrinks by the
    /// font's `gClefChange : gClef` height ratio.
    pub fn scale(&self) -> f64 {
        let dedicated_change_glyph = matches!(
            self.glyph,
            Glyph::GClefChange | Glyph::FClefChange | Glyph::CClefChange
        );
        if self.size == ClefSize::Full || dedicated_change_glyph {
            return 1.0;
        }
        match (glyph_box(Glyph::GClefChange), glyph_box(Glyph::GClef)) {
            (Some(change), Some(full)) if full.height() > 0.0 => change.height() / full.height(),
            _ => 1.0,
        }
    }

    /// Inked extent of the clef as drawn, relative to its origin (which sits
    /// on [`Self::staff_position`]), in staff spaces with the drawing scale
    /// applied. Falls back to a 2.8 × 4 staff-space box if the font has no
    /// bounding box for the glyph.
    pub fn ink_box(&self) -> GlyphBox {
        glyph_box(self.glyph)
            .unwrap_or(GlyphBox {
                x_left: 0.0,
                x_right: 2.8,
                y_top: -2.0,
                y_bottom: 2.0,
            })
            .scaled(self.scale())
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
    fn alto_clef_maps_to_cclef_on_middle_line() {
        for cl in [
            ClefLayout::from_clef(Clef::Alto),
            ClefLayout::from_clef_ref(&Clef::Alto),
        ] {
            assert_eq!(cl.glyph, Glyph::CClef);
            assert_eq!(cl.staff_position, 4);
        }
    }

    #[test]
    fn tenor_clef_maps_to_cclef_on_fourth_line() {
        for cl in [
            ClefLayout::from_clef(Clef::Tenor),
            ClefLayout::from_clef_ref(&Clef::Tenor),
        ] {
            assert_eq!(cl.glyph, Glyph::CClef);
            assert_eq!(cl.staff_position, 6);
        }
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
