# Overview: Architecture Comparison

## Design Philosophy

### Python `pitch_set_lib`

The Python library uses a **memoization-centric** approach where musical objects are cached globally and reused via identity checks. This provides convenience but introduces subtle bugs around enharmonic equivalence and memory management.

```python
# Python: Memoized metaclass ensures singleton behavior
class Note(metaclass=Memoized):
    @classmethod
    def _mem(cls, name):
        return name.lower()  # Case-insensitive key
```

### Rust `music`

The Rust library uses an **enum-centric** approach where pitch classes and notes are exhaustively enumerated variants. This provides compile-time guarantees and eliminates entire classes of bugs.

```rust
// Rust: Enum variants are distinct types
pub enum Pc { Pc0, Pc1, Pc2, Pc3, Pc4, Pc5, Pc6, Pc7, Pc8, Pc9, Pc10, Pc11 }
pub enum Note { C, Cis, Des, D, Dis, Es, /* ... 53 total variants */ }
```

## Module Structure Comparison

### Python Structure

```
pitch_set_lib/
├── pitch_content/
│   ├── note.py           # Note class (~310 lines)
│   ├── pitch.py          # Pitch class (~296 lines)
│   ├── chord.py          # Chord class (~370 lines)
│   ├── voicing.py        # Voicing class (~142 lines)
│   ├── voiceleading.py   # Voiceleading (~98 lines)
│   ├── fretboard.py      # Fretboard, FrettedPitch (~668 lines)
│   ├── gtr_shapes.py     # GtrShape (~230 lines)
│   ├── chord_transformations.py  # Pitch set theory (~300 lines)
│   ├── name_gen.py       # Chord naming (~700 lines)
│   ├── name_parser.py    # Name parsing (~400 lines)
│   ├── CONSTANTS.py      # Lookup tables (~540 lines)
│   └── utils.py          # Memoized metaclass (~35 lines)
├── typesetting_tools/
│   └── ly_tools.py       # Lilypond output
└── rhythm.py             # Duration handling
```

### Rust Structure

```
music/
├── note/
│   ├── note.rs           # Note enum (53 variants)
│   ├── pitch_class.rs    # Pc enum (12 variants)
│   ├── pitch.rs          # Pitch struct
│   └── spelling.rs       # Letter + Accidental decomposition
├── note_collections/
│   ├── pc_set.rs         # PcSet (normalized pitch class sets)
│   ├── voicing.rs        # Voicing + StackedIntervals
│   ├── spelling.rs       # Context-aware spelling rules
│   ├── interval_class.rs # IntervalClass (mod-12 intervals)
│   ├── octave_partition.rs  # Intervals summing to 12
│   ├── chord_name/
│   │   ├── quality/      # ChordQuality, ScaleQuality
│   │   └── naming_heuristics/  # Inference algorithms
│   └── geometry/
│       ├── sets.rs       # Subchord enumeration
│       ├── contour.rs    # Melodic contour
│       └── symmetry/     # Transpositional, intervallic, voiceleading
├── fretboard/
│   ├── fretted_note.rs   # FrettedNote enum, SoundedNote struct
│   └── fretboard_shape/  # Shape search algorithms
├── notation/
│   ├── vextab.rs         # VexTab output (NEW)
│   ├── rhythm/           # Duration, meter handling
│   └── lilypond/         # Lilypond document generation
└── error.rs              # MusicSemanticsError (41 variants)
```

## Key Architectural Differences

### 1. Error Handling

**Python**: Uses exceptions and silent failures
```python
# Python: May return None or raise AttributeError
def note_at_pc(self, pc):
    for note in self.spelling:
        if note.pc == pc:
            return note
    return None  # Silent failure
```

**Rust**: Uses explicit `Result` types
```rust
// Rust: Caller must handle the error case
pub fn try_spell(&self, root: &Note) -> Result<Vec<Note>, MusicSemanticsError> {
    // Returns Err if spelling rules fail
}
```

### 2. Collection Invariants

**Python**: Invariants maintained by convention
```python
# Python: pitch_set_sanitize() called manually
def pitch_set_sanitize(pcs):
    return tuple(sorted(set(pc % 12 for pc in pcs)))
```

**Rust**: Invariants enforced at construction
```rust
// Rust: PcSet::new() always normalizes
impl PcSet {
    pub fn new(pcs: Vec<Pc>) -> Self {
        // Deduplicates, sorts, zeros - cannot create invalid PcSet
    }
}
```

### 3. Object Identity vs Value Semantics

**Python**: Identity-based (memoization)
```python
# Python: Same arguments return same object
note1 = Note("C")
note2 = Note("C")
assert note1 is note2  # Same object in memory
```

**Rust**: Value-based (Copy/Clone)
```rust
// Rust: Enum variants are values
let note1 = Note::C;
let note2 = Note::C;
assert_eq!(note1, note2);  // Equal values
// Both are Copy types - no heap allocation
```

### 4. Lifetime Management

**Python**: Fretboard references are implicit
```python
# Python: FrettedPitch holds reference but not enforced
class FrettedPitch(Pitch, metaclass=Memoized):
    def __init__(self, pitch, string, fret, fretboard):
        self.fretboard = fretboard  # Could become invalid
```

**Rust**: Fretboard references are explicit
```rust
// Rust: Lifetime parameter prevents dangling references
pub struct SoundedNote<'a> {
    pub string: u8,
    pub fret: u8,
    pub pitch: Pitch,
    pub fretboard: &'a Fretboard,  // Compiler enforces validity
}
```

## Feature Comparison Matrix

| Feature | Python | Rust | Notes |
|---------|--------|------|-------|
| Pitch class arithmetic | `pc % 12` | `Pc::transpose()` | Rust prevents non-mod-12 values |
| Enharmonic detection | `Note.equiv_class` | `Note::is_enharmonic()` | Rust uses trait-based comparison |
| Pitch construction | `Pitch(note, octave)` | `Pitch::new()` | Rust validates MIDI bounds |
| Set normalization | Manual `sanitize()` | Auto in `PcSet::new()` | Rust enforces invariants |
| Mode generation | `modes_from_pitch_set()` | `PcSet::modes()` | Similar algorithms |
| Spelling rules | `SPELL_RULE_DICT` | `SpellingRule` structs | Both context-aware |
| Chord naming | `name()` function | `ChordName` struct | Rust more structured |
| Fretboard shapes | `GtrShape.find_voicing_shapes()` | `FretboardShape` search | Rust adds validation |
| Lilypond output | Jinja2 templates | Tera templates | Feature-gated in Rust |
| VexTab output | Not supported | `ToVextabString` trait | **Rust-only feature** |
| Rhythm handling | `Rhythm` class | `Duration`, `Tuplet` | Rust more complete |
| Error types | Mixed exceptions | 41-variant enum | Rust comprehensive |

## Performance Characteristics

### Python

- **GIL limitation**: Single-threaded effective execution
- **Memoization overhead**: Global dictionary lookups
- **Dynamic dispatch**: Runtime method resolution
- **Memory**: Objects live until program exit (memoization leak)

### Rust

- **No GIL**: Full parallelism available
- **Zero-cost abstractions**: Enums compile to integers
- **Static dispatch**: Monomorphization
- **Memory**: Deterministic cleanup via RAII
- **WASM compilation**: Browser deployment possible

## Integration Patterns

### Python Usage
```python
from pitch_set_lib.pitch_content import Note, Chord, Voicing

chord = Chord("CMaj7")
voicing = Voicing.from_chord(chord, [1, 5, 3, 7])
print(voicing.ly)  # Lilypond output
```

### Rust Usage
```rust
use music::{Note, PcSet, Voicing, pcs, pitch};
use music::notation::lilypond::ToLilypondString;

let chord = pcs!(0, 4, 7, 11);  // CMaj7 pitch classes
let spelled = chord.try_spell(&Note::C)?;
let voicing = Voicing::new(vec![
    pitch!(c, 3), pitch!(g, 3), pitch!(e, 4), pitch!(b, 4)
]);
println!("{}", voicing.to_lilypond_string());
```

## Recommended Reading Order

1. Start with [Type System](./02-type-system.md) to understand the primitive differences
2. Review [Algorithms](./03-algorithms.md) for the core musical computations
3. See [Chord Naming & Bugs](./05-chord-naming-bugs.md) for known Python issues
4. Use [Migration Guide](./06-migration-guide.md) for porting code
