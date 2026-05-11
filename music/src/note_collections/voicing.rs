use std::hash::{Hash, Hasher};
use std::ops::Deref;
use crate::error::MusicSemanticsError;
use crate::notation::clef::Clef;
use crate::note::Note;
use crate::note::pitch_class::Pc;
use crate::note_collections::pc_set::PcContent;
use crate::note_collections::spelling::{HasSpelling, spell_content};
use crate::note::pitch::Pitch;
use crate::note_collections::geometry::symmetry::transpositional::TryTranspose;
use crate::NoteSet;


/// Returns a vector of increasing midi note values, based on a series of
/// vertically stacked intervals and a starting pitch.
fn stack_midi_from_intervals(pitch: &Pitch, intervals: &StackedIntervals) -> Vec<u8> {
    let mut midi_notes = vec![pitch.midi_note];
    intervals.iter()
        .for_each(|i| midi_notes.push(midi_notes.last().unwrap() + i));
    midi_notes
}

/// A collection of [crate::note::Pitch] with no guarantees on its contents, except
/// that it is sorted from low to high on initialization.
/// For example, [crate::note::Note] duplicates (both octaves and unisons) and
/// enharmonic equivalents are allowed.
///
/// A [Voicing] can be played/notated in a definite manner,
/// all it needs is rhythmic information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voicing(Vec<Pitch>);

impl Voicing {
    /// Sorts the pitches, but does not perform any deduplication of unisons/enharmonics.
    pub fn new(mut pitches: Vec<Pitch>) -> Self {
        pitches.sort_by(|a,b| a.partial_cmp(b).unwrap());
        Self(pitches)
    }

    /// Shifts the voicing up or down by some number of octaves.
    /// Spelling remains the same.
    pub fn move_by_octaves(&self, n: isize) -> Result<Self, MusicSemanticsError> {
        Ok(Self(self.iter()
            .map(|p| Ok::<_, MusicSemanticsError>(p.raise_octaves(n)?))
            .into_iter()
            .flatten()
            .collect()))
    }

    /// Applies a set of voiceleading paths to `self`. Produces
    /// a Vector of [Pitch] instead of a [Voicing], because the resultant
    /// pitch ordering carries information about voice crossings.
    ///
    /// We perform a length check, so that `paths.len()` must be equal to `self.len()`.
    pub fn apply_paths(&self, paths: &Vec<i8>, notes: Option<&Vec<Note>>) -> Result<Vec<Pitch>, MusicSemanticsError> {
        if paths.len() != self.0.len() {
            return Err(MusicSemanticsError::MismatchedCollectionSize(
                self.0.len(), paths.len()
            ));
        }
        Ok(self
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut transposed = p.try_transpose(paths[i])?;
                if let Some(notes) = notes {
                    transposed = transposed.spelled_as_in(notes)?;
                }
                Ok::<_, MusicSemanticsError>(transposed)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .collect()
        )
    }

    /// Return the min and max pitches.
    pub fn span(&self) -> Option<(Pitch, Pitch)> {
        if self.is_empty() {
            return None;
        }
        let min = self.0.iter().min_by(|a,b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max = self.0.iter().max_by(|a,b| a.partial_cmp(b).unwrap())
            .unwrap();
        Some((min.clone(), max.clone()))
    }

    /// Given a [Pitch], we can infer the others using a [StackedIntervals] instance.
    pub fn from_intervals(root: &Pitch, intervals: &StackedIntervals) -> Result<Self, MusicSemanticsError> {
        let midi_notes = stack_midi_from_intervals(root, intervals);
        let pc_content = PcContent::new(midi_notes.iter().map(|m| Pc::from(&(m % 12))).collect());
        let spelling = spell_content(&root.note, &pc_content)?;
        let mut pitches = midi_notes.iter()
            .map(|m| Pitch::from_midi_spelled_as(*m, &spelling).unwrap())
            .collect::<Vec<_>>();
        pitches.sort_by(|a,b| a.partial_cmp(b).unwrap());
        Ok(Self(pitches))
    }

    /// Whether any adjacent pair of pitches is an octave or more apart.
    pub fn has_wide_intervals(&self) -> bool {
        let s: StackedIntervals = self.into();
        s.has_wide_intervals()
    }

    /// Tries to return an instance of self moved up/down a number of octaves to optimize
    /// its presentation toward the middle of a given clef.
    /// In very extreme cases, this attempt can fail, but those have to be very contrived
    /// cases, and this function would not be applicable to such scenarios anyway.
    pub fn normalize_register_to_clef(&self, clef: Clef) -> Result<Self, MusicSemanticsError> {
        if self.is_empty() {
            return Ok(self.clone());
        }
        let (clef_bottom, clef_top) = clef.bounds();
        let mut cloned = self.clone();

        let (mut min, mut max) = cloned.span().unwrap();
        let mut bottom_distance = min.diatonic_distance(&clef_bottom);
        let mut top_distance = clef_top.diatonic_distance(&max);
        while (bottom_distance > 0 || top_distance > 0) &&
            top_distance < bottom_distance - 7 {
            cloned = cloned.move_by_octaves(1)?;
            (min, max) = cloned.span().unwrap();
            bottom_distance = min.diatonic_distance(&clef_bottom);
            top_distance = clef_top.diatonic_distance(&max);
        }
        while (bottom_distance > 0 || top_distance > 0) &&
            bottom_distance <= top_distance - 7 {
            cloned = cloned.move_by_octaves(-1)?;
            (min, max) = cloned.span().unwrap();
            bottom_distance = min.diatonic_distance(&clef_bottom);
            top_distance = clef_top.diatonic_distance(&max);
        }
        // Final adjustment: if the voicing is biased toward the lower ledger lines
        // when there's room to move it up, shift up an octave.
        // We check if:
        // 1. The lowest note is below the staff bottom (bottom_distance > 0)
        // 2. There's room to move up (top note is more than an octave below staff top)
        (min, max) = cloned.span().unwrap();
        bottom_distance = min.diatonic_distance(&clef_bottom);
        top_distance = clef_top.diatonic_distance(&max);

        // If the voicing is entirely below the staff and could fit better if raised
        if bottom_distance > 0 && top_distance > 7 {
            cloned = cloned.move_by_octaves(1)?;
        }

        Ok(cloned)
    }
}

impl From<Voicing> for StackedIntervals {
    fn from(voicing: Voicing) -> Self {
        Self::from(&voicing)
    }
}

impl From<&Voicing> for StackedIntervals {
    fn from(voicing: &Voicing) -> Self {
        StackedIntervals(
            voicing.0.iter()
                .zip(&voicing.0[1..])
                .map(|(a,b)| {
                    b.midi_note - a.midi_note
                })
                .collect()
        )
    }
}

impl Hash for Voicing {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_empty() {
            "Voicing:<empty>".hash(state);
        } else {
            let intervals: StackedIntervals = self.into();
            let first = self.0.first().unwrap();
            first.hash(state);
            intervals.hash(state);
        }
    }
}

impl Deref for Voicing {
    type Target = Vec<Pitch>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Voicing {
    type Item = Pitch;
    type IntoIter = std::vec::IntoIter<Pitch>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Voicing {
    type Item = &'a Pitch;
    type IntoIter = std::slice::Iter<'a, Pitch>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<&Voicing> for NoteSet {
    /// Extracts the [Note]s from a [Voicing], discarding octave information.
    /// The resulting [NoteSet] is sorted by pitch class and deduplicated,
    /// oriented from the first pitch's note.
    fn from(voicing: &Voicing) -> Self {
        let notes: Vec<Note> = voicing.iter().map(|p| p.note).collect();
        match voicing.first() {
            Some(p) => NoteSet::with_root(notes, &p.note),
            None => NoteSet::new(notes),
        }
    }
}

/// Consecutive vertical stacking of intervals, taken to be ordered from low to high.
/// These are non-negative, root-agnostic, and spelling-agnostic semitone distances
/// between consecutive, ordered notes of a harmony.
///
/// Each unique value in this space defines a "unique way to play a particular chord type",
/// generalized over any possible choice of root [crate::note::Note].
///
/// If we consider only the space of values `vec[e_1, e_2, ..., e_i]` where all `e_i < 12`,
/// and pick any chord type that is not rotationally (i.e. transpositionally) symmetrical,
/// then there are `factorial(chord.len())` points in this space that uniquely correspond
/// to that chord type. One for each permutation of its notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackedIntervals(pub Vec<u8>);

impl StackedIntervals {
    pub fn new(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl Hash for StackedIntervals {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.iter().for_each(|item| item.hash(state));
    }
}

impl Deref for StackedIntervals {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for StackedIntervals {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a StackedIntervals {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl StackedIntervals {
    pub fn has_wide_intervals(&self) -> bool {
        self.iter().any(|interval| *interval >= 12)
    }
}

#[macro_export]
macro_rules! voicing {
    ($( $p:expr ),+) => {
        Voicing::new([$($p),+].to_vec())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::note::Note;
    use crate::pitch;

    #[test]
    fn voicing_macro() {
        assert_eq!(
            Voicing::new(vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ]),
            voicing!(
                pitch!(c, 4),
                pitch!(e, 4),
                pitch!(g, 4)
            )
        );
    }

    #[test]
    fn normalizing_to_treble() {
        let v0 = Voicing::new(vec![
            Pitch::new(Note::C, 3),
            Pitch::new(Note::Fis, 3),
        ]);
        let v1 = Voicing::new(vec![
            Pitch::new(Note::C, 4),
            Pitch::new(Note::Fis, 4),
        ]);
        assert_eq!(
            v0.normalize_register_to_clef(Clef::Treble).unwrap(),
            v1.normalize_register_to_clef(Clef::Treble).unwrap()
        );
        let v2 = Voicing::new(vec![
            Pitch::new(Note::C, 5),
            Pitch::new(Note::Fis, 5),
        ]);
        assert_eq!(
            v1.normalize_register_to_clef(Clef::Treble).unwrap(),
            v2.normalize_register_to_clef(Clef::Treble).unwrap()
        );
        let v3 = Voicing::new(vec![
            Pitch::new(Note::C, 6),
            Pitch::new(Note::Fis, 6),
        ]);
        assert_eq!(
            v2.normalize_register_to_clef(Clef::Treble).unwrap(),
            v3.normalize_register_to_clef(Clef::Treble).unwrap()
        );
        let v4 = Voicing::new(vec![
            Pitch::new(Note::C, 7),
            Pitch::new(Note::Fis, 7),
        ]);
        assert_eq!(
            v3.normalize_register_to_clef(Clef::Treble).unwrap(),
            v4.normalize_register_to_clef(Clef::Treble).unwrap()
        );
    }

    #[test]
    fn low_voicing_gets_raised_to_staff() {
        // A voicing in the low register below the treble staff
        // Treble staff bounds: E4 (bottom) to F5 (top)
        let low = Voicing::new(vec![
            Pitch::new(Note::C, 2),
            Pitch::new(Note::E, 2),
            Pitch::new(Note::G, 2),
        ]);
        let normalized = low.normalize_register_to_clef(Clef::Treble).unwrap();
        // The lowest note should be raised to be closer to the staff
        let (min, _max) = normalized.span().unwrap();
        // Should be at least C4 (not C2)
        assert!(min.midi_note >= 48, "Expected low note to be C4 or higher, got {:?}", min);
    }

    #[test]
    fn normalize_to_bass_clef() {
        // Bass clef bounds: G2 (bottom) to A3 (top)
        // A high voicing should be moved down
        let high = Voicing::new(vec![
            Pitch::new(Note::C, 5),
            Pitch::new(Note::E, 5),
        ]);
        let normalized = high.normalize_register_to_clef(Clef::Bass).unwrap();
        let (min, _max) = normalized.span().unwrap();
        // Should be moved much lower for bass clef
        assert!(min.midi_note < 60, "Expected note below C4 for bass clef, got {:?}", min);
    }

    #[test]
    fn voicing_from_intervals_unchanged_for_cmaj7() {
        let root = Pitch::from_midi_spelled_as(60 /*C4*/, &vec![Note::C]).unwrap();
        let voicing = Voicing::from_intervals(&root, &StackedIntervals(vec![4, 3, 4])).unwrap();
        assert_eq!(voicing.len(), 4);
        assert_eq!(voicing[0].note, Note::C);
        assert_eq!(voicing[1].note, Note::E);
        assert_eq!(voicing[2].note, Note::G);
        assert_eq!(voicing[3].note, Note::B);
    }
}
