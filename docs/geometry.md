# Geometry & Symmetry

The geometry module provides tools for analyzing the structural properties of pitch collections, including symmetries, modes, and transformations.

**Location:** `music::note_collections::geometry`

## Module Structure

```
geometry/
├── symmetry/
│   ├── transpositional.rs  - Transpose trait, modes, symmetry detection
│   ├── intervallic.rs      - Intervallic analysis
│   └── voiceleading.rs     - Voice leading calculations
├── sets.rs                 - Set operations
└── contour.rs              - Melodic contour analysis
```

## Transposition

**Location:** `music::note_collections::geometry::symmetry::transpositional`

### Transpose Trait

For types that can be transposed in mod-12 space:

```rust
pub trait Transpose: PartialEq + Eq + Hash + Sized {
    fn transpose(&self, semitones: i8) -> Self;
}
```

Implemented for:
- `Pc` - Pitch classes
- `Vec<Pc>` - Vectors of pitch classes
- `Note` - Spelled notes (may change spelling)
- `NoteSet` - Collections of notes

```rust
use music::{Pc, pc, Note, NoteSet};
use music::geometry::symmetry::transpositional::Transpose;

// Transpose a pitch class
assert_eq!(pc!(0).transpose(7), pc!(7));
assert_eq!(pc!(10).transpose(5), pc!(3));  // 10 + 5 = 15 mod 12 = 3

// Transpose a note (spelling follows simplest path)
assert_eq!(Note::C.transpose(6), Note::Fis);
assert_eq!(Note::A.transpose(5), Note::D);

// Transpose a vec of Pc
let pcs: Vec<Pc> = vec![pc!(0), pc!(4), pc!(7)].transpose(2);
assert_eq!(pcs, vec![pc!(2), pc!(6), pc!(9)]);
```

### TryTranspose Trait

For types with bounded range (can fail on overflow):

```rust
pub trait TryTranspose: PartialEq + Eq + Hash + Sized {
    fn try_transpose(&self, semitones: i8) -> Result<Self, MusicSemanticsError>;
}
```

Implemented for:
- `Pitch` - Can fail if result is out of MIDI range
- `Voicing` - Can fail if any pitch goes out of range

```rust
use music::{pitch, Voicing, voicing};
use music::geometry::symmetry::transpositional::TryTranspose;

// Transpose a pitch
let p = pitch!(c, 4).try_transpose(12).unwrap();
assert_eq!(p, pitch!(c, 5));

// Fails if out of range
let result = pitch!(c, 8).try_transpose(12);  // Would be C9
assert!(result.is_err());

// Transpose a voicing
let v = voicing!(pitch!(c, 4), pitch!(e, 4));
let higher = v.try_transpose(5).unwrap();
```

## Modes

**Location:** `music::note_collections::geometry::symmetry::transpositional`

### Modes Trait

For types that can be expressed as rotations (modes/inversions):

```rust
pub trait Modes: PartialEq + Sized {
    fn modes(&self) -> Vec<Self>;
    fn is_mode(&self, other: &Self) -> Option<usize>;
}
```

Implemented for `PcSet`:

```rust
use music::{PcSet, pcs};
use music::geometry::symmetry::transpositional::Modes;

let major = pcs!(0, 4, 7);

// Get all modes (rotations)
let modes = major.modes();
assert_eq!(modes.len(), 3);
assert!(modes.contains(&pcs!(0, 3, 8)));  // First inversion
assert!(modes.contains(&pcs!(0, 5, 9)));  // Second inversion

// Check if one set is a mode of another
assert_eq!(major.is_mode(&pcs!(0, 3, 8)), Some(1));  // First rotation
assert_eq!(major.is_mode(&pcs!(0, 5, 9)), Some(2));  // Second rotation
assert_eq!(major.is_mode(&pcs!(0, 4, 7)), Some(0));  // Same (zeroth rotation)
```

### PcSet Rotation

```rust
let major = pcs!(0, 4, 7);

// Rotate forward/backward
assert_eq!(major.rotate(1), pcs!(0, 3, 8));
assert_eq!(major.rotate(2), pcs!(0, 5, 9));
assert_eq!(major.rotate(-1), pcs!(0, 5, 9));  // Same as rotate(2)

// rotate_fwd / rotate_back helpers
assert_eq!(major.rotate_fwd(), pcs!(0, 3, 8));
assert_eq!(major.rotate_back(), pcs!(0, 5, 9));
```

## Transpositional Symmetry

Some pitch collections are symmetrical - they map to themselves when transposed by certain intervals less than an octave.

### TranspositionalSymmetry Enum

```rust
pub enum TranspositionalSymmetry {
    T2,  // Whole-tone scale symmetry (only whole-tone scales)
    T3,  // Minor third symmetry (dim7, etc.)
    T4,  // Major third symmetry (augmented triads, etc.)
    T6,  // Tritone symmetry (many chords/scales)
}
```

### Finding Symmetries

```rust
use music::note_collections::geometry::symmetry::transpositional::find_transpositional_symmetries;
use music::Pc::*;

// Tritone has T6 symmetry
let tritone = vec![Pc0, Pc6];
let symmetries = find_transpositional_symmetries(&tritone);
// Both Pc0 and Pc6 have T6 symmetry

// Diminished 7th has T3 and T6 symmetry
let dim7 = vec![Pc0, Pc3, Pc6, Pc9];
let symmetries = find_transpositional_symmetries(&dim7);
// All notes have both T3 and T6 symmetry

// Augmented triad has T4 symmetry
let aug = vec![Pc0, Pc4, Pc8];
let symmetries = find_transpositional_symmetries(&aug);
// All notes have T4 symmetry
```

### On PcSet

```rust
let augmented = pcs!(0, 4, 8);
let symmetry_map = augmented.transpositional_symmetry();
// Returns HashMap<Pc, HashSet<TranspositionalSymmetry>>
```

### On NoteSet

NoteSet provides symmetries indexed by Note instead of Pc:

```rust
use music::{NoteSet, Note};

let notes = NoteSet::new(vec![Note::C, Note::E, Note::Gis], None);
let symmetries = notes.find_transpositional_symmetries();
// Returns HashMap<Note, HashSet<TranspositionalSymmetry>>
```

## Symmetry Detection by Chord Size

The possible symmetries depend on the number of notes:

| Size | Possible Symmetries |
|------|---------------------|
| 2, 10 | T6 |
| 3, 9 | T4, T6 |
| 4, 8 | T3, T4, T6 |
| 6 | T2, T3, T4, T6 |

## Musical Implications

Transpositionally symmetrical chords are associated with:
- **Tonal ambiguity** - Multiple possible roots
- **Dissonance** - Often used for tension
- **Voice leading efficiency** - Can move to many destinations

### Examples

**Augmented triad (T4 symmetry):**
C+ = E+ = G#+ (all are the same chord)

**Diminished 7th (T3 symmetry):**
Cdim7 = E♭dim7 = G♭dim7 = Adim7 (all are the same chord)

**Whole-tone scale (T2 symmetry):**
Only two distinct whole-tone scales exist

## Set Operations

**Location:** `music::note_collections::geometry::sets`

Operations on pitch class sets (intersection, union, complement, etc.).

## Contour Analysis

**Location:** `music::note_collections::geometry::contour`

Tools for analyzing melodic contour patterns.

## Voice Leading

**Location:** `music::note_collections::geometry::symmetry::voiceleading`

The `Voicing` type supports voice leading operations:

```rust
use music::{Voicing, voicing, pitch, Note};

let c_major = voicing!(pitch!(c, 4), pitch!(e, 4), pitch!(g, 4));

// Apply chromatic voice leading paths
// Each number is semitones to move that voice
let paths = vec![0, 1, 2];  // C stays, E→F, G→A
let f_major = c_major.apply_paths(&paths, Some(&vec![Note::C, Note::F, Note::A])).unwrap();

assert_eq!(f_major, vec![pitch!(c, 4), pitch!(f, 4), pitch!(a, 4)]);
```

The optional `notes` parameter provides a spelling context for the resulting pitches.
