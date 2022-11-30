use anyhow::anyhow;
use std::ops::{Deref, DerefMut};
use crate::note_collections::octave_partition::IntervalClass;

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
#[derive(Debug, Clone, PartialEq)]
pub struct Alt(pub(crate) Vec<AltChoice>);

impl Alt {
    pub fn empty() -> Self {
        Alt(vec![])
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    Seventh,
    Ninth,
    Eleventh,
    Thirteenth,
}

/// Chords based around a Major triad.
#[derive(Debug, Clone, PartialEq)]
pub enum MajorSubtype {
    Maj(Alt),
    Maj6(Alt),
    MajN(Vec<Extension>, Alt),
    N(Vec<Extension>, Alt),
}

/// Chords based around a minor triad.
#[derive(Debug, Clone, PartialEq)]
pub enum MinorSubtype {
    Min(Alt),
    Min6(Alt),
    MinMajN(Vec<Extension>, Alt),
    MinN(Vec<Extension>, Alt),
}

/// Chords based around an Augmented triad.
#[derive(Debug, Clone, PartialEq)]
pub enum AugSubtype {
    /// e.g. C+
    Aug(Alt),
    /// e.g. C+Maj7
    AugMajN(Vec<Extension>, Alt),
    /// e.g. C+7
    AugN(Vec<Extension>, Alt),
}

/// Chords based around a diminished triad.
#[derive(Debug, Clone, PartialEq)]
pub enum DimSubtype {
    /// e.g. Cdim
    Dim(Alt),
    /// e.g. Cmin7b5
    MinNb5(Vec<Extension>, Alt),
    /// e.g. Cdim7
    DimN(Vec<Extension>, Alt),
    /// Edge case -- e.g. CdimMaj7
    DimMajN(Vec<Extension>, Alt),
}

/// Chords based around a diminished triad.
#[derive(Debug, Clone, PartialEq)]
pub enum SusSubtype {
    Sus2(Alt),
    Sus4(Alt),
    DomNSus(Vec<Extension>, Alt),
    MajNSus(Vec<Extension>, Alt),
    SixNineSus(Alt),
}

/// Basic categories for chords >=3 pitch classes,
/// and special variants for the trivial cases of
/// [ChordQuality::Interval] and [ChordQuality::SingleNote].
#[derive(Debug, Clone, PartialEq)]
pub enum ChordQuality {
    Major(MajorSubtype),
    Minor(MinorSubtype),
    Aug(AugSubtype),
    Dim(DimSubtype),
    Sus(SusSubtype),
    /// Any pair of distinct pitch-classes
    Interval(IntervalClass),
    SingleNote,
}
