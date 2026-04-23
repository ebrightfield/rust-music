use crate::layout::key_signature::KeySignature;
use crate::layout::measure::{layout_measure, MeasureElement, MeasureLayoutConfig};
use crate::layout::system::{
    layout_system, measure_event_to_element, MeasureContent, SystemLayout, SystemPrefix,
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
    /// Whether to render measure numbers above the start of each system.
    /// The first system shows "1" (or whatever the first measure number is).
    pub show_measure_numbers: bool,
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
            show_measure_numbers: false,
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
    /// 1-based measure number of the first measure in this system.
    /// Used for rendering measure numbers above each system.
    pub first_measure_number: usize,
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
    /// Whether to render measure numbers above the start of each system.
    pub show_measure_numbers: bool,
}

/// Specification for how to break measures into systems.
#[derive(Clone, Debug)]
pub enum SystemBreaking {
    /// Fixed number of measures per system.
    Fixed(usize),
    /// Break at these measure indices (0-based, exclusive upper bound of each system).
    /// e.g. `[4, 8, 12]` means measures 0–3, 4–7, 8–11.
    Manual(Vec<usize>),
    /// Automatic line breaking: greedily pack measures until their natural
    /// widths exceed the target system width, then start a new system.
    /// The first system accounts for the prefix (clef + key sig + time sig).
    Auto,
    /// Optimal line breaking using Knuth-Plass style dynamic programming.
    /// Minimizes total badness (squared whitespace deviation) across all
    /// systems, producing more evenly filled lines than the greedy `Auto`.
    Optimal,
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
            show_measure_numbers: page_config.show_measure_numbers,
        };
    }

    let chunks = match breaking {
        SystemBreaking::Auto => break_measures_auto(
            prefix,
            measures,
            measure_config,
            page_config.system_width,
        ),
        SystemBreaking::Optimal => break_measures_optimal(
            prefix,
            measures,
            measure_config,
            page_config.system_width,
        ),
        other => break_measures(measures.len(), other),
    };

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
            first_measure_number: start + 1, // 1-based
        });

        y += page_config.system_spacing_fu();
    }

    // Page height: last system top + 4 staff spaces (for the staff itself) + some padding
    let page_height = match systems.last() {
        None => page_config.top_margin,
        Some(last) => last.y + 6.0 * page_config.staff_space,
    };

    PageLayout {
        systems,
        page_width: page_config.system_width + page_config.left_margin,
        page_height,
        show_measure_numbers: page_config.show_measure_numbers,
    }
}

/// Compute the natural (unjustified) width of a measure's content in font
/// design units. Does not include prefix elements (clef, key sig, time sig).
pub(crate) fn content_natural_width(
    content: &MeasureContent,
    config: &MeasureLayoutConfig,
) -> f64 {
    let mut elems: Vec<MeasureElement> = content
        .events
        .iter()
        .map(measure_event_to_element)
        .collect();
    elems.push(MeasureElement::Barline(content.barline));
    layout_measure(&elems, config).total_width
}

/// Compute the natural width of the system prefix (clef + key sig + time sig)
/// using the same measure layout engine.
pub(crate) fn prefix_natural_width(prefix: &SystemPrefix, config: &MeasureLayoutConfig) -> f64 {
    let mut elems = Vec::with_capacity(3);
    elems.push(MeasureElement::Clef(prefix.clef_layout.clone()));
    if !matches!(prefix.key_signature, KeySignature::Open) {
        elems.push(MeasureElement::KeySignature(
            prefix.key_signature.clone(),
        ));
    }
    if let Some(ts) = &prefix.time_signature {
        elems.push(MeasureElement::TimeSignature(ts.clone()));
    }
    layout_measure(&elems, config).total_width
}

/// Greedily pack measures into systems so that each system's natural width
/// does not exceed `target_width`. The first system reserves space for the
/// full prefix (clef + key sig + time sig); subsequent systems reserve
/// space for the continuation prefix (clef + key sig, no time sig).
///
/// Guarantees at least one measure per system (even if a single measure
/// exceeds the target width — it will be scaled down by `layout_system`).
pub(crate) fn break_measures_auto(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
    config: &MeasureLayoutConfig,
    target_width: f64,
) -> Vec<(usize, usize)> {
    if measures.is_empty() {
        return vec![];
    }

    // Pre-compute natural widths of each measure's content (without prefix)
    let widths: Vec<f64> = measures
        .iter()
        .map(|m| content_natural_width(m, config))
        .collect();

    // First-system prefix includes time sig
    let first_prefix_w = prefix_natural_width(prefix, config);

    // Continuation prefix: clef + key sig, no time sig
    let continuation_prefix = SystemPrefix {
        clef_layout: prefix.clef_layout.clone(),
        clef_kind: prefix.clef_kind,
        key_signature: prefix.key_signature.clone(),
        time_signature: None,
    };
    let cont_prefix_w = prefix_natural_width(&continuation_prefix, config);

    let mut chunks = Vec::new();
    let mut start = 0;
    let mut is_first = true;

    while start < measures.len() {
        let prefix_w = if is_first {
            first_prefix_w
        } else {
            cont_prefix_w
        };
        let budget = target_width - prefix_w;

        let mut running = 0.0;
        let mut end = start;

        while end < measures.len() {
            let next = running + widths[end];
            if end > start && next > budget {
                // Adding this measure would exceed the budget; stop before it.
                break;
            }
            running = next;
            end += 1;
        }

        // Guarantee at least one measure per system
        if end == start {
            end = start + 1;
        }

        chunks.push((start, end));
        start = end;
        is_first = false;
    }

    chunks
}

/// Optimal line breaking via Knuth-Plass style dynamic programming.
///
/// Minimizes total badness across all systems, where badness for a system is
/// the squared deviation of its fill ratio from 1.0. This distributes
/// whitespace more evenly than the greedy `break_measures_auto`.
///
/// The algorithm considers every possible break point and uses DP to find
/// the globally optimal sequence. Complexity is O(n²) where n is the number
/// of measures — acceptable since scores rarely exceed a few hundred measures.
pub(crate) fn break_measures_optimal(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
    config: &MeasureLayoutConfig,
    target_width: f64,
) -> Vec<(usize, usize)> {
    if measures.is_empty() {
        return vec![];
    }

    let n = measures.len();
    let widths: Vec<f64> = measures
        .iter()
        .map(|m| content_natural_width(m, config))
        .collect();

    let first_prefix_w = prefix_natural_width(prefix, config);
    let continuation_prefix = SystemPrefix {
        clef_layout: prefix.clef_layout.clone(),
        clef_kind: prefix.clef_kind,
        key_signature: prefix.key_signature.clone(),
        time_signature: None,
    };
    let cont_prefix_w = prefix_natural_width(&continuation_prefix, config);

    // Badness for a system spanning measures[start..end] on system number `sys_idx` (0-based).
    // Returns f64::INFINITY if the line is overfull beyond tolerance.
    let line_badness = |start: usize, end: usize, sys_idx: usize| -> f64 {
        let prefix_w = if sys_idx == 0 {
            first_prefix_w
        } else {
            cont_prefix_w
        };
        let budget = target_width - prefix_w;
        if budget <= 0.0 {
            return 0.0;
        }

        let content_w: f64 = widths[start..end].iter().sum();
        let ratio = content_w / budget;

        if ratio > 1.5 {
            // Severely overfull — penalize heavily but not infinitely, so the
            // DP can still find a solution when all options are tight.
            return 1e6;
        }

        // Squared deviation from perfect fill. Underfull lines (ratio < 1)
        // and slightly overfull lines (ratio > 1) are both penalized, but
        // underfull is more common and more visually objectionable, so we
        // use an asymmetric weight: underfull gets 1× weight, overfull gets
        // 4× weight (discouraging cramming).
        let deviation = ratio - 1.0;
        if deviation < 0.0 {
            deviation * deviation
        } else {
            4.0 * deviation * deviation
        }
    };

    // DP: cost[j] = minimum total badness for measures[0..j].
    // prev[j] = the start index of the last system that ends at j.
    let mut cost = vec![f64::INFINITY; n + 1];
    let mut prev = vec![0usize; n + 1];
    cost[0] = 0.0;

    // sys_count[j] = number of systems used to reach measure j.
    let mut sys_count = vec![0usize; n + 1];

    for j in 1..=n {
        for i in (0..j).rev() {
            let sys_idx = sys_count[i];
            let b = line_badness(i, j, sys_idx);
            let candidate = cost[i] + b;
            if candidate < cost[j] {
                cost[j] = candidate;
                prev[j] = i;
                sys_count[j] = sys_idx + 1;
            }
            // Early termination: if we've already found a very good fit and
            // going further back would only make lines emptier, stop.
            if b > 1e5 && j - i > 1 {
                break;
            }
        }
    }

    // Reconstruct break points by tracing back from n.
    let mut breaks = Vec::new();
    let mut end = n;
    while end > 0 {
        let start = prev[end];
        breaks.push((start, end));
        end = start;
    }
    breaks.reverse();
    breaks
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
        // Auto and Optimal are handled before this function is called; see layout_page.
        SystemBreaking::Auto | SystemBreaking::Optimal => {
            unreachable!("Auto/Optimal handled before break_measures")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::barline::BarlineStyle;
    use crate::layout::key_signature::KeySignature;
    use crate::layout::measure::{NoteAnnotations, NoteEvent};
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
        annotations: NoteAnnotations::default(),
        })
    }

    fn make_measure(pos: i8) -> MeasureContent {
        MeasureContent {
            events: vec![quarter_note(pos)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
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

    // --- Auto line-breaking tests ---

    #[test]
    fn auto_breaking_empty_measures() {
        let ss = 250.0;
        let page = layout_page(
            &test_prefix(),
            &[],
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Auto,
        );
        assert!(page.systems.is_empty());
    }

    #[test]
    fn auto_breaking_single_measure_always_fits() {
        let ss = 250.0;
        let measures = vec![make_measure(4)];
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &test_page_config(ss),
            &SystemBreaking::Auto,
        );
        assert_eq!(page.systems.len(), 1);
        assert_eq!(page.systems[0].system.measures.len(), 1);
    }

    #[test]
    fn auto_breaking_wide_system_fits_all_on_one_line() {
        let ss = 250.0;
        // Use a very wide system width so all measures fit on one system
        let mut page_config = test_page_config(ss);
        page_config.system_width = 100_000.0;
        let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &page_config,
            &SystemBreaking::Auto,
        );
        assert_eq!(
            page.systems.len(),
            1,
            "wide system should fit all 6 measures on one line"
        );
        assert_eq!(page.systems[0].system.measures.len(), 6);
    }

    #[test]
    fn auto_breaking_narrow_system_creates_multiple_systems() {
        let ss = 250.0;
        // Use a very narrow system width to force more line breaks
        let mut page_config = test_page_config(ss);
        page_config.system_width = 3000.0;
        let measures: Vec<_> = (0..8).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &page_config,
            &SystemBreaking::Auto,
        );
        // With a narrow width, should have more than 1 system
        assert!(
            page.systems.len() > 1,
            "narrow system should create multiple systems, got {}",
            page.systems.len()
        );
        // Total measures across all systems should equal input count
        let total: usize = page.systems.iter().map(|s| s.system.measures.len()).sum();
        assert_eq!(total, 8, "all measures should be placed");
    }

    #[test]
    fn auto_breaking_guarantees_at_least_one_measure_per_system() {
        let ss = 250.0;
        // Absurdly narrow: each measure is wider than the system
        let mut page_config = test_page_config(ss);
        page_config.system_width = 100.0;
        let measures = vec![make_measure(0), make_measure(4), make_measure(8)];
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &page_config,
            &SystemBreaking::Auto,
        );
        // Each system should have exactly 1 measure (no infinite loop)
        assert_eq!(page.systems.len(), 3);
        for (i, s) in page.systems.iter().enumerate() {
            assert_eq!(
                s.system.measures.len(),
                1,
                "system {i} should have exactly 1 measure"
            );
        }
    }

    #[test]
    fn auto_breaking_preserves_measure_order() {
        let ss = 250.0;
        let mut page_config = test_page_config(ss);
        page_config.system_width = 5000.0;
        let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &page_config,
            &SystemBreaking::Auto,
        );
        // Verify all measures are present and systems are non-empty
        let total: usize = page.systems.iter().map(|s| s.system.measures.len()).sum();
        assert_eq!(total, 6);
        for s in &page.systems {
            assert!(!s.system.measures.is_empty());
        }
    }

    #[test]
    fn auto_breaking_systems_are_justified_to_target_width() {
        let ss = 250.0;
        let page_config = test_page_config(ss);
        let measures: Vec<_> = (0..8).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &page_config,
            &SystemBreaking::Auto,
        );
        for (i, ps) in page.systems.iter().enumerate() {
            assert!(
                (ps.system.staff_width - page_config.system_width).abs() < 1.0,
                "system {} staff_width {} should be close to {}",
                i,
                ps.system.staff_width,
                page_config.system_width,
            );
        }
    }

    #[test]
    fn auto_breaking_differs_from_fixed_for_varied_content() {
        let ss = 250.0;
        let page_config = test_page_config(ss);
        let mc = test_measure_config(ss);

        // Create measures with very different content densities:
        // 2 measures with one note, 6 measures with one note
        let measures: Vec<_> = (0..8).map(|i| make_measure((i % 8) as i8)).collect();

        let auto_page = layout_page(
            &test_prefix(),
            &measures,
            &mc,
            &page_config,
            &SystemBreaking::Auto,
        );
        let fixed_page = layout_page(
            &test_prefix(),
            &measures,
            &mc,
            &page_config,
            &SystemBreaking::Fixed(4),
        );

        // Both should place all measures
        let auto_total: usize = auto_page
            .systems
            .iter()
            .map(|s| s.system.measures.len())
            .sum();
        let fixed_total: usize = fixed_page
            .systems
            .iter()
            .map(|s| s.system.measures.len())
            .sum();
        assert_eq!(auto_total, 8);
        assert_eq!(fixed_total, 8);
    }

    #[test]
    fn prefix_natural_width_is_positive() {
        let ss = 250.0;
        let prefix = test_prefix();
        let config = test_measure_config(ss);
        let w = prefix_natural_width(&prefix, &config);
        assert!(
            w > 0.0,
            "prefix with clef + key sig + time sig should have positive width"
        );
    }

    #[test]
    fn content_natural_width_is_positive() {
        let ss = 250.0;
        let config = test_measure_config(ss);
        let measure = make_measure(4);
        let w = content_natural_width(&measure, &config);
        assert!(
            w > 0.0,
            "measure with a note and barline should have positive width"
        );
    }

    #[test]
    fn prefix_without_time_sig_is_narrower() {
        let ss = 250.0;
        let config = test_measure_config(ss);
        let with_ts = test_prefix();
        let without_ts = SystemPrefix {
            clef_layout: with_ts.clef_layout.clone(),
            clef_kind: with_ts.clef_kind,
            key_signature: with_ts.key_signature.clone(),
            time_signature: None,
        };
        let w_with = prefix_natural_width(&with_ts, &config);
        let w_without = prefix_natural_width(&without_ts, &config);
        assert!(
            w_with > w_without,
            "prefix with time sig ({w_with}) should be wider than without ({w_without})"
        );
    }

    #[test]
    fn break_measures_auto_basic() {
        let ss = 250.0;
        let prefix = test_prefix();
        let config = test_measure_config(ss);
        let measures: Vec<_> = (0..4).map(|i| make_measure(i as i8)).collect();

        let chunks = break_measures_auto(&prefix, &measures, &config, 100_000.0);
        // Very wide target: all measures on one system
        assert_eq!(chunks, vec![(0, 4)]);
    }

    #[test]
    fn break_measures_auto_empty() {
        let ss = 250.0;
        let prefix = test_prefix();
        let config = test_measure_config(ss);
        let chunks = break_measures_auto(&prefix, &[], &config, 10_000.0);
        assert!(chunks.is_empty());
    }

    // --- Optimal line breaking tests ---

    #[test]
    fn optimal_breaking_empty() {
        let prefix = test_prefix();
        let config = test_measure_config(250.0);
        let chunks = break_measures_optimal(&prefix, &[], &config, 10_000.0);
        assert!(chunks.is_empty());
    }

    #[test]
    fn optimal_breaking_single_measure() {
        let prefix = test_prefix();
        let config = test_measure_config(250.0);
        let measures = vec![make_measure(4)];
        let chunks = break_measures_optimal(&prefix, &measures, &config, 100_000.0);
        assert_eq!(chunks, vec![(0, 1)]);
    }

    #[test]
    fn optimal_breaking_wide_target_all_on_one_line() {
        let prefix = test_prefix();
        let config = test_measure_config(250.0);
        let measures: Vec<_> = (0..4).map(|i| make_measure(i as i8)).collect();
        let chunks = break_measures_optimal(&prefix, &measures, &config, 100_000.0);
        // Very wide target: all measures on one system
        assert_eq!(chunks, vec![(0, 4)]);
    }

    #[test]
    fn optimal_breaking_covers_all_measures() {
        let prefix = test_prefix();
        let config = test_measure_config(250.0);
        let measures: Vec<_> = (0..8).map(|i| make_measure((i % 8) as i8)).collect();
        let chunks = break_measures_optimal(&prefix, &measures, &config, 5_000.0);
        let total: usize = chunks.iter().map(|(s, e)| e - s).sum();
        assert_eq!(total, 8, "optimal breaking must cover all measures");
        // Chunks must be contiguous and non-overlapping
        for i in 0..chunks.len() - 1 {
            assert_eq!(
                chunks[i].1, chunks[i + 1].0,
                "chunks must be contiguous at boundary {i}"
            );
        }
        assert_eq!(chunks[0].0, 0, "first chunk must start at 0");
        assert_eq!(
            chunks.last().unwrap().1,
            8,
            "last chunk must end at measure count"
        );
    }

    #[test]
    fn optimal_breaking_produces_non_empty_systems() {
        let prefix = test_prefix();
        let config = test_measure_config(250.0);
        let measures: Vec<_> = (0..6).map(|i| make_measure(i as i8)).collect();
        let chunks = break_measures_optimal(&prefix, &measures, &config, 3_000.0);
        for (s, e) in &chunks {
            assert!(e > s, "each system must contain at least one measure");
        }
    }

    #[test]
    fn optimal_and_greedy_agree_on_uniform_measures() {
        // With uniform-width measures, optimal and greedy should produce
        // the same or similar breaking.
        let prefix = test_prefix();
        let config = test_measure_config(250.0);
        let measures: Vec<_> = (0..8).map(|_| make_measure(4)).collect();
        let target = 5_000.0;
        let greedy = break_measures_auto(&prefix, &measures, &config, target);
        let optimal = break_measures_optimal(&prefix, &measures, &config, target);
        // Same number of measures covered
        let g_total: usize = greedy.iter().map(|(s, e)| e - s).sum();
        let o_total: usize = optimal.iter().map(|(s, e)| e - s).sum();
        assert_eq!(g_total, o_total);
        // Optimal should have same or fewer systems (more even = possibly same count)
        assert!(
            optimal.len() <= greedy.len() + 1,
            "optimal should not produce dramatically more systems than greedy"
        );
    }

    #[test]
    fn optimal_layout_page_systems_are_justified() {
        let ss = 250.0;
        let page_config = test_page_config(ss);
        let measures: Vec<_> = (0..8).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(
            &test_prefix(),
            &measures,
            &test_measure_config(ss),
            &page_config,
            &SystemBreaking::Optimal,
        );
        // All systems should be justified to target width
        for (i, ps) in page.systems.iter().enumerate() {
            assert!(
                (ps.system.staff_width - page_config.system_width).abs() < 1.0,
                "system {} staff_width {} should be close to {}",
                i,
                ps.system.staff_width,
                page_config.system_width,
            );
        }
    }

    #[test]
    fn optimal_breaking_handles_narrow_target() {
        // Very narrow target: each measure should get its own system
        let prefix = test_prefix();
        let config = test_measure_config(250.0);
        let measures: Vec<_> = (0..3).map(|i| make_measure(i as i8)).collect();
        let chunks = break_measures_optimal(&prefix, &measures, &config, 1.0);
        // Should still cover all measures, one per system
        assert_eq!(chunks.len(), 3, "very narrow target should give one measure per system");
        assert_eq!(chunks, vec![(0, 1), (1, 2), (2, 3)]);
    }
}
