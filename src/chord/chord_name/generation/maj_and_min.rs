use std::collections::HashSet;
use crate::chord::chord_name::generation::{generate_alt, generate_alt_and_extensions, TriadContext};
use crate::chord::chord_name::naming_heuristics::NamingHeuristic;
use crate::chord::chord_name::quality::{ChordQuality, MajorSubtype, MinorSubtype};
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
    }
    None
}

/// Common entry function for handling cases where we more of less have a Major or minor
/// chord with possible extensions.
pub fn maj_min_types(pcs: &HashSet<Pc>) -> Option<ChordQuality> {
    if pcs.len() == 7 {
        // Do the strategy for scale names instead
    }
    common_prefix(pcs)
}

/// Triads, sans 5th, with a 6th, and possibly also with a 9th.
/// Triads with only 9ths are covered in [MajOrMinN].
pub struct MajOrMin69;
impl NamingHeuristic for MajOrMin69 {
    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3, Pc4]), HashSet::from([Pc9])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc1, Pc2])]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        maj_min_types(pcs)
    }
}

/// When both the third and sharp ninth are in the chord.
/// We assume a P5th here. It's not tonally important in this case, and is covered elsewhere.
pub struct MajSharpNine;
impl NamingHeuristic for MajSharpNine {
    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc3]), HashSet::from([Pc4])]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![HashSet::from([Pc1]), HashSet::from([Pc9]), HashSet::from([Pc10, Pc11])]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        maj_min_types(pcs)
    }
}

/// This covers everything from an explicit Major or Minor Triad all the way to
/// any combination of extensions, up through to a seven note scale.
pub struct MajOrMinN;
impl NamingHeuristic for MajOrMinN {
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
        maj_min_types(pcs)
    }
}
