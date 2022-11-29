use std::ops::{Deref, DerefMut};
use anyhow::anyhow;
use crate::chord::octave_partition::BaseChromaticInterval;
use crate::note::note::Note;
use crate::note::pc::Pc;

/// The "ninth", "eleventh", etc in Maj9th or min11th chords, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum AltChoice {
    FlatNine,
    Nine,
    SharpNine,
    FlatEleven,
    Eleven,
    SharpEleven,
    FlatThirteenth,
    Thirteenth,
    SharpThirteenth,
}

impl TryFrom<usize> for AltChoice {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(AltChoice::FlatNine),
            2 => Ok(AltChoice::Nine),
            3 => Ok(AltChoice::SharpNine),
            4 => Ok(AltChoice::FlatEleven),
            5 => Ok(AltChoice::Eleven),
            6 => Ok(AltChoice::SharpEleven),
            8 => Ok(AltChoice::FlatThirteenth),
            9 => Ok(AltChoice::Thirteenth),
            10 => Ok(AltChoice::SharpThirteenth),
            _ => Err(anyhow!("not a proper pc for an alteration: {}", value))
        }
    }
}

/// Chord Quality Alterations
#[derive(Debug, Clone)]
pub struct Alt(Vec<AltChoice>);

impl From<Vec<AltChoice>> for Alt {
    fn from(value: Vec<AltChoice>) -> Self {
        Self(value)
    }
}

impl Deref for Alt {
    type Target = Vec<AltChoice>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Alt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub enum Extension {
    Seventh,
    Ninth,
    Eleventh,
    Thirteenth,
}

/// Chords based around a Major triad.
#[derive(Debug, Clone)]
pub enum MajorSubtype {
    Maj(Option<Alt>),
    Maj6(Option<Alt>),
    MajN(Vec<Extension>, Option<Alt>),
    N(Vec<Extension>, Option<Alt>),
}

/// Chords based around a minor triad.
#[derive(Debug, Clone)]
pub enum MinorSubtype {
    Min(Option<Alt>),
    Min6(Option<Alt>),
    MinMajN(Vec<Extension>, Option<Alt>),
    MinN(Vec<Extension>, Option<Alt>),
}

/// Chords based around an Augmented triad.
#[derive(Debug, Clone)]
pub enum AugSubtype {
    /// e.g. C+
    Aug(Option<Alt>),
    /// e.g. C+Maj7
    AugMajN(Extension, Option<Alt>),
    /// e.g. C+7
    AugN(Extension, Option<Alt>),
}

/// Chords based around a diminished triad.
#[derive(Debug, Clone)]
pub enum DimSubtype {
    /// e.g. Cdim
    Dim(Option<Alt>),
    /// e.g. Cmin7b5
    MinNb5(Extension, Option<Alt>),
    /// e.g. Cdim7
    DimN(Extension, Option<Alt>),
    /// Edge case -- e.g. CdimMaj7
    DimMajN(Extension, Option<Alt>),
}

/// Basic categories for chords >=3 pitch classes,
/// except for [ChordQuality::Interval] and [ChordQuality::SingleNote].
#[derive(Debug, Clone)]
pub enum ChordQuality {
    Major(MajorSubtype),
    Minor(MinorSubtype),
    Aug(AugSubtype),
    Dim(DimSubtype),
    Sus, // TODO Subtype
    AssumedThird, // TODO This should mirror Major/Minor subtypes, or be removed
    /// Any pair of distinct pitch-classes
    Interval(BaseChromaticInterval),
    SingleNote,
}

/// Whether or not something is a slash chord.
/// All specified notes are assumed to be members of their associated [NoteSet].
#[derive(Debug, Clone)]
pub enum TonalSpecification {
    /// If it's a slash chord, the bass note will be supplied here.
    SlashChord {
        bass: Note,
        root: Note,
    },
    /// Root note relative to the defined chord quality.
    RootPosition(Note),
    /// No tonal specification, possibly slash chord denoted via Pc.
    None(Option<Pc>)
}
