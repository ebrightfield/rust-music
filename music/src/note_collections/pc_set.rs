use crate::note::pitch_class::Pc;
use std::collections::HashSet;
use std::ops::Deref;
use crate::error::MusicSemanticsError;
use crate::note::note::Note;
use crate::note_collections::NoteSet;

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
        .map(|i| Pc::from(&i))
        .collect()
}

/// An intervallic shape: a transposition-invariant set of pitch classes
/// anchored at `Pc0`.
///
/// # Invariants
/// - Inner `Vec<Pc>` is sorted ascending.
/// - Contains no duplicates.
/// - When non-empty, the first element is `Pc::Pc0`.
///
/// # Semantic role
/// `PcShape` represents the *shape* of a chord or scale — the pattern of
/// intervals above a hypothetical root at `Pc0`. Use this type when:
/// - Enumerating modes or rotations of a scale.
/// - Matching a chord or scale against a quality template.
/// - Testing transpositional symmetry or subchord containment.
///
/// To obtain the absolute sounding PCs at a specific root, call
/// [`PcShape::at_root`].
///
/// # Example
/// ```
/// use music::note_collections::pc_set::PcShape;
/// use music::note::pitch_class::Pc;
///
/// // Major triad shape — same regardless of root
/// let major = PcShape::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
/// // Transpose to G (Pc7) to get the sounding content
/// let g_major = major.at_root(Pc::Pc7);
/// ```
// REQ-O1, REQ-O3, REQ-O39
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PcShape(Vec<Pc>);

/// Absolute pitch-class content at a fixed transposition.
///
/// # Invariants
/// - Inner `Vec<Pc>` is sorted ascending.
/// - Contains no duplicates.
/// - (NOT zero-anchored — the first element may be any `Pc`.)
///
/// # Semantic role
/// `PcContent` represents the *actual sounding* pitch classes of a chord or
/// voicing — the PCs you would hear. Use this type when:
/// - Constructing a voicing from MIDI notes.
/// - Reasoning about concrete pitch content, not abstract patterns.
/// - Spelling absolute sounding notes.
///
/// To derive the intervallic shape (zeroed, root-relative), call
/// [`PcContent::to_shape`].
///
/// # Example
/// ```
/// use music::note_collections::pc_set::{PcShape, PcContent};
/// use music::note::pitch_class::Pc;
///
/// // F#m7b5 sounding content at its root
/// let content = PcContent::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc6, Pc::Pc9]);
/// // Extract the interval template
/// let shape = content.to_shape();
/// assert_eq!(shape, PcShape::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc6, Pc::Pc9]));
/// ```
// REQ-O1, REQ-O4, REQ-O39
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PcContent(Vec<Pc>);

/// Shared read-only access to the underlying pc slice.
// REQ-O34
pub trait AsPcSlice {
    fn as_pc_slice(&self) -> &[Pc];
}

impl PcShape {
    /// Deduplicate, sort, then zero-anchor.
    // REQ-O5
    pub fn new(pcs: Vec<Pc>) -> Self {
        let mut pcs = deduplicate_pcs(&pcs);
        pcs.sort();
        Self(zeroed_pcs(&pcs))
    }

    /// Transpose this shape so that its first element maps to `root`,
    /// producing the absolute sounding [`PcContent`] for that root.
    ///
    /// For an empty shape, returns an empty [`PcContent`].
    ///
    /// # Roundtrip note
    /// `shape.at_root(Pc0).to_shape() == shape` holds when `root == Pc0`.
    /// For any other root the `to_shape()` call re-zeros from the lowest PC,
    /// which may differ from the original shape.
    ///
    /// # Example
    /// ```
    /// use music::note_collections::pc_set::PcShape;
    /// use music::note::pitch_class::Pc;
    ///
    /// let shape = PcShape::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
    /// // at Pc0 the content equals the shape (both start at 0)
    /// let content = shape.at_root(Pc::Pc0);
    /// assert_eq!(content.to_shape(), shape);
    /// ```
    // REQ-O9, REQ-O39
    pub fn at_root(&self, root: Pc) -> PcContent {
        if self.0.is_empty() { return PcContent(vec![]); }
        let shift = i32::from(&root);
        let shifted: Vec<Pc> = self.0.iter()
            .map(|pc| { let v: i32 = pc.into(); v + shift })
            .map(|i| Pc::from(&i))
            .collect();
        PcContent::new(shifted)
    }

    /// Rotate self backwards by one. This is equivalent to walking
    /// to the previous mode of a scale, or inversion of a chord.
    // REQ-O13
    pub fn rotate_back(&self) -> Self {
        if self.0.is_empty() { return Self(vec![]); }
        let mut copy = self.0.clone();
        copy.rotate_right(1);
        Self(zeroed_pcs(&copy))
    }

    /// Rotate self forward by one. This is equivalent to walking
    /// to the next mode of a scale, or inversion of a chord.
    // REQ-O13
    pub fn rotate_fwd(&self) -> Self {
        if self.0.is_empty() { return Self(vec![]); }
        let mut copy = self.0.clone();
        copy.rotate_left(1);
        Self(zeroed_pcs(&copy))
    }

    /// Rotation of a PC-shape entails re-orienting it
    /// so that some non-zero [Pc] is treated as [Pc::Pc0].
    // REQ-O13
    pub fn rotate(&self, times: isize) -> Self {
        if self.0.is_empty() { return Self(vec![]); }
        let mut copy = self.0.clone();
        let n = isize::try_from(self.0.len()).unwrap();
        let times = times.rem_euclid(n);
        copy.rotate_left(usize::try_from(times).unwrap());
        Self(zeroed_pcs(&copy))
    }

    /// Returns a `HashMap` of all the transpositional symmetries
    /// that self might have.
    // REQ-O13
    pub fn transpositional_symmetry(&self) -> crate::note_collections::geometry::symmetry::transpositional::TranspositionalSymmetryMap {
        crate::note_collections::geometry::symmetry::transpositional::find_transpositional_symmetries(&self.0)
    }

    /// Whether self can be transposed into other.
    // REQ-O13
    pub fn is_transposed_version_of(&self, other: &Vec<Pc>) -> bool {
        if self.is_empty() || other.is_empty() { return false; }
        let len = self.len();
        if len != other.len() { return false; }
        if len == 1 { return true; }
        let other = PcShape::new(other.clone());
        (0..len).any(|i| other.rotate(isize::try_from(i).unwrap()) == *self)
    }

    /// Thin wrapper delegating to `spell_shape`.
    // REQ-O20
    pub fn try_spell(&self, root: &Note) -> Result<Vec<Note>, MusicSemanticsError> {
        crate::note_collections::spelling::spell_shape(root, self)
    }
}

impl PcContent {
    /// Deduplicate and sort; do NOT zero-anchor.
    // REQ-O5
    pub fn new(pcs: Vec<Pc>) -> Self {
        let mut pcs = deduplicate_pcs(&pcs);
        pcs.sort();
        Self(pcs)
    }

    /// Zero the content to produce a [`PcShape`].
    ///
    /// Subtracts the lowest pitch class from all elements, making the result
    /// root-relative. The inverse of [`PcShape::at_root`] when `root == Pc0`.
    // REQ-O8
    pub fn to_shape(&self) -> PcShape {
        PcShape::new(self.0.clone())
    }

    /// Thin wrapper delegating to `spell_content`.
    // REQ-O20
    pub fn try_spell(&self, root: &Note) -> Result<Vec<Note>, MusicSemanticsError> {
        crate::note_collections::spelling::spell_content(root, self)
    }
}

impl AsPcSlice for PcShape {
    fn as_pc_slice(&self) -> &[Pc] { &self.0 }
}
impl AsPcSlice for PcContent {
    fn as_pc_slice(&self) -> &[Pc] { &self.0 }
}

// REQ-O36
impl Deref for PcShape {
    type Target = Vec<Pc>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl Deref for PcContent {
    type Target = Vec<Pc>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

// REQ-O6
impl FromIterator<Pc> for PcShape {
    fn from_iter<I: IntoIterator<Item = Pc>>(iter: I) -> Self {
        PcShape::new(iter.into_iter().collect())
    }
}
impl FromIterator<Pc> for PcContent {
    fn from_iter<I: IntoIterator<Item = Pc>>(iter: I) -> Self {
        PcContent::new(iter.into_iter().collect())
    }
}

impl IntoIterator for PcShape {
    type Item = Pc;
    type IntoIter = std::vec::IntoIter<Pc>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a> IntoIterator for &'a PcShape {
    type Item = &'a Pc;
    type IntoIter = std::slice::Iter<'a, Pc>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}
impl IntoIterator for PcContent {
    type Item = Pc;
    type IntoIter = std::vec::IntoIter<Pc>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
impl<'a> IntoIterator for &'a PcContent {
    type Item = &'a Pc;
    type IntoIter = std::slice::Iter<'a, Pc>;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

// REQ-O10: PcContent from NoteSet (un-zeroed). NO From<&NoteSet> for PcShape per O10.
impl From<&NoteSet> for PcContent {
    fn from(value: &NoteSet) -> Self {
        PcContent::new(value.iter().map(|n| Pc::from(n)).collect())
    }
}
impl From<NoteSet> for PcContent {
    fn from(value: NoteSet) -> Self { PcContent::from(&value) }
}

// REQ-O11: PcShape from PcContent (via to_shape / zeroing)
impl From<&PcContent> for PcShape {
    fn from(value: &PcContent) -> Self { value.to_shape() }
}
impl From<PcContent> for PcShape {
    fn from(value: PcContent) -> Self { value.to_shape() }
}

// REQ-O12: explicitly NOT implemented:
// impl From<PcShape> for PcContent { ... }  // absent by design — shape→content requires a root

impl Into<HashSet<Pc>> for PcContent {
    fn into(self) -> HashSet<Pc> { self.0.into_iter().collect() }
}
impl Into<HashSet<Pc>> for &PcContent {
    fn into(self) -> HashSet<Pc> { self.0.iter().copied().collect() }
}
impl Into<HashSet<Pc>> for PcShape {
    fn into(self) -> HashSet<Pc> { self.0.into_iter().collect() }
}
impl Into<HashSet<Pc>> for &PcShape {
    fn into(self) -> HashSet<Pc> { self.0.iter().copied().collect() }
}

/// Macro for shape literals: normalizes via `PcShape::new`.
/// Compile-time validates each argument is in `0..12`.
// REQ-O30, REQ-O32, REQ-O33
#[macro_export]
macro_rules! pc_shape {
    ($( $pc:expr ),+ $(,)?) => {{
        const _: () = { $( assert!($pc < 12u8, "Pitch class must be 0-11"); )+ };
        $crate::note_collections::pc_set::PcShape::new(
            vec![$( $crate::note::pitch_class::Pc::from(&($pc as u8)) ),+]
        )
    }};
}

/// Macro for content literals: normalizes via `PcContent::new`.
/// Compile-time validates each argument is in `0..12`.
// REQ-O30, REQ-O32, REQ-O33
#[macro_export]
macro_rules! content {
    ($( $pc:expr ),+ $(,)?) => {{
        const _: () = { $( assert!($pc < 12u8, "Pitch class must be 0-11"); )+ };
        $crate::note_collections::pc_set::PcContent::new(
            vec![$( $crate::note::pitch_class::Pc::from(&($pc as u8)) ),+]
        )
    }};
}

// NOTE (2026-04-20): The old shape/content type and all its From impls,
// old macros, and spell_pc_set free fn have been deleted
// from the workspace atomically in Phase 1.
// See docs/implementation-plan-pcset-type-split.md §2.7 for per-impl disposition.

#[cfg(test)]
mod shape_invariant_tests {
    use super::*;
    use crate::note::pitch_class::Pc::*;

    #[test]
    fn pcshape_new_empty_is_empty() {
        let s = PcShape::new(vec![]);
        assert_eq!(s.as_pc_slice(), &[] as &[Pc]);
    }

    #[test]
    fn pcshape_new_zeroes() {
        let s = PcShape::new(vec![Pc7, Pc11, Pc2]);
        assert_eq!(s.as_pc_slice(), &[Pc0, Pc5, Pc9]);
    }

    #[test]
    fn pcshape_new_dedupes() {
        let s = PcShape::new(vec![Pc0, Pc0, Pc4, Pc7, Pc7]);
        assert_eq!(s.as_pc_slice(), &[Pc0, Pc4, Pc7]);
    }

    #[test]
    fn pcshape_new_singleton_zeroes() {
        let s = PcShape::new(vec![Pc7]);
        assert_eq!(s.as_pc_slice(), &[Pc0]);
    }

    #[test]
    fn pcshape_first_is_always_pc0_when_nonempty() {
        for input in [
            vec![Pc1],
            vec![Pc0, Pc4, Pc7],
            vec![Pc7, Pc11, Pc2],
            vec![Pc11],
        ] {
            let s = PcShape::new(input.clone());
            if !s.is_empty() {
                assert_eq!(s[0], Pc0, "shape from {:?} not anchored at Pc0", input);
            }
        }
    }

    #[test]
    fn rotate_preserves_invariant() {
        let s = PcShape::new(vec![Pc0, Pc4, Pc7]);
        let r = s.rotate(1);
        assert!(r.is_empty() || r[0] == Pc0);
    }

    #[test]
    fn is_transposed_version_of_matches_old_semantics() {
        let s = PcShape::new(vec![Pc0, Pc4, Pc7]);
        assert!(s.is_transposed_version_of(&vec![Pc2, Pc6, Pc9]));
        assert!(!s.is_transposed_version_of(&vec![Pc0, Pc3, Pc9]));
    }

    #[test]
    fn into_iterator_by_ref_yields_pc_refs() {
        let shape = pc_shape!(0, 4, 7);
        let collected: Vec<Pc> = (&shape).into_iter().copied().collect();
        assert_eq!(collected, vec![Pc0, Pc4, Pc7]);
        // The original is still usable afterwards.
        assert_eq!(shape.len(), 3);
    }

    #[test]
    fn into_iterator_by_value_consumes() {
        let shape = pc_shape!(0, 4, 7);
        let collected: Vec<Pc> = shape.into_iter().collect();
        assert_eq!(collected, vec![Pc0, Pc4, Pc7]);
    }

    #[test]
    fn from_iterator_collects_into_pc_shape() {
        // Collecting via FromIterator goes through PcShape::new, so the result
        // is sorted, deduplicated, and zeroed.
        let shape: PcShape = [Pc7, Pc11, Pc2].into_iter().collect();
        assert_eq!(shape, pc_shape!(0, 5, 9));
    }
}

#[cfg(test)]
mod content_invariant_tests {
    use super::*;
    use crate::note::pitch_class::Pc::*;

    #[test]
    fn pccontent_new_does_not_zero() {
        let c = PcContent::new(vec![Pc7, Pc11, Pc2]);
        assert_eq!(c.as_pc_slice(), &[Pc2, Pc7, Pc11]);
    }

    #[test]
    fn pccontent_new_sorts_and_dedupes() {
        let c = PcContent::new(vec![Pc7, Pc2, Pc2, Pc11]);
        assert_eq!(c.as_pc_slice(), &[Pc2, Pc7, Pc11]);
    }

    #[test]
    fn at_root_then_to_shape_roundtrips() {
        let s = PcShape::new(vec![Pc0, Pc4, Pc7]);
        let c = s.at_root(Pc::Pc7);
        // PcContent is sorted ascending: [7+0=7, 7+4=11, 7+7=14%12=2] sorted = [2, 7, 11]
        assert_eq!(c.as_pc_slice(), &[Pc2, Pc7, Pc11]);
        // to_shape() zeros from the lowest PC (Pc2), giving [0, 5, 9] not the original [0, 4, 7].
        // The shape is preserved only when at_root uses Pc0.
        let c0 = s.at_root(Pc::Pc0);
        assert_eq!(c0.to_shape(), s);
    }
}
