use std::collections::HashSet;
use crate::note_collections::chord_name::naming_heuristics::alts_and_extensions::{generate_alt, generate_alt_and_extensions, TriadContext};
use crate::note_collections::chord_name::naming_heuristics::NamingHeuristic;
use crate::note_collections::chord_name::quality::chord::{ChordQuality, MajorSubtype, MinorSubtype};
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

pub fn common_prefix(pcs: &HashSet<Pc>) -> Option<ChordQuality> {
    if pcs.contains(&Pc4) {
        if pcs.contains(&Pc11) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Major);
            return Some(ChordQuality::Major(MajorSubtype::MajN(ext, alt)));
        }
        if pcs.contains(&Pc10) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Major);
            return Some(ChordQuality::Major(MajorSubtype::N(ext, alt)));
        }
        if pcs.contains(&Pc9) {
            let alt = generate_alt(pcs, TriadContext::Major);
            return Some(ChordQuality::Major(MajorSubtype::Maj6(alt)));
        }
        let alt = generate_alt(pcs, TriadContext::Major);
        return Some(ChordQuality::Major(MajorSubtype::Maj(alt)));
    }
    if pcs.contains(&Pc3) {
        if pcs.contains(&Pc11) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Minor);
            return Some(ChordQuality::Minor(MinorSubtype::MinMajN(ext, alt)));
        }
        if pcs.contains(&Pc10) {
            let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Minor);
            return Some(ChordQuality::Minor(MinorSubtype::MinN(ext, alt)));
        }
        if pcs.contains(&Pc9) {
            let alt = generate_alt(pcs, TriadContext::Minor);
            return Some(ChordQuality::Minor(MinorSubtype::Min6(alt)));
        }
        let alt = generate_alt(pcs, TriadContext::Minor);
        return Some(ChordQuality::Minor(MinorSubtype::Min(alt)));
    }
    None
}

/// Common Logic across all heuristics based on diminished chords.
pub fn search_for_maj_min_quality(pcs: &HashSet<Pc>) -> Option<ChordQuality> {
    if pcs.len() == 7 {
        // Do the strategy for scale names instead
    }
    common_prefix(pcs)
}

/// Triads, sans 5th, with a 6th, and possibly also with a 9th.
/// Triads with only 9ths are covered in [MajOrMinN].
#[derive(Debug)]
pub struct MajOrMin69;
impl NamingHeuristic for MajOrMin69 {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3, Pc4]), HashSet::from([Pc9])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc1, Pc2])]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

/// When both the third and sharp ninth are in the chord.
/// We assume a P5th here. It's not tonally important in this case, and is covered elsewhere.
#[derive(Debug)]
pub struct MajSharpNine;
impl NamingHeuristic for MajSharpNine {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3]), HashSet::from([Pc4])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc1]), HashSet::from([Pc9]), HashSet::from([Pc10, Pc11])]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

/// This covers everything from an explicit Major or Minor Triad all the way to
/// any combination of extensions, up through to a seven note scale.
#[derive(Debug)]
pub struct MajOrMinN;
impl NamingHeuristic for MajOrMinN {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc3, Pc4]),
            HashSet::from([Pc7]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5, Pc6]),
            HashSet::from([Pc8, Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

/// When both the third and sharp ninth are in the chord.
/// We assume a P5th here. It's not tonally important in this case, and is covered elsewhere.
#[derive(Debug)]
pub struct MajNSharpNine;
impl NamingHeuristic for MajNSharpNine {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3]), HashSet::from([Pc4]), HashSet::from([Pc7])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc1]), HashSet::from([Pc5, Pc6]), HashSet::from([Pc8, Pc9]), HashSet::from([Pc10, Pc11])]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

#[derive(Debug)]
pub struct MajChordShell;
impl NamingHeuristic for MajChordShell {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc4]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5]),
            HashSet::from([Pc9])
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

#[derive(Debug)]
pub struct MinChordShell;
impl NamingHeuristic for MinChordShell {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc3]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5]),
            HashSet::from([Pc8, Pc9])
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

#[derive(Debug)]
pub struct RootToThirdCluster;
impl NamingHeuristic for RootToThirdCluster {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc3, Pc4]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

#[derive(Debug)]
pub struct ThirdAndFourth;
impl NamingHeuristic for ThirdAndFourth {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc3, Pc4]),
            HashSet::from([Pc5]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}

#[derive(Debug)]
pub struct ThirdAndSharpFourth;
impl NamingHeuristic for ThirdAndSharpFourth {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc4]),
            HashSet::from([Pc6]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        search_for_maj_min_quality(pcs)
    }
}