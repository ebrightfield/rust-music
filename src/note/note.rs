use std::str::FromStr;
use crate::note::spelling::{Accidental, Letter, Spelling};
use anyhow::anyhow;
use crate::note::pc::Pc;

/// Every chromatic note in all possible enharmonic spellings,
/// with the following caveats:
/// - Nothing more extreme than a double-accidental.
/// - No "C" or "F" flattened more than once.
/// - No "B" or "E" sharpened more than once.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Note {
    C,
    Deses,
    Cis,
    Des,
    Cisis,
    D,
    Eeses,
    Dis,
    Ees,
    Disis,
    E,
    Fes,
    Eis,
    F,
    Geses,
    Fis,
    Ges,
    Fisis,
    G,
    Aeses,
    Gis,
    Aes,
    Gisis,
    A,
    Beses,
    Ais,
    Bes,
    Aisis,
    B,
    Ces,
    Bis,
}

impl Note {
    // Return a note with an enharmonic spelling. Returns a clone of naturals.
    pub fn enharmonic(&self) -> Note {
        let spelling: Spelling = self.into();
        match spelling.acc {
            Accidental::Natural => self.clone(),
            Accidental::DoubleFlat => {
                Spelling {
                    letter: spelling.letter.prev(),
                    acc: Accidental::Natural,
                }.try_into().unwrap()
            },
            Accidental::DoubleSharp => {
                Spelling {
                    letter: spelling.letter.next(),
                    acc: Accidental::Natural,
                }.try_into().unwrap()
            },
            Accidental::Flat => {
                if spelling.letter == Letter::C ||
                    spelling.letter == Letter::F {
                    return Spelling {
                        letter: spelling.letter.prev(),
                        acc: Accidental::Natural,
                    }.try_into().unwrap();
                }
                Spelling {
                    letter: spelling.letter.prev(),
                    acc: Accidental::Sharp,
                }.try_into().unwrap()
            },
            Accidental::Sharp => {
                if spelling.letter == Letter::B ||
                    spelling.letter == Letter::E {
                    return Spelling {
                        letter: spelling.letter.next(),
                        acc: Accidental::Natural,
                    }.try_into().unwrap();
                }
                Spelling {
                    letter: spelling.letter.next(),
                    acc: Accidental::Flat,
                }.try_into().unwrap()
            },
        }
    }

    /// Same as the regular enharmonic method,
    /// but aggressively switches B to Cb, C to B#, E to Fb, F to E#.
    pub fn enharmonic_flip_bcef(&self) -> Note {
        if *self == Note::B {
            return Note::Ces;
        }
        if *self == Note::C {
            return Note::Bis;
        }
        if *self == Note::E {
            return Note::Fes;
        }
        if *self == Note::F {
            return Note::Eis;
        }
        self.enharmonic()
    }

    /// Control for spelling by including a "palette" of possible note values.
    pub fn spelled_as_in(&self, notes: &Vec<Note>) -> anyhow::Result<Self> {
        let pc = Pc::from(self);
        for note in notes {
            if Pc::from(note) == pc {
                return Ok(note.clone());
            }
        }
        Err(anyhow!("{:?} not in the notes {:?}", pc.notes(), notes))
    }
}

impl FromStr for Note {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let spelling = Spelling::from_str(&s)?;
        Ok(Self::try_from(spelling)?)
    }
}

impl TryFrom<Spelling> for Note {
    type Error = anyhow::Error;

    fn try_from(spelling: Spelling) -> Result<Self, Self::Error> {
        match spelling.letter {
            Letter::C => match spelling.acc {
                Accidental::Natural => Ok(Note::C),
                Accidental::Flat => Ok(Note::Ces),
                Accidental::Sharp => Ok(Note::Cis),
                Accidental::DoubleFlat => Err(anyhow!(
                    "Invalid spelling: {}, {}",
                    Letter::C,
                    Accidental::DoubleFlat,
                )),
                Accidental::DoubleSharp => Ok(Note::Cisis),
            },
            Letter::D => match spelling.acc {
                Accidental::Natural => Ok(Note::D),
                Accidental::Flat => Ok(Note::Des),
                Accidental::Sharp => Ok(Note::Dis),
                Accidental::DoubleFlat => Ok(Note::Deses),
                Accidental::DoubleSharp => Ok(Note::Disis),
            },
            Letter::E => match spelling.acc {
                Accidental::Natural => Ok(Note::E),
                Accidental::Flat => Ok(Note::Ees),
                Accidental::Sharp => Ok(Note::Eis),
                Accidental::DoubleFlat => Ok(Note::Eeses),
                Accidental::DoubleSharp => Err(anyhow!(
                    "Invalid spelling: {}, {}",
                    Letter::E,
                    Accidental::DoubleSharp,
                )),
            },
            Letter::F => match spelling.acc {
                Accidental::Natural => Ok(Note::F),
                Accidental::Flat => Ok(Note::Fes),
                Accidental::Sharp => Ok(Note::Fis),
                Accidental::DoubleFlat => Err(anyhow!(
                    "Invalid spelling: {}, {}",
                    Letter::F,
                    Accidental::DoubleFlat,
                )),
                Accidental::DoubleSharp => Ok(Note::Fisis),
            },
            Letter::G => match spelling.acc {
                Accidental::Natural => Ok(Note::G),
                Accidental::Flat => Ok(Note::Ges),
                Accidental::Sharp => Ok(Note::Gis),
                Accidental::DoubleFlat => Ok(Note::Geses),
                Accidental::DoubleSharp => Ok(Note::Gisis),
            },
            Letter::A => match spelling.acc {
                Accidental::Natural => Ok(Note::A),
                Accidental::Flat => Ok(Note::Aes),
                Accidental::Sharp => Ok(Note::Ais),
                Accidental::DoubleFlat => Ok(Note::Aeses),
                Accidental::DoubleSharp => Ok(Note::Aisis),
            },
            Letter::B => match spelling.acc {
                Accidental::Natural => Ok(Note::B),
                Accidental::Flat => Ok(Note::Bes),
                Accidental::Sharp => Ok(Note::Bis),
                Accidental::DoubleFlat => Ok(Note::Beses),
                Accidental::DoubleSharp => Err(anyhow!(
                    "Invalid spelling: {}, {}",
                    Letter::B,
                    Accidental::DoubleSharp,
                )),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_parsing() {
        assert_eq!(Note::Cisis, Note::from_str("C##").unwrap());
        assert_eq!(Note::C, Note::from_str("C").unwrap());
        assert_eq!(Note::Bes, Note::from_str("Bb").unwrap());
    }

    #[test]
    fn test_enharmonics() {
        assert_eq!(Note::Cis.enharmonic(), Note::Des);
        assert_eq!(Note::Cisis.enharmonic(), Note::D);
        assert_eq!(Note::Bes.enharmonic(), Note::Ais);
        assert_eq!(Note::C.enharmonic_flip_bcef(), Note::Bis);
    }
}