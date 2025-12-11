pub mod chord_shape_search;
pub mod melodic_shape_search;

use std::fmt::{Display, Formatter};
use std::iter::zip;
use std::ops::Deref;
use crate::error::MusicSemanticsError;
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::{StackedIntervals, Voicing};
use crate::fretboard::Fretboard;
use crate::fretboard::fretted_note::{FrettedNote, SoundedNote};
use crate::note::note::Note;

/// Meant for vertically oriented fretboard shapes.
#[derive(Debug, Clone)]
pub struct FretboardShape<'a> {
    pub fretted_notes: Vec<FrettedNote<'a>>,
    pub fretboard: &'a Fretboard,
}

impl<'a> Deref for FretboardShape<'a> {
    type Target = Vec<FrettedNote<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.fretted_notes
    }
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

impl<'a> FretboardShape<'a> {
    /// Create a [FretboardShape] from a slice of fret positions.
    /// Use `None` for muted strings, `Some(fret)` for fretted/open strings.
    ///
    /// # Example
    /// ```
    /// use music::fretboard::{Fretboard, STD_6STR_GTR};
    /// use music::fretboard::fretboard_shape::FretboardShape;
    ///
    /// // Open C chord: x-3-2-0-1-0
    /// let c_chord = FretboardShape::from_frets(
    ///     &[None, Some(3), Some(2), Some(0), Some(1), Some(0)],
    ///     &STD_6STR_GTR
    /// ).unwrap();
    /// ```
    pub fn from_frets(
        frets: &[Option<u8>],
        fretboard: &'a Fretboard,
    ) -> Result<Self, MusicSemanticsError> {
        let fretted_notes: Result<Vec<FrettedNote>, MusicSemanticsError> = frets
            .iter()
            .enumerate()
            .map(|(string, fret)| {
                match fret {
                    Some(f) => FrettedNote::fretted(string as u8, *f, fretboard),
                    None => FrettedNote::muted(string as u8, fretboard),
                }
            })
            .collect();
        Ok(Self {
            fretted_notes: fretted_notes?,
            fretboard,
        })
    }

    /// Create a [FretboardShape] from a string notation like "x-3-2-0-1-0".
    /// Supports:
    /// - `x` or `X` for muted strings
    /// - Numbers for fret positions
    ///
    /// # Example
    /// ```
    /// use music::fretboard::{Fretboard, STD_6STR_GTR};
    /// use music::fretboard::fretboard_shape::FretboardShape;
    ///
    /// let c_chord = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
    /// let g_chord = FretboardShape::from_string("3-2-0-0-0-3", &STD_6STR_GTR).unwrap();
    /// ```
    pub fn from_string(
        shape_str: &str,
        fretboard: &'a Fretboard,
    ) -> Result<Self, MusicSemanticsError> {
        let parts: Vec<&str> = shape_str.split('-').collect();
        let frets: Result<Vec<Option<u8>>, MusicSemanticsError> = parts
            .iter()
            .map(|part| {
                let trimmed = part.trim();
                if trimmed.eq_ignore_ascii_case("x") {
                    Ok(None)
                } else {
                    trimmed
                        .parse::<u8>()
                        .map(Some)
                        .map_err(|_| MusicSemanticsError::InvalidFretNotation(trimmed.to_string()))
                }
            })
            .collect();
        Self::from_frets(&frets?, fretboard)
    }

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

    pub fn spelled_as_in(&self, notes: &Vec<Note>) -> Result<Self, MusicSemanticsError> {
        Ok(Self {
            fretboard: self.fretboard,
            fretted_notes: self.fretted_notes
                .iter()
                .map(|value| Ok::<_, MusicSemanticsError>(value.spelled_as_in(notes)?))
                .into_iter()
                .flatten()
                .collect()
        })
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

impl<'a> From<&'a FretboardShape<'a>> for Voicing {
    fn from(value: &'a FretboardShape<'a>) -> Self {
        Voicing::new(
            value.fretted_notes.iter()
                .filter(|item| item.is_sounded())
                .map(|item| item.pitch().unwrap().clone())
                .collect()
        )
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
    use super::*;
    use crate::fretboard::STD_6STR_GTR;

    #[test]
    fn test_fretboard_shape_creation() {
        // Create an open C chord shape: x-3-2-0-1-0
        let shape = FretboardShape::from_frets(
            &[None, Some(3), Some(2), Some(0), Some(1), Some(0)],
            &STD_6STR_GTR,
        ).unwrap();
        assert_eq!(shape.len(), 6);
        assert_eq!(shape.size(), 5); // 5 sounded strings (one muted)
    }

    #[test]
    fn test_from_string_constructor() {
        // Open C chord: x-3-2-0-1-0
        let c_chord = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert_eq!(c_chord.size(), 5);
        assert_eq!(format!("{}", c_chord), "x-3-2-0-1-0");

        // Open G chord: 3-2-0-0-0-3
        let g_chord = FretboardShape::from_string("3-2-0-0-0-3", &STD_6STR_GTR).unwrap();
        assert_eq!(g_chord.size(), 6);
        assert!(g_chord.contains_open_strings());

        // Uppercase X should also work
        let d_chord = FretboardShape::from_string("X-X-0-2-3-2", &STD_6STR_GTR).unwrap();
        assert_eq!(d_chord.size(), 4);
    }

    #[test]
    fn test_from_string_invalid_input() {
        // Invalid character
        let result = FretboardShape::from_string("x-3-abc-0-1-0", &STD_6STR_GTR);
        assert!(result.is_err());

        // Fret too high (above Fretboard::MAX)
        let result = FretboardShape::from_string("x-3-40-0-1-0", &STD_6STR_GTR);
        assert!(result.is_err());
    }

    #[test]
    fn test_playability_check() {
        // Playable open C chord: x-3-2-0-1-0
        let c_chord = FretboardShape::from_frets(
            &[None, Some(3), Some(2), Some(0), Some(1), Some(0)],
            &STD_6STR_GTR,
        ).unwrap();
        assert!(c_chord.is_playable(), "Open C chord should be playable");

        // Playable barre chord: 1-3-3-2-1-1 (F major barre)
        let f_barre = FretboardShape::from_frets(
            &[Some(1), Some(3), Some(3), Some(2), Some(1), Some(1)],
            &STD_6STR_GTR,
        ).unwrap();
        assert!(f_barre.is_playable(), "F barre chord should be playable");

        // Unplayable: huge span across frets
        let unplayable = FretboardShape::from_frets(
            &[Some(1), Some(10), Some(3), Some(12), Some(1), Some(1)],
            &STD_6STR_GTR,
        ).unwrap();
        assert!(!unplayable.is_playable(), "Large span should be unplayable");
    }

    #[test]
    fn test_span_calculation() {
        // Open G chord: 3-2-0-0-0-3
        let g_chord = FretboardShape::from_string("3-2-0-0-0-3", &STD_6STR_GTR).unwrap();
        let (min, max) = g_chord.span();
        assert_eq!(min, 0, "Minimum fret should be 0 (open string)");
        assert_eq!(max, 3, "Maximum fret should be 3");

        // Barre chord at 5th fret: 5-7-7-6-5-5
        let a_barre = FretboardShape::from_string("5-7-7-6-5-5", &STD_6STR_GTR).unwrap();
        let (min, max) = a_barre.span();
        assert_eq!(min, 5);
        assert_eq!(max, 7);
    }

    #[test]
    fn test_contains_open_strings() {
        // Open D chord: x-x-0-2-3-2
        let d_open = FretboardShape::from_string("x-x-0-2-3-2", &STD_6STR_GTR).unwrap();
        assert!(d_open.contains_open_strings());

        // Barre chord with no open strings: 5-7-7-6-5-5
        let barre = FretboardShape::from_string("5-7-7-6-5-5", &STD_6STR_GTR).unwrap();
        assert!(!barre.contains_open_strings());
    }

    #[test]
    fn test_without_open_strings() {
        // Open G chord: 3-2-0-0-0-3
        let g_chord = FretboardShape::from_string("3-2-0-0-0-3", &STD_6STR_GTR).unwrap();
        let without_open = g_chord.without_open_strings();

        // Should have only 3 sounded notes now (frets 3, 2, and 3 on first, second, and sixth strings)
        assert_eq!(without_open.size(), 3);
        assert!(!without_open.contains_open_strings());
    }

    #[test]
    fn test_chord_shape_classification() {
        // Playable open chord
        let c_chord = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        assert!(matches!(c_chord.classify(), ChordShapeClassification::Playable));

        // Open E chord: 0-2-2-1-0-0
        let e_chord = FretboardShape::from_string("0-2-2-1-0-0", &STD_6STR_GTR).unwrap();
        assert!(matches!(e_chord.classify(), ChordShapeClassification::Playable));
    }

    #[test]
    fn test_range_calculation() {
        // Simple three-note shape across strings
        let shape = FretboardShape::from_frets(
            &[None, None, None, Some(5), Some(5), Some(5)],
            &STD_6STR_GTR,
        ).unwrap();
        let (low, high) = shape.range();
        assert!(low.midi_note < high.midi_note, "Low pitch should be lower than high pitch");
    }

    #[test]
    fn test_display_format() {
        // Open C chord: x-3-2-0-1-0
        let c_chord = FretboardShape::from_string("x-3-2-0-1-0", &STD_6STR_GTR).unwrap();
        let display = format!("{}", c_chord);
        assert_eq!(display, "x-3-2-0-1-0");

        // Barre chord: 5-7-7-6-5-5
        let barre = FretboardShape::from_string("5-7-7-6-5-5", &STD_6STR_GTR).unwrap();
        let display = format!("{}", barre);
        assert_eq!(display, "5-7-7-6-5-5");
    }

    #[test]
    fn test_stacked_intervals_from_shape() {
        // Power chord: root and fifth (e.g., 5-7-7-x-x-x on low strings)
        let power_chord = FretboardShape::from_string("5-7-7-x-x-x", &STD_6STR_GTR).unwrap();
        let intervals: StackedIntervals = StackedIntervals::from(&power_chord);
        // A power chord is root-fifth-octave: intervals of 7 and 5 semitones
        assert_eq!(intervals.0.len(), 2);
        assert_eq!(intervals.0[0], 7); // root to fifth
        assert_eq!(intervals.0[1], 5); // fifth to octave
    }

    #[test]
    fn test_roundtrip_from_string_to_display() {
        // Test that from_string + Display gives back the same string
        let shapes = [
            "x-3-2-0-1-0",
            "3-2-0-0-0-3",
            "0-2-2-1-0-0",
            "5-7-7-6-5-5",
            "x-x-0-2-3-2",
        ];
        for s in shapes {
            let shape = FretboardShape::from_string(s, &STD_6STR_GTR).unwrap();
            assert_eq!(format!("{}", shape), s);
        }
    }
}
