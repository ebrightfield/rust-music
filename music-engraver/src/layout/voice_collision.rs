use crate::layout::measure::{MeasureElement, MeasureLayout};
use crate::layout::staff::StaffPosition;
use crate::layout::stem::StemDirection;

/// X-offset to apply to an additional voice's notehead to avoid collision
/// with the primary voice.
///
/// Stored as a multiplier of notehead width (e.g. 1.0 = shift right by one
/// notehead width, -1.0 = shift left).
#[derive(Clone, Debug, PartialEq)]
pub struct VoiceCollisionOffset {
    /// Index into the voice layout's `elements` vec.
    pub element_index: usize,
    /// X-offset in notehead-width units.
    pub x_offset_noteheads: f64,
}

/// Collect the staff positions of notes/chords at each positioned element
/// in a measure layout. Returns `(x, Vec<staff_position>)` pairs.
fn collect_voice_positions(layout: &MeasureLayout) -> Vec<(f64, Vec<StaffPosition>)> {
    let mut result = Vec::new();
    for elem in &layout.elements {
        let positions = element_staff_positions(&elem.element);
        if !positions.is_empty() {
            result.push((elem.x, positions));
        }
    }
    result
}

/// Extract staff positions from a measure element (notes and chords only).
fn element_staff_positions(element: &MeasureElement) -> Vec<StaffPosition> {
    match element {
        MeasureElement::Note(n) => vec![n.staff_position],
        MeasureElement::Chord(c) => c.staff_positions.clone(),
        MeasureElement::BeamGroup(bg) => bg.notes.iter().map(|e| e.staff_position).collect(),
        MeasureElement::TupletGroup(tg) => tg.beam_group.notes.iter().map(|e| e.staff_position).collect(),
        _ => Vec::new(),
    }
}

/// Get the stem direction of a measure element, if it has one.
fn element_stem_direction(element: &MeasureElement) -> Option<StemDirection> {
    match element {
        MeasureElement::Note(n) => n.stem_direction,
        MeasureElement::Chord(c) => c.stem_direction,
        _ => None,
    }
}

/// Detect collisions between a primary voice layout and an additional voice
/// layout, returning x-offsets for the additional voice's elements.
///
/// Standard two-voice collision rules:
/// - **Unison** (same position, same notehead kind): noteheads overlap — no offset.
///   This is the traditional engraving convention: a shared notehead position
///   means only one notehead is visible.
/// - **Unison** (same position, different notehead kind): the down-stem voice's
///   notehead shifts right by one notehead width so both are visible.
/// - **Second** (adjacent positions, difference = 1): the up-stem voice keeps
///   its notehead on the normal (left) side; the down-stem voice's notehead
///   shifts right by one notehead width.
///
/// For beam groups and tuplet groups, collision detection uses the first note's
/// position as a representative — full per-note detection within groups is
/// deferred.
pub fn compute_voice_collision_offsets(
    primary: &MeasureLayout,
    additional: &MeasureLayout,
) -> Vec<VoiceCollisionOffset> {
    let primary_positions = collect_voice_positions(primary);
    let mut offsets = Vec::new();

    for (elem_idx, elem) in additional.elements.iter().enumerate() {
        let add_positions = element_staff_positions(&elem.element);
        if add_positions.is_empty() {
            continue;
        }

        // Find the primary voice element at approximately the same x-position.
        // Tolerance: within 1.0 font unit (elements at the "same beat" are
        // assigned exactly matching x by the layout system, but floating-point
        // rounding could differ slightly).
        let matching_primary = primary_positions
            .iter()
            .find(|(px, _)| (px - elem.x).abs() < 1.0);

        let Some((_, primary_pos)) = matching_primary else {
            continue;
        };

        // Check for collisions between any additional-voice note and any
        // primary-voice note at this x-position.
        let collision = detect_collision(&add_positions, primary_pos);

        if let Some(offset) = collision {
            // Determine direction: additional voice is typically voice 1 (stems down)
            // so its noteheads shift right when colliding.
            let add_dir = element_stem_direction(&elem.element);
            let x_mult = match add_dir {
                Some(StemDirection::Down) | None => 1.0,  // shift right
                Some(StemDirection::Up) => -1.0,           // shift left (rare)
            };

            offsets.push(VoiceCollisionOffset {
                element_index: elem_idx,
                x_offset_noteheads: x_mult * offset,
            });
        }
    }

    offsets
}

/// Check if any note position in `voice_a` collides with any note in `voice_b`.
///
/// Returns `Some(offset_magnitude)` if a collision is found:
/// - Unison (distance 0): `1.0` notehead widths (shift to make both visible,
///   unless both are the same kind — but we can't distinguish kinds here,
///   so always offset to be safe in multi-voice context).
/// - Second (distance 1): `1.0` notehead widths.
/// - No collision: `None`.
fn detect_collision(
    voice_a: &[StaffPosition],
    voice_b: &[StaffPosition],
) -> Option<f64> {
    for &pos_a in voice_a {
        for &pos_b in voice_b {
            let distance = (pos_a - pos_b).unsigned_abs();
            match distance {
                0 => {
                    // Unison: offset to make both noteheads visible.
                    // In strict traditional engraving, same-kind unisons
                    // share a notehead. However, in multi-voice rendering it's
                    // clearer to show both noteheads slightly offset — many
                    // modern editors do this.
                    return Some(1.0);
                }
                1 => {
                    // Second: noteheads would overlap vertically.
                    return Some(1.0);
                }
                _ => {}
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::measure::{
        MeasureLayout, NoteAnnotations, NoteEvent, ChordEvent, RestEvent, PositionedElement,
    };

    fn note_element(pos: i8, dir: Option<StemDirection>) -> MeasureElement {
        MeasureElement::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: dir,
            annotations: NoteAnnotations::default(),
        })
    }

    fn chord_element(positions: Vec<i8>, dir: Option<StemDirection>) -> MeasureElement {
        let n = positions.len();
        MeasureElement::Chord(ChordEvent {
            staff_positions: positions,
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None; n],
            stem_direction: dir,
            annotations: NoteAnnotations::default(),
        })
    }

    fn rest_element() -> MeasureElement {
        MeasureElement::Rest(RestEvent { duration_log2: 2, dots: 0 })
    }

    fn layout_with(elements: Vec<(f64, MeasureElement)>) -> MeasureLayout {
        MeasureLayout {
            elements: elements
                .into_iter()
                .map(|(x, element)| PositionedElement {
                    x,
                    element,
                    width: 250.0,
                })
                .collect(),
            total_width: 1000.0,
        }
    }

    #[test]
    fn no_collision_when_positions_far_apart() {
        let primary = layout_with(vec![(100.0, note_element(8, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, note_element(2, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(offsets.is_empty(), "notes 6 positions apart should not collide");
    }

    #[test]
    fn collision_at_unison() {
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].element_index, 0);
        assert_eq!(offsets[0].x_offset_noteheads, 1.0); // down-stem shifts right
    }

    #[test]
    fn collision_at_second() {
        let primary = layout_with(vec![(100.0, note_element(5, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].x_offset_noteheads, 1.0);
    }

    #[test]
    fn no_collision_at_third() {
        let primary = layout_with(vec![(100.0, note_element(6, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(offsets.is_empty(), "notes a third apart (distance 2) should not collide");
    }

    #[test]
    fn no_collision_at_different_x_positions() {
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(300.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(offsets.is_empty(), "notes at different x should not collide");
    }

    #[test]
    fn rest_elements_ignored() {
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, rest_element())]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(offsets.is_empty(), "rest vs note should not collide");
    }

    #[test]
    fn chord_collision_with_note() {
        let primary = layout_with(vec![(100.0, chord_element(vec![4, 8], Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1, "note at pos 4 collides with chord containing pos 4");
    }

    #[test]
    fn chord_collision_with_chord_at_second() {
        let primary = layout_with(vec![(100.0, chord_element(vec![4, 8], Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, chord_element(vec![3, 7], Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1, "chord with pos 3 is a second from chord pos 4");
    }

    #[test]
    fn multiple_elements_some_collide() {
        let primary = layout_with(vec![
            (100.0, note_element(4, Some(StemDirection::Up))),
            (500.0, note_element(8, Some(StemDirection::Up))),
        ]);
        let additional = layout_with(vec![
            (100.0, note_element(4, Some(StemDirection::Down))),  // collides
            (500.0, note_element(2, Some(StemDirection::Down))),  // no collision
        ]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].element_index, 0);
    }

    #[test]
    fn up_stem_additional_voice_shifts_left() {
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let additional = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].x_offset_noteheads, -1.0, "up-stem additional voice shifts left");
    }

    #[test]
    fn detect_collision_no_overlap() {
        assert_eq!(detect_collision(&[0, 2, 4], &[6, 8]), None);
    }

    #[test]
    fn detect_collision_unison_found() {
        assert_eq!(detect_collision(&[4], &[4]), Some(1.0));
    }

    #[test]
    fn detect_collision_second_found() {
        assert_eq!(detect_collision(&[3], &[4]), Some(1.0));
        assert_eq!(detect_collision(&[5], &[4]), Some(1.0));
    }

    #[test]
    fn detect_collision_third_or_more_no_collision() {
        assert_eq!(detect_collision(&[2], &[4]), None);
        assert_eq!(detect_collision(&[6], &[4]), None);
    }

    #[test]
    fn empty_primary_no_collisions() {
        let primary = layout_with(vec![]);
        let additional = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(offsets.is_empty());
    }

    #[test]
    fn empty_additional_no_collisions() {
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(offsets.is_empty());
    }
}
