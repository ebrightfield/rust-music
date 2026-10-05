//! Explicit system breaks: break directives at measure boundaries and splits
//! of a logical measure at a forced mid-measure break.
//!
//! Score builders record where a break was requested (`system_break()`,
//! `no_break()`) as a position inside the primary voice of a logical measure.
//! [`LineBreakPlan::from_requests`] resolves the requests against the built
//! [`MeasureContent`]s:
//!
//! - at a measure boundary it becomes that measure's
//!   [`MeasureMeta::line_break`](crate::layout::measure_meta::MeasureMeta::line_break);
//! - inside a measure, a forced break becomes a *split*: the musical onset at
//!   which [`LineBreakPlan::apply`] cuts the measure into visual pieces. Every
//!   piece after the first is flagged `meta.continuation` and keeps the
//!   logical measure's number and lengths. Accidentals were resolved for the
//!   whole logical measure before the split, so a piece never restarts
//!   accidental state.
//!
//! Splits are onsets rather than event indices so that every stave of a
//! multi-staff score can be cut at the same musical moment.

use std::collections::{BTreeMap, BTreeSet};

use crate::layout::barline::BarlineStyle;
use crate::layout::group::{GroupMark, TupletSpec};
use crate::layout::measure_meta::{LineBreak, MeasureLength};
use crate::layout::system::{MeasureContent, MeasureEvent};
use crate::layout::volta::{VoltaAnnotation, VoltaHooks};

/// Written length of a note value: `duration_log2` (-1 breve, 0 whole,
/// 1 half, …) extended by `dots` augmentation dots.
pub(crate) fn written_length(duration_log2: i8, dots: u8) -> MeasureLength {
    let (numerator, denominator) = if duration_log2 >= 0 {
        (1u64, 1u64 << duration_log2)
    } else {
        (1u64 << duration_log2.unsigned_abs(), 1)
    };
    // `dots` dots make a value (2^(dots+1) - 1) / 2^dots of itself.
    MeasureLength::new(numerator * ((1u64 << (dots + 1)) - 1), denominator << dots)
}

/// Written length of a single rhythmic event. A span mark or an inline
/// barline consumes no time; tuplet scaling belongs to its surrounding voice.
pub(crate) fn event_length(event: &MeasureEvent) -> MeasureLength {
    match event {
        MeasureEvent::Note(note) => written_length(note.duration_log2, note.dots),
        MeasureEvent::Rest(rest) => written_length(rest.duration_log2, rest.dots),
        MeasureEvent::Chord(chord) => written_length(chord.duration_log2, chord.dots),
        MeasureEvent::Spacer(spacer) => spacer
            .duration_log2
            .map_or(MeasureLength::ZERO, |log2| written_length(log2, spacer.dots)),
        MeasureEvent::GroupMark(_)
        | MeasureEvent::MultiMeasureRest { .. }
        | MeasureEvent::Barline(_)
        | MeasureEvent::ClefChange(_)
        | MeasureEvent::TimeSignature(_) => MeasureLength::ZERO,
    }
}

/// Each voice has its own independent stack of tuplets, including tuplets
/// resumed at a logical barline. Advance marks before scaling the next event.
fn performed_length(event: &MeasureEvent, tuplets: &mut Vec<TupletSpec>) -> MeasureLength {
    match event {
        MeasureEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => {
            tuplets.push(*spec);
            MeasureLength::ZERO
        }
        MeasureEvent::GroupMark(GroupMark::TupletEnd { .. }) => {
            tuplets.pop();
            MeasureLength::ZERO
        }
        _ => {
            let written = event_length(event);
            tuplets.iter().fold(written, |length, spec| {
                MeasureLength::new(
                    length.numerator() * u64::from(spec.in_time_of),
                    length.denominator() * u64::from(spec.number),
                )
            })
        }
    }
}

fn voice_length(events: &[MeasureEvent]) -> MeasureLength {
    let mut tuplets = Vec::new();
    events.iter().fold(MeasureLength::ZERO, |total, event| {
        total + performed_length(event, &mut tuplets)
    })
}

/// Length of the longest voice of a measure.
fn content_length(content: &MeasureContent) -> MeasureLength {
    content
        .additional_voices
        .iter()
        .map(|voice| voice_length(voice))
        .fold(voice_length(&content.events), MeasureLength::max)
}

/// A `system_break()` / `no_break()` call: `kind` requested after the first
/// `position` primary-voice events of logical measure `measure`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineBreakRequest {
    pub measure: usize,
    pub position: usize,
    pub kind: LineBreak,
}

/// Explicit line-break directives of a score, by logical measure.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LineBreakPlan {
    /// Directive after logical measure `index` (never `Auto`).
    boundaries: BTreeMap<usize, LineBreak>,
    /// Forced breaks inside logical measures: (measure index, onset).
    splits: BTreeSet<(usize, MeasureLength)>,
}

impl LineBreakPlan {
    /// Resolve a stave's requests, in call order, against its logical
    /// `contents` (see [`Self::request`]).
    pub(crate) fn from_requests(
        requests: &[LineBreakRequest],
        contents: &[MeasureContent],
    ) -> Self {
        let mut plan = Self::default();
        for request in requests {
            plan.request(contents, *request);
        }
        plan
    }

    /// Record `request`, resolved against the score's logical `contents`. A
    /// later request at the same boundary replaces an earlier one.
    ///
    /// A request before anything sounds in the measure applies to the
    /// boundary before it (and is meaningless before the first measure); a
    /// request after everything in it has sounded applies to the boundary
    /// after it. Inside the measure, `Force` splits it at the request's onset
    /// and `Forbid` has nothing to forbid: no policy breaks inside a measure.
    fn request(&mut self, contents: &[MeasureContent], request: LineBreakRequest) {
        let LineBreakRequest {
            measure,
            position,
            kind,
        } = request;
        let Some(content) = contents.get(measure) else {
            // Requested after the final barline: nothing follows to break to.
            return;
        };
        let at = voice_length(&content.events[..position.min(content.events.len())]);
        if at == MeasureLength::ZERO {
            if let Some(previous) = measure.checked_sub(1) {
                self.set_boundary(previous, kind);
            }
        } else if at >= content_length(content) {
            self.set_boundary(measure, kind);
        } else {
            // The last directive at this onset wins, including a `no_break`
            // following an earlier mid-measure `system_break`.
            if kind == LineBreak::Force {
                self.splits.insert((measure, at));
            } else {
                self.splits.remove(&(measure, at));
            }
        }
    }

    fn set_boundary(&mut self, measure: usize, kind: LineBreak) {
        if kind == LineBreak::Auto {
            self.boundaries.remove(&measure);
        } else {
            self.boundaries.insert(measure, kind);
        }
    }

    /// Combine the directives of another stave of the same score: every split
    /// applies to all staves, and at a shared boundary a forced break wins
    /// over a forbidden one.
    pub(crate) fn merge(&mut self, other: &Self) {
        for (&measure, &kind) in &other.boundaries {
            let combined = match (self.boundaries.get(&measure), kind) {
                (Some(LineBreak::Force), _) | (_, LineBreak::Force) => LineBreak::Force,
                _ => kind,
            };
            self.boundaries.insert(measure, combined);
        }
        self.splits.extend(other.splits.iter().copied());
    }

    /// Apply the plan to one stave's logical measures: set each boundary's
    /// `meta.line_break` and cut split measures into pieces.
    ///
    /// Each piece but the last ends at a split with a forced break and closes
    /// with the inline barline that sits at the split onset (an invisible
    /// barline when there is none); the last piece keeps the measure's own
    /// barline and boundary directive. Events are assigned to pieces by
    /// onset; an event sounding across a split stays in the earlier piece.
    /// Zero-duration events at a split onset other than inline barlines
    /// (e.g. a clef change) open the later piece.
    pub(crate) fn apply(&self, contents: Vec<MeasureContent>) -> Vec<MeasureContent> {
        let mut pieces = Vec::with_capacity(contents.len() + self.splits.len());
        for (index, mut content) in contents.into_iter().enumerate() {
            if let Some(&kind) = self.boundaries.get(&index) {
                content.meta.line_break = kind;
            }
            let points: Vec<MeasureLength> = self
                .splits
                .range((index, MeasureLength::ZERO)..(index + 1, MeasureLength::ZERO))
                .map(|&(_, at)| at)
                .collect();
            if points.is_empty() {
                pieces.push(content);
            } else {
                pieces.extend(split_content(content, &points));
            }
        }
        pieces
    }
}

/// Distribute one voice's events over `points.len() + 1` pieces by onset.
fn split_voice(
    events: Vec<MeasureEvent>,
    points: &[MeasureLength],
) -> (Vec<Vec<MeasureEvent>>, Vec<MeasureLength>) {
    let mut pieces: Vec<Vec<MeasureEvent>> = vec![Vec::new(); points.len() + 1];
    let mut first_onsets: Vec<Option<MeasureLength>> = vec![None; pieces.len()];
    let mut onset = MeasureLength::ZERO;
    let mut tuplets = Vec::new();
    for event in events {
        let closes_piece = matches!(
            event,
            MeasureEvent::Barline(_)
                | MeasureEvent::GroupMark(GroupMark::BeamEnd { .. } | GroupMark::TupletEnd { .. })
        );
        let piece = points
            .iter()
            .filter(|&&at| at < onset || (at == onset && !closes_piece))
            .count();
        first_onsets[piece].get_or_insert(onset);
        onset = onset + performed_length(&event, &mut tuplets);
        pieces[piece].push(event);
    }
    // Conversion closed spans at logical barlines. Also close them at each
    // visual system break so partial beams/brackets appear on both systems.
    let mut open_tuplets = Vec::new();
    let mut open_beam = None;
    for index in 0..points.len() {
        for event in &pieces[index] {
            match event {
                MeasureEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => {
                    open_tuplets.push(*spec)
                }
                MeasureEvent::GroupMark(GroupMark::TupletEnd { .. }) => {
                    open_tuplets.pop();
                }
                MeasureEvent::GroupMark(GroupMark::BeamStart { spec, .. }) => {
                    open_beam = Some(*spec)
                }
                MeasureEvent::GroupMark(GroupMark::BeamEnd { .. }) => open_beam = None,
                _ => {}
            }
        }
        let closing_barline = matches!(pieces[index].last(), Some(MeasureEvent::Barline(_)))
            .then(|| pieces[index].pop())
            .flatten();
        if open_beam.is_some() {
            pieces[index].push(MeasureEvent::GroupMark(GroupMark::BeamEnd {
                continues: true,
            }));
        }
        for _ in &open_tuplets {
            pieces[index].push(MeasureEvent::GroupMark(GroupMark::TupletEnd {
                continues: true,
            }));
        }
        pieces[index].extend(closing_barline);
        let mut resume: Vec<MeasureEvent> = open_tuplets
            .iter()
            .map(|&spec| {
                MeasureEvent::GroupMark(GroupMark::TupletStart {
                    spec,
                    continued: true,
                })
            })
            .collect();
        if let Some(spec) = open_beam {
            resume.push(MeasureEvent::GroupMark(GroupMark::BeamStart {
                spec,
                continued: true,
            }));
        }
        pieces[index + 1].splice(0..0, resume);
    }
    let starts = first_onsets
        .into_iter()
        .enumerate()
        .map(|(piece, first)| {
            let boundary = if piece == 0 {
                MeasureLength::ZERO
            } else {
                points[piece - 1]
            };
            first.unwrap_or(boundary).max(boundary)
        })
        .collect();
    (pieces, starts)
}

/// The volta annotation of each of `count` pieces of one measure: the text
/// and left hook stay on the first piece, the right hook on the last.
fn split_volta(volta: Option<VoltaAnnotation>, count: usize) -> Vec<Option<VoltaAnnotation>> {
    let Some(volta) = volta else {
        return vec![None; count];
    };
    let (left, right) = match volta.hooks {
        VoltaHooks::Both => (true, true),
        VoltaHooks::LeftOnly => (true, false),
        VoltaHooks::RightOnly => (false, true),
        VoltaHooks::Neither => (false, false),
    };
    (0..count)
        .map(|piece| {
            let first = piece == 0;
            let last = piece + 1 == count;
            Some(VoltaAnnotation {
                text: if first { volta.text.clone() } else { None },
                hooks: match (left && first, right && last) {
                    (true, true) => VoltaHooks::Both,
                    (true, false) => VoltaHooks::LeftOnly,
                    (false, true) => VoltaHooks::RightOnly,
                    (false, false) => VoltaHooks::Neither,
                },
            })
        })
        .collect()
}

fn split_content(content: MeasureContent, points: &[MeasureLength]) -> Vec<MeasureContent> {
    let MeasureContent {
        events,
        barline,
        volta,
        additional_voices,
        meta,
    } = content;
    let count = points.len() + 1;
    let (primary, primary_onsets) = split_voice(events, points);
    let mut voices: Vec<_> = additional_voices
        .into_iter()
        .map(|voice| split_voice(voice, points))
        .collect();
    let voltas = split_volta(volta, count);

    primary
        .into_iter()
        .zip(voltas)
        .enumerate()
        .map(|(piece, (mut events, volta))| {
            let last = piece + 1 == count;
            let closing = if last {
                barline
            } else if let Some(MeasureEvent::Barline(style)) = events.last() {
                let style = *style;
                events.pop();
                style
            } else {
                BarlineStyle::Invisible
            };
            let mut piece_meta = meta.clone();
            piece_meta.continuation = meta.continuation || piece > 0;
            piece_meta.visual_start = if piece == 0 {
                meta.visual_start
            } else {
                points[piece - 1]
            };
            piece_meta.visual_voice_onsets = std::iter::once(primary_onsets[piece])
                .chain(voices.iter().map(|(_, onsets)| onsets[piece]))
                .collect();
            if !last {
                piece_meta.line_break = LineBreak::Force;
            }
            MeasureContent {
                events,
                barline: closing,
                volta,
                additional_voices: voices
                    .iter_mut()
                    .map(|(events, _)| std::mem::take(&mut events[piece]))
                    .collect(),
                meta: piece_meta,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
