use crate::layout::staff::StaffLayout;
use crate::layout::StaffPosition;
use smufl::Glyph;

/// Maps a rest duration to its SMuFL glyph.
///
/// `log2_duration`: -1 = breve, 0 = whole, 1 = half, 2 = quarter, 3 = eighth,
/// 4 = sixteenth, 5 = 32nd, 6 = 64th, 7 = 128th.
pub fn rest_glyph(log2_duration: i8) -> Option<Glyph> {
    match log2_duration {
        -1 => Some(Glyph::RestDoubleWhole),
        0 => Some(Glyph::RestWhole),
        1 => Some(Glyph::RestHalf),
        2 => Some(Glyph::RestQuarter),
        3 => Some(Glyph::Rest8th),
        4 => Some(Glyph::Rest16th),
        5 => Some(Glyph::Rest32nd),
        6 => Some(Glyph::Rest64th),
        7 => Some(Glyph::Rest128th),
        _ => None,
    }
}

/// Default staff position for a rest glyph.
///
/// Rests are vertically centered on the staff. Breve, whole, and half rests
/// have special positions (breve fills the 3rd space between the 3rd and 4th
/// lines, whole hangs from 4th line, half sits on 3rd line); all others are
/// centered at the middle line.
pub fn rest_staff_position(log2_duration: i8) -> StaffPosition {
    match log2_duration {
        // Breve rest: SMuFL `restDoubleWhole` rises one staff space from its
        // origin, so anchoring on the 3rd line (position 4) fills the space up
        // to the 4th line.
        -1 => 4,
        // Whole rest hangs from the 4th line (position 6)
        0 => 6,
        // Half rest sits on the 3rd line (position 4)
        1 => 4,
        // All other rests centered at middle line
        _ => 4,
    }
}

/// Compute y-coordinate for a rest glyph.
///
/// Uses the staff's coordinate mapping to translate the rest's default
/// staff position to a y-coordinate.
pub fn rest_y(staff: &StaffLayout, log2_duration: i8) -> f64 {
    let pos = rest_staff_position(log2_duration);
    staff.y_of(pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    #[test]
    fn rest_glyph_breve_is_double_whole_rest() {
        assert_eq!(rest_glyph(-1), Some(Glyph::RestDoubleWhole));
        assert_ne!(rest_glyph(-1), rest_glyph(0));
    }

    #[test]
    fn rest_glyph_whole() {
        assert_eq!(rest_glyph(0), Some(Glyph::RestWhole));
    }

    #[test]
    fn rest_glyph_half() {
        assert_eq!(rest_glyph(1), Some(Glyph::RestHalf));
    }

    #[test]
    fn rest_glyph_quarter() {
        assert_eq!(rest_glyph(2), Some(Glyph::RestQuarter));
    }

    #[test]
    fn rest_glyph_eighth() {
        assert_eq!(rest_glyph(3), Some(Glyph::Rest8th));
    }

    #[test]
    fn rest_glyph_sixteenth() {
        assert_eq!(rest_glyph(4), Some(Glyph::Rest16th));
    }

    #[test]
    fn rest_glyph_32nd() {
        assert_eq!(rest_glyph(5), Some(Glyph::Rest32nd));
    }

    #[test]
    fn rest_glyph_64th() {
        assert_eq!(rest_glyph(6), Some(Glyph::Rest64th));
    }

    #[test]
    fn rest_glyph_128th() {
        assert_eq!(rest_glyph(7), Some(Glyph::Rest128th));
    }

    #[test]
    fn rest_glyph_invalid_returns_none() {
        assert_eq!(rest_glyph(8), None);
        assert_eq!(rest_glyph(i8::MAX), None);
        assert_eq!(rest_glyph(-2), None);
    }

    #[test]
    fn all_rest_glyphs_are_distinct() {
        let glyphs: Vec<Glyph> = (-1..=7).filter_map(rest_glyph).collect();
        assert_eq!(glyphs.len(), 9);
        for i in 0..glyphs.len() {
            for j in (i + 1)..glyphs.len() {
                assert_ne!(glyphs[i], glyphs[j]);
            }
        }
    }

    #[test]
    fn breve_rest_fills_space_between_third_and_fourth_lines() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = crate::layout::staff::StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        let bbox = font
            .glyph_bbox_design_units(Glyph::RestDoubleWhole)
            .expect("Bravura provides restDoubleWhole metrics");
        let origin_y = rest_y(&staff, -1);
        assert_eq!(rest_staff_position(-1), 4);
        assert!((origin_y + bbox.y_bottom - staff.y_of(4)).abs() < 1e-9);
        assert!((origin_y + bbox.y_top - staff.y_of(6)).abs() < 1e-9);
    }

    #[test]
    fn whole_rest_position_is_6() {
        assert_eq!(rest_staff_position(0), 6);
    }

    #[test]
    fn half_rest_position_is_4() {
        assert_eq!(rest_staff_position(1), 4);
    }

    #[test]
    fn quarter_and_smaller_rest_position_is_4() {
        for d in 2..=7 {
            assert_eq!(rest_staff_position(d), 4, "log2_duration={d}");
        }
    }

    #[test]
    fn rest_y_uses_staff_position() {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = crate::layout::staff::StaffLayout::from_config(0.0, 0.0, 5000.0, &config);

        // Whole rest at pos 6: y = (8-6)*125 = 250
        let y_whole = rest_y(&staff, 0);
        assert!((y_whole - staff.y_of(6)).abs() < f64::EPSILON);

        // Quarter rest at pos 4: y = (8-4)*125 = 500
        let y_quarter = rest_y(&staff, 2);
        assert!((y_quarter - staff.y_of(4)).abs() < f64::EPSILON);
    }

    #[test]
    fn all_rest_glyphs_exist_in_bravura() {
        let font = bravura_font();
        for d in -1..=7 {
            let glyph = rest_glyph(d).unwrap();
            assert!(
                font.glyph_outline(glyph).is_ok(),
                "rest glyph for log2_duration={d} should exist in Bravura"
            );
        }
    }
}
