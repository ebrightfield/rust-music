# Native Rust Output (the `music` crate)

When the user wants a transcription as Rust code rather than (or in addition to) a `.ly` file, target this repo's native types. The canonical worked example is `music/examples/generate_lilypond.rs`.

## What "native Rust output" means here

A Rust program that:

1. Builds notes/chords/rhythms with the prelude macros and `RhythmicNotatedEvent` / `Tuplet` constructors.
2. Wraps them into `LilypondStaff` → `LilypondScore` → `LilypondBuilder`.
3. (Optionally) compiles to PDF/PNG via `LilypondCmdBuilder`.

The output is `.rs` source, not `.ly` source. The Rust path is best when the user wants the score embedded in code (tests, examples, runtime generation). For static notation, `.ly` is simpler.

## Imports

```rust
use std::path::PathBuf;
use music::{pitch, voicing, Pitch, Note, Voicing};
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::notation::rhythm::{RhythmicNotatedEvent, Tuplet};
use music::notation::lilypond::command::LilypondCmdBuilder;
use music::notation::lilypond::document::LilypondBuilder;
use music::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
use music::notation::lilypond::document::staff::LilypondStaff;
```

## Pitches

The `pitch!` macro takes a Dutch note name (matches LilyPond) and an octave number where `pitch!(c, 4)` is middle C.

| LilyPond | Rust macro |
|----------|------------|
| `c'` | `pitch!(c, 4)` |
| `cis'` | `pitch!(cis, 4)` |
| `bes` | `pitch!(bes, 3)` |
| `f,,` | `pitch!(f, 1)` |
| `gisis''` | `pitch!(gisis, 5)` |

Accidentals follow LilyPond Dutch convention: `is` = sharp, `es` = flat, `isis` = double sharp, `eses` = double flat. Note exceptions: `es` (not `ees`) for E-flat is also accepted in LilyPond; in the macro use whichever spelling the `Note` enum exposes (`Ees`, `As`, `Bes` are the canonical names — confirm against `music/src/note/note.rs` if unsure).

## Chords / Voicings

A `Voicing` is a stack of pitches sounded together. The `voicing!` macro takes pitches and orders them.

```rust
// LilyPond:  <c e g>
voicing![pitch!(c, 4), pitch!(e, 4), pitch!(g, 4)]
```

## Durations

`Duration` has associated constants for plain values. For dotted notes, set `dots` on the struct.

| LilyPond | Rust |
|----------|------|
| `c4` | `Duration::QTR` |
| `c2` | `Duration::HALF` |
| `c1` | `Duration::WHOLE` |
| `c8` | `Duration::EIGHTH` |
| `c16` | `Duration::SIXTEENTH` |
| `c4.` | `Duration { kind: DurationKind::Quarter, dots: 1, .. }` (or use the constructor — check `duration.rs` for the canonical builder) |

Always confirm the exact constructor signature by reading `music/src/notation/rhythm/duration.rs` — the layout may have changed since this reference was written.

## Building events

A `RhythmicNotatedEvent` pairs musical content (pitch, voicing, or rest) with a duration.

```rust
RhythmicNotatedEvent::pitch(pitch!(c, 4), Duration::QTR)
RhythmicNotatedEvent::voicing(
    voicing![pitch!(c, 4), pitch!(e, 4), pitch!(g, 4)],
    Duration::HALF,
)
RhythmicNotatedEvent::rest(Duration::QTR)  // confirm exact API in rhythm/mod.rs
```

## Tuplets

```rust
// LilyPond:  \tuplet 3/2 { c8 d e }
let triplet: RhythmicNotatedEvent = Tuplet::new(
    vec![
        RhythmicNotatedEvent::pitch(pitch!(c, 4), Duration::EIGHTH),
        RhythmicNotatedEvent::pitch(pitch!(d, 4), Duration::EIGHTH),
        RhythmicNotatedEvent::pitch(pitch!(e, 4), Duration::EIGHTH),
    ],
    3, 2, DurationKind::Eighth,
).into();
```

The arguments are `(events, actual, normal, base_duration)` — 3 in the time of 2 eighth notes.

## Assembling a score

Voices are `Vec<...>` of LilyPond-wrapped events. Multiple voices on a staff share a `LilypondStaff` via `.add_voice(...)`.

```rust
// Convert plain events into the LilyPond-typed events the staff expects:
let voice_1: Vec<_> = events.into_iter().map(|e| e.into()).collect();
let voice_2: Vec<_> = other_events.into_iter().map(|e| e.into()).collect();

let staff = LilypondStaff::new()
    .add_voice(voice_1)
    .add_voice(voice_2);

let score = LilypondScore::new()
    .staff_group(LilypondStaffGroup::new(vec![staff]));

let doc = LilypondBuilder::new()
    .path(Some(PathBuf::from("target/transcription.ly")))
    .score(Some(score));

LilypondCmdBuilder::new()
    .builder(doc)
    .output(Some(PathBuf::from("target/")))
    .build_and_compile()
    .unwrap();
```

The `build_and_compile()` call shells out to the `lilypond` binary, so it requires `lilypond` to be installed and the `lilypond` cargo feature enabled (`cargo run --features lilypond --example ...`).

## Worked example

LilyPond source:

```lilypond
\version "2.24.4"
\score {
  {
    \clef treble
    \key c \major
    \time 4/4
    <c' e' bes'>8 a'8 g'4
    \tuplet 3/2 { c''8 dis''8 e''8 }
    g''4
  }
}
```

Equivalent Rust (mirrors `music/examples/generate_lilypond.rs`):

```rust
let musical_events = vec![
    RhythmicNotatedEvent::voicing(
        voicing![pitch!(c, 4), pitch!(e, 4), pitch!(bes, 4)],
        Duration::EIGHTH,
    ),
    RhythmicNotatedEvent::pitch(pitch!(a, 4), Duration::EIGHTH),
    RhythmicNotatedEvent::pitch(pitch!(g, 4), Duration::QTR),
    Tuplet::new(
        vec![
            RhythmicNotatedEvent::pitch(pitch!(c, 5), Duration::EIGHTH),
            RhythmicNotatedEvent::pitch(pitch!(dis, 5), Duration::EIGHTH),
            RhythmicNotatedEvent::pitch(pitch!(e, 5), Duration::EIGHTH),
        ],
        3, 2, DurationKind::Eighth,
    ).into(),
    RhythmicNotatedEvent::pitch(pitch!(g, 5), Duration::QTR),
];
```

## What this path does not cover (well)

The Rust types are deliberately scoped. For these, prefer raw `.ly` output:

- Articulations (`-.`, `->`, etc.) and dynamics (`\p`, `\f`, hairpins)
- Lyrics and `\lyricmode`
- `\repeat volta`, `\alternative`, voltas
- Markup, text expressions, ornaments
- Chord-mode (`\chordmode`) and `ChordNames` contexts
- `\partial` / pickup measures
- 8va / ottava brackets

If the score uses these, either:

1. Generate `.ly` directly and skip the Rust path, or
2. Generate the note content in Rust and **inject the missing markup via `staff_elements`** if such hooks exist (check `music/src/notation/lilypond/staff_elements.rs`), or
3. Emit the Rust score and tell the user that the marked-up output is not feasible through the native types — recommend `.ly` for that fidelity.

## Verifying the Rust output

```bash
# Compile-check
cargo build -p music --features lilypond

# Run (writes .ly + invokes lilypond if installed)
cargo run --features lilypond --example <your-example>

# Then visually compare target/<output>.png with the source image.
```

If you placed the example under `music/examples/`, `cargo run --example <name>` picks it up automatically.
