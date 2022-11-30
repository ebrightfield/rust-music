use std::collections::HashSet;
use crate::chord::chord_name::naming_heuristics::alts_and_extensions::{generate_alt, generate_alt_and_extensions, TriadContext};
use crate::chord::chord_name::naming_heuristics::inferred_third_qualities::assumed_third_common_prefix;
use crate::chord::chord_name::naming_heuristics::NamingHeuristic;
use crate::chord::chord_name::quality::{AugSubtype, ChordQuality, MajorSubtype};
use crate::note::pc::Pc;
use crate::note::pc::Pc::*;

/// Common Logic across all heuristics based on diminished chords.
pub fn search_for_aug_quality(pcs: &HashSet<Pc>) -> ChordQuality {
    if pcs.contains(&Pc10) {
        let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Aug);
        return ChordQuality::Aug(AugSubtype::AugN(ext, alt));
    }
    if pcs.contains(&Pc11) {
        let (alt, ext) = generate_alt_and_extensions(pcs, TriadContext::Aug);
        return ChordQuality::Aug(AugSubtype::AugMajN(ext, alt));
    }
    let alt = generate_alt(pcs, TriadContext::Aug);
    return ChordQuality::Aug(AugSubtype::Aug(alt));
}

#[derive(Debug)]
pub struct AugChordQualities;
impl NamingHeuristic for AugChordQualities {
    type T = ChordQuality;

    fn required(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc4]),
            HashSet::from([Pc8]),
        ]
    }

    fn optional(&self) -> Vec<HashSet<Pc>> {
        vec![
            HashSet::from([Pc1, Pc2]),
            HashSet::from([Pc5, Pc6]),
            HashSet::from([Pc9]),
            HashSet::from([Pc10, Pc11]),
        ]
    }

    fn generate_name(&self, pcs: &HashSet<Pc>) -> Option<ChordQuality> {
        Some(search_for_aug_quality(pcs))
    }
}

