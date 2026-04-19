use crate::layout::measure::MeasureLayoutConfig;
use crate::layout::system::{
    layout_system, MeasureContent, SystemLayout, SystemPrefix,
};

/// Configuration for page-level layout.
#[derive(Clone, Debug)]
pub struct PageLayoutConfig {
    /// Width of each system (staff lines) in font design units.
    pub system_width: f64,
    /// Vertical distance between the top line of one system and the top line
    /// of the next, in staff spaces. Typical range: 8–14 staff spaces depending
    /// on whether lyrics, dynamics, or other annotations are present.
    pub system_spacing_ss: f64,
    /// Left margin in font design units.
    pub left_margin: f64,
    /// Top margin (y of first system's top line) in font design units.
    pub top_margin: f64,
    /// Staff space in font design units (from the font).
    pub staff_space: f64,
}

impl PageLayoutConfig {
    /// Construct with sensible defaults for the given staff space.
    pub fn new(staff_space: f64, system_width: f64) -> Self {
        Self {
            system_width,
            // 10 staff spaces between system top lines: 4 for the staff itself
            // plus 6 of clearance — a comfortable default.
            system_spacing_ss: 10.0,
            left_margin: 0.0,
            top_margin: 0.0,
            staff_space,
        }
    }

    /// System spacing in font design units.
    pub fn system_spacing_fu(&self) -> f64 {
        self.system_spacing_ss * self.staff_space
    }
}

/// A system positioned on a page, with its vertical offset.
#[derive(Clone, Debug)]
pub struct PageSystem {
    /// Horizontal offset (usually `left_margin`).
    pub x: f64,
    /// Vertical offset of this system's top staff line.
    pub y: f64,
    /// The laid-out system.
    pub system: SystemLayout,
}

/// A complete page of music: multiple systems stacked vertically.
#[derive(Clone, Debug)]
pub struct PageLayout {
    /// Systems arranged top-to-bottom.
    pub systems: Vec<PageSystem>,
    /// Total page width in font design units.
    pub page_width: f64,
    /// Total page height in font design units (from top margin to bottom of last system + some padding).
    pub page_height: f64,
}

/// Specification for how to break measures into systems.
///
/// For v1 this is a simple fixed-measures-per-system model. A smarter
/// line-breaking algorithm (Knuth-Plass style) can replace this later
/// without changing the rest of the pipeline.
#[derive(Clone, Debug)]
pub enum SystemBreaking {
    /// Fixed number of measures per system.
    Fixed(usize),
    /// Break at these measure indices (0-based, exclusive upper bound of each system).
    /// e.g. `[4, 8, 12]` means measures 0–3, 4–7, 8–11.
    Manual(Vec<usize>),
}

/// Lay out a full page of music.
///
/// Takes a prefix (clef, key sig, time sig) and a flat list of measures,
/// breaks them into systems per the `breaking` strategy, and positions
/// each system vertically on the page.
pub fn layout_page(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
    measure_config: &MeasureLayoutConfig,
    page_config: &PageLayoutConfig,
    breaking: &SystemBreaking,
) -> PageLayout {
    if measures.is_empty() {
        return PageLayout {
            systems: vec![],
            page_width: page_config.system_width + page_config.left_margin,
            page_height: page_config.top_margin,
        };
    }

    let chunks = break_measures(measures.len(), breaking);

    let mut systems = Vec::with_capacity(chunks.len());
    let mut y = page_config.top_margin;

    for (i, (start, end)) in chunks.iter().enumerate() {
        let slice = &measures[*start..*end];

        // First system gets full prefix; subsequent systems get prefix without
        // time signature (conventional: time sig only on first system).
        let sys_prefix = if i == 0 {
            prefix.clone()
        } else {
            SystemPrefix {
                clef_layout: prefix.clef_layout.clone(),
                clef_kind: prefix.clef_kind,
                key_signature: prefix.key_signature.clone(),
                time_signature: None,
            }
        };

        let system = layout_system(
            &sys_prefix,
            slice,
            measure_config,
            Some(page_config.system_width),
        );

        systems.push(PageSystem {
            x: page_config.left_margin,
            y,
            system,
        });

        y += page_config.system_spacing_fu();
    }

    // Page height: last system top + 4 staff spaces (for the staff itself) + some padding
    let page_height = if systems.is_empty() {
        page_config.top_margin
    } else {
        let last_y = systems.last().unwrap().y;
        // Staff occupies 4 staff spaces below top line; add 2 more for bottom padding
        last_y + 6.0 * page_config.staff_space
    };

    PageLayout {
        systems,
        page_width: page_config.system_width + page_config.left_margin,
        page_height,
    }
}

/// Split N measures into (start, end) ranges per the breaking strategy.
fn break_measures(n: usize, breaking: &SystemBreaking) -> Vec<(usize, usize)> {
    match breaking {
        SystemBreaking::Fixed(per_system) => {
            let per = (*per_system).max(1);
            let mut chunks = Vec::new();
            let mut start = 0;
            while start < n {
                let end = (start + per).min(n);
                chunks.push((start, end));
                start = end;
            }
            chunks
        }
        SystemBreaking::Manual(breaks) => {
            let mut chunks = Vec::new();
            let mut start = 0;
            for &end in breaks {
                let end = end.min(n);
                if end > start {
                    chunks.push((start, end));
                }
                start = end;
                if start >= n {
                    break;
                }
            }
            // Any remaining measures go in a final system
            if start < n {
                chunks.push((start, n));
            }
            chunks
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::barline::BarlineStyle;
    use crate::layout::key_signature::KeySignature;
    use crate::layout::measure::NoteEvent;
    use crate::layout::system::{MeasureEvent, SystemPrefix};
    use crate::layout::time_signature::TimeSignatureKind;
    use music::notation::clef::Clef;

    fn test_prefix() -> SystemPrefix {
        SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Sharps(2),
            Some(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
        )
    }

    fn quarter_note(pos: i8) -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        })
    }

    fn make_measure(pos: i8) -> MeasureContent {
        MeasureContent {
            events: vec![quarter_note(pos)],
            barline: BarlineStyle::Single,
        }
    }

    fn test_page_config(ss: f64) -> PageLayoutConfig {
        PageLayoutConfig::new(ss, 10000.0)
    }

    fn test_measure_config(ss: f64) -> MeasureLayoutConfig {
        MeasureLayoutConfig::from_staff_space(ss)
    }

    // --- break_measures tests ---

    #[test]
    fn break_fixed_evenly_divisible() {
        let chunks = break_measures(8, &SystemBreaking::Fixed(4));
        assert_eq!(chunks, vec![(0, 4), (4, 8)]);
    }

    #[test]
    fn break_fixed_remainder() {
        let chunks = break_measures(7, &SystemBreaking::Fixed(3));
        assert_eq!(chunks, vec![(0, 3), (3, 6), (6, 7)]);
    }

    #[test]
    fn break_fixed_more_per_system_than_measures() {
        let chunks = break_measures(2, &SystemBreaking::Fixed(5));
        assert_eq!(chunks, vec![(0, 2)]);
    }

    #[test]
    fn break_fixed_zero_clamps_to_one() {
        let chunks = break_measures(3, &SystemBreaking::Fixed(0));
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn break_manual_basic() {
        let chunks = break_measures(10, &SystemBreaking::Manual(vec![3, 6, 10]));
        assert_eq!(chunks, vec![(0, 3), (3, 6), (6, 10)]);
    }

    #[test]
    fn break_manual_with_remainder() {
        let chunks = break_measures(10, &SystemBreaking::Manual(vec![4, 7]));
        assert_eq!(chunks, vec![(0, 4), (4, 7), (7, 10)]);
    }

    #[test]
    fn break_manual_exceeds_count() {
        let chunks = break_measures(5, &SystemBreaking::Manual(vec![3, 20]));
        assert_eq!(chunks, vec![(0, 3), (3, 5)]);
    }

    // --- layout_page tests ---

    #[test]
    fn empty_measures_produce_empty_page() {
        let ss = 250.0;
        let page = layout_page(
            &test_prefix(),
            &[],
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Fixed(4),
        );
        assert!(page.systems.is_empty());
    }

    #[test]
    fn single_system_layout() {
        let ss = 250.0;
        let measures: Vec<_> = (0..4).map(|i| make_measure(i * 2)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Fixed(4),
        );
        assert_eq!(page.systems.len(), 1);
        assert_eq!(page.systems[0].system.measures.len(), 4);
        assert!((page.systems[0].y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn two_systems_layout() {
        let ss = 250.0;
        let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Fixed(3),
        );
        assert_eq!(page.systems.len(), 2);
        assert_eq!(page.systems[0].system.measures.len(), 3);
        assert_eq!(page.systems[1].system.measures.len(), 3);

        // Second system should be offset by system_spacing
        let expected_spacing = 10.0 * ss;
        assert!(
            (page.systems[1].y - expected_spacing).abs() < 1e-6,
            "second system y {} should be {}",
            page.systems[1].y,
            expected_spacing,
        );
    }

    #[test]
    fn three_systems_monotonic_y() {
        let ss = 250.0;
        let measures: Vec<_> = (0..9).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Fixed(3),
        );
        assert_eq!(page.systems.len(), 3);
        for i in 1..page.systems.len() {
            assert!(
                page.systems[i].y > page.systems[i - 1].y,
                "system {} y ({}) should be below system {} y ({})",
                i,
                page.systems[i].y,
                i - 1,
                page.systems[i - 1].y,
            );
        }
    }

    #[test]
    fn systems_justified_to_system_width() {
        let ss = 250.0;
        let config = test_page_config(ss);
        let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &config,
            &SystemBreaking::Fixed(3),
        );
        for (i, ps) in page.systems.iter().enumerate() {
            assert!(
                (ps.system.staff_width - config.system_width).abs() < 1.0,
                "system {} staff_width {} should be close to {}",
                i,
                ps.system.staff_width,
                config.system_width,
            );
        }
    }

    #[test]
    fn first_system_has_time_sig_subsequent_do_not() {
        let ss = 250.0;
        let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Fixed(3),
        );

        // First system's first measure should have time sig element
        let first_elems = &page.systems[0].system.measures[0].layout.elements;
        let has_time_sig = first_elems
            .iter()
            .any(|e| matches!(e.element, crate::layout::measure::MeasureElement::TimeSignature(_)));
        assert!(has_time_sig, "first system should have time signature");

        // Second system's first measure should NOT have time sig
        let second_elems = &page.systems[1].system.measures[0].layout.elements;
        let has_time_sig_2 = second_elems
            .iter()
            .any(|e| matches!(e.element, crate::layout::measure::MeasureElement::TimeSignature(_)));
        assert!(!has_time_sig_2, "second system should not have time signature");
    }

    #[test]
    fn page_height_covers_all_systems() {
        let ss = 250.0;
        let measures: Vec<_> = (0..9).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Fixed(3),
        );
        let last_y = page.systems.last().unwrap().y;
        assert!(
            page.page_height > last_y,
            "page height {} should exceed last system y {}",
            page.page_height,
            last_y,
        );
        // Should include at least 4 staff spaces for the staff itself
        assert!(
            page.page_height >= last_y + 4.0 * ss,
            "page height should include staff height below last system",
        );
    }

    #[test]
    fn left_margin_offsets_systems() {
        let ss = 250.0;
        let mut config = test_page_config(ss);
        config.left_margin = 500.0;
        let measures = vec![make_measure(4)];
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &config,
            &SystemBreaking::Fixed(4),
        );
        assert!((page.systems[0].x - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn top_margin_offsets_first_system() {
        let ss = 250.0;
        let mut config = test_page_config(ss);
        config.top_margin = 300.0;
        let measures = vec![make_measure(4)];
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &config,
            &SystemBreaking::Fixed(4),
        );
        assert!((page.systems[0].y - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn manual_breaking_respects_boundaries() {
        let ss = 250.0;
        let measures: Vec<_> = (0..10).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Manual(vec![2, 6, 10]),
        );
        assert_eq!(page.systems.len(), 3);
        assert_eq!(page.systems[0].system.measures.len(), 2);
        assert_eq!(page.systems[1].system.measures.len(), 4);
        assert_eq!(page.systems[2].system.measures.len(), 4);
    }
}
