use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use std::ops::{Deref, DerefMut};
use crate::note_collections::chord_name::{ChordNameDisplayConfig, ExtensionStyle};
use crate::note_collections::interval_class::IntervalClass;

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

impl Display for AltChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            AltChoice::FlatNine => "b9".to_string(),
            AltChoice::Nine => "9".to_string(),
            AltChoice::SharpNine => "#9".to_string(),
            AltChoice::FlatEleven => "b11".to_string(),
            AltChoice::Eleven => "11".to_string(),
            AltChoice::SharpEleven => "#11".to_string(),
            AltChoice::FlatThirteenth => "b13".to_string(),
            AltChoice::Thirteenth => "13".to_string(),
            AltChoice::SharpThirteenth => "#13".to_string(),
        })
    }
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

impl Display for Alt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "");
        }
        let s: Vec<_> = self
            .iter()
            .map(|alteration| alteration.to_string())
            .collect();
        write!(f, "{}", "(".to_string() + &s.join(", ") + ")")
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extension {
    Seventh,
    Ninth,
    Eleventh,
    Thirteenth,
}

impl Extension {
    /// Naming configurations make various assumptions
    /// about what extensions are "implied" by the chord name.
    /// Anything else is converted to an [AltChoice] with this method.
    pub fn to_alt_choice(&self) -> Option<AltChoice> {
        match self {
            Extension::Seventh => None,
            Extension::Ninth => Some(AltChoice::Nine),
            Extension::Eleventh => Some(AltChoice::Eleven),
            Extension::Thirteenth => Some(AltChoice::Thirteenth),
        }
    }
}

impl Extension {
    pub fn to_string(&self) -> String {
        match &self {
            Extension::Seventh => "7".to_string(),
            Extension::Ninth => "9".to_string(),
            Extension::Eleventh => "11".to_string(),
            Extension::Thirteenth => "13".to_string(),
        }
    }
}

fn pick_strict_extension(ext: &Vec<Extension>) -> (Extension, Vec<Extension>) {
    let mut remainder = ext.clone();
    if ext.contains(&Extension::Thirteenth)
        && ext.contains(&Extension::Eleventh)
        && ext.contains(&Extension::Ninth)
    {
        remainder.retain(|e| *e != Extension::Thirteenth);
        return (Extension::Thirteenth, remainder);
    }
    if ext.contains(&Extension::Eleventh)
        && ext.contains(&Extension::Ninth)
    {
        remainder.retain(|e| *e != Extension::Eleventh);
        return (Extension::Eleventh, remainder);
    }
    if ext.contains(&Extension::Ninth)
    {
        remainder.retain(|e| *e != Extension::Ninth);
        return (Extension::Ninth, remainder);
    }
    (Extension::Seventh, remainder)
}

// Pick the highest extension.
fn pick_highest_extension(ext: &Vec<Extension>) -> (Extension, Vec<Extension>) {
    let mut remainder = ext.clone();
    if ext.contains(&Extension::Thirteenth) {
        remainder.retain(|e| *e != Extension::Thirteenth);
        return (Extension::Thirteenth, remainder);
    }
    if ext.contains(&Extension::Eleventh) {
        remainder.retain(|e| *e != Extension::Eleventh);
        return (Extension::Eleventh, remainder);
    }
    if ext.contains(&Extension::Ninth) {
        remainder.retain(|e| *e != Extension::Ninth);
        return (Extension::Ninth, remainder);
    }
    // Returning an empty vec, since remainder logically can't have
    // anything other than sevenths.
    (Extension::Seventh, vec![])
}

pub fn resolve_extension(
    ext: &Vec<Extension>,
    style: ExtensionStyle,
) -> (Extension, Vec<AltChoice>) {
    let to_alts = |exts: &Vec<Extension>| {
        return exts.iter()
            .map(|e| e.to_alt_choice())
            .into_iter()
            .flatten()
            .collect::<Vec<AltChoice>>();
    };
    match style {
        ExtensionStyle::None => {
            (Extension::Seventh, to_alts(ext))
        }
        ExtensionStyle::Strict => {
            let (ext, remainder) = pick_strict_extension(ext);
            (ext, to_alts(&remainder))
        }
        ExtensionStyle::Highest => {
            let (ext, remainder) = pick_highest_extension(ext);
            (ext, to_alts(&remainder))
        }
        ExtensionStyle::HighestUnlessOne => {
            if ext.len() > 1 {
                let (ext, remainder) = pick_highest_extension(ext);
                return (ext, to_alts(&remainder));
            }
            (Extension::Seventh, to_alts(ext))
        }
    }
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

impl ChordQuality {
    pub fn to_string(&self, cfg: &ChordNameDisplayConfig) -> String {
        let style = cfg.extension_style;
        let ext_and_alts = |alt: &Alt, ext: &Vec<Extension>, style| {
            let (ext, mut alts) = resolve_extension(ext, style);
            alts.extend(alt.0.clone());
            (ext, Alt::from(alts))
        };
        match &self {
            ChordQuality::Major(subtype) => {
                match subtype {
                    MajorSubtype::Maj(alt) => {
                        format!("Maj {}", alt.to_string())
                    }
                    MajorSubtype::Maj6(alt) => {
                        format!("Maj {}", alt.to_string())
                    }
                    MajorSubtype::MajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("Maj{} {}", ext.to_string(), alt.to_string())
                    }
                    MajorSubtype::N(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{} {}", ext.to_string(), alt.to_string())
                    }
                }
            },
            ChordQuality::Minor(subtype) => {
                match subtype {
                    MinorSubtype::Min(alt) => {
                        format!("min {}", alt.to_string())
                    }
                    MinorSubtype::Min6(alt) => {
                        format!("min {}", alt.to_string())
                    }
                    MinorSubtype::MinMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("minMaj{} {}", ext.to_string(), alt.to_string())
                    }
                    MinorSubtype::MinN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("min{} {}", ext.to_string(), alt.to_string())
                    }
                }
            },
            ChordQuality::Aug(subtype) => {
                match subtype {
                    AugSubtype::Aug(alt) => {
                        format!("Aug {}", alt.to_string())
                    }
                    AugSubtype::AugMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("+Maj{} {}", ext.to_string(), alt.to_string())
                    }
                    AugSubtype::AugN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("+{} {}", ext.to_string(), alt.to_string())
                    }
                }
            },
            ChordQuality::Dim(subtype) => {
                match subtype {
                    DimSubtype::Dim(alt) => {
                        format!("dim {}", alt.to_string())
                    }
                    DimSubtype::MinNb5(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("min{}b5 {}", ext.to_string(), alt.to_string())
                    }
                    DimSubtype::DimN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("dim{} {}", ext.to_string(), alt.to_string())
                    }
                    DimSubtype::DimMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("dimMaj{} {}", ext.to_string(), alt.to_string())
                    }
                }
            },
            ChordQuality::Sus(subtype) => {
                match subtype {
                    SusSubtype::Sus2(alt) => {
                        format!("sus2 {}", alt.to_string())
                    }
                    SusSubtype::Sus4(alt) => {
                        format!("sus4 {}", alt.to_string())
                    }
                    SusSubtype::DomNSus(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}sus {}", ext.to_string(), alt.to_string())
                    }
                    SusSubtype::MajNSus(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("Maj{}sus {}", ext.to_string(), alt.to_string())
                    }
                    SusSubtype::SixNineSus(alt) => {
                        format!("6/9sus {}", alt.to_string())
                    }
                }
            },
            ChordQuality::Interval(ic) => ic.to_string(),
            ChordQuality::SingleNote => "note".to_owned(),
        }.trim().to_string()
    }
}