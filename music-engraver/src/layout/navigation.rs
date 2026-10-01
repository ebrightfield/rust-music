/// Navigation sign layout — coda, segno, and related repeat navigation markers.
///
/// These glyphs are placed above the staff, centered horizontally on a note
/// or barline position. They use SMuFL glyphs from the "Repeats" range.
use smufl::Glyph;

use crate::layout::staff::StaffLayout;

/// Navigation sign type for repeat navigation markers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavigationSign {
    /// Segno (𝄋) — marks the beginning of a repeated section.
    Segno,
    /// Coda (𝄌) — marks the coda (ending section) jump target.
    Coda,
    /// Square coda — an alternate coda symbol with straight sides.
    CodaSquare,
}

impl NavigationSign {
    /// Return the SMuFL glyph for this navigation sign.
    pub fn glyph(self) -> Glyph {
        match self {
            Self::Segno => Glyph::Segno,
            Self::Coda => Glyph::Coda,
            Self::CodaSquare => Glyph::CodaSquare,
        }
    }

    /// All navigation sign variants, useful for iteration in tests.
    pub fn all() -> &'static [NavigationSign] {
        &[Self::Segno, Self::Coda, Self::CodaSquare]
    }
}

/// Computed navigation sign position.
#[derive(Clone, Debug)]
pub struct NavigationSignLayout {
    /// X-coordinate (centered on the notehead or barline).
    pub x: f64,
    /// Y-coordinate of the glyph anchor (above the staff).
    pub y: f64,
    /// The SMuFL glyph to render.
    pub glyph: Glyph,
}

/// Distance (in staff spaces) between the top staff line and the navigation
/// sign glyph anchor. These signs are large and placed well above the staff
/// to avoid collision with notes, rehearsal marks, and other markings.
const NAVIGATION_ABOVE_STAFF_SS: f64 = 2.5;

/// Compute the position for a navigation sign above the staff.
///
/// Navigation signs are always placed above the top staff line, centered on
/// `x_center`. Unlike ornaments, they do not move up for high notes — they
/// occupy a fixed position above the staff since they are structural markers,
/// not note-level decorations.
pub fn layout_navigation_sign(
    sign: NavigationSign,
    x_center: f64,
    staff: &StaffLayout,
) -> NavigationSignLayout {
    let glyph = sign.glyph();
    let offset_fu = NAVIGATION_ABOVE_STAFF_SS * staff.staff_space;
    let y = staff.y_of(8) - offset_fu;

    NavigationSignLayout {
        x: x_center,
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
    fn segno_glyph() {
        assert_eq!(NavigationSign::Segno.glyph(), Glyph::Segno);
    }

    #[test]
    fn coda_glyph() {
        assert_eq!(NavigationSign::Coda.glyph(), Glyph::Coda);
    }

    #[test]
    fn coda_square_glyph() {
        assert_eq!(NavigationSign::CodaSquare.glyph(), Glyph::CodaSquare);
    }

    #[test]
    fn all_returns_three_variants() {
        assert_eq!(NavigationSign::all().len(), 3);
    }

    #[test]
    fn all_produce_distinct_glyphs() {
        let glyphs: Vec<_> = NavigationSign::all().iter().map(|s| s.glyph()).collect();
        for i in 0..glyphs.len() {
            for j in (i + 1)..glyphs.len() {
                assert_ne!(
                    glyphs[i],
                    glyphs[j],
                    "{:?} and {:?} should produce distinct glyphs",
                    NavigationSign::all()[i],
                    NavigationSign::all()[j]
                );
            }
        }
    }

    #[test]
    fn layout_above_top_staff_line() {
        let staff = test_staff();
        let layout = layout_navigation_sign(NavigationSign::Segno, 500.0, &staff);
        let top_line_y = staff.y_of(8);
        assert!(
            layout.y < top_line_y,
            "sign y ({}) should be above top staff line y ({})",
            layout.y,
            top_line_y
        );
    }

    #[test]
    fn x_center_preserved() {
        let staff = test_staff();
        let layout = layout_navigation_sign(NavigationSign::Coda, 1234.0, &staff);
        assert_eq!(layout.x, 1234.0);
    }

    #[test]
    fn glyph_matches_sign() {
        let staff = test_staff();
        let layout = layout_navigation_sign(NavigationSign::CodaSquare, 100.0, &staff);
        assert_eq!(layout.glyph, Glyph::CodaSquare);
    }

    #[test]
    fn different_x_produces_different_layout() {
        let staff = test_staff();
        let a = layout_navigation_sign(NavigationSign::Segno, 100.0, &staff);
        let b = layout_navigation_sign(NavigationSign::Segno, 900.0, &staff);
        assert!((a.x - b.x).abs() > 700.0);
    }

    #[test]
    fn y_is_fixed_regardless_of_sign_type() {
        let staff = test_staff();
        let segno = layout_navigation_sign(NavigationSign::Segno, 100.0, &staff);
        let coda = layout_navigation_sign(NavigationSign::Coda, 100.0, &staff);
        assert!(
            (segno.y - coda.y).abs() < f64::EPSILON,
            "all signs at same position should have same y: {} vs {}",
            segno.y,
            coda.y
        );
    }

    #[test]
    fn y_offset_is_2_5_staff_spaces() {
        let staff = test_staff();
        let layout = layout_navigation_sign(NavigationSign::Segno, 100.0, &staff);
        let expected_y = staff.y_of(8) - 2.5 * staff.staff_space;
        assert!(
            (layout.y - expected_y).abs() < 0.01,
            "y ({}) should be top_line_y - 2.5×ss ({})",
            layout.y,
            expected_y
        );
    }
}
