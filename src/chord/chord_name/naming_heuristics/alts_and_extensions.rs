use std::collections::HashSet;
use crate::chord::chord_name::quality::{Alt, AltChoice, Extension};
use crate::note::pc::Pc;

/// This controls how we search for potential alterations,
/// as the presence of some notes in certain contexts is an alteration,
/// but in other contexts is not.
#[derive(Debug, Clone, PartialEq)]
pub enum TriadContext {
    Major,
    Minor,
    Aug,
    Dim,
    Sus,
}

const MINOR_ALTS: &[usize] = &[1, 2, 4, 5, 6, 8, 9];
/// Generate an [Alt] to describe what alterations should be added to a chord name.
pub fn generate_alt(pcs: &HashSet<Pc>, triad_context: TriadContext) -> Option<Alt> {
    let mut alterations = vec![];
    // Based on some starting values and a [TriadContext],
    // we can modify the [possible_alts] local to something tailored to each context.
    let mut possible_alts: Vec<usize> = MINOR_ALTS.to_vec();
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

/// Generate an [Alt] to describe what alterations should be added to a chord name.
/// Also generate a [Vec<Extension>].
/// Used for "xxxN" qualities, i.e. Maj7, dom7, min7, min7b5, etc.
pub fn generate_alt_and_extensions(pcs: &HashSet<Pc>, triad_context: TriadContext) -> (Option<Alt>, Vec<Extension>) {
    let mut extensions = vec![Extension::Seventh];
    let mut alts = generate_alt(pcs, triad_context.clone());
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
            // 6ths/13ths on a diminished chord are diminished 7ths.
            if triad_context != TriadContext::Dim {
                extensions.push(Extension::Thirteenth);
            }
        }
    }
    (alts, extensions)
}
