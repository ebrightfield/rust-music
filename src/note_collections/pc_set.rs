use crate::note::pc::Pc;
use std::collections::{HashMap, HashSet};
use crate::note_collections::geometry::{find_transpositional_symmetries, TranspositionalSymmetry};

pub fn deduplicate_pcs(pcs: &[Pc]) -> Vec<Pc> {
    let mut pc_set = HashSet::new();
    pcs.iter().for_each(|pc| {
        pc_set.insert(pc.clone());
    });
    Vec::from_iter(pc_set)
}

// Assumes ordered elements
pub fn zeroed_pcs(pcs: &[Pc]) -> Vec<Pc> {
    // Screen empty collections because we assume a first element.
    if pcs.is_empty() {
        return vec![];
    }
    // We need to tolerate negative numbers in our subtraction, so using i32
    let magnitude = i32::from(&pcs[0]);
    pcs.iter()
        .map(|pc| {
            let pc: i32 = pc.into();
            pc - magnitude
        })
        .map(|i| Pc::from(i))
        .collect()
}

/// Represents a set of pitch-classes.
#[derive(Debug, Clone, PartialEq)]
pub struct PcSet(pub Vec<Pc>);

impl PcSet {
    pub fn has_duplicates(&self) -> bool {
        let deduped = deduplicate_pcs(&self.0);
        self.0.len() != deduped.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len_unique(&self) -> usize {
        deduplicate_pcs(&self.0).len()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn has_only_one_of(&self, pcs: &[Pc]) -> bool {
        let len = pcs.len();
        let pcs: Vec<&Pc> = pcs.iter().filter(|pc| !self.0.contains(pc)).collect();
        pcs.len() == len - 1
    }

    pub fn from_midi_notes(midi_notes: &Vec<u8>) -> Self {
        let pcs = midi_notes.iter()
            .map(|i| Pc::from(i))
            .collect();
        Self::new(pcs)
    }

    /// Deduplicated, ordered, and zeroed
    pub fn new(pcs: Vec<Pc>) -> Self {
        let mut pcs = deduplicate_pcs(&pcs);
        pcs.sort();
        Self(zeroed_pcs(&pcs))
    }

    pub fn rotate_back(&self) -> Self {
        if self.0.is_empty() {
            return Self(vec![]);
        }
        let mut copy = self.0.clone();
        copy.rotate_right(1);
        Self(zeroed_pcs(&copy))
    }

    pub fn rotate_fwd(&self) -> Self {
        if self.0.is_empty() {
            return Self(vec![]);
        }
        let mut copy = self.0.clone();
        copy.rotate_left(1);
        Self(zeroed_pcs(&copy))
    }

    /// Rotation of a PC-set entails re-orienting it
    /// so that some non-zero [Pc] is treated as [Pc::Pc0].
    pub fn rotate(&self, times: isize) -> Self {
        if self.0.is_empty() {
            return Self(vec![]);
        }
        let mut copy = self.0.clone();
        let times = times.rem_euclid(isize::try_from(self.0.len()).unwrap());
        copy.rotate_right(usize::try_from(times).unwrap());
        Self(zeroed_pcs(&copy))
    }

    pub fn transpositional_symmetry(&self) -> HashMap<Pc, HashSet<TranspositionalSymmetry>> {
        find_transpositional_symmetries(&self.0)
    }
}

impl From<&[Pc]> for PcSet {
    fn from(pcs: &[Pc]) -> Self {
        PcSet(pcs.to_vec())
    }
}

impl<const N: usize> From<&[Pc; N]> for PcSet {
    fn from(pcs: &[Pc; N]) -> Self {
        PcSet(pcs.to_vec())
    }
}

impl<const N: usize> From<[Pc; N]> for PcSet {
    fn from(pcs: [Pc; N]) -> Self {
        PcSet(pcs.to_vec())
    }
}

impl Into<HashSet<Pc>> for PcSet {
    fn into(self) -> HashSet<Pc> {
        let mut set = HashSet::new();
        self.0.iter().for_each(|pc| { set.insert(*pc); });
        set
    }
}