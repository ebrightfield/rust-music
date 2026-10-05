//! Score event types and conversion from `music` crate types to layout events.
//!
//! Handles accidental resolution against key signatures, within-measure
//! accidental state (cancellations, reinstated key-signature accidentals,
//! suppression of redundant accidentals, forced and cautionary display),
//! and conversion of `Pitch`/`Duration` to the internal layout representation.

use std::collections::HashMap;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::pitch::Pitch;
use music::note::spelling::{Accidental, Spelling};

use crate::layout::accidental::{
    accidental_glyph, AccidentalDisplay, AccidentalPolicy, ResolvedAccidental,
};
use crate::layout::barline::BarlineStyle;
use crate::layout::grace::{GraceGroup, GraceNoteEvent};
use crate::layout::key_signature::KeySignature;
use crate::layout::group::{GroupMark, TupletSpec};
use crate::layout::measure::{ChordEvent, NoteAnnotations, NoteEvent, RestEvent, SpacerEvent};
use crate::layout::measure_meta::MeasureLength;
use crate::layout::note_placement::pitch_to_staff_position;
use crate::layout::system::{ClefChange, MeasureEvent};
use crate::layout::time_signature::TimeSignature;

/// An event being accumulated in the current measure.
#[derive(Clone, Debug)]
pub(crate) enum ScoreEvent {
    Note {
        pitch: Pitch,
        duration: Duration,
        annotations: NoteAnnotations,
    },
    Rest {
        duration: Duration,
    },
    Chord {
        pitches: Vec<Pitch>,
        duration: Duration,
        annotations: NoteAnnotations,
    },
    /// A zero-duration beam or tuplet span boundary. The notes, chords, and
    /// rests between a start and its matching end (in the same voice) are
    /// the span's members.
    GroupMark(GroupMark),
    /// Multi-measure rest: the rendered measure consists of an H-bar (or a
    /// church-rest cluster, for small counts) with a count number indicating
    /// how many consecutive measures of rest this single measure-shaped
    /// frame represents (engraved convention for empty passages in parts).
    MultiMeasureRest {
        count: u32,
        style: crate::layout::multi_measure_rest::MultiMeasureRestStyle,
    },
    /// Zero-duration inline barline inside the measure. It does not end the
    /// measure, advance the measure number, or reset accidental state.
    Barline(BarlineStyle),
    /// Invisible rhythmic placeholder: takes `duration` but draws nothing.
    Spacer {
        duration: Duration,
    },
    /// Zero-duration clef change; later pitches on the staff use the new clef.
    ClefChange(ClefChange),
    /// Zero-duration meter change, valid only at the start of a measure.
    TimeSignatureChange(TimeSignature),
}

impl ScoreEvent {
    /// Whether this is a zero-duration structural change (clef or meter).
    pub(crate) fn is_structural(&self) -> bool {
        matches!(self, Self::ClefChange(_) | Self::TimeSignatureChange(_))
    }
}

/// Convert a `DurationKind` to the log2 representation used by the layout engine.
///
/// Layout uses: -1=breve (double whole), 0=whole, 1=half, 2=quarter,
/// 3=eighth, etc. The breve keeps its own value so glyph selection, stem
/// rules, and spacing never confuse it with a whole note.
pub(crate) fn duration_kind_to_log2(kind: DurationKind) -> i8 {
    match kind {
        DurationKind::Breve => -1,
        DurationKind::Whole => 0,
        DurationKind::Half => 1,
        DurationKind::Qtr => 2,
        DurationKind::Eighth => 3,
        DurationKind::Sixteenth => 4,
        DurationKind::ThirtySecond => 5,
        DurationKind::SixtyFourth => 6,
        DurationKind::OneTwentyEighth => 7,
    }
}

/// Key for tracking accidentals within a measure: (diatonic letter index 0–6, octave).
/// Uses `i32::from(&Letter)` since `Letter` doesn't implement `Hash`/`Eq`.
type NoteKey = (i32, i8);

/// In-measure accidental state for one staff: the alteration most recently
/// notated for each letter+octave in the current measure. A key absent from
/// the map is governed by the key signature. Callers start a fresh map at
/// every barline.
pub(crate) type AccidentalTracker = HashMap<NoteKey, Accidental>;

pub(crate) fn note_key(pitch: &Pitch) -> NoteKey {
    let spelling = Spelling::from(&pitch.note);
    (i32::from(&spelling.letter), pitch.octave)
}

/// The alteration the key signature gives `letter`: sharp or flat for letters
/// it alters, natural otherwise. This is the measure's baseline state.
pub(crate) fn key_signature_accidental(
    letter: music::note::spelling::Letter,
    key_sig: &KeySignature,
) -> Accidental {
    if !note_altered_in_key(letter, key_sig) {
        return Accidental::Natural;
    }
    match key_sig {
        KeySignature::Sharps(_) => Accidental::Sharp,
        KeySignature::Flats(_) => Accidental::Flat,
        KeySignature::Open => Accidental::Natural,
    }
}

/// Resolve whether an accidental should be displayed for a given pitch in a key signature.
///
/// Each note is resolved independently against the key signature, with
/// automatic display and no within-measure state.
#[cfg(test)]
pub(crate) fn should_show_accidental(
    pitch: &Pitch,
    key_sig: &KeySignature,
) -> Option<smufl::Glyph> {
    resolve_accidental(pitch, key_sig, AccidentalDisplay::Auto, None).map(|resolved| resolved.glyph)
}

/// Resolve which accidental, if any, to engrave for `pitch`.
///
/// The *requested* alteration is the pitch's own spelling (natural for an
/// unaltered letter). The *active* alteration for its letter and octave is the
/// in-measure override from `seen_in_measure` if present, otherwise the key
/// signature's ([`key_signature_accidental`]). With `seen_in_measure = None`
/// the key signature alone is active.
///
/// - [`AccidentalDisplay::Auto`]: engrave the requested accidental iff it
///   differs from the active one. One comparison covers naturals, sharps,
///   flats, double sharps, and double flats alike: it cancels, reinstates the
///   key signature after an in-measure alteration, and suppresses repeats.
/// - [`AccidentalDisplay::Force`]: always engrave the plain requested accidental.
/// - [`AccidentalDisplay::Cautionary`]: always engrave it in parentheses.
/// - [`AccidentalDisplay::Hide`]: never engrave it.
pub(crate) fn resolve_accidental(
    pitch: &Pitch,
    key_sig: &KeySignature,
    display: AccidentalDisplay,
    seen_in_measure: Option<&AccidentalTracker>,
) -> Option<ResolvedAccidental> {
    let spelling = Spelling::from(&pitch.note);
    let requested = spelling.acc;
    let glyph = accidental_glyph(requested);
    match display {
        AccidentalDisplay::Auto => {
            let active = seen_in_measure
                .and_then(|seen| seen.get(&note_key(pitch)).copied())
                .unwrap_or_else(|| key_signature_accidental(spelling.letter, key_sig));
            (requested != active).then_some(ResolvedAccidental::plain(glyph))
        }
        AccidentalDisplay::Force => Some(ResolvedAccidental::plain(glyph)),
        AccidentalDisplay::Cautionary => Some(ResolvedAccidental::cautionary(glyph)),
        AccidentalDisplay::Hide => None,
    }
}

/// Check whether a letter name is altered (has a sharp or flat) in the given key signature.
pub(crate) fn note_altered_in_key(
    letter: music::note::spelling::Letter,
    key_sig: &KeySignature,
) -> bool {
    use music::note::spelling::Letter;

    let sharp_order = [
        Letter::F,
        Letter::C,
        Letter::G,
        Letter::D,
        Letter::A,
        Letter::E,
        Letter::B,
    ];
    let flat_order = [
        Letter::B,
        Letter::E,
        Letter::A,
        Letter::D,
        Letter::G,
        Letter::C,
        Letter::F,
    ];

    match key_sig {
        KeySignature::Open => false,
        KeySignature::Sharps(n) => {
            let n = (*n as usize).min(7);
            sharp_order[..n].contains(&letter)
        }
        KeySignature::Flats(n) => {
            let n = (*n as usize).min(7);
            flat_order[..n].contains(&letter)
        }
    }
}

/// Resolve an accidental under the staff's [`AccidentalPolicy`] and, with a
/// tracker, record the pitch's own alteration as the in-measure state for its
/// letter and octave.
///
/// Every display policy records state: a forced, cautionary, or hidden sign
/// tells the reader the alteration just as an automatic one does. Under
/// [`AccidentalPolicy::Forget`] the recorded state is never consulted, so
/// every note is compared against the key signature alone.
fn resolve_and_track(
    pitch: &Pitch,
    key_sig: &KeySignature,
    display: AccidentalDisplay,
    policy: AccidentalPolicy,
    seen: Option<&mut AccidentalTracker>,
) -> Option<ResolvedAccidental> {
    let in_force = match policy {
        AccidentalPolicy::Default => seen.as_deref(),
        AccidentalPolicy::Forget => None,
    };
    let resolved = resolve_accidental(pitch, key_sig, display, in_force);
    if let Some(seen) = seen {
        seen.insert(note_key(pitch), Spelling::from(&pitch.note).acc);
    }
    resolved
}

/// Pre-resolved accidentals, one per notated pitch, in the order
/// [`visit_pitches`] visits an event's pitches.
pub(crate) type ResolvedAccidentals<'a> = std::slice::Iter<'a, Option<ResolvedAccidental>>;

fn next_accidental(accidentals: &mut ResolvedAccidentals<'_>) -> Option<ResolvedAccidental> {
    *accidentals
        .next()
        .expect("one resolved accidental per notated pitch")
}

/// Resolve written grace pitches ahead of their principal event.
fn convert_annotations(
    annotations: &NoteAnnotations,
    clef: &Clef,
    accidentals: &mut ResolvedAccidentals<'_>,
) -> NoteAnnotations {
    let mut converted = annotations.clone();
    if let Some(graces) = &annotations.grace_notes {
        converted.grace_group = Some(GraceGroup {
            kind: graces.kind,
            slur: graces.slur,
            notes: graces.notes.iter().map(|grace| GraceNoteEvent {
                staff_position: pitch_to_staff_position(&grace.pitch, clef),
                duration_log2: duration_kind_to_log2(grace.duration.kind()),
                dots: grace.duration.num_dots(),
                accidental: next_accidental(accidentals),
            }).collect(),
        });
        converted.grace_notes = None;
    }
    converted
}

/// Convert a `ScoreEvent` into a `MeasureEvent` for the layout engine.
///
/// Accidentals are not resolved here: each notated pitch takes the next entry
/// of `accidentals`, which [`resolve_measure_accidentals`] produced for the
/// whole measure in musical order. A note or chord's requested stem
/// direction ([`NoteAnnotations::stem_direction`]) becomes the event's
/// `stem_direction`.
pub(crate) fn convert_resolved_event(
    event: &ScoreEvent,
    clef: &Clef,
    accidentals: &mut ResolvedAccidentals<'_>,
) -> MeasureEvent {
    match event {
        ScoreEvent::Note {
            pitch,
            duration,
            annotations,
        } => {
            let annotations = convert_annotations(annotations, clef, accidentals);
            let staff_pos = pitch_to_staff_position(pitch, clef);
            let log2 = duration_kind_to_log2(duration.kind());
            let dots = duration.num_dots();
            let acc = next_accidental(accidentals);

            MeasureEvent::Note(NoteEvent {
                staff_position: staff_pos,
                duration_log2: log2,
                dots,
                accidental: acc,
                stem_direction: annotations.stem_direction,
                annotations: annotations.clone(),
            })
        }
        ScoreEvent::Rest { duration } => {
            let log2 = duration_kind_to_log2(duration.kind());
            let dots = duration.num_dots();

            MeasureEvent::Rest(RestEvent {
                duration_log2: log2,
                dots,
            })
        }
        ScoreEvent::Chord {
            pitches,
            duration,
            annotations,
        } => {
            let annotations = convert_annotations(annotations, clef, accidentals);
            let log2 = duration_kind_to_log2(duration.kind());
            let dots = duration.num_dots();
            let staff_positions: Vec<i8> = pitches
                .iter()
                .map(|p| pitch_to_staff_position(p, clef))
                .collect();
            let chord_accidentals: Vec<Option<ResolvedAccidental>> = pitches
                .iter()
                .map(|_| next_accidental(accidentals))
                .collect();

            MeasureEvent::Chord(ChordEvent {
                staff_positions,
                duration_log2: log2,
                dots,
                accidentals: chord_accidentals,
                stem_direction: annotations.stem_direction,
                annotations: annotations.clone(),
            })
        }
        ScoreEvent::GroupMark(mark) => MeasureEvent::GroupMark(*mark),
        ScoreEvent::MultiMeasureRest { count, style } => MeasureEvent::MultiMeasureRest {
            count: *count,
            style: *style,
        },
        ScoreEvent::Barline(style) => MeasureEvent::Barline(*style),
        ScoreEvent::Spacer { duration } => MeasureEvent::Spacer(SpacerEvent {
            duration_log2: duration_kind_to_log2(duration.kind()),
            dots: duration.num_dots(),
        }),
        ScoreEvent::ClefChange(change) => MeasureEvent::ClefChange(*change),
        ScoreEvent::TimeSignatureChange(time_signature) => {
            MeasureEvent::TimeSignature(time_signature.kind.clone())
        }
    }
}

/// Time of a pitch within its measure, in 128th-note ticks multiplied by the
/// measure's [`tuplet_tick_scale`] so that every tuplet member starts on an
/// exact integer.
pub(crate) type Onset = u64;

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Performed-time ratio of the open tuplet stack: written ticks are
/// multiplied by `.0` (the product of `in_time_of`) and divided by `.1` (the
/// product of tuplet numbers). `(1, 1)` outside tuplets.
type TupletRatio = (u64, u64);

fn tuplet_ratio(open: &[TupletSpec]) -> TupletRatio {
    open.iter().fold((1, 1), |(num, den), spec| {
        (
            num * u64::from(spec.in_time_of),
            den * u64::from(spec.number),
        )
    })
}

/// Least common multiple of the tuplet-number products of every tuplet stack
/// that occurs in `voiced_events` (starting from each voice's `open_tuplets`
/// at the measure start; 1 without tuplets): scaling ticks by it makes every
/// tuplet member's onset integral, nested tuplets included.
fn tuplet_tick_scale(voiced_events: &[(u8, ScoreEvent)], open_tuplets: &[Vec<TupletSpec>]) -> u64 {
    let lcm = |scale: u64, number: u64| scale / gcd(scale, number) * number;
    let mut stacks: Vec<Vec<TupletSpec>> = open_tuplets.to_vec();
    let mut scale = stacks
        .iter()
        .fold(1, |scale, stack| lcm(scale, tuplet_ratio(stack).1));
    for (voice, event) in voiced_events {
        let voice = usize::from(*voice);
        if stacks.len() <= voice {
            stacks.resize_with(voice + 1, Vec::new);
        }
        match event {
            ScoreEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => {
                stacks[voice].push(*spec);
                scale = lcm(scale, tuplet_ratio(&stacks[voice]).1);
            }
            ScoreEvent::GroupMark(GroupMark::TupletEnd { .. }) => {
                stacks[voice].pop();
            }
            _ => {}
        }
    }
    scale
}

/// Scaled ticks an event of written `duration` advances, performed at the
/// open tuplet stack's `ratio` (`scale` must be a multiple of `ratio.1`).
fn scaled_ticks(duration: &Duration, scale: u64, (num, den): TupletRatio) -> Onset {
    duration.ticks() as u64 * scale * num / den
}

/// Display policy for pitch `index` of an annotated note/chord; `None` for
/// unpitched heads, which neither display nor track accidentals.
fn pitch_display(annotations: &NoteAnnotations, index: usize) -> Option<AccidentalDisplay> {
    (!annotations.unpitched).then(|| {
        annotations
            .accidental_displays
            .get(index)
            .copied()
            .unwrap_or_default()
    })
}

/// Visit pitches in conversion order: written graces before the principal,
/// followed by its chord members. A grace sounds before the shared onset.
///
/// Returns the event's duration in ticks scaled by `scale` and performed at
/// the open tuplets' `ratio`; span marks and multi-measure rests take none.
fn visit_pitches(
    event: &ScoreEvent,
    scale: u64,
    ratio: TupletRatio,
    mut visit: impl FnMut(Pitch, Option<AccidentalDisplay>, bool),
) -> Onset {
    let (pitches, duration, annotations) = match event {
        ScoreEvent::Note {
            pitch,
            duration,
            annotations,
        } => (std::slice::from_ref(pitch), duration, annotations),
        ScoreEvent::Chord {
            pitches,
            duration,
            annotations,
        } => (pitches.as_slice(), duration, annotations),
        ScoreEvent::Rest { duration } | ScoreEvent::Spacer { duration } => {
            return scaled_ticks(duration, scale, ratio);
        }
        ScoreEvent::GroupMark(_) | ScoreEvent::Barline(_) | ScoreEvent::MultiMeasureRest { .. }
        | ScoreEvent::ClefChange(_) | ScoreEvent::TimeSignatureChange(_) => return 0,
    };
    if let Some(graces) = &annotations.grace_notes {
        for grace in &graces.notes {
            visit(grace.pitch, Some(grace.accidental), true);
        }

    }
    for (index, pitch) in pitches.iter().enumerate() {
        visit(*pitch, pitch_display(annotations, index), false);
    }
    scaled_ticks(duration, scale, ratio)
}

/// One measure's onsets and performed length at an exact tuplet-aware tick scale.
pub(crate) struct MeasureTimeline {
    pub onsets: Vec<Onset>,
    pub scale: u64,
    pub length: Onset,
}

impl MeasureTimeline {
    pub fn to_length(&self, ticks: Onset) -> MeasureLength {
        MeasureLength::new(ticks, 128 * self.scale)
    }
}

pub(crate) fn measure_timeline(
    voiced_events: &[(u8, ScoreEvent)],
    open_tuplets: &[Vec<TupletSpec>],
) -> MeasureTimeline {
    let scale = tuplet_tick_scale(voiced_events, open_tuplets);
    let mut clocks: Vec<Onset> = Vec::new();
    let mut stacks = open_tuplets.to_vec();
    let mut onsets = Vec::with_capacity(voiced_events.len());
    for (voice, event) in voiced_events {
        let index = usize::from(*voice);
        if clocks.len() <= index {
            clocks.resize(index + 1, 0);
        }
        if stacks.len() <= index {
            stacks.resize_with(index + 1, Vec::new);
        }
        onsets.push(clocks[index]);
        match event {
            ScoreEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => {
                stacks[index].push(*spec);
            }
            ScoreEvent::GroupMark(GroupMark::TupletEnd { .. }) => {
                stacks[index].pop();
            }
            _ => {
                clocks[index] += visit_pitches(event, scale, tuplet_ratio(&stacks[index]), |_, _, _| {});
            }
        }
    }
    MeasureTimeline { onsets, scale, length: clocks.into_iter().max().unwrap_or(0) }
}

/// Resolve every notated pitch's accidental in one measure of one staff, in
/// musical rather than builder order, under the staff's [`AccidentalPolicy`].
///
/// Accidental state is staff-wide: every voice reads and writes one tracker,
/// which starts empty (key signature only) at each barline. Each voice keeps
/// its own clock from the measure start, advancing by written durations
/// (dots included, members of open tuplet spans scaled by the product of
/// their ratios). `open_tuplets[voice]` lists the tuplet spans still open
/// from the previous measure in that voice, outermost first. Pitches are
/// then resolved in order of onset, ties broken by voice index ascending and
/// then by builder order within the voice (chord members in input order).
/// Entering voices in a different builder order therefore cannot change the
/// result.
///
/// Returns one entry per notated pitch in builder order, ready for
/// [`convert_resolved_event`].
pub(crate) fn resolve_measure_accidentals(
    voiced_events: &[(u8, ScoreEvent)],
    key_sig: &KeySignature,
    open_tuplets: &[Vec<TupletSpec>],
    policy: AccidentalPolicy,
) -> Vec<Option<ResolvedAccidental>> {
    let scale = tuplet_tick_scale(voiced_events, open_tuplets);
    let mut voice_clocks: Vec<Onset> = Vec::new();
    let mut stacks: Vec<Vec<TupletSpec>> = open_tuplets.to_vec();
    let mut pitches: Vec<(Onset, bool, u8, Pitch, Option<AccidentalDisplay>)> = Vec::new();
    for (voice, event) in voiced_events {
        let voice_index = usize::from(*voice);
        if voice_clocks.len() <= voice_index {
            voice_clocks.resize(voice_index + 1, 0);
        }
        if stacks.len() <= voice_index {
            stacks.resize_with(voice_index + 1, Vec::new);
        }
        match event {
            ScoreEvent::GroupMark(GroupMark::TupletStart { spec, .. }) => {
                stacks[voice_index].push(*spec);
            }
            ScoreEvent::GroupMark(GroupMark::TupletEnd { .. }) => {
                stacks[voice_index].pop();
            }
            _ => {
                let start = voice_clocks[voice_index];
                let ratio = tuplet_ratio(&stacks[voice_index]);
                voice_clocks[voice_index] += visit_pitches(event, scale, ratio, |pitch, display, grace| {
                    pitches.push((start, grace, *voice, pitch, display));
                });
            }
        }
    }

    let mut order: Vec<usize> = (0..pitches.len()).collect();
    order.sort_unstable_by_key(|&index| {
        let (onset, grace, voice, _, _) = pitches[index];
        (onset, !grace, voice, index)
    });
    let mut seen = AccidentalTracker::new();
    let mut resolved = vec![None; pitches.len()];
    for index in order {
        let (_, _, _, pitch, display) = &pitches[index];
        resolved[index] = display.and_then(|display| {
            resolve_and_track(pitch, key_sig, display, policy, Some(&mut seen))
        });
    }
    resolved
}

/// Resolve one event's accidentals in its own member order against `seen`
/// (or the key signature alone when `None`) under the default policy, then
/// convert it.
#[cfg(test)]
pub(crate) fn convert_event(
    event: &ScoreEvent,
    clef: &Clef,
    key_sig: &KeySignature,
    mut seen: Option<&mut AccidentalTracker>,
) -> MeasureEvent {
    let mut resolved = Vec::new();
    visit_pitches(event, 1, (1, 1), |pitch, display, _| {
        resolved.push(display.and_then(|display| {
            resolve_and_track(&pitch, key_sig, display, AccidentalPolicy::Default, seen.as_deref_mut())
        }));
    });
    convert_resolved_event(event, clef, &mut resolved.iter())
}
