use std::hash::{Hash, Hasher};
use std::ops::Deref;
use crate::chord::pc_set::PcSet;
use crate::chord::spelling::spell_pc_set;
use crate::pitch::Pitch;

/// Returns a vector of increasing midi note values, based on a series of
/// vertically stacked intervals and a starting pitch.
fn stack_midi_from_intervals(pitch: &Pitch, intervals: &StackedIntervals) -> Vec<u8> {
    let mut midi_notes = vec![pitch.midi_note];
    intervals.iter()
        .for_each(|i| midi_notes.push(midi_notes.last().unwrap() + i));
    midi_notes
}

/// A collection of [Pitch] with no constraints on its contents.
/// For example, [Note] duplicates or enharmonic equivalents are allowed, as are sonic unisons.
/// A [Voicing] can be played/notated in a definite manner,
/// all it needs is rhythmic information.
#[derive(Debug, Clone)]
pub struct Voicing(pub Vec<Pitch>);

impl Voicing {
    pub fn from_intervals(root: &Pitch, intervals: &StackedIntervals) -> anyhow::Result<Self> {
        let midi_notes = stack_midi_from_intervals(root, intervals);
        let pc_set = PcSet::from_midi_notes(&midi_notes);
        let spelling = spell_pc_set(&root.note, &pc_set)?;
        let pitches = midi_notes.iter()
            .map(|m| Pitch::spelled_as_in(*m, &spelling).unwrap())
            .collect();
        Ok(Self(pitches))
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