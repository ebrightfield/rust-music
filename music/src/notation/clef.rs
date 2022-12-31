use crate::note::note::Note;
use crate::note::pitch::Pitch;

/// A simple enum, with descriptive functions that return
/// [Pitch] instances for the top, bottom, and middle of the clef.
///
/// We use this type to assist in normalizing the octave register
/// of pitch content.
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
                Pitch::new(Note::E, 4).unwrap(),
                Pitch::new(Note::F, 5).unwrap(),
            ),
            Clef::Treble8va => (
                Pitch::new(Note::E, 5).unwrap(),
                Pitch::new(Note::F, 6).unwrap(),
            ),
            Clef::Treble8ba => (
                Pitch::new(Note::E, 3).unwrap(),
                Pitch::new(Note::F, 4).unwrap(),
            ),
            Clef::Bass => (
                Pitch::new(Note::G, 2).unwrap(),
                Pitch::new(Note::A, 3).unwrap(),
            ),
        }
    }

    /// Returns the middle line of a clef. Useful for octave normalization.
    pub fn middle(&self) -> Pitch {
        match &self {
            Clef::Treble => Pitch::new(Note::B, 4).unwrap(),
            Clef::Treble8va => Pitch::new(Note::B, 5).unwrap(),
            Clef::Treble8ba => Pitch::new(Note::B, 3).unwrap(),
            Clef::Bass => Pitch::new(Note::D, 3).unwrap(),
        }
    }
}
