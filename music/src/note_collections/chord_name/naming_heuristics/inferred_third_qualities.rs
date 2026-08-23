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
        maj_and_min_qualities::common_prefix(&clone)
    } else {
        clone.insert(Pc3);
        maj_and_min_qualities::common_prefix(&clone)
    }
}


/// A perfect fifth, and possibly a sharp fourth, a sixth, and/or a seventh.
///
/// After Tier 1.3, bare P5 (`{Pc0, Pc7}`) is handled by the fast-path in
/// [`infer_chord_quality`] as `Interval(Ic7)` → "P5", so this heuristic must
/// not claim it. We additionally require at least one upper extension
/// (Pc2/Pc5/Pc6/Pc8/Pc9/Pc10/Pc11) before inferring a triad-quality here;
/// otherwise a minimal P5 fragment risks being promoted to a full major /
/// minor chord (see Tier 1.5).
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

    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        // Delegate to the default required/optional cover check first.
        if !default_validate(self, pcs) { return false; }
        // Additional guard: at least one upper-extension pc must be present.
        // Pc6 alone is not enough — see Tier 1.5: a bare P5 + sharp-fourth is
        // not a triad-with-inferred-third.
        const UPPER_EXT: &[Pc] = &[Pc2, Pc5, Pc9, Pc10, Pc11];
        pcs.iter().any(|pc| UPPER_EXT.contains(pc))
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        assumed_third_common_prefix(pcs)
    }
}

/// Helper: run the trait's default validator logic over a `NamingHeuristic`
/// impl. This lets custom `validate` overrides compose with the shared
/// required/optional cover check without duplicating that logic.
fn default_validate<H: NamingHeuristic + ?Sized>(h: &H, pcs: &HashSet<Pc>) -> bool {
    let mut pcs = pcs.clone();
    pcs.remove(&Pc0);
    let mut matched = vec![];
    for subset in h.required().iter() {
        let intersection: Vec<Pc> = subset
            .intersection(&pcs)
            .copied()
            .collect();
        if intersection.len() == 1 {
            matched.extend(intersection);
        } else {
            return false;
        }
    }
    for subset in h.optional().iter() {
        let intersection: Vec<Pc> = subset
            .intersection(&pcs)
            .copied()
            .collect();
        if intersection.len() == 1 {
            matched.extend(intersection);
        }
    }
    matched.len() == pcs.len()
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
