pub mod fretboard_shape;

use std::ops::Deref;
use anyhow::{anyhow, Result};
use crate::pitch::Pitch;

#[derive(Clone, Debug)]
pub struct OpenString(Pitch);

impl Deref for OpenString {
    type Target = Pitch;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Fretboard {
    /// The number and tuning of a fretboard is entirely defined here.
    strings: Vec<OpenString>,
}

impl Fretboard {
    pub fn num_strings(&self) -> usize {
        self.strings.len()
    }

    pub fn at(&self, string: usize, fret: Option<u8>) -> Result<FrettedNote> {
        if let Some(fret) = fret {
            if fret > 35 {
                return Err(anyhow!("Excessively high fret number"));
            }
            let open_string = self.strings.get(string)
                .ok_or(anyhow!("String too high for fretboard {:?}", self))?;
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
}

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
    string: usize,
    fretboard: &'a Fretboard,
}