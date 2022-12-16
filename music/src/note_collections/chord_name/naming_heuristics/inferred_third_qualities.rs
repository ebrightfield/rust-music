use std::collections::HashSet;
use crate::note_collections::chord_name::naming_heuristics::{maj_and_min_qualities, NamingHeuristic};
use crate::note_collections::chord_name::quality::chord::ChordQuality;
use crate::note::pitch_class::Pc;
use crate::note::pitch_class::Pc::*;

const SUGGESTIVE_OF_MAJOR_THIRD: &[Pc] = &[Pc0, Pc2, Pc5, Pc7, Pc9, Pc10, Pc11];

/// These reduce to an evaluation of Major / minor qualities, but with an assumed
/// third.
pub fn assumed_third_common_prefix(pcs: &HashSet<Pc>) -> Option<ChordQuality> {
    let mut clone = pcs.clone();
    if pcs.iter().all(|pc| SUGGESTIVE_OF_MAJOR_THIRD.contains(pc)) {
        clone.insert(Pc4);
        return maj_and_min_qualities::common_prefix(&clone);
    } else {
        clone.insert(Pc3);
        return maj_and_min_qualities::common_prefix(&clone);
    }
}


/// A perfect fifth, and possibly a sharp fourth, a sixth, and/or a seventh.
#[derive(Debug)]
pub struct FifthAndUpperNotes;

impl NamingHeuristic for FifthAndUpperNotes {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc7]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc6]),
            HashSet::from([Pc8, Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        assumed_third_common_prefix(pcs)
    }
}

#[derive(Debug)]
pub struct NinthAndSixthNoThird;

impl NamingHeuristic for NinthAndSixthNoThird {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc9]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        assumed_third_common_prefix(pcs)
    }
}

#[derive(Debug)]
pub struct TritoneAndSeventh;

impl NamingHeuristic for TritoneAndSeventh {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc6]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        assumed_third_common_prefix(pcs)
    }
}

#[derive(Debug)]
pub struct NinthAndSeventh;

impl NamingHeuristic for NinthAndSeventh {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        assumed_third_common_prefix(pcs)
    }
}
