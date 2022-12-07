pub mod symmetry;

use itertools::Itertools;
use crate::error::MusicSemanticsError;
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

pub fn get_subchords(pcs: &PcSet, size: u8) -> Result<Vec<Vec<Pc>>, MusicSemanticsError> {
    if size < 3 {
        return Err(MusicSemanticsError::SizeTooSmallForChords(size as usize));
    }
    if size as usize > pcs.len() - 1 {
        return Err(MusicSemanticsError::SizeTooLargeForSubchords(size, pcs.clone()));
    }
    Ok((**pcs).clone()
        .into_iter()
        .combinations(size as usize)
        .collect()
    )
}

