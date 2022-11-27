use crate::chord::octave_partition::OctavePartition;
use crate::chord::pc_set::PcSet;
use crate::chord::three_note_chords::ThreeNoteChordQuality;
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;
use anyhow::anyhow;

// A chord quality for every possible four-note [PcSet].
#[derive(Debug, Clone, PartialEq)]
pub enum FourNoteChordQuality {
    // Seventh Chords
    Maj7,
    Dom7,
    Min7,
    MinMaj7,
    Dim7,
    Min7Flat5,
    Aug7,
    AugMaj7,
    Dom7Flat5,
    // Triads + 9th
    Maj9,
    MinFlat9,
    MajFlat9,
    MajSharp9,
    Min9,
    Dim9,
    DimFlat9,
    // Triads + 11th
    Maj11,
    MajSharp11,
    Min11,
    MinSharp11,
    Dim11,
    DimFlat11,
    // Stacked Fourths (P = Perfect fourth, A = Augmented fourth)
    PPP,
    APP,
    PAP,
    PPA,
    // Clusters (A = Augmented second, W = Wholetone, H = Semitone)
    WWW,
    HWW,
    WHW,
    WWH,
    HAH,
    AHH,
    HHA,
    HWH,
    WHH,
    HHW,
    HHM,
    MHH,
    HAW,
    WAH,
    HHH,
    // Perfect Fourths + Half steps (two of each)
    PHP,
    PPH,
}

pub const MAJ7_PCS: &[Pc] = &[Pc0, Pc4, Pc7, Pc11];
pub const DOM7_PCS: &[Pc] = &[Pc0, Pc4, Pc7, Pc10];
pub const MIN7_PCS: &[Pc] = &[Pc0, Pc3, Pc7, Pc10];
pub const MINMAJ7_PCS: &[Pc] = &[Pc0, Pc3, Pc7, Pc11];
pub const DIM7_PCS: &[Pc] = &[Pc0, Pc3, Pc6, Pc9];
pub const MIN7_FLAT5_PCS: &[Pc] = &[Pc0, Pc3, Pc6, Pc10];
pub const AUG7_PCS: &[Pc] = &[Pc0, Pc4, Pc8, Pc10];
pub const AUG_MAJ7_PCS: &[Pc] = &[Pc0, Pc4, Pc8, Pc11];
pub const DOM7_FLAT5_PCS: &[Pc] = &[Pc0, Pc4, Pc6, Pc10];
pub const MAJ9_PCS: &[Pc] = &[Pc0, Pc2, Pc4, Pc7];
pub const MAJ_SHARP9_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc7];
pub const MAJ_FLAT9_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc7];
pub const MIN9_PCS: &[Pc] = &[Pc0, Pc2, Pc3, Pc7];
pub const MIN_FLAT9_PCS: &[Pc] = &[Pc0, Pc1, Pc3, Pc7];
pub const DIM9_PCS: &[Pc] = &[Pc0, Pc2, Pc3, Pc6];
pub const DIM_FLAT9_PCS: &[Pc] = &[Pc0, Pc1, Pc3, Pc6];
pub const MAJ11_PCS: &[Pc] = &[Pc0, Pc4, Pc5, Pc7];
pub const MAJ_SHARP11_PCS: &[Pc] = &[Pc0, Pc4, Pc6, Pc7];
pub const MIN11_PCS: &[Pc] = &[Pc0, Pc3, Pc5, Pc7];
pub const MIN_SHARP11_PCS: &[Pc] = &[Pc0, Pc3, Pc6, Pc7];
pub const DIM11_PCS: &[Pc] = &[Pc0, Pc3, Pc5, Pc6];
pub const DIM_FLAT11_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc6];
pub const PPP_PCS: &[Pc] = &[Pc0, Pc3, Pc5, Pc10];
pub const APP_PCS: &[Pc] = &[Pc0, Pc4, Pc6, Pc11];
pub const PAP_PCS: &[Pc] = &[Pc0, Pc4, Pc5, Pc11];
pub const PPA_PCS: &[Pc] = &[Pc0, Pc4, Pc5, Pc10];
pub const WWW_PCS: &[Pc] = &[Pc0, Pc2, Pc4, Pc6];
pub const HWW_PCS: &[Pc] = &[Pc0, Pc1, Pc3, Pc5];
pub const WHW_PCS: &[Pc] = &[Pc0, Pc2, Pc3, Pc5];
pub const WWH_PCS: &[Pc] = &[Pc0, Pc2, Pc4, Pc5];
pub const HAH_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc5];
pub const AHH_PCS: &[Pc] = &[Pc0, Pc3, Pc4, Pc5];
pub const HHA_PCS: &[Pc] = &[Pc0, Pc1, Pc2, Pc5];
pub const HWH_PCS: &[Pc] = &[Pc0, Pc1, Pc3, Pc4];
pub const WHH_PCS: &[Pc] = &[Pc0, Pc2, Pc3, Pc4];
pub const HHW_PCS: &[Pc] = &[Pc0, Pc1, Pc2, Pc4];
pub const HHM_PCS: &[Pc] = &[Pc0, Pc1, Pc2, Pc6];
pub const MHH_PCS: &[Pc] = &[Pc0, Pc4, Pc5, Pc6];
pub const HAW_PCS: &[Pc] = &[Pc0, Pc1, Pc4, Pc6];
pub const WAH_PCS: &[Pc] = &[Pc0, Pc2, Pc5, Pc6];
pub const HHH_PCS: &[Pc] = &[Pc0, Pc1, Pc2, Pc3];
pub const PHP_PCS: &[Pc] = &[Pc0, Pc5, Pc6, Pc11];
pub const PPH_PCS: &[Pc] = &[Pc0, Pc5, Pc10, Pc11];

impl From<FourNoteChordQuality> for OctavePartition {
    fn from(value: FourNoteChordQuality) -> Self {
        OctavePartition::from(&value)
    }
}

impl From<&FourNoteChordQuality> for OctavePartition {
    fn from(value: &FourNoteChordQuality) -> Self {
        OctavePartition::from(match value {
            FourNoteChordQuality::Maj7 => MAJ7_PCS,
            FourNoteChordQuality::Dom7 => DOM7_PCS,
            FourNoteChordQuality::Min7 => MIN7_PCS,
            FourNoteChordQuality::MinMaj7 => MINMAJ7_PCS,
            FourNoteChordQuality::Dim7 => DIM7_PCS,
            FourNoteChordQuality::Min7Flat5 => MIN7_FLAT5_PCS,
            FourNoteChordQuality::Aug7 => AUG7_PCS,
            FourNoteChordQuality::AugMaj7 => AUG_MAJ7_PCS,
            FourNoteChordQuality::Dom7Flat5 => DOM7_FLAT5_PCS,
            FourNoteChordQuality::Maj9 => MAJ9_PCS,
            FourNoteChordQuality::MinFlat9 => MIN_FLAT9_PCS,
            FourNoteChordQuality::MajFlat9 => MAJ_FLAT9_PCS,
            FourNoteChordQuality::MajSharp9 => MAJ_SHARP9_PCS,
            FourNoteChordQuality::Min9 => MIN9_PCS,
            FourNoteChordQuality::Dim9 => DIM9_PCS,
            FourNoteChordQuality::DimFlat9 => DIM_FLAT9_PCS,
            FourNoteChordQuality::Maj11 => MAJ11_PCS,
            FourNoteChordQuality::MajSharp11 => MAJ_SHARP11_PCS,
            FourNoteChordQuality::Min11 => MIN11_PCS,
            FourNoteChordQuality::MinSharp11 => MIN_SHARP11_PCS,
            FourNoteChordQuality::Dim11 => DIM11_PCS,
            FourNoteChordQuality::DimFlat11 => DIM_FLAT11_PCS,
            FourNoteChordQuality::PPP => PPP_PCS,
            FourNoteChordQuality::APP => APP_PCS,
            FourNoteChordQuality::PAP => PAP_PCS,
            FourNoteChordQuality::PPA => PPA_PCS,
            FourNoteChordQuality::WWW => WWW_PCS,
            FourNoteChordQuality::HWW => HWW_PCS,
            FourNoteChordQuality::WHW => WHW_PCS,
            FourNoteChordQuality::WWH => WWH_PCS,
            FourNoteChordQuality::HAH => HAH_PCS,
            FourNoteChordQuality::AHH => AHH_PCS,
            FourNoteChordQuality::HHA => HHA_PCS,
            FourNoteChordQuality::HWH => HWH_PCS,
            FourNoteChordQuality::WHH => WHH_PCS,
            FourNoteChordQuality::HHW => HHW_PCS,
            FourNoteChordQuality::HHM => HHM_PCS,
            FourNoteChordQuality::MHH => MHH_PCS,
            FourNoteChordQuality::HAW => HAW_PCS,
            FourNoteChordQuality::WAH => WAH_PCS,
            FourNoteChordQuality::HHH => HHH_PCS,
            FourNoteChordQuality::PHP => PHP_PCS,
            FourNoteChordQuality::PPH => PPH_PCS,
        })
    }
}

impl TryFrom<&PcSet> for FourNoteChordQuality {
    type Error = anyhow::Error;

    fn try_from(value: &PcSet) -> Result<Self, Self::Error> {
        if value.0.len() != 4 {
            return Err(anyhow!("wrong size for four note chord: {:?}", value.0));
        }
        let pitches = value.0.as_slice();
        match pitches {
            MAJ7_PCS => Ok(FourNoteChordQuality::Maj7),
            DOM7_PCS => Ok(FourNoteChordQuality::Dom7),
            MIN7_PCS => Ok(FourNoteChordQuality::Min7),
            MINMAJ7_PCS => Ok(FourNoteChordQuality::MinMaj7),
            DIM7_PCS => Ok(FourNoteChordQuality::Dim7),
            MIN7_FLAT5_PCS => Ok(FourNoteChordQuality::Min7Flat5),
            AUG7_PCS => Ok(FourNoteChordQuality::Aug7),
            AUG_MAJ7_PCS => Ok(FourNoteChordQuality::AugMaj7),
            DOM7_FLAT5_PCS => Ok(FourNoteChordQuality::Dom7Flat5),
            MAJ9_PCS => Ok(FourNoteChordQuality::Maj9),
            MIN_FLAT9_PCS => Ok(FourNoteChordQuality::MinFlat9),
            MAJ_FLAT9_PCS => Ok(FourNoteChordQuality::MajFlat9),
            MAJ_SHARP9_PCS => Ok(FourNoteChordQuality::MajSharp9),
            MIN9_PCS => Ok(FourNoteChordQuality::Min9),
            DIM9_PCS => Ok(FourNoteChordQuality::Dim9),
            DIM_FLAT9_PCS => Ok(FourNoteChordQuality::DimFlat9),
            MAJ11_PCS => Ok(FourNoteChordQuality::Maj11),
            MAJ_SHARP11_PCS => Ok(FourNoteChordQuality::MajSharp11),
            MIN11_PCS => Ok(FourNoteChordQuality::Min11),
            MIN_SHARP11_PCS => Ok(FourNoteChordQuality::MinSharp11),
            DIM11_PCS => Ok(FourNoteChordQuality::Dim11),
            DIM_FLAT11_PCS => Ok(FourNoteChordQuality::DimFlat11),
            PPP_PCS => Ok(FourNoteChordQuality::PPP),
            APP_PCS => Ok(FourNoteChordQuality::APP),
            PAP_PCS => Ok(FourNoteChordQuality::PAP),
            PPA_PCS => Ok(FourNoteChordQuality::PPA),
            WWW_PCS => Ok(FourNoteChordQuality::WWW),
            HWW_PCS => Ok(FourNoteChordQuality::HWW),
            WHW_PCS => Ok(FourNoteChordQuality::WHW),
            WWH_PCS => Ok(FourNoteChordQuality::WWH),
            HAH_PCS => Ok(FourNoteChordQuality::HAH),
            AHH_PCS => Ok(FourNoteChordQuality::AHH),
            HHA_PCS => Ok(FourNoteChordQuality::HHA),
            HWH_PCS => Ok(FourNoteChordQuality::HWH),
            WHH_PCS => Ok(FourNoteChordQuality::WHH),
            HHW_PCS => Ok(FourNoteChordQuality::HHW),
            HHM_PCS => Ok(FourNoteChordQuality::HHM),
            MHH_PCS => Ok(FourNoteChordQuality::MHH),
            HAW_PCS => Ok(FourNoteChordQuality::HAW),
            WAH_PCS => Ok(FourNoteChordQuality::WAH),
            HHH_PCS => Ok(FourNoteChordQuality::HHH),
            PHP_PCS => Ok(FourNoteChordQuality::PHP),
            PPH_PCS => Ok(FourNoteChordQuality::PPH),
            _ => Err(anyhow!("4NC not recognized: {:?}", pitches)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Convert to and from intervallically descriptive types.
    fn test_quality(quality: FourNoteChordQuality) {
        let partition = OctavePartition::from(&quality);
        let pc_set = PcSet::from(&partition);
        assert_eq!(quality, FourNoteChordQuality::try_from(&pc_set).unwrap())
    }

    #[test]
    fn chord_quality_identification() {
        /// 7th chords
        test_quality(FourNoteChordQuality::Maj7);
        test_quality(FourNoteChordQuality::Dom7);
        test_quality(FourNoteChordQuality::Min7);
        test_quality(FourNoteChordQuality::MinMaj7);
        test_quality(FourNoteChordQuality::Dim7);
        test_quality(FourNoteChordQuality::Min7Flat5);
        test_quality(FourNoteChordQuality::Aug7);
        test_quality(FourNoteChordQuality::AugMaj7);
        test_quality(FourNoteChordQuality::Dom7Flat5);
        test_quality(FourNoteChordQuality::Maj9);
        test_quality(FourNoteChordQuality::MinFlat9);
        test_quality(FourNoteChordQuality::MajFlat9);
        test_quality(FourNoteChordQuality::MajSharp9);
        test_quality(FourNoteChordQuality::Min9);
        test_quality(FourNoteChordQuality::Dim9);
        test_quality(FourNoteChordQuality::DimFlat9);
        test_quality(FourNoteChordQuality::Maj11);
        test_quality(FourNoteChordQuality::MajSharp11);
        test_quality(FourNoteChordQuality::Min11);
        test_quality(FourNoteChordQuality::MinSharp11);
        test_quality(FourNoteChordQuality::Dim11);

        // FourNoteChordQuality::Dim11 => DIM11_PCS,
        // FourNoteChordQuality::DimFlat11 => DIM_FLAT11_PCS,
        // FourNoteChordQuality::PPP => PPP_PCS,
        // FourNoteChordQuality::APP => APP_PCS,
        // FourNoteChordQuality::PAP => PAP_PCS,
        // FourNoteChordQuality::PPA => PPA_PCS,
        // FourNoteChordQuality::WWW => WWW_PCS,
        // FourNoteChordQuality::HWW => HWW_PCS,
        // FourNoteChordQuality::WHW => WHW_PCS,
        // FourNoteChordQuality::WWH => WWH_PCS,
        // FourNoteChordQuality::HAH => HAH_PCS,
        // FourNoteChordQuality::AHH => AHH_PCS,
        // FourNoteChordQuality::HHA => HHA_PCS,
        // FourNoteChordQuality::HWH => HWH_PCS,
        // FourNoteChordQuality::WHH => WHH_PCS,
        // FourNoteChordQuality::HHW => HHW_PCS,
        // FourNoteChordQuality::HHM => HHM_PCS,
        // FourNoteChordQuality::MHH => MHH_PCS,
        // FourNoteChordQuality::HAW => HAW_PCS,
        // FourNoteChordQuality::WAH => WAH_PCS,
        // FourNoteChordQuality::HHH => HHH_PCS,
        // FourNoteChordQuality::PHP => PHP_PCS,
        // FourNoteChordQuality::PPH => PPH_PCS,
    }
}