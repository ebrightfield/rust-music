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
use crate::layout::grace::{GraceGroup, GraceNoteEvent};
use crate::layout::key_signature::KeySignature;
use crate::layout::measure::{
    BeamGroupEvent, ChordEvent, GroupedChordMember, NoteAnnotations, NoteEvent, RestEvent,
    TupletGroupEvent,
};
use crate::layout::note_placement::pitch_to_staff_position;
use crate::layout::system::MeasureEvent;

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
    BeamGroup {
        notes: Vec<(Pitch, Duration)>,
    },
    StyledBeamGroup {
        members: Vec<(Vec<Pitch>, Duration, NoteAnnotations)>,
    },
    TupletGroup {
        notes: Vec<(Pitch, Duration)>,
        tuplet_number: u32,
        in_time_of: u32,
    },
    StyledTupletGroup {
        members: Vec<(Vec<Pitch>, Duration, NoteAnnotations)>,
        tuplet_number: u32,
        in_time_of: u32,
    },
    /// Multi-measure rest: the rendered measure consists of an H-bar (or a
    /// church-rest cluster, for small counts) with a count number indicating
    /// how many consecutive measures of rest this single measure-shaped
    /// frame represents (engraved convention for empty passages in parts).
    MultiMeasureRest {
        count: u32,
        style: crate::layout::multi_measure_rest::MultiMeasureRestStyle,
    },
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

/// Convert a single pitch+duration into a `NoteEvent` for beam/tuplet groups.
fn pitch_to_note_event(
    pitch: &Pitch,
    duration: &Duration,
    clef: &Clef,
    accidentals: &mut ResolvedAccidentals<'_>,
) -> NoteEvent {
    let staff_pos = pitch_to_staff_position(pitch, clef);
    let log2 = duration_kind_to_log2(duration.kind());
    let dots = duration.num_dots();
    let acc = next_accidental(accidentals);
    NoteEvent {
        staff_position: staff_pos,
        duration_log2: log2,
        dots,
        accidental: acc,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    }
}

/// Resolve the written grace notes of `annotations` into the layout's
/// [`GraceGroup`], taking one pre-resolved accidental per grace note.
fn resolve_grace_group(
    annotations: &NoteAnnotations,
    clef: &Clef,
    accidentals: &mut ResolvedAccidentals<'_>,
) -> Option<GraceGroup> {
    annotations.grace_notes.as_ref().map(|graces| GraceGroup {
        kind: graces.kind,
        slur: graces.slur,
        notes: graces
            .notes
            .iter()
            .map(|grace| GraceNoteEvent {
                staff_position: pitch_to_staff_position(&grace.pitch, clef),
                duration_log2: duration_kind_to_log2(grace.duration.kind()),
                dots: grace.duration.num_dots(),
                accidental: next_accidental(accidentals),
            })
            .collect(),
    })
}

/// The layout's copy of a note/chord's annotations: written grace notes are
/// resolved into [`NoteAnnotations::grace_group`] (consuming their
/// accidentals, which precede the principal's own).
fn convert_annotations(
    annotations: &NoteAnnotations,
    clef: &Clef,
    accidentals: &mut ResolvedAccidentals<'_>,
) -> NoteAnnotations {
    let mut converted = annotations.clone();
    if let Some(group) = resolve_grace_group(annotations, clef, accidentals) {
        converted.grace_group = Some(group);
        converted.grace_notes = None;
    }
    converted
}

fn pitches_to_styled_group_member(
    pitches: &[Pitch],
    duration: &Duration,
    annotations: &NoteAnnotations,
    clef: &Clef,
    accidentals: &mut ResolvedAccidentals<'_>,
) -> NoteEvent {
    let mut annotations = convert_annotations(annotations, clef, accidentals);
    let mut staff_positions = Vec::with_capacity(pitches.len());
    let mut member_accidentals = Vec::with_capacity(pitches.len());
    for pitch in pitches {
        staff_positions.push(pitch_to_staff_position(pitch, clef));
        member_accidentals.push(next_accidental(accidentals));
    }
    let staff_position = staff_positions[0];
    let accidental = member_accidentals[0];
    if pitches.len() > 1 {
        annotations.grouped_chord = Some(GroupedChordMember {
            staff_positions,
            accidentals: member_accidentals,
        });
    }
    NoteEvent {
        staff_position,
        duration_log2: duration_kind_to_log2(duration.kind()),
        dots: duration.num_dots(),
        accidental,
        stem_direction: None,
        annotations,
    }
}

/// Convert a `ScoreEvent` into a `MeasureEvent` for the layout engine.
///
/// Accidentals are not resolved here: each notated pitch takes the next entry
/// of `accidentals`, which [`resolve_measure_accidentals`] produced for the
/// whole measure in musical order.
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
                stem_direction: None,
                annotations,
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
                stem_direction: None,
                annotations,
            })
        }
        ScoreEvent::BeamGroup { notes } => {
            let note_events: Vec<NoteEvent> = notes
                .iter()
                .map(|(pitch, duration)| pitch_to_note_event(pitch, duration, clef, accidentals))
                .collect();
            MeasureEvent::BeamGroup(BeamGroupEvent {
                notes: note_events,
                stem_direction: None,
            })
        }
        ScoreEvent::StyledBeamGroup { members } => {
            let note_events = members
                .iter()
                .map(|(pitches, duration, annotations)| {
                    pitches_to_styled_group_member(
                        pitches,
                        duration,
                        annotations,
                        clef,
                        accidentals,
                    )
                })
                .collect();
            MeasureEvent::BeamGroup(BeamGroupEvent {
                notes: note_events,
                stem_direction: None,
            })
        }
        ScoreEvent::TupletGroup {
            notes,
            tuplet_number,
            in_time_of,
        } => {
            let note_events: Vec<NoteEvent> = notes
                .iter()
                .map(|(pitch, duration)| pitch_to_note_event(pitch, duration, clef, accidentals))
                .collect();
            MeasureEvent::TupletGroup(TupletGroupEvent {
                beam_group: BeamGroupEvent {
                    notes: note_events,
                    stem_direction: None,
                },
                tuplet_number: *tuplet_number,
                in_time_of: *in_time_of,
            })
        }
        ScoreEvent::StyledTupletGroup {
            members,
            tuplet_number,
            in_time_of,
        } => {
            let note_events = members
                .iter()
                .map(|(pitches, duration, annotations)| {
                    pitches_to_styled_group_member(
                        pitches,
                        duration,
                        annotations,
                        clef,
                        accidentals,
                    )
                })
                .collect();
            MeasureEvent::TupletGroup(TupletGroupEvent {
                beam_group: BeamGroupEvent {
                    notes: note_events,
                    stem_direction: None,
                },
                tuplet_number: *tuplet_number,
                in_time_of: *in_time_of,
            })
        }
        ScoreEvent::MultiMeasureRest { count, style } => MeasureEvent::MultiMeasureRest {
            count: *count,
            style: *style,
        },
    }
}

/// Time of a pitch within its measure, in 128th-note ticks multiplied by the
/// measure's [`tuplet_tick_scale`] so that every tuplet member starts on an
/// exact integer.
type Onset = u64;

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Least common multiple of the tuplet numbers among `events` (1 without
/// tuplets): scaling ticks by it makes every tuplet member's onset integral.
fn tuplet_tick_scale<'a>(events: impl IntoIterator<Item = &'a ScoreEvent>) -> u64 {
    events.into_iter().fold(1, |scale, event| match event {
        ScoreEvent::TupletGroup { tuplet_number, .. }
        | ScoreEvent::StyledTupletGroup { tuplet_number, .. }
            if *tuplet_number > 0 =>
        {
            let number = u64::from(*tuplet_number);
            scale / gcd(scale, number) * number
        }
        _ => scale,
    })
}

/// Scaled ticks a member of written `duration` advances, performed at
/// `tuplet_number : in_time_of` (`1:1` outside tuplets; a zero ratio term
/// leaves written time unscaled, as in measure layout).
fn scaled_ticks(duration: &Duration, scale: u64, tuplet_number: u32, in_time_of: u32) -> Onset {
    let ticks = duration.ticks() as u64 * scale;
    if tuplet_number == 0 || in_time_of == 0 {
        ticks
    } else {
        ticks * u64::from(in_time_of) / u64::from(tuplet_number)
    }
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

/// One notated pitch as [`visit_pitches`] reports it.
#[derive(Clone, Copy, Debug)]
struct VisitedPitch {
    /// Onset offset from the start of the event, in scaled ticks.
    offset: Onset,
    /// Whether this is a grace note, sounding just before `offset`.
    grace: bool,
    pitch: Pitch,
    /// Display policy; `None` for unpitched heads, which neither display nor
    /// track accidentals.
    display: Option<AccidentalDisplay>,
}

/// One rhythmic member of an event: its pitches, written duration, and
/// annotations (`None` for plain group members, which display `Auto`).
type Member<'a> = (&'a [Pitch], &'a Duration, Option<&'a NoteAnnotations>);

/// Visit the pitches of consecutive `members` performed at
/// `tuplet_number : in_time_of`, each member's written grace notes first;
/// returns their total scaled duration.
fn visit_members<'a>(
    members: impl Iterator<Item = Member<'a>>,
    scale: u64,
    tuplet_number: u32,
    in_time_of: u32,
    visit: &mut impl FnMut(VisitedPitch),
) -> Onset {
    let mut offset = 0;
    for (pitches, duration, annotations) in members {
        let graces = annotations
            .and_then(|annotations| annotations.grace_notes.as_ref())
            .map_or(&[][..], |graces| graces.notes.as_slice());
        for grace in graces {
            visit(VisitedPitch {
                offset,
                grace: true,
                pitch: grace.pitch,
                display: Some(grace.accidental),
            });
        }
        for (index, pitch) in pitches.iter().enumerate() {
            let display = annotations.map_or(Some(AccidentalDisplay::Auto), |annotations| {
                pitch_display(annotations, index)
            });
            visit(VisitedPitch {
                offset,
                grace: false,
                pitch: *pitch,
                display,
            });
        }
        offset += scaled_ticks(duration, scale, tuplet_number, in_time_of);
    }
    offset
}

fn plain_member((pitch, duration): &(Pitch, Duration)) -> Member<'_> {
    (std::slice::from_ref(pitch), duration, None)
}

fn styled_member(
    (pitches, duration, annotations): &(Vec<Pitch>, Duration, NoteAnnotations),
) -> Member<'_> {
    (pitches.as_slice(), duration, Some(annotations))
}

/// Visit every notated pitch of `event` in the order [`convert_resolved_event`]
/// consumes accidentals: a member's written grace notes, then its pitches
/// (chord members in input order), group members in sequence. Each visit
/// carries the pitch's onset offset from the start of the event (in ticks
/// scaled by `scale`, which must be a multiple of the event's tuplet number)
/// and its display policy.
///
/// Returns the event's duration in scaled ticks.
fn visit_pitches(event: &ScoreEvent, scale: u64, mut visit: impl FnMut(VisitedPitch)) -> Onset {
    match event {
        ScoreEvent::Note {
            pitch,
            duration,
            annotations,
        } => visit_members(
            std::iter::once((std::slice::from_ref(pitch), duration, Some(annotations))),
            scale,
            1,
            1,
            &mut visit,
        ),
        ScoreEvent::Chord {
            pitches,
            duration,
            annotations,
        } => visit_members(
            std::iter::once((pitches.as_slice(), duration, Some(annotations))),
            scale,
            1,
            1,
            &mut visit,
        ),
        ScoreEvent::Rest { duration } => scaled_ticks(duration, scale, 1, 1),
        ScoreEvent::BeamGroup { notes } => {
            visit_members(notes.iter().map(plain_member), scale, 1, 1, &mut visit)
        }
        ScoreEvent::StyledBeamGroup { members } => {
            visit_members(members.iter().map(styled_member), scale, 1, 1, &mut visit)
        }
        ScoreEvent::TupletGroup {
            notes,
            tuplet_number,
            in_time_of,
        } => visit_members(
            notes.iter().map(plain_member),
            scale,
            *tuplet_number,
            *in_time_of,
            &mut visit,
        ),
        ScoreEvent::StyledTupletGroup {
            members,
            tuplet_number,
            in_time_of,
        } => visit_members(
            members.iter().map(styled_member),
            scale,
            *tuplet_number,
            *in_time_of,
            &mut visit,
        ),
        ScoreEvent::MultiMeasureRest { .. } => 0,
    }
}

/// Resolve every notated pitch's accidental in one measure of one staff, in
/// musical rather than builder order, under the staff's [`AccidentalPolicy`].
///
/// Accidental state is staff-wide: every voice reads and writes one tracker,
/// which starts empty (key signature only) at each barline. Each voice keeps
/// its own clock from the measure start, advancing by written durations
/// (dots included, tuplet members scaled by their ratio). Pitches are then
/// resolved in order of onset; at one onset grace notes (which sound just
/// before it) come first, then ties are broken by voice index ascending and
/// then by builder order within the voice (grace notes in written order,
/// chord members in input order). Entering voices in a different builder
/// order therefore cannot change the result.
///
/// Returns one entry per notated pitch in builder order, ready for
/// [`convert_resolved_event`].
pub(crate) fn resolve_measure_accidentals(
    voiced_events: &[(u8, ScoreEvent)],
    key_sig: &KeySignature,
    policy: AccidentalPolicy,
) -> Vec<Option<ResolvedAccidental>> {
    let scale = tuplet_tick_scale(voiced_events.iter().map(|(_, event)| event));
    let mut voice_clocks: Vec<Onset> = Vec::new();
    let mut pitches: Vec<(Onset, u8, VisitedPitch)> = Vec::new();
    for (voice, event) in voiced_events {
        let voice_index = usize::from(*voice);
        if voice_clocks.len() <= voice_index {
            voice_clocks.resize(voice_index + 1, 0);
        }
        let start = voice_clocks[voice_index];
        voice_clocks[voice_index] += visit_pitches(event, scale, |visited| {
            pitches.push((start + visited.offset, *voice, visited));
        });
    }

    let mut order: Vec<usize> = (0..pitches.len()).collect();
    order.sort_unstable_by_key(|&index| {
        let (onset, voice, visited) = pitches[index];
        (onset, !visited.grace, voice, index)
    });
    let mut seen = AccidentalTracker::new();
    let mut resolved = vec![None; pitches.len()];
    for index in order {
        let (_, _, visited) = &pitches[index];
        resolved[index] = visited.display.and_then(|display| {
            resolve_and_track(&visited.pitch, key_sig, display, policy, Some(&mut seen))
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
    visit_pitches(event, tuplet_tick_scale(std::iter::once(event)), |visited| {
        resolved.push(visited.display.and_then(|display| {
            resolve_and_track(
                &visited.pitch,
                key_sig,
                display,
                AccidentalPolicy::Default,
                seen.as_deref_mut(),
            )
        }));
    });
    convert_resolved_event(event, clef, &mut resolved.iter())
}
