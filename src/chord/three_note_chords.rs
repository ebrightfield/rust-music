use crate::chord::octave_partition::OctavePartition;
use crate::chord::pc_set::PcSet;
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;
use anyhow::anyhow;

/// The various possible octave partitions with three notes.
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

impl From<ThreeNoteChordQuality> for OctavePartition {
    fn from(chord_quality: ThreeNoteChordQuality) -> Self {
        match chord_quality {
            ThreeNoteChordQuality::Major => OctavePartition::from(MAJOR_PCS),
            ThreeNoteChordQuality::Minor => OctavePartition::from(MINOR_PCS),
            ThreeNoteChordQuality::Aug => OctavePartition::from(AUG_PCS),
            ThreeNoteChordQuality::Dim => OctavePartition::from(DIM_PCS),
            ThreeNoteChordQuality::PP => OctavePartition::from(PP_PCS),
            ThreeNoteChordQuality::AP => OctavePartition::from(AP_PCS),
            ThreeNoteChordQuality::PA => OctavePartition::from(PA_PCS),
            ThreeNoteChordQuality::MW => OctavePartition::from(MW_PCS),
            ThreeNoteChordQuality::WM => OctavePartition::from(WM_PCS),
            ThreeNoteChordQuality::MH => OctavePartition::from(MH_PCS),
            ThreeNoteChordQuality::HM => OctavePartition::from(HM_PCS),
            ThreeNoteChordQuality::AW => OctavePartition::from(AW_PCS),
            ThreeNoteChordQuality::WA => OctavePartition::from(WA_PCS),
            ThreeNoteChordQuality::HA => OctavePartition::from(HA_PCS),
            ThreeNoteChordQuality::AH => OctavePartition::from(AH_PCS),
            ThreeNoteChordQuality::WW => OctavePartition::from(WW_PCS),
            ThreeNoteChordQuality::WH => OctavePartition::from(WH_PCS),
            ThreeNoteChordQuality::HW => OctavePartition::from(HW_PCS),
            ThreeNoteChordQuality::HH => OctavePartition::from(HH_PCS),
        }
    }
}

impl TryFrom<&PcSet> for ThreeNoteChordQuality {
    type Error = anyhow::Error;

    fn try_from(value: &PcSet) -> Result<Self, Self::Error> {
        if value.0.len() != 3 {
            return Err(anyhow!("wrong size for three note chord: {:?}", value.0));
        }
        let pitches = value.0.as_slice();
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
            _ => Err(anyhow!("3NC not recognized: {:?}", value.0)),
        }
    }
}
