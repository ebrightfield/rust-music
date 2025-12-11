# Migration Guide: Python to Rust

This guide provides side-by-side code examples for porting from Python `pitch_set_lib` to Rust `music`.

## Basic Types

### Notes

**Python:**
```python
from pitch_set_lib.pitch_content import Note

# Create notes
c = Note("C")
c_sharp = Note("C#")
d_flat = Note("Db")

# Properties
print(c.pc)           # 0
print(c_sharp.pc)     # 1
print(c_sharp.letter) # "C"
print(c_sharp.acc)    # "#"

# Enharmonic
print(c_sharp.enharmonic)  # Note("Db")
print(c_sharp.is_enharmonic(d_flat))  # True (via equiv_class)

# Distance
print(c.distance(d_flat))  # 1
print(c.diat_distance(d_flat))  # 1 (diatonic)
```

**Rust:**
```rust
use music::{Note, Pc};
use std::str::FromStr;

// Create notes
let c = Note::C;
let c_sharp = Note::Cis;  // "Cis" = C#
let d_flat = Note::Des;   // "Des" = Db

// Or from strings
let c_sharp = Note::from_str("C#").unwrap();
let d_flat = Note::from_str("Db").unwrap();

// Properties
let pc: Pc = Pc::from(&c);           // Pc::Pc0
let pc_sharp: Pc = Pc::from(&c_sharp); // Pc::Pc1

// Enharmonic
let enharmonic = c_sharp.enharmonic();  // Note::Des
let is_enh = c_sharp.is_enharmonic(&d_flat);  // true

// Distance
let dist = c.distance_up_to_note(&d_flat);  // 1
let diat = c.diatonic_distance_up(&d_flat); // 1
```

### Pitches

**Python:**
```python
from pitch_set_lib.pitch_content import Pitch

# Create pitches
middle_c = Pitch("C4")
# or
middle_c = Pitch("C", 4)

# Properties
print(middle_c.midi_note)  # 60
print(middle_c.octave)     # 4

# From MIDI
pitch = Pitch.from_midi(60)  # Pitch("C4") - spelling may vary

# Operations
higher = middle_c.raise_octave()  # C5
lower = middle_c.reduce_octave()  # C3
```

**Rust:**
```rust
use music::{Pitch, Note, pitch};

// Create pitches
let middle_c = Pitch::new(Note::C, 4).unwrap();
// or using macro
let middle_c = pitch!(c, 4);

// Properties
assert_eq!(middle_c.midi_note, 60);
assert_eq!(middle_c.octave, 4);

// From MIDI
let pitch = Pitch::from_midi(60).unwrap();

// Operations
let higher = middle_c.raise_octaves(1).unwrap();
let lower = middle_c.clone().at_distance_from(-12).unwrap();

// Navigation
let next_g = middle_c.up_to_note(&Note::G).unwrap();  // G4
let prev_g = middle_c.down_to_note(&Note::G).unwrap(); // G3
```

### Pitch Classes

**Python:**
```python
# Python uses plain integers for pitch classes
pc = 7  # G
transposed = (pc + 5) % 12  # 0 (C)

# Distance
def distance(pc1, pc2):
    return (pc2 - pc1) % 12
```

**Rust:**
```rust
use music::{Pc, pc};
use music::geometry::symmetry::transpositional::Transpose;

// Rust uses enum
let pc = Pc::Pc7;  // G
let pc = pc!(7);   // macro shorthand

// Transposition
let transposed = pc.transpose(5);  // Pc::Pc0

// Distance
let dist = pc!(0).distance_up_to(&pc!(7));   // 7
let dist = pc!(0).distance_down_to(&pc!(7)); // 5

// Navigation
let next = pc.next();      // Pc::Pc8
let prev = pc.previous();  // Pc::Pc6
```

## Collections

### Pitch Sets

**Python:**
```python
from pitch_set_lib.pitch_content.chord_transformations import (
    pitch_set_sanitize,
    pitch_set_to_int_row,
    rotate_pitch_set,
    modes_from_pitch_set
)

# Create and normalize
pcs = pitch_set_sanitize([0, 4, 7])  # (0, 4, 7)

# Interval row
int_row = pitch_set_to_int_row(pcs)  # (4, 3, 5)

# Modes
mode1 = rotate_pitch_set(pcs, 1)  # (0, 3, 8)
all_modes = modes_from_pitch_set(pcs)  # [(0,4,7), (0,3,8), (0,5,9)]
```

**Rust:**
```rust
use music::{PcSet, pcs, pc};
use music::geometry::symmetry::transpositional::Modes;
use music::OctavePartition;

// Create (auto-normalized)
let pc_set = PcSet::new(vec![pc!(0), pc!(4), pc!(7)]);
// or macro
let pc_set = pcs!(0, 4, 7);

// Interval row (via OctavePartition)
let partition: OctavePartition = (&pc_set).into();

// Modes
let mode1 = pc_set.rotate(1);
let all_modes = pc_set.modes();

// Check if another set is a mode
let minor = pcs!(0, 3, 7);
let is_mode = pc_set.is_mode(&minor);  // Some(1)
```

### Voicings

**Python:**
```python
from pitch_set_lib.pitch_content import Voicing, Chord

# From MIDI notes
v = Voicing(midi_notes=[60, 64, 67])  # C4, E4, G4

# From chord
chord = Chord("CMaj")
v = Voicing.from_chord(chord, note_order=[1, 3, 5])

# Properties
print(v.pitches)    # [Pitch("C4"), Pitch("E4"), Pitch("G4")]
print(v.vint_row)   # (4, 3) - intervals between adjacent notes
print(v.bass_note)  # Pitch("C4")

# Get all voicings
all_voicings = Voicing.get_voicings(chord)
```

**Rust:**
```rust
use music::{Voicing, Pitch, voicing, pitch};
use music::StackedIntervals;

// From pitches
let v = Voicing::new(vec![
    pitch!(c, 4),
    pitch!(e, 4),
    pitch!(g, 4),
]);
// or macro
let v = voicing!(c 4, e 4, g 4);

// From intervals
let v = Voicing::from_intervals(
    pitch!(c, 4),
    &[4, 3]  // Semitone distances
).unwrap();

// Properties
let span = v.span().unwrap();  // (lowest, highest)
let intervals: StackedIntervals = (&v).into();

// Move by octaves
let higher = v.move_by_octaves(1).unwrap();

// Normalize to clef
use music::notation::Clef;
let normalized = v.normalize_register_to_clef(Clef::Treble).unwrap();
```

### Note Sets (Rust only)

```rust
use music::NoteSet;

// Spelled collection without octave
let notes = NoteSet::new(
    vec![Note::C, Note::E, Note::G],
    Some(Note::C)  // Starting note for orientation
);

// Navigate within set
let next_from_e = notes.up_n_steps(&Note::E, 1).unwrap();  // G
let prev_from_e = notes.down_n_steps(&Note::E, 1).unwrap(); // C
```

## Chords and Naming

### Python

```python
from pitch_set_lib.pitch_content import Chord

# From name
chord = Chord("CMaj7")
print(chord.root)       # Note("C")
print(chord.pitch_set)  # (0, 4, 7, 11)
print(chord.quality)    # "Maj7"
print(chord.spelling)   # [Note("C"), Note("E"), Note("G"), Note("B")]

# From pitch set
chord = Chord(root="C", pitch_set=(0, 4, 7, 11))
print(chord.name)  # "CMaj7"

# Modes
modes = chord.modes()
mode2 = chord.get_mode(1)  # First inversion

# Subchords
subchords = chord.get_subchords(size=3)
```

### Rust

```rust
use music::{PcSet, Note, pcs};
use music::chord_name::{ChordName, ChordQuality, TonalSpecification};

// Create from pitch set + root
let pc_set = pcs!(0, 4, 7, 11);
let spelled = pc_set.try_spell(&Note::C).unwrap();
// spelled = [Note::C, Note::E, Note::G, Note::B]

// Chord name (work in progress - see naming_heuristics)
let name = ChordName {
    tonality: TonalSpecification::RootPosition(Note::C),
    quality: ChordQuality::maj7(),  // Helper constructor
    pc_set: pc_set.clone(),
};

// Modes
use music::geometry::symmetry::transpositional::Modes;
let modes = pc_set.modes();
let mode2 = pc_set.rotate(1);

// Subchords
use music::geometry::sets::get_subchords;
let subchords = get_subchords(&pc_set, 3).unwrap();
```

## Fretboard

### Python

```python
from pitch_set_lib.pitch_content import Fretboard, FrettedPitch
from pitch_set_lib.pitch_content.gtr_shapes import GtrShape

# Standard guitar
fb = Fretboard()  # Default 6-string standard tuning

# Get note at position
note = fb.note_at(0, 5)  # 6th string, 5th fret = A

# FrettedPitch
fp = FrettedPitch(
    pitch=Pitch("A2"),
    string=0,
    fret=5,
    fretboard=fb
)

# Navigate
next_note = fp.next_note_same_string(chord)
up_string = fp.up_a_string()

# Find shapes
from pitch_set_lib.pitch_content import Voicing, Chord
chord = Chord("CMaj")
voicing = Voicing.from_chord(chord)
shapes = GtrShape.find_voicing_shapes(voicing, fb)
```

### Rust

```rust
use music::fretboard::{Fretboard, SoundedNote, FrettedNote, FretboardShape, STD_6STR_GTR};

// Standard guitar
let fb = &*STD_6STR_GTR;

// Get note at position
let note = fb.sounded_note(0, 5).unwrap();  // String 0, fret 5

// SoundedNote
let sn = SoundedNote::fretted(0, 5, fb).unwrap();
assert_eq!(sn.pitch.note, Note::A);

// Navigate
let next = sn.up_n_frets(2).unwrap();
let next_in_chord = sn.next_note_same_string(&[Note::C, Note::E, Note::G]).unwrap();

// Muted notes
let muted = FrettedNote::muted(1, fb).unwrap();

// Spell note in context
let spelled = sn.spelled_as_in(&[Note::A, Note::Cis, Note::E]).unwrap();

// Find shapes
let shapes = FretboardShape::search_chord_shapes(
    &[Note::C, Note::E, Note::G],
    fb
);

// Check playability
for shape in shapes {
    if shape.is_playable() {
        let class = shape.classify();
        println!("{:?}", class);
    }
}
```

## Voiceleading

### Python

```python
from pitch_set_lib.pitch_content import Voiceleading, Voicing, Chord

v1 = Voicing(midi_notes=[60, 64, 67])  # C major
ch2 = Chord("FMaj")

# Generate all voiceleadings
voiceleadings = Voiceleading.gen(v1, ch2)

# Best (smoothest)
best = voiceleadings[0]
print(best.naive_distance)  # Sum of voice movements
print(best.paths)           # Movement for each voice
```

### Rust

```rust
use music::{Voicing, Note, voicing, pitch};
use music::geometry::symmetry::voiceleading::{Voiceleading, NoVoxCrossings};

let v1 = voicing!(c 4, e 4, g 4);
let destination_notes = vec![Note::F, Note::A, Note::C];

// Generate all voiceleadings with no voice crossings rule
let voiceleadings = Voiceleading::find_all(
    &v1,
    &destination_notes,
    &[NoVoxCrossings],
).unwrap();

// Best (smoothest) - already sorted by score
if let Some((score, best)) = voiceleadings.first() {
    println!("Distance: {}", score);
    println!("Paths: {:?}", best.paths);
}
```

## Notation Output

### Python (Lilypond only)

```python
from pitch_set_lib.typesetting_tools.ly_tools import ly_to_file, ly_compile

# Objects have .ly property
note = Note("C#")
print(note.ly)  # "cis"

pitch = Pitch("C#4")
print(pitch.ly)  # "cis'"

voicing = Voicing(midi_notes=[60, 64, 67])
print(voicing.ly)  # "<c' e' g'>"

# Compile to PDF
ly_to_file(content, "output.ly")
ly_compile("output.ly")
```

### Rust (Lilypond + VexTab)

```rust
use music::{Note, Pitch, Voicing, pitch, voicing};

// Lilypond (requires "lilypond" feature)
#[cfg(feature = "lilypond")]
{
    use music::notation::lilypond::ToLilypondString;

    let note = Note::Cis;
    println!("{}", note.to_lilypond_string());  // "cis"

    let pitch = pitch!(cis, 4);
    println!("{}", pitch.to_lilypond_string());  // "cis'"

    let v = voicing!(c 4, e 4, g 4);
    println!("{}", v.to_lilypond_string());  // "<c' e' g'>"

    // Full document generation
    use music::notation::lilypond::{Document, Score, Staff};
    let doc = Document::new()
        .add_section(Score::new().add_staff(Staff::new()));
    let content = doc.render().unwrap();
}

// VexTab (always available)
use music::notation::vextab::ToVextabString;

let pitch = pitch!(c, 4);
println!("{}", pitch.to_vextab_string());  // "C/4"

let v = voicing!(c 4, e 4, g 4);
println!("{}", v.to_vextab_string());  // "(C/4.E/4.G/4)"
```

## Rhythm

### Python

```python
from pitch_set_lib.rhythm import Rhythm
from fractions import Fraction

# Create rhythm
r = Rhythm(Fraction(1, 4))  # Quarter note
print(r.ly)  # "4"

# Dotted
r_dotted = Rhythm(Fraction(3, 8))  # Dotted quarter
print(r_dotted.ly)  # "4."

# With associated content
r_with_note = Rhythm(Fraction(1, 4), Pitch("C4"))
```

### Rust

```rust
use music::notation::rhythm::{Duration, DurationKind, RhythmicNotatedEvent, SingleEvent, NotatedEvent};
use music::{Pitch, pitch};

// Create duration
let quarter = Duration::new(DurationKind::Qtr, 0);  // Quarter, no dots
let dotted_quarter = Duration::new(DurationKind::Qtr, 1);  // One dot

// Check ticks (internal time unit)
assert_eq!(quarter.ticks(), 32);
assert_eq!(dotted_quarter.ticks(), 48);

// With content
let event = RhythmicNotatedEvent {
    tied: false,
    event: NotatedEvent::SingleEvent(
        SingleEvent::Pitch(pitch!(c, 4)),
        quarter,
    ),
};

// Tuplets
use music::notation::rhythm::Tuplet;
let triplet = Tuplet {
    events: vec![/* three eighth notes */],
    numerator: 3,
    denominator: 2,
    base_unit: DurationKind::Eighth,
};
```

## Symmetry Analysis

### Python

```python
from pitch_set_lib.pitch_content.chord_transformations import is_symmetrical

pcs = (0, 4, 8)  # Augmented triad
symmetry = is_symmetrical(pcs)  # 4 (T4 symmetry)

pcs = (0, 3, 6, 9)  # Diminished 7th
symmetry = is_symmetrical(pcs)  # 3 (T3 symmetry)
```

### Rust

```rust
use music::{PcSet, pcs};

let aug = pcs!(0, 4, 8);
let symmetries = aug.transpositional_symmetry();
// HashMap<Pc, HashSet<TranspositionalSymmetry>>
// All PCs have T4 symmetry

let dim7 = pcs!(0, 3, 6, 9);
let symmetries = dim7.transpositional_symmetry();
// All PCs have T3 symmetry

// Check for any symmetry
use music::geometry::symmetry::transpositional::TranspositionalSymmetry;
let has_t4 = symmetries.values()
    .any(|s| s.contains(&TranspositionalSymmetry::T4));
```

## Error Handling

### Python

```python
# Python uses exceptions or None returns
try:
    pitch = Pitch("C99")  # Invalid octave
except ValueError:
    pass

note = chord.note_at_pc(5)
if note is None:
    # Handle missing note
    pass
```

### Rust

```rust
use music::{Pitch, Note, MusicSemanticsError};

// Rust uses Result types
match Pitch::new(Note::C, 99) {
    Ok(pitch) => { /* use pitch */ }
    Err(MusicSemanticsError::OctaveTooHigh(oct)) => {
        println!("Octave {} is too high", oct);
    }
    Err(e) => { /* handle other errors */ }
}

// Or with ? operator in functions returning Result
fn process() -> Result<(), MusicSemanticsError> {
    let pitch = Pitch::new(Note::C, 4)?;
    let higher = pitch.raise_octaves(10)?;  // Might fail
    Ok(())
}
```

## Common Patterns

### Iterating Through All Notes

**Python:**
```python
from pitch_set_lib.pitch_content.CONSTANTS import CHR_SCALE

for pc in range(12):
    notes = CHR_SCALE[pc]
    for note_name in notes:
        note = Note(note_name)
        print(note.name, note.pc)
```

**Rust:**
```rust
use music::{Pc, Note};

// All pitch classes
for pc in [Pc::Pc0, Pc::Pc1, /* ... */] {
    // Get possible spellings
    let notes = pc.notes();
    for note in notes {
        println!("{:?} -> {:?}", note, Pc::from(&note));
    }
}

// Or iterate all Note variants
use strum::IntoEnumIterator;
for note in Note::iter() {
    let pc = Pc::from(&note);
    println!("{:?} -> {:?}", note, pc);
}
```

### Building Chord Progressions

**Python:**
```python
chords = [
    Chord("CMaj7"),
    Chord("Am7"),
    Chord("Dm7"),
    Chord("G7"),
]

voicings = []
prev_voicing = None
for chord in chords:
    if prev_voicing is None:
        v = Voicing.from_chord(chord)
    else:
        vls = Voiceleading.gen(prev_voicing, chord)
        v = vls[0].v2  # Smoothest voiceleading
    voicings.append(v)
    prev_voicing = v
```

**Rust:**
```rust
use music::{PcSet, Note, Voicing, pcs, voicing, pitch};
use music::geometry::symmetry::voiceleading::{Voiceleading, NoVoxCrossings};

let progressions = vec![
    (Note::C, pcs!(0, 4, 7, 11)),   // CMaj7
    (Note::A, pcs!(0, 3, 7, 10)),   // Am7
    (Note::D, pcs!(0, 3, 7, 10)),   // Dm7
    (Note::G, pcs!(0, 4, 7, 10)),   // G7
];

let mut voicings = Vec::new();
let mut prev_voicing: Option<Voicing> = None;

for (root, pc_set) in progressions {
    let notes = pc_set.try_spell(&root).unwrap();

    let voicing = if let Some(ref prev) = prev_voicing {
        // Find smooth voiceleading
        let vls = Voiceleading::find_all(prev, &notes, &[NoVoxCrossings]).unwrap();
        vls.first().map(|(_, vl)| vl.destinations.clone())
            .unwrap_or_else(|| Voicing::from_notes_octave(&notes, 4).unwrap())
    } else {
        // First chord - simple voicing
        Voicing::from_notes_octave(&notes, 4).unwrap()
    };

    voicings.push(voicing.clone());
    prev_voicing = Some(voicing);
}
```

## Summary: Key Differences to Remember

| Python | Rust |
|--------|------|
| `Note("C#")` | `Note::Cis` or `Note::from_str("C#")?` |
| `Pitch("C4")` | `Pitch::new(Note::C, 4)?` |
| `pc % 12` | `pc.transpose(n)` |
| `pitch_set_sanitize(pcs)` | `PcSet::new(pcs)` (auto) |
| `chord.spelling` | `pc_set.try_spell(&root)?` |
| `Fretboard()` | `&*STD_6STR_GTR` |
| `note.ly` | `note.to_lilypond_string()` |
| `raise ValueError` | `Err(MusicSemanticsError::...)` |
| `return None` | `return Err(...)` |
| `for x in list:` | `for x in vec.iter()` |
| Memoization | Ownership + Clone |
