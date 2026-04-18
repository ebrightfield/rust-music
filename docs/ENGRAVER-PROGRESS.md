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

## 2026-04-18 — Phase 4, barlines example + time signature layout & rendering
- Did: Created `examples/barlines.rs` rendering all 5 barline styles (single, double, final, start repeat, end repeat) on a treble staff — produces `examples/output/barlines.svg`. Created `layout/time_signature.rs` with `TimeSignatureKind` enum (Numeric/Common/CutCommon), `digit_glyph()`, `time_signature_layout()` (resolves digit glyphs, centres numerator at staff pos 6 and denominator at pos 2, handles multi-digit numbers up to 99). Created `render/time_sig_renderer.rs` with `draw_time_signature()`. 12 layout tests (digit mapping, centering, all kinds, multi-digit), 8 renderer tests (path counts, positions, differentiation).
- Verified: `cargo test -p music-engraver` — 270 unit + 3 integration = 273 tests, all pass. `cargo run --example barlines` produces valid SVG (5 paths, 14 lines). `cargo check --workspace` passes.
- Next: Phase 4 continuation — create time signatures example SVG (showing 4/4, 6/8, 12/8, common, cut-common on a staff), then begin key signature layout.
- Open issues: None.

## 2026-04-18 — Phase 4, time signatures example + key signature layout & rendering
- Did: Created `examples/time_signatures.rs` rendering 5 time signatures (4/4, 6/8, 12/8, common, cut common) on treble staff → `examples/output/time_signatures.svg` (10 paths, 5 lines). Created `layout/key_signature.rs` with `KeySignature` enum (Sharps/Flats/Open), `key_signature_layout()` (maps key to ordered accidental positions per clef — treble, treble 8va/8ba, bass), `sharp_positions()`/`flat_positions()` tables, spacing at 1 staff space. Created `render/key_sig_renderer.rs` with `draw_key_signature()`. Handles Treble/Treble8va/Treble8ba/Bass clefs (no Alto/Tenor since `music::Clef` doesn't have them).
- Verified: `cargo test -p music-engraver` — 294 unit + 3 integration = 297 tests, all pass. `cargo run --example time_signatures` produces valid SVG. `cargo check --workspace` passes. 17 key_sig layout tests + 9 renderer tests.
- Next: Phase 4 continuation — create key signatures example SVG showing sharps and flats on treble and bass staves, then Phase 4 exit criteria check.
- Open issues: None.

## 2026-04-18 — Phase 4 complete + Phase 5, measure layout types
- Did: Created `examples/key_signatures.rs` rendering sharps (1–4) and flats (1–4) on treble staff, sharps (5–7) and flats (5–7) on bass staff → `examples/output/key_signatures.svg` (58 paths, 10 lines). **Phase 4 exit criteria met**: barlines (5 styles), time signatures (numeric/common/cut), key signatures (sharps/flats, treble/bass), all with layout+renderer+tests+examples. Then started Phase 5: created `layout/measure.rs` with `MeasureElement` enum (Clef/KeySig/TimeSig/Note/Rest/Barline), `NoteEvent`/`RestEvent` structs, `MeasureLayoutConfig`, `MeasureLayout`, and `layout_measure()` function implementing proportional duration spacing (Gourlay-style power-of-ratio model). Fixed `Clef` not deriving Clone/Debug by using `ClefLayout` wrapper in enum.
- Verified: `cargo test -p music-engraver` — 310 unit + 3 integration = 313 tests, all pass (16 new measure layout tests). `cargo run --example key_signatures` produces valid SVG. `cargo check --workspace` passes.
- Next: Phase 5 continuation — create a measure renderer that uses `MeasureLayout` to draw a complete measure (composing staff lines, clef, key sig, time sig, notes, barline), then an example rendering a simple measure.
- Open issues: None.

## 2026-04-18 — Phase 5, measure renderer
- Did: Created `render/measure_renderer.rs` with `draw_measure()` that composes all existing renderers (clef, key sig, time sig, notehead, stem, flag, dot, accidental, rest, barline) using a `MeasureLayout`. Includes `draw_note_event()` helper that renders a complete note (accidental + notehead + ledger lines + stem + flag + dots). Also refactored `key_signature_layout()` and `draw_key_signature()` to take `&Clef` instead of `Clef` (since `music::Clef` doesn't impl Copy/Clone), propagated change to examples. 18 new measure renderer tests covering: empty, single note, whole note (no stem), eighth (flag), dotted quarter, accidental, rest, barline, clef+key sig, time sig, x_offset, full measure with all elements, ledger lines, stem direction override.
- Verified: `cargo test -p music-engraver` — 326 unit + 3 integration = 329 tests, all pass. `cargo check --workspace` passes.
- Next: Phase 5 continuation — create `examples/measure.rs` rendering a complete measure (clef + key sig + time sig + notes + barline) to `examples/output/measure.svg`.
- Open issues: None.

## 2026-04-18 — Phase 5, measure example
- Did: Created `examples/measure.rs` rendering a complete measure in D major 3/4 time: treble clef + 2-sharp key sig + 3/4 time sig + dotted quarter F4 + eighth A4 with natural accidental + quarter rest + final barline. All element types exercised in one SVG. Output: 11 paths, 9 lines — exact match with expected counts.
- Verified: `cargo run --example measure` produces valid SVG (8492 bytes). `cargo test -p music-engraver` — 326 unit + 3 integration = 329 tests, all pass. `cargo check --workspace` passes.
- Next: Phase 5 completion check. Phase 5 elements complete: measure layout (proportional spacing), measure renderer (composing all element renderers), measure example. Consider multi-measure rendering or begin Phase 6 (beams, ties, slurs). Next concrete chunk: multi-measure layout or beam grouping.
- Open issues: None.

## 2026-04-18 — Phase 6, beam layout
- Did: Created `layout/beam.rs` with beam grouping and geometry computation. `BeamedNote` struct (x, staff_position, duration_log2). `beam_group_stem_direction()` (farthest-from-middle-line rule). `compute_beam_counts()` (left/right beam connectivity per note — handles primary beams, secondary beams for 16th/32nd, fractional beams). `layout_beam_group()` (computes stem tip y-coordinates via linear interpolation between first/last note tips, slope clamping to ~18°, minimum stem length enforcement, extra stem length for secondary beams). Helper `staff_position_to_y()`.
- Verified: `cargo test -p music-engraver` — 347 unit + 3 integration = 350 tests, all pass. 21 new beam layout tests covering: direction selection (empty, below/above/mixed/equidistant), beam counts (single/pair/four eighths, sixteenths, mixed, 32nds, empty), layout geometry (ascending/descending, flat beam, minimum stem length, slope constraint, interpolation, mixed durations, single note, stems-down). No warnings. `cargo check --workspace` passes.
- Next: Phase 6 continuation — create `render/beam_renderer.rs` with `draw_beam_group()` that renders beam lines (primary + secondary) and stems for a beam group, then an example SVG.
- Open issues: None.

## 2026-04-18 — Phase 6, beam renderer + example
- Did: Created `render/beam_renderer.rs` with `draw_beam_group()` that renders stems (lines) and beam segments (filled polygons) for a `BeamGroupLayout`. Handles primary beams spanning entire group, secondary beams connecting subgroups of shorter notes, and fractional beam stubs for isolated secondary beams. Added `add_rect()` and `add_polygon()` to `SvgWriter`. Created `examples/beamed_notes.rs` rendering 4 beam groups on treble staff: (1) four ascending eighth notes with stems up, (2) two descending sixteenth notes with stems down, (3) mixed eighth + two sixteenths, (4) three 32nd notes descending with stems down. Output: 13 paths, 18 lines, 8 polygons.
- Verified: `cargo test -p music-engraver` — 364 unit + 3 integration = 367 tests, all pass. 11 beam renderer tests (two eighths, two sixteenths, four eighths, stems down, mixed, empty, polygon points, beam thickness, 32nds, single note fractional, stem x direction) + 3 svg_writer tests (rect, polygon, empty polygon). `cargo run --example beamed_notes` produces valid SVG (5399 bytes). `cargo check --workspace` passes.
- Next: Phase 6 continuation — tie/slur rendering (curved paths between notes), or consider Phase 6 exit criteria check. Beam rendering is complete for v1 scope.
- Open issues: None.

## 2026-04-18 — Phase 6, tie layout + rendering
- Did: Created `layout/tie.rs` with `TieDirection` enum (Over/Under), `tie_direction_from_stem()`, and `layout_tie()` — computes cubic Bézier control points for a filled crescent shape representing a tie. Height scales with span (0.15× ratio, clamped 0.4–1.5 staff spaces). Crescent thickness derived from `tie_endpoint_thickness` / `tie_midpoint_thickness` in `EngravingConfig`. Created `render/tie_renderer.rs` with `draw_tie()` rendering two cubic Bézier curves (outer + inner) as a closed filled path. Added `add_filled_path()` to SvgWriter. Wired into `layout/mod.rs` and `render/mod.rs`.
- Verified: `cargo test -p music-engraver` — 385 unit + 3 integration = 388 tests, all pass. 12 tie layout tests (direction mapping, endpoint above/below, apex geometry, control points, height scaling, symmetry, crescent thickness, min height). 9 tie renderer tests (path structure, Bézier count, over/under differentiation, SVG element attributes, short/long ties). `cargo check --workspace` passes.
- Next: Phase 6 continuation — create a ties example SVG showing tied notes on a staff, then consider slur rendering (similar curve but different attachment points) or Phase 6 exit criteria check.
- Open issues: None.

## 2026-04-18 — Phase 6, tied notes example
- Did: Created `examples/tied_notes.rs` rendering 5 tied note pairs on treble staff: (1) E4 quarter–quarter stems up, tie under; (2) B4 half–quarter stems down, tie over; (3) A5 quarter–quarter above staff with ledger lines, tie over; (4) C4 quarter–quarter below staff with ledger lines, short tie under; (5) G4 with explicit Over direction override. Output: 16 paths (10 noteheads + 1 clef + 5 ties), 19 lines (5 staff + 10 stems + 4 ledger).
- Verified: `cargo run --example tied_notes` produces valid SVG (5533 bytes). `cargo test -p music-engraver` — 385 unit + 3 integration = 388 tests, all pass. `cargo check --workspace` passes (via build).
- Next: Phase 6 completion check — beam rendering complete, tie rendering complete. Consider slur rendering (optional for v1?) or Phase 6 exit criteria assessment and move to Phase 7 (multi-measure systems, line breaking).
- Open issues: Slurs are architecturally similar to ties but with different attachment points (near notehead vs at stem tip); could reuse TieLayout with different parameters. Deferring slurs to post-v1 is reasonable since ties cover the basic curved-path rendering proof-of-concept.

## 2026-04-18 — Phase 7, system layout + renderer
- Did: Created `layout/system.rs` with `SystemLayout`, `SystemPrefix`, `SystemMeasure`, `MeasureContent`, `MeasureEvent`, `ClefKind` (local Clone/Copy mirror of `music::Clef` which lacks derives), and `layout_system()` function. First measure gets prefix (clef, key sig, time sig); subsequent measures get only rhythmic content + barlines. Supports `target_width` for horizontal justification (scales all spacing proportionally). Created `render/system_renderer.rs` with `draw_system()` composing staff lines + all measures. Added `ClefLayout::from_clef_ref(&Clef)` for borrow-based construction. Created `examples/system.rs` rendering 3 measures in D major 4/4 (ascending quarter notes, dotted half + rest, eighths + half) with final barline, justified to 10000 font units.
- Verified: `cargo test -p music-engraver` — 406 unit + 3 integration = 409 tests, all pass. 13 system layout tests + 8 system renderer tests. `cargo run --example system` produces valid SVG (21 paths, 19 lines, 11676 bytes). `cargo check --workspace` passes.
- Next: Phase 7 continuation — multi-system layout (multiple lines on a page, e.g. for pieces longer than one line), or begin looking at a higher-level API that accepts `music` crate types directly (Pitch + Duration → system of measures).
- Open issues: None.

## 2026-04-18 — Phase 7, page layout (multi-system)
- Did: Created `layout/page.rs` with `PageLayoutConfig` (system_width, system_spacing, margins, staff_space), `PageLayout`/`PageSystem` structs, `SystemBreaking` enum (Fixed/Manual), `layout_page()` function. Breaks measures into systems per breaking strategy, stacks vertically with configurable spacing, time signature only on first system, all systems justified to system_width. Created `render/page_renderer.rs` with `draw_page()` that auto-computes viewBox/pixel dimensions and renders all systems. Created `examples/multi_system.rs` rendering 8 measures (melody in D major) across 2 systems. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 429 unit + 3 integration = 432 tests, all pass. 17 page layout tests (breaking strategies, margins, system counts, y-offsets, time sig presence, justification) + 6 page renderer tests (empty page, single/two/three systems, dimensions, y-separation). `cargo run --example multi_system` produces valid SVG (30 paths, 37 lines, 18514 bytes). `cargo check --workspace` passes.
- Next: Phase 7 completion check — consider a higher-level API that accepts `music` crate types (Pitch + Duration → page of music), or Phase 8 (public API, documentation, README).
- Open issues: None.

## 2026-04-18 — Phase 8, high-level ScoreBuilder API
- Did: Created `score.rs` module with `ScoreBuilder` — fluent builder API that accepts `music` crate types (`Pitch`, `Duration`, `Clef`) and renders to SVG. Bridges music types to internal layout pipeline: `duration_kind_to_log2()` converts `DurationKind` → layout log2, `should_show_accidental()` resolves accidental display against key signature (shows naturals when cancelling key sig sharps/flats), `note_altered_in_key()` checks sharp/flat order. Builder supports: `.clef()`, `.key_signature()`, `.time_signature()`, `.note()`, `.rest()`, `.barline()`, `.end_barline()`, `.barline_style()`, `.measures_per_system()`, `.system_width_fu()`, `.render_svg()`. Auto-flushes pending events as final barline. Uses `ClefKind` internally to avoid `Clef`'s missing Clone/Debug derives. Created `examples/score_builder.rs` rendering a 4-measure melody in D major across 2 systems.
- Verified: `cargo test -p music-engraver` — 461 unit + 3 integration + 1 doc-test = 465 tests, all pass. 29 new score tests (duration conversion, key sig accidental logic, accidental display rules, SVG output validation for single/multi-measure/dotted/eighth/rest/bass clef scores, measures_per_system effect, event conversion). `cargo run --example score_builder` produces valid SVG (31 paths, 26 lines, 20944 bytes). `cargo check --workspace` passes.
- Next: Phase 8 continuation — add `README.md` for the crate, or add `CommonTime`/`CutTime` convenience methods to ScoreBuilder, or begin polish (error handling, documentation).
- Open issues: None.

## 2026-04-18 — Phase 8, smart accidental display
- Did: Fixed `should_show_accidental()` in `score.rs` to suppress accidentals that are redundant with the key signature. F# in D major (2 sharps) no longer shows a sharp; Bb in Bb major (2 flats) no longer shows a flat. Naturals still shown when cancelling key sig alterations (e.g. F♮ in D major). Double sharps/flats always shown (never in key signatures). Sharp in flat key and flat in sharp key correctly shown. Updated existing test, added 7 new tests covering all combinations: sharp suppressed in sharp key, flat suppressed in flat key, sharp shown in flat key, sharp shown when letter not in sharp key, flat shown in sharp key, flat shown when letter not in flat key, double sharp/flat shown even in matching key.
- Verified: `cargo test -p music-engraver` — 468 unit + 3 integration + 1 doc-test = 472 tests, all pass. `cargo check --workspace` passes.
- Next: Phase 8 continuation — add `README.md` for the crate, or add `CommonTime`/`CutTime` convenience methods to ScoreBuilder, or begin polish (error handling, pub doc comments on all public items).

## 2026-04-18 — Phase 8, README + common/cut time convenience methods
- Did: Created `music-engraver/README.md` with quick-start example, feature list, example commands, architecture table, and license info. Added `common_time()` and `cut_time()` convenience methods to `ScoreBuilder` — these set the time signature display to the Common (C) or CutCommon (₵) symbol respectively instead of numeric digits. Added `time_sig_kind` field to `ScoreBuilder` for display style override. 3 new tests verifying common vs numeric, cut vs common, and cut vs numeric 2/2 produce distinct SVGs.
- Verified: `cargo test -p music-engraver` — 471 unit + 3 integration + 1 doc-test = 475 tests, all pass. `cargo check --workspace` passes.
- Next: Phase 8 polish — add pub doc comments on all public types/functions, or add within-measure accidental tracking (courtesy accidentals), or consider the crate done for v1 scope.

## 2026-04-18 — Phase 8, within-measure accidental tracking
- Did: Implemented within-measure accidental state tracking in `score.rs`. When the same pitch (letter+octave) appears multiple times in a measure, the accidental is only shown on the first occurrence. If a different accidental follows (e.g. F# then F♮), a courtesy natural is displayed. Tracking resets at each barline. Added `AccidentalTracker` type alias, `note_key()`, `effective_accidental()`, and `resolve_accidental()` (with optional tracking map). `render_svg()` now uses `convert_event_tracked()` per measure with a fresh tracking map. 15 new tests covering: suppression of repeated sharps/flats, courtesy naturals after sharps/flats, different-octave independence, tracked convert_event suppression/natural display, cross-barline reset, within-measure suppression (SVG path count verification), effective_accidental states, note_key identity/enharmonic distinction.
- Verified: `cargo test -p music-engraver` — 486 unit + 3 integration + 1 doc-test = 490 tests, all pass. `cargo check --workspace` passes. No warnings.
- Next: Phase 8 polish — add pub doc comments on all public types/functions, or consider the crate done for v1 scope.
- Open issues: None.

## 2026-04-18 — Phase 8, clippy cleanup + crate/module doc comments
- Did: Fixed all 9 clippy warnings in `music-engraver`: needless borrows in `key_signature.rs`, manual `RangeInclusive::contains` in `staff.rs`, `saturating_sub` for arithmetic check in `measure_renderer.rs`, `#[allow(clippy::too_many_arguments)]` on 3 private beam_renderer helpers + 1 pub `draw_stemmed_note`. Added crate-level doc comment to `lib.rs` (with usage example), module-level doc comments to `font/mod.rs`, `layout/mod.rs`, `render/mod.rs`. Crate-level doc example is now a doc-test (2 doc-tests total).
- Verified: `cargo test -p music-engraver` — 486 unit + 3 integration + 2 doc-tests = 491 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings. `cargo doc -p music-engraver --no-deps` — clean. `cargo check --workspace` passes.
- Next: Phase 8 polish — add doc comments to remaining public types/functions across layout/render submodules, or consider the crate done for v1 scope.
- Open issues: None.

## 2026-04-18 — Phase 8, try_render_svg + remaining doc comments
- Did: Added `try_render_svg()` method to `ScoreBuilder` returning `Result<String, FontError>` instead of panicking. Existing `render_svg()` now delegates to `try_render_svg().expect(...)`. Added doc comments to `StaffLayout::new()`, `ClefKind::from_clef()`, `ClefKind::to_clef()`. 3 new tests: `try_render_svg` returns Ok for valid input, empty score, and matches `render_svg` output.
- Verified: `cargo test -p music-engraver` — 489 unit + 3 integration + 2 doc-tests = 494 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Consider the crate done for v1 scope, or add chord support (multiple simultaneous pitches) to ScoreBuilder as a stretch goal.
- Open issues: None.
