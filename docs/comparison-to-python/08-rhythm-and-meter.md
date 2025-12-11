# Rhythm, Meter, and Duration Comparison

This document provides a detailed comparison of rhythm handling between Python `pitch_set_lib` and Rust `music`, including meter-aware duration scoring and beat grid calculations.

## Architecture Overview

### Python: Fraction-Based with Recursive Splitting

```python
# rhythm.py
from fractions import Fraction

class Rhythm:
    def __init__(self, dur, obj=None, meter=None):
        self.fraction = Fraction(dur)
        self.num_beats = float(self.fraction)
        self.obj = obj
        if meter:
            self.ly = meter.determine_split_across_meter(self, onset)
```

### Rust: Tick-Based with Duration Types

```rust
// notation/rhythm/duration.rs
pub enum DurationKind {
    Breve, Whole, Half, Qtr, Eighth, Sixteenth,
    ThirtySecond, SixtyFourth, OneTwentyEighth
}

pub struct Duration {
    dur: DurationKind,
    dot: u8,  // 0-5 dots
}

impl Duration {
    pub fn ticks(&self) -> usize {
        let base = self.dur.ticks();
        // Dots: sum of geometric series d/2 + d/4 + d/8...
        let dot_value = base - (base >> (self.dot + 1));
        base + dot_value
    }
}
```

## Duration Representation

### Python: Arbitrary Fractions

```python
# Python allows any fraction, then validates
r = Rhythm(Fraction(5, 16))  # 5 sixteenths - requires ties
r = Rhythm(Fraction(3, 8))   # Dotted quarter - valid
r = Rhythm(Fraction(7, 4))   # 7 beats - valid
r = Rhythm(Fraction(3, 17))  # Invalid - not representable

# Validation in constructor (lines 26-29):
if self.fraction.denominator not in [1, 2, 4, 8]:
    raise ValueError("Only 8th note resolution supported")
if self.fraction > 8:
    raise ValueError("Maximum duration is double-whole-note")
```

**Issues:**
- Limited to 8th note resolution (no 16ths, 32nds)
- Arbitrary limit of 8 beats max
- Validation happens at runtime

### Rust: Enumerated Durations with Dots

```rust
impl DurationKind {
    pub fn ticks(&self) -> usize {
        match self {
            DurationKind::Breve => 256,
            DurationKind::Whole => 128,
            DurationKind::Half => 64,
            DurationKind::Qtr => 32,
            DurationKind::Eighth => 16,
            DurationKind::Sixteenth => 8,
            DurationKind::ThirtySecond => 4,
            DurationKind::SixtyFourth => 2,
            DurationKind::OneTwentyEighth => 1,
        }
    }
}

// Up to 128th note resolution (vs Python's 8th)
// Dots extend via geometric series
```

**Advantages:**
- Type-safe duration representation
- 128th note resolution (16x finer than Python)
- Dots handled mathematically
- No arbitrary limits

**Suggestion - Add duration arithmetic:**

```rust
impl Duration {
    /// Try to add two durations, returning None if result isn't representable
    pub fn try_add(&self, other: &Duration) -> Option<Duration> {
        let total_ticks = self.ticks() + other.ticks();
        Duration::try_from_ticks(total_ticks)
    }

    /// Try to create duration from tick count
    pub fn try_from_ticks(ticks: usize) -> Option<Duration> {
        // Check if ticks can be represented as a single (possibly dotted) note
        for kind in DurationKind::all() {
            let base = kind.ticks();
            if ticks == base {
                return Some(Duration::new(kind, 0));
            }
            // Check dotted versions
            for dots in 1..=5 {
                let dotted = Duration::new(kind, dots);
                if dotted.ticks() == ticks {
                    return Some(dotted);
                }
            }
        }
        None
    }

    /// Split duration into tied notes that sum to this duration
    pub fn split_for_ties(&self) -> Vec<Duration> {
        let mut remaining = self.ticks();
        let mut result = Vec::new();

        for kind in DurationKind::all() {
            while kind.ticks() <= remaining {
                result.push(Duration::new(kind, 0));
                remaining -= kind.ticks();
            }
        }

        result
    }
}
```

## Meter Representation

### Python: Class with Beat Pattern

```python
# rhythm.py lines 227-323
class Meter:
    def __init__(self, meter_str, beat_pattern=None):
        parts = meter_str.split('/')
        self.n_size = int(parts[0])        # Numerator
        self.base_unit = int(parts[1])     # Denominator
        self.base_unit_duration = 4 / self.base_unit
        self.total_duration = self.n_size * self.base_unit_duration
        self.beat_pattern = beat_pattern
        self.big_beats = self.get_big_beats()
```

### Rust: Enum-Based Denominator

```rust
// notation/rhythm/meter.rs
pub enum MeterDenominator {
    One,     // Whole note = 1 beat
    Two,     // Half note = 1 beat
    Four,    // Quarter note = 1 beat
    Eight,   // Eighth note = 1 beat
    Sixteen, // Sixteenth = 1 beat
}

// Meter construction (partial - needs expansion)
pub fn get_big_beats(
    num_beats: usize,
    base_unit_duration: DurationKind
) -> Vec<DurationTicks> {
    // ... implementation
}
```

**Suggestion - Complete meter implementation:**

```rust
pub struct Meter {
    pub numerator: u8,
    pub denominator: MeterDenominator,
    pub beat_pattern: Option<Vec<u8>>,  // Custom grouping
}

impl Meter {
    pub fn new(numerator: u8, denominator: MeterDenominator) -> Self {
        Self {
            numerator,
            denominator,
            beat_pattern: None,
        }
    }

    pub fn with_beat_pattern(mut self, pattern: Vec<u8>) -> Self {
        self.beat_pattern = Some(pattern);
        self
    }

    /// Get the duration of one base unit in ticks
    pub fn base_unit_ticks(&self) -> usize {
        match self.denominator {
            MeterDenominator::One => 128,
            MeterDenominator::Two => 64,
            MeterDenominator::Four => 32,
            MeterDenominator::Eight => 16,
            MeterDenominator::Sixteen => 8,
        }
    }

    /// Total measure duration in ticks
    pub fn measure_ticks(&self) -> usize {
        self.numerator as usize * self.base_unit_ticks()
    }

    /// Get the "big beats" - primary rhythmic groupings
    pub fn big_beats(&self) -> Vec<usize> {
        if let Some(pattern) = &self.beat_pattern {
            // Custom pattern
            pattern.iter()
                .map(|&n| n as usize * self.base_unit_ticks())
                .collect()
        } else {
            // Automatic pattern detection
            self.detect_beat_pattern()
        }
    }

    fn detect_beat_pattern(&self) -> Vec<usize> {
        let base = self.base_unit_ticks();
        let n = self.numerator as usize;

        match self.denominator {
            MeterDenominator::Eight => {
                // Compound meters: group in 3s
                match n {
                    6 => vec![3 * base, 3 * base],           // 6/8 = 2 dotted quarters
                    9 => vec![3 * base; 3],                  // 9/8 = 3 groups
                    12 => vec![3 * base; 4],                 // 12/8 = 4 groups
                    _ => vec![base; n],                      // Simple
                }
            }
            MeterDenominator::Four => {
                // Common time signatures
                match n {
                    2 => vec![base; 2],                      // 2/4
                    3 => vec![base; 3],                      // 3/4
                    4 => vec![base; 4],                      // 4/4
                    5 => vec![3 * base, 2 * base],           // 5/4 = 3+2
                    7 => vec![2 * base, 2 * base, 3 * base], // 7/4 = 2+2+3
                    _ => vec![base; n],
                }
            }
            _ => vec![base; n],
        }
    }
}
```

## Meter-Aware Duration Splitting

### Python: Recursive Algorithm

```python
# rhythm.py lines 31-101
def determine_split_across_meter(self, rhythm, onset):
    """Split duration across barlines respecting notation conventions."""

    # Handle barline crossing
    remaining_in_measure = self.total_duration - onset
    if rhythm.num_beats > remaining_in_measure:
        # Split at barline
        first_part = Rhythm(remaining_in_measure, rhythm.obj)
        second_part = Rhythm(rhythm.num_beats - remaining_in_measure, rhythm.obj)
        return first_part.ly + '~' + second_part.ly

    # 4/4 exception: quarter notes on "and" of beats 1 and 3
    if (self.meter_str == "4/4" and
        rhythm.num_beats == 1 and
        onset in [0.5, 2.5]):
        return rhythm.ly  # Don't split

    # Check against big beat grid
    big_beat_positions = self.get_big_beat_positions()
    for pos in big_beat_positions:
        if onset < pos < onset + rhythm.num_beats:
            # Duration crosses big beat - split
            first_dur = pos - onset
            second_dur = rhythm.num_beats - first_dur
            return (Rhythm(first_dur, rhythm.obj).ly + '~' +
                    Rhythm(second_dur, rhythm.obj).ly)

    return rhythm.ly
```

### Rust: Needs Implementation

The Rust crate currently lacks meter-aware splitting. Here's a suggested implementation:

```rust
// notation/rhythm/meter.rs

pub struct MeterContext {
    pub meter: Meter,
    pub position: usize,  // Current position in ticks within measure
}

impl MeterContext {
    pub fn new(meter: Meter) -> Self {
        Self { meter, position: 0 }
    }

    /// Advance position, wrapping at measure boundary
    pub fn advance(&mut self, ticks: usize) {
        self.position = (self.position + ticks) % self.meter.measure_ticks();
    }

    /// Check if a duration at current position needs splitting
    pub fn needs_split(&self, duration: &Duration) -> Option<usize> {
        let dur_ticks = duration.ticks();
        let end_pos = self.position + dur_ticks;
        let measure_end = self.meter.measure_ticks();

        // Check barline crossing first
        if end_pos > measure_end {
            return Some(measure_end - self.position);
        }

        // Check big beat crossings
        let big_beats = self.meter.big_beats();
        let mut cumulative = 0;

        for beat_dur in &big_beats {
            cumulative += beat_dur;
            if self.position < cumulative && cumulative < end_pos {
                // Would cross a big beat boundary
                return Some(cumulative - self.position);
            }
        }

        None
    }

    /// Split a duration respecting meter, returning tied durations
    pub fn split_for_meter(&self, duration: &Duration) -> Vec<(Duration, bool)> {
        let mut result = Vec::new();
        let mut remaining = duration.ticks();
        let mut pos = self.position;

        while remaining > 0 {
            // Create temporary context at current position
            let temp_ctx = MeterContext {
                meter: self.meter.clone(),
                position: pos,
            };

            // Check if we need to split
            if let Some(split_at) = temp_ctx.needs_split(&Duration::from_ticks_approx(remaining)) {
                // Add first part (tied if not last)
                if let Some(dur) = Duration::try_from_ticks(split_at) {
                    result.push((dur, true));  // tied = true
                } else {
                    // Split into representable durations
                    let splits = Duration::from_ticks_approx(split_at).split_for_ties();
                    for (i, d) in splits.iter().enumerate() {
                        result.push((d.clone(), i < splits.len() - 1 || remaining > split_at));
                    }
                }
                remaining -= split_at;
                pos = (pos + split_at) % self.meter.measure_ticks();
            } else {
                // No split needed - add final duration
                if let Some(dur) = Duration::try_from_ticks(remaining) {
                    result.push((dur, false));  // tied = false (last note)
                } else {
                    let splits = Duration::from_ticks_approx(remaining).split_for_ties();
                    for (i, d) in splits.iter().enumerate() {
                        result.push((d.clone(), i < splits.len() - 1));
                    }
                }
                remaining = 0;
            }
        }

        result
    }
}
```

## Dotted Rhythm Handling

### Python: Bit Manipulation

```python
# rhythm.py lines 140-154
def ly_notate_duration(frac):
    """Convert Fraction to Lilypond notation."""
    n, d = frac.numerator, frac.denominator

    # Dotted rhythms have numerators 3, 7, 15, 31...
    if n in [3, 7, 15]:
        num_dots = n.bit_length() - 1
        base_dur = d // 2
        return str(int((base_dur * 4))) + '.' * num_dots

    # Simple duration
    return str(int((d * 4) / n))
```

### Rust: Geometric Series

```rust
impl Duration {
    pub fn ticks(&self) -> usize {
        let base = self.dur.ticks();
        if self.dot == 0 {
            return base;
        }

        // Dots add: base/2 + base/4 + base/8 + ...
        // Sum of geometric series: base * (1 - (1/2)^n) / (1 - 1/2)
        // = base * (2 - 2^(1-n)) = base * 2 - base * 2^(1-n)
        // = 2*base - base >> (n-1) = 2*base - base/2^(n-1)
        let dot_total: usize = (0..self.dot)
            .map(|i| base >> (i + 1))
            .sum();
        base + dot_total
    }

    pub fn to_lilypond_string(&self) -> String {
        let base = match self.dur {
            DurationKind::Breve => "\\breve",
            DurationKind::Whole => "1",
            DurationKind::Half => "2",
            DurationKind::Qtr => "4",
            DurationKind::Eighth => "8",
            DurationKind::Sixteenth => "16",
            DurationKind::ThirtySecond => "32",
            DurationKind::SixtyFourth => "64",
            DurationKind::OneTwentyEighth => "128",
        };

        let dots = ".".repeat(self.dot as usize);
        format!("{}{}", base, dots)
    }
}
```

## Tuplet Handling

### Python: Ratio-Based

```python
# rhythm.py lines 200-225
class Tuplet:
    def __init__(self, ratio_str, rhythms, base_unit_nth=None):
        # Parse "3/2" -> n=3, d=2
        parts = ratio_str.split('/')
        self.n = int(parts[0])  # Number of notes
        self.d = int(parts[1])  # In time of d notes
        self.rhythms = rhythms
        self.base_unit_nth = base_unit_nth or self._calc_base()

    @property
    def ly(self):
        inner = ' '.join(r.ly for r in self.rhythms)
        return f"\\tuplet {self.n}/{self.d} {{ {inner} }}"
```

### Rust: Structured Tuplet

```rust
// notation/rhythm/mod.rs
pub struct Tuplet<'a> {
    pub events: Vec<RhythmicNotatedEvent<'a>>,
    pub numerator: usize,    // Play this many notes
    pub denominator: usize,  // In time of this many
    pub base_unit: DurationKind,
}

impl<'a> Tuplet<'a> {
    /// Calculate actual duration of entire tuplet
    pub fn total_ticks(&self) -> usize {
        // Tuplet plays `numerator` notes in time of `denominator`
        // So each note is scaled by denominator/numerator
        let base_ticks = self.base_unit.ticks();
        base_ticks * self.denominator
    }

    /// Duration each tuplet note occupies
    pub fn note_ticks(&self) -> usize {
        self.total_ticks() / self.numerator
    }
}
```

**Suggestion - Add tuplet validation and common patterns:**

```rust
impl<'a> Tuplet<'a> {
    /// Create a standard triplet (3 in time of 2)
    pub fn triplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base: DurationKind
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 3 {
            return Err(MusicSemanticsError::InvalidTuplet(
                "Triplet must have exactly 3 events".to_string()
            ));
        }
        Ok(Self {
            events,
            numerator: 3,
            denominator: 2,
            base_unit: base,
        })
    }

    /// Create a duplet (2 in time of 3) - common in compound time
    pub fn duplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base: DurationKind
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 2 {
            return Err(MusicSemanticsError::InvalidTuplet(
                "Duplet must have exactly 2 events".to_string()
            ));
        }
        Ok(Self {
            events,
            numerator: 2,
            denominator: 3,
            base_unit: base,
        })
    }

    /// Create quintuplet (5 in time of 4)
    pub fn quintuplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base: DurationKind
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 5 {
            return Err(MusicSemanticsError::InvalidTuplet(
                "Quintuplet must have exactly 5 events".to_string()
            ));
        }
        Ok(Self {
            events,
            numerator: 5,
            denominator: 4,
            base_unit: base,
        })
    }

    /// Validate that tuplet events fit within the tuplet's total duration
    pub fn validate(&self) -> Result<(), MusicSemanticsError> {
        let expected_note_ticks = self.note_ticks();
        let total_event_ticks: usize = self.events.iter()
            .map(|e| e.duration_ticks())
            .sum();

        if total_event_ticks != self.total_ticks() {
            return Err(MusicSemanticsError::InvalidTuplet(
                format!(
                    "Tuplet events sum to {} ticks, expected {}",
                    total_event_ticks, self.total_ticks()
                )
            ));
        }

        Ok(())
    }
}
```

## Rhythmic Event Types

### Python: Simple with Optional Object

```python
class Rhythm:
    def __init__(self, dur, obj=None, meter=None, tied=False):
        self.fraction = Fraction(dur)
        self.obj = obj  # Could be Note, Pitch, Chord, Rest, etc.
        self.tied = tied

class Rest:
    ly = 'r'

class RestType(Rhythm):
    def __init__(self, dur, meter=None):
        super().__init__(dur, obj=Rest(), meter=meter)
```

### Rust: Enum-Based Events

```rust
// notation/rhythm/mod.rs
pub struct RhythmicNotatedEvent<'a> {
    pub tied: bool,
    pub event: NotatedEvent<'a>,
}

pub enum NotatedEvent<'a> {
    SingleEvent(SingleEvent<'a>, Duration),
    Tuplet(Tuplet<'a>),
}

pub enum SingleEvent<'a> {
    Pitch(Pitch),
    Voicing(Voicing),
    Fretted(SoundedNote<'a>),
    FrettedMany(Vec<SoundedNote<'a>>),
    Rest,
}
```

**Suggestion - Add duration methods to events:**

```rust
impl<'a> RhythmicNotatedEvent<'a> {
    pub fn duration_ticks(&self) -> usize {
        match &self.event {
            NotatedEvent::SingleEvent(_, dur) => dur.ticks(),
            NotatedEvent::Tuplet(tup) => tup.total_ticks(),
        }
    }

    pub fn is_rest(&self) -> bool {
        matches!(&self.event,
            NotatedEvent::SingleEvent(SingleEvent::Rest, _))
    }

    pub fn pitches(&self) -> Vec<&Pitch> {
        match &self.event {
            NotatedEvent::SingleEvent(single, _) => {
                match single {
                    SingleEvent::Pitch(p) => vec![p],
                    SingleEvent::Voicing(v) => v.0.iter().collect(),
                    SingleEvent::Fretted(sn) => vec![&sn.pitch],
                    SingleEvent::FrettedMany(sns) => {
                        sns.iter().map(|sn| &sn.pitch).collect()
                    }
                    SingleEvent::Rest => vec![],
                }
            }
            NotatedEvent::Tuplet(tup) => {
                tup.events.iter()
                    .flat_map(|e| e.pitches())
                    .collect()
            }
        }
    }
}
```

## Beat Grid Calculation

### Python: Complex Heuristics

```python
# rhythm.py lines 266-298
def get_big_beats(self):
    """Determine primary beats in measure."""
    if self.beat_pattern:
        return [b * self.base_unit_duration for b in self.beat_pattern]

    # Compound meter detection
    if self.base_unit == 8:
        for divisor in [7, 5, 3, 2]:
            if self.n_size % divisor == 0:
                group_size = self.n_size // divisor
                return [group_size * self.base_unit_duration] * divisor

    # Odd meters
    if self.n_size == 5:
        return [3, 2]  # 3+2 grouping
    elif self.n_size == 7:
        return [2, 2, 3]  # 2+2+3 grouping
    elif self.n_size == 11:
        return [3, 3, 3, 2]
    elif self.n_size == 13:
        return [3, 3, 3, 2, 2]

    # Simple meter
    return [self.base_unit_duration] * self.n_size
```

### Rust: Suggested Complete Implementation

```rust
// notation/rhythm/beat_grid.rs

pub struct BeatGrid {
    positions: Vec<usize>,  // Tick positions of each beat
    strengths: Vec<BeatStrength>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeatStrength {
    Downbeat,   // First beat of measure
    Strong,     // Primary beats
    Medium,     // Secondary beats
    Weak,       // Off-beats
}

impl BeatGrid {
    pub fn from_meter(meter: &Meter) -> Self {
        let big_beats = meter.big_beats();
        let mut positions = vec![0];  // Always start at 0
        let mut strengths = vec![BeatStrength::Downbeat];

        let mut current_pos = 0;
        for (i, &beat_dur) in big_beats.iter().enumerate() {
            current_pos += beat_dur;
            if current_pos < meter.measure_ticks() {
                positions.push(current_pos);
                strengths.push(if i == 0 {
                    BeatStrength::Strong
                } else {
                    BeatStrength::Medium
                });
            }
        }

        // Add subdivisions for weak beats
        let subdivision = meter.base_unit_ticks();
        let mut pos = 0;
        while pos < meter.measure_ticks() {
            if !positions.contains(&pos) {
                positions.push(pos);
                strengths.push(BeatStrength::Weak);
            }
            pos += subdivision;
        }

        // Sort by position
        let mut pairs: Vec<_> = positions.iter()
            .zip(strengths.iter())
            .map(|(&p, &s)| (p, s))
            .collect();
        pairs.sort_by_key(|&(p, _)| p);

        Self {
            positions: pairs.iter().map(|&(p, _)| p).collect(),
            strengths: pairs.iter().map(|&(_, s)| s).collect(),
        }
    }

    /// Find the next strong beat at or after the given position
    pub fn next_strong_beat(&self, from: usize) -> Option<usize> {
        self.positions.iter()
            .zip(self.strengths.iter())
            .find(|&(&pos, &strength)| {
                pos >= from && matches!(strength,
                    BeatStrength::Downbeat | BeatStrength::Strong)
            })
            .map(|(&pos, _)| pos)
    }

    /// Get beat strength at a position
    pub fn strength_at(&self, pos: usize) -> BeatStrength {
        self.positions.iter()
            .zip(self.strengths.iter())
            .find(|&(&p, _)| p == pos)
            .map(|(_, &s)| s)
            .unwrap_or(BeatStrength::Weak)
    }

    /// Check if position is on a beat (any strength)
    pub fn is_on_beat(&self, pos: usize) -> bool {
        self.positions.contains(&pos)
    }
}
```

## Summary Comparison

| Feature | Python | Rust | Notes |
|---------|--------|------|-------|
| Resolution | 8th notes | 128th notes | Rust 16x finer |
| Duration representation | Fraction | Enum + dots | Rust type-safe |
| Meter representation | Class | Partial | Needs expansion |
| Beat grid | Heuristic | Suggested | Needs implementation |
| Barline splitting | Recursive | Suggested | Needs implementation |
| Tuplets | Basic | Structured | Both work |
| Dotted rhythms | Bit math | Geometric | Both correct |
| Event types | Any object | Typed enum | Rust safer |
| Debug output | Print statements | None | Rust cleaner |

## Priority Implementation Order

1. **Complete `Meter` struct** with all common time signatures
2. **Implement `BeatGrid`** for rhythmic position analysis
3. **Add `MeterContext`** for meter-aware splitting
4. **Duration arithmetic** (`try_add`, `split_for_ties`)
5. **Tuplet validation** and common patterns
6. **Integration tests** for complex rhythmic scenarios
