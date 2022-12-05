use std::collections::HashSet;
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

/// Scale alteration.
#[derive(Debug, Clone, PartialEq)]
pub enum Alt2nd {
    Sharp,
    Natural,
    Flat
}

impl Alt2nd {
    pub fn from_pcs(pcs: &HashSet<Pc>) -> Vec<Self> {
        pcs
            .iter()
            .map(|pc| match pc {
                Pc1 => Some(Alt2nd::Flat),
                Pc2 => Some(Alt2nd::Natural),
                Pc3 => Some(Alt2nd::Sharp),
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn to_string_2nd(&self) -> String {
        match &self {
            Alt2nd::Sharp => "#2".to_string(),
            Alt2nd::Natural => "2".to_string(),
            Alt2nd::Flat => "b2".to_string(),
        }
    }

    pub fn to_string_9th(&self) -> String {
        match &self {
            Alt2nd::Sharp => "#9".to_string(),
            Alt2nd::Natural => "9".to_string(),
            Alt2nd::Flat => "b9".to_string(),
        }
    }
}

/// Scale alteration.
#[derive(Debug, Clone, PartialEq)]
pub enum Alt2ndMinor {
    Natural,
    Flat
}

impl Alt2ndMinor {
    pub fn from_pcs(pcs: &HashSet<Pc>) -> Vec<Self> {
        pcs
            .iter()
            .map(|pc| match pc {
                Pc1 => Some(Alt2ndMinor::Flat),
                Pc2 => Some(Alt2ndMinor::Natural),
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn to_string_2nd(&self) -> String {
        match &self {
            Alt2ndMinor::Natural => "2".to_string(),
            Alt2ndMinor::Flat => "b2".to_string(),
        }
    }

    pub fn to_string_9th(&self) -> String {
        match &self {
            Alt2ndMinor::Natural => "9".to_string(),
            Alt2ndMinor::Flat => "b9".to_string(),
        }
    }
}

/// Scale alteration.
#[derive(Debug, Clone, PartialEq)]
pub enum Alt4th {
    Sharp,
    Natural,
}

impl Alt4th {
    pub fn from_pcs(pcs: &HashSet<Pc>) -> Vec<Self> {
        pcs
            .iter()
            .map(|pc| match pc {
                Pc5 => Some(Alt4th::Natural),
                Pc6 => Some(Alt4th::Sharp),
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn to_string_4th(&self) -> String {
        match &self {
            Alt4th::Natural => "4".to_string(),
            Alt4th::Sharp => "#4".to_string(),
        }
    }

    pub fn to_string_11th(&self) -> String {
        match &self {
            Alt4th::Natural => "11".to_string(),
            Alt4th::Sharp => "#11".to_string(),
        }
    }
}

/// Scale alteration.
#[derive(Debug, Clone, PartialEq)]
pub enum Alt4thMinor {
    Sharp,
    Natural,
    Flat,
}

impl Alt4thMinor {
    pub fn from_pcs(pcs: &HashSet<Pc>) -> Vec<Self> {
        pcs
            .iter()
            .map(|pc| match pc {
                Pc4 => Some(Alt4thMinor::Flat),
                Pc5 => Some(Alt4thMinor::Natural),
                Pc6 => Some(Alt4thMinor::Sharp),
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn to_string_4th(&self) -> String {
        match &self {
            Alt4thMinor::Flat => "b4".to_string(),
            Alt4thMinor::Natural => "4".to_string(),
            Alt4thMinor::Sharp => "#4".to_string(),
        }
    }

    pub fn to_string_11th(&self) -> String {
        match &self {
            Alt4thMinor::Flat => "b4".to_string(),
            Alt4thMinor::Natural => "11".to_string(),
            Alt4thMinor::Sharp => "#11".to_string(),
        }
    }
}

/// Scale alteration.
#[derive(Debug, Clone, PartialEq)]
pub enum Alt6thMaj7 {
    Sharp,
    Natural,
    Flat,
}

impl Alt6thMaj7 {
    pub fn from_pcs(pcs: &HashSet<Pc>) -> Vec<Self> {
        pcs
            .iter()
            .map(|pc| match pc {
                Pc8 => Some(Alt6thMaj7::Flat),
                Pc9 => Some(Alt6thMaj7::Natural),
                Pc10 => Some(Alt6thMaj7::Sharp),
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn to_string_6th(&self) -> String {
        match &self {
            Alt6thMaj7::Flat => "b6".to_string(),
            Alt6thMaj7::Natural => "6".to_string(),
            Alt6thMaj7::Sharp => "#6".to_string(),
        }
    }

    pub fn to_string_13th(&self) -> String {
        match &self {
            Alt6thMaj7::Flat => "b13".to_string(),
            Alt6thMaj7::Natural => "13".to_string(),
            Alt6thMaj7::Sharp => "#13".to_string(),
        }
    }
}

/// Scale alteration.
#[derive(Debug, Clone, PartialEq)]
pub enum Alt6thDom7 {
    Natural,
    Flat,
}

impl Alt6thDom7 {
    pub fn from_pcs(pcs: &HashSet<Pc>) -> Vec<Self> {
        pcs
            .iter()
            .map(|pc| match pc {
                Pc8 => Some(Alt6thDom7::Flat),
                Pc9 => Some(Alt6thDom7::Natural),
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn to_string_6th(&self) -> String {
        match &self {
            Alt6thDom7::Flat => "b6".to_string(),
            Alt6thDom7::Natural => "6".to_string(),
        }
    }

    pub fn to_string_13th(&self) -> String {
        match &self {
            Alt6thDom7::Flat => "b13".to_string(),
            Alt6thDom7::Natural => "13".to_string(),
        }
    }
}

/// Scale alteration.
#[derive(Debug, Clone, PartialEq)]
pub enum Alt6thAugMaj7 {
    Sharp,
    Natural,
}

impl Alt6thAugMaj7 {
    pub fn from_pcs(pcs: &HashSet<Pc>) -> Vec<Self> {
        pcs
            .iter()
            .map(|pc| match pc {
                Pc9 => Some(Alt6thAugMaj7::Natural),
                Pc10 => Some(Alt6thAugMaj7::Sharp),
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn to_string_6th(&self) -> String {
        match &self {
            Alt6thAugMaj7::Natural => "6".to_string(),
            Alt6thAugMaj7::Sharp => "#6".to_string(),
        }
    }

    pub fn to_string_13th(&self) -> String {
        match &self {
            Alt6thAugMaj7::Natural => "13".to_string(),
            Alt6thAugMaj7::Sharp => "#13".to_string(),
        }
    }
}

/// The primary categories of scales, modes, which we can then further characterize
/// by alterations.
#[derive(Debug, Clone, PartialEq)]
pub enum ScaleQuality {
    Major(Vec<Alt2nd>, Vec<Alt6thMaj7>),
    IonianAug(Vec<Alt2nd>, Vec<Alt6thAugMaj7>),
    Dorian(Vec<Alt2ndMinor>, Vec<Alt4thMinor>),
    Phrygian(Vec<Alt4thMinor>),
    Lydian(Vec<Alt2nd>, Vec<Alt6thMaj7>),
    LydianAug(Vec<Alt2nd>, Vec<Alt6thAugMaj7>),
    Mixolydian(Vec<Alt2nd>, Vec<Alt4th>, Vec<Alt6thDom7>),
    MixolydianAug(Vec<Alt2nd>, Vec<Alt4th>),
    NaturalMinor(Vec<Alt4thMinor>),
    MelodicMinor(Vec<Alt2ndMinor>, Vec<Alt4thMinor>),
    HarmonicMajor,
    HarmonicMinor,
    Locrian(Vec<Alt2ndMinor>, Vec<Alt6thDom7>),
    Altered,
    // Six Notes
    WholeTone,
    AugAH,
    AugHA,
    // Eight Notes
    DimHW,
    DimWH,
    // TODO Major and minor pentatonic scale I guess?
    // TODO Any other scales to more-or-less manually index?
}
