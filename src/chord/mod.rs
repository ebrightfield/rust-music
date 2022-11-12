use crate::chord::chord_name::ChordName;

pub mod chord_name;
mod four_note_chords;
mod octave_partition;
pub mod pc_set;
mod three_note_chords;
pub mod spelling;
mod voicing;

pub trait NumUniqueNotes {
    fn unique_notes(&self) -> usize;
}

pub trait HasChordName {
    fn chord_name(&self) -> ChordName;
}
