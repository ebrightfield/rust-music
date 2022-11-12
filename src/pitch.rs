use crate::note::note::Note;
use crate::note::pc::Pc;
use anyhow::anyhow;

fn calc_midi_note(note: &Note, octave: &u8) -> u8 {
    (octave + 1) * 12 + u8::from(Pc::from(note))
}

/// Note with octave information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pitch {
    pub note: Note,
    pub octave: u8,
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
}
