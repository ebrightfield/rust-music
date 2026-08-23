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

### Static musl binaries (for containers)

`scripts/build-musl.sh` produces **fully static** `x86_64` binaries that run in any container
image — including `scratch` and `alpine` — with no libc, no dynamic loader, and no shared
libraries. Bravura.otf and its SMuFL metadata are `include_bytes!`-embedded in
`music-engraver`, so no font files are needed at runtime either.

```bash
./scripts/build-musl.sh              # containerized build (podman, falls back to docker)
./scripts/build-musl.sh --host       # host cargo + rustup musl target — faster for iteration
./scripts/build-musl.sh --strip      # also strip symbols (9.3M -> 7.9M)
```

- **Containerized** (default) pins the toolchain in `Containerfile.musl`, so the artifact is
  reproducible and buildable on a machine without the musl target installed. Artifacts land in
  `target/musl-build/`.
- **`--host`** needs `rustup target add x86_64-unknown-linux-musl` and writes to
  `target/x86_64-unknown-linux-musl/release/`.

Both modes end with a verification gate, and `--prove` adds a behavioural one:

```bash
./scripts/build-musl.sh --prove      # also RUNs it in scratch / alpine / debian / busybox
```

**Structural checks** (always run). Two properties, both read straight off the ELF:

- no `INTERP` program header → no dynamic loader is invoked;
- zero `DT_NEEDED` entries → no shared library is required.

A static-PIE *does* still have a dynamic *section* (it self-relocates), so the section's presence
is not a failure — only `DT_NEEDED` is. **Do not use `file` for this**: it printed
"dynamically linked" for binaries `ldd` simultaneously called "statically linked", and printed
plausible output for artifacts that segfaulted outside the builder.

**Behavioural checks** (`--prove`). Runs the artifact in four environments — `scratch` (empty),
`alpine:3.20` (a *different* musl version than the builder), `debian:bookworm-slim` (glibc, a
different libc family), and `busybox` — then renders a PNG inside `scratch` to exercise the
embedded Bravura font and the `resvg` rasterizer.

The Containerfile independently runs the binary in a `FROM scratch` stage; that stage is
depended on by the final stage on purpose, so it cannot be skipped.

Both gates are discriminating, not decorative: swapping a dynamically-linked glibc binary into
the output directory makes the script exit 1 with
`FAIL slonimsky: requests a program interpreter (not static)`.

Currently only `slonimsky` produces a binary; the other six workspace members are libraries.
When a crate gains a binary, add it to the `cp` list in `Containerfile.musl` (stage 4) — and if
a new *workspace member* is added, add its `COPY <member>/Cargo.toml` and `COPY <member>/`
lines too. `scripts/build-musl.sh` fails fast with an explicit message if a member is missing
from that list, because an omission silently breaks the dependency cache layer.

Consuming the artifact — drop it into any image, no base-image requirements:

```dockerfile
FROM scratch
COPY slonimsky /slonimsky
ENTRYPOINT ["/slonimsky"]
```

Verified working in `scratch`: `--version`, `spell Cm7` → `C Eb G Bb`, and both SVG and PNG
output (the embedded Bravura font and the `resvg`/`tiny-skia` rasterizer are compiled in).


## Architecture

### Workspace Structure
- **`music/`** - Core music theory library
- **`music-engraver/`** (in progress) - Native Rust SVG engraver consuming `music` types. Built per `docs/vexflow-port-research/00-port-plan.md`; progress tracked in `docs/ENGRAVER-PROGRESS.md`. Bravura OTF + SMuFL metadata bundled in `music-engraver/fonts/`. Bravura-first but font-agnostic architecture.
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
- `rhythm/` - Duration, meter, beat grid, tuplets
- `clef/` - Clef definitions

### Melody Module (`melody/`)
- `MelodicSequencer` - Pattern-based melody generation with boundary handling
- `IntervalPattern` - Multi-level interval patterns with master steps
- `ChordProgression` - Harmonic context with position tracking
- `Direction`, `TurnaroundMode`, `PitchBounds` - Configuration types

### SVG Generation (`svg/`)
- `PitchCircleBuilder` - Pitch class circle diagrams
- `FretboardBuilder` - Vertical/horizontal fretboard diagrams with barre notation
- `IntervalBuilder` - Interval vector and matrix visualizations
- `SvgTheme` - Theme presets (default, dark, print, colorful)

## Key Design Patterns

- Enharmonic spellings are distinct types (`Note::Cis` vs `Note::Des`)
- `Pc::from(&Note)` maps spellings to pitch classes; `Pc::notes()` returns possible spellings
- Collections normalize via sorting and deduplication on construction
- The `lilypond` feature enables Tera templates for document generation
