//! Layout geometry for tablature vibrato notation.
//!
//! Vibrato is shown as a wavy line above the fret number, indicating
//! the player should oscillate the string to vary the pitch slightly.
//! Wide vibrato uses a taller wave amplitude.

use super::tab::TabStaffLayout;

/// Vibrato wave amplitude relative to staff space.
const VIBRATO_AMPLITUDE_RATIO: f64 = 0.25;

/// Wide vibrato uses a taller wave.
const WIDE_VIBRATO_AMPLITUDE_RATIO: f64 = 0.45;

/// Vibrato wave half-period (horizontal distance per half-cycle)
/// relative to staff space.
const VIBRATO_HALF_PERIOD_RATIO: f64 = 0.25;

/// Vertical offset from the string line to the vibrato wave center,
/// in staff spaces. Positioned above the fret number.
const VIBRATO_ABOVE_STRING_SS: f64 = 0.8;

/// Default vibrato line width (number of half-cycles).
const DEFAULT_WIDTH_HALF_CYCLES: usize = 6;

/// Vibrato intensity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VibratoKind {
    /// Standard vibrato — moderate wave amplitude.
    Normal,
    /// Wide vibrato — exaggerated wave amplitude.
    Wide,
}

/// Layout result for a vibrato wavy line above a tab fret number.
#[derive(Clone, Debug)]
pub struct TabVibratoLayout {
    /// SVG path data for the wavy line (a sequence of quadratic Bézier curves).
    pub path_data: String,
    /// Stroke width for the wavy line.
    pub stroke_width: f64,
}

/// Compute layout for a vibrato wavy line above a fret position.
///
/// The wave is centered horizontally on `x` and positioned above the
/// string line. The path is a series of quadratic Bézier half-cycles
/// forming a sine-like oscillation.
pub fn layout_tab_vibrato(
    tab_staff: &TabStaffLayout,
    string: u8,
    x: f64,
    kind: VibratoKind,
    stroke_width: f64,
) -> TabVibratoLayout {
    let ss = tab_staff.staff_space;
    let amplitude = ss
        * match kind {
            VibratoKind::Normal => VIBRATO_AMPLITUDE_RATIO,
            VibratoKind::Wide => WIDE_VIBRATO_AMPLITUDE_RATIO,
        };
    let half_period = ss * VIBRATO_HALF_PERIOD_RATIO;
    let n = DEFAULT_WIDTH_HALF_CYCLES;
    let total_width = n as f64 * half_period;

    let string_y = tab_staff.string_y(string);
    let center_y = string_y - ss * VIBRATO_ABOVE_STRING_SS;
    let start_x = x - total_width / 2.0;

    // Build path: M start, then alternating Q curves up/down
    let mut path = format!("M{:.2} {:.2}", start_x, center_y);
    for i in 0..n {
        let end_x = start_x + (i + 1) as f64 * half_period;
        let ctrl_x = start_x + (i as f64 + 0.5) * half_period;
        // Odd half-cycles go up (negative y), even go down
        let ctrl_y = if i % 2 == 0 {
            center_y - amplitude
        } else {
            center_y + amplitude
        };
        path.push_str(&format!(
            " Q{:.2} {:.2} {:.2} {:.2}",
            ctrl_x, ctrl_y, end_x, center_y
        ));
    }

    TabVibratoLayout {
        path_data: path,
        stroke_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staff() -> TabStaffLayout {
        TabStaffLayout::new(0.0, 0.0, 1000.0, 250.0, 6)
    }

    #[test]
    fn layout_returns_nonempty_path() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        assert!(!layout.path_data.is_empty());
        assert!(layout.path_data.starts_with('M'));
    }

    #[test]
    fn path_contains_quadratic_bezier_commands() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 3, 500.0, VibratoKind::Normal, 5.0);
        let q_count = layout.path_data.matches(" Q").count();
        assert_eq!(q_count, DEFAULT_WIDTH_HALF_CYCLES);
    }

    #[test]
    fn wide_vibrato_has_larger_amplitude_in_path() {
        let staff = test_staff();
        let normal = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        let wide = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Wide, 5.0);
        // Different paths due to different amplitude
        assert_ne!(normal.path_data, wide.path_data);
    }

    #[test]
    fn stroke_width_preserved() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 7.5);
        assert!((layout.stroke_width - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn different_strings_produce_different_paths() {
        let staff = test_staff();
        let s1 = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        let s6 = layout_tab_vibrato(&staff, 6, 500.0, VibratoKind::Normal, 5.0);
        assert_ne!(s1.path_data, s6.path_data);
    }

    #[test]
    fn different_x_positions_produce_different_paths() {
        let staff = test_staff();
        let a = layout_tab_vibrato(&staff, 1, 300.0, VibratoKind::Normal, 5.0);
        let b = layout_tab_vibrato(&staff, 1, 700.0, VibratoKind::Normal, 5.0);
        assert_ne!(a.path_data, b.path_data);
    }

    #[test]
    fn path_starts_at_correct_x_region() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        // Path should start near x=500 (centered), shifted left by half total width
        let half_period = 250.0 * VIBRATO_HALF_PERIOD_RATIO;
        let total_width = DEFAULT_WIDTH_HALF_CYCLES as f64 * half_period;
        let expected_start_x = 500.0 - total_width / 2.0;
        let prefix = format!("M{:.2}", expected_start_x);
        assert!(
            layout.path_data.starts_with(&prefix),
            "expected path to start with '{}', got '{}'",
            prefix,
            &layout.path_data[..50.min(layout.path_data.len())]
        );
    }

    #[test]
    fn wave_positioned_above_string_line() {
        let staff = test_staff();
        let string_y = staff.string_y(3);
        let layout = layout_tab_vibrato(&staff, 3, 500.0, VibratoKind::Normal, 5.0);
        // The M command y coordinate should be above the string line
        let parts: Vec<&str> = layout.path_data.split_whitespace().collect();
        // parts[0] is "M<x>" or "M<x> <y>" — parse M value
        let m_part = parts[0].trim_start_matches('M');
        let start_y: f64 = parts[1].parse().unwrap();
        let _ = m_part; // x coordinate
        assert!(
            start_y < string_y,
            "vibrato y ({}) should be above string y ({})",
            start_y,
            string_y
        );
    }

    #[test]
    fn default_half_cycles_count() {
        assert_eq!(DEFAULT_WIDTH_HALF_CYCLES, 6);
    }
}
