use thiserror::Error;
use crate::note::note::Note;
use crate::note::pc::Pc;
use crate::note::pitch::Pitch;
use crate::note_collections::interval_class::IntervalClass;

#[derive(Debug, Clone, Error)]
pub enum MusicTheoryError {
    #[error("The midi note {0} is too high.")]
    MidiTooHigh(u8),
    #[error("The fret {0} is too high.")]
    FretTooHigh(u8),
    #[error("Octave {0} is too high.")]
    OctaveTooHigh(u8),
    #[error("Empty collection of notes, pitch classes, or similar kind of collection")]
    EmptySetOfNotes,
    #[error("The note {0} is not a member of the collection {1:?}")]
    NotAMember(Note, Vec<Note>),
    #[error("The pitch {0} is lower than the next string {1}")]
    FretBelowZero(Pitch, Pitch),
    #[error("Invalid or excessive accidental: {0}")]
    InvalidAccidental(String),
    #[error("Invalid note letter: {0}")]
    InvalidNoteLetter(String),
    #[error("Cannot use double accidental as root: {0}")]
    NoDoubleAccidentalRoot(Note),
    #[error("Invalid octave partition, does not wrap around: {0:?}")]
    InvalidOctavePartition(Vec<IntervalClass>),
    /// This variant should never be seen by a user.
    #[error("Not a proper Pc for an alteration: {0:?}")]
    PcNotAnAlteration(Pc),
    #[error("A bad thing occurred that the developer didn't anticipate.")]
    Unreachable,
}