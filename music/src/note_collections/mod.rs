use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use crate::note::note::Note;
use crate::note::pitch_class::Pc;

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
        let orientation = starting_note.map_or(0, |n| u8::from(&Pc::from(n)));
        notes.sort_by(|a, b| {
            // We add 12 in the arithmetic because we want to ensure
            let a = (u8::from(&Pc::from(a)) + 12 - orientation).rem_euclid(12);
            let b = (u8::from(&Pc::from(b)) + 12 - orientation).rem_euclid(12);
            a.partial_cmp(&b).unwrap()
        });
        notes.dedup_by(|a, b| Pc::from(&*a) == Pc::from(&*b));
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

    /// Same as the `up_n_steps` method, but in the downward direction.
    pub fn down_n_steps(&self, from: &Note, n: u8) -> Result<Note, MusicSemanticsError> {
        let index = self.0.iter().position(|i| *i == *from)
            .ok_or(MusicSemanticsError::NotAMember(from.clone(), (**self).clone()))?;
        let equivalent_up = self.0.len() - (n as usize).rem_euclid(self.0.len());
        let n = (index + equivalent_up).rem_euclid(self.0.len());
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

    /// Get a pitch N diatonic steps from given pitch within this note set.
    ///
    /// Steps can be positive (up) or negative (down). The octave is adjusted
    /// appropriately when wrapping around the note set.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting pitch
    /// * `steps` - Number of steps (positive = up, negative = down)
    ///
    /// # Returns
    ///
    /// The pitch at the given step distance, with appropriate octave adjustment.
    pub fn pitch_n_steps_from(
        &self,
        from: &crate::note::pitch::Pitch,
        steps: i8,
    ) -> Result<crate::note::pitch::Pitch, MusicSemanticsError> {
        use crate::note::pitch::Pitch;

        if self.0.is_empty() {
            return Err(MusicSemanticsError::EmptySetOfNotes);
        }

        // Find the closest note in the set to the pitch's note
        let from_note = self.closest_to_note(&from.note)?;
        let pos = self.0.iter()
            .position(|n| n == from_note)
            .ok_or(MusicSemanticsError::EmptySetOfNotes)?;

        let len = self.0.len() as i32;
        let new_pos_raw = pos as i32 + steps as i32;

        // Calculate position within the set (with wrapping)
        let new_pos = new_pos_raw.rem_euclid(len) as usize;
        let new_note = &self.0[new_pos];

        // Calculate octave change
        // Each full cycle through the set is one octave
        let octave_change = if new_pos_raw >= 0 {
            new_pos_raw / len
        } else {
            // For negative, we need to handle the division differently
            (new_pos_raw - len + 1) / len
        };

        let new_octave = (from.octave as i32 + octave_change) as u8;
        Pitch::new(new_note.clone(), new_octave)
    }

    /// Find the note in this set closest (by pitch class distance) to the given pitch.
    ///
    /// Returns the note that minimizes the distance around the pitch class circle.
    pub fn closest_to(&self, pitch: &crate::note::pitch::Pitch) -> Result<&Note, MusicSemanticsError> {
        self.closest_to_note(&pitch.note)
    }

    /// Find the note in this set closest (by pitch class distance) to the given note.
    ///
    /// Returns the note that minimizes the distance around the pitch class circle.
    pub fn closest_to_note(&self, note: &Note) -> Result<&Note, MusicSemanticsError> {
        if self.0.is_empty() {
            return Err(MusicSemanticsError::EmptySetOfNotes);
        }

        let target_pc = Pc::from(note);

        self.0
            .iter()
            .min_by_key(|n| {
                let pc = Pc::from(*n);
                let up = target_pc.distance_up_to(&pc);
                let down = target_pc.distance_down_to(&pc);
                up.min(down)
            })
            .ok_or(MusicSemanticsError::EmptySetOfNotes)
    }

    /// Find the note in this set enharmonically equivalent to the given note.
    ///
    /// Returns the note if found, or None if no enharmonic match exists.
    pub fn find_enharmonic(&self, note: &Note) -> Option<&Note> {
        let target_pc = Pc::from(note);
        self.0.iter().find(|n| Pc::from(*n) == target_pc)
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

    #[test]
    fn test_closest_to_note() {
        // C major triad: C, E, G
        let notes = NoteSet::new(vec![Note::C, Note::E, Note::G], None);

        // Exact matches
        assert_eq!(notes.closest_to_note(&Note::C).unwrap(), &Note::C);
        assert_eq!(notes.closest_to_note(&Note::E).unwrap(), &Note::E);
        assert_eq!(notes.closest_to_note(&Note::G).unwrap(), &Note::G);

        // D is between C and E - closer to C (2 semitones) than E (2 semitones)
        // tie goes to first in set
        let closest = notes.closest_to_note(&Note::D).unwrap();
        assert!(closest == &Note::C || closest == &Note::E);

        // F is between E (1 semitone up) and G (2 semitones down) - closer to E
        assert_eq!(notes.closest_to_note(&Note::F).unwrap(), &Note::E);

        // A is between G (2 up) and C (3 up) - closer to G
        assert_eq!(notes.closest_to_note(&Note::A).unwrap(), &Note::G);

        // B is between G (4 up) and C (1 up) - closer to C
        assert_eq!(notes.closest_to_note(&Note::B).unwrap(), &Note::C);
    }

    #[test]
    fn test_pitch_n_steps_from() {
        use crate::note::pitch::Pitch;

        // C major scale: C, D, E, F, G, A, B
        let scale = NoteSet::new(
            vec![Note::C, Note::D, Note::E, Note::F, Note::G, Note::A, Note::B],
            None,
        );

        let c4 = Pitch::new(Note::C, 4).unwrap();

        // Step up within octave
        let d4 = scale.pitch_n_steps_from(&c4, 1).unwrap();
        assert_eq!(d4.note, Note::D);
        assert_eq!(d4.octave, 4);

        let g4 = scale.pitch_n_steps_from(&c4, 4).unwrap();
        assert_eq!(g4.note, Note::G);
        assert_eq!(g4.octave, 4);

        // Step across octave boundary
        let c5 = scale.pitch_n_steps_from(&c4, 7).unwrap();
        assert_eq!(c5.note, Note::C);
        assert_eq!(c5.octave, 5);

        let d5 = scale.pitch_n_steps_from(&c4, 8).unwrap();
        assert_eq!(d5.note, Note::D);
        assert_eq!(d5.octave, 5);

        // Step down within octave
        let b3 = scale.pitch_n_steps_from(&c4, -1).unwrap();
        assert_eq!(b3.note, Note::B);
        assert_eq!(b3.octave, 3);

        let a3 = scale.pitch_n_steps_from(&c4, -2).unwrap();
        assert_eq!(a3.note, Note::A);
        assert_eq!(a3.octave, 3);

        // Step down across octave
        let c3 = scale.pitch_n_steps_from(&c4, -7).unwrap();
        assert_eq!(c3.note, Note::C);
        assert_eq!(c3.octave, 3);
    }

    #[test]
    fn test_pitch_n_steps_from_triad() {
        use crate::note::pitch::Pitch;

        // C major triad: C, E, G (3 notes per octave)
        let triad = NoteSet::new(vec![Note::C, Note::E, Note::G], None);
        let c4 = Pitch::new(Note::C, 4).unwrap();

        // Up through triad
        let e4 = triad.pitch_n_steps_from(&c4, 1).unwrap();
        assert_eq!(e4.note, Note::E);
        assert_eq!(e4.octave, 4);

        let g4 = triad.pitch_n_steps_from(&c4, 2).unwrap();
        assert_eq!(g4.note, Note::G);
        assert_eq!(g4.octave, 4);

        let c5 = triad.pitch_n_steps_from(&c4, 3).unwrap();
        assert_eq!(c5.note, Note::C);
        assert_eq!(c5.octave, 5);

        let e5 = triad.pitch_n_steps_from(&c4, 4).unwrap();
        assert_eq!(e5.note, Note::E);
        assert_eq!(e5.octave, 5);

        // Down through triad
        let g3 = triad.pitch_n_steps_from(&c4, -1).unwrap();
        assert_eq!(g3.note, Note::G);
        assert_eq!(g3.octave, 3);

        let e3 = triad.pitch_n_steps_from(&c4, -2).unwrap();
        assert_eq!(e3.note, Note::E);
        assert_eq!(e3.octave, 3);

        let c3 = triad.pitch_n_steps_from(&c4, -3).unwrap();
        assert_eq!(c3.note, Note::C);
        assert_eq!(c3.octave, 3);
    }

    #[test]
    fn test_find_enharmonic() {
        let notes = NoteSet::new(vec![Note::C, Note::Des, Note::E], None);

        // Exact match
        assert_eq!(notes.find_enharmonic(&Note::Des), Some(&Note::Des));

        // Enharmonic match (C# == Db)
        assert_eq!(notes.find_enharmonic(&Note::Cis), Some(&Note::Des));

        // No match
        assert_eq!(notes.find_enharmonic(&Note::D), None);
    }
}