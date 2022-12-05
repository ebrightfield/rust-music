use crate::note::note::Note;
use anyhow::anyhow;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

/// Nothing more extreme than a double-accidental is represented here.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
    DoubleSharp,
    DoubleFlat,
}

impl Accidental {
    pub fn is_double(&self) -> bool {
        *self == Accidental::DoubleFlat || *self == Accidental::DoubleSharp
    }
}

impl FromStr for Accidental {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Accidental::Natural),
            "b" => Ok(Accidental::Flat),
            "#" => Ok(Accidental::Sharp),
            "bb" => Ok(Accidental::DoubleFlat),
            "##" => Ok(Accidental::DoubleSharp),
            // TODO Match fancy Utf-8 chars
            _ => Err(anyhow!("Invalid note accidental: {}", s)),
        }
    }
}

// TODO also impl block with a toFancyStr that uses UTF-8 chars.

impl Display for Accidental {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Accidental::Natural => "".to_string(),
            Accidental::Flat => "b".to_string(),
            Accidental::Sharp => "#".to_string(),
            Accidental::DoubleFlat => "bb".to_string(),
            Accidental::DoubleSharp => "##".to_string(),
        };
        f.write_str(&s)
    }
}

/// Chromatic musical letter, irrespective of octaves or attached accidentals.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Letter {
    pub fn next(&self) -> Letter {
        match &self {
            Letter::A => Letter::B,
            Letter::B => Letter::C,
            Letter::C => Letter::D,
            Letter::D => Letter::E,
            Letter::E => Letter::F,
            Letter::F => Letter::G,
            Letter::G => Letter::A,
        }
    }

    pub fn prev(&self) -> Letter {
        match &self {
            Letter::A => Letter::G,
            Letter::B => Letter::A,
            Letter::C => Letter::B,
            Letter::D => Letter::C,
            Letter::E => Letter::D,
            Letter::F => Letter::E,
            Letter::G => Letter::F,
        }
    }

    pub fn diatonic_distance_up(&self, other: &Letter) -> u8 {
        u8::try_from(
            (i32::from(other) - i32::from(self)).rem_euclid(7)
        ).unwrap()
    }
}

impl From<&Letter> for i32 {
    fn from(value: &Letter) -> Self {
        match value {
            Letter::A => 5,
            Letter::B => 6,
            Letter::C => 0,
            Letter::D => 1,
            Letter::E => 2,
            Letter::F => 3,
            Letter::G => 4,
        }
    }
}

impl From<Letter> for i32 {
    fn from(value: Letter) -> Self {
        i32::from(&value)
    }
}

impl FromStr for Letter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "a" => Ok(Letter::A),
            "b" => Ok(Letter::B),
            "c" => Ok(Letter::C),
            "d" => Ok(Letter::D),
            "e" => Ok(Letter::E),
            "f" => Ok(Letter::F),
            "g" => Ok(Letter::G),
            _ => Err(anyhow!("Invalid note letter: {}", s)),
        }
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Letter::A => "A".to_string(),
            Letter::B => "B".to_string(),
            Letter::C => "C".to_string(),
            Letter::D => "D".to_string(),
            Letter::E => "E".to_string(),
            Letter::F => "F".to_string(),
            Letter::G => "G".to_string(),
        };
        f.write_str(&s)
    }
}

/// A combination of letter and accidental information.
#[derive(Debug, PartialEq)]
pub struct Spelling {
    pub letter: Letter,
    pub acc: Accidental,
}

impl Spelling {
    pub fn new(letter: Letter, acc: Accidental) -> Self {
        Self { letter, acc }
    }
}

impl Display for Spelling {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let letter = self.letter.to_string();
        let acc = self.acc.to_string();
        let s = letter.add(&acc);
        f.write_str(&s)
    }
}

impl FromStr for Spelling {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let letter = Letter::from_str(&s[0..1])?;
        let acc = Accidental::from_str(&s[1..s.len()])?;
        Ok(Self { letter, acc })
    }
}

impl From<Note> for Spelling {
    fn from(note: Note) -> Self {
        Spelling::from(&note)
    }
}

impl From<&Note> for Spelling {
    fn from(note: &Note) -> Self {
        match note {
            Note::C => Spelling::new(Letter::C, Accidental::Natural),
            Note::Deses => Spelling::new(Letter::D, Accidental::DoubleSharp),
            Note::Cis => Spelling::new(Letter::C, Accidental::Sharp),
            Note::Des => Spelling::new(Letter::D, Accidental::Flat),
            Note::Cisis => Spelling::new(Letter::C, Accidental::DoubleSharp),
            Note::D => Spelling::new(Letter::D, Accidental::Natural),
            Note::Eeses => Spelling::new(Letter::E, Accidental::DoubleFlat),
            Note::Dis => Spelling::new(Letter::D, Accidental::Sharp),
            Note::Ees => Spelling::new(Letter::E, Accidental::Flat),
            Note::Disis => Spelling::new(Letter::D, Accidental::DoubleSharp),
            Note::E => Spelling::new(Letter::E, Accidental::Natural),
            Note::Fes => Spelling::new(Letter::F, Accidental::Flat),
            Note::Eis => Spelling::new(Letter::E, Accidental::Sharp),
            Note::F => Spelling::new(Letter::F, Accidental::Natural),
            Note::Geses => Spelling::new(Letter::G, Accidental::DoubleFlat),
            Note::Fis => Spelling::new(Letter::F, Accidental::Sharp),
            Note::Ges => Spelling::new(Letter::G, Accidental::Flat),
            Note::Fisis => Spelling::new(Letter::F, Accidental::DoubleSharp),
            Note::G => Spelling::new(Letter::G, Accidental::Natural),
            Note::Aeses => Spelling::new(Letter::A, Accidental::DoubleFlat),
            Note::Gis => Spelling::new(Letter::G, Accidental::Sharp),
            Note::Aes => Spelling::new(Letter::A, Accidental::Flat),
            Note::Gisis => Spelling::new(Letter::G, Accidental::DoubleSharp),
            Note::A => Spelling::new(Letter::A, Accidental::Natural),
            Note::Beses => Spelling::new(Letter::B, Accidental::DoubleFlat),
            Note::Ais => Spelling::new(Letter::A, Accidental::Sharp),
            Note::Bes => Spelling::new(Letter::B, Accidental::Flat),
            Note::Aisis => Spelling::new(Letter::A, Accidental::DoubleSharp),
            Note::B => Spelling::new(Letter::B, Accidental::Natural),
            Note::Ces => Spelling::new(Letter::C, Accidental::Flat),
            Note::Bis => Spelling::new(Letter::B, Accidental::Sharp),
        }
    }
}
