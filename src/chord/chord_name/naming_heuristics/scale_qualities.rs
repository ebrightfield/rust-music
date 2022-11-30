use std::collections::HashSet;
use once_cell::sync::Lazy;
use crate::chord::chord_name::naming_heuristics::maj_and_min_qualities::search_for_maj_min_quality;
use crate::chord::chord_name::naming_heuristics::NamingHeuristic;
use crate::chord::chord_name::quality::ChordQuality;
use crate::chord::chord_name::quality::scale::{Alt2nd, Alt2ndMinor, Alt4th, Alt4thMinor, Alt6thAugMaj7, Alt6thDom7, Alt6thMaj7, ScaleQuality};
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

static WT_SCALE_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc2, Pc4, Pc6, Pc8, Pc10])
});
#[derive(Debug)]
pub struct WholetoneScale;
impl NamingHeuristic for WholetoneScale {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *HARMONIC_MAJOR_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::HarmonicMajor)
    }
}

static AUG_AH_SCALE_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc3, Pc4, Pc7, Pc8, Pc11])
});
#[derive(Debug)]
pub struct AugAHScale;
impl NamingHeuristic for AugAHScale {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *AUG_AH_SCALE_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::AugAH)
    }
}

static AUG_HA_SCALE_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc1, Pc4, Pc5, Pc8, Pc9])
});
#[derive(Debug)]
pub struct AugHAScale;
impl NamingHeuristic for AugHAScale {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *AUG_HA_SCALE_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::AugHA)
    }
}

static DIM_HW_SCALE_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc1, Pc3, Pc4, Pc6, Pc7, Pc9, Pc10])
});
#[derive(Debug)]
pub struct DimHWScale;
impl NamingHeuristic for DimHWScale {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *DIM_HW_SCALE_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::DimHW)
    }
}

static DIM_WH_SCALE_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc2, Pc3, Pc5, Pc6, Pc8, Pc9, Pc11])
});
#[derive(Debug)]
pub struct DimWHScale;
impl NamingHeuristic for DimWHScale {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *DIM_WH_SCALE_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::DimWH)
    }
}

static HARMONIC_MINOR_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc2, Pc3, Pc5, Pc7, Pc8, Pc11])
});
#[derive(Debug)]
pub struct HarmonicMinor;
impl NamingHeuristic for HarmonicMinor {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *HARMONIC_MINOR_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::HarmonicMinor)
    }
}

static HARMONIC_MAJOR_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc2, Pc4, Pc5, Pc7, Pc8, Pc11])
});
#[derive(Debug)]
pub struct HarmonicMajor;
impl NamingHeuristic for HarmonicMajor {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *HARMONIC_MAJOR_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::HarmonicMajor)
    }
}

static ALTERED_NO_ROOT: Lazy<HashSet<Pc>> = Lazy::new(|| {
    HashSet::from([Pc1, Pc3, Pc4, Pc6, Pc8, Pc10])
});
#[derive(Debug)]
pub struct AlteredScale;
impl NamingHeuristic for AlteredScale {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        *pcs == *ALTERED_NO_ROOT
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        Some(ScaleQuality::Altered)
    }
}


#[derive(Debug)]
pub struct MajorScale;
impl NamingHeuristic for MajorScale {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc4, Pc5, Pc7, Pc11].iter().all(|pc| pcs.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let mut seconds = Alt2nd::from_pcs(pcs);
        let mut sixths = Alt6thMaj7::from_pcs(pcs);
        if seconds == vec![Alt2nd::Natural] {
            seconds = vec![];
        }
        if sixths == vec![Alt6thMaj7::Natural] {
            sixths = vec![];
        }
        Some(ScaleQuality::Major(seconds, sixths))
    }
}

#[derive(Debug)]
pub struct IonianAug;
impl NamingHeuristic for IonianAug {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc4, Pc5, Pc8, Pc11].iter().all(|pc| pcs.contains(pc)) &&
            !pcs.contains(&Pc7) && !pcs.contains(&Pc6)
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let seconds = Alt2nd::from_pcs(pcs);
        let sixths = Alt6thAugMaj7::from_pcs(pcs);
        Some(ScaleQuality::IonianAug(seconds, sixths))
    }
}

#[derive(Debug)]
pub struct Dorian;
impl NamingHeuristic for Dorian {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc3, Pc7, Pc9, Pc10].iter().all(|pc| pcs.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let mut seconds = Alt2ndMinor::from_pcs(pcs);
        let mut fourths = Alt4thMinor::from_pcs(pcs);
        if seconds == vec![Alt2ndMinor::Natural] {
            seconds = vec![];
        }
        if fourths == vec![Alt4thMinor::Natural] {
            fourths = vec![];
        }
        Some(ScaleQuality::Dorian(seconds, fourths))
    }
}

#[derive(Debug)]
pub struct Phrygian;
impl NamingHeuristic for Phrygian {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc1, Pc3, Pc7, Pc8, Pc10].iter().all(|pc| pcs.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let mut fourths = Alt4thMinor::from_pcs(pcs);
        if fourths == vec![Alt4thMinor::Natural] {
            fourths = vec![];
        }
        Some(ScaleQuality::Phrygian(fourths))
    }
}

#[derive(Debug)]
pub struct Lydian;
impl NamingHeuristic for Lydian {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc4, Pc6, Pc7, Pc11].iter().all(|pc| pcs.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let mut seconds = Alt2nd::from_pcs(pcs);
        let mut sixths = Alt6thMaj7::from_pcs(pcs);
        if seconds == vec![Alt2nd::Natural] {
            seconds = vec![];
        }
        if sixths == vec![Alt6thMaj7::Natural] {
            sixths = vec![];
        }
        Some(ScaleQuality::Lydian(seconds, sixths))
    }
}

#[derive(Debug)]
pub struct LydianAug;
impl NamingHeuristic for LydianAug {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc4, Pc6, Pc8, Pc11].iter().all(|pc| pcs.contains(pc)) &&
            !pcs.contains(&Pc7) && !pcs.contains(&Pc6)
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let seconds = Alt2nd::from_pcs(pcs);
        let sixths = Alt6thAugMaj7::from_pcs(pcs);
        Some(ScaleQuality::LydianAug(seconds, sixths))
    }
}


#[derive(Debug)]
pub struct Mixolydian;
impl NamingHeuristic for Mixolydian {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc4, Pc7, Pc10].iter().all(|pc| pcs.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let seconds = Alt2nd::from_pcs(pcs);
        let fourths = Alt4th::from_pcs(pcs);
        let sixths = Alt6thDom7::from_pcs(pcs);
        Some(ScaleQuality::Mixolydian(seconds, fourths, sixths))
    }
}

#[derive(Debug)]
pub struct MixolydianAug;
impl NamingHeuristic for MixolydianAug {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc4, Pc8, Pc10].iter().all(|pc| pcs.contains(pc)) &&
            !pcs.contains(&Pc7) && !pcs.contains(&Pc11)
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let seconds = Alt2nd::from_pcs(pcs);
        let fourths = Alt4th::from_pcs(pcs);
        Some(ScaleQuality::MixolydianAug(seconds, fourths))
    }
}

#[derive(Debug)]
pub struct NaturalMinor;
impl NamingHeuristic for NaturalMinor {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc2, Pc3, Pc7, Pc8, Pc10].iter().all(|pc| pcs.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let fourths = Alt4thMinor::from_pcs(pcs);
        Some(ScaleQuality::NaturalMinor(fourths))
    }
}

#[derive(Debug)]
pub struct MelodicMinor;
impl NamingHeuristic for MelodicMinor {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc3, Pc7, Pc9, Pc11].iter().all(|pc| pcs.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let seconds = Alt2ndMinor::from_pcs(pcs);
        let fourths = Alt4thMinor::from_pcs(pcs);
        Some(ScaleQuality::MelodicMinor(seconds, fourths))
    }
}

#[derive(Debug)]
pub struct Locrian;
impl NamingHeuristic for Locrian {
    type T = ScaleQuality;

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        [Pc3, Pc6, Pc10].iter().all(|pc| pcs.contains(pc)) &&
            !pcs.contains(&Pc7) && !pcs.contains(&Pc9) && !pcs.contains(&Pc11)
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ScaleQuality> {
        let mut seconds = Alt2ndMinor::from_pcs(pcs);
        let mut sixths = Alt6thDom7::from_pcs(pcs);
        // If we only found what's expected, then no need to keep, those alterations.
        if seconds == vec![Alt2ndMinor::Flat] {
            seconds = vec![];
        }
        if sixths == vec![Alt6thDom7::Flat] {
            sixths = vec![];
        }
        Some(ScaleQuality::Locrian(seconds, sixths))
    }
}
