use smufl::Glyph;

use crate::layout::tab::TabStaffLayout;

/// Layout result for a natural harmonic indicator above a tab fret number.
///
/// Natural harmonics on guitar are produced by lightly touching the string
/// at specific fret positions (5, 7, 12, etc.). In tablature, they are
/// notated with a small diamond/circle symbol above the fret number.
#[derive(Clone, Debug)]
pub struct TabHarmonicLayout {
    /// X-coordinate (centered on the fret number).
    pub x: f64,
    /// Y-coordinate (above the fret number, offset from the string line).
    pub y: f64,
    /// The SMuFL glyph to render.
    pub glyph: Glyph,
    /// Scale factor relative to standard glyph size.
    pub scale: f64,
}

/// Distance above the string line for the harmonic indicator, in staff spaces.
///
/// Placed just above the fret number's background rect so it doesn't
/// overlap the number text.
pub const HARMONIC_ABOVE_FRET_SS: f64 = 0.9;

/// Scale factor for the harmonic glyph — smaller than full-size since
/// it's an annotation above a fret number, not a notehead.
pub const HARMONIC_GLYPH_SCALE: f64 = 0.6;

/// Compute layout for a natural harmonic indicator on a tab staff.
///
/// `string` is 1-based (1 = highest pitch = bottom line).
/// `x` is the horizontal position of the fret event.
pub fn layout_tab_harmonic(
    tab_staff: &TabStaffLayout,
    string: u8,
    x: f64,
) -> TabHarmonicLayout {
    let string_y = tab_staff.string_y(string);
    let offset = tab_staff.staff_space * HARMONIC_ABOVE_FRET_SS;
    // Place above the string line (negative y direction in SVG)
    let y = string_y - offset;

    TabHarmonicLayout {
        x,
        y,
        glyph: Glyph::StringsHarmonic,
        scale: HARMONIC_GLYPH_SCALE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::font::EngravingConfig;

    fn guitar_staff() -> (TabStaffLayout, EngravingConfig) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 0.0, 5000.0, &config);
        (staff, config)
    }

    #[test]
    fn harmonic_uses_strings_harmonic_glyph() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_harmonic(&staff, 1, 1000.0);
        assert_eq!(layout.glyph, Glyph::StringsHarmonic);
    }

    #[test]
    fn harmonic_x_matches_input() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_harmonic(&staff, 1, 1234.5);
        assert!((layout.x - 1234.5).abs() < f64::EPSILON);
    }

    #[test]
    fn harmonic_y_is_above_string_line() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_harmonic(&staff, 3, 500.0);
        let string_y = staff.string_y(3);
        assert!(
            layout.y < string_y,
            "harmonic indicator should be above the string line: y={} < string_y={}",
            layout.y, string_y
        );
    }

    #[test]
    fn harmonic_offset_scales_with_staff_space() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_harmonic(&staff, 1, 500.0);
        let string_y = staff.string_y(1);
        let expected_offset = staff.staff_space * HARMONIC_ABOVE_FRET_SS;
        let actual_offset = string_y - layout.y;
        assert!(
            (actual_offset - expected_offset).abs() < f64::EPSILON,
            "offset should be {} but got {}",
            expected_offset,
            actual_offset
        );
    }

    #[test]
    fn different_strings_produce_different_y() {
        let (staff, _) = guitar_staff();
        let l1 = layout_tab_harmonic(&staff, 1, 500.0);
        let l6 = layout_tab_harmonic(&staff, 6, 500.0);
        assert!(
            (l1.y - l6.y).abs() > f64::EPSILON,
            "harmonics on different strings should have different y"
        );
    }

    #[test]
    fn scale_is_correct() {
        let (staff, _) = guitar_staff();
        let layout = layout_tab_harmonic(&staff, 1, 500.0);
        assert!((layout.scale - HARMONIC_GLYPH_SCALE).abs() < f64::EPSILON);
    }

    #[test]
    fn nonzero_origin_shifts_y() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = TabStaffLayout::guitar(0.0, 500.0, 5000.0, &config);
        let layout = layout_tab_harmonic(&staff, 1, 100.0);
        // String 1 is at bottom: y_origin + height, so harmonic y should be offset from there
        assert!(layout.y > 400.0, "y should be shifted by nonzero origin, got {}", layout.y);
    }

    #[test]
    fn harmonic_on_string_1_is_near_bottom() {
        let (staff, _) = guitar_staff();
        let l1 = layout_tab_harmonic(&staff, 1, 500.0);
        let l6 = layout_tab_harmonic(&staff, 6, 500.0);
        // String 1 is bottom (highest y), string 6 is top (lowest y)
        assert!(l1.y > l6.y, "string 1 harmonic should be below string 6 harmonic");
    }
}
