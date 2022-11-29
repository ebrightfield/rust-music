use crate::chord::pc_set::PcSet;
use crate::note::note::Note;
use crate::note::spelling::Spelling;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use crate::chord::octave_partition::BaseChromaticInterval;

/// The "ninth", "eleventh", etc in Maj9th or min11th chords, etc.
#[derive(Debug, Clone)]
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

/// Chord Quality Alterations
#[derive(Debug, Clone)]
pub struct Alt(Vec<AltChoice>);

/// Chords based around a Major triad.
#[derive(Debug, Clone)]
pub enum MajorSubtype {
    Maj6(Option<Alt>),
    MajN(Option<Alt>),
    N(Option<Alt>),
}

/// Chords based around a minor triad.
#[derive(Debug, Clone)]
pub enum MinorSubtype {
    Min6(Option<Alt>),
    MinMajN(Option<Alt>),
    MinN(Option<Alt>),
}

/// Chords based around an Augmented triad.
#[derive(Debug, Clone)]
pub enum AugSubtype {
    /// e.g. C+Maj7
    AugMajN(Option<Alt>),
    /// e.g. C+7
    AugN(Option<Alt>),
}

/// Chords based around a diminished triad.
#[derive(Debug, Clone)]
pub enum DimSubtype {
    /// e.g. Cmin7b5
    MinNb5(Option<Alt>),
    /// e.g. Cdim7
    DimN(Option<Alt>),
    /// Edge case -- e.g. CdimMaj7
    DimMajN(Option<Alt>),
}

/// Basic categories for chords >=3 pitch classes,
/// except for [ChordQuality::Interval] and [ChordQuality::SingleNote].
#[derive(Debug, Clone)]
pub enum ChordQuality {
    Major(MajorSubtype),
    Minor(MinorSubtype),
    Aug(AugSubtype),
    Dim(DimSubtype),
    Sus,
    AssumedThird,
    /// Any pair of distinct pitch-classes
    Interval(BaseChromaticInterval),
    SingleNote,
}

#[derive(Debug, Clone)]
pub enum TonalSpecification {
    /// If it's a slash chord, the bass note will be supplied here.
    SlashChord {
        bass: Note,
        root: Note,
    },
    /// Root note relative to the defined chord quality.
    RootPosition(Note),
}

#[derive(Debug, Clone)]
pub struct ChordName {
    /// Information regarding any choice of root notes, etc.
    tonality: Option<TonalSpecification>,
    /// "Flavor" of chord, when built off the root.
    quality: ChordQuality,
    /// Underlying set of notes.
    pc_set: PcSet,
}

// fn construct_quality_string(quality: &ChordQuality, ext: &ChordExtension) -> String {
//     match quality {
//         ChordQuality::Major => {
//             match ext {
//                 ChordExtension::None => "Maj",
//                 ChordExtension::N => "Maj",
//             }
//         }
//     }
// }

impl Display for ChordName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // let mut bass = Spelling::from(&self.root).to_string();
        // let mut is_slash_chord = false;
        // if let Some(note) = &self.bass {
        //     bass = Spelling::from(note).to_string();
        //     is_slash_chord = true;
        // }
        // f.write_str(&bass)?;
        Ok(())
    }
}
