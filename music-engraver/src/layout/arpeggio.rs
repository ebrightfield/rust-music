use smufl::Glyph;

use crate::layout::staff::StaffLayout;

/// Arpeggio direction: upward (default) or downward roll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArpeggioDirection {
    /// Standard upward arpeggio (low to high).
    Up,
    /// Downward arpeggio (high to low) — indicated by a downward arrow.
    Down,
}

/// Layout result for an arpeggio wavy line placed before a chord.
#[derive(Clone, Debug)]
pub struct ArpeggioLayout {
    /// SMuFL glyph to render (repeated/scaled vertically).
    pub glyph: Glyph,
    /// X position (left of the chord noteheads).
    pub x: f64,
    /// Y coordinate of the top of the arpeggio line.
    pub y_top: f64,
    /// Y coordinate of the bottom of the arpeggio line.
    pub y_bottom: f64,
    /// Vertical scale factor to stretch the glyph to span the chord.
    pub scale_y: f64,
}

/// Padding between the arpeggio wavy line and the leftmost notehead,
/// in staff spaces.
pub const ARPEGGIO_PADDING_SS: f64 = 0.4;

/// Minimum span in staff spaces for the arpeggio line (used when a chord
/// has only two notes a second apart, or for single-note arpeggios).
const MIN_SPAN_SS: f64 = 2.0;

/// Compute the layout for an arpeggio wavy line to the left of a chord.
///
/// `staff_positions` are the vertical positions of all notes in the chord.
/// The arpeggio line spans from the lowest to the highest note, with a
/// minimum height to remain visually readable.
///
/// `notehead_x` is the leftmost x of the chord noteheads (before any
/// accidental offsets). The arpeggio is placed to its left with padding.
pub fn layout_arpeggio(
    direction: ArpeggioDirection,
    staff_positions: &[i8],
    notehead_x: f64,
    staff: &StaffLayout,
) -> Option<ArpeggioLayout> {
    if staff_positions.is_empty() {
        return None;
    }

    let glyph = match direction {
        ArpeggioDirection::Up => Glyph::ArpeggiatoUp,
        ArpeggioDirection::Down => Glyph::ArpeggiatoDown,
    };

    let (min_pos, max_pos) = if staff_positions.len() == 1 {
        // Single note: create a small arpeggio centered on the note
        let pos = staff_positions[0];
        (pos - 2, pos + 2)
    } else {
        let min = *staff_positions.iter().min().unwrap();
        let max = *staff_positions.iter().max().unwrap();
        (min, max)
    };

    let y_top = staff.y_of(max_pos);
    let y_bottom = staff.y_of(min_pos);
    let raw_span = y_bottom - y_top; // positive since y increases downward

    let ss = staff.staff_space;
    let min_span = MIN_SPAN_SS * ss;
    let span = if raw_span < min_span { min_span } else { raw_span };

    // The ArpeggiatoUp/Down glyphs in SMuFL are designed to span approximately
    // 1 staff space in their default size. We scale to cover the full chord span.
    let scale_y = span / ss;

    // Position to the left of the notehead
    let x = notehead_x - ARPEGGIO_PADDING_SS * ss;

    // Center the scaled glyph on the chord's vertical extent
    let center_y = (y_top + y_bottom) / 2.0;
    let actual_y_top = center_y - span / 2.0;
    let actual_y_bottom = center_y + span / 2.0;

    Some(ArpeggioLayout {
        glyph,
        x,
        y_top: actual_y_top,
        y_bottom: actual_y_bottom,
        scale_y,
    })
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
    fn up_uses_arpeggiato_up_glyph() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 4, 8], 500.0, &staff).unwrap();
        assert_eq!(layout.glyph, Glyph::ArpeggiatoUp);
    }

    #[test]
    fn down_uses_arpeggiato_down_glyph() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Down, &[0, 4, 8], 500.0, &staff).unwrap();
        assert_eq!(layout.glyph, Glyph::ArpeggiatoDown);
    }

    #[test]
    fn empty_positions_returns_none() {
        let staff = test_staff();
        assert!(layout_arpeggio(ArpeggioDirection::Up, &[], 500.0, &staff).is_none());
    }

    #[test]
    fn x_is_left_of_notehead() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 500.0, &staff).unwrap();
        assert!(layout.x < 500.0, "arpeggio x should be left of notehead x");
    }

    #[test]
    fn spans_from_lowest_to_highest_note() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 500.0, &staff).unwrap();
        let y_bottom_note = staff.y_of(0);
        let y_top_note = staff.y_of(8);
        // The layout should span at least from the top note to the bottom note
        assert!(layout.y_top <= y_top_note, "top should be at or above highest note");
        assert!(layout.y_bottom >= y_bottom_note, "bottom should be at or below lowest note");
    }

    #[test]
    fn single_note_gets_minimum_span() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[4], 500.0, &staff).unwrap();
        let span = layout.y_bottom - layout.y_top;
        let min = MIN_SPAN_SS * staff.staff_space;
        assert!(span >= min - 0.01, "single note should get at least minimum span");
    }

    #[test]
    fn wider_chord_produces_larger_scale() {
        let staff = test_staff();
        let narrow = layout_arpeggio(ArpeggioDirection::Up, &[2, 6], 500.0, &staff).unwrap();
        let wide = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 500.0, &staff).unwrap();
        assert!(
            wide.scale_y > narrow.scale_y,
            "wider chord should have larger scale_y"
        );
    }

    #[test]
    fn up_and_down_have_same_position_different_glyph() {
        let staff = test_staff();
        let up = layout_arpeggio(ArpeggioDirection::Up, &[0, 4, 8], 500.0, &staff).unwrap();
        let down = layout_arpeggio(ArpeggioDirection::Down, &[0, 4, 8], 500.0, &staff).unwrap();
        assert_ne!(up.glyph, down.glyph);
        assert!((up.x - down.x).abs() < 0.01, "same x position");
        assert!((up.y_top - down.y_top).abs() < 0.01, "same y_top");
    }

    #[test]
    fn scale_y_positive() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 500.0, &staff).unwrap();
        assert!(layout.scale_y > 0.0, "scale_y should be positive");
    }

    #[test]
    fn three_note_chord_spans_all() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 4, 8], 500.0, &staff).unwrap();
        let y_mid = staff.y_of(4);
        assert!(layout.y_top < y_mid, "top should be above middle note");
        assert!(layout.y_bottom > y_mid, "bottom should be below middle note");
    }

    #[test]
    fn padding_respects_staff_space() {
        let staff = test_staff();
        let layout = layout_arpeggio(ArpeggioDirection::Up, &[0, 8], 1000.0, &staff).unwrap();
        let expected_x = 1000.0 - ARPEGGIO_PADDING_SS * staff.staff_space;
        assert!(
            (layout.x - expected_x).abs() < 0.01,
            "x should be notehead_x - padding"
        );
    }
}
