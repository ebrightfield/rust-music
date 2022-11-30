use crate::note::note::Note;
use crate::note::pc::Pc;
use anyhow::anyhow;

/// This is the MIDI-compliant formula for calculating how:
/// Note + octave = Pitch
fn calc_midi_note(note: &Note, octave: &u8) -> u8 {
    (octave + 1) * 12 + u8::from(Pc::from(note))
}

/// [Note] with octave information.
#[derive(Debug, Clone, Copy, PartialEq)]
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

    // TODO down_to_note method
    /// Returns the next [Pitch] above [self] whose note is equivalent to
    /// to the input [Note]. For when you want to "go up to G from B3".
    pub fn up_to_note(&self, note: &Note) -> anyhow::Result<Self> {
        let d = self.note.distance_up_to_note(note);
        Ok(self.at_distance_from(d as isize)?)
    }
}
