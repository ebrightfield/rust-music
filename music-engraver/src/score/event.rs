//! Score event types and conversion from `music` crate types to layout events.
//!
//! Handles accidental resolution against key signatures, within-measure
//! accidental tracking (courtesy naturals, suppression of redundant accidentals),
//! and conversion of `Pitch`/`Duration` to the internal layout representation.

use std::collections::HashMap;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::pitch::Pitch;
use music::note::spelling::{Accidental, Spelling};

use crate::layout::accidental::accidental_glyph;
use crate::layout::dynamics::Dynamic;
use crate::layout::hairpin::HairpinType;
use crate::layout::key_signature::KeySignature;
use crate::layout::measure::{BeamGroupEvent, ChordEvent, NoteAnnotations, NoteEvent, RestEvent, TupletGroupEvent};
use crate::layout::note_placement::pitch_to_staff_position;
use crate::layout::rehearsal::RehearsalStyle;
use crate::layout::system::MeasureEvent;
use crate::layout::tempo::TempoMark;

/// An event being accumulated in the current measure.
#[derive(Clone, Debug)]
pub(crate) enum ScoreEvent {
    Note {
        pitch: Pitch,
        duration: Duration,
        tie_forward: bool,
        dynamic: Option<Dynamic>,
        slur_start: bool,
        slur_end: bool,
        hairpin_start: Option<HairpinType>,
        hairpin_end: bool,
        rehearsal_mark: Option<(String, RehearsalStyle)>,
        tempo_mark: Option<TempoMark>,
        expression: Option<String>,
    },
    Rest {
        duration: Duration,
    },
    Chord {
        pitches: Vec<Pitch>,
        duration: Duration,
        tie_forward: bool,
        dynamic: Option<Dynamic>,
        slur_start: bool,
        slur_end: bool,
        hairpin_start: Option<HairpinType>,
        hairpin_end: bool,
        rehearsal_mark: Option<(String, RehearsalStyle)>,
        tempo_mark: Option<TempoMark>,
        expression: Option<String>,
    },
    BeamGroup {
        notes: Vec<(Pitch, Duration)>,
    },
    TupletGroup {
        notes: Vec<(Pitch, Duration)>,
        tuplet_number: u32,
    },
}

/// Convert a `DurationKind` to the log2 representation used by the layout engine.
///
/// Layout uses: 0=whole, 1=half, 2=quarter, 3=eighth, etc.
/// Breve (double whole) maps to 0 as well since the layout engine doesn't
/// distinguish breves from wholes for spacing purposes.
pub(crate) fn duration_kind_to_log2(kind: DurationKind) -> u8 {
    match kind {
        DurationKind::Breve => 0,
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
type NoteKey = (i32, u8);

/// Map tracking which accidental was last shown for each note (letter+octave) in a measure.
pub(crate) type AccidentalTracker = HashMap<NoteKey, Accidental>;

pub(crate) fn note_key(pitch: &Pitch) -> NoteKey {
    let spelling = Spelling::from(&pitch.note);
    (i32::from(&spelling.letter), pitch.octave)
}

/// The effective accidental state for a note: what accidental applies to this letter+octave.
///
/// For tracking purposes: Natural on an unaltered note = None (no accidental in effect),
/// Natural on an altered note = Natural (cancelling), Sharp/Flat/Double = themselves.
pub(crate) fn effective_accidental(pitch: &Pitch, key_sig: &KeySignature) -> Option<Accidental> {
    let spelling = Spelling::from(&pitch.note);
    let acc = spelling.acc;
    let altered = note_altered_in_key(spelling.letter, key_sig);

    match acc {
        Accidental::Natural => {
            if altered {
                Some(Accidental::Natural)
            } else {
                None
            }
        }
        _ => Some(acc),
    }
}

/// Resolve whether an accidental should be displayed for a given pitch in a key signature.
///
/// Suppresses accidentals that are redundant with the key signature (e.g., F# in D major).
/// Shows naturals that cancel key-signature alterations (e.g., F♮ in D major).
/// Double sharps/flats are always shown since they never appear in key signatures.
/// Does not track within-measure accidental state — each note is resolved independently.
#[cfg(test)]
pub(crate) fn should_show_accidental(pitch: &Pitch, key_sig: &KeySignature) -> Option<smufl::Glyph> {
    resolve_accidental(pitch, key_sig, None)
}

/// Resolve whether an accidental should be displayed, with optional within-measure tracking.
///
/// `seen_in_measure`: if Some, maps note identity (letter+octave) to the last accidental
/// shown for that note in this measure. Suppresses repeated accidentals and shows courtesy
/// naturals when a previous accidental in the measure is cancelled.
pub(crate) fn resolve_accidental(
    pitch: &Pitch,
    key_sig: &KeySignature,
    seen_in_measure: Option<&AccidentalTracker>,
) -> Option<smufl::Glyph> {
    let spelling = Spelling::from(&pitch.note);
    let acc = spelling.acc;
    let altered = note_altered_in_key(spelling.letter, key_sig);
    let key = note_key(pitch);

    // Check within-measure tracking
    if let Some(seen) = seen_in_measure {
        if let Some(&prev_acc) = seen.get(&key) {
            let current_effective = effective_accidental(pitch, key_sig);
            if current_effective == Some(prev_acc) {
                // Same accidental already displayed — suppress
                return None;
            }
            // Different accidental — show it, including naturals cancelling
            // a previous accidental shown within this measure
            if acc == Accidental::Natural {
                return accidental_glyph(Accidental::Natural, true);
            }
        }
    }

    match acc {
        Accidental::Natural => {
            if altered {
                accidental_glyph(Accidental::Natural, true)
            } else {
                // Courtesy natural: if a previous note in this measure had an accidental
                // on the same letter+octave, show a natural to clarify
                if let Some(seen) = seen_in_measure {
                    if seen.contains_key(&key) {
                        return accidental_glyph(Accidental::Natural, true);
                    }
                }
                None
            }
        }
        Accidental::Sharp => {
            if altered && matches!(key_sig, KeySignature::Sharps(_)) {
                None
            } else {
                accidental_glyph(Accidental::Sharp, false)
            }
        }
        Accidental::Flat => {
            if altered && matches!(key_sig, KeySignature::Flats(_)) {
                None
            } else {
                accidental_glyph(Accidental::Flat, false)
            }
        }
        Accidental::DoubleSharp | Accidental::DoubleFlat => accidental_glyph(acc, false),
    }
}

/// Check whether a letter name is altered (has a sharp or flat) in the given key signature.
pub(crate) fn note_altered_in_key(letter: music::note::spelling::Letter, key_sig: &KeySignature) -> bool {
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

/// Resolve an accidental and update the within-measure tracker if present.
fn resolve_and_track(
    pitch: &Pitch,
    key_sig: &KeySignature,
    seen: Option<&mut AccidentalTracker>,
) -> Option<smufl::Glyph> {
    let acc = resolve_accidental(pitch, key_sig, seen.as_deref());
    if let Some(seen) = seen {
        let key = note_key(pitch);
        if let Some(eff) = effective_accidental(pitch, key_sig) {
            seen.insert(key, eff);
        } else {
            seen.remove(&key);
        }
    }
    acc
}

/// Convert a single pitch+duration into a `NoteEvent` for beam/tuplet groups.
fn pitch_to_note_event(
    pitch: &Pitch,
    duration: &Duration,
    clef: &Clef,
    key_sig: &KeySignature,
    seen: Option<&mut AccidentalTracker>,
) -> NoteEvent {
    let staff_pos = pitch_to_staff_position(pitch, clef);
    let log2 = duration_kind_to_log2(duration.kind());
    let dots = duration.num_dots();
    let acc = resolve_and_track(pitch, key_sig, seen);
    NoteEvent {
        staff_position: staff_pos,
        duration_log2: log2,
        dots,
        accidental: acc,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    }
}

/// Convert a `ScoreEvent` into a `MeasureEvent` for the layout engine.
///
/// When `seen` is `Some`, tracks accidentals within the measure: suppresses
/// redundant accidentals and shows courtesy naturals. Caller provides a fresh
/// map per measure; it resets at each measure boundary.
///
/// When `seen` is `None`, each note is resolved independently against the key
/// signature without within-measure tracking.
pub(crate) fn convert_event(
    event: &ScoreEvent,
    clef: &Clef,
    key_sig: &KeySignature,
    mut seen: Option<&mut AccidentalTracker>,
) -> MeasureEvent {
    match event {
        ScoreEvent::Note { pitch, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
            let staff_pos = pitch_to_staff_position(pitch, clef);
            let log2 = duration_kind_to_log2(duration.kind());
            let dots = duration.num_dots();
            let acc = resolve_and_track(pitch, key_sig, seen);

            MeasureEvent::Note(NoteEvent {
                staff_position: staff_pos,
                duration_log2: log2,
                dots,
                accidental: acc,
                stem_direction: None,
                annotations: NoteAnnotations { tie_forward: *tie_forward, dynamic: *dynamic, slur_start: *slur_start, slur_end: *slur_end, hairpin_start: *hairpin_start, hairpin_end: *hairpin_end, rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone() },
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
        ScoreEvent::Chord { pitches, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
            let log2 = duration_kind_to_log2(duration.kind());
            let dots = duration.num_dots();
            let staff_positions: Vec<i8> = pitches
                .iter()
                .map(|p| pitch_to_staff_position(p, clef))
                .collect();
            let accidentals: Vec<Option<smufl::Glyph>> = pitches
                .iter()
                .map(|p| resolve_and_track(p, key_sig, seen.as_deref_mut()))
                .collect();

            MeasureEvent::Chord(ChordEvent {
                staff_positions,
                duration_log2: log2,
                dots,
                accidentals,
                stem_direction: None,
                annotations: NoteAnnotations { tie_forward: *tie_forward, dynamic: *dynamic, slur_start: *slur_start, slur_end: *slur_end, hairpin_start: *hairpin_start, hairpin_end: *hairpin_end, rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone() },
            })
        }
        ScoreEvent::BeamGroup { notes } => {
            let note_events: Vec<NoteEvent> = notes
                .iter()
                .map(|(pitch, duration)| {
                    pitch_to_note_event(pitch, duration, clef, key_sig, seen.as_deref_mut())
                })
                .collect();
            MeasureEvent::BeamGroup(BeamGroupEvent {
                notes: note_events,
                stem_direction: None,
            })
        }
        ScoreEvent::TupletGroup { notes, tuplet_number } => {
            let note_events: Vec<NoteEvent> = notes
                .iter()
                .map(|(pitch, duration)| {
                    pitch_to_note_event(pitch, duration, clef, key_sig, seen.as_deref_mut())
                })
                .collect();
            MeasureEvent::TupletGroup(TupletGroupEvent {
                beam_group: BeamGroupEvent {
                    notes: note_events,
                    stem_direction: None,
                },
                tuplet_number: *tuplet_number,
            })
        }
    }
}
