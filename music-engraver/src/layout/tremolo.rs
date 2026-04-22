use smufl::Glyph;

use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;

/// Number of tremolo slashes (1–5) placed across a stem.
///
/// Single-note tremolos indicate rapid repetition: 1 slash = eighth-note
/// subdivision, 2 slashes = sixteenth, 3 slashes = thirty-second.
/// Values above 3 are rare but supported per SMuFL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TremoloCount {
    /// One slash — eighth-note tremolo.
    Single,
    /// Two slashes — sixteenth-note tremolo.
    Double,
    /// Three slashes — thirty-second tremolo.
    Triple,
}

impl TremoloCount {
    /// Return the SMuFL glyph for this tremolo count.
    pub fn glyph(self) -> Glyph {
        match self {
            Self::Single => Glyph::Tremolo1,
            Self::Double => Glyph::Tremolo2,
            Self::Triple => Glyph::Tremolo3,
        }
    }

    /// Number of slashes.
    pub fn count(self) -> u8 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
        }
    }
}

/// Computed layout for tremolo slashes on a stem.
#[derive(Clone, Debug)]
pub struct TremoloLayout {
    /// The SMuFL glyph to render (Tremolo1/2/3).
    pub glyph: Glyph,
    /// X-coordinate: centered on the stem.
    pub x: f64,
    /// Y-coordinate: centered on the stem between notehead and tip.
    pub y: f64,
}

/// Distance from the notehead center toward the stem tip, as a fraction
/// of stem length. The tremolo glyph is placed at this fraction along
/// the stem. 0.5 = midpoint, but convention places slashes slightly
/// closer to the notehead (~40% from notehead) to avoid collision with
/// beam areas and flag attachment points.
const TREMOLO_STEM_FRACTION: f64 = 0.4;

/// Compute layout for tremolo slashes on a note stem.
///
/// The glyph is centered horizontally on the stem x-coordinate and
/// vertically at `TREMOLO_STEM_FRACTION` of the way from the notehead
/// toward the stem tip.
pub fn layout_tremolo(
    count: TremoloCount,
    stem_x: f64,
    notehead_y: f64,
    stem_tip_y: f64,
    _staff: &StaffLayout,
    _direction: StemDirection,
) -> TremoloLayout {
    let glyph = count.glyph();

    // Interpolate between notehead and stem tip
    let y = notehead_y + TREMOLO_STEM_FRACTION * (stem_tip_y - notehead_y);

    TremoloLayout {
        glyph,
        x: stem_x,
        y,
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
    fn single_maps_to_tremolo1() {
        assert_eq!(TremoloCount::Single.glyph(), Glyph::Tremolo1);
    }

    #[test]
    fn double_maps_to_tremolo2() {
        assert_eq!(TremoloCount::Double.glyph(), Glyph::Tremolo2);
    }

    #[test]
    fn triple_maps_to_tremolo3() {
        assert_eq!(TremoloCount::Triple.glyph(), Glyph::Tremolo3);
    }

    #[test]
    fn count_values() {
        assert_eq!(TremoloCount::Single.count(), 1);
        assert_eq!(TremoloCount::Double.count(), 2);
        assert_eq!(TremoloCount::Triple.count(), 3);
    }

    #[test]
    fn all_three_glyphs_are_distinct() {
        let g1 = TremoloCount::Single.glyph();
        let g2 = TremoloCount::Double.glyph();
        let g3 = TremoloCount::Triple.glyph();
        assert_ne!(g1, g2);
        assert_ne!(g2, g3);
        assert_ne!(g1, g3);
    }

    #[test]
    fn layout_x_equals_stem_x() {
        let staff = test_staff();
        let layout = layout_tremolo(
            TremoloCount::Single,
            100.0,
            500.0,
            150.0,
            &staff,
            StemDirection::Up,
        );
        assert!((layout.x - 100.0).abs() < 0.001);
    }

    #[test]
    fn layout_y_between_notehead_and_tip_stem_up() {
        let staff = test_staff();
        // Stem up: tip is above (lower y), notehead is below (higher y)
        let notehead_y = 500.0;
        let tip_y = 150.0;
        let layout = layout_tremolo(
            TremoloCount::Double,
            100.0,
            notehead_y,
            tip_y,
            &staff,
            StemDirection::Up,
        );
        // y should be between tip and notehead
        assert!(layout.y < notehead_y, "y ({}) should be above notehead ({})", layout.y, notehead_y);
        assert!(layout.y > tip_y, "y ({}) should be below tip ({})", layout.y, tip_y);
    }

    #[test]
    fn layout_y_between_notehead_and_tip_stem_down() {
        let staff = test_staff();
        // Stem down: tip is below (higher y), notehead is above (lower y)
        let notehead_y = 150.0;
        let tip_y = 500.0;
        let layout = layout_tremolo(
            TremoloCount::Triple,
            100.0,
            notehead_y,
            tip_y,
            &staff,
            StemDirection::Down,
        );
        assert!(layout.y > notehead_y, "y ({}) should be below notehead ({})", layout.y, notehead_y);
        assert!(layout.y < tip_y, "y ({}) should be above tip ({})", layout.y, tip_y);
    }

    #[test]
    fn layout_y_at_expected_fraction() {
        let staff = test_staff();
        let notehead_y = 500.0;
        let tip_y = 150.0;
        let layout = layout_tremolo(
            TremoloCount::Single,
            100.0,
            notehead_y,
            tip_y,
            &staff,
            StemDirection::Up,
        );
        let expected = notehead_y + TREMOLO_STEM_FRACTION * (tip_y - notehead_y);
        assert!((layout.y - expected).abs() < 0.001);
    }

    #[test]
    fn different_counts_produce_different_glyphs_in_layout() {
        let staff = test_staff();
        let l1 = layout_tremolo(TremoloCount::Single, 100.0, 500.0, 200.0, &staff, StemDirection::Up);
        let l2 = layout_tremolo(TremoloCount::Double, 100.0, 500.0, 200.0, &staff, StemDirection::Up);
        let l3 = layout_tremolo(TremoloCount::Triple, 100.0, 500.0, 200.0, &staff, StemDirection::Up);
        assert_ne!(l1.glyph, l2.glyph);
        assert_ne!(l2.glyph, l3.glyph);
    }

    #[test]
    fn same_position_different_direction_same_y_computation() {
        let staff = test_staff();
        // With same notehead_y and tip_y, direction doesn't affect placement
        let l_up = layout_tremolo(TremoloCount::Single, 100.0, 500.0, 200.0, &staff, StemDirection::Up);
        let l_down = layout_tremolo(TremoloCount::Single, 100.0, 500.0, 200.0, &staff, StemDirection::Down);
        assert!((l_up.y - l_down.y).abs() < 0.001);
    }
}
