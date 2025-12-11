# Implementation Priorities

This document outlines the recommended priority order for completing the Rust `music` crate to fully replace Python `pitch_set_lib`.

## Critical Issues (Fix Immediately)

### 1. Remove Debug Print Statements

**Files affected:**
- `notation/rhythm/duration.rs:148` - println in ticks calculation
- `note_collections/chord_name/naming_heuristics/mod.rs:221,225,229,233` - debug output
- `note_collections/geometry/symmetry/voiceleading.rs:127` - debug output
- `notation/lilypond/fretboard_diagram.rs:54` - debug output

**Action:** Replace with `log` crate calls or remove entirely.

```rust
// Before
println!("Debug: {:?}", value);

// After
use log::debug;
debug!("Value: {:?}", value);
```

### 2. Handle `concat_idents` Feature Removal

**File:** `lib.rs:1`

The `concat_idents` feature has been removed from Rust. Refactor macros that depend on it.

```rust
// Before
#![feature(concat_idents)]

// After - use proc_macro or alternative approaches
```

### 3. Add Empty Test Module Content

**File:** `fretboard/fretboard_shape/mod.rs:223-225`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fretboard_shape_creation() {
        // TODO: Implement
    }

    #[test]
    fn test_playability_check() {
        // TODO: Implement
    }
}
```

---

## High Priority (Core Functionality)

### 4. Complete Chord Naming Heuristics

**Location:** `note_collections/chord_name/naming_heuristics/`

**Required implementations:**

```rust
// maj_and_min_qualities.rs
pub fn infer_major_quality(pc_set: &PcSet, root: &Note) -> Option<ChordQuality>;
pub fn infer_minor_quality(pc_set: &PcSet, root: &Note) -> Option<ChordQuality>;

// dim_qualities.rs
pub fn infer_diminished_quality(pc_set: &PcSet, root: &Note) -> Option<ChordQuality>;
pub fn infer_half_diminished(pc_set: &PcSet, root: &Note) -> Option<ChordQuality>;

// aug_qualities.rs
pub fn infer_augmented_quality(pc_set: &PcSet, root: &Note) -> Option<ChordQuality>;

// sus_qualities.rs
pub fn infer_sus2_quality(pc_set: &PcSet, root: &Note) -> Option<ChordQuality>;
pub fn infer_sus4_quality(pc_set: &PcSet, root: &Note) -> Option<ChordQuality>;

// alts_and_extensions.rs
pub fn determine_extensions(pc_set: &PcSet, base: &BaseQuality) -> (Vec<Extension>, Alt);
pub fn format_alterations(alts: &[AltChoice]) -> String;

// scale_qualities.rs
pub fn infer_scale_quality(pc_set: &PcSet, root: &Note) -> Option<ScaleQuality>;
pub fn match_seven_note_scale(pc_set: &PcSet, root: &Note) -> Option<(ScaleQuality, Vec<Pc>)>;
```

### 5. Fix Fretboard Shape Search

**Location:** `fretboard/fretboard_shape/chord_shape_search.rs`

**Issue:** High-fret shapes (>= 12) are not found.

```rust
// Add option to include high-position shapes
pub struct SearchOptions {
    pub include_high_position: bool,  // Default: true
    pub max_span: u8,
    pub include_open_strings: bool,
}

// Ensure fret 0 and octave positions are included
fn find_all_positions_for_note(note: &Note, fretboard: &Fretboard) -> Vec<(u8, u8)> {
    // Include: base_fret, base_fret + 12, and base_fret - 12 (if > 0)
}
```

### 6. Implement Meter-Aware Duration Splitting

**Location:** `notation/rhythm/meter.rs`

```rust
pub struct Meter {
    pub numerator: u8,
    pub denominator: MeterDenominator,
    pub beat_pattern: Option<Vec<u8>>,
}

pub struct MeterContext {
    pub meter: Meter,
    pub position: usize,
}

impl MeterContext {
    pub fn split_for_meter(&self, duration: &Duration) -> Vec<(Duration, bool)>;
}
```

### 7. Add Common Fretboard Tunings

**Location:** `fretboard/mod.rs`

```rust
pub mod tunings {
    pub static DROP_D: Lazy<Fretboard> = /* ... */;
    pub static DADGAD: Lazy<Fretboard> = /* ... */;
    pub static OPEN_G: Lazy<Fretboard> = /* ... */;
    pub static STANDARD_7: Lazy<Fretboard> = /* ... */;
    pub static BASS_4: Lazy<Fretboard> = /* ... */;
    pub static BASS_5: Lazy<Fretboard> = /* ... */;
}
```

---

## Medium Priority (Feature Completeness)

### 8. Add Pentatonic Scale Support

**Location:** `note_collections/chord_name/quality/scale.rs`

```rust
pub enum ScaleQuality {
    // Existing...
    MajorPentatonic,
    MinorPentatonic,
    BluesMajor,
    BluesMinor,
}

const PENTATONIC_PATTERNS: &[(ScaleQuality, &[Pc])] = &[
    (ScaleQuality::MajorPentatonic, &[Pc::Pc0, Pc::Pc2, Pc::Pc4, Pc::Pc7, Pc::Pc9]),
    (ScaleQuality::MinorPentatonic, &[Pc::Pc0, Pc::Pc3, Pc::Pc5, Pc::Pc7, Pc::Pc10]),
];
```

### 9. Complete Lilypond Document Features

**Location:** `notation/lilypond/document/`

**TODO items from source:**
- Page and paper block configuration
- MIDI block generation
- Specific Staff, StaffGroup, TabStaff, TabVoice types
- Repeat blocks, line breaks, barline control

### 10. Add UTF-8 Spelling Support

**Location:** `note/spelling.rs`

```rust
impl Note {
    pub fn to_unicode_string(&self) -> String {
        match self {
            Note::Cis => "C♯".to_string(),
            Note::Des => "D♭".to_string(),
            Note::Cisis => "C𝄪".to_string(),
            // ...
        }
    }
}
```

### 11. Implement Interval Matrix

**Location:** `note_collections/geometry/mod.rs`

```rust
pub struct IntervalMatrix {
    pcs: PcSet,
    matrix: Vec<Vec<IntervalClass>>,
}

impl IntervalMatrix {
    pub fn new(pc_set: &PcSet) -> Self;
    pub fn get(&self, row: usize, col: usize) -> Option<&IntervalClass>;
    pub fn interval_vector(&self) -> [usize; 6];
    pub fn find_interval(&self, ic: IntervalClass) -> Vec<(Pc, Pc)>;
}
```

### 12. Add Beat Grid for Rhythmic Analysis

**Location:** `notation/rhythm/beat_grid.rs` (new file)

```rust
pub struct BeatGrid {
    positions: Vec<usize>,
    strengths: Vec<BeatStrength>,
}

pub enum BeatStrength {
    Downbeat,
    Strong,
    Medium,
    Weak,
}
```

---

## Low Priority (Polish & Convenience)

### 13. Add Duration Arithmetic

**Location:** `notation/rhythm/duration.rs`

```rust
impl Duration {
    pub fn try_add(&self, other: &Duration) -> Option<Duration>;
    pub fn try_from_ticks(ticks: usize) -> Option<Duration>;
    pub fn split_for_ties(&self) -> Vec<Duration>;
}
```

### 14. Complete Voicing Register Optimization

**Location:** `note_collections/voicing.rs:136-141`

```rust
impl Voicing {
    pub fn normalize_register_to_clef(&self, clef: Clef) -> Result<Self, MusicSemanticsError> {
        // Add additional conditional for edge cases
        // where current logic might favor lower register
    }
}
```

### 15. Add FretboardShape Easy Constructor

**Location:** `fretboard/fretboard_shape/mod.rs:45`

```rust
impl<'a> FretboardShape<'a> {
    /// Create shape from fret positions (None = muted)
    pub fn from_frets(
        frets: &[Option<u8>],
        fretboard: &'a Fretboard
    ) -> Result<Self, MusicSemanticsError> {
        // Parse fret positions to FrettedNote variants
    }

    /// Create from string like "x-2-3-0-0-x"
    pub fn from_string(
        shape_str: &str,
        fretboard: &'a Fretboard
    ) -> Result<Self, MusicSemanticsError> {
        // Parse string notation
    }
}
```

### 16. Add Chord Name Parsing (Reverse)

Currently the crate can infer names from pitch sets, but not parse names to pitch sets.

**Location:** `note_collections/chord_name/` (new module)

```rust
pub fn parse_chord_name(name: &str) -> Result<(Note, PcSet), MusicSemanticsError> {
    // "CMaj7" -> (Note::C, pcs!(0, 4, 7, 11))
    // "F#m7b5" -> (Note::Fis, pcs!(0, 3, 6, 10))
}
```

### 17. Add Contour Similarity Metrics

**Location:** `note_collections/geometry/contour.rs`

```rust
impl Contour {
    pub fn similarity(&self, other: &Contour) -> f64;
    pub fn retrograde(&self) -> Self;
    pub fn inversion(&self) -> Self;
    pub fn is_equivalent(&self, other: &Contour) -> bool;
}
```

### 18. Implement Melodic Sequencer (NEW MODULE)

**Location:** `melody/` (new module)

This is a major missing feature from Python. See [10-melodic-patterns.md](./10-melodic-patterns.md) for full implementation details.

**Required components:**

```rust
// melody/mod.rs
pub mod pattern;
pub mod context;
pub mod sequencer;

// Core types
pub enum Direction { Up, Down }
pub enum TurnaroundMode { Reflect, Ricochet, StartOver, Stop, Wrap }
pub struct PitchBounds { lowest: Pitch, highest: Pitch }
pub struct IntervalPattern { levels: Vec<PatternLevel>, master_step: i8 }
pub struct ChordProgression { chords: Vec<TimedChord>, position: usize }
pub struct MelodicSequencer { config: MelodicSequencerConfig, ... }
pub struct MelodicEvent { pitch: Pitch, duration: Duration, tied: bool }
```

**Key features:**
- Multi-level interval patterns (e.g., `[[1,1], [2]]`)
- Chord progression tracking with timing
- 5 boundary turnaround modes
- Diatonic stepping within harmonic context
- Rhythm pattern cycling

**Estimated effort:** ~400 lines, 1-2 days

---

## Test Coverage Priorities

### Must Have Tests

1. `FretboardShape` - currently empty test module
2. `ChordName` / naming heuristics
3. `Meter` / duration splitting
4. `Voiceleading` - replace println with assertions

### Should Have Tests

1. Scale quality detection
2. Spelling rules edge cases
3. Tuplet validation
4. Beat grid calculations

### Nice to Have Tests

1. Lilypond output formatting
2. VexTab output formatting
3. Interval matrix operations
4. Contour analysis

---

## Estimated Effort

| Priority | Items | Est. Lines | Est. Time |
|----------|-------|------------|-----------|
| Critical | 3 | ~50 | 1 hour |
| High | 4 | ~800 | 1-2 days |
| Medium | 5 | ~600 | 1-2 days |
| Low | 5 | ~400 | 1 day |
| Melodic Sequencer | 1 | ~400 | 1-2 days |
| Tests | 8 | ~500 | 1 day |
| **Total** | **26** | **~2750** | **~1.5 weeks** |

---

## Dependency Order

```
Critical Issues (1-3)
        ↓
Chord Naming (4) ←── Scale Support (8)
        ↓
Fretboard Fix (5)
        ↓
Meter Implementation (6) ←── Beat Grid (12) ←── Duration Arithmetic (13)
        ↓                           ↓
Tunings (7)              Melodic Sequencer (18)
        ↓                           ↓
Lilypond Features (9)    NoteSet Extensions
        ↓
UTF-8 Support (10)
        ↓
Remaining Features (11, 14-17)
        ↓
Tests
```

## Migration Checklist

Use this checklist when migrating from Python to Rust:

- [ ] Note/Pitch creation works equivalently
- [ ] PcSet normalization matches Python behavior
- [ ] Chord spelling produces same results
- [ ] Chord naming produces same results (minus bugs)
- [ ] Fretboard shape search finds all Python shapes + high-position fixes
- [ ] Voiceleading generation matches (with enharmonic fix)
- [ ] Lilypond output is equivalent
- [ ] Duration/Meter handling works correctly
- [ ] Melodic sequencer generates equivalent patterns
- [ ] Multi-level interval patterns work correctly
- [ ] Boundary turnaround modes match Python behavior
- [ ] All Python tests pass with Rust equivalents
