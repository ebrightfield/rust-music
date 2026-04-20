use crate::layout::barline::BarlineStyle;
use crate::layout::clef::ClefLayout;
use crate::layout::dynamics::Dynamic;
use crate::layout::hairpin::HairpinType;
use crate::layout::key_signature::KeySignature;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::stem::StemDirection;
use crate::layout::tempo::TempoMark;
use crate::layout::time_signature::TimeSignatureKind;

/// A group of notes to be beamed together.
///
/// All notes in a beam group share a common beam line; individual notes
/// must have `duration_log2 >= 3` (eighth notes or shorter).
#[derive(Clone, Debug)]
pub struct BeamGroupEvent {
    /// The notes in the beam group, in temporal order.
    pub notes: Vec<NoteEvent>,
    /// Stem direction override. `None` uses auto-detection based on the
    /// group's collective staff positions.
    pub stem_direction: Option<StemDirection>,
}

/// A tuplet group: a beam group (or sequence of notes) with a tuplet bracket and number.
///
/// The underlying notes are beamed together; the tuplet bracket and number
/// are drawn above or below the group. Placement defaults to above for
/// stems-up, below for stems-down.
#[derive(Clone, Debug)]
pub struct TupletGroupEvent {
    /// The underlying beam group (notes are beamed and drawn normally).
    pub beam_group: BeamGroupEvent,
    /// The tuplet number to display (e.g. 3 for triplet, 5 for quintuplet).
    pub tuplet_number: u32,
}

/// A chord (multiple simultaneous notes) to be laid out within a measure.
#[derive(Clone, Debug)]
pub struct ChordEvent {
    /// Staff positions of notes in the chord (bottom line = 0), in any order.
    /// Will be sorted during layout.
    pub staff_positions: Vec<i8>,
    /// Log2 of the duration denominator: 0=whole, 1=half, 2=quarter, 3=eighth, etc.
    /// All notes in a chord share the same duration.
    pub duration_log2: u8,
    /// Number of augmentation dots (0–3).
    pub dots: u8,
    /// Accidental glyphs to display, parallel to `staff_positions`.
    /// `None` entries mean no accidental for that note.
    pub accidentals: Vec<Option<smufl::Glyph>>,
    /// Stem direction override. `None` uses auto-detection based on chord extent.
    pub stem_direction: Option<StemDirection>,
    /// Whether all notes in this chord are tied forward to the next chord or
    /// notes at the same staff positions. Tie curves are drawn by the system
    /// renderer after all measures are laid out.
    pub tie_forward: bool,
    /// Optional dynamic marking (e.g. pp, mf, ff) displayed below the staff,
    /// centered on this chord.
    pub dynamic: Option<Dynamic>,
    /// Whether this chord is the start of a slur.
    pub slur_start: bool,
    /// Whether this chord is the end of a slur.
    pub slur_end: bool,
    /// Whether this chord is the start of a hairpin (crescendo/decrescendo wedge).
    pub hairpin_start: Option<HairpinType>,
    /// Whether this chord is the end of a hairpin wedge.
    pub hairpin_end: bool,
    /// Optional rehearsal mark displayed above the staff at this chord's position.
    /// Tuple of (text content, enclosure style).
    pub rehearsal_mark: Option<(String, RehearsalStyle)>,
    /// Optional tempo marking displayed above the staff at this chord's position.
    pub tempo_mark: Option<TempoMark>,
    /// Optional expression text displayed below the staff in italic (e.g. "dolce").
    pub expression: Option<String>,
}

/// A musical event within a measure that occupies horizontal space.
#[derive(Clone, Debug)]
pub enum MeasureElement {
    /// Clef (typically only at the start of the first measure or on clef change).
    /// Uses `ClefLayout` which wraps `music::Clef` info without requiring Clone on Clef.
    Clef(ClefLayout),
    /// Key signature.
    KeySignature(KeySignature),
    /// Time signature.
    TimeSignature(TimeSignatureKind),
    /// A note event: staff position, notehead kind, stem direction, flag count, dot count,
    /// optional accidental glyph (already resolved to SMuFL glyph).
    Note(NoteEvent),
    /// A rest event: log2 duration (0=whole, 1=half, 2=quarter, etc.), dot count.
    Rest(RestEvent),
    /// A chord (multiple simultaneous notes).
    Chord(ChordEvent),
    /// A beam group: multiple notes connected by beam lines instead of flags.
    BeamGroup(BeamGroupEvent),
    /// A tuplet group: a beam group with a tuplet bracket and number overlay.
    TupletGroup(TupletGroupEvent),
    /// Barline at the end of the measure.
    Barline(BarlineStyle),
}

/// A single note to be laid out within a measure.
#[derive(Clone, Debug)]
pub struct NoteEvent {
    /// Staff position (bottom line = 0).
    pub staff_position: i8,
    /// Log2 of the duration denominator: 0=whole, 1=half, 2=quarter, 3=eighth, etc.
    pub duration_log2: u8,
    /// Number of augmentation dots (0–3).
    pub dots: u8,
    /// Accidental to display (if any). Uses `smufl::Glyph` for the accidental glyph.
    pub accidental: Option<smufl::Glyph>,
    /// Stem direction override. `None` uses auto-detection.
    pub stem_direction: Option<StemDirection>,
    /// Whether this note is tied forward to the next note at the same staff position.
    /// The tie curve is drawn by the system renderer after all measures are laid out.
    pub tie_forward: bool,
    /// Optional dynamic marking (e.g. pp, mf, ff) displayed below the staff,
    /// centered on this note.
    pub dynamic: Option<Dynamic>,
    /// Whether this note is the start of a slur (curved line to a following note).
    /// The slur curve is drawn by the system renderer after all measures are laid out.
    pub slur_start: bool,
    /// Whether this note is the end of a slur.
    pub slur_end: bool,
    /// Whether this note is the start of a hairpin (crescendo/decrescendo wedge).
    /// The hairpin is drawn by the system renderer after all measures are laid out.
    pub hairpin_start: Option<HairpinType>,
    /// Whether this note is the end of a hairpin wedge.
    pub hairpin_end: bool,
    /// Optional rehearsal mark displayed above the staff at this note's position.
    /// Tuple of (text content, enclosure style).
    pub rehearsal_mark: Option<(String, RehearsalStyle)>,
    /// Optional tempo marking displayed above the staff at this note's position.
    pub tempo_mark: Option<TempoMark>,
    /// Optional expression text displayed below the staff in italic (e.g. "dolce").
    pub expression: Option<String>,
}

/// A rest to be laid out within a measure.
#[derive(Clone, Debug)]
pub struct RestEvent {
    /// Log2 of the duration denominator: 0=whole, 1=half, 2=quarter, etc.
    pub duration_log2: u8,
    /// Number of augmentation dots (0–3).
    pub dots: u8,
}

/// A positioned element within a laid-out measure.
#[derive(Clone, Debug)]
pub struct PositionedElement {
    /// X-coordinate (in font design units) from the start of the measure.
    pub x: f64,
    /// The element.
    pub element: MeasureElement,
    /// Advance width of this element (in font design units).
    pub width: f64,
}

/// The result of laying out a measure: elements with assigned x-positions.
#[derive(Clone, Debug)]
pub struct MeasureLayout {
    /// Positioned elements in left-to-right order.
    pub elements: Vec<PositionedElement>,
    /// Total width of the measure in font design units.
    pub total_width: f64,
}

/// Configuration for measure layout.
#[derive(Clone, Debug)]
pub struct MeasureLayoutConfig {
    /// Width to allocate for a clef, in font design units.
    pub clef_width: f64,
    /// Padding after clef before next element.
    pub clef_padding: f64,
    /// Width allocated per key signature accidental.
    pub key_sig_accidental_width: f64,
    /// Padding after key signature.
    pub key_sig_padding: f64,
    /// Width for a time signature.
    pub time_sig_width: f64,
    /// Padding after time signature before first note.
    pub time_sig_padding: f64,
    /// Width for a barline.
    pub barline_width: f64,
    /// Minimum note spacing (in font design units) for the shortest note in the measure.
    pub min_note_spacing: f64,
    /// Spacing ratio base: longer notes get proportionally more space.
    /// A quarter note gets `min_note_spacing * spacing_ratio`, a half note gets
    /// `min_note_spacing * spacing_ratio^2`, etc.
    pub spacing_ratio: f64,
}

impl MeasureLayoutConfig {
    /// Default config using a staff space value (typically from EngravingConfig).
    pub fn from_staff_space(ss: f64) -> Self {
        Self {
            clef_width: 2.5 * ss,
            clef_padding: 0.5 * ss,
            key_sig_accidental_width: 1.0 * ss,
            key_sig_padding: 0.75 * ss,
            time_sig_width: 2.0 * ss,
            time_sig_padding: 0.75 * ss,
            barline_width: 0.5 * ss,
            min_note_spacing: 1.5 * ss,
            spacing_ratio: 1.6,
        }
    }
}

/// Compute the horizontal spacing factor for a given duration.
///
/// Uses a power-of-ratio model (similar to Gourlay's approach): the shortest
/// duration gets factor 1.0, each doubling of duration multiplies by `ratio`.
/// `duration_log2`: 0=whole, 1=half, 2=quarter, 3=eighth, etc.
/// `shortest_log2`: the largest log2 value (shortest note) in the measure.
fn duration_spacing_factor(duration_log2: u8, shortest_log2: u8, ratio: f64) -> f64 {
    let steps = shortest_log2 as f64 - duration_log2 as f64;
    ratio.powf(steps)
}

/// Lay out a sequence of measure elements with horizontal positions.
///
/// Non-rhythmic elements (clef, key sig, time sig, barline) get fixed widths.
/// Rhythmic elements (notes, rests) get proportional spacing based on duration.
pub fn layout_measure(elements: &[MeasureElement], config: &MeasureLayoutConfig) -> MeasureLayout {
    // First pass: separate fixed-width prefix/suffix from rhythmic content
    let mut positioned = Vec::with_capacity(elements.len());
    let mut x = 0.0;

    // Find the shortest duration for proportional spacing
    let shortest_log2 = elements
        .iter()
        .filter_map(|e| match e {
            MeasureElement::Note(n) => Some(n.duration_log2),
            MeasureElement::Rest(r) => Some(r.duration_log2),
            MeasureElement::Chord(c) => Some(c.duration_log2),
            MeasureElement::BeamGroup(bg) => bg.notes.iter().map(|n| n.duration_log2).max(),
            MeasureElement::TupletGroup(tg) => tg.beam_group.notes.iter().map(|n| n.duration_log2).max(),
            _ => None,
        })
        .max()
        .unwrap_or(2); // default to quarter note if no rhythmic content

    for elem in elements {
        let width = match elem {
            MeasureElement::Clef(_) => {
                let w = config.clef_width;
                positioned.push(PositionedElement {
                    x,
                    element: elem.clone(),
                    width: w,
                });
                x += w + config.clef_padding;
                continue;
            }
            MeasureElement::KeySignature(key) => {
                let count = match key {
                    KeySignature::Sharps(n) | KeySignature::Flats(n) => *n as f64,
                    KeySignature::Open => 0.0,
                };
                let w = count * config.key_sig_accidental_width;
                positioned.push(PositionedElement {
                    x,
                    element: elem.clone(),
                    width: w,
                });
                if w > 0.0 {
                    x += w + config.key_sig_padding;
                }
                continue;
            }
            MeasureElement::TimeSignature(_) => {
                let w = config.time_sig_width;
                positioned.push(PositionedElement {
                    x,
                    element: elem.clone(),
                    width: w,
                });
                x += w + config.time_sig_padding;
                continue;
            }
            MeasureElement::Note(n) => {
                let factor =
                    duration_spacing_factor(n.duration_log2, shortest_log2, config.spacing_ratio);
                config.min_note_spacing * factor
            }
            MeasureElement::Rest(r) => {
                let factor =
                    duration_spacing_factor(r.duration_log2, shortest_log2, config.spacing_ratio);
                config.min_note_spacing * factor
            }
            MeasureElement::Chord(c) => {
                let factor =
                    duration_spacing_factor(c.duration_log2, shortest_log2, config.spacing_ratio);
                config.min_note_spacing * factor
            }
            MeasureElement::BeamGroup(bg) => {
                // Total width is the sum of each note's proportional spacing
                let total: f64 = bg
                    .notes
                    .iter()
                    .map(|n| {
                        let factor = duration_spacing_factor(
                            n.duration_log2,
                            shortest_log2,
                            config.spacing_ratio,
                        );
                        config.min_note_spacing * factor
                    })
                    .sum();
                total
            }
            MeasureElement::TupletGroup(tg) => {
                let total: f64 = tg.beam_group
                    .notes
                    .iter()
                    .map(|n| {
                        let factor = duration_spacing_factor(
                            n.duration_log2,
                            shortest_log2,
                            config.spacing_ratio,
                        );
                        config.min_note_spacing * factor
                    })
                    .sum();
                total
            }
            MeasureElement::Barline(_) => config.barline_width,
        };

        positioned.push(PositionedElement {
            x,
            element: elem.clone(),
            width,
        });
        x += width;
    }

    MeasureLayout {
        total_width: x,
        elements: positioned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::notation::clef::Clef;

    fn test_config() -> MeasureLayoutConfig {
        MeasureLayoutConfig::from_staff_space(250.0)
    }

    #[test]
    fn empty_measure() {
        let layout = layout_measure(&[], &test_config());
        assert!(layout.elements.is_empty());
        assert!((layout.total_width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn single_quarter_note() {
        let cfg = test_config();
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 0,
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
        })];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 1);
        assert!((layout.elements[0].x - 0.0).abs() < f64::EPSILON);
        // Single note: shortest is quarter (log2=2), factor=1.0, width = min_note_spacing
        let expected_width = cfg.min_note_spacing;
        assert!(
            (layout.elements[0].width - expected_width).abs() < f64::EPSILON,
            "quarter note width should be min_note_spacing"
        );
    }

    #[test]
    fn clef_then_note() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
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
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 2);
        // Note should start after clef_width + clef_padding
        let expected_x = cfg.clef_width + cfg.clef_padding;
        assert!(
            (layout.elements[1].x - expected_x).abs() < f64::EPSILON,
            "note should start after clef, got {} expected {}",
            layout.elements[1].x,
            expected_x,
        );
    }

    #[test]
    fn full_prefix_ordering() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::KeySignature(KeySignature::Sharps(3)),
            MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
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
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 5);
        // Each element's x should be strictly greater than or equal to the previous
        for i in 1..layout.elements.len() {
            assert!(
                layout.elements[i].x >= layout.elements[i - 1].x,
                "element {} x={} should be >= element {} x={}",
                i,
                layout.elements[i].x,
                i - 1,
                layout.elements[i - 1].x,
            );
        }
        // Total width should be positive
        assert!(layout.total_width > 0.0);
    }

    #[test]
    fn proportional_spacing_half_vs_quarter() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 1, // half note
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
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 2, // quarter note
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
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        // Half note (log2=1) should get more space than quarter (log2=2)
        assert!(
            layout.elements[0].width > layout.elements[1].width,
            "half note (w={}) should be wider than quarter (w={})",
            layout.elements[0].width,
            layout.elements[1].width,
        );
    }

    #[test]
    fn proportional_spacing_whole_vs_eighth() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 0, // whole note
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
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 3, // eighth note
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
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        // Whole note should get significantly more space than eighth
        let ratio = layout.elements[0].width / layout.elements[1].width;
        assert!(
            ratio > 2.0,
            "whole note should get >2x the space of an eighth, got ratio {}",
            ratio,
        );
    }

    #[test]
    fn equal_durations_get_equal_spacing() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 0,
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
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
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
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 8,
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
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert!((layout.elements[0].width - layout.elements[1].width).abs() < f64::EPSILON);
        assert!((layout.elements[1].width - layout.elements[2].width).abs() < f64::EPSILON);
        // Second note starts at first note's x + width
        assert!(
            (layout.elements[1].x - layout.elements[0].width).abs() < f64::EPSILON,
        );
    }

    #[test]
    fn key_sig_open_adds_no_space() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::KeySignature(KeySignature::Open),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
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
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        // Open key signature should not push the note forward
        assert!(
            (layout.elements[1].x - 0.0).abs() < f64::EPSILON,
            "note should start at x=0 after open key sig, got {}",
            layout.elements[1].x,
        );
    }

    #[test]
    fn barline_at_end() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
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
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert_eq!(layout.elements.len(), 2);
        // Barline starts after note's width
        assert!(layout.elements[1].x > 0.0);
        assert!((layout.elements[1].width - cfg.barline_width).abs() < f64::EPSILON);
    }

    #[test]
    fn total_width_is_sum_of_all() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
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
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        // Last element's x + width should equal total_width
        let last = layout.elements.last().unwrap();
        assert!(
            (last.x + last.width - layout.total_width).abs() < f64::EPSILON,
            "total_width {} should equal last element end {}",
            layout.total_width,
            last.x + last.width,
        );
    }

    #[test]
    fn rest_uses_proportional_spacing() {
        let cfg = test_config();
        let elements = vec![
            MeasureElement::Rest(RestEvent {
                duration_log2: 1, // half rest
                dots: 0,
            }),
            MeasureElement::Rest(RestEvent {
                duration_log2: 2, // quarter rest
                dots: 0,
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        assert!(
            layout.elements[0].width > layout.elements[1].width,
            "half rest should be wider than quarter rest"
        );
    }

    #[test]
    fn duration_spacing_factor_same_duration() {
        let factor = duration_spacing_factor(3, 3, 1.6);
        assert!((factor - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn duration_spacing_factor_one_step_longer() {
        let factor = duration_spacing_factor(2, 3, 1.6);
        assert!((factor - 1.6).abs() < f64::EPSILON);
    }

    #[test]
    fn duration_spacing_factor_two_steps_longer() {
        let factor = duration_spacing_factor(1, 3, 1.6);
        let expected = 1.6 * 1.6;
        assert!((factor - expected).abs() < 1e-10);
    }

    #[test]
    fn multiple_key_sig_accidentals_widen() {
        let cfg = test_config();
        let layout3 = layout_measure(
            &[
                MeasureElement::KeySignature(KeySignature::Sharps(3)),
                MeasureElement::Barline(BarlineStyle::Single),
            ],
            &cfg,
        );
        let layout5 = layout_measure(
            &[
                MeasureElement::KeySignature(KeySignature::Sharps(5)),
                MeasureElement::Barline(BarlineStyle::Single),
            ],
            &cfg,
        );
        assert!(
            layout5.total_width > layout3.total_width,
            "5 sharps measure should be wider than 3 sharps"
        );
    }

    #[test]
    fn config_from_staff_space_scales() {
        let cfg1 = MeasureLayoutConfig::from_staff_space(100.0);
        let cfg2 = MeasureLayoutConfig::from_staff_space(200.0);
        assert!((cfg2.clef_width - 2.0 * cfg1.clef_width).abs() < f64::EPSILON);
        assert!((cfg2.min_note_spacing - 2.0 * cfg1.min_note_spacing).abs() < f64::EPSILON);
    }
}
