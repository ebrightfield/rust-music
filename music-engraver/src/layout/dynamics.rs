use smufl::Glyph;

/// A dynamic marking placed below (or above) the staff.
///
/// Each variant maps to one or more SMuFL glyph(s) from the Dynamics range.
/// Composite dynamics (like sfz) use dedicated SMuFL composite glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dynamic {
    /// Pianississimo (ppp)
    Ppp,
    /// Pianissimo (pp)
    Pp,
    /// Piano (p)
    Piano,
    /// Mezzo-piano (mp)
    Mp,
    /// Mezzo-forte (mf)
    Mf,
    /// Forte (f)
    Forte,
    /// Fortissimo (ff)
    Ff,
    /// Fortississimo (fff)
    Fff,
    /// Forte-piano (fp)
    Fp,
    /// Sforzando (sfz)
    Sfz,
    /// Sforzando-piano (sfp)
    Sfp,
}

impl Dynamic {
    /// Return the SMuFL glyph for this dynamic marking.
    ///
    /// SMuFL provides dedicated composite glyphs for standard dynamics,
    /// so each dynamic is a single glyph (not a sequence of letter-glyphs).
    pub fn glyph(self) -> Glyph {
        match self {
            Self::Ppp => Glyph::DynamicPpp,
            Self::Pp => Glyph::DynamicPp,
            Self::Piano => Glyph::DynamicPiano,
            Self::Mp => Glyph::DynamicMp,
            Self::Mf => Glyph::DynamicMf,
            Self::Forte => Glyph::DynamicForte,
            Self::Ff => Glyph::DynamicFf,
            Self::Fff => Glyph::DynamicFff,
            Self::Fp => Glyph::DynamicFortePiano,
            Self::Sfz => Glyph::DynamicSforzando1,
            Self::Sfp => Glyph::DynamicSforzandoPiano,
        }
    }
}

/// Vertical distance from the bottom staff line to the dynamics baseline,
/// in staff spaces. Standard engraving places dynamics roughly 2 staff
/// spaces below the bottom line, adjusted for ledger lines when present.
pub const DYNAMICS_BELOW_STAFF_SS: f64 = 2.5;

/// Layout result for a dynamic marking.
#[derive(Clone, Debug)]
pub struct DynamicLayout {
    /// SMuFL glyph to render.
    pub glyph: Glyph,
    /// X-position in font design units (centered on the note it applies to).
    pub x: f64,
    /// Y-position in font design units (below the staff, baseline of the glyph).
    pub y: f64,
}

/// Compute the position for a dynamics marking below the staff.
///
/// `note_center_x` is the horizontal center of the note the dynamic applies to.
/// `dynamic_advance_width` is the advance width of the dynamic glyph (from the font).
/// `staff_bottom_y` is the y-coordinate of the bottom staff line.
/// `staff_space` is one staff space in font design units.
///
/// Returns a `DynamicLayout` with the glyph horizontally centered on the note
/// and placed at the standard distance below the staff.
pub fn layout_dynamic(
    dynamic: Dynamic,
    note_center_x: f64,
    dynamic_advance_width: f64,
    staff_bottom_y: f64,
    staff_space: f64,
) -> DynamicLayout {
    let glyph = dynamic.glyph();
    // Center the dynamic glyph horizontally on the note
    let x = note_center_x - dynamic_advance_width / 2.0;
    // Place below the staff (positive y = downward in SVG coordinates,
    // but font design units have y-up, so "below" means higher y value
    // in the flipped SVG coordinate space used by our layout)
    let y = staff_bottom_y + DYNAMICS_BELOW_STAFF_SS * staff_space;

    DynamicLayout { glyph, x, y }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_dynamic_maps_to_distinct_glyph() {
        let dynamics = [
            Dynamic::Ppp,
            Dynamic::Pp,
            Dynamic::Piano,
            Dynamic::Mp,
            Dynamic::Mf,
            Dynamic::Forte,
            Dynamic::Ff,
            Dynamic::Fff,
            Dynamic::Fp,
            Dynamic::Sfz,
            Dynamic::Sfp,
        ];
        let glyphs: Vec<_> = dynamics.iter().map(|d| d.glyph()).collect();
        // All glyphs should be unique
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(g1, g2, "{:?} and {:?} share glyph", dynamics[i], dynamics[j]);
                }
            }
        }
    }

    #[test]
    fn piano_maps_to_smufl_dynamic_piano() {
        assert_eq!(Dynamic::Piano.glyph(), Glyph::DynamicPiano);
    }

    #[test]
    fn forte_maps_to_smufl_dynamic_forte() {
        assert_eq!(Dynamic::Forte.glyph(), Glyph::DynamicForte);
    }

    #[test]
    fn mf_maps_to_smufl_dynamic_mf() {
        assert_eq!(Dynamic::Mf.glyph(), Glyph::DynamicMf);
    }

    #[test]
    fn ff_maps_to_smufl_dynamic_ff() {
        assert_eq!(Dynamic::Ff.glyph(), Glyph::DynamicFf);
    }

    #[test]
    fn sfz_maps_to_smufl_sforzando1() {
        assert_eq!(Dynamic::Sfz.glyph(), Glyph::DynamicSforzando1);
    }

    #[test]
    fn layout_centers_horizontally() {
        let layout = layout_dynamic(
            Dynamic::Forte,
            500.0, // note center
            200.0, // glyph width
            1000.0,
            250.0,
        );
        // x should be centered: 500 - 200/2 = 400
        assert!((layout.x - 400.0).abs() < 1e-6, "got {}", layout.x);
    }

    #[test]
    fn layout_places_below_staff() {
        let staff_bottom_y = 1000.0;
        let staff_space = 250.0;
        let layout = layout_dynamic(
            Dynamic::Piano,
            500.0,
            200.0,
            staff_bottom_y,
            staff_space,
        );
        // y = 1000 + 2.5 * 250 = 1625
        let expected_y = staff_bottom_y + DYNAMICS_BELOW_STAFF_SS * staff_space;
        assert!((layout.y - expected_y).abs() < 1e-6, "got {}", layout.y);
        assert!(layout.y > staff_bottom_y, "dynamic should be below staff");
    }

    #[test]
    fn layout_preserves_glyph() {
        let layout = layout_dynamic(Dynamic::Pp, 0.0, 100.0, 0.0, 250.0);
        assert_eq!(layout.glyph, Glyph::DynamicPp);
    }

    #[test]
    fn layout_with_zero_width_centers_at_note() {
        let layout = layout_dynamic(Dynamic::Ff, 300.0, 0.0, 1000.0, 250.0);
        assert!((layout.x - 300.0).abs() < 1e-6);
    }

    #[test]
    fn layout_wider_glyph_shifts_left() {
        let narrow = layout_dynamic(Dynamic::Piano, 500.0, 100.0, 1000.0, 250.0);
        let wide = layout_dynamic(Dynamic::Fff, 500.0, 400.0, 1000.0, 250.0);
        assert!(wide.x < narrow.x, "wider glyph should have lower x");
    }

    #[test]
    fn layout_different_staff_space_scales_y() {
        let small = layout_dynamic(Dynamic::Forte, 500.0, 200.0, 1000.0, 200.0);
        let large = layout_dynamic(Dynamic::Forte, 500.0, 200.0, 1000.0, 400.0);
        assert!(large.y > small.y, "larger staff space → further below staff");
    }
}
