use crate::layout::measure::{MeasureElement, MeasureLayout};
use crate::layout::staff::StaffPosition;
use crate::layout::stem::{auto_stem_direction, auto_stem_direction_chord, StemDirection};

/// X-offset to apply to an additional voice's notehead to avoid collision
/// with the primary voice.
///
/// Stored as a multiplier of notehead width (e.g. 1.0 = shift right by one
/// notehead width, -1.0 = shift left).
///
/// Beam and tuplet members are ordinary note/chord elements, so every
/// offset targets one element. The renderer shifts a standalone event as a
/// whole; a beamed member's noteheads (with their accidentals, ledger lines,
/// and dots) shift while its stem and the beam stay at the unshifted beat
/// position — the engraving convention for second/unison collisions inside a
/// beamed voice.
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
    layout
        .elements
        .iter()
        .filter_map(|elem| {
            let positions = element_staff_positions(&elem.element);
            (!positions.is_empty()).then_some((elem.x, positions))
        })
        .collect()
}

/// Extract staff positions from a measure element (notes and chords only).
fn element_staff_positions(element: &MeasureElement) -> Vec<StaffPosition> {
    match element {
        MeasureElement::Note(n) => vec![n.staff_position],
        MeasureElement::Chord(c) => c.staff_positions.clone(),
        _ => Vec::new(),
    }
}

/// Resolve the stem direction for a note-bearing element using the same
/// rule the renderer applies when the explicit `stem_direction` is `None`.
///
/// The collision-shift direction must match where the renderer will
/// actually place the stem: shifting an additional voice's notehead "to
/// the opposite side of its stem" only works if the detector and renderer
/// agree on which side that is. Beam members carry their beam's resolved
/// direction in `stem_direction` (set during score conversion).
///
/// Returns `None` only for elements without note-bearing staff positions
/// (rests, etc.) — those are filtered out by `element_staff_positions`
/// upstream, so callers in this module never see a `None` for a colliding
/// element.
fn resolved_element_stem_direction(element: &MeasureElement) -> Option<StemDirection> {
    match element {
        MeasureElement::Note(n) => Some(
            n.stem_direction
                .unwrap_or_else(|| auto_stem_direction(n.staff_position)),
        ),
        MeasureElement::Chord(c) => Some(
            c.stem_direction
                .unwrap_or_else(|| auto_stem_direction_chord(&c.staff_positions)),
        ),
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
pub fn compute_voice_collision_offsets(
    primary: &MeasureLayout,
    additional: &MeasureLayout,
) -> Vec<VoiceCollisionOffset> {
    let primary_positions = collect_voice_positions(primary);
    let mut offsets = Vec::new();

    // Tolerance for matching x-positions: elements at the "same beat" share
    // an exact x but floating-point rounding can drift slightly. 1.0 font
    // unit ≈ 1/250 of a staff space, far smaller than any meaningful musical
    // separation.
    const X_MATCH_TOLERANCE: f64 = 1.0;

    for (elem_idx, elem) in additional.elements.iter().enumerate() {
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
            });
        }
    }

    offsets
}

/// Look up the primary voice's notes at approximately `x` and return the
/// collision magnitude (in notehead widths) if any additional-voice
/// position collides.
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
fn detect_collision(voice_a: &[StaffPosition], voice_b: &[StaffPosition]) -> Option<f64> {
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
        ChordEvent, MeasureLayout, NoteAnnotations, NoteEvent, PositionedElement, RestEvent,
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
        MeasureElement::Rest(RestEvent {
            duration_log2: 2,
            dots: 0,
            annotations: Default::default(),
        })
    }

    fn layout_with(elements: Vec<(f64, MeasureElement)>) -> MeasureLayout {
        let positioned: Vec<PositionedElement> = elements
            .into_iter()
            .map(|(x, element)| PositionedElement {
                x,
                element,
                width: 250.0,
                rod: 250.0,
                spring: 0.0,
            })
            .collect();
        MeasureLayout {
            total_rod: positioned.iter().map(|p| p.rod).sum(),
            total_spring: 0.0,
            elements: positioned,
            total_width: 1000.0,
        }
    }

    #[test]
    fn no_collision_when_positions_far_apart() {
        let primary = layout_with(vec![(100.0, note_element(8, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(100.0, note_element(2, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(
            offsets.is_empty(),
            "notes 6 positions apart should not collide"
        );
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
        assert!(
            offsets.is_empty(),
            "notes a third apart (distance 2) should not collide"
        );
    }

    #[test]
    fn no_collision_at_different_x_positions() {
        let primary = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![(300.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert!(
            offsets.is_empty(),
            "notes at different x should not collide"
        );
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
        let primary = layout_with(vec![(
            100.0,
            chord_element(vec![4, 8], Some(StemDirection::Up)),
        )]);
        let additional = layout_with(vec![(100.0, note_element(4, Some(StemDirection::Down)))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(
            offsets.len(),
            1,
            "note at pos 4 collides with chord containing pos 4"
        );
    }

    #[test]
    fn chord_collision_with_chord_at_second() {
        let primary = layout_with(vec![(
            100.0,
            chord_element(vec![4, 8], Some(StemDirection::Up)),
        )]);
        let additional = layout_with(vec![(
            100.0,
            chord_element(vec![3, 7], Some(StemDirection::Down)),
        )]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(
            offsets.len(),
            1,
            "chord with pos 3 is a second from chord pos 4"
        );
    }

    #[test]
    fn multiple_elements_some_collide() {
        let primary = layout_with(vec![
            (100.0, note_element(4, Some(StemDirection::Up))),
            (500.0, note_element(8, Some(StemDirection::Up))),
        ]);
        let additional = layout_with(vec![
            (100.0, note_element(4, Some(StemDirection::Down))), // collides
            (500.0, note_element(2, Some(StemDirection::Down))), // no collision
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
        assert_eq!(
            offsets[0].x_offset_noteheads, -1.0,
            "up-stem additional voice shifts left"
        );
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
        let primary = layout_with(vec![(
            100.0,
            chord_element(vec![-1, 1], Some(StemDirection::Down)),
        )]);
        let additional = layout_with(vec![(100.0, chord_element(vec![-1, 1], None))]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0].x_offset_noteheads, -1.0);
    }

    // --- Direct resolution helper tests ---

    #[test]
    fn resolved_direction_note_explicit_wins_over_auto() {
        let n = note_element(0, Some(StemDirection::Down)); // low pos, would auto-up
        assert_eq!(
            resolved_element_stem_direction(&n),
            Some(StemDirection::Down)
        );
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
        assert_eq!(
            resolved_element_stem_direction(&n),
            Some(StemDirection::Down)
        );
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
        assert_eq!(
            resolved_element_stem_direction(&c),
            Some(StemDirection::Down)
        );
    }

    #[test]
    fn resolved_direction_rest_returns_none() {
        assert_eq!(resolved_element_stem_direction(&rest_element()), None);
    }

    #[test]
    fn beam_member_offset_targets_the_member_element_past_span_marks() {
        // Beam members are ordinary notes interleaved with zero-width span
        // marks; the offset must index the colliding member itself so the
        // renderer shifts that notehead and nothing else.
        use crate::layout::group::{BeamSpec, GroupMark};
        let eighth = |pos: i8| {
            MeasureElement::Note(NoteEvent {
                staff_position: pos,
                duration_log2: 3,
                dots: 0,
                accidental: None,
                stem_direction: Some(StemDirection::Down),
                annotations: NoteAnnotations::default(),
            })
        };
        let primary = layout_with(vec![(300.0, note_element(4, Some(StemDirection::Up)))]);
        let additional = layout_with(vec![
            (
                100.0,
                MeasureElement::GroupMark(GroupMark::BeamStart {
                    spec: BeamSpec::default(),
                    continued: false,
                }),
            ),
            (100.0, eighth(8)),
            (300.0, eighth(5)),
            (
                500.0,
                MeasureElement::GroupMark(GroupMark::BeamEnd { continues: false }),
            ),
        ]);
        let offsets = compute_voice_collision_offsets(&primary, &additional);
        assert_eq!(
            offsets,
            vec![VoiceCollisionOffset {
                element_index: 2,
                x_offset_noteheads: 1.0,
            }]
        );
    }
}
