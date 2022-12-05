pub mod fretboard_shape;
pub mod fretted_note;

use std::ops::Deref;
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use crate::note::note::Note;
use crate::note::pc::Pc;
use crate::note::pitch::Pitch;

pub use fretboard_shape::{FretboardShape, ChordShapeClassification};
pub use fretted_note::{SoundedNote, FrettedNote};

// TODO Add more such common guitar tunings as a convenience.
/// Standard tuning on a 6-string guitar.
pub static STD_6STR_GTR: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::E, 3).unwrap(),
            Pitch::new(Note::A, 3).unwrap(),
            Pitch::new(Note::D, 4).unwrap(),
            Pitch::new(Note::G, 4).unwrap(),
            Pitch::new(Note::B, 4).unwrap(),
            Pitch::new(Note::E, 5).unwrap(),
        ],
    }
});

/// Represents a fretboard with any arbitrary tuning or number of strings.
#[derive(Clone, Debug)]
pub struct Fretboard {
    /// The number and tuning of a fretboard is entirely defined here.
    /// Canonically, we use `open_strings[0]` to represent the thickest string
    /// on an instrument.
    pub open_strings: Vec<Pitch>,
}

impl Fretboard {

    /// Allowing a hypothetical three-octave fretboard allows for performing various
    /// melodic/harmonic fretboard shape search patterns higher up the neck, avoiding
    /// running into the open strings
    const MAX: u8 = 35;
    const OPEN: u8 = u8::MIN;

    /// The number of strings on the fretboard.
    pub fn num_strings(&self) -> u8 {
        u8::try_from(self.open_strings.len()).unwrap()
    }

    /// Fallible indexing for an element in [self.open_strings].
    /// It is important to remember that colloquially, the thickest string on a guitar
    /// is "the 6th string", but it is indexed here as `self.open_strings[0]`.
    pub fn get_string(&self, string: u8) -> Result<&Pitch> {
        Ok(self.open_strings.get(string as usize)
            .ok_or(anyhow!("String too high for fretboard {:?}", self))?
        )
    }

    /// This is the preferred way to create a [SoundedNote] instance, because it
    /// validates the initialization parameters against [self].
    pub fn sounded_note(&self, string: u8, fret: u8) -> Result<SoundedNote> {
        if fret > Self::MAX {
            return Err(anyhow!("Excessively high fret number"));
        }
        let open_string = self.get_string(string)?;
        if fret == Self::OPEN {
            return Ok(SoundedNote {
                string,
                fret: Self::OPEN,
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

    /// Given a string and target note, returns the first available
    /// [SoundedNote] whose fret equals a given [Note].
    pub fn note_on_string(&self, note: &Note, string: u8) -> Result<SoundedNote> {
        let fret = self.which_fret(note, string)?;
        Ok(self.sounded_note(string, fret)?)
    }

    /// Returns the fret where a given [Note] resides on a given string.
    /// e.g. "where is the place I can find an F# on the 3rd string of this fretboard?"
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

impl Deref for Fretboard {
    type Target = Vec<Pitch>;

    fn deref(&self) -> &Self::Target {
        &self.open_strings
    }
}
