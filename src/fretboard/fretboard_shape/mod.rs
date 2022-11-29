pub mod chord_shape_search;
pub mod melodic_shape_search;

use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter, Pointer};
use std::iter::zip;
use std::ops::Index;
use crate::chord::pc_set::PcSet;
use crate::fretboard::{Fretboard, FrettedNote, FrettedNoteKind};
use crate::pitch::Pitch;
use itertools::{all, Itertools};
use melodic_shape_search::MelodicFretboardShape;
use crate::chord::voicing::StackedIntervals;
use crate::note::note::Note;

#[derive(Debug)]
pub struct FretboardShape<'a> {
    pub fretted_notes: Vec<FrettedNote<'a>>,
    pub fretboard: &'a Fretboard,
}

impl<'a> Display for FretboardShape<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: Vec<String> = self.fretted_notes
            .iter()
            .map(|string| {
                match string.kind {
                    FrettedNoteKind::Muted => "x".to_string(),
                    FrettedNoteKind::Open(_) => "0".to_string(),
                    FrettedNoteKind::Fretted { fret, ..} => fret.to_string(),
                }
            })
            .collect();
        let s = s.join("-");
        write!(f, "{}", s)
    }
}

// TODO implement an easy constructor for Chord Shapes.
impl<'a> FretboardShape<'a> {

    /// Creates a new [FretboardShape] where all the open strings are converted to muted strings.
    /// This is useful for analyzing playability, if you consider open strings negligibly costly to play.
    pub fn without_open_strings(&'a self) -> Self {
        FretboardShape {
            fretboard: self.fretboard,
            fretted_notes: self.fretted_notes
                .iter()
                .map(|string| match string.kind {
                    FrettedNoteKind::Open(_) => FrettedNote {
                        kind: FrettedNoteKind::Muted,
                        string: string.string,
                        fretboard: &string.fretboard,
                    },
                    FrettedNoteKind::Muted => FrettedNote {
                        kind: FrettedNoteKind::Muted,
                        string: string.string,
                        fretboard: &string.fretboard,
                    },
                    FrettedNoteKind::Fretted {fret, pitch} => FrettedNote {
                        kind: FrettedNoteKind::Fretted { fret, pitch },
                        string: string.string,
                        fretboard: &string.fretboard,
                    }
                })
                .collect()
        }
    }

    pub fn is_playable(&self) -> bool {
        let (min, max) = self.span();
        let span = max - min;
        let size = self.size();
        if (size <= 3 && span > 4) || (size > 3 && span > 3) {
            return false;
        }
        true
    }

    /// Number of strings not muted.
    pub fn size(&self) -> u8 {
        self.fretted_notes.iter()
            .fold(0, |value, item| match item.kind {
                FrettedNoteKind::Fretted { .. } => value + 1,
                _ => value,
            })
    }

    pub fn range(&self) -> (Pitch, Pitch) {
        let mut pitches: Vec<Pitch> = self.fretted_notes
            .iter()
            .map(|p| match &p.kind {
                FrettedNoteKind::Fretted { pitch, .. } => Some(pitch.clone()),
                FrettedNoteKind::Open(string) => Some((**string).clone()),
                FrettedNoteKind::Muted => None,
            })
            .into_iter()
            .flatten()
            .collect();
        pitches.sort_by(|a, b| a.midi_note.partial_cmp(&b.midi_note).unwrap());
        (pitches.first().unwrap().clone(), pitches.last().unwrap().clone())
    }

    /// Minimum and maximum fret numbers, *including* open strings.
    pub fn span(&self) -> (u8, u8) {
        let mut lowest: u8 = u8::MAX;
        let mut highest: u8  = u8::MIN;
        for fretted_note in &self.fretted_notes {
            match &fretted_note.kind {
                FrettedNoteKind::Fretted { fret, .. } => {
                    if *fret < lowest {
                        lowest = *fret;
                    }
                    if *fret > highest {
                        highest = *fret;
                    }
                },
                FrettedNoteKind::Open(_) => {
                    lowest = 0;
                },
                _ => {}
            }
        }
        (lowest, highest)
    }

    pub fn contains_open_strings(&self) -> bool {
        self.fretted_notes.iter().any(|string| {
            match string.kind {
                FrettedNoteKind::Open(_) => true,
                _ => false,
            }
        })
    }

    /// Since frets are spelling-agnostic, we compare by MIDI note value
    /// to equivocate over accidentals but not octaves.
    pub fn contains(&self, member: &Pitch) -> bool {
        for fretted_note in &self.fretted_notes {
            match &fretted_note.kind {
                FrettedNoteKind::Fretted { pitch, .. } => {
                    if pitch.midi_note == member.midi_note {
                        return true;
                    }
                },
                FrettedNoteKind::Open(pitch) => {
                    if pitch.midi_note == member.midi_note {
                        return true;
                    }
                },
                _ => {}
            }
        }
        false
    }

    pub fn classify(&self) -> ChordShapeClassification {
        if self.is_playable() {
            if self.fretted_notes.iter().all(|string| {
                match string.kind {
                    FrettedNoteKind::Fretted { fret, .. } => fret > 12,
                    _ => true,
                }
            }) {
                return ChordShapeClassification::AllAbove12thFret;
            } else {
                return ChordShapeClassification::Playable;
            }
        }
        if self.contains_open_strings() {
            let without_open_strings = self.without_open_strings();
            if without_open_strings.is_playable() {
                return ChordShapeClassification::NonTransposable;
            }
        }
        ChordShapeClassification::Unplayable
    }
}

impl<'a> From<&'a FretboardShape<'a>> for StackedIntervals {
    fn from(value: &'a FretboardShape<'a>) -> Self {
        let mut pitches: Vec<Pitch> = value.fretted_notes.iter()
            .map(|fretted_note| match &fretted_note.kind {
                FrettedNoteKind::Fretted { pitch, .. } => {
                  return Some(pitch.clone());
                },
                FrettedNoteKind::Open(open_string) => {
                  return Some(open_string.0.clone())
                },
                _ => None,
            })
            .into_iter()
            .flatten()
            .collect();
        pitches.sort_by(|a, b| a.midi_note.partial_cmp(&b.midi_note).unwrap());
        let sorted_midi: Vec<u8> = pitches.iter().map(|p| p.midi_note).collect();
        let consecutive_intervals = zip(&sorted_midi, &sorted_midi[1..sorted_midi.len()])
            .map(|(a, b)| b - a)
            .collect();
        StackedIntervals(consecutive_intervals)
    }
}

#[derive(Debug, Clone)]
pub enum ChordShapeClassification {
    Playable,
    Unplayable,
    AllAbove12thFret,
    NonTransposable,
}

#[cfg(test)]
mod tests {
    use crate::fretboard::fretboard_shape::chord_shape_search::find_chord_shapes;
    use crate::fretboard::OpenString;
    use crate::pitch::Pitch;
    use super::*;

    #[test]
    fn finding_chord_shapes() {
        let chord = vec![Note::C, Note::E, Note::G, Note::B];
        let fretboard = Fretboard {
            open_strings: vec![
              OpenString(Pitch::new(Note::E, 3).unwrap()),
              OpenString(Pitch::new(Note::A, 3).unwrap()),
              OpenString(Pitch::new(Note::D, 4).unwrap()),
              OpenString(Pitch::new(Note::G, 4).unwrap()),
              OpenString(Pitch::new(Note::B, 4).unwrap()),
              OpenString(Pitch::new(Note::E, 5).unwrap()),
            ],
        };
        let shapes = find_chord_shapes(
            &chord,
            &fretboard
        ).unwrap();
        println!("VALID SHAPES:");
        for shape in shapes.playable {
            println!("{:?}", shape.0);
            println!("{}", shape.1.iter().map(|s| s.to_string()).collect::<Vec<String>>().join("\n"));
        }
    }
}