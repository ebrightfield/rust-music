use std::ops::Deref;
use crate::note_collections::{OctavePartition, PcSet};

pub trait IntervallicSymmetry: Sized {
    /// Returns the same type as self, but with its interval content
    /// inverted (i.e. reflected).
    ///
    /// Returns `None` when the reflection produces the same type.
    fn invert_intervals(&self) -> Option<Self>;

    fn is_inversionally_symmetric(&self) -> bool {
        self.invert_intervals().is_none()
    }
}

impl IntervallicSymmetry for OctavePartition {
    fn invert_intervals(&self) -> Option<Self> {
        let inverted = OctavePartition::new(
            self.deref().iter().rev().map(|i| *i).collect()).unwrap();
        if *self == inverted {
            return None;
        }
        Some(inverted)
    }
}

impl IntervallicSymmetry for PcSet {
    fn invert_intervals(&self) -> Option<Self> {
        let partition = OctavePartition::from(self);
        let inverted_partition = partition.invert_intervals();
        inverted_partition.map(
            |partition| PcSet::from(&partition)
        )
    }
}
