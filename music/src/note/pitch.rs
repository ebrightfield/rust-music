use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::note::note::Note;
use crate::note::pc::Pc;
use anyhow::anyhow;
use crate::note::spelling::Spelling;

/// This is the MIDI-compliant formula for calculating how:
/// Note + octave = Pitch
fn calc_midi_note(note: &Note, octave: &u8) -> u8 {
    (octave + 1) * 12 + u8::from(Pc::from(note))
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
    pub fn new(note: Note, octave: u8) -> anyhow::Result<Self> {
        if octave > 8 {
            return Err(anyhow!("Octave too high: {}", octave));
        }
        let midi_note = calc_midi_note(&note, &octave);
        Ok(Self {
            note,
            octave,
            midi_note,
        })
    }

    /// Produce a pitch from a MIDI note.
    pub fn from_midi(midi_note_value: u8) -> anyhow::Result<Self> {
        if midi_note_value >= 108 {
            return Err(anyhow!("Note is too high: {}", midi_note_value));
        }
        let octave = (midi_note_value / 12) - 1;
        if octave > 8 {
            return Err(anyhow!("Octave too high: {}", octave));
        }
        let pc = midi_note_value - (octave * 12);
        let pc = Pc::from(pc);
        let note = pc.notes().first().unwrap().clone();
        Ok(Self {
            note,
            octave,
            midi_note: midi_note_value,
        })
    }

    /// Control for spelling by including a "palette" of possible note values.
    pub fn spelled_as_in(midi_note_value: u8, notes: &Vec<Note>) -> anyhow::Result<Self> {
        let octave = (midi_note_value / 12) - 1;
        if octave > 8 {
            return Err(anyhow!("Octave too high: {}", octave));
        }
        let pc = midi_note_value - (octave * 12);
        let pc = Pc::from(pc);
        for note in notes {
            if Pc::from(note) == pc {
                return Ok(Self {
                    note: note.clone(),
                    octave,
                    midi_note: midi_note_value,
                });
            }
        }
        Err(anyhow!("{:?} not in the notes {:?}", pc.notes(), notes))
    }

    /// Subtract up or down from a pitch to arrive at another one.
    /// This does not control spelling.
    pub fn at_distance_from(&self, distance: isize) -> anyhow::Result<Self> {
        let new_pitch = self.midi_note as isize + distance;
        //let new_pitch = 0;
        let new_pitch = u8::try_from(new_pitch)
                .map_err( |_| anyhow!(
                    "Subtracting {} from {:?} goes beyond the bounds of practical musical pitches",
                    new_pitch,
                    self,
                )
            )?;
        Ok(Self::from_midi(new_pitch)?)
    }

    /// Compare pitches by their MIDI note, to equivocate over
    /// spellings but not octaves.
    pub fn is_same_frequency(&self, other: &Pitch) -> bool {
        self.midi_note == other.midi_note
    }

    /// Returns the next [Pitch] above [self] whose note is equivalent to
    /// to the input [Note]. For when you want to "go up to G from B3".
    pub fn up_to_note(&self, note: &Note) -> anyhow::Result<Self> {
        let d = self.note.distance_up_to_note(note);
        Ok(self.at_distance_from(d as isize)?)
    }

    /// Returns the next [Pitch] below [self] whose note is equivalent to
    /// to the input [Note]. For when you want to "go down to G from B3".
    pub fn down_to_note(&self, note: &Note) -> anyhow::Result<Self> {
        let d = self.note.distance_down_to_note(note);
        Ok(self.at_distance_from(d as isize)?)
    }

    /// Returns the number of letters up/down between self and other,
    /// accounting for octaves.
    pub fn diatonic_distance(&self, other: &Pitch) -> i32 {
        let self_diat = (self.octave * 7) as i32
            + i32::from(Spelling::from(self.note).letter);
        let other_diat = (other.octave * 7) as i32
            + i32::from(Spelling::from(other.note).letter);
        other_diat - self_diat
    }

    /// Shift a pitch by some number of octaves.
    pub fn raise_octaves(&self, n: isize) -> anyhow::Result<Self> {
        let i = Self::new(self.note, u8::try_from(self.octave as isize + n)?);
        Self::new(self.note, u8::try_from(self.octave as isize + n)?)
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

#[cfg(test)]
mod tests {
    use super::*;

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