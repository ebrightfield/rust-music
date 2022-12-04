use crate::note::note::Note;
use crate::note::pitch::Pitch;

pub enum Clef {
    Treble,
    // Treble8va,
    // Treble8ba,
    // Bass,
}

impl Clef {
    /// Returns pitch of the bottom and top lines of the clef respectively.
    pub fn bounds(&self) -> (Pitch, Pitch) {
        match &self {
            Clef::Treble => (
                Pitch::new(Note::E, 4).unwrap(),
                Pitch::new(Note::F, 5).unwrap(),
            )
        }
    }

    /// Returns the middle line of a clef. Useful for octave normalization.
    pub fn middle(&self) -> Pitch {
        match &self {
            Clef::Treble => Pitch::new(Note::B, 4).unwrap()
        }
    }
}
