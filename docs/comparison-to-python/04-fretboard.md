# Fretboard Module Comparison

This document compares the fretboard modeling between Python `pitch_set_lib` and Rust `music`.

## Core Fretboard Type

### Python Implementation

```python
# fretboard.py
class Fretboard(metaclass=Memoized):
    def __init__(self, num_strings=6, tuning=GTR_6STR_STD_TUNING):
        self.num_strings = num_strings
        self.tuning = tuning  # List of MIDI notes
        self.midi_notes = self.make_fretboard_midi()
        self.pitches = self.make_pitches()
        self.harmonic_context = None  # For spelling

    def make_fretboard_midi(self):
        """Generate 2D array of MIDI notes [string][fret]."""
        return [
            [open_note + fret for fret in range(25)]
            for open_note in self.tuning
        ]

    def note_at(self, string, fret, context=None):
        """Get Note at position."""
        midi = self.midi_notes[string][fret]
        # Spelling depends on context - problematic
        return Note.from_midi(midi, context or self.harmonic_context)

    def __getitem__(self, string):
        """Enable fretboard[string][fret] syntax."""
        return self.pitches[string]
```

**Python Issues:**
1. String indexing is 0-based (opposite of notation convention)
2. `harmonic_context` is mutable global state
3. `note_at` spelling is ambiguous
4. No maximum fret constant

### Rust Implementation

```rust
// fretboard/mod.rs
pub struct Fretboard {
    pub open_strings: Vec<Pitch>,
}

pub const MAX: u8 = 35;   // ~3 octaves
pub const OPEN: u8 = 0;

impl Fretboard {
    pub fn new(open_strings: Vec<Pitch>) -> Self {
        Self { open_strings }
    }

    pub fn num_strings(&self) -> u8 {
        self.open_strings.len() as u8
    }

    pub fn get_string(&self, string: u8) -> Result<&Pitch, MusicSemanticsError> {
        self.open_strings.get(string as usize)
            .ok_or(MusicSemanticsError::InvalidString(string))
    }

    pub fn sounded_note(
        &self,
        string: u8,
        fret: u8
    ) -> Result<SoundedNote, MusicSemanticsError> {
        if fret > MAX {
            return Err(MusicSemanticsError::FretTooHigh(fret));
        }
        SoundedNote::fretted(string, fret, self)
    }

    pub fn which_fret(
        &self,
        note: &Note,
        string: u8
    ) -> Result<u8, MusicSemanticsError> {
        let open = self.get_string(string)?;
        let target_pc = Pc::from(note);
        let open_pc = Pc::from(&open.note);
        Ok(open_pc.distance_up_to(&target_pc))
    }
}

// Standard tuning constant
pub static STD_6STR_GTR: Lazy<Fretboard> = Lazy::new(|| {
    Fretboard::new(vec![
        Pitch::new(Note::E, 2).unwrap(),  // E2
        Pitch::new(Note::A, 2).unwrap(),  // A2
        Pitch::new(Note::D, 3).unwrap(),  // D3
        Pitch::new(Note::G, 3).unwrap(),  // G3
        Pitch::new(Note::B, 3).unwrap(),  // B3
        Pitch::new(Note::E, 4).unwrap(),  // E4
    ])
});
```

**Rust Advantages:**
1. Explicit fret limits with `MAX` constant
2. No mutable global state
3. `Result` return types for all fallible operations
4. `Lazy` initialization for standard tuning

**Suggestion - Add common tuning presets:**

```rust
pub mod tunings {
    use super::*;

    pub static STANDARD_6: Lazy<Fretboard> = Lazy::new(|| {
        Fretboard::new(vec![
            Pitch::new(Note::E, 2).unwrap(),
            Pitch::new(Note::A, 2).unwrap(),
            Pitch::new(Note::D, 3).unwrap(),
            Pitch::new(Note::G, 3).unwrap(),
            Pitch::new(Note::B, 3).unwrap(),
            Pitch::new(Note::E, 4).unwrap(),
        ])
    });

    pub static DROP_D: Lazy<Fretboard> = Lazy::new(|| {
        Fretboard::new(vec![
            Pitch::new(Note::D, 2).unwrap(),
            Pitch::new(Note::A, 2).unwrap(),
            Pitch::new(Note::D, 3).unwrap(),
            Pitch::new(Note::G, 3).unwrap(),
            Pitch::new(Note::B, 3).unwrap(),
            Pitch::new(Note::E, 4).unwrap(),
        ])
    });

    pub static DADGAD: Lazy<Fretboard> = Lazy::new(|| {
        Fretboard::new(vec![
            Pitch::new(Note::D, 2).unwrap(),
            Pitch::new(Note::A, 2).unwrap(),
            Pitch::new(Note::D, 3).unwrap(),
            Pitch::new(Note::G, 3).unwrap(),
            Pitch::new(Note::A, 3).unwrap(),
            Pitch::new(Note::D, 4).unwrap(),
        ])
    });

    pub static STANDARD_7: Lazy<Fretboard> = Lazy::new(|| {
        Fretboard::new(vec![
            Pitch::new(Note::B, 1).unwrap(),
            Pitch::new(Note::E, 2).unwrap(),
            Pitch::new(Note::A, 2).unwrap(),
            Pitch::new(Note::D, 3).unwrap(),
            Pitch::new(Note::G, 3).unwrap(),
            Pitch::new(Note::B, 3).unwrap(),
            Pitch::new(Note::E, 4).unwrap(),
        ])
    });

    pub static BASS_4: Lazy<Fretboard> = Lazy::new(|| {
        Fretboard::new(vec![
            Pitch::new(Note::E, 1).unwrap(),
            Pitch::new(Note::A, 1).unwrap(),
            Pitch::new(Note::D, 2).unwrap(),
            Pitch::new(Note::G, 2).unwrap(),
        ])
    });
}
```

## Fretted Note Types

### Python: FrettedPitch

```python
class FrettedPitch(Pitch, metaclass=Memoized):
    def __init__(self, pitch, string, fret, fretboard):
        self.pitch = pitch if isinstance(pitch, Pitch) else Pitch(pitch)
        self.string = string
        self.fret = fret
        self.open = (fret == 0)
        self.fretboard = fretboard
        # Inherit from Pitch
        super().__init__(self.pitch.name, self.pitch.octave)

    @property
    def ly_string_num(self):
        """Lilypond uses opposite string numbering."""
        return self.fretboard.num_strings - self.string

    def next_note_same_string(self, chord):
        """Find next higher note of chord on same string."""
        for n in sorted(chord.spelling, key=lambda x: x.pc):
            if n.pc > self.pc:
                fret = self.fret + self.pitch.distance(n)
                return FrettedPitch(
                    Pitch(n.name, self.octave + (fret > 11)),
                    self.string, fret, self.fretboard
                )
        return None  # No higher note found
```

**Python Issues:**
1. Inherits from Pitch (complex hierarchy)
2. `next_note_same_string` can return `None` silently
3. String number conversion is confusing

### Rust: SoundedNote and FrettedNote

```rust
// fretboard/fretted_note.rs

/// A note that is sounded (has pitch)
pub struct SoundedNote<'a> {
    pub string: u8,
    pub fret: u8,
    pub pitch: Pitch,
    pub fretboard: &'a Fretboard,
}

/// A fret position that may or may not sound
pub enum FrettedNote<'a> {
    Sounded(SoundedNote<'a>),
    Muted { string: u8, fretboard: &'a Fretboard },
}

impl<'a> SoundedNote<'a> {
    pub fn fretted(
        string: u8,
        fret: u8,
        fretboard: &'a Fretboard
    ) -> Result<Self, MusicSemanticsError> {
        let open_string = fretboard.get_string(string)?;
        let pitch = open_string.at_distance_from(fret as isize)?;
        Ok(Self { string, fret, pitch, fretboard })
    }

    pub fn open(string: u8, fretboard: &'a Fretboard) -> Result<Self, MusicSemanticsError> {
        Self::fretted(string, 0, fretboard)
    }

    pub fn up_n_frets(&self, n: u8) -> Result<Self, MusicSemanticsError> {
        let new_fret = self.fret + n;
        if new_fret > fretboard::MAX {
            return Err(MusicSemanticsError::FretTooHigh(new_fret));
        }
        Self::fretted(self.string, new_fret, self.fretboard)
    }

    pub fn next_note_same_string(
        &self,
        notes: &[Note]
    ) -> Result<Self, MusicSemanticsError> {
        let current_pc = Pc::from(&self.pitch.note);

        for note in notes {
            let note_pc = Pc::from(note);
            if u8::from(&note_pc) > u8::from(&current_pc) {
                let distance = current_pc.distance_up_to(&note_pc);
                return self.up_n_frets(distance);
            }
        }

        Err(MusicSemanticsError::NotAMember(
            "No higher note in set".to_string()
        ))
    }
}
```

**Rust Advantages:**
1. Composition over inheritance
2. Lifetime parameter ensures fretboard outlives notes
3. Separate `SoundedNote` vs `FrettedNote` (muted support)
4. All operations return `Result`

**Suggestion - Add string-to-string navigation:**

```rust
impl<'a> SoundedNote<'a> {
    /// Move to next higher string at same pitch class
    pub fn same_note_higher_string(&self) -> Result<Self, MusicSemanticsError> {
        if self.string == 0 {
            return Err(MusicSemanticsError::InvalidString(0));
        }

        let target_string = self.string - 1;
        let fret = self.fretboard.which_fret(&self.pitch.note, target_string)?;
        Self::fretted(target_string, fret, self.fretboard)
    }

    /// Move to next lower string at same pitch class
    pub fn same_note_lower_string(&self) -> Result<Self, MusicSemanticsError> {
        let target_string = self.string + 1;
        if target_string >= self.fretboard.num_strings() {
            return Err(MusicSemanticsError::InvalidString(target_string));
        }

        let fret = self.fretboard.which_fret(&self.pitch.note, target_string)?;
        Self::fretted(target_string, fret, self.fretboard)
    }

    /// Find all positions on fretboard for same pitch
    pub fn all_positions(&self) -> Vec<Self> {
        let mut positions = Vec::new();
        for string in 0..self.fretboard.num_strings() {
            if let Ok(fret) = self.fretboard.which_fret(&self.pitch.note, string) {
                if fret <= fretboard::MAX {
                    if let Ok(pos) = Self::fretted(string, fret, self.fretboard) {
                        positions.push(pos);
                    }
                }
                // Also check octave above
                let fret_high = fret + 12;
                if fret_high <= fretboard::MAX {
                    if let Ok(pos) = Self::fretted(string, fret_high, self.fretboard) {
                        positions.push(pos);
                    }
                }
            }
        }
        positions
    }
}
```

## Fretboard Shape / Guitar Shape

### Python: GtrShape

```python
# gtr_shapes.py
class GtrShape:
    VALID = 0
    NON_TRANS = 1      # Only playable with open strings
    INVALID = -1
    PAST_12TH_FRET = -2

    def __init__(self, fretboard, shape, chord=None, voicing=None):
        self.fretboard = fretboard
        self.shape = shape  # List: 'x' for muted, int for fret
        self.chord = chord
        self.voicing = voicing

        # Calculate derived properties
        self.midi_notes = self._calculate_midi()
        self.pitches = self._calculate_pitches()

    def is_playable(self):
        """Basic span-based playability check."""
        frets = [f for f in self.shape if isinstance(f, int) and f > 0]
        if not frets:
            return True
        span = max(frets) - min(frets)
        num_notes = len([f for f in self.shape if f != 'x'])
        return span <= 3 if num_notes > 3 else span <= 4

    def validate(self):
        """Classify shape validity."""
        if not self.is_playable():
            return self.INVALID

        min_fret = min(f for f in self.shape if isinstance(f, int) and f > 0)
        if min_fret > 12:
            return self.PAST_12TH_FRET

        has_open = 0 in self.shape
        if has_open:
            return self.NON_TRANS

        return self.VALID

    @classmethod
    def find_voicing_shapes(cls, voicing, fretboard):
        """Search for all playable shapes for a voicing."""
        # Recursive search algorithm (thing2)
        # ... complex recursive logic
```

**Python Issues:**
1. Mixed type list (`'x'` and `int`) is error-prone
2. `chord=None` creates "major flaw" allowing empty shapes
3. Search algorithm has documented bugs (CMaj/9 missing)
4. Constants as class attributes (not enum)

### Rust: FretboardShape

```rust
// fretboard/fretboard_shape/mod.rs
pub struct FretboardShape<'a> {
    pub fretted_notes: Vec<FrettedNote<'a>>,
    pub fretboard: &'a Fretboard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChordShapeClassification {
    Valid,
    ValidNonTransposable,  // Has open strings
    ValidHighPosition,     // Above 12th fret
    Invalid,               // Unplayable span
}

impl<'a> FretboardShape<'a> {
    pub fn is_playable(&self) -> bool {
        let sounded: Vec<_> = self.fretted_notes.iter()
            .filter_map(|f| match f {
                FrettedNote::Sounded(s) => Some(s),
                FrettedNote::Muted { .. } => None,
            })
            .collect();

        if sounded.is_empty() {
            return false;
        }

        let frets: Vec<u8> = sounded.iter()
            .filter(|s| s.fret > 0)
            .map(|s| s.fret)
            .collect();

        if frets.is_empty() {
            return true;  // All open strings
        }

        let span = frets.iter().max().unwrap() - frets.iter().min().unwrap();
        let size = sounded.len();

        match size {
            1..=3 => span <= 4,
            _ => span <= 3,
        }
    }

    pub fn classify(&self) -> ChordShapeClassification {
        if !self.is_playable() {
            return ChordShapeClassification::Invalid;
        }

        let (min_fret, max_fret) = self.span();

        if min_fret > 12 {
            return ChordShapeClassification::ValidHighPosition;
        }

        if self.contains_open_strings() {
            return ChordShapeClassification::ValidNonTransposable;
        }

        ChordShapeClassification::Valid
    }

    pub fn span(&self) -> (u8, u8) {
        let frets: Vec<u8> = self.fretted_notes.iter()
            .filter_map(|f| f.fret())
            .filter(|&f| f > 0)
            .collect();

        if frets.is_empty() {
            return (0, 0);
        }

        (*frets.iter().min().unwrap(), *frets.iter().max().unwrap())
    }

    pub fn contains_open_strings(&self) -> bool {
        self.fretted_notes.iter().any(|f| f.fret() == Some(0))
    }
}
```

**Rust Advantages:**
1. Type-safe `FrettedNote` enum (no mixed-type list)
2. Enum for classification (not magic numbers)
3. Cannot create shape without fretboard reference
4. Empty shapes impossible (would have no `fretted_notes`)

## Shape Search Algorithm

### Python Implementation

```python
def thing2(voicing, fretboard, string_idx=0, partial_shape=None, results=None):
    """Recursive shape search."""
    if partial_shape is None:
        partial_shape = ['x'] * fretboard.num_strings
    if results is None:
        results = []

    if string_idx >= fretboard.num_strings:
        # Base case: complete shape
        shape = GtrShape(fretboard, partial_shape[:], voicing=voicing)
        if shape.is_playable():
            results.append(shape)
        return results

    note = voicing.pitches[string_idx] if string_idx < len(voicing.pitches) else None

    if note is None:
        # No note for this string - try muted
        partial_shape[string_idx] = 'x'
        thing2(voicing, fretboard, string_idx + 1, partial_shape, results)
    else:
        # Find all frets for this note on this string
        for fret in fretboard.frets_for_note(note, string_idx):
            partial_shape[string_idx] = fret
            thing2(voicing, fretboard, string_idx + 1, partial_shape, results)

    return results
```

**Python Issues:**
1. Algorithm misses some valid shapes (CMaj/9 bug documented)
2. No pruning of clearly invalid partial shapes
3. Results as mutable parameter (side-effect)

### Rust Implementation

```rust
// fretboard/fretboard_shape/chord_shape_search.rs

impl<'a> FretboardShape<'a> {
    pub fn search_chord_shapes(
        notes: &[Note],
        fretboard: &'a Fretboard
    ) -> Vec<Self> {
        let mut results = Vec::new();
        let num_strings = fretboard.num_strings() as usize;

        // Find all possible positions for each note
        let positions: Vec<Vec<(u8, u8)>> = notes.iter()
            .map(|note| Self::find_positions_for_note(note, fretboard))
            .collect();

        // Generate all combinations
        Self::search_recursive(
            fretboard,
            &positions,
            0,
            Vec::new(),
            &mut results,
        );

        results
    }

    fn find_positions_for_note(
        note: &Note,
        fretboard: &Fretboard
    ) -> Vec<(u8, u8)> {
        let mut positions = Vec::new();
        let target_pc = Pc::from(note);

        for string in 0..fretboard.num_strings() {
            let open = fretboard.get_string(string).unwrap();
            let open_pc = Pc::from(&open.note);
            let base_fret = open_pc.distance_up_to(&target_pc);

            // Check base position and octave above
            for offset in [0, 12] {
                let fret = base_fret + offset;
                if fret <= fretboard::MAX {
                    positions.push((string, fret));
                }
            }
        }

        positions
    }

    fn search_recursive(
        fretboard: &'a Fretboard,
        positions: &[Vec<(u8, u8)>],
        note_idx: usize,
        current: Vec<FrettedNote<'a>>,
        results: &mut Vec<Self>,
    ) {
        if note_idx >= positions.len() {
            // Complete shape - validate
            let shape = FretboardShape {
                fretted_notes: current,
                fretboard,
            };
            if shape.is_playable() {
                results.push(shape);
            }
            return;
        }

        for &(string, fret) in &positions[note_idx] {
            // Check string not already used
            if current.iter().any(|f| f.string() == Some(string)) {
                continue;
            }

            // Early pruning: check partial playability
            if let Ok(sounded) = SoundedNote::fretted(string, fret, fretboard) {
                let mut next = current.clone();
                next.push(FrettedNote::Sounded(sounded));

                // Prune if already too wide
                if Self::partial_span_ok(&next) {
                    Self::search_recursive(
                        fretboard,
                        positions,
                        note_idx + 1,
                        next,
                        results,
                    );
                }
            }
        }
    }

    fn partial_span_ok(notes: &[FrettedNote]) -> bool {
        let frets: Vec<u8> = notes.iter()
            .filter_map(|f| f.fret())
            .filter(|&f| f > 0)
            .collect();

        if frets.len() < 2 {
            return true;
        }

        let span = frets.iter().max().unwrap() - frets.iter().min().unwrap();
        span <= 5  // Allow some slack for partial shapes
    }
}
```

**Rust Advantages:**
1. Early pruning prevents exploring invalid branches
2. No mutable parameter side-effects
3. String collision check built-in
4. Type-safe throughout

**Suggestion - Add parallel search:**

```rust
use rayon::prelude::*;

impl<'a> FretboardShape<'a> {
    pub fn search_chord_shapes_parallel(
        notes: &[Note],
        fretboard: &'a Fretboard
    ) -> Vec<Self>
    where
        Self: Send + Sync
    {
        let positions: Vec<Vec<(u8, u8)>> = notes.iter()
            .map(|note| Self::find_positions_for_note(note, fretboard))
            .collect();

        // First-level parallel branching
        if let Some(first_positions) = positions.first() {
            first_positions.par_iter()
                .flat_map(|&(string, fret)| {
                    let mut results = Vec::new();
                    if let Ok(sounded) = SoundedNote::fretted(string, fret, fretboard) {
                        let current = vec![FrettedNote::Sounded(sounded)];
                        Self::search_recursive(
                            fretboard,
                            &positions,
                            1,
                            current,
                            &mut results,
                        );
                    }
                    results
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
```

## String Numbering Convention

### Python

```python
# Confusing: internal 0-indexed, Lilypond wants opposite
@property
def ly_string_num(self):
    return self.fretboard.num_strings - self.string
```

### Rust

```rust
// Consistent: 0-indexed internally, conversion for output
impl<'a> SoundedNote<'a> {
    pub fn lilypond_string_number(&self) -> u8 {
        self.fretboard.num_strings() - self.string
    }
}
```

**Suggestion - Add string convention enum:**

```rust
#[derive(Debug, Clone, Copy)]
pub enum StringConvention {
    ZeroIndexedFromLow,  // Internal: 0 = lowest string
    OneIndexedFromHigh,  // Lilypond/TAB: 1 = highest string
}

impl<'a> SoundedNote<'a> {
    pub fn string_number(&self, convention: StringConvention) -> u8 {
        match convention {
            StringConvention::ZeroIndexedFromLow => self.string,
            StringConvention::OneIndexedFromHigh => {
                self.fretboard.num_strings() - self.string
            }
        }
    }
}
```

## Summary: Fretboard Comparison

| Feature | Python | Rust | Notes |
|---------|--------|------|-------|
| Fretboard type | Memoized class | Struct with lifetime | Rust prevents dangling refs |
| Note types | `FrettedPitch` (inheritance) | `SoundedNote`/`FrettedNote` | Rust uses composition |
| Muted strings | String `'x'` in list | Enum variant | Rust type-safe |
| Shape search | Recursive, buggy | Recursive with pruning | Rust more efficient |
| Validation | `is_playable()` + `validate()` | `is_playable()` + `classify()` | Rust uses enum |
| Tuning presets | `GTR_6STR_STD_TUNING` | `STD_6STR_GTR` + suggestions | Rust could add more |
| Error handling | Returns None | Returns Result | Rust explicit |
| String convention | Confusing conversion | Explicit conversion method | Both need improvement |
