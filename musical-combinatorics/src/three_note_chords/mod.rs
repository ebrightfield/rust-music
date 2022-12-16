use music::note_collections::octave_partition::OctavePartition;
use music::note_collections::pc_set::PcSet;
use music::note::pitch_class::Pc;
use music::note::pitch_class::Pc::*;
use anyhow::anyhow;
use crate::canonical_voicings::CanonicalVoicings;

/// The various possible octave partitions with three notes.
///
/// This type is exhaustive. Any combination of
/// three unique notes (not enharmonic) will correspond to
/// exactly one of these variants, at a particular mode.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreeNoteChordQuality {
    Major,
    Minor,
    Aug,
    Dim,
    PP,
    AP,
    PA,
    MW,
    WM,
    MH,
    HM,
    AW,
    WA,
    HA,
    AH,
    WW,
    WH,
    HW,
    HH,
}

impl ThreeNoteChordQuality {
    pub fn identify(pcs: &PcSet) -> anyhow::Result<(usize, ThreeNoteChordQuality)> {
        if pcs.len() != 3 {
            return Err(anyhow!("wrong size for three note chord: {:?}", pcs));
        }
        // try the first mode
        if let Ok(quality) = ThreeNoteChordQuality::try_from(pcs) {
            return Ok((0, quality));
        }
        // then try the other two, rotating each time
        let mut copied = pcs.clone();
        for i in 0usize..2 {
            copied = copied.rotate_fwd();
            if let Ok(quality) = ThreeNoteChordQuality::try_from(&copied) {
                return Ok((i+1, quality));
            }
        }
        // this chord quality is combinatorically exhaustive, it should always
        // find a quality at inversion 0, 1, or 2.
        unreachable!()
    }
}

// Canonical "root position" spellings,
// or spellings that clearly express the interval-based names.
pub const MAJOR_PCS: &[Pc] = &[Pc0, Pc4, Pc7];
pub const MINOR_PCS: &[Pc] = &[Pc0, Pc3, Pc7];
pub const AUG_PCS: &[Pc] = &[Pc0, Pc4, Pc8];
pub const DIM_PCS: &[Pc] = &[Pc0, Pc3, Pc6];
pub const PP_PCS: &[Pc] = &[Pc0, Pc5, Pc7];
pub const AP_PCS: &[Pc] = &[Pc0, Pc1, Pc7];
pub const PA_PCS: &[Pc] = &[Pc0, Pc6, Pc7];
pub const MW_PCS: &[Pc] = &[Pc0, Pc4, Pc6];
pub const WM_PCS: &[Pc] = &[Pc0, Pc2, Pc6];
pub const MH_PCS: &[Pc] = &[Pc0, Pc4, Pc5];
pub const HM_PCS: &[Pc] = &[Pc0, Pc1, Pc5];
pub const AW_PCS: &[Pc] = &[Pc0, Pc3, Pc5];
pub const WA_PCS: &[Pc] = &[Pc0, Pc2, Pc5];
pub const AH_PCS: &[Pc] = &[Pc0, Pc3, Pc4];
pub const HA_PCS: &[Pc] = &[Pc0, Pc1, Pc4];
pub const WW_PCS: &[Pc] = &[Pc0, Pc2, Pc4];
pub const WH_PCS: &[Pc] = &[Pc0, Pc2, Pc3];
pub const HW_PCS: &[Pc] = &[Pc0, Pc1, Pc3];
pub const HH_PCS: &[Pc] = &[Pc0, Pc1, Pc2];


impl From<&ThreeNoteChordQuality> for OctavePartition {
    fn from(value: &ThreeNoteChordQuality) -> Self {
        OctavePartition::from(match value {
            ThreeNoteChordQuality::Major => MAJOR_PCS,
            ThreeNoteChordQuality::Minor => MINOR_PCS,
            ThreeNoteChordQuality::Aug => AUG_PCS,
            ThreeNoteChordQuality::Dim => DIM_PCS,
            ThreeNoteChordQuality::PP => PP_PCS,
            ThreeNoteChordQuality::AP => AP_PCS,
            ThreeNoteChordQuality::PA => PA_PCS,
            ThreeNoteChordQuality::MW => MW_PCS,
            ThreeNoteChordQuality::WM => WM_PCS,
            ThreeNoteChordQuality::MH => MH_PCS,
            ThreeNoteChordQuality::HM => HM_PCS,
            ThreeNoteChordQuality::AW => AW_PCS,
            ThreeNoteChordQuality::WA => WA_PCS,
            ThreeNoteChordQuality::HA => HA_PCS,
            ThreeNoteChordQuality::AH => AH_PCS,
            ThreeNoteChordQuality::WW => WW_PCS,
            ThreeNoteChordQuality::WH => WH_PCS,
            ThreeNoteChordQuality::HW => HW_PCS,
            ThreeNoteChordQuality::HH => HH_PCS,
        })
    }
}

impl From<ThreeNoteChordQuality> for OctavePartition {
    fn from(value: ThreeNoteChordQuality) -> Self {
        OctavePartition::from(&value)
    }
}

impl TryFrom<&PcSet> for ThreeNoteChordQuality {
    type Error = anyhow::Error;

    fn try_from(value: &PcSet) -> Result<Self, Self::Error> {
        if value.len() != 3 {
            return Err(anyhow!("wrong size for three note chord: {:?}", value));
        }
        let pitches = value.as_slice();
        match pitches {
            MAJOR_PCS => Ok(ThreeNoteChordQuality::Major),
            MINOR_PCS => Ok(ThreeNoteChordQuality::Minor),
            AUG_PCS => Ok(ThreeNoteChordQuality::Aug),
            DIM_PCS => Ok(ThreeNoteChordQuality::Dim),
            PP_PCS => Ok(ThreeNoteChordQuality::PP),
            AP_PCS => Ok(ThreeNoteChordQuality::AP),
            PA_PCS => Ok(ThreeNoteChordQuality::PA),
            MW_PCS => Ok(ThreeNoteChordQuality::MW),
            WM_PCS => Ok(ThreeNoteChordQuality::WM),
            HM_PCS => Ok(ThreeNoteChordQuality::HM),
            MH_PCS => Ok(ThreeNoteChordQuality::MH),
            WA_PCS => Ok(ThreeNoteChordQuality::WA),
            AW_PCS => Ok(ThreeNoteChordQuality::AW),
            HA_PCS => Ok(ThreeNoteChordQuality::HA),
            AH_PCS => Ok(ThreeNoteChordQuality::AH),
            WW_PCS => Ok(ThreeNoteChordQuality::WW),
            WH_PCS => Ok(ThreeNoteChordQuality::WH),
            HW_PCS => Ok(ThreeNoteChordQuality::HW),
            HH_PCS => Ok(ThreeNoteChordQuality::HH),
            _ => Err(anyhow!("3NC not recognized: {:?}", value)),
        }
    }
}

impl CanonicalVoicings for ThreeNoteChordQuality {
    const N: usize = 3;
    const FAMILIES: &'static [&'static[usize]] = &[&[0,1,2], &[0,2,1]];
}

// TODO Should this be something more like a vector of possible qualities, one on each mode?
// impl From<ThreeNoteChordQuality> for ChordQuality {
//     fn from(chord_quality: ThreeNoteChordQuality) -> Self {
//         match chord_quality {
//             ThreeNoteChordQuality::Major => ChordQuality::Major(MajorSubtype::Major(None)),
//             ThreeNoteChordQuality::Minor => ChordQuality::Minor(MinorSubtype::Min(None)),
//             ThreeNoteChordQuality::Aug => ChordQuality::Aug(AugSubtype::Aug(None)),
//             ThreeNoteChordQuality::Dim => ChordQuality::Dim(DimSubtype::Dim(None)),
//             ThreeNoteChordQuality::PP => {},
//             ThreeNoteChordQuality::AP => {},
//             ThreeNoteChordQuality::PA => {},
//             ThreeNoteChordQuality::MW => {},
//             ThreeNoteChordQuality::WM => {},
//             ThreeNoteChordQuality::MH => {},
//             ThreeNoteChordQuality::HM => {},
//             ThreeNoteChordQuality::AW => {},
//             ThreeNoteChordQuality::WA => {},
//             ThreeNoteChordQuality::HA => {},
//             ThreeNoteChordQuality::AH => {},
//             ThreeNoteChordQuality::WW => {},
//             ThreeNoteChordQuality::WH => {},
//             ThreeNoteChordQuality::HW => {},
//             ThreeNoteChordQuality::HH => {},
//         }
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;
    use music::note::Note;

    // Convert to and from intervallically descriptive types.
    fn test_quality(quality: ThreeNoteChordQuality) {
        let partition = OctavePartition::from(&quality);
        let pc_set = PcSet::from(&partition);
        assert_eq!(quality, ThreeNoteChordQuality::try_from(&pc_set).unwrap())
    }

    #[test]
    fn chord_quality_identification() {
        test_quality(ThreeNoteChordQuality::Major);
        test_quality(ThreeNoteChordQuality::Minor);
        test_quality(ThreeNoteChordQuality::Aug);
        test_quality(ThreeNoteChordQuality::Dim);
        test_quality(ThreeNoteChordQuality::PP);
        test_quality(ThreeNoteChordQuality::AP);
        test_quality(ThreeNoteChordQuality::PA);
        test_quality(ThreeNoteChordQuality::MW);
        test_quality(ThreeNoteChordQuality::WM);
        test_quality(ThreeNoteChordQuality::MH);
        test_quality(ThreeNoteChordQuality::HM);
        test_quality(ThreeNoteChordQuality::WA);
        test_quality(ThreeNoteChordQuality::AW);
        test_quality(ThreeNoteChordQuality::HA);
        test_quality(ThreeNoteChordQuality::AH);
        test_quality(ThreeNoteChordQuality::WW);
        test_quality(ThreeNoteChordQuality::WH);
        test_quality(ThreeNoteChordQuality::HW);
        test_quality(ThreeNoteChordQuality::HH);
    }

    #[test]
    fn voicings_3nc() {
        let notes = vec![Note::C, Note::E, Note::G];
        let _ = ThreeNoteChordQuality::voicings(&notes);
        //println!("{:#?}", voicings);
    }
}
