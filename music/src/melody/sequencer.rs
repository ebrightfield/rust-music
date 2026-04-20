//! Melodic sequence generation based on patterns and harmonic context.
//!
//! This module provides the main `MelodicSequencer` struct which generates
//! melodies based on:
//! - Multi-level interval patterns
//! - Chord progressions
//! - Rhythm patterns
//! - Pitch boundaries with configurable turnaround behavior

use super::{Direction, TurnaroundMode, PitchBounds};
use super::pattern::IntervalPattern;
use super::context::ChordProgression;
use crate::error::MusicSemanticsError;
use crate::note::pitch::Pitch;
use crate::note_collections::NoteSet;
use crate::notation::rhythm::duration::Duration;

/// A single melodic event with pitch and duration.
#[derive(Debug, Clone)]
pub struct MelodicEvent {
    /// The pitch of this event.
    pub pitch: Pitch,
    /// The duration of this event.
    pub duration: Duration,
    /// Whether this note is tied to the previous note.
    pub tied: bool,
}

impl MelodicEvent {
    /// Create a new melodic event.
    pub fn new(pitch: Pitch, duration: Duration) -> Self {
        Self {
            pitch,
            duration,
            tied: false,
        }
    }

    /// Create a tied melodic event.
    pub fn tied(pitch: Pitch, duration: Duration) -> Self {
        Self {
            pitch,
            duration,
            tied: true,
        }
    }
}

/// Configuration for the melodic sequencer.
#[derive(Debug, Clone)]
pub struct MelodicSequencerConfig {
    /// The chord progression providing harmonic context.
    pub chord_progression: ChordProgression,
    /// The interval pattern for melodic movement.
    pub interval_pattern: IntervalPattern,
    /// The rhythm pattern to cycle through.
    pub rhythm_pattern: Vec<Duration>,
    /// Pitch boundaries for the melody.
    pub bounds: PitchBounds,
    /// The starting pitch.
    pub starting_pitch: Pitch,
    /// Initial direction of melodic movement.
    pub direction: Direction,
    /// Behavior when hitting pitch boundaries.
    pub turnaround_mode: TurnaroundMode,
    /// Maximum number of events to generate.
    pub max_length: usize,
}

/// Melodic sequence generator.
///
/// Generates melodies by stepping through an interval pattern,
/// constrained by chord progressions and pitch boundaries.
///
/// # Example
///
/// ```ignore
/// use music::melody::*;
/// use music::NoteSet;
/// use music::note::note::Note;
/// use music::note::pitch::Pitch;
/// use music::notation::rhythm::Duration;
///
/// // Create a C major scale
/// let scale = NoteSet::new(
///     vec![Note::C, Note::D, Note::E, Note::F, Note::G, Note::A, Note::B],
///     None,
/// );
///
/// // Simple ascending pattern: step, step, skip
/// let pattern = IntervalPattern::new(vec![vec![1, 1], vec![2]], 1);
///
/// // Create sequencer config
/// let config = MelodicSequencerConfig {
///     chord_progression: ChordProgression::static_chord(scale),
///     interval_pattern: pattern,
///     rhythm_pattern: vec![Duration::EIGHTH],
///     bounds: PitchBounds::try_new(
///         Pitch::new(Note::C, 3),
///         Pitch::new(Note::C, 6),
///     ).unwrap(),
///     starting_pitch: Pitch::new(Note::C, 4),
///     direction: Direction::Up,
///     turnaround_mode: TurnaroundMode::Reflect,
///     max_length: 16,
/// };
///
/// let mut sequencer = MelodicSequencer::new(config);
/// let melody = sequencer.generate().unwrap();
/// ```
pub struct MelodicSequencer {
    config: MelodicSequencerConfig,
    /// Current direction (may change due to turnaround)
    direction: Direction,
    /// Current position in rhythm pattern
    rhythm_position: usize,
    /// Last note at each pattern level (for multi-level patterns)
    level_notes: Vec<Pitch>,
}

impl MelodicSequencer {
    /// Create a new melodic sequencer from configuration.
    pub fn new(config: MelodicSequencerConfig) -> Self {
        let num_levels = config.interval_pattern.num_levels() + 1;
        let level_notes = vec![config.starting_pitch.clone(); num_levels];

        Self {
            direction: config.direction,
            rhythm_position: 0,
            level_notes,
            config,
        }
    }

    /// Generate the complete melody.
    pub fn generate(&mut self) -> Result<Vec<MelodicEvent>, MusicSemanticsError> {
        let mut melody = Vec::with_capacity(self.config.max_length);

        // First note
        let first_rhythm = self.next_rhythm();
        melody.push(MelodicEvent::new(
            self.config.starting_pitch.clone(),
            first_rhythm,
        ));

        // Generate remaining events
        while melody.len() < self.config.max_length {
            let event = self.next_event()?;
            melody.push(event);
        }

        Ok(melody)
    }

    /// Generate events as an iterator.
    pub fn iter(&mut self) -> MelodicSequenceIter<'_> {
        MelodicSequenceIter {
            sequencer: self,
            count: 0,
            first: true,
        }
    }

    /// Get the next rhythm duration from the pattern.
    fn next_rhythm(&mut self) -> Duration {
        let rhythm = self.config.rhythm_pattern[self.rhythm_position];
        self.rhythm_position = (self.rhythm_position + 1)
            % self.config.rhythm_pattern.len();
        rhythm
    }

    /// Generate the next melodic event.
    fn next_event(&mut self) -> Result<MelodicEvent, MusicSemanticsError> {
        let duration = self.next_rhythm();

        // Advance chord progression
        let chord = self.config.chord_progression.advance(&duration);
        let chord_notes = chord.notes.clone();

        // Get next interval from pattern
        let (interval, level) = self.config.interval_pattern.next_interval();
        let directed_interval = interval * self.direction.multiplier();

        // Get the previous pitch for this level (clone to avoid borrow issues)
        // For the master step (level >= pattern's num_levels), always use the most recent pitch (level 0)
        let pattern_num_levels = self.config.interval_pattern.num_levels();
        let level_for_pitch = if level >= pattern_num_levels {
            0
        } else {
            level
        };
        let previous_pitch = self.level_notes[level_for_pitch].clone();

        // Calculate next pitch
        let next_pitch = self.calculate_next_pitch(
            &previous_pitch,
            directed_interval,
            &chord_notes,
        )?;

        // Apply boundary logic
        let final_pitch = self.apply_boundary_logic(
            &previous_pitch,
            next_pitch,
            directed_interval,
            &chord_notes,
        )?;

        // Update level notes: update this level and all lower levels
        // For master step, update all levels
        let level_index = if level >= pattern_num_levels {
            self.level_notes.len() - 1
        } else {
            level
        };
        for i in 0..=level_index {
            self.level_notes[i] = final_pitch.clone();
        }

        Ok(MelodicEvent::new(final_pitch, duration))
    }

    /// Calculate the next pitch by stepping through the chord/scale.
    fn calculate_next_pitch(
        &self,
        from: &Pitch,
        steps: i8,
        chord: &NoteSet,
    ) -> Result<Pitch, MusicSemanticsError> {
        chord.pitch_n_steps_from(from, steps)
    }

    /// Apply boundary handling logic.
    fn apply_boundary_logic(
        &mut self,
        previous: &Pitch,
        next: Pitch,
        steps: i8,
        chord: &NoteSet,
    ) -> Result<Pitch, MusicSemanticsError> {
        // In bounds - return as-is
        if self.config.bounds.contains(&next) {
            return Ok(next);
        }

        match self.config.turnaround_mode {
            TurnaroundMode::Stop => {
                // Clamp to boundary
                if next.midi_note < self.config.bounds.lowest.midi_note {
                    Ok(self.config.bounds.lowest.clone())
                } else {
                    Ok(self.config.bounds.highest.clone())
                }
            }

            TurnaroundMode::StartOver => {
                Ok(self.config.starting_pitch.clone())
            }

            TurnaroundMode::Reflect => {
                self.direction = self.direction.flip();
                let reflected_steps = -steps;
                let result = self.calculate_next_pitch(previous, reflected_steps, chord)?;

                if !self.config.bounds.contains(&result) {
                    // If still out of bounds, just clamp
                    if result.midi_note < self.config.bounds.lowest.midi_note {
                        Ok(self.config.bounds.lowest.clone())
                    } else {
                        Ok(self.config.bounds.highest.clone())
                    }
                } else {
                    Ok(result)
                }
            }

            TurnaroundMode::Ricochet => {
                self.apply_ricochet(previous, &next, chord)
            }

            TurnaroundMode::Wrap => {
                // Move to opposite boundary
                if next.midi_note < self.config.bounds.lowest.midi_note {
                    Ok(self.config.bounds.highest.clone())
                } else {
                    Ok(self.config.bounds.lowest.clone())
                }
            }
        }
    }

    /// Apply ricochet boundary handling (bounce off boundary).
    fn apply_ricochet(
        &mut self,
        previous: &Pitch,
        attempted: &Pitch,
        chord: &NoteSet,
    ) -> Result<Pitch, MusicSemanticsError> {
        self.direction = self.direction.flip();

        // Determine which boundary we hit
        let boundary = if attempted.midi_note < self.config.bounds.lowest.midi_note {
            &self.config.bounds.lowest
        } else {
            &self.config.bounds.highest
        };

        // Find closest chord tone to boundary
        let boundary_tone = chord.closest_to(boundary)?;

        // Calculate overshoot
        let total_distance = (attempted.midi_note as i16 - previous.midi_note as i16).abs();
        let to_boundary = (boundary.midi_note as i16 - previous.midi_note as i16).abs();
        let overshoot = total_distance.saturating_sub(to_boundary);

        // Create pitch at boundary tone
        let boundary_pitch = Pitch::try_new(boundary_tone.clone(), boundary.octave)?;

        // Move from boundary by overshoot amount in new direction
        // Estimate steps based on average interval size (assume ~2 semitones per step)
        let bounce_steps = ((overshoot / 2).max(1) as i8) * self.direction.multiplier();
        let result = chord.pitch_n_steps_from(&boundary_pitch, bounce_steps)?;

        if self.config.bounds.contains(&result) {
            Ok(result)
        } else {
            // If still out of bounds after bounce, clamp to boundary
            Ok(boundary.clone())
        }
    }

    /// Reset the sequencer to its initial state.
    pub fn reset(&mut self) {
        self.direction = self.config.direction;
        self.rhythm_position = 0;
        let num_levels = self.config.interval_pattern.num_levels() + 1;
        self.level_notes = vec![self.config.starting_pitch.clone(); num_levels];
        self.config.interval_pattern.reset();
        self.config.chord_progression.reset();
    }
}

/// Iterator over melodic events.
pub struct MelodicSequenceIter<'a> {
    sequencer: &'a mut MelodicSequencer,
    count: usize,
    first: bool,
}

impl<'a> Iterator for MelodicSequenceIter<'a> {
    type Item = Result<MelodicEvent, MusicSemanticsError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.sequencer.config.max_length {
            return None;
        }

        self.count += 1;

        if self.first {
            self.first = false;
            let rhythm = self.sequencer.next_rhythm();
            return Some(Ok(MelodicEvent::new(
                self.sequencer.config.starting_pitch.clone(),
                rhythm,
            )));
        }

        Some(self.sequencer.next_event())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::note::Note;

    fn c_major_scale() -> NoteSet {
        NoteSet::new(vec![Note::C, Note::D, Note::E, Note::F, Note::G, Note::A, Note::B])
    }

    fn simple_config() -> MelodicSequencerConfig {
        MelodicSequencerConfig {
            chord_progression: ChordProgression::static_chord(c_major_scale()),
            interval_pattern: IntervalPattern::simple(vec![1], 1),
            rhythm_pattern: vec![Duration::EIGHTH],
            bounds: PitchBounds::try_new(
                Pitch::new(Note::C, 3),
                Pitch::new(Note::C, 6),
            ).unwrap(),
            starting_pitch: Pitch::new(Note::C, 4),
            direction: Direction::Up,
            turnaround_mode: TurnaroundMode::Reflect,
            max_length: 8,
        }
    }

    #[test]
    fn test_simple_ascending() {
        let config = simple_config();
        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        assert_eq!(melody.len(), 8);

        // First note is starting pitch
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[0].pitch.octave, 4);

        // Second note should be D4
        assert_eq!(melody[1].pitch.note, Note::D);
        assert_eq!(melody[1].pitch.octave, 4);

        // Continue ascending
        assert_eq!(melody[2].pitch.note, Note::E);
        assert_eq!(melody[3].pitch.note, Note::F);
    }

    #[test]
    fn test_descending() {
        let mut config = simple_config();
        config.direction = Direction::Down;

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // First note is starting pitch C4
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[0].pitch.octave, 4);

        // Second note should be B3
        assert_eq!(melody[1].pitch.note, Note::B);
        assert_eq!(melody[1].pitch.octave, 3);
    }

    #[test]
    fn test_step_skip_pattern() {
        let mut config = simple_config();
        // Pattern: [[1, 1], [2]] - level 0 steps, level 1 skips
        // Multi-level patterns track each level independently
        config.interval_pattern = IntervalPattern::new(
            vec![vec![1, 1], vec![2]],
            1,
        );
        config.max_length = 6;

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // C4 (start)
        // D4 (level 0 step from C4)
        // E4 (level 0 step from D4)
        // E4 (level 1 skip from C4, its remembered position - 2 steps = E)
        // F4 (level 0 step from E4, level 1 also updates to E4)
        // G4 (level 0 step from F4)
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[1].pitch.note, Note::D);
        assert_eq!(melody[2].pitch.note, Note::E);
        assert_eq!(melody[3].pitch.note, Note::E); // Level 1 skip from C4
        assert_eq!(melody[4].pitch.note, Note::F);
        assert_eq!(melody[5].pitch.note, Note::G);
    }

    #[test]
    fn test_boundary_stop() {
        let mut config = simple_config();
        config.bounds = PitchBounds::try_new(
            Pitch::new(Note::C, 4),
            Pitch::new(Note::E, 4),
        ).unwrap();
        config.turnaround_mode = TurnaroundMode::Stop;
        config.max_length = 6;

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // Should stop at E4
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[1].pitch.note, Note::D);
        assert_eq!(melody[2].pitch.note, Note::E);
        // Further notes should stay at E
        assert_eq!(melody[3].pitch.note, Note::E);
        assert_eq!(melody[4].pitch.note, Note::E);
    }

    #[test]
    fn test_boundary_reflect() {
        let mut config = simple_config();
        config.bounds = PitchBounds::try_new(
            Pitch::new(Note::C, 4),
            Pitch::new(Note::E, 4),
        ).unwrap();
        config.turnaround_mode = TurnaroundMode::Reflect;
        config.max_length = 8;

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // C D E, then reflect: D C, then reflect: D E
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[1].pitch.note, Note::D);
        assert_eq!(melody[2].pitch.note, Note::E);
        // Direction flips, goes down
        assert_eq!(melody[3].pitch.note, Note::D);
        assert_eq!(melody[4].pitch.note, Note::C);
    }

    #[test]
    fn test_boundary_wrap() {
        let mut config = simple_config();
        config.bounds = PitchBounds::try_new(
            Pitch::new(Note::C, 4),
            Pitch::new(Note::E, 4),
        ).unwrap();
        config.turnaround_mode = TurnaroundMode::Wrap;
        config.max_length = 5;

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // C D E, then wrap to lowest
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[1].pitch.note, Note::D);
        assert_eq!(melody[2].pitch.note, Note::E);
        // Wraps to C4
        assert_eq!(melody[3].pitch.note, Note::C);
        assert_eq!(melody[3].pitch.octave, 4);
    }

    #[test]
    fn test_boundary_start_over() {
        let mut config = simple_config();
        config.bounds = PitchBounds::try_new(
            Pitch::new(Note::C, 4),
            Pitch::new(Note::E, 4),
        ).unwrap();
        config.turnaround_mode = TurnaroundMode::StartOver;
        config.max_length = 5;

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // C D E, then start over at C
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[1].pitch.note, Note::D);
        assert_eq!(melody[2].pitch.note, Note::E);
        // Returns to starting pitch
        assert_eq!(melody[3].pitch.note, Note::C);
        assert_eq!(melody[3].pitch.octave, 4);
    }

    #[test]
    fn test_rhythm_pattern_cycling() {
        let mut config = simple_config();
        config.rhythm_pattern = vec![
            Duration::QTR,
            Duration::EIGHTH,
            Duration::EIGHTH,
        ];
        config.max_length = 6;

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // Rhythm should cycle: Q E E Q E E
        assert_eq!(melody[0].duration, Duration::QTR);
        assert_eq!(melody[1].duration, Duration::EIGHTH);
        assert_eq!(melody[2].duration, Duration::EIGHTH);
        assert_eq!(melody[3].duration, Duration::QTR);
        assert_eq!(melody[4].duration, Duration::EIGHTH);
        assert_eq!(melody[5].duration, Duration::EIGHTH);
    }

    #[test]
    fn test_reset() {
        let config = simple_config();
        let mut sequencer = MelodicSequencer::new(config);

        let melody1 = sequencer.generate().unwrap();
        sequencer.reset();
        let melody2 = sequencer.generate().unwrap();

        // After reset, should produce same melody
        for (e1, e2) in melody1.iter().zip(melody2.iter()) {
            assert_eq!(e1.pitch.midi_note, e2.pitch.midi_note);
            assert_eq!(e1.duration, e2.duration);
        }
    }

    #[test]
    fn test_iterator() {
        let config = simple_config();
        let mut sequencer = MelodicSequencer::new(config);

        let events: Result<Vec<_>, _> = sequencer.iter().collect();
        let melody = events.unwrap();

        assert_eq!(melody.len(), 8);
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[1].pitch.note, Note::D);
    }

    #[test]
    fn test_triad_stepping() {
        // Test with a triad instead of full scale
        let triad = NoteSet::new(vec![Note::C, Note::E, Note::G]);

        let config = MelodicSequencerConfig {
            chord_progression: ChordProgression::static_chord(triad),
            interval_pattern: IntervalPattern::simple(vec![1], 1),
            rhythm_pattern: vec![Duration::EIGHTH],
            bounds: PitchBounds::try_new(
                Pitch::new(Note::C, 3),
                Pitch::new(Note::C, 6),
            ).unwrap(),
            starting_pitch: Pitch::new(Note::C, 4),
            direction: Direction::Up,
            turnaround_mode: TurnaroundMode::Reflect,
            max_length: 6,
        };

        let mut sequencer = MelodicSequencer::new(config);
        let melody = sequencer.generate().unwrap();

        // Should step through triad: C E G C E G
        assert_eq!(melody[0].pitch.note, Note::C);
        assert_eq!(melody[0].pitch.octave, 4);
        assert_eq!(melody[1].pitch.note, Note::E);
        assert_eq!(melody[1].pitch.octave, 4);
        assert_eq!(melody[2].pitch.note, Note::G);
        assert_eq!(melody[2].pitch.octave, 4);
        assert_eq!(melody[3].pitch.note, Note::C);
        assert_eq!(melody[3].pitch.octave, 5);
    }
}
