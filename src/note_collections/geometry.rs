use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

pub const WT_SCALE: &[Pc] = &[Pc0, Pc2, Pc4, Pc6, Pc8, Pc10];
pub const WT_SCALE_ODD: &[Pc] = &[Pc1, Pc3, Pc5, Pc7, Pc9, Pc11];

pub fn find_transpositional_symmetries(pcs: &Vec<Pc>) -> HashMap<Pc, HashSet<TranspositionalSymmetry>> {
    let mut symmetries = HashMap::new();
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
            // TODO Do the Aug Scale ones too, then factor this out into a function.
            // Check WT Scale
            if *pcs == WT_SCALE {
                let mut to_add = HashMap::new();
                for pc in WT_SCALE {
                    to_add.insert(pc.clone(), HashSet::from([TranspositionalSymmetry::T2]));
                }
                add_entries(to_add, &mut symmetries);
            } else if *pcs == WT_SCALE_ODD {
                for pc in WT_SCALE_ODD {
                    let mut to_add = HashMap::new();
                    for pc in WT_SCALE {
                        to_add.insert(pc.clone(), HashSet::from([TranspositionalSymmetry::T2]));
                    }
                    add_entries(to_add, &mut symmetries);
                }
            }
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T6), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T4), &mut symmetries);
            add_entries(check_for_symmetry(pcs, TranspositionalSymmetry::T3), &mut symmetries);
        },
        _ => {}
    }
    symmetries
}

pub fn transpose(pcs: &Vec<Pc>, semitones: u8) -> Vec<Pc> {
    pcs
        .iter()
        .map(|pc| Pc::from(u8::from(pc) + 12 - semitones.rem_euclid(12)))
        .collect()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TranspositionalSymmetry {
    T2,
    T3,
    T4,
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

/// The input of this function assumes a well-ordered, deduped [Vec<Pc>],
/// but does not have to be normalized to Pc0.
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
        let mut maybe_same = transpose(&rotated, symmetry_u8);
        maybe_same.sort();
        if rotated == maybe_same {
            let pt_of_symmetry = u8::from(pc);
            let related_points_of_symmetry: Vec<Pc> = (0u8..(12/symmetry_u8))
                .map(|i| (pt_of_symmetry + symmetry_u8 * i))
                .map(|i| Pc::from(i))
                .collect();
            for pc in related_points_of_symmetry {
                let mut entry = symmetries.entry(pc.clone())
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
    use crate::note_collections::pc_set::PcSet;
    use super::*;

    #[test]
    fn test_rotate() {
        let pc_set = PcSet::new(vec![Pc0, Pc3, Pc7]);
        let rotated = pc_set.rotate_fwd();
        assert_eq!(PcSet::new(vec![Pc0, Pc4, Pc9]), rotated);
        let rotated_back = rotated.rotate_back();
        assert_eq!(pc_set, rotated_back);
    }

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
    }
}
