# Primitives: Pc, Note, Pitch, and Spelling

The library provides a hierarchy of single-pitch types with increasing levels of specificity:

```
Pc (most abstract) → Note → Pitch (most concrete)
```

## Pc (Pitch Class)

**Location:** `music::note::pitch_class`

A pitch class is a mod-12 integer representing a position in chromatic space, agnostic to both spelling (letter name) and octave. Think of it as a position on a clock face with 12 positions.

```rust
use music::{Pc, pc};

// Pc is an enum with 12 variants
let p = Pc::Pc0;  // By convention, Pc0 = C

// Navigation
assert_eq!(p.next(), Pc::Pc1);      // Semitone up
assert_eq!(p.previous(), Pc::Pc11); // Semitone down

// The pc! macro provides shorthand
assert_eq!(pc!(11).next(), pc!(0));

// Distance calculations (mod-12)
assert_eq!(pc!(3).distance_up_to(&pc!(9)), 6);
assert_eq!(pc!(9).distance_up_to(&pc!(3)), 6);  // 9 → 10 → 11 → 0 → 1 → 2 → 3
assert_eq!(pc!(0).distance_down_to(&pc!(9)), 3);
```

### Transposition

```rust
use music::geometry::symmetry::transpositional::Transpose;

assert_eq!(pc!(7).transpose(7), pc!(2));   // 7 + 7 = 14 mod 12 = 2
assert_eq!(pc!(7).transpose(-7), pc!(0));  // 7 - 7 = 0
```

### Mapping to Notes

A single Pc maps to multiple possible Note spellings:

```rust
assert_eq!(Pc::Pc0.notes(), vec![Note::C, Note::Bis, Note::Deses]);
assert_eq!(Pc::Pc1.notes(), vec![Note::Cis, Note::Des]);
assert_eq!(Pc::Pc6.notes(), vec![Note::Fis, Note::Ges]);
```

### PcIter

Iterate through pitch classes:

```rust
use music::note::pitch_class::PcIter;

// All 12 pitch classes starting from Pc0
let all: Vec<Pc> = PcIter::default().collect();
assert_eq!(all.len(), 12);

// Starting from a different pitch class
let from_e: Vec<Pc> = PcIter::starting_on(&Pc::Pc4).collect();
assert_eq!(from_e.first(), Some(&Pc::Pc4));

// A section (exclusive of endpoint)
let section: Vec<Pc> = PcIter::section(&Pc::Pc4, &Pc::Pc3).collect();
assert_eq!(section.len(), 11);  // Pc4 through Pc2
```

## Note

**Location:** `music::note::note`

A Note represents a spelled pitch without octave information. It includes the letter name and accidental (sharp, flat, natural, double-sharp, double-flat).

### Naming Convention

Accidental suffixes come from Lilypond (which uses solfege naming):
- Flat (♭) = "es" (e.g., `Bes` = B♭)
- Sharp (♯) = "is" (e.g., `Fis` = F♯)
- Double-flat = "eses" (e.g., `Beses` = B♭♭)
- Double-sharp = "isis" (e.g., `Fisis` = F𝄪)

### Supported Spellings

The library supports all enharmonic spellings with these constraints:
- Nothing more extreme than double-accidentals
- No C or F flattened more than once (no Cbb, Fbb)
- No B or E sharpened more than once (no B##, E##)

```rust
use music::Note;

// All variants are enum members
let c_sharp = Note::Cis;
let d_flat = Note::Des;
let f_double_sharp = Note::Fisis;
```

### Enharmonic Operations

```rust
// Get enharmonic equivalent
assert_eq!(Note::Cis.enharmonic(), Note::Des);
assert_eq!(Note::Fisis.enharmonic(), Note::G);
assert_eq!(Note::C.enharmonic(), Note::C);  // Naturals unchanged

// Aggressive flip for B/C and E/F boundaries
assert_eq!(Note::C.enharmonic_flip_bcef(), Note::Bis);
assert_eq!(Note::B.enharmonic_flip_bcef(), Note::Ces);
assert_eq!(Note::E.enharmonic_flip_bcef(), Note::Fes);
assert_eq!(Note::F.enharmonic_flip_bcef(), Note::Eis);

// Check enharmonic equivalence
assert!(Note::Cis.is_enharmonic(&Note::Des));
```

### Distance Calculations

```rust
// Chromatic distance (semitones)
assert_eq!(Note::C.distance_up_to_note(&Note::E), 4);

// Diatonic distance (letter count, mod 7)
assert_eq!(Note::C.diatonic_distance_up(&Note::E), 2);  // C-D-E = 2 steps
```

### Spelling Control

```rust
// Re-spell a note using a palette of acceptable spellings
let palette = vec![Note::Cis, Note::E, Note::Gis];
let note = Note::Des.spelled_as_in(&palette).unwrap();
assert_eq!(note, Note::Cis);  // Des respelled as enharmonic Cis
```

### String Parsing

```rust
use std::str::FromStr;

assert_eq!(Note::from_str("C").unwrap(), Note::C);
assert_eq!(Note::from_str("C#").unwrap(), Note::Cis);
assert_eq!(Note::from_str("Bb").unwrap(), Note::Bes);
assert_eq!(Note::from_str("C##").unwrap(), Note::Cisis);
```

## Pitch

**Location:** `music::note::pitch`

A Pitch combines a Note with octave information, creating a concrete frequency that maps to MIDI.

### Construction

```rust
use music::{Pitch, Note, pitch};

// Direct construction (validates octave range)
let middle_c = Pitch::new(Note::C, 4).unwrap();
assert_eq!(middle_c.midi_note, 60);

// From MIDI note value
let also_middle_c = Pitch::from_midi(60).unwrap();
assert_eq!(also_middle_c.octave, 4);

// Macro shorthand (panics on invalid input)
assert_eq!(pitch!(fis, 4), Pitch::new(Note::Fis, 4).unwrap());
```

### MIDI Mapping

Middle C = C4 = MIDI note 60. `octave` is the scientific octave of the *written* letter, and the formula is:
`midi_note = (octave + 1) * 12 + natural letter semitone + alteration`
(C=0, D=2, E=4, F=5, G=7, A=9, B=11; ♭ = −1, ♯ = +1, double accidentals ±2).

Spellings that cross the C boundary therefore sound in the neighbouring MIDI octave block:

```rust
assert_eq!(Pitch::new(Note::Ces, 4).midi_note, 59); // C♭4 sounds as B3
assert_eq!(Pitch::new(Note::Bis, 3).midi_note, 60); // B♯3 sounds as C4
assert_eq!(Pitch::new(Note::Ces, 4).to_string(), "Cb4");

// Conversions from MIDI derive the octave from the chosen spelling
assert_eq!(Pitch::from_midi_as(71, Note::Ces).unwrap(), Pitch::new(Note::Ces, 5));
assert_eq!(Pitch::from_midi_spelled_as(60, &vec![Note::Bis]).unwrap().octave, 3);
```

`try_new` accepts octaves -1..=9 and rejects any pitch outside MIDI 0..=127 (e.g. C♭-1).
LilyPond (`ces'` = C♭4), VexTab, staff placement, and `Display` all use this written octave directly.

### Navigation

```rust
let c4 = pitch!(c, 4);

// Find next occurrence of a note above/below
assert_eq!(c4.up_to_note(&Note::B).unwrap(), pitch!(b, 4));
assert_eq!(c4.down_to_note(&Note::B).unwrap(), pitch!(b, 3));

// Transpose by semitones
use music::geometry::symmetry::transpositional::TryTranspose;
let transposed = pitch!(fis, 4).try_transpose(13).unwrap();
assert_eq!(transposed, pitch!(g, 5));

// Shift by octaves
assert_eq!(c4.raise_octaves(2).unwrap(), pitch!(c, 6));
```

### Distance Calculations

```rust
let c4 = pitch!(c, 4);

// Diatonic distance accounts for octaves
assert_eq!(c4.diatonic_distance(&pitch!(g, 5)), 11);
assert_eq!(c4.diatonic_distance(&pitch!(g, 2)), -10);
```

### Pitch Comparison

```rust
// Compare by MIDI value (ignores spelling)
assert!(pitch!(bes, 5).is_same_frequency(&pitch!(ais, 5)));
assert!(!pitch!(bes, 5).is_same_frequency(&pitch!(ais, 4)));  // Different octave
```

## Spelling

**Location:** `music::note::spelling`

The Spelling struct decomposes a Note into its constituent Letter and Accidental:

```rust
use music::note::spelling::{Spelling, Letter, Accidental};
use music::Note;

let spelling = Spelling::from(&Note::Cis);
assert_eq!(spelling.letter, Letter::C);
assert_eq!(spelling.acc, Accidental::Sharp);
```

### Letter

An enum of the seven musical letters (A through G):

```rust
use music::note::spelling::Letter;

let c = Letter::C;
assert_eq!(c.next(), Letter::D);
assert_eq!(c.prev(), Letter::B);

// Diatonic distance
assert_eq!(Letter::C.diatonic_distance_up(&Letter::E), 2);
```

### Accidental

```rust
use music::note::spelling::Accidental;

let acc = Accidental::Sharp;
assert!(!acc.is_double());

let double = Accidental::DoubleSharp;
assert!(double.is_double());

// Parse from strings
use std::str::FromStr;
assert_eq!(Accidental::from_str("#").unwrap(), Accidental::Sharp);
assert_eq!(Accidental::from_str("b").unwrap(), Accidental::Flat);
assert_eq!(Accidental::from_str("##").unwrap(), Accidental::DoubleSharp);
```

## Type Conversions Summary

| From | To | Method |
|------|-----|--------|
| `Note` | `Pc` | `Pc::from(&note)` |
| `Pc` | `Vec<Note>` | `pc.notes()` |
| `Note` | `Spelling` | `Spelling::from(&note)` |
| `Spelling` | `Note` | `Note::try_from(spelling)` |
| `Pitch` | `Note` | `pitch.note` (field access) |
| `MIDI u8` | `Pitch` | `Pitch::from_midi(60)` |
| `Note + octave` | `Pitch` | `Pitch::new(note, octave)` |
