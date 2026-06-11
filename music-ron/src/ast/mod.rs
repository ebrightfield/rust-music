//! Owned AST types deserialized from RON input.
//!
//! Each document variant (e.g. [`OwnedSnippet`], [`OwnedTab`]) carries
//! owned `String`/`Vec` fields suitable for serde. The top-level
//! [`Document`] enum is `#[serde(tag = "kind")]`-dispatched (REQ-O4).
//! Shorthand pitch/duration strings are validated at deserialize time
//! via custom `Deserialize` impls on [`common::OwnedPitch`] and
//! [`common::OwnedDuration`] (REQ-O12/O14).

use serde::{Deserialize, Serialize};

pub mod common;
pub mod document_kind;
pub mod snippet;
pub mod tab;
pub mod fretboard_shape;
pub mod pitch_circle;
pub mod chord_progression;
pub mod scale_diagram;
pub mod interval_matrix;

pub use document_kind::DocumentKind;
pub use common::*;
pub use snippet::OwnedSnippet;
pub use tab::{OwnedTab, OwnedTabEvent};
pub use fretboard_shape::{OwnedFretboardShape, OwnedFretValue};
pub use pitch_circle::OwnedPitchCircle;
pub use chord_progression::OwnedChordProgression;
pub use scale_diagram::{OwnedScaleDiagram, OwnedOrientation};
pub use interval_matrix::OwnedIntervalMatrix;

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
        }
    }
}
