use crate::note::pitch_class::Pc;
use std::collections::HashSet;
use std::ops::Deref;
use crate::error::MusicSemanticsError;
use crate::note::note::Note;
use crate::note_collections::geometry::symmetry::transpositional::{find_transpositional_symmetries, TranspositionalSymmetryMap};
use crate::note_collections::geometry::symmetry::transpositional::Transpose;
use crate::note_collections::NoteSet;
use crate::note_collections::spelling::spell_pc_set;

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
pub struct PcSet(Vec<Pc>);

impl PcSet {
    /// Deduplicated, ordered, and zeroed
    pub fn new(pcs: Vec<Pc>) -> Self {
        let mut pcs = deduplicate_pcs(&pcs);
        pcs.sort();
        Self(zeroed_pcs(&pcs))
    }

    /// Rotate self backwards by one. This is equivalent to walking
    /// to the previous mode of a scale, or inversion of a chord.
    pub fn rotate_back(&self) -> Self {
        if self.0.is_empty() {
            return Self(vec![]);
        }
        let mut copy = self.0.clone();
        copy.rotate_right(1);
        Self(zeroed_pcs(&copy))
    }

    /// Rotate self forward by one. This is equivalent to walking
    /// to the next mode of a scale, or inversion of a chord.
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
        copy.rotate_left(usize::try_from(times).unwrap());
        Self(zeroed_pcs(&copy))
    }

    /// Returns a `HashMap` of all the transpositional symmetries
    /// that self might have.
    /// For more detailss, see
    /// [crate::note_collections::geometry::symmetry::find_transpositional_symmetries].
    pub fn transpositional_symmetry(&self) -> TranspositionalSymmetryMap {
        find_transpositional_symmetries(&self.0)
    }

    /// Whether self can be transposed into other. For example,
    /// `PcSet(vec![Pc0, Pc3])` is a transposed version of `&vec![Pc1, Pc4]`.
    pub fn is_transposed_version_of(&self, other: &Vec<Pc>) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        let pcs_len = self.len();
        if pcs_len != other.len() {
            return false;
        }
        if pcs_len == 1 {
            return true;
        }
        let other = PcSet::new(other.clone());
        (0..pcs_len)
            .any(|i| other.rotate(isize::try_from(i).unwrap()) == *self)
    }

    /// Move up (i.e. rotate clockwise around the "circle of [Pc]s") some
    /// non-zero number of semitones.
    /// This returns a Vec of [crate::note::Pc],
    /// because we aren't normalizing the value to [crate::note::Pc::Pc0],
    /// which affords a bit more flexibility in how one might use this.
    pub fn transpose_nonzeroed(&self, semitones: i8) -> Vec<Pc> {
        self.0
            .iter()
            .map(|pc| pc.transpose(semitones))
            .collect()
    }

    /// Attempt to spell a [PcSet] using this library's provided spelling function,
    /// [crate::note_collections::spelling::spell_pc_set].
    pub fn try_spell(&self, root: &Note) -> Result<Vec<Note>, MusicSemanticsError> {
        spell_pc_set(root, self)
    }
}

impl Deref for PcSet {
    type Target = Vec<Pc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&[Pc]> for PcSet {
    fn from(pcs: &[Pc]) -> Self {
        Self(pcs.to_vec())
    }
}

impl<const N: usize> From<&[Pc; N]> for PcSet {
    fn from(pcs: &[Pc; N]) -> Self {
        Self(pcs.to_vec())
    }
}

impl<const N: usize> From<[Pc; N]> for PcSet {
    fn from(pcs: [Pc; N]) -> Self {
        Self(pcs.to_vec())
    }
}

impl From<Vec<Pc>> for PcSet {
    fn from(pcs: Vec<Pc>) -> Self {
        Self::from(&pcs)
    }
}

impl From<&Vec<Pc>> for PcSet {
    fn from(pcs: &Vec<Pc>) -> Self {
        Self(pcs.clone())
    }
}

impl From<&Vec<u8>> for PcSet {
    fn from(pcs: &Vec<u8>) -> Self {
        Self(pcs.iter().map(|pc| Pc::from(pc)).collect())
    }
}

impl From<Vec<u8>> for PcSet {
    fn from(pcs: Vec<u8>) -> Self {
        Self::from(&pcs)
    }
}

impl Into<HashSet<Pc>> for PcSet {
    fn into(self) -> HashSet<Pc> {
        let mut set = HashSet::new();
        self.0.iter().for_each(|pc| { set.insert(*pc); });
        set
    }
}

impl Into<HashSet<Pc>> for &PcSet {
    fn into(self) -> HashSet<Pc> {
        let mut set = HashSet::new();
        self.0.iter().for_each(|pc| { set.insert(*pc); });
        set
    }
}

impl From<&NoteSet> for PcSet {
    fn from(value: &NoteSet) -> Self {
        PcSet::new(value
            .iter()
            .map(|note| Pc::from(note))
            .collect()
        )
    }
}
impl From<NoteSet> for PcSet {
    fn from(value: NoteSet) -> Self {
        PcSet::from(&value)
    }
}

/* TODO Should I create a PcMultiSet? Methods below...

    pub fn has_duplicates(&self) -> bool {
        let deduped = deduplicate_pcs(&self.0);
        self.0.len() != deduped.len()
    }

    /// The number of unique
    pub fn len_unique(&self) -> usize {
        deduplicate_pcs(&self.0).len()
    }

    pub fn has_only_one_of(&self, pcs: &[Pc]) -> bool {
        let len = pcs.len();
        let pcs: Vec<&Pc> = pcs.iter().filter(|pc| !self.0.contains(pc)).collect();
        pcs.len() == len - 1
    }

 */

#[macro_export]
macro_rules! pcs {
    ($( $pc:expr ),+) => {
        PcSet::from([$($pc),+].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::pitch_class::Pc::*;

    #[test]
    fn pcs_macro() {
        assert_eq!(
            PcSet(vec![Pc0, Pc1, Pc5]),
            pcs!(0, 1, 5),
        )
    }

    #[test]
    fn transposed_comparison() {
        let pc_set = PcSet::new(vec![Pc0, Pc4, Pc7]);
        let pc_set2 = vec![Pc2, Pc6, Pc9];
        assert!(pc_set.is_transposed_version_of(&pc_set2));
        let pc_set = PcSet::new(vec![Pc0, Pc4, Pc7]);
        let pc_set2 = vec![Pc0, Pc3, Pc9];
        assert!(!pc_set.is_transposed_version_of(&pc_set2));
    }
}