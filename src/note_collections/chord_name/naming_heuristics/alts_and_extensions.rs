use std::collections::HashSet;
use crate::note_collections::chord_name::quality::chord::{Alt, AltChoice, Extension};
use crate::note::pc::Pc;

/// This controls how we search for potential note_collections alterations,
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

/// Generate an [Alt] to describe what alterations should be added to a note_collections name.
pub fn generate_alt(pcs: &HashSet<Pc>, triad_context: TriadContext) -> Alt {
    let mut alterations = vec![];
    // Based on some starting values and a [TriadContext],
    // we can modify the [possible_alts] local to something tailored to each context.
    let possible_alts: Vec<usize> = match triad_context {
        TriadContext::Major => vec![1,2,3,5,6,8,9],
        TriadContext::Minor => vec![1,2,4,5,6,8,9],
        TriadContext::Aug => vec![1,2,3,5,6,9],
        TriadContext::Dim => vec![1,2,4,5,8],
        TriadContext::Sus => vec![1,6,8,9]
    };
    for alt_num in possible_alts {
        // We can use unwraps in this block because we only use hardcoded numbers that we
        // know are going to be valid for the type conversions.
        let alt_as_u8 = u8::try_from(alt_num).unwrap();
        if pcs.contains(&Pc::from(alt_as_u8)) {
            alterations.push(AltChoice::try_from(alt_num).unwrap())
        }
    }
    alterations.into()
}

/// Generate an [Alt] to describe what alterations should be added to a note_collections name.
/// Also generate a [Vec<Extension>].
/// Used for "xxxN" qualities, i.e. Maj7, dom7, min7, min7b5, etc.
pub fn generate_alt_and_extensions(pcs: &HashSet<Pc>, triad_context: TriadContext) -> (Alt, Vec<Extension>) {
    let mut extensions = vec![Extension::Seventh];
    let mut alts = generate_alt(pcs, triad_context.clone());
    // Generate extensions
    if alts.contains(&AltChoice::Nine) {
        alts.retain(|x| *x != AltChoice::Nine);
        extensions.push(Extension::Ninth);
    }
    if alts.contains(&AltChoice::Eleven) {
        alts.retain(|x| *x != AltChoice::Eleven);
        // 4ths/11ths on Sus chords are the Sus part!
        if triad_context != TriadContext::Sus {
            extensions.push(Extension::Eleventh);
        }
    }
    if alts.contains(&AltChoice::Thirteenth) {
        alts.retain(|x| *x != AltChoice::Thirteenth);
        // 6ths/13ths on a diminished note_collections are diminished 7ths.
        if triad_context != TriadContext::Dim {
            extensions.push(Extension::Thirteenth);
        }
    }
    (alts, extensions)
}
