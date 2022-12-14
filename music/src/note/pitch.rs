use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::note::note::Note;
use crate::note::pitch_class::Pc;
use crate::error::MusicSemanticsError;
use crate::note::spelling::Spelling;
use crate::note_collections::spelling::HasSpelling;

// TODO Expand range out to MIDI note 128, this may require checking some of the
//    guitar search algorithms

pub const MIDDLE_C: u8 = 60;

/// This is the MIDI-compliant formula for calculating how:
/// Note + octave = Pitch
fn calc_midi_note(note: &Note, octave: &u8) -> u8 {
    (octave + 1) * 12 + u8::from(&Pc::from(note))
}

/// [Note] with octave information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pitch {
    /// Associated [Note] instance.
    pub note: Note,
    /// The octave register. Middle C = C4.
    pub octave: u8,
    /// The associated MIDI note, which also serves as a good means of measurement in
    /// semitone space. Middle C = 60.
    pub midi_note: u8,
}

impl Pitch {
    /// Sanitizes for octave height
    pub fn new(note: Note, octave: u8) -> Result<Self, MusicSemanticsError> {
        if octave > 8 {
            return Err(MusicSemanticsError::OctaveTooHigh(octave));
        }
        let midi_note = calc_midi_note(&note, &octave);
        Ok(Self {
            note,
            octave,
            midi_note,
        })
    }

    /// Produce a pitch from a MIDI note value. Middle C = 60.
    pub fn from_midi(midi_note_value: u8) -> Result<Self, MusicSemanticsError> {
        if midi_note_value >= 108 {
            return Err(MusicSemanticsError::MidiTooHigh(midi_note_value));
        }
        let octave = (midi_note_value / 12) - 1;
        if octave > 8 {
            return Err(MusicSemanticsError::OctaveTooHigh(octave));
        }
        let pc = midi_note_value - (octave * 12);
        let pc = Pc::from(&pc);
        let note = pc.notes().first().unwrap().clone();
        Ok(Self {
            note,
            octave,
            midi_note: midi_note_value,
        })
    }

    /// Control for spelling by including a "palette" of possible note values.
    pub fn new_spelled_as_in(midi_note_value: u8, notes: &Vec<Note>) -> Result<Self, MusicSemanticsError> {
        let octave = (midi_note_value / 12) - 1;
        if octave > 8 {
            return Err(MusicSemanticsError::OctaveTooHigh(octave));
        }
        let pc = midi_note_value - (octave * 12);
        let pc = Pc::from(&pc);
        for note in notes {
            if Pc::from(note) == pc {
                return Ok(Self {
                    note: note.clone(),
                    octave,
                    midi_note: midi_note_value,
                });
            }
        }
        Err(MusicSemanticsError::NotAMember(
            pc.notes().first().unwrap().clone(),
            notes.clone())
        )
    }

    /// Subtract up or down from a pitch to arrive at another one.
    /// This does not control spelling.
    pub fn at_distance_from(&self, distance: isize) -> Result<Self, MusicSemanticsError> {
        let new_pitch = self.midi_note as isize + distance;
        let new_pitch = u8::try_from(new_pitch)
            .map_err(|_| MusicSemanticsError::OutOfBoundsLower(self.midi_note))?;
        Self::from_midi(new_pitch)
    }

    /// Compare pitches by their MIDI note, to equivocate over
    /// spellings but not octaves.
    pub fn is_same_frequency(&self, other: &Pitch) -> bool {
        self.midi_note == other.midi_note
    }

    /// Returns the next [Pitch] above [self] whose note is equivalent to
    /// to the input [Note]. For when you want to "go up to G from B3".
    pub fn up_to_note(&self, note: &Note) -> Result<Self, MusicSemanticsError> {
        let d = self.note.distance_up_to_note(note);
        self.at_distance_from(d as isize)?.spelled_as_in(&vec![note.clone()])
    }

    /// Returns the next [Pitch] below [self] whose note is equivalent to
    /// to the input [Note]. For when you want to "go down to G from B3".
    pub fn down_to_note(&self, note: &Note) -> Result<Self, MusicSemanticsError> {
        let d = self.note.distance_down_to_note(note);
        self.at_distance_from((d as isize) * -1)?.spelled_as_in(&vec![note.clone()])
    }

    /// Returns the number of letters up/down between self and other,
    /// accounting for octaves.
    pub fn diatonic_distance(&self, other: &Pitch) -> i32 {
        let self_diat = (self.octave * 7) as i32
            + i32::from(&Spelling::from(&self.note).letter);
        let other_diat = (other.octave * 7) as i32
            + i32::from(&Spelling::from(&other.note).letter);
        other_diat - self_diat
    }

    /// Shift a pitch by some number of octaves.
    pub fn raise_octaves(&self, n: isize) -> Result<Self, MusicSemanticsError> {
        Self::new(self.note, u8::try_from(self.octave as isize + n)
            .map_err(|_| MusicSemanticsError::MidiTooHigh(u8::MAX))?
        )
    }
}

impl Display for Pitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.note, self.octave)
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.midi_note.partial_cmp(&other.midi_note)
    }
}

impl Hash for Pitch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.note.hash(state);
        self.octave.hash(state);
    }
}

/// As this is meant to afford a shorthand syntax, this _will_ unwrap the pitch.
/// If that's not the behavior you want, use `Pitch::new` directly.
#[macro_export]
macro_rules! pitch {
    (bis, $octave:expr) => { Pitch::new(Note::Bis, $octave).unwrap()};
    (c, $octave:expr) => { Pitch::new(Note::C, $octave).unwrap()};
    (deses, $octave:expr) => { Pitch::new(Note::Deses, $octave).unwrap()};
    (cis, $octave:expr) => { Pitch::new(Note::Cis, $octave).unwrap()};
    (des, $octave:expr) => { Pitch::new(Note::Des, $octave).unwrap()};
    (d, $octave:expr) => { Pitch::new(Note::D, $octave).unwrap()};
    (cisis, $octave:expr) => { Pitch::new(Note::Cisis, $octave).unwrap()};
    (eeses, $octave:expr) => { Pitch::new(Note::Eeses, $octave).unwrap()};
    (dis, $octave:expr) => { Pitch::new(Note::Dis, $octave).unwrap()};
    (ees, $octave:expr) => { Pitch::new(Note::Ees, $octave).unwrap()};
    (e, $octave:expr) => { Pitch::new(Note::E, $octave).unwrap()};
    (disis, $octave:expr) => { Pitch::new(Note::Disis, $octave).unwrap()};
    (fes, $octave:expr) => { Pitch::new(Note::Fes, $octave).unwrap()};
    (f, $octave:expr) => { Pitch::new(Note::F, $octave).unwrap()};
    (eis, $octave:expr) => { Pitch::new(Note::Eis, $octave).unwrap()};
    (geses, $octave:expr) => { Pitch::new(Note::Geses, $octave).unwrap()};
    (fis, $octave:expr) => { Pitch::new(Note::Fis, $octave).unwrap()};
    (ges, $octave:expr) => { Pitch::new(Note::Ges, $octave).unwrap()};
    (g, $octave:expr) => { Pitch::new(Note::G, $octave).unwrap()};
    (fisis, $octave:expr) => { Pitch::new(Note::Fisis, $octave).unwrap()};
    (aeses, $octave:expr) => { Pitch::new(Note::Aeses, $octave).unwrap()};
    (gis, $octave:expr) => { Pitch::new(Note::Gis, $octave).unwrap()};
    (aes, $octave:expr) => { Pitch::new(Note::Aes, $octave).unwrap()};
    (a, $octave:expr) => { Pitch::new(Note::A, $octave).unwrap()};
    (gisis, $octave:expr) => { Pitch::new(Note::Gisis, $octave).unwrap()};
    (beses, $octave:expr) => { Pitch::new(Note::Beses, $octave).unwrap()};
    (ais, $octave:expr) => { Pitch::new(Note::Ais, $octave).unwrap()};
    (bes, $octave:expr) => { Pitch::new(Note::Bes, $octave).unwrap()};
    (b, $octave:expr) => { Pitch::new(Note::B, $octave).unwrap()};
    (ces, $octave:expr) => { Pitch::new(Note::Ces, $octave).unwrap()};
    (aisis, $octave:expr) => { Pitch::new(Note::Aisis, $octave).unwrap()};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_macro() {
        assert_eq!(Pitch::new(Note::C, 4).unwrap(), pitch!(c, 4));
    }

    #[test]
    fn diatonic_distance_works() {
        let p1 = Pitch::new(Note::C, 5).unwrap();
        let p2 = Pitch::new(Note::B, 4).unwrap();
        assert_eq!(p1.diatonic_distance(&p2), -1);
        let p2 = Pitch::new(Note::B, 5).unwrap();
        assert_eq!(p1.diatonic_distance(&p2), 6);
        let p2 = Pitch::new(Note::B, 6).unwrap();
        assert_eq!(p1.diatonic_distance(&p2), 13);
        let p1 = Pitch::new(Note::G, 3).unwrap();
        let p2 = Pitch::new(Note::F, 3).unwrap();
        assert_eq!(p1.diatonic_distance(&p2), -1);
        let p2 = Pitch::new(Note::F, 4).unwrap();
        assert_eq!(p1.diatonic_distance(&p2), 6);
    }
}