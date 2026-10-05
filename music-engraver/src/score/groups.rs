//! Beam and tuplet span validation and post-conversion processing.
//!
//! The builder records spans as zero-duration [`GroupMark`] events between
//! ordinary notes, chords, and rests. This module checks that the marks form
//! well-nested spans ([`validate_group_spans`]), tracks the tuplets open at
//! each barline for accidental onsets ([`advance_open_tuplets`]), and, after
//! conversion, gives every beam one stem direction and makes each measure's
//! span marks self-contained ([`finish_group_spans`]).

use crate::layout::group::{BeamSpec, GroupMark, TupletSpec};
use crate::layout::stem::{auto_stem_direction_chord, StemDirection};
use crate::layout::system::{MeasureContent, MeasureEvent};

use super::event::ScoreEvent;
use super::CompletedMeasure;

/// A beam or tuplet span that cannot be engraved.
///
/// Returned (inside [`crate::error::EngraverError::Group`]) by the render
/// entry points; nothing is dropped or silently re-grouped. `measure` is the
/// 1-based number of the measure where the problem was detected and `voice`
/// the builder voice index.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GroupSpanError {
    /// `begin_beam` while a beam is already open in the same voice.
    #[error("measure {measure}, voice {voice}: a beam was begun inside an open beam (beams do not nest)")]
    NestedBeam {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
    /// `end_beam` with no open beam in the voice.
    #[error("measure {measure}, voice {voice}: end_beam without a matching begin_beam")]
    UnmatchedBeamEnd {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
    /// `end_tuplet` with no open tuplet in the voice.
    #[error("measure {measure}, voice {voice}: end_tuplet without a matching begin_tuplet")]
    UnmatchedTupletEnd {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
    /// A beam still open when the score ends.
    #[error("measure {measure}, voice {voice}: beam begun here is never ended")]
    UnclosedBeam {
        /// 1-based measure number where the beam began.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
    /// A tuplet still open when the score ends.
    #[error("measure {measure}, voice {voice}: tuplet begun here is never ended")]
    UnclosedTuplet {
        /// 1-based measure number where the tuplet began.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
    /// A beam with no note, chord, or rest between its begin and end.
    #[error("measure {measure}, voice {voice}: beam has no members")]
    EmptyBeam {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
    /// A tuplet with no note, chord, or rest between its begin and end.
    #[error("measure {measure}, voice {voice}: tuplet has no members")]
    EmptyTuplet {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
    /// A note or chord longer than an eighth inside a beam: it has no flag
    /// to turn into a beam.
    #[error(
        "measure {measure}, voice {voice}: a note of duration_log2 {duration_log2} cannot be beamed (eighth or shorter required)"
    )]
    UnbeamableNote {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
        /// The offending member's duration (`2` = quarter, `1` = half, …).
        duration_log2: i8,
    },
    /// A tuplet ratio with a zero term.
    #[error("measure {measure}, voice {voice}: tuplet ratio {number}:{in_time_of} has a zero term")]
    InvalidTupletRatio {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
        /// Actual-note count of the rejected spec.
        number: u32,
        /// Normal-note count of the rejected spec.
        in_time_of: u32,
    },
    /// A multi-measure rest inside an open beam or tuplet.
    #[error("measure {measure}, voice {voice}: a multi-measure rest cannot be a beam or tuplet member")]
    MultiMeasureRestInSpan {
        /// 1-based measure number.
        measure: usize,
        /// Voice index.
        voice: u8,
    },
}

/// Per-voice span state while validating.
#[derive(Default)]
struct VoiceSpans {
    /// Open beam: (measure number where it began, members so far).
    beam: Option<(usize, usize)>,
    /// Open tuplets, outermost first: (measure number, members so far).
    tuplets: Vec<(usize, usize)>,
}

/// Check that every voice's span marks form properly nested, non-empty
/// spans whose members can be engraved. Spans may cross barlines.
pub(crate) fn validate_group_spans(measures: &[CompletedMeasure]) -> Result<(), GroupSpanError> {
    let mut voices: Vec<VoiceSpans> = Vec::new();
    for (index, completed) in measures.iter().enumerate() {
        let measure = index + 1;
        for (voice, event) in &completed.events {
            let slot = usize::from(*voice);
            if voices.len() <= slot {
                voices.resize_with(slot + 1, VoiceSpans::default);
            }
            let state = &mut voices[slot];
            let voice = *voice;
            match event {
                ScoreEvent::GroupMark(GroupMark::BeamStart { .. }) => {
                    if state.beam.is_some() {
                        return Err(GroupSpanError::NestedBeam { measure, voice });
                    }
                    state.beam = Some((measure, 0));
                }
                ScoreEvent::GroupMark(GroupMark::BeamEnd { .. }) => match state.beam.take() {
                    None => return Err(GroupSpanError::UnmatchedBeamEnd { measure, voice }),
                    Some((_, 0)) => return Err(GroupSpanError::EmptyBeam { measure, voice }),
                    Some(_) => {}
                },
                ScoreEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => {
                    if spec.number == 0 || spec.in_time_of == 0 {
                        return Err(GroupSpanError::InvalidTupletRatio {
                            measure,
                            voice,
                            number: spec.number,
                            in_time_of: spec.in_time_of,
                        });
                    }
                    state.tuplets.push((measure, 0));
                }
                ScoreEvent::GroupMark(GroupMark::TupletEnd { .. }) => match state.tuplets.pop() {
                    None => return Err(GroupSpanError::UnmatchedTupletEnd { measure, voice }),
                    Some((_, 0)) => return Err(GroupSpanError::EmptyTuplet { measure, voice }),
                    Some(_) => {}
                },
                ScoreEvent::Note { duration, .. } | ScoreEvent::Chord { duration, .. } => {
                    if let Some((_, members)) = &mut state.beam {
                        let duration_log2 = super::event::duration_kind_to_log2(duration.kind());
                        if duration_log2 < 3 {
                            return Err(GroupSpanError::UnbeamableNote {
                                measure,
                                voice,
                                duration_log2,
                            });
                        }
                        *members += 1;
                    }
                    count_tuplet_member(state);
                }
                ScoreEvent::Rest { .. } => {
                    if let Some((_, members)) = &mut state.beam {
                        *members += 1;
                    }
                    count_tuplet_member(state);
                }
                ScoreEvent::MultiMeasureRest { .. } => {
                    if state.beam.is_some() || !state.tuplets.is_empty() {
                        return Err(GroupSpanError::MultiMeasureRestInSpan { measure, voice });
                    }
                }
                ScoreEvent::Spacer { .. }
                | ScoreEvent::ClefChange(_)
                | ScoreEvent::TimeSignatureChange(_) => {}
            }
        }
    }
    for (voice, state) in voices.iter().enumerate() {
        let voice = u8::try_from(voice).expect("voice indices come from u8 builder voices");
        if let Some((measure, _)) = state.beam {
            return Err(GroupSpanError::UnclosedBeam { measure, voice });
        }
        if let Some(&(measure, _)) = state.tuplets.first() {
            return Err(GroupSpanError::UnclosedTuplet { measure, voice });
        }
    }
    Ok(())
}

fn count_tuplet_member(state: &mut VoiceSpans) {
    for (_, members) in &mut state.tuplets {
        *members += 1;
    }
}

/// Advance each voice's open-tuplet stack (`stacks[voice]`, outermost first)
/// past one measure's events.
pub(crate) fn advance_open_tuplets(stacks: &mut Vec<Vec<TupletSpec>>, events: &[(u8, ScoreEvent)]) {
    for (voice, event) in events {
        let voice = usize::from(*voice);
        if stacks.len() <= voice {
            stacks.resize_with(voice + 1, Vec::new);
        }
        match event {
            ScoreEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => stacks[voice].push(*spec),
            ScoreEvent::GroupMark(GroupMark::TupletEnd { .. }) => {
                stacks[voice].pop();
            }
            _ => {}
        }
    }
}

fn voice_events(content: &MeasureContent, voice: usize) -> Option<&Vec<MeasureEvent>> {
    match voice {
        0 => Some(&content.events),
        _ => content.additional_voices.get(voice - 1),
    }
}

fn voice_events_mut(content: &mut MeasureContent, voice: usize) -> Option<&mut Vec<MeasureEvent>> {
    match voice {
        0 => Some(&mut content.events),
        _ => content.additional_voices.get_mut(voice - 1),
    }
}

/// Stem direction a voice's events take in a multi-voice measure: even
/// voices up, odd voices down.
pub(crate) fn voice_stem_direction(voice: usize) -> StemDirection {
    if voice.is_multiple_of(2) {
        StemDirection::Up
    } else {
        StemDirection::Down
    }
}

/// Finish converted measures' beam and tuplet spans, voice by voice:
///
/// 1. Every beam's note and chord members get one stem direction: the beam
///    spec's override, else the first member's own requested direction, else
///    the voice direction when the beam starts in a multi-voice measure, else
///    the automatic farthest-from-the-middle-line rule over all members'
///    staff positions (chord tones included).
/// 2. A span crossing a barline is closed at the measure's end with a
///    `continues` end mark and resumed at the next measure's start with a
///    `continued` start mark, so each measure's marks are self-contained
///    while scans across measures still see one span.
pub(crate) fn finish_group_spans(contents: &mut [MeasureContent]) {
    let voices = contents
        .iter()
        .map(|content| content.additional_voices.len() + 1)
        .max()
        .unwrap_or(0);
    for voice in 0..voices {
        resolve_beam_directions(contents, voice);
        split_spans_at_barlines(contents, voice);
    }
}

fn resolve_beam_directions(contents: &mut [MeasureContent], voice: usize) {
    let mut beams: Vec<(BeamSpec, Vec<(usize, usize)>)> = Vec::new();
    let mut open: Option<(BeamSpec, Vec<(usize, usize)>)> = None;
    for (measure, content) in contents.iter().enumerate() {
        let Some(events) = voice_events(content, voice) else {
            continue;
        };
        for (index, event) in events.iter().enumerate() {
            match event {
                MeasureEvent::GroupMark(GroupMark::BeamStart { spec, .. }) => {
                    beams.extend(open.replace((*spec, Vec::new())));
                }
                MeasureEvent::GroupMark(GroupMark::BeamEnd { .. }) => beams.extend(open.take()),
                MeasureEvent::Note(_) | MeasureEvent::Chord(_) => {
                    if let Some((_, members)) = &mut open {
                        members.push((measure, index));
                    }
                }
                _ => {}
            }
        }
    }
    beams.extend(open);
    for (spec, members) in beams {
        let direction = beam_direction(contents, voice, &spec, &members);
        set_member_directions(contents, voice, &members, direction);
    }
}

fn member_event(contents: &[MeasureContent], voice: usize, (measure, index): (usize, usize)) -> &MeasureEvent {
    &voice_events(&contents[measure], voice).expect("member voice exists")[index]
}

fn beam_direction(
    contents: &[MeasureContent],
    voice: usize,
    spec: &BeamSpec,
    members: &[(usize, usize)],
) -> StemDirection {
    if let Some(direction) = spec.stem_direction {
        return direction;
    }
    let requested = members
        .iter()
        .find_map(|&member| match member_event(contents, voice, member) {
            MeasureEvent::Note(note) => note.stem_direction,
            MeasureEvent::Chord(chord) => chord.stem_direction,
            _ => None,
        });
    if let Some(direction) = requested {
        return direction;
    }
    if let Some(&(measure, _)) = members.first() {
        if !contents[measure].additional_voices.is_empty() {
            return voice_stem_direction(voice);
        }
    }
    let positions: Vec<i8> = members
        .iter()
        .flat_map(|&member| match member_event(contents, voice, member) {
            MeasureEvent::Note(note) => vec![note.staff_position],
            MeasureEvent::Chord(chord) => chord.staff_positions.clone(),
            _ => Vec::new(),
        })
        .collect();
    auto_stem_direction_chord(&positions)
}

fn set_member_directions(
    contents: &mut [MeasureContent],
    voice: usize,
    members: &[(usize, usize)],
    direction: StemDirection,
) {
    for &(measure, index) in members {
        let events = voice_events_mut(&mut contents[measure], voice).expect("member voice exists");
        match &mut events[index] {
            MeasureEvent::Note(note) => note.stem_direction = Some(direction),
            MeasureEvent::Chord(chord) => chord.stem_direction = Some(direction),
            _ => {}
        }
    }
}

fn split_spans_at_barlines(contents: &mut [MeasureContent], voice: usize) {
    let mut open_beam: Option<BeamSpec> = None;
    let mut open_tuplets: Vec<TupletSpec> = Vec::new();
    for content in contents.iter_mut() {
        let Some(events) = voice_events_mut(content, voice) else {
            continue;
        };
        let mut resumed: Vec<MeasureEvent> = open_tuplets
            .iter()
            .map(|&spec| {
                MeasureEvent::GroupMark(GroupMark::TupletStart {
                    spec,
                    continued: true,
                })
            })
            .collect();
        if let Some(spec) = open_beam {
            resumed.push(MeasureEvent::GroupMark(GroupMark::BeamStart {
                spec,
                continued: true,
            }));
        }
        for event in events.iter() {
            match event {
                MeasureEvent::GroupMark(GroupMark::BeamStart { spec, .. }) => {
                    open_beam = Some(*spec);
                }
                MeasureEvent::GroupMark(GroupMark::BeamEnd { .. }) => open_beam = None,
                MeasureEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => {
                    open_tuplets.push(*spec);
                }
                MeasureEvent::GroupMark(GroupMark::TupletEnd { .. }) => {
                    open_tuplets.pop();
                }
                _ => {}
            }
        }
        events.splice(0..0, resumed);
        if open_beam.is_some() {
            events.push(MeasureEvent::GroupMark(GroupMark::BeamEnd { continues: true }));
        }
        events.extend(
            open_tuplets
                .iter()
                .map(|_| MeasureEvent::GroupMark(GroupMark::TupletEnd { continues: true })),
        );
    }
}
