use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use crate::error::MusicSemanticsError;
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
    type Error = MusicSemanticsError;

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
            _ => Err(MusicSemanticsError::PcNotAnAlteration(value))
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

impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Extension::Seventh => write!(f, "7"),
            Extension::Ninth => write!(f, "9"),
            Extension::Eleventh => write!(f, "11"),
            Extension::Thirteenth => write!(f, "13"),
        }
    }
}

fn pick_strict_extension(ext: &[Extension]) -> (Extension, Vec<Extension>) {
    let mut remainder = ext.to_vec();
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
fn pick_highest_extension(ext: &[Extension]) -> (Extension, Vec<Extension>) {
    let mut remainder = ext.to_vec();
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
    ext: &[Extension],
    style: ExtensionStyle,
) -> (Extension, Vec<AltChoice>) {
    let to_alts = |exts: &[Extension]| {
        exts.iter()
            .map(|e| e.to_alt_choice())
            .flatten()
            .collect::<Vec<AltChoice>>()
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

/// Represents ambiguity in chord naming analysis.
///
/// Certain pitch class combinations don't map cleanly to a single
/// chord interpretation. This enum captures those ambiguous cases.
#[derive(Debug, Clone, PartialEq)]
pub enum QualityAmbiguity {
    /// A chord contains two versions of the same scale degree.
    /// For example, both natural 11 (F) and #11 (F#) over a C chord.
    /// The `degree` is the scale degree (e.g., 11), and `intervals`
    /// contains the semitone distances from the root for each version.
    DuplicateScaleDegree {
        /// The scale degree in question (e.g., 3, 5, 9, 11, 13)
        degree: u8,
        /// The semitone intervals from the root that map to this degree
        intervals: Vec<u8>,
    },
    /// The chord could be interpreted as multiple different qualities.
    /// For example, Am7 is also C6/A in certain contexts.
    MultipleInterpretations(Vec<String>),
    /// A 6th chord that might be confused with a 13th chord.
    /// Records whether a 7th is present (which would make it a true extended chord).
    SixthVsThirteenth {
        has_seventh: bool,
    },
}

impl QualityAmbiguity {
    /// Check if a set of intervals (from root) contains duplicate scale degrees.
    /// Returns a list of ambiguities found.
    pub fn find_duplicate_degrees(intervals: &[u8]) -> Vec<Self> {
        use std::collections::HashMap;
        let mut degree_map: HashMap<u8, Vec<u8>> = HashMap::new();

        for &interval in intervals {
            let degree = Self::interval_to_degree(interval);
            degree_map.entry(degree).or_default().push(interval);
        }

        degree_map
            .into_iter()
            .filter(|(_, intervals)| intervals.len() > 1)
            .map(|(degree, intervals)| QualityAmbiguity::DuplicateScaleDegree { degree, intervals })
            .collect()
    }

    /// Maps a semitone interval to its scale degree.
    /// Note: This is a simplified mapping that groups enharmonic equivalents.
    fn interval_to_degree(interval: u8) -> u8 {
        match interval % 12 {
            0 => 1,       // Root
            1 | 2 => 9,   // b9, 9
            3 | 4 => 3,   // b3, 3
            5 | 6 => 11,  // 11, #11
            7 => 5,       // 5
            8 | 9 => 13,  // b13/b6, 13/6
            10 | 11 => 7, // b7, 7
            _ => unreachable!(),
        }
    }
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
        let ext_and_alts = |alt: &Alt, ext: &[Extension], style| {
            let (ext, mut alts) = resolve_extension(ext, style);
            alts.extend(alt.0.clone());
            (ext, Alt::from(alts))
        };
        match &self {
            ChordQuality::Major(subtype) => {
                match subtype {
                    MajorSubtype::Maj(alt) => {
                        format!("Maj {}", alt)
                    }
                    MajorSubtype::Maj6(alt) => {
                        format!("Maj {}", alt)
                    }
                    MajorSubtype::MajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("Maj{} {}", ext, alt)
                    }
                    MajorSubtype::N(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{} {}", ext, alt)
                    }
                }
            },
            ChordQuality::Minor(subtype) => {
                match subtype {
                    MinorSubtype::Min(alt) => {
                        format!("min {}", alt)
                    }
                    MinorSubtype::Min6(alt) => {
                        format!("min {}", alt)
                    }
                    MinorSubtype::MinMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("minMaj{} {}", ext, alt)
                    }
                    MinorSubtype::MinN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("min{} {}", ext, alt)
                    }
                }
            },
            ChordQuality::Aug(subtype) => {
                match subtype {
                    AugSubtype::Aug(alt) => {
                        format!("Aug {}", alt)
                    }
                    AugSubtype::AugMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("+Maj{} {}", ext, alt)
                    }
                    AugSubtype::AugN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("+{} {}", ext, alt)
                    }
                }
            },
            ChordQuality::Dim(subtype) => {
                match subtype {
                    DimSubtype::Dim(alt) => {
                        format!("dim {}", alt)
                    }
                    DimSubtype::MinNb5(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("min{}b5 {}", ext, alt)
                    }
                    DimSubtype::DimN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("dim{} {}", ext, alt)
                    }
                    DimSubtype::DimMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("dimMaj{} {}", ext, alt)
                    }
                }
            },
            ChordQuality::Sus(subtype) => {
                match subtype {
                    SusSubtype::Sus2(alt) => {
                        format!("sus2 {}", alt)
                    }
                    SusSubtype::Sus4(alt) => {
                        format!("sus4 {}", alt)
                    }
                    SusSubtype::DomNSus(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}sus {}", ext, alt)
                    }
                    SusSubtype::MajNSus(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("Maj{}sus {}", ext, alt)
                    }
                    SusSubtype::SixNineSus(alt) => {
                        format!("6/9sus {}", alt)
                    }
                }
            },
            ChordQuality::Interval(ic) => ic.to_string(),
            ChordQuality::SingleNote => "note".to_owned(),
        }.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_ambiguity_no_duplicates() {
        // C major triad: 0 (C), 4 (E), 7 (G) - no duplicate degrees
        let intervals = vec![0, 4, 7];
        let ambiguities = QualityAmbiguity::find_duplicate_degrees(&intervals);
        assert!(ambiguities.is_empty());
    }

    #[test]
    fn test_quality_ambiguity_duplicate_third() {
        // Chord with both b3 (3) and 3 (4) - duplicate 3rd degree
        let intervals = vec![0, 3, 4, 7];
        let ambiguities = QualityAmbiguity::find_duplicate_degrees(&intervals);
        assert_eq!(ambiguities.len(), 1);
        match &ambiguities[0] {
            QualityAmbiguity::DuplicateScaleDegree { degree, intervals } => {
                assert_eq!(*degree, 3);
                assert!(intervals.contains(&3));
                assert!(intervals.contains(&4));
            }
            _ => panic!("Expected DuplicateScaleDegree"),
        }
    }

    #[test]
    fn test_quality_ambiguity_duplicate_eleventh() {
        // Chord with both 11 (5) and #11 (6) - duplicate 11th degree
        let intervals = vec![0, 4, 5, 6, 7, 10];
        let ambiguities = QualityAmbiguity::find_duplicate_degrees(&intervals);
        assert_eq!(ambiguities.len(), 1);
        match &ambiguities[0] {
            QualityAmbiguity::DuplicateScaleDegree { degree, intervals } => {
                assert_eq!(*degree, 11);
                assert!(intervals.contains(&5));
                assert!(intervals.contains(&6));
            }
            _ => panic!("Expected DuplicateScaleDegree"),
        }
    }

    #[test]
    fn test_quality_ambiguity_duplicate_ninth() {
        // Chord with both b9 (1) and 9 (2) - duplicate 9th degree
        let intervals = vec![0, 1, 2, 4, 7, 10];
        let ambiguities = QualityAmbiguity::find_duplicate_degrees(&intervals);
        assert_eq!(ambiguities.len(), 1);
        match &ambiguities[0] {
            QualityAmbiguity::DuplicateScaleDegree { degree, intervals } => {
                assert_eq!(*degree, 9);
                assert!(intervals.contains(&1));
                assert!(intervals.contains(&2));
            }
            _ => panic!("Expected DuplicateScaleDegree"),
        }
    }

    #[test]
    fn test_quality_ambiguity_multiple_duplicates() {
        // Chord with duplicate 3rd (b3, 3) AND duplicate 7th (b7, 7)
        let intervals = vec![0, 3, 4, 7, 10, 11];
        let ambiguities = QualityAmbiguity::find_duplicate_degrees(&intervals);
        assert_eq!(ambiguities.len(), 2);

        let degrees: Vec<u8> = ambiguities.iter().map(|a| {
            match a {
                QualityAmbiguity::DuplicateScaleDegree { degree, .. } => *degree,
                _ => panic!("Expected DuplicateScaleDegree"),
            }
        }).collect();

        assert!(degrees.contains(&3));  // Duplicate third
        assert!(degrees.contains(&7));  // Duplicate seventh
    }

    #[test]
    fn test_sixth_vs_thirteenth_ambiguity() {
        // Test the SixthVsThirteenth variant
        let with_seventh = QualityAmbiguity::SixthVsThirteenth { has_seventh: true };
        let without_seventh = QualityAmbiguity::SixthVsThirteenth { has_seventh: false };

        match with_seventh {
            QualityAmbiguity::SixthVsThirteenth { has_seventh } => {
                assert!(has_seventh);
            }
            _ => panic!("Expected SixthVsThirteenth"),
        }

        match without_seventh {
            QualityAmbiguity::SixthVsThirteenth { has_seventh } => {
                assert!(!has_seventh);
            }
            _ => panic!("Expected SixthVsThirteenth"),
        }
    }

    #[test]
    fn test_multiple_interpretations() {
        // Test the MultipleInterpretations variant
        let ambiguity = QualityAmbiguity::MultipleInterpretations(vec![
            "Am7".to_string(),
            "C6/A".to_string(),
        ]);

        match ambiguity {
            QualityAmbiguity::MultipleInterpretations(interpretations) => {
                assert_eq!(interpretations.len(), 2);
                assert!(interpretations.contains(&"Am7".to_string()));
                assert!(interpretations.contains(&"C6/A".to_string()));
            }
            _ => panic!("Expected MultipleInterpretations"),
        }
    }
}