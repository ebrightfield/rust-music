/// Breath mark layout — comma, tick, and caesura markers placed above the
/// staff between notes.
///
/// Breath marks indicate a brief pause or lift. They are positioned above
/// the top staff line, slightly to the right of the note they follow
/// (between the current note and the next). SMuFL provides several variants;
/// the comma is by far the most common.
use smufl::Glyph;

use crate::layout::staff::StaffLayout;

/// Breath mark type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BreathMark {
    /// Comma breath (most common) — a short curved mark above the staff.
    Comma,
    /// Tick breath — a short diagonal stroke above the staff.
    Tick,
    /// Caesura (railroad tracks / "tram lines") — two diagonal lines
    /// indicating a longer break.
    Caesura,
}

impl BreathMark {
    /// Return the SMuFL glyph for this breath mark.
    pub fn glyph(self) -> Glyph {
        match self {
            Self::Comma => Glyph::BreathMarkComma,
            Self::Tick => Glyph::BreathMarkTick,
            Self::Caesura => Glyph::Caesura,
        }
    }

    /// All breath mark variants, useful for iteration in tests.
    pub fn all() -> &'static [BreathMark] {
        &[Self::Comma, Self::Tick, Self::Caesura]
    }
}

/// Computed breath mark position.
#[derive(Clone, Debug)]
pub struct BreathMarkLayout {
    /// X-coordinate (to the right of the note, between current and next).
    pub x: f64,
    /// Y-coordinate of the glyph anchor (above the staff).
    pub y: f64,
    /// The SMuFL glyph to render.
    pub glyph: Glyph,
}

/// Horizontal padding (in staff spaces) between the note's right edge and
/// the breath mark glyph. Positions the mark clearly after the note.
pub const BREATH_MARK_RIGHT_PADDING_SS: f64 = 0.5;

/// Distance (in staff spaces) above the top staff line for the breath mark.
/// Breath marks sit just above the staff — lower than rehearsal marks or
/// navigation signs since they're rhythmic rather than structural.
pub const BREATH_MARK_ABOVE_STAFF_SS: f64 = 1.5;

/// Compute the position for a breath mark above the staff.
///
/// `note_right_x` is the x-coordinate of the right edge of the notehead
/// (i.e., `note_x + advance_width`). The breath mark is placed to the right
/// of this position, above the top staff line.
pub fn layout_breath_mark(
    mark: BreathMark,
    note_right_x: f64,
    staff: &StaffLayout,
) -> BreathMarkLayout {
    let glyph = mark.glyph();
    let x = note_right_x + BREATH_MARK_RIGHT_PADDING_SS * staff.staff_space;
    let offset_fu = BREATH_MARK_ABOVE_STAFF_SS * staff.staff_space;
    let y = staff.y_of(8) - offset_fu;

    BreathMarkLayout { x, y, glyph }
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
    fn comma_glyph() {
        assert_eq!(BreathMark::Comma.glyph(), Glyph::BreathMarkComma);
    }

    #[test]
    fn tick_glyph() {
        assert_eq!(BreathMark::Tick.glyph(), Glyph::BreathMarkTick);
    }

    #[test]
    fn caesura_glyph() {
        assert_eq!(BreathMark::Caesura.glyph(), Glyph::Caesura);
    }

    #[test]
    fn all_returns_three_variants() {
        assert_eq!(BreathMark::all().len(), 3);
    }

    #[test]
    fn all_produce_distinct_glyphs() {
        let glyphs: Vec<_> = BreathMark::all().iter().map(|b| b.glyph()).collect();
        for i in 0..glyphs.len() {
            for j in (i + 1)..glyphs.len() {
                assert_ne!(
                    glyphs[i],
                    glyphs[j],
                    "{:?} and {:?} should produce distinct glyphs",
                    BreathMark::all()[i],
                    BreathMark::all()[j]
                );
            }
        }
    }

    #[test]
    fn layout_above_top_staff_line() {
        let staff = test_staff();
        let layout = layout_breath_mark(BreathMark::Comma, 500.0, &staff);
        let top_line_y = staff.y_of(8);
        assert!(
            layout.y < top_line_y,
            "breath mark y ({}) should be above top staff line y ({})",
            layout.y,
            top_line_y
        );
    }

    #[test]
    fn x_is_right_of_note() {
        let staff = test_staff();
        let note_right_x = 500.0;
        let layout = layout_breath_mark(BreathMark::Comma, note_right_x, &staff);
        assert!(
            layout.x > note_right_x,
            "breath mark x ({}) should be to the right of note_right_x ({})",
            layout.x,
            note_right_x
        );
    }

    #[test]
    fn x_offset_is_0_5_staff_spaces() {
        let staff = test_staff();
        let note_right_x = 500.0;
        let layout = layout_breath_mark(BreathMark::Tick, note_right_x, &staff);
        let expected_x = note_right_x + 0.5 * staff.staff_space;
        assert!(
            (layout.x - expected_x).abs() < 0.01,
            "x ({}) should be note_right_x + 0.5×ss ({})",
            layout.x,
            expected_x
        );
    }

    #[test]
    fn y_offset_is_1_5_staff_spaces() {
        let staff = test_staff();
        let layout = layout_breath_mark(BreathMark::Caesura, 100.0, &staff);
        let expected_y = staff.y_of(8) - 1.5 * staff.staff_space;
        assert!(
            (layout.y - expected_y).abs() < 0.01,
            "y ({}) should be top_line_y - 1.5×ss ({})",
            layout.y,
            expected_y
        );
    }

    #[test]
    fn glyph_matches_mark() {
        let staff = test_staff();
        let layout = layout_breath_mark(BreathMark::Caesura, 100.0, &staff);
        assert_eq!(layout.glyph, Glyph::Caesura);
    }

    #[test]
    fn different_note_x_produces_different_layout() {
        let staff = test_staff();
        let a = layout_breath_mark(BreathMark::Comma, 100.0, &staff);
        let b = layout_breath_mark(BreathMark::Comma, 900.0, &staff);
        assert!((a.x - b.x).abs() > 700.0);
    }

    #[test]
    fn y_is_fixed_regardless_of_mark_type() {
        let staff = test_staff();
        let comma = layout_breath_mark(BreathMark::Comma, 100.0, &staff);
        let tick = layout_breath_mark(BreathMark::Tick, 100.0, &staff);
        let caesura = layout_breath_mark(BreathMark::Caesura, 100.0, &staff);
        assert!(
            (comma.y - tick.y).abs() < f64::EPSILON,
            "comma ({}) and tick ({}) should have same y",
            comma.y,
            tick.y
        );
        assert!(
            (tick.y - caesura.y).abs() < f64::EPSILON,
            "tick ({}) and caesura ({}) should have same y",
            tick.y,
            caesura.y
        );
    }
}
