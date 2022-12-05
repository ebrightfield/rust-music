use std::hash::{Hash, Hasher};
use std::ops::Deref;
use crate::notation::clef::Clef;
use crate::note_collections::pc_set::PcSet;
use crate::note_collections::spelling::spell_pc_set;
use crate::note::pitch::Pitch;

/// Returns a vector of increasing midi note values, based on a series of
/// vertically stacked intervals and a starting pitch.
fn stack_midi_from_intervals(pitch: &Pitch, intervals: &StackedIntervals) -> Vec<u8> {
    let mut midi_notes = vec![pitch.midi_note];
    intervals.iter()
        .for_each(|i| midi_notes.push(midi_notes.last().unwrap() + i));
    midi_notes
}

/// A collection of [Pitch] with no guarantees on its contents, except
/// that it is sorted from low to high on initialization.
/// For example, [Note] duplicates (both octaves and unisons) and
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
    pub fn move_by_octaves(&self, n: isize) -> anyhow::Result<Self> {
        Ok(Self(self.iter()
            .map(|p| p.raise_octaves(n))
            .into_iter()
            .flatten()
            .collect()))
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
    pub fn from_intervals(root: &Pitch, intervals: &StackedIntervals) -> anyhow::Result<Self> {
        let midi_notes = stack_midi_from_intervals(root, intervals);
        let pc_set = PcSet::from(&midi_notes);
        let spelling = spell_pc_set(&root.note, &pc_set)?;
        let mut pitches = midi_notes.iter()
            .map(|m| Pitch::spelled_as_in(*m, &spelling).unwrap())
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
    pub fn normalize_register_to_clef(&self, clef: Clef) -> anyhow::Result<Self> {
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
        Ok(cloned)
        // TODO Maybe add one more conditional, and potentially
        //    raise the entire thing up an octave.
    }
}

impl Into<StackedIntervals> for Voicing {
    fn into(self) -> StackedIntervals {
        (&self).into()
    }
}

impl Into<StackedIntervals> for &Voicing {
    fn into(self) -> StackedIntervals {
        StackedIntervals(
            self.0.iter()
                .zip(&self.0[1..])
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
            "<empty>".hash(state);
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

/// Consecutive vertical stacking of intervals, taken to be ordered from low to high.
/// These are non-negative, root-agnostic, and spelling-agnostic semitone distances
/// between consecutive, ordered notes of a harmony.
///
/// Each unique value in this space defines a "unique way to play a particular chord type",
/// generalized over any possible choice of root [Note].
///
/// If we consider only the space of values `vec[e_1, e_2, ..., e_i]` where all `e_i < 12`,
/// and pick any chord type that is not rotationally (i.e. transpositionally) symmetrical,
/// then there are `factorial(chord.len())` points in this space that uniquely correspond
/// to that chord type. One for each permutation of its notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackedIntervals(pub Vec<u8>);

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

impl StackedIntervals {
    pub fn has_wide_intervals(&self) -> bool {
        self.iter().any(|interval| *interval >= 12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::note::Note;

    #[test]
    fn normalizing_to_treble() {
        let v0 = Voicing::new(vec![
            Pitch::new(Note::C, 3).unwrap(),
            Pitch::new(Note::Fis, 3).unwrap(),
        ]);
        let v1 = Voicing::new(vec![
            Pitch::new(Note::C, 4).unwrap(),
            Pitch::new(Note::Fis, 4).unwrap(),
        ]);
        assert_eq!(
            v0.normalize_register_to_clef(Clef::Treble).unwrap(),
            v1.normalize_register_to_clef(Clef::Treble).unwrap()
        );
        let v2 = Voicing::new(vec![
            Pitch::new(Note::C, 5).unwrap(),
            Pitch::new(Note::Fis, 5).unwrap(),
        ]);
        assert_eq!(
            v1.normalize_register_to_clef(Clef::Treble).unwrap(),
            v2.normalize_register_to_clef(Clef::Treble).unwrap()
        );
        let v3 = Voicing::new(vec![
            Pitch::new(Note::C, 6).unwrap(),
            Pitch::new(Note::Fis, 6).unwrap(),
        ]);
        assert_eq!(
            v2.normalize_register_to_clef(Clef::Treble).unwrap(),
            v3.normalize_register_to_clef(Clef::Treble).unwrap()
        );
        let v4 = Voicing::new(vec![
            Pitch::new(Note::C, 7).unwrap(),
            Pitch::new(Note::Fis, 7).unwrap(),
        ]);
        assert_eq!(
            v3.normalize_register_to_clef(Clef::Treble).unwrap(),
            v4.normalize_register_to_clef(Clef::Treble).unwrap()
        );
    }
}