use crate::layout::beam::beam_group_note_x_offsets;
use crate::layout::measure::{MeasureElement, MeasureLayout};
use crate::layout::staff::StaffPosition;
use crate::layout::stem::{auto_stem_direction, auto_stem_direction_chord, StemDirection};

/// X-offset to apply to an additional voice's notehead to avoid collision
/// with the primary voice.
///
/// Stored as a multiplier of notehead width (e.g. 1.0 = shift right by one
/// notehead width, -1.0 = shift left).
///
/// `inner_note_index` distinguishes per-element offsets (notes, chords) from
/// per-note offsets within a beam or tuplet group:
/// - `None` → shift the entire element (notes/chords).
/// - `Some(i)` → shift only the `i`-th note inside the BeamGroup or
///   TupletGroup at `element_index`. The stems and beam line remain at
///   their unshifted positions; only the notehead (and its accidental,
///   ledger lines, dots) move. This matches the engraving convention for
///   second/unison collisions inside a beamed voice: the stem-down voice's
///   colliding notehead shifts to the opposite side of its stem.
#[derive(Clone, Debug, PartialEq)]
pub struct VoiceCollisionOffset {
    /// Index into the voice layout's `elements` vec.
    pub element_index: usize,
    /// X-offset in notehead-width units.
    pub x_offset_noteheads: f64,
    /// For BeamGroup/TupletGroup elements, the index of the specific note
    /// within the group's `notes` vec that should receive the offset. `None`
    /// for standalone Note/Chord elements (entire element shifts).
    pub inner_note_index: Option<usize>,
}

/// Collect the staff positions of notes/chords at each positioned element
/// in a measure layout. Returns `(x, Vec<staff_position>)` pairs.
///
/// For BeamGroup and TupletGroup, this expands each member note to its
/// absolute x (group_x + intra-group offset) so per-beat alignment with
/// the additional voice's per-note collision check matches the renderer's
/// actual note placement.
fn collect_voice_positions(layout: &MeasureLayout) -> Vec<(f64, Vec<StaffPosition>)> {
    let mut result = Vec::new();
    for elem in &layout.elements {
        match &elem.element {
            MeasureElement::BeamGroup(bg) => {
                let durations: Vec<u8> = bg.notes.iter().map(|n| n.duration_log2).collect();
                let offsets = beam_group_note_x_offsets(&durations, elem.width);
                for (i, note) in bg.notes.iter().enumerate() {
                    result.push((elem.x + offsets[i], vec![note.staff_position]));
                }
            }
            MeasureElement::TupletGroup(tg) => {
                let durations: Vec<u8> =
                    tg.beam_group.notes.iter().map(|n| n.duration_log2).collect();
                let offsets = beam_group_note_x_offsets(&durations, elem.width);
                for (i, note) in tg.beam_group.notes.iter().enumerate() {
                    result.push((elem.x + offsets[i], vec![note.staff_position]));
                }
            }
            _ => {
                let positions = element_staff_positions(&elem.element);
                if !positions.is_empty() {
                    result.push((elem.x, positions));
                }
            }
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

/// Resolve the stem direction for a note-bearing element using the same
/// rule the renderer applies when the explicit `stem_direction` is `None`.
///
/// The collision-shift direction must match where the renderer will
/// actually place the stem: shifting an additional voice's notehead "to
/// the opposite side of its stem" only works if the detector and renderer
/// agree on which side that is. Previously a beam group with
/// `stem_direction = None` was passed straight to `offset_shift_direction`,
/// which defaulted to "shift right" — correct for the common voice-2 case
/// where the group sits high and auto-resolves to stems-down, but wrong
/// when the same group sits low and the renderer auto-resolves to
/// stems-up. Resolving here closes that gap.
///
/// Returns `None` only for elements without note-bearing staff positions
/// (rests, etc.) — those are filtered out by `element_staff_positions`
/// upstream, so callers in this module never see a `None` for a colliding
/// element.
fn resolved_element_stem_direction(element: &MeasureElement) -> Option<StemDirection> {
    match element {
        MeasureElement::Note(n) => {
            Some(n.stem_direction.unwrap_or_else(|| auto_stem_direction(n.staff_position)))
        }
        MeasureElement::Chord(c) => Some(
            c.stem_direction
                .unwrap_or_else(|| auto_stem_direction_chord(&c.staff_positions)),
        ),
        MeasureElement::BeamGroup(bg) => {
            let positions: Vec<StaffPosition> =
                bg.notes.iter().map(|n| n.staff_position).collect();
            Some(
                bg.stem_direction
                    .unwrap_or_else(|| auto_stem_direction_chord(&positions)),
            )
        }
        MeasureElement::TupletGroup(tg) => {
            let positions: Vec<StaffPosition> = tg
                .beam_group
                .notes
                .iter()
                .map(|n| n.staff_position)
                .collect();
            Some(
                tg.beam_group
                    .stem_direction
                    .unwrap_or_else(|| auto_stem_direction_chord(&positions)),
            )
        }
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

    // Tolerance for matching x-positions: elements at the "same beat" share
    // an exact x but floating-point rounding (especially inside beam-group
    // proportional spacing) can drift slightly. 1.0 font unit ≈ 1/250 of a
    // staff space, far smaller than any meaningful musical separation.
    const X_MATCH_TOLERANCE: f64 = 1.0;

    for (elem_idx, elem) in additional.elements.iter().enumerate() {
        match &elem.element {
            MeasureElement::BeamGroup(bg) => {
                let durations: Vec<u8> = bg.notes.iter().map(|n| n.duration_log2).collect();
                let local_offsets = beam_group_note_x_offsets(&durations, elem.width);
                // Resolve direction once: auto-resolution applies the same
                // farthest-from-middle-line rule the renderer uses, so the
                // shift side here matches the side the stem will draw on.
                let group_dir = resolved_element_stem_direction(&elem.element);
                for (note_idx, note) in bg.notes.iter().enumerate() {
                    let note_x = elem.x + local_offsets[note_idx];
                    if let Some(offset) = collision_at_x(
                        note_x,
                        &[note.staff_position],
                        &primary_positions,
                        X_MATCH_TOLERANCE,
                    ) {
                        offsets.push(VoiceCollisionOffset {
                            element_index: elem_idx,
                            x_offset_noteheads: offset_shift_direction(group_dir) * offset,
                            inner_note_index: Some(note_idx),
                        });
                    }
                }
            }
            MeasureElement::TupletGroup(tg) => {
                let durations: Vec<u8> =
                    tg.beam_group.notes.iter().map(|n| n.duration_log2).collect();
                let local_offsets = beam_group_note_x_offsets(&durations, elem.width);
                let group_dir = resolved_element_stem_direction(&elem.element);
                for (note_idx, note) in tg.beam_group.notes.iter().enumerate() {
                    let note_x = elem.x + local_offsets[note_idx];
                    if let Some(offset) = collision_at_x(
                        note_x,
                        &[note.staff_position],
                        &primary_positions,
                        X_MATCH_TOLERANCE,
                    ) {
                        offsets.push(VoiceCollisionOffset {
                            element_index: elem_idx,
                            x_offset_noteheads: offset_shift_direction(group_dir) * offset,
                            inner_note_index: Some(note_idx),
                        });
                    }
                }
            }
            _ => {
                let add_positions = element_staff_positions(&elem.element);
                if add_positions.is_empty() {
                    continue;
                }
                if let Some(offset) = collision_at_x(
                    elem.x,
                    &add_positions,
                    &primary_positions,
                    X_MATCH_TOLERANCE,
                ) {
                    let add_dir = resolved_element_stem_direction(&elem.element);
                    offsets.push(VoiceCollisionOffset {
                        element_index: elem_idx,
                        x_offset_noteheads: offset_shift_direction(add_dir) * offset,
                        inner_note_index: None,
                    });
                }
            }
        }
    }

    offsets
}

/// Look up the primary voice's notes at approximately `x` and return the
/// collision magnitude (in notehead widths) if any additional-voice
/// position collides. Encapsulates the find-by-x + detect_collision pair
/// so beam-group per-note and standalone-element paths share one logic.
fn collision_at_x(
    x: f64,
    add_positions: &[StaffPosition],
    primary_positions: &[(f64, Vec<StaffPosition>)],
    tolerance: f64,
) -> Option<f64> {
    let (_, primary_pos) = primary_positions
        .iter()
        .find(|(px, _)| (px - x).abs() < tolerance)?;
    detect_collision(add_positions, primary_pos)
}

/// Direction multiplier for the collision shift. The additional voice's
/// stem direction dictates which side of the stem the displaced notehead
/// lands on: stems-down voices (the common odd-numbered additional voice
/// case) shift right; the rarer stems-up additional voice shifts left.
///
/// Callers in this module always pass `Some(_)` because
/// `resolved_element_stem_direction` resolves auto-direction up front (see
/// its docstring for why); the `None` arm is a defensive fallback that
/// preserves the historical "shift right" convention.
fn offset_shift_direction(dir: Option<StemDirection>) -> f64 {
    match dir {
        Some(StemDirection::Down) | None => 1.0,
        Some(StemDirection::Up) => -1.0,
    }
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

    fn beamed_eighth(pos: i8) -> NoteEvent {
        NoteEvent {
            staff_position: pos,
            duration_log2: 3,
            dots: 0,
            accidental: None,
            stem_direction: None,
            annotations: NoteAnnotations::default(),
        }
    }

    fn beam_group_element(
        positions: Vec<i8>,
        dir: Option<StemDirection>,
    ) -> MeasureElement {
        use crate::layout::measure::BeamGroupEvent;
        MeasureElement::BeamGroup(BeamGroupEvent {
            notes: positions.into_iter().map(beamed_eighth).collect(),
            stem_direction: dir,
        })
    }

    fn tuplet_group_element(
        positions: Vec<i8>,
        dir: Option<StemDirection>,
        tuplet_number: u32,
    ) -> MeasureElement {
        use crate::layout::measure::{BeamGroupEvent, TupletGroupEvent};
        MeasureElement::TupletGroup(TupletGroupEvent {
            beam_group: BeamGroupEvent {
                notes: positions.into_iter().map(beamed_eighth).collect(),
                stem_direction: dir,
            },
            tuplet_number,
        })
    }

    fn layout_with_widths(
        elements: Vec<(f64, MeasureElement, f64)>,
    ) -> MeasureLayout {
        MeasureLayout {
            elements: elements
                .into_iter()
                .map(|(x, element, width)| PositionedElement {
                    x,
                    element,
                    width,
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
        assert_eq!(
            offsets[0].inner_note_index, None,
            "standalone note collision is per-element, not per-inner-note"
        );
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

    // --- Per-note collision detection within beam groups ---

    #[test]
    fn beam_group_first_note_collides_only_first_note_offset() {
        // Primary: two quarter notes at x=100, 500. The first lines up with
        // beat 0; the second is at beat 2 of a 4-eighth additional beam group
        // (so beat 0 of the second pair).
        //
        // Additional: beam group of 4 eighths starting at x=100, width 400.
        // With equal durations, local offsets are 0, 100, 200, 300. So note
        // 0 is at x=100 (collides), note 2 is at x=300 (no primary), note 3
        // is at x=400 (no primary). Note 1 at x=200 is also unmatched.
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![4, 6, 4, 6], Some(StemDirection::Down)),
            400.0,
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(
            offsets.len(),
            1,
            "only the first note of the beam group lines up with primary"
        );
        assert_eq!(offsets[0].element_index, 0);
        assert_eq!(offsets[0].inner_note_index, Some(0));
        assert_eq!(offsets[0].x_offset_noteheads, 1.0);
    }

    #[test]
    fn beam_group_middle_note_collides_per_note_offset() {
        // Primary at x=300 only. Additional: 4-eighth beam at x=100, width=400.
        // Equal-duration offsets: 0, 100, 200, 300 → absolute xs: 100, 200,
        // 300, 400. Only note index 2 (x=300) lines up with primary.
        let primary = layout_with(vec![(300.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![6, 6, 4, 6], Some(StemDirection::Down)),
            400.0,
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].inner_note_index, Some(2));
        assert_eq!(offsets[0].x_offset_noteheads, 1.0);
    }

    #[test]
    fn beam_group_no_collisions_emits_no_offsets() {
        // Same beam group but primary's staff position is a sixth away from
        // every beam group note (positions 4 vs 10). No collisions anywhere.
        let primary = layout_with(vec![(100.0, note_element(10, Some(StemDirection::Down)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![4, 4, 4, 4], Some(StemDirection::Down)),
            400.0,
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(
            offsets.is_empty(),
            "positions 6 apart should never collide, got {:?}",
            offsets
        );
    }

    #[test]
    fn beam_group_multiple_notes_collide_multiple_offsets() {
        // Primary: three quarter notes at the same x's as beam notes 0, 2, 3.
        // At those x's the additional voice's position equals the primary's,
        // producing unison collisions. Note 1 at x=200 doesn't align.
        let primary = layout_with(vec![
            (100.0, note_element(4, Some(StemDirection::Up))),
            (300.0, note_element(4, Some(StemDirection::Up))),
            (400.0, note_element(6, Some(StemDirection::Up))), // unison with note 3 (pos 6)
        ]);
        let additional = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![4, 6, 4, 6], Some(StemDirection::Down)),
            400.0,
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 3);
        let mut inner: Vec<_> = offsets.iter().map(|o| o.inner_note_index).collect();
        inner.sort();
        assert_eq!(inner, vec![Some(0), Some(2), Some(3)]);
        for o in &offsets {
            assert_eq!(o.element_index, 0);
            assert_eq!(o.x_offset_noteheads, 1.0);
        }
    }

    #[test]
    fn beam_group_up_stem_additional_voice_shifts_left() {
        // Additional voice with explicit up-stems: collision shifts noteheads
        // left rather than right. Direction lives on the beam group, not the
        // individual notes.
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![4, 6], Some(StemDirection::Up)),
            400.0,
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].inner_note_index, Some(0));
        assert_eq!(offsets[0].x_offset_noteheads, -1.0);
    }

    #[test]
    fn beam_group_none_stem_direction_resolves_via_auto_rule_high_shifts_right() {
        // Beam group with stem_direction = None at high staff positions
        // (max=6, min=4) auto-resolves to stems-down (farthest-above wins
        // when tied or above), so the additional voice's notehead shifts
        // RIGHT (+1.0). This exercises the resolution path: the detector
        // must apply the same farthest-from-middle rule the renderer
        // applies at draw time, otherwise the shift would land on the
        // wrong side of the (still-to-be-drawn) stem.
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![4, 6], None),
            400.0,
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].x_offset_noteheads, 1.0);
    }

    #[test]
    fn beam_group_none_stem_direction_resolves_via_auto_rule_low_shifts_left() {
        // Companion to the high-staff case: when the beam group sits below
        // the middle line (positions -2, 0), auto-resolution picks
        // stems-UP, so the additional voice's collided notehead shifts
        // LEFT (-1.0). This is the case that motivated the fix: before
        // resolution was added, this beam group would also shift right
        // (the historical None-defaults-to-right fallback), placing the
        // displaced notehead on the wrong side of the auto-up stem.
        //
        // Geometry: beam group at x=100 with width 400, two equal
        // eighths → local offsets 0, 200 → absolute xs 100, 300. Note 1
        // (pos 0) lands at x=300, where the primary's pos 0 sits.
        let primary = layout_with(vec![(300.0, note_element(0, Some(StemDirection::Down)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![-2, 0], None),
            400.0,
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].inner_note_index, Some(1));
        assert_eq!(
            offsets[0].x_offset_noteheads, -1.0,
            "auto-resolved Up stem must shift colliding notehead LEFT"
        );
    }

    #[test]
    fn tuplet_group_none_stem_direction_auto_resolves_low_shifts_left() {
        // Same auto-resolution must apply to a tuplet wrapping a beam
        // group with stem_direction = None. Positions -4, -2, 0 sit fully
        // below the middle line — auto-direction = Up → shift LEFT.
        let primary = layout_with(vec![(200.0, note_element(-2, Some(StemDirection::Down)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            tuplet_group_element(vec![-4, -2, 0], None, 3),
            300.0,
        )]);
        // Triplet of equal eighths: local offsets 0, 100, 200; note 1
        // lands at x=200, staff_position -2 → unison with primary.
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].inner_note_index, Some(1));
        assert_eq!(offsets[0].x_offset_noteheads, -1.0);
    }

    #[test]
    fn note_none_stem_direction_auto_resolves_low_shifts_left() {
        // Standalone note with stem_direction = None at position 0 (below
        // the middle line) auto-resolves to Up via `auto_stem_direction`.
        // Before the fix this fell through to the None-defaults-to-right
        // path and incorrectly shifted RIGHT.
        let primary = layout_with(vec![(100.0, note_element(0, Some(StemDirection::Down)))]);
        let additional = layout_with(vec![(100.0, note_element(0, None))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].inner_note_index, None);
        assert_eq!(
            offsets[0].x_offset_noteheads, -1.0,
            "auto-up note must shift colliding notehead LEFT, not right"
        );
    }

    #[test]
    fn note_none_stem_direction_auto_resolves_high_shifts_right() {
        // Companion: note at staff_position 5 (above middle, ≥4) →
        // auto-direction Down → shift RIGHT. Confirms the auto-rule
        // boundary at position 4 (middle line resolves to Down).
        let primary = layout_with(vec![(100.0, note_element(5, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, note_element(5, None))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].x_offset_noteheads, 1.0);
    }

    #[test]
    fn chord_none_stem_direction_auto_resolves_low_shifts_left() {
        // Chord at positions [-1, 1] sits entirely below middle: max=1,
        // min=-1, dist_above = -3, dist_below = 5 → auto Up → shift LEFT.
        let primary =
            layout_with(vec![(100.0, chord_element(vec![-1, 1], Some(StemDirection::Down)))]);
        let additional = layout_with(vec![(100.0, chord_element(vec![-1, 1], None))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].x_offset_noteheads, -1.0);
    }

    // --- Direct resolution helper tests ---

    #[test]
    fn resolved_direction_note_explicit_wins_over_auto() {
        let n = note_element(0, Some(StemDirection::Down)); // low pos, would auto-up
        assert_eq!(resolved_element_stem_direction(&n), Some(StemDirection::Down));
    }

    #[test]
    fn resolved_direction_note_auto_low_returns_up() {
        let n = note_element(0, None);
        assert_eq!(resolved_element_stem_direction(&n), Some(StemDirection::Up));
    }

    #[test]
    fn resolved_direction_note_auto_high_returns_down() {
        // Position 4 = middle line; auto_stem_direction picks Down at >=4.
        let n = note_element(4, None);
        assert_eq!(resolved_element_stem_direction(&n), Some(StemDirection::Down));
    }

    #[test]
    fn resolved_direction_chord_auto_uses_farthest_from_middle() {
        // Chord [-2, 6]: dist_above = 2, dist_below = 6 → Up.
        let c = chord_element(vec![-2, 6], None);
        assert_eq!(resolved_element_stem_direction(&c), Some(StemDirection::Up));
    }

    #[test]
    fn resolved_direction_chord_auto_ties_go_down() {
        // Equidistant: [2, 6] → dist_above = 2, dist_below = 2 → Down by
        // the equidistant tiebreak.
        let c = chord_element(vec![2, 6], None);
        assert_eq!(resolved_element_stem_direction(&c), Some(StemDirection::Down));
    }

    #[test]
    fn resolved_direction_beam_group_auto_low_returns_up() {
        let bg = beam_group_element(vec![-2, 0, 2], None);
        assert_eq!(resolved_element_stem_direction(&bg), Some(StemDirection::Up));
    }

    #[test]
    fn resolved_direction_beam_group_auto_high_returns_down() {
        let bg = beam_group_element(vec![4, 6, 8], None);
        assert_eq!(resolved_element_stem_direction(&bg), Some(StemDirection::Down));
    }

    #[test]
    fn resolved_direction_tuplet_group_inherits_beam_group_rule() {
        // Tuplet wrapping a low-sitting beam group must resolve to Up,
        // same as the bare beam group case.
        let tg = tuplet_group_element(vec![-2, 0, 2], None, 3);
        assert_eq!(resolved_element_stem_direction(&tg), Some(StemDirection::Up));
    }

    #[test]
    fn resolved_direction_rest_returns_none() {
        assert_eq!(resolved_element_stem_direction(&rest_element()), None);
    }

    #[test]
    fn tuplet_group_per_note_collision_matches_beam_group() {
        // Tuplets wrap a beam group; collision detection should treat the
        // inner notes the same way (per-note offsets keyed off the inner
        // index inside the beam_group).
        let primary = layout_with(vec![(200.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with_widths(vec![(
            100.0,
            tuplet_group_element(vec![6, 4, 6], Some(StemDirection::Down), 3),
            300.0,
        )]);
        // Three equal eighth-notes in a triplet: local offsets 0, 100, 200.
        // Note 1 lands at x=200 → collides with primary's pos 4.
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].inner_note_index, Some(1));
        assert_eq!(offsets[0].x_offset_noteheads, 1.0);
    }

    #[test]
    fn beam_group_in_primary_voice_detects_per_note_collision_from_additional() {
        // Symmetric case: when the *primary* voice contains the beam group
        // and the *additional* voice has a single note matching one inner
        // note's x, the collision detector must see that beam-group note.
        // This exercises `collect_voice_positions` expanding the primary's
        // beam-group into per-note x positions.
        let primary = layout_with_widths(vec![(
            100.0,
            beam_group_element(vec![4, 6, 4, 6], Some(StemDirection::Up)),
            400.0,
        )]);
        let additional = layout_with(vec![(300.0, note_element(4, Some(StemDirection::Down)))]);
        // Beam local offsets 0,100,200,300 → primary note index 2 sits at
        // x=300, position 4. Additional sits at x=300, position 4. Unison.
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        // Additional voice element is the simple Note → inner_note_index None.
        assert_eq!(offsets[0].inner_note_index, None);
        assert_eq!(offsets[0].x_offset_noteheads, 1.0);
    }
}
