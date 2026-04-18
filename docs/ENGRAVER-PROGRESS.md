# Music Engraver Progress Log

## 2026-04-18 — Phase 0, crate scaffold
- Did: Created `music-engraver/` as workspace member. Set up `Cargo.toml` with dependencies (music, ttf-parser, smufl 0.2, serde, serde_json, thiserror; optional png feature with resvg/tiny-skia/fontdb). Created `src/lib.rs` with `font` module. Bundled Bravura.otf (v1.380, 508KB), OFL.txt license, and bravura_metadata.json under `fonts/`. Font module exposes `BRAVURA_OTF` and `BRAVURA_METADATA` via `include_bytes!`.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo test -p music-engraver` passes (3 tests: OTF magic bytes, ttf-parser parse, metadata JSON validity).
- Next: Phase 1 font pipeline — parse Bravura with ttf-parser, extract a single glyph (`noteheadBlack`) outline as SVG path data, expose via font API.
- Open issues: None.

## 2026-04-18 — Phase 1, font pipeline core (glyph outline extraction)
- Did: Created `font/glyph_outline.rs` (SvgPathBuilder implementing ttf-parser OutlineBuilder, y-flip for SVG, GlyphOutline struct) and `font/music_font.rs` (MusicFont struct wrapping ttf-parser Face + smufl Metadata, glyph lookup by SMuFL codepoint, outline extraction, advance width). Added `bravura_font()` convenience constructor. All glyph lookup is by `smufl::Glyph` enum — layout/render code never touches codepoints or glyph IDs directly.
- Verified: `cargo test -p music-engraver` — 13 tests pass (format_coord, svg_path_builder, notehead_black outline, treble_clef outline, multiple glyphs differ, metadata font name, units_per_em, etc). `cargo check --workspace` passes.
- Next: Phase 1 continuation — write a minimal test SVG that renders the noteheadBlack glyph path to `examples/output/`, proving the end-to-end pipeline from font bytes to SVG output.
- Open issues: None.

## 2026-04-18 — Phase 1, end-to-end SVG render + render module
- Did: Created `render/svg_writer.rs` (SvgWriter struct: viewBox, add_path, add_line, add_raw, to_svg). Created `examples/notehead_svg.rs` example that renders noteheadBlack on 5 staff lines to `examples/output/notehead_black.svg`. Added integration tests in `tests/svg_glyph_render.rs` (SVG structure validation, multi-glyph differentiation, advance width sanity checks).
- Verified: `cargo test -p music-engraver` — 19 tests pass (16 unit + 3 integration). `cargo run --example notehead_svg` produces valid SVG. `cargo check --workspace` passes.
- Next: Phase 1 completion — extract engraving defaults from SMuFL metadata (staff line thickness, stem width, beam thickness etc.) and expose via `MusicFont` API. Then Phase 1 exit criteria check.
- Open issues: None.

## 2026-04-18 — Phase 1, engraving defaults from metadata
- Did: Created `font/engraving_config.rs` with `EngravingConfig` struct — resolves all `Option<StaffSpaces>` values from `smufl::EngravingDefaults` into guaranteed `f64`s with SMuFL-recommended fallbacks. Includes 22 engraving constants (staff line thickness, stem, beam, barlines, leger lines, slurs, ties, etc.), `staff_space` in font units, `to_font_units()` converter, and `_fu()` convenience accessors for the most-used values. Added `MusicFont::engraving_config()` method. Wired into `font/mod.rs` as public export.
- Verified: `cargo test -p music-engraver` — 28 tests pass (25 unit + 3 integration). All 9 new tests verify Bravura-specific values (0.13 staff spaces → 32.5 font units for staff lines, etc.), fallback behavior with empty metadata, and font-unit conversions. `cargo check --workspace` passes.
- Next: Phase 1 exit criteria check — verify we have: font parsing, glyph outline extraction, engraving defaults, SVG render pipeline. If complete, begin Phase 2 (staff lines, clef placement, basic layout types).
- Open issues: None.

## 2026-04-18 — Phase 2, staff layout geometry
- Did: Created `layout/` module with `layout/staff.rs`. `StaffLayout` struct models 5-line staff geometry: maps staff positions (i8, bottom line=0, top line=8) to y-coordinates in font design units. Includes `line_ys()`, `y_of()`, `ledger_line_count()`, `ledger_line_ys()`, `is_on_line()`, `is_in_space()`, `needs_ledger_lines()`. Also `from_config()` constructor using `EngravingConfig`. 18 unit tests covering positions, ledger lines, edge cases, non-zero origins.
- Verified: `cargo test -p music-engraver` — 46 tests pass (43 unit + 3 integration). `cargo check --workspace` passes.
- Next: Phase 2 continuation — add clef placement to layout (map clef type to SMuFL glyph + staff position), then staff rendering function that draws staff lines + clef via SvgWriter.
- Open issues: None. Note: `CARGO_HOME` must be set to `$HOME/.cargo` due to read-only `/opt/rust/cargo/` in this environment.

## 2026-04-18 — Phase 2, clef placement + staff rendering
- Did: Created `layout/clef.rs` with `ClefLayout` (maps `music::Clef` → SMuFL `Glyph` + staff position). Created `render/staff_renderer.rs` with `draw_staff_lines()` and `draw_clef()` functions. Added `examples/staff_with_clef.rs` that renders treble and bass clef staves to `examples/output/`. 6 clef layout tests, 7 staff renderer tests.
- Verified: `cargo test -p music-engraver` — 59 tests pass (56 unit + 3 integration). `cargo run --example staff_with_clef` produces valid SVGs with 5 lines + clef path. `cargo check --workspace` passes.
- Next: Phase 2 continuation — add `add_text()` to SvgWriter (needed for time signatures and other text), or begin note placement layout (mapping pitch to staff position for a given clef).
- Open issues: None.

## 2026-04-18 — Phase 2, note placement (pitch → staff position)
- Did: Created `layout/note_placement.rs` with `pitch_to_staff_position(pitch, clef) -> StaffPosition`. Maps any `Pitch` + `Clef` to vertical staff position using diatonic distance from clef reference pitch (treble: G4 at pos 2, bass: F3 at pos 6, octave-transposing clefs shift reference octave). Accidentals don't affect position; enharmonic spellings (B#3 vs C4) produce distinct positions as expected.
- Verified: `cargo test -p music-engraver` — 84 tests pass (81 unit + 3 integration). 22 new note_placement tests cover all 4 clef types, accidentals (sharp, flat, double), enharmonic distinctions, ledger-line territory above/below, extreme registers. `cargo check --workspace` passes.
- Next: Phase 2 continuation — notehead rendering (draw a notehead glyph at the correct staff position given a pitch), then an example that renders a single note on a staff with clef and ledger lines.
- Open issues: None.

## 2026-04-18 — Phase 2, notehead rendering + ledger lines
- Did: Created `render/note_renderer.rs` with `NoteheadKind` enum (Whole/Half/Filled → SMuFL glyph), `draw_notehead()`, `draw_ledger_lines()`, and `draw_note()` (composite). Ledger lines extend symmetrically past notehead by `leger_line_extension`. Created `examples/single_note.rs` rendering 5 notes on a treble staff (middle C with ledger line below, E4 on bottom line, B4 half note, F5 on top line, A5 whole note with ledger line above).
- Verified: `cargo test -p music-engraver` — 102 tests pass (99 unit + 3 integration). 18 new note_renderer tests cover notehead placement, ledger line counts, extension geometry, thickness from config, composite draw, advance width comparison. `cargo run --example single_note` produces valid SVG (6 paths, 7 lines). `cargo check --workspace` passes.
- Next: Phase 2 completion — add `add_text()` to SvgWriter for time signatures, or begin stem rendering (stem direction rules, draw stem line from notehead). Then Phase 2 exit criteria check.
- Open issues: None.

## 2026-04-18 — Phase 2, stem layout + rendering
- Did: Created `layout/stem.rs` with `StemDirection` enum, `auto_stem_direction()` (single note — on/above middle line → down, below → up), `auto_stem_direction_chord()` (farthest-from-middle-line rule, ties → down), `stem_length_staff_spaces()` (default 3.5ss, extends so tip reaches middle line for distant notes, min 2.5ss). Created `render/stem_renderer.rs` with `stem_x()` (right side for up, left for down), `stem_endpoints()` (y_top, y_bottom), `draw_stem()`. 15 layout tests + 12 renderer tests.
- Verified: `cargo test -p music-engraver` — 129 unit + 3 integration = 132 tests, all pass. No warnings. `cargo check --workspace` passes.
- Next: Phase 2 continuation — create an example that renders stemmed notes on a staff (combining draw_note + draw_stem with auto stem direction). Then consider `add_text()` for time signatures, or Phase 2 exit criteria check.
- Open issues: None.

## 2026-04-18 — Phase 2, stemmed notes example + draw_stemmed_note
- Did: Created `examples/stemmed_notes.rs` rendering 7 notes (C4–A5) on treble staff with auto stem direction. Added `draw_stemmed_note()` convenience function to `render/note_renderer.rs` combining notehead + ledger lines + stem in one call (accepts `Option<StemDirection>`, None skips stem). 5 new tests covering stem+ledger combos, None direction, advance width identity, y-coordinate correctness.
- Verified: `cargo test -p music-engraver` — 134 unit + 3 integration = 137 tests, all pass. `cargo run --example stemmed_notes` produces valid SVG (8 paths, 14 lines — exact match). `cargo check --workspace` passes.
- Next: Phase 2 completion check — add `add_text()` to SvgWriter for time signatures (needed for key/time sig rendering), or assess Phase 2 exit criteria and begin Phase 3 (accidentals, dots, flags).
- Open issues: None.

## 2026-04-18 — Phase 3, accidental layout + rendering
- Did: Created `layout/accidental.rs` with `accidental_glyph()` (maps `music::Accidental` → `smufl::Glyph`, with `show_natural` toggle), `accidental_x()` (computes x-position left of notehead with standard padding of 0.12 staff spaces), and `ACCIDENTAL_NOTEHEAD_PADDING_SS` constant. Created `render/accidental_renderer.rs` with `draw_accidental()` (renders accidental glyph at correct position, returns drawn x-pos or None). 10 layout tests + 9 renderer tests covering all 5 accidental types, natural visibility, padding math, position differentiation.
- Verified: `cargo test -p music-engraver` — 153 unit + 3 integration = 156 tests, all pass. `cargo check --workspace` passes. No warnings.
- Next: Phase 3 continuation — augmentation dots (layout + render), then flags for eighth/sixteenth notes.
- Open issues: None.

## 2026-04-18 — Phase 3, augmentation dot layout + rendering
- Did: Created `layout/dot.rs` with `dot_staff_position()` (shifts dots on lines up to the space above), `first_dot_x()`, `dot_xs()` (computes x-positions for 1–N dots), and constants for padding/spacing. Created `render/dot_renderer.rs` with `draw_dots()` rendering augmentation dot glyphs (`Glyph::AugmentationDot`) at correct positions, returning x past last dot. 17 layout tests (position shifting for all line/space/ledger cases, x-position math, multi-dot spacing), 9 renderer tests (zero/single/double/triple dots, line-shift in SVG, x-position verification, return value).
- Verified: `cargo test -p music-engraver` — 182 unit + 3 integration = 185 tests, all pass. `cargo check --workspace` passes. No warnings.
- Next: Phase 3 continuation — flags for eighth/sixteenth notes (layout + render).
- Open issues: None.

## 2026-04-18 — Phase 3, flag layout + rendering
- Did: Created `layout/flag.rs` with `flag_glyph()` (maps flag count 1–5 + `StemDirection` → SMuFL `Glyph` for 8th through 128th flags), `flag_position()` (documents convention: flag placed at stem tip). Created `render/flag_renderer.rs` with `draw_flag()` that renders flag glyph at stem tip position, returning `Ok(true)` if drawn, `Ok(false)` for 0 flags. 13 layout tests (all durations × both directions, zero/out-of-range, direction symmetry), 10 renderer tests (eighth/sixteenth/32nd/64th/128th glyphs, up vs down differentiation, zero count, realistic stem-tip coordinates).
- Verified: `cargo test -p music-engraver` — 206 unit + 3 integration = 209 tests, all pass. `cargo check --workspace` passes. No warnings.
- Next: Phase 3 continuation — create an example rendering flagged notes (8th, 16th, 32nd) on a staff combining stem + flag + notehead. Then consider rest glyphs or begin Phase 3 exit criteria check.
- Open issues: None.

## 2026-04-18 — Phase 3, flagged notes example + rest glyphs
- Did: Created `examples/flagged_notes.rs` rendering 7 notes (C4–A5) with flags (8th, 16th, 32nd) on a treble staff — combines noteheads, stems, flags, and ledger lines in one SVG. Created `layout/rest.rs` with `rest_glyph()` (maps log2 duration 0–7 to SMuFL rest glyphs), `rest_staff_position()` (whole at pos 6, half at pos 4, others at pos 4), `rest_y()`. Created `render/rest_renderer.rs` with `draw_rest()`. 16 layout tests + 8 renderer tests covering all 8 rest durations, distinct paths, advance widths, invalid input.
- Verified: `cargo test -p music-engraver` — 229 unit + 3 integration = 232 tests, all pass. `cargo run --example flagged_notes` produces valid SVG (15 paths, 14 lines — exact match). `cargo check --workspace` passes.
- Next: Phase 3 completion — create a rest example SVG, then Phase 3 exit criteria check. Phase 3 elements complete: accidentals, dots, flags, rests. Next after that: Phase 4 (measure layout, barlines, time/key signatures).
- Open issues: None.

## 2026-04-18 — Phase 3 complete + Phase 4, barline layout + rendering
- Did: Created `examples/rests.rs` rendering all 8 rest durations (whole through 128th) on treble staff — completes Phase 3 examples. **Phase 3 exit criteria met**: accidentals (5 types), augmentation dots (up to triple), flags (8th–128th, up/down), rests (whole–128th), all with layout + renderer + tests + examples. Then started Phase 4: created `layout/barline.rs` with `BarlineStyle` enum (Single/Double/Final/StartRepeat/EndRepeat), `BarlineLayout` struct (strokes + optional repeat dots + width), `barline_layout()` function. Created `render/barline_renderer.rs` with `draw_barline()`. 13 layout tests + 9 renderer tests covering all 5 styles, dimensions, stroke ordering, repeat dots.
- Verified: `cargo test -p music-engraver` — 251 unit + 3 integration = 254 tests, all pass. `cargo run --example rests` produces valid SVG (9 paths, 5 lines). `cargo check --workspace` passes.
- Next: Phase 4 continuation — create barlines example SVG showing all 5 styles, then begin time signature rendering (text-based numerals or SMuFL time sig glyphs).
- Open issues: None.
