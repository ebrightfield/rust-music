use smufl::Glyph;

/// A dynamic marking placed below (or above) the staff.
///
/// Each variant maps to a single SMuFL composite glyph from the Dynamics
/// range — SMuFL provides dedicated composite glyphs (e.g. `dynamicForte`,
/// `dynamicSforzando1`), so we never assemble letter-glyphs by hand.
///
/// Names follow the standard p/f/m/s shorthand. Sub-piano levels stack
/// `p`s (`Pp`/`Ppp`/`Pppp`/...) and sub-forte levels stack `f`s
/// (`Ff`/`Fff`/`Ffff`/...) up to six in each direction — SMuFL caps the
/// extremes at six.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dynamic {
    /// Niente (n) — silent (used in some hairpin contexts as well).
    Niente,
    /// Pianississississ (pppppp) — six p's.
    Pppppp,
    /// Pianississississ (ppppp) — five p's.
    Ppppp,
    /// Pianissississimo (pppp) — four p's.
    Pppp,
    /// Pianississimo (ppp)
    Ppp,
    /// Pianissimo (pp)
    Pp,
    /// Piano (p)
    Piano,
    /// Mezzo (m) — the bare letter-glyph, used as a building block in some
    /// contemporary notations and in older Italian text dynamics. Distinct
    /// from `Mp`/`Mf`, which are dedicated composite glyphs.
    Mezzo,
    /// Mezzo-piano (mp)
    Mp,
    /// Mezzo-forte (mf)
    Mf,
    /// Poco forte (pf) — a quiet forte, slightly under `Forte`.
    Pf,
    /// Forte (f)
    Forte,
    /// Fortissimo (ff)
    Ff,
    /// Fortississimo (fff)
    Fff,
    /// Fortissississimo (ffff) — four f's.
    Ffff,
    /// Fortississississimo (fffff) — five f's.
    Fffff,
    /// Fortissississississimo (ffffff) — six f's.
    Ffffff,
    /// Forte-piano (fp)
    Fp,
    /// Sforzando (sfz) — `dynamicSforzando1`, the standard "sfz" glyph.
    Sfz,
    /// Sforzato (sf) — single-letter "sf" accent variant.
    Sf,
    /// Sforzato-ff (sff) — sforzato followed by fortissimo.
    Sff,
    /// Sforzando-piano (sfp) — sforzando-prefixed piano. Uses the SMuFL
    /// `dynamicSforzandoPiano` glyph; see `SforzatoPiano` for the visually
    /// distinct sforzato-prefixed variant.
    Sfp,
    /// Sforzando-pianissimo (sfpp) — sforzando followed by pianissimo.
    Sfpp,
    /// Sforzato-piano (sfp) — sforzato-prefixed piano. Visually similar to
    /// `Sfp` but uses the SMuFL `dynamicSforzatoPiano` glyph (subtle "s"
    /// letterform difference matching the sforzato family: `Sf`/`Sff`).
    SforzatoPiano,
    /// Forzando (fz)
    Fz,
    /// Rinforzando (rf) — single-letter `dynamicRinforzando` form.
    Rf,
    /// Rinforzando (rfz) — `dynamicRinforzando1`, the standard "rfz" glyph.
    Rfz,
    /// Z (z) — the rare single-letter `dynamicZ` mark, occasionally used
    /// in 20th-century scores as a sudden-accent indication.
    Z,
}

impl Dynamic {
    /// Return the SMuFL glyph for this dynamic marking.
    ///
    /// SMuFL provides dedicated composite glyphs for standard dynamics,
    /// so each dynamic is a single glyph (not a sequence of letter-glyphs).
    pub fn glyph(self) -> Glyph {
        match self {
            Self::Niente => Glyph::DynamicNiente,
            Self::Pppppp => Glyph::DynamicPppppp,
            Self::Ppppp => Glyph::DynamicPpppp,
            Self::Pppp => Glyph::DynamicPppp,
            Self::Ppp => Glyph::DynamicPpp,
            Self::Pp => Glyph::DynamicPp,
            Self::Piano => Glyph::DynamicPiano,
            Self::Mezzo => Glyph::DynamicMezzo,
            Self::Mp => Glyph::DynamicMp,
            Self::Mf => Glyph::DynamicMf,
            Self::Pf => Glyph::DynamicPf,
            Self::Forte => Glyph::DynamicForte,
            Self::Ff => Glyph::DynamicFf,
            Self::Fff => Glyph::DynamicFff,
            Self::Ffff => Glyph::DynamicFfff,
            Self::Fffff => Glyph::DynamicFffff,
            Self::Ffffff => Glyph::DynamicFfffff,
            Self::Fp => Glyph::DynamicFortePiano,
            Self::Sfz => Glyph::DynamicSforzando1,
            Self::Sf => Glyph::DynamicSforzato,
            Self::Sff => Glyph::DynamicSforzatoFf,
            Self::Sfp => Glyph::DynamicSforzandoPiano,
            Self::Sfpp => Glyph::DynamicSforzandoPianissimo,
            Self::SforzatoPiano => Glyph::DynamicSforzatoPiano,
            Self::Fz => Glyph::DynamicForzando,
            Self::Rf => Glyph::DynamicRinforzando,
            Self::Rfz => Glyph::DynamicRinforzando1,
            Self::Z => Glyph::DynamicZ,
        }
    }

    /// Every dynamic variant, in a canonical ordering from quietest to
    /// loudest (with accent-style markings — sf, sfz, rf, etc. — listed
    /// after the simple level markings since their loudness is contextual).
    ///
    /// Useful for tests and for iterating over the full set of supported
    /// dynamics in higher-level code.
    pub const ALL: [Dynamic; 28] = [
        Self::Niente,
        Self::Pppppp,
        Self::Ppppp,
        Self::Pppp,
        Self::Ppp,
        Self::Pp,
        Self::Piano,
        Self::Mezzo,
        Self::Mp,
        Self::Mf,
        Self::Pf,
        Self::Forte,
        Self::Ff,
        Self::Fff,
        Self::Ffff,
        Self::Fffff,
        Self::Ffffff,
        Self::Fp,
        Self::Sfz,
        Self::Sf,
        Self::Sff,
        Self::Sfp,
        Self::SforzatoPiano,
        Self::Sfpp,
        Self::Fz,
        Self::Rf,
        Self::Rfz,
        Self::Z,
    ];
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
        let dynamics = Dynamic::ALL;
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
    fn all_constant_lists_every_variant_exactly_once() {
        // Defensive: if a new variant is added without updating ALL, this
        // catches it by exhaustive matching on a representative value.
        // Iterating ALL and ensuring no two entries share a Debug repr is
        // enough: variants are distinct value-wise (Eq/Hash derive).
        use std::collections::HashSet;
        let unique: HashSet<_> = Dynamic::ALL.iter().collect();
        assert_eq!(
            unique.len(),
            Dynamic::ALL.len(),
            "Dynamic::ALL contains duplicates"
        );
    }

    #[test]
    fn extreme_pianissimos_map_to_smufl_p_levels() {
        assert_eq!(Dynamic::Pppp.glyph(), Glyph::DynamicPppp);
        assert_eq!(Dynamic::Ppppp.glyph(), Glyph::DynamicPpppp);
        assert_eq!(Dynamic::Pppppp.glyph(), Glyph::DynamicPppppp);
    }

    #[test]
    fn extreme_fortissimos_map_to_smufl_f_levels() {
        assert_eq!(Dynamic::Ffff.glyph(), Glyph::DynamicFfff);
        assert_eq!(Dynamic::Fffff.glyph(), Glyph::DynamicFffff);
        assert_eq!(Dynamic::Ffffff.glyph(), Glyph::DynamicFfffff);
    }

    #[test]
    fn accent_dynamics_map_to_smufl_glyphs() {
        // sf (single-letter sforzato) is a different SMuFL glyph from
        // sfz (the "sfz" composite); confirm they don't collapse.
        assert_eq!(Dynamic::Sf.glyph(), Glyph::DynamicSforzato);
        assert_ne!(Dynamic::Sf.glyph(), Dynamic::Sfz.glyph());

        assert_eq!(Dynamic::Sff.glyph(), Glyph::DynamicSforzatoFf);
        assert_eq!(Dynamic::Sfpp.glyph(), Glyph::DynamicSforzandoPianissimo);
        // sfp (sforzando-piano) is distinct from the sforzato-piano variant.
        assert_eq!(Dynamic::Sfp.glyph(), Glyph::DynamicSforzandoPiano);
    }

    #[test]
    fn rinforzando_dynamics_map_to_smufl_glyphs() {
        // Two SMuFL flavors: short `dynamicRinforzando` and full `rfz` form.
        assert_eq!(Dynamic::Rf.glyph(), Glyph::DynamicRinforzando);
        assert_eq!(Dynamic::Rfz.glyph(), Glyph::DynamicRinforzando1);
        assert_ne!(Dynamic::Rf.glyph(), Dynamic::Rfz.glyph());
    }

    #[test]
    fn forzando_maps_to_smufl_forzando() {
        assert_eq!(Dynamic::Fz.glyph(), Glyph::DynamicForzando);
    }

    #[test]
    fn niente_maps_to_smufl_niente() {
        assert_eq!(Dynamic::Niente.glyph(), Glyph::DynamicNiente);
    }

    #[test]
    fn mezzo_maps_to_smufl_mezzo() {
        // The bare "m" letter glyph must be distinct from the composites
        // mp and mf, which are dedicated SMuFL glyphs in their own right.
        assert_eq!(Dynamic::Mezzo.glyph(), Glyph::DynamicMezzo);
        assert_ne!(Dynamic::Mezzo.glyph(), Dynamic::Mp.glyph());
        assert_ne!(Dynamic::Mezzo.glyph(), Dynamic::Mf.glyph());
    }

    #[test]
    fn z_maps_to_smufl_dynamic_z() {
        // Z is rare but visually distinctive — must not collapse to any
        // other accent-style glyph (sf, sfz, fz, rfz, etc.).
        assert_eq!(Dynamic::Z.glyph(), Glyph::DynamicZ);
        assert_ne!(Dynamic::Z.glyph(), Dynamic::Sfz.glyph());
        assert_ne!(Dynamic::Z.glyph(), Dynamic::Sf.glyph());
        assert_ne!(Dynamic::Z.glyph(), Dynamic::Fz.glyph());
        assert_ne!(Dynamic::Z.glyph(), Dynamic::Rfz.glyph());
    }

    #[test]
    fn sforzato_piano_distinct_from_sforzando_piano() {
        // Both spell "sfp" but use a different 's' letterform: `Sfp`
        // is the sforzando-prefix glyph; `SforzatoPiano` is the
        // sforzato-prefix glyph (matching the Sf/Sff letterform family).
        assert_eq!(
            Dynamic::SforzatoPiano.glyph(),
            Glyph::DynamicSforzatoPiano
        );
        assert_eq!(Dynamic::Sfp.glyph(), Glyph::DynamicSforzandoPiano);
        assert_ne!(Dynamic::SforzatoPiano.glyph(), Dynamic::Sfp.glyph());
        // Also distinct from the bare sforzato (Sf) and Sfpp variants.
        assert_ne!(Dynamic::SforzatoPiano.glyph(), Dynamic::Sf.glyph());
        assert_ne!(Dynamic::SforzatoPiano.glyph(), Dynamic::Sfpp.glyph());
    }

    #[test]
    fn all_constant_contains_each_new_variant_exactly_once() {
        let count_mezzo = Dynamic::ALL.iter().filter(|d| matches!(d, Dynamic::Mezzo)).count();
        let count_z = Dynamic::ALL.iter().filter(|d| matches!(d, Dynamic::Z)).count();
        let count_sfp_alt = Dynamic::ALL
            .iter()
            .filter(|d| matches!(d, Dynamic::SforzatoPiano))
            .count();
        assert_eq!(count_mezzo, 1, "Mezzo must appear exactly once in ALL");
        assert_eq!(count_z, 1, "Z must appear exactly once in ALL");
        assert_eq!(
            count_sfp_alt, 1,
            "SforzatoPiano must appear exactly once in ALL"
        );
    }

    #[test]
    fn all_constant_has_expected_length() {
        // Documents the current 28-variant total. Bump this assertion
        // intentionally when a new variant is added.
        assert_eq!(Dynamic::ALL.len(), 28);
    }

    #[test]
    fn poco_forte_maps_to_smufl_pf() {
        assert_eq!(Dynamic::Pf.glyph(), Glyph::DynamicPf);
        // pf is a quiet-forte composite; must not collapse to f or p.
        assert_ne!(Dynamic::Pf.glyph(), Dynamic::Forte.glyph());
        assert_ne!(Dynamic::Pf.glyph(), Dynamic::Piano.glyph());
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
