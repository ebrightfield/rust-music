use crate::error::MusicSemanticsError;
use crate::note::{Note, Pitch};
use crate::note_collections::{NoteSet, Voicing};

pub struct Voiceleading {
    from: Voicing,
    to: Voicing,
    paths: Vec<i8>,
    distance_metric: fn(&Voiceleading) -> usize,
    rules: Vec<fn(&Voiceleading, &Vec<i8>, &Vec<Pitch>) -> bool>,
}

impl Voiceleading {
    pub fn new(
        from: Voicing,
        paths: Vec<i8>,
        to: Option<&Vec<Note>>,
    ) -> Result<Self, MusicSemanticsError> {
        let paths_applied = from.apply_paths(&paths, to)?;
        Ok(Self {
            from: from,
            to: Voicing::new(paths_applied),
            paths,
            distance_metric: naive_distance,
            rules: vec![],
        })
    }
}

/// A distance metric where we simply sum the absolute values of all the paths of a voiceleading.
pub fn naive_distance(v: &Voiceleading) -> usize {
    v.paths.iter().map(|p| usize::try_from(p.abs()).unwrap()).sum()
}