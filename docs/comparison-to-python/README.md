# Comparison: Rust `music` vs Python `pitch_set_lib`

This directory documents the comparison between the Rust `music` crate and the Python `pitch_set_lib` library, analyzing how the Rust implementation serves as a full replacement and improvement.

## Document Index

### Core Comparisons
1. **[Overview](./01-overview.md)** - High-level architecture comparison and design philosophy
2. **[Type System](./02-type-system.md)** - Primitive types, collections, and type safety analysis
3. **[Algorithms](./03-algorithms.md)** - Core algorithm comparison (spelling, naming, search)
4. **[Fretboard](./04-fretboard.md)** - Fretted instrument modeling comparison
5. **[Chord Naming & Bugs](./05-chord-naming-bugs.md)** - Naming heuristics and Python bug analysis
6. **[Migration Guide](./06-migration-guide.md)** - Code examples for porting from Python to Rust

### Detailed Analysis
7. **[Complete Bug Registry](./07-complete-bug-registry.md)** - Exhaustive list of all Python bugs with fixes
8. **[Rhythm and Meter](./08-rhythm-and-meter.md)** - Duration, meter, beat grid, and rhythmic scoring
9. **[Implementation Priorities](./09-implementation-priorities.md)** - Ordered list of remaining work
10. **[Melodic Patterns](./10-melodic-patterns.md)** - Melodic sequence generation and pattern systems

## Summary

| Aspect | Python `pitch_set_lib` | Rust `music` |
|--------|------------------------|--------------|
| **Lines of Code** | ~4,800 | ~6,500+ |
| **Type Safety** | Runtime checks, memoization | Compile-time guarantees |
| **Error Handling** | Exceptions, silent failures | `Result<T, E>` types |
| **Concurrency** | GIL-limited | Thread-safe by default |
| **Memory Model** | Memoization singletons | Ownership + lifetimes |
| **Notation Output** | Lilypond only | Lilypond + VexTab |
| **Test Coverage** | Partial, many print-only | Needs expansion |
| **Duration Resolution** | 8th notes | 128th notes |
| **Known Bugs** | 11 documented | Most fixed by design |
| **Melodic Sequencer** | Full (~270 lines) | Not implemented |
| **Fretboard Shapes** | Basic | Advanced (scale shapes) |

## Key Improvements in Rust

1. **Type-safe pitch representation** - Enums prevent invalid states
2. **Explicit error handling** - No silent failures or undefined behavior
3. **Fixed Python bugs** - Chord naming, voiceleading, and spelling issues addressed
4. **Performance** - No GIL, zero-cost abstractions, potential WASM compilation
5. **VexTab support** - Enables browser-based notation rendering
6. **Lifetime safety** - Fretboard references cannot dangle
7. **Finer rhythmic resolution** - 128th notes vs Python's 8th note limit
8. **Structured meter handling** - Type-safe time signatures and beat grids

## Python Bugs Fixed in Rust

| Bug | Python Issue | Rust Solution |
|-----|--------------|---------------|
| Enharmonic voiceleading | `.index()` crash | Pitch class matching |
| Equality comparison | Wrong attribute | Correct `PartialEq` impl |
| High-fret shapes missing | Filter excludes >=12 | No arbitrary filter |
| `only_one()` with PC 0 | Falsy value bug | Option-based return |
| Syntax error `[8.9]` | Float instead of list | Enum matching |
| Debug print statements | In production code | Use `log` crate |

## Quick Reference

### Python → Rust Type Mapping

| Python | Rust |
|--------|------|
| `Note("C#")` | `Note::Cis` |
| `Pitch("C4")` | `Pitch::new(Note::C, 4)?` |
| `pitch_set_sanitize([0,4,7])` | `PcSet::new(vec![...])` |
| `Chord("CMaj7")` | `pcs!(0, 4, 7, 11).try_spell(&Note::C)?` |
| `Fretboard()` | `&*STD_6STR_GTR` |
| `Rhythm(Fraction(1,4))` | `Duration::new(DurationKind::Qtr, 0)` |
| `Meter("4/4")` | `Meter::new(4, MeterDenominator::Four)` |

### Import Patterns

```rust
// Core types
use music::{Note, Pitch, Pc, PcSet, Voicing};
use music::{pc, pcs, pitch, voicing, note};  // Macros

// Collections and analysis
use music::geometry::symmetry::transpositional::{Transpose, Modes};
use music::geometry::symmetry::voiceleading::Voiceleading;

// Fretboard
use music::fretboard::{Fretboard, SoundedNote, FretboardShape, STD_6STR_GTR};

// Notation
use music::notation::rhythm::{Duration, DurationKind, Meter};
use music::notation::lilypond::ToLilypondString;
use music::notation::vextab::ToVextabString;
```
