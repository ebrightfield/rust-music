# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build the workspace
cargo build

# Build with lilypond feature (required for lilypond notation output)
cargo build --features lilypond

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run examples
cargo run --example primitives
cargo run --example collections
cargo run --example generate_lilypond --features lilypond
```

**Note:** This crate uses the nightly feature `concat_idents`. Build with `rustup run nightly cargo build` if needed.

## Architecture

### Workspace Structure
- **`music/`** - Core music theory library
- **`musical-combinatorics/`** - Enumerated chord/scale quality types (depends on `music`)

### Core Type Hierarchy (music crate)

**Single-Pitch Primitives** (`note/`):
- `Pc` - Pitch class (mod-12 integer, spelling/octave agnostic)
- `Note` - Spelled note without octave (e.g., C, C#, Db)
- `Pitch` - Note with octave (maps to MIDI)
- `Spelling` - Letter + Accidental decomposition

**Collections** (`note_collections/`):
- `PcSet` - Set of pitch classes (basis for chord/scale analysis)
- `NoteSet` - Spelled notes (a `PcSet` with defined spellings)
- `Voicing` / `StackedIntervals` - Concrete pitch arrangements
- `OctavePartition` - Intervals that sum to 12
- `IntervalClass` - Mod-6 interval classification

**Chord Naming** (`note_collections/chord_name/`):
- `ChordName` - Root + quality + underlying PcSet
- `ChordQuality` - Combination of tonal flavors
- `naming_heuristics/` - Algorithms to infer chord names from PcSets

**Geometry** (`note_collections/geometry/`):
- `symmetry/` - Transpositional, intervallic, and voiceleading symmetries
- `sets/` - Set operations on pitch classes
- `contour/` - Melodic contour analysis

### Fretboard Module (`fretboard/`)
- `Fretboard` - Arbitrary tuning/string count instrument model
- `FrettedNote` / `SoundedNote` - Notes on specific string/fret positions
- `FretboardShape` - Chord/melodic shape search on fretboards
- `STD_6STR_GTR` - Predefined standard guitar tuning

### Notation Output (`notation/`)
- `lilypond/` - Lilypond source generation (feature-gated)
- `vextab/` - VexTab source generation for JS frontends
- `rhythm/` - Duration, meter types
- `clef/` - Clef definitions

## Key Design Patterns

- Enharmonic spellings are distinct types (`Note::Cis` vs `Note::Des`)
- `Pc::from(&Note)` maps spellings to pitch classes; `Pc::notes()` returns possible spellings
- Collections normalize via sorting and deduplication on construction
- The `lilypond` feature enables Tera templates for document generation
