# Fretboard Module

The fretboard module provides types for modeling fretted instruments (guitar, bass, etc.) and searching for chord/melodic shapes.

**Location:** `music::fretboard`

## Module Structure

```
fretboard/
├── mod.rs            - Fretboard type and STD_6STR_GTR
├── fretted_note.rs   - FrettedNote and SoundedNote types
└── fretboard_shape/
    ├── mod.rs              - FretboardShape type
    ├── chord_shape_search.rs    - Chord shape search algorithms
    └── melodic_shape_search.rs  - Melodic pattern search
```

## Fretboard

The `Fretboard` struct represents any fretted instrument with arbitrary tuning and number of strings:

```rust
use music::fretboard::Fretboard;
use music::{Pitch, Note};

// Custom tuning
let drop_d = Fretboard {
    open_strings: vec![
        Pitch::new(Note::D, 3).unwrap(),  // Low string (thickest)
        Pitch::new(Note::A, 3).unwrap(),
        Pitch::new(Note::D, 4).unwrap(),
        Pitch::new(Note::G, 4).unwrap(),
        Pitch::new(Note::B, 4).unwrap(),
        Pitch::new(Note::E, 5).unwrap(),  // High string (thinnest)
    ],
};
```

### Standard Guitar Tuning

A predefined standard 6-string guitar tuning is available:

```rust
use music::fretboard::STD_6STR_GTR;

// E-A-D-G-B-E standard tuning
let guitar = &*STD_6STR_GTR;
```

### String Indexing Convention

**Important:** Strings are indexed from 0, where `open_strings[0]` is the **thickest** string (low E on standard guitar). This is the opposite of common guitar terminology where "the 6th string" refers to the thickest string.

```rust
let guitar = &*STD_6STR_GTR;

// Index 0 = thickest string (low E)
assert_eq!(guitar.get_string(0).unwrap().note, Note::E);
// Index 5 = thinnest string (high E)
assert_eq!(guitar.get_string(5).unwrap().note, Note::E);
```

### Fretboard Methods

```rust
let guitar = &*STD_6STR_GTR;

// Number of strings
assert_eq!(guitar.num_strings(), 6);

// Get a string (returns the open string pitch)
let third_string = guitar.get_string(2).unwrap();
assert_eq!(third_string.note, Note::D);

// Find where a note is on a string
let fret = guitar.which_fret(&Note::Fis, 0).unwrap();  // F# on low E
assert_eq!(fret, 2);

// Create a sounded note at a position
let note = guitar.sounded_note(0, 3).unwrap();  // 3rd fret of low E
assert_eq!(note.pitch.note, Note::G);

// Get a note on a specific string
let f_on_low_e = guitar.note_on_string(&Note::F, 0).unwrap();
assert_eq!(f_on_low_e.fret, 1);
```

## FrettedNote and SoundedNote

**Location:** `music::fretboard::fretted_note`

### SoundedNote

A note that is actually sounded (not muted):

```rust
pub struct SoundedNote<'a> {
    pub string: u8,           // String index (0 = thickest)
    pub fret: u8,             // Fret number (0 = open)
    pub pitch: Pitch,         // The resulting pitch
    pub fretboard: &'a Fretboard,
}
```

### FrettedNote

An enum that can be either sounded or muted:

```rust
pub enum FrettedNote<'a> {
    Sounded(SoundedNote<'a>),
    Muted { string: u8, fretboard: &'a Fretboard },
}
```

Usage:

```rust
let guitar = &*STD_6STR_GTR;

// Check if sounded
let note = guitar.sounded_note(0, 5).unwrap();
let fretted = FrettedNote::Sounded(note);
assert!(fretted.is_sounded());

// Get pitch (only for sounded notes)
if let Some(pitch) = fretted.pitch() {
    println!("Pitch: {}", pitch);
}
```

### Spelling Control

```rust
let note = guitar.sounded_note(0, 6).unwrap();  // F#/Gb
let spelled = note.spelled_as_in(&vec![Note::Ges]).unwrap();
assert_eq!(spelled.pitch.note, Note::Ges);
```

## FretboardShape

**Location:** `music::fretboard::fretboard_shape`

A `FretboardShape` represents a vertical arrangement of fretted notes - a chord voicing on the fretboard:

```rust
pub struct FretboardShape<'a> {
    pub fretted_notes: Vec<FrettedNote<'a>>,
    pub fretboard: &'a Fretboard,
}
```

### Shape Properties

```rust
// Display format: fret numbers joined by dashes
// e.g., "x-3-2-0-1-0" for a standard C chord
println!("{}", shape);

// Number of sounded strings
let size = shape.size();

// Lowest and highest fret numbers
let (min_fret, max_fret) = shape.span();

// Lowest and highest pitches
let (low_pitch, high_pitch) = shape.range();

// Check for open strings
if shape.contains_open_strings() {
    println!("Contains open strings");
}

// Check for wide intervals
if shape.has_wide_intervals() {
    println!("Has intervals >= octave");
}
```

### Playability

```rust
// Check if physically playable
if shape.is_playable() {
    println!("This shape is playable");
}

// Get detailed classification
match shape.classify() {
    ChordShapeClassification::Playable => println!("Easy to play"),
    ChordShapeClassification::AllAbove12thFret => println!("High position"),
    ChordShapeClassification::NonTransposable => println!("Uses open strings specially"),
    ChordShapeClassification::Unplayable => println!("Too wide a stretch"),
}
```

Playability is determined by the span (distance between lowest and highest fret):
- 3 or fewer notes: max span of 4 frets
- More than 3 notes: max span of 3 frets

### Open String Handling

```rust
// Get version with open strings converted to mutes
// Useful for analyzing transposability
let moveable = shape.without_open_strings();
```

### Conversion to Other Types

```rust
use music::{Voicing, StackedIntervals};

// Convert to a Voicing
let voicing: Voicing = (&shape).into();

// Convert to StackedIntervals
let intervals: StackedIntervals = (&shape).into();
```

## Chord Shape Search

**Location:** `music::fretboard::fretboard_shape::chord_shape_search`

Algorithms for finding chord voicings on the fretboard.

## Melodic Shape Search

**Location:** `music::fretboard::fretboard_shape::melodic_shape_search`

Algorithms for finding scale patterns and melodic shapes.

## Constants

```rust
// Maximum fret (allowing for three-octave searches)
const MAX: u8 = 35;

// Open string constant
const OPEN: u8 = 0;
```

## Example: Building a Chord Shape

```rust
use music::fretboard::{STD_6STR_GTR, FretboardShape};
use music::fretboard::fretted_note::FrettedNote;

let guitar = &*STD_6STR_GTR;

// Build a C major chord manually
let c_chord = FretboardShape {
    fretboard: guitar,
    fretted_notes: vec![
        FrettedNote::Muted { string: 0, fretboard: guitar },
        FrettedNote::Sounded(guitar.sounded_note(1, 3).unwrap()),  // C
        FrettedNote::Sounded(guitar.sounded_note(2, 2).unwrap()),  // E
        FrettedNote::Sounded(guitar.sounded_note(3, 0).unwrap()),  // G
        FrettedNote::Sounded(guitar.sounded_note(4, 1).unwrap()),  // C
        FrettedNote::Sounded(guitar.sounded_note(5, 0).unwrap()),  // E
    ],
};

println!("C chord: {}", c_chord);  // x-3-2-0-1-0
assert!(c_chord.is_playable());
```

## ChordShapeClassification

```rust
pub enum ChordShapeClassification {
    Playable,           // Within comfortable reach
    Unplayable,         // Too wide a stretch
    AllAbove12thFret,   // Playable but in high position
    NonTransposable,    // Uses open strings in a non-moveable way
}
```
