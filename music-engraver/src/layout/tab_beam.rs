//! Beam layout for rhythm stems above tablature staves.
//!
//! Tab beams are simpler than standard notation beams: all stems point up
//! from a fixed height above the staff, so beams are horizontal lines
//! connecting stem tips at the same y-coordinate.

use super::tab::TabStaffLayout;
use super::tab_rhythm::{layout_tab_rhythm, tab_flag_count, TabRhythmLayout};

/// A note in a tab beam group.
#[derive(Clone, Debug)]
pub struct TabBeamedNote {
    /// X-coordinate of the note/fret number position.
    pub x: f64,
    /// Duration log2 (3=eighth, 4=sixteenth, etc.). Must be >= 3 for beaming.
    pub duration_log2: u8,
}

/// Computed layout for a tab beam group.
#[derive(Clone, Debug)]
pub struct TabBeamGroupLayout {
    /// Individual stem layouts (one per note in the group).
    pub stems: Vec<TabRhythmLayout>,
    /// Number of beam levels: 1 for all-eighths, 2 if any sixteenths, etc.
    pub max_beam_level: u8,
    /// Per-note beam counts on the left side.
    pub beams_left: Vec<u8>,
    /// Per-note beam counts on the right side.
    pub beams_right: Vec<u8>,
    /// Beam thickness in font design units.
    pub beam_thickness: f64,
    /// Gap between stacked beams in font design units.
    pub beam_gap: f64,
}

/// Compute beam connectivity for a group of tab notes.
///
/// Uses the same rules as standard notation beaming: primary beam spans the
/// entire group, secondary beams connect adjacent notes with matching depth.
/// Isolated secondary beams become fractional stubs pointing inward.
pub fn compute_tab_beam_counts(notes: &[TabBeamedNote]) -> (Vec<u8>, Vec<u8>) {
    let n = notes.len();
    if n == 0 {
        return (vec![], vec![]);
    }

    let mut beams_left = vec![0u8; n];
    let mut beams_right = vec![0u8; n];

    for i in 0..n {
        let own = tab_flag_count(notes[i].duration_log2).max(1);
        let left_beams = if i > 0 {
            tab_flag_count(notes[i - 1].duration_log2).max(1)
        } else {
            0
        };
        let right_beams = if i + 1 < n {
            tab_flag_count(notes[i + 1].duration_log2).max(1)
        } else {
            0
        };

        beams_left[i] = own.min(left_beams.max(1).min(own));
        beams_right[i] = own.min(right_beams.max(1).min(own));

        // First note has no left beams; last note has no right beams
        if i == 0 {
            beams_left[i] = 0;
        }
        if i == n - 1 {
            beams_right[i] = 0;
        }

        // Ensure connectivity: if a note has more beams than both neighbors,
        // the extras become fractional stubs. For the primary beam, always
        // connect across the full group.
        if own > left_beams && own > right_beams && n > 1 {
            // Fractional stubs point toward the nearer neighbor, or left by default.
            // Both left and right get at least 1 for the primary beam.
            if i > 0 {
                beams_left[i] = beams_left[i].max(own.min(left_beams).max(1));
            }
            if i + 1 < n {
                beams_right[i] = beams_right[i].max(own.min(right_beams).max(1));
            }
        }
    }

    (beams_left, beams_right)
}

/// Layout a tab beam group.
///
/// All notes get stems at the fixed position above the staff. The beam
/// connects stem tips horizontally. Returns `None` if fewer than 2 notes
/// or none have beamable durations (duration_log2 >= 3).
pub fn layout_tab_beam_group(
    tab_staff: &TabStaffLayout,
    notes: &[TabBeamedNote],
    stem_width: f64,
    beam_thickness: f64,
    beam_gap: f64,
) -> Option<TabBeamGroupLayout> {
    if notes.len() < 2 {
        return None;
    }

    // All notes must be eighth or shorter (duration_log2 >= 3)
    if !notes.iter().all(|n| n.duration_log2 >= 3) {
        return None;
    }

    let max_beam_level = notes
        .iter()
        .map(|n| tab_flag_count(n.duration_log2))
        .max()
        .unwrap_or(1)
        .max(1);

    let stems: Vec<_> = notes
        .iter()
        .filter_map(|n| layout_tab_rhythm(tab_staff, n.x, n.duration_log2, stem_width))
        .collect();

    if stems.len() != notes.len() {
        return None;
    }

    let (beams_left, beams_right) = compute_tab_beam_counts(notes);

    Some(TabBeamGroupLayout {
        stems,
        max_beam_level,
        beams_left,
        beams_right,
        beam_thickness,
        beam_gap,
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

    fn make_notes(xs: &[f64], dur: u8) -> Vec<TabBeamedNote> {
        xs.iter()
            .map(|&x| TabBeamedNote {
                x,
                duration_log2: dur,
            })
            .collect()
    }

    #[test]
    fn two_eighths_produces_layout() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1500.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0);
        assert!(layout.is_some());
        let l = layout.unwrap();
        assert_eq!(l.stems.len(), 2);
        assert_eq!(l.max_beam_level, 1);
    }

    #[test]
    fn four_sixteenths_has_two_beam_levels() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1200.0, 1400.0, 1600.0], 4);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        assert_eq!(layout.max_beam_level, 2);
    }

    #[test]
    fn single_note_returns_none() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0], 3);
        assert!(layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).is_none());
    }

    #[test]
    fn empty_returns_none() {
        let staff = guitar_staff();
        assert!(layout_tab_beam_group(&staff, &[], 5.0, 20.0, 10.0).is_none());
    }

    #[test]
    fn quarter_notes_not_beamable() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1500.0], 2);
        assert!(layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).is_none());
    }

    #[test]
    fn all_stems_at_same_y() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1500.0, 2000.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        let y_tips: Vec<f64> = layout.stems.iter().map(|s| s.y_tip).collect();
        for y in &y_tips {
            assert!(
                (*y - y_tips[0]).abs() < 0.01,
                "all stem tips should be at same y: {y_tips:?}"
            );
        }
    }

    #[test]
    fn stems_have_correct_x_positions() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 2000.0, 3000.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        assert!((layout.stems[0].x - 1000.0).abs() < 0.01);
        assert!((layout.stems[1].x - 2000.0).abs() < 0.01);
        assert!((layout.stems[2].x - 3000.0).abs() < 0.01);
    }

    #[test]
    fn beam_counts_two_eighths() {
        let notes = make_notes(&[100.0, 200.0], 3);
        let (bl, br) = compute_tab_beam_counts(&notes);
        assert_eq!(bl[0], 0, "first note has no left beam");
        assert_eq!(br[0], 1, "first note has right beam");
        assert_eq!(bl[1], 1, "last note has left beam");
        assert_eq!(br[1], 0, "last note has no right beam");
    }

    #[test]
    fn beam_counts_four_sixteenths() {
        let notes = make_notes(&[100.0, 200.0, 300.0, 400.0], 4);
        let (bl, br) = compute_tab_beam_counts(&notes);
        // Primary beam: all connected
        assert_eq!(bl[0], 0);
        assert_eq!(br[0], 2);
        assert_eq!(bl[1], 2);
        assert_eq!(br[1], 2);
        assert_eq!(bl[2], 2);
        assert_eq!(br[2], 2);
        assert_eq!(bl[3], 2);
        assert_eq!(br[3], 0);
    }

    #[test]
    fn beam_counts_empty() {
        let (bl, br) = compute_tab_beam_counts(&[]);
        assert!(bl.is_empty());
        assert!(br.is_empty());
    }

    #[test]
    fn mixed_durations_eighths_and_sixteenths() {
        let notes = vec![
            TabBeamedNote { x: 100.0, duration_log2: 3 },
            TabBeamedNote { x: 200.0, duration_log2: 4 },
            TabBeamedNote { x: 300.0, duration_log2: 4 },
            TabBeamedNote { x: 400.0, duration_log2: 3 },
        ];
        let staff = guitar_staff();
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        assert_eq!(layout.max_beam_level, 2);
        // Primary beam spans all 4
        assert!(layout.beams_left[0] == 0);
        assert!(layout.beams_right[0] >= 1);
        assert!(layout.beams_left[3] >= 1);
        assert!(layout.beams_right[3] == 0);
    }

    #[test]
    fn beam_thickness_and_gap_preserved() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1500.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 25.0, 12.0).unwrap();
        assert!((layout.beam_thickness - 25.0).abs() < 0.01);
        assert!((layout.beam_gap - 12.0).abs() < 0.01);
    }

    #[test]
    fn thirty_second_notes_have_three_levels() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1200.0], 5);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        assert_eq!(layout.max_beam_level, 3);
    }
}
