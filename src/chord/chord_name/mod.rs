pub mod quality;
pub mod naming_heuristics;
pub mod generation;

use std::collections::HashSet;
use crate::chord::pc_set::PcSet;
use crate::note::note::Note;
use std::fmt::{Display, Formatter};
use quality::{ChordQuality, TonalSpecification};
use crate::chord::chord_name::naming_heuristics::sanitize_pcs;
use crate::chord::octave_partition::BaseChromaticInterval;
use crate::note::pc::Pc;

/// All information necessary to describe a chord using typical
/// western chord names. There are some occasionally weird chords, those
/// need to be handled uniquely.
#[derive(Debug, Clone)]
pub struct ChordName {
    /// Information regarding any choice of root notes, slash chord, or
    /// specifying that we are not generalizing over notes at all.
    tonality: TonalSpecification,
    /// Combination of tonal "flavors" asserted to be in the chord.
    quality: ChordQuality,
    /// Underlying set of pitch classes on which the name is being asserted.
    pc_set: PcSet,
}

// fn construct_quality_string(quality: &ChordQuality, ext: &ChordExtension) -> String {
//     match quality {
//         ChordQuality::Major => {
//             match ext {
//                 ChordExtension::None => "Maj",
//                 ChordExtension::N => "Maj",
//             }
//         }
//     }
// }

impl Display for ChordName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // let mut bass = Spelling::from(&self.root).to_string();
        // let mut is_slash_chord = false;
        // if let Some(note) = &self.bass {
        //     bass = Spelling::from(note).to_string();
        //     is_slash_chord = true;
        // }
        // f.write_str(&bass)?;
        Ok(())
    }
}

pub fn infer_chord_quality(pcs: &Vec<Pc>) -> Option<ChordQuality> {
    let pcs = sanitize_pcs(pcs);
    for heuristic in naming_heuristics::heuristics() {
        if heuristic.validate(&pcs) {
            return heuristic.generate_name(&pcs);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::pc::Pc::*;

    #[test]
    fn chord_names() {
        let notes = vec![Pc0, Pc4, Pc7, Pc11];
        let quality = infer_chord_quality(&notes);
        println!("{:?}", quality);
    }
}