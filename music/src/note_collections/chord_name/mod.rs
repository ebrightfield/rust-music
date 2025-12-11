pub mod quality;
pub mod naming_heuristics;
pub mod parsing;

use crate::note_collections::pc_set::PcSet;
use crate::note::note::Note;
use crate::note::pitch_class::Pc;

pub use quality::chord::ChordQuality;

/// The means by which to stylize the text that denotes
/// a chord's extensions. There are a number of mutually incompatible
/// conventions, so we just provide them all as options.
#[derive(Debug, Default, Copy, Clone)]
pub enum ExtensionStyle {
    /// Label everything as a 7th chord, and show extensions as alterations
    #[default]
    None,
    /// Any Nth must have all extensions below it. e.g. a 13th chord must contain an 11th
    /// and a 9th.
    Strict,
    /// Labels the extension with whatever is highest
    Highest,
    /// Labels the extension with whatever is highest, except if there is only one
    /// higher extension, in which case we label it as a 7th chord with the alteration.
    HighestUnlessOne,
}

/// Chords can be displayed in a number of ways, and users might have different
/// preferences over the matter.
/// This configuration struct provides fine-grained control over a number
/// of formatting parameters.
#[derive(Debug, Default, Clone)]
pub struct ChordNameDisplayConfig {
    // /// How to style the chord alterations.
    // alt_notation: AlterationNotationStyle,
    /// Whether or not to express sus4, 7sus4, 9sus4, etc.
    /// as sus, 7sus, 9sus.
    pub explicit_sus4: bool,
    /// Use fancy utf-8 chars for notes.
    pub uft8_accidentals: bool,
    /// Number of space chars to put between the root note and the chord quality.
    pub space_between_root_and_quality: usize,
    /// Number of space chars to put between the chord quality and the slash in a slash chord.
    pub space_between_quality_and_slash: usize,
    /// Number of space chars to put after the slash symbol in a chord.
    pub space_after_slash: usize,
    /// Whether to only to e.g. label a min11 chord if it contains the 9th.
    /// This is a practical assumption that usually doesn't apply in settings
    /// outside of classical music theory.
    pub extension_style: ExtensionStyle,
}

/// Describes a [PcSet] using the chord lexicon fleshed out in [ChordQuality].
/// The [TonalSpecification] provides optional means of specifying a particular
/// root note, and/or bass note, and can also specify "no root".
#[derive(Debug, Clone)]
pub struct ChordName {
    /// Information regarding any choice of root notes, slash chord, or
    /// specifying that we are not generalizing over notes at all.
    pub tonality: TonalSpecification,
    /// Combination of tonal "flavors" asserted to be in the chord.
    pub quality: ChordQuality,
    /// Underlying set of pitch classes on which the name is being asserted.
    pub pc_set: PcSet,
}

impl ChordName {
    pub fn to_string(&self, cfg: Option<&ChordNameDisplayConfig>) -> String {
        let cfg = cfg
            .map(|cfg| cfg.clone())
            .unwrap_or_default();
        self.quality.to_string(&cfg)
    }
}

/// Whether or not something is a slash chord.
/// All specified notes are assumed to be members of their associated `Vec<Pc>`.
#[derive(Debug, Clone)]
pub enum TonalSpecification {
    /// If it's a slash chord, the bass note will be supplied here.
    SlashChord {
        bass: Note,
        root: Note,
    },
    /// Root note relative to the defined chord quality.
    RootPosition(Note),
    /// No tonal specification. The `Option<Pc>` specifies any possible bass note.
    /// The relevant bass note must be an element in the `Vec<Pc>` being named.
    None(Option<Pc>)
}

/// Configuration for chord naming heuristics.
///
/// This struct provides fine-grained control over how the naming algorithm
/// interprets and labels chord qualities.
#[derive(Debug, Clone)]
pub struct NamingConfig {
    /// How to label extensions (7th, 9th, 11th, 13th).
    pub extension_style: ExtensionStyle,
    /// Whether to prefer "add" notation over extension labels.
    /// When true, "Cadd9" instead of "C9" when only the 9th is present
    /// above a triad (no 7th).
    pub prefer_add_notation: bool,
    /// Whether to show omitted notes in the chord name (e.g., "no5").
    pub show_omissions: bool,
    /// Minimum number of notes to trigger slash chord detection.
    /// Default is 4 (don't analyze triads for inversions as slash chords).
    pub slash_chord_threshold: usize,
    /// Whether to analyze for 6th chord vs 13th chord ambiguity.
    /// When true, chords with a 6th but no 7th are labeled as 6th chords,
    /// not 13th chords.
    pub distinguish_sixth_from_thirteenth: bool,
    /// Whether to report ambiguities found during analysis.
    pub report_ambiguities: bool,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            extension_style: ExtensionStyle::Highest,
            prefer_add_notation: false,
            show_omissions: true,
            slash_chord_threshold: 4,
            distinguish_sixth_from_thirteenth: true,
            report_ambiguities: false,
        }
    }
}

impl NamingConfig {
    /// Create a new NamingConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method to set extension style.
    pub fn extension_style(mut self, style: ExtensionStyle) -> Self {
        self.extension_style = style;
        self
    }

    /// Builder method to enable/disable "add" notation preference.
    pub fn prefer_add_notation(mut self, prefer: bool) -> Self {
        self.prefer_add_notation = prefer;
        self
    }

    /// Builder method to enable/disable showing omitted notes.
    pub fn show_omissions(mut self, show: bool) -> Self {
        self.show_omissions = show;
        self
    }

    /// Builder method to set the minimum notes for slash chord detection.
    pub fn slash_chord_threshold(mut self, threshold: usize) -> Self {
        self.slash_chord_threshold = threshold;
        self
    }

    /// Builder method to enable/disable 6th vs 13th distinction.
    pub fn distinguish_sixth_from_thirteenth(mut self, distinguish: bool) -> Self {
        self.distinguish_sixth_from_thirteenth = distinguish;
        self
    }

    /// Builder method to enable/disable ambiguity reporting.
    pub fn report_ambiguities(mut self, report: bool) -> Self {
        self.report_ambiguities = report;
        self
    }

    /// Create a "strict" configuration that follows classical theory rules.
    /// - Extensions require all lower extensions present
    /// - Shows omissions
    /// - Reports ambiguities
    pub fn strict() -> Self {
        Self {
            extension_style: ExtensionStyle::Strict,
            prefer_add_notation: true,
            show_omissions: true,
            slash_chord_threshold: 4,
            distinguish_sixth_from_thirteenth: true,
            report_ambiguities: true,
        }
    }

    /// Create a "jazz" configuration common in jazz lead sheets.
    /// - Uses highest extension labeling
    /// - Doesn't show omissions (common in jazz)
    /// - Lower slash chord threshold for inversions
    pub fn jazz() -> Self {
        Self {
            extension_style: ExtensionStyle::Highest,
            prefer_add_notation: false,
            show_omissions: false,
            slash_chord_threshold: 3,
            distinguish_sixth_from_thirteenth: true,
            report_ambiguities: false,
        }
    }

    /// Create a "pop" configuration common in pop/rock chord charts.
    /// - Simple labeling
    /// - No omissions shown
    /// - Prefers "add" notation
    pub fn pop() -> Self {
        Self {
            extension_style: ExtensionStyle::HighestUnlessOne,
            prefer_add_notation: true,
            show_omissions: false,
            slash_chord_threshold: 4,
            distinguish_sixth_from_thirteenth: true,
            report_ambiguities: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naming_config_default() {
        let config = NamingConfig::default();
        assert!(!config.prefer_add_notation);
        assert!(config.show_omissions);
        assert_eq!(config.slash_chord_threshold, 4);
        assert!(config.distinguish_sixth_from_thirteenth);
        assert!(!config.report_ambiguities);
    }

    #[test]
    fn test_naming_config_builder() {
        let config = NamingConfig::new()
            .extension_style(ExtensionStyle::Strict)
            .prefer_add_notation(true)
            .show_omissions(false)
            .slash_chord_threshold(3)
            .report_ambiguities(true);

        assert!(config.prefer_add_notation);
        assert!(!config.show_omissions);
        assert_eq!(config.slash_chord_threshold, 3);
        assert!(config.report_ambiguities);
    }

    #[test]
    fn test_naming_config_presets() {
        let strict = NamingConfig::strict();
        assert!(strict.prefer_add_notation);
        assert!(strict.report_ambiguities);

        let jazz = NamingConfig::jazz();
        assert!(!jazz.show_omissions);
        assert_eq!(jazz.slash_chord_threshold, 3);

        let pop = NamingConfig::pop();
        assert!(pop.prefer_add_notation);
        assert!(!pop.show_omissions);
    }
}