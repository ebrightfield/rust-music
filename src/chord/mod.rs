use crate::chord::chord_name::ChordName;
use crate::note::note::Note;
use crate::note::pc::Pc;

pub mod chord_name;
mod four_note_chords;
pub mod octave_partition;
pub mod pc_set;
mod three_note_chords;
pub mod spelling;
pub mod voicing;
pub mod geometry;


pub trait NumUniqueNotes {
    fn unique_notes(&self) -> usize;
}

// I *could* put a "naming" trait on any struct where it's pitched enough to have a name.
// pub trait HasChordName {
//     fn chord_name(&self) -> ChordName;
// }

/// Wraps a [Vec<Note>] to provide some ordering guarantees on construction.
/// This is music-theoretically the closest thing to e.g. a "C major chord" in the abstract.
/// It contains no information about how that chord might be played, but it does entail
/// certain intervallic properties and therefore can be converted to [PcSet], or vice versa if
/// a root [Note] is provided (spellings for the remaining [Pc] can be inferred).
#[derive(Debug, Clone, PartialEq)]
pub struct NoteSet(Vec<Note>);

impl NoteSet {
    /// Deduped and ordered by [Pc], normalized to Pc::0 by default,
    /// or whatever Pc is passed in.
    pub fn new(mut notes: Vec<Note>, starting_note: Option<&Note>) -> Self {
        notes.dedup_by(|a, b| Pc::from(a.clone()) == Pc::from(b.clone()));
        let orientation = starting_note.map_or(0, |n| u8::from(Pc::from(n)));
        notes.sort_by(|a, b| {
            // We add 12 in the arithmetic because we want to ensure
            let a = (u8::from(Pc::from(a)) + 12 - orientation).rem_euclid(12);
            let b = (u8::from(Pc::from(b)) + 12 - orientation).rem_euclid(12);
            a.partial_cmp(&b).unwrap()
        });
        Self(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_set_constructor() {
        let notes = NoteSet::new(vec![Note::D, Note::Cisis, Note::C], None);
        let could_be = NoteSet::new(vec![Note::C, Note::D], None);
        let could_be2 = NoteSet(vec![Note::C, Note::Cisis]);
        assert!(could_be == notes || could_be2 == notes);
        let notes = NoteSet::new(vec![Note::D, Note::Cis, Note::C], None);
        let should_be = NoteSet::new(vec![Note::C, Note::Cis, Note::D], None);
        assert_eq!(notes, should_be);
        let notes = NoteSet::new(vec![Note::D, Note::Cis, Note::C], Some(&Note::Cis));
        let should_be = NoteSet(vec![Note::Cis, Note::D, Note::C]);
        assert_eq!(notes, should_be);
    }
}