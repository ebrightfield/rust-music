//! Rhythm notation above tablature staves.
//!
//! Guitar tablature often shows rhythm stems and flags above the staff
//! to indicate duration. Stems always point up from a configurable position
//! above the top staff line.

use super::tab::TabStaffLayout;

/// Vertical geometry of TAB rhythm stems, measured in staff spaces.
///
/// Both values are relative to the TAB staff's top line. The same geometry
/// applies to standalone stems and to the stems joined by TAB beams.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabRhythmStyle {
    stem_base_above_staff_ss: f64,
    stem_length_ss: f64,
}

impl TabRhythmStyle {
    /// Create a style with a nonnegative base distance and a positive stem
    /// length. Non-finite values are rejected so page bounds remain valid.
    pub fn new(stem_base_above_staff_ss: f64, stem_length_ss: f64) -> Option<Self> {
        (stem_base_above_staff_ss.is_finite()
            && stem_base_above_staff_ss >= 0.0
            && stem_length_ss.is_finite()
            && stem_length_ss > 0.0
            && (stem_base_above_staff_ss + stem_length_ss).is_finite())
        .then_some(Self {
            stem_base_above_staff_ss,
            stem_length_ss,
        })
    }

    /// Distance from the top staff line to the stem base.
    pub fn stem_base_above_staff_ss(self) -> f64 {
        self.stem_base_above_staff_ss
    }

    /// Length from stem base to tip.
    pub fn stem_length_ss(self) -> f64 {
        self.stem_length_ss
    }

    /// Distance from the top staff line to the stem tip.
    pub fn above_staff_reach_ss(self) -> f64 {
        self.stem_base_above_staff_ss + self.stem_length_ss
    }
}

impl Default for TabRhythmStyle {
    fn default() -> Self {
        Self {
            stem_base_above_staff_ss: 1.5,
            stem_length_ss: 3.0,
        }
    }
}

/// Layout result for a rhythm stem above a tab staff.
#[derive(Clone, Debug)]
pub struct TabRhythmLayout {
    /// X-coordinate of the stem (centered on the fret number position).
    pub x: f64,
    /// Y of the stem base (bottom, closer to the staff).
    pub y_base: f64,
    /// Y of the stem tip (top, farther from the staff).
    pub y_tip: f64,
    /// Stem stroke width in font design units.
    pub stem_width: f64,
    /// Duration log2 (-1=breve, 0=whole, 1=half, 2=quarter, 3=eighth, etc.)
    pub duration_log2: i8,
    /// Number of flags (0 for quarter and longer, 1 for eighth, 2 for sixteenth, etc.)
    pub flag_count: u8,
}

/// Compute the number of flags for a given duration_log2.
///
/// -1 (breve) → 0, 0 (whole) → 0, 1 (half) → 0, 2 (quarter) → 0,
/// 3 (eighth) → 1, 4 (sixteenth) → 2, 5 (32nd) → 3, etc.
pub fn tab_flag_count(duration_log2: i8) -> u8 {
    u8::try_from(duration_log2.saturating_sub(2)).unwrap_or(0)
}

/// Whether a duration_log2 needs a stem drawn.
///
/// Breves (-1) and whole notes (0) have no stem. Half notes and shorter have stems.
pub fn needs_stem(duration_log2: i8) -> bool {
    duration_log2 >= 1
}

/// Compute rhythm stem layout above a tab staff.
///
/// The stem base is `tab_staff.rhythm_style.stem_base_above_staff_ss()`
/// staff spaces above the top line. The tip is another
/// `tab_staff.rhythm_style.stem_length_ss()` spaces higher (SVG y-down).
///
/// Returns `None` for breves and whole notes (duration_log2 <= 0), which have no stem.
pub fn layout_tab_rhythm(
    tab_staff: &TabStaffLayout,
    note_x: f64,
    duration_log2: i8,
    stem_width: f64,
) -> Option<TabRhythmLayout> {
    if !needs_stem(duration_log2) {
        return None;
    }

    let ss = tab_staff.staff_space;
    let y_base = tab_staff.y_origin - tab_staff.rhythm_style.stem_base_above_staff_ss * ss;
    let y_tip = y_base - tab_staff.rhythm_style.stem_length_ss * ss;

    Some(TabRhythmLayout {
        x: note_x,
        y_base,
        y_tip,
        stem_width,
        duration_log2,
        flag_count: tab_flag_count(duration_log2),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn guitar_staff() -> TabStaffLayout {
        let font = bravura_font();
        let config = font.engraving_config();
        TabStaffLayout::guitar(0.0, 500.0, 5000.0, &config)
    }

    #[test]
    fn tab_flag_count_breve_is_zero() {
        assert_eq!(tab_flag_count(-1), 0);
    }

    #[test]
    fn tab_flag_count_whole() {
        assert_eq!(tab_flag_count(0), 0);
    }

    #[test]
    fn tab_flag_count_half() {
        assert_eq!(tab_flag_count(1), 0);
    }

    #[test]
    fn tab_flag_count_quarter() {
        assert_eq!(tab_flag_count(2), 0);
    }

    #[test]
    fn tab_flag_count_eighth() {
        assert_eq!(tab_flag_count(3), 1);
    }

    #[test]
    fn tab_flag_count_sixteenth() {
        assert_eq!(tab_flag_count(4), 2);
    }

    #[test]
    fn tab_flag_count_thirty_second() {
        assert_eq!(tab_flag_count(5), 3);
    }

    #[test]
    fn breve_has_no_tab_rhythm_stem() {
        let staff = guitar_staff();
        assert!(!needs_stem(-1));
        assert!(layout_tab_rhythm(&staff, 1000.0, -1, 5.0).is_none());
    }

    #[test]
    fn needs_stem_whole_is_false() {
        assert!(!needs_stem(0));
    }

    #[test]
    fn needs_stem_half_is_true() {
        assert!(needs_stem(1));
    }

    #[test]
    fn needs_stem_quarter_is_true() {
        assert!(needs_stem(2));
    }

    #[test]
    fn whole_note_returns_none() {
        let staff = guitar_staff();
        assert!(layout_tab_rhythm(&staff, 1000.0, 0, 5.0).is_none());
    }

    #[test]
    fn quarter_note_returns_some() {
        let staff = guitar_staff();
        let layout = layout_tab_rhythm(&staff, 1000.0, 2, 5.0);
        assert!(layout.is_some());
        let l = layout.unwrap();
        assert_eq!(l.flag_count, 0);
        assert_eq!(l.duration_log2, 2);
    }

    #[test]
    fn eighth_note_has_one_flag() {
        let staff = guitar_staff();
        let l = layout_tab_rhythm(&staff, 1000.0, 3, 5.0).unwrap();
        assert_eq!(l.flag_count, 1);
    }

    #[test]
    fn stem_is_above_staff() {
        let staff = guitar_staff();
        let l = layout_tab_rhythm(&staff, 1000.0, 2, 5.0).unwrap();
        // y_base should be above (less than) y_origin
        assert!(
            l.y_base < staff.y_origin,
            "stem base ({}) should be above staff top ({})",
            l.y_base,
            staff.y_origin
        );
    }

    #[test]
    fn tip_is_above_base() {
        let staff = guitar_staff();
        let l = layout_tab_rhythm(&staff, 1000.0, 2, 5.0).unwrap();
        // SVG y: smaller = higher
        assert!(
            l.y_tip < l.y_base,
            "stem tip ({}) should be above base ({})",
            l.y_tip,
            l.y_base
        );
    }

    #[test]
    fn stem_length_is_correct() {
        let staff = guitar_staff();
        let l = layout_tab_rhythm(&staff, 1000.0, 2, 5.0).unwrap();
        let expected_length = TabRhythmStyle::default().stem_length_ss() * staff.staff_space;
        let actual_length = l.y_base - l.y_tip;
        assert!(
            (actual_length - expected_length).abs() < 0.01,
            "stem length {actual_length} should be {expected_length}"
        );
    }

    #[test]
    fn custom_style_moves_base_and_tip_without_changing_beam_membership() {
        let mut staff = guitar_staff();
        staff.rhythm_style = TabRhythmStyle::new(2.0, 4.0).unwrap();
        let layout = layout_tab_rhythm(&staff, 1000.0, 4, 5.0).unwrap();
        assert_eq!(layout.y_base, staff.y_origin - 2.0 * staff.staff_space);
        assert_eq!(layout.y_tip, staff.y_origin - 6.0 * staff.staff_space);
        assert_eq!(layout.flag_count, 2);
    }

    #[test]
    fn invalid_style_cannot_produce_nonfinite_bounds() {
        for (base, length) in [
            (f64::NAN, 3.0),
            (f64::INFINITY, 3.0),
            (-1.0, 3.0),
            (1.0, 0.0),
            (1.0, f64::NEG_INFINITY),
            (f64::MAX, f64::MAX),
        ] {
            assert!(TabRhythmStyle::new(base, length).is_none());
        }
    }

    #[test]
    fn x_matches_input() {
        let staff = guitar_staff();
        let l = layout_tab_rhythm(&staff, 2345.0, 3, 5.0).unwrap();
        assert!((l.x - 2345.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stem_width_preserved() {
        let staff = guitar_staff();
        let l = layout_tab_rhythm(&staff, 1000.0, 2, 7.5).unwrap();
        assert!((l.stem_width - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn half_note_has_stem_no_flags() {
        let staff = guitar_staff();
        let l = layout_tab_rhythm(&staff, 1000.0, 1, 5.0).unwrap();
        assert_eq!(l.flag_count, 0);
        assert_eq!(l.duration_log2, 1);
    }
}
