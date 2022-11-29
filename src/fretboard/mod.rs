pub mod fretboard_shape;
pub mod new_approach;
pub mod new_fretboard_shape;

use std::ops::Deref;
use anyhow::{anyhow, Result};
use crate::note::note::Note;
use crate::note::pc::Pc;
use crate::pitch::Pitch;

#[derive(Clone, Debug)]
pub struct OpenString(Pitch);

impl Deref for OpenString {
    type Target = Pitch;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<Pitch> for OpenString {
    fn into(self) -> Pitch {
        self.0.clone()
    }
}

impl Into<Pitch> for &OpenString {
    fn into(self) -> Pitch {
        self.0.clone()
    }
}

impl From<&Pitch> for OpenString {
    fn from(value: &Pitch) -> Self {
        Self(value.clone())
    }
}

/// Represents a fretboard with any arbitrary tuning or number of strings.
#[derive(Clone, Debug)]
pub struct Fretboard {
    /// The number and tuning of a fretboard is entirely defined here.
    /// Canonically, we use `open_strings[0]` to represent the thickest string
    /// on an instrument.
    pub open_strings: Vec<OpenString>,
}

impl Fretboard {
    pub fn num_strings(&self) -> u8 {
        u8::try_from(self.open_strings.len()).unwrap()
    }

    pub fn get_string(&self, string: u8) -> Result<&OpenString> {
        Ok(self.open_strings.get(string as usize)
            .ok_or(anyhow!("String too high for fretboard {:?}", self))?
        )
    }

    pub fn at(&self, string: u8, fret: Option<u8>) -> Result<FrettedNote> {
        if let Some(fret) = fret {
            if fret > 35 {
                return Err(anyhow!("Excessively high fret number"));
            }
            let open_string = self.get_string(string)?;
            if fret == 0 {
                return Ok(FrettedNote {
                    kind: FrettedNoteKind::Open(open_string.clone()),
                    string,
                    fretboard: self,
                });
            }
            let pitch =
                open_string.at_distance_from(fret as isize)?;
            return Ok(FrettedNote {
                kind: FrettedNoteKind::Fretted { fret, pitch },
                string,
                fretboard: self,
            });
        }
        Ok(FrettedNote {
            kind: FrettedNoteKind::Muted,
            string,
            fretboard: self,
        })
    }

    /// Returns the first available fret on a given string that equals a given [Note].
    pub fn note_on_string(&self, note: &Note, string: u8) -> Result<FrettedNote> {
        let fret = self.which_fret(note, string)?;
        Ok(self.at(string, Some(fret))?)
    }

    /// At which fret you will find a target note on a target string.
    /// e.g. "where is F# on the 3rd string of this fretboard?"
    pub fn which_fret(&self, note: &Note, string: u8) -> Result<u8> {
        let open_string = self.get_string(string)?;
        let mut pc = Pc::from(open_string.note);
        let fretted_pc = Pc::from(note);
        for i in 0..12 {
            if pc == fretted_pc {
                return Ok(i);
            }
            pc = pc.next();
        }
        Err(anyhow!("Unreachable: Should reach target note in at most twelve steps"))
    }
}

/// This essentially enhances the limitation of using [Option<Pitch>] in a [FrettedNote],
/// offering three variants, two of which contain a [Pitch], and one of which also
/// needs to contain a fret number.
#[derive(Clone, Debug)]
pub enum FrettedNoteKind {
    Open(OpenString),
    Fretted {
        fret: u8,
        pitch: Pitch,
    },
    Muted,
}

#[derive(Debug)]
pub struct FrettedNote<'a> {
    kind: FrettedNoteKind,
    string: u8,
    fretboard: &'a Fretboard,
}

impl<'a> FrettedNote<'a> {
    pub fn muted(string: u8, fretboard: &'a Fretboard) -> Result<Self> {
        let _ = fretboard.get_string(string)?;
        Ok(Self {
            kind: FrettedNoteKind::Muted,
            string,
            fretboard,
        })
    }

    pub fn open(string: u8, fretboard: &'a Fretboard) -> Result<Self> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self {
            kind: FrettedNoteKind::Open(open_string.clone()),
            string,
            fretboard,
        })
    }

    /// Construct a [FrettedNoteKind::Fretted] variant.
    pub fn fretted(string: u8, fret: u8, fretboard: &'a Fretboard) -> Result<Self> {
        Ok(fretboard.at(string, Some(fret))?)
    }

    /// Returns the fret, unless it's a Muted variant.
    fn fret(&self) -> Option<u8> {
        match &self.kind {
            FrettedNoteKind::Open(_) => Some(0),
            FrettedNoteKind::Fretted { fret, .. } => Some(*fret),
            FrettedNoteKind::Muted => None,
        }
    }

    /// Returns the pitch, unless it's a Muted variant.
    fn pitch(&self) -> Option<Pitch> {
        match &self.kind {
            FrettedNoteKind::Open(s) => Some(s.0.clone()),
            FrettedNoteKind::Fretted { pitch, .. } => Some(pitch.clone()),
            FrettedNoteKind::Muted => None,
        }
    }

    fn up_n_frets(&self, n: u8) -> Option<Self> {
        match &self.kind {
            FrettedNoteKind::Open(open_string) =>
                self.fretboard.at(self.string, Some(n)).ok(),
            FrettedNoteKind::Fretted { fret, pitch } =>
                self.fretboard.at(self.string, Some(fret + n)).ok(),
            FrettedNoteKind::Muted =>
                None,
        }
    }
}