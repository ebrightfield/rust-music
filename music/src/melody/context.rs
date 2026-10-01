//! Harmonic context for melodic sequence generation.
//!
//! This module provides chord progression tracking with timing,
//! allowing melodies to be generated with awareness of the current
//! harmonic context.

use crate::notation::rhythm::duration::Duration;
use crate::note_collections::NoteSet;

/// A chord with its duration in the progression.
#[derive(Debug, Clone)]
pub struct TimedChord {
    /// The notes that make up this chord (or scale).
    pub notes: NoteSet,
    /// How long this chord lasts.
    pub duration: Duration,
}

impl TimedChord {
    /// Create a new timed chord.
    pub fn new(notes: NoteSet, duration: Duration) -> Self {
        Self { notes, duration }
    }

    /// Get the duration in ticks.
    pub fn ticks(&self) -> usize {
        self.duration.ticks()
    }
}

/// Chord progression with timing and position tracking.
///
/// Tracks the current position within a chord progression,
/// automatically advancing to the next chord when enough
/// duration has elapsed.
#[derive(Debug, Clone)]
pub struct ChordProgression {
    chords: Vec<TimedChord>,
    position: usize,
    tick_in_chord: usize,
}

impl ChordProgression {
    /// Create a new chord progression.
    ///
    /// # Panics
    ///
    /// Panics if the chord list is empty.
    pub fn new(chords: Vec<TimedChord>) -> Self {
        assert!(!chords.is_empty(), "Chord progression cannot be empty");
        Self {
            chords,
            position: 0,
            tick_in_chord: 0,
        }
    }

    /// Create a static progression (single chord that never changes).
    pub fn static_chord(notes: NoteSet) -> Self {
        Self::new(vec![TimedChord::new(
            notes,
            Duration::new(crate::notation::rhythm::duration::DurationKind::Breve, 0),
        )])
    }

    /// Get the current chord.
    pub fn current(&self) -> &TimedChord {
        &self.chords[self.position]
    }

    /// Get the current chord's notes.
    pub fn current_notes(&self) -> &NoteSet {
        &self.current().notes
    }

    /// Advance by a duration, returning the (possibly new) current chord.
    ///
    /// If the duration extends past the current chord, this will
    /// advance to subsequent chords (wrapping at the end).
    pub fn advance(&mut self, duration: &Duration) -> &TimedChord {
        self.tick_in_chord += duration.ticks();

        while self.tick_in_chord >= self.current().ticks() {
            self.tick_in_chord -= self.current().ticks();
            self.position = (self.position + 1) % self.chords.len();
        }

        self.current()
    }

    /// Advance by a specific number of ticks.
    pub fn advance_ticks(&mut self, ticks: usize) -> &TimedChord {
        self.tick_in_chord += ticks;

        while self.tick_in_chord >= self.current().ticks() {
            self.tick_in_chord -= self.current().ticks();
            self.position = (self.position + 1) % self.chords.len();
        }

        self.current()
    }

    /// Reset to the beginning of the progression.
    pub fn reset(&mut self) {
        self.position = 0;
        self.tick_in_chord = 0;
    }

    /// Get the current position index.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get how many ticks into the current chord we are.
    pub fn tick_in_chord(&self) -> usize {
        self.tick_in_chord
    }

    /// Get the total number of chords in the progression.
    pub fn len(&self) -> usize {
        self.chords.len()
    }

    /// Check if the progression is empty (always false after construction).
    pub fn is_empty(&self) -> bool {
        self.chords.is_empty()
    }

    /// Get the total duration of the progression in ticks.
    pub fn total_ticks(&self) -> usize {
        self.chords.iter().map(|c| c.ticks()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::rhythm::duration::DurationKind;
    use crate::note::note::Note;

    fn c_major() -> NoteSet {
        NoteSet::new(vec![Note::C, Note::E, Note::G])
    }

    fn g_major() -> NoteSet {
        NoteSet::new(vec![Note::G, Note::B, Note::D])
    }

    fn a_minor() -> NoteSet {
        NoteSet::new(vec![Note::A, Note::C, Note::E])
    }

    #[test]
    fn test_timed_chord() {
        let chord = TimedChord::new(c_major(), Duration::QTR);
        assert_eq!(chord.ticks(), 32);
    }

    #[test]
    fn test_static_progression() {
        let prog = ChordProgression::static_chord(c_major());
        assert_eq!(prog.len(), 1);
        assert_eq!(prog.current_notes().len(), 3);
    }

    #[test]
    fn test_progression_advance() {
        let mut prog = ChordProgression::new(vec![
            TimedChord::new(c_major(), Duration::HALF), // 64 ticks
            TimedChord::new(g_major(), Duration::HALF), // 64 ticks
        ]);

        // Start at C major
        assert_eq!(prog.position(), 0);

        // Advance by quarter note (32 ticks) - still on C
        prog.advance(&Duration::QTR);
        assert_eq!(prog.position(), 0);
        assert_eq!(prog.tick_in_chord(), 32);

        // Advance another quarter - now on G
        prog.advance(&Duration::QTR);
        assert_eq!(prog.position(), 1);
        assert_eq!(prog.tick_in_chord(), 0);

        // Advance half - back to C (wrap)
        prog.advance(&Duration::HALF);
        assert_eq!(prog.position(), 0);
        assert_eq!(prog.tick_in_chord(), 0);
    }

    #[test]
    fn test_progression_longer_duration() {
        let mut prog = ChordProgression::new(vec![
            TimedChord::new(c_major(), Duration::QTR), // 32 ticks
            TimedChord::new(g_major(), Duration::QTR), // 32 ticks
            TimedChord::new(a_minor(), Duration::QTR), // 32 ticks
        ]);

        // Advance by whole note (128 ticks) - should wrap around
        // Total cycle = 96 ticks
        // 128 ticks = 96 (one full cycle) + 32 (one more chord)
        prog.advance(&Duration::WHOLE);
        // After 96 ticks: back at chord 0
        // After 32 more: exactly finishes chord 0, now at chord 1
        assert_eq!(prog.position(), 1); // On G major
        assert_eq!(prog.tick_in_chord(), 0); // Exactly at start of chord 1
    }

    #[test]
    fn test_progression_reset() {
        let mut prog = ChordProgression::new(vec![
            TimedChord::new(c_major(), Duration::HALF),
            TimedChord::new(g_major(), Duration::HALF),
        ]);

        prog.advance(&Duration::new(DurationKind::Whole, 0));
        prog.reset();

        assert_eq!(prog.position(), 0);
        assert_eq!(prog.tick_in_chord(), 0);
    }

    #[test]
    fn test_total_ticks() {
        let prog = ChordProgression::new(vec![
            TimedChord::new(c_major(), Duration::HALF), // 64 ticks
            TimedChord::new(g_major(), Duration::QTR),  // 32 ticks
            TimedChord::new(a_minor(), Duration::QTR),  // 32 ticks
        ]);

        assert_eq!(prog.total_ticks(), 128);
    }
}
