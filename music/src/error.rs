use thiserror::Error;
use crate::fretboard::Fretboard;
use crate::note::note::Note;
use crate::note::pitch::Pitch;
use crate::note::spelling::{Accidental, Letter};
use crate::note_collections::interval_class::IntervalClass;
use crate::note_collections::PcSet;

#[derive(Debug, Clone, Error)]
pub enum MusicSemanticsError {
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
    #[error("The current fretted note's fret is less than {0}")]
    CantMoveDownFrets(u8),
    #[error("Index {0} too high for strings on fretboard {1:?}")]
    StringTooHighForFretboard(u8, Fretboard),
    #[error("Excessive accidental: cannot spell {0} with {1}")]
    ExcessiveAccidental(Letter, Accidental),
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
    PcNotAnAlteration(usize),
    #[error("A bad thing occurred that the developer didn't anticipate.")]
    Unreachable,
    #[error("Moving midi_note {0} down an octave would put it below midi_note zero.")]
    OutOfBoundsLower(u8),
    #[error("Moving midi_note {0} up an octave would put it above midi_note bounds.")]
    OutOfBoundsUpper(u8),
    #[error("Not a valid choice for N notes per string: {0:?}")]
    InvalidNNotesPerString((usize, usize)),
    #[error("Size {0} too large for subchords on {1:?}")]
    SizeTooLargeForSubchords(u8, PcSet),
    #[error("Size {0} too small for chords")]
    SizeTooSmallForChords(usize),
}