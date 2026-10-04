# music-engraver

Native Rust SVG music engraver that consumes types from the [`music`](../music/) crate.

Renders standard notation (notes, rests, accidentals, stems, flags, beams, ties,
barlines, clefs, key signatures, time signatures) to SVG using glyph outlines
extracted from the bundled [Bravura](https://www.smufl.org/fonts/) font.

## Quick start

```rust
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::pitch::Pitch;
use music::note::note::Note;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

let svg = ScoreBuilder::new()
    .clef(Clef::Treble)
    .key_signature(KeySignature::Sharps(2))  // D major
    .time_signature(4, 4)
    .note(Pitch::new(Note::D, 4), Duration::QTR)
    .note(Pitch::new(Note::E, 4), Duration::QTR)
    .note(Pitch::new(Note::Fis, 4), Duration::QTR)
    .note(Pitch::new(Note::G, 4), Duration::QTR)
    .barline()
    .rest(Duration::WHOLE)
    .end_barline()
    .render_svg();

std::fs::write("score.svg", &svg).unwrap();
```

## Features

- **SVG output** from glyph outlines (no text rendering required)
- **SMuFL-compliant** layout using Bravura metadata for all engraving constants
- **Font-agnostic architecture** -- glyph lookup by SMuFL canonical name, not
  hard-coded to Bravura. Adding Petaluma or Leland requires no layout/render changes.
- **Proportional spacing** using a Gourlay-style power-of-ratio model
- **Accidentals as measure state**: each pitch is compared with the alteration in force
  for its letter and octave (the in-measure override, else the key signature), so
  cancelling naturals, reinstated key-signature accidentals, and suppressed repeats all
  follow one rule; voices on a staff are resolved in onset order. `AccidentalDisplay::Force`
  and `AccidentalDisplay::Cautionary` (parenthesized) override display per pitch
  (`note_with_accidental`, `chord_with_accidentals`, `beam_group_with_accidentals`,
  `tuplet_ratio_with_accidentals`). Chord accidentals stack into non-colliding columns, and
  every accidental's width is reserved to the left of its note.
- **Multi-system page layout** with configurable measures-per-system and justification

## Publication boundary

`music-engraver` produces notation SVG (and optional PNG); it does not own
titles, source attributions, prose, paper size, pagination, or PDF output.
Assemble its SVG scores into fixed-page documents with Typst. A native PDF
renderer and a `DocumentBuilder` are not required for Modus Novus fidelity.
The current Modus Novus harvest remains published through LilyPond until
source-derived notation, alignment, and visual checks pass for this backend.

## Running examples

```bash
# Single measure with clef, key sig, time sig, notes, and barline
cargo run --example measure -p music-engraver

# Multi-system score via the high-level ScoreBuilder API
cargo run --example score_builder -p music-engraver

# Beamed note groups
cargo run --example beamed_notes -p music-engraver

# All examples write SVG to music-engraver/examples/output/
```

## Architecture

| Layer | Modules | Purpose |
|-------|---------|---------|
| **Font** | `font::music_font`, `font::glyph_outline`, `font::engraving_config` | OTF parsing, glyph outline extraction, engraving defaults |
| **Layout** | `layout::staff`, `layout::note_placement`, `layout::stem`, `layout::beam`, `layout::measure`, `layout::system`, `layout::page` | Geometry computation in font design units |
| **Render** | `render::svg_writer`, `render::staff_renderer`, `render::note_renderer`, `render::beam_renderer`, `render::measure_renderer`, `render::system_renderer`, `render::page_renderer` | SVG generation from layout structs |
| **Score** | `score::ScoreBuilder` | High-level fluent API bridging `music` types to the layout/render pipeline |

## License

Dual-licensed under MIT or Apache-2.0 at your option.

The bundled Bravura font is licensed under the SIL Open Font License 1.1
(see `fonts/OFL.txt`).
