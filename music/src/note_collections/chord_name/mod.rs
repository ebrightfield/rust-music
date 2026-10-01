pub mod naming_heuristics;
pub mod parsing;
pub mod quality;

use crate::error::MusicSemanticsError;
use crate::note::note::Note;
use crate::note::pitch_class::Pc;
use crate::note_collections::pc_set::{PcContent, PcShape};
use std::collections::HashSet;

pub use naming_heuristics::{infer_chord_quality, infer_scale_quality};
pub use parsing::parse_chord_name;
pub use quality::chord::{
    Alt, AltChoice, AugSubtype, ChordQuality, DimSubtype, Extension, MajorSubtype, MinorSubtype,
    QualityAmbiguity, SusSubtype,
};

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

/// Canonical label for the major-seventh flavor.
///
/// Chord-chart notation uses different glyphs for the same concept. This enum
/// makes the choice explicit so round-trip tests and snapshot output can pin
/// exactly one canonical rendering while still supporting the common dialects.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum MajNotation {
    /// Delta — `Δ7`, `Δ9`. Canonical default per the correction plan.
    #[default]
    Delta,
    /// `Maj7`, `Maj9`. Most common lead-sheet convention.
    Maj,
    /// `M7`, `M9`. Compact ASCII.
    MajCap,
    /// `maj7`, `maj9`. Lowercase variant.
    LowerMaj,
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
    /// Use fancy utf-8 chars for accidentals (`♭`, `♯`) instead of ASCII
    /// (`b`, `#`). Default is ASCII in this Phase-2 build to keep the positive
    /// test matrix stable; the ASCII→utf8 toggle is orthogonal to rendering
    /// correctness.
    pub utf8_accidentals: bool,
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
    /// Which label to use for the major-seventh quality. See [`MajNotation`].
    pub maj_notation: MajNotation,
}

/// Describes a [PcShape] using the chord lexicon fleshed out in [ChordQuality].
/// The [TonalSpecification] provides optional means of specifying a particular
/// root note, and/or bass note, and can also specify "no root".
#[derive(Debug, Clone)]
pub struct ChordName {
    /// Information regarding any choice of root notes, slash chord, or
    /// specifying that we are not generalizing over notes at all.
    pub tonality: TonalSpecification,
    /// Combination of tonal "flavors" asserted to be in the chord.
    pub quality: ChordQuality,
    /// Underlying intervallic shape (zero-anchored) on which the name is being asserted.
    /// REQ-O21: renamed from pc_set; type narrowed to PcShape.
    pub pc_shape: PcShape,
}

impl ChordName {
    /// Construct a [`ChordName`] directly from its parts.
    pub fn new(tonality: TonalSpecification, quality: ChordQuality, pc_shape: PcShape) -> Self {
        Self {
            tonality,
            quality,
            pc_shape,
        }
    }

    /// Infer a rooted chord name from spelled chord members and an explicit bass.
    ///
    /// The bass must be one of `notes`. Below
    /// [`NamingConfig::slash_chord_threshold`], the bass is the only root
    /// candidate. At or above the threshold, every distinct pitch class is
    /// tried as a root. Candidates are ranked by conventional tertian evidence;
    /// ties prefer the bass, then the lowest pitch class, avoiding unstable or
    /// speculative slash chords. A winning root different from the bass
    /// produces [`TonalSpecification::SlashChord`].
    ///
    /// The supplied spellings are retained for both root and bass. Duplicate
    /// enharmonic pitch classes do not increase the threshold cardinality.
    pub fn infer(
        notes: &[Note],
        bass: Note,
        cfg: &NamingConfig,
    ) -> Result<Self, MusicSemanticsError> {
        if notes.is_empty() {
            return Err(MusicSemanticsError::EmptySetOfNotes);
        }

        let bass_pc = Pc::from(&bass);
        let content: HashSet<Pc> = notes.iter().map(Pc::from).collect();
        if !content.contains(&bass_pc) {
            return Err(MusicSemanticsError::InvalidChordBass(bass));
        }

        let candidates: Vec<Note> = if content.len() < cfg.slash_chord_threshold {
            vec![bass]
        } else {
            let mut seen = HashSet::new();
            notes
                .iter()
                .copied()
                .filter(|note| seen.insert(Pc::from(note)))
                .collect()
        };

        let mut best = None;
        for root in candidates {
            let root_pc = Pc::from(&root);
            let relative: HashSet<Pc> = content
                .iter()
                .map(|pc| Pc::from(i32::from(pc) - i32::from(root_pc)))
                .collect();
            let Some((_, Some(quality))) =
                naming_heuristics::infer_chord_quality_with(&relative, cfg)
            else {
                continue;
            };
            let score = root_candidate_score(&relative);
            let is_bass = root_pc == bass_pc;
            let replaces_best =
                best.as_ref()
                    .is_none_or(|(best_root, _, best_score, best_is_bass)| {
                        score > *best_score
                            || (score == *best_score
                                && ((is_bass && !best_is_bass)
                                    || (is_bass == *best_is_bass
                                        && u8::from(&root_pc) < u8::from(&Pc::from(best_root)))))
                    });
            if replaces_best {
                best = Some((root, quality, score, is_bass));
            }
        }

        let (root, quality, _, root_is_bass) =
            best.ok_or_else(|| MusicSemanticsError::InvalidChordQuality(format!("{notes:?}")))?;
        let root_pc = Pc::from(&root);
        let pc_shape = PcShape::new(
            content
                .iter()
                .map(|pc| Pc::from(i32::from(pc) - i32::from(root_pc)))
                .collect(),
        );
        let tonality = if root_is_bass {
            TonalSpecification::RootPosition(root)
        } else {
            TonalSpecification::SlashChord { bass, root }
        };

        Ok(Self::new(tonality, quality, pc_shape))
    }

    /// Parse a chord symbol string (e.g. `"Cmaj7"`, `"F#m7b5"`) into a
    /// [`ChordName`] in root position.
    ///
    /// The resulting [`ChordName`] has `tonality = TonalSpecification::RootPosition(root)`
    /// and a quality derived from the parsed interval content. For finer-grained
    /// control (inversions, slash chords, custom qualities) construct with
    /// [`ChordName::new`].
    ///
    /// ```
    /// use music::prelude::*;
    ///
    /// let cmaj7 = ChordName::from_symbol("Cmaj7").unwrap();
    /// assert_eq!(cmaj7.pc_shape.len(), 4);
    /// ```
    pub fn from_symbol(symbol: &str) -> Result<Self, crate::error::MusicSemanticsError> {
        let (root, pc_shape) = parsing::parse_chord_name(symbol)?;
        let pcs_hashset: std::collections::HashSet<Pc> = pc_shape.iter().copied().collect();
        let quality = naming_heuristics::infer_chord_quality(&pcs_hashset)
            .and_then(|(_, q)| q)
            .ok_or_else(|| {
                crate::error::MusicSemanticsError::InvalidChordQuality(symbol.to_string())
            })?;
        Ok(Self {
            tonality: TonalSpecification::RootPosition(root),
            quality,
            pc_shape,
        })
    }

    /// Returns the absolute sounding [`PcContent`] of this chord at its stored
    /// root, or `None` if the chord is rootless.
    ///
    /// Delegates to [`TonalSpecification::root_pc`] to extract the root pitch
    /// class, then calls [`PcShape::at_root`] to transpose the interval template.
    ///
    /// # Returns
    /// - `Some(content)` — the sorted, deduplicated set of absolute sounding
    ///   pitch classes at this chord's root.
    /// - `None` — if [`TonalSpecification::None`] (rootless chord or unrooted
    ///   set).
    ///
    /// # Example
    /// ```
    /// use music::prelude::*;
    ///
    /// let fsm7b5 = ChordName::from_symbol("F#m7b5").unwrap();
    /// // F# root (Pc6) + shape [0,3,6,10] → sounding [0,4,6,9]
    /// assert_eq!(
    ///     fsm7b5.sounding_content(),
    ///     Some(PcContent::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc6, Pc::Pc9]))
    /// );
    /// ```
    // REQ-O27
    pub fn sounding_content(&self) -> Option<PcContent> {
        self.tonality
            .root_pc()
            .map(|root| self.pc_shape.at_root(root))
    }

    pub fn to_string(&self, cfg: Option<&ChordNameDisplayConfig>) -> String {
        let cfg = cfg.cloned().unwrap_or_default();
        self.quality.to_string(&cfg)
    }
}

/// Score how strongly a root candidate resembles a conventional tertian chord.
///
/// Thirds and the perfect fifth are the strongest root evidence, sevenths are
/// supporting evidence, and chromatic root-adjacent tones or a tritone count
/// against the candidate. Extensions remain mostly neutral so altered and
/// suspended qualities can still win through their underlying chord tones.
fn root_candidate_score(shape: &HashSet<Pc>) -> i16 {
    shape
        .iter()
        .map(|pc| match u8::from(pc) {
            3 | 4 => 8,
            7 => 6,
            10 | 11 => 4,
            9 => 1,
            1 | 6 | 8 => -2,
            _ => 0,
        })
        .sum()
}

/// Whether or not something is a slash chord.
/// All specified notes are assumed to be members of their associated `Vec<Pc>`.
#[derive(Debug, Clone)]
pub enum TonalSpecification {
    /// If it's a slash chord, the bass note will be supplied here.
    SlashChord { bass: Note, root: Note },
    /// Root note relative to the defined chord quality.
    RootPosition(Note),
    /// No tonal specification. The `Option<Pc>` specifies any possible bass note.
    /// The relevant bass note must be an element in the `Vec<Pc>` being named.
    None(Option<Pc>),
}

impl TonalSpecification {
    /// Returns the pitch class that governs shape-to-content transposition
    /// for this chord.
    ///
    /// - [`RootPosition(note)`][TonalSpecification::RootPosition] returns
    ///   `Some(Pc::from(&note))`.
    /// - [`SlashChord { root, .. }`][TonalSpecification::SlashChord] returns
    ///   the **root's** pitch class — NOT the bass's. The bass is a voicing
    ///   concern and does not affect the intervallic shape's anchor point.
    /// - [`None(_)`][TonalSpecification::None] returns `None`: rootless chords
    ///   have no sounding transposition to compute.
    // REQ-O26, REQ-O39
    pub fn root_pc(&self) -> Option<Pc> {
        match self {
            // REQ-O26: RootPosition → Some(root pc)
            TonalSpecification::RootPosition(note) => Some(Pc::from(note)),
            // REQ-O26: SlashChord → root (NOT bass)
            TonalSpecification::SlashChord { root, .. } => Some(Pc::from(root)),
            // REQ-O26: None(_) → None (rootless)
            TonalSpecification::None(_) => None,
        }
    }
}

/// Configuration for chord naming heuristics.
///
/// This struct provides fine-grained control over how the naming algorithm
/// interprets and labels chord qualities.
#[derive(Debug, Clone)]
pub struct NamingConfig {
    /// Whether to prefer "add" notation over extension labels.
    /// When true, "Cadd9" instead of "C9" when only the 9th is present
    /// above a triad (no 7th).
    pub prefer_add_notation: bool,
    /// Whether to show omitted notes in the chord name (e.g., "no5").
    pub show_omissions: bool,
    /// Minimum number of distinct pitch classes at which bass-aware inference
    /// tries every chord member as a possible root. Below this threshold, the
    /// explicit bass is the root. At or above it, a non-bass winning candidate
    /// is represented as a slash chord.
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
    use crate::note::pitch_class::Pc::*;

    #[test]
    fn chord_name_from_symbol_major7() {
        let cmaj7 = ChordName::from_symbol("Cmaj7").unwrap();
        assert_eq!(cmaj7.pc_shape, PcShape::new(vec![Pc0, Pc4, Pc7, Pc11]));
        assert!(matches!(
            cmaj7.tonality,
            TonalSpecification::RootPosition(Note::C)
        ));
        // Quality inference should find a Major-family quality, not fall back to SingleNote.
        assert!(matches!(cmaj7.quality, ChordQuality::Major(_)));
    }

    #[test]
    fn chord_name_from_symbol_sharp_root() {
        let fsm7b5 = ChordName::from_symbol("F#m7b5").unwrap();
        // F#m7b5 interval template: root-relative intervals [0, 3, 6, 10]
        // (minor third, diminished fifth, minor seventh).
        // REQ-O23: parser emits interval template, not zeroed-absolute sounding pcs.
        assert_eq!(fsm7b5.pc_shape, PcShape::new(vec![Pc0, Pc3, Pc6, Pc10]));
        assert!(matches!(
            fsm7b5.tonality,
            TonalSpecification::RootPosition(Note::Fis)
        ));
    }

    #[test]
    fn chord_name_from_symbol_rejects_garbage() {
        assert!(ChordName::from_symbol("").is_err());
        assert!(ChordName::from_symbol("XYZ").is_err());
    }

    #[test]
    fn chord_name_new_preserves_parts() {
        let quality = ChordQuality::SingleNote;
        let tonality = TonalSpecification::RootPosition(Note::D);
        let pc_shape = PcShape::new(vec![Pc2]);
        let chord = ChordName::new(tonality.clone(), quality.clone(), pc_shape.clone());
        assert_eq!(chord.pc_shape, pc_shape);
        assert_eq!(chord.quality, quality);
        assert!(matches!(
            chord.tonality,
            TonalSpecification::RootPosition(Note::D)
        ));
    }

    #[test]
    fn root_pc_of_root_position() {
        let t = TonalSpecification::RootPosition(Note::Fis);
        assert_eq!(t.root_pc(), Some(Pc::Pc6));
    }

    #[test]
    fn root_pc_of_slash_chord_is_root_not_bass() {
        let t = TonalSpecification::SlashChord {
            bass: Note::C,
            root: Note::Fis,
        };
        assert_eq!(t.root_pc(), Some(Pc::Pc6)); // NOT Pc0
    }

    #[test]
    fn root_pc_of_none_is_none() {
        let t = TonalSpecification::None(None);
        assert_eq!(t.root_pc(), None);
        let t = TonalSpecification::None(Some(Pc::Pc4));
        assert_eq!(t.root_pc(), None);
    }

    #[test]
    fn fsm7b5_pc_shape_is_interval_template() {
        let c = ChordName::from_symbol("F#m7b5").unwrap();
        assert_eq!(c.pc_shape, PcShape::new(vec![Pc0, Pc3, Pc6, Pc10]));
    }

    #[test]
    fn cmaj7_pc_shape_is_interval_template() {
        let c = ChordName::from_symbol("Cmaj7").unwrap();
        assert_eq!(c.pc_shape, PcShape::new(vec![Pc0, Pc4, Pc7, Pc11]));
    }

    #[test]
    fn fmaj7_and_cmaj7_share_shape() {
        let c = ChordName::from_symbol("Cmaj7").unwrap();
        let f = ChordName::from_symbol("Fmaj7").unwrap();
        assert_eq!(c.pc_shape, f.pc_shape);
    }

    #[test]
    fn fsm7b5_sounding_content() {
        let c = ChordName::from_symbol("F#m7b5").unwrap();
        // F# root (Pc6) + shape [0,3,6,10] = [6,9,0,4] sorted = [0,4,6,9]
        assert_eq!(
            c.sounding_content(),
            Some(PcContent::new(vec![Pc0, Pc4, Pc6, Pc9]))
        );
    }

    #[test]
    fn cmaj7_sounding_content() {
        let c = ChordName::from_symbol("Cmaj7").unwrap();
        assert_eq!(
            c.sounding_content(),
            Some(PcContent::new(vec![Pc0, Pc4, Pc7, Pc11]))
        );
    }

    #[test]
    fn rootless_chord_has_no_sounding_content() {
        let c = ChordName::new(
            TonalSpecification::None(None),
            ChordQuality::SingleNote,
            PcShape::new(vec![Pc0, Pc4, Pc7]),
        );
        assert_eq!(c.sounding_content(), None);
    }

    #[test]
    fn infer_keeps_root_position_when_bass_wins() {
        let chord = ChordName::infer(
            &[Note::C, Note::E, Note::G, Note::B],
            Note::C,
            &NamingConfig::default(),
        )
        .unwrap();

        assert!(matches!(
            chord.tonality,
            TonalSpecification::RootPosition(Note::C)
        ));
        assert_eq!(chord.pc_shape, PcShape::new(vec![Pc0, Pc4, Pc7, Pc11]));
    }

    #[test]
    fn infer_emits_slash_chord_when_non_bass_root_wins() {
        let chord = ChordName::infer(
            &[Note::E, Note::G, Note::B, Note::C],
            Note::E,
            &NamingConfig::default(),
        )
        .unwrap();

        assert!(matches!(
            chord.tonality,
            TonalSpecification::SlashChord {
                bass: Note::E,
                root: Note::C
            }
        ));
        assert_eq!(chord.pc_shape, PcShape::new(vec![Pc0, Pc4, Pc7, Pc11]));
    }

    #[test]
    fn infer_threshold_controls_triadic_inversion_detection() {
        let notes = [Note::E, Note::G, Note::C];
        let default = ChordName::infer(&notes, Note::E, &NamingConfig::default());
        let jazz = ChordName::infer(&notes, Note::E, &NamingConfig::jazz()).unwrap();

        assert!(matches!(
            default,
            Err(MusicSemanticsError::InvalidChordQuality(_))
        ));
        assert!(matches!(
            jazz.tonality,
            TonalSpecification::SlashChord {
                bass: Note::E,
                root: Note::C
            }
        ));
    }

    #[test]
    fn infer_prefers_complete_seventh_chord_over_slash_reinterpretation() {
        let chord = ChordName::infer(
            &[Note::E, Note::G, Note::A, Note::C],
            Note::E,
            &NamingConfig::default(),
        )
        .unwrap();

        assert!(matches!(
            chord.tonality,
            TonalSpecification::SlashChord {
                bass: Note::E,
                root: Note::A
            }
        ));
        assert_eq!(chord.pc_shape, PcShape::new(vec![Pc0, Pc3, Pc7, Pc10]));
    }

    #[test]
    fn infer_rejects_bass_outside_chord() {
        let error = ChordName::infer(&[Note::C, Note::E, Note::G], Note::D, &NamingConfig::jazz())
            .unwrap_err();
        assert!(matches!(
            error,
            MusicSemanticsError::InvalidChordBass(Note::D)
        ));
    }

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
