use crate::note::note::Note;
use crate::note::pitch::Pitch;

/// A simple enum, with descriptive functions that return
/// [Pitch] instances for the top, bottom, and middle of the clef.
///
/// We use this type to assist in normalizing the octave register
/// of pitch content.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clef {
    Treble,
    Treble8va,
    Treble8ba,
    Bass,
}

impl Clef {
    /// Returns pitch of the bottom and top lines of the clef respectively.
    pub fn bounds(&self) -> (Pitch, Pitch) {
        match &self {
            Clef::Treble => (
                Pitch::new(Note::E, 4),
                Pitch::new(Note::F, 5),
            ),
            Clef::Treble8va => (
                Pitch::new(Note::E, 5),
                Pitch::new(Note::F, 6),
            ),
            Clef::Treble8ba => (
                Pitch::new(Note::E, 3),
                Pitch::new(Note::F, 4),
            ),
            Clef::Bass => (
                Pitch::new(Note::G, 2),
                Pitch::new(Note::A, 3),
            ),
        }
    }

    /// Returns the middle line of a clef. Useful for octave normalization.
    pub fn middle(&self) -> Pitch {
        match &self {
            Clef::Treble => Pitch::new(Note::B, 4),
            Clef::Treble8va => Pitch::new(Note::B, 5),
            Clef::Treble8ba => Pitch::new(Note::B, 3),
            Clef::Bass => Pitch::new(Note::D, 3),
        }
    }
}
