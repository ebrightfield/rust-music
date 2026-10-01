use crate::layout::stem::StemDirection;
use smufl::Glyph;

/// Number of flags (hooks) for a given duration.
/// Eighth = 1, sixteenth = 2, 32nd = 3, 64th = 4, 128th = 5.
/// Returns 0 for durations that don't have flags (whole, half, quarter).
pub fn flag_count(duration_flags: u8) -> u8 {
    duration_flags
}

/// Map a flag count (1–5) and stem direction to the appropriate SMuFL glyph.
///
/// Returns `None` for count 0 (no flag needed) or counts beyond 5.
/// SMuFL flag glyphs are designed for the corresponding stem direction —
/// up-flags attach at the stem tip going right, down-flags going left.
pub fn flag_glyph(flag_count: u8, direction: StemDirection) -> Option<Glyph> {
    match (flag_count, direction) {
        (1, StemDirection::Up) => Some(Glyph::Flag8thUp),
        (1, StemDirection::Down) => Some(Glyph::Flag8thDown),
        (2, StemDirection::Up) => Some(Glyph::Flag16thUp),
        (2, StemDirection::Down) => Some(Glyph::Flag16thDown),
        (3, StemDirection::Up) => Some(Glyph::Flag32ndUp),
        (3, StemDirection::Down) => Some(Glyph::Flag32ndDown),
        (4, StemDirection::Up) => Some(Glyph::Flag64thUp),
        (4, StemDirection::Down) => Some(Glyph::Flag64thDown),
        (5, StemDirection::Up) => Some(Glyph::Flag128thUp),
        (5, StemDirection::Down) => Some(Glyph::Flag128thDown),
        _ => None,
    }
}

/// Compute the flag glyph attachment point (x, y) relative to the notehead.
///
/// Flags are placed at the stem tip. The x-coordinate is the stem x
/// (right side for stem-up, left side for stem-down). The y-coordinate
/// is the stem tip y (top for stem-up, bottom for stem-down).
///
/// The caller provides `stem_tip_x` and `stem_tip_y` directly — this
/// function exists for documentation and to clarify the convention.
pub fn flag_position(stem_tip_x: f64, stem_tip_y: f64) -> (f64, f64) {
    (stem_tip_x, stem_tip_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- flag_glyph ---

    #[test]
    fn eighth_up_returns_flag_8th_up() {
        assert_eq!(flag_glyph(1, StemDirection::Up), Some(Glyph::Flag8thUp));
    }

    #[test]
    fn eighth_down_returns_flag_8th_down() {
        assert_eq!(flag_glyph(1, StemDirection::Down), Some(Glyph::Flag8thDown));
    }

    #[test]
    fn sixteenth_up_returns_flag_16th_up() {
        assert_eq!(flag_glyph(2, StemDirection::Up), Some(Glyph::Flag16thUp));
    }

    #[test]
    fn sixteenth_down_returns_flag_16th_down() {
        assert_eq!(
            flag_glyph(2, StemDirection::Down),
            Some(Glyph::Flag16thDown)
        );
    }

    #[test]
    fn thirty_second_up() {
        assert_eq!(flag_glyph(3, StemDirection::Up), Some(Glyph::Flag32ndUp));
    }

    #[test]
    fn thirty_second_down() {
        assert_eq!(
            flag_glyph(3, StemDirection::Down),
            Some(Glyph::Flag32ndDown)
        );
    }

    #[test]
    fn sixty_fourth_up() {
        assert_eq!(flag_glyph(4, StemDirection::Up), Some(Glyph::Flag64thUp));
    }

    #[test]
    fn sixty_fourth_down() {
        assert_eq!(
            flag_glyph(4, StemDirection::Down),
            Some(Glyph::Flag64thDown)
        );
    }

    #[test]
    fn one_twenty_eighth_up() {
        assert_eq!(flag_glyph(5, StemDirection::Up), Some(Glyph::Flag128thUp));
    }

    #[test]
    fn one_twenty_eighth_down() {
        assert_eq!(
            flag_glyph(5, StemDirection::Down),
            Some(Glyph::Flag128thDown)
        );
    }

    #[test]
    fn zero_flags_returns_none() {
        assert_eq!(flag_glyph(0, StemDirection::Up), None);
        assert_eq!(flag_glyph(0, StemDirection::Down), None);
    }

    #[test]
    fn six_flags_returns_none() {
        assert_eq!(flag_glyph(6, StemDirection::Up), None);
        assert_eq!(flag_glyph(6, StemDirection::Down), None);
    }

    // --- flag_position ---

    #[test]
    fn flag_position_returns_stem_tip() {
        let (x, y) = flag_position(780.0, 125.0);
        assert!((x - 780.0).abs() < f64::EPSILON);
        assert!((y - 125.0).abs() < f64::EPSILON);
    }

    // --- flag_glyph direction symmetry ---

    #[test]
    fn up_and_down_flags_differ_for_all_counts() {
        for count in 1..=5 {
            let up = flag_glyph(count, StemDirection::Up);
            let down = flag_glyph(count, StemDirection::Down);
            assert_ne!(
                up, down,
                "up and down flags should differ for count {count}"
            );
        }
    }
}
