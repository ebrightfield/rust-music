# Type System Comparison

This document provides a detailed comparison of the type systems between Python `pitch_set_lib` and Rust `music`.

## Pitch Class Representation

### Python: Integer-Based

```python
# Python: Pitch classes are plain integers
pc = 7  # G (when C=0)
transposed = (pc + 5) % 12  # Manual mod-12 arithmetic

# No type safety - any integer accepted
invalid = 15  # Silently wraps or causes bugs
```

**Issues:**
- No compile-time validation
- Easy to forget mod-12 arithmetic
- Mixing pitch classes with other integers

### Rust: Enum-Based

```rust
// Rust: Exhaustive enum for pitch classes
pub enum Pc {
    Pc0, Pc1, Pc2, Pc3, Pc4, Pc5,
    Pc6, Pc7, Pc8, Pc9, Pc10, Pc11,
}

impl Pc {
    pub fn transpose(&self, semitones: i8) -> Pc {
        // Always returns valid Pc - cannot create Pc13
    }
}

// Type-safe: Cannot confuse Pc with other integers
let pc = Pc::Pc7;
let transposed = pc.transpose(5);  // Always mod-12
```

**Rust Suggestion - Consider adding `From<u8>` with validation:**

```rust
impl TryFrom<u8> for Pc {
    type Error = MusicSemanticsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value % 12 {
            0 => Ok(Pc::Pc0),
            1 => Ok(Pc::Pc1),
            // ... etc
            _ => unreachable!()
        }
    }
}
```

## Note Representation

### Python: String-Based with Memoization

```python
class Note(metaclass=Memoized):
    def __init__(self, name):
        self.name = name.capitalize()
        # Parse into components
        self.letter = name[0].upper()
        self.acc = self._parse_accidental(name[1:])
        self.pc = self._calculate_pc()

    @classmethod
    def _mem(cls, name):
        return name.lower()  # Normalization key
```

**Python Issues:**
1. String parsing at runtime
2. Invalid note names only caught at runtime
3. Case sensitivity bugs possible
4. Memoization leaks memory

### Rust: Exhaustive Enum

```rust
pub enum Note {
    // Natural notes
    C, D, E, F, G, A, B,
    // Single sharps
    Cis, Dis, Eis, Fis, Gis, Ais, Bis,
    // Single flats
    Ces, Des, Es, Fes, Ges, As, Bes,
    // Double sharps
    Cisis, Disis, Eisis, Fisis, Gisis, Aisis, Bisis,
    // Double flats
    Ceses, Deses, Eses, Feses, Geses, Ases, Beses,
    // Restricted enharmonics (to avoid triple accidentals)
    // C cannot have double-flat (would be Bbb), etc.
}
```

**Rust Advantages:**
1. Invalid notes impossible at compile time
2. Pattern matching ensures exhaustive handling
3. Zero-cost - enum variants are integers
4. `FromStr` for parsing with proper error handling

```rust
impl FromStr for Note {
    type Err = MusicSemanticsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "c" => Ok(Note::C),
            "c#" | "cis" => Ok(Note::Cis),
            "db" | "des" => Ok(Note::Des),
            // ... comprehensive matching
            _ => Err(MusicSemanticsError::InvalidNoteLetter)
        }
    }
}
```

## Pitch Representation

### Python: Inherited Class

```python
class Pitch(Note, metaclass=Memoized):
    def __init__(self, name, octave=None):
        # Parse "C4" or ("C", 4)
        if octave is None:
            name, octave = self._parse_pitch_name(name)
        super().__init__(name)
        self.octave = octave
        self.midi_note = self._calculate_midi()

    @classmethod
    def from_midi(cls, midi_note, chord_context=None):
        # Reverse MIDI to pitch - spelling ambiguous
        pass
```

**Python Issues:**
1. MIDI bounds not validated (can create MIDI 200)
2. `from_midi` spelling is ambiguous without context
3. Inheritance from Note adds complexity

### Rust: Composition over Inheritance

```rust
pub struct Pitch {
    pub note: Note,      // Has-a relationship
    pub octave: u8,
    pub midi_note: u8,   // Cached, validated
}

impl Pitch {
    pub fn new(note: Note, octave: u8) -> Result<Self, MusicSemanticsError> {
        // Validate octave produces valid MIDI (0-127)
        let midi_note = Self::calculate_midi(&note, octave)?;
        if midi_note > 127 {
            return Err(MusicSemanticsError::MidiTooHigh);
        }
        Ok(Self { note, octave, midi_note })
    }

    pub fn from_midi(midi_note: u8) -> Result<Self, MusicSemanticsError> {
        // Returns Result, not arbitrary spelling
        // Default spelling based on pitch class
    }
}
```

**Rust Improvement Suggestion - Add spelling hint to `from_midi`:**

```rust
impl Pitch {
    /// Create pitch from MIDI with spelling hint
    pub fn from_midi_spelled(
        midi_note: u8,
        prefer_sharp: bool
    ) -> Result<Self, MusicSemanticsError> {
        let pc = Pc::from(midi_note % 12);
        let octave = midi_note / 12;
        let note = if prefer_sharp {
            pc.default_sharp_spelling()
        } else {
            pc.default_flat_spelling()
        };
        Self::new(note, octave)
    }
}
```

## Collection Types

### PcSet / Pitch Set

**Python:**
```python
# Python: Tuple of integers, manually sanitized
def pitch_set_sanitize(pcs):
    return tuple(sorted(set(pc % 12 for pc in pcs)))

# Usage - sanitization is optional/forgettable
chord_pcs = pitch_set_sanitize([0, 4, 7])
# or
chord_pcs = (0, 4, 7)  # Not sanitized!
```

**Rust:**
```rust
pub struct PcSet(Vec<Pc>);

impl PcSet {
    pub fn new(pcs: Vec<Pc>) -> Self {
        // ALWAYS: deduplicate, sort, zero
        let mut deduped: Vec<Pc> = pcs.into_iter().collect::<HashSet<_>>()
            .into_iter().collect();
        deduped.sort();
        // Zero to Pc0 (transpose so lowest is Pc0)
        // ... zeroing logic
        Self(deduped)
    }
}
```

**Key Difference:** Rust enforces invariants at construction - you cannot create an un-normalized `PcSet`.

### Voicing

**Python:**
```python
class Voicing(metaclass=Memoized):
    def __init__(self, midi_notes=None, vint_row=None, chord=None,
                 note_order=None, pitches=None):
        # Multiple construction paths - complex __init__
        if midi_notes:
            self.pitches = [Pitch.from_midi(m) for m in midi_notes]
        elif vint_row and chord:
            # Calculate from vertical interval row
        elif pitches:
            self.pitches = pitches
        # ... etc
```

**Rust:**
```rust
pub struct Voicing(Vec<Pitch>);

impl Voicing {
    pub fn new(pitches: Vec<Pitch>) -> Self {
        let mut sorted = pitches;
        sorted.sort_by_key(|p| p.midi_note);
        Self(sorted)
    }

    // Separate constructors for clarity
    pub fn from_intervals(
        root: Pitch,
        intervals: &[u8]
    ) -> Result<Self, MusicSemanticsError> {
        // Build from stacked intervals
    }
}
```

**Rust Advantage:** Clear, separate constructors instead of overloaded `__init__`.

### NoteSet (Rust-only)

Python doesn't have an explicit NoteSet type - spelled notes are typically `list[Note]`.

```rust
pub struct NoteSet(Vec<Note>);

impl NoteSet {
    pub fn new(notes: Vec<Note>, starting_note: Option<Note>) -> Self {
        // Sorts by Pc with orientation awareness
        // Deduplicates
    }

    pub fn up_n_steps(&self, from: &Note, n: usize) -> Result<Note, MusicSemanticsError> {
        // Circular stepping through the set
    }
}
```

**Suggestion for Rust - Add `From<&Voicing>` for NoteSet:**

```rust
impl From<&Voicing> for NoteSet {
    fn from(voicing: &Voicing) -> Self {
        let notes: Vec<Note> = voicing.0.iter()
            .map(|p| p.note.clone())
            .collect();
        NoteSet::new(notes, None)
    }
}
```

## Interval Representations

### Python: Ad-hoc

```python
# Python: Intervals as plain integers
int_row = (4, 3, 5)  # Major triad intervals

def int_row_to_pitch_set(int_row):
    from itertools import accumulate
    return (0,) + tuple(accumulate(int_row))[:-1]
```

### Rust: Dedicated Types

```rust
// IntervalClass: Unordered pitch interval (mod-12)
pub enum IntervalClass {
    Ic0, Ic1, Ic2, Ic3, Ic4, Ic5,
    Ic6, Ic7, Ic8, Ic9, Ic10, Ic11,
}

// OctavePartition: Intervals that sum to 12
pub struct OctavePartition(Vec<IntervalClass>);

impl OctavePartition {
    pub fn new(intervals: Vec<IntervalClass>) -> Result<Self, MusicSemanticsError> {
        let sum: u8 = intervals.iter().map(|ic| u8::from(ic)).sum();
        if sum != 12 {
            return Err(MusicSemanticsError::InvalidOctavePartition);
        }
        Ok(Self(intervals))
    }
}

// StackedIntervals: Root-agnostic semitone distances (no mod-12)
pub struct StackedIntervals(pub Vec<u8>);
```

**Rust Advantage:** Type system distinguishes between:
- `IntervalClass`: Mod-12 unordered intervals
- `OctavePartition`: Intervals that sum to exactly 12
- `StackedIntervals`: Raw semitone distances for voicings

## Error Type Comparison

### Python: Mixed Exceptions

```python
# Python: Various exception types, sometimes None returns
class Chord:
    def note_at_pc(self, pc):
        for note in self.spelling:
            if note.pc == pc:
                return note
        return None  # Silent failure

    def get_mode(self, degree):
        if degree >= len(self.pitch_set):
            raise IndexError("Invalid mode degree")
```

### Rust: Unified Error Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum MusicSemanticsError {
    // Range errors
    #[error("MIDI note {0} exceeds maximum 127")]
    MidiTooHigh(u8),

    #[error("Octave {0} too high for note")]
    OctaveTooHigh(u8),

    // Collection errors
    #[error("Empty set of notes")]
    EmptySetOfNotes,

    #[error("{0} is not a member of this collection")]
    NotAMember(String),

    // Spelling errors
    #[error("Cannot apply {0} accidental to {1}")]
    InvalidAccidental(String, String),

    // ... 41 variants total
}
```

**Rust Advantage:** Exhaustive error handling via pattern matching:

```rust
match voicing.apply_paths(paths, notes) {
    Ok(new_voicing) => { /* use it */ }
    Err(MusicSemanticsError::VoiceleadingViolation(rules)) => {
        println!("Violated rules: {:?}", rules);
    }
    Err(e) => return Err(e),
}
```

## Conversion Trait Implementations

### Python: Ad-hoc Methods

```python
# Python: Conversions scattered across classes
note = Note("C")
pc = note.pc  # Property access

pitch = Pitch("C4")
note = Note(pitch.name)  # Manual conversion

chord = Chord("CMaj7")
pcs = chord.pitch_set  # Property
```

### Rust: Trait-Based

```rust
// Rust: Standard conversion traits
impl From<&Note> for Pc {
    fn from(note: &Note) -> Pc {
        match note {
            Note::C | Note::Bis | Note::Deses => Pc::Pc0,
            Note::Cis | Note::Des => Pc::Pc1,
            // ... exhaustive
        }
    }
}

impl From<&Pitch> for Note {
    fn from(pitch: &Pitch) -> Note {
        pitch.note.clone()
    }
}

impl From<&Voicing> for PcSet {
    fn from(voicing: &Voicing) -> PcSet {
        let pcs: Vec<Pc> = voicing.0.iter()
            .map(|p| Pc::from(&p.note))
            .collect();
        PcSet::new(pcs)
    }
}
```

**Rust Advantage:** Consistent conversion patterns enable generic code:

```rust
fn analyze<T: Into<PcSet>>(input: T) {
    let pc_set = input.into();
    // Works with Voicing, Vec<Pc>, etc.
}
```

## Macro Support (Rust-only)

Python has no equivalent to Rust's compile-time macros for music types:

```rust
// Convenient shorthand with compile-time validation
let pc = pc!(7);           // Pc::Pc7
let note = note!(fis);     // Note::Fis
let pitch = pitch!(c, 4);  // Pitch { note: Note::C, octave: 4, midi: 60 }
let set = pcs!(0, 4, 7);   // PcSet([Pc0, Pc4, Pc7])
let v = voicing!(c 4, e 4, g 4);  // Voicing([Pitch...])
```

**Suggestion - Add validation macro:**

```rust
/// Validates pitch class set at compile time
macro_rules! validated_pcs {
    ($($pc:expr),+ $(,)?) => {{
        const _: () = {
            $(
                assert!($pc < 12, "Pitch class must be 0-11");
            )+
        };
        pcs!($($pc),+)
    }};
}
```

## Summary Table

| Aspect | Python | Rust | Winner |
|--------|--------|------|--------|
| Pitch class safety | Integer, unchecked | Enum, exhaustive | Rust |
| Note representation | String-parsed | Enum variants | Rust |
| Pitch validation | Optional | Required | Rust |
| Collection invariants | Manual | Automatic | Rust |
| Error handling | Mixed | Unified enum | Rust |
| Conversions | Ad-hoc methods | Traits | Rust |
| Memory model | Memoization | Ownership | Rust |
| Construction patterns | Overloaded `__init__` | Named constructors | Rust |
