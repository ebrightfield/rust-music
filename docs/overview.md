# Overview & Quick Start

## What is Rust Music Semantics?

Rust Music Semantics is a music theory library focused on precision, correctness, and expressiveness. It provides:

1. **A powerful music theory engine** with precise types appropriate for all manner of situations
2. **Chord/scale inference** - Functions for inferring chord/scale spellings and names from pitch class sets
3. **Notation output** - Convert Rust types to Lilypond or VexTab source code for document generation or application development

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
music = { path = "path/to/music" }

# For Lilypond output support:
music = { path = "path/to/music", features = ["lilypond"] }
```

**Note:** This crate uses the nightly feature `concat_idents`. You may need to use `rustup run nightly cargo build`.

## Quick Start

### Pitch Classes (Pc)

The most abstract pitch representation - a mod-12 integer representing position in chromatic space:

```rust
use music::{Pc, pc};

let p = Pc::Pc0;  // Equivalent to "C" when mapped to letters
assert_eq!(p.next(), Pc::Pc1);
assert_eq!(p.previous(), Pc::Pc11);

// Distance calculations
assert_eq!(pc!(3).distance_up_to(&pc!(9)), 6);
assert_eq!(pc!(0).distance_down_to(&pc!(9)), 3);

// Transposition
use music::geometry::symmetry::transpositional::Transpose;
assert_eq!(pc!(7).transpose(7), pc!(2));
```

### Notes

Notes add spelling information (letter + accidental) to pitch classes:

```rust
use music::Note;

// Enharmonic calculations
assert_eq!(Note::Cis.enharmonic(), Note::Des);
assert!(Note::C.is_enharmonic(&Note::Bis));

// Diatonic distance
assert_eq!(Note::C.diatonic_distance_up(&Note::E), 2);

// Parse from strings
use std::str::FromStr;
let note = Note::from_str("C#").unwrap();
assert_eq!(note, Note::Cis);
```

### Pitches

Pitches add octave information to notes, mapping directly to MIDI:

```rust
use music::{Pitch, Note, pitch};

// Middle C = C4 = MIDI note 60
let p = Pitch::new(Note::C, 4).unwrap();
assert_eq!(p.midi_note, 60);

// Macro shorthand
assert_eq!(pitch!(fis, 4), Pitch::new(Note::Fis, 4).unwrap());

// Navigate to specific notes
assert_eq!(p.up_to_note(&Note::B).unwrap(), pitch!(b, 4));
assert_eq!(p.down_to_note(&Note::B).unwrap(), pitch!(b, 3));
```

### Pitch Class Sets (PcSet)

Collections of pitch classes for chord/scale analysis:

```rust
use music::{PcSet, pcs, pc, Note};
use music::geometry::symmetry::transpositional::{Modes, Transpose};

let major_triad = PcSet::new(vec![pc!(0), pc!(4), pc!(7)]);

// Get modes/inversions
let modes = major_triad.modes();
assert!(modes.contains(&pcs!(0, 3, 8)));  // First inversion

// Spell with a root note
let spelled = major_triad.try_spell(&Note::C).unwrap();
assert_eq!(spelled, vec![Note::C, Note::E, Note::G]);
```

### Voicings

Concrete pitch arrangements ready for notation:

```rust
use music::{Voicing, voicing, pitch};

let v = Voicing::new(vec![
    pitch!(c, 4),
    pitch!(g, 4),
    pitch!(e, 5),
]);

// Get span (lowest and highest)
let (lowest, highest) = v.span().unwrap();
assert_eq!(lowest, pitch!(c, 4));
assert_eq!(highest, pitch!(e, 5));

// Transpose by octaves
let higher = v.move_by_octaves(1).unwrap();
```

## Use Cases

### Tonal Analysis
Answer questions like which chords/scales fit into which others, find voice-leading paths between chords, find symmetries, iterate through possible voicings.

### Automated Scoring
Output Lilypond source code for document generation, or VexTab for integration with JS frontends.

### Application Development
Using WebAssembly, Rust integrates easily into browser-based frontend frameworks. VexTab provides a powerful way to generate music notation in a UI.

## Philosophy

This crate takes influence from Dmitri Tymoczko's work (*A Geometry of Music*) and endeavors to:

- Create a type system that enforces clear musical intent while remaining ergonomic
- Code solutions for sound music-theoretic inference, deduction, and search
- Solutions that apply over combinatorically complete spaces of musical possibilities
- Mathematically precise types accounting for various gradations of available information
- Easy-to-use API for converting musical objects into engraved notation
