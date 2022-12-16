use std::collections::HashSet;
use crate::note_collections::chord_name::naming_heuristics::alts_and_extensions::{generate_alt, generate_alt_and_extensions, TriadContext};
use crate::note_collections::chord_name::naming_heuristics::NamingHeuristic;
use crate::note_collections::chord_name::quality::chord::{Alt, ChordQuality, SusSubtype};
use crate::note::pitch_class::Pc;
use crate::note::pitch_class::Pc::*;

/// Common Logic across all heuristics based on diminished chords.
pub fn search_for_sus_quality(pcs: &HashSet<Pc>) -> ChordQuality {
    if *pcs == HashSet::from([Pc0, Pc2, Pc5]) {
        let alt = generate_alt(pcs, TriadContext::Sus);
        return ChordQuality::Sus(SusSubtype::Sus4(alt));
    }
    if *pcs == HashSet::from([Pc0, Pc2, Pc5, Pc7, Pc9]) {
        return ChordQuality::Sus(SusSubtype::SixNineSus(Alt::empty()));
    }
    if pcs.contains(&Pc2) && !pcs.contains(&Pc5) {
        let alt = generate_alt(pcs, TriadContext::Sus);
        return ChordQuality::Sus(SusSubtype::Sus2(alt));
    }
    if pcs.contains(&Pc10) {
        let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Sus);
        return ChordQuality::Sus(SusSubtype::DomNSus(ext, alt));
    }
    if pcs.contains(&Pc11) {
        let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Sus);
        return ChordQuality::Sus(SusSubtype::MajNSus(ext, alt));
    }
    let alt = generate_alt(pcs, TriadContext::Sus);
    return ChordQuality::Sus(SusSubtype::Sus4(alt));
}

#[derive(Debug)]
pub struct SusNChords;
impl NamingHeuristic for SusNChords {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc2, Pc5]),
            HashSet::from([Pc7]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc8, Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_sus_quality(pcs))
    }
}

#[derive(Debug)]
pub struct BothSecondAndFourth;
impl NamingHeuristic for BothSecondAndFourth {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc2]),
            HashSet::from([Pc5]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc7]),
            HashSet::from([Pc8, Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_sus_quality(pcs))
    }
}


#[derive(Debug)]
pub struct Altered13Sus;
impl NamingHeuristic for Altered13Sus {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc5]),
            HashSet::from([Pc8, Pc9]),
            HashSet::from([Pc10]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_sus_quality(pcs))
    }
}

#[derive(Debug)]
pub struct FourthAndSeventh;

impl NamingHeuristic for FourthAndSeventh {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc5]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_sus_quality(pcs))
    }
}

#[derive(Debug)]
pub struct FlatSecondAndFourth;

impl NamingHeuristic for FlatSecondAndFourth {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1]),
            HashSet::from([Pc5]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_sus_quality(pcs))
    }
}
