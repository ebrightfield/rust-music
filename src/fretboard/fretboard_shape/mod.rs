pub mod chord_shape_search;
pub mod melodic_shape_search;

use std::fmt::{Display, Formatter};
use std::iter::zip;
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::StackedIntervals;
use crate::fretboard::Fretboard;
use crate::fretboard::fretted_note::{FrettedNote, SoundedNote};

#[derive(Debug)]
pub struct FretboardShape<'a> {
    pub fretted_notes: Vec<FrettedNote<'a>>,
    pub fretboard: &'a Fretboard,
}

impl<'a> Display for FretboardShape<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: Vec<String> = self.fretted_notes
            .iter()
            .map(|value| {
                match value {
                    FrettedNote::Muted { .. } => "x".to_string(),
                    FrettedNote::Sounded(SoundedNote { fret, .. }) => fret.to_string(),
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
        Self {
            fretboard: self.fretboard,
            fretted_notes: self.fretted_notes
                .iter()
                .map(|value| match &value {
                    FrettedNote::Muted { string, fretboard } => FrettedNote::Muted {
                        string: *string, fretboard,
                    },
                    FrettedNote::Sounded(
                        SoundedNote { fret: 0, string, fretboard, .. }
                    ) => FrettedNote::Muted {
                        string: *string, fretboard,
                    },
                    FrettedNote::Sounded(
                        SoundedNote { fret, pitch, string, fretboard }
                    ) => FrettedNote::Sounded(SoundedNote {
                        fret: *fret, pitch: pitch.clone(), string: *string, fretboard,
                    }),
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
            .fold(0, |value, item| match item {
                FrettedNote::Sounded(_) => value + 1,
                _ => value,
            })
    }

    pub fn range(&self) -> (Pitch, Pitch) {
        let mut pitches: Vec<Pitch> = self.fretted_notes
            .iter()
            .map(|p| match &p {
                FrettedNote::Sounded(SoundedNote { pitch, .. }) => Some(pitch.clone()),
                FrettedNote::Muted { .. } => None,
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
            if let FrettedNote::Sounded(SoundedNote { fret, .. }) = fretted_note {
                if *fret < lowest {
                    lowest = *fret;
                }
                if *fret > highest {
                    highest = *fret;
                }
            }
        }
        (lowest, highest)
    }

    pub fn contains_open_strings(&self) -> bool {
        self.fretted_notes.iter().any(|value| {
            match value {
                FrettedNote::Sounded(SoundedNote {fret: 0, ..}) => true,
                _ => false,
            }
        })
    }

    /// Since frets are spelling-agnostic, we compare by MIDI note value
    /// to equivocate over accidentals but not octaves.
    pub fn contains(&self, member: &Pitch) -> bool {
        for fretted_note in &self.fretted_notes {
            if let FrettedNote::Sounded(SoundedNote { pitch, ..}) = fretted_note {
                if pitch.midi_note == member.midi_note {
                    return true;
                }
            }
        }
        false
    }

    pub fn classify(&self) -> ChordShapeClassification {
        if self.is_playable() {
            if self.fretted_notes.iter().all(|value| {
                match value {
                    FrettedNote::Sounded(SoundedNote { fret, .. }) => *fret > 12,
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
        let mut pitches: Vec<Pitch> = value
            .fretted_notes
            .iter()
            .map(|fretted_note| match &fretted_note {
                FrettedNote::Sounded(SoundedNote { pitch, .. }) => {
                    return Some(pitch.clone());
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
}
