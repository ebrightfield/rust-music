//! Beam and tuplet spans.
//!
//! Grouping is orthogonal to event kind, as in LilyPond: a beam or tuplet is a
//! *span* opened and closed around ordinary notes, chords, and rests. Spans are
//! carried through the pipeline as zero-duration [`GroupMark`] events in score
//! order (`ScoreEvent::GroupMark` → `MeasureEvent::GroupMark` →
//! [`MeasureElement::GroupMark`]), so every member keeps its own identity,
//! duration, accidentals, annotations, and laid-out x position.
//!
//! Beam and tuplet spans are independent of each other: a tuplet need not be
//! beamed, a beam may cover part of a tuplet, and either may contain rests.
//! Tuplets nest; beams do not. A span may cross a barline: score conversion
//! closes it at the end of each measure with a `continues` end mark and
//! reopens it at the next measure's start with a `continued` start mark, so
//! every measure's elements are self-contained while scanners over several
//! measures (a system) see one joined span.

use crate::layout::measure::MeasureElement;
use crate::layout::stem::StemDirection;
use crate::layout::tuplet::TupletPlacement;

/// Options for one beam span.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BeamSpec {
    /// Stem direction forced on every member (`\stemUp` / `\stemDown` around
    /// the beam). `None` derives one direction for the whole beam from its
    /// members (see the score builder for the precedence rules).
    pub stem_direction: Option<StemDirection>,
    /// LilyPond `subdivideBeams`: break secondary beams at every multiple of
    /// this written duration (as `duration_log2`: 3 = eighth, 4 = sixteenth),
    /// measured from the start of the measure. Only the beams the subdivision
    /// value itself carries (one for an eighth, two for a sixteenth) stay
    /// connected across a subdivision boundary. `None` never subdivides.
    pub subdivide_log2: Option<i8>,
}

impl BeamSpec {
    /// A beam with automatic stem direction and no subdivision.
    pub fn new() -> Self {
        Self::default()
    }

    /// Force every member's stem to `direction`.
    pub fn stem_direction(mut self, direction: StemDirection) -> Self {
        self.stem_direction = Some(direction);
        self
    }

    /// Subdivide secondary beams at multiples of the written duration
    /// `interval_log2` (3 = eighth, 4 = sixteenth, …).
    pub fn subdivide(mut self, interval_log2: i8) -> Self {
        self.subdivide_log2 = Some(interval_log2);
        self
    }
}

/// What a tuplet prints as its number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TupletNumberDisplay {
    /// The actual-note count alone ("3").
    #[default]
    Number,
    /// The full ratio ("3:2").
    Ratio,
    /// No number.
    Hidden,
}

/// When a tuplet draws its bracket.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TupletBracketVisibility {
    /// LilyPond's `if-no-beam`: draw the bracket unless the tuplet's first and
    /// last stemmed members are joined by one beam.
    #[default]
    Auto,
    /// Always draw the bracket.
    Always,
    /// Never draw the bracket (the number, if any, is still printed).
    Never,
}

/// A tuplet span's ratio and appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TupletSpec {
    /// Actual notes: members play `number` notes in the time of `in_time_of`.
    pub number: u32,
    /// Normal notes the tuplet's written durations are fitted into.
    pub in_time_of: u32,
    /// What the tuplet prints as its number.
    pub number_display: TupletNumberDisplay,
    /// When the bracket is drawn.
    pub bracket: TupletBracketVisibility,
    /// Forced placement (`\tupletUp` / `\tupletDown`); `None` places the
    /// tuplet on the stem side of its members.
    pub placement: Option<TupletPlacement>,
}

impl TupletSpec {
    /// A `number : in_time_of` tuplet with default appearance (number shown,
    /// automatic bracket and placement). `TupletSpec::new(3, 2)` is a triplet.
    pub fn new(number: u32, in_time_of: u32) -> Self {
        Self {
            number,
            in_time_of,
            number_display: TupletNumberDisplay::default(),
            bracket: TupletBracketVisibility::default(),
            placement: None,
        }
    }

    /// Set what the tuplet prints as its number.
    pub fn number_display(mut self, display: TupletNumberDisplay) -> Self {
        self.number_display = display;
        self
    }

    /// Set when the bracket is drawn.
    pub fn bracket(mut self, visibility: TupletBracketVisibility) -> Self {
        self.bracket = visibility;
        self
    }

    /// Force the tuplet above or below its members.
    pub fn placement(mut self, placement: TupletPlacement) -> Self {
        self.placement = Some(placement);
        self
    }

    /// Factor applied to members' written durations: `in_time_of / number`.
    pub fn time_scale(&self) -> f64 {
        f64::from(self.in_time_of) / f64::from(self.number)
    }
}

/// A zero-duration span boundary carried in score order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupMark {
    /// Opens a beam span; following members up to the matching
    /// [`GroupMark::BeamEnd`] are beamed together.
    BeamStart {
        /// Beam options.
        spec: BeamSpec,
        /// The span was opened in an earlier measure; this mark resumes it.
        continued: bool,
    },
    /// Closes the open beam span.
    BeamEnd {
        /// The span goes on in the next measure.
        continues: bool,
    },
    /// Opens a tuplet span (nested inside any tuplet already open).
    TupletStart {
        /// Tuplet ratio and appearance.
        spec: TupletSpec,
        /// The span was opened in an earlier measure; this mark resumes it.
        continued: bool,
    },
    /// Closes the innermost open tuplet span.
    TupletEnd {
        /// The span goes on in the next measure.
        continues: bool,
    },
}

/// One beam or tuplet span as seen inside a scanned element sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupSegment<S> {
    /// The span's options.
    pub spec: S,
    /// Indices (into the scanned sequence) of the span's rhythmic members —
    /// notes, chords, and rests — in order.
    pub members: Vec<usize>,
    /// The span began before the scanned sequence.
    pub open_start: bool,
    /// The span continues past the scanned sequence.
    pub open_end: bool,
    /// Tuplet nesting depth (0 = outermost). Always 0 for beams.
    pub depth: usize,
}

/// Beam and tuplet spans found in a sequence of measure elements.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroupScan {
    /// Beam spans in order of their start.
    pub beams: Vec<GroupSegment<BeamSpec>>,
    /// Tuplet spans in order of their start.
    pub tuplets: Vec<GroupSegment<TupletSpec>>,
    /// Per scanned element: the index into [`Self::beams`] of the beam it
    /// belongs to (`None` outside beams and for non-rhythmic elements).
    pub beam_of: Vec<Option<usize>>,
    /// Per scanned element: performed onset in whole notes from the start of
    /// its measure (tuplets scaled; non-rhythmic elements get the onset of
    /// the next rhythmic one).
    pub onsets: Vec<f64>,
}

/// Performed length of a written duration in whole notes, before tuplet
/// scaling (`duration_log2` -1 = breve, 0 = whole, 2 = quarter, …).
pub fn written_whole_notes(duration_log2: i8, dots: u8) -> f64 {
    let base = 2.0_f64.powi(-i32::from(duration_log2));
    base * (2.0 - 0.5_f64.powi(i32::from(dots)))
}

/// Written duration and dots of a rhythmic element that can be a span member.
pub(crate) fn member_duration(element: &MeasureElement) -> Option<(i8, u8)> {
    match element {
        MeasureElement::Note(note) => Some((note.duration_log2, note.dots)),
        MeasureElement::Chord(chord) => Some((chord.duration_log2, chord.dots)),
        MeasureElement::Rest(rest) => Some((rest.duration_log2, rest.dots)),
        _ => None,
    }
}

/// Product of the open tuplets' time scales.
pub(crate) fn tuplet_time_scale<'a>(open: impl IntoIterator<Item = &'a TupletSpec>) -> f64 {
    open.into_iter().map(TupletSpec::time_scale).product()
}

/// Scan `elements` for beam and tuplet spans.
///
/// A `continues` end mark followed by a `continued` start mark joins one span
/// across a barline. A span already open when the sequence starts (its first
/// mark is a `continued` start) has `open_start`; one still open at the end
/// has `open_end`. Malformed input from hand-built layouts is tolerated: an
/// end mark with nothing open is ignored and a beam start inside an open beam
/// closes the earlier beam.
pub fn scan_groups<'a>(elements: impl IntoIterator<Item = &'a MeasureElement>) -> GroupScan {
    let mut scan = GroupScan::default();
    let mut open_beam: Option<usize> = None;
    let mut suspended_beam: Option<usize> = None;
    let mut open_tuplets: Vec<usize> = Vec::new();
    let mut suspended_tuplets: Vec<usize> = Vec::new();
    let mut onset = 0.0;

    for (index, element) in elements.into_iter().enumerate() {
        scan.beam_of.push(None);
        scan.onsets.push(onset);
        match element {
            MeasureElement::GroupMark(GroupMark::BeamStart { spec, continued }) => {
                if let Some(resumed) = suspended_beam.take().filter(|_| *continued) {
                    open_beam = Some(resumed);
                } else {
                    open_beam = Some(scan.beams.len());
                    scan.beams.push(GroupSegment {
                        spec: *spec,
                        members: Vec::new(),
                        open_start: *continued,
                        open_end: false,
                        depth: 0,
                    });
                }
            }
            MeasureElement::GroupMark(GroupMark::BeamEnd { continues }) => {
                if let Some(beam) = open_beam.take() {
                    if *continues {
                        suspended_beam = Some(beam);
                    }
                }
            }
            MeasureElement::GroupMark(GroupMark::TupletStart { spec, continued }) => {
                let resumed = if *continued {
                    suspended_tuplets.pop()
                } else {
                    None
                };
                let tuplet = resumed.unwrap_or_else(|| {
                    scan.tuplets.push(GroupSegment {
                        spec: *spec,
                        members: Vec::new(),
                        open_start: *continued,
                        open_end: false,
                        depth: open_tuplets.len(),
                    });
                    scan.tuplets.len() - 1
                });
                open_tuplets.push(tuplet);
            }
            MeasureElement::GroupMark(GroupMark::TupletEnd { continues }) => {
                if let Some(tuplet) = open_tuplets.pop() {
                    if *continues {
                        suspended_tuplets.push(tuplet);
                    }
                }
            }
            MeasureElement::Barline(_) => {
                onset = 0.0;
            }
            _ => {
                let Some((duration_log2, dots)) = member_duration(element) else {
                    continue;
                };
                if let Some(beam) = open_beam {
                    scan.beams[beam].members.push(index);
                    scan.beam_of[index] = Some(beam);
                }
                for &tuplet in &open_tuplets {
                    scan.tuplets[tuplet].members.push(index);
                }
                let scale = tuplet_time_scale(open_tuplets.iter().map(|&t| &scan.tuplets[t].spec));
                onset += written_whole_notes(duration_log2, dots) * scale;
            }
        }
    }

    for beam in open_beam.into_iter().chain(suspended_beam) {
        scan.beams[beam].open_end = true;
    }
    for tuplet in open_tuplets.into_iter().chain(suspended_tuplets) {
        scan.tuplets[tuplet].open_end = true;
    }
    scan
}
