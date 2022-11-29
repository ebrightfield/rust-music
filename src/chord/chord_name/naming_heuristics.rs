use std::collections::HashSet;
use crate::chord::chord_name::generation::maj_and_min;
use crate::chord::chord_name::quality::{Alt, AltChoice, ChordQuality, MajorSubtype, MinorSubtype};
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

/// A Chord Naming Rule contains two sets:
/// - Required Pcs -- Vec of subsets of Pcs, the input must contain only one element in each subset.
/// - Optional Pcs -- Vec of subsets of Pcs, the "only one" requirement likewise applies.
///
/// In order to "match" a naming heuristic's requirements, all elements of [pcs] must match,
/// and all [HashSet]s in [self.required] should intersect on only one element of [pcs].
pub trait NamingHeuristic {

    /// We want to our chord in question to have only _one_ element in common with each [HashSet].
    /// This property must hold true for each element.
    fn required(&self) -> Vec<HashSet<Pc>>;
    /// We want to our chord in question to have only _one_ element in common with each [HashSet].
    /// These properties are optional, all or none of them could match.
    fn optional(&self) -> Vec<HashSet<Pc>>;

    /// If the naming heuristic matches with the content of [pcs], then
    /// attempt to generate a chord name.
    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality>;

    /// Does a given [Vec<Pc>] satisfy the following:
    /// 1. All intersections with required [HashSet]s have only one element.
    /// 2. All elements in [pcs] are matched,
    ///    whether through required or optional [HashSet] intersections.
    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        // Clone and remove any Pc0.
        //let pcs = sanitize_pcs(pcs);
        let mut matched = vec![];
        for subset in self.required().iter() {
            let intersection: Vec<Pc> = subset
                .intersection(&pcs)
                .map(|pc| pc.clone())
                .collect();
            if intersection.len() != 1 {
                return false;
            } else {
                matched.extend(intersection);
            }
        }
        for subset in self.optional().iter() {
            let intersection: Vec<Pc> = subset
                .intersection(&pcs)
                .map(|pc| pc.clone())
                .collect();
            if intersection.len() == 1 {
                matched.extend(intersection);
            }
        }
        matched.len() == pcs.len()
    }
}

/// Convert a [Vec<Pc>] into a [HashSet<Pc>] for processing in these naming modules.
pub fn sanitize_pcs(pcs: &Vec<Pc>) -> HashSet<Pc> {
    let mut pcs = pcs.clone();
    pcs.retain(|pc| *pc != Pc0);
    pcs.into_iter().collect()
}

/// An order-sensitive list of all the various naming heuristics.
/// The first heuristic to match on the content is applied to generating a name.
pub fn heuristics() -> Vec<Box<dyn NamingHeuristic>> {
    // Order matters here!
    vec![
        Box::new(maj_and_min::MajOrMin69),
        Box::new(maj_and_min::MajSharpNine),
        Box::new(maj_and_min::MajOrMinN),
    ]
}