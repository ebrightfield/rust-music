# Collections: PcSet, NoteSet, Voicing, and Related Types

The library provides several collection types for representing groups of pitches at different levels of abstraction.

## PcSet (Pitch Class Set)

**Location:** `music::note_collections::pc_set`

A PcSet represents a set of pitch classes - the foundation for chord and scale analysis. On construction, elements are:
1. Deduplicated
2. Sorted
3. Zeroed (normalized so the first element is Pc0)

```rust
use music::{PcSet, pcs, pc};

// Constructor normalizes the input
let major_triad = PcSet::new(vec![pc!(0), pc!(4), pc!(7)]);

// Macro shorthand
assert_eq!(pcs!(0, 4, 7), major_triad);

// From various sources
let from_u8: PcSet = vec![0u8, 4, 7].into();
```

### Rotation (Modes/Inversions)

Rotation reorients the set so a different element becomes Pc0:

```rust
use music::geometry::symmetry::transpositional::Modes;

let major = pcs!(0, 4, 7);

// Rotate forward (next mode/inversion)
assert_eq!(major.rotate(1), pcs!(0, 3, 8));  // First inversion
assert_eq!(major.rotate(2), pcs!(0, 5, 9));  // Second inversion

// Get all modes
let modes = major.modes();
assert_eq!(modes.len(), 3);
```

### Transposition Comparison

```rust
let c_major = pcs!(0, 4, 7);
let d_major = vec![pc!(2), pc!(6), pc!(9)];

// Check if one is a transposed version of another
assert!(c_major.is_transposed_version_of(&d_major));
```

### Spelling

Convert a PcSet to spelled notes using a root:

```rust
use music::Note;

let major7 = pcs!(0, 4, 7, 11);
let spelled = major7.try_spell(&Note::Bes).unwrap();
assert_eq!(spelled, vec![Note::Bes, Note::D, Note::F, Note::A]);

// Handles complex chords
let min7b5 = pcs!(0, 3, 6, 10);
let spelled = min7b5.try_spell(&Note::A).unwrap();
assert_eq!(spelled, vec![Note::A, Note::C, Note::Ees, Note::G]);
```

### Transpositional Symmetry

```rust
// Find symmetries (e.g., augmented triads have T4 symmetry)
let symmetries = pcs!(0, 4, 8).transpositional_symmetry();
// Returns a HashMap<Pc, HashSet<TranspositionalSymmetry>>
```

## NoteSet

**Location:** `music::note_collections`

A NoteSet is a PcSet with defined spellings. It wraps a `Vec<Note>` that is sorted and deduplicated on construction.

```rust
use music::{NoteSet, Note};

// Constructor deduplicates and sorts (C = Pc0 by default)
let notes = NoteSet::new(
    vec![Note::Ees, Note::G, Note::Ees, Note::C],
    None
);
assert_eq!(&*notes, &vec![Note::C, Note::Ees, Note::G]);

// Specify a different "zero point"
let from_g = NoteSet::new(
    vec![Note::C, Note::Ees, Note::G],
    Some(&Note::G)
);
// G is now treated as the "root"
```

### Stepping Through Notes

```rust
let c_minor = NoteSet::new(vec![Note::C, Note::Ees, Note::G], None);

// Step up through the chord
let mut note = c_minor[0];
note = c_minor.up_n_steps(&note, 1).unwrap();
assert_eq!(note, Note::Ees);
note = c_minor.up_n_steps(&note, 1).unwrap();
assert_eq!(note, Note::G);
note = c_minor.up_n_steps(&note, 1).unwrap();
assert_eq!(note, Note::C);  // Wraps around

// Step down
note = c_minor.down_n_steps(&Note::G, 2).unwrap();
assert_eq!(note, Note::C);
```

### Conversion to PcSet

```rust
let notes = NoteSet::new(vec![Note::C, Note::E, Note::G], None);
let pcs: PcSet = PcSet::from(&notes);
```

## Voicing

**Location:** `music::note_collections::voicing`

A Voicing is a collection of Pitches (notes with octaves) - a concrete arrangement that can be directly notated or played. Unlike PcSet, it allows duplicates and preserves ordering.

```rust
use music::{Voicing, voicing, pitch};

let v = Voicing::new(vec![
    pitch!(c, 4),
    pitch!(g, 4),
    pitch!(e, 5),
]);

// Macro shorthand
let v2 = voicing!(pitch!(c, 4), pitch!(g, 4), pitch!(e, 5));
```

### Span and Range

```rust
// Get lowest and highest pitches
let (lowest, highest) = v.span().unwrap();
assert_eq!(lowest, pitch!(c, 4));
assert_eq!(highest, pitch!(e, 5));
```

### Transposition

```rust
// Move by octaves (preserves spelling)
let higher = v.move_by_octaves(2).unwrap();

// Transpose by semitones
use music::geometry::symmetry::transpositional::TryTranspose;
let transposed = v.try_transpose(5).unwrap();
```

### Voice Leading

Apply chromatic paths to transform one voicing to another:

```rust
let c_major = voicing!(pitch!(c, 4), pitch!(e, 4), pitch!(g, 4));

// Move each voice: C stays, E goes up 1, G goes up 2
let f_major = c_major.apply_paths(
    &vec![0, 1, 2],
    Some(&vec![Note::C, Note::F, Note::A])  // Spelling context
).unwrap();
```

### Register Normalization

Optimize voicing placement for a specific clef:

```rust
use music::notation::clef::Clef;

let high_voicing = voicing!(pitch!(c, 7), pitch!(g, 7));
let normalized = high_voicing.normalize_register_to_clef(Clef::Treble).unwrap();
// Moves to a more readable register
```

### Construction from Intervals

Build a voicing from stacked intervals:

```rust
use music::StackedIntervals;

let intervals = StackedIntervals::new(vec![7, 4, 5]);  // P5, M3, P4
let voicing = Voicing::from_intervals(&pitch!(c, 4), &intervals).unwrap();
// Results in C4, G4, B4, E5
```

## StackedIntervals

**Location:** `music::note_collections::voicing`

Represents consecutive vertical intervals from low to high. This is a root-agnostic, spelling-agnostic representation of a voicing's shape.

```rust
use music::StackedIntervals;

// Intervals in semitones
let close_triad = StackedIntervals::new(vec![4, 3]);  // M3, m3 = major triad

// Check for wide intervals (octave or more)
assert!(!close_triad.has_wide_intervals());

let spread = StackedIntervals::new(vec![7, 12, 5]);
assert!(spread.has_wide_intervals());
```

### Conversion from Voicing

```rust
let v = voicing!(pitch!(c, 4), pitch!(e, 4), pitch!(g, 4));
let intervals: StackedIntervals = (&v).into();
assert_eq!(intervals.0, vec![4, 3]);
```

## OctavePartition

**Location:** `music::note_collections::octave_partition`

An ordered, cyclic series of intervals that sum to 12 (one octave). This is another way to represent a PcSet - by its interval content rather than absolute positions.

```rust
use music::OctavePartition;
use music::note_collections::interval_class::IntervalClass;

// Major triad: M3 + m3 + P4 = 4 + 3 + 5 = 12
let partition = OctavePartition::new(vec![
    IntervalClass::Ic4,
    IntervalClass::Ic3,
    IntervalClass::Ic5,
]).unwrap();

// Invalid partitions are rejected
let invalid = OctavePartition::new(vec![
    IntervalClass::Ic4,
    IntervalClass::Ic3,
    IntervalClass::Ic6,  // 4 + 3 + 6 = 13, not 12!
]);
assert!(invalid.is_err());
```

### Conversion

```rust
// PcSet → OctavePartition
let major = pcs!(0, 4, 7);
let partition = OctavePartition::from(&major);

// OctavePartition → PcSet
let back_to_pcs = PcSet::from(&partition);
```

## IntervalClass

**Location:** `music::note_collections::interval_class`

A mod-6 interval classification representing the smallest distance between two pitch classes (since any interval > 6 semitones has a smaller complement).

```rust
use music::IntervalClass;

// Intervals 0-6 semitones
let tritone = IntervalClass::Ic6;  // 6 semitones
let minor_second = IntervalClass::Ic1;  // 1 semitone
```

## Collection Relationships

```
                    ┌─────────────┐
                    │   Voicing   │ (concrete pitches)
                    └──────┬──────┘
                           │ loses octaves
                           ▼
                    ┌─────────────┐
                    │   NoteSet   │ (spelled notes)
                    └──────┬──────┘
                           │ loses spelling
                           ▼
┌─────────────────┐        ┌─────────────┐
│ OctavePartition │◄──────►│    PcSet    │ (abstract pitch classes)
│   (intervals)   │        └─────────────┘
└─────────────────┘
```

## Deref Behavior

All collection types dereference to their inner `Vec`, making iteration easy:

```rust
let major = pcs!(0, 4, 7);
major.iter().for_each(|pc| println!("{}", pc));

let v = voicing!(pitch!(c, 4), pitch!(e, 4));
v.iter().for_each(|p| println!("{}", p.midi_note));
```
