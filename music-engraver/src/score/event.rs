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
use crate::layout::measure::{BeamGroupEvent, ChordEvent, NoteEvent, RestEvent, TupletGroupEvent};
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

/// Convert a `ScoreEvent` into a `MeasureEvent` for the layout engine.
/// Does not track within-measure accidental state (each note resolved independently).
#[cfg(test)]
pub(crate) fn convert_event(
    event: &ScoreEvent,
    clef: &Clef,
    key_sig: &KeySignature,
) -> MeasureEvent {
    match event {
        ScoreEvent::Note { pitch, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
            let staff_pos = pitch_to_staff_position(pitch, clef);
            let log2 = duration_kind_to_log2(duration.kind());
            let dots = duration.num_dots();
            let acc = should_show_accidental(pitch, key_sig);

            MeasureEvent::Note(NoteEvent {
                staff_position: staff_pos,
                duration_log2: log2,
                dots,
                accidental: acc,
                stem_direction: None,
                tie_forward: *tie_forward,
                dynamic: *dynamic,
                slur_start: *slur_start,
                slur_end: *slur_end,
                hairpin_start: *hairpin_start,
                hairpin_end: *hairpin_end,
                rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
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
                .map(|p| should_show_accidental(p, key_sig))
                .collect();

            MeasureEvent::Chord(ChordEvent {
                staff_positions,
                duration_log2: log2,
                dots,
                accidentals,
                stem_direction: None,
                tie_forward: *tie_forward,
                dynamic: *dynamic,
                slur_start: *slur_start,
                slur_end: *slur_end,
                hairpin_start: *hairpin_start,
                hairpin_end: *hairpin_end,
                rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
            })
        }
        ScoreEvent::BeamGroup { notes } => {
            let note_events: Vec<NoteEvent> = notes
                .iter()
                .map(|(pitch, duration)| {
                    let staff_pos = pitch_to_staff_position(pitch, clef);
                    let log2 = duration_kind_to_log2(duration.kind());
                    let dots = duration.num_dots();
                    let acc = should_show_accidental(pitch, key_sig);
                    NoteEvent {
                        staff_position: staff_pos,
                        duration_log2: log2,
                        dots,
                        accidental: acc,
                        stem_direction: None,
                        tie_forward: false,
                        dynamic: None,
                        slur_start: false,
                        slur_end: false,
                        hairpin_start: None,
                        hairpin_end: false,
                        rehearsal_mark: None, tempo_mark: None, expression: None,
                    }
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
                    let staff_pos = pitch_to_staff_position(pitch, clef);
                    let log2 = duration_kind_to_log2(duration.kind());
                    let dots = duration.num_dots();
                    let acc = should_show_accidental(pitch, key_sig);
                    NoteEvent {
                        staff_position: staff_pos,
                        duration_log2: log2,
                        dots,
                        accidental: acc,
                        stem_direction: None,
                        tie_forward: false,
                        dynamic: None,
                        slur_start: false,
                        slur_end: false,
                        hairpin_start: None,
                        hairpin_end: false,
                        rehearsal_mark: None, tempo_mark: None, expression: None,
                    }
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

/// Convert a `ScoreEvent` into a `MeasureEvent`, tracking accidentals within the measure.
///
/// `seen` maps note identity (letter+octave) to the accidental last displayed for that
/// note in this measure. Suppresses redundant accidentals and shows courtesy naturals.
/// Resets at each measure boundary (caller provides a fresh map per measure).
pub(crate) fn convert_event_tracked(
    event: &ScoreEvent,
    clef: &Clef,
    key_sig: &KeySignature,
    seen: &mut AccidentalTracker,
) -> MeasureEvent {
    match event {
        ScoreEvent::Note { pitch, duration, tie_forward, dynamic, slur_start, slur_end, hairpin_start, hairpin_end, rehearsal_mark, tempo_mark, expression } => {
            let staff_pos = pitch_to_staff_position(pitch, clef);
            let log2 = duration_kind_to_log2(duration.kind());
            let dots = duration.num_dots();
            let acc = resolve_accidental(pitch, key_sig, Some(seen));

            // Update tracking: record what accidental state this note establishes
            let key = note_key(pitch);
            if let Some(eff) = effective_accidental(pitch, key_sig) {
                seen.insert(key, eff);
            } else {
                seen.remove(&key);
            }

            MeasureEvent::Note(NoteEvent {
                staff_position: staff_pos,
                duration_log2: log2,
                dots,
                accidental: acc,
                stem_direction: None,
                tie_forward: *tie_forward,
                dynamic: *dynamic,
                slur_start: *slur_start,
                slur_end: *slur_end,
                hairpin_start: *hairpin_start,
                hairpin_end: *hairpin_end,
                rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
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
                .map(|p| {
                    let acc = resolve_accidental(p, key_sig, Some(seen));
                    // Update tracking for each note in the chord
                    let key = note_key(p);
                    if let Some(eff) = effective_accidental(p, key_sig) {
                        seen.insert(key, eff);
                    } else {
                        seen.remove(&key);
                    }
                    acc
                })
                .collect();

            MeasureEvent::Chord(ChordEvent {
                staff_positions,
                duration_log2: log2,
                dots,
                accidentals,
                stem_direction: None,
                tie_forward: *tie_forward,
                dynamic: *dynamic,
                slur_start: *slur_start,
                slur_end: *slur_end,
                hairpin_start: *hairpin_start,
                hairpin_end: *hairpin_end,
                rehearsal_mark: rehearsal_mark.clone(), tempo_mark: tempo_mark.clone(), expression: expression.clone(),
            })
        }
        ScoreEvent::BeamGroup { notes } => {
            let note_events: Vec<NoteEvent> = notes
                .iter()
                .map(|(pitch, duration)| {
                    let staff_pos = pitch_to_staff_position(pitch, clef);
                    let log2 = duration_kind_to_log2(duration.kind());
                    let dots = duration.num_dots();
                    let acc = resolve_accidental(pitch, key_sig, Some(seen));

                    // Update tracking
                    let key = note_key(pitch);
                    if let Some(eff) = effective_accidental(pitch, key_sig) {
                        seen.insert(key, eff);
                    } else {
                        seen.remove(&key);
                    }

                    NoteEvent {
                        staff_position: staff_pos,
                        duration_log2: log2,
                        dots,
                        accidental: acc,
                        stem_direction: None,
                        tie_forward: false,
                        dynamic: None,
                        slur_start: false,
                        slur_end: false,
                        hairpin_start: None,
                        hairpin_end: false,
                        rehearsal_mark: None, tempo_mark: None, expression: None,
                    }
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
                    let staff_pos = pitch_to_staff_position(pitch, clef);
                    let log2 = duration_kind_to_log2(duration.kind());
                    let dots = duration.num_dots();
                    let acc = resolve_accidental(pitch, key_sig, Some(seen));

                    let key = note_key(pitch);
                    if let Some(eff) = effective_accidental(pitch, key_sig) {
                        seen.insert(key, eff);
                    } else {
                        seen.remove(&key);
                    }

                    NoteEvent {
                        staff_position: staff_pos,
                        duration_log2: log2,
                        dots,
                        accidental: acc,
                        stem_direction: None,
                        tie_forward: false,
                        dynamic: None,
                        slur_start: false,
                        slur_end: false,
                        hairpin_start: None,
                        hairpin_end: false,
                        rehearsal_mark: None, tempo_mark: None, expression: None,
                    }
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
