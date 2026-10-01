use crate::layout::staff::StaffPosition;
use crate::layout::stem::StemDirection;

/// A note participating in a beam group.
#[derive(Clone, Debug)]
pub struct BeamedNote {
    /// X-coordinate of the notehead center (or stem attachment point).
    pub x: f64,
    /// Staff position of the notehead.
    pub staff_position: StaffPosition,
    /// Log2 of the duration: 3=eighth, 4=sixteenth, 5=32nd, etc.
    /// Must be >= 3 to participate in beaming.
    pub duration_log2: u8,
}

/// A computed beam group ready for rendering.
#[derive(Clone, Debug)]
pub struct BeamGroupLayout {
    /// Stem direction shared by all notes in the group.
    pub direction: StemDirection,
    /// For each note in the group, the y-coordinate of the stem tip
    /// (beam attachment point), **relative to the staff's top line**.
    /// Renderers must add the staff's `y_origin` to place these on the canvas.
    pub stem_tip_ys: Vec<f64>,
    /// Number of beam levels. 1 for eighths, 2 for sixteenths, etc.
    pub max_beam_level: u8,
    /// For each note, how many beams connect on its left side (0 for first note).
    pub beams_left: Vec<u8>,
    /// For each note, how many beams connect on its right side (0 for last note).
    pub beams_right: Vec<u8>,
}

/// Maximum allowed beam slope in half-spaces per staff space of horizontal distance.
/// Keeps beams from getting too steep. ~18 degrees max is standard practice.
const MAX_SLOPE_HS_PER_SS: f64 = 1.0;

/// Minimum stem length in staff spaces for beamed notes.
/// Slightly shorter than unbeamed because the beam itself provides visual weight.
const MIN_BEAMED_STEM_SS: f64 = 2.5;

/// Additional stem length per extra beam level beyond the primary beam,
/// in staff spaces. Each secondary beam needs room.
const EXTRA_STEM_PER_BEAM_LEVEL_SS: f64 = 0.5;

/// Determine stem direction for a beam group.
///
/// Uses the same farthest-from-middle-line rule as chords: the note in the
/// group that is farthest from the middle line determines direction. When
/// equidistant, stems go down.
pub fn beam_group_stem_direction(notes: &[BeamedNote]) -> StemDirection {
    let (first, rest) = match notes.split_first() {
        Some(pair) => pair,
        None => return StemDirection::Up,
    };

    let (min, max) = rest.iter().fold(
        (first.staff_position, first.staff_position),
        |(lo, hi), n| (lo.min(n.staff_position), hi.max(n.staff_position)),
    );

    let dist_above = max - 4;
    let dist_below = 4 - min;

    if dist_below > dist_above {
        StemDirection::Up
    } else {
        StemDirection::Down
    }
}

/// Compute how many beam lines each note needs on its left and right.
///
/// Rules:
/// - The primary beam spans the entire group (all notes get at least 1 on each interior side).
/// - Additional beams for shorter notes connect to the adjacent note with
///   the same or shorter duration; if neither neighbor qualifies, a fractional
///   beam (stub) extends toward the rhythmically stronger side.
///
/// For simplicity in v1: beams always connect to the neighbor with more beams.
/// Fractional beams point left (toward beat start) by default.
pub fn compute_beam_counts(notes: &[BeamedNote]) -> (Vec<u8>, Vec<u8>) {
    let n = notes.len();
    if n == 0 {
        return (vec![], vec![]);
    }
    if n == 1 {
        // A single beamed note gets a fractional beam on the left
        let level = notes[0].duration_log2.saturating_sub(2);
        return (vec![level], vec![0]);
    }

    // beam_level: how many beams this note "wants" (eighth=1, 16th=2, etc.)
    let levels: Vec<u8> = notes
        .iter()
        .map(|n| n.duration_log2.saturating_sub(2))
        .collect();

    let mut beams_left = vec![0u8; n];
    let mut beams_right = vec![0u8; n];

    for i in 0..n {
        let my_level = levels[i];

        if i == 0 {
            // First note: right side connects to next note
            beams_right[i] = my_level.min(levels[i + 1]).max(1);
            // For beams beyond what connects right, add fractional beams on right
            if my_level > beams_right[i] {
                beams_right[i] = my_level;
            }
        } else if i == n - 1 {
            // Last note: left side connects to previous note
            beams_left[i] = my_level.min(levels[i - 1]).max(1);
            if my_level > beams_left[i] {
                beams_left[i] = my_level;
            }
        } else {
            // Interior note
            let left_level = levels[i - 1];
            let right_level = levels[i + 1];

            // Primary beam always connects
            beams_left[i] = 1.max(my_level.min(left_level));
            beams_right[i] = 1.max(my_level.min(right_level));

            // For beams beyond what connects to neighbors:
            // add fractional beams toward the side with more beams
            if my_level > beams_left[i] && my_level > beams_right[i] {
                // Neither neighbor has enough beams; add fractional beams
                // pointing toward the rhythmically stronger neighbor (left by convention)
                beams_left[i] = my_level;
            } else if my_level > beams_right[i] {
                beams_left[i] = beams_left[i].max(my_level);
            } else if my_level > beams_left[i] {
                beams_right[i] = beams_right[i].max(my_level);
            }
        }
    }

    (beams_left, beams_right)
}

/// Lay out a beam group: compute stem tip y-coordinates and beam connectivity.
///
/// The beam line is determined by the first and last note positions, constrained
/// to a maximum slope. All stems extend to meet the beam line, with a minimum
/// length guarantee.
///
/// `staff_space`: distance between adjacent staff lines in font design units.
pub fn layout_beam_group(
    notes: &[BeamedNote],
    direction: StemDirection,
    staff_space: f64,
) -> BeamGroupLayout {
    assert!(!notes.is_empty(), "beam group must have at least one note");

    let half_space = staff_space / 2.0;

    // Max beam level determines extra stem length needed
    let max_beam_level = notes
        .iter()
        .map(|n| n.duration_log2.saturating_sub(2))
        .max()
        .unwrap_or(1)
        .max(1);

    // Base stem length: minimum + extra for secondary beams
    let base_stem_ss =
        MIN_BEAMED_STEM_SS + (max_beam_level as f64 - 1.0) * EXTRA_STEM_PER_BEAM_LEVEL_SS;
    let base_stem_fu = base_stem_ss * staff_space;

    // Compute "natural" stem tip for each note (extending by base_stem_fu)
    let natural_tips: Vec<f64> = notes
        .iter()
        .map(|n| {
            let notehead_y = staff_position_to_y(n.staff_position, half_space);
            match direction {
                StemDirection::Up => notehead_y - base_stem_fu,
                StemDirection::Down => notehead_y + base_stem_fu,
            }
        })
        .collect();

    // Determine beam line from first and last notes' natural tips.
    // Safety: natural_tips is non-empty because notes is non-empty (asserted above).
    let first_tip = natural_tips[0];
    let last_tip = natural_tips[natural_tips.len() - 1];

    // Constrain slope
    let first_note = &notes[0];
    let last_note = &notes[notes.len() - 1];
    let x_span = last_note.x - first_note.x;
    let (beam_y_first, beam_y_last) = if x_span.abs() < f64::EPSILON {
        // All notes at same x (degenerate): flat beam
        let avg = natural_tips.iter().sum::<f64>() / natural_tips.len() as f64;
        (avg, avg)
    } else {
        let raw_slope = (last_tip - first_tip) / x_span;
        let max_slope = MAX_SLOPE_HS_PER_SS * half_space / staff_space;
        let clamped_slope = raw_slope.clamp(-max_slope, max_slope);
        let mid_y = (first_tip + last_tip) / 2.0;
        let bf = mid_y - clamped_slope * x_span / 2.0;
        let bl = mid_y + clamped_slope * x_span / 2.0;
        (bf, bl)
    };

    // Compute beam y at each note's x via linear interpolation
    let mut stem_tip_ys: Vec<f64> = if x_span.abs() < f64::EPSILON {
        vec![beam_y_first; notes.len()]
    } else {
        notes
            .iter()
            .map(|n| {
                let t = (n.x - notes[0].x) / x_span;
                beam_y_first + t * (beam_y_last - beam_y_first)
            })
            .collect()
    };

    // Ensure minimum stem length for every note
    for (i, note) in notes.iter().enumerate() {
        let notehead_y = staff_position_to_y(note.staff_position, half_space);
        let min_stem_fu = MIN_BEAMED_STEM_SS * staff_space;
        match direction {
            StemDirection::Up => {
                let max_tip = notehead_y - min_stem_fu;
                if stem_tip_ys[i] > max_tip {
                    // Need to shift entire beam up
                    let shift = stem_tip_ys[i] - max_tip;
                    for tip in stem_tip_ys.iter_mut() {
                        *tip -= shift;
                    }
                }
            }
            StemDirection::Down => {
                let min_tip = notehead_y + min_stem_fu;
                if stem_tip_ys[i] < min_tip {
                    let shift = min_tip - stem_tip_ys[i];
                    for tip in stem_tip_ys.iter_mut() {
                        *tip += shift;
                    }
                }
            }
        }
    }

    let (beams_left, beams_right) = compute_beam_counts(notes);

    BeamGroupLayout {
        direction,
        stem_tip_ys,
        max_beam_level,
        beams_left,
        beams_right,
    }
}

/// Convert a staff position to y-coordinate relative to staff origin (top line at y=0).
/// This mirrors StaffLayout::y_of but without needing the full struct.
fn staff_position_to_y(position: StaffPosition, half_space: f64) -> f64 {
    (8 - position) as f64 * half_space
}

/// Spacing ratio per doubled duration used when distributing horizontal space
/// across a beam group. The shortest note in the group gets factor 1.0; each
/// doubling of duration multiplies the factor by this constant. Mirrors the
/// proportional-spacing exponent in `draw_beam_group_event` so that the
/// collision detector and the renderer agree on per-note x-positions.
const BEAM_GROUP_SPACING_RATIO: f64 = 1.6;

/// X-offsets (relative to the beam group's start x) of each note within the
/// group's `total_width`. Returns a `Vec<f64>` of length `durations.len()`
/// where `result[i]` is the local x-position of note `i` measured from the
/// group's anchor.
///
/// The shortest note (largest `duration_log2`) gets the unit horizontal
/// share; each doubling of duration is `BEAM_GROUP_SPACING_RATIO` times
/// wider. This matches Behind-Bars-style "longer notes consume more space"
/// proportional spacing within a beam group.
///
/// Empty input returns an empty vec. A single-note group returns `vec![0.0]`.
/// All-identical durations distribute the width evenly.
///
/// This helper exists so both the rendering path and the cross-voice
/// collision detector compute identical per-note absolute x-coordinates;
/// any divergence would silently misalign collision offsets.
pub fn beam_group_note_x_offsets(durations: &[u8], total_width: f64) -> Vec<f64> {
    let n = durations.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0.0];
    }

    let shortest_log2 = durations.iter().copied().max().unwrap_or(3);
    let factors: Vec<f64> = durations
        .iter()
        .map(|&d| {
            let steps = shortest_log2 as f64 - d as f64;
            BEAM_GROUP_SPACING_RATIO.powf(steps)
        })
        .collect();
    let total_factor: f64 = factors.iter().sum();
    if total_factor == 0.0 {
        return vec![0.0; n];
    }

    let mut offsets = Vec::with_capacity(n);
    let mut x = 0.0_f64;
    for factor in &factors {
        offsets.push(x);
        x += total_width * factor / total_factor;
    }
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: f64 = 250.0; // standard Bravura staff space
    const HS: f64 = 125.0;

    fn make_notes(positions: &[(f64, StaffPosition, u8)]) -> Vec<BeamedNote> {
        positions
            .iter()
            .map(|&(x, pos, dur)| BeamedNote {
                x,
                staff_position: pos,
                duration_log2: dur,
            })
            .collect()
    }

    fn eighth_notes(positions: &[(f64, StaffPosition)]) -> Vec<BeamedNote> {
        positions
            .iter()
            .map(|&(x, pos)| BeamedNote {
                x,
                staff_position: pos,
                duration_log2: 3,
            })
            .collect()
    }

    // --- beam_group_stem_direction ---

    #[test]
    fn empty_group_defaults_up() {
        assert_eq!(beam_group_stem_direction(&[]), StemDirection::Up);
    }

    #[test]
    fn all_below_middle_line_stems_up() {
        let notes = eighth_notes(&[(0.0, 0), (250.0, 2)]);
        assert_eq!(beam_group_stem_direction(&notes), StemDirection::Up);
    }

    #[test]
    fn all_above_middle_line_stems_down() {
        let notes = eighth_notes(&[(0.0, 6), (250.0, 8)]);
        assert_eq!(beam_group_stem_direction(&notes), StemDirection::Down);
    }

    #[test]
    fn mixed_farthest_below_stems_up() {
        // Positions 0 and 6: dist_below=4, dist_above=2 → up
        let notes = eighth_notes(&[(0.0, 0), (250.0, 6)]);
        assert_eq!(beam_group_stem_direction(&notes), StemDirection::Up);
    }

    #[test]
    fn mixed_farthest_above_stems_down() {
        // Positions 2 and 8: dist_below=2, dist_above=4 → down
        let notes = eighth_notes(&[(0.0, 2), (250.0, 8)]);
        assert_eq!(beam_group_stem_direction(&notes), StemDirection::Down);
    }

    #[test]
    fn equidistant_defaults_down() {
        // Positions 2 and 6: both 2 away → down
        let notes = eighth_notes(&[(0.0, 2), (250.0, 6)]);
        assert_eq!(beam_group_stem_direction(&notes), StemDirection::Down);
    }

    // --- compute_beam_counts ---

    #[test]
    fn single_eighth_note_beam_counts() {
        let notes = eighth_notes(&[(0.0, 4)]);
        let (left, right) = compute_beam_counts(&notes);
        assert_eq!(left, vec![1]); // fractional beam on left
        assert_eq!(right, vec![0]);
    }

    #[test]
    fn two_eighth_notes_beam_counts() {
        let notes = eighth_notes(&[(0.0, 2), (250.0, 4)]);
        let (left, right) = compute_beam_counts(&notes);
        assert_eq!(left, vec![0, 1]);
        assert_eq!(right, vec![1, 0]);
    }

    #[test]
    fn four_eighth_notes_beam_counts() {
        let notes = eighth_notes(&[(0.0, 0), (250.0, 2), (500.0, 4), (750.0, 6)]);
        let (left, right) = compute_beam_counts(&notes);
        // All eighths: single beam connecting all
        assert_eq!(left, vec![0, 1, 1, 1]);
        assert_eq!(right, vec![1, 1, 1, 0]);
    }

    #[test]
    fn sixteenth_notes_get_two_beams() {
        let notes = make_notes(&[(0.0, 2, 4), (250.0, 4, 4)]);
        let (left, right) = compute_beam_counts(&notes);
        assert_eq!(left, vec![0, 2]);
        assert_eq!(right, vec![2, 0]);
    }

    #[test]
    fn mixed_eighth_and_sixteenth() {
        // Eighth, sixteenth, sixteenth
        let notes = make_notes(&[(0.0, 2, 3), (250.0, 4, 4), (500.0, 6, 4)]);
        let (left, right) = compute_beam_counts(&notes);
        // First note (eighth): right connects with at least 1 beam
        assert_eq!(right[0], 1);
        // Second note (16th): left connects 1 (min with eighth), right connects 2 (both 16ths)
        assert_eq!(left[1], 1);
        assert_eq!(right[1], 2);
        // Third note (16th): left connects 2
        assert_eq!(left[2], 2);
        assert_eq!(right[2], 0);
    }

    #[test]
    fn empty_beam_counts() {
        let (left, right) = compute_beam_counts(&[]);
        assert!(left.is_empty());
        assert!(right.is_empty());
    }

    #[test]
    fn thirty_second_notes_get_three_beams() {
        let notes = make_notes(&[(0.0, 2, 5), (250.0, 4, 5)]);
        let (left, right) = compute_beam_counts(&notes);
        assert_eq!(right[0], 3);
        assert_eq!(left[1], 3);
    }

    // --- layout_beam_group ---

    #[test]
    fn two_eighths_ascending_stems_up() {
        let notes = eighth_notes(&[(0.0, 0), (500.0, 2)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        assert_eq!(layout.direction, StemDirection::Up);
        assert_eq!(layout.stem_tip_ys.len(), 2);
        assert_eq!(layout.max_beam_level, 1);

        // Both tips should be above their respective noteheads
        let y0 = staff_position_to_y(0, HS); // 1000
        let y1 = staff_position_to_y(2, HS); // 750
        assert!(
            layout.stem_tip_ys[0] < y0,
            "first tip should be above notehead"
        );
        assert!(
            layout.stem_tip_ys[1] < y1,
            "second tip should be above notehead"
        );
    }

    #[test]
    fn two_eighths_descending_stems_down() {
        let notes = eighth_notes(&[(0.0, 8), (500.0, 6)]);
        let layout = layout_beam_group(&notes, StemDirection::Down, SS);

        let y0 = staff_position_to_y(8, HS); // 0
        let y1 = staff_position_to_y(6, HS); // 250
        assert!(
            layout.stem_tip_ys[0] > y0,
            "first tip should be below notehead"
        );
        assert!(
            layout.stem_tip_ys[1] > y1,
            "second tip should be below notehead"
        );
    }

    #[test]
    fn flat_beam_for_same_position_notes() {
        let notes = eighth_notes(&[(0.0, 4), (500.0, 4)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        // Same position notes should produce a flat beam (tips at same y)
        assert!(
            (layout.stem_tip_ys[0] - layout.stem_tip_ys[1]).abs() < f64::EPSILON,
            "flat beam expected: tips {} and {}",
            layout.stem_tip_ys[0],
            layout.stem_tip_ys[1]
        );
    }

    #[test]
    fn minimum_stem_length_enforced() {
        // Two notes far apart in pitch — the one closer to the beam
        // still needs minimum stem length
        let notes = eighth_notes(&[(0.0, 0), (500.0, 8)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let min_stem_fu = MIN_BEAMED_STEM_SS * SS;
        for (i, note) in notes.iter().enumerate() {
            let notehead_y = staff_position_to_y(note.staff_position, HS);
            let stem_length = notehead_y - layout.stem_tip_ys[i];
            assert!(
                stem_length >= min_stem_fu - f64::EPSILON,
                "note {} stem length {} < minimum {}",
                i,
                stem_length,
                min_stem_fu
            );
        }
    }

    #[test]
    fn slope_is_constrained() {
        // Very steep interval: position -4 to position 12, 500 units apart
        let notes = eighth_notes(&[(0.0, -4), (500.0, 12)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let x_span = 500.0;
        let y_diff = (layout.stem_tip_ys[1] - layout.stem_tip_ys[0]).abs();
        let slope_hs_per_ss = y_diff / x_span * SS / HS;
        assert!(
            slope_hs_per_ss <= MAX_SLOPE_HS_PER_SS + 0.01,
            "slope {} exceeds max {}",
            slope_hs_per_ss,
            MAX_SLOPE_HS_PER_SS
        );
    }

    #[test]
    fn three_notes_middle_has_interpolated_tip() {
        let notes = eighth_notes(&[(0.0, 2), (250.0, 4), (500.0, 2)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        // First and last notes at same position → flat beam, so all tips equal
        assert!(
            (layout.stem_tip_ys[0] - layout.stem_tip_ys[2]).abs() < f64::EPSILON,
            "first and last tips should match for symmetric group"
        );
        // Middle tip should be the same as endpoints (flat beam)
        assert!(
            (layout.stem_tip_ys[1] - layout.stem_tip_ys[0]).abs() < f64::EPSILON,
            "middle tip should match endpoints for flat beam"
        );
    }

    #[test]
    fn sixteenth_group_has_max_beam_level_2() {
        let notes = make_notes(&[(0.0, 2, 4), (250.0, 4, 4), (500.0, 6, 4)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);
        assert_eq!(layout.max_beam_level, 2);
    }

    #[test]
    fn beam_group_with_mixed_durations() {
        // Eighth, sixteenth, sixteenth, eighth
        let notes = make_notes(&[(0.0, 2, 3), (200.0, 4, 4), (400.0, 4, 4), (600.0, 2, 3)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);
        assert_eq!(layout.max_beam_level, 2);
        assert_eq!(layout.stem_tip_ys.len(), 4);
        // All beam counts should be populated
        assert_eq!(layout.beams_left.len(), 4);
        assert_eq!(layout.beams_right.len(), 4);
    }

    #[test]
    fn single_note_beam_group() {
        let notes = eighth_notes(&[(0.0, 4)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);
        assert_eq!(layout.stem_tip_ys.len(), 1);
        assert_eq!(layout.max_beam_level, 1);
        // Single note: fractional beam on left
        assert_eq!(layout.beams_left[0], 1);
        assert_eq!(layout.beams_right[0], 0);
    }

    #[test]
    fn stems_down_tips_below_noteheads() {
        let notes = eighth_notes(&[(0.0, 6), (300.0, 8), (600.0, 6)]);
        let layout = layout_beam_group(&notes, StemDirection::Down, SS);

        for (i, note) in notes.iter().enumerate() {
            let notehead_y = staff_position_to_y(note.staff_position, HS);
            assert!(
                layout.stem_tip_ys[i] > notehead_y,
                "stem-down tip {} should be below notehead y {}",
                layout.stem_tip_ys[i],
                notehead_y
            );
        }
    }

    // --- staff_position_to_y ---

    #[test]
    fn position_to_y_matches_staff_layout() {
        // Position 8 (top) → y=0, Position 0 (bottom) → y=1000
        assert!((staff_position_to_y(8, HS) - 0.0).abs() < f64::EPSILON);
        assert!((staff_position_to_y(0, HS) - 1000.0).abs() < f64::EPSILON);
        assert!((staff_position_to_y(4, HS) - 500.0).abs() < f64::EPSILON);
        assert!((staff_position_to_y(-2, HS) - 1250.0).abs() < f64::EPSILON);
    }

    // --- beam_group_note_x_offsets ---

    #[test]
    fn beam_group_note_x_offsets_empty_returns_empty() {
        assert!(beam_group_note_x_offsets(&[], 1000.0).is_empty());
    }

    #[test]
    fn beam_group_note_x_offsets_single_note_is_zero() {
        assert_eq!(beam_group_note_x_offsets(&[3], 1000.0), vec![0.0]);
    }

    #[test]
    fn beam_group_note_x_offsets_equal_durations_distribute_evenly() {
        // Four equal eighth notes → step = total_width / 4 each.
        let offsets = beam_group_note_x_offsets(&[3, 3, 3, 3], 1000.0);
        assert_eq!(offsets.len(), 4);
        assert!((offsets[0] - 0.0).abs() < 1e-9);
        assert!((offsets[1] - 250.0).abs() < 1e-9);
        assert!((offsets[2] - 500.0).abs() < 1e-9);
        assert!((offsets[3] - 750.0).abs() < 1e-9);
    }

    #[test]
    fn beam_group_note_x_offsets_first_note_always_zero() {
        for durations in [
            vec![3, 3],
            vec![3, 4, 5],
            vec![4, 3, 4],
            vec![3, 3, 3, 4, 4],
        ] {
            let offsets = beam_group_note_x_offsets(&durations, 1234.5);
            assert!(
                (offsets[0] - 0.0).abs() < 1e-9,
                "offsets[0] must be 0 for durations={durations:?}, got {}",
                offsets[0]
            );
        }
    }

    #[test]
    fn beam_group_note_x_offsets_monotonically_increasing() {
        // For any positive duration sequence, offsets must be strictly increasing
        // (each note consumes positive width).
        let offsets = beam_group_note_x_offsets(&[3, 4, 5, 3], 1000.0);
        for w in offsets.windows(2) {
            assert!(w[1] > w[0], "non-monotonic offsets: {:?}", offsets);
        }
    }

    #[test]
    fn beam_group_note_x_offsets_longer_note_consumes_more_width() {
        // [quarter, eighth]: shortest_log2 = 3 (eighth). Quarter's factor =
        // 1.6^(3-2) = 1.6, eighth's factor = 1.6^0 = 1.0. Total = 2.6.
        // Width consumed by note 0 (quarter) before note 1 = 1000 * 1.6/2.6.
        let offsets = beam_group_note_x_offsets(&[2, 3], 1000.0);
        let expected_step = 1000.0 * 1.6 / 2.6;
        assert!(
            (offsets[1] - expected_step).abs() < 1e-9,
            "expected step {expected_step}, got {}",
            offsets[1]
        );
    }

    #[test]
    fn beam_group_note_x_offsets_total_consumed_strictly_less_than_total_width() {
        // The last note's offset + its share = total_width, so the last
        // offset must be strictly less than total_width (each note occupies
        // positive horizontal space).
        let offsets = beam_group_note_x_offsets(&[3, 3, 3, 3], 1000.0);
        assert!(*offsets.last().unwrap() < 1000.0);
    }

    #[test]
    fn beam_group_note_x_offsets_matches_renderer_logic_three_eighths() {
        // Independent reimplementation of the renderer's spacing logic to
        // assert byte-identical behaviour. Any drift in the helper would
        // break this and silently misalign collision detection from rendering.
        let durations = vec![3u8, 3, 3];
        let total_width = 750.0;
        let shortest = *durations.iter().max().unwrap();
        let factors: Vec<f64> = durations
            .iter()
            .map(|&d| 1.6_f64.powf(shortest as f64 - d as f64))
            .collect();
        let total: f64 = factors.iter().sum();
        let mut expected = Vec::new();
        let mut x = 0.0;
        for f in &factors {
            expected.push(x);
            x += total_width * f / total;
        }
        let got = beam_group_note_x_offsets(&durations, total_width);
        assert_eq!(expected.len(), got.len());
        for (e, g) in expected.iter().zip(&got) {
            assert!((e - g).abs() < 1e-12, "expected {e}, got {g}");
        }
    }
}
