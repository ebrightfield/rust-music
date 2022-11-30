pub mod quality;
pub mod naming_heuristics;

use crate::note_collections::pc_set::PcSet;
use crate::note::note::Note;
use std::fmt::{Display, Formatter};
use quality::chord::ChordQuality;
use crate::note::pc::Pc;

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
    explicit_sus4: bool,
    /// Use fancy utf-8 chars for notes.
    uft8_accidentals: bool,
    /// Number of space chars to put between the root note and the chord quality.
    space_between_root_and_quality: usize,
    /// Number of space chars to put between the chord quality and the slash in a slash chord.
    space_between_quality_and_slash: usize,
    /// Number of space chars to put after the slash symbol in a chord.
    space_after_slash: usize,
    /// Whether to only to e.g. label a min11 chord if it contains the 9th.
    /// This is a practical assumption that usually doesn't apply in settings
    /// outside of classical music theory.
    extension_style: ExtensionStyle,
}

/// Describes a [PcSet] using the chord lexicon fleshed out in [ChordQuality].
/// The [TonalSpecification] provides optional means of specifying a particular
/// root note, and/or bass note, and can also specify "no root".
#[derive(Debug, Clone)]
pub struct ChordName {
    /// Information regarding any choice of root notes, slash chord, or
    /// specifying that we are not generalizing over notes at all.
    tonality: TonalSpecification,
    /// Combination of tonal "flavors" asserted to be in the chord.
    quality: ChordQuality,
    /// Underlying set of pitch classes on which the name is being asserted.
    pc_set: PcSet,
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
/// All specified notes are assumed to be members of their associated [Vec<Pc>].
#[derive(Debug, Clone)]
pub enum TonalSpecification {
    /// If it's a slash chord, the bass note will be supplied here.
    SlashChord {
        bass: Note,
        root: Note,
    },
    /// Root note relative to the defined chord quality.
    RootPosition(Note),
    /// No tonal specification. The [Option<Pc>] specifies any possible bass note.
    /// The relevant bass note must be an element in the [Vec<Pc>] being named.
    None(Option<Pc>)
}