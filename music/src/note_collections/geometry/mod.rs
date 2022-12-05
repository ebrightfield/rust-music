pub mod symmetry;

use anyhow::anyhow;
use itertools::Itertools;
use crate::note::pc::Pc;
use crate::note_collections::pc_set::PcSet;

pub trait IntervallicSymmetry {
    fn is_intervallically_symmetric(&self) -> bool;
}

// TODO Voiceleading search built off of this type.
pub struct IntervalMatrix(Vec<Vec<i8>>);

pub fn get_modes(pcs: &PcSet) -> Vec<PcSet> {
    (0..pcs.len())
        .map(|i| {
            pcs.rotate(isize::try_from(i).unwrap())
        })
        .collect()
}

pub fn get_subchords(pcs: &PcSet, size: u8) -> anyhow::Result<Vec<Vec<Pc>>> {
    if size < 3 {
        return Err(anyhow!("Size too small for subchords: {}", size));
    }
    if size as usize > pcs.len() - 1 {
        return Err(anyhow!("Size too large: {}. Needs to be <= {}", size, pcs.len() - 1));
    }
    Ok((**pcs).clone()
        .into_iter()
        .combinations(size as usize)
        .collect()
    )
}

