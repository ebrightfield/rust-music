use crate::layout::staff::StaffPosition;
use crate::layout::stem::StemDirection;

/// A single note within a chord, carrying its staff position and optional accidental glyph.
#[derive(Clone, Debug)]
pub struct ChordNote {
    /// Staff position (bottom line = 0).
    pub staff_position: StaffPosition,
    /// Accidental glyph to display, if any.
    pub accidental: Option<smufl::Glyph>,
}

/// Layout result for a single note within a chord, indicating whether its
/// notehead should be offset to the opposite side of the stem.
#[derive(Clone, Debug, PartialEq)]
pub struct ChordNoteLayout {
    /// Staff position of this note.
    pub staff_position: StaffPosition,
    /// Whether the notehead is offset to the far side of the stem.
    /// For stem-up: offset means to the right of the stem.
    /// For stem-down: offset means to the left of the stem.
    pub offset: bool,
    /// Accidental glyph, if any.
    pub accidental: Option<smufl::Glyph>,
}

/// Determine which noteheads in a chord need to be offset to avoid collision.
///
/// When two notes are a second apart (adjacent staff positions), their noteheads
/// overlap. The convention is:
/// - **Stem up**: scan from bottom; when a second is found, the upper note
///   moves to the right of the stem.
/// - **Stem down**: scan from top; when a second is found, the lower note
///   moves to the left of the stem.
///
/// Within a cluster of consecutive seconds, offsets alternate so that no two
/// adjacent noteheads share the same column.
///
/// Returns notes sorted bottom-to-top regardless of stem direction.
pub fn layout_chord_noteheads(
    notes: &[ChordNote],
    direction: StemDirection,
) -> Vec<ChordNoteLayout> {
    if notes.is_empty() {
        return Vec::new();
    }

    // Sort by staff position (bottom to top)
    let mut sorted: Vec<&ChordNote> = notes.iter().collect();
    sorted.sort_by_key(|n| n.staff_position);

    let mut result: Vec<ChordNoteLayout> = Vec::with_capacity(sorted.len());

    match direction {
        StemDirection::Up => {
            // Scan bottom-to-top. When a second is found, the upper note is offset.
            // Within a cluster, alternate: first note normal, second offset, third normal, etc.
            let mut i = 0;
            while i < sorted.len() {
                let is_second = i > 0
                    && sorted[i].staff_position - sorted[i - 1].staff_position == 1;

                if is_second {
                    // Previous note was normal → this one is offset
                    // But if previous was already offset, this one goes back to normal
                    let prev_offset = result.last().is_some_and(|r| r.offset);
                    result.push(ChordNoteLayout {
                        staff_position: sorted[i].staff_position,
                        offset: !prev_offset,
                        accidental: sorted[i].accidental,
                    });
                } else {
                    result.push(ChordNoteLayout {
                        staff_position: sorted[i].staff_position,
                        offset: false,
                        accidental: sorted[i].accidental,
                    });
                }
                i += 1;
            }
        }
        StemDirection::Down => {
            // Scan top-to-bottom. When a second is found, the lower note is offset.
            // Build in reverse, then flip.
            let n = sorted.len();
            let mut offsets = vec![false; n];

            let mut j = n;
            while j > 0 {
                j -= 1;
                let is_second =
                    j + 1 < n && sorted[j + 1].staff_position - sorted[j].staff_position == 1;

                if is_second {
                    let next_offset = offsets[j + 1];
                    offsets[j] = !next_offset;
                }
                // else: no second → offset stays false
            }

            for (idx, note) in sorted.iter().enumerate() {
                result.push(ChordNoteLayout {
                    staff_position: note.staff_position,
                    offset: offsets[idx],
                    accidental: note.accidental,
                });
            }
        }
    }

    result
}

/// Compute the x-offset (in notehead-widths) for an offset notehead.
///
/// Returns `0.0` for non-offset noteheads, `+1.0` for stem-up offset (right),
/// `-1.0` for stem-down offset (left). Caller multiplies by actual notehead width.
pub fn notehead_x_offset(offset: bool, direction: StemDirection) -> f64 {
    if !offset {
        return 0.0;
    }
    match direction {
        StemDirection::Up => 1.0,
        StemDirection::Down => -1.0,
    }
}

/// Check whether any note in the chord has an offset notehead,
/// which affects the overall width of the chord column.
pub fn chord_has_offsets(layouts: &[ChordNoteLayout]) -> bool {
    layouts.iter().any(|n| n.offset)
}

/// Get the lowest and highest staff positions in a chord (for stem computation).
/// Returns `None` for empty input.
pub fn chord_extent(notes: &[ChordNote]) -> Option<(StaffPosition, StaffPosition)> {
    let (first, rest) = notes.split_first()?;
    let (min, max) = rest.iter().fold(
        (first.staff_position, first.staff_position),
        |(lo, hi), n| (lo.min(n.staff_position), hi.max(n.staff_position)),
    );
    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(pos: StaffPosition) -> ChordNote {
        ChordNote {
            staff_position: pos,
            accidental: None,
        }
    }

    fn note_with_acc(pos: StaffPosition, acc: smufl::Glyph) -> ChordNote {
        ChordNote {
            staff_position: pos,
            accidental: Some(acc),
        }
    }

    // --- layout_chord_noteheads: empty ---

    #[test]
    fn empty_chord_returns_empty() {
        let result = layout_chord_noteheads(&[], StemDirection::Up);
        assert!(result.is_empty());
    }

    // --- single note ---

    #[test]
    fn single_note_no_offset() {
        let result = layout_chord_noteheads(&[note(4)], StemDirection::Up);
        assert_eq!(result.len(), 1);
        assert!(!result[0].offset);
        assert_eq!(result[0].staff_position, 4);
    }

    // --- two notes, no second ---

    #[test]
    fn two_notes_third_apart_no_offset_up() {
        // C4 (pos 0) and E4 (pos 2) — a third, no collision
        let result = layout_chord_noteheads(&[note(0), note(2)], StemDirection::Up);
        assert_eq!(result.len(), 2);
        assert!(!result[0].offset, "bottom note should not be offset");
        assert!(!result[1].offset, "top note should not be offset");
    }

    #[test]
    fn two_notes_third_apart_no_offset_down() {
        let result = layout_chord_noteheads(&[note(4), note(6)], StemDirection::Down);
        assert_eq!(result.len(), 2);
        assert!(!result[0].offset);
        assert!(!result[1].offset);
    }

    // --- two notes, second ---

    #[test]
    fn two_notes_second_stem_up_upper_offset() {
        // Adjacent positions: 2 and 3 (a second)
        let result = layout_chord_noteheads(&[note(2), note(3)], StemDirection::Up);
        assert_eq!(result.len(), 2);
        assert!(!result[0].offset, "lower note stays on stem side (stem up)");
        assert!(result[1].offset, "upper note offset right of stem (stem up)");
    }

    #[test]
    fn two_notes_second_stem_down_lower_offset() {
        // Adjacent positions: 4 and 5
        let result = layout_chord_noteheads(&[note(4), note(5)], StemDirection::Down);
        assert_eq!(result.len(), 2);
        assert!(
            result[0].offset,
            "lower note offset left of stem (stem down)"
        );
        assert!(
            !result[1].offset,
            "upper note stays on stem side (stem down)"
        );
    }

    // --- three notes, cluster of seconds ---

    #[test]
    fn three_consecutive_seconds_stem_up_alternates() {
        // Positions 2, 3, 4 — all adjacent
        let result = layout_chord_noteheads(&[note(2), note(3), note(4)], StemDirection::Up);
        assert_eq!(result.len(), 3);
        // Bottom-to-top scan: 2=normal, 3=offset (second with 2), 4=normal (alternates)
        assert!(!result[0].offset, "pos 2: normal");
        assert!(result[1].offset, "pos 3: offset");
        assert!(!result[2].offset, "pos 4: back to normal");
    }

    #[test]
    fn three_consecutive_seconds_stem_down_alternates() {
        // Positions 4, 5, 6 — all adjacent
        let result = layout_chord_noteheads(&[note(4), note(5), note(6)], StemDirection::Down);
        assert_eq!(result.len(), 3);
        // Top-to-bottom scan: 6=normal, 5=offset (second with 6), 4=normal (alternates)
        assert!(!result[0].offset, "pos 4: normal");
        assert!(result[1].offset, "pos 5: offset");
        assert!(!result[2].offset, "pos 6: normal");
    }

    // --- mixed intervals ---

    #[test]
    fn second_plus_third_stem_up() {
        // Positions 2, 3, 5 — second between 2-3, then skip to 5
        let result = layout_chord_noteheads(&[note(2), note(3), note(5)], StemDirection::Up);
        assert_eq!(result.len(), 3);
        assert!(!result[0].offset, "pos 2: normal");
        assert!(result[1].offset, "pos 3: offset (second with 2)");
        assert!(!result[2].offset, "pos 5: normal (not a second with 3)");
    }

    #[test]
    fn third_plus_second_stem_up() {
        // Positions 2, 4, 5 — third between 2-4, second between 4-5
        let result = layout_chord_noteheads(&[note(2), note(4), note(5)], StemDirection::Up);
        assert_eq!(result.len(), 3);
        assert!(!result[0].offset, "pos 2: normal");
        assert!(!result[1].offset, "pos 4: normal (not second with 2)");
        assert!(result[2].offset, "pos 5: offset (second with 4)");
    }

    // --- input order doesn't matter ---

    #[test]
    fn unsorted_input_produces_sorted_output() {
        let result = layout_chord_noteheads(&[note(5), note(2), note(3)], StemDirection::Up);
        assert_eq!(result[0].staff_position, 2);
        assert_eq!(result[1].staff_position, 3);
        assert_eq!(result[2].staff_position, 5);
    }

    // --- four notes with two separate seconds ---

    #[test]
    fn two_separate_seconds_stem_up() {
        // Positions 0,1, 4,5 — two pairs of seconds
        let notes = vec![note(0), note(1), note(4), note(5)];
        let result = layout_chord_noteheads(&notes, StemDirection::Up);
        assert_eq!(result.len(), 4);
        assert!(!result[0].offset, "pos 0: normal");
        assert!(result[1].offset, "pos 1: offset (second with 0)");
        assert!(!result[2].offset, "pos 4: normal (gap from 1)");
        assert!(result[3].offset, "pos 5: offset (second with 4)");
    }

    // --- unison (same position) treated as second ---

    #[test]
    fn unison_treated_as_non_second() {
        // Two notes at same position: difference is 0, not 1, so no offset
        let result = layout_chord_noteheads(&[note(4), note(4)], StemDirection::Up);
        assert_eq!(result.len(), 2);
        assert!(!result[0].offset);
        assert!(!result[1].offset);
    }

    // --- accidentals preserved ---

    #[test]
    fn accidentals_carried_through() {
        let notes = vec![
            note_with_acc(2, smufl::Glyph::AccidentalSharp),
            note(4),
        ];
        let result = layout_chord_noteheads(&notes, StemDirection::Up);
        assert_eq!(result[0].accidental, Some(smufl::Glyph::AccidentalSharp));
        assert_eq!(result[1].accidental, None);
    }

    // --- notehead_x_offset ---

    #[test]
    fn no_offset_returns_zero() {
        assert_eq!(notehead_x_offset(false, StemDirection::Up), 0.0);
        assert_eq!(notehead_x_offset(false, StemDirection::Down), 0.0);
    }

    #[test]
    fn offset_stem_up_returns_positive() {
        assert_eq!(notehead_x_offset(true, StemDirection::Up), 1.0);
    }

    #[test]
    fn offset_stem_down_returns_negative() {
        assert_eq!(notehead_x_offset(true, StemDirection::Down), -1.0);
    }

    // --- chord_has_offsets ---

    #[test]
    fn no_offsets_detected_for_simple_chord() {
        let layouts = layout_chord_noteheads(&[note(0), note(4)], StemDirection::Up);
        assert!(!chord_has_offsets(&layouts));
    }

    #[test]
    fn offsets_detected_for_second() {
        let layouts = layout_chord_noteheads(&[note(2), note(3)], StemDirection::Up);
        assert!(chord_has_offsets(&layouts));
    }

    // --- chord_extent ---

    #[test]
    fn chord_extent_empty_returns_none() {
        assert_eq!(chord_extent(&[]), None);
    }

    #[test]
    fn chord_extent_single_note() {
        assert_eq!(chord_extent(&[note(5)]), Some((5, 5)));
    }

    #[test]
    fn chord_extent_multiple_notes() {
        let notes = vec![note(7), note(2), note(5)];
        assert_eq!(chord_extent(&notes), Some((2, 7)));
    }

    // --- large cluster ---

    #[test]
    fn five_note_cluster_alternates_correctly_stem_up() {
        // Positions 0,1,2,3,4
        let notes: Vec<ChordNote> = (0..5).map(note).collect();
        let result = layout_chord_noteheads(&notes, StemDirection::Up);
        // Pattern: normal, offset, normal, offset, normal
        let expected_offsets = [false, true, false, true, false];
        for (i, (layout, &expected)) in result.iter().zip(expected_offsets.iter()).enumerate() {
            assert_eq!(
                layout.offset, expected,
                "pos {}: expected offset={expected}, got {}",
                layout.staff_position, layout.offset
            );
            assert_eq!(layout.staff_position, i as i8);
        }
    }

    #[test]
    fn five_note_cluster_alternates_correctly_stem_down() {
        // Positions 4,5,6,7,8
        let notes: Vec<ChordNote> = (4..9).map(note).collect();
        let result = layout_chord_noteheads(&notes, StemDirection::Down);
        // Top-to-bottom scan: 8=normal, 7=offset, 6=normal, 5=offset, 4=normal
        let expected_offsets = [false, true, false, true, false];
        for (layout, &expected) in result.iter().zip(expected_offsets.iter()) {
            assert_eq!(
                layout.offset, expected,
                "pos {}: expected offset={expected}, got {}",
                layout.staff_position, layout.offset
            );
        }
    }
}
