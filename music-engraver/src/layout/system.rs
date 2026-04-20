use music::notation::clef::Clef;

use crate::layout::barline::BarlineStyle;
use crate::layout::clef::ClefLayout;
use crate::layout::key_signature::KeySignature;
use crate::layout::measure::{
    layout_measure, BeamGroupEvent, ChordEvent, MeasureElement, MeasureLayout,
    MeasureLayoutConfig, NoteEvent, RestEvent, TupletGroupEvent,
};
use crate::layout::time_signature::TimeSignatureKind;

/// Input description of a single measure's musical content (no layout yet).
#[derive(Clone, Debug)]
pub struct MeasureContent {
    /// Note and rest events in temporal order.
    pub events: Vec<MeasureEvent>,
    /// Barline at the end of this measure.
    pub barline: BarlineStyle,
}

/// A rhythmic event within a measure — a note, rest, chord, or beam group.
#[derive(Clone, Debug)]
pub enum MeasureEvent {
    Note(NoteEvent),
    Rest(RestEvent),
    Chord(ChordEvent),
    /// A group of notes connected by beams (eighth notes or shorter).
    BeamGroup(BeamGroupEvent),
    /// A tuplet group: beamed notes with a tuplet bracket and number.
    TupletGroup(TupletGroupEvent),
}

/// Describes the frontmatter (clef, key, time sig) that appears at the start of a system.
///
/// `clef_layout` is the visual layout. The original `Clef` is stored internally
/// (as a `ClefKind`) for key sig accidental placement and note rendering.
#[derive(Clone, Debug)]
pub struct SystemPrefix {
    pub clef_layout: ClefLayout,
    pub clef_kind: ClefKind,
    pub key_signature: KeySignature,
    pub time_signature: Option<TimeSignatureKind>,
}

/// Local mirror of `music::Clef` variants that derives Clone/Copy/Debug.
/// `music::Clef` lacks these derives and we cannot modify it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClefKind {
    Treble,
    Treble8va,
    Treble8ba,
    Bass,
}

impl ClefKind {
    /// Convert a `&Clef` reference into its `ClefKind` equivalent.
    pub fn from_clef(clef: &Clef) -> Self {
        match clef {
            Clef::Treble => Self::Treble,
            Clef::Treble8va => Self::Treble8va,
            Clef::Treble8ba => Self::Treble8ba,
            Clef::Bass => Self::Bass,
        }
    }

    /// Convert back to a `music::Clef` value.
    pub fn to_clef(self) -> Clef {
        match self {
            Self::Treble => Clef::Treble,
            Self::Treble8va => Clef::Treble8va,
            Self::Treble8ba => Clef::Treble8ba,
            Self::Bass => Clef::Bass,
        }
    }
}

impl SystemPrefix {
    /// Create a prefix from a `Clef`, resolving the layout automatically.
    pub fn new(clef: &Clef, key_signature: KeySignature, time_signature: Option<TimeSignatureKind>) -> Self {
        Self {
            clef_layout: ClefLayout::from_clef_ref(clef),
            clef_kind: ClefKind::from_clef(clef),
            key_signature,
            time_signature,
        }
    }
}

/// A laid-out system: multiple measures arranged horizontally on one line.
#[derive(Clone, Debug)]
pub struct SystemLayout {
    /// The clef used for this system (for key signature accidental placement, note rendering).
    pub clef_kind: ClefKind,
    /// Laid-out measures with their x-offsets within the system.
    pub measures: Vec<SystemMeasure>,
    /// Total width of the system in font design units.
    pub total_width: f64,
    /// Staff width (may equal total_width, or may be fixed).
    pub staff_width: f64,
}

/// A single measure within a laid-out system, with its horizontal offset.
#[derive(Clone, Debug)]
pub struct SystemMeasure {
    /// X-offset of this measure's left edge within the system.
    pub x_offset: f64,
    /// The laid-out measure.
    pub layout: MeasureLayout,
}

/// Lay out a system of measures.
///
/// The first measure gets the system prefix (clef, key sig, optional time sig).
/// Subsequent measures get only their rhythmic content plus barlines.
/// Measures are arranged left-to-right with no gap between them.
///
/// If `target_width` is `Some(w)`, the layout will scale note spacing so the
/// system fills exactly that width. If `None`, measures use natural widths.
pub fn layout_system(
    prefix: &SystemPrefix,
    measures: &[MeasureContent],
    config: &MeasureLayoutConfig,
    target_width: Option<f64>,
) -> SystemLayout {
    if measures.is_empty() {
        return SystemLayout {
            clef_kind: prefix.clef_kind,
            measures: vec![],
            total_width: 0.0,
            staff_width: 0.0,
        };
    }

    // First pass: lay out each measure at natural width
    let mut measure_elements: Vec<Vec<MeasureElement>> = Vec::with_capacity(measures.len());

    // First measure: prefix + events + barline
    let mut first_elems = Vec::new();
    first_elems.push(MeasureElement::Clef(prefix.clef_layout.clone()));
    if !matches!(prefix.key_signature, KeySignature::Open) {
        first_elems.push(MeasureElement::KeySignature(prefix.key_signature.clone()));
    }
    if let Some(ts) = &prefix.time_signature {
        first_elems.push(MeasureElement::TimeSignature(ts.clone()));
    }
    for event in &measures[0].events {
        first_elems.push(measure_event_to_element(event));
    }
    first_elems.push(MeasureElement::Barline(measures[0].barline));
    measure_elements.push(first_elems);

    // Subsequent measures: events + barline only
    for measure in &measures[1..] {
        let mut elems = Vec::new();
        for event in &measure.events {
            elems.push(measure_event_to_element(event));
        }
        elems.push(MeasureElement::Barline(measure.barline));
        measure_elements.push(elems);
    }

    // Lay out each measure
    let mut layouts: Vec<MeasureLayout> = measure_elements
        .iter()
        .map(|elems| layout_measure(elems, config))
        .collect();

    let natural_width: f64 = layouts.iter().map(|l| l.total_width).sum();

    // If target width is specified, scale to fit
    if let Some(target) = target_width {
        if natural_width > 0.0 {
            let scale = target / natural_width;
            for layout in &mut layouts {
                for elem in &mut layout.elements {
                    elem.x *= scale;
                    elem.width *= scale;
                }
                layout.total_width *= scale;
            }
        }
    }

    // Assign x-offsets
    let mut system_measures = Vec::with_capacity(layouts.len());
    let mut x = 0.0;
    for layout in layouts {
        system_measures.push(SystemMeasure {
            x_offset: x,
            layout: layout.clone(),
        });
        x += layout.total_width;
    }

    let total = x;
    let staff_w = target_width.unwrap_or(total);

    SystemLayout {
        clef_kind: prefix.clef_kind,
        measures: system_measures,
        total_width: total,
        staff_width: staff_w,
    }
}

fn measure_event_to_element(event: &MeasureEvent) -> MeasureElement {
    match event {
        MeasureEvent::Note(n) => MeasureElement::Note(n.clone()),
        MeasureEvent::Rest(r) => MeasureElement::Rest(r.clone()),
        MeasureEvent::Chord(c) => MeasureElement::Chord(c.clone()),
        MeasureEvent::BeamGroup(bg) => MeasureElement::BeamGroup(bg.clone()),
        MeasureEvent::TupletGroup(tg) => MeasureElement::TupletGroup(tg.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_config() -> MeasureLayoutConfig {
        MeasureLayoutConfig::from_staff_space(250.0)
    }

    fn test_prefix() -> SystemPrefix {
        SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Open,
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
        dynamic: None,
        slur_start: false,
        slur_end: false,
        hairpin_start: None,
        hairpin_end: false,
        rehearsal_mark: None, tempo_mark: None, expression: None,
        })
    }

    fn quarter_rest() -> MeasureEvent {
        MeasureEvent::Rest(RestEvent {
            duration_log2: 2,
            dots: 0,
        })
    }

    #[test]
    fn empty_measures_produce_empty_system() {
        let layout = layout_system(&test_prefix(), &[], &test_config(), None);
        assert!(layout.measures.is_empty());
        assert!((layout.total_width).abs() < f64::EPSILON);
    }

    #[test]
    fn single_measure_has_prefix_elements() {
        let cfg = test_config();
        let prefix = SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Sharps(2),
            Some(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
        );
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        }];
        let layout = layout_system(&prefix, &measures, &cfg, None);

        assert_eq!(layout.measures.len(), 1);
        assert!(layout.total_width > 0.0);

        // First measure should contain: clef, key sig, time sig, note, barline = 5 elements
        let elems = &layout.measures[0].layout.elements;
        assert_eq!(elems.len(), 5);
        assert!(matches!(elems[0].element, MeasureElement::Clef(_)));
        assert!(matches!(elems[1].element, MeasureElement::KeySignature(_)));
        assert!(matches!(elems[2].element, MeasureElement::TimeSignature(_)));
        assert!(matches!(elems[3].element, MeasureElement::Note(_)));
        assert!(matches!(elems[4].element, MeasureElement::Barline(_)));
    }

    #[test]
    fn second_measure_has_no_prefix() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Single,
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        assert_eq!(layout.measures.len(), 2);

        // Second measure: only note + barline = 2 elements
        let elems = &layout.measures[1].layout.elements;
        assert_eq!(elems.len(), 2);
        assert!(matches!(elems[0].element, MeasureElement::Note(_)));
        assert!(matches!(elems[1].element, MeasureElement::Barline(_)));
    }

    #[test]
    fn measures_are_contiguous() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(8)],
                barline: BarlineStyle::Final,
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        // Each measure starts where the previous ended
        for i in 1..layout.measures.len() {
            let prev_end =
                layout.measures[i - 1].x_offset + layout.measures[i - 1].layout.total_width;
            let curr_start = layout.measures[i].x_offset;
            assert!(
                (prev_end - curr_start).abs() < 1e-6,
                "measure {} should start at {}, got {}",
                i,
                prev_end,
                curr_start,
            );
        }
    }

    #[test]
    fn total_width_equals_sum_of_measures() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4), quarter_note(6)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(2)],
                barline: BarlineStyle::Final,
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        let sum: f64 = layout.measures.iter().map(|m| m.layout.total_width).sum();
        assert!(
            (layout.total_width - sum).abs() < 1e-6,
            "total {} != sum {}",
            layout.total_width,
            sum,
        );
    }

    #[test]
    fn target_width_scales_system() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Final,
            },
        ];

        let natural = layout_system(&test_prefix(), &measures, &cfg, None);
        let target = 10000.0;
        let scaled = layout_system(&test_prefix(), &measures, &cfg, Some(target));

        // Scaled total should match target
        assert!(
            (scaled.total_width - target).abs() < 1.0,
            "scaled width {} should be close to target {}",
            scaled.total_width,
            target,
        );

        // Scaled should differ from natural unless they happen to match
        assert!(
            (natural.total_width - target).abs() > 1.0,
            "natural width should differ from target for this test to be meaningful"
        );
    }

    #[test]
    fn first_measure_x_offset_is_zero() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);
        assert!((layout.measures[0].x_offset).abs() < f64::EPSILON);
    }

    #[test]
    fn open_key_signature_omitted_from_elements() {
        let cfg = test_config();
        let prefix = SystemPrefix::new(&Clef::Treble, KeySignature::Open, None);
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        }];
        let layout = layout_system(&prefix, &measures, &cfg, None);
        let elems = &layout.measures[0].layout.elements;

        // Should have: clef, note, barline = 3 (no key sig, no time sig)
        assert_eq!(elems.len(), 3);
        assert!(matches!(elems[0].element, MeasureElement::Clef(_)));
        assert!(matches!(elems[1].element, MeasureElement::Note(_)));
        assert!(matches!(elems[2].element, MeasureElement::Barline(_)));
    }

    #[test]
    fn clef_is_preserved_in_layout() {
        let cfg = test_config();
        let prefix = SystemPrefix::new(&Clef::Bass, KeySignature::Open, None);
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        }];
        let layout = layout_system(&prefix, &measures, &cfg, None);
        assert_eq!(layout.clef_kind, ClefKind::Bass);
    }

    #[test]
    fn multiple_events_in_measure() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![
                quarter_note(0),
                quarter_note(4),
                quarter_rest(),
                quarter_note(8),
            ],
            barline: BarlineStyle::Single,
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        // clef + time sig + 4 events + barline = 7
        let elems = &layout.measures[0].layout.elements;
        assert_eq!(elems.len(), 7);
    }

    #[test]
    fn three_measures_second_starts_after_first() {
        let cfg = test_config();
        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Final,
            },
        ];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);

        assert!(layout.measures[1].x_offset > 0.0);
        assert!(layout.measures[2].x_offset > layout.measures[1].x_offset);
    }

    #[test]
    fn staff_width_equals_target_when_specified() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, Some(8000.0));
        assert!((layout.staff_width - 8000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn staff_width_equals_total_when_no_target() {
        let cfg = test_config();
        let measures = vec![MeasureContent {
            events: vec![quarter_note(4)],
            barline: BarlineStyle::Single,
        }];
        let layout = layout_system(&test_prefix(), &measures, &cfg, None);
        assert!((layout.staff_width - layout.total_width).abs() < f64::EPSILON);
    }
}
