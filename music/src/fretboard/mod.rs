pub mod fretboard_shape;
pub mod fretted_note;

use std::ops::Deref;
use once_cell::sync::Lazy;
use crate::note::note::Note;
use crate::note::pitch_class::Pc;
use crate::note::pitch::Pitch;

pub use fretboard_shape::{FretboardShape, ChordShapeClassification};
pub use fretted_note::{SoundedNote, FrettedNote};
use crate::error::MusicSemanticsError;
// StringConvention is defined in this file and exported directly

/// String numbering conventions used in different contexts.
///
/// Different notation systems and software use different conventions for numbering guitar strings:
/// - Internally, this library uses 0-indexed from the lowest (thickest) string
/// - Lilypond/TAB notation uses 1-indexed from the highest (thinnest) string
/// - Some systems use 1-indexed from the lowest string
///
/// This enum provides a way to convert between these conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringConvention {
    /// Internal representation: 0 = lowest pitch string (thickest on guitar).
    /// This is the default convention used throughout this library.
    ZeroIndexedFromLow,
    /// Standard TAB/Lilypond convention: 1 = highest pitch string (thinnest on guitar).
    /// This matches what players typically see in sheet music and tablature.
    OneIndexedFromHigh,
    /// Alternative convention: 1 = lowest pitch string (thickest on guitar).
    /// Used by some software and notation systems.
    OneIndexedFromLow,
}

/// Standard tuning on a 6-string guitar (E A D G B E).
pub static STD_6STR_GTR: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::E, 3),
            Pitch::new(Note::A, 3),
            Pitch::new(Note::D, 4),
            Pitch::new(Note::G, 4),
            Pitch::new(Note::B, 4),
            Pitch::new(Note::E, 5),
        ],
    }
});

/// Drop D tuning on a 6-string guitar (D A D G B E).
pub static DROP_D: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::D, 3),
            Pitch::new(Note::A, 3),
            Pitch::new(Note::D, 4),
            Pitch::new(Note::G, 4),
            Pitch::new(Note::B, 4),
            Pitch::new(Note::E, 5),
        ],
    }
});

/// DADGAD tuning, popular in Celtic and fingerstyle guitar (D A D G A D).
pub static DADGAD: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::D, 3),
            Pitch::new(Note::A, 3),
            Pitch::new(Note::D, 4),
            Pitch::new(Note::G, 4),
            Pitch::new(Note::A, 4),
            Pitch::new(Note::D, 5),
        ],
    }
});

/// Open G tuning (D G D G B D), popular for slide guitar and blues.
pub static OPEN_G: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::D, 3),
            Pitch::new(Note::G, 3),
            Pitch::new(Note::D, 4),
            Pitch::new(Note::G, 4),
            Pitch::new(Note::B, 4),
            Pitch::new(Note::D, 5),
        ],
    }
});

/// Standard tuning on a 7-string guitar (B E A D G B E).
pub static STANDARD_7: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::B, 2),
            Pitch::new(Note::E, 3),
            Pitch::new(Note::A, 3),
            Pitch::new(Note::D, 4),
            Pitch::new(Note::G, 4),
            Pitch::new(Note::B, 4),
            Pitch::new(Note::E, 5),
        ],
    }
});

/// Standard tuning on a 4-string bass (E A D G).
pub static BASS_4: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::E, 2),
            Pitch::new(Note::A, 2),
            Pitch::new(Note::D, 3),
            Pitch::new(Note::G, 3),
        ],
    }
});

/// Standard tuning on a 5-string bass (B E A D G).
pub static BASS_5: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard {
        open_strings: vec![
            Pitch::new(Note::B, 1),
            Pitch::new(Note::E, 2),
            Pitch::new(Note::A, 2),
            Pitch::new(Note::D, 3),
            Pitch::new(Note::G, 3),
        ],
    }
});

/// Represents a fretboard with any arbitrary tuning or number of strings.
#[derive(Clone, Debug, PartialEq)]
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
    pub const MAX: u8 = 35;
    const OPEN: u8 = u8::MIN;

    /// The number of strings on the fretboard.
    pub fn num_strings(&self) -> u8 {
        u8::try_from(self.open_strings.len()).unwrap()
    }

    /// Fallible indexing for an element in [self.open_strings].
    /// It is important to remember that colloquially, the thickest string on a guitar
    /// is "the 6th string", but it is indexed here as `self.open_strings[0]`.
    pub fn get_string(&self, string: u8) -> Result<&Pitch, MusicSemanticsError> {
        Ok(self.open_strings.get(string as usize)
            .ok_or(MusicSemanticsError::StringTooHighForFretboard(string, self.clone()))?
        )
    }

    /// This is the preferred way to create a [SoundedNote] instance, because it
    /// validates the initialization parameters against [self].
    pub fn sounded_note(&self, string: u8, fret: u8) -> Result<SoundedNote<'_>, MusicSemanticsError> {
        if fret > Self::MAX {
            return Err(MusicSemanticsError::FretTooHigh(fret));
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
        Ok(SoundedNote {
            string,
            fret,
            pitch,
            fretboard: self,
        })
    }

    /// Given a string and target note, returns the first available
    /// [SoundedNote] whose fret equals a given [Note].
    pub fn note_on_string(&self, note: &Note, string: u8) -> Result<SoundedNote<'_>, MusicSemanticsError> {
        let fret = self.which_fret(note, string)?;
        self.sounded_note(string, fret)
    }

    /// Returns the fret where a given [Note] resides on a given string.
    /// e.g. "where is the place I can find an F# on the 3rd string of this fretboard?"
    pub fn which_fret(&self, note: &Note, string: u8) -> Result<u8, MusicSemanticsError> {
        let open_string = self.get_string(string)?;
        let mut pc = Pc::from(&open_string.note);
        let fretted_pc = Pc::from(note);
        for i in 0..12 {
            if pc == fretted_pc {
                return Ok(i);
            }
            pc = pc.next();
        }
        // Should be guaranteed to reach the target note in at most twelve steps
        unreachable!()
    }
}

impl Deref for Fretboard {
    type Target = Vec<Pitch>;

    fn deref(&self) -> &Self::Target {
        &self.open_strings
    }
}

impl IntoIterator for Fretboard {
    type Item = Pitch;
    type IntoIter = std::vec::IntoIter<Pitch>;

    fn into_iter(self) -> Self::IntoIter {
        self.open_strings.into_iter()
    }
}

impl<'a> IntoIterator for &'a Fretboard {
    type Item = &'a Pitch;
    type IntoIter = std::slice::Iter<'a, Pitch>;

    fn into_iter(self) -> Self::IntoIter {
        self.open_strings.iter()
    }
}
