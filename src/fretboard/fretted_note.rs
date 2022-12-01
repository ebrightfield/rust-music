use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use crate::fretboard::Fretboard;
use crate::note::pitch::Pitch;
use crate::note_collections::NoteSet;

/// Useful for times when we want to be able to mark a string as muted.
#[derive(Debug, Clone)]
pub enum FrettedNote<'a> {
    /// A note that is played on the fretboard.
    Sounded(SoundedNote<'a>),
    /// Denotes a muted string. Usually most useful for chord diagrams.
    Muted {
        string: u8,
        fretboard: &'a Fretboard,
    },
}

/// A note played on a fretboard. A reference to the fretboard ensures
/// that each existing [FrettedNote] instance refers in code back to an
/// actual [Fretboard] instance, which is often useful for performing calculations.
#[derive(Debug, Clone)]
pub struct SoundedNote<'a> {
    pub string: u8,
    pub fret: u8,
    pub pitch: Pitch,
    pub fretboard: &'a Fretboard,
}

impl<'a> Display for SoundedNote<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}({})", self.string+1, self.fret, self.pitch.note)
    }
}

impl<'a> SoundedNote<'a> {
    /// Preferred constructor for an open string. Validates using
    /// the methods on the [Fretboard] passed in.
    pub fn open(string: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self {
            fret: Fretboard::OPEN,
            pitch: open_string.clone(),
            string,
            fretboard,
        })
    }

    /// Preferred constructor for a fretted string. Validates using
    /// the methods on the [Fretboard] passed in.
    pub fn fretted(string: u8, fret: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        Ok(fretboard.sounded_note(string, fret)?)
    }

    /// Moves up the same string to a new fret `n` semitones higher.
    pub fn up_n_frets(&self, n: u8) -> anyhow::Result<Self> {
        Ok(self.fretboard.sounded_note(self.string, self.fret + n)?)
    }

    /// Moves up the same string to a new fret `n` semitones higher.
    pub fn down_n_frets(&self, n: u8) -> anyhow::Result<Self> {
        if n < self.fret {
            return Err(anyhow!("Fret goes off the fretboard"));
        }
        Ok(self.fretboard.sounded_note(self.string, self.fret - n)?)
    }

    /// Moves up the same string to a new fret `n` semitones higher.
    pub fn up_an_octave(&self) -> anyhow::Result<Self> {
        Ok(self.fretboard.sounded_note(self.string, self.fret + 12)?)
    }

    /// Moves down the same string 12 semitones, if possible.
    pub fn down_an_octave(&self) -> anyhow::Result<Self> {
        if self.fret < 12 {
            return Err(anyhow!("Fret goes off the fretboard"));
        }
        Ok(self.fretboard.sounded_note(self.string, self.fret - 12)?)
    }

    /// Produces a [SoundedNote] on the next chord/scale degree, on the same string.
    pub fn next_note_same_string(&self, notes: &NoteSet) -> anyhow::Result<Self> {
        let next_note = notes.up_n_steps(&self.pitch.note, 1)?;
        let pitch = self.pitch.up_to_note(&next_note)?;
        let this_string = self.fretboard.get_string(self.string).unwrap();
        let fret = pitch.midi_note - this_string.midi_note;
        Ok(self.fretboard.sounded_note(self.string, fret)?)
    }

    /// Produces a [SoundedNote] on the next chord/scale degree, on the next string up.
    pub fn next_note_next_string(&self, notes: &NoteSet) -> anyhow::Result<Self> {
        let next_note = notes.up_n_steps(&self.pitch.note, 1)?;
        let pitch = self.pitch.up_to_note(&next_note)?;
        let next_string = self.fretboard.get_string(self.string + 1)?;
        if next_string.midi_note > pitch.midi_note {
            return Err(anyhow!("Pitch {:?} is lower than the next string {:?}",
                &pitch, next_string,
            ))
        }
        Ok(self.fretboard.sounded_note(
            self.string + 1, pitch.midi_note - next_string.midi_note
        )?)
    }
}

impl<'a> FrettedNote<'a> {

    /// Constructor for a [FrettedNote::Muted] variant.
    pub fn muted(string: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        let _ = fretboard.get_string(string)?;
        Ok(Self::Muted {
            string,
            fretboard,
        })
    }

    /// Constructor for a [FrettedNote::Sounded] variant of an open string.
    pub fn open(string: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self::Sounded(SoundedNote {
            pitch: open_string.clone(),
            string,
            fretboard,
            fret: Fretboard::OPEN,
        }))
    }

    /// Construct a [FrettedNote::Sounded] variant that is fretted.
    pub fn fretted(string: u8, fret: u8, fretboard: &'a Fretboard) -> anyhow::Result<Self> {
        Ok(Self::Sounded(fretboard.sounded_note(string, fret)?))
    }

    /// Returns the [string] value of either variant.
    pub fn string(&self) -> u8 {
        match &self {
            FrettedNote::Sounded(SoundedNote { string, ..}) => *string,
            FrettedNote::Muted { string, .. } => *string,
        }
    }

    /// Returns the [fret], unless it's a [FrettedNote::Muted] variant.
    pub fn fret(&self) -> Option<u8> {
        match &self {
            FrettedNote::Sounded(SoundedNote { fret, ..}) => Some(*fret),
            FrettedNote::Muted { .. } => None
        }
    }

    /// Returns the [pitch], unless it's a [FrettedNote::Muted] variant.
    pub fn pitch(&self) -> Option<Pitch> {
        match &self {
            FrettedNote::Sounded(SoundedNote { pitch, ..}) => Some(pitch.clone()),
            FrettedNote::Muted { .. } => None
        }
    }
}

impl<'a> From<SoundedNote<'a>> for FrettedNote<'a> {
    fn from(value: SoundedNote<'a>) -> Self {
        FrettedNote::Sounded(value)
    }
}