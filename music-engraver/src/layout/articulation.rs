use smufl::Glyph;

use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;

/// Standard articulation markings that attach close to the notehead.
///
/// Articulations are placed either above or below the note depending on stem
/// direction (opposite side from the stem, except fermata which is always above).
/// Each variant maps to a pair of SMuFL glyphs (above/below variants).
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
}

/// Whether an articulation appears above or below the notehead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArticulationPlacement {
    Above,
    Below,
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
            (Self::Staccatissimo, ArticulationPlacement::Above) => {
                Glyph::ArticStaccatissimoAbove
            }
            (Self::Staccatissimo, ArticulationPlacement::Below) => {
                Glyph::ArticStaccatissimoBelow
            }
            (Self::Fermata, ArticulationPlacement::Above) => Glyph::FermataAbove,
            (Self::Fermata, ArticulationPlacement::Below) => Glyph::FermataBelow,
        }
    }

    /// Default placement relative to stem direction.
    ///
    /// Convention: articulations go on the opposite side from the stem.
    /// Fermata is an exception — it is always placed above.
    pub fn default_placement(self, stem_dir: StemDirection) -> ArticulationPlacement {
        if self == Self::Fermata {
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
    /// The SMuFL glyph to render.
    pub glyph: Glyph,
    /// Whether the articulation is placed above or below.
    pub placement: ArticulationPlacement,
}

/// Compute the position for an articulation relative to a note.
///
/// `notehead_x` is the x-center of the notehead. `note_staff_position` is the
/// staff position of the note (0 = bottom line). `stem_dir` determines default
/// placement (articulation goes opposite the stem; fermata always above).
/// `staff` provides the coordinate mapping.
pub fn layout_articulation(
    articulation: Articulation,
    notehead_x: f64,
    note_staff_position: i8,
    stem_dir: StemDirection,
    staff: &StaffLayout,
) -> ArticulationLayout {
    let placement = articulation.default_placement(stem_dir);
    let glyph = articulation.glyph(placement);

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

    let y = match placement {
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
    };

    ArticulationLayout {
        x: notehead_x,
        y,
        glyph,
        placement,
    }
}

/// Compute positions for multiple stacked articulations on a single note.
///
/// Articulations are stacked outward from the notehead: the first in the list
/// is closest to the note, each subsequent one is placed further away.
/// Fermata is always placed above, even when other articulations are below.
/// If the list contains a fermata mixed with non-fermata articulations,
/// the fermata is separated and placed above, while the rest follow the
/// normal stem-direction rule.
pub fn layout_articulation_stack(
    articulations: &[Articulation],
    notehead_x: f64,
    note_staff_position: i8,
    stem_dir: StemDirection,
    staff: &StaffLayout,
) -> Vec<ArticulationLayout> {
    if articulations.is_empty() {
        return Vec::new();
    }

    // Separate fermata(s) from other articulations since fermata always goes above
    let mut normal: Vec<Articulation> = Vec::new();
    let mut fermatas: Vec<Articulation> = Vec::new();
    for &a in articulations {
        if a == Articulation::Fermata {
            fermatas.push(a);
        } else {
            normal.push(a);
        }
    }

    let mut result = Vec::with_capacity(articulations.len());
    let stack_spacing = ARTICULATION_STACK_SPACING_SS * staff.staff_space;

    // Place normal articulations on the stem-opposite side, stacking outward
    if !normal.is_empty() {
        let first_layout =
            layout_articulation(normal[0], notehead_x, note_staff_position, stem_dir, staff);
        let base_y = first_layout.y;
        let placement = first_layout.placement;
        result.push(first_layout);

        for (i, &artic) in normal.iter().enumerate().skip(1) {
            let glyph = artic.glyph(placement);
            let y = match placement {
                ArticulationPlacement::Above => base_y - (i as f64) * stack_spacing,
                ArticulationPlacement::Below => base_y + (i as f64) * stack_spacing,
            };
            result.push(ArticulationLayout {
                x: notehead_x,
                y,
                glyph,
                placement,
            });
        }
    }

    // Place fermata(s) above, stacked above any above-placement articulations
    if !fermatas.is_empty() {
        // Find the topmost y already used (smallest y value for Above placement)
        let topmost_above = result
            .iter()
            .filter(|l| l.placement == ArticulationPlacement::Above)
            .map(|l| l.y)
            .fold(f64::INFINITY, f64::min);

        let fermata_base_y = if topmost_above.is_finite() {
            // Stack above existing above-articulations
            topmost_above - stack_spacing
        } else {
            // No above articulations yet — use the normal layout position
            let fl = layout_articulation(
                Articulation::Fermata,
                notehead_x,
                note_staff_position,
                stem_dir,
                staff,
            );
            fl.y
        };

        for (i, &f) in fermatas.iter().enumerate() {
            let glyph = f.glyph(ArticulationPlacement::Above);
            result.push(ArticulationLayout {
                x: notehead_x,
                y: fermata_base_y - (i as f64) * stack_spacing,
                glyph,
                placement: ArticulationPlacement::Above,
            });
        }
    }

    result
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
        let result =
            layout_articulation_stack(&[], 100.0, 4, StemDirection::Up, &staff);
        assert!(result.is_empty());
    }

    #[test]
    fn stack_single_matches_layout_articulation() {
        let staff = test_staff();
        let stack = layout_articulation_stack(
            &[Articulation::Staccato],
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
            &[Articulation::Staccato, Articulation::Accent],
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
            &[Articulation::Tenuto, Articulation::Marcato],
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
            &[Articulation::Staccato, Articulation::Fermata],
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
            &[Articulation::Staccato, Articulation::Fermata],
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
            &[Articulation::Staccato, Articulation::Accent, Articulation::Tenuto],
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
            &[Articulation::Staccato, Articulation::Accent, Articulation::Fermata],
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
            &[Articulation::Fermata],
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
}
