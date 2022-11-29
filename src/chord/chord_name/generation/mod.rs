pub mod maj_and_min;

use std::collections::HashSet;
use crate::chord::chord_name::naming_heuristics::NamingHeuristic;
use crate::chord::chord_name::quality::{Alt, AltChoice, ChordQuality, Extension, MajorSubtype, MinorSubtype};
use crate::note::pc::Pc;

/// This controls how we search for potential alterations,
/// as the presence of some notes in certain contexts is an alteration,
/// but in other contexts is not.
pub enum TriadContext {
    Major,
    Minor,
    Aug,
    Dim,
    Sus,
}

pub fn generate_alt(pcs: &HashSet<Pc>, triad_context: TriadContext) -> Option<Alt> {
    let mut alterations = vec![];
    let mut possible_alts: Vec<usize> = vec![1, 2, 4, 5, 6, 8, 9];
    match triad_context {
        TriadContext::Major => {
            // Override the Major third with a minor third.
            possible_alts.remove(2);
            possible_alts.insert(2, 3);
        }
        TriadContext::Minor => {
            // Nothing to do here
        }
        TriadContext::Aug => {
            possible_alts = vec![1,2,3,5,6,9];
        }
        TriadContext::Dim => {
            possible_alts = vec![1,2,4,5,8];
        }
        TriadContext::Sus => {
            possible_alts = vec![1,6,8,9];
        }
    }
    for alt_num in possible_alts {
        // We can use unwraps in this block because we only use hardcoded numbers that we
        // know are going to be valid for the type conversions.
        let alt_as_u8 = u8::try_from(alt_num).unwrap();
        if pcs.contains(&Pc::from(alt_as_u8)) {
            alterations.push(AltChoice::try_from(alt_num).unwrap())
        }
    }
    Some(alterations.into())
}

/// Used for "xxxN" qualities, i.e. Maj7, dom7, min7, min7b5, etc.
pub fn generate_alt_and_extensions(pcs: &HashSet<Pc>, triad_context: TriadContext) -> (Option<Alt>, Vec<Extension>) {
    let mut extensions = vec![Extension::Seventh];
    let mut alts = generate_alt(pcs, triad_context);
    if let Some(inner) = alts.as_mut() {
        if inner.contains(&AltChoice::Nine) {
            inner.retain(|x| *x != AltChoice::Nine);
            extensions.push(Extension::Ninth);
        }
        if inner.contains(&AltChoice::Eleven) {
            inner.retain(|x| *x != AltChoice::Eleven);
            extensions.push(Extension::Eleventh);
        }
        if inner.contains(&AltChoice::Thirteenth) {
            inner.retain(|x| *x != AltChoice::Thirteenth);
            extensions.push(Extension::Thirteenth);
        }
    }
    (alts, extensions)
}
