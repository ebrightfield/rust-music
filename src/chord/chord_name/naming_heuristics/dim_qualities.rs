use std::collections::HashSet;
use crate::chord::chord_name::naming_heuristics::alts_and_extensions::{generate_alt, generate_alt_and_extensions, TriadContext};
use crate::chord::chord_name::naming_heuristics::inferred_third_qualities::assumed_third_common_prefix;
use crate::chord::chord_name::naming_heuristics::NamingHeuristic;
use crate::chord::chord_name::quality::{AugSubtype, ChordQuality, DimSubtype};
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

/// Common Logic across all heuristics based on diminished chords.
pub fn search_for_dim_quality(pcs: &HashSet<Pc>) -> ChordQuality {
    if pcs.contains(&Pc9) {
        let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Dim);
        return ChordQuality::Dim(DimSubtype::DimN(ext, alt));
    }
    if pcs.contains(&Pc10) {
        let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Dim);
        return ChordQuality::Dim(DimSubtype::MinNb5(ext, alt));
    }
    if pcs.contains(&Pc11) {
        let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Dim);
        return ChordQuality::Dim(DimSubtype::DimMajN(ext, alt));
    }
    let alt = generate_alt(pcs, TriadContext::Dim);
    return ChordQuality::Dim(DimSubtype::Dim(alt));
}

#[derive(Debug)]
pub struct DimNChords;
impl NamingHeuristic for DimNChords {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc3]),
            HashSet::from([Pc6]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5]),
            HashSet::from([Pc8]),
            HashSet::from([Pc9, Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_dim_quality(pcs))
    }
}

#[derive(Debug)]
pub struct NotMin6Chord;
impl NamingHeuristic for NotMin6Chord {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc3]),
            HashSet::from([Pc8]),
            HashSet::from([Pc9]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_dim_quality(pcs))
    }
}

#[derive(Debug)]
pub struct TritoneAndDimSeventh;
impl NamingHeuristic for TritoneAndDimSeventh {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc6]),
            HashSet::from([Pc9]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5]),
            HashSet::from([Pc8]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_dim_quality(pcs))
    }
}

