use crate::layout::staff::StaffPosition;
use smufl::Glyph;

/// The SMuFL glyph used for augmentation dots.
pub const DOT_GLYPH: Glyph = Glyph::AugmentationDot;

/// Horizontal padding between the right edge of the notehead and the left
/// edge of the first augmentation dot, in staff spaces.
///
/// SMuFL and standard engraving practice place the dot roughly 0.5 staff
/// spaces from the notehead's right edge. In practice, the distance is from
/// the notehead right edge to the dot's left edge (i.e., the dot center is
/// further right by half the dot's advance width).
pub const DOT_NOTEHEAD_PADDING_SS: f64 = 0.5;

/// Horizontal spacing between the left edges of consecutive augmentation
/// dots, in staff spaces.
///
/// For double- and triple-dotted notes, subsequent dots are spaced at
/// roughly 0.35 staff spaces center-to-center, which is approximately
/// the dot's advance width plus a small gap.
pub const DOT_INTER_DOT_SPACING_SS: f64 = 0.35;

/// Compute the x-position of the first augmentation dot's left edge.
///
/// All values in font design units.
pub fn first_dot_x(notehead_x: f64, notehead_advance: f64, staff_space: f64) -> f64 {
    notehead_x + notehead_advance + DOT_NOTEHEAD_PADDING_SS * staff_space
}

/// Compute x-positions for `dot_count` augmentation dots.
///
/// Returns x-positions (left edge of each dot glyph) in font design units.
pub fn dot_xs(
    notehead_x: f64,
    notehead_advance: f64,
    staff_space: f64,
    dot_count: u8,
) -> Vec<f64> {
    if dot_count == 0 {
        return vec![];
    }
    let first_x = first_dot_x(notehead_x, notehead_advance, staff_space);
    let inter_dot = DOT_INTER_DOT_SPACING_SS * staff_space;
    (0..dot_count)
        .map(|i| first_x + i as f64 * inter_dot)
        .collect()
}

/// Compute the y staff position for an augmentation dot.
///
/// If the note sits on a staff line (even-numbered position), the dot
/// is shifted up by one half-space so it sits in the space above.
/// Notes in spaces keep their original position.
pub fn dot_staff_position(note_position: StaffPosition) -> StaffPosition {
    if note_position % 2 == 0 {
        // On a line — shift to space above
        note_position + 1
    } else {
        note_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: f64 = 250.0; // Bravura staff space

    // --- first_dot_x ---

    #[test]
    fn first_dot_x_basic() {
        let x = first_dot_x(500.0, 295.0, SS);
        // 500 + 295 + 0.5 * 250 = 920
        assert!((x - 920.0).abs() < 1e-6, "got {x}");
    }

    #[test]
    fn first_dot_x_at_origin() {
        let x = first_dot_x(0.0, 295.0, SS);
        // 0 + 295 + 125 = 420
        assert!((x - 420.0).abs() < 1e-6, "got {x}");
    }

    #[test]
    fn first_dot_x_scales_with_staff_space() {
        let x_small = first_dot_x(0.0, 295.0, 200.0);
        let x_large = first_dot_x(0.0, 295.0, 400.0);
        // Larger staff space → more padding → dot further right
        assert!(x_large > x_small);
    }

    // --- dot_xs ---

    #[test]
    fn dot_xs_zero_dots_returns_empty() {
        let xs = dot_xs(500.0, 295.0, SS, 0);
        assert!(xs.is_empty());
    }

    #[test]
    fn dot_xs_single_dot() {
        let xs = dot_xs(500.0, 295.0, SS, 1);
        assert_eq!(xs.len(), 1);
        let expected = first_dot_x(500.0, 295.0, SS);
        assert!((xs[0] - expected).abs() < 1e-6);
    }

    #[test]
    fn dot_xs_double_dot() {
        let xs = dot_xs(500.0, 295.0, SS, 2);
        assert_eq!(xs.len(), 2);
        let first = first_dot_x(500.0, 295.0, SS);
        let inter = DOT_INTER_DOT_SPACING_SS * SS;
        assert!((xs[0] - first).abs() < 1e-6);
        assert!((xs[1] - (first + inter)).abs() < 1e-6, "got {}", xs[1]);
    }

    #[test]
    fn dot_xs_triple_dot() {
        let xs = dot_xs(500.0, 295.0, SS, 3);
        assert_eq!(xs.len(), 3);
        let inter = DOT_INTER_DOT_SPACING_SS * SS;
        // Each successive dot is `inter` further right
        assert!((xs[1] - xs[0] - inter).abs() < 1e-6);
        assert!((xs[2] - xs[1] - inter).abs() < 1e-6);
    }

    #[test]
    fn dot_xs_are_monotonically_increasing() {
        let xs = dot_xs(0.0, 295.0, SS, 3);
        for window in xs.windows(2) {
            assert!(window[1] > window[0]);
        }
    }

    // --- dot_staff_position ---

    #[test]
    fn on_bottom_line_shifts_up() {
        // Position 0 (bottom line) → 1 (first space)
        assert_eq!(dot_staff_position(0), 1);
    }

    #[test]
    fn in_first_space_stays() {
        // Position 1 (first space) → 1
        assert_eq!(dot_staff_position(1), 1);
    }

    #[test]
    fn on_second_line_shifts_up() {
        // Position 2 (second line) → 3 (second space)
        assert_eq!(dot_staff_position(2), 3);
    }

    #[test]
    fn on_middle_line_shifts_up() {
        // Position 4 (middle line) → 5
        assert_eq!(dot_staff_position(4), 5);
    }

    #[test]
    fn on_top_line_shifts_up() {
        // Position 8 (top line) → 9 (just above staff)
        assert_eq!(dot_staff_position(8), 9);
    }

    #[test]
    fn in_top_space_stays() {
        // Position 7 (top space) → 7
        assert_eq!(dot_staff_position(7), 7);
    }

    #[test]
    fn below_staff_on_ledger_line_shifts_up() {
        // Position -2 (first ledger line below) → -1
        assert_eq!(dot_staff_position(-2), -1);
    }

    #[test]
    fn below_staff_in_space_stays() {
        // Position -1 (space below staff) → -1
        assert_eq!(dot_staff_position(-1), -1);
    }

    #[test]
    fn above_staff_on_ledger_line_shifts_up() {
        // Position 10 (first ledger line above) → 11
        assert_eq!(dot_staff_position(10), 11);
    }

    #[test]
    fn above_staff_in_space_stays() {
        // Position 9 → 9
        assert_eq!(dot_staff_position(9), 9);
    }

    #[test]
    fn negative_odd_position_stays() {
        // Position -3 (odd, in a space) → -3
        assert_eq!(dot_staff_position(-3), -3);
    }

    #[test]
    fn negative_even_position_shifts_up() {
        // Position -4 → -3
        assert_eq!(dot_staff_position(-4), -3);
    }
}
