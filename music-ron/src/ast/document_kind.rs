// REQ-O7 (R3-support): DocumentKind trait.

use super::Document;

/// Implemented by each of the seven `Document` variant payload types.
/// The `KIND` constant MUST match the serde `tag` discriminant verbatim.
pub trait DocumentKind: Sized {
    const KIND: &'static str;

    /// Extract this variant's payload from a `Document`, returning `None`
    /// if the document is a different variant.
    fn from_document(doc: Document) -> Option<Self>;
}

impl DocumentKind for super::OwnedSnippet {
    const KIND: &'static str = "Snippet";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::Snippet(v) => Some(v),
            _ => None,
        }
    }
}

impl DocumentKind for super::OwnedTab {
    const KIND: &'static str = "Tab";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::Tab(v) => Some(v),
            _ => None,
        }
    }
}

impl DocumentKind for super::OwnedFretboardShape {
    const KIND: &'static str = "FretboardShape";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::FretboardShape(v) => Some(v),
            _ => None,
        }
    }
}

impl DocumentKind for super::OwnedPitchCircle {
    const KIND: &'static str = "PitchCircle";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::PitchCircle(v) => Some(v),
            _ => None,
        }
    }
}

impl DocumentKind for super::OwnedChordProgression {
    const KIND: &'static str = "ChordProgression";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::ChordProgression(v) => Some(v),
            _ => None,
        }
    }
}

impl DocumentKind for super::OwnedScaleDiagram {
    const KIND: &'static str = "ScaleDiagram";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::ScaleDiagram(v) => Some(v),
            _ => None,
        }
    }
}

impl DocumentKind for super::OwnedIntervalMatrix {
    const KIND: &'static str = "IntervalMatrix";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::IntervalMatrix(v) => Some(v),
            _ => None,
        }
    }
}

impl DocumentKind for super::OwnedScore {
    const KIND: &'static str = "Score";
    fn from_document(doc: Document) -> Option<Self> {
        match doc {
            Document::Score(v) => Some(v),
            _ => None,
        }
    }
}
