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

/// A collection of [Pitch]. [Note] duplicates are allowed.
/// When we know a voicing, we know not only how a chord is to be spelled, but exactly
/// in which octave(s) to express each pitch.
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
/// Roughly speaking, this represents how many semitones are between each consecutive
/// note of a harmony, from low to high. These are root and spelling agnostic,
/// non-negative distances between consecutive, ordered notes of a harmony.
/// They serve as a means of classifying "unique ways to play a chord type", agnostic
/// to root or spelling.
#[derive(Debug, Clone)]
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