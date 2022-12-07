use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use crate::note::note::Note;
use crate::note::pc::Pc;

pub mod chord_name;
pub mod octave_partition;
pub mod pc_set;
pub mod spelling;
pub mod voicing;
pub mod geometry;
pub mod interval_class;

pub use pc_set::PcSet;
pub use interval_class::IntervalClass;
pub use octave_partition::OctavePartition;
pub use voicing::{StackedIntervals, Voicing};
use crate::error::MusicSemanticsError;
use crate::note_collections::geometry::symmetry::transpositional::TranspositionalSymmetry;

/// Wraps a vector of [Note]s to provide some ordering guarantees on construction.
///
/// It entails all the same intervallic information as a [PcSet], but also
/// conveys note spelling information.
/// So you can think of it as "a [PcSet] with a defined note spelling."
/// It's the minimal required information to talk about e.g. "a C major chord"
/// in the abstract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSet(Vec<Note>);

impl NoteSet {
    /// This is the preferred way to created a [NoteSet], as it guarantees
    /// deduplication and sorting by [Pc].
    /// It is normalized to Pc::0 by default, or whatever Pc is passed in.
    pub fn new(mut notes: Vec<Note>, starting_note: Option<&Note>) -> Self {
        if notes.is_empty() {
            return Self(vec![]);
        }
        let orientation = starting_note.map_or(0, |n| u8::from(Pc::from(n)));
        notes.dedup_by(|a, b| Pc::from(a.clone()) == Pc::from(b.clone()));
        notes.sort_by(|a, b| {
            // We add 12 in the arithmetic because we want to ensure
            let a = (u8::from(Pc::from(a)) + 12 - orientation).rem_euclid(12);
            let b = (u8::from(Pc::from(b)) + 12 - orientation).rem_euclid(12);
            a.partial_cmp(&b).unwrap()
        });
        Self(notes)
    }

    /// Same as [NoteSet::new], but orders elements treating
    /// the first element of the [Vec] as [Pc::Pc0].
    pub fn starting_from_first_note(notes: Vec<Note>) -> Self {
        if notes.is_empty() {
            return Self(vec![]);
        }
        let starting_note = notes[0].clone();
        Self::new(notes, Some(&starting_note))
    }

    /// Retrieves the note n "steps" up in a [NoteSet], starting from a given
    /// note that is expected to be in the [NoteSet] itself. Here we define
    /// "step" arbitrarily as just any interval between adjacent elements in [self].
    /// This assumes the data in [self] is well-ordered,
    /// but the [NoteSet] constructor takes care of this.
    pub fn up_n_steps(&self, from: &Note, n: u8) -> Result<Note, MusicSemanticsError> {
        let index: usize = self.0.iter().position(|i| *i == *from)
            .ok_or(MusicSemanticsError::NotAMember(from.clone(), (**self).clone()))?;
        let n = (index + (n as usize)).rem_euclid(self.0.len());
        Ok(self.0[n].clone())
    }

    /// Indexed by [Note] instead of [Pc] as in
    /// [crate::note_collections::geometry::symmetry::find_transpositional_symmetries].
    /// See that function's docs for more details.
    pub fn find_transpositional_symmetries(&self) -> TranspositionalSymmetryMap {
        let pcs = PcSet::from(self);
        let mut symmetries = pcs.transpositional_symmetry();
        let mut indexed_by_note = HashMap::new();
        for (i, note) in self.iter().enumerate() {
            indexed_by_note.insert(
                note.clone(),
                symmetries.remove(&pcs[i]).unwrap(),
            );
        }
        indexed_by_note
    }
}

pub type TranspositionalSymmetryMap = HashMap<Note, HashSet<TranspositionalSymmetry>>;

impl Hash for NoteSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_empty() {
            "NoteSet:<empty>".hash(state);
        } else {
            for note in &self.0 {
                note.hash(state);
            }
        }
    }
}

impl Deref for NoteSet {
    type Target = Vec<Note>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

    #[test]
    fn test_n_steps_up() {
        let notes = NoteSet::new(vec![Note::C, Note::E, Note::G], None);
        assert_eq!(notes.up_n_steps(&Note::C, 1).unwrap(), Note::E);
        assert_eq!(notes.up_n_steps(&Note::C, 2).unwrap(), Note::G);
        assert_eq!(notes.up_n_steps(&Note::C, 3).unwrap(), Note::C);
        assert_eq!(notes.up_n_steps(&Note::C, 4).unwrap(), Note::E);
        assert_eq!(notes.up_n_steps(&Note::C, 0).unwrap(), Note::C);
        assert_eq!(notes.up_n_steps(&Note::E, 2).unwrap(), Note::C);
        assert_eq!(notes.up_n_steps(&Note::G, 2).unwrap(), Note::E);
    }
}