//! Melodic pattern generation and sequence construction.
//!
//! This module provides tools for generating melodies based on:
//! - Interval patterns (multi-level sequences)
//! - Chord progressions (harmonic context)
//! - Rhythm patterns
//! - Pitch boundaries and turnaround behavior
//!
//! # Example
//!
//! ```ignore
//! use music::melody::{
//!     MelodicSequencer, MelodicSequencerConfig,
//!     IntervalPattern, ChordProgression, TimedChord,
//!     Direction, TurnaroundMode, PitchBounds,
//! };
//!
//! // Create a simple ascending pattern
//! let pattern = IntervalPattern::new(vec![vec![1, 1], vec![2]], 1);
//!
//! // Generate melody...
//! ```

pub mod pattern;
pub mod context;
pub mod sequencer;

pub use pattern::{PatternLevel, IntervalPattern};
pub use context::{TimedChord, ChordProgression};
pub use sequencer::{MelodicSequencer, MelodicSequencerConfig, MelodicEvent};

use crate::error::MusicSemanticsError;
use crate::note::pitch::Pitch;

/// Direction of melodic movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    /// Flip direction to its opposite.
    pub fn flip(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    /// Get a multiplier for interval calculations.
    /// Up = 1, Down = -1.
    pub fn multiplier(&self) -> i8 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
        }
    }
}

/// Behavior when melody hits pitch boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnaroundMode {
    /// Reverse direction, move same number of steps from previous note.
    Reflect,
    /// Bounce off boundary: remaining steps go in opposite direction from boundary.
    Ricochet,
    /// Return to starting note when boundary is reached.
    StartOver,
    /// Stop at the boundary (clamp to boundary pitch).
    Stop,
    /// Wrap to opposite boundary (for circular/looping melodies).
    Wrap,
}

/// Configuration for pitch boundaries.
#[derive(Debug, Clone)]
pub struct PitchBounds {
    /// Lowest allowed pitch.
    pub lowest: Pitch,
    /// Highest allowed pitch.
    pub highest: Pitch,
}

impl PitchBounds {
    /// Create new pitch bounds. Lowest must be strictly lower than highest.
    pub fn new(lowest: Pitch, highest: Pitch) -> Result<Self, MusicSemanticsError> {
        if lowest.midi_note >= highest.midi_note {
            return Err(MusicSemanticsError::InvalidPitchBounds);
        }
        Ok(Self { lowest, highest })
    }

    /// Check if a pitch is within bounds (inclusive).
    pub fn contains(&self, pitch: &Pitch) -> bool {
        pitch.midi_note >= self.lowest.midi_note
            && pitch.midi_note <= self.highest.midi_note
    }

    /// Get the span in semitones between lowest and highest.
    pub fn span(&self) -> u8 {
        self.highest.midi_note - self.lowest.midi_note
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::note::Note;

    #[test]
    fn test_direction_flip() {
        assert_eq!(Direction::Up.flip(), Direction::Down);
        assert_eq!(Direction::Down.flip(), Direction::Up);
    }

    #[test]
    fn test_direction_multiplier() {
        assert_eq!(Direction::Up.multiplier(), 1);
        assert_eq!(Direction::Down.multiplier(), -1);
    }

    #[test]
    fn test_pitch_bounds_valid() {
        let low = Pitch::new(Note::C, 3).unwrap();
        let high = Pitch::new(Note::C, 5).unwrap();
        let bounds = PitchBounds::new(low, high).unwrap();
        assert_eq!(bounds.span(), 24); // 2 octaves
    }

    #[test]
    fn test_pitch_bounds_invalid() {
        let high = Pitch::new(Note::C, 5).unwrap();
        let low = Pitch::new(Note::C, 3).unwrap();
        // Reversed order should fail
        let result = PitchBounds::new(high, low);
        assert!(result.is_err());
    }

    #[test]
    fn test_pitch_bounds_contains() {
        let low = Pitch::new(Note::C, 3).unwrap();
        let high = Pitch::new(Note::C, 5).unwrap();
        let bounds = PitchBounds::new(low, high).unwrap();

        let middle = Pitch::new(Note::C, 4).unwrap();
        assert!(bounds.contains(&middle));

        let below = Pitch::new(Note::C, 2).unwrap();
        assert!(!bounds.contains(&below));

        let above = Pitch::new(Note::C, 6).unwrap();
        assert!(!bounds.contains(&above));

        // Boundaries are inclusive
        let at_low = Pitch::new(Note::C, 3).unwrap();
        let at_high = Pitch::new(Note::C, 5).unwrap();
        assert!(bounds.contains(&at_low));
        assert!(bounds.contains(&at_high));
    }
}
