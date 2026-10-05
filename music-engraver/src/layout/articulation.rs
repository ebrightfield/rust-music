use smufl::Glyph;

use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;

/// Standard articulation markings that attach close to the notehead.
///
/// Articulations are placed either above or below the note depending on stem
/// direction (opposite side from the stem, except fermata which is always above).
/// Each variant maps to a pair of SMuFL glyphs (above/below variants).
///
/// The fermata family covers the standard semicircle plus the duration-coded
/// variants common in 20th-century scores: long (square) and short (triangle)
/// fermatas indicate longer/shorter pauses, "very long" and "very short" push
/// even further, and the Henze pair (named after Hans Werner Henze) is a
/// commonly-used alternative notation for long/short with bracket-style
/// shapes. All fermata variants behave identically with respect to placement
/// (always above) and stacking — they only differ in glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Articulation {
    /// Staccato — a dot, shortens the note to roughly half its value.
    Staccato,
    /// Tenuto — a horizontal line, sustain for full value.
    Tenuto,
    /// Accent — a > mark, emphatic attack.
    Accent,
    /// Marcato — a ^ mark, strong accent with sharp attack.
    Marcato,
    /// Staccatissimo — a wedge/stroke, very short.
    Staccatissimo,
    /// Fermata — a pause; conventionally placed above the staff regardless
    /// of stem direction.
    Fermata,
    /// Long fermata — square-shaped fermata indicating a held pause longer
    /// than the standard fermata.
    FermataLong,
    /// Short fermata — triangle-shaped fermata indicating a held pause shorter
    /// than the standard fermata.
    FermataShort,
    /// Very long fermata — even longer than the long fermata; rare but
    /// occasionally used in 20th-century scores.
    FermataVeryLong,
    /// Very short fermata — even shorter than the short fermata; rare but
    /// occasionally used in 20th-century scores.
    FermataVeryShort,
    /// Henze long fermata — bracket-style alternative to `FermataLong`,
    /// named after Hans Werner Henze.
    FermataHenzeLong,
    /// Henze short fermata — bracket-style alternative to `FermataShort`,
    /// named after Hans Werner Henze.
    FermataHenzeShort,
    /// Up bow — string articulation indicating the bow moves upward (from
    /// frog toward tip). SMuFL provides a single glyph; conventionally
    /// placed above the staff regardless of stem direction.
    UpBow,
    /// Down bow — string articulation indicating the bow moves downward
    /// (from tip toward frog). SMuFL provides a single glyph; conventionally
    /// placed above the staff regardless of stem direction.
    DownBow,
    /// Accent + staccato — a single SMuFL glyph combining an accent mark
    /// and a staccato dot. Commonly used to mark a short, emphasized
    /// attack without taking up two stack slots. Behaves like a standard
    /// articulation: placed opposite the stem.
    AccentStaccato,
    /// Marcato + staccato — combined sharp accent and staccato dot. Rarer
    /// than `AccentStaccato` but used for the sharpest possible short
    /// attack. Opposite-side-of-stem placement.
    MarcatoStaccato,
    /// Tenuto + staccato — also called *portato* or *mezzo-staccato*:
    /// a tenuto line over a staccato dot. Indicates a slightly detached
    /// but sustained articulation. Opposite-side-of-stem placement.
    TenutoStaccato,
    /// Tenuto + accent — combined sustaining line and emphatic attack.
    /// Frequent in 19th–20th-century scores for "leaning into" a sustained
    /// note. Opposite-side-of-stem placement.
    TenutoAccent,
    /// Soft accent — SMuFL `articSoftAccent` (a parenthesized accent).
    /// Indicates a gentle emphasis, less prominent than a plain accent;
    /// used in 20th-century and contemporary scores. Follows the standard
    /// "opposite from stem" placement rule and lives in the normal
    /// articulation stack bucket.
    SoftAccent,
    /// Stress — SMuFL `articStress` (sometimes written as a small "u").
    /// Used in prosodic and contemporary notation to indicate a stressed
    /// note, between an unmarked attack and a full accent. Standard
    /// opposite-stem placement.
    Stress,
    /// Unstress — SMuFL `articUnstress` (a small inverted "u").
    /// The prosodic counterpart to `Stress`: marks a deliberately
    /// de-emphasized note. Standard opposite-stem placement.
    Unstress,
    /// Laissez vibrer — SMuFL `articLaissezVibrer` ("l.v."): a short
    /// tie-like curve attached to a notehead, instructing the player to
    /// let the note ring (decay naturally without damping). Standard
    /// in piano, harp, vibraphone, percussion, and arco-string scores.
    /// Bravura ships a true above/below pair; placement follows the
    /// standard opposite-stem rule (above-glyph for stem-down notes,
    /// below-glyph for stem-up notes), and the variant lives in the
    /// normal articulation stack bucket alongside Staccato/Accent/etc.
    LaissezVibrer,
}

/// Whether an articulation appears above or below the notehead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArticulationPlacement {
    Above,
    Below,
}

/// What an articulation-like mark draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArticulationKind {
    /// A built-in articulation: its SMuFL above/below glyph pair and
    /// conventional placement.
    Standard(Articulation),
    /// A caller-chosen SMuFL glyph for a mark the built-ins do not cover (an
    /// unidentified printed sign, a house style). The same glyph is drawn on
    /// either side; without an explicit placement it goes opposite the stem
    /// and stacks like an ordinary articulation.
    Custom(Glyph),
    /// Source-derived broad mark: a long horizontal bar with a short thick
    /// block under its centre (`mn-c12-r009`'s `\broadMark` markup).
    BroadMark,
}

/// One articulation-like mark on a note or chord: what it draws, on which
/// side, and whether it is parenthesized.
///
/// Marks on one side of the note stack outward in this order: ordinary
/// articulations and custom marks (in the order given), then bow strokes, then
/// fermatas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArticulationMark {
    /// The glyph source.
    pub kind: ArticulationKind,
    /// Forced side (LilyPond `^` / `_`); `None` keeps the conventional side.
    pub placement: Option<ArticulationPlacement>,
    /// Whether the mark is enclosed in parentheses (e.g. a parenthesized
    /// fermata).
    pub parenthesized: bool,
}

impl From<Articulation> for ArticulationMark {
    fn from(articulation: Articulation) -> Self {
        Self {
            kind: ArticulationKind::Standard(articulation),
            placement: None,
            parenthesized: false,
        }
    }
}

impl ArticulationMark {
    /// A custom mark drawing `glyph`, placed opposite the stem.
    pub fn custom(glyph: Glyph) -> Self {
        Self {
            kind: ArticulationKind::Custom(glyph),
            placement: None,
            parenthesized: false,
        }
    }

    /// The broad mark printed in mn-c12-r009: a thin horizontal line with
    /// a thick, centred block hanging below it, placed opposite the stem
    /// unless `.above()` / `.below()` is requested.
    pub fn broad_mark() -> Self {
        Self {
            kind: ArticulationKind::BroadMark,
            placement: None,
            parenthesized: false,
        }
    }

    /// Force the mark above the note.
    pub fn above(self) -> Self {
        Self {
            placement: Some(ArticulationPlacement::Above),
            ..self
        }
    }

    /// Force the mark below the note.
    pub fn below(self) -> Self {
        Self {
            placement: Some(ArticulationPlacement::Below),
            ..self
        }
    }

    /// Enclose the mark in parentheses.
    pub fn parenthesized(self) -> Self {
        Self {
            parenthesized: true,
            ..self
        }
    }

    /// The side the mark is engraved on for a note with stem `stem_dir`.
    pub fn resolved_placement(self, stem_dir: StemDirection) -> ArticulationPlacement {
        self.placement.unwrap_or(match self.kind {
            ArticulationKind::Standard(articulation) => articulation.default_placement(stem_dir),
            ArticulationKind::Custom(_) | ArticulationKind::BroadMark => match stem_dir {
                StemDirection::Up => ArticulationPlacement::Below,
                StemDirection::Down => ArticulationPlacement::Above,
            },
        })
    }

    /// The glyph drawn on `placement`'s side.
    pub fn glyph(self, placement: ArticulationPlacement) -> Glyph {
        match self.kind {
            ArticulationKind::Standard(articulation) => articulation.glyph(placement),
            ArticulationKind::Custom(glyph) => glyph,
            ArticulationKind::BroadMark => Glyph::ArticTenutoAbove,
        }
    }

    /// Stacking tier on its side: ordinary and custom marks (0) sit nearest
    /// the note, then bow strokes (1), then fermatas (2).
    fn stack_tier(self) -> u8 {
        match self.kind {
            ArticulationKind::Standard(articulation) if articulation.is_fermata() => 2,
            ArticulationKind::Standard(articulation) if articulation.is_bow_stroke() => 1,
            _ => 0,
        }
    }
}

impl Articulation {
    /// Return the SMuFL glyph for this articulation and placement.
    pub fn glyph(self, placement: ArticulationPlacement) -> Glyph {
        match (self, placement) {
            (Self::Staccato, ArticulationPlacement::Above) => Glyph::ArticStaccatoAbove,
            (Self::Staccato, ArticulationPlacement::Below) => Glyph::ArticStaccatoBelow,
            (Self::Tenuto, ArticulationPlacement::Above) => Glyph::ArticTenutoAbove,
            (Self::Tenuto, ArticulationPlacement::Below) => Glyph::ArticTenutoBelow,
            (Self::Accent, ArticulationPlacement::Above) => Glyph::ArticAccentAbove,
            (Self::Accent, ArticulationPlacement::Below) => Glyph::ArticAccentBelow,
            (Self::Marcato, ArticulationPlacement::Above) => Glyph::ArticMarcatoAbove,
            (Self::Marcato, ArticulationPlacement::Below) => Glyph::ArticMarcatoBelow,
            (Self::Staccatissimo, ArticulationPlacement::Above) => Glyph::ArticStaccatissimoAbove,
            (Self::Staccatissimo, ArticulationPlacement::Below) => Glyph::ArticStaccatissimoBelow,
            (Self::Fermata, ArticulationPlacement::Above) => Glyph::FermataAbove,
            (Self::Fermata, ArticulationPlacement::Below) => Glyph::FermataBelow,
            (Self::FermataLong, ArticulationPlacement::Above) => Glyph::FermataLongAbove,
            (Self::FermataLong, ArticulationPlacement::Below) => Glyph::FermataLongBelow,
            (Self::FermataShort, ArticulationPlacement::Above) => Glyph::FermataShortAbove,
            (Self::FermataShort, ArticulationPlacement::Below) => Glyph::FermataShortBelow,
            (Self::FermataVeryLong, ArticulationPlacement::Above) => Glyph::FermataVeryLongAbove,
            (Self::FermataVeryLong, ArticulationPlacement::Below) => Glyph::FermataVeryLongBelow,
            (Self::FermataVeryShort, ArticulationPlacement::Above) => Glyph::FermataVeryShortAbove,
            (Self::FermataVeryShort, ArticulationPlacement::Below) => Glyph::FermataVeryShortBelow,
            (Self::FermataHenzeLong, ArticulationPlacement::Above) => Glyph::FermataLongHenzeAbove,
            (Self::FermataHenzeLong, ArticulationPlacement::Below) => Glyph::FermataLongHenzeBelow,
            (Self::FermataHenzeShort, ArticulationPlacement::Above) => {
                Glyph::FermataShortHenzeAbove
            }
            (Self::FermataHenzeShort, ArticulationPlacement::Below) => {
                Glyph::FermataShortHenzeBelow
            }
            // SMuFL ships a single glyph for each bow stroke (no above/below
            // pair). Bow strokes are conventionally placed above the staff;
            // the placement arg is accepted for API uniformity but does not
            // change the glyph.
            (Self::UpBow, _) => Glyph::StringsUpBow,
            (Self::DownBow, _) => Glyph::StringsDownBow,
            (Self::AccentStaccato, ArticulationPlacement::Above) => Glyph::ArticAccentStaccatoAbove,
            (Self::AccentStaccato, ArticulationPlacement::Below) => Glyph::ArticAccentStaccatoBelow,
            (Self::MarcatoStaccato, ArticulationPlacement::Above) => {
                Glyph::ArticMarcatoStaccatoAbove
            }
            (Self::MarcatoStaccato, ArticulationPlacement::Below) => {
                Glyph::ArticMarcatoStaccatoBelow
            }
            (Self::TenutoStaccato, ArticulationPlacement::Above) => Glyph::ArticTenutoStaccatoAbove,
            (Self::TenutoStaccato, ArticulationPlacement::Below) => Glyph::ArticTenutoStaccatoBelow,
            (Self::TenutoAccent, ArticulationPlacement::Above) => Glyph::ArticTenutoAccentAbove,
            (Self::TenutoAccent, ArticulationPlacement::Below) => Glyph::ArticTenutoAccentBelow,
            (Self::SoftAccent, ArticulationPlacement::Above) => Glyph::ArticSoftAccentAbove,
            (Self::SoftAccent, ArticulationPlacement::Below) => Glyph::ArticSoftAccentBelow,
            (Self::Stress, ArticulationPlacement::Above) => Glyph::ArticStressAbove,
            (Self::Stress, ArticulationPlacement::Below) => Glyph::ArticStressBelow,
            (Self::Unstress, ArticulationPlacement::Above) => Glyph::ArticUnstressAbove,
            (Self::Unstress, ArticulationPlacement::Below) => Glyph::ArticUnstressBelow,
            (Self::LaissezVibrer, ArticulationPlacement::Above) => Glyph::ArticLaissezVibrerAbove,
            (Self::LaissezVibrer, ArticulationPlacement::Below) => Glyph::ArticLaissezVibrerBelow,
        }
    }

    /// Whether this articulation is a member of the fermata family. All
    /// fermata variants share placement and stacking rules (always above,
    /// always after normal articulations in a stack); this predicate keeps
    /// those rules consistent across the codebase.
    pub fn is_fermata(self) -> bool {
        matches!(
            self,
            Self::Fermata
                | Self::FermataLong
                | Self::FermataShort
                | Self::FermataVeryLong
                | Self::FermataVeryShort
                | Self::FermataHenzeLong
                | Self::FermataHenzeShort
        )
    }

    /// Whether this articulation is a bow stroke (up-bow or down-bow). Bow
    /// strokes share placement rules (always above) and stack between normal
    /// articulations and fermatas — they're closer to the note than fermata
    /// but on the bow-marking-conventional "above" side.
    pub fn is_bow_stroke(self) -> bool {
        matches!(self, Self::UpBow | Self::DownBow)
    }

    /// Default placement relative to stem direction.
    ///
    /// Convention: articulations go on the opposite side from the stem.
    /// Exceptions:
    /// - Fermata (and all its duration variants) is always placed above.
    /// - Bow strokes (up-bow, down-bow) are always placed above per the
    ///   Gould convention for string-articulation markings.
    pub fn default_placement(self, stem_dir: StemDirection) -> ArticulationPlacement {
        if self.is_fermata() || self.is_bow_stroke() {
            return ArticulationPlacement::Above;
        }
        match stem_dir {
            StemDirection::Up => ArticulationPlacement::Below,
            StemDirection::Down => ArticulationPlacement::Above,
        }
    }
}

/// Offset (in staff spaces) from the notehead center to the articulation glyph
/// anchor. This is the minimum clearance between the notehead and the
/// articulation.
const ARTICULATION_OFFSET_SS: f64 = 0.5;

/// Vertical spacing (in staff spaces) between stacked articulations.
/// Each successive articulation in a stack is placed this far from the
/// previous one, moving away from the notehead.
const ARTICULATION_STACK_SPACING_SS: f64 = 0.6;

/// Computed articulation position.
#[derive(Clone, Debug)]
pub struct ArticulationLayout {
    /// X-coordinate (centered on notehead).
    pub x: f64,
    /// Y-coordinate of the articulation glyph anchor.
    pub y: f64,
    /// The source kind; [`ArticulationKind::BroadMark`] adds a centred
    /// block to the stretched tenuto glyph.
    pub kind: ArticulationKind,
    /// The SMuFL glyph that supplies the mark's outline.
    pub glyph: Glyph,
    /// Whether the articulation is placed above or below.
    pub placement: ArticulationPlacement,
    /// Whether the glyph is enclosed in parentheses.
    pub parenthesized: bool,
}

/// Y of the first (innermost) mark on `placement`'s side of a note.
fn side_base_y(
    placement: ArticulationPlacement,
    note_staff_position: i8,
    staff: &StaffLayout,
) -> f64 {
    let offset_fu = ARTICULATION_OFFSET_SS * staff.staff_space;

    // Position the articulation just outside the notehead.
    // For "above" placement: move up (lower y) from the note.
    // For "below" placement: move down (higher y) from the note.
    //
    // If the note is on a line, add extra clearance so the articulation
    // doesn't collide with the staff line.
    let note_y = staff.y_of(note_staff_position);
    let on_line = StaffLayout::is_on_line(note_staff_position);
    let extra_clearance = if on_line { offset_fu * 0.5 } else { 0.0 };

    match placement {
        ArticulationPlacement::Above => {
            // Ensure we don't place inside the staff for notes below the top line.
            let min_y = staff.y_of(8) - offset_fu;
            let natural_y = note_y - offset_fu - extra_clearance;
            natural_y.min(min_y)
        }
        ArticulationPlacement::Below => {
            // Ensure we don't place inside the staff for notes above the bottom line.
            let max_y = staff.y_of(0) + offset_fu;
            let natural_y = note_y + offset_fu + extra_clearance;
            natural_y.max(max_y)
        }
    }
}

/// Compute the position for an articulation-like mark relative to a note.
///
/// `notehead_x` is the x-center of the notehead. `note_staff_position` is the
/// staff position of the note (0 = bottom line). `stem_dir` determines default
/// placement (articulation goes opposite the stem; fermata always above)
/// unless the mark forces a side. `staff` provides the coordinate mapping.
pub fn layout_articulation(
    mark: impl Into<ArticulationMark>,
    notehead_x: f64,
    note_staff_position: i8,
    stem_dir: StemDirection,
    staff: &StaffLayout,
) -> ArticulationLayout {
    let mark = mark.into();
    let placement = mark.resolved_placement(stem_dir);
    ArticulationLayout {
        x: notehead_x,
        y: side_base_y(placement, note_staff_position, staff),
        glyph: mark.glyph(placement),
        kind: mark.kind,
        placement,
        parenthesized: mark.parenthesized,
    }
}

/// Compute positions for multiple stacked marks on a single note.
///
/// Each mark takes its own side (forced, or conventional: opposite the stem;
/// fermatas and bow strokes above). On each side marks stack outward from
/// the notehead by tier — ordinary articulations and custom marks first (in
/// the order given), then bow strokes (Gould: closer to the note than the
/// fermata), then fermatas. The result lists marks in that tier order.
pub fn layout_articulation_stack(
    marks: &[ArticulationMark],
    notehead_x: f64,
    note_staff_position: i8,
    stem_dir: StemDirection,
    staff: &StaffLayout,
) -> Vec<ArticulationLayout> {
    layout_chord_articulation_stack(
        marks,
        notehead_x,
        (note_staff_position, note_staff_position),
        stem_dir,
        staff,
    )
}

/// [`layout_articulation_stack`] for a chord spanning `(lowest, highest)`
/// staff positions: marks above stack from the highest notehead, marks below
/// from the lowest.
pub fn layout_chord_articulation_stack(
    marks: &[ArticulationMark],
    notehead_x: f64,
    (lowest, highest): (i8, i8),
    stem_dir: StemDirection,
    staff: &StaffLayout,
) -> Vec<ArticulationLayout> {
    let mut ordered: Vec<ArticulationMark> = marks.to_vec();
    ordered.sort_by_key(|mark| mark.stack_tier());

    let stack_spacing = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
    let mut above = 0_u32;
    let mut below = 0_u32;
    ordered
        .into_iter()
        .map(|mark| {
            let attach = match mark.resolved_placement(stem_dir) {
                ArticulationPlacement::Above => highest,
                ArticulationPlacement::Below => lowest,
            };
            let mut layout = layout_articulation(mark, notehead_x, attach, stem_dir, staff);
            let depth = match layout.placement {
                ArticulationPlacement::Above => &mut above,
                ArticulationPlacement::Below => &mut below,
            };
            let offset = f64::from(*depth) * stack_spacing;
            layout.y = match layout.placement {
                ArticulationPlacement::Above => layout.y - offset,
                ArticulationPlacement::Below => layout.y + offset,
            };
            *depth += 1;
            layout
        })
        .collect()
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
    fn staccato_above_glyph() {
        assert_eq!(
            Articulation::Staccato.glyph(ArticulationPlacement::Above),
            Glyph::ArticStaccatoAbove
        );
    }

    #[test]
    fn staccato_below_glyph() {
        assert_eq!(
            Articulation::Staccato.glyph(ArticulationPlacement::Below),
            Glyph::ArticStaccatoBelow
        );
    }

    #[test]
    fn tenuto_above_glyph() {
        assert_eq!(
            Articulation::Tenuto.glyph(ArticulationPlacement::Above),
            Glyph::ArticTenutoAbove
        );
    }

    #[test]
    fn accent_below_glyph() {
        assert_eq!(
            Articulation::Accent.glyph(ArticulationPlacement::Below),
            Glyph::ArticAccentBelow
        );
    }

    #[test]
    fn marcato_above_glyph() {
        assert_eq!(
            Articulation::Marcato.glyph(ArticulationPlacement::Above),
            Glyph::ArticMarcatoAbove
        );
    }

    #[test]
    fn staccatissimo_below_glyph() {
        assert_eq!(
            Articulation::Staccatissimo.glyph(ArticulationPlacement::Below),
            Glyph::ArticStaccatissimoBelow
        );
    }

    #[test]
    fn fermata_above_glyph() {
        assert_eq!(
            Articulation::Fermata.glyph(ArticulationPlacement::Above),
            Glyph::FermataAbove
        );
    }

    #[test]
    fn fermata_below_glyph() {
        assert_eq!(
            Articulation::Fermata.glyph(ArticulationPlacement::Below),
            Glyph::FermataBelow
        );
    }

    #[test]
    fn default_placement_stem_up_is_below() {
        assert_eq!(
            Articulation::Staccato.default_placement(StemDirection::Up),
            ArticulationPlacement::Below
        );
        assert_eq!(
            Articulation::Accent.default_placement(StemDirection::Up),
            ArticulationPlacement::Below
        );
    }

    #[test]
    fn default_placement_stem_down_is_above() {
        assert_eq!(
            Articulation::Staccato.default_placement(StemDirection::Down),
            ArticulationPlacement::Above
        );
        assert_eq!(
            Articulation::Tenuto.default_placement(StemDirection::Down),
            ArticulationPlacement::Above
        );
    }

    #[test]
    fn fermata_always_above_regardless_of_stem() {
        assert_eq!(
            Articulation::Fermata.default_placement(StemDirection::Up),
            ArticulationPlacement::Above
        );
        assert_eq!(
            Articulation::Fermata.default_placement(StemDirection::Down),
            ArticulationPlacement::Above
        );
    }

    #[test]
    fn all_six_articulations_produce_distinct_above_glyphs() {
        let all = [
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
            Articulation::Fermata,
        ];
        let glyphs: Vec<Glyph> = all
            .iter()
            .map(|a| a.glyph(ArticulationPlacement::Above))
            .collect();
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(g1, g2, "articulations {i} and {j} should differ");
                }
            }
        }
    }

    // --- Fermata duration-variant tests ---

    /// Convenience: the 7 fermata variants in canonical order.
    const FERMATA_VARIANTS: [Articulation; 7] = [
        Articulation::Fermata,
        Articulation::FermataLong,
        Articulation::FermataShort,
        Articulation::FermataVeryLong,
        Articulation::FermataVeryShort,
        Articulation::FermataHenzeLong,
        Articulation::FermataHenzeShort,
    ];

    #[test]
    fn fermata_long_glyph_pair() {
        assert_eq!(
            Articulation::FermataLong.glyph(ArticulationPlacement::Above),
            Glyph::FermataLongAbove
        );
        assert_eq!(
            Articulation::FermataLong.glyph(ArticulationPlacement::Below),
            Glyph::FermataLongBelow
        );
    }

    #[test]
    fn fermata_short_glyph_pair() {
        assert_eq!(
            Articulation::FermataShort.glyph(ArticulationPlacement::Above),
            Glyph::FermataShortAbove
        );
        assert_eq!(
            Articulation::FermataShort.glyph(ArticulationPlacement::Below),
            Glyph::FermataShortBelow
        );
    }

    #[test]
    fn fermata_very_long_glyph_pair() {
        assert_eq!(
            Articulation::FermataVeryLong.glyph(ArticulationPlacement::Above),
            Glyph::FermataVeryLongAbove
        );
        assert_eq!(
            Articulation::FermataVeryLong.glyph(ArticulationPlacement::Below),
            Glyph::FermataVeryLongBelow
        );
    }

    #[test]
    fn fermata_very_short_glyph_pair() {
        assert_eq!(
            Articulation::FermataVeryShort.glyph(ArticulationPlacement::Above),
            Glyph::FermataVeryShortAbove
        );
        assert_eq!(
            Articulation::FermataVeryShort.glyph(ArticulationPlacement::Below),
            Glyph::FermataVeryShortBelow
        );
    }

    #[test]
    fn fermata_henze_long_glyph_pair() {
        assert_eq!(
            Articulation::FermataHenzeLong.glyph(ArticulationPlacement::Above),
            Glyph::FermataLongHenzeAbove
        );
        assert_eq!(
            Articulation::FermataHenzeLong.glyph(ArticulationPlacement::Below),
            Glyph::FermataLongHenzeBelow
        );
    }

    #[test]
    fn fermata_henze_short_glyph_pair() {
        assert_eq!(
            Articulation::FermataHenzeShort.glyph(ArticulationPlacement::Above),
            Glyph::FermataShortHenzeAbove
        );
        assert_eq!(
            Articulation::FermataHenzeShort.glyph(ArticulationPlacement::Below),
            Glyph::FermataShortHenzeBelow
        );
    }

    #[test]
    fn all_fermata_variants_recognized_by_is_fermata() {
        for &f in &FERMATA_VARIANTS {
            assert!(f.is_fermata(), "{f:?} should be recognized as fermata");
        }
    }

    #[test]
    fn non_fermata_articulations_not_flagged_by_is_fermata() {
        for &a in &[
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
        ] {
            assert!(!a.is_fermata(), "{a:?} should not be flagged as fermata");
        }
    }

    #[test]
    fn all_fermata_variants_default_to_above() {
        for &f in &FERMATA_VARIANTS {
            assert_eq!(
                f.default_placement(StemDirection::Up),
                ArticulationPlacement::Above,
                "{f:?} should default to Above (stem up)",
            );
            assert_eq!(
                f.default_placement(StemDirection::Down),
                ArticulationPlacement::Above,
                "{f:?} should default to Above (stem down)",
            );
        }
    }

    #[test]
    fn all_fermata_variants_produce_distinct_above_glyphs() {
        // The whole point of the duration-coded family: each variant must
        // render as a visibly distinct glyph, otherwise users can't
        // distinguish long/short/very-long etc. from the plain fermata.
        let glyphs: Vec<Glyph> = FERMATA_VARIANTS
            .iter()
            .map(|f| f.glyph(ArticulationPlacement::Above))
            .collect();
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(g1, g2, "fermata variants {i} and {j} share glyph: {g1:?}",);
                }
            }
        }
    }

    #[test]
    fn all_fermata_variants_produce_distinct_below_glyphs() {
        let glyphs: Vec<Glyph> = FERMATA_VARIANTS
            .iter()
            .map(|f| f.glyph(ArticulationPlacement::Below))
            .collect();
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(g1, g2, "fermata variants {i} and {j} share below-glyph",);
                }
            }
        }
    }

    #[test]
    fn fermata_variants_above_below_differ_within_each() {
        // Each fermata variant must have a distinct above- vs below-glyph
        // pair — Bravura provides separate glyphs because they aren't
        // mirror-images of each other.
        for &f in &FERMATA_VARIANTS {
            assert_ne!(
                f.glyph(ArticulationPlacement::Above),
                f.glyph(ArticulationPlacement::Below),
                "{f:?} above and below glyphs should differ",
            );
        }
    }

    #[test]
    fn fermata_variants_share_y_when_layout_alone() {
        // All fermata variants share placement rules — only the glyph differs.
        // Laying out each variant on the same note position should yield
        // identical x/y/placement but distinct glyphs.
        let staff = test_staff();
        let layouts: Vec<ArticulationLayout> = FERMATA_VARIANTS
            .iter()
            .map(|&f| layout_articulation(f, 200.0, 4, StemDirection::Up, &staff))
            .collect();
        let first_y = layouts[0].y;
        let first_x = layouts[0].x;
        for (i, l) in layouts.iter().enumerate() {
            assert!(
                (l.y - first_y).abs() < 1e-9,
                "variant {i} y={} differs from first y={first_y}",
                l.y,
            );
            assert_eq!(l.x, first_x, "variant {i} x differs");
            assert_eq!(l.placement, ArticulationPlacement::Above);
        }
        // Glyphs must still be distinct between variants.
        for (i, a) in layouts.iter().enumerate() {
            for (j, b) in layouts.iter().enumerate() {
                if i != j {
                    assert_ne!(a.glyph, b.glyph);
                }
            }
        }
    }

    #[test]
    fn stack_long_fermata_with_staccato_separates_placement() {
        // Regression: the stack splitter must recognize FermataLong (and the
        // other variants) as "always above," same as the plain Fermata.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::Staccato.into(),
                Articulation::FermataLong.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::FermataLongAbove);
        // The fermata-long should sit higher (lower y) than the below
        // articulation — i.e. on the opposite side of the staff.
        assert!(stack[1].y < stack[0].y);
    }

    #[test]
    fn stack_multiple_fermata_variants_all_above() {
        // Stacking two different fermata variants should put both above,
        // separated vertically by the standard stack spacing.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::FermataShort.into(),
                Articulation::FermataLong.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        for l in &stack {
            assert_eq!(l.placement, ArticulationPlacement::Above);
        }
        // Second fermata should stack further from the note (lower y for Above).
        assert!(stack[1].y < stack[0].y);
        // Spacing equals ARTICULATION_STACK_SPACING_SS × staff_space.
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let actual_gap = stack[0].y - stack[1].y;
        assert!(
            (actual_gap - expected_gap).abs() < 1e-6,
            "fermata stack spacing {actual_gap} should match expected {expected_gap}",
        );
        // Glyphs preserved per variant.
        assert_eq!(stack[0].glyph, Glyph::FermataShortAbove);
        assert_eq!(stack[1].glyph, Glyph::FermataLongAbove);
    }

    #[test]
    fn layout_staccato_below_stem_up_note_on_line() {
        let staff = test_staff();
        let layout =
            layout_articulation(Articulation::Staccato, 100.0, 4, StemDirection::Up, &staff);
        assert_eq!(layout.placement, ArticulationPlacement::Below);
        assert_eq!(layout.x, 100.0);
        // Note on middle line (pos 4), below placement → y > note_y
        let note_y = staff.y_of(4);
        assert!(
            layout.y > note_y,
            "below articulation should be below note: {} > {}",
            layout.y,
            note_y
        );
    }

    #[test]
    fn layout_accent_above_stem_down_note_in_space() {
        let staff = test_staff();
        let layout =
            layout_articulation(Articulation::Accent, 200.0, 3, StemDirection::Down, &staff);
        assert_eq!(layout.placement, ArticulationPlacement::Above);
        // Note in space (pos 3), above placement → y < note_y
        let note_y = staff.y_of(3);
        assert!(
            layout.y < note_y,
            "above articulation should be above note: {} < {}",
            layout.y,
            note_y
        );
    }

    #[test]
    fn layout_articulation_not_inside_staff() {
        let staff = test_staff();
        // Note on bottom line (pos 0), stem up → articulation below
        let layout =
            layout_articulation(Articulation::Staccato, 100.0, 0, StemDirection::Up, &staff);
        let bottom_y = staff.y_of(0);
        assert!(
            layout.y >= bottom_y,
            "below articulation on bottom line should be at or below bottom: {} >= {}",
            layout.y,
            bottom_y
        );
    }

    #[test]
    fn layout_fermata_above_regardless_of_stem_up() {
        let staff = test_staff();
        let layout =
            layout_articulation(Articulation::Fermata, 100.0, 4, StemDirection::Up, &staff);
        assert_eq!(layout.placement, ArticulationPlacement::Above);
        assert_eq!(layout.glyph, Glyph::FermataAbove);
    }

    #[test]
    fn on_line_note_gets_extra_clearance() {
        let staff = test_staff();
        // Pos 4 is on a line, pos 3 is in a space
        let on_line =
            layout_articulation(Articulation::Staccato, 100.0, 4, StemDirection::Up, &staff);
        let in_space =
            layout_articulation(Articulation::Staccato, 100.0, 3, StemDirection::Up, &staff);
        // Both are below placement; on-line should have more clearance
        let note_y_line = staff.y_of(4);
        let note_y_space = staff.y_of(3);
        let clearance_line = on_line.y - note_y_line;
        let clearance_space = in_space.y - note_y_space;
        assert!(
            clearance_line > clearance_space,
            "on-line clearance ({}) should exceed in-space ({})",
            clearance_line,
            clearance_space
        );
    }

    // --- Stacked articulation tests ---

    #[test]
    fn stack_empty_returns_empty() {
        let staff = test_staff();
        let result = layout_articulation_stack(&[], 100.0, 4, StemDirection::Up, &staff);
        assert!(result.is_empty());
    }

    #[test]
    fn stack_single_matches_layout_articulation() {
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        let single =
            layout_articulation(Articulation::Staccato, 100.0, 4, StemDirection::Up, &staff);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].x, single.x);
        assert!((stack[0].y - single.y).abs() < 0.001);
        assert_eq!(stack[0].glyph, single.glyph);
        assert_eq!(stack[0].placement, single.placement);
    }

    #[test]
    fn stack_two_below_second_further_from_note() {
        let staff = test_staff();
        // Stem up → articulations below → higher y = further from note
        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into(), Articulation::Accent.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert!(
            stack[1].y > stack[0].y,
            "second below-articulation should be further from note: {} > {}",
            stack[1].y,
            stack[0].y
        );
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::ArticAccentBelow);
    }

    #[test]
    fn stack_two_above_second_further_from_note() {
        let staff = test_staff();
        // Stem down → articulations above → lower y = further from note
        let stack = layout_articulation_stack(
            &[Articulation::Tenuto.into(), Articulation::Marcato.into()],
            100.0,
            4,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert!(
            stack[1].y < stack[0].y,
            "second above-articulation should be further from note: {} < {}",
            stack[1].y,
            stack[0].y
        );
        assert_eq!(stack[0].glyph, Glyph::ArticTenutoAbove);
        assert_eq!(stack[1].glyph, Glyph::ArticMarcatoAbove);
    }

    #[test]
    fn stack_fermata_with_staccato_separates_placement() {
        let staff = test_staff();
        // Stem up: staccato goes below, fermata goes above
        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into(), Articulation::Fermata.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::FermataAbove);
    }

    #[test]
    fn stack_fermata_above_staccato_below_non_overlapping() {
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into(), Articulation::Fermata.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        // Fermata (above) should have lower y than staccato (below)
        assert!(
            stack[1].y < stack[0].y,
            "fermata above ({}) should have lower y than staccato below ({})",
            stack[1].y,
            stack[0].y
        );
    }

    #[test]
    fn stack_three_articulations_all_spaced_apart() {
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::Staccato.into(),
                Articulation::Accent.into(),
                Articulation::Tenuto.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 3);
        // All below (stem up), each further from note
        assert!(stack[1].y > stack[0].y);
        assert!(stack[2].y > stack[1].y);
        // Spacing should be consistent
        let gap1 = stack[1].y - stack[0].y;
        let gap2 = stack[2].y - stack[1].y;
        assert!(
            (gap1 - gap2).abs() < 0.001,
            "spacing between stacked articulations should be uniform: {} vs {}",
            gap1,
            gap2
        );
    }

    #[test]
    fn stack_all_share_same_x() {
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::Staccato.into(),
                Articulation::Accent.into(),
                Articulation::Fermata.into(),
            ],
            250.0,
            4,
            StemDirection::Up,
            &staff,
        );
        for layout in &stack {
            assert_eq!(layout.x, 250.0, "all stacked articulations should share x");
        }
    }

    #[test]
    fn stack_fermata_only_behaves_like_single() {
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Fermata.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        let single =
            layout_articulation(Articulation::Fermata, 100.0, 4, StemDirection::Up, &staff);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].placement, ArticulationPlacement::Above);
        assert!((stack[0].y - single.y).abs() < 0.001);
    }

    // --- Bow stroke tests ---

    #[test]
    fn up_bow_glyph_is_strings_up_bow() {
        // SMuFL has only one glyph for each bow stroke; the placement argument
        // is accepted for API uniformity but does not change the glyph.
        assert_eq!(
            Articulation::UpBow.glyph(ArticulationPlacement::Above),
            Glyph::StringsUpBow
        );
        assert_eq!(
            Articulation::UpBow.glyph(ArticulationPlacement::Below),
            Glyph::StringsUpBow
        );
    }

    #[test]
    fn down_bow_glyph_is_strings_down_bow() {
        assert_eq!(
            Articulation::DownBow.glyph(ArticulationPlacement::Above),
            Glyph::StringsDownBow
        );
        assert_eq!(
            Articulation::DownBow.glyph(ArticulationPlacement::Below),
            Glyph::StringsDownBow
        );
    }

    #[test]
    fn up_bow_and_down_bow_use_distinct_glyphs() {
        // Regression: a typo would otherwise let both variants share the same
        // glyph and collapse the up/down distinction visually.
        assert_ne!(
            Articulation::UpBow.glyph(ArticulationPlacement::Above),
            Articulation::DownBow.glyph(ArticulationPlacement::Above)
        );
    }

    #[test]
    fn bow_strokes_recognized_by_is_bow_stroke() {
        assert!(Articulation::UpBow.is_bow_stroke());
        assert!(Articulation::DownBow.is_bow_stroke());
    }

    #[test]
    fn non_bow_articulations_not_flagged_by_is_bow_stroke() {
        for &a in &[
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
            Articulation::Fermata,
            Articulation::FermataLong,
        ] {
            assert!(
                !a.is_bow_stroke(),
                "{a:?} should not be flagged as bow stroke"
            );
        }
    }

    #[test]
    fn bow_strokes_not_flagged_by_is_fermata() {
        // Defensive: bow strokes must not collide with the fermata bucket.
        assert!(!Articulation::UpBow.is_fermata());
        assert!(!Articulation::DownBow.is_fermata());
    }

    #[test]
    fn bow_strokes_always_default_to_above() {
        for &b in &[Articulation::UpBow, Articulation::DownBow] {
            assert_eq!(
                b.default_placement(StemDirection::Up),
                ArticulationPlacement::Above,
                "{b:?} should default to Above with stem up"
            );
            assert_eq!(
                b.default_placement(StemDirection::Down),
                ArticulationPlacement::Above,
                "{b:?} should default to Above with stem down"
            );
        }
    }

    #[test]
    fn bow_stroke_alone_lays_out_above_note() {
        let staff = test_staff();
        // Stem-up, middle-line note: a normal articulation would go below.
        // A bow stroke must go above instead.
        let up_bow = layout_articulation(Articulation::UpBow, 100.0, 4, StemDirection::Up, &staff);
        assert_eq!(up_bow.placement, ArticulationPlacement::Above);
        assert_eq!(up_bow.glyph, Glyph::StringsUpBow);
        let note_y = staff.y_of(4);
        assert!(
            up_bow.y < note_y,
            "above-placement should have lower y than note: {} < {}",
            up_bow.y,
            note_y
        );
    }

    #[test]
    fn stack_bow_stroke_with_staccato_stem_up_separates_placement() {
        // Stem-up: staccato goes below, bow stroke goes above.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into(), Articulation::DownBow.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsDownBow);
        // The bow stroke (above) must sit higher than the staccato (below).
        assert!(stack[1].y < stack[0].y);
    }

    #[test]
    fn stack_bow_stroke_with_staccato_stem_down_stacks_outward() {
        // Stem-down: staccato goes above, bow stroke goes above above the
        // staccato (further from note).
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into(), Articulation::UpBow.into()],
            100.0,
            4,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Above);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoAbove);
        assert_eq!(stack[1].glyph, Glyph::StringsUpBow);
        // Bow stroke must sit further from note (lower y) than the staccato.
        assert!(stack[1].y < stack[0].y);
        // Spacing equals one ARTICULATION_STACK_SPACING_SS × staff_space.
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let actual_gap = stack[0].y - stack[1].y;
        assert!(
            (actual_gap - expected_gap).abs() < 1e-6,
            "bow-stacks-above-staccato gap {actual_gap} should match expected {expected_gap}"
        );
    }

    #[test]
    fn stack_bow_then_fermata_orders_fermata_outermost() {
        // Engraving convention: from notehead outward, normal articulations →
        // bow strokes → fermata. Locks that ordering.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::UpBow.into(), Articulation::Fermata.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].glyph, Glyph::StringsUpBow);
        assert_eq!(stack[1].glyph, Glyph::FermataAbove);
        // Both above. Fermata must sit further out (smaller y).
        assert_eq!(stack[0].placement, ArticulationPlacement::Above);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert!(stack[1].y < stack[0].y);
        // Spacing is one stack-step.
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let actual_gap = stack[0].y - stack[1].y;
        assert!(
            (actual_gap - expected_gap).abs() < 1e-6,
            "fermata-stacks-above-bow gap {actual_gap} should match expected {expected_gap}"
        );
    }

    #[test]
    fn stack_full_triple_stems_up_orders_correctly() {
        // Full three-tier stack: staccato (below), down-bow (above), fermata
        // (above, outermost). With stem up, the staccato lands below; the bow
        // and fermata both land above with the fermata further out.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::Staccato.into(),
                Articulation::DownBow.into(),
                Articulation::Fermata.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 3);
        // Order: input order preserved within each bucket, but buckets are
        // emitted normal → bow → fermata.
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsDownBow);
        assert_eq!(stack[2].glyph, Glyph::FermataAbove);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[2].placement, ArticulationPlacement::Above);
        // Bow above note (smaller y); fermata above bow.
        let note_y = staff.y_of(4);
        assert!(stack[0].y > note_y, "staccato below note");
        assert!(stack[1].y < note_y, "bow above note");
        assert!(stack[2].y < stack[1].y, "fermata above bow");
    }

    #[test]
    fn stack_full_triple_stems_down_orders_correctly() {
        // Stem-down: staccato goes above, bow stroke goes above-above, fermata
        // outermost. The three above-placement layouts must be in stacking
        // order: staccato closest to note, bow next, fermata furthest.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::Staccato.into(),
                Articulation::UpBow.into(),
                Articulation::FermataLong.into(),
            ],
            100.0,
            4,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(stack.len(), 3);
        for l in &stack {
            assert_eq!(l.placement, ArticulationPlacement::Above);
        }
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoAbove);
        assert_eq!(stack[1].glyph, Glyph::StringsUpBow);
        assert_eq!(stack[2].glyph, Glyph::FermataLongAbove);
        // Strict outward monotonic y.
        assert!(stack[1].y < stack[0].y, "bow above staccato");
        assert!(stack[2].y < stack[1].y, "fermata above bow");
        // Uniform spacing.
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let gap1 = stack[0].y - stack[1].y;
        let gap2 = stack[1].y - stack[2].y;
        assert!((gap1 - expected_gap).abs() < 1e-6);
        assert!((gap2 - expected_gap).abs() < 1e-6);
    }

    #[test]
    fn stack_two_bow_strokes_stacked_outward() {
        // Pathological but legal: two bow strokes on the same note (e.g.
        // multi-edition reconciliations). Both above, second further out.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::DownBow.into(), Articulation::UpBow.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].glyph, Glyph::StringsDownBow);
        assert_eq!(stack[1].glyph, Glyph::StringsUpBow);
        for l in &stack {
            assert_eq!(l.placement, ArticulationPlacement::Above);
        }
        assert!(stack[1].y < stack[0].y);
    }

    // --- Combined-articulation tests ---
    //
    // The four combined-articulation glyphs (AccentStaccato, MarcatoStaccato,
    // TenutoStaccato, TenutoAccent) are SMuFL shorthand for what would
    // otherwise be two stacked normal articulations. They behave exactly like
    // normal articulations: stem-opposite placement, single above/below glyph
    // pair, and they live in the "normal" stack bucket — not the always-above
    // bucket reserved for bow strokes and fermatas.

    /// Convenience: the 4 combined-articulation variants in canonical order.
    const COMBINED_VARIANTS: [Articulation; 4] = [
        Articulation::AccentStaccato,
        Articulation::MarcatoStaccato,
        Articulation::TenutoStaccato,
        Articulation::TenutoAccent,
    ];

    #[test]
    fn accent_staccato_glyph_pair() {
        assert_eq!(
            Articulation::AccentStaccato.glyph(ArticulationPlacement::Above),
            Glyph::ArticAccentStaccatoAbove
        );
        assert_eq!(
            Articulation::AccentStaccato.glyph(ArticulationPlacement::Below),
            Glyph::ArticAccentStaccatoBelow
        );
    }

    #[test]
    fn marcato_staccato_glyph_pair() {
        assert_eq!(
            Articulation::MarcatoStaccato.glyph(ArticulationPlacement::Above),
            Glyph::ArticMarcatoStaccatoAbove
        );
        assert_eq!(
            Articulation::MarcatoStaccato.glyph(ArticulationPlacement::Below),
            Glyph::ArticMarcatoStaccatoBelow
        );
    }

    #[test]
    fn tenuto_staccato_glyph_pair() {
        assert_eq!(
            Articulation::TenutoStaccato.glyph(ArticulationPlacement::Above),
            Glyph::ArticTenutoStaccatoAbove
        );
        assert_eq!(
            Articulation::TenutoStaccato.glyph(ArticulationPlacement::Below),
            Glyph::ArticTenutoStaccatoBelow
        );
    }

    #[test]
    fn tenuto_accent_glyph_pair() {
        assert_eq!(
            Articulation::TenutoAccent.glyph(ArticulationPlacement::Above),
            Glyph::ArticTenutoAccentAbove
        );
        assert_eq!(
            Articulation::TenutoAccent.glyph(ArticulationPlacement::Below),
            Glyph::ArticTenutoAccentBelow
        );
    }

    #[test]
    fn all_combined_variants_produce_distinct_above_glyphs() {
        let glyphs: Vec<Glyph> = COMBINED_VARIANTS
            .iter()
            .map(|c| c.glyph(ArticulationPlacement::Above))
            .collect();
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        g1, g2,
                        "combined variants {i} and {j} share above-glyph: {g1:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn all_combined_variants_produce_distinct_below_glyphs() {
        let glyphs: Vec<Glyph> = COMBINED_VARIANTS
            .iter()
            .map(|c| c.glyph(ArticulationPlacement::Below))
            .collect();
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(g1, g2, "combined variants {i} and {j} share below-glyph",);
                }
            }
        }
    }

    #[test]
    fn combined_variants_above_below_differ_within_each() {
        // Each combined-variant must have distinct above- vs below-glyphs;
        // Bravura supplies separate glyphs because they aren't mirror-images.
        for &c in &COMBINED_VARIANTS {
            assert_ne!(
                c.glyph(ArticulationPlacement::Above),
                c.glyph(ArticulationPlacement::Below),
                "{c:?} above and below glyphs should differ",
            );
        }
    }

    #[test]
    fn combined_variants_differ_from_simple_articulations() {
        // Regression net: a wiring typo could collapse e.g. AccentStaccato
        // back onto plain Accent. Verify no combined glyph aliases any of
        // the five simple-articulation glyphs (above placement).
        let simple_above: Vec<Glyph> = [
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
        ]
        .iter()
        .map(|a| a.glyph(ArticulationPlacement::Above))
        .collect();
        for &c in &COMBINED_VARIANTS {
            let g = c.glyph(ArticulationPlacement::Above);
            for s in &simple_above {
                assert_ne!(&g, s, "{c:?} above glyph {g:?} must not alias simple {s:?}",);
            }
        }
    }

    #[test]
    fn combined_variants_not_flagged_by_is_fermata() {
        for &c in &COMBINED_VARIANTS {
            assert!(
                !c.is_fermata(),
                "{c:?} must not be flagged as a fermata variant",
            );
        }
    }

    #[test]
    fn combined_variants_not_flagged_by_is_bow_stroke() {
        for &c in &COMBINED_VARIANTS {
            assert!(
                !c.is_bow_stroke(),
                "{c:?} must not be flagged as a bow stroke",
            );
        }
    }

    #[test]
    fn combined_variants_default_placement_follows_stem_opposite() {
        // Combined articulations must follow the standard "opposite from stem"
        // rule, not the "always above" rule used for fermatas and bow strokes.
        for &c in &COMBINED_VARIANTS {
            assert_eq!(
                c.default_placement(StemDirection::Up),
                ArticulationPlacement::Below,
                "{c:?} with stem-up should default to Below",
            );
            assert_eq!(
                c.default_placement(StemDirection::Down),
                ArticulationPlacement::Above,
                "{c:?} with stem-down should default to Above",
            );
        }
    }

    #[test]
    fn combined_alone_lays_out_opposite_stem() {
        // A combined articulation alone on a stem-up note must land below
        // the note (just like a plain Accent would). Specifically: y > note_y
        // and placement == Below.
        let staff = test_staff();
        let layout = layout_articulation(
            Articulation::AccentStaccato,
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(layout.placement, ArticulationPlacement::Below);
        assert_eq!(layout.glyph, Glyph::ArticAccentStaccatoBelow);
        let note_y = staff.y_of(4);
        assert!(
            layout.y > note_y,
            "below-placed combined articulation should have y > note_y: {} > {}",
            layout.y,
            note_y,
        );
    }

    #[test]
    fn stack_combined_with_fermata_separates_placement() {
        // Stem-up: combined articulation goes below, fermata always above.
        // Locks the bucket-partition rule for the combined variants.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::TenutoAccent.into(),
                Articulation::Fermata.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticTenutoAccentBelow);
        assert_eq!(stack[1].glyph, Glyph::FermataAbove);
        // Fermata above, combined below: fermata y < combined y.
        assert!(stack[1].y < stack[0].y);
    }

    #[test]
    fn stack_combined_with_bow_stem_up_separates_buckets() {
        // Stem-up: combined goes below (normal bucket), bow stroke above.
        // Confirms combined variants land in the `normal` bucket, not the
        // `bow_strokes` bucket — otherwise both would land above and stack.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::AccentStaccato.into(),
                Articulation::DownBow.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticAccentStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsDownBow);
    }

    #[test]
    fn stack_combined_with_simple_articulation_stacks_outward_same_side() {
        // Stem-up: both a plain Staccato and an AccentStaccato should go
        // below, in input order, stacked outward by one stack-step.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::Staccato.into(),
                Articulation::AccentStaccato.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        for l in &stack {
            assert_eq!(l.placement, ArticulationPlacement::Below);
        }
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::ArticAccentStaccatoBelow);
        // Second further from the note (higher y for Below).
        assert!(stack[1].y > stack[0].y);
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let actual_gap = stack[1].y - stack[0].y;
        assert!(
            (actual_gap - expected_gap).abs() < 1e-6,
            "stack spacing {actual_gap} should match expected {expected_gap}",
        );
    }

    #[test]
    fn stack_combined_full_triple_orders_correctly_stem_up() {
        // Full triple stack: combined articulation (below) + bow (above) +
        // fermata (above, outermost). Same ordering rule as a plain
        // normal+bow+fermata stack, since combined variants sit in the
        // normal bucket.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::MarcatoStaccato.into(),
                Articulation::UpBow.into(),
                Articulation::Fermata.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0].glyph, Glyph::ArticMarcatoStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsUpBow);
        assert_eq!(stack[2].glyph, Glyph::FermataAbove);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[2].placement, ArticulationPlacement::Above);
        let note_y = staff.y_of(4);
        assert!(stack[0].y > note_y, "combined below note");
        assert!(stack[1].y < note_y, "bow above note");
        assert!(stack[2].y < stack[1].y, "fermata above bow");
    }

    #[test]
    fn stack_combined_only_matches_single_layout() {
        // A single combined-articulation through the stacker must match
        // calling layout_articulation directly — no spurious offset.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::TenutoStaccato.into()],
            175.0,
            3,
            StemDirection::Down,
            &staff,
        );
        let direct = layout_articulation(
            Articulation::TenutoStaccato,
            175.0,
            3,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].x, direct.x);
        assert!((stack[0].y - direct.y).abs() < 1e-9);
        assert_eq!(stack[0].glyph, direct.glyph);
        assert_eq!(stack[0].placement, direct.placement);
    }

    #[test]
    fn stack_bow_only_matches_single_layout_y() {
        // A single bow stroke through the stack helper must position
        // identically to calling layout_articulation directly — no
        // unintended offset just because the stacker is involved.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::UpBow.into()],
            150.0,
            4,
            StemDirection::Down,
            &staff,
        );
        let direct =
            layout_articulation(Articulation::UpBow, 150.0, 4, StemDirection::Down, &staff);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].x, direct.x);
        assert!((stack[0].y - direct.y).abs() < 1e-9);
        assert_eq!(stack[0].glyph, direct.glyph);
        assert_eq!(stack[0].placement, direct.placement);
    }

    // --- Soft accent / stress / unstress (SMuFL accent extensions) ---
    //
    // These three articulations are extensions of the basic Accent for
    // prosodic / contemporary notation: SoftAccent is a parenthesized
    // accent (gentle emphasis), Stress marks a slightly emphasized note,
    // Unstress marks a deliberately de-emphasized one. They behave like
    // standard articulations: stem-opposite placement, distinct above/below
    // glyph pair per variant, and they live in the `normal` bucket of the
    // stacker (not the always-above bucket reserved for bow strokes and
    // fermatas).

    /// Convenience: the 3 SMuFL accent-extension variants in canonical order.
    const ACCENT_EXTENSION_VARIANTS: [Articulation; 3] = [
        Articulation::SoftAccent,
        Articulation::Stress,
        Articulation::Unstress,
    ];

    #[test]
    fn soft_accent_glyph_pair() {
        assert_eq!(
            Articulation::SoftAccent.glyph(ArticulationPlacement::Above),
            Glyph::ArticSoftAccentAbove
        );
        assert_eq!(
            Articulation::SoftAccent.glyph(ArticulationPlacement::Below),
            Glyph::ArticSoftAccentBelow
        );
    }

    #[test]
    fn stress_glyph_pair() {
        assert_eq!(
            Articulation::Stress.glyph(ArticulationPlacement::Above),
            Glyph::ArticStressAbove
        );
        assert_eq!(
            Articulation::Stress.glyph(ArticulationPlacement::Below),
            Glyph::ArticStressBelow
        );
    }

    #[test]
    fn unstress_glyph_pair() {
        assert_eq!(
            Articulation::Unstress.glyph(ArticulationPlacement::Above),
            Glyph::ArticUnstressAbove
        );
        assert_eq!(
            Articulation::Unstress.glyph(ArticulationPlacement::Below),
            Glyph::ArticUnstressBelow
        );
    }

    #[test]
    fn accent_extensions_produce_distinct_above_glyphs() {
        let glyphs: Vec<Glyph> = ACCENT_EXTENSION_VARIANTS
            .iter()
            .map(|a| a.glyph(ArticulationPlacement::Above))
            .collect();
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        g1, g2,
                        "accent-extension variants {i} and {j} share above-glyph: {g1:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn accent_extensions_produce_distinct_below_glyphs() {
        let glyphs: Vec<Glyph> = ACCENT_EXTENSION_VARIANTS
            .iter()
            .map(|a| a.glyph(ArticulationPlacement::Below))
            .collect();
        for (i, g1) in glyphs.iter().enumerate() {
            for (j, g2) in glyphs.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        g1, g2,
                        "accent-extension variants {i} and {j} share below-glyph",
                    );
                }
            }
        }
    }

    #[test]
    fn accent_extensions_above_below_differ_within_each() {
        for &a in &ACCENT_EXTENSION_VARIANTS {
            assert_ne!(
                a.glyph(ArticulationPlacement::Above),
                a.glyph(ArticulationPlacement::Below),
                "{a:?} above and below glyphs should differ",
            );
        }
    }

    #[test]
    fn accent_extensions_differ_from_plain_accent() {
        // Critical regression net: an enum-arm typo could collapse
        // SoftAccent/Stress/Unstress back onto the plain Accent. Each
        // must have a glyph distinct from `Articulation::Accent` for both
        // placements.
        let plain_above = Articulation::Accent.glyph(ArticulationPlacement::Above);
        let plain_below = Articulation::Accent.glyph(ArticulationPlacement::Below);
        for &a in &ACCENT_EXTENSION_VARIANTS {
            assert_ne!(
                a.glyph(ArticulationPlacement::Above),
                plain_above,
                "{a:?} above glyph must not alias plain Accent",
            );
            assert_ne!(
                a.glyph(ArticulationPlacement::Below),
                plain_below,
                "{a:?} below glyph must not alias plain Accent",
            );
        }
    }

    #[test]
    fn accent_extensions_not_flagged_by_is_fermata() {
        for &a in &ACCENT_EXTENSION_VARIANTS {
            assert!(
                !a.is_fermata(),
                "{a:?} must not be flagged as a fermata variant",
            );
        }
    }

    #[test]
    fn accent_extensions_not_flagged_by_is_bow_stroke() {
        for &a in &ACCENT_EXTENSION_VARIANTS {
            assert!(
                !a.is_bow_stroke(),
                "{a:?} must not be flagged as a bow stroke",
            );
        }
    }

    #[test]
    fn accent_extensions_default_placement_follows_stem_opposite() {
        // Standard articulation rule: opposite from stem. Crucial because
        // the previous post-v1 chunk (bow strokes) introduced "always above"
        // placement for a different family — a copy-paste mistake could put
        // these in the wrong bucket.
        for &a in &ACCENT_EXTENSION_VARIANTS {
            assert_eq!(
                a.default_placement(StemDirection::Up),
                ArticulationPlacement::Below,
                "{a:?} with stem-up should default to Below",
            );
            assert_eq!(
                a.default_placement(StemDirection::Down),
                ArticulationPlacement::Above,
                "{a:?} with stem-down should default to Above",
            );
        }
    }

    #[test]
    fn soft_accent_alone_lays_out_opposite_stem() {
        let staff = test_staff();
        // Stem-up, middle-line note: SoftAccent should land below.
        let layout = layout_articulation(
            Articulation::SoftAccent,
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(layout.placement, ArticulationPlacement::Below);
        assert_eq!(layout.glyph, Glyph::ArticSoftAccentBelow);
        let note_y = staff.y_of(4);
        assert!(
            layout.y > note_y,
            "below-placed soft accent should have y > note_y: {} > {}",
            layout.y,
            note_y,
        );
    }

    #[test]
    fn stress_alone_lays_out_opposite_stem_down() {
        let staff = test_staff();
        // Stem-down, middle-line note: Stress should land above.
        let layout =
            layout_articulation(Articulation::Stress, 150.0, 4, StemDirection::Down, &staff);
        assert_eq!(layout.placement, ArticulationPlacement::Above);
        assert_eq!(layout.glyph, Glyph::ArticStressAbove);
        let note_y = staff.y_of(4);
        assert!(
            layout.y < note_y,
            "above-placed stress should have y < note_y: {} < {}",
            layout.y,
            note_y,
        );
    }

    #[test]
    fn stack_accent_extension_with_fermata_separates_placement() {
        // Stem-up: accent extension goes below, fermata always above.
        // Locks the bucket-partition rule for the new variants.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::SoftAccent.into(),
                Articulation::Fermata.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticSoftAccentBelow);
        assert_eq!(stack[1].glyph, Glyph::FermataAbove);
        assert!(stack[1].y < stack[0].y);
    }

    #[test]
    fn stack_accent_extension_with_bow_stem_up_separates_buckets() {
        // Stem-up: accent extension goes below (normal bucket), bow stroke
        // above. Catches a regression where Stress or Unstress would be
        // incorrectly classified as bow-stroke-equivalent.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Stress.into(), Articulation::DownBow.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticStressBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsDownBow);
    }

    #[test]
    fn stack_accent_extension_with_simple_articulation_stacks_outward_same_side() {
        // Stem-up: a Staccato and an Unstress both go below, input order
        // preserved, stacked outward by one stack-step.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into(), Articulation::Unstress.into()],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        for l in &stack {
            assert_eq!(l.placement, ArticulationPlacement::Below);
        }
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::ArticUnstressBelow);
        assert!(stack[1].y > stack[0].y);
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let actual_gap = stack[1].y - stack[0].y;
        assert!(
            (actual_gap - expected_gap).abs() < 1e-6,
            "stack spacing {actual_gap} should match expected {expected_gap}",
        );
    }

    #[test]
    fn stack_accent_extension_full_triple_orders_correctly_stem_up() {
        // Full triple stack: accent extension (below) + bow (above) +
        // fermata (above, outermost). Same ordering rule as plain
        // normal+bow+fermata stack, since accent extensions sit in the
        // normal bucket.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::SoftAccent.into(),
                Articulation::UpBow.into(),
                Articulation::Fermata.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0].glyph, Glyph::ArticSoftAccentBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsUpBow);
        assert_eq!(stack[2].glyph, Glyph::FermataAbove);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[2].placement, ArticulationPlacement::Above);
        let note_y = staff.y_of(4);
        assert!(stack[0].y > note_y, "soft accent below note");
        assert!(stack[1].y < note_y, "bow above note");
        assert!(stack[2].y < stack[1].y, "fermata above bow");
    }

    #[test]
    fn stack_accent_extension_only_matches_single_layout() {
        // A single accent-extension through the stacker must produce a
        // layout identical to calling layout_articulation directly — no
        // spurious offset.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Unstress.into()],
            175.0,
            3,
            StemDirection::Down,
            &staff,
        );
        let direct = layout_articulation(
            Articulation::Unstress,
            175.0,
            3,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].x, direct.x);
        assert!((stack[0].y - direct.y).abs() < 1e-9);
        assert_eq!(stack[0].glyph, direct.glyph);
        assert_eq!(stack[0].placement, direct.placement);
    }

    // --- LaissezVibrer tests ---
    //
    // l.v. ("let ring") is a single articulation variant that ships a real
    // above/below pair in SMuFL (codepoints E4BA/E4BB) and behaves
    // identically to a standard articulation: opposite-stem placement,
    // lives in the normal stack bucket (not always-above like fermata or
    // bow strokes). These tests lock both the glyph mapping and the
    // stacking-bucket classification.

    #[test]
    fn laissez_vibrer_glyph_pair() {
        // Locks the exact (LaissezVibrer, Above/Below) → SMuFL glyph
        // mapping. Any swap to a different glyph (e.g. ArticTenutoAbove)
        // would silently render as the wrong symbol.
        assert_eq!(
            Articulation::LaissezVibrer.glyph(ArticulationPlacement::Above),
            Glyph::ArticLaissezVibrerAbove
        );
        assert_eq!(
            Articulation::LaissezVibrer.glyph(ArticulationPlacement::Below),
            Glyph::ArticLaissezVibrerBelow
        );
    }

    #[test]
    fn laissez_vibrer_above_below_differ() {
        // Bravura ships distinct above/below glyphs (not a draw-time flip
        // of a single glyph). The pair must differ at the glyph level.
        assert_ne!(
            Articulation::LaissezVibrer.glyph(ArticulationPlacement::Above),
            Articulation::LaissezVibrer.glyph(ArticulationPlacement::Below),
            "l.v. above and below glyphs should be distinct",
        );
    }

    #[test]
    fn laissez_vibrer_glyph_differs_from_all_other_articulations() {
        // Regression net against an enum-arm typo that would collapse
        // LaissezVibrer onto any other articulation's glyph. Checks both
        // placements against every other variant in the enum.
        let others: &[Articulation] = &[
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
            Articulation::Fermata,
            Articulation::FermataLong,
            Articulation::FermataShort,
            Articulation::FermataVeryLong,
            Articulation::FermataVeryShort,
            Articulation::FermataHenzeLong,
            Articulation::FermataHenzeShort,
            Articulation::UpBow,
            Articulation::DownBow,
            Articulation::AccentStaccato,
            Articulation::MarcatoStaccato,
            Articulation::TenutoStaccato,
            Articulation::TenutoAccent,
            Articulation::SoftAccent,
            Articulation::Stress,
            Articulation::Unstress,
        ];
        let lv_above = Articulation::LaissezVibrer.glyph(ArticulationPlacement::Above);
        let lv_below = Articulation::LaissezVibrer.glyph(ArticulationPlacement::Below);
        for &o in others {
            assert_ne!(
                lv_above,
                o.glyph(ArticulationPlacement::Above),
                "l.v.-above must not alias {o:?}-above",
            );
            assert_ne!(
                lv_below,
                o.glyph(ArticulationPlacement::Below),
                "l.v.-below must not alias {o:?}-below",
            );
        }
    }

    #[test]
    fn laissez_vibrer_not_flagged_by_is_fermata() {
        // Defensive: l.v. is not a fermata, so the stack splitter must
        // not route it through the fermata bucket.
        assert!(!Articulation::LaissezVibrer.is_fermata());
    }

    #[test]
    fn laissez_vibrer_not_flagged_by_is_bow_stroke() {
        // Defensive: l.v. is not a bow stroke, so the stack splitter must
        // not route it through the bow-stroke bucket.
        assert!(!Articulation::LaissezVibrer.is_bow_stroke());
    }

    #[test]
    fn laissez_vibrer_default_placement_follows_stem_opposite() {
        // Standard articulation rule: opposite from stem. A copy-paste
        // typo could put l.v. in the always-above bucket (where it would
        // appear on the wrong side for stem-down notes).
        assert_eq!(
            Articulation::LaissezVibrer.default_placement(StemDirection::Up),
            ArticulationPlacement::Below,
            "l.v. with stem-up should default to Below",
        );
        assert_eq!(
            Articulation::LaissezVibrer.default_placement(StemDirection::Down),
            ArticulationPlacement::Above,
            "l.v. with stem-down should default to Above",
        );
    }

    #[test]
    fn laissez_vibrer_alone_lays_out_below_stem_up_note() {
        // Stem-up, middle-line note: l.v. lands below with the
        // ArticLaissezVibrerBelow glyph and y > note_y.
        let staff = test_staff();
        let layout = layout_articulation(
            Articulation::LaissezVibrer,
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(layout.placement, ArticulationPlacement::Below);
        assert_eq!(layout.glyph, Glyph::ArticLaissezVibrerBelow);
        assert_eq!(layout.x, 100.0);
        let note_y = staff.y_of(4);
        assert!(
            layout.y > note_y,
            "below-placed l.v. should have y > note_y: {} > {}",
            layout.y,
            note_y,
        );
    }

    #[test]
    fn laissez_vibrer_alone_lays_out_above_stem_down_note() {
        // Symmetric to the previous test: stem-down, in-space note → l.v.
        // lands above with the ArticLaissezVibrerAbove glyph.
        let staff = test_staff();
        let layout = layout_articulation(
            Articulation::LaissezVibrer,
            150.0,
            3,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(layout.placement, ArticulationPlacement::Above);
        assert_eq!(layout.glyph, Glyph::ArticLaissezVibrerAbove);
        assert_eq!(layout.x, 150.0);
        let note_y = staff.y_of(3);
        assert!(
            layout.y < note_y,
            "above-placed l.v. should have y < note_y: {} < {}",
            layout.y,
            note_y,
        );
    }

    #[test]
    fn stack_laissez_vibrer_with_fermata_separates_placement() {
        // Stem-up: l.v. goes below (normal bucket), fermata always above.
        // Verifies that the stack splitter routes l.v. to the normal
        // bucket — not the always-above bucket reserved for fermatas.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::LaissezVibrer.into(),
                Articulation::Fermata.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticLaissezVibrerBelow);
        assert_eq!(stack[1].glyph, Glyph::FermataAbove);
        assert!(stack[1].y < stack[0].y);
    }

    #[test]
    fn stack_laissez_vibrer_with_bow_stem_up_separates_buckets() {
        // Stem-up: l.v. below (normal bucket), bow above (bow bucket).
        // Catches a misclassification that would route l.v. through the
        // bow-stroke bucket.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::LaissezVibrer.into(),
                Articulation::UpBow.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[0].glyph, Glyph::ArticLaissezVibrerBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsUpBow);
    }

    #[test]
    fn stack_laissez_vibrer_with_simple_articulation_stacks_outward_same_side() {
        // Both Staccato and LaissezVibrer go to the normal bucket. On a
        // stem-up note they both render below; the second is offset by
        // exactly one ARTICULATION_STACK_SPACING_SS × staff_space.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::Staccato.into(),
                Articulation::LaissezVibrer.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 2);
        for l in &stack {
            assert_eq!(l.placement, ArticulationPlacement::Below);
        }
        assert_eq!(stack[0].glyph, Glyph::ArticStaccatoBelow);
        assert_eq!(stack[1].glyph, Glyph::ArticLaissezVibrerBelow);
        assert!(stack[1].y > stack[0].y);
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let actual_gap = stack[1].y - stack[0].y;
        assert!(
            (actual_gap - expected_gap).abs() < 1e-6,
            "stack spacing {actual_gap} should match expected {expected_gap}",
        );
    }

    #[test]
    fn stack_laissez_vibrer_full_triple_orders_correctly_stem_up() {
        // Full triple: l.v. (normal bucket, below) + DownBow (bow bucket,
        // above) + FermataLong (fermata bucket, above-outermost). Locks
        // the normal→bow→fermata bucket-emit order for the new variant.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[
                Articulation::LaissezVibrer.into(),
                Articulation::DownBow.into(),
                Articulation::FermataLong.into(),
            ],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0].glyph, Glyph::ArticLaissezVibrerBelow);
        assert_eq!(stack[1].glyph, Glyph::StringsDownBow);
        assert_eq!(stack[2].glyph, Glyph::FermataLongAbove);
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Above);
        assert_eq!(stack[2].placement, ArticulationPlacement::Above);
        let note_y = staff.y_of(4);
        assert!(stack[0].y > note_y, "l.v. below note");
        assert!(stack[1].y < note_y, "bow above note");
        assert!(stack[2].y < stack[1].y, "fermata above bow");
        // The bow→fermata gap is exactly one stack spacing (cascading-
        // above rule shared with the other always-above buckets).
        let expected_gap = ARTICULATION_STACK_SPACING_SS * staff.staff_space;
        let bow_to_fermata = stack[1].y - stack[2].y;
        assert!(
            (bow_to_fermata - expected_gap).abs() < 1e-6,
            "bow→fermata gap {bow_to_fermata} should match {expected_gap}",
        );
    }

    #[test]
    fn stack_laissez_vibrer_only_matches_single_layout() {
        // A single LaissezVibrer through the stacker must produce a
        // layout identical to calling layout_articulation directly — no
        // spurious offset. Mirrors the equivalent guard for accent
        // extensions and bow strokes.
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::LaissezVibrer.into()],
            220.5,
            5,
            StemDirection::Down,
            &staff,
        );
        let direct = layout_articulation(
            Articulation::LaissezVibrer,
            220.5,
            5,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].x, direct.x);
        assert!((stack[0].y - direct.y).abs() < 1e-9);
        assert_eq!(stack[0].glyph, direct.glyph);
        assert_eq!(stack[0].placement, direct.placement);
    }

    #[test]
    fn custom_glyph_supports_auto_and_forced_side_and_stacks_with_staccato() {
        let staff = test_staff();
        let mark = ArticulationMark::custom(Glyph::ArticMarcatoAbove);
        let auto = layout_articulation(mark, 100.0, 4, StemDirection::Up, &staff);
        let above = layout_articulation(mark.above(), 100.0, 4, StemDirection::Up, &staff);
        assert_eq!(auto.glyph, Glyph::ArticMarcatoAbove);
        assert_eq!(auto.placement, ArticulationPlacement::Below);
        assert_eq!(above.placement, ArticulationPlacement::Above);
        let below = layout_articulation(mark.below(), 100.0, 4, StemDirection::Down, &staff);
        assert_eq!(below.placement, ArticulationPlacement::Below);

        let stack = layout_articulation_stack(
            &[Articulation::Staccato.into(), mark],
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        assert_eq!(stack[0].placement, ArticulationPlacement::Below);
        assert_eq!(stack[1].placement, ArticulationPlacement::Below);
        assert!(stack[1].y > stack[0].y, "the second mark clears the first");
    }
}
