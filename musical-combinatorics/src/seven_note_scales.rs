use anyhow::anyhow;
use music::note::Pc;
use music::note::Pc::*;
use music::note_collections::{OctavePartition, PcSet};

/// This is a non-exhaustive collection of pertinent seven-note scales.
/// They all are derived from the following procedure:
/// - Start from a Root and Perfect Fifth (two notes so far)
/// - Pick either a Major or minor Third (up to three notes now)
/// - Pick a Fourth to sit between the Third and Fifth.
/// - Pick a Second to sit between the Root and Third.
/// - Pick a minor or Major Sixth, and a minor or Major Seventh.
///
/// This yields many scales which are in fact modes of each other,
/// but it reduces down to the 22 scales below.
///
/// Some notable properties of these 22 scales:
/// - You easily derive the obvious seven-note scales like Major, melodic minor, Harmonic minor/Major.
/// - Each scale has a mode that looks either Ionian, Lydian, Mixolydian, or melodic minor,
///   with some potential combination of alterations to the 2nd, 4th, or 6th.
/// - There are no 4-semitone "steps" between adjacent notes.
/// - There is never more than two consecutive half-steps.
/// - There are no adjacent augmented steps (adjacent 3-semitone steps).
/// - No augmented step surrounded on both sides by a whole-step.
/// - Diminished and augmented intervals in all their forms (augmented 6ths, diminished fourths, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum SevenNoteScaleQuality {
    Major,
    MelodicMinor,
    MajorFlat9Flat13,
    MixolydianSharp11Flat13,
    HarmonicMinor,
    HarmonicMajor,
    MajorFlat9,
    MajorSharp9,
    MixolydianSharp9,
    LydianFlat9,
    MixolydianSharp9Sharp11,
    MixolydianFlat9Sharp11,
    MelodicMinorFlat9Sharp11,
    MixolydianFlat9Sharp11Flat13,
    MixolydianSharp9Flat13,
    MelodicMinorFlat11,
    MixolydianSharp9Sharp11Flat13,
    MelodicMinorFlat9Flat11,
    LydianFlat9Flat13,
    MelodicMinorFlat9Sharp11Flat13,
    LydianSharp9Flat13,
    MajorSharp9Flat13,
}

pub const MAJOR_SCALE_PCS: &[Pc] = &[Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11];
pub const MELODIC_MINOR_PCS: &[Pc] = &[Pc0, Pc2, Pc3, Pc5, Pc7, Pc9, Pc11];
pub const MAJOR_FLAT9_FLAT13_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc5, Pc7, Pc8, Pc11];
pub const MIXOLYDIAN_SHARP11_FLAT13_PCS: &[Pc] = &[Pc0, Pc2, Pc4, Pc6, Pc7, Pc8, Pc10];
pub const HARMONIC_MINOR_PCS: &[Pc] = &[Pc0, Pc2, Pc3, Pc5, Pc7, Pc8, Pc11];
pub const HARMONIC_MAJOR_PCS: &[Pc] = &[Pc0, Pc2, Pc4, Pc5, Pc7, Pc8, Pc11];
pub const MAJOR_FLAT9_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc5, Pc7, Pc9, Pc11];
pub const MAJOR_SHARP9_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc5, Pc7, Pc9, Pc11];
pub const MIXOLYDIAN_SHARP9_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc5, Pc7, Pc9, Pc10];
pub const LYDIAN_FLAT9_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc6, Pc7, Pc9, Pc11];
pub const MIXOLYDIAN_SHARP9_SHARP11_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc6, Pc7, Pc9, Pc10];
pub const MIXOLYDIAN_FLAT9_SHARP11_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc6, Pc7, Pc9, Pc10];
pub const MELODIC_MINOR_FLAT9_SHARP11_PCS: &[Pc] = &[Pc0, Pc1, Pc3, Pc6, Pc7, Pc9, Pc11];
pub const MIXOLYDIAN_FLAT9_SHARP11_FLAT13_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc6, Pc7, Pc8, Pc10];
pub const MIXOLYDIAN_SHARP9_FLAT13_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc5, Pc7, Pc8, Pc10];
pub const MELODIC_MINOR_FLAT11_PCS: &[Pc] = &[Pc0, Pc2, Pc3, Pc4, Pc7, Pc9, Pc11];
pub const MIXOLYDIAN_SHARP9_SHARP11_FLAT13_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc6, Pc7, Pc8, Pc10];
pub const MELODIC_MINOR_FLAT9_FLAT11_PCS: &[Pc] = &[Pc0, Pc1, Pc3, Pc4, Pc7, Pc9, Pc11];
pub const LYDIAN_FLAT9_FLAT13_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc6, Pc7, Pc8, Pc11];
pub const MELODIC_MINOR_FLAT9_SHARP11_FLAT13_PCS: &[Pc] = &[Pc0, Pc1, Pc3, Pc6, Pc7, Pc8, Pc11];
pub const LYDIAN_SHARP9_FLAT13_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc6, Pc7, Pc8, Pc11];
pub const MAJOR_SHARP9_FLAT13_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc5, Pc7, Pc8, Pc11];

impl From<&SevenNoteScaleQuality> for OctavePartition {
    fn from(value: &SevenNoteScaleQuality) -> Self {
        OctavePartition::from(match value {
            SevenNoteScaleQuality::Major => MAJOR_SCALE_PCS,
            SevenNoteScaleQuality::MelodicMinor => MELODIC_MINOR_PCS,
            SevenNoteScaleQuality::MajorFlat9Flat13 => MAJOR_FLAT9_FLAT13_PCS,
            SevenNoteScaleQuality::MixolydianSharp11Flat13 => MIXOLYDIAN_SHARP11_FLAT13_PCS,
            SevenNoteScaleQuality::HarmonicMinor => HARMONIC_MINOR_PCS,
            SevenNoteScaleQuality::HarmonicMajor => HARMONIC_MAJOR_PCS,
            SevenNoteScaleQuality::MajorFlat9 => MAJOR_FLAT9_PCS,
            SevenNoteScaleQuality::MajorSharp9 => MAJOR_SHARP9_PCS,
            SevenNoteScaleQuality::MixolydianSharp9 => MIXOLYDIAN_SHARP9_PCS,
            SevenNoteScaleQuality::LydianFlat9 => LYDIAN_FLAT9_PCS,
            SevenNoteScaleQuality::MixolydianSharp9Sharp11 => MIXOLYDIAN_SHARP9_SHARP11_PCS,
            SevenNoteScaleQuality::MixolydianFlat9Sharp11 => MIXOLYDIAN_FLAT9_SHARP11_PCS,
            SevenNoteScaleQuality::MelodicMinorFlat9Sharp11 => MELODIC_MINOR_FLAT9_SHARP11_PCS,
            SevenNoteScaleQuality::MixolydianFlat9Sharp11Flat13 => MIXOLYDIAN_FLAT9_SHARP11_FLAT13_PCS,
            SevenNoteScaleQuality::MixolydianSharp9Flat13 => MIXOLYDIAN_SHARP9_FLAT13_PCS,
            SevenNoteScaleQuality::MelodicMinorFlat11 => MELODIC_MINOR_FLAT11_PCS,
            SevenNoteScaleQuality::MixolydianSharp9Sharp11Flat13 => MIXOLYDIAN_SHARP9_SHARP11_FLAT13_PCS,
            SevenNoteScaleQuality::MelodicMinorFlat9Flat11 => MELODIC_MINOR_FLAT9_FLAT11_PCS,
            SevenNoteScaleQuality::LydianFlat9Flat13 => LYDIAN_FLAT9_FLAT13_PCS,
            SevenNoteScaleQuality::MelodicMinorFlat9Sharp11Flat13 => MELODIC_MINOR_FLAT9_SHARP11_FLAT13_PCS,
            SevenNoteScaleQuality::LydianSharp9Flat13 => LYDIAN_SHARP9_FLAT13_PCS,
            SevenNoteScaleQuality::MajorSharp9Flat13 => MAJOR_SHARP9_FLAT13_PCS,
        })
    }
}

impl From<SevenNoteScaleQuality> for OctavePartition {
    fn from(value: SevenNoteScaleQuality) -> Self {
        OctavePartition::from(&value)
    }
}

impl TryFrom<&PcSet> for SevenNoteScaleQuality {
    type Error = anyhow::Error;

    fn try_from(value: &PcSet) -> Result<Self, Self::Error> {
        if value.len() != 7 {
            return Err(anyhow!("wrong size for seven note scale: {:?}", value));
        }
        let pitches = value.as_slice();
        match pitches {
            MAJOR_SCALE_PCS => Ok(SevenNoteScaleQuality::Major),
            MELODIC_MINOR_PCS => Ok(SevenNoteScaleQuality::MelodicMinor),
            MAJOR_FLAT9_FLAT13_PCS => Ok(SevenNoteScaleQuality::MajorFlat9Flat13),
            MIXOLYDIAN_SHARP11_FLAT13_PCS => Ok(SevenNoteScaleQuality::MixolydianSharp11Flat13),
            HARMONIC_MINOR_PCS => Ok(SevenNoteScaleQuality::HarmonicMinor),
            HARMONIC_MAJOR_PCS => Ok(SevenNoteScaleQuality::HarmonicMajor),
            MAJOR_FLAT9_PCS => Ok(SevenNoteScaleQuality::MajorFlat9),
            MAJOR_SHARP9_PCS => Ok(SevenNoteScaleQuality::MajorSharp9),
            MIXOLYDIAN_SHARP9_PCS => Ok(SevenNoteScaleQuality::MixolydianSharp9),
            LYDIAN_FLAT9_PCS => Ok(SevenNoteScaleQuality::LydianFlat9),
            MIXOLYDIAN_SHARP9_SHARP11_PCS => Ok(SevenNoteScaleQuality::MixolydianSharp9Sharp11),
            MIXOLYDIAN_FLAT9_SHARP11_PCS => Ok(SevenNoteScaleQuality::MixolydianFlat9Sharp11),
            MELODIC_MINOR_FLAT9_SHARP11_PCS => Ok(SevenNoteScaleQuality::MelodicMinorFlat9Sharp11),
            MIXOLYDIAN_FLAT9_SHARP11_FLAT13_PCS => Ok(SevenNoteScaleQuality::MixolydianFlat9Sharp11Flat13),
            MIXOLYDIAN_SHARP9_FLAT13_PCS => Ok(SevenNoteScaleQuality::MixolydianSharp9Flat13),
            MELODIC_MINOR_FLAT11_PCS => Ok(SevenNoteScaleQuality::MelodicMinorFlat11),
            MIXOLYDIAN_SHARP9_SHARP11_FLAT13_PCS => Ok(SevenNoteScaleQuality::MixolydianSharp9Sharp11Flat13),
            MELODIC_MINOR_FLAT9_FLAT11_PCS => Ok(SevenNoteScaleQuality::MelodicMinorFlat9Flat11),
            LYDIAN_FLAT9_FLAT13_PCS => Ok(SevenNoteScaleQuality::LydianFlat9Flat13),
            MELODIC_MINOR_FLAT9_SHARP11_FLAT13_PCS => Ok(SevenNoteScaleQuality::MelodicMinorFlat9Sharp11Flat13),
            LYDIAN_SHARP9_FLAT13_PCS => Ok(SevenNoteScaleQuality::LydianSharp9Flat13),
            MAJOR_SHARP9_FLAT13_PCS => Ok(SevenNoteScaleQuality::MajorSharp9Flat13),
            _ => Err(anyhow!("Seven note scale not recognized: {:?}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Convert to and from intervallically descriptive types.
    fn test_quality(quality: SevenNoteScaleQuality) {
        let partition = OctavePartition::from(&quality);
        let pc_set = PcSet::from(&partition);
        assert_eq!(quality, SevenNoteScaleQuality::try_from(&pc_set).unwrap())
    }
    #[test]
    fn chord_quality_identification() {
        test_quality(SevenNoteScaleQuality::Major);
        test_quality(SevenNoteScaleQuality::MelodicMinor);
        test_quality(SevenNoteScaleQuality::MajorFlat9Flat13);
        test_quality(SevenNoteScaleQuality::MixolydianSharp11Flat13);
        test_quality(SevenNoteScaleQuality::HarmonicMinor);
        test_quality(SevenNoteScaleQuality::HarmonicMajor);
        test_quality(SevenNoteScaleQuality::MajorFlat9);
        test_quality(SevenNoteScaleQuality::MajorSharp9);
        test_quality(SevenNoteScaleQuality::MixolydianSharp9);
        test_quality(SevenNoteScaleQuality::LydianFlat9);
        test_quality(SevenNoteScaleQuality::MixolydianSharp9Sharp11);
        test_quality(SevenNoteScaleQuality::MixolydianFlat9Sharp11);
        test_quality(SevenNoteScaleQuality::MelodicMinorFlat9Sharp11);
        test_quality(SevenNoteScaleQuality::MixolydianFlat9Sharp11Flat13);
        test_quality(SevenNoteScaleQuality::MixolydianSharp9Flat13);
        test_quality(SevenNoteScaleQuality::MelodicMinorFlat11);
        test_quality(SevenNoteScaleQuality::MixolydianSharp9Sharp11Flat13);
        test_quality(SevenNoteScaleQuality::MelodicMinorFlat9Flat11);
        test_quality(SevenNoteScaleQuality::LydianFlat9Flat13);
        test_quality(SevenNoteScaleQuality::MelodicMinorFlat9Sharp11Flat13);
        test_quality(SevenNoteScaleQuality::LydianSharp9Flat13);
        test_quality(SevenNoteScaleQuality::MajorSharp9Flat13);
    }
}