use crate::note::note::Note;
use crate::note::pitch_class::Pc;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

pub mod chord_name;
pub mod geometry;
pub mod interval_class;
pub mod octave_partition;
pub mod pc_set;
pub mod spelling;
pub mod voicing;

use crate::error::MusicSemanticsError;
use crate::note_collections::geometry::symmetry::transpositional::TranspositionalSymmetry;
pub use interval_class::IntervalClass;
pub use octave_partition::OctavePartition;
pub use pc_set::{AsPcSlice, PcContent, PcShape};
pub use voicing::{StackedIntervals, Voicing};

/// Wraps a vector of [Note]s to provide some ordering guarantees on construction.
///
/// It entails all the same intervallic information as a [PcShape], but also
/// conveys note spelling information.
/// So you can think of it as "a [PcShape] with a defined note spelling."
/// It's the minimal required information to talk about e.g. "a C major chord"
/// in the abstract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteSet(Vec<Note>);

impl NoteSet {
    /// Create a [NoteSet], sorted by [Pc] ascending from Pc::0 and deduplicated
    /// by pitch class.
    ///
    /// For a sort anchored to a specific root note, use [`NoteSet::with_root`].
    pub fn new(notes: Vec<Note>) -> Self {
        Self::build(notes, None)
    }

    /// Create a [NoteSet], sorted by [Pc] ascending from `root` and deduplicated
    /// by pitch class. The `root` does not need to appear in `notes`.
    pub fn with_root(notes: Vec<Note>, root: &Note) -> Self {
        Self::build(notes, Some(root))
    }

    /// Same as [`NoteSet::new`], but treats the first element of the vector
    /// as the root.
    pub fn starting_from_first_note(notes: Vec<Note>) -> Self {
        if notes.is_empty() {
            return Self(vec![]);
        }
        let starting_note = notes[0];
        Self::with_root(notes, &starting_note)
    }

    fn build(mut notes: Vec<Note>, root: Option<&Note>) -> Self {
        if notes.is_empty() {
            return Self(vec![]);
        }
        let orientation = root.map_or(0, |n| u8::from(&Pc::from(n)));
        notes.sort_by(|a, b| {
            let a = (u8::from(&Pc::from(a)) + 12 - orientation).rem_euclid(12);
            let b = (u8::from(&Pc::from(b)) + 12 - orientation).rem_euclid(12);
            a.partial_cmp(&b).unwrap()
        });
        notes.dedup_by(|a, b| Pc::from(&*a) == Pc::from(&*b));
        Self(notes)
    }

    /// Retrieves the note n "steps" up in a [NoteSet], starting from a given
    /// note that is expected to be in the [NoteSet] itself. Here we define
    /// "step" arbitrarily as just any interval between adjacent elements in [self].
    /// This assumes the data in [self] is well-ordered,
    /// but the [NoteSet] constructor takes care of this.
    pub fn up_n_steps(&self, from: &Note, n: u8) -> Result<Note, MusicSemanticsError> {
        let index: usize = self
            .0
            .iter()
            .position(|i| *i == *from)
            .ok_or(MusicSemanticsError::NotAMember(*from, (**self).clone()))?;
        let n = (index + (n as usize)).rem_euclid(self.0.len());
        Ok(self.0[n])
    }

    /// Same as the `up_n_steps` method, but in the downward direction.
    pub fn down_n_steps(&self, from: &Note, n: u8) -> Result<Note, MusicSemanticsError> {
        let index = self
            .0
            .iter()
            .position(|i| *i == *from)
            .ok_or(MusicSemanticsError::NotAMember(*from, (**self).clone()))?;
        let equivalent_up = self.0.len() - (n as usize).rem_euclid(self.0.len());
        let n = (index + equivalent_up).rem_euclid(self.0.len());
        Ok(self.0[n])
    }

    /// Indexed by [Note] instead of [Pc] as in
    /// [crate::note_collections::geometry::symmetry::find_transpositional_symmetries].
    /// See that function's docs for more details.
    pub fn find_transpositional_symmetries(&self) -> TranspositionalSymmetryMap {
        let content = PcContent::from(self);
        let pcs = content.to_shape();
        let mut symmetries = pcs.transpositional_symmetry();
        let mut indexed_by_note = HashMap::new();
        for (i, note) in self.iter().enumerate() {
            indexed_by_note.insert(*note, symmetries.remove(&pcs[i]).unwrap());
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
        let pos = self
            .0
            .iter()
            .position(|n| n == from_note)
            .ok_or(MusicSemanticsError::EmptySetOfNotes)?;

        let len = self.0.len() as i32;

        // Walk one note at a time in MIDI space, then spell the arrival with
        // its set note so the written octave follows that spelling (a C♭
        // reached from B♭4 is C♭5 = MIDI 71; a B♯ reached from A♯3 is
        // B♯3 = MIDI 60). Each step moves to the next occurrence of the
        // neighbouring note's pitch class — a full octave when it repeats the
        // current one. The walk starts from `from_note` in `from`'s sounding
        // octave block (MIDI C..B), which is `from` itself whenever its pitch
        // class is in the set. Indices are not used for octave tracking
        // because the set may be anchored to any root (e.g. a Bb-rooted scale
        // is ordered [Bb, C, D, …]); see `test_pitch_n_steps_from_non_c_root`.
        let mut cur = pos as i32;
        let mut midi =
            i16::from(from.midi_note / 12 * 12) + i16::from(u8::from(&Pc::from(from_note)));
        if steps > 0 {
            for _ in 0..steps {
                let next = cur + 1;
                let cur_pc = Pc::from(&self.0[cur.rem_euclid(len) as usize]);
                let next_pc = Pc::from(&self.0[next.rem_euclid(len) as usize]);
                let up = cur_pc.distance_up_to(&next_pc);
                midi += i16::from(if up == 0 { 12 } else { up });
                cur = next;
            }
        } else {
            for _ in 0..(-steps) {
                let prev = cur - 1;
                let cur_pc = Pc::from(&self.0[cur.rem_euclid(len) as usize]);
                let prev_pc = Pc::from(&self.0[prev.rem_euclid(len) as usize]);
                let down = cur_pc.distance_down_to(&prev_pc);
                midi -= i16::from(if down == 0 { 12 } else { down });
                cur = prev;
            }
        }

        let new_note = self.0[cur.rem_euclid(len) as usize];
        let midi = u8::try_from(midi).map_err(|_| {
            if midi < 0 {
                MusicSemanticsError::OutOfBoundsLower(from.midi_note)
            } else {
                MusicSemanticsError::OutOfBoundsUpper(from.midi_note)
            }
        })?;
        Pitch::from_midi_as(midi, new_note)
    }

    /// Find the note in this set closest (by pitch class distance) to the given pitch.
    ///
    /// Returns the note that minimizes the distance around the pitch class circle.
    pub fn closest_to(
        &self,
        pitch: &crate::note::pitch::Pitch,
    ) -> Result<&Note, MusicSemanticsError> {
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

impl IntoIterator for NoteSet {
    type Item = Note;
    type IntoIter = std::vec::IntoIter<Note>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a NoteSet {
    type Item = &'a Note;
    type IntoIter = std::slice::Iter<'a, Note>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_set_constructor() {
        // Stable sort by pitch class preserves the relative order of enharmonics,
        // and dedup_by keeps the first of adjacent equal elements.
        //
        // Input [D, Cisis, C]: sort keys are [2, 2, 0] → stably sorted to
        // [C(0), D(2), Cisis(2)] → deduped → [C, D].
        let notes = NoteSet::new(vec![Note::D, Note::Cisis, Note::C]);
        assert_eq!(notes, NoteSet(vec![Note::C, Note::D]));

        // To make Cisis win, it must come before D in the input:
        // [Cisis, D, C] → [C, Cisis, D] → [C, Cisis].
        let notes = NoteSet::new(vec![Note::Cisis, Note::D, Note::C]);
        assert_eq!(notes, NoteSet(vec![Note::C, Note::Cisis]));

        let notes = NoteSet::new(vec![Note::D, Note::Cis, Note::C]);
        let should_be = NoteSet::new(vec![Note::C, Note::Cis, Note::D]);
        assert_eq!(notes, should_be);
        let notes = NoteSet::with_root(vec![Note::D, Note::Cis, Note::C], &Note::Cis);
        let should_be = NoteSet(vec![Note::Cis, Note::D, Note::C]);
        assert_eq!(notes, should_be);
    }

    #[test]
    fn test_n_steps_up() {
        let notes = NoteSet::new(vec![Note::C, Note::E, Note::G]);
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
        let notes = NoteSet::new(vec![Note::C, Note::E, Note::G]);

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
        let scale = NoteSet::new(vec![
            Note::C,
            Note::D,
            Note::E,
            Note::F,
            Note::G,
            Note::A,
            Note::B,
        ]);

        let c4 = Pitch::new(Note::C, 4);

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
        let triad = NoteSet::new(vec![Note::C, Note::E, Note::G]);
        let c4 = Pitch::new(Note::C, 4);

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
    fn test_pitch_n_steps_from_non_c_root() {
        use crate::note::pitch::Pitch;

        // Bb major scale anchored to Bb: ordered [Bb, C, D, Eb, F, G, A].
        // Regression: octave must increment at the C boundary (mid-array), not
        // at the array wrap point. Previously stepping up from Bb4 gave C4
        // (midi 60, a major-7th DOWN) instead of C5 (midi 72).
        let scale = NoteSet::with_root(
            vec![
                Note::Bes,
                Note::C,
                Note::D,
                Note::Ees,
                Note::F,
                Note::G,
                Note::A,
            ],
            &Note::Bes,
        );
        let bb4 = Pitch::new(Note::Bes, 4);

        // Ascending one step crosses the C boundary → C5, monotonically up.
        let c5 = scale.pitch_n_steps_from(&bb4, 1).unwrap();
        assert_eq!(c5.note, Note::C);
        assert_eq!(c5.octave, 5);
        assert_eq!(c5.midi_note, 72);

        // The whole ascending scale must be strictly increasing in MIDI.
        let mut prev = bb4.midi_note;
        for steps in 1..=7 {
            let p = scale.pitch_n_steps_from(&bb4, steps).unwrap();
            assert!(
                p.midi_note > prev,
                "step {steps}: {}{} (midi {}) should be higher than previous midi {prev}",
                p.note,
                p.octave,
                p.midi_note,
            );
            prev = p.midi_note;
        }
        // Top of the scale: Bb5 an octave above the start.
        let bb5 = scale.pitch_n_steps_from(&bb4, 7).unwrap();
        assert_eq!(bb5.note, Note::Bes);
        assert_eq!(bb5.octave, 5);
        assert_eq!(bb5.midi_note, bb4.midi_note + 12);

        // Descending must be strictly decreasing too.
        let mut prev = bb4.midi_note;
        for steps in 1..=7 {
            let p = scale.pitch_n_steps_from(&bb4, -(steps as i8)).unwrap();
            assert!(
                p.midi_note < prev,
                "step -{steps}: {}{} (midi {}) should be lower than previous midi {prev}",
                p.note,
                p.octave,
                p.midi_note,
            );
            prev = p.midi_note;
        }
        // A is the note just below Bb (a step down), at octave 4.
        let a4 = scale.pitch_n_steps_from(&bb4, -1).unwrap();
        assert_eq!(a4.note, Note::A);
        assert_eq!(a4.octave, 4);
        assert_eq!(a4.midi_note, 69);
    }

    #[test]
    fn test_pitch_n_steps_from_spells_c_flat_in_its_written_octave() {
        use crate::note::pitch::Pitch;

        // G♭ major contains C♭. Stepping up from B♭4 (70) reaches MIDI 71,
        // which is written C♭5 — not C♭4 (MIDI 59).
        let g_flat_major = NoteSet::new(vec![
            Note::Ges,
            Note::Aes,
            Note::Bes,
            Note::Ces,
            Note::Des,
            Note::Ees,
            Note::F,
        ]);
        let bes4 = Pitch::new(Note::Bes, 4);
        let ces5 = g_flat_major.pitch_n_steps_from(&bes4, 1).unwrap();
        assert_eq!((ces5.note, ces5.octave, ces5.midi_note), (Note::Ces, 5, 71));
        let des5 = g_flat_major.pitch_n_steps_from(&bes4, 2).unwrap();
        assert_eq!((des5.note, des5.octave, des5.midi_note), (Note::Des, 5, 73));

        // Starting on C♭5 itself and stepping both ways.
        let up = g_flat_major.pitch_n_steps_from(&ces5, 1).unwrap();
        assert_eq!((up.note, up.octave, up.midi_note), (Note::Des, 5, 73));
        let down = g_flat_major.pitch_n_steps_from(&ces5, -1).unwrap();
        assert_eq!((down.note, down.octave, down.midi_note), (Note::Bes, 4, 70));
        let back = g_flat_major.pitch_n_steps_from(&des5, -1).unwrap();
        assert_eq!(back, ces5);
    }

    #[test]
    fn test_pitch_n_steps_from_spells_b_sharp_in_its_written_octave() {
        use crate::note::pitch::Pitch;

        // C♯ major contains B♯. Stepping up from A♯3 (58) reaches MIDI 60,
        // which is written B♯3 — not B♯4 (MIDI 72).
        let c_sharp_major = NoteSet::new(vec![
            Note::Cis,
            Note::Dis,
            Note::Eis,
            Note::Fis,
            Note::Gis,
            Note::Ais,
            Note::Bis,
        ]);
        let ais3 = Pitch::new(Note::Ais, 3);
        let bis3 = c_sharp_major.pitch_n_steps_from(&ais3, 1).unwrap();
        assert_eq!((bis3.note, bis3.octave, bis3.midi_note), (Note::Bis, 3, 60));
        let cis4 = c_sharp_major.pitch_n_steps_from(&bis3, 1).unwrap();
        assert_eq!((cis4.note, cis4.octave, cis4.midi_note), (Note::Cis, 4, 61));
        let back = c_sharp_major.pitch_n_steps_from(&cis4, -1).unwrap();
        assert_eq!(back, bis3);
    }

    #[test]
    fn test_find_enharmonic() {
        let notes = NoteSet::new(vec![Note::C, Note::Des, Note::E]);

        // Exact match
        assert_eq!(notes.find_enharmonic(&Note::Des), Some(&Note::Des));

        // Enharmonic match (C# == Db)
        assert_eq!(notes.find_enharmonic(&Note::Cis), Some(&Note::Des));

        // No match
        assert_eq!(notes.find_enharmonic(&Note::D), None);
    }
}
