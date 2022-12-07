use itertools::Itertools;
use crate::error::MusicSemanticsError;
use crate::note::Pc;
use crate::note_collections::PcSet;

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
