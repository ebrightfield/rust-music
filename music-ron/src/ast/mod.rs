//! Owned AST types deserialized from RON input.
//!
//! Each document variant (e.g. [`OwnedSnippet`], [`OwnedTab`]) carries
//! owned `String`/`Vec` fields suitable for serde. The top-level
//! [`Document`] enum is `#[serde(tag = "kind")]`-dispatched (REQ-O4).
//! Shorthand pitch/duration strings are validated at deserialize time
//! via custom `Deserialize` impls on [`common::OwnedPitch`] and
//! [`common::OwnedDuration`] (REQ-O12/O14).

use serde::{Deserialize, Serialize};

pub mod chord_progression;
pub mod common;
pub mod document_kind;
pub mod fretboard_shape;
pub mod interval_matrix;
pub mod pitch_circle;
pub mod scale_diagram;
pub mod score;
pub mod snippet;
pub mod tab;

pub use chord_progression::OwnedChordProgression;
pub use common::*;
pub use document_kind::DocumentKind;
pub use fretboard_shape::{OwnedFretValue, OwnedFretboardShape};
pub use interval_matrix::OwnedIntervalMatrix;
pub use pitch_circle::OwnedPitchCircle;
pub use scale_diagram::{OwnedOrientation, OwnedScaleDiagram};
pub use score::{
    OwnedMeasure, OwnedMeasurePart, OwnedPartDefinition, OwnedReading, OwnedScore,
    OwnedScoreSource, OwnedVoice,
};
pub use snippet::OwnedSnippet;
pub use tab::{OwnedTab, OwnedTabEvent};

/// Root document enum (REQ-O8). No `rename_all`; discriminant casing is verbatim PascalCase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Document {
    Snippet(OwnedSnippet),
    Tab(OwnedTab),
    FretboardShape(OwnedFretboardShape),
    PitchCircle(OwnedPitchCircle),
    ChordProgression(OwnedChordProgression),
    ScaleDiagram(OwnedScaleDiagram),
    IntervalMatrix(OwnedIntervalMatrix),
    Score(OwnedScore),
}

impl Document {
    /// Returns the serde tag discriminant for the active variant.
    pub fn kind_str(&self) -> &'static str {
        match self {
            Document::Snippet(_) => "Snippet",
            Document::Tab(_) => "Tab",
            Document::FretboardShape(_) => "FretboardShape",
            Document::PitchCircle(_) => "PitchCircle",
            Document::ChordProgression(_) => "ChordProgression",
            Document::ScaleDiagram(_) => "ScaleDiagram",
            Document::IntervalMatrix(_) => "IntervalMatrix",
            Document::Score(_) => "Score",
        }
    }
}
