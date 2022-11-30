pub mod quality;
pub mod naming_heuristics;

use crate::note_collections::pc_set::PcSet;
use crate::note::note::Note;
use std::fmt::{Display, Formatter};
use quality::chord::ChordQuality;
use crate::note::pc::Pc;

/*
There are a number of options I should add to configuring how note_collections names are converted to strings.
1. How to notate extensions?
2. Explicit Sus4?
3. Fancy chars?
4. Treat extensions as alterations, or enforce that it's gotta be stacked 9th, 11th, 13th.., or allow subsets

 */

// TODO Display config and implementation.
// TODO Add interval and single note solutions.

/// All information necessary to describe a note_collections using typical
/// western note_collections names. There are some occasionally weird chords, those
/// need to be handled uniquely.
#[derive(Debug, Clone)]
pub struct ChordName {
    /// Information regarding any choice of root notes, slash note_collections, or
    /// specifying that we are not generalizing over notes at all.
    tonality: TonalSpecification,
    /// Combination of tonal "flavors" asserted to be in the note_collections.
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

/// Whether or not something is a slash note_collections.
/// All specified notes are assumed to be members of their associated [Vec<Pc>].
#[derive(Debug, Clone)]
pub enum TonalSpecification {
    /// If it's a slash note_collections, the bass note will be supplied here.
    SlashChord {
        bass: Note,
        root: Note,
    },
    /// Root note relative to the defined note_collections quality.
    RootPosition(Note),
    /// No tonal specification. The [Option<Pc>] specifies any possible bass note.
    /// The relevant bass note must be an element in the [Vec<Pc>] being named.
    None(Option<Pc>)
}