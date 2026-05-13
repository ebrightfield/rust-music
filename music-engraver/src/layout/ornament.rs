use smufl::Glyph;

use crate::layout::staff::StaffLayout;

/// Ornament markings placed above the staff.
///
/// Ornaments are conventionally placed above the note, centered horizontally.
/// Unlike articulations, ornaments do not flip based on stem direction — they
/// always appear above (or very rarely below, which we don't support in v1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ornament {
    /// Trill — a "tr" symbol, often followed by a wavy line.
    Trill,
    /// Mordent — a short squiggle indicating a rapid alternation with the
    /// note below.
    Mordent,
    /// Inverted mordent (Pralltriller) — rapid alternation with the note above.
    /// Visually similar to mordent but with a vertical line through the middle.
    InvertedMordent,
    /// Turn — an S-shaped ornament indicating a four-note figure
    /// (upper neighbor → principal → lower neighbor → principal).
    Turn,
    /// Inverted turn — the turn figure reversed.
    InvertedTurn,
    /// Turn with slash — a turn with a vertical slash through the center,
    /// indicating the turn happens between two notes rather than on the beat.
    TurnSlash,
    /// Short trill (trill without wavy line) — a compact trill symbol used
    /// when no extended wavy line follows.
    ShortTrill,
    /// Vertical-axis turn — a turn rotated 90° clockwise. Used as Mozart's
    /// turn sign and notated for clarinet/wind articulation in some
    /// 19th-century repertoire.
    TurnUp,
    /// Vertical-axis turn with slash — the slashed variant of [`Self::TurnUp`].
    TurnUpSlash,
    /// Tremblement — a French Baroque trill-like ornament (Couperin /
    /// D'Anglebert). Distinct from the wavy-line trill: rendered as a
    /// stacked-zigzag glyph above the note.
    Tremblement,
    /// Tremblement (Couperin variant) — the specific zigzag form used in
    /// François Couperin's tables.
    TremblementCouperin,
    /// Haydn ornament — a stuttered turn-figure introduced in Joseph Haydn's
    /// keyboard works.
    Haydn,
    /// Shake — a three-line wavy mark indicating a rapid alternation,
    /// historically distinct from the trill in 17th–18th century practice.
    Shake,
    /// Schleifer (slide) — a German Baroque slide-into-note ornament,
    /// rendered as a small upward-curving glyph before the principal note.
    Schleifer,
    /// Trill-with-mordent — a precomposed compound ornament (trill suffixed
    /// with a mordent), commonly notated in Baroque keyboard music as a
    /// single symbol rather than two stacked marks.
    TrillWithMordent,
}

impl Ornament {
    /// Return the SMuFL glyph for this ornament.
    pub fn glyph(self) -> Glyph {
        match self {
            Self::Trill => Glyph::OrnamentTrill,
            Self::Mordent => Glyph::OrnamentMordent,
            Self::InvertedMordent => Glyph::OrnamentShortTrill,
            Self::Turn => Glyph::OrnamentTurn,
            Self::InvertedTurn => Glyph::OrnamentTurnInverted,
            Self::TurnSlash => Glyph::OrnamentTurnSlash,
            Self::ShortTrill => Glyph::OrnamentShortTrill,
            Self::TurnUp => Glyph::OrnamentTurnUp,
            Self::TurnUpSlash => Glyph::OrnamentTurnUpS,
            Self::Tremblement => Glyph::OrnamentTremblement,
            Self::TremblementCouperin => Glyph::OrnamentTremblementCouperin,
            Self::Haydn => Glyph::OrnamentHaydn,
            Self::Shake => Glyph::OrnamentShake3,
            Self::Schleifer => Glyph::OrnamentSchleifer,
            Self::TrillWithMordent => Glyph::OrnamentPrecompTrillWithMordent,
        }
    }

    /// Every ornament variant, in a canonical ordering (trills first, then
    /// mordent family, then turn family in horizontal+vertical pairs, then
    /// historical/precomposed ornaments).
    ///
    /// Useful for tests and for iterating the full set of supported
    /// ornaments in higher-level code.
    pub const ALL: [Ornament; 15] = [
        Self::Trill,
        Self::ShortTrill,
        Self::Mordent,
        Self::InvertedMordent,
        Self::Turn,
        Self::InvertedTurn,
        Self::TurnSlash,
        Self::TurnUp,
        Self::TurnUpSlash,
        Self::Tremblement,
        Self::TremblementCouperin,
        Self::Haydn,
        Self::Shake,
        Self::Schleifer,
        Self::TrillWithMordent,
    ];

    /// All standard ornament variants, useful for iteration in tests.
    ///
    /// Returns the same set as [`Self::ALL`] but as a slice — preserved for
    /// callers that prefer the `&[Ornament]` shape.
    pub fn all() -> &'static [Ornament] {
        &Self::ALL
    }
}

/// Distance (in staff spaces) between the top staff line and the ornament
/// glyph anchor. Ornaments sit above the staff with comfortable clearance.
const ORNAMENT_ABOVE_STAFF_SS: f64 = 1.5;

/// Computed ornament position.
#[derive(Clone, Debug)]
pub struct OrnamentLayout {
    /// X-coordinate (centered on notehead).
    pub x: f64,
    /// Y-coordinate of the ornament glyph anchor.
    pub y: f64,
    /// The SMuFL glyph to render.
    pub glyph: Glyph,
}

/// Compute the position for an ornament relative to a note.
///
/// Ornaments are always placed above the staff, centered on `notehead_x`.
/// If the note is above the staff, the ornament moves up to maintain
/// clearance above the notehead.
pub fn layout_ornament(
    ornament: Ornament,
    notehead_x: f64,
    note_staff_position: i8,
    staff: &StaffLayout,
) -> OrnamentLayout {
    let glyph = ornament.glyph();
    let offset_fu = ORNAMENT_ABOVE_STAFF_SS * staff.staff_space;

    // Default: above the top staff line
    let above_staff_y = staff.y_of(8) - offset_fu;

    // If the note is above the staff, place the ornament above the note instead
    let above_note_y = staff.y_of(note_staff_position) - offset_fu;

    // Use whichever is higher (lower y value = higher on page)
    let y = above_staff_y.min(above_note_y);

    OrnamentLayout {
        x: notehead_x,
        y,
        glyph,
    }
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
    fn trill_glyph() {
        assert_eq!(Ornament::Trill.glyph(), Glyph::OrnamentTrill);
    }

    #[test]
    fn mordent_glyph() {
        assert_eq!(Ornament::Mordent.glyph(), Glyph::OrnamentMordent);
    }

    #[test]
    fn turn_glyph() {
        assert_eq!(Ornament::Turn.glyph(), Glyph::OrnamentTurn);
    }

    #[test]
    fn inverted_turn_glyph() {
        assert_eq!(Ornament::InvertedTurn.glyph(), Glyph::OrnamentTurnInverted);
    }

    #[test]
    fn turn_slash_glyph() {
        assert_eq!(Ornament::TurnSlash.glyph(), Glyph::OrnamentTurnSlash);
    }

    #[test]
    fn inverted_mordent_glyph() {
        // Inverted mordent uses the "short trill" glyph per SMuFL convention
        assert_eq!(Ornament::InvertedMordent.glyph(), Glyph::OrnamentShortTrill);
    }

    #[test]
    fn all_returns_fifteen_variants() {
        assert_eq!(Ornament::all().len(), 15);
        assert_eq!(Ornament::ALL.len(), 15);
    }

    #[test]
    fn all_and_const_yield_same_slice() {
        let from_fn: &[Ornament] = Ornament::all();
        let from_const: &[Ornament] = &Ornament::ALL;
        assert_eq!(from_fn, from_const);
    }

    #[test]
    fn all_contains_no_duplicates() {
        use std::collections::HashSet;
        let unique: HashSet<_> = Ornament::ALL.iter().collect();
        assert_eq!(
            unique.len(),
            Ornament::ALL.len(),
            "Ornament::ALL contains duplicates"
        );
    }

    #[test]
    fn all_variants_map_to_distinct_glyphs_except_short_trill_alias() {
        // InvertedMordent and ShortTrill share the SMuFL OrnamentShortTrill
        // glyph by long-standing engraving convention — the Pralltriller is
        // visually identical to a short trill in modern notation. Every
        // other pair must map to a different glyph.
        use std::collections::HashMap;
        let mut by_glyph: HashMap<Glyph, Vec<Ornament>> = HashMap::new();
        for o in Ornament::ALL {
            by_glyph.entry(o.glyph()).or_default().push(o);
        }
        for (glyph, variants) in &by_glyph {
            if variants.len() > 1 {
                let mut sorted = variants.clone();
                sorted.sort_by_key(|o| format!("{o:?}"));
                let expected = vec![Ornament::InvertedMordent, Ornament::ShortTrill];
                assert_eq!(
                    sorted, expected,
                    "unexpected glyph collision for {glyph:?}: {variants:?}"
                );
            }
        }
    }

    #[test]
    fn trill_and_mordent_produce_distinct_glyphs() {
        assert_ne!(Ornament::Trill.glyph(), Ornament::Mordent.glyph());
    }

    #[test]
    fn turn_and_inverted_turn_produce_distinct_glyphs() {
        assert_ne!(Ornament::Turn.glyph(), Ornament::InvertedTurn.glyph());
    }

    #[test]
    fn turn_up_glyph() {
        assert_eq!(Ornament::TurnUp.glyph(), Glyph::OrnamentTurnUp);
    }

    #[test]
    fn turn_up_slash_glyph() {
        assert_eq!(Ornament::TurnUpSlash.glyph(), Glyph::OrnamentTurnUpS);
    }

    #[test]
    fn turn_up_and_turn_up_slash_differ() {
        assert_ne!(Ornament::TurnUp.glyph(), Ornament::TurnUpSlash.glyph());
    }

    #[test]
    fn horizontal_and_vertical_turns_differ() {
        // Mozart-style vertical turn (TurnUp) must not collapse to the
        // horizontal Turn glyph — they are visually distinct in every SMuFL
        // font and conflating them silently would be an engraving regression.
        assert_ne!(Ornament::Turn.glyph(), Ornament::TurnUp.glyph());
        assert_ne!(Ornament::TurnSlash.glyph(), Ornament::TurnUpSlash.glyph());
    }

    #[test]
    fn tremblement_glyph() {
        assert_eq!(Ornament::Tremblement.glyph(), Glyph::OrnamentTremblement);
    }

    #[test]
    fn tremblement_couperin_glyph() {
        assert_eq!(
            Ornament::TremblementCouperin.glyph(),
            Glyph::OrnamentTremblementCouperin
        );
    }

    #[test]
    fn tremblement_variants_differ() {
        assert_ne!(
            Ornament::Tremblement.glyph(),
            Ornament::TremblementCouperin.glyph()
        );
    }

    #[test]
    fn tremblement_is_distinct_from_trill() {
        // Tremblement and Trill are both trill-family ornaments but the
        // glyphs must be visually distinct — French Baroque notation
        // depends on the differentiation.
        assert_ne!(Ornament::Tremblement.glyph(), Ornament::Trill.glyph());
    }

    #[test]
    fn haydn_glyph() {
        assert_eq!(Ornament::Haydn.glyph(), Glyph::OrnamentHaydn);
    }

    #[test]
    fn shake_glyph() {
        assert_eq!(Ornament::Shake.glyph(), Glyph::OrnamentShake3);
    }

    #[test]
    fn schleifer_glyph() {
        assert_eq!(Ornament::Schleifer.glyph(), Glyph::OrnamentSchleifer);
    }

    #[test]
    fn trill_with_mordent_glyph() {
        assert_eq!(
            Ornament::TrillWithMordent.glyph(),
            Glyph::OrnamentPrecompTrillWithMordent
        );
    }

    #[test]
    fn trill_with_mordent_distinct_from_trill_and_mordent() {
        let twm = Ornament::TrillWithMordent.glyph();
        assert_ne!(twm, Ornament::Trill.glyph());
        assert_ne!(twm, Ornament::Mordent.glyph());
    }

    #[test]
    fn all_contains_each_new_variant_exactly_once() {
        for v in [
            Ornament::TurnUp,
            Ornament::TurnUpSlash,
            Ornament::Tremblement,
            Ornament::TremblementCouperin,
            Ornament::Haydn,
            Ornament::Shake,
            Ornament::Schleifer,
            Ornament::TrillWithMordent,
        ] {
            let count = Ornament::ALL.iter().filter(|&&o| o == v).count();
            assert_eq!(count, 1, "{v:?} should appear exactly once in ALL");
        }
    }

    #[test]
    fn layout_above_staff_for_middle_note() {
        let staff = test_staff();
        let layout = layout_ornament(Ornament::Trill, 100.0, 4, &staff);
        let top_line_y = staff.y_of(8);
        assert!(
            layout.y < top_line_y,
            "ornament y ({}) should be above top staff line y ({})",
            layout.y,
            top_line_y
        );
        assert_eq!(layout.x, 100.0);
        assert_eq!(layout.glyph, Glyph::OrnamentTrill);
    }

    #[test]
    fn layout_above_note_for_high_note() {
        let staff = test_staff();
        // Note at staff position 12 (well above the staff)
        let layout = layout_ornament(Ornament::Turn, 200.0, 12, &staff);
        let note_y = staff.y_of(12);
        assert!(
            layout.y < note_y,
            "ornament y ({}) should be above note y ({})",
            layout.y,
            note_y
        );
    }

    #[test]
    fn high_note_ornament_higher_than_middle_note_ornament() {
        let staff = test_staff();
        let layout_mid = layout_ornament(Ornament::Trill, 100.0, 4, &staff);
        let layout_high = layout_ornament(Ornament::Trill, 100.0, 12, &staff);
        assert!(
            layout_high.y < layout_mid.y,
            "ornament on high note ({}) should be higher (lower y) than on middle note ({})",
            layout_high.y,
            layout_mid.y
        );
    }

    #[test]
    fn layout_for_low_note_stays_above_staff() {
        let staff = test_staff();
        // Note on bottom line — ornament should still be above the staff
        let layout = layout_ornament(Ornament::Mordent, 100.0, 0, &staff);
        let top_line_y = staff.y_of(8);
        assert!(
            layout.y < top_line_y,
            "ornament should be above top staff line even for low note: {} < {}",
            layout.y,
            top_line_y
        );
    }

    #[test]
    fn x_is_preserved() {
        let staff = test_staff();
        let layout = layout_ornament(Ornament::Turn, 500.0, 6, &staff);
        assert_eq!(layout.x, 500.0);
    }

    #[test]
    fn different_ornaments_at_same_position_share_y() {
        let staff = test_staff();
        let trill = layout_ornament(Ornament::Trill, 100.0, 4, &staff);
        let mordent = layout_ornament(Ornament::Mordent, 100.0, 4, &staff);
        assert!(
            (trill.y - mordent.y).abs() < f64::EPSILON,
            "same position should yield same y: {} vs {}",
            trill.y,
            mordent.y
        );
    }
}
