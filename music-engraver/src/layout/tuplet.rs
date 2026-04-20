use smufl::Glyph;

use crate::layout::staff::StaffPosition;
use crate::layout::stem::StemDirection;

/// Placement of the tuplet bracket and number relative to the staff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TupletPlacement {
    /// Bracket and number above the notes (typical for stems-down notes).
    Above,
    /// Bracket and number below the notes (typical for stems-up notes).
    Below,
}

/// Choose tuplet placement opposite the stem direction, as is standard:
/// stems up → bracket below, stems down → bracket above.
pub fn tuplet_placement_from_stem(direction: StemDirection) -> TupletPlacement {
    match direction {
        StemDirection::Up => TupletPlacement::Above,
        StemDirection::Down => TupletPlacement::Below,
    }
}

/// Return the SMuFL tuplet digit glyph for 0–9. Returns `None` for values ≥ 10.
pub fn tuplet_digit_glyph(digit: u8) -> Option<Glyph> {
    match digit {
        0 => Some(Glyph::Tuplet0),
        1 => Some(Glyph::Tuplet1),
        2 => Some(Glyph::Tuplet2),
        3 => Some(Glyph::Tuplet3),
        4 => Some(Glyph::Tuplet4),
        5 => Some(Glyph::Tuplet5),
        6 => Some(Glyph::Tuplet6),
        7 => Some(Glyph::Tuplet7),
        8 => Some(Glyph::Tuplet8),
        9 => Some(Glyph::Tuplet9),
        _ => None,
    }
}

/// Return the sequence of SMuFL digit glyphs for a tuplet number (e.g. 3 → [Tuplet3],
/// 12 → [Tuplet1, Tuplet2]). Returns empty vec for 0.
pub fn tuplet_number_glyphs(n: u32) -> Vec<Glyph> {
    if n == 0 {
        return vec![];
    }
    let digits: Vec<u8> = n
        .to_string()
        .bytes()
        .map(|b| b - b'0')
        .collect();
    digits
        .into_iter()
        .filter_map(tuplet_digit_glyph)
        .collect()
}

/// Distance from the outermost note to the bracket line, in staff spaces.
const BRACKET_OFFSET_SS: f64 = 1.0;

/// Height of the bracket hook (the small vertical ticks at each end), in staff spaces.
const BRACKET_HOOK_SS: f64 = 0.5;

/// Gap between bracket line and number glyph on each side, in staff spaces.
const NUMBER_GAP_SS: f64 = 0.5;

/// Computed layout for a tuplet bracket with number.
#[derive(Clone, Debug)]
pub struct TupletBracketLayout {
    /// X-coordinate of the left end of the bracket.
    pub x_left: f64,
    /// X-coordinate of the right end of the bracket.
    pub x_right: f64,
    /// Y-coordinate of the bracket line (shared by hooks and number baseline reference).
    pub bracket_y: f64,
    /// Height of the hook ticks (absolute value), extending toward the notes.
    pub hook_height: f64,
    /// SMuFL glyphs for the tuplet number, to be drawn centered in the bracket gap.
    pub number_glyphs: Vec<Glyph>,
    /// X-coordinate where the number glyphs should start (centered in bracket).
    pub number_x: f64,
    /// Y-coordinate for the number glyphs (same as bracket_y — baseline aligned).
    pub number_y: f64,
    /// X-coordinate where the gap for the number begins (left bracket line ends here).
    pub gap_left_x: f64,
    /// X-coordinate where the gap for the number ends (right bracket line starts here).
    pub gap_right_x: f64,
    /// Bracket line thickness in font design units.
    pub bracket_thickness: f64,
    /// Placement (above or below).
    pub placement: TupletPlacement,
}

/// Lay out a tuplet bracket and number for a group of notes.
///
/// # Parameters
/// - `x_left`, `x_right`: horizontal span of the tuplet group (first notehead to last).
/// - `extreme_positions`: the staff positions of all notes in the tuplet, used to
///   determine the bracket's vertical position (most extreme note + offset).
/// - `placement`: above or below the notes.
/// - `tuplet_number`: the number to display (e.g. 3 for triplet, 5 for quintuplet).
/// - `staff_space`: font design units per staff space.
/// - `bracket_thickness_ss`: bracket line thickness in staff spaces (from `EngravingConfig`).
/// - `number_advance_width`: total advance width of the tuplet number glyph(s) in font units,
///   used for centering. If not known, pass 0.0 and the number will be centered by estimated width.
#[allow(clippy::too_many_arguments)]
pub fn layout_tuplet_bracket(
    x_left: f64,
    x_right: f64,
    extreme_positions: &[StaffPosition],
    placement: TupletPlacement,
    tuplet_number: u32,
    staff_space: f64,
    bracket_thickness_ss: f64,
    number_advance_width: f64,
) -> TupletBracketLayout {
    let half_space = staff_space / 2.0;
    let offset = BRACKET_OFFSET_SS * staff_space;
    let hook_height = BRACKET_HOOK_SS * staff_space;

    // Find the most extreme note position
    let bracket_y = if extreme_positions.is_empty() {
        // No notes: place bracket at middle of staff
        match placement {
            TupletPlacement::Above => 0.0 - offset,
            TupletPlacement::Below => 4.0 * staff_space + offset,
        }
    } else {
        match placement {
            TupletPlacement::Above => {
                // Highest note (largest staff position) → smallest y
                let max_pos = *extreme_positions.iter().max().unwrap();
                let extreme_y = (8 - max_pos) as f64 * half_space;
                extreme_y - offset
            }
            TupletPlacement::Below => {
                // Lowest note (smallest staff position) → largest y
                let min_pos = *extreme_positions.iter().min().unwrap();
                let extreme_y = (8 - min_pos) as f64 * half_space;
                extreme_y + offset
            }
        }
    };

    let number_glyphs = tuplet_number_glyphs(tuplet_number);

    // Center the number horizontally in the bracket span
    let bracket_center_x = (x_left + x_right) / 2.0;
    let number_x = bracket_center_x - number_advance_width / 2.0;

    // Gap in the bracket line around the number
    let gap_padding = NUMBER_GAP_SS * staff_space;
    let gap_left_x = bracket_center_x - number_advance_width / 2.0 - gap_padding;
    let gap_right_x = bracket_center_x + number_advance_width / 2.0 + gap_padding;

    TupletBracketLayout {
        x_left,
        x_right,
        bracket_y,
        hook_height,
        number_glyphs,
        number_x,
        number_y: bracket_y,
        gap_left_x,
        gap_right_x,
        bracket_thickness: bracket_thickness_ss * staff_space,
        placement,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: f64 = 250.0;
    const HS: f64 = 125.0;

    // --- tuplet_digit_glyph ---

    #[test]
    fn digit_glyph_all_valid() {
        assert_eq!(tuplet_digit_glyph(0), Some(Glyph::Tuplet0));
        assert_eq!(tuplet_digit_glyph(3), Some(Glyph::Tuplet3));
        assert_eq!(tuplet_digit_glyph(9), Some(Glyph::Tuplet9));
    }

    #[test]
    fn digit_glyph_out_of_range() {
        assert_eq!(tuplet_digit_glyph(10), None);
        assert_eq!(tuplet_digit_glyph(255), None);
    }

    // --- tuplet_number_glyphs ---

    #[test]
    fn number_glyphs_single_digit() {
        let glyphs = tuplet_number_glyphs(3);
        assert_eq!(glyphs, vec![Glyph::Tuplet3]);
    }

    #[test]
    fn number_glyphs_two_digits() {
        let glyphs = tuplet_number_glyphs(12);
        assert_eq!(glyphs, vec![Glyph::Tuplet1, Glyph::Tuplet2]);
    }

    #[test]
    fn number_glyphs_zero() {
        assert!(tuplet_number_glyphs(0).is_empty());
    }

    // --- tuplet_placement_from_stem ---

    #[test]
    fn stems_up_bracket_above() {
        assert_eq!(
            tuplet_placement_from_stem(StemDirection::Up),
            TupletPlacement::Above
        );
    }

    #[test]
    fn stems_down_bracket_below() {
        assert_eq!(
            tuplet_placement_from_stem(StemDirection::Down),
            TupletPlacement::Below
        );
    }

    // --- layout_tuplet_bracket ---

    #[test]
    fn bracket_above_is_above_highest_note() {
        // Notes at positions 4, 6, 8 (middle line to top line)
        let layout = layout_tuplet_bracket(
            0.0, 1000.0, &[4, 6, 8], TupletPlacement::Above,
            3, SS, 0.16, 100.0,
        );
        // Top line (pos 8) → y=0; bracket should be above that
        let top_y = (8 - 8) as f64 * HS; // 0.0
        assert!(layout.bracket_y < top_y,
            "bracket_y {} should be above top note y {}", layout.bracket_y, top_y);
    }

    #[test]
    fn bracket_below_is_below_lowest_note() {
        // Notes at positions 0, 2, 4
        let layout = layout_tuplet_bracket(
            0.0, 1000.0, &[0, 2, 4], TupletPlacement::Below,
            3, SS, 0.16, 100.0,
        );
        // Bottom line (pos 0) → y=1000
        let bottom_y = 8.0 * HS;
        assert!(layout.bracket_y > bottom_y,
            "bracket_y {} should be below bottom note y {}", layout.bracket_y, bottom_y);
    }

    #[test]
    fn number_centered_horizontally() {
        let layout = layout_tuplet_bracket(
            100.0, 900.0, &[4], TupletPlacement::Above,
            3, SS, 0.16, 200.0,
        );
        // Center of bracket = (100+900)/2 = 500; number_x = 500 - 200/2 = 400
        assert!((layout.number_x - 400.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hook_height_is_half_staff_space() {
        let layout = layout_tuplet_bracket(
            0.0, 500.0, &[4], TupletPlacement::Above,
            3, SS, 0.16, 0.0,
        );
        assert!((layout.hook_height - BRACKET_HOOK_SS * SS).abs() < f64::EPSILON);
    }

    #[test]
    fn bracket_thickness_from_config() {
        let thickness_ss = 0.16;
        let layout = layout_tuplet_bracket(
            0.0, 500.0, &[4], TupletPlacement::Above,
            3, SS, thickness_ss, 0.0,
        );
        assert!((layout.bracket_thickness - thickness_ss * SS).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_positions_uses_default() {
        let layout_above = layout_tuplet_bracket(
            0.0, 500.0, &[], TupletPlacement::Above,
            3, SS, 0.16, 0.0,
        );
        // Should be above the staff (y < 0)
        assert!(layout_above.bracket_y < 0.0);

        let layout_below = layout_tuplet_bracket(
            0.0, 500.0, &[], TupletPlacement::Below,
            3, SS, 0.16, 0.0,
        );
        // Should be below the staff (y > staff height = 4*SS = 1000)
        assert!(layout_below.bracket_y > 4.0 * SS);
    }

    #[test]
    fn placement_preserved() {
        let layout = layout_tuplet_bracket(
            0.0, 500.0, &[4], TupletPlacement::Below,
            5, SS, 0.16, 0.0,
        );
        assert_eq!(layout.placement, TupletPlacement::Below);
    }

    #[test]
    fn number_y_equals_bracket_y() {
        let layout = layout_tuplet_bracket(
            0.0, 500.0, &[4], TupletPlacement::Above,
            3, SS, 0.16, 0.0,
        );
        assert!((layout.number_y - layout.bracket_y).abs() < f64::EPSILON);
    }

    #[test]
    fn quintuplet_number_glyphs() {
        let layout = layout_tuplet_bracket(
            0.0, 500.0, &[4], TupletPlacement::Above,
            5, SS, 0.16, 0.0,
        );
        assert_eq!(layout.number_glyphs, vec![Glyph::Tuplet5]);
    }

    #[test]
    fn x_bounds_preserved() {
        let layout = layout_tuplet_bracket(
            123.0, 789.0, &[4], TupletPlacement::Above,
            3, SS, 0.16, 0.0,
        );
        assert!((layout.x_left - 123.0).abs() < f64::EPSILON);
        assert!((layout.x_right - 789.0).abs() < f64::EPSILON);
    }

    #[test]
    fn above_bracket_moves_higher_for_higher_notes() {
        let low = layout_tuplet_bracket(
            0.0, 500.0, &[2], TupletPlacement::Above,
            3, SS, 0.16, 0.0,
        );
        let high = layout_tuplet_bracket(
            0.0, 500.0, &[10], TupletPlacement::Above,
            3, SS, 0.16, 0.0,
        );
        // Higher note (pos 10) → lower y → bracket_y should be lower (more negative)
        assert!(high.bracket_y < low.bracket_y,
            "high note bracket_y {} should be < low note bracket_y {}",
            high.bracket_y, low.bracket_y);
    }
}
