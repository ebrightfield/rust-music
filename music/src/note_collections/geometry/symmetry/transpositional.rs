use std::hash::{Hash, Hasher};
use std::collections::{HashMap, HashSet};
use crate::error::MusicSemanticsError;
use crate::note::{Note, Pitch};
use crate::note::pitch_class::Pc;
use crate::note_collections::{NoteSet, PcSet, Voicing};

/// A type that can be transposed. Any type that has modes is necessarily
/// transposable, but not every transposable type has modes, because only
/// types that represent colle
pub trait Transpose: PartialEq + Eq + Hash + Sized {
    /// Returns a version of `self` rotated some number of semitones
    /// up or down.
    fn transpose(&self, semitones: i8) -> Self;
}

/// For items with octave bounds, we have strict upper and lower pitch bounds
/// and therefore transposition is fallible.
pub trait TryTranspose: PartialEq + Eq + Hash + Sized {
    /// Returns a version of `self` rotated some number of semitones
    /// up or down.
    fn try_transpose(&self, semitones: i8) -> Result<Self, MusicSemanticsError>;
}

impl Transpose for Pc {
    fn transpose(&self, semitones: i8) -> Self {
        Self::from(i32::from(self) + semitones as i32)
    }
}

/// A [PcSet] transposed is still just itself or a mode of itself, because it's zeroed.
/// We therefore put the [Transpose] trait on `Vec<Pc>` instead.
impl Transpose for Vec<Pc> {
    fn transpose(&self, semitones: i8) -> Self {
        self
            .iter()
            .map(|pc| pc.transpose(semitones))
            .collect()
    }
}

impl Transpose for Note {
    fn transpose(&self, semitones: i8) -> Self {
        *Pc::from(self).transpose(semitones).notes().first().unwrap()
    }
}

impl TryTranspose for Pitch {
    /// Does not control for spelling.
    fn try_transpose(&self, semitones: i8) -> Result<Self, MusicSemanticsError> {
        let new_pitch = self.midi_note as isize + semitones as isize;
        let new_pitch = u8::try_from(new_pitch)
            .map_err(|_| MusicSemanticsError::OutOfBoundsLower(self.midi_note))?;
        Self::from_midi(new_pitch)
    }
}

impl TryTranspose for Voicing {
    fn try_transpose(&self, semitones: i8) -> Result<Self, MusicSemanticsError> {
        Ok(Voicing::new(self
            .iter()
            .map(|p| Ok::<_, MusicSemanticsError>(p.try_transpose(semitones)?))
            .into_iter()
            .flatten()
            .collect()
        ))
    }
}

impl Transpose for NoteSet {
    fn transpose(&self, semitones: i8) -> Self {
        Self::starting_from_first_note(self
            .iter()
            .map(|note| note.transpose(semitones))
            .collect())
    }
}

/// This type describes the general interface for types that can
/// be described as having modes or inversions.
pub trait Modes: PartialEq + Sized {
    /// Returns all rotations of self.
    fn modes(&self) -> Vec<Self>;

    /// Whether `other` is a mode of self, and if so, which mode it is.
    fn is_mode(&self, other: &Self) -> Option<usize> {
        self.modes().iter().position(|item| item == other)
    }
}

impl Modes for PcSet {
    fn modes(&self) -> Vec<Self> {
        (0..self.len())
            .map(|i| {
                self.rotate(isize::try_from(i).unwrap())
            })
           .collect()
    }
}


/// Type alias for the [HashMap] that stores the results of a search
/// for transpositional symmetries.
pub type TranspositionalSymmetryMap = HashMap<Pc, HashSet<TranspositionalSymmetry>>;

/// Returns a [HashMap] that contains a listing of all transpositional symmetries
/// that occur in the intervallic content of a given collection of [Pc].
///
/// Some chords (e.g. Augmented triads, Dom7b5 chords) are transpositionally
/// symmetrical at intervals smaller than the octave. This means
/// that rather than needing to "rotate" the chord twelve semitones before you arrive
/// at the same set of (enharmonically equivalent) notes, you can achieve the
/// same effect after only e.g. 6 semitones.
///
/// The simplest example of a collection with this behavior is the tritone.
/// If you raise a tritone by six semitones, you end up with the same notes.
pub fn find_transpositional_symmetries(pcs: &Vec<Pc>) -> TranspositionalSymmetryMap {
    let mut symmetries = HashMap::new();
    // Closure for the complicated process of adding entries to our symmetries HashMap.
    let add_entries = |
        hash_map: HashMap<Pc, HashSet<TranspositionalSymmetry>>,
        symmetries: &mut HashMap<Pc, HashSet<TranspositionalSymmetry>>,
    | {
        hash_map
            .iter()
            .for_each(|(pc, sub_list)|{
                // Get the main HashMap at Pc(n).
                let entry: &mut HashSet<TranspositionalSymmetry> = symmetries
                    .entry(pc.clone())
                    .or_insert_with(|| HashSet::new());
                for symmetry in sub_list {
                    if !entry.contains(symmetry) {
                        entry.insert(symmetry.clone());
                    }
                }
            });
    };
    // The size of the chord determines the possible symmetries associated with it.
    // For example, it is impossible for a two-note chord to have a [TranspositionalSymmetry::T3].
    match pcs.len() {
        2 | 10 => {
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T6), &mut symmetries);
        },
        3 | 9 => {
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T6), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T4), &mut symmetries);
        },
        4 | 8 => {
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T6), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T4), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T3), &mut symmetries);
        },
        6 => {
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T6), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T4), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T3), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T2), &mut symmetries);
        },
        _ => {}
    }
    symmetries
}

/// Move up (i.e. rotate clockwise around the "circle of [Pc]s") some
/// non-zero number of semitones.
pub fn transpose(pcs: &Vec<Pc>, semitones: u8) -> Vec<Pc> {
    pcs
        .iter()
        .map(|pc| Pc::from(u8::from(pc) + 12 - semitones.rem_euclid(12)))
        .collect()
}

/// A pitch set possesses a [TranspositionalSymmetry] of `N` when
/// it can be transposed up or down by `N` semitones to arrive at an equivalent
/// [crate::note_collections::PcSet].
/// Most chords and scales do not exhibit any kind of transpositional symmetry,
/// and it is highly associated with musical dissonance.
/// Transpositionally symmetrical chord or scales can easily produce tonal ambiguity.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TranspositionalSymmetry {
    /// Only whole-tone scales have this symmetry.
    T2,
    /// Four-note chords can have this symmetry.
    T3,
    /// Chords/scales of three, four, or six notes can have this symmetry.
    T4,
    /// Many chords and scales can have this transpositional symmetry.
    T6,
}

impl Hash for TranspositionalSymmetry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Into::<u8>::into(self).hash(state)
    }
}

impl Into<u8> for TranspositionalSymmetry {
    fn into(self) -> u8 {
        match self {
            TranspositionalSymmetry::T2 => 2,
            TranspositionalSymmetry::T3 => 3,
            TranspositionalSymmetry::T4 => 4,
            TranspositionalSymmetry::T6 => 6,
        }
    }
}

impl Into<u8> for &TranspositionalSymmetry {
    fn into(self) -> u8 {
        match self {
            TranspositionalSymmetry::T2 => 2,
            TranspositionalSymmetry::T3 => 3,
            TranspositionalSymmetry::T4 => 4,
            TranspositionalSymmetry::T6 => 6,
        }
    }
}

impl Into<i8> for TranspositionalSymmetry {
    fn into(self) -> i8 {
        match self {
            TranspositionalSymmetry::T2 => 2,
            TranspositionalSymmetry::T3 => 3,
            TranspositionalSymmetry::T4 => 4,
            TranspositionalSymmetry::T6 => 6,
        }
    }
}

impl Into<i8> for &TranspositionalSymmetry {
    fn into(self) -> i8 {
        match self {
            TranspositionalSymmetry::T2 => 2,
            TranspositionalSymmetry::T3 => 3,
            TranspositionalSymmetry::T4 => 4,
            TranspositionalSymmetry::T6 => 6,
        }
    }
}

/// Checks for only a specific [TranspositionalSymmetry], rather than all of them.
/// The input of this function assumes a well-ordered, deduped [Vec],
/// but does not have to be normalized to [Pc::Pc0]
/// (i.e. does not need to contain [Pc::Pc0].
pub fn check_for_symmetry(pcs: &Vec<Pc>, symmetry: TranspositionalSymmetry) -> HashMap<Pc, HashSet<TranspositionalSymmetry>> {
    let symmetry_u8: u8 = symmetry.clone().into();
    let mut symmetries = HashMap::new();
    // Performing this check ahead of time prevents both unnecessary computation and
    // potential index errors that would occur in the logic below.
    if pcs.len() < (12 / symmetry_u8) as usize {
        return symmetries;
    };
    // We only need to check half of the "chromatic circle" as we rotate through the vector.
    let mut checked_through: i32 = 0;
    // We'll rotate this
    let mut rotated = pcs.clone();
    for pc in pcs {
        // We should technically always hit this before completing iteration over pcs,
        // but we need access to the nth [Pc], and thus cannot use a while loop.
        if checked_through >= 6 {
            return symmetries;
        }
        let mut maybe_same = rotated.transpose(i8::try_from(symmetry_u8).unwrap());
        maybe_same.sort();
        if rotated == maybe_same {
            let pt_of_symmetry = u8::from(pc);
            let related_points_of_symmetry: Vec<Pc> = (0u8..(12/symmetry_u8))
                .map(|i| (pt_of_symmetry + symmetry_u8 * i))
                .map(|i| Pc::from(i))
                .collect();
            for pc in related_points_of_symmetry {
                let entry = symmetries.entry(pc.clone())
                    .or_insert_with(|| {
                        HashSet::new()
                    });
                if !entry.contains(&symmetry) {
                    entry.insert(symmetry.clone());
                }
            }
        }
        checked_through += (i32::from(rotated[1]) - i32::from(rotated[0])).rem_euclid(12);
        rotated.rotate_left(1);
        rotated.sort();
    }
    symmetries
}

#[cfg(test)]
mod tests {
    use crate::note::pitch_class::Pc::*;
    use super::*;

    #[test]
    fn test_symmetry() {
        let pc_set = vec![Pc0, Pc6];
        let mut should_be: HashMap<Pc, HashSet<TranspositionalSymmetry>> = HashMap::new();
        should_be.insert(Pc0, HashSet::from([TranspositionalSymmetry::T6]));
        should_be.insert(Pc6, HashSet::from([TranspositionalSymmetry::T6]));
        assert_eq!(find_transpositional_symmetries(&pc_set), should_be);

        let pc_set = vec![Pc0, Pc4, Pc6, Pc10];
        should_be.insert(Pc4, HashSet::from([TranspositionalSymmetry::T6]));
        should_be.insert(Pc10, HashSet::from([TranspositionalSymmetry::T6]));
        assert_eq!(find_transpositional_symmetries(&pc_set), should_be);

        let mut should_be: HashMap<Pc, HashSet<TranspositionalSymmetry>> = HashMap::new();
        let pc_set = vec![Pc1, Pc4, Pc7, Pc10];
        should_be.insert(Pc1, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc4, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc7, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc10, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        assert_eq!(find_transpositional_symmetries(&pc_set), should_be);

        // Augmented AH, every note should have a symmetry at T4.
        let mut should_be: HashMap<Pc, HashSet<TranspositionalSymmetry>> = HashMap::new();
        let pc_set = vec![Pc0, Pc3, Pc4, Pc7, Pc8, Pc11];
        should_be.insert(Pc0, HashSet::from([TranspositionalSymmetry::T4]));
        should_be.insert(Pc3, HashSet::from([TranspositionalSymmetry::T4]));
        should_be.insert(Pc4, HashSet::from([TranspositionalSymmetry::T4]));
        should_be.insert(Pc7, HashSet::from([TranspositionalSymmetry::T4]));
        should_be.insert(Pc8, HashSet::from([TranspositionalSymmetry::T4]));
        should_be.insert(Pc11, HashSet::from([TranspositionalSymmetry::T4]));
        assert_eq!(find_transpositional_symmetries(&pc_set), should_be);

        // Dim HW scale should be transpositionally symmetrical at every note on T6 and T3.
        let mut should_be: HashMap<Pc, HashSet<TranspositionalSymmetry>> = HashMap::new();
        let pc_set = vec![Pc0, Pc1, Pc3, Pc4, Pc6, Pc7, Pc9, Pc10];
        should_be.insert(Pc0, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc1, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc3, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc4, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc6, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc7, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc9, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        should_be.insert(Pc10, HashSet::from([TranspositionalSymmetry::T3, TranspositionalSymmetry::T6]));
        assert_eq!(find_transpositional_symmetries(&pc_set), should_be);
    }
}
