use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use crate::error::MusicSemanticsError;
use crate::note_collections::chord_name::{ChordNameDisplayConfig, ExtensionStyle, MajNotation};
use crate::note_collections::interval_class::IntervalClass;

/// The "ninth", "eleventh", etc in Maj9th or min11th chords, etc.
///
/// `FlatFive` and `SharpFive` were added in Tier 1.1 for `7♭5` / `Maj7♭5`
/// style naming. These are distinct from `SharpEleven` and `FlatThirteenth`:
/// the "five" variants are emitted only when the corresponding triad lacks a
/// natural P5, so the symbol is understood as *replacing* the fifth.
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
    /// Pc6 with no P5 present over a major/minor triad (Tier 1.1).
    FlatFive,
    /// Pc8 with no P5 present over a major/minor triad. Parallel to FlatFive;
    /// added for API symmetry. Not emitted automatically yet.
    SharpFive,
}

impl Display for AltChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ascii())
    }
}

impl AltChoice {
    /// ASCII rendering (`b9`, `#11`, …). The `Display` impl calls this.
    pub fn to_ascii(&self) -> String {
        match self {
            AltChoice::FlatNine => "b9".to_string(),
            AltChoice::Nine => "9".to_string(),
            AltChoice::SharpNine => "#9".to_string(),
            AltChoice::FlatEleven => "b11".to_string(),
            AltChoice::Eleven => "11".to_string(),
            AltChoice::SharpEleven => "#11".to_string(),
            AltChoice::FlatThirteenth => "b13".to_string(),
            AltChoice::Thirteenth => "13".to_string(),
            AltChoice::SharpThirteenth => "#13".to_string(),
            AltChoice::FlatFive => "b5".to_string(),
            AltChoice::SharpFive => "#5".to_string(),
        }
    }

    /// utf-8 rendering (`♭9`, `♯11`, …). Used when
    /// `ChordNameDisplayConfig::utf8_accidentals` is `true`.
    pub fn to_utf8(&self) -> String {
        match self {
            AltChoice::FlatNine => "♭9".to_string(),
            AltChoice::Nine => "9".to_string(),
            AltChoice::SharpNine => "♯9".to_string(),
            AltChoice::FlatEleven => "♭11".to_string(),
            AltChoice::Eleven => "11".to_string(),
            AltChoice::SharpEleven => "♯11".to_string(),
            AltChoice::FlatThirteenth => "♭13".to_string(),
            AltChoice::Thirteenth => "13".to_string(),
            AltChoice::SharpThirteenth => "♯13".to_string(),
            AltChoice::FlatFive => "♭5".to_string(),
            AltChoice::SharpFive => "♯5".to_string(),
        }
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

impl Alt {
    /// Render the alterations honoring [`ChordNameDisplayConfig`].
    /// When the Alt is empty returns the empty string.
    pub fn to_string_with(&self, cfg: &ChordNameDisplayConfig) -> String {
        if self.is_empty() {
            return String::new();
        }
        let s: Vec<String> = self
            .iter()
            .map(|alt| {
                if cfg.utf8_accidentals {
                    alt.to_utf8()
                } else {
                    alt.to_ascii()
                }
            })
            .collect();
        format!("({})", s.join(", "))
    }

    /// Like [`Self::to_string_with`] but returns a leading-space-prefixed
    /// string when non-empty. Lets the caller concatenate without trailing
    /// whitespace in the empty case. Introduced in Tier 3.4 so chord
    /// rendering no longer relies on a trailing `.trim()`.
    pub fn to_string_prefixed(&self, cfg: &ChordNameDisplayConfig) -> String {
        if self.is_empty() {
            return String::new();
        }
        format!(" {}", self.to_string_with(cfg))
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

impl IntoIterator for Alt {
    type Item = AltChoice;
    type IntoIter = std::vec::IntoIter<AltChoice>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Alt {
    type Item = &'a AltChoice;
    type IntoIter = std::slice::Iter<'a, AltChoice>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
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

/// A member of an "add" chord (Tier 2.2). Only 6, 9, 11, and 13 are valid
/// adds; `Add6` is kept alongside `Add9`/`Add11`/`Add13` so 6-chord variants
/// route through the same code path when `prefer_add_notation` is on.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AddMember {
    Add6,
    Add9,
    Add11,
    Add13,
}

impl AddMember {
    pub fn label(&self) -> &'static str {
        match self {
            AddMember::Add6 => "6",
            AddMember::Add9 => "9",
            AddMember::Add11 => "11",
            AddMember::Add13 => "13",
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
    /// `add9`/`add11`/`add13` family — extensions over a triad with no 7th.
    /// Emitted when `NamingConfig::prefer_add_notation` is `true`. Tier 2.2.
    Add(Vec<AddMember>, Alt),
    /// Dominant-altered ("7alt"): a dominant 7 chord with three-plus
    /// alterations spanning both the 9-axis (♭9 / ♯9) and the 5-axis
    /// (♭5 / ♯5 / ♯11 / ♭13). Tier 1.4.
    DomAlt,
    /// Inferred major triad with the 3rd missing from the pc-set. Tier 2.3
    /// `show_omissions`: only emitted when the config opts in.
    NoThird(Alt),
    /// `CMaj7(no5)` — Maj triad with a 7th/9th/11th/13th extension but no
    /// P5. Tier 2.3 `show_omissions`.
    NoFifth(Vec<Extension>, Alt),
    /// `C7(no5)` — dominant with no P5. Tier 2.3 `show_omissions`.
    DomNoFifth(Vec<Extension>, Alt),
}

/// Chords based around a minor triad.
#[derive(Debug, Clone, PartialEq)]
pub enum MinorSubtype {
    Min(Alt),
    Min6(Alt),
    MinMajN(Vec<Extension>, Alt),
    MinN(Vec<Extension>, Alt),
    /// `madd9`/`madd11`/`madd13` family. Tier 2.2.
    Add(Vec<AddMember>, Alt),
    /// `Cm7(no5)` — min triad with extension but no P5. Tier 2.3.
    NoFifth(Vec<Extension>, Alt),
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

/// Chords based around a sus triad (sus2 or sus4).
#[derive(Debug, Clone, PartialEq)]
pub enum SusSubtype {
    Sus2(Alt),
    Sus4(Alt),
    DomNSus(Vec<Extension>, Alt),
    MajNSus(Vec<Extension>, Alt),
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

/// Render the "major" root-label fragment per `maj_notation`.
/// Examples: `maj_label(Delta) == "Δ"`, `maj_label(Maj) == "Maj"`,
/// `maj_label(MajCap) == "M"`, `maj_label(LowerMaj) == "maj"`.
fn maj_label(kind: MajNotation) -> &'static str {
    match kind {
        MajNotation::Delta => "Δ",
        MajNotation::Maj => "Maj",
        MajNotation::MajCap => "M",
        MajNotation::LowerMaj => "maj",
    }
}

/// The letter used for the minor triad label. Chord-chart convention ties
/// minor casing to the major label style: `Maj` / `maj` pair with `m`,
/// `M` pairs with `m`, and `Δ` pairs with `m` as well. We therefore always
/// render `m` (lowercase) here; the prior implementation used `min` which is
/// preserved when `maj_notation == Maj` for backward-compat.
fn min_label(kind: MajNotation) -> &'static str {
    match kind {
        // Historical default: `min` with `Maj`. Preserved so tier-1 matrix
        // outputs (which pin `"min7"`, `"minMaj7"`, `"Maj"`) still pass.
        MajNotation::Maj => "min",
        // All other styles use short `m`.
        MajNotation::Delta | MajNotation::MajCap | MajNotation::LowerMaj => "m",
    }
}

impl ChordQuality {
    /// Render honoring [`ChordNameDisplayConfig::maj_notation`] and
    /// [`ChordNameDisplayConfig::utf8_accidentals`].
    pub fn to_string(&self, cfg: &ChordNameDisplayConfig) -> String {
        let style = cfg.extension_style;
        let maj = maj_label(cfg.maj_notation);
        let min = min_label(cfg.maj_notation);
        let alt_px = |alt: &Alt| alt.to_string_prefixed(cfg);
        let ext_and_alts = |alt: &Alt, ext: &[Extension], style| {
            let (ext, mut alts) = resolve_extension(ext, style);
            alts.extend(alt.0.clone());
            (ext, Alt::from(alts))
        };
        match &self {
            ChordQuality::Major(subtype) => {
                match subtype {
                    MajorSubtype::Maj(alt) => {
                        format!("Maj{}", alt_px(alt))
                    }
                    MajorSubtype::Maj6(alt) => {
                        format!("Maj6{}", alt_px(alt))
                    }
                    MajorSubtype::MajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}{}{}", maj, ext, alt_px(&alt))
                    }
                    MajorSubtype::N(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}{}", ext, alt_px(&alt))
                    }
                    MajorSubtype::Add(adds, alt) => {
                        format_add(None, adds, alt, cfg)
                    }
                    MajorSubtype::DomAlt => "7alt".to_string(),
                    MajorSubtype::NoThird(alt) => {
                        format!("Maj (no3){}", alt_px(alt))
                    }
                    MajorSubtype::NoFifth(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}{} (no5){}", maj, ext, alt_px(&alt))
                    }
                    MajorSubtype::DomNoFifth(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{} (no5){}", ext, alt_px(&alt))
                    }
                }
            },
            ChordQuality::Minor(subtype) => {
                match subtype {
                    MinorSubtype::Min(alt) => {
                        format!("{}{}", min, alt_px(alt))
                    }
                    MinorSubtype::Min6(alt) => {
                        format!("{}6{}", min, alt_px(alt))
                    }
                    MinorSubtype::MinMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}{}{}{}", min, maj, ext, alt_px(&alt))
                    }
                    MinorSubtype::MinN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}{}{}", min, ext, alt_px(&alt))
                    }
                    MinorSubtype::Add(adds, alt) => {
                        format_add(Some(min), adds, alt, cfg)
                    }
                    MinorSubtype::NoFifth(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}{} (no5){}", min, ext, alt_px(&alt))
                    }
                }
            },
            ChordQuality::Aug(subtype) => {
                match subtype {
                    AugSubtype::Aug(alt) => {
                        format!("Aug{}", alt_px(alt))
                    }
                    AugSubtype::AugMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("+{}{}{}", maj, ext, alt_px(&alt))
                    }
                    AugSubtype::AugN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("+{}{}", ext, alt_px(&alt))
                    }
                }
            },
            ChordQuality::Dim(subtype) => {
                match subtype {
                    DimSubtype::Dim(alt) => {
                        format!("dim{}", alt_px(alt))
                    }
                    DimSubtype::MinNb5(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        let b5 = if cfg.utf8_accidentals { "♭5" } else { "b5" };
                        format!("{}{}{}{}", min, ext, b5, alt_px(&alt))
                    }
                    DimSubtype::DimN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("dim{}{}", ext, alt_px(&alt))
                    }
                    DimSubtype::DimMajN(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("dim{}{}{}", maj, ext, alt_px(&alt))
                    }
                }
            },
            ChordQuality::Sus(subtype) => {
                match subtype {
                    SusSubtype::Sus2(alt) => {
                        format!("sus2{}", alt_px(alt))
                    }
                    SusSubtype::Sus4(alt) => {
                        format!("sus4{}", alt_px(alt))
                    }
                    SusSubtype::DomNSus(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}sus{}", ext, alt_px(&alt))
                    }
                    SusSubtype::MajNSus(ext, alt) => {
                        let (ext, alt) = ext_and_alts(alt, ext, style);
                        format!("{}{}sus{}", maj, ext, alt_px(&alt))
                    }
                }
            },
            ChordQuality::Interval(ic) => ic.to_string(),
            ChordQuality::SingleNote => "note".to_owned(),
        }
    }
}

/// Render an `add` subtype: `Cadd9`, `Cmadd11`, `C6/9`, etc. `prefix` is the
/// minor label when rendering a minor add chord; `None` leaves no prefix
/// (major add).
fn format_add(
    prefix: Option<&str>,
    adds: &[AddMember],
    alt: &Alt,
    cfg: &ChordNameDisplayConfig,
) -> String {
    let head = prefix.unwrap_or("");
    let alt_str = alt.to_string_with(cfg);
    // Special shorthand: adds == [Add6, Add9] renders as "6/9".
    if adds.len() == 2
        && adds.contains(&AddMember::Add6)
        && adds.contains(&AddMember::Add9)
    {
        return if alt.is_empty() {
            format!("{}6/9", head)
        } else {
            format!("{}6/9 {}", head, alt_str)
        };
    }
    let body = adds
        .iter()
        .map(|m| format!("add{}", m.label()))
        .collect::<Vec<_>>()
        .join("");
    if alt.is_empty() {
        format!("{}{}", head, body)
    } else {
        format!("{}{} {}", head, body, alt_str)
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