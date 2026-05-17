use crate::layout::arpeggio::ArpeggioDirection;
use crate::layout::articulation::Articulation;
use crate::layout::barline::BarlineStyle;
use crate::layout::breath::BreathMark;
use crate::layout::clef::ClefLayout;
use crate::layout::dynamics::Dynamic;
use crate::layout::glissando::GlissandoStyle;
use crate::layout::grace::GraceNoteKind;
use crate::layout::hairpin::HairpinType;
use crate::layout::lyric::LyricSyllable;
use crate::layout::key_signature::KeySignature;
use crate::layout::navigation::NavigationSign;
use crate::layout::ornament::Ornament;
use crate::layout::ottava::OttavaKind;
use crate::layout::pedal::PedalMark;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::stem::StemDirection;
use crate::layout::tempo::TempoMark;
use crate::layout::tremolo::TremoloCount;
use crate::layout::trill_bracket::{HookDirection, TrillBracketSide};
use crate::layout::trill_extension::{TrillSpeedRampSpec, TrillWiggleSpeed};
use crate::layout::time_signature::TimeSignatureKind;

/// Articulation and expression annotations attached to a note or chord event.
///
/// These fields are shared between [`NoteEvent`] and [`ChordEvent`], covering
/// ties, slurs, hairpins, dynamics, rehearsal marks, tempo marks, and expression text.
/// All fields default to "no annotation" (`false` / `None`).
#[derive(Clone, Debug, Default)]
pub struct NoteAnnotations {
    /// Whether this note/chord is tied forward to the next note at the same
    /// staff position. The tie curve is drawn by the system renderer after
    /// all measures are laid out.
    pub tie_forward: bool,
    /// Optional dynamic marking (e.g. pp, mf, ff) displayed below the staff,
    /// centered on this note/chord.
    pub dynamic: Option<Dynamic>,
    /// Whether this note/chord is the start of a slur (curved line to a following note).
    /// The slur curve is drawn by the system renderer after all measures are laid out.
    pub slur_start: bool,
    /// Whether this note/chord is the end of a slur.
    pub slur_end: bool,
    /// Whether this note/chord is the start of a hairpin (crescendo/decrescendo wedge).
    /// The hairpin is drawn by the system renderer after all measures are laid out.
    pub hairpin_start: Option<HairpinType>,
    /// Whether this note/chord is the end of a hairpin wedge.
    pub hairpin_end: bool,
    /// Optional rehearsal mark displayed above the staff at this note/chord's position.
    /// Tuple of (text content, enclosure style).
    pub rehearsal_mark: Option<(String, RehearsalStyle)>,
    /// Optional tempo marking displayed above the staff at this note/chord's position.
    pub tempo_mark: Option<TempoMark>,
    /// Optional expression text displayed below the staff in italic (e.g. "dolce").
    pub expression: Option<String>,
    /// Articulation markings (staccato, tenuto, accent, marcato, etc.)
    /// placed near the notehead. Multiple articulations stack outward from
    /// the note (e.g., staccato + accent = portato accent).
    pub articulations: Vec<Articulation>,
    /// Optional grace note preceding the principal note.
    /// Tuple of (grace note staff position, grace note kind).
    pub grace_note: Option<(i8, GraceNoteKind)>,
    /// Whether to draw a connecting slur from the grace note to the principal note.
    /// Has no effect when `grace_note` is `None`. The slur arcs away from the
    /// principal note's stem in the conventional direction. This is the canonical
    /// engraving for acciaccatura and is also common for appoggiatura.
    pub grace_note_slur: bool,
    /// Optional lyric syllable displayed below the staff under this note/chord.
    pub lyric: Option<LyricSyllable>,
    /// Optional chord symbol displayed above the staff (e.g. "Cmaj7", "Am").
    pub chord_symbol: Option<String>,
    /// Optional ornament marking (trill, mordent, turn, etc.) placed above the staff.
    pub ornament: Option<Ornament>,
    /// Optional navigation sign (segno, coda) placed above the staff.
    pub navigation_sign: Option<NavigationSign>,
    /// Whether this note/chord starts an ottava bracket (8va, 8vb, etc.).
    /// The bracket extends from this note to the note with `ottava_end = true`.
    pub ottava_start: Option<OttavaKind>,
    /// Whether this note/chord ends an ottava bracket.
    pub ottava_end: bool,
    /// Optional pedal marking (Ped. down or * up) placed below the staff.
    pub pedal: Option<PedalMark>,
    /// Optional tremolo slashes (1–3) drawn on the stem of this note/chord.
    pub tremolo: Option<TremoloCount>,
    /// Optional arpeggio (rolled chord) wavy line drawn to the left of the
    /// chord noteheads. Primarily used on chords but valid on single notes.
    pub arpeggio: Option<ArpeggioDirection>,
    /// Optional breath mark (comma, tick, or caesura) placed above the staff
    /// to the right of this note/chord, indicating a brief pause or lift.
    pub breath_mark: Option<BreathMark>,
    /// Whether this note/chord starts a glissando line to the next note.
    /// The diagonal line is drawn by the system renderer after all measures
    /// are laid out.
    pub glissando_start: Option<GlissandoStyle>,
    /// Whether this trill ornament has a wavy-line extension that continues
    /// to the next note. Has no effect unless `ornament` is `Some(Ornament::Trill)`;
    /// the renderer skips the extension cleanly when the ornament isn't a trill.
    /// The wavy line is drawn by the system renderer after all measures are
    /// laid out, tiling the SMuFL `wiggleTrill` segment between the trill
    /// glyph and the next note.
    pub trill_extension: bool,
    /// Optional bracket form for the trill extension: a vertical hook capping
    /// the start, end, or both ends of the wavy line. Has no effect unless
    /// both `ornament == Some(Ornament::Trill)` and `trill_extension == true`
    /// — a bracket without a wiggle to bracket is silently dropped. For
    /// cross-system trills, a `Both` bracket places the start hook on the
    /// source system (with the outgoing wiggle) and the end hook on the
    /// target system (with the incoming wiggle), so the bracket frames the
    /// trill's true semantic range rather than the per-system wiggle
    /// fragments.
    pub trill_bracket: Option<TrillBracketSide>,
    /// Optional override for the direction in which trill bracket hooks
    /// extend from the wiggle baseline. Has no effect unless `trill_bracket`
    /// is `Some`. `None` selects the conventional `Down` direction (hook
    /// points back toward the staff for trills sitting above the staff).
    /// Set to `Some(HookDirection::Up)` for the rare case of a trill rendered
    /// below the staff where the hook should still point back toward the
    /// affected notes.
    pub trill_bracket_direction: Option<HookDirection>,
    /// Optional override for trill bracket hook length, in staff spaces.
    /// Has no effect unless `trill_bracket` is `Some`. `None` selects the
    /// default of ~0.75 staff spaces. Behind Bars shows hooks ranging from
    /// roughly 0.5 to 1.0 staff spaces depending on the surrounding density;
    /// callers can opt into a thinner or chunkier hook here without changing
    /// the engraving config globally.
    pub trill_bracket_length_ss: Option<f64>,
    /// Optional speed/density variant for the trill wavy-line extension.
    /// Has no effect unless `trill_extension == true`. `None` selects the
    /// neutral `Standard` wiggle (Bravura's `wiggleTrill` glyph). Choosing
    /// a faster or slower variant communicates trill speed visually while
    /// keeping the gesture's meaning unchanged.
    pub trill_wiggle_speed: Option<TrillWiggleSpeed>,
    /// Optional explicit termination length for the trill wavy-line
    /// extension, in staff spaces. Has no effect unless `trill_extension ==
    /// true`. `None` (the default) lets the wiggle extend to the next note
    /// (within-system) or to the system's right edge (cross-system) per the
    /// usual convention. `Some(length_ss)` clamps the wiggle so it terminates
    /// no later than `length_ss` staff spaces past its natural start —
    /// useful when a trill should visually "run out" before the next note
    /// (e.g. a trill on a half note where the trill is intended to release
    /// partway through the held duration).
    ///
    /// Clamping is one-sided: if the requested length is larger than the
    /// natural span, the natural span wins (no overrun past the next note
    /// or the system edge). When set to a positive value, the wiggle never
    /// propagates across a system break — even if the natural span would
    /// have extended to the system edge — because the explicit length
    /// already specifies a definite endpoint.
    ///
    /// Non-positive values produce no wiggle (the renderer's same
    /// fail-safe as for spans too short to fit one tile).
    pub trill_extension_length_ss: Option<f64>,
    /// Optional explicit end-anchor for the trill wavy-line extension,
    /// expressed as a positive note offset (in the system's flat note
    /// sequence) from the trilled note. Has no effect unless
    /// `trill_extension == true`. `None` (the default) terminates the
    /// wiggle at the *immediately following* note (offset = 1 implicitly)
    /// or at the system's right edge if this is the last note. `Some(n)`
    /// with `n >= 1` terminates the wiggle at the note `n` positions after
    /// the trilled note — useful when the trill should visibly hold across
    /// one or more intervening notes before releasing into a specific
    /// successor.
    ///
    /// Offsets that walk past the end of the system fall back to the
    /// "extend to system right edge" behavior (same as a trilled last
    /// note), and the trill propagates across the system break only when
    /// `n` walks past the *last* note (i.e. the natural last-note case);
    /// any other walk-past-end terminates at the system edge without
    /// cross-system continuation.
    ///
    /// `Some(0)` is rejected at the renderer's fail-safe as no wiggle (a
    /// zero-offset target is the trilled note itself, so start_x ==
    /// end_x — same suppression as a non-positive `trill_extension_length_ss`).
    ///
    /// Independent of [`trill_extension_length_ss`](Self::trill_extension_length_ss):
    /// when both are set, the explicit length wins (the length field is
    /// the more specific termination). This matches the documented
    /// "explicit length specifies a definite endpoint" semantic of the
    /// length field; the to-note offset is a softer "stretch to note N"
    /// hint that yields to the explicit length when both are present.
    pub trill_extension_to_note_offset: Option<usize>,
    /// Optional multi-speed ramp spec for the trill wavy-line extension.
    /// Has no effect unless `trill_extension == true`. `None` (the default)
    /// renders a single-speed wiggle using `trill_wiggle_speed` (or the
    /// `Standard` default when that is also `None`). `Some(spec)` engages
    /// the multi-speed renderer path: the system renderer evenly partitions
    /// the wiggle's span into `spec.region_count` regions and tiles each
    /// region with the speed produced by `spec.ramp.synthesize_regions`.
    ///
    /// When a ramp is present, it *supersedes* `trill_wiggle_speed` for
    /// glyph selection — the speed field's value is ignored by the
    /// renderer's wiggle path (the bracket/length fields continue to apply
    /// as normal). Both fields are permitted to coexist on the annotation
    /// so widening a single-speed annotation by layering a ramp does not
    /// force the caller to first clear the speed field.
    ///
    /// A degenerate spec (e.g. `region_count == 0`, or `Linear` with
    /// `region_count == 1`) makes `synthesize_regions` return `None` at
    /// draw time, and the renderer falls back to no wiggle — same
    /// fail-safe as for spans too short to tile.
    pub trill_speed_ramp: Option<TrillSpeedRampSpec>,
}

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
    /// Articulation/expression annotations (ties, dynamics, slurs, hairpins, etc.).
    pub annotations: NoteAnnotations,
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
    /// Multi-measure rest: H-bar (default) or church-rest cluster spanning
    /// the measure width with a count number. `count` is the number of
    /// measures of rest; `style` controls the visual depiction.
    MultiMeasureRest {
        /// Number of consecutive measures of rest.
        count: u32,
        /// Visual style (H-bar or church-rest).
        style: crate::layout::multi_measure_rest::MultiMeasureRestStyle,
    },
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
    /// Articulation/expression annotations (ties, dynamics, slurs, hairpins, etc.).
    pub annotations: NoteAnnotations,
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
            MeasureElement::MultiMeasureRest { .. } => {
                // Multi-measure rest occupies the full rhythmic width of the measure.
                // Use whole-note spacing as the base allocation; the renderer draws
                // the H-bar (or church-rest cluster) spanning from the preceding
                // element to the barline.
                let factor =
                    duration_spacing_factor(0, shortest_log2, config.spacing_ratio);
                config.min_note_spacing * factor
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
        annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 2, // quarter note
                dots: 0,
                accidental: None,
                stem_direction: None,
            annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 3, // eighth note
                dots: 0,
                accidental: None,
                stem_direction: None,
            annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
            annotations: NoteAnnotations::default(),
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 8,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
            annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
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
            annotations: NoteAnnotations::default(),
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
