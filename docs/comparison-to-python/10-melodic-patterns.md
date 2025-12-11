# Melodic Patterns and Sequence Generation

This document compares melodic pattern generation between Python `pitch_set_lib` and Rust `music`, and provides a complete implementation plan for the missing Rust functionality.

## Overview

### Python: Full Melodic Sequencer

The Python library has a complete `MelodicSequencer` class (~270 lines) that generates melodies based on:
- Interval sequences (patterns)
- Chord progressions
- Rhythm patterns
- Pitch boundaries
- Turnaround modes (boundary behavior)

### Rust: Partial Implementation

The Rust library has:
- `MelodicFretboardShape` - Scale shapes on fretboard (~200 lines)
- `Contour` enum - Basic melodic contour types (~28 lines)
- **Missing**: Melodic sequence generation, pattern application, boundary handling

## Python MelodicSequencer Analysis

### Core Architecture

```python
class MelodicSequencer:
    """
    Produces a sequence of timed pitch content according to a given
    sequence of steps and a pitch set or progression
    """

    # Direction constants
    UP = 1
    DOWN = -1

    # Boundary behavior modes
    REFLECT = "REFLECT"      # Reverse direction, same step count
    RICOCHET = "RICOCHET"    # Reverse direction, bounce off boundary
    STARTOVER = "STARTOVER"  # Return to starting note

    def __init__(self, *,
        chord_prog,           # Chord progression (Rhythm-wrapped chords)
        interval_sequence,    # 2D pattern: [[1,1], [2]] = nested steps
        master_step=1,        # Default step when pattern exhausted
        lowest_pitch,         # Pitch boundary (low)
        highest_pitch,        # Pitch boundary (high)
        starting_note,        # Initial pitch
        time_signature,       # Meter object
        rhythm,               # List of durations
        max_length,           # Maximum melody length
        direction,            # UP or DOWN
        turnaround_mode,      # Boundary behavior
    ):
        # ...
```

### Key Algorithms

#### 1. Step Counter (Multi-Level Pattern Tracking)

```python
def step_counter(self):
    """
    Tracks through interval_sequence with multiple levels.
    e.g. [[1,1], [2]] yields: (1,0), (1,0), (2,1), (1,0), (1,0), (master_step,2)
    """
    num_levels = len(self.interval_sequence)
    counters = [iter(n) for n in self.interval_sequence]

    while True:
        level = 0
        while True:
            try:
                val = next(counters[level])
                break
            except StopIteration:
                counters[level] = iter(self.interval_sequence[level])
                level = (level + 1) % num_levels
                if level == 0:
                    yield (self.master_step, num_levels)
        yield (val, level)
```

**Pattern Behavior Example:**
```
interval_sequence = [[9,8], [7,6,5], [4]]
Yields: (9,0) (8,0) (7,1) (9,0) (8,0) (6,1) (9,0) (8,0) (5,1) (9,0) (8,0) (4,2)
        (9,0) (8,0) (7,1) (9,0) (8,0) (6,1) (9,0) (8,0) (5,1) (9,0) (8,0) (master,3)
```

#### 2. Note Generator (Coroutine-Based)

```python
def next_note(self):
    """Generator that tracks multiple melodic levels."""
    step = self.step_counter()
    num_levels = len(self.interval_sequence)
    last_note = [self.starting_note] * (num_levels + 1)

    chord = yield self.starting_note
    while True:
        next_step, level = next(step)
        next_step *= self.direction
        previous_pitch = last_note[level]

        closest_note = previous_pitch.closest_in_chord_to(chord)
        next_note = closest_note.n_steps_up_in_chord(next_step, chord)
        next_note = self.direction_and_pitch_boundary_logic(
            next_step, previous_pitch, next_note, chord
        )

        # Update tracked notes for this and lower levels
        if level == 0:
            last_note[0] = next_note
        else:
            for i in range(level + 1):
                last_note[i] = next_note

        chord = yield next_note
```

#### 3. Boundary Handling

```python
def direction_and_pitch_boundary_logic(self, num_steps, previous_pitch, next_note, chord):
    # In bounds - return as-is
    if self.lowest_pitch <= next_note <= self.highest_pitch:
        return next_note

    # STARTOVER mode
    if self.turnaround_mode == MelodicSequencer.STARTOVER:
        return self.starting_note

    # REFLECT mode - flip direction, same steps
    if self.turnaround_mode == MelodicSequencer.REFLECT:
        self.flip_direction()
        num_steps *= self.direction
        closest_note = previous_pitch.closest_in_chord_to(chord)
        next_note = closest_note.n_steps_up_in_chord(num_steps, chord)
        # Validate result is in bounds
        if not self.lowest_pitch <= next_note <= self.highest_pitch:
            raise ValueError(...)

    # RICOCHET mode - bounce off boundary
    if self.turnaround_mode == MelodicSequencer.RICOCHET:
        while not self.lowest_pitch <= next_note <= self.highest_pitch:
            next_note = self.apply_ricochet(next_note, previous_pitch, chord)

    return next_note

def apply_ricochet(self, next_note, previous_pitch, chord):
    """Bounce off pitch boundary."""
    self.flip_direction()
    num_steps = previous_pitch.distance_in_steps(next_note, chord)

    if self.lowest_pitch > next_note:  # Too low
        closest = self.lowest_pitch.closest_in_chord_to(chord, force_above=True)
        distance = previous_pitch.distance_in_steps(closest, chord)
        remainder = abs(num_steps - distance)
        next_note = closest.n_steps_up_in_chord(remainder, chord)
    else:  # Too high
        closest = self.highest_pitch.closest_in_chord_to(chord, force_below=True)
        distance = previous_pitch.distance_in_steps(closest, chord)
        remainder = abs(num_steps - distance)
        next_note = closest.n_steps_up_in_chord(-1 * remainder, chord)

    return next_note
```

#### 4. Main Run Loop

```python
def run(self):
    melody = []
    next_rhythm = cycle(self.rhythm)
    next_chord = self.next_chord()
    next_note = self.next_note()
    beat_onset = 0

    # First note
    rhy = next(next_rhythm)
    chord = next(next_chord)
    note = next(next_note)
    melody.append(Rhythm(rhy, note, meter=self.time_signature, offset=0))

    # Generate melody
    while len(melody) <= self.max_length:
        rhy = next(next_rhythm)
        chord = next_chord.send(rhy)
        note = next_note.send(chord)
        beat_onset = (beat_onset + rhy) % self.time_signature.total_duration
        melody.append(Rhythm(rhy, note, meter=self.time_signature, offset=beat_onset))

    return melody
```

## Rust: Existing Melodic Infrastructure

### MelodicFretboardShape

```rust
// fretboard/fretboard_shape/melodic_shape_search.rs

pub struct MelodicFretboardShape<'a> {
    pub shape: Vec<SoundedNote<'a>>,
    pub score: usize,  // Playability cost
    pub fretboard: &'a Fretboard,
}

impl<'a> MelodicFretboardShape<'a> {
    pub fn is_complete(&self) -> bool;      // Two-octave range?
    pub fn range(&self) -> (Pitch, Pitch);  // High/low
    pub fn span(&self) -> (u8, u8);         // Fret span
    pub fn mirrored_outer_strings(&self) -> Self;
    pub fn subsumes_other(&self, other: &Self) -> bool;
}
```

### Scale Shape Search

```rust
pub struct ScaleShapeSearchResult<'a> {
    pub simple: Vec<MelodicFretboardShape<'a>>,
    pub open: MelodicFretboardShape<'a>,
    pub n_per_string_2_2: HashMap<Note, MelodicFretboardShape<'a>>,
    pub n_per_string_2_3: HashMap<Note, MelodicFretboardShape<'a>>,
    pub n_per_string_3_3: HashMap<Note, MelodicFretboardShape<'a>>,
    pub other: HashMap<Note, Vec<MelodicFretboardShape<'a>>>,
}

// Key functions
pub fn find_open_scale_shape(...) -> Result<MelodicFretboardShape, ...>;
pub fn find_all_scale_shapes(...) -> HashMap<Note, Vec<MelodicFretboardShape>>;
pub fn n_note_per_string_shape(...) -> Result<MelodicFretboardShape, ...>;
```

### Contour Types

```rust
// geometry/contour.rs

pub enum Movement {
    Ascending,
    Descending,
}

pub enum Contour {
    Movement(Movement),
    Repeat,
}

pub enum IntervalChange {
    Expanding,
    Contracting,
}

pub enum CompositeMovement {
    Parallel(Movement),
    Similar(Movement, IntervalChange),
    Oblique(Movement, IntervalChange),
    Contrary(IntervalChange),
    Repeat,
}
```

## Suggested Rust Implementation

### Core Types

```rust
// melody/mod.rs (new module)

use crate::note::pitch::Pitch;
use crate::note_collections::NoteSet;
use crate::notation::rhythm::{Duration, RhythmicNotatedEvent};

/// Direction of melodic movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn flip(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn multiplier(&self) -> i8 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
        }
    }
}

/// Behavior when melody hits pitch boundaries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnaroundMode {
    /// Reverse direction, move same number of steps
    Reflect,
    /// Bounce off boundary, remaining steps go opposite direction
    Ricochet,
    /// Return to starting note
    StartOver,
    /// Stop at boundary
    Stop,
    /// Wrap to opposite boundary (for circular melodies)
    Wrap,
}

/// Configuration for pitch boundaries
#[derive(Debug, Clone)]
pub struct PitchBounds {
    pub lowest: Pitch,
    pub highest: Pitch,
}

impl PitchBounds {
    pub fn new(lowest: Pitch, highest: Pitch) -> Result<Self, MusicSemanticsError> {
        if lowest.midi_note >= highest.midi_note {
            return Err(MusicSemanticsError::InvalidPitchBounds);
        }
        Ok(Self { lowest, highest })
    }

    pub fn contains(&self, pitch: &Pitch) -> bool {
        pitch.midi_note >= self.lowest.midi_note
            && pitch.midi_note <= self.highest.midi_note
    }

    pub fn span(&self) -> u8 {
        self.highest.midi_note - self.lowest.midi_note
    }
}
```

### Interval Pattern System

```rust
// melody/pattern.rs

/// A single level of an interval pattern
#[derive(Debug, Clone)]
pub struct PatternLevel {
    intervals: Vec<i8>,
    position: usize,
}

impl PatternLevel {
    pub fn new(intervals: Vec<i8>) -> Self {
        Self { intervals, position: 0 }
    }

    pub fn next(&mut self) -> Option<i8> {
        if self.position >= self.intervals.len() {
            return None;
        }
        let val = self.intervals[self.position];
        self.position += 1;
        Some(val)
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn is_exhausted(&self) -> bool {
        self.position >= self.intervals.len()
    }
}

/// Multi-level interval pattern (like Python's interval_sequence)
#[derive(Debug, Clone)]
pub struct IntervalPattern {
    levels: Vec<PatternLevel>,
    master_step: i8,
}

impl IntervalPattern {
    pub fn new(pattern: Vec<Vec<i8>>, master_step: i8) -> Self {
        let levels = pattern.into_iter()
            .map(PatternLevel::new)
            .collect();
        Self { levels, master_step }
    }

    /// Get next interval and the level it came from
    pub fn next_interval(&mut self) -> (i8, usize) {
        if self.levels.is_empty() {
            return (self.master_step, 0);
        }

        for level in 0..self.levels.len() {
            if let Some(interval) = self.levels[level].next() {
                return (interval, level);
            }
            // This level exhausted - reset and try next
            self.levels[level].reset();
        }

        // All levels completed one cycle - return master step
        (self.master_step, self.levels.len())
    }

    /// Reset all levels to starting position
    pub fn reset(&mut self) {
        for level in &mut self.levels {
            level.reset();
        }
    }
}

impl Iterator for IntervalPattern {
    type Item = (i8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_interval())
    }
}
```

### Melodic Context (Harmonic Environment)

```rust
// melody/context.rs

use crate::note_collections::NoteSet;
use crate::notation::rhythm::Duration;

/// A chord with its duration in the progression
#[derive(Debug, Clone)]
pub struct TimedChord {
    pub notes: NoteSet,
    pub duration: Duration,
}

/// Chord progression with timing
#[derive(Debug, Clone)]
pub struct ChordProgression {
    chords: Vec<TimedChord>,
    position: usize,
    tick_in_chord: usize,
}

impl ChordProgression {
    pub fn new(chords: Vec<TimedChord>) -> Self {
        Self {
            chords,
            position: 0,
            tick_in_chord: 0,
        }
    }

    /// Get current chord
    pub fn current(&self) -> &TimedChord {
        &self.chords[self.position]
    }

    /// Advance by duration, returning new current chord
    pub fn advance(&mut self, duration: &Duration) -> &TimedChord {
        self.tick_in_chord += duration.ticks();

        while self.tick_in_chord >= self.current().duration.ticks() {
            self.tick_in_chord -= self.current().duration.ticks();
            self.position = (self.position + 1) % self.chords.len();
        }

        self.current()
    }

    /// Reset to beginning
    pub fn reset(&mut self) {
        self.position = 0;
        self.tick_in_chord = 0;
    }
}
```

### Melodic Sequencer

```rust
// melody/sequencer.rs

use super::*;

/// Configuration for melody generation
#[derive(Debug, Clone)]
pub struct MelodicSequencerConfig {
    pub chord_progression: ChordProgression,
    pub interval_pattern: IntervalPattern,
    pub rhythm_pattern: Vec<Duration>,
    pub bounds: PitchBounds,
    pub starting_pitch: Pitch,
    pub direction: Direction,
    pub turnaround_mode: TurnaroundMode,
    pub max_length: usize,
}

/// State for melody generation
pub struct MelodicSequencer {
    config: MelodicSequencerConfig,
    direction: Direction,
    rhythm_position: usize,
    level_notes: Vec<Pitch>,  // Last note at each pattern level
}

impl MelodicSequencer {
    pub fn new(config: MelodicSequencerConfig) -> Self {
        let num_levels = config.interval_pattern.levels.len() + 1;
        let level_notes = vec![config.starting_pitch.clone(); num_levels];

        Self {
            direction: config.direction,
            rhythm_position: 0,
            level_notes,
            config,
        }
    }

    /// Generate complete melody
    pub fn generate(&mut self) -> Result<Vec<MelodicEvent>, MusicSemanticsError> {
        let mut melody = Vec::with_capacity(self.config.max_length);

        // First note
        melody.push(MelodicEvent {
            pitch: self.config.starting_pitch.clone(),
            duration: self.next_rhythm(),
            tied: false,
        });

        while melody.len() < self.config.max_length {
            let event = self.next_event()?;
            melody.push(event);
        }

        Ok(melody)
    }

    fn next_rhythm(&mut self) -> Duration {
        let rhythm = self.config.rhythm_pattern[self.rhythm_position].clone();
        self.rhythm_position = (self.rhythm_position + 1)
            % self.config.rhythm_pattern.len();
        rhythm
    }

    fn next_event(&mut self) -> Result<MelodicEvent, MusicSemanticsError> {
        let duration = self.next_rhythm();

        // Advance chord progression
        let chord = self.config.chord_progression.advance(&duration);

        // Get next interval from pattern
        let (interval, level) = self.config.interval_pattern.next_interval();
        let directed_interval = interval * self.direction.multiplier();

        // Calculate next pitch
        let previous_pitch = &self.level_notes[level];
        let next_pitch = self.calculate_next_pitch(
            previous_pitch,
            directed_interval,
            &chord.notes,
        )?;

        // Apply boundary logic
        let final_pitch = self.apply_boundary_logic(
            previous_pitch,
            next_pitch,
            directed_interval,
            &chord.notes,
        )?;

        // Update level notes
        for i in 0..=level {
            self.level_notes[i] = final_pitch.clone();
        }

        Ok(MelodicEvent {
            pitch: final_pitch,
            duration,
            tied: false,
        })
    }

    fn calculate_next_pitch(
        &self,
        from: &Pitch,
        steps: i8,
        chord: &NoteSet,
    ) -> Result<Pitch, MusicSemanticsError> {
        // Find closest chord tone to current pitch
        let closest = self.closest_chord_tone(from, chord)?;

        // Move by diatonic steps within the chord/scale
        chord.pitch_n_steps_from(&closest, steps)
    }

    fn closest_chord_tone(
        &self,
        pitch: &Pitch,
        chord: &NoteSet,
    ) -> Result<Pitch, MusicSemanticsError> {
        // Find the chord tone closest to the given pitch
        let pc = Pc::from(&pitch.note);
        let mut best_match: Option<Note> = None;
        let mut best_distance = u8::MAX;

        for note in chord.iter() {
            let note_pc = Pc::from(note);
            let up = pc.distance_up_to(&note_pc);
            let down = pc.distance_down_to(&note_pc);
            let dist = up.min(down);

            if dist < best_distance {
                best_distance = dist;
                best_match = Some(note.clone());
            }
        }

        match best_match {
            Some(note) => {
                // Preserve octave from original pitch
                Pitch::new(note, pitch.octave)
            }
            None => Err(MusicSemanticsError::EmptySetOfNotes),
        }
    }

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
                let reflected_steps = steps * -1;
                let result = self.calculate_next_pitch(previous, reflected_steps, chord)?;

                if !self.config.bounds.contains(&result) {
                    return Err(MusicSemanticsError::MelodyOutOfBounds);
                }
                Ok(result)
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

    fn apply_ricochet(
        &mut self,
        previous: &Pitch,
        attempted: &Pitch,
        chord: &NoteSet,
    ) -> Result<Pitch, MusicSemanticsError> {
        self.direction = self.direction.flip();

        // Calculate how far past the boundary we went
        let boundary = if attempted.midi_note < self.config.bounds.lowest.midi_note {
            &self.config.bounds.lowest
        } else {
            &self.config.bounds.highest
        };

        // Find closest chord tone to boundary
        let boundary_tone = self.closest_chord_tone(boundary, chord)?;

        // Calculate overshoot and bounce back
        let total_distance = (attempted.midi_note as i16 - previous.midi_note as i16).abs();
        let to_boundary = (boundary.midi_note as i16 - previous.midi_note as i16).abs();
        let overshoot = total_distance - to_boundary;

        // Move from boundary by overshoot amount in opposite direction
        let bounce_steps = (overshoot / 2) as i8 * self.direction.multiplier();
        let result = chord.pitch_n_steps_from(&boundary_tone, bounce_steps)?;

        if self.config.bounds.contains(&result) {
            Ok(result)
        } else {
            // Recursive ricochet if still out of bounds
            self.apply_ricochet(&boundary_tone, &result, chord)
        }
    }
}

/// A single melodic event with pitch and duration
#[derive(Debug, Clone)]
pub struct MelodicEvent {
    pub pitch: Pitch,
    pub duration: Duration,
    pub tied: bool,
}
```

### NoteSet Extensions

The melodic sequencer requires additional methods on `NoteSet`:

```rust
// Add to note_collections/mod.rs

impl NoteSet {
    /// Get pitch N diatonic steps from given pitch within this note set
    pub fn pitch_n_steps_from(
        &self,
        from: &Pitch,
        steps: i8,
    ) -> Result<Pitch, MusicSemanticsError> {
        let from_note = &from.note;

        // Find position of closest note in set
        let pos = self.0.iter()
            .position(|n| n.is_enharmonic(from_note))
            .ok_or(MusicSemanticsError::NotAMember(from_note.to_string()))?;

        // Calculate new position with wrapping
        let new_pos = (pos as i32 + steps as i32).rem_euclid(self.0.len() as i32) as usize;
        let new_note = &self.0[new_pos];

        // Calculate octave change
        let octave_change = (pos as i32 + steps as i32) / self.0.len() as i32;
        let new_octave = (from.octave as i32 + octave_change) as u8;

        Pitch::new(new_note.clone(), new_octave)
    }

    /// Find the note closest (by pitch class) to given pitch
    pub fn closest_to(&self, pitch: &Pitch) -> Result<&Note, MusicSemanticsError> {
        if self.0.is_empty() {
            return Err(MusicSemanticsError::EmptySetOfNotes);
        }

        let target_pc = Pc::from(&pitch.note);
        self.0.iter()
            .min_by_key(|note| {
                let pc = Pc::from(*note);
                let up = target_pc.distance_up_to(&pc);
                let down = target_pc.distance_down_to(&pc);
                up.min(down)
            })
            .ok_or(MusicSemanticsError::EmptySetOfNotes)
    }
}
```

## Usage Example

```rust
use music::melody::{
    MelodicSequencer, MelodicSequencerConfig,
    IntervalPattern, ChordProgression, TimedChord,
    Direction, TurnaroundMode, PitchBounds,
};
use music::{Note, Pitch, NoteSet, pitch};
use music::notation::rhythm::{Duration, DurationKind};

fn main() -> Result<(), MusicSemanticsError> {
    // Define chord progression: 4 bars of C major scale
    let c_major = NoteSet::new(
        vec![Note::C, Note::D, Note::E, Note::F, Note::G, Note::A, Note::B],
        Some(Note::C),
    );

    let progression = ChordProgression::new(vec![
        TimedChord {
            notes: c_major,
            duration: Duration::new(DurationKind::Whole, 0),  // 4 beats
        },
    ]);

    // Define interval pattern: [[1,1], [2]] = step step, skip, step step, skip...
    let pattern = IntervalPattern::new(
        vec![vec![1, 1], vec![2]],
        1,  // master step
    );

    // Define rhythm: eighth notes
    let rhythm = vec![Duration::new(DurationKind::Eighth, 0)];

    // Configure sequencer
    let config = MelodicSequencerConfig {
        chord_progression: progression,
        interval_pattern: pattern,
        rhythm_pattern: rhythm,
        bounds: PitchBounds::new(pitch!(c, 3), pitch!(c, 6))?,
        starting_pitch: pitch!(c, 4),
        direction: Direction::Up,
        turnaround_mode: TurnaroundMode::Ricochet,
        max_length: 32,
    };

    // Generate melody
    let mut sequencer = MelodicSequencer::new(config);
    let melody = sequencer.generate()?;

    for event in &melody {
        println!("{:?} for {:?}", event.pitch, event.duration);
    }

    Ok(())
}
```

## Comparison Summary

| Feature | Python | Rust (Current) | Rust (Proposed) |
|---------|--------|----------------|-----------------|
| Multi-level patterns | Yes | No | Yes |
| Chord progression tracking | Yes | No | Yes |
| Boundary handling | 3 modes | No | 5 modes |
| Diatonic stepping | Yes | Partial | Yes |
| Fretboard shapes | No | Yes | Keep |
| Contour analysis | Basic | Basic | Expand |
| Lilypond output | Yes | Via trait | Via trait |
| Rhythm integration | Yes | Partial | Yes |

## Implementation Priority

1. **Core pattern system** (`IntervalPattern`) - Essential
2. **Boundary handling** (`TurnaroundMode`) - Essential
3. **NoteSet extensions** (diatonic stepping) - Required
4. **MelodicSequencer** - Main API
5. **ChordProgression** - For harmonic context
6. **Contour expansion** - Analysis features
