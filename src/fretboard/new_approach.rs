use anyhow::{anyhow, Result};
use crate::fretboard::{Fretboard, FrettedNote, FrettedNoteKind};
use crate::note::note::Note;
use crate::note::pc::Pc;
use crate::pitch::Pitch;

#[derive(Debug)]
pub enum NewFrettedNote<'a> {
    Sounded(SoundedNote<'a>),
    Muted {
        string: u8,
        fretboard: &'a NewFretboard,
    },
}

#[derive(Debug)]
pub struct SoundedNote<'a> {
    pub string: u8,
    pub fret: u8,
    pub pitch: Pitch,
    pub fretboard: &'a NewFretboard,
}

impl<'a> SoundedNote<'a> {
    pub fn open(string: u8, fretboard: &'a NewFretboard) -> Result<Self> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self {
            fret: 0,
            pitch: open_string.clone(),
            string,
            fretboard,
        })
    }

    pub fn fretted(string: u8, fret: u8, fretboard: &'a NewFretboard) -> Result<Self> {
        Ok(fretboard.at(string, fret)?)
    }

    pub fn up_n_frets(&self, n: u8) -> Result<Self> {
        Ok(self.fretboard.at(self.string, self.fret + n)?)
    }
}

/// Represents a fretboard with any arbitrary tuning or number of strings.
#[derive(Clone, Debug)]
pub struct NewFretboard {
    /// The number and tuning of a fretboard is entirely defined here.
    /// Canonically, we use `open_strings[0]` to represent the thickest string
    /// on an instrument.
    pub open_strings: Vec<Pitch>,
}

impl NewFretboard {
    pub fn num_strings(&self) -> u8 {
        u8::try_from(self.open_strings.len()).unwrap()
    }

    pub fn get_string(&self, string: u8) -> Result<&Pitch> {
        Ok(self.open_strings.get(string as usize)
            .ok_or(anyhow!("String too high for fretboard {:?}", self))?
        )
    }

    pub fn at(&self, string: u8, fret: u8) -> Result<SoundedNote> {
        if fret > 35 {
            return Err(anyhow!("Excessively high fret number"));
        }
        let open_string = self.get_string(string)?;
        if fret == 0 {
            return Ok(SoundedNote {
                string,
                fret: 0,
                pitch: open_string.clone(),
                fretboard: self,
            });
        }
        let pitch =
            open_string.at_distance_from(fret as isize)?;
        return Ok(SoundedNote {
            string,
            fret: fret,
            pitch: pitch,
            fretboard: self,
        });
    }

    /// Returns the first available fret on a given string that equals a given [Note].
    pub fn note_on_string(&self, note: &Note, string: u8) -> Result<SoundedNote> {
        let fret = self.which_fret(note, string)?;
        Ok(self.at(string, fret)?)
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

impl<'a> NewFrettedNote<'a> {
    pub fn muted(string: u8, fretboard: &'a NewFretboard) -> Result<Self> {
        let _ = fretboard.get_string(string)?;
        Ok(Self::Muted {
            string,
            fretboard,
        })
    }

    pub fn open(string: u8, fretboard: &'a NewFretboard) -> Result<Self> {
        let open_string = fretboard.get_string(string)?;
        Ok(Self::Sounded(SoundedNote {
            pitch: open_string.clone(),
            string,
            fretboard,
            fret: 0,
        }))
    }

    /// Construct a [FrettedNoteKind::Fretted] variant.
    pub fn fretted(string: u8, fret: u8, fretboard: &'a NewFretboard) -> Result<Self> {
        Ok(Self::Sounded(fretboard.at(string, fret)?))
    }

    fn string(&self) -> u8 {
        match &self {
            NewFrettedNote::Sounded(SoundedNote { string, ..}) => *string,
            NewFrettedNote::Muted { string, .. } => *string,
        }
    }

    /// Returns the fret, unless it's a Muted variant.
    fn fret(&self) -> Option<u8> {
        match &self {
            NewFrettedNote::Sounded(SoundedNote { fret, ..}) => Some(*fret),
            NewFrettedNote::Muted { .. } => None
        }
    }

    /// Returns the pitch, unless it's a Muted variant.
    fn pitch(&self) -> Option<Pitch> {
        match &self {
            NewFrettedNote::Sounded(SoundedNote { pitch, ..}) => Some(pitch.clone()),
            NewFrettedNote::Muted { .. } => None
        }
    }

    fn up_n_frets(&self, n: u8) -> Result<Option<Self>> {
        match &self {
            NewFrettedNote::Sounded(sounded_note) => Ok(Some(
                NewFrettedNote::Sounded(sounded_note.up_n_frets(n)?)
            )),
            NewFrettedNote::Muted { .. } => Ok(None),
        }
    }
}
