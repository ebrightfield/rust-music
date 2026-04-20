//! Chord symbol layout — harmony labels above the staff.
//!
//! Standard engraving convention: chord symbols (e.g. "Cmaj7", "Am", "F#dim")
//! appear above the staff in a bold sans-serif or serif font, centered on the
//! beat they apply to. Positioned above rehearsal marks and tempo markings
//! when those are absent, or at a consistent height above the staff.

use crate::layout::staff::StaffLayout;

/// Result of laying out a chord symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordSymbolLayout {
    /// The chord symbol text content (e.g. "Cmaj7", "Am").
    pub text: String,
    /// Center x-position (aligned with the note/beat it applies to).
    pub x_center: f64,
    /// Baseline y-position of the text (above the staff).
    pub y_baseline: f64,
    /// Font size in font design units.
    pub font_size: f64,
}

/// Distance above the top staff line for chord symbol placement, in staff spaces.
/// Above rehearsal marks (2.5ss) to avoid collision. Chord symbols are the
/// topmost text layer in standard engraving.
pub const CHORD_SYMBOL_ABOVE_STAFF_SS: f64 = 3.5;

/// Font size for chord symbols, in staff spaces.
const CHORD_SYMBOL_FONT_SIZE_SS: f64 = 1.6;

/// Lay out a chord symbol above the staff.
///
/// `text` is the chord symbol content (e.g. "Cmaj7", "Am7", "F#dim").
/// `note_center_x` is the horizontal center of the note/beat it applies to.
/// `staff` provides vertical reference for placement above the top staff line.
/// `staff_space` is the staff space size in font design units.
pub fn layout_chord_symbol(
    text: &str,
    note_center_x: f64,
    staff: &StaffLayout,
    staff_space: f64,
) -> ChordSymbolLayout {
    let font_size = CHORD_SYMBOL_FONT_SIZE_SS * staff_space;
    let above_offset = CHORD_SYMBOL_ABOVE_STAFF_SS * staff_space;

    // Top staff line y (smaller y = higher in SVG)
    let top_line_y = staff.y_of(8);
    let y_baseline = top_line_y - above_offset;

    ChordSymbolLayout {
        text: text.to_string(),
        x_center: note_center_x,
        y_baseline,
        font_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    #[test]
    fn text_content_preserved() {
        let layout = layout_chord_symbol("Cmaj7", 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "Cmaj7");
    }

    #[test]
    fn x_center_preserved() {
        let layout = layout_chord_symbol("Am", 1234.0, &test_staff(), 250.0);
        assert_eq!(layout.x_center, 1234.0);
    }

    #[test]
    fn baseline_above_top_staff_line() {
        let staff = test_staff();
        let top_y = staff.y_of(8);
        let layout = layout_chord_symbol("G7", 0.0, &staff, 250.0);
        assert!(
            layout.y_baseline < top_y,
            "baseline {} should be above top line {}",
            layout.y_baseline,
            top_y
        );
    }

    #[test]
    fn font_size_scales_with_staff_space() {
        let small = layout_chord_symbol("C", 0.0, &test_staff(), 125.0);
        let large = layout_chord_symbol("C", 0.0, &test_staff(), 250.0);
        assert!(
            (large.font_size - 2.0 * small.font_size).abs() < 0.01,
            "font size should scale linearly with staff_space"
        );
    }

    #[test]
    fn different_positions_produce_different_layouts() {
        let a = layout_chord_symbol("C", 100.0, &test_staff(), 250.0);
        let b = layout_chord_symbol("C", 900.0, &test_staff(), 250.0);
        assert!((a.x_center - b.x_center).abs() > 700.0);
    }

    #[test]
    fn font_size_is_positive() {
        let layout = layout_chord_symbol("X", 0.0, &test_staff(), 250.0);
        assert!(layout.font_size > 0.0);
    }

    #[test]
    fn above_staff_offset_is_correct() {
        let staff = test_staff();
        let top_y = staff.y_of(8);
        let layout = layout_chord_symbol("Dm", 0.0, &staff, 250.0);
        let expected = top_y - CHORD_SYMBOL_ABOVE_STAFF_SS * 250.0;
        assert!(
            (layout.y_baseline - expected).abs() < 0.01,
            "expected y={expected}, got {}",
            layout.y_baseline
        );
    }

    #[test]
    fn different_texts_produce_different_layouts() {
        let a = layout_chord_symbol("C", 500.0, &test_staff(), 250.0);
        let b = layout_chord_symbol("Am7", 500.0, &test_staff(), 250.0);
        assert_ne!(a.text, b.text);
    }

    #[test]
    fn empty_text_allowed() {
        let layout = layout_chord_symbol("", 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "");
    }

    #[test]
    fn complex_symbol_preserved() {
        let layout = layout_chord_symbol("F#m7b5", 500.0, &test_staff(), 250.0);
        assert_eq!(layout.text, "F#m7b5");
    }

    #[test]
    fn higher_than_rehearsal_marks() {
        // Chord symbols at 3.5ss above vs rehearsal at 2.5ss above
        let staff = test_staff();
        let ss = 250.0;
        let chord = layout_chord_symbol("C", 500.0, &staff, ss);
        let rehearsal_y = staff.y_of(8) - 2.5 * ss;
        assert!(
            chord.y_baseline < rehearsal_y,
            "chord symbol y {} should be higher (smaller) than rehearsal y {}",
            chord.y_baseline,
            rehearsal_y
        );
    }
}
