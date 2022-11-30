use std::collections::HashSet;
use crate::chord::chord_name::quality::{Alt, AltChoice, ChordQuality, MajorSubtype, MinorSubtype};
use crate::chord::chord_name::quality::scale::ScaleQuality;
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

pub mod maj_and_min_qualities;
pub mod alts_and_extensions;
pub mod aug_qualities;
pub mod dim_qualities;
pub mod sus_qualities;
pub mod inferred_third_qualities;
pub mod scale_qualities;

/// A Chord Naming Heuristic contains two sets:
/// - Required Pcs -- Vec of subsets of Pcs, the input must contain only one element in each subset.
/// - Optional Pcs -- Vec of subsets of Pcs, the "only one" requirement likewise applies.
///
/// In order to "match" a naming heuristic's requirements, all elements of [pcs] must match,
/// and all [HashSet]s in [self.required] should intersect on only one element of [pcs].
///
/// Many naming heuristics are built, which can then be iterated over. When a call to
/// [NamingHeuristic::validate] returns true, we can then call [NamingHeuristic::generate_name].
///
/// It is not required that a heuristic generate a name.
pub trait NamingHeuristic: std::fmt::Debug {
    /// For our purposes, either a [ChordQuality] or a [ScaleQuality].
    /// In principle, one could build their own naming system and put anything here,
    /// even a simple string.
    type T;

    /// We want to our chord in question to have only _one_ element in common with each [HashSet].
    /// This property must hold true for each element.
    fn required(&self) -> Vec<HashSet<Pc>> { vec! [] }
    /// We want to our chord in question to have only _one_ element in common with each [HashSet].
    /// These properties are optional, all or none of them could match.
    fn optional(&self) -> Vec<HashSet<Pc>> { vec! [] }

    /// One-shot execution of an attempt at applying this heuristic to naming a chord.
    /// If the heuristic simply doesn't apply, it returns [None].
    /// Likewise, [self.generate_name] can sometimes return [None].
    fn apply(&self, pcs: &HashSet<Pc>) -> Option<Self::T> {
        if self.validate(pcs) {
            return self.generate_name(pcs);
        }
        None
    }

    /// Try to generate a name based on the content of [pcs].
    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<Self::T>;

    /// Does a given [Vec<Pc>] satisfy the following:
    /// 1. All intersections with required [HashSet]s have only one element.
    /// 2. All elements in [pcs] are matched,
    ///    whether through required or optional [HashSet] intersections.
    fn validate(&self, pcs: &HashSet<Pc>) -> bool {
        // Clone and remove any Pc0.
        let mut pcs = pcs.clone();
        pcs.remove(&Pc0);
        let mut matched = vec![];
        for subset in self.required().iter() {
            let intersection: Vec<Pc> = subset
                .intersection(&pcs)
                .map(|pc| pc.clone())
                .collect();
            if intersection.len() == 1 {
                matched.extend(intersection);
            } else {
                // Failed a subset requirement
                return false;
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

type ChordHeuristic = Box<dyn NamingHeuristic<T=ChordQuality>>;

/// An order-sensitive list of all the various naming heuristics.
/// The first heuristic to match on the content is applied to generating a name.
pub fn chord_heuristics() -> Vec<ChordHeuristic> {
    // Order matters here! The first match will be dispatched to name generation.
    vec![
        // Major / minor
        Box::new(maj_and_min_qualities::MajOrMin69),
        Box::new(maj_and_min_qualities::MajSharpNine),
        Box::new(maj_and_min_qualities::MajOrMinN),
        Box::new(maj_and_min_qualities::MajNSharpNine),
        Box::new(maj_and_min_qualities::MajChordShell),
        Box::new(maj_and_min_qualities::MinChordShell),
        Box::new(maj_and_min_qualities::RootToThirdCluster),
        Box::new(maj_and_min_qualities::ThirdAndFourth),
        Box::new(maj_and_min_qualities::ThirdAndSharpFourth),
        // Inferred Major / minor (no third in the actual set)
        Box::new(inferred_third_qualities::FifthAndUpperNotes),
        Box::new(inferred_third_qualities::NinthAndSixthNoThird),
        Box::new(inferred_third_qualities::TritoneAndSeventh),
        Box::new(inferred_third_qualities::NinthAndSeventh),
        // Aug chords
        Box::new(aug_qualities::AugChordQualities),
        // Dim chords
        Box::new(dim_qualities::DimNChords),
        Box::new(dim_qualities::NotMin6Chord),
        Box::new(dim_qualities::TritoneAndDimSeventh),
        // Sus chords
        Box::new(sus_qualities::SusNChords),
        Box::new(sus_qualities::BothSecondAndFourth),
        Box::new(sus_qualities::Altered13Sus),
        Box::new(sus_qualities::FourthAndSeventh),
        Box::new(sus_qualities::FlatSecondAndFourth),
    ]
}

/// Infer a [ChordQuality] from a [HashSet<Pc>]. This is a not guaranteed to produce a quality.
/// Assumes at least three unique [Pc] in [pcs].
/// Other possibilities should be screened out ahead of time
pub fn infer_chord_quality(pcs: &HashSet<Pc>) -> Option<(ChordHeuristic, Option<ChordQuality>)> {

    let mut heuristics = vec![];
    let result: Vec<_> = chord_heuristics()
        .iter()
        .map(|h| heuristics.push(h.apply(pcs)))
        .collect();
    for heuristic in chord_heuristics() {
        if heuristic.validate(&pcs) {
            let name = heuristic.generate_name(&pcs);
            return Some((heuristic, name));
        }
    }
    None
}

type ScaleHeuristic = Box<dyn NamingHeuristic<T=ScaleQuality>>;

pub fn scale_heuristics() -> Vec<ScaleHeuristic> {
    // Order matters here! The first match will be dispatched to name generation.
    vec![
        // Literal equivalence checks
        Box::new(scale_qualities::WholetoneScale),
        Box::new(scale_qualities::AugAHScale),
        Box::new(scale_qualities::AugHAScale),
        Box::new(scale_qualities::DimHWScale),
        Box::new(scale_qualities::DimWHScale),
        Box::new(scale_qualities::HarmonicMinor),
        Box::new(scale_qualities::HarmonicMajor),
        Box::new(scale_qualities::AlteredScale),
        // Scales with possible alterations
        Box::new(scale_qualities::MajorScale),
        Box::new(scale_qualities::IonianAug),
        Box::new(scale_qualities::Dorian),
        Box::new(scale_qualities::Phrygian),
        Box::new(scale_qualities::Lydian),
        Box::new(scale_qualities::LydianAug),
        Box::new(scale_qualities::Mixolydian),
        Box::new(scale_qualities::MixolydianAug),
        Box::new(scale_qualities::NaturalMinor),
        Box::new(scale_qualities::MelodicMinor),
        Box::new(scale_qualities::Locrian),
    ]
}

/// Note that in this case, our [Box<dyn NamingHeuristic>] in the return type is
/// wrapped in its own [Option]. This is because unlike with chords,
pub fn infer_scale_quality(pcs: &HashSet<Pc>) -> Option<(Option<ScaleHeuristic>, Option<ScaleQuality>)> {
    let mut pcs = pcs.clone();
    pcs.remove(&Pc0);
    for heuristic in scale_heuristics() {
        if heuristic.validate(&pcs) {
            let name = heuristic.generate_name(&pcs);
            return Some((Some(heuristic), name));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::chord::chord_name::naming_heuristics::maj_and_min_qualities::MajOrMinN;
    use crate::chord::chord_name::quality::Extension;
    use super::*;

    #[test]
    fn chord_names() {
        let notes = vec![Pc0, Pc4, Pc5, Pc7, Pc11];
        let notes = sanitize_pcs(&notes);
        let quality = infer_chord_quality(&notes);
        assert_eq!(
            quality.unwrap().1,
             Some(ChordQuality::Major(MajorSubtype::MajN(
                vec![Extension::Seventh, Extension::Eleventh], Alt(vec![])
            )))
        );
        let notes = vec![Pc0, Pc2, Pc4, Pc5, Pc7, Pc9, Pc11];
        let notes = sanitize_pcs(&notes);
        let quality = infer_scale_quality(&notes);
        assert_eq!(quality.unwrap().1,
            Some(
             ScaleQuality::Major(vec![], vec![])
            )
        );
        let notes = vec![Pc0, Pc1, Pc4, Pc5, Pc7, Pc9, Pc11];
        let notes = sanitize_pcs(&notes);
        let quality = infer_scale_quality(&notes);
        println!("{:?}", quality);
        let notes = vec![Pc0, Pc1, Pc4, Pc6, Pc7, Pc8, Pc10];
        let notes = sanitize_pcs(&notes);
        let quality = infer_scale_quality(&notes);
        println!("{:?}", quality);
        let notes = vec![Pc0, Pc1, Pc3, Pc5, Pc6, Pc8, Pc10];
        let notes = sanitize_pcs(&notes);
        let quality = infer_scale_quality(&notes);
        println!("{:?}", quality);
        let notes = vec![Pc0, Pc2, Pc3, Pc5, Pc6, Pc8, Pc10];
        let notes = sanitize_pcs(&notes);
        let quality = infer_scale_quality(&notes);
        println!("{:?}", quality);
    }
}

