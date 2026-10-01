use crate::layout::staff::StaffPosition;

/// Stem direction for a note or chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemDirection {
    Up,
    Down,
}

/// Default stem length in staff spaces (3.5 is the standard convention).
/// Notes far from the middle line may need longer stems — see `stem_length_staff_spaces`.
pub const DEFAULT_STEM_LENGTH_SS: f64 = 3.5;

/// Minimum stem length in staff spaces. Even notes far from the staff
/// should not have stems shorter than this.
pub const MIN_STEM_LENGTH_SS: f64 = 2.5;

/// Determine automatic stem direction for a single note based on its
/// staff position. Notes on or above the middle line (position 4) get
/// stems down; notes below get stems up.
pub fn auto_stem_direction(position: StaffPosition) -> StemDirection {
    if position >= 4 {
        StemDirection::Down
    } else {
        StemDirection::Up
    }
}

/// Determine automatic stem direction for a chord (multiple positions).
/// Uses the note farthest from the middle line: if the farthest note is
/// above the middle, stem goes down; if below, stem goes up. When
/// equidistant, stem goes down (convention).
pub fn auto_stem_direction_chord(positions: &[StaffPosition]) -> StemDirection {
    let Some((&first, rest)) = positions.split_first() else {
        return StemDirection::Up;
    };

    let (min, max) = rest
        .iter()
        .fold((first, first), |(lo, hi), &p| (lo.min(p), hi.max(p)));

    // Distance from middle line (position 4)
    let dist_above = max - 4; // positive if above
    let dist_below = 4 - min; // positive if below

    if dist_below > dist_above {
        StemDirection::Up
    } else {
        // Equal distance or farther above → stem down
        StemDirection::Down
    }
}

/// Compute stem length in staff spaces, extending beyond the default 3.5
/// when notes are far above or below the staff so the stem tip reaches
/// at least the middle line.
///
/// For stem-up notes at very low positions, the stem must extend up to
/// at least the middle line (position 4). For stem-down notes at very
/// high positions, the stem must reach down to at least the middle line.
pub fn stem_length_staff_spaces(position: StaffPosition, direction: StemDirection) -> f64 {
    let default_half_spaces = (DEFAULT_STEM_LENGTH_SS * 2.0) as i8; // 7 half-spaces

    let tip_position = match direction {
        StemDirection::Up => position + default_half_spaces,
        StemDirection::Down => position - default_half_spaces,
    };

    // If the stem tip already reaches past the middle line, use default length
    let needs_extension = match direction {
        StemDirection::Up => tip_position < 4, // tip should reach at least middle line
        StemDirection::Down => tip_position > 4, // tip should reach at least middle line
    };

    if needs_extension {
        // Extend so tip lands on the middle line (position 4)
        let half_spaces_needed = match direction {
            StemDirection::Up => 4 - position, // distance from note to middle line
            StemDirection::Down => position - 4,
        };
        let length = half_spaces_needed as f64 / 2.0;
        length.max(MIN_STEM_LENGTH_SS)
    } else {
        DEFAULT_STEM_LENGTH_SS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- auto_stem_direction (single note) ---

    #[test]
    fn below_middle_line_stem_up() {
        assert_eq!(auto_stem_direction(0), StemDirection::Up);
        assert_eq!(auto_stem_direction(1), StemDirection::Up);
        assert_eq!(auto_stem_direction(2), StemDirection::Up);
        assert_eq!(auto_stem_direction(3), StemDirection::Up);
    }

    #[test]
    fn middle_line_stem_down() {
        // B4 in treble (position 4 = middle line) conventionally gets stem down
        assert_eq!(auto_stem_direction(4), StemDirection::Down);
    }

    #[test]
    fn above_middle_line_stem_down() {
        assert_eq!(auto_stem_direction(5), StemDirection::Down);
        assert_eq!(auto_stem_direction(6), StemDirection::Down);
        assert_eq!(auto_stem_direction(8), StemDirection::Down);
        assert_eq!(auto_stem_direction(10), StemDirection::Down);
    }

    #[test]
    fn ledger_line_below_stem_up() {
        assert_eq!(auto_stem_direction(-2), StemDirection::Up);
        assert_eq!(auto_stem_direction(-6), StemDirection::Up);
    }

    // --- auto_stem_direction_chord ---

    #[test]
    fn chord_empty_defaults_up() {
        assert_eq!(auto_stem_direction_chord(&[]), StemDirection::Up);
    }

    #[test]
    fn chord_single_note_same_as_single() {
        assert_eq!(auto_stem_direction_chord(&[2]), StemDirection::Up);
        assert_eq!(auto_stem_direction_chord(&[6]), StemDirection::Down);
    }

    #[test]
    fn chord_farther_below_gets_stem_up() {
        // Notes at positions 0 and 6: distance below = 4, above = 2 → stem up
        assert_eq!(auto_stem_direction_chord(&[0, 6]), StemDirection::Up);
    }

    #[test]
    fn chord_farther_above_gets_stem_down() {
        // Notes at positions 2 and 8: distance below = 2, above = 4 → stem down
        assert_eq!(auto_stem_direction_chord(&[2, 8]), StemDirection::Down);
    }

    #[test]
    fn chord_equidistant_gets_stem_down() {
        // Notes at positions 2 and 6: both 2 away from middle → stem down
        assert_eq!(auto_stem_direction_chord(&[2, 6]), StemDirection::Down);
    }

    #[test]
    fn chord_symmetric_around_middle() {
        // 0 and 8: dist_below=4, dist_above=4 → tie → stem down
        assert_eq!(auto_stem_direction_chord(&[0, 8]), StemDirection::Down);
    }

    #[test]
    fn chord_three_notes() {
        // 1, 4, 5: farthest below = 4-1=3, farthest above = 5-4=1 → stem up
        assert_eq!(auto_stem_direction_chord(&[1, 4, 5]), StemDirection::Up);
    }

    // --- stem_length_staff_spaces ---

    #[test]
    fn default_length_for_middle_notes() {
        // Note on middle line, stem down: tip at 4 - 7 = -3, which is past middle
        // Actually for stem down from pos 4, tip = 4-7 = -3, that's below middle, OK
        assert!((stem_length_staff_spaces(4, StemDirection::Down) - 3.5).abs() < f64::EPSILON);
        assert!((stem_length_staff_spaces(3, StemDirection::Up) - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn default_length_for_staff_notes() {
        // E4 on treble = position 0, stem up: tip = 0+7 = 7, which is > 4, fine
        assert!((stem_length_staff_spaces(0, StemDirection::Up) - 3.5).abs() < f64::EPSILON);
        // F5 on treble = position 8, stem down: tip = 8-7 = 1, which is < 4, fine
        assert!((stem_length_staff_spaces(8, StemDirection::Down) - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn extended_stem_for_high_note_stem_down() {
        // Position 12 (two ledger lines above), stem down:
        // default tip = 12-7=5, which is > 4, so needs extension
        // need tip at 4: length = (12-4)/2 = 4.0 staff spaces
        assert!((stem_length_staff_spaces(12, StemDirection::Down) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn extended_stem_for_low_note_stem_up() {
        // Position -4, stem up:
        // default tip = -4+7=3, which is < 4, needs extension
        // need tip at 4: length = (4-(-4))/2 = 4.0
        assert!((stem_length_staff_spaces(-4, StemDirection::Up) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn minimum_stem_length_enforced() {
        // Position 5, stem up: tip = 5+7=12, fine → default 3.5
        // But position 5, stem down: tip = 5-7=-2, past middle → default 3.5
        // For minimum to kick in, we'd need a note very close to middle going wrong way
        // Position 5, stem up — 3.5 is already > min
        assert!(stem_length_staff_spaces(5, StemDirection::Up) >= MIN_STEM_LENGTH_SS);
        assert!(stem_length_staff_spaces(-10, StemDirection::Up) >= MIN_STEM_LENGTH_SS);
    }

    #[test]
    fn very_far_below_note_extends_stem() {
        // Position -10, stem up: tip = -10+7=-3, needs extension
        // length = (4-(-10))/2 = 7.0 staff spaces
        assert!((stem_length_staff_spaces(-10, StemDirection::Up) - 7.0).abs() < f64::EPSILON);
    }
}
