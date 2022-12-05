use std::collections::HashMap;
use itertools::Itertools;
use crate::note_collections::voicing::Voicing;
use crate::fretboard::Fretboard;
use crate::fretboard::fretboard_shape::{ChordShapeClassification, FretboardShape};
use crate::fretboard::fretted_note::FrettedNote;
use crate::notation::clef::Clef;
use crate::note::note::Note;

#[derive(Debug)]
pub struct ChordShapeSearchResult<'a> {
    pub playable: HashMap<Voicing, Vec<FretboardShape<'a>>>,
    pub wide_intervals: HashMap<Voicing, Vec<FretboardShape<'a>>>,
    pub nontransposable: HashMap<Voicing, Vec<FretboardShape<'a>>>,
    pub all_above_12th_fret: HashMap<Voicing, Vec<FretboardShape<'a>>>,
    pub unplayable: HashMap<Voicing, Vec<FretboardShape<'a>>>,
}

impl<'a> ChordShapeSearchResult<'a> {
    pub fn new() -> Self {
        Self {
            playable: HashMap::new(),
            wide_intervals: HashMap::new(),
            nontransposable: HashMap::new(),
            all_above_12th_fret: HashMap::new(),
            unplayable: HashMap::new(),
        }
    }
}

/// Chord shapes are [FretboardShape]s where there is exactly one [FrettedNote] per string.
/// If the string is not played in the chord, we denote it with a [FrettedNote::Muted].
pub fn find_chord_shapes<'a>(chord: &Vec<Note>, fretboard: &'a Fretboard) -> anyhow::Result<ChordShapeSearchResult<'a>> {
    let chord_len = chord.len();
    let num_strings: u8 = fretboard.num_strings();
    // String groupings are e.g. 0x0000. Note that x0000x is distinct from 0000xx.
    let string_groupings = (0u8..num_strings).combinations(chord_len);

    let mut valid_shapes = ChordShapeSearchResult::new();

    for grouping in string_groupings {
        // Ordered permutations of notes
        for permutation in chord.iter().permutations(chord_len) {
            // Determine whether to test the voicing with a particular value moved up an octave.
            // This causes redundancies in the search, but in all practical circumstances
            // the loss is acceptable.
            let frets: Vec<Vec<u8>> = permutation
                .iter()
                .enumerate()
                .map(|(i, note)| {
                    let fret = fretboard.which_fret(note, grouping[i])?;
                    if fret < 6 {
                        return Ok::<_, anyhow::Error>(vec![fret, fret + 12]);
                    }
                    Ok::<_, anyhow::Error>(vec![fret])
                })
                .into_iter()
                .flatten()
                .collect();
            // Flip through each possible combination of octave choices on each string
            for fret_shape in frets.iter().multi_cartesian_product() {
                // Making a [FretboardShape]
                let strings = (0u8..num_strings)
                    .map(|i| {
                        let index = grouping.iter().position(|item| *item == i);
                        if let Some(index) = index {
                            return Ok::<_, anyhow::Error>(FrettedNote::Sounded(
                                fretboard.sounded_note(i, *fret_shape[index])?
                            ));
                        }
                        Ok::<_, anyhow::Error>(FrettedNote::Muted {
                            string: i,
                            fretboard,
                        })
                    })
                    .into_iter()
                    .flatten()
                    .collect();
                let shape = FretboardShape {
                    fretted_notes: strings,
                    fretboard,
                };
                // Classifying it, and indexing it into the search results.
                let key: Voicing = (&shape).into();
                let key = key.normalize_register_to_clef(Clef::Treble).unwrap();
                match shape.classify() {
                    ChordShapeClassification::Playable => {
                        if key.has_wide_intervals() {
                            valid_shapes.wide_intervals
                                .entry(key)
                                .or_insert_with(|| vec![])
                                .push(shape);
                        } else {
                            valid_shapes.playable
                                .entry(key)
                                .or_insert_with(|| vec![])
                                .push(shape);
                        }
                    },
                    ChordShapeClassification::AllAbove12thFret => {
                        valid_shapes.all_above_12th_fret
                            .entry(key)
                            .or_insert_with(|| vec![])
                            .push(shape);
                    },
                    ChordShapeClassification::NonTransposable => {
                        valid_shapes.nontransposable
                            .entry(key)
                            .or_insert_with(|| vec![])
                            .push(shape);
                    },
                    ChordShapeClassification::Unplayable => {
                        valid_shapes.unplayable
                            .entry(key)
                            .or_insert_with(|| vec![])
                            .push(shape);
                    },
                }
            }
        }
    }
    Ok(valid_shapes)
}
