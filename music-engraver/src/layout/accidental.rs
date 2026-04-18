use music::note::spelling::Accidental;
use smufl::Glyph;

/// Map a `music::Accidental` to the corresponding SMuFL glyph.
///
/// Returns `None` for `Natural` when `show_natural` is false (the common case
/// in running notation where naturals are only shown to cancel a prior accidental).
pub fn accidental_glyph(acc: Accidental, show_natural: bool) -> Option<Glyph> {
    match acc {
        Accidental::Natural if show_natural => Some(Glyph::AccidentalNatural),
        Accidental::Natural => None,
        Accidental::Sharp => Some(Glyph::AccidentalSharp),
        Accidental::Flat => Some(Glyph::AccidentalFlat),
        Accidental::DoubleSharp => Some(Glyph::AccidentalDoubleSharp),
        Accidental::DoubleFlat => Some(Glyph::AccidentalDoubleFlat),
    }
}

/// Padding between the right edge of an accidental glyph and the left edge
/// of the notehead, in staff spaces.
///
/// Standard engraving practice places the accidental close to but not touching
/// the notehead. Typical values range from 0.1 to 0.2 staff spaces; we use
/// approximately 1/8 staff space (0.12).
pub const ACCIDENTAL_NOTEHEAD_PADDING_SS: f64 = 0.12;

/// Compute the x-position at which to draw an accidental, given the notehead's
/// x-position and the advance width of the accidental glyph.
///
/// The accidental is placed to the left of the notehead with standard padding.
/// All values are in font design units.
pub fn accidental_x(
    notehead_x: f64,
    accidental_advance_width: f64,
    staff_space: f64,
) -> f64 {
    let padding = ACCIDENTAL_NOTEHEAD_PADDING_SS * staff_space;
    notehead_x - accidental_advance_width - padding
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sharp_maps_to_smufl_sharp() {
        assert_eq!(
            accidental_glyph(Accidental::Sharp, false),
            Some(Glyph::AccidentalSharp)
        );
    }

    #[test]
    fn flat_maps_to_smufl_flat() {
        assert_eq!(
            accidental_glyph(Accidental::Flat, false),
            Some(Glyph::AccidentalFlat)
        );
    }

    #[test]
    fn double_sharp_maps_correctly() {
        assert_eq!(
            accidental_glyph(Accidental::DoubleSharp, false),
            Some(Glyph::AccidentalDoubleSharp)
        );
    }

    #[test]
    fn double_flat_maps_correctly() {
        assert_eq!(
            accidental_glyph(Accidental::DoubleFlat, false),
            Some(Glyph::AccidentalDoubleFlat)
        );
    }

    #[test]
    fn natural_hidden_by_default() {
        assert_eq!(accidental_glyph(Accidental::Natural, false), None);
    }

    #[test]
    fn natural_shown_when_requested() {
        assert_eq!(
            accidental_glyph(Accidental::Natural, true),
            Some(Glyph::AccidentalNatural)
        );
    }

    #[test]
    fn accidental_x_places_left_of_notehead() {
        let notehead_x = 500.0;
        let acc_width = 150.0;
        let staff_space = 250.0;
        let x = accidental_x(notehead_x, acc_width, staff_space);

        // Expected: 500 - 150 - (0.12 * 250) = 500 - 150 - 30 = 320
        assert!((x - 320.0).abs() < 1e-6, "got {x}");
    }

    #[test]
    fn accidental_x_with_zero_width() {
        let x = accidental_x(100.0, 0.0, 250.0);
        // 100 - 0 - 30 = 70
        assert!((x - 70.0).abs() < 1e-6, "got {x}");
    }

    #[test]
    fn accidental_x_increases_with_smaller_accidental() {
        let staff_space = 250.0;
        let x_wide = accidental_x(500.0, 200.0, staff_space);
        let x_narrow = accidental_x(500.0, 100.0, staff_space);
        // Narrower accidental → placed closer to notehead (higher x)
        assert!(x_narrow > x_wide);
    }

    #[test]
    fn accidental_x_padding_scales_with_staff_space() {
        let acc_width = 150.0;
        let x_small = accidental_x(500.0, acc_width, 200.0);
        let x_large = accidental_x(500.0, acc_width, 400.0);
        // Larger staff space → more padding → accidental farther left (lower x)
        assert!(x_small > x_large);
    }
}
