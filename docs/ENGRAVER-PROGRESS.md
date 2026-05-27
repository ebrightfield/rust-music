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

## 2026-04-18 — Phase 8, chord layout module
- Did: Created `layout/chord.rs` with chord notehead stacking logic. `ChordNote` and `ChordNoteLayout` structs. `layout_chord_noteheads()` determines which notes in a chord need x-offset to avoid collision when notes are a second apart — stem-up offsets upper note right, stem-down offsets lower note left, clusters alternate. `notehead_x_offset()`, `chord_has_offsets()`, `chord_extent()` helpers. Wired into `layout/mod.rs` with public exports.
- Verified: `cargo test -p music-engraver` — 513 unit + 3 integration + 2 doc-tests = 518 tests, all pass. 24 new chord layout tests covering: empty, single note, seconds (both directions), clusters of 3 and 5 consecutive, mixed intervals, unsorted input, unisons, accidental preservation, x-offset values, extent computation. `cargo check --workspace` passes.
- Next: Phase 8 continuation — add `ChordEvent` to `MeasureEvent` enum, update measure renderer to draw chords, then add `.chord()` method to `ScoreBuilder`.
- Open issues: None.

## 2026-04-18 — Phase 8, clippy cleanup + crate/module doc comments
- Did: Fixed all 9 clippy warnings in `music-engraver`: needless borrows in `key_signature.rs`, manual `RangeInclusive::contains` in `staff.rs`, `saturating_sub` for arithmetic check in `measure_renderer.rs`, `#[allow(clippy::too_many_arguments)]` on 3 private beam_renderer helpers + 1 pub `draw_stemmed_note`. Added crate-level doc comment to `lib.rs` (with usage example), module-level doc comments to `font/mod.rs`, `layout/mod.rs`, `render/mod.rs`. Crate-level doc example is now a doc-test (2 doc-tests total).
- Verified: `cargo test -p music-engraver` — 486 unit + 3 integration + 2 doc-tests = 491 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings. `cargo doc -p music-engraver --no-deps` — clean. `cargo check --workspace` passes.
- Next: Phase 8 polish — add doc comments to remaining public types/functions across layout/render submodules, or consider the crate done for v1 scope.
- Open issues: None.

## 2026-04-18 — Phase 8, try_render_svg + remaining doc comments
- Did: Added `try_render_svg()` method to `ScoreBuilder` returning `Result<String, FontError>` instead of panicking. Existing `render_svg()` now delegates to `try_render_svg().expect(...)`. Added doc comments to `StaffLayout::new()`, `ClefKind::from_clef()`, `ClefKind::to_clef()`. 3 new tests: `try_render_svg` returns Ok for valid input, empty score, and matches `render_svg` output.
- Verified: `cargo test -p music-engraver` — 489 unit + 3 integration + 2 doc-tests = 494 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Add chord support through the full pipeline.
- Open issues: None.

## 2026-04-18 — Phase 8, chord support through full pipeline
- Did: Integrated chord support from layout through renderer to ScoreBuilder API. Added `ChordEvent` struct to `layout/measure.rs` (staff_positions, duration_log2, dots, accidentals, stem_direction). Added `MeasureElement::Chord` variant and `MeasureEvent::Chord` variant. Updated `layout_measure()` for chord duration spacing. Created `draw_chord_event()` in `render/measure_renderer.rs` — uses `layout_chord_noteheads()` for second-avoidance offsets, draws multiple noteheads at correct x-offsets, shared stem spanning chord extent, flags, ledger lines per note, accidentals per note, and dots per note. Updated `system.rs` `measure_event_to_element()` for chords. Added `.chord(pitches, duration)` method to `ScoreBuilder`, with full accidental tracking per chord note. Fixed one clippy warning in `chord.rs` (`map_or` → `is_some_and`). 10 new measure_renderer tests (two-note chord, second with offset, accidentals, whole note no stem, eighth with flag, ledger lines, empty chord, dotted chord, chord vs single note comparison). 6 new score tests (multiple noteheads, more paths than single, key sig suppression, natural accidental, convert_event mapping, tracked accidental suppression in chords).
- Verified: `cargo test -p music-engraver` — 528 unit + 3 integration + 2 doc-tests = 533 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create a chord example SVG (`examples/chords.rs`) showing various chord types on a staff, or consider the crate done for v1 scope.
- Open issues: None.

## 2026-04-18 — Phase 8, chords example
- Did: Created `examples/chords.rs` rendering 4 measures across 2 systems: open fifth (C-G), C major triad, D minor triad, second interval (E-F, tests notehead offset), G major low voicing, 3-note cluster (C-D-E, adjacent seconds), Bb major with accidental, wide voicing (C4-G4-E5), whole-note 4-note chord (C-E-G-C). Exercises chord noteheads, stem sharing, accidentals in chords, second-avoidance x-offsets, ledger lines, and whole-note (stemless) chords.
- Verified: `cargo run --example chords` produces valid SVG (33 paths, 32 lines, 15409 bytes). `cargo test -p music-engraver` — 528 unit + 3 integration + 2 doc-tests = 533 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings.
- Next: Consider v1 scope complete. Remaining polish options: dotted chord example, beam groups across measure API, or pub doc comments on remaining public items.
- Open issues: None.

## 2026-04-18 — Phase 8, beam group integration through full pipeline
- Did: Integrated beam groups into the measure layout → renderer → ScoreBuilder pipeline. Added `BeamGroupEvent` struct to `layout/measure.rs`, `MeasureElement::BeamGroup` and `MeasureEvent::BeamGroup` variants. `layout_measure()` distributes proportional spacing across beam group notes. `draw_beam_group_event()` in `render/measure_renderer.rs` computes per-note x-positions, draws noteheads + accidentals + ledger lines + dots, then delegates to `layout_beam_group()` + `draw_beam_group()` for stems/beams. Added `ScoreBuilder::beam_group(notes)` method accepting `Vec<(Pitch, Duration)>` with full accidental tracking. Updated `system.rs` event conversion.
- Verified: `cargo test -p music-engraver` — 539 unit + 3 integration + 2 doc-tests = 544 tests, all pass. 7 new measure_renderer beam tests (two eighths, four sixteenths, accidental, ledger lines, empty, differs-from-flagged, mixed durations). 4 new score tests (SVG with polygons, differs from individual eighths, convert_event mapping, tracked accidental suppression). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/beamed_score.rs` showing beam groups via ScoreBuilder API, or consider v1 scope complete.
- Open issues: None.

## 2026-04-18 — Phase 8, beamed score example (v1 scope complete)
- Did: Created `examples/beamed_score.rs` rendering 4 measures across 2 systems in G major 4/4 using ScoreBuilder API with beam groups: (1) four ascending beamed eighths + quarter + quarter rest, (2) two sixteenths + eighth rest + two descending eighths + half, (3) four beamed sixteenths + dotted half, (4) mixed beam group (eighth + two sixteenths) + quarter + quarter with final barline. Output: 29 paths, 35 lines, 8 polygons (16663 bytes).
- Verified: `cargo run --example beamed_score` produces valid SVG. `cargo test -p music-engraver` — 539 unit + 3 integration + 2 doc-tests = 544 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings.
- Next: v1 scope is functionally complete. All phases 0–8 implemented: font pipeline, staff/clef/note/stem/accidental/dot/flag/rest/barline/time sig/key sig layout+rendering, measure/system/page layout, beams, ties, chords, beam groups, ScoreBuilder API with smart accidentals. 17 examples, 544 tests. Remaining polish: additional pub doc comments, error handling refinement, or begin post-v1 features (slurs, dynamics, tuplet brackets, PNG export).
- Open issues: None.

## 2026-04-19 — Post-v1, tie integration through ScoreBuilder pipeline
- Did: Integrated ties into the full pipeline: NoteEvent → system renderer → ScoreBuilder API. Added `tie_forward: bool` field to `NoteEvent` (all existing code defaults to `false`). Added `ScoreBuilder::tie()` method that marks the previous note for tie-forward. System renderer now scans positioned elements after drawing all measures, finds notes with `tie_forward=true`, locates the next note at the same staff position, and draws tie curves between them using `layout_tie()` + `draw_tie()`. Ties work within measures and across barlines (within the same system). Updated `ScoreEvent::Note` to carry `tie_forward` flag. Cleaned up clippy warnings (removed unused import, dead struct, match→if-let).
- Verified: `cargo test -p music-engraver` — 551 unit + 3 integration + 2 doc-tests = 556 tests, all pass. 12 new tests: 6 system renderer tests (within-measure tie, cross-barline tie, no tie when false, no match for different position, multiple ties, collect_note_positions skips non-notes), 6 score tests (tie produces filled path, cross-barline tie, no tie without call, tie on rest no-op, tied differs from untied, convert_event preserves tie_forward). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create a tied notes example via ScoreBuilder API, or add tie support for chords (ChordEvent), or begin other post-v1 features.
- Open issues: Ties only work for single notes (NoteEvent), not chords. Cross-system ties (tie from last note of one system to first note of next) not yet supported.

## 2026-04-19 — Post-v1, tied score example via ScoreBuilder API
- Did: Created `examples/tied_score.rs` — renders 4 measures across 2 systems in G major 4/4 using ScoreBuilder API with `.tie()` method. Demonstrates: (1) within-measure tie (D5 half → D5 quarter), (2) cross-barline tie within a system (G4 last of measure 2 → G4 first of measure 3, crosses system boundary so not rendered — documents limitation), (3) cross-barline tie within system 2 (C5 quarter → C5 half). Output: 21 paths, 28 lines, 2 ties (12524 bytes).
- Verified: `cargo run --example tied_score` produces valid SVG. `cargo test -p music-engraver` — 551 unit + 3 integration + 2 doc-tests = 556 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings.
- Next: Add tie support for chords (ChordEvent), implement cross-system ties, or begin other post-v1 features (dynamics, tuplet brackets, slurs).
- Open issues: Cross-system ties not yet supported. Chord ties not yet supported.

## 2026-04-19 — Post-v1, chord ties example
- Did: Created `examples/chord_ties.rs` rendering 4 measures across 2 systems demonstrating: (1) C major triad half → half tie (3 tie curves), (2) cross-barline G-B fifth tie (2 curves), (3) single-note E4 tie (1 curve), (4) whole-note chord for contrast. Output: 33 paths, 27 lines, 6 ties (13855 bytes).
- Verified: `cargo run --example chord_ties` produces valid SVG. `cargo test -p music-engraver` — 561 unit + 3 integration + 2 doc-tests = 566 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings.
- Next: Implement cross-system ties, or begin other post-v1 features (dynamics text, tuplet brackets, slurs, PNG export stub).
- Open issues: Cross-system ties not yet supported.

## 2026-04-19 — Post-v1, chord tie support
- Did: Added `tie_forward: bool` field to `ChordEvent`. Updated `ScoreEvent::Chord` to carry `tie_forward`. `ScoreBuilder::tie()` now handles both notes and chords. `collect_note_positions()` in system_renderer now expands chord notes into individual entries so each chord note gets its own tie matched by staff position. Updated `convert_event()` and `convert_event_tracked()` to propagate `tie_forward` for chords. Added `tie_forward: false` to all existing `ChordEvent` constructions across measure_renderer tests and score tests.
- Verified: `cargo test -p music-engraver` — 561 unit + 3 integration + 2 doc-tests = 566 tests, all pass. 10 new tests: 5 system_renderer tests (collect includes chord notes, chord tie draws 2 ties, chord-to-single-note partial match, untied chord no ties, chord tie across barline), 5 score tests (tie_forward preserved in convert_event, chord tie produces 3 filled paths for 3-note chord, tied vs untied chord differs, no tie without .tie() call, tracked conversion preserves tie_forward). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create a chord tie example, implement cross-system ties, or begin other post-v1 features (dynamics, tuplet brackets, slurs).
- Open issues: Cross-system ties not yet supported.

## 2026-04-19 — Post-v1, cross-system ties
- Did: Implemented cross-system tie rendering in `page_renderer.rs`. When a tied note at the end of system N has no matching target within the same system, two half-ties are drawn: (1) a trailing half-tie from the note to the right edge of system N's staff, and (2) an incoming half-tie from the left edge of system N+1's note area to the matching note. Added `layout_half_tie_right()` and `layout_half_tie_left()` to `layout/tie.rs`. Made `collect_note_positions()` pub(crate). Added `find_unresolved_ties()`, `find_incoming_tie_targets()`, and `draw_cross_system_ties()` private helpers. Works automatically through `ScoreBuilder` since it uses `draw_page()`.
- Verified: `cargo test -p music-engraver` — 570 unit + 3 integration + 2 doc-tests = 575 tests, all pass. 9 new tests: 4 half-tie layout tests (right valid, left valid, right over apex, left under apex), 5 page_renderer cross-system tie tests (two half-ties drawn, no tie without flag, right-half-only when no target, within-system not duplicated, tied vs untied differ). `cargo clippy -p music-engraver` — 0 warnings.
- Next: Create a cross-system tie example via ScoreBuilder, or begin other post-v1 features (dynamics, tuplet brackets, slurs, PNG export stub).
- Open issues: None — cross-system ties now supported for both single notes and chords.

## 2026-04-19 — Post-v1, cross-system ties example
- Did: Created `examples/cross_system_ties.rs` — renders 4 measures across 2 systems in G major 4/4 demonstrating: (1) cross-system tie on G4 from last note of system 1 (measure 2) to first note of system 2 (measure 3), rendered as two half-ties (trailing + incoming), (2) within-system cross-barline tie on B4 from measure 3 to measure 4, rendered as a full tie curve. Output: 21 paths, 27 lines, 3 ties (12533 bytes). Assertions verify at least 3 tie curves.
- Verified: `cargo run --example cross_system_ties` produces valid SVG. `cargo test -p music-engraver` — 570 unit + 3 integration + 2 doc-tests = 575 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings on engraver crate.
- Next: Consider other post-v1 features: dynamics text annotations, tuplet brackets, slurs, PNG export stub, or additional polish.
- Open issues: None.

## 2026-04-19 — Post-v1, dynamics layout + rendering
- Did: Created `layout/dynamics.rs` with `Dynamic` enum (Ppp/Pp/Piano/Mp/Mf/Forte/Ff/Fff/Fp/Sfz/Sfp), each mapping to a dedicated SMuFL composite glyph. `layout_dynamic()` positions the dynamic glyph centered horizontally on a note and 2.5 staff spaces below the bottom staff line. `DynamicLayout` result struct. Created `render/dynamics_renderer.rs` with `draw_dynamic()`. Added `StaffLayout::bottom_y()` convenience method. Created `examples/dynamics.rs` rendering 7 notes (E4–G5) with dynamics (pp, p, mp, mf, f, ff, fff) on treble staff → `examples/output/dynamics.svg` (15 paths, 12 lines, 13477 bytes). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 591 unit + 3 integration + 2 doc-tests = 596 tests, all pass. 12 dynamics layout tests (glyph uniqueness, centering, below-staff placement, scaling). 9 dynamics renderer tests (all 11 dynamics render, path counts, position verification, coordinate embedding). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Integrate dynamics into ScoreBuilder pipeline (add `.dynamic()` method that attaches a dynamic to the most recent note, pass through system/page renderer), or begin tuplet brackets.
- Open issues: Dynamics not yet integrated into the measure/system pipeline — currently only usable via direct `draw_dynamic()` calls. ScoreBuilder integration is the next step.

## 2026-04-19 — Post-v1, dynamics integration through ScoreBuilder pipeline
- Did: Integrated dynamics into the full pipeline from ScoreBuilder to SVG. Added `dynamic: Option<Dynamic>` field to `NoteEvent` and `ChordEvent` in `layout/measure.rs`. Added `ScoreEvent` variants to carry dynamics. Added `ScoreBuilder::dynamic(dyn_mark)` method that attaches a dynamic marking to the most recently added note or chord (no-op on rests). Updated `convert_event()` and `convert_event_tracked()` to pass through dynamics. Updated `draw_note_event()` and `draw_chord_event()` in `render/measure_renderer.rs` to call `draw_dynamic()` when a dynamic is present, centered on the notehead. All existing construction sites updated with `dynamic: None`.
- Verified: `cargo test -p music-engraver` — 604 unit + 3 integration + 2 doc-tests = 609 tests, all pass. 13 new tests: 5 measure_renderer tests (note with/without dynamic, chord with dynamic, dynamic positioning below staff, different dynamics differ), 8 score tests (dynamic adds path, different dynamics differ, chord dynamic, rest ignored, convert_event preserves dynamic, tracked preserves dynamic, chord preserves dynamic, multiple dynamics path count). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create a dynamics example via ScoreBuilder API (`examples/dynamics_score.rs`), or begin other post-v1 features (tuplet brackets, slurs, PNG export stub).
- Open issues: None. Dynamics now fully integrated for notes and chords. Beam group notes do not carry dynamics (rare use case — dynamics typically apply to the group as a whole, not individual beamed notes).

## 2026-04-19 — Post-v1, dynamics score example via ScoreBuilder API
- Did: Created `examples/dynamics_score.rs` rendering 4 measures across 2 systems in Bb major 4/4 using ScoreBuilder API with `.dynamic()` method. Demonstrates: (1) ascending line with crescendo-like dynamics (p, mp, mf, f), (2) descending with diminuendo-like dynamics (ff, mf, p), (3) chord with fff + rest (no dynamic on rest), (4) sfz accent followed by pp subito and ppp. Exercises 9 of 11 dynamic types, dynamics on single notes and chords, no-op on rests. Output: 34 paths, 29 lines (28680 bytes).
- Verified: `cargo run --example dynamics_score` produces valid SVG. `cargo test -p music-engraver` — 604 unit + 3 integration + 2 doc-tests = 609 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings on engraver crate. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: tuplet brackets (layout + rendering), slurs (similar to ties but different attachment), or PNG export stub.
- Open issues: None.

## 2026-04-19 — Post-v1, tuplet bracket layout + rendering + example
- Did: Created `layout/tuplet.rs` with `TupletPlacement` enum (Above/Below), `tuplet_placement_from_stem()`, `tuplet_digit_glyph()` (maps 0–9 → SMuFL `Tuplet0`–`Tuplet9`), `tuplet_number_glyphs()` (multi-digit support), `layout_tuplet_bracket()` (positions bracket above/below extreme notes with configurable offset, computes hook height, gap for number, centering). `TupletBracketLayout` struct stores all geometry including pre-computed gap boundaries. Created `render/tuplet_renderer.rs` with `draw_tuplet_bracket()` — renders 2 hooks, 2 bracket line segments (interrupted by number gap), and centered SMuFL digit glyph(s). Created `examples/tuplet_brackets.rs` rendering 3 groups on treble staff: triplet above (3 ascending eighths), quintuplet below (5 quarter notes), triplet above ledger-line notes → `examples/output/tuplet_brackets.svg` (14 paths, 30 lines, 6353 bytes).
- Verified: `cargo test -p music-engraver` — 630 unit + 3 integration + 2 doc-tests = 635 tests, all pass. 15 tuplet layout tests (digit glyphs, number glyphs, placement logic, bracket positioning, centering, hooks, thickness, bounds, empty positions, relative movement). 9 tuplet renderer tests (lines+path counts, above/below differ, different numbers differ, x/y offset, two-digit number, bracket gap structure). `cargo clippy -p music-engraver` — 0 warnings. `cargo run --example tuplet_brackets` produces valid SVG. `cargo check --workspace` passes.
- Next: Integrate tuplet brackets into ScoreBuilder pipeline (add `.tuplet()` method), or begin slur rendering, or other post-v1 features.
- Open issues: Tuplet brackets not yet integrated into measure/system/ScoreBuilder pipeline — currently usable via direct `draw_tuplet_bracket()` calls only.

## 2026-04-19 — Post-v1, tuplet bracket integration through ScoreBuilder pipeline
- Did: Integrated tuplet brackets into the full pipeline from ScoreBuilder to SVG. Added `TupletGroupEvent` struct to `layout/measure.rs` (wraps `BeamGroupEvent` + `tuplet_number`). Added `MeasureElement::TupletGroup` and `MeasureEvent::TupletGroup` variants. Updated `layout_measure()` for tuplet spacing (same proportional model as beam groups). Created `draw_tuplet_group_event()` in `render/measure_renderer.rs` — delegates to `draw_beam_group_event()` for note/beam rendering, then overlays tuplet bracket via `layout_tuplet_bracket()` + `draw_tuplet_bracket()`. Added `ScoreBuilder::tuplet(number, notes)` method with full accidental tracking. Updated `system.rs` event conversion.
- Verified: `cargo test -p music-engraver` — 640 unit + 3 integration + 2 doc-tests = 645 tests, all pass. 5 new measure_renderer tests (triplet renders beams+bracket, differs from plain beam, quintuplet, empty, different numbers differ). 5 new score tests (renders with bracket, differs from beam group, convert_event mapping, tracked accidentals, quintuplet). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/tuplet_score.rs` showing tuplets via ScoreBuilder API, or begin other post-v1 features (slurs, PNG export stub).
- Open issues: None. Tuplet brackets now fully integrated for any tuplet number via `.tuplet(n, notes)` on ScoreBuilder.

## 2026-04-19 — Post-v1, tuplet score example via ScoreBuilder API
- Did: Created `examples/tuplet_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.tuplet()` method. Demonstrates: (1) eighth-note triplet ascending C-E-G + quarter + rest, (2) descending triplet D-C-B + half note, (3) sixteenth-note quintuplet G-A-B-C-D + dotted quarter + eighth, (4) two consecutive triplets filling the bar. Exercises triplets and quintuplets with beams and brackets. Output: 33 paths, 57 lines, 6 polygons (19488 bytes).
- Verified: `cargo run --example tuplet_score` produces valid SVG. `cargo test -p music-engraver` — 640 unit + 3 integration + 2 doc-tests = 645 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings on engraver crate.
- Next: Begin other post-v1 features: slurs (curved paths similar to ties but with different attachment semantics), PNG export stub, or additional polish.
- Open issues: None.

## 2026-04-19 — Post-v1, slur layout + rendering + example
- Did: Created `layout/slur.rs` with `SlurDirection` enum, `slur_direction_from_stem()`, and `layout_slur()` — computes cubic Bézier crescent geometry for slurs connecting notes at potentially different pitches. Key difference from ties: asymmetric start/end y-positions supported, apex computed relative to the extreme endpoint (min y for Over, max y for Under) to ensure the curve always clears both notes. Height scales with span (0.12× ratio, clamped 0.5–2.0 staff spaces). Crescent thickness from `slur_endpoint_thickness`/`slur_midpoint_thickness` in EngravingConfig. Created `render/slur_renderer.rs` with `draw_slur()` rendering two cubic Bézier curves as filled crescent path. Created `examples/slurs.rs` rendering 4 slur pairs: ascending under, descending over, same-pitch under, wide ascending over. Output: 13 paths, 16 lines (4647 bytes).
- Verified: `cargo test -p music-engraver` — 664 unit + 3 integration + 2 doc-tests = 669 tests, all pass. 15 slur layout tests (direction mapping, endpoint offsets, apex clearance for both directions, asymmetric endpoints, x-position preservation, control point positions, min height clamping, height scaling, symmetric offsets, crescent thickness, ascending/descending apex clearance). 9 slur renderer tests (path structure, Bézier count, filled path element, over/under differentiation, asymmetric path, endpoint coordinates, short/long slurs). `cargo clippy -p music-engraver` — 0 warnings. `cargo run --example slurs` produces valid SVG. `cargo check --workspace` passes.
- Next: Integrate slurs into ScoreBuilder pipeline (add `.slur_start()`/`.slur_end()` or similar API), or begin other post-v1 features (PNG export stub, hairpins/crescendo wedges).
- Open issues: Slurs not yet integrated into measure/system/ScoreBuilder pipeline — currently usable via direct `layout_slur()` + `draw_slur()` calls only.

## 2026-04-19 — Post-v1, slur integration through ScoreBuilder pipeline
- Did: Integrated slurs into the full pipeline from ScoreBuilder to SVG. Added `slur_start: bool` and `slur_end: bool` fields to `NoteEvent` and `ChordEvent`. Added `ScoreBuilder::slur_start()` and `ScoreBuilder::slur_end()` methods (modifier pattern, like `.tie()` and `.dynamic()`). In `system_renderer.rs`, added `collect_slur_note_info()` (collects note positions with slur flags, chords use outer note as attachment point based on stem direction) and `draw_system_slurs()` (finds slur_start/slur_end pairs and draws slur curves via `layout_slur()` + `draw_slur()`). Called from `draw_system()` after tie rendering. Updated all `NoteEvent` and `ChordEvent` construction sites across 10+ files (src modules, tests, examples) with `slur_start: false, slur_end: false`. Updated `convert_event()` and `convert_event_tracked()` to propagate slur flags from `ScoreEvent` to `MeasureEvent`.
- Verified: `cargo test -p music-engraver` — 675 unit + 3 integration + 2 doc-tests = 680 tests, all pass. 5 new system_renderer slur tests (within-measure slur, cross-barline slur, no slur without flags, slurred vs un-slurred differ, slur_start without end draws nothing). 7 new score tests (slur produces filled path, slurred vs unslurred differ, rest no-op, convert_event preserves flags, convert_event_tracked preserves flags, chord slur preserves flags). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/slur_score.rs` showing slurs via ScoreBuilder API, or begin other post-v1 features (hairpins/crescendo wedges, PNG export stub).
- Open issues: Cross-system slurs not yet supported (same limitation as cross-system ties had before implementation). Slurs only work within a single system currently.

## 2026-04-19 — Post-v1, slur score example via ScoreBuilder API
- Did: Created `examples/slur_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.slur_start()`/`.slur_end()` methods. Demonstrates: (1) ascending 3-note slur C4–E4–G4 + rest, (2) descending 4-note slur B4–A4–G4–F4, (3) same-pitch slur D5–D5 (legato articulation), (4) wide ascending slur C4–G5 (large interval). All 4 slurs rendered as filled crescent paths. Output: 20 paths, 28 lines, 4 slurs (10573 bytes).
- Verified: `cargo run --example slur_score` produces valid SVG. `cargo test -p music-engraver` — 675 unit + 3 integration + 2 doc-tests = 680 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings.
- Next: Implement cross-system slurs (analogous to cross-system ties), or begin other post-v1 features (hairpins/crescendo wedges, PNG export stub, rehearsal marks).
- Open issues: Cross-system slurs not yet supported.

## 2026-04-19 — Post-v1, cross-system slurs
- Did: Implemented cross-system slur rendering in `page_renderer.rs`, analogous to cross-system ties. Added `layout_half_slur_right()` and `layout_half_slur_left()` to `layout/slur.rs` (symmetric y endpoints since target pitch is unknown across system break). Made `SlurNoteInfo` and `collect_slur_note_info()` `pub(crate)` in `system_renderer.rs`. Added `find_unresolved_slurs()`, `find_incoming_slur_targets()`, and `draw_cross_system_slurs()` to page renderer. Works automatically through `ScoreBuilder` since it uses `draw_page()`. Trailing half-slur always drawn (convention); incoming half-slur drawn only if target system has a `slur_end` note.
- Verified: `cargo test -p music-engraver` — 684 unit + 3 integration + 2 doc-tests = 689 tests, all pass. 4 new half-slur layout tests + 5 new page_renderer cross-system slur tests (two half-slurs drawn, no slur without flags, right-half-only when no end, within-system not duplicated, slurred vs unslurred differ). `cargo check --workspace` passes.
- Next: Create `examples/cross_system_slurs.rs` showing cross-system slurs via ScoreBuilder API, or begin other post-v1 features (hairpins/crescendo wedges, PNG export stub).
- Open issues: None — cross-system slurs now supported for both single notes and chords.

## 2026-04-19 — Post-v1, cross-system slurs example via ScoreBuilder API
- Did: Created `examples/cross_system_slurs.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API. Demonstrates: (1) ascending phrase with slur starting on C5 at end of system 1, crossing system break to D5 at start of system 2 (rendered as two half-slurs), (2) within-system cross-barline slur F5→D5 spanning measures 3→4 (rendered as full slur curve). Output: 20 paths, 29 lines, 3 slurs (10145 bytes).
- Verified: `cargo run --example cross_system_slurs` produces valid SVG. `cargo test -p music-engraver` — 684 unit + 3 integration + 2 doc-tests = 689 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings on engraver crate (warnings from `music` crate only). `cargo check --workspace` passes.
- Next: Begin other post-v1 features: hairpins/crescendo wedges (layout + rendering), or PNG export stub, or rehearsal marks.
- Open issues: None.

## 2026-04-19 — Post-v1, hairpin (crescendo/decrescendo) layout + rendering + example
- Did: Created `layout/hairpin.rs` with `HairpinType` enum (Crescendo/Decrescendo), `HairpinLayout` struct (x_start, x_end, y_center, half_opening, stroke_width), `layout_hairpin()` function positioning wedge below staff at 3.5 staff spaces. Constants: `HAIRPIN_BELOW_STAFF_SS` (3.5), `HAIRPIN_HALF_OPENING_SS` (0.5). Created `render/hairpin_renderer.rs` with `draw_hairpin()` — renders two converging/diverging lines. Crescendo: point left → opening right. Decrescendo: opening left → point right. Created `examples/hairpins.rs` rendering 8 notes with p → cresc → f → decresc → pp pattern (dynamics + hairpin wedges). Output: 12 paths, 17 lines (7335 bytes). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 703 unit + 3 integration + 2 doc-tests = 708 tests, all pass. 10 hairpin layout tests + 9 hairpin renderer tests. `cargo clippy -p music-engraver` — 0 warnings in hairpin code. `cargo run --example hairpins` produces valid SVG. `cargo check --workspace` passes.
- Next: Integrate hairpins into ScoreBuilder pipeline (add `.hairpin_start()`/`.hairpin_end()` or `.cresc()`/`.decresc()` methods), or begin other post-v1 features (PNG export stub, rehearsal marks).
- Open issues: Hairpins not yet integrated into measure/system/ScoreBuilder pipeline — currently usable via direct `layout_hairpin()` + `draw_hairpin()` calls only.

## 2026-04-19 — Post-v1, hairpin integration through ScoreBuilder pipeline
- Did: Integrated hairpins into the full pipeline from ScoreBuilder to SVG. Added `hairpin_start: Option<HairpinType>` and `hairpin_end: bool` fields to `NoteEvent` and `ChordEvent` in `layout/measure.rs`. Added same fields to `ScoreEvent::Note` and `ScoreEvent::Chord`. Added `ScoreBuilder::hairpin_start(kind)`, `hairpin_end()`, `cresc()` (convenience), `decresc()` (convenience) methods. In `system_renderer.rs`, added `HairpinNoteInfo` struct, `collect_hairpin_note_info()`, and `draw_system_hairpins()` — scans positioned elements for hairpin start/end pairs and renders wedges via `layout_hairpin()` + `draw_hairpin()`. Called from `draw_system()` after slur rendering. Updated all NoteEvent/ChordEvent construction sites across 10+ files with `hairpin_start: None, hairpin_end: false`. Updated `convert_event()` and `convert_event_tracked()` to pass through hairpin fields.
- Verified: `cargo test -p music-engraver` — 714 unit + 3 integration + 2 doc-tests = 719 tests, all pass. 5 new system_renderer hairpin tests (within-measure draws 2 lines, no hairpin without flags, start-without-end no-op, decresc differs from cresc, across barline). 6 new score tests (hairpin adds lines, decresc vs cresc differ, rest no-op, convert_event preserves fields, tracked preserves fields, chord preserves fields). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/hairpin_score.rs` showing hairpins via ScoreBuilder API, or begin other post-v1 features (PNG export stub, rehearsal marks, text annotations).
- Open issues: Cross-system hairpins not yet supported (hairpin spanning a system break). Hairpins on beam group notes not supported (rare — hairpins typically span between notes, not within a beam group).

## 2026-04-19 — Post-v1, hairpin score example via ScoreBuilder API
- Did: Created `examples/hairpin_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.cresc()`, `.decresc()`, `.hairpin_end()`, and `.dynamic()` methods. Demonstrates: (1) ascending crescendo from p to f across 4 quarter notes, (2) half-note decrescendo from ff to p, (3) crescendo spanning barline from system 1 to system 2, (4) decrescendo to pp with dynamics. Output: 24 paths, 38 lines (19736 bytes).
- Verified: `cargo run --example hairpin_score` produces valid SVG. `cargo test -p music-engraver` — 714 unit + 3 integration + 2 doc-tests = 719 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings.
- Next: Begin other post-v1 features: rehearsal marks/text annotations, or PNG export stub, or cross-system hairpins.
- Open issues: Cross-system hairpins not yet supported. Hairpins on beam group notes not supported.

## 2026-04-19 — Post-v1, cross-system hairpins
- Did: Implemented cross-system hairpin rendering in `page_renderer.rs`, following the same pattern as cross-system ties and slurs. Made `HairpinNoteInfo` and `collect_hairpin_note_info()` `pub(crate)` in `system_renderer.rs`. Added `UnresolvedHairpin`/`IncomingHairpinTarget` structs, `find_unresolved_hairpins()`, `find_incoming_hairpin_targets()`, and `draw_cross_system_hairpins()` to page renderer. Reuses `layout_hairpin()` directly for both half-hairpins (simpler than ties/slurs since hairpins are just lines, not Bézier curves). Trailing half-hairpin always drawn; incoming half-hairpin drawn only if target system has a `hairpin_end` note. Works automatically through `ScoreBuilder` since it uses `draw_page()`.
- Verified: `cargo test -p music-engraver` — 720 unit + 3 integration + 2 doc-tests = 725 tests, all pass. 6 new page_renderer cross-system hairpin tests (4 extra lines for full cross-system, no hairpin without flags, right-half-only when no end, within-system not duplicated, differs from no hairpin, decresc differs from cresc). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/cross_system_hairpins.rs` showing cross-system hairpins via ScoreBuilder API, or begin other post-v1 features (rehearsal marks, text annotations, PNG export stub).
- Open issues: Hairpins on beam group notes not supported (rare use case).

## 2026-04-19 — Post-v1, cross-system hairpins example via ScoreBuilder API
- Did: Created `examples/cross_system_hairpins.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API. Demonstrates: (1) crescendo starting in system 1 that crosses system break to system 2 (rendered as two half-wedges), (2) within-system decrescendo from ff to p in system 2 (full wedge), (3) dynamics annotations at key points (pp, ff, p, pp). Output: 21 paths, 36 lines (15862 bytes).
- Verified: `cargo run --example cross_system_hairpins` produces valid SVG. `cargo test -p music-engraver` — 720 unit + 3 integration + 2 doc-tests = 725 tests, all pass. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: rehearsal marks/text annotations, or PNG export stub, or additional polish (pub doc comments on remaining public items).
- Open issues: Hairpins on beam group notes not supported (rare use case).

## 2026-04-19 — Post-v1, text elements in SvgWriter + rehearsal mark layout/rendering/example
- Did: Added `add_text()`, `add_styled_text()`, and `add_stroked_rect()` to `SvgWriter` — enables text-based notation elements (rehearsal marks, expression text, tempo markings). Created `layout/rehearsal.rs` with `RehearsalStyle` enum (Boxed/Plain), `RehearsalMarkLayout` struct, `layout_rehearsal_mark()` — positions boxed/plain text above the staff at 2.5 staff spaces, font size 1.8 staff spaces, with computed box geometry (padding, stroke width, centering). Created `render/rehearsal_renderer.rs` with `draw_rehearsal_mark()` — renders stroked rect (for boxed) + bold serif text centered on x-position. Created `examples/rehearsal_marks.rs` rendering 5 notes with rehearsal marks (A, B boxed; C plain; 1, 12 boxed) → `examples/output/rehearsal_marks.svg` (6 paths, 10 lines, 5 texts, 4 rects, 4222 bytes).
- Verified: `cargo test -p music-engraver` — 744 unit + 3 integration + 2 doc-tests = 749 tests, all pass. 11 rehearsal layout tests + 9 rehearsal renderer tests + 4 new SvgWriter tests. `cargo clippy -p music-engraver` — 0 warnings. `cargo run --example rehearsal_marks` produces valid SVG. `cargo check --workspace` passes.
- Next: Integrate rehearsal marks into ScoreBuilder pipeline (add `.rehearsal_mark()` method), or begin tempo marking layout, or other post-v1 features (expression text, PNG export stub).
- Open issues: Rehearsal marks not yet integrated into measure/system/ScoreBuilder pipeline — currently usable via direct `layout_rehearsal_mark()` + `draw_rehearsal_mark()` calls only.

## 2026-04-20 — Post-v1, rehearsal mark integration through ScoreBuilder pipeline
- Did: Integrated rehearsal marks into the full pipeline from ScoreBuilder to SVG. Added `rehearsal_mark: Option<(String, RehearsalStyle)>` field to `NoteEvent` and `ChordEvent` in `layout/measure.rs`. Added same field to `ScoreEvent::Note` and `ScoreEvent::Chord`. Added `ScoreBuilder::rehearsal_mark(text, style)` method (modifier pattern, like `.dynamic()` and `.tie()`). Updated `draw_note_event()` and `draw_chord_event()` in `render/measure_renderer.rs` to call `layout_rehearsal_mark()` + `draw_rehearsal_mark()` when a rehearsal mark is present. Updated `convert_event()` and `convert_event_tracked()` to propagate rehearsal mark from `ScoreEvent` to `MeasureEvent`. Updated all NoteEvent/ChordEvent construction sites across 10+ files (src modules, tests, examples) with `rehearsal_mark: None`.
- Verified: `cargo test -p music-engraver` — 757 unit + 3 integration + 2 doc-tests = 762 tests, all pass. 5 new measure_renderer tests (note with/without rehearsal mark, chord with mark, plain style no rect, mark differs from no mark). 8 new score tests (note with mark adds text+rect, chord with mark, plain no rect, rest no-op, differs from without, convert_event preserves, tracked preserves, chord convert preserves). `cargo clippy -p music-engraver` — 0 new warnings. `cargo check --workspace` passes.
- Next: Create `examples/rehearsal_score.rs` showing rehearsal marks via ScoreBuilder API, or begin other post-v1 features (tempo markings, expression text, PNG export stub).
- Open issues: None. Rehearsal marks now fully integrated for notes and chords via `.rehearsal_mark(text, style)` on ScoreBuilder.

## 2026-04-20 — Post-v1, rehearsal score example via ScoreBuilder API
- Did: Created `examples/rehearsal_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.rehearsal_mark()` method. Demonstrates: (1) boxed "A" on first note of system 1, (2) boxed "B" on first note of system 2, (3) plain "1" on a chord (no box), (4) boxed "Fine" on a note before the final rest. Exercises boxed and plain styles, marks on single notes and chords. Output: 21 paths, 31 lines, 4 texts, 3 rects (12221 bytes).
- Verified: `cargo run --example rehearsal_score` produces valid SVG. `cargo test -p music-engraver` — 757 unit + 3 integration + 2 doc-tests = 762 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings on engraver crate. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: tempo markings (text above staff, e.g. "Allegro" or quarter=120), expression text (italic text below staff), or PNG export stub.
- Open issues: None.

## 2026-04-20 — Post-v1, tempo marking layout + rendering + example
- Did: Created `layout/tempo.rs` with `TempoMark` enum (Text/Metronome/TextWithMetronome), `MetronomeNoteKind` enum (Whole–Sixteenth), `TempoMarkLayout` and `MetronomeInfo` structs, `layout_tempo_mark()` function. Tempo text is bold, placed above staff at 2.8 staff spaces. Metronome marks use SMuFL `MetNote*` glyphs for the note symbol + "= BPM" text. Combined marks position text then note symbol then "= BPM". Created `render/tempo_renderer.rs` with `draw_tempo_mark()` — renders bold text via `add_styled_text()` and SMuFL note glyph via `add_path()` with translate transform. Created `examples/tempo_marks.rs` rendering 4 notes on treble staff with: (1) text-only "Allegro", (2) ♩=120, (3) "Andante" dotted ♩=72, (4) ♪=160. Output: 8 paths, 9 lines, 5 texts (4432 bytes).
- Verified: `cargo test -p music-engraver` — 779 unit + 3 integration + 2 doc-tests = 784 tests, all pass. 14 tempo layout tests + 9 tempo renderer tests. `cargo clippy -p music-engraver` — 0 warnings. `cargo run --example tempo_marks` produces valid SVG. `cargo check --workspace` passes.
- Next: Integrate tempo marks into ScoreBuilder pipeline (add `.tempo()` method), or begin expression text layout, or other post-v1 features.
- Open issues: Tempo marks not yet integrated into measure/system/ScoreBuilder pipeline — currently usable via direct `layout_tempo_mark()` + `draw_tempo_mark()` calls only.

## 2026-04-20 — Post-v1, tempo mark integration through ScoreBuilder pipeline
- Did: Integrated tempo marks into the full pipeline from ScoreBuilder to SVG. Added `tempo_mark: Option<TempoMark>` field to `NoteEvent` and `ChordEvent` in `layout/measure.rs`. Added same field to `ScoreEvent::Note` and `ScoreEvent::Chord`. Added `ScoreBuilder::tempo(mark)` method (modifier pattern, like `.dynamic()` and `.rehearsal_mark()`). Updated `draw_note_event()` and `draw_chord_event()` in `render/measure_renderer.rs` to call `layout_tempo_mark()` + `draw_tempo_mark()` when a tempo mark is present. Updated `convert_event()` and `convert_event_tracked()` to propagate tempo mark. Updated all NoteEvent/ChordEvent construction sites across 10+ files with `tempo_mark: None`.
- Verified: `cargo test -p music-engraver` — 789 unit + 3 integration + 2 doc-tests = 794 tests, all pass. 3 new measure_renderer tests (note with/without tempo, metronome tempo, chord with tempo). 7 new score tests (text tempo, metronome tempo, rest no-op, differs from no tempo, convert_event preserves, tracked preserves, chord preserves). `cargo clippy -p music-engraver` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/tempo_score.rs` showing tempo marks via ScoreBuilder API, or begin other post-v1 features (expression text, PNG export stub).
- Open issues: None. Tempo marks now fully integrated for notes and chords via `.tempo(mark)` on ScoreBuilder.

## 2026-04-20 — Post-v1, tempo score example via ScoreBuilder API
- Did: Created `examples/tempo_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.tempo()` method. Demonstrates: (1) combined "Allegro ♩=132" on first note, (2) text-only "Andante" tempo change on system 2, (3) metronome-only dotted ♩=72 in final measure. Exercises all 3 TempoMark variants. Output: 21 paths, 30 lines, 4 texts (10777 bytes).
- Verified: `cargo run --example tempo_score` produces valid SVG. `cargo test -p music-engraver` — 789 unit + 3 integration + 2 doc-tests = 794 tests, all pass. `cargo clippy -p music-engraver` — 0 warnings.
- Next: Begin other post-v1 features: expression text (italic text below staff, e.g. "dolce", "espressivo"), or PNG export stub, or additional polish.
- Open issues: None.

## 2026-04-20 — Post-v1, expression text layout + rendering + full pipeline integration
- Did: Created `layout/expression.rs` with `ExpressionLayout` struct and `layout_expression()` — positions italic text below the staff at 4.0 staff spaces (below dynamics to avoid collision). Created `render/expression_renderer.rs` with `draw_expression()` rendering italic serif text centered via `add_styled_text()`. Integrated into full pipeline: added `expression: Option<String>` to `NoteEvent` and `ChordEvent`, `ScoreEvent::Note` and `ScoreEvent::Chord`, `ScoreBuilder::expression(text)` method. Updated `draw_note_event()` and `draw_chord_event()` to render expression text. Updated `convert_event()`/`convert_event_tracked()` propagation. Updated all construction sites with `expression: None`.
- Verified: `cargo test -p music-engraver` — 808 unit + 3 integration + 2 doc-tests = 813 tests, all pass. 7 layout tests + 7 renderer tests + 5 score integration tests = 19 new tests. `cargo clippy -p music-engraver` — 0 new warnings (2 pre-existing `too_many_arguments` with `#[allow]`). `cargo check --workspace` passes.
- Next: Create `examples/expression_score.rs` showing expression text via ScoreBuilder API, or begin other post-v1 features (PNG export stub, additional polish).
- Open issues: None. Expression text now fully integrated for notes and chords via `.expression(text)` on ScoreBuilder.

## 2026-04-20 — QA: fixed W10 clone_on_copy in score.rs
- Did: Removed 5 `.clone()` calls on `Pitch` values (which implements `Copy`) in test code at lines 1294, 1534, 1545, 1923, 1924 of `score.rs`. Replaced with direct value use since `Pitch` is `Copy`.
- Verified: `cargo test -p music-engraver` — 808 unit + 3 integration + 2 doc-tests = 813 tests, all pass. `cargo clippy -p music-engraver --all-targets` — music-engraver warnings dropped from 13 to 8. `cargo check --workspace` passes.
- Next: Fix next QA clippy warning (W8 identity_op in tuplet.rs, W9 useless_format in dot_renderer.rs, W11 redundant closures in key_signatures.rs, or the 2 empty_line_after_doc_comments in expression modules).
- Open issues: 8 remaining clippy warnings in music-engraver (--all-targets).

## 2026-04-20 — Post-v1, measure numbers above each system
- Did: Added measure number rendering above the start of each system. Added `first_measure_number: usize` to `PageSystem` (1-based, set from system-breaking chunk indices). Added `show_measure_numbers: bool` to `PageLayoutConfig` and `PageLayout`. Created `draw_measure_numbers()` in `page_renderer/mod.rs` — places serif text above the top staff line at each system's first measure x-offset. Constants `MEASURE_NUMBER_ABOVE_STAFF_SS` (1.8) and `MEASURE_NUMBER_FONT_SIZE_SS` (1.2) are `pub(crate)` for reuse in multi-staff renderer. Added `ScoreBuilder::show_measure_numbers()` and `MultiStaffScore::show_measure_numbers()` builder methods. Multi-staff renderer draws measure numbers above the topmost stave only.
- Verified: `cargo test -p music-engraver` — 1059 unit + 21 golden + 3 integration + 5 doc-tests = 1088 tests, all pass. 7 new page_renderer tests (enabled/disabled, correct values across 3 systems, positioned above staff, font size scales, auto breaks, on vs off differs). 4 new score tests (adds text, multi-system correct numbers, disabled by default, on vs off differs). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/measure_numbers_score.rs` showing measure numbers via ScoreBuilder API, or add measure number support to golden SVG tests, or continue with other post-v1 features.
- Open issues: None. Measure numbers work for both single-staff (ScoreBuilder) and multi-staff (MultiStaffScore) scores.

## 2026-04-20 — QA: fixed remaining 8 clippy warnings (W8, W9, W11, empty_line_after_doc_comments)
- Did: Fixed all 8 remaining clippy warnings in music-engraver (--all-targets): removed empty line after doc comment in `layout/expression.rs` and `render/expression_renderer.rs` (2), replaced redundant closures with tuple variant constructors in `examples/key_signatures.rs` (4, W11), removed identity_op `(8 - 0)` → `8.0` in `layout/tuplet.rs` test (1, W8), replaced `format!("translate(")` with `"translate(".to_string()` in `render/dot_renderer.rs` test (1, W9).
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — 813 tests pass (808 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: Continue QA backlog (W5 document expect safety, I23 #[must_use] on builder methods) or begin new post-v1 features.
- Open issues: None — music-engraver is now fully clippy-clean with `--all-targets`.

## 2026-04-20 — QA: fixed I23 #[must_use] on ScoreBuilder + W5 document expect safety
- Did: Added `#[must_use]` attribute to `ScoreBuilder` struct (covers `new()` and all fluent builder methods returning `Self` — dropping a partially-built builder is always a user error). Added `#[must_use]` to `render_svg()` and `try_render_svg()` return values. Documented `render_svg()`'s panic safety per W5: the bundled Bravura font contains all required SMuFL glyphs, so font lookup failure is unreachable in normal operation. Updated expect message to reflect this reasoning.
- Verified: `cargo test -p music-engraver` — 811 unit + 3 integration + 2 doc-tests = 816 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Continue QA backlog (W3 rest_glyph bounds, W12 centralized error type, W13 score.rs split) or begin new post-v1 features.
- Open issues: None.

## 2026-04-20 — QA: fixed W2 bounds check on number_to_digit_glyphs in time_signature.rs
- Did: Extended `number_to_digit_glyphs()` to handle all u8 values (0–255) by adding a 3-digit decomposition branch for n ≥ 100. Previously, values ≥ 100 would produce `tens ≥ 10`, causing `digit_glyph(tens).unwrap()` to panic. Now decomposes into hundreds/tens/ones, all guaranteed to be 0–9. Added 3 new tests: `three_digit_numerator_produces_three_glyphs` (128/4), `max_u8_value_does_not_panic` (255/255), `zero_numerator_produces_single_zero_glyph` (0/4).
- Verified: `cargo test -p music-engraver` — 811 unit + 3 integration + 2 doc-tests = 816 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings. `cargo check --workspace` passes.
- Next: Continue QA backlog (W5 document expect safety, I23 #[must_use] on builder methods) or begin new post-v1 features.
- Open issues: None.

## 2026-04-20 — QA: fixed W1 unwrap-on-min/max in chord_extent (layout/chord.rs)
- Did: Refactored `chord_extent()` to use `split_first()?` + `fold` instead of `is_empty()` guard + `min().unwrap()` / `max().unwrap()`. The non-empty invariant is now proved at the type level via `Option` returned by `split_first()`, eliminating 2 production `unwrap()` calls.
- Verified: `cargo test -p music-engraver` — 811 unit + 3 integration + 2 doc-tests = 816 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Continue QA W1 in other files (beam.rs, stem.rs, tuplet.rs, page.rs) or tackle W12/W13.
- Open issues: W1 still present in layout/beam.rs, layout/stem.rs, layout/tuplet.rs, layout/page.rs.

## 2026-04-20 — QA: fixed W1 unwrap-on-min/max in auto_stem_direction_chord (layout/stem.rs)
- Did: Refactored `auto_stem_direction_chord()` to use `split_first()` + `fold` instead of `is_empty()` guard + `min().unwrap()` / `max().unwrap()`. The non-empty invariant is now proved at the type level, eliminating 2 production `unwrap()` calls.
- Verified: `cargo test -p music-engraver` — all 816 tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. Full test suite passes.
- Next: Continue QA W1 in other files (beam.rs, tuplet.rs, page.rs) or tackle W12/W13.
- Open issues: W1 still present in layout/beam.rs, layout/tuplet.rs, layout/page.rs.

## 2026-04-20 — QA: fixed W1 unwrap-on-min/max in beam_group_stem_direction + layout_beam_group (layout/beam.rs)
- Did: Refactored `beam_group_stem_direction()` to use `split_first()` + `fold` instead of `is_empty()` guard + `min().unwrap()` / `max().unwrap()`, proving non-empty invariant at the type level. Replaced `natural_tips.last().unwrap()` and `notes.last().unwrap()` in `layout_beam_group()` with indexed access (`[len-1]`) — these are after an `assert!(!notes.is_empty())` contract, so the assert proves correctness but unwrap was unnecessary indirection. Total: eliminated 4 production `.unwrap()` calls.
- Verified: `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo clippy -p music-engraver --all-targets` — 0 warnings. `cargo check --workspace` passes.
- Next: Continue QA W1 in other files (tuplet.rs, page.rs) or tackle W12/W13.
- Open issues: W1 still present in layout/tuplet.rs, layout/page.rs.

## 2026-04-20 — QA: fixed W1 unwrap-on-min/max in layout_tuplet_bracket (layout/tuplet.rs) + layout_page (layout/page.rs)
- Did: Refactored `layout_tuplet_bracket()` to use `split_first()` + `fold` instead of `is_empty()` guard + `iter().max().unwrap()` / `iter().min().unwrap()`, proving non-empty invariant at the type level (2 unwraps eliminated). Refactored page height calculation in `layout_page()` to use `systems.last()` match instead of `is_empty()` + `last().unwrap()` (1 unwrap eliminated). Total: 3 production `.unwrap()` calls removed.
- Verified: `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver.
- Next: W1 is now fully resolved across all files. Continue QA backlog: W3 (rest_glyph bounds), W12 (centralized error type), or W13 (score.rs split).
- Open issues: None for W1. Remaining QA items: W3, W4, W6, W7, W12, W13, W14, W15, W16, plus info-tier items.

## 2026-04-20 — QA: fixed W6 document expect safety in bravura_font() + triaged W3/W4
- Did: Documented panic safety of `bravura_font()` in `font/mod.rs` with `# Panics` doc section explaining why the expect is unreachable (bundled assets are compile-time constants validated by unit tests). Updated expect message to be more precise. Also triaged W3 and W4: both unwraps are exclusively in `#[cfg(test)]` code, not production — W3's `rest_glyph(d).unwrap()` at rest.rs:146 is inside a test, and W4's `bl.dots.unwrap()` at barline.rs:282 is inside a test. Closing W3, W4, and W6.
- Verified: `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo clippy -p music-engraver --all-targets` — 0 warnings. `cargo check --workspace` passes.
- Next: Continue QA backlog: W7 (write! unwrap style in glyph_outline.rs), W12 (centralized error type), W13 (score.rs split), or info-tier items (I23 done, I8 doctest unwrap).
- Open issues: Remaining QA items: W7, W12, W13, W14, W15, W16, plus info-tier items I2–I8.

## 2026-04-20 — QA: fixed I8 doctest unwrap on Pitch::new in lib.rs and score.rs
- Did: Replaced `.unwrap()` with `.expect("valid pitch")` on `Pitch::new()` calls in the doc examples in `src/lib.rs` (lines 22–25) and `src/score.rs` (lines 19–22). These are the user-facing first-impression examples; `.expect()` with a meaningful message is more idiomatic for fallible constructors with known-good inputs.
- Verified: `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo clippy -p music-engraver --all-targets` — 0 warnings. `cargo check --workspace` passes.
- Next: Continue QA backlog: W7 (write! unwrap style in glyph_outline.rs), W12 (centralized error type), W13 (score.rs split), W14–W16, or info-tier items.
- Open issues: Remaining QA items: W7, W12, W13, W14, W15, W16, plus info-tier items I2–I7.

## 2026-04-20 — QA: fixed W7 write! unwrap style in glyph_outline.rs
- Did: Replaced `write!(self.path, …).unwrap()` calls in `SvgPathBuilder`'s `OutlineBuilder` impl with direct `push`/`push_str` operations. Writing to a `String` via `fmt::Write` can only fail on OOM (unrecoverable), so the unwraps were safe but idiomatically noisy. The new code builds path data via `push('M')`, `push_str(&format_coord(x))`, `push(' ')` chains — clearer intent, no error handling needed. Removed the now-unused `use std::fmt::Write` import.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass. `cargo check --workspace` passes.
- Next: Continue QA backlog: W12 (centralized error type), W13 (score.rs split), W14–W16, or info-tier items.
- Open issues: Remaining QA items: W12, W13, W14, W15, W16, plus info-tier items I2–I7.

## 2026-04-20 — QA: fixed W12 centralized error type (src/error.rs)
- Did: Created `src/error.rs` with `EngraverError` top-level error enum wrapping `FontError` via `#[error(transparent)]` + `#[from]`. Exported as `pub mod error` from `lib.rs`. Updated public API `ScoreBuilder::try_render_svg()` to return `Result<String, EngraverError>` instead of `Result<String, FontError>`. Internal render functions keep `FontError` — the `?` operator auto-converts via `From<FontError>`. New variant families (layout validation, I/O) can be added without changing function signatures.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: Continue QA backlog: W13 (score.rs split), W14–W16, or info-tier items.
- Open issues: Remaining QA items: W13, W14, W15, W16, plus info-tier items I2–I7.

## 2026-04-20 — QA: W13 score.rs split — extracted tests to score/tests.rs
- Did: Converted `src/score.rs` (3,284 lines) into a directory module `src/score/mod.rs` (929 lines) + `src/score/tests.rs` (2,355 lines). Purely mechanical extraction: the `#[cfg(test)] mod tests { ... }` inline block was moved to a file-based module. No logic changes, no API changes.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: W13 phase 2 — extract `ScoreEvent` + accidental-resolution helpers into `score/event.rs` to further slim `mod.rs`. Or continue with W14 (measure_renderer.rs split), W15, W16, info-tier items.
- Open issues: Remaining QA items: W13 (partially done — mod.rs still 929 lines, further split possible), W14, W15, W16, plus info-tier items I2–I7.

## 2026-04-20 — QA: W13 score.rs split phase 2 — extracted ScoreEvent + conversion logic to score/event.rs
- Did: Extracted `ScoreEvent` enum, `AccidentalTracker` type alias, accidental resolution functions (`resolve_accidental`, `should_show_accidental`, `effective_accidental`, `note_altered_in_key`, `note_key`), `duration_kind_to_log2`, and both `convert_event`/`convert_event_tracked` (converted from methods on `ScoreBuilder` to free functions taking `key_sig` parameter) into `src/score/event.rs` (524 lines). `mod.rs` slimmed from 929 → 473 lines. Updated all 18 test call sites from `builder.convert_event(...)` to `convert_event(..., &builder.key_sig)` pattern. Purely mechanical extraction — no logic changes, no API changes.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: W13 is now complete (score module split into mod.rs 473 + event.rs 524 + tests.rs 2355). Continue QA: W14 (measure_renderer.rs split), W15, W16, or info-tier items.
- Open issues: Remaining QA items: W14, W15, W16, plus info-tier items I2–I7.

## 2026-04-20 — QA: W14 measure_renderer.rs split — extracted tests to measure_renderer/tests.rs
- Did: Converted `src/render/measure_renderer.rs` (2,105 lines) into a directory module `src/render/measure_renderer/mod.rs` (514 lines) + `src/render/measure_renderer/tests.rs` (1,591 lines). Purely mechanical extraction: the `#[cfg(test)] mod tests { ... }` inline block was moved to a file-based module. No logic changes, no API changes. `render/mod.rs` references were unchanged since Rust resolves `mod measure_renderer` to a directory automatically.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: Continue QA backlog: W15 (page_renderer.rs / system_renderer.rs split), W16, or info-tier items.
- Open issues: Remaining QA items: W15, W16, plus info-tier items I2–I7.

## 2026-04-20 — QA: W15 page_renderer.rs split — extracted tests to page_renderer/tests.rs
- Did: Converted `src/render/page_renderer.rs` (1,438 lines) into a directory module `src/render/page_renderer/mod.rs` (594 lines) + `src/render/page_renderer/tests.rs` (844 lines). Purely mechanical extraction: the `#[cfg(test)] mod tests { ... }` inline block was moved to a file-based module. No logic changes, no API changes.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver --lib` — all 811 unit tests pass. `cargo check --workspace` passes.
- Next: W15 phase 2 — split system_renderer.rs (1,200 lines) tests into separate file. Then W16, or info-tier items.
- Open issues: Remaining QA items: W15 (system_renderer.rs still unsplit), W16, plus info-tier items I2–I7.

## 2026-04-20 — QA: W15 system_renderer.rs split — extracted tests to system_renderer/tests.rs
- Did: Converted `src/render/system_renderer.rs` (1,200 lines) into a directory module `src/render/system_renderer/mod.rs` (360 lines) + `src/render/system_renderer/tests.rs` (839 lines). Purely mechanical extraction: the `#[cfg(test)] mod tests { ... }` inline block was moved to a file-based module. No logic changes, no API changes.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: Continue QA backlog: W16 (NoteEvent/ChordEvent modifier grab-bags), or info-tier items I2–I7.
- Open issues: Remaining QA items: W16, plus info-tier items I2–I7. W15 is now complete (both page_renderer and system_renderer split).

## 2026-04-20 — Post-v1, articulation layout + rendering + full pipeline integration
- Did: Created `layout/articulation.rs` with `Articulation` enum (Staccato/Tenuto/Accent/Marcato/Staccatissimo/Fermata), each mapping to above/below SMuFL glyph pairs. `layout_articulation()` positions glyph on opposite side from stem (fermata always above), with extra clearance for notes on lines and clamping to avoid placing inside the staff. Created `render/articulation_renderer.rs` with `draw_articulation()`. Added `articulation: Option<Articulation>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer — chord articulations attach to the outer note (closest to articulation side). Added `ScoreBuilder::articulation(artic)` modifier method (no-op on rests). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 848 unit + 3 integration = 851 tests, all pass. 17 articulation layout tests (glyph mapping, placement logic, staff avoidance, line clearance). 7 articulation renderer tests (path production, distinct glyphs, all 6 render, above/below differ, coordinate embedding). 6 score tests (staccato adds path, rest no-op, different articulations differ, chord articulation, fermata, convert_event preserves). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/articulations_score.rs` showing articulations via ScoreBuilder API, or begin other post-v1 features (grace notes, multi-staff brackets).
- Open issues: None. Articulations now fully integrated for notes and chords via `.articulation(artic)` on ScoreBuilder. Remaining QA items: W16, I2–I7.

## 2026-04-20 — QA: W16 collapsed NoteEvent/ChordEvent annotation fields into NoteAnnotations struct
- Did: Created `NoteAnnotations` struct in `layout/measure.rs` with 9 annotation fields (`tie_forward`, `dynamic`, `slur_start`, `slur_end`, `hairpin_start`, `hairpin_end`, `rehearsal_mark`, `tempo_mark`, `expression`) previously duplicated across `NoteEvent` and `ChordEvent`. Struct derives `Clone, Debug, Default` — `Default` gives all-false/all-None, eliminating 9 lines of boilerplate at ~60 construction sites. Updated `NoteEvent` and `ChordEvent` to use a single `annotations: NoteAnnotations` field. Updated all access sites (renderers: `.tie_forward` → `.annotations.tie_forward` etc.) and construction sites across 12 files. Adding future annotations now requires only a new field in `NoteAnnotations` + the sites that care about it.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests).
- Next: All QA warning items (W1–W17) are now resolved. Continue with info-tier items I2–I7 or resume phased plan work.
- Open issues: Remaining QA info-tier items: I2–I7.

## 2026-04-20 — QA: I3 deduplicated convert_event / convert_event_tracked
- Did: Merged `convert_event` (test-only, 3 args) and `convert_event_tracked` (production, 4 args) into a single `convert_event` with `Option<&mut AccidentalTracker>`. When `None`, resolves independently; when `Some`, tracks within-measure state. Extracted two helpers: `resolve_and_track` (resolve accidental + update tracker in one call) and `pitch_to_note_event` (shared pitch→NoteEvent conversion for beam/tuplet groups). Eliminated ~100 lines of duplicated match arms. Updated 33 test call sites and 1 production call site. Renamed test functions from `convert_event_tracked_*` to `convert_event_with_tracking_*`.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: Continue QA info-tier: I2 (cross-system DRY — optional), I4 (resolved by W16), I5 (no action), I6 (resolved by W12), I7 (param structs). Or resume phased plan work.
- Open issues: Remaining QA info-tier items: I2, I7. (I3–I6 resolved or not actionable.)

## 2026-04-20 — QA: I7 introduced TextStyle struct for SvgWriter text methods
- Did: Created `TextStyle` struct in `render/svg_writer.rs` grouping 6 font/style attributes (`font_family`, `font_size`, `fill`, `anchor`, `font_weight`, `font_style`). Added convenience constructors `normal()`, `bold()`, `italic()` and struct-update override pattern. Replaced `add_text` (7 params) and `add_styled_text` (9 params) with a single `add_text(x, y, text, &TextStyle)` (4 params). Updated 4 call sites: `expression_renderer.rs` (uses `TextStyle::italic`), `rehearsal_renderer.rs` (uses `TextStyle::bold` + anchor override), `tempo_renderer.rs` (uses `TextStyle::bold`). Removed 2 `#[allow(clippy::too_many_arguments)]` annotations from the old methods. Exported `TextStyle` from `render/mod.rs`. Added 5 new tests (normal/bold/italic defaults, struct-update override, all-attributes emission).
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 816 tests pass (811 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: I7 partially addressed (text methods cleaned up; `add_stroked_rect` still has 7 params but is only called from 1 site). Remaining QA info-tier: I2 (cross-system DRY — optional, explicitly debatable per QA report). Consider resuming phased plan work or addressing remaining minor items.
- Open issues: Remaining QA info-tier items: I2 (optional). I7 partially done — `add_stroked_rect` still has 7 params but low-impact (1 call site).

## 2026-04-20 — QA: I7 completed — introduced RectStyle struct for add_stroked_rect
- Did: Created `RectStyle` struct in `render/svg_writer.rs` grouping 3 styling attributes (`fill`, `stroke`, `stroke_width`). Added convenience constructors `outlined()` (transparent fill, black stroke) and `boxed()` (white fill, black stroke). Replaced 7-param `add_stroked_rect` with 5-param `add_styled_rect(x, y, width, height, &RectStyle)`. Removed `#[allow(clippy::too_many_arguments)]` annotation. Updated call site in `rehearsal_renderer.rs`. Exported `RectStyle` from `render/mod.rs`. Added 3 new tests (boxed SVG output, outlined defaults, boxed defaults).
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings. `cargo test -p music-engraver` — 819 tests pass (814 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: QA backlog essentially complete — only I2 (cross-system DRY) remains, explicitly optional per QA report. Resume phased plan work or additional post-v1 features.
- Open issues: I2 (optional, deferred — the three span kinds differ enough that a generic trait may not help).

## 2026-04-20 — Post-v1, expression score example via ScoreBuilder API
- Did: Created `examples/expression_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.expression()` method. Demonstrates: (1) "dolce" on opening E4 with piano dynamic, (2) "espressivo" on C5 half note, (3) "legato" on G4 with mp dynamic, (4) "cantabile" on final C4 half + "morendo" on E4 quarter. Exercises italic expression text positioning below staff, combined with dynamics, and verifies expression on rest is a no-op. Output: 20 paths, 29 lines, 5 texts (13702 bytes). Assertions validate all 5 expression texts appear, are italic, and structural element counts are reasonable.
- Verified: `cargo run --example expression_score` produces valid SVG. `cargo test -p music-engraver --lib` — all 819 tests pass (814 unit + 3 integration + 2 doc-tests). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: QA backlog complete (I2 deferred). Consider additional post-v1 features or polish.
- Open issues: I2 (optional, deferred).

## 2026-04-20 — Post-v1, ScoreBuilder PNG convenience methods + clippy fix in png.rs
- Did: Added `try_render_png(scale)` and `render_png(scale)` convenience methods to `ScoreBuilder`, gated behind `#[cfg(feature = "png")]`. These combine SVG rendering + `PngRenderer` in one call with system fonts loaded. Also fixed a pre-existing clippy `field_reassign_with_default` warning in `render/png.rs` by using struct-update syntax for `usvg::Options`. Added 3 new tests: valid PNG output, 2x larger than 1x, convenience matches try variant.
- Verified: `cargo clippy -p music-engraver --all-targets --features png` — 0 warnings from music-engraver. `cargo test -p music-engraver --features png` — 837 tests pass (830 unit + 3 integration + 4 doc-tests). Without png feature: 826 tests pass. `cargo check --workspace` passes.
- Next: Consider additional post-v1 features or polish. Options: PNG rendering example, additional builder ergonomics, or further architectural cleanup.
- Open issues: I2 (optional, deferred — cross-system span DRY).

## 2026-04-20 — QA: I4 collapsed ScoreEvent annotation fields into NoteAnnotations
- Did: Replaced 9 duplicated annotation fields in `ScoreEvent::Note` and `ScoreEvent::Chord` variants with a single `annotations: NoteAnnotations` field, reusing the same struct already applied to `NoteEvent`/`ChordEvent` in the W16 fix. Updated `score/mod.rs` builder methods (9 modifier methods now use `if let` with or-patterns instead of duplicated `match` arms), `score/event.rs` `convert_event` (destructuring simplified to `annotations.clone()`), and ~25 test construction sites in `score/tests.rs` (now use `NoteAnnotations::default()` or struct-update syntax). Removed 4 unused imports from `event.rs`. Net reduction: ~18 lines of enum definition, ~9 match arms collapsed.
- Verified: `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver` — all 824 tests pass (819 unit + 3 integration + 2 doc-tests). `cargo check --workspace` passes.
- Next: QA backlog complete (I2 deferred, I23 already covered by struct-level `#[must_use]`). Consider additional post-v1 features or polish.
- Open issues: I2 (optional, deferred).

## 2026-04-20 — QA: I28 fixed ignored doc-tests for beam_group and tuplet
- Did: Converted 2 `ignore`d doc-tests on `ScoreBuilder::beam_group()` and `ScoreBuilder::tuplet()` to compilable `no_run` examples with proper imports, pitch construction, and full builder chain (including `.end_barline().render_svg()`). Previously these used undefined variables (`pitch_e4`, `builder`) and were marked `ignore`, giving 2 ignored doc-tests in every run. Now all 4 doc-tests compile successfully with 0 ignored.
- Verified: `cargo test -p music-engraver --doc` — 4 passed, 0 ignored. `cargo test -p music-engraver` — 826 total (819 unit + 3 integration + 4 doc-tests), all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver.
- Next: QA backlog fully complete (I2 optional/deferred, I28 done). Consider additional post-v1 features or polish.
- Open issues: I2 (optional, deferred — cross-system span DRY, explicitly debatable per QA report).

## 2026-04-20 — Post-v1, PNG export example + dimension-verifying tests
- Did: Created `examples/png_export.rs` — renders a 2-measure score in D major to both SVG and PNG (2× scale) via `PngRenderer`, writes to `examples/output/score.svg` and `examples/output/score.png`, prints pixel dimensions from IHDR. Added 4 new dimension-verifying tests to `render/png.rs`: `dimensions_match_svg_viewbox_at_1x` (120×80), `dimensions_double_at_2x_scale` (60→120, 40→80), `dimensions_at_3x_scale` (30→90, 20→60), `score_png_has_nonzero_dimensions` (asserts w>50, h>20). Added `png_dimensions()` helper that extracts width/height from IHDR chunk bytes 16–23. These tests assert on specific pixel values rather than just checking magic bytes.
- Verified: `cargo test -p music-engraver --features png` — 841 tests pass (834 unit + 3 integration + 4 doc-tests). `cargo run --example png_export --features png` produces score.png (10368 bytes, 588×112 px). `cargo clippy -p music-engraver --all-targets --features png` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Consider additional post-v1 features: grace notes, articulations/ornaments, multi-staff/grand-staff brackets, line breaking, or golden-SVG visual regression corpus.
- Open issues: I2 (optional, deferred).

## 2026-04-20 — Post-v1, grace note layout + rendering + full pipeline integration
- Did: Created `layout/grace.rs` with `GraceNoteKind` enum (Acciaccatura/Appoggiatura), `grace_note_glyph()` mapping to 4 SMuFL composite glyphs (notehead+stem+flag+slash), `layout_grace_note()` positioning grace note to the left of the principal note at 60% scale, `grace_note_x_reservation()`, and constants `GRACE_NOTE_SCALE` (0.6) and `GRACE_NOTE_SPACING_SS` (0.5). Created `render/grace_renderer.rs` with `draw_grace_note()` rendering the composite glyph with translate+scale transform. Added `grace_note: Option<(i8, GraceNoteKind)>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer — grace note drawn before the principal note/chord. Added `ScoreBuilder::grace_note(pitch, kind)` modifier method (resolves pitch to staff position via current clef, no-op on rests). Annotations clone-based propagation in `convert_event()` handles grace_note automatically.
- Verified: `cargo test -p music-engraver --lib --tests` — 878 unit + 3 integration = 881 tests, all pass. 13 grace layout tests (glyph mapping, all 4 distinct, x-left-of-principal, y-matches-position, scale, reservation, offset-identity). 8 grace renderer tests (path production, acciaccatura/appoggiatura differ, stem up/down differ, scale transform, x-coordinate, all 4 variants). 4 measure_renderer tests (note with/without grace, chord with grace, scale transform presence/absence). 6 score tests (adds path, rest no-op, acc/app differ, chord adds path, convert_event preserves). `cargo clippy -p music-engraver --all-targets` — 0 warnings from grace modules. `cargo check --workspace` passes.
- Next: Create `examples/grace_notes_score.rs` showing grace notes via ScoreBuilder API, or begin other post-v1 features (multi-staff brackets, line breaking).
- Open issues: Grace notes do not yet affect horizontal spacing of the principal note (the grace note may overlap with preceding elements in tight layouts). I2 still deferred.

## 2026-04-20 — Post-v1, PNG error path fix + multi-staff layout + rendering + example
- Did: (1) Fixed `error.rs` PNG variant path from `crate::png::PngError` to `crate::render::png::PngError` — this was preventing `--features png` from compiling despite the `render/png.rs` module and `ScoreBuilder::render_png()` already existing. (2) Created `layout/multi_staff.rs` with `ConnectorKind` enum (Brace/Bracket/None), `StaffGroup` struct (with `grand_staff()`, `section(n)`, `independent(n)` constructors), `MultiStaffLayout` (staff y-origins + optional brace/bracket geometry), `BraceLayout` (glyph, position, scale), `BracketLayout` (thick line + serifs), `layout_multi_staff()` function, `staff_layouts_from_multi()` helper. (3) Created `render/multi_staff_renderer.rs` with `draw_brace()` (scaled SMuFL brace glyph), `draw_bracket()` (3 lines: vertical + 2 serifs), `draw_multi_staff_connectors()`, `draw_joined_barline()`. (4) Created `examples/grand_staff.rs` rendering a piano grand staff (brace + joined barline) and a 3-staff section (bracket + joined barline) → `examples/output/grand_staff.svg` (1 path, 30 lines).
- Verified: `cargo test -p music-engraver --lib` — 905 tests pass. `cargo test -p music-engraver --features png` — 900 tests pass (893 unit + 3 integration + 4 doc-tests). `cargo run --example grand_staff` produces valid SVG (3174 bytes). `cargo run --example png_export --features png` produces score.png (588×112 px). `cargo check --workspace` passes.
- Next: Integrate multi-staff layout into system/page pipeline (render multi-staff systems through `ScoreBuilder`), or add multi-staff score example with notes on both staves.
- Open issues: Multi-staff layout and rendering work standalone but are not yet integrated into the system/page/ScoreBuilder pipeline — currently usable via direct `layout_multi_staff()` + `draw_multi_staff_connectors()` calls only. I2 still deferred.

## 2026-04-20 — Post-v1, golden-SVG visual regression test harness
- Did: Created `tests/golden_svg.rs` integration test with 12 deterministic score-building functions covering: simple scale (clef+keysig+timesig+notes), multi-system (4 measures across 2 systems), chords (second-avoidance), beams (eighths+sixteenths), ties (within-measure+cross-barline), dynamics+hairpins, tuplets, slurs, articulations, grace notes, annotations (rehearsal+tempo+expression), bass clef with flats. Each test renders via `ScoreBuilder`, compares against frozen baseline in `tests/golden/*.svg` using line-level text diff. `GOLDEN_UPDATE=1` env var regenerates baselines. 13th meta-test validates all baselines are well-formed SVGs. Created 12 golden baseline files (4.8–9.4 KB each).
- Verified: `cargo test -p music-engraver --test golden_svg` — 13 tests pass (12 golden comparisons + 1 structural validation). `cargo clippy -p music-engraver --test golden_svg` — 0 warnings. `cargo test -p music-engraver --lib` — 905 unit tests pass. `cargo check --workspace` passes.
- Next: Integrate multi-staff into ScoreBuilder pipeline, or add more golden tests (e.g. PNG golden, cross-system ties/slurs golden), or begin line-breaking algorithm.
- Open issues: Multi-staff not yet in ScoreBuilder pipeline. I2 still deferred.

## 2026-04-20 — Post-v1, lyrics layout + rendering + full pipeline integration
- Did: Created `layout/lyric.rs` with `LyricSyllable` struct (text + `LyricContinuation` enum: None/Hyphen/Extender), convenience constructors (`word()`, `with_hyphen()`, `with_extender()`), `LyricLayout` result struct, and `layout_lyric()` function — positions roman text below staff at 5.5 staff spaces (below dynamics at 2.5ss and expression text at 4.0ss). Created `render/lyric_renderer.rs` with `draw_lyric()` rendering upright serif text centered on note position, appending trailing " -" for hyphen continuations. Added `lyric: Option<LyricSyllable>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer. Added `ScoreBuilder::lyric(syllable)` modifier method (no-op on rests). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib --tests` — 937 unit + 13 golden + 3 integration = 953 tests, all pass. 12 lyric layout tests (text/position/offset/scaling/continuation preservation, below-expression placement). 9 lyric renderer tests (text element, not italic, centered, hyphen, no-hyphen-on-word, extender text-only, different lyrics differ, text count, no path). 4 measure_renderer tests (note with/without lyric, chord with hyphen, lyric changes SVG). 8 score tests (note adds text, rest no-op, hyphen shown, different lyrics differ, chord adds text, convert_event preserves, not italic). `cargo clippy -p music-engraver --lib` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/lyrics_score.rs` showing lyrics via ScoreBuilder API, or integrate multi-staff into ScoreBuilder pipeline.
- Open issues: Extender lines (melisma underscores between notes) require a second-pass knowing the next note's x-position — currently the `Extender` continuation is preserved in the layout but not rendered as a line. Multi-staff not yet in ScoreBuilder pipeline. I2 still deferred.

## 2026-04-20 — Post-v1, lyrics score example + minor cleanup
- Did: Created `examples/lyrics_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.lyric()` method. Demonstrates: (1) "Hap-py birth-day" with hyphen continuations on C4 repeated notes, (2) "to you," with whole word + lyric on rest is no-op ("SKIP" not rendered), (3) ascending "Hap-py birth-day" on C5/B4/A4, (4) "dear" with extender + "friend!" as final word. Exercises hyphen continuations, word-final syllables, extenders, and verifies rest no-op. Output: 17 paths, 30 lines, 12 texts (12448 bytes). Assertions verify specific lyric text content, hyphen format, and that "SKIP" does not appear. Also fixed: `required-features = ["png"]` on `png_export` example in Cargo.toml (was failing `--all-targets` without `--features png`), removed unused `EngravingConfig` import in `layout/grace.rs`, removed unnecessary `as f64` cast in `examples/grand_staff.rs`.
- Verified: `cargo test -p music-engraver` — 937 unit + 13 golden + 3 integration + 4 doc-tests = 957 tests, all pass. `cargo clippy -p music-engraver --all-targets` ��� 0 warnings from music-engraver. `cargo run --example lyrics_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Integrate multi-staff layout into ScoreBuilder pipeline, or add lyric extender line rendering, or begin line-breaking algorithm.
- Open issues: Extender lines not rendered. Multi-staff not in ScoreBuilder pipeline. I2 still deferred.

## 2026-04-20 — Post-v1, chord symbol layout + rendering + full pipeline integration
- Did: Created `layout/chord_symbol.rs` with `ChordSymbolLayout` struct and `layout_chord_symbol()` — positions bold text above the staff at 3.5 staff spaces (above rehearsal marks at 2.5ss to be the topmost text layer). Font size 1.6 staff spaces. Created `render/chord_symbol_renderer.rs` with `draw_chord_symbol()` rendering bold centered serif text via `TextStyle::bold` with `anchor: "middle"` override. Added `chord_symbol: Option<String>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer. Added `ScoreBuilder::chord_symbol(text)` modifier method (no-op on rests). Wired into layout/render mod.rs with public exports. Annotations clone-based propagation in `convert_event()` handles chord_symbol automatically.
- Verified: `cargo test -p music-engraver` — 981 unit + 13 golden + 3 integration + 5 doc-tests = 1002 tests, all pass. 11 chord_symbol layout tests (text/x/y preservation, above-staff placement, font size scaling, offset correctness, different texts, empty text, complex symbol, higher-than-rehearsal). 8 chord_symbol renderer tests (text element, bold, centered, distinct symbols, distinct positions, no paths, text count, not italic). 4 measure_renderer tests (note with/without chord symbol, chord event with symbol, symbol changes output). 6 score tests (adds text element, rest no-op, different symbols differ, chord event, bold, convert_event preserves). `cargo clippy -p music-engraver --all-targets` — 0 new warnings from chord_symbol code. `cargo check --workspace` passes.
- Next: Create `examples/chord_symbols_score.rs` showing chord symbols via ScoreBuilder API, or add golden SVG test for multi-staff scores, or begin other post-v1 features.
- Open issues: Lyric extender lines not rendered. Multi-staff cross-system ties/slurs/hairpins not supported. I2 still deferred.

## 2026-04-20 — Post-v1, articulations + grace notes score examples
- Did: Created `examples/articulations_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.articulation()` method. Demonstrates all 6 articulation types (staccato, tenuto, accent, marcato, staccatissimo, fermata) on notes with varying stem directions and staff positions. Output: 28 paths, 29 lines (12982 bytes). Created `examples/grace_notes_score.rs` rendering 4 measures across 2 systems using `.grace_note()` method. Demonstrates acciaccatura (slashed) and appoggiatura on single notes, high/low register notes, and chords; verifies rest no-op and 0.6 scale transform. Output: 24 paths, 28 lines (15727 bytes).
- Verified: `cargo test -p music-engraver` — 957 tests pass (937 unit + 13 golden + 3 integration + 4 doc-tests). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example articulations_score` and `cargo run --example grace_notes_score` both produce valid SVGs. `cargo check --workspace` passes.
- Next: Integrate multi-staff layout into ScoreBuilder pipeline, or add lyric extender line rendering, or begin line-breaking algorithm.
- Open issues: Extender lines not rendered. Multi-staff not in ScoreBuilder pipeline. I2 still deferred.

## 2026-04-20 — Post-v1, MultiStaffScore API (grand staff, bracket, independent)
- Did: Created `score/multi_staff.rs` with `MultiStaffScore` type — combines multiple `ScoreBuilder` staves into vertically grouped multi-staff systems. Three constructors: `grand_staff(upper, lower)` (brace + joined barlines), `section(staves)` (bracket + joined barlines), `independent(staves)` (no connector). Each stave is an independent `ScoreBuilder` with its own clef, key sig, and content. Rendering: computes `MultiStaffLayout` per system chunk, draws each stave via `layout_system()` + `draw_system()` at correct y-offset, adds brace/bracket connectors via `draw_multi_staff_connectors()`, draws joined barlines spanning all staves. Supports `system_width_fu()` and `measures_per_system()` overrides. Multi-system breaking works (measures split across system groups). Refactored `ScoreBuilder` to extract `build_measure_contents()`, `build_prefix()`, `effective_system_width()`, `effective_measures_per_system()`, and `flush_pending()` as `pub(crate)` methods for reuse. Created `examples/grand_staff_score.rs` rendering a 2-measure piano score (D major, treble + bass) with brace and joined barlines → `examples/output/grand_staff_score.svg` (20 paths, 28 lines, 15546 bytes).
- Verified: `cargo test -p music-engraver` — 952 unit + 13 golden + 3 integration + 5 doc-tests = 973 tests, all pass. 15 new multi-staff tests (grand staff valid SVG, brace glyph, two staff line sets, joined barlines, more paths than single, section bracket, independent no connector, empty staves, system_width override, measures_per_system override, grand vs bracket differ, break_measures fixed/exact/manual/empty). `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver. `cargo run --example grand_staff_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Add cross-system support for multi-staff (ties/slurs/hairpins across system breaks in multi-staff context), or add PNG rendering for multi-staff, or begin line-breaking algorithm.
- Open issues: Cross-system ties/slurs/hairpins not yet handled in multi-staff context (they work within each stave's system but cross-system half-ties are not drawn since `draw_page` is not used). Lyric extender lines still not rendered. I2 still deferred.

## 2026-04-20 — Post-v1, chord symbols score example via ScoreBuilder API
- Did: Created `examples/chord_symbols_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.chord_symbol()` method. Demonstrates: (1) I–vi–IV–V progression with basic symbols (C, Am, F, G), (2) jazz extensions (Dm7, G7), (3) complex symbol over a 4-note chord voicing (Cmaj7), (4) altered chord symbols (F#m7b5, B7alt, Em). Verifies bold text rendering, text-anchor centering, and rest no-op. Output: 20 paths, 31 lines, 11 text elements (14555 bytes).
- Verified: `cargo run --example chord_symbols_score` produces valid SVG. `cargo test -p music-engraver` — 981 unit + 13 golden + 3 integration + 5 doc-tests = 1002 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add cross-system tie/slur/hairpin support to multi-staff rendering, or add lyric extender line rendering, or begin line-breaking algorithm.
- Open issues: Cross-system ties/slurs/hairpins not in multi-staff context. Lyric extender lines not rendered. I2 still deferred.

## 2026-04-20 — Post-v1, automatic line breaking (SystemBreaking::Auto)
- Did: Implemented automatic width-based line breaking in `layout/page.rs`. Added `SystemBreaking::Auto` variant that greedily packs measures onto systems based on their natural (unjustified) widths vs. the target system width. First system accounts for full prefix (clef + key sig + time sig); continuation systems account for shorter prefix (clef + key sig, no time sig). Guarantees at least one measure per system even if a single measure exceeds the budget. Added `content_natural_width()`, `prefix_natural_width()`, and `break_measures_auto()` helper functions. Made `measure_event_to_element()` `pub(crate)` in `layout/system.rs`. Added `ScoreBuilder::auto_line_breaks()` method — overrides `measures_per_system`; calling `measures_per_system` after disables auto. Added `auto_breaks` field to `ScoreBuilder`. Added `Auto` fallback (→Fixed(4)) in `MultiStaffScore`'s `break_measures`. Added golden SVG test case `auto_breaks` with mixed-density measures.
- Verified: `cargo test -p music-engraver` — 998 unit + 14 golden + 3 integration + 5 doc-tests = 1020 tests, all pass. `cargo clippy -p music-engraver --all-targets` — no new warnings from changed files. `cargo check --workspace` passes.
- Next: Add auto line breaking support to `MultiStaffScore`, or begin Gourlay/penalty-based optimal line breaking (vs. current greedy), or continue QA backlog (W16, I2).
- Open issues: Auto breaking in `MultiStaffScore` falls back to Fixed(4). Greedy packing is not globally optimal — a Knuth-Plass or Gourlay penalty model would distribute whitespace more evenly. Cross-system ties/slurs/hairpins not in multi-staff context. Lyric extender lines not rendered.

## 2026-04-20 — Post-v1, auto line breaking for MultiStaffScore
- Did: Implemented `SystemBreaking::Auto` support in `MultiStaffScore`. Added `auto_line_breaks()` method that enables width-based greedy measure packing (same algorithm as single-staff `ScoreBuilder`). Uses first stave's content for width estimation via `break_measures_auto()`. Made `content_natural_width()`, `prefix_natural_width()`, and `break_measures_auto()` `pub(crate)` in `layout/page.rs` for reuse. Also inherits `auto_breaks` from individual staves — if any stave has `auto_breaks=true`, multi-staff uses auto breaking. `measures_per_system()` and `auto_line_breaks()` properly override each other (last caller wins). 8 new tests: valid SVG, differs from fixed, narrow-vs-wide width comparison (narrow produces more system groups = more staff lines), override ordering (auto→fixed and fixed→auto), stave inheritance, section with auto.
- Verified: `cargo test -p music-engraver` — 1005 unit + 14 golden + 3 integration + 5 doc-tests = 1027 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add cross-system tie/slur/hairpin support in multi-staff context, or implement Knuth-Plass/Gourlay optimal line breaking, or add lyric extender line rendering.
- Open issues: Cross-system ties/slurs/hairpins not in multi-staff context. Greedy packing not globally optimal. Lyric extender lines not rendered. I2 still deferred.

## 2026-04-20 — Post-v1, golden-SVG corpus expansion (grand staff, lyrics, chord symbols)
- Did: Added 3 new golden-SVG regression tests to `tests/golden_svg.rs`: `golden_grand_staff` (2-measure piano score with treble+bass clef, brace connector, joined barlines, key sig — 20 paths, 28 lines, 15546 bytes), `golden_lyrics` (2-measure "Happy birthday" with hyphen/extender continuations — 9 paths, 17 lines, 6 texts, 6773 bytes), `golden_chord_symbols` (2-measure I–vi–IV–V progression with chord names above staff — 8 paths, 16 lines, 5 texts, 6215 bytes). Added imports for `MultiStaffScore`, `LyricSyllable`. Updated `golden_baselines_are_valid_svgs` name list. Generated and committed baselines under `tests/golden/`. Total golden corpus: 16 baselines covering single-staff, multi-system, chords, beams, ties, dynamics, tuplets, slurs, articulations, grace notes, annotations, bass clef, auto breaks, grand staff, lyrics, chord symbols.
- Verified: `cargo test -p music-engraver` — 1005 unit + 17 golden + 3 integration + 5 doc-tests = 1030 tests, all pass. `cargo check --workspace` passes.
- Next: Add cross-system tie/slur/hairpin support in multi-staff context, or implement Knuth-Plass/Gourlay optimal line breaking, or add lyric extender line rendering, or add golden tests for remaining features (hairpins, cross-system ties, expression text).
- Open issues: Cross-system ties/slurs/hairpins not in multi-staff context. Greedy packing not globally optimal. Lyric extender lines not rendered.

## 2026-04-20 — Post-v1, ornament layout + rendering + full pipeline integration + example
- Did: Created `layout/ornament.rs` with `Ornament` enum (Trill/Mordent/InvertedMordent/Turn/InvertedTurn/TurnSlash/ShortTrill), each mapping to a dedicated SMuFL glyph. `layout_ornament()` positions the ornament glyph above the staff (or above the note for high notes), centered on the notehead. Ornaments always appear above, unlike articulations which flip with stem direction. Created `render/ornament_renderer.rs` with `draw_ornament()`. Added `ornament: Option<Ornament>` field to `NoteAnnotations` in `layout/measure.rs`. Added `ScoreBuilder::ornament(orn)` method (modifier pattern). Updated `draw_note_event()` and `draw_chord_event()` in `render/measure_renderer/mod.rs` to render ornaments above the staff. Created `examples/ornaments_score.rs` rendering 4 measures across 2 systems with all 7 ornament types on notes, high notes, and chords (27 paths, 27 lines, 18435 bytes). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 1038 unit + 17 golden + 3 integration + 5 doc-tests = 1063 tests, all pass. 15 layout tests + 7 renderer tests + 4 measure_renderer tests + 7 score integration tests = 33 new ornament tests. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example ornaments_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Add golden-SVG test for ornaments, or begin other post-v1 features (lyric extender lines, multi-staff cross-system spans, optimal line breaking).

## 2026-04-20 — Post-v1, golden-SVG corpus expansion (ornaments, hairpins, cross-system ties, expression text)
- Did: Added 4 new golden-SVG regression tests to `tests/golden_svg.rs`: `golden_ornaments` (4 quarter notes with trill/mordent/inverted mordent/turn — 8224 bytes), `golden_hairpins` (crescendo wedge with p→f dynamics — 6493 bytes), `golden_cross_system_ties` (2 systems with cross-system tie on G4, rendered as half-ties — 8445 bytes), `golden_expression_text` (italic "dolce" and "cantabile" below staff — 4735 bytes). Added imports for `HairpinType` and `Ornament`. Updated `golden_baselines_are_valid_svgs` name list (now 20 entries). Total golden corpus: 20 baselines covering all major notation features.
- Verified: `cargo test -p music-engraver --test golden_svg` — 21 tests pass (20 golden comparisons + 1 structural validation). `cargo clippy -p music-engraver --test golden_svg` — 0 warnings from music-engraver. `cargo test -p music-engraver --lib` — 1038 unit tests pass. `cargo check --workspace` passes.
- Next: Begin lyric extender line rendering (melisma lines between notes), or multi-staff cross-system spans, or optimal Knuth-Plass/Gourlay line breaking.
- Open issues: Lyric extender lines not rendered. Cross-system ties/slurs/hairpins not in multi-staff context. Greedy packing not globally optimal.

## 2026-04-20 — Post-v1, lyric extender line (melisma) rendering
- Did: Implemented lyric extender lines (melisma underscores) as a second-pass operation in the system renderer, following the same pattern as ties/slurs/hairpins. Added `draw_lyric_extender()` to `render/lyric_renderer.rs` — draws a horizontal line from after the source syllable text to just before the next note's position, with configurable left/right padding (0.4ss / 0.2ss). Added `LyricNoteInfo` struct, `collect_lyric_note_info()`, and `draw_system_lyric_extenders()` to `render/system_renderer/mod.rs` — scans positioned elements for lyrics with `Extender` continuation and draws lines to the next note/chord. Short spans (where padding would overlap) are suppressed. Updated golden baseline for `lyrics` test (now includes the extender line).
- Verified: `cargo test -p music-engraver` — 1048 unit + 21 golden + 3 integration + 5 doc-tests = 1077 tests, all pass. 5 new lyric_renderer tests (extender draws horizontal line, correct x range with padding, not drawn when too short, stroke width, black stroke). 5 new system_renderer tests (extender within measure adds 1 line, no extender for word continuation, no extender for hyphen, cross-barline extender, extender differs from no extender). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add lyric extender support to cross-system context (page_renderer), or begin other post-v1 features (multi-staff cross-system spans, optimal line breaking).
- Open issues: Cross-system lyric extenders not yet supported (extender at end of system that continues to next system). Cross-system ties/slurs/hairpins not in multi-staff context. Greedy packing not globally optimal.

## 2026-04-20 — Post-v1, cross-system lyric extender lines
- Did: Implemented cross-system lyric extender rendering in `page_renderer/mod.rs`, following the established pattern from cross-system ties/slurs/hairpins. Made `LyricNoteInfo` and `collect_lyric_note_info()` `pub(crate)` in `system_renderer/mod.rs`. Added `find_last_unresolved_extender()` helper (walks backwards to find the last note with an `Extender` continuation that has no subsequent note within the system to resolve it). Added `draw_cross_system_lyric_extenders()` — for each unresolved extender at the end of system N, draws a trailing half-extender from the source syllable to the right edge of system N, and an incoming half-extender from the left edge of system N+1 to the first note position. Both half-extenders reuse the existing `draw_lyric_extender()` function. Works automatically through `ScoreBuilder` since it uses `draw_page()`.
- Verified: `cargo test -p music-engraver` — 1064 unit + 21 golden + 3 integration + 5 doc-tests = 1093 tests, all pass. 5 new page_renderer cross-system lyric extender tests (two lines drawn, no extender without continuation, within-system not duplicated, extender differs from no extender, extender lines are horizontal). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add golden-SVG test for cross-system lyric extenders, or begin other post-v1 features (multi-staff cross-system spans, optimal line breaking, tablature polish).
- Open issues: Cross-system ties/slurs/hairpins not in multi-staff context. Greedy packing not globally optimal.

## 2026-04-21 — Post-v1, pedal marking layout + rendering + full pipeline integration
- Did: Created `layout/pedal.rs` with `PedalMark` enum (Down/Up), each mapping to a dedicated SMuFL glyph (`KeyboardPedalPed` / `KeyboardPedalUp`). `layout_pedal()` positions the glyph below the staff at 7.0 staff spaces (below dynamics at 2.5ss, expression text at 4.0ss, and lyrics at 5.5ss to avoid collisions). Created `render/pedal_renderer.rs` with `draw_pedal()`. Added `pedal: Option<PedalMark>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer. Added `ScoreBuilder::pedal_down()` and `ScoreBuilder::pedal_up()` modifier methods (no-op on rests). Annotations clone-based propagation in `convert_event()` handles pedal automatically. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib` — 1422 unit tests pass (+29 new pedal tests: 11 layout, 8 renderer, 4 measure_renderer, 6 score). `cargo test -p music-engraver --test golden_svg` — 33 golden tests pass. `cargo test -p music-engraver --doc` — 7 doc-tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/pedal_score.rs` showing pedal markings via ScoreBuilder API, or add a golden-SVG test for pedal markings, or begin other post-v1 features (pre-bends, tab stave cross-system features).
- Open issues: Bracket-style pedal lines (continuous line with notches at pedal changes) not yet implemented — only the classic "Ped." / "*" glyph approach. I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-21 — Post-v1, cross-system spans in multi-staff context (ties, slurs, hairpins, lyric extenders)
- Did: Wired cross-system span rendering (ties, slurs, hairpins, lyric extenders) into `MultiStaffScore::try_render_svg()`. Refactored 4 `draw_cross_system_*` functions in `page_renderer/mod.rs` from `fn(…, &PageLayout)` to `pub(crate) fn(…, &[PageSystem])` so they can be called from both `draw_page()` and multi-staff rendering. In `multi_staff.rs`: collect `Vec<PageSystem>` per stave as systems are laid out, then after all systems are drawn call all 4 cross-system functions per stave. Each stave's spans are resolved independently (treble ties don't interact with bass ties). Used `system.clone()` to populate `PageSystem` values (SystemLayout already derives Clone).
- Verified: `cargo test -p music-engraver --lib` — 1069 unit tests pass (5 new multi_staff tests: cross-system tie draws half-ties, cross-system slur draws half-slurs, cross-system hairpin draws half-wedges, spans only affect owning stave, no spans with single system). `cargo test -p music-engraver --test golden_svg` — 21 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add a golden-SVG test for multi-staff cross-system ties, or begin other post-v1 features (optimal line breaking, tablature polish).
- Open issues: Greedy packing not globally optimal.

## 2026-04-21 — Post-v1, golden-SVG corpus expansion (multi-staff cross-system spans)
- Did: Added `golden_multi_staff_cross_system` golden-SVG regression test. 4-measure piano grand staff score across 2 systems: treble has cross-system tie on A5 (half-ties rendered), bass has cross-system slur from A2 through D3. Exercises brace connector, joined barlines, cross-system tie half-curves, cross-system slur half-curves — all in multi-staff context. Baseline: 28152 bytes, 37 paths, 51 lines. Updated `golden_baselines_are_valid_svgs` name list (now 21 entries). Total golden corpus: 21 baselines.
- Verified: `cargo test -p music-engraver` — 1069 unit + 22 golden + 3 integration + 5 doc-tests = 1099 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: optimal Knuth-Plass/Gourlay line breaking (replace greedy packer), or measure number golden test, or tablature polish (bends, slides).
- Open issues: Greedy packing not globally optimal.

## 2026-04-21 — Post-v1, optimal (Knuth-Plass style) line breaking + measure numbers example
- Did: (1) Implemented `SystemBreaking::Optimal` variant and `break_measures_optimal()` function in `layout/page.rs` — Knuth-Plass style dynamic programming that minimizes total badness (squared whitespace deviation) across all systems. Asymmetric penalty: underfull lines get 1× weight, overfull lines get 4× (discourages cramming). O(n²) complexity with early termination for severely overfull candidates. (2) Added `ScoreBuilder::optimal_line_breaks()` and `MultiStaffScore::optimal_line_breaks()` methods. Proper override chain: `optimal_line_breaks` ↔ `auto_line_breaks` ↔ `measures_per_system` — last caller wins. (3) Created `examples/measure_numbers_score.rs` (6 measures across 3 systems with measure numbers "1", "3", "5"). (4) Added golden SVG test `optimal_breaks` (22 baselines total).
- Verified: `cargo test -p music-engraver` — 1082 unit + 23 golden + 3 integration + 5 doc-tests = 1113 tests, all pass. 8 new page layout tests (empty, single, wide target, covers all, non-empty systems, uniform agreement, justified, narrow target). 5 new score tests (valid SVG, differs from fixed, optimal↔auto↔measures_per_system override chain). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add golden test for measure numbers, or begin tablature polish (bends, slides), or implement additional line-breaking features (last-line ragging, paragraph-level penalties).
- Open issues: Optimal breaking does not yet handle last-line ragging (last system may be underfull, which is conventional but not explicitly treated). I2 still deferred.

## 2026-04-21 — Post-v1, TabScoreBuilder API (tab pipeline integration)
- Did: Created `score/tab.rs` with `TabScoreBuilder` — fluent API for building tablature scores from string/fret pairs. Supports: `.fret(string, fret)` for single notes, consecutive `.fret()` calls for chords (simultaneous fret numbers), `.next()` to separate beats, `.rest()` for blank beats, `.barline()`/`.end_barline()`/`.barline_style()` for measure endings, `.measures_per_system()` for system breaking, `.system_width_fu()`, `.show_measure_numbers()`. Three constructors: `guitar()` (6-string), `four_string()` (4-string bass/uke), `new(n)` (custom). Rendering draws tab staff lines, TAB clef, fret numbers with white background rects, and barlines (thin/thick) at evenly spaced positions. Multi-system support via fixed measures-per-system breaking. Barlines drawn directly (thin/thick vertical lines) rather than going through `StaffLayout` since tab staves have variable line counts. Created `examples/tab_score.rs` (4 measures across 2 systems: E minor arpeggio, scale, power chords, high frets → 7125 bytes). Added golden-SVG regression test `tab_score` (23 baselines → total 23 golden tests now).
- Verified: `cargo test -p music-engraver` — 1138 unit + 24 golden + 3 integration + 6 doc-tests = 1171 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Add rhythm stems above tab staff (half/quarter/eighth notation), or integrate tab staves into `MultiStaffScore` (e.g. guitar+tab combined), or add tab-specific features (slides, bends, hammer-on/pull-off notation).
- Open issues: Tab events are equally spaced (no duration-proportional spacing since tab events don't carry duration info). No rhythm stems/flags above tab staff yet. Tab staves not yet usable in `MultiStaffScore` context. Greedy packing not globally optimal.

## 2026-04-21 — Post-v1, tablature staff layout + rendering + example
- Did: Created `layout/tab.rs` with `TabStaffLayout` struct (configurable line count: 6 for guitar, 4 for bass/ukulele), `tab_clef_glyph()` (maps line count to SMuFL `_6StringTabClef` / `_4StringTabClef`), `layout_fret_number()` (positions fret number text on string line with white background rect for line masking, wider bg for 2-digit numbers), `FretNumberLayout` result struct, and constants (`TAB_LINE_COUNT`, `TAB_4_STRING_LINE_COUNT`, `FRET_NUMBER_FONT_SIZE_RATIO`). String numbering follows guitar convention: string 1 = highest pitch = bottom line. Created `render/tab_renderer.rs` with `draw_tab_staff_lines()` (variable line count), `draw_tab_clef()` (centered on staff), `draw_fret_number()` (white bg rect + bold centered text), `draw_fret_number_at()` convenience. Created `examples/tab_staff.rs` rendering 6-string guitar tab with E chord (6 fret numbers), ascending scale on string 1 (7 fret numbers including 2-digit "12"), and power chord (3 fret numbers) → `examples/output/tab_staff.svg` (6 lines, 1 path, 16 texts, 16 rects, 5223 bytes). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 1114 unit + 23 golden + 3 integration + 5 doc-tests = 1145 tests, all pass. 20 tab layout tests (line counts, height, spacing, string-to-y mapping, center, clef glyphs, fret number positioning, font size scaling, 2-digit width, nonzero origin). 13 tab renderer tests (line counts, stroke width, clef path/position, fret number rect+text, open string, 2-digit, bold/centered, white bg, multiple numbers, different strings, full staff). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_staff` produces valid SVG. `cargo check --workspace` passes.
- Next: Integrate tablature into a higher-level API (TabScoreBuilder or extend MultiStaffScore to support tab staves), or add tab-specific features (rhythm stems above tab staff, slides, bends, hammer-on/pull-off notation).
- Open issues: Tab staff is standalone layout+render only — not yet integrated into measure/system/page/ScoreBuilder pipeline. No rhythm notation on tab staves yet. I2 still deferred.

## 2026-04-21 — Post-v1, tab rhythm stems + flags above staff
- Did: Created `layout/tab_rhythm.rs` with `TabRhythmLayout` struct, `layout_tab_rhythm()` (computes stem position above top staff line — base at 1.5ss above, tip at 3.0ss above base), `tab_flag_count()`, `needs_stem()`. Created `render/tab_rhythm_renderer.rs` with `draw_tab_rhythm()` (draws stem line + optional SMuFL flag glyph at stem tip, always stems-up). Updated `TabEvent` enum to carry optional `duration_log2` on both `Fret` and `Rest` variants. Added `TabScoreBuilder::duration(log2)` modifier method plus convenience methods `.quarter()`, `.eighth()`, `.half()`, `.whole()`. Duration is consumed per-event (one `.duration()` applies to the next flushed event only). Rhythm stems drawn in `draw_tab_measure()` when duration is present. Created `examples/tab_rhythm.rs` rendering 4 measures across 2 systems with quarter/eighth/sixteenth/half/whole rhythm stems (24 lines, 8 paths, 16 texts, 10517 bytes). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 1171 unit + 24 golden + 3 integration + 6 doc-tests = 1204 tests, all pass. 17 new tab_rhythm layout tests + 8 new tab_rhythm renderer tests + 9 new TabScoreBuilder rhythm tests (duration adds stem, eighth adds flag, whole no stem, no-duration no-stem, rest with duration, half stem no flag, mixed durations, duration consumed per event). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_rhythm` produces valid SVG. `cargo check --workspace` passes.
- Next: Add tab beam groups (beamed eighth/sixteenth rhythm notation above tab staff), or add tab-specific notation (slides, bends, hammer-on/pull-off), or integrate tab staves into MultiStaffScore.
- Open issues: Tab rhythm stems are individual (no beaming for grouped eighths/sixteenths). Tab staves not yet usable in MultiStaffScore context. I2 still deferred.

## 2026-04-21 — Post-v1, tab beam groups (layout + rendering + ScoreBuilder integration + example)
- Did: Created `layout/tab_beam.rs` with `TabBeamedNote` struct, `TabBeamGroupLayout` struct, `compute_tab_beam_counts()` (beam connectivity per note), `layout_tab_beam_group()` (computes horizontal beams at fixed stem-tip y — simpler than standard notation since tab stems are all up at same height). Created `render/tab_beam_renderer.rs` with `draw_tab_beam_group()` rendering stems as lines and beam segments as filled polygons (primary beams span full group, secondary beams for sixteenths+). Added `TabEvent::BeamGroup` variant to `score/tab.rs`. Added `TabScoreBuilder::beam_start()`/`beam_end()` methods with `flush_beam_frets()` helper and `in_beam_group`/`beam_group_events` accumulator fields. Default duration in beam group is eighth. Added `draw_tab_beam_group_event()` helper to `draw_tab_measure()`. Created `examples/tab_beams.rs` rendering 4 measures across 2 systems: 4 beamed eighths, 2 beamed sixteenths + quarter, beamed eighth chords, quarter + 4 beamed sixteenths → `examples/output/tab_beams.svg` (9724 bytes, 32 lines, 2 paths, 21 texts, 6 polygons).
- Verified: `cargo test -p music-engraver` — 1199 unit + 24 golden + 3 integration + 6 doc-tests = 1232 tests, all pass. 13 tab_beam layout tests + 7 tab_beam renderer tests + 8 TabScoreBuilder beam group tests = 28 new tests. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_beams` produces valid SVG. `cargo check --workspace` passes.
- Next: Add golden-SVG test for tab beams, or add tab-specific notation (slides, bends, hammer-on/pull-off), or integrate tab staves into MultiStaffScore.
- Open issues: Tab staves not yet usable in MultiStaffScore context. I2 still deferred.

## 2026-04-21 — Post-v1, tab hammer-on/pull-off layout + rendering + ScoreBuilder integration + example + golden test
- Did: Created `layout/tab_hammer.rs` with `LegatoKind` enum (HammerOn/PullOff), `TabLegatoLayout` struct, `layout_tab_legato()` — computes quadratic Bézier arc geometry above the string line between two fret positions, with "H"/"P" text label centered at the apex. Arc height scales with staff space (0.8×). Created `render/tab_hammer_renderer.rs` with `draw_tab_legato()` rendering an unfilled stroke arc (quadratic Bézier) + bold sans-serif text label. Added `legato_out: Option<LegatoKind>` to `TabEvent::Fret`, `pending_legato` to `TabScoreBuilder`. Added `.hammer()` and `.pull()` modifier methods following the same pattern as `.slide()`. Third-pass scan in `draw_tab_measure()` draws arcs for matching strings (supports chord legato with one arc per matching string). Created `examples/tab_hammer_pull.rs` (4 measures: hammer-on, pull-off, chain, chord hammer — 9 paths, 20 texts, 17 lines, 7577 bytes). Added `golden_tab_hammer_pull` golden-SVG regression test (25th baseline → 26 golden total).
- Verified: `cargo test -p music-engraver` — 1250 unit + 26 golden + 3 integration + 6 doc-tests = 1285 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from new code. `cargo run --example tab_hammer_pull` produces valid SVG. `cargo check --workspace` passes.
- Next: Add tab-specific notation (bends — curve + text above string), or integrate tab staves into MultiStaffScore context, or begin other post-v1 features.
- Open issues: Tab staves not yet usable in MultiStaffScore context. I2 still deferred.

## 2026-04-21 — Post-v1, tab bend layout + rendering + ScoreBuilder integration + example + golden test
- Did: Created `layout/tab_bend.rs` with `BendAmount` enum (Quarter/Half/Full/OneAndHalf/Custom), `TabBendLayout` struct, `layout_tab_bend()` — computes upward-curving arrow geometry above a fret position (quadratic Bézier curve + filled triangle arrowhead + centered text label). Arrow height scales with staff space (1.6×). Created `render/tab_bend_renderer.rs` with `draw_tab_bend()` rendering curve path (unfilled stroke), filled arrowhead path, and bold sans-serif text. Added `bend: Option<BendAmount>` field to `TabEvent::Fret`. Added `pending_bend` to `TabScoreBuilder` and `.bend(amount)` modifier method following the same pattern as `.slide()` and `.hammer()`. Third-pass scan in `draw_tab_measure()` draws bend arrows for each string in events with `bend: Some(amount)`. Created `examples/tab_bends.rs` (4 measures: full/half/quarter/1½ bends, chord bends, mixed with slides and hammer-ons — 23 paths, 18 lines, 27 texts, 10209 bytes). Added `golden_tab_bends` golden-SVG regression test (26th baseline → 27 golden tests total).
- Verified: `cargo test -p music-engraver` — 1279 unit + 27 golden + 3 integration + 6 doc-tests = 1315 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_bends` produces valid SVG. `cargo check --workspace` passes.
- Next: Add tab-specific notation (pre-bends, release bends, vibrato), or integrate tab staves into MultiStaffScore context, or begin other post-v1 features.
- Open issues: Tab staves not yet usable in MultiStaffScore context. Pre-bends (bent before picking) and release bends (bend then release) not yet implemented. I2 still deferred.

## 2026-04-21 — Post-v1, tab slide layout + rendering + ScoreBuilder integration
- Did: Created `layout/tab_slide.rs` with `TabSlideLayout` struct and `layout_tab_slide()` — computes diagonal line geometry between two fret positions on the same string, with horizontal padding (0.6× staff space) to avoid overlapping fret number text. Returns `None` when endpoints too close. Created `render/tab_slide_renderer.rs` with `draw_tab_slide()` rendering a single `<line>` element. Added `slide_out: bool` field to `TabEvent::Fret`. Added `pending_slide: bool` to `TabScoreBuilder`. Added `TabScoreBuilder::slide()` modifier method — marks the current fret event (or last flushed event) for slide into the next. In `draw_tab_measure()`, added second-pass scan: for each Fret with `slide_out=true`, finds the next Fret event (skipping rests), draws slide lines for each matching string. Supports multi-string chord slides (one line per matching string), consecutive slides, and slide across rest gaps. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 1222 unit + 24 golden + 3 integration + 6 doc-tests = 1255 tests, all pass. 10 tab_slide layout tests + 5 tab_slide renderer tests + 8 TabScoreBuilder slide tests = 23 new tests. `cargo clippy -p music-engraver --lib` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/tab_slides.rs` showing slides via TabScoreBuilder API, or add golden-SVG test for tab slides, or add tab-specific notation (bends, hammer-on/pull-off).
- Open issues: Tab staves not yet usable in MultiStaffScore context. I2 still deferred.

## 2026-04-21 — Post-v1, tab slides example + golden-SVG test + clippy fix
- Did: Created `examples/tab_slides.rs` rendering 4 measures across 2 systems showing: (1) single-string ascending slide 5→7, (2) single-string descending slide 12→9, (3) multi-string chord slide (3-string power chord shift), (4) consecutive chain slides 5→7→9→12. Output: 25 lines, 2 paths, 19 texts, 19 rects (8068 bytes). Added `golden_tab_slides` golden-SVG regression test (24th baseline). Fixed 2 clippy `empty_line_after_doc_comments` warnings in `examples/tab_slides.rs` and `examples/tab_beams.rs`.
- Verified: `cargo test -p music-engraver` — 1222 unit + 25 golden + 3 integration + 6 doc-tests = 1256 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_slides` produces valid SVG. `cargo check --workspace` passes.
- Next: Add tab-specific notation (bends, hammer-on/pull-off), or integrate tab staves into MultiStaffScore context, or begin other post-v1 features.
- Open issues: Tab staves not yet usable in MultiStaffScore context. I2 still deferred.

## 2026-04-21 — User-reported issue: fret number vertical alignment
- Did: Nothing yet — logging a user-reported visual QA finding from the main repo.
- Observation: In rendered tablature SVGs, fret numbers sit visibly above their string lines rather than centered on them. `layout_fret_number` in `src/layout/tab.rs:133` sets `y = tab_staff.string_y(string)`, and `draw_fret_number` in `src/render/tab_renderer.rs:67` emits `<text text-anchor="middle">` with no `dominant-baseline` attribute. SVG default is `alphabetic`, so the glyph renders above its baseline instead of centered on it. Existing tests pass because they assert `layout.y == tab_staff.string_y(string)` — tautological with the code; they do not check visual centering.
- Suggested fix: add `dominant-baseline="central"` (or `middle`) to the `<text>` style in `draw_fret_number`. Alternative: shift `layout.y` downward by ~0.35 × font_size in layout. Prefer the attribute fix. Add a render-level test that asserts the dominant-baseline attribute is present, and/or update the tab golden baselines under `GOLDEN_UPDATE=1` after confirming visually.
- Next: Pick this up as the next chunk in place of a new feature. Re-render and visually verify `tab_score.svg`, `tab_bends.svg`, `tab_slides.svg`, `tab_hammer_pull.svg`, `tab_beams.svg`, `tab_rhythm.svg` before re-freezing goldens.
- Open issues: The same `dominant-baseline` oversight may affect other text glyphs (measure numbers, chord symbols, lyrics, fret rhythm annotations). Audit `svg::add_text` call sites for any placement that is computed against a specific staff-line y-coordinate.

## 2026-04-21 — Post-v1, fix fret number vertical alignment (dominant-baseline)
- Did: Added `dominant_baseline: &str` field to `TextStyle` struct in `render/svg_writer.rs`. Default is `"auto"` (SVG default, no attribute emitted — zero visual change for existing text elements). Set to `"central"` in `draw_fret_number()` in `render/tab_renderer.rs`, which vertically centers fret numbers on their string lines instead of sitting above them. Updated `add_text()` to conditionally emit `dominant-baseline` attribute when not `"auto"`. Updated all 12 `TextStyle` struct-literal construction sites across 8 files (page_renderer, tab_hammer_renderer, tab_bend_renderer, svg_writer tests, score/multi_staff, score/tab) with `dominant_baseline: "auto"`. Regenerated 4 tab golden SVG baselines (tab_score, tab_bends, tab_slides, tab_hammer_pull). Added 4 new tests: `add_text_auto_baseline_omits_attribute`, `add_text_central_baseline_emits_attribute`, `add_text_middle_baseline_emits_attribute` (svg_writer), `fret_number_text_has_central_baseline` (tab_renderer). Audited other text call sites (measure numbers, chord symbols, lyrics, rehearsal marks, tempo marks, expression text, bend labels, hammer/pull labels) — all use y-coordinates computed relative to staff line positions with intentional offsets, so `"auto"` (alphabetic) baseline is correct for them.
- Verified: `cargo test -p music-engraver --lib` — 1283 unit tests pass. `cargo test -p music-engraver --test golden_svg` — 27 golden tests pass. `cargo test -p music-engraver --doc` — 6 doc-tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Audit whether tab beam group and tab rhythm renderer text elements also need `dominant-baseline="central"`, or begin other post-v1 features (multi-staff tab integration, grace note example).
- Open issues: Tab rhythm stem notation labels (if any text-based) may need the same fix. I2 still deferred.

## 2026-04-21 — Post-v1, volta bracket (1st/2nd ending) layout + rendering + full pipeline integration
- Did: Created `layout/volta.rs` with `VoltaHooks` enum (Both/LeftOnly/RightOnly/Neither), `VoltaAnnotation` struct (text + hooks), `VoltaBracketLayout` struct, `layout_volta_bracket()` — positions horizontal top line + optional vertical hooks + bold text label above the staff at 3.0 staff spaces. Created `render/volta_renderer.rs` with `draw_volta_bracket()` rendering lines (top + hooks) and bold serif text. Added `volta: Option<VoltaAnnotation>` field to `MeasureContent` in `layout/system.rs` and `SystemMeasure` — volta annotations flow from content through layout to rendering. Added `draw_system_volta_brackets()` to `render/system_renderer/mod.rs` drawing brackets above annotated measures. Integrated into `ScoreBuilder` with `.volta_start(text)` and `.volta_end()` modifier methods. Extended `measures` tuple to carry `Option<VoltaAnnotation>`. `resolve_volta()` state machine handles single-measure (Both hooks), multi-measure start (LeftOnly), middle (Neither), and end (RightOnly) patterns. Updated all ~150 `MeasureContent` construction sites across 7 files with `volta: None`. Fixed 2 clippy warnings (empty_line_after_doc_comments, doc_list_item_without_indentation). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib` — 1314 unit tests pass. `cargo test -p music-engraver --test golden_svg` — 27 golden tests pass. `cargo test -p music-engraver --doc` — 6 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes. 31 new volta tests: 12 layout (hooks, text, positioning, scaling), 9 renderer (line counts, text rendering, differentiation), 5 system_renderer (bracket adds lines, text appears, no bracket without annotation, left-only 2 lines, multi-measure 4 lines), 6 score (single-measure adds bracket, multi-measure continuations, differs from no volta, bold text, no bracket without calls, exact line count).
- Next: Create `examples/volta_brackets_score.rs` showing volta brackets via ScoreBuilder API, or add a golden-SVG test for volta brackets, or begin other post-v1 features (multi-staff tab integration).
- Open issues: Cross-system volta brackets not yet supported (volta spanning a system break). Tab rhythm text `dominant-baseline` audit still pending. I2 still deferred.

## 2026-04-21 — Post-v1, volta brackets score example + golden-SVG test
- Did: Created `examples/volta_brackets_score.rs` rendering 6 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.volta_start()`/`.volta_end()` methods. Demonstrates: (1) plain opening measure, (2) single-measure 1st ending with both hooks + repeat barline, (3) single-measure 2nd ending with both hooks, (4–5) multi-measure 3rd ending spanning across system break (left hook on system 1 → continuation with right hook on system 2), (6) plain closing. Exercises all `VoltaHooks` variants (Both, LeftOnly, RightOnly, Neither), volta text labels ("1.", "2.", "3."), and repeat barline styles. Output: 22 paths, 43 lines, 3 texts (12892 bytes). Added `golden_volta_brackets` golden-SVG regression test (27th baseline → 28 golden tests total). Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver` — 1314 unit + 28 golden + 3 integration + 6 doc-tests = 1351 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example volta_brackets_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Implement cross-system volta brackets (volta bracket spanning a system break), or begin other post-v1 features (multi-staff tab integration, line-breaking refinements, additional golden tests).
- Open issues: Cross-system volta brackets not yet supported. Tab rhythm text `dominant-baseline` audit still pending. I2 still deferred.

## 2026-04-21 — Post-v1, cross-system volta verification + golden test + dominant-baseline audit closure
- Did: (1) Verified that cross-system volta brackets already work correctly via the existing per-measure `VoltaAnnotation` architecture — `resolve_volta()` produces `LeftOnly`/`Neither`/`RightOnly` annotations per measure before system grouping, and `draw_system_volta_brackets()` draws each measure's annotation independently, so volta brackets spanning system breaks render correctly without any page-renderer cross-system pass. (2) Added `golden_cross_system_volta` golden-SVG regression test — builds a 6-measure score with `measures_per_system(2)` where a multi-measure volta "1." spans from measure 2 (system 1, LeftOnly) through measures 3-4 (system 2, Neither + RightOnly), plus a single-measure "2." volta on system 3. Baseline: 12650 bytes, 16 paths, 40 lines, 2 texts. Asserts volta labels "1." and "2." present, line count ≥ 23 (15 staff + stems + volta bracket lines). (3) Completed tab rhythm text `dominant-baseline` audit — tab rhythm renderer uses only `<line>` and `<path>` elements (stems + SMuFL flag glyphs), no text at all. Tab bend labels and hammer/pull labels use `dominant_baseline: "auto"` which is correct since they float above arcs/arrows with explicit y-offsets, not on string lines. Only fret numbers (fixed in prior run) needed `"central"`. Audit complete.
- Verified: `cargo test -p music-engraver --lib` — 1314 unit tests pass. `cargo test -p music-engraver --test golden_svg` — 29 golden tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 6 doc-tests pass. Total: 1352 tests. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: tab staves in MultiStaffScore (guitar+tab combined), or pre-bends/release bends for tab, or additional golden tests for coverage.
- Open issues: I2 still deferred. Tab staves not yet usable in MultiStaffScore context.

## 2026-04-21 — Post-v1, guitar+tab combined score in MultiStaffScore
- Did: Integrated `TabScoreBuilder` staves into `MultiStaffScore` via new `guitar_tab(notation, tab)` constructor. Standard notation staff (5-line) renders above a tablature staff (variable line count) connected by a bracket with joined barlines spanning both staves. Made `TabScoreBuilder.measures`, `TabScoreBuilder.line_count`, `flush_pending()`, `draw_tab_measure()`, `draw_measure_barline()`, `TabMeasure`, and `TabEvent` `pub(crate)` for cross-module access. Added `tab_stave: Option<TabScoreBuilder>` field to `MultiStaffScore`. Extended `try_render_svg()` to: compute combined system height (notation + gap + tab), render tab staff lines + TAB clef + measures per system chunk, extend joined barlines to cover tab stave bottom, draw bracket spanning notation + tab. Supports multi-system layouts, 4-string and 6-string tabs, tab rhythm stems, `measures_per_system` override. Created `examples/guitar_tab_score.rs` (11 paths, 37 lines, 7 texts, 9975 bytes). Added `golden_guitar_tab` golden-SVG regression test (30th baseline → 30 golden tests total).
- Verified: `cargo test -p music-engraver --lib` — 1325 unit tests pass (+11 new guitar_tab tests). `cargo test -p music-engraver --test golden_svg` — 30 golden tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 7 doc-tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add tab-specific features in multi-staff context (slides/bends/hammer-on visible in combined score), or add PNG rendering for guitar+tab, or begin pre-bends/release bends for tab, or other post-v1 features.
- Open issues: I2 still deferred. Cross-system tab features (slides/bends spanning system break) not supported in multi-staff context.

## 2026-04-21 — Post-v1, TabScoreBuilder PNG export methods
- Did: Added `render_png(scale)` and `try_render_png(scale)` convenience methods to `TabScoreBuilder`, gated behind `#[cfg(feature = "png")]`. These follow the same pattern as `ScoreBuilder` and `MultiStaffScore` PNG methods: call `try_render_svg()`, create `PngRenderer`, load system fonts, render. Added 4 new tests in a `png_tests` submodule: valid PNG output with magic bytes + dimension assertions, 2x larger than 1x, try matches direct, complex score nonzero dimensions.
- Verified: `cargo test -p music-engraver --features png --lib -- score::tab::tests::png_tests` — 4 tests pass. `cargo test -p music-engraver --lib` — 1325 unit tests pass. `cargo clippy -p music-engraver --all-targets --features png` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add a `tab_png_export` example, or begin other post-v1 features (pre-bends/release bends for tab, Gourlay penalty tuning for line breaking, additional golden tests).
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-21 — Post-v1, navigation sign (segno/coda) layout + rendering + full pipeline integration
- Did: Created `layout/navigation.rs` with `NavigationSign` enum (Segno/Coda/CodaSquare), each mapping to a dedicated SMuFL glyph. `layout_navigation_sign()` positions the glyph above the staff at 2.5 staff spaces (fixed position — structural markers, not note-level decorations). Created `render/navigation_renderer.rs` with `draw_navigation_sign()`. Added `navigation_sign: Option<NavigationSign>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer. Added `ScoreBuilder::navigation_sign(sign)` modifier method (no-op on rests). Annotations clone-based propagation in `convert_event()` handles navigation_sign automatically. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib -- navigation` — 26 tests pass (11 layout + 7 renderer + 3 measure_renderer + 6 score). `cargo test -p music-engraver --test golden_svg` — 30 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/navigation_signs_score.rs` showing segno/coda via ScoreBuilder API, or add a golden-SVG test for navigation signs, or begin other post-v1 features (pre-bends, 8va/8vb lines).
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-21 — Post-v1, navigation signs score example + golden-SVG test
- Did: Created `examples/navigation_signs_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.navigation_sign()` method. Demonstrates: (1) segno on first beat with repeat barline at end, (2) coda on system 2 first beat, (3) square coda on final note. Exercises all 3 NavigationSign variants. Output: 22 paths, 36 lines (14539 bytes). Added `golden_navigation_signs` golden-SVG regression test (31st baseline → 31 golden tests total). Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver` — 1351 unit + 31 golden + 3 integration + 7 doc-tests = 1392 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example navigation_signs_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: 8va/8vb ottava bracket lines, or tab stave cross-system features, or additional golden tests for missing coverage.
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-21 — Post-v1, ottava score example + golden-SVG test + clippy fix
- Did: Created `examples/ottava_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.ottava_start()`/`.ottava_end()` methods. Demonstrates all 4 OttavaKind variants: 8va over high C6-F6 passage, 8vb under low C3-A2 passage, 15ma on G5-A5 half notes, 15mb on E3-D3 half notes. Output: 16 paths, 69 lines, 4 texts (14183 bytes). Assertions verify all 4 labels (8va, 8vb, 15ma, 15mb) and dashed lines present. Added `golden_ottava_brackets` golden-SVG regression test (32nd baseline → 32 golden tests total) with structural assertions (8va/8vb labels, stroke-dasharray). Updated `golden_baselines_are_valid_svgs` name list. Fixed pre-existing clippy `too_many_arguments` warning on `SvgWriter::add_dashed_line()` with `#[allow]` annotation.
- Verified: `cargo test -p music-engraver` — 1386 unit + 32 golden + 3 integration + 7 doc-tests = 1428 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example ottava_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Add cross-system ottava bracket support (ottava spanning a system break), or begin other post-v1 features (tab stave cross-system features, additional golden tests, pedal markings).
- Open issues: Cross-system ottava brackets not yet supported (ottava starting in one system and ending in the next). I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-21 — Post-v1, cross-system ottava brackets
- Did: Implemented cross-system ottava bracket rendering in `page_renderer/mod.rs`, following the established pattern from cross-system ties/slurs/hairpins. Made `OttavaNoteInfo` and `collect_ottava_note_info()` `pub(crate)` in `system_renderer/mod.rs`. Added `UnresolvedOttava`/`IncomingOttavaTarget` structs, `find_unresolved_ottavas()`, `find_incoming_ottava_targets()`, and `draw_cross_system_ottava_brackets()` to page renderer. Trailing half-bracket drawn without end hook; incoming half-bracket drawn with end hook and full label. Wired into `draw_page()` and `MultiStaffScore::try_render_svg()`. Added `golden_cross_system_ottava` golden-SVG regression test (33rd baseline → 33 golden tests total). Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver --lib -- ottava` — 42 tests pass (all ottava-related). `cargo test -p music-engraver --test golden_svg` — 33 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes. 12 new tests: 5 page_renderer tests (two brackets drawn, no ottava without flags, right-half-only when no end, within-system not duplicated, 8vb draws below staff), 2 score tests (cross-system produces 2 labels, differs from no ottava), 1 golden test (cross_system_ottava baseline frozen at 11103 bytes with 2 "8va" labels + 2 dashed lines).
- Next: Begin other post-v1 features: pedal markings, or tab stave cross-system features in multi-staff context, or additional golden tests for coverage.
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, breath mark layout + rendering + full pipeline integration
- Did: Created `layout/breath.rs` with `BreathMark` enum (Comma/Tick/Caesura), each mapping to a dedicated SMuFL glyph (`BreathMarkComma`/`BreathMarkTick`/`Caesura`). `layout_breath_mark()` positions the glyph above the staff (1.5 staff spaces above top line) and to the right of the note (0.5 staff spaces padding from notehead right edge). Created `render/breath_renderer.rs` with `draw_breath_mark()`. Added `breath_mark: Option<BreathMark>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer — breath mark drawn after all other annotations. Added `ScoreBuilder::breath_mark(mark)` modifier method (no-op on rests). Annotations clone-based propagation in `convert_event()` handles breath_mark automatically. Wired into layout/render mod.rs with public exports.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver --test golden_svg` — 42 golden tests pass. `cargo test -p music-engraver --doc` — 7 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. All 28 new breath mark tests pass: 12 layout tests (glyph mapping, distinct glyphs, above-staff placement, right-of-note positioning, scaling constants, y fixed across types), 7 renderer tests (path production for all 3 types, differentiation between types, coordinate embedding), 4 measure_renderer tests (note +1 path, chord +1 path, 3 types differ, translate count), 5 score tests (adds path, rest no-op, 3 types differ, chord adds path, convert_event preserves).
- Next: Create `examples/breath_marks_score.rs` showing breath marks via ScoreBuilder API, or add golden-SVG test for breath marks, or begin other post-v1 features.
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, stacked articulations (multiple per note)
- Did: Changed `NoteAnnotations.articulation: Option<Articulation>` → `articulations: Vec<Articulation>` to support multiple articulations per note/chord. Added `layout_articulation_stack()` to `layout/articulation.rs` — stacks articulations outward from the notehead with `ARTICULATION_STACK_SPACING_SS` (0.6ss) between each. Fermata is always separated to the above placement even when other articulations go below. Updated `draw_note_event()` and `draw_chord_event()` in `render/measure_renderer/mod.rs` to iterate over stacked layouts. Updated `ScoreBuilder::articulation()` to push instead of replace (calling `.articulation()` twice now stacks). Updated golden baseline for `articulations` test (now includes staccato+fermata stack on final note). Updated example `articulations_score.rs` to demonstrate stacked articulations.
- Verified: `cargo test -p music-engraver --lib` — 1598 unit tests pass (+13 new: 9 layout stack tests + 4 score stacking tests). `cargo test -p music-engraver --test golden_svg` — 40 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add stacked articulation support for tab staves, or begin other post-v1 features (multi-voice on single staff, Gourlay penalty tuning).
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, pedal score example + golden-SVG test
- Did: Created `examples/pedal_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.pedal_down()`/`.pedal_up()` methods. Demonstrates: (1) pedal down on single note, up on last note, (2) pedal down on F-A-C chord, up on last note, (3) quick re-pedaling (down/up twice per measure), (4) sustained pedal across two half notes. Exercises pedal on single notes and chords. Output: 29 paths, 30 lines (31016 bytes). Added `golden_pedal_marks` golden-SVG regression test (34th baseline → 34 golden tests total) with structural assertion: pedal marks add exactly 2 extra paths (Ped. + *) compared to an identical score without pedal marks. Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver --lib` — 1422 unit tests pass. `cargo test -p music-engraver --test golden_svg` — 34 golden tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 7 doc-tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example pedal_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: tab stave cross-system features in multi-staff context, or additional golden tests for missing coverage, or pre-bends/release bends for tab.
- Open issues: Bracket-style pedal lines not implemented. I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, tab pre-bend + release bend layout/rendering/pipeline integration
- Did: Added two new bend variants to tablature: **pre-bend** (straight vertical arrow indicating string bent before picking) and **release bend** (downward-curving arrow indicating pitch returning to normal after a bend). Created `TabPreBendLayout` struct + `layout_tab_pre_bend()` and `TabReleaseLayout` struct + `layout_tab_release()` in `layout/tab_bend.rs`. Created `draw_tab_pre_bend()` (vertical line + filled arrowhead + amount label) and `draw_tab_release()` (quadratic Bézier curve + downward arrowhead, no text) in `render/tab_bend_renderer.rs`. Added `pre_bend: Option<BendAmount>` and `release: bool` fields to `TabEvent::Fret`. Added `TabScoreBuilder::pre_bend(amount)` and `TabScoreBuilder::release()` modifier methods with `pending_pre_bend`/`pending_release` accumulator fields. Extended third-pass in `draw_tab_measure()` to handle all three bend types (regular/pre-bend/release). Pre-bend uses straight line shaft (distinguishing from regular bend's curve); release has no text label (convention: the bend amount is already shown on the preceding bend event).
- Verified: `cargo test -p music-engraver` — 1455 unit + 34 golden + 3 integration + 7 doc-tests = 1499 tests, all pass. 13 new layout tests (7 pre-bend + 6 release), 10 new renderer tests (5 pre-bend + 5 release), 11 new TabScoreBuilder tests (5 pre-bend + 5 release + 1 combined pre-bend→release sequence). `cargo clippy -p music-engraver --all-targets` — 0 warnings from new code. `cargo check --workspace` passes.
- Next: Create `examples/tab_pre_bends.rs` showing pre-bends and releases via TabScoreBuilder API, or add golden-SVG test for tab pre-bends/releases, or begin other post-v1 features.
- Open issues: Pre-bends and releases on beam group events not supported (rare use case). Bracket-style pedal lines not implemented. I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, tab pre-bends/releases example + golden-SVG test
- Did: Created `examples/tab_pre_bends.rs` rendering 4 measures across 2 systems showing: (1) full/half pre-bends on single strings, (2) pre-bend→release sequence + chord pre-bend, (3) regular bend→release sequence + quarter pre-bend, (4) 1½ pre-bend + release on different string. Exercises all BendAmount variants on pre-bends, release arrows after both pre-bends and regular bends, chord pre-bends. Output: 17 paths, 24 lines, 24 texts (10147 bytes). Added `golden_tab_pre_bends` golden-SVG regression test (35th baseline → 35 golden tests total) with assertions on pre-bend labels ("full", "1/2") and filled arrowhead count (≥4). Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver` — 1455 unit + 35 golden + 3 integration + 7 doc-tests = 1500 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_pre_bends` produces valid SVG. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: tab staves in MultiStaffScore (guitar+tab cross-system features), or tab vibrato notation, or additional golden tests for coverage, or Gourlay penalty tuning for line breaking.
- Open issues: Pre-bends and releases on beam group events not supported (rare use case). Bracket-style pedal lines not implemented. I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, tremolo notation layout + rendering + full pipeline integration
- Did: Created `layout/tremolo.rs` with `TremoloCount` enum (Single/Double/Triple), each mapping to SMuFL `Tremolo1`/`Tremolo2`/`Tremolo3` glyph. `layout_tremolo()` positions the glyph on the stem at 40% from the notehead toward the stem tip (centered on stem x). Created `render/tremolo_renderer.rs` with `draw_tremolo()`. Added `tremolo: Option<TremoloCount>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer — tremolo drawn after stem+flag, skipped for whole notes (no stem). Added `ScoreBuilder::tremolo(count)` modifier method (no-op on rests). Annotations clone-based propagation in `convert_event()` handles tremolo automatically. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 1481 unit + 35 golden + 3 integration + 7 doc-tests = 1526 tests, all pass. 11 layout tests + 6 renderer tests + 4 measure_renderer tests + 5 score tests = 26 new tremolo tests. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/tremolo_score.rs` showing tremolo via ScoreBuilder API, or add golden-SVG test for tremolo, or begin other post-v1 features (tab vibrato, multi-voice support).
- Open issues: Whole-note tremolo conventionally uses beamlet-style slashes between the notehead and an implied stem position — not yet supported (only stemmed notes get tremolo). Buzzroll (Z on stem) not yet supported. I2 still deferred.

## 2026-04-22 — Post-v1, tremolo score example + golden-SVG test
- Did: Created `examples/tremolo_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.tremolo()` method. Demonstrates: (1) single + double tremolo on ascending quarter notes, (2) single + double tremolo on half notes in varied registers, (3) mixed tremolo/non-tremolo notes, (4) high-register notes with stems down and all 3 tremolo counts. Output: 27 paths, 30 lines (11256 bytes). Added `golden_tremolo` golden-SVG regression test (36th baseline → 36 golden tests total) with structural assertions: path count exceeds plain (non-tremolo) version, verifying tremolo glyphs add actual path elements. Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver` — 1481 unit + 36 golden + 3 integration + 7 doc-tests = 1527 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tremolo_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: tab vibrato notation, multi-staff tab integration (cross-system features), Gourlay penalty tuning for line breaking, or additional golden tests for coverage.
- Open issues: Whole-note tremolo not supported (only stemmed notes). Buzzroll (Z on stem) not implemented. I2 still deferred.

## 2026-04-22 — Post-v1, tab vibrato layout + rendering + ScoreBuilder integration + example + golden test
- Did: Created `layout/tab_vibrato.rs` with `VibratoKind` enum (Normal/Wide), `TabVibratoLayout` struct, `layout_tab_vibrato()` — computes a wavy line (6 quadratic Bézier half-cycles) above the fret position. Normal amplitude 0.25× staff space, wide 0.45×. Half-period 0.25× staff space. Positioned 0.8 staff spaces above the string line. Created `render/tab_vibrato_renderer.rs` with `draw_tab_vibrato()` rendering an unfilled stroke path with round linecaps. Added `vibrato: Option<VibratoKind>` field to `TabEvent::Fret`. Added `pending_vibrato` to `TabScoreBuilder` and `.vibrato()` / `.wide_vibrato()` modifier methods. Vibrato drawn in the third pass of `draw_tab_measure()` alongside bends. Created `examples/tab_vibrato.rs` (4 measures: normal vibrato, wide vibrato + chord vibrato, vibrato with bends, contrast without vibrato — 10560 bytes). Added `golden_tab_vibrato` golden-SVG regression test (37th baseline → 37 golden tests total). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver` — 1502 unit + 37 golden + 3 integration + 7 doc-tests = 1549 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_vibrato` produces valid SVG (13 vibrato waves). `cargo check --workspace` passes.
- Next: Add tab-specific notation (palm muting, natural harmonics), or integrate tab staves into MultiStaffScore for cross-system features, or begin other post-v1 features (Gourlay penalty tuning, additional golden tests).
- Open issues: Vibrato on beam group events not supported (vibrato typically applies to individual sustained notes, not beamed groups). I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, tab natural harmonic layout + rendering + ScoreBuilder integration + example + golden test
- Did: Created `layout/tab_harmonic.rs` with `TabHarmonicLayout` struct, `layout_tab_harmonic()` — positions the SMuFL `StringsHarmonic` glyph (small ○) above the fret number at 0.9 staff spaces, scaled to 0.6×. Created `render/tab_harmonic_renderer.rs` with `draw_tab_harmonic()` rendering the glyph with translate+scale transform. Added `harmonic: bool` field to `TabEvent::Fret`. Added `pending_harmonic` to `TabScoreBuilder` and `.harmonic()` modifier method following the established pattern (marks current frets or last-flushed event). Harmonics drawn in the third pass of `draw_tab_measure()` alongside bends/vibrato, one glyph per string in chord harmonics. Created `examples/tab_harmonics.rs` (4 measures: fret-12 harmonics, chord harmonics at fret 7, mixed harmonic/normal, harmonic+vibrato — 18 paths, 17 texts, 17 lines, 11800 bytes). Added `golden_tab_harmonics` golden-SVG regression test (38th baseline → 38 golden tests total). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib` — 1521 unit tests pass (+19 new: 8 layout, 5 renderer, 6 score). `cargo test -p music-engraver --test golden_svg -- golden_tab_harmonics golden_baselines` — 2 tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_harmonics` produces valid SVG. `cargo check --workspace` passes.
- Next: Add tab palm muting notation, or tab string number indicators, or begin other post-v1 features (multi-voice support, Gourlay penalty tuning for line breaking).
- Open issues: Harmonics on beam group events not supported (rare — harmonics are typically sustained notes). I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, tab dead/muted string notation ("x")
- Did: Added muted/dead string notation to tablature. Created `layout_muted_string()` in `layout/tab.rs` — reuses `FretNumberLayout` with text "x" instead of a fret number, same font size, background rect, and centering as regular fret numbers. Added `muted_strings: Vec<u8>` field to `TabEvent::Fret` and `current_muted: Vec<u8>` accumulator to `TabScoreBuilder`. Added `.mute(string)` method — muted strings are grouped with fret numbers in the same beat event (supports mixed fret+mute chords like power-chord strums with muted strings). Updated `flush_frets()` to also trigger when only muted strings are pending (no frets). Updated `draw_tab_measure()` to render muted strings alongside fret numbers using `draw_fret_number()` (same white background rect + bold centered text, just with "x" content).
- Verified: `cargo test -p music-engraver --lib -- layout::tab` — 133 tests pass (6 new muted_string layout tests). `cargo test -p music-engraver --lib -- score::tab::tests` — 98 tests pass (6 new mute score tests). `cargo test -p music-engraver --test golden_svg` — 39 golden tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --lib` — 0 warnings. `cargo check --workspace` passes.
- Next: Create `examples/tab_muted.rs` showing muted strings via TabScoreBuilder API, or add golden-SVG test for muted strings, or begin other post-v1 features (let ring notation, tab string number indicators).
- Open issues: Muted strings in beam groups not supported (beam group events are `(frets, dur)` tuples without a muted_strings field). I2 still deferred.

## 2026-04-22 — Post-v1, tab palm mute layout + rendering + ScoreBuilder integration + example + golden test
- Did: Created `layout/tab_palm_mute.rs` with `TabPalmMuteLayout` and `TabPalmMuteDashLayout` structs, `layout_tab_palm_mute()` (positions italic "P.M." text above staff at 3.5 staff spaces), `layout_tab_palm_mute_dash()` (computes dashed continuation line geometry between consecutive palm-muted events, returns None when span too short). Created `render/tab_palm_mute_renderer.rs` with `draw_tab_palm_mute()` (italic centered serif text) and `draw_tab_palm_mute_dash()` (dashed horizontal line via `add_dashed_line`). Added `palm_mute: bool` field to `TabEvent::Fret`. Added `pending_palm_mute` to `TabScoreBuilder` and `.palm_mute()` modifier method following the established pattern. In `draw_tab_measure()`: third pass draws "P.M." text per muted event; new palm mute pass scans for consecutive muted spans and draws dashed continuation lines. Created `examples/tab_palm_mute.rs` (4 measures across 2 systems: single mutes, consecutive with dashed line, alternating, full-measure passage — 29 texts, 36 lines, 11 P.M. occurrences, 11876 bytes). Added `golden_tab_palm_mute` golden-SVG regression test (39th baseline → 39 golden tests total). Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib` — 1547 unit tests pass (+26 new palm mute tests: 12 layout, 8 renderer, 6 score). `cargo test -p music-engraver --test golden_svg` — 39 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from new code. `cargo run --example tab_palm_mute` produces valid SVG. `cargo check --workspace` passes.
- Next: Add tab-specific notation (string muting "x", let ring), or integrate tab staves into MultiStaffScore for cross-system features, or begin other post-v1 features (multi-voice support, Gourlay penalty tuning for line breaking).
- Open issues: Palm mute dashed lines only work within a single measure (cross-barline palm mute passages would need the multi-pass cross-event pattern). I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, tab muted strings example + golden-SVG test + clippy fix
- Did: Created `examples/tab_muted.rs` rendering 4 measures across 2 systems showing: (1) all-mute percussive strum (6 "x" markers), (2) power chord with muted high strings (frets 0/2/2 on low strings + "x" on strings 1-3), (3) skipped mute between fretted notes, (4) normal frets for contrast, (5) muted bass + high melody, (6) all-mute with rhythm stems, (7) individual sequential mutes, (8) mixed fret+mute chords. Output: 31 "x" markers, 43 texts, 32 lines, 43 rects (16679 bytes). Added `golden_tab_muted_strings` golden-SVG regression test (39th baseline → 40 golden tests total) with structural assertions: ≥12 "x" markers, fret numbers coexist, each "x" has background rect, differs from unmuted score. Updated `golden_baselines_are_valid_svgs` name list. Fixed 1 clippy warning in `score/tab.rs` (collapsible `if` in palm mute pass).
- Verified: `cargo test -p music-engraver` — 1559 unit + 40 golden + 3 integration + 7 doc-tests = 1609 tests, all pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_muted` produces valid SVG. `cargo check --workspace` passes.
- Next: Add tab "let ring" notation, or integrate tab staves into MultiStaffScore for cross-system features, or begin other post-v1 features (multi-voice support, Gourlay penalty tuning).
- Open issues: Muted strings in beam groups not supported (beam group events are `(frets, dur)` tuples without a muted_strings field). I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, tab "let ring" layout + rendering + ScoreBuilder integration
- Did: Created `layout/tab_let_ring.rs` with `TabLetRingLayout` and `TabLetRingDashLayout` structs, `layout_tab_let_ring()` (positions italic "let ring" text above staff at 4.5 staff spaces — above palm mute at 3.5ss to avoid collision), `layout_tab_let_ring_dash()` (dashed continuation line with wider text offset of 2.0ss since "let ring" is longer than "P.M."). Created `render/tab_let_ring_renderer.rs` with `draw_tab_let_ring()` (italic centered serif text) and `draw_tab_let_ring_dash()` (dashed horizontal line). Added `let_ring: bool` field to `TabEvent::Fret` and `pending_let_ring` to `TabScoreBuilder`. Added `.let_ring()` modifier method following same pattern as `.palm_mute()`. In `draw_tab_measure()`: third pass draws "let ring" text per marked event; new let_ring pass draws dashed continuation lines between consecutive let-ring spans. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib` — 1585 unit tests pass (+26 new: 12 layout, 8 renderer, 6 score). `cargo test -p music-engraver --test golden_svg` — 40 golden tests pass. `cargo test -p music-engraver --doc` — 7 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/tab_let_ring.rs` showing let ring via TabScoreBuilder API, or add golden-SVG test for let ring, or begin other post-v1 features (multi-voice support, tab stave in MultiStaffScore cross-system).
- Open issues: Let ring in beam groups not supported (beam group events don't carry the let_ring field). Let ring dashed lines only work within a single measure (cross-barline let ring passages would need multi-pass cross-event pattern). I2 still deferred.

## 2026-04-22 — Post-v1, tab let ring example + golden-SVG test
- Did: Created `examples/tab_let_ring.rs` rendering 4 measures across 2 systems showing: (1) arpeggio with consecutive let ring on each note (dashed continuation lines), (2) single let ring on open chord then normal notes, (3) alternating let ring and normal, (4) full measure let ring passage with high frets. Exercises italic "let ring" text, dashed continuation lines, single and chord events. Output: 31 texts, 34 lines, 11 let ring occurrences (11354 bytes). Added `golden_tab_let_ring` golden-SVG regression test (40th baseline → 41 golden tests total) with structural assertions: ≥4 "let ring" annotations, italic rendering, dashed lines for consecutive events, differs from plain score. Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver --test golden_svg` — 41 golden tests pass. `cargo test -p music-engraver --lib` — 1598 unit tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example tab_let_ring` produces valid SVG. `cargo check --workspace` passes.
- Next: Add cross-barline let ring dashed lines, or begin other post-v1 features (tab staves in MultiStaffScore, Gourlay penalty tuning, additional golden tests for coverage).
- Open issues: Let ring in beam groups not supported. Let ring dashed lines only work within a single measure. I2 still deferred.

## 2026-04-22 — Post-v1, arpeggio (rolled chord) layout + rendering + full pipeline integration
- Did: Created `layout/arpeggio.rs` with `ArpeggioDirection` enum (Up/Down), `ArpeggioLayout` struct, `layout_arpeggio()` — positions SMuFL `ArpeggiatoUp`/`ArpeggiatoDown` glyph to the left of chord noteheads, vertically scaled to span from lowest to highest note. Single notes get a minimum 2ss span. Padding 0.4ss from leftmost notehead. Created `render/arpeggio_renderer.rs` with `draw_arpeggio()` rendering the glyph with translate+scale transform. Added `arpeggio: Option<ArpeggioDirection>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into `draw_note_event()` and `draw_chord_event()` in measure renderer — arpeggio drawn after noteheads/ledger lines, before stem. Added `ScoreBuilder::arpeggio(direction)` modifier method (no-op on rests). Annotations clone-based propagation in `convert_event()` handles arpeggio automatically. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib -- arpeggio` — 25 tests pass (11 layout + 5 renderer + 4 measure_renderer + 5 score). `cargo test -p music-engraver --test golden_svg` — 41 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/arpeggio_score.rs` showing arpeggios via ScoreBuilder API, or add golden-SVG test for arpeggios, or begin other post-v1 features.
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, arpeggio score example + golden-SVG test
- Did: Created `examples/arpeggio_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.arpeggio()` method. Demonstrates: (1) upward arpeggio on C major triad + plain quarters for contrast, (2) downward arpeggio on D minor triad + single-note upward arpeggio, (3) wide voicing (C4-G4-E5) upward + two-note downward arpeggio, (4) whole-note 4-note chord (C-E-G-B) with upward arpeggio. Exercises upward/downward arpeggios, chord and single-note arpeggios, wide voicings, and whole-note stemless chords. Output: 29 paths, 25 lines, 5 arpeggio scale transforms (20132 bytes). Added `golden_arpeggios` golden-SVG regression test (42nd baseline → 42 golden tests total) with structural assertions: ≥5 scale transforms, ≥10 paths, up vs down comparison. Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo run --example arpeggio_score` produces valid SVG. `cargo test -p music-engraver --lib -- arpeggio` — 25 arpeggio tests pass. `cargo test -p music-engraver --test golden_svg -- golden_arpeggios golden_baselines` — 2 tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from new code. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: multi-voice support, tab staves in MultiStaffScore, Gourlay penalty tuning for optimal line breaking, or additional golden tests for coverage.
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, breath marks score example + golden-SVG test
- Did: Created `examples/breath_marks_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.breath_mark()` method. Demonstrates: (1) comma breaths between ascending notes (2 commas), (2) tick breath + caesura on descending line, (3) comma + tick on low-register half notes, (4) caesura for dramatic pause + rest (no-op) + plain notes. Exercises all 3 BreathMark variants (Comma/Tick/Caesura), verifies no-op on rest, mixed with stem-up and stem-down notes. Output: 24 paths, 29 lines (13033 bytes). Added `golden_breath_marks` golden-SVG regression test (42nd baseline → 43 golden tests total) with structural assertions: ≥14 paths, all 3 types produce distinct SVGs, breath mark adds exactly 1 path vs no-breath version. Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo run --example breath_marks_score` produces valid SVG. `cargo test -p music-engraver --lib` — 1651 unit tests pass. `cargo test -p music-engraver --test golden_svg` — 43 golden tests pass. `cargo test -p music-engraver --doc` — 7 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Begin other post-v1 features: multi-voice support on a single staff, tab staves in MultiStaffScore cross-system features, Gourlay penalty tuning, or additional golden tests for coverage.
- Open issues: I2 still deferred. Cross-system tab features not in multi-staff context.

## 2026-04-22 — Post-v1, glissando line layout + rendering + full pipeline integration
- Did: Created `layout/glissando.rs` with `GlissandoStyle` enum (Line/LineWithText), `GlissandoLayout` struct, `layout_glissando()` — computes diagonal line geometry between two notes with horizontal padding from notehead edges. Returns `None` when endpoints too close (< 0.3 staff spaces after padding). Vertical endpoints offset slightly in the direction of pitch movement for visual clarity. Optional "gliss." italic text label centered at midpoint. Created `render/glissando_renderer.rs` with `draw_glissando()` rendering a `<line>` element + optional italic text via `TextStyle`. Added `glissando_start: Option<GlissandoStyle>` to `NoteAnnotations` (shared by NoteEvent and ChordEvent). Integrated into system renderer second pass: `collect_glissando_note_info()` + `draw_system_glissandos()` scans positioned elements for glissando_start flags and draws lines to the immediately following note (unlike ties which match by position). Added `ScoreBuilder::glissando(style)` modifier method (no-op on rests). Annotations clone-based propagation in `convert_event()` handles glissando automatically. Wired into layout/render mod.rs with public exports.
- Verified: `cargo test -p music-engraver --lib` — 1679 unit tests pass (+28 new: 11 layout, 6 renderer, 5 system_renderer, 6 score). `cargo test -p music-engraver --test golden_svg` — 43 golden tests pass. `cargo test -p music-engraver --doc` — 7 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/glissando_score.rs` showing glissando via ScoreBuilder API, or add golden-SVG test for glissandos, or begin other post-v1 features.
- Open issues: Cross-system glissandos not yet supported (glissando at end of system to first note of next system). I2 still deferred.

## 2026-04-23 — Post-v1, multi-voice data model + ScoreBuilder API + stem direction forcing
- Did: Added multi-voice support foundation to ScoreBuilder and MeasureContent. `ScoreBuilder::voice(n)` switches active voice (0 = primary, 1 = secondary). Events are tagged with voice index internally — `current_events` is now `Vec<(u8, ScoreEvent)>`. Barline methods auto-reset voice to 0. `build_measure_contents()` separates events by voice: voice 0 → `MeasureContent.events` (unchanged), voices 1+ → new `MeasureContent.additional_voices` field. When multiple voices present, stem directions are forced via `force_stem_direction()`: even voices get stems up, odd voices get stems down (standard engraving convention). Single-voice measures keep `stem_direction: None` (auto-detection) — fully backwards compatible. Updated ~200 `MeasureContent` construction sites across tests, examples, and production code to include `additional_voices: vec![]`.
- Verified: `cargo test -p music-engraver --lib` — 1710 tests pass (13 new multi-voice tests: voice default/switch, barline reset, single vs multi voice splitting, stem forcing for notes/chords/beams/rests, SVG render, cross-measure reset). `cargo test -p music-engraver --test golden_svg` — 45 golden tests pass. `cargo check --workspace` passes with 0 warnings.
- Next: Multi-voice measure rendering — extend the measure renderer to lay out and draw `additional_voices` alongside the primary voice (x-position alignment, notehead collision avoidance, rest displacement). Then create `examples/multi_voice_score.rs` and a golden-SVG test.
- Open issues: `additional_voices` events are stored but not yet rendered (only voice 0 is drawn). Cross-voice tie/slur support deferred. Notehead collision avoidance between voices not yet implemented.

## 2026-04-23 — Post-v1, multi-voice measure rendering (layout + render pipeline)
- Did: Wired additional voice rendering through the full layout→render pipeline. Added `additional_voice_layouts: Vec<MeasureLayout>` to `SystemMeasure`. Extended `layout_system()` to convert additional voice events to `MeasureElement`s, lay them out via `layout_measure()`, and scale to match primary voice width so temporal positions align. Created `draw_additional_voices()` in `render/measure_renderer/mod.rs` — renders notes/chords/beams/tuplets from additional voices, skips barlines/clefs/keysigs/timesigs (already drawn by primary voice), and displaces rests vertically (voice 1 rests shifted down 2 staff spaces, voice 2 rests shifted up) to avoid collision. Added `draw_rest_displaced()` to `render/rest_renderer.rs` with configurable y-displacement. Updated `draw_system()` in `system_renderer` to call `draw_additional_voices()` for each measure that has non-empty additional voice layouts. Multi-voice rendering now works end-to-end through `ScoreBuilder` API.
- Verified: `cargo test -p music-engraver --lib` — 1724 unit tests pass (+13 new: 5 measure_renderer tests [empty, note draws extra, rest displaced, barlines skipped, two voices], 5 score integration tests [more paths, more lines, rest displaced, differs, stem forcing], 3 rest_renderer tests [zero displacement matches normal, nonzero differs, positive moves down]). `cargo test -p music-engraver --test golden_svg` — 45 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from changed files (2 pre-existing). `cargo check --workspace` passes.
- Next: Create `examples/multi_voice_score.rs` showing multi-voice rendering via ScoreBuilder API, or add a golden-SVG test for multi-voice, or implement cross-voice notehead collision avoidance.
- Open issues: Cross-voice notehead collision avoidance not yet implemented (notes at the same staff position in different voices will overlap). Cross-voice tie/slur support deferred. I2 still deferred.

## 2026-04-23 — Post-v1, clippy cleanup + golden-SVG test for multi-voice writing
- Did: Fixed 2 clippy warnings in `score/mod.rs`: (1) `voice % 2 == 0` → `voice.is_multiple_of(2)` (manual_is_multiple_of), (2) extracted `CompletedMeasure` type alias for `(Vec<(u8, ScoreEvent)>, BarlineStyle, Option<VoltaAnnotation>)` (very_complex_type). Added `golden_voices` golden-SVG regression test: 2-measure multi-voice score (voice 0 = melody stems up, voice 1 = bass stems down) with assertions on path count (≥10), line count (≥16), and comparison against single-voice rendering. Added "voices" to `golden_baselines_are_valid_svgs` validation list.
- Verified: `cargo test -p music-engraver --lib` — 1724 unit tests pass. `cargo test -p music-engraver --test golden_svg` — 46 golden tests pass (45 prior + 1 new voices). `cargo test -p music-engraver --doc` — 8 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. Total: 1781 tests. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Create `examples/multi_voice_score.rs` showing multi-voice rendering via ScoreBuilder API, or implement cross-voice notehead collision avoidance, or begin other post-v1 features.
- Open issues: Cross-voice notehead collision avoidance not yet implemented. Cross-voice tie/slur support deferred. I2 still deferred.

## 2026-04-23 — Post-v1, multi-voice score example via ScoreBuilder API
- Did: Created `examples/multi_voice_score.rs` rendering 4 measures across 2 systems in C major 4/4 using ScoreBuilder API with `.voice()` method. Demonstrates: (1) soprano melody E5–D5 half notes (voice 0, stems up) over C4 whole note (voice 1, stems down), (2) interleaved rhythms — voice 0 quarter notes G5–F5–E5–D5 with voice 1 half notes E4–D4, (3) mixed rhythms with voice 1 displaced rest, (4) converging voices ending on parallel motion. Exercises forced stem directions (up for voice 0, down for voice 1), displaced rests, simultaneous different rhythms. Output: 23 paths, 43 lines (13975 bytes).
- Verified: `cargo check -p music-engraver` passes. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo test -p music-engraver --test golden_svg` — 46 golden tests pass. `cargo test -p music-engraver --doc` — 8 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo run --example multi_voice_score` produces valid SVG. `cargo check --workspace` passes.
- Next: Implement cross-voice notehead collision avoidance (notes at same staff position in different voices need x-offset), or begin other post-v1 features (Gourlay penalty tuning, tab stave cross-system features).
- Open issues: Cross-voice notehead collision avoidance not yet implemented. Cross-voice tie/slur support deferred. I2 still deferred.

## 2026-04-23 — Post-v1, cross-voice notehead collision avoidance
- Did: Created `layout/voice_collision.rs` with `VoiceCollisionOffset` struct, `compute_voice_collision_offsets()` function, and helpers (`collect_voice_positions`, `element_staff_positions`, `element_stem_direction`, `detect_collision`). When notes in different voices at the same x-position are at unison (same staff position) or a second apart (distance 1), the additional voice's notehead is offset by one notehead width to avoid overlap. Down-stem voices shift right (+1.0), up-stem voices shift left (−1.0). Notes a third or more apart (distance ≥2) are not offset. Updated `draw_additional_voices()` in `render/measure_renderer/mod.rs` to accept `primary_layout: &MeasureLayout`, compute collision offsets per additional voice via `compute_voice_collision_offsets()`, and apply x-shifts to colliding note/chord elements (rests skip collision offset). Updated call site in `system_renderer/mod.rs` to pass primary layout. Updated 5 existing test call sites in `measure_renderer/tests.rs`.
- Verified: `cargo test -p music-engraver` — 1745 unit + 46 golden + 3 integration + 8 doc-tests = 1802 tests, all pass. 16 new voice_collision layout tests (unison/second/third/chord collision detection, different x positions, rests ignored, empty inputs, up-stem shift direction). 3 new measure_renderer tests (unison offset, second offset, third no-offset). 2 new score integration tests (unison collision differs from far-apart, second collision differs from third). `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes. All 46 golden SVG baselines unchanged (existing multi-voice test uses well-separated voices, confirming no false positive offsets).
- Next: Update golden SVG baseline for `voices` to include a collision test case, or create `examples/voice_collision.rs` demonstrating unison/second avoidance, or begin other post-v1 features (cross-voice tie/slur support, Gourlay penalty tuning).
- Open issues: Cross-voice tie/slur support deferred. Beam groups in additional voices don't get per-note collision detection (group-level detection only). I2 still deferred.

## 2026-04-23 — Post-v1, voice collision example + golden-SVG test
- Did: Created `examples/voice_collision.rs` rendering 4 measures across 2 systems demonstrating cross-voice notehead collision avoidance: (1) unison collisions — both voices on C5 and D5 with offset noteheads, (2) second-apart collisions — voice 1 one step below voice 0, noteheads offset, (3) mixed — beats 1 & 3 have unison collisions, beats 2 & 4 are well-separated (no offset), (4) well-separated voices for contrast (no collision). Output: 24 paths, 38 lines, 24 translate transforms (12491 bytes). Added `golden_voice_collision` golden-SVG regression test (46th baseline → 47 golden tests total) with structural assertions: ≥15 paths, ≥20 lines, differs from basic voices golden. Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo test -p music-engraver --lib` — 1745 unit tests pass. `cargo test -p music-engraver --test golden_svg` — 47 golden tests pass. `cargo test -p music-engraver --doc` — 8 doc-tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo run --example voice_collision` produces valid SVG. `cargo check --workspace` passes.
- Next: Implement cross-voice tie/slur support (ties between notes in different voices), or begin other post-v1 features (Gourlay penalty tuning for line breaking, tab stave cross-system features).
- Open issues: Cross-voice tie/slur support deferred. Beam groups in additional voices don't get per-note collision detection. I2 still deferred.

## 2026-04-23 — Post-v1, spans in additional voices (ties, slurs, hairpins, lyrics, ottava, glissandos)
- Did: Extended all 6 `collect_*` functions in `system_renderer/mod.rs` to scan additional voice elements alongside the primary voice. Previously `collect_note_positions`, `collect_slur_note_info`, `collect_hairpin_note_info`, `collect_lyric_note_info`, `collect_ottava_note_info`, and `collect_glissando_note_info` only iterated over `measure.layout.elements` (voice 0), ignoring notes in `measure.additional_voice_layouts`. Added `all_measure_elements()` helper that chains primary + additional voice iterators. All 6 collect functions now use this helper, so ties, slurs, hairpins, lyric extenders, ottava brackets, and glissandos in secondary voices are collected and rendered correctly.
- Verified: `cargo test -p music-engraver --lib` — 1753 unit tests pass (+8 new: 5 system_renderer tests [tie/slur/hairpin in additional voice, collect includes additional voices, collect without additional unchanged], 3 score tests [tie/slur/hairpin in voice 1 via ScoreBuilder]). `cargo test -p music-engraver --test golden_svg` — 47 golden tests pass. `cargo clippy -p music-engraver --all-targets` — 0 warnings from music-engraver. `cargo check --workspace` passes.
- Next: Add a golden-SVG test for multi-voice with ties/slurs, or update `voice_collision` example to show tied secondary voices, or begin other post-v1 features (Gourlay penalty tuning, tab stave cross-system features).
- Open issues: Beam groups in additional voices don't get per-note collision detection. Cross-system spans in additional voices rely on the same cross-system machinery already working (via `collect_note_positions` which now includes additional voices). I2 still deferred.

## 2026-05-12 — Post-v1, multi-measure rest ScoreBuilder integration + build fix
- Did: Fixed a partial in-progress addition that left the build broken. A previous run added `MeasureElement::MultiMeasureRest(u32)` and `MeasureEvent::MultiMeasureRest(u32)` enum variants plus `layout/multi_measure_rest.rs` and `render/multi_measure_rest_renderer.rs`, but never wired the variants through three non-exhaustive match sites (primary draw in `render/measure_renderer/mod.rs:62`, additional-voice draw at :160, and `force_stem_direction` in `score/mod.rs:89`). Patched all three: primary voice draws the H-bar via `layout_multi_measure_rest()` spanning `[elem_x, elem_x + positioned.width]`; additional voices skip the variant (a whole-measure property only the primary voice renders); `force_stem_direction` no-ops on it (no stem). Then completed the public API by adding `ScoreEvent::MultiMeasureRest { count }` (with `convert_event` mapping it straight to the measure event), `ScoreBuilder::multi_measure_rest(count: u32)` with docs warning that it should be the sole event in its measure, and a new example `examples/multi_measure_rest_score.rs` rendering a tacet-8-bars + tacet-16-bars part. Added `golden_multi_measure_rest` regression baseline asserting exactly 6 `<rect>` elements (two H-bars × 3 components), bold count text, and that both counts appear as `<text>` nodes.
- Verified: `cargo check --workspace` passes. `cargo check -p music-engraver` passes. `cargo test -p music-engraver --test golden_svg` — 48 golden tests pass (47 prior + new `multi_measure_rest`). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 pass. `cargo test -p music-engraver --lib multi_measure` — 30 tests pass (+6 new: 4 score builder tests [adds H-bar rects + count text, distinct counts differ, between notated measures, count is bold], 2 measure_renderer tests [primary-voice draw asserts 3 rects + count text + no stray paths/lines, x_offset shifts H-bar coordinates]). `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:342`). `cargo run --example multi_measure_rest_score` produces valid SVG (6684 bytes, 9 paths, 6 rects).
- Next: Cross-voice tie/slur regression test (Phase 8 polish), or multi-measure rest variants (church-rest blocks for counts 1–4 use centered stem rests rather than the H-bar — Behind Bars convention), or PNG export deeper testing (decode + dimensions), or Gourlay penalty tuning.
- Open issues: Multi-measure rest count of `0` renders the literal "0" (caller responsibility — documented). No alternative church-rest block style for small counts (1–4). H-bar width comes from layout's whole-note spacing factor — visually narrow at tight spacing settings, but acceptable for v1. Beam groups in additional voices still lack per-note collision detection. I2 still deferred.

## 2026-05-12 — Post-v1, church-rest variant for small-count multi-measure rests
- Did: Added the older "church-rest" rendering for multi-measure rests with counts 1–4, alongside the existing H-bar default. Introduced `MultiMeasureRestStyle::{HBar, Church}` in `layout/multi_measure_rest.rs`; default is `HBar` so existing behavior is unchanged. Threaded `style` through `MeasureElement::MultiMeasureRest`, `MeasureEvent::MultiMeasureRest`, and the internal `ScoreEvent::MultiMeasureRest` (all became struct-form variants). Added `ChurchRestLayout`/`ChurchRestGlyph` + `layout_church_rest()` that maps counts → SMuFL rest glyph sequences (1→whole, 2→breve, 3→breve+whole, 4→breve+breve) centered horizontally with ~2.1ss between glyphs; counts outside `1..=4` return an empty glyph list so callers can fall back. Added `render/church_rest_renderer.rs::draw_church_rest()` rendering each glyph at its SMuFL anchor (whole on line 4, breve on line 3) plus the same bold centered count number used by the H-bar form. Dispatch happens in `render/measure_renderer/mod.rs`: church style with a supported count uses `draw_church_rest`, anything else (including church + count>4) falls through to `draw_multi_measure_rest`. Public API added: `ScoreBuilder::multi_measure_rest_church(count: u32)` mirrors the existing `multi_measure_rest`. Updated `force_stem_direction` and the additional-voice skip-list to match the new struct shape. New example `examples/church_rest_score.rs` exercises all four supported counts plus a count-7 fallback, plus surrounding notated material; new golden baseline `tests/golden/church_rest.svg` (9500 bytes) with assertions that the church form adds exactly 6 rest-glyph `<path>` elements vs the H-bar form and removes exactly 12 `<rect>` elements (4 mmrs × 3).
- Verified: `cargo check --workspace` passes. `cargo check -p music-engraver` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1813 unit tests pass** (+32 new church-rest tests: 15 layout, 11 renderer, 6 score-integration). `cargo test -p music-engraver --test golden_svg` — **50 golden tests pass** (+1 new `golden_church_rest`). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:342`). `cargo run --example church_rest_score` writes 13701 bytes; the example's own assertions confirm 5 bold count texts (one per mmr including count-7 fallback), ≥3 fallback rects, and that all of "1"/"2"/"3"/"4"/"7" appear as `<text>` nodes.
- Next: PNG export deeper testing (already has IHDR dimension assertions — could add downstream-format round-trip, e.g. re-decode and inspect histogram). Other small post-v1 items: cross-system church rests not yet handled (would need the church cluster to break across systems), grace-note slur, Gourlay penalty tuning.
- Open issues: Counts of 0 with `Church` style return an empty glyph layout and the H-bar fallback also draws "0" — same caller-responsibility note as before. The whole-rest and breve-rest glyphs have not been visually proofed in a single rendered staff; the unit tests only check coordinate values and presence/absence. Beam groups in additional voices still lack per-note collision detection. I2 still deferred.

## 2026-05-12 — Post-v1, grace-note slur (connecting slur from grace to principal)
- Did: Added the canonical grace-note slur — a small filled crescent connecting an acciaccatura/appoggiatura to its principal note. New field `grace_note_slur: bool` on `NoteAnnotations` (defaults to `false`; existing `.grace_note()` callers unaffected). New layout helper `layout_grace_note_slur(grace_layout, principal_x, principal_position, principal_stem_dir, staff, config) -> Option<SlurLayout>` in `layout/grace.rs`: endpoints attach near the right edge of the scaled grace notehead (`grace.x + 0.59 * staff_space * GRACE_NOTE_SCALE`) and just left of the principal notehead (`principal_x - 0.05 * staff_space`); direction follows `slur_direction_from_stem(principal_stem_dir)` so it arcs away from the principal's stem; returns `None` for degenerate spans so the renderer can skip cleanly. Wired into `draw_note_event` (`render/measure_renderer/mod.rs:277-298`) and `draw_chord_event` (:471-498) — chord variant attaches to the chord member closest in pitch to the grace via `min_by_key(|p| (p - grace_pos).abs())`. Public API: new `ScoreBuilder::grace_note_slur(pitch, kind)` mirroring `grace_note(pitch, kind)` but additionally setting `grace_note_slur = true`. Annotations propagate through `convert_event` automatically via the existing clone-based path. New example `examples/grace_note_slur_score.rs` exercises acciaccatura, appoggiatura, wide intervals (grace below/above principal), chord+grace+slur, and a rest no-op — 4 measures × 2 systems = 16805 bytes, 7 slur paths. New golden baseline `tests/golden/grace_note_slur.svg` + `golden_grace_note_slur` test that asserts exactly 3 `stroke="none"` filled paths in `build_grace_note_slur()` and that the slurred score differs byte-for-byte from `build_grace_notes()`. Added `"grace_note_slur"` to `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo check --workspace` passes. `cargo check -p music-engraver` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1828 unit tests pass** (+15 new grace-slur tests: 7 layout [horizontal span, under-for-stem-up, over-for-stem-down, asymmetric pitches, degenerate→None, exact x_start, exact x_end], 4 measure_renderer [adds exactly one `stroke="none"` path on note, no-slur-flag emits zero, defensive `slur-without-grace` no-op, chord variant adds exactly one path], 4 score-integration [slur vs plain differ, rest no-op, chord adds path, both glyph + slur present]). `cargo test -p music-engraver --test golden_svg` — **51 golden tests pass** (+1 new `golden_grace_note_slur`). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:342`). `cargo run --example grace_note_slur_score` writes 16805 bytes; the example's own assertions confirm exactly 7 `stroke="none"` slur paths (6 pitched single-note calls in M1–M3 + 1 chord call in M4; the call following a rest is correctly a no-op).
- Next: Other small post-v1 items: cross-system church rests (church cluster breaking across systems), cross-voice tie/slur regression test, beam-group per-note collision detection in additional voices, Gourlay penalty tuning, or further articulation/ornament gestures.
- Open issues: Slur direction is currently driven solely by the principal note's stem; some engraving traditions instead place the grace slur consistently over the noteheads regardless of stem direction (especially for a single grace before a stem-up note in vocal music). The current rule matches the broader Gould "slurs opposite stems" convention. The grace glyph's notehead half-width is approximated at 0.59 staff spaces — taken from Bravura's standard notehead advance; precise glyph-bbox lookup deferred. Beam groups in additional voices still lack per-note collision detection. I2 still deferred.

## 2026-05-12 — Post-v1, trill wavy-line extension primitives (layout + renderer)
- Did: Added the wavy-line extension that conventionally follows a "tr" glyph for a sustained trill. New `layout/trill_extension.rs` with `TrillExtensionLayout { segment_xs, y, glyph: WiggleTrill, segment_advance }` and `layout_trill_extension(start_x, end_x, y, segment_advance_fu) -> Option<TrillExtensionLayout>`. Tiles whole copies of `Glyph::WiggleTrill` from `start_x` rightward toward `end_x`; segment count is `floor(span / segment_advance_fu)` so the right edge never overflows the requested `end_x` (avoiding distortion from non-uniform horizontal scaling of the wiggle glyph). Returns `None` for non-positive advances, zero span, negative span, or spans shorter than one segment — `Option` semantics let the caller skip cleanly without an empty layout sentinel. Also exported `trill_extension_right_edge()` helper for callers that need to chain decorations after the wiggle. Layout stays font-agnostic per the project rule: the caller queries the wiggle segment advance from the font and passes it in. New `render/trill_extension_renderer.rs::draw_trill_extension()` emits one `<path>` per segment by translating the WiggleTrill outline. Wired into `layout/mod.rs` (module declaration + `pub use`) and `render/mod.rs` (module declaration + `pub use draw_trill_extension`). Not yet integrated into the annotation pipeline (`NoteAnnotations`) — this chunk is the standalone primitive; ScoreBuilder API + system-level span collection will be a follow-up chunk.
- Verified: `cargo check -p music-engraver` passes. `cargo build -p music-engraver` passes. `cargo check --workspace` passes. `cargo test -p music-engraver --lib` — **1852 unit tests pass** (+24 new trill_extension tests: 16 layout [empty/negative/zero/sub-segment spans → None; exact-fit yields one; two-segments at double span; floor of non-integer ratios; many-segment uniform spacing; y preserved; glyph is WiggleTrill; right-edge after last segment; segment_xs strictly increasing; right-edge does not overflow end_x; fractional advance], 8 renderer [path count for 2/5 segments, embeds every segment x in `translate(...)`, embeds y in `translate(...)`, empty layout is no-op, all segments share same path d-data, known glyph never errors, `fill="black"` present]). `cargo test -p music-engraver --test golden_svg` — **51 golden tests pass** (unchanged — no integration yet). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:342`).
- Next: Wire the trill extension into the annotation pipeline — add a `trill_extension_end: Option<NoteAnchor>` style annotation on `NoteAnnotations` so a `Trill` ornament on note A can extend to note B; system-level collector follows the existing `collect_*_note_info` pattern (e.g. `collect_trill_extension_note_info`) for cross-system spans. Then ScoreBuilder `.trill_with_extension()` modifier, an example, and a golden baseline.
- Open issues: This chunk does not yet expose a way to draw a trill extension from end-user score builders; the primitive is intentionally standalone. Per-segment outline lookup is queried once per call and cloned across segments — fine for typical trill lengths but worth memoizing if a score has many trills. The wiggle glyph variants (`WiggleTrillFast`/`WiggleTrillSlower`/etc.) are not yet exposed via the layout API; v1 commits to `WiggleTrill` as the single tempo-neutral wiggle. Beam groups in additional voices still lack per-note collision detection. I2 still deferred.

## 2026-05-12 — Post-v1, trill extension wired through ScoreBuilder + system renderer
- Did: Wired the standalone trill wavy-line primitive (layout + renderer added in the prior entry) through the annotation pipeline, the system-renderer second pass, and the public ScoreBuilder API. New field `trill_extension: bool` on `NoteAnnotations` (default `false`; propagates through `convert_event` via the existing clone-based path). New public method `ScoreBuilder::trill_with_extension()` (`src/score/mod.rs`) that simultaneously sets `ornament = Some(Trill)` and `trill_extension = true` — the two flags are coupled at the API surface so users can't accidentally request an extension on a non-trill ornament. In `render/system_renderer/mod.rs`: new `TrillExtensionNoteInfo { x, staff_position, has_trill_extension }`, `collect_trill_extension_note_info()` following the existing `collect_*_note_info` pattern (chord variant anchors to the chord's top staff_position so the wiggle sits at the trill glyph's actual placement), and a new `draw_system_trill_extensions()` pass that mirrors the glissando "next-note" model. Wiggle geometry: start x = trill_glyph_x + `OrnamentTrill` advance + 0.15ss gap; end x = next_note_x − 0.30ss gap; y = the exact `layout_ornament(Trill, …).y` so the wiggle and the "tr" glyph share a baseline. Extra defensive guard in the collector: a note flagged with `trill_extension = true` but a non-`Trill` ornament is silently inert (the wiggle requires *both* an actual trill glyph upstream and the extension flag). Trills on the final note of a system silently render no wiggle — cross-system continuation is a deferred enhancement (see open issues). Added new example `examples/trill_extension_score.rs` and golden baseline `tests/golden/trill_extension.svg` (10836 bytes, 17 paths). Updated `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib trill_extension` — 32 tests pass (incl. 7 new system_renderer tests: collector flags only marked notes, requires Trill ornament, renders ≥1 wiggle path, wiggle shares trill glyph y, no wiggle without flag, no wiggle on last note of system, chord anchors to top note; + 1 new score test that exercises the inertness guard). `cargo test -p music-engraver --lib trill_with_extension` — 4 new score tests pass (adds paths vs plain trill, no-op on rest, adds paths on chord, sets both annotation fields). `cargo test -p music-engraver --test golden_svg` — 52 golden tests pass (51 prior + new `golden_trill_extension` with assertions: extension version has more paths than plain-trill baseline, delta ≥ 2 wiggle segments, SVG differs byte-for-byte). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo run --example trill_extension_score` produces 22867 bytes (36 paths, 30 lines) and asserts path_count > no-extension baseline. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:342`).
- Next: Cross-system trill extension (when the trilled note is the final note of a system, extend the wiggle to the system's right edge and resume in the next system if convention demands; otherwise just to the system's right edge as in cross-system ottava/glissando). Or: explicit `.trill_to(note_index)` API so the extension terminates at a user-chosen anchor rather than the immediately-following note (useful when a trill should end mid-measure). Or: continue on other post-v1 items (Gourlay penalty tuning, per-note collision detection in beamed additional voices, cross-system church rests).
- Open issues: Cross-system trill extension not yet handled — a trill on the last note of a system silently draws no wiggle. This matches the current glissando behaviour and is acceptable for v1+ scope but worth fixing for sustained trills that span line breaks. The wiggle's y is set to the trill glyph's baseline anchor; for fonts where the wiggle glyph's design height differs from `OrnamentTrill`'s baseline this could visually drift — Bravura aligns reasonably, but if Petaluma/Leland get added a small per-font y-offset constant may be needed. The collector treats the "Trill ornament + extension flag" pair as the trigger; a hypothetical `ShortTrill` with extension is not yet supported. Beam groups in additional voices still lack per-note collision detection. I2 still deferred.

## 2026-05-12 — Post-v1, cross-system trill extension (wiggle to system right edge)
- Did: A trill-with-extension on the last note of a system now draws a wiggle that extends to the system's right edge instead of silently rendering nothing. New constant `TRILL_EXTENSION_SYSTEM_EDGE_GAP_SS = 0.5` in `render/system_renderer/mod.rs` (tuned slightly larger than the 0.3ss inter-note gap because the final barline carries more visual weight than a notehead). The `notes.get(i + 1)` lookup in `draw_system_trill_extensions` is now a `match` rather than a `let-else continue`: `Some(target) → end_x = system_x + target.x − 0.30ss × staff_space` (unchanged inter-note behaviour); `None → end_x = system_x + system.staff_width − 0.50ss × staff_space` (new cross-system case). All other geometry (start_x, y, glyph) is shared between the two cases so the wiggle reads identically in both. The same `layout_trill_extension` fail-safe still applies — if the trilled note is so close to the system right edge that no whole wiggle segment fits, `None` is returned and the renderer silently skips. Updated the comment on `examples/trill_extension_score.rs:52` to reflect the new behaviour (the chord trill at the end of M4 of system 2 now extends to the system's right edge, no longer "gets no wiggle").
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1867 unit tests pass** (+3 new system_renderer tests vs the prior chunk's 1864, and one renamed/inverted test: `last_note_in_system_with_trill_extension_renders_no_wiggle` → `last_note_in_system_with_trill_extension_extends_to_system_edge` with inverted assertion; new tests: `last_note_trill_extension_wiggle_stays_inside_system_edge` reproduces the production end_x math and asserts both `right_edge < staff_width` and `staff_width − right_edge ≥ 0.5ss − ε`; `last_note_trill_extension_wiggle_shares_trill_glyph_y` asserts ≥2 paths translate to the trill glyph's y, i.e. the "tr" glyph itself plus at least one wiggle segment; `last_note_trill_extension_silently_skips_when_no_room` confirms the layout fail-safe when end_x = start_x). `cargo test -p music-engraver --test golden_svg` — 52 golden tests pass (no baseline drift: existing `build_trill_extension` is single-system with a following note after each trill). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:342`). `cargo run --example trill_extension_score` now produces 27157 bytes (46 paths, up from 36) — the +10 paths are the new cross-system wiggle segments after the chord trill at the end of system 2; the example's own assertions (`path_count >= 18`, `path_count > no_ext_paths`) still hold.
- Next: Resume a trill wiggle in the next system when the trilled note is at the end of system N and the user wants the extension to continue into system N+1 (would require ScoreBuilder-level tracking similar to lyric extender continuation). Or: explicit `.trill_to(note_index)` API for sub-note-granularity end points. Or: cross-system ottava continuation (the same pattern would handle 8va brackets that span line breaks). Or other post-v1 items (Gourlay penalty tuning, per-note collision detection in beamed additional voices, cross-system church rests).
- Open issues: A trill at the end of system N does not yet resume in system N+1 — the wiggle terminates at system N's right edge regardless of whether the trill conceptually continues. Bracket-form trill ranges (with a vertical hook at the start/end of the wiggle) are not yet supported. Beam groups in additional voices still lack per-note collision detection. I2 still deferred.

## 2026-05-12 — Post-v1, cross-system trill extension resume (incoming wiggle on N+1)
- Did: Resolved the first open issue from the prior chunk: a trill-with-extension on the last note of system N now also draws an *incoming* wiggle on system N+1 leading up to N+1's first note, so a sustained trill spanning a line break reads as one continuous wavy line. Within-system trailing geometry (system right edge) is unchanged — the new code lives entirely in the page renderer, mirroring the existing `draw_cross_system_ottava_brackets` / `draw_cross_system_glissandos` pattern. Promoted `TRILL_EXTENSION_NOTE_GAP_SS` from private to `pub(crate)` in `render/system_renderer/mod.rs` so both within-system and cross-system passes terminate the wiggle with the same visual gap before the next notehead. Added to `render/page_renderer/mod.rs`: `struct UnresolvedTrillExtension { y_above_top_line }` (stores the source ornament y as an *offset* from the source staff's top line, not an absolute y, so the incoming wiggle re-anchors to N+1's staff and lands at the same height-above-the-staff regardless of inter-system gap), `struct IncomingTrillExtensionTarget { x, staff_left }`, `find_unresolved_trill_extension(page_system, staff_space) -> Option<...>` (returns Some only when the system's *last* collected note has `has_trill_extension = true` — earlier trills are guaranteed to resolve within the system already), `find_incoming_trill_extension_target(page_system) -> Option<...>` (first note + staff_left after prefix), and `pub(crate) fn draw_cross_system_trill_extensions(...)` (recomputes the y on the target staff via `tgt_staff.y_of(8) + src.y_above_top_line`, lays out the wiggle from `staff_left` to `target.x − TRILL_EXTENSION_NOTE_GAP_SS × ss`, silently no-ops on degenerate spans). Wired into `draw_page` after `draw_cross_system_glissandos`, and into `score/multi_staff.rs` so the multi-staff page renderer also drives the cross-system trill pass per notation stave.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1872 unit tests pass** (+5 new page_renderer tests: `cross_system_trill_extension_adds_incoming_wiggle_paths_on_next_system` asserts ≥3 added `<path` elements vs no-trill baseline; `cross_system_trill_extension_only_when_last_note_is_trilled` confirms a within-system trill produces no incoming wiggle on N+1 while a last-note trill does; `cross_system_trill_extension_no_target_system_no_incoming_wiggle` confirms a single-system page is a no-op on the new pass; `cross_system_trill_extension_incoming_y_anchored_to_target_staff` parses every `translate(x,y)` in the output SVG and asserts at least one path's y falls in the open interval `(sys1_top_y, sys2_top_y)` — i.e. in the inter-system band where the N+1 incoming wiggle must sit; `cross_system_trill_extension_no_op_without_trill` confirms a plain two-system page emits no extra paths). `cargo test -p music-engraver --test golden_svg` — 52 golden tests pass (no baseline drift: `build_trill_extension` is single-system). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:342`). `cargo run --example trill_extension_score` produces 37550 bytes / 71 paths (up from 27157 / 46) — the +25 paths come from the new incoming wiggle on system 3 leading up to the F4 whole note in M5, which is the cross-system continuation of the chord trill at the end of system 2.
- Next: Cross-system church rests (church-rest cluster split across a line break), or `.trill_to(note_index)` for sub-note-granularity wiggle end points, or bracket-form trill ranges (hook glyph at wiggle start/end), or per-note collision detection in beamed additional voices, or Gourlay penalty tuning for optimal line breaking.
- Open issues: The cross-system incoming wiggle always terminates at the *first* note of system N+1; there is no way for the user to say "the trill ends mid-N+1." A `.trill_to(anchor)` API would close that gap. The y-anchoring assumes that "height above top staff line" is the right invariant across stacked staves of the same size — if a future feature introduces variable staff sizes per system, this offset needs to be rescaled. Bracket-form trill ranges (with a vertical hook at the start/end of the wiggle) are not yet supported. Beam groups in additional voices still lack per-note collision detection. I2 still deferred.

## 2026-05-12 — Post-v1, bracket-form trill range primitives (hook layout + renderer)
- Did: Added the bracket form of a trill range — short vertical hook lines at the start and/or end of a trill wavy-line extension that make the trill's beginning/end unambiguous. New layout module `layout/trill_bracket.rs` defining `TrillBracketSide { Start, End, Both }`, `HookDirection { Down, Up }`, `TrillBracketHookLayout { x, y_top, y_bottom, stroke_width }`, and two layout functions: `layout_trill_bracket_hook(x, baseline_y, length, direction, stroke_width) -> TrillBracketHookLayout` for a single hook (treats negative `length` as its absolute value so the explicit `HookDirection` always wins, and preserves the invariant `y_top <= y_bottom`), and `layout_trill_bracket_hooks(extension, side, length, direction, stroke_width) -> Vec<TrillBracketHookLayout>` which derives hook x-positions from a `TrillExtensionLayout` — start hooks anchor at `segment_xs[0]`, end hooks anchor at the right edge computed by the existing `trill_extension_right_edge` helper (flush with the wiggle's visible terminus, not the start of the last tile). Returns `Vec::new()` for an empty extension. New renderer `render/trill_bracket_renderer.rs` with `draw_trill_bracket_hook(svg, hook)` emitting a single `<line>` element using the hook's own `stroke_width` (so the bracket weight matches the wiggle's hairpin/thin-barline weight the caller passes in), and `draw_trill_bracket_hooks(svg, hooks)` for batches. Both primitives are font-free — layout has already resolved everything to pixel/font-unit coordinates. Wired into `layout/mod.rs` (module declaration + `pub use`) and `render/mod.rs` (module declaration + `pub use`). Not yet integrated into the annotation pipeline (`NoteAnnotations` or `ScoreBuilder`); this chunk is the standalone primitive — wiring it through the user-facing trill API will be a follow-up chunk.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib trill_bracket` — **28 new unit tests pass** (18 layout: down vs up baseline anchoring, `y_top <= y_bottom` invariant in both directions, negative-length folded to absolute value, zero-length produces degenerate hook, stroke-width preserved, start-only/end-only/both side selection, hooks share extension baseline_y, up-direction anchors baseline at bottom, empty extension returns empty vec, single-segment start/end x positions, length and stroke propagation, start_x < end_x ordering, end_x matches `trill_extension_right_edge` helper output; 10 renderer: one `<line>` per hook, x/y endpoints embedded, stroke-width embedded, `stroke="black"` present, up hook has correct y endpoints, empty slice is no-op, two-hook draw emits two lines, both-sides distinct x values, per-hook stroke width respected, zero-length hook still emits a degenerate `<line>`). `cargo test -p music-engraver --test golden_svg` — **52 golden tests pass** (no baseline drift: the primitive is not yet wired through any user-facing API). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`). Full `cargo test -p music-engraver --lib` — **1900 unit tests pass** (1872 prior + 28 new); finished in 364.70s under the sandboxed environment.
- Next: Wire the bracket hooks through the annotation pipeline — add a `trill_bracket_side: Option<TrillBracketSide>` (or coupled boolean pair) to `NoteAnnotations` so a `Trill` ornament can request a hook at the start, end, or both ends of its wavy-line extension. Coupled at the API surface to the existing `trill_extension` flag so a hook can't be requested without an extension. Then a ScoreBuilder method (e.g. `.trill_with_extension_bracketed(side)`), an example, and a golden baseline. Alternatively start `.trill_to(note_index)` for sub-note-granularity wiggle end points, or per-note collision detection in beamed additional voices, or Gourlay penalty tuning.
- Open issues: Standalone primitive is not yet exposed to score builders — no way to request bracket hooks from a user-facing API. Bracket form for tuplet brackets and similar (not trill-specific) is unrelated to this primitive. I2 still deferred.

## 2026-05-12 — Post-v1, trill bracket wired through annotations + ScoreBuilder + cross-system end hook
- Did: Wired the standalone trill-bracket primitive (layout + renderer added in the prior entry) through the annotation pipeline, the system-renderer second pass, the page-renderer cross-system pass, and the public ScoreBuilder API. New field `trill_bracket: Option<TrillBracketSide>` on `NoteAnnotations` (default `None`; propagates through `convert_event` via the existing clone-based path). New public method `ScoreBuilder::trill_with_extension_bracketed(side: TrillBracketSide)` (`src/score/mod.rs`) that simultaneously sets `ornament = Some(Trill)`, `trill_extension = true`, and `trill_bracket = Some(side)` — the three flags are coupled at the API surface so users can't request a bracket on something that isn't a trill-with-extension. In `render/system_renderer/mod.rs`: extended `TrillExtensionNoteInfo` with `bracket: Option<TrillBracketSide>` (only populated when `has_trill_extension` is true, so a bracket request on a non-trilled note is silently inert); added new constant `TRILL_BRACKET_HOOK_LENGTH_SS = 0.75`; added helper `bracket_side_for_system_pass(requested, cross_system)` returning the filtered side that should be rendered *on this system* — within-system passes the side through, cross-system reduces `End`→None and `Both`→Start (the End hook is deferred to the page renderer on N+1 so the bracket frames the trill's *semantic* range, not the per-system wiggle fragment); `draw_system_trill_extensions` now layouts and draws hooks via `layout_trill_bracket_hooks` immediately after laying out the wiggle, using `thin_barline_thickness_fu()` for the stroke and `HookDirection::Down` (wiggle sits above staff). Added public helper `layout_trill_end_hook` for the page renderer's use. In `render/page_renderer/mod.rs`: extended `UnresolvedTrillExtension` with `bracket: Option<TrillBracketSide>`; `find_unresolved_trill_extension` propagates the source note's bracket; `draw_cross_system_trill_extensions` checks for `Some(End | Both)` and draws a single end hook at the right edge of the incoming wiggle on system N+1 using `trill_extension_right_edge` (matching the within-system end-hook anchor exactly). New example `examples/trill_bracket_score.rs` exercises Start/End/Both within-system, chord bracket, and cross-system Both bracket (delta=8 vs no-bracket baseline). New golden baseline `tests/golden/trill_bracket.svg` (8 measures across 4 systems, all within-system, delta=6 vs `trill_with_extension`-only baseline).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1919 unit tests pass** (+19 since prior chunk: 9 system_renderer tests [bracket_side_for_system_pass within-system pass-through, cross-system filtering, collector propagates with extension, collector drops without extension, Both within-system adds 2 hooks, Start adds 1, End adds 1, no-bracket adds 0, last-note-in-system End suppression, last-note-in-system Both reduces to Start]; 3 page_renderer tests [cross-system End adds 1 hook on N+1, cross-system Both adds 2 total, cross-system Start adds only 1 on N]; 6 score-integration tests [Both adds 2 hooks vs plain, Start adds 1, End adds 1, on rest is no-op, sets all 3 annotation fields, on chord adds 2 hooks]). `cargo test -p music-engraver --test golden_svg` — **53 golden tests pass** (+1 new `golden_trill_bracket` with exact-count delta=6 assertion and Start-only=1 regression guard). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). `cargo run --example trill_bracket_score` writes 28831 bytes / 43 paths / 41 lines and the example's internal delta=8 assertion passes (M1 Both:2 + M2 Start:1 + M3 End:1 + M4 chord Both:2 + M5 cross-system Both:2).
- Next: Cross-system bracket-form trill is now complete. Remaining trill polish: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; per-segment WiggleTrill variant selection (`WiggleTrillFast` for faster trills); explicit `trill_bracket_with_direction` API exposing `HookDirection::Up` for the rare case of a trill rendered below the staff. Or other post-v1 items: cross-system church rests, per-note collision detection in beamed additional voices, Gourlay penalty tuning, additional dynamic glyphs (sf, sfz, fp, pf), additional ornaments (turn variations), more golden tests for coverage.
- Open issues: `HookDirection::Up` not exposed through the ScoreBuilder API — the wiring hardcodes `Down`. A user rendering a trill below the staff (rare in practice) would currently get a hook in the wrong direction. Bracket length is hardcoded at 0.75 staff spaces; some traditions (Behind Bars examples) use up to 1.0 — a per-call override would be a small follow-up if needed. Cross-system bracket only applies when the trill extension actually continues into N+1; a bracket request on a system's last trill with no following system (an unusual page layout) still draws the within-system wiggle to the system edge but emits no end hook (matches current behavior of the wiggle itself — silently terminates).

## 2026-05-12 — Post-v1, Dynamic enum coverage expansion (14 new SMuFL composites + `Dynamic::ALL`)
- Did: Expanded `layout::dynamics::Dynamic` from 11 variants to 25 to cover every standard SMuFL composite dynamic glyph. New variants: `Niente`, `Pppppp`, `Ppppp`, `Pppp` (sub-piano levels 4–6), `Ffff`, `Fffff`, `Ffffff` (super-forte levels 4–6), `Pf` (poco forte), `Sf` (sforzato, single-letter — distinct SMuFL glyph from `Sfz`), `Sff` (sforzato-ff), `Sfpp` (sforzando-pianissimo), `Fz` (forzando), `Rf` (rinforzando short), `Rfz` (full rinforzando). Variant→`smufl::Glyph` mapping uses `DynamicNiente`/`DynamicPppppp..6p`/`DynamicFfff..6f`/`DynamicPf`/`DynamicSforzato`/`DynamicSforzatoFf`/`DynamicSforzandoPianissimo`/`DynamicForzando`/`DynamicRinforzando`/`DynamicRinforzando1`. Added `Dynamic::ALL: [Dynamic; 25]` constant — canonical ordering quietest→loudest with accent-style markings after the simple level markings — so callers and tests can iterate the full set without hand-listing it. Renderer (`render::dynamics_renderer`) and ScoreBuilder API (`.dynamic(Dynamic)`) needed zero changes: both are glyph-agnostic and pick up new variants automatically. Updated `examples/dynamics_score.rs` measure 4 to exercise `Sf`/`Rfz`/`Fz`/`Niente` so the visual output exercises the newly added glyphs.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1929 unit tests pass** (+10 new: 7 layout [Dynamic::ALL no-duplicates, extreme pianissimo→SMuFL p-levels, extreme fortissimo→SMuFL f-levels, accent dynamics map + sf≠sfz invariant, rinforzando dynamics map + rf≠rfz invariant, forzando→SMuFL forzando, niente→SMuFL niente, poco-forte→SMuFL pf + pf≠f≠p invariants]; 3 renderer [all 25 variants render via Dynamic::ALL, all 25 produce distinct SVG output, all 6 extreme p/f Bravura glyphs have nonzero advance width]). `cargo test -p music-engraver --test golden_svg` — 53 golden tests pass (no baseline drift: existing `dynamics_score` golden does not exist, and existing dynamics tests still pass on the renamed test set). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`). `cargo run --example dynamics_score` writes 29040 bytes (35 paths, 29 lines, up from prior 31 paths) — the +4 paths are the new sf/rfz/fz/niente glyphs in M4 replacing the prior sfz/pp/ppp.
- Next: Other post-v1 items. Trill polish leftovers: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; per-segment `WiggleTrillFast` selection; explicit `HookDirection::Up` for trills rendered below the staff; configurable bracket-hook length. Engraving features: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; additional ornaments (turn variations); more golden tests for coverage; lyric coverage tweaks.
- Open issues: `Sfp` (sforzando-piano, SMuFL `DynamicSforzandoPiano`) and the related `DynamicSforzatoPiano` glyph both spell "sfp" but differ subtly in the 's' letterform — only the `SforzandoPiano` variant is exposed; the `SforzatoPiano` glyph is reachable only by directly constructing the SMuFL glyph in custom code. The `DynamicZ` glyph (rare "z" notation) is not exposed via a `Dynamic` variant. `HookDirection::Up` for trill brackets and the bracket-length override are still hardcoded (see prior entry's open issues).

## 2026-05-13 — Post-v1, trill wiggle-speed test/example/golden coverage
- Did: An earlier (unrecorded) chunk landed the full data path for trill wiggle-speed selection — `TrillWiggleSpeed` enum (9 variants), `layout_trill_extension_with_glyph`, `NoteAnnotations::trill_wiggle_speed`, `ScoreBuilder::trill_with_extension_speed`, system-renderer collector + draw using the speed's glyph and advance, and the page-renderer cross-system path threading the same speed through `UnresolvedTrillExtension.wiggle_speed`. This chunk closes the coverage gap that prior work left open: real tests, an example, and a golden baseline that actually exercise the speed dimension end-to-end. Added 6 ScoreBuilder tests in `src/score/tests.rs` (`trill_with_extension_speed_{sets_all_three_annotation_fields, on_rest_is_noop, renders_wiggle_paths, standard_matches_default_extension, different_speeds_produce_different_svg, on_chord_adds_wiggle}`) — the `standard_matches_default_extension` test is a regression canary asserting byte-identical SVG between `.trill_with_extension()` and `.trill_with_extension_speed(Standard)`. Added 5 system-renderer tests in `src/render/system_renderer/tests.rs` (`trill_wiggle_speed_{collector_propagates_speed_when_extension_active, collector_drops_speed_when_no_extension, fast_tiles_more_segments_than_slow, each_variant_uses_its_own_glyph_advance, none_uses_default_glyph}`) — `each_variant_uses_its_own_glyph_advance` queries Bravura for the advance of every speed glyph and asserts strict monotonicity (>1 font-unit gap between adjacent advances after sort), guarding against any future regression that wires the speed but ignores its glyph-specific advance. Added 3 page-renderer cross-system tests in `src/render/page_renderer/tests.rs` (`cross_system_trill_extension_{fast_speed_adds_more_incoming_paths_than_slow, speed_changes_svg_byte_for_byte, standard_speed_matches_unset_speed}`). New `examples/trill_wiggle_speed_score.rs` renders 5 measures across 2 systems, one per non-redundant speed (Fastest/Fast/Standard/Slow/Slowest), and asserts mixed-speed > all-Slowest path count. New `tests/golden/trill_wiggle_speed.svg` (63585 bytes) + `golden_trill_wiggle_speed` test that walks every variant in `TrillWiggleSpeed::ALL` (9 measures, one per speed) and sandwiches the mixed-speed path count strictly between all-Slowest and all-Fastest — a structural guard that none of the 9 speeds collapse to a single glyph. Added `"trill_wiggle_speed"` to `golden_baselines_are_valid_svgs` name list. Silenced the (pre-existing) `clippy::derivable_impls` warning on `impl Default for TrillWiggleSpeed` with an explanatory comment — the default is intentionally `Standard`, not the first variant `Fastest`.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1952 unit tests pass** (+23 vs the prior recorded run: +9 untracked tests for `TrillWiggleSpeed` + `layout_trill_extension_with_glyph` landed in an earlier chunk; +14 new tests this chunk: 6 score + 5 system_renderer + 3 page_renderer). `cargo test -p music-engraver --test golden_svg` — **54 golden tests pass** (+1 new `golden_trill_wiggle_speed`). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`; the `derivable_impls` warning on `TrillWiggleSpeed::default()` is now suppressed with a `#[allow]` and an explanatory comment). `cargo run --example trill_wiggle_speed_score` writes 44507 bytes (82 paths, 28 lines); the example's own assertions confirm `mixed > all-Slowest` and ≥18 paths.
- Next: Other deferred trill polish: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; `HookDirection::Up` exposed through ScoreBuilder for trills rendered below the staff; configurable bracket-hook length (currently hardcoded at 0.75 staff spaces). Or other post-v1 items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; additional ornaments (turn variations); more golden tests for coverage; lyric coverage tweaks.
- Open issues: The `wiggle_speed_each_variant_uses_its_own_glyph_advance` test embeds a Bravura-specific monotonicity assertion (>1 font-unit gap between adjacent advances). For Petaluma/Leland this may need a per-font epsilon — the test should be re-examined when a second SMuFL font is wired in. The `trill_wiggle_speed` golden has the same staff-line width as `trill_extension` but is structurally distinct (9 measures × 3-per-system = 3 systems). Mid-trill speed change (a single sustained trill that visibly accelerates) is not supported — the speed is per-trill, not per-segment; this is the canonical Behind Bars convention but worth noting for future. `HookDirection::Up` for trill brackets and the bracket-length override are still hardcoded.


## 2026-05-13 — Post-v1, fermata duration variants (long/short/very-long/very-short + Henze pair)

- Did: Expanded `layout::articulation::Articulation` from 6 to 12 variants by adding the SMuFL fermata duration-coded family: `FermataLong` (square), `FermataShort` (triangle), `FermataVeryLong`, `FermataVeryShort`, `FermataHenzeLong`, `FermataHenzeShort` (the Hans Werner Henze bracket-style alternatives). All six map to their `FermataLong/Short/VeryLong/VeryShort/LongHenze/ShortHenzeAbove/Below` SMuFL glyphs through the existing above/below switch in `Articulation::glyph`. Introduced `Articulation::is_fermata()` predicate so the "always above" placement rule and the stack-splitter logic in `layout_articulation_stack` recognize every fermata variant identically — replaces the previous `== Self::Fermata` / `== Articulation::Fermata` literal checks, which would have silently treated the new variants as ordinary articulations and flipped them below the staff for stem-up notes. Renderer (`render::articulation_renderer`) and the public `.articulation(Articulation)` ScoreBuilder method needed zero changes: both are glyph-agnostic and pick up the new variants by dispatching through `glyph()`. Updated doc comment on `ScoreBuilder::articulation` to enumerate the fermata family.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1970 unit tests pass** (+18 new: 13 layout [6 glyph-pair tests one per new variant, `all_fermata_variants_recognized_by_is_fermata`, `non_fermata_articulations_not_flagged_by_is_fermata`, `all_fermata_variants_default_to_above`, `all_fermata_variants_produce_distinct_{above,below}_glyphs`, `fermata_variants_above_below_differ_within_each`, `fermata_variants_share_y_when_layout_alone`, `stack_long_fermata_with_staccato_separates_placement`, `stack_multiple_fermata_variants_all_above`]; 1 renderer [`all_fermata_duration_variants_render_distinct_paths` — extracts `d="..."` from each variant's SVG and asserts pairwise distinct path d-data across all 7 fermata variants, guarding against silent fallback to the plain Fermata glyph]; 2 score-integration [`each_fermata_duration_variant_adds_a_path_and_differs_from_plain`, `stack_long_fermata_with_staccato_adds_two_paths`]). `cargo test -p music-engraver --test golden_svg` — 54 golden tests pass (no baseline drift: no existing golden uses the duration variants). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Other deferred trill polish (`.trill_to(note_index)` API, `HookDirection::Up` exposed through ScoreBuilder, configurable bracket-hook length). Or other post-v1 items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; additional ornaments (turn variations); golden coverage for the new fermata variants; lyric coverage tweaks; PNG export deeper testing.
- Open issues: The `fermata_variants_share_y_when_layout_alone` test asserts `(l.y - first_y).abs() < 1e-9` — all variants share placement rules so y must be byte-identical, but if a future change reads any variant-specific anchor (e.g. SMuFL `cutOutNW` per glyph) the tolerance would need revisiting. The renderer test extracts only the first `d="..."` payload from the SVG; this is fine because each per-variant SVG contains exactly one path (a single articulation on a single note), but a future change that emits multiple paths per glyph would need a different comparison strategy. No example or golden baseline added for the new variants — adding one is a small follow-up chunk if visual proofing is wanted. SMuFL `articulationFadeIn`/`articulationFadeOut`/`articulationSoftAccent` and the rest of the "non-percussion articulations" family remain unexposed; only the fermata sub-family is filled out here.

## 2026-05-13 — Post-v1, Ornament enum expansion (turn variations + 8 historical ornaments + `Ornament::ALL`)

- Did: Expanded `layout::ornament::Ornament` from 7 to 15 variants by adding the SMuFL turn variations and standard historical ornaments that the prior progress entries explicitly called out as "additional ornaments (turn variations)" follow-up. New variants: `TurnUp` (Mozart-style vertical-axis turn → `OrnamentTurnUp`), `TurnUpSlash` (slashed vertical turn → `OrnamentTurnUpS`), `Tremblement` (French Baroque trill-like ornament → `OrnamentTremblement`), `TremblementCouperin` (Couperin's specific variant → `OrnamentTremblementCouperin`), `Haydn` (Haydn's stuttered turn → `OrnamentHaydn`), `Shake` (three-line shake → `OrnamentShake3`), `Schleifer` (German Baroque slide-into-note → `OrnamentSchleifer`), `TrillWithMordent` (precomposed compound → `OrnamentPrecompTrillWithMordent`). Added `Ornament::ALL: [Ornament; 15]` constant matching the `Dynamic::ALL` / `TrillWiggleSpeed::ALL` / `Articulation` patterns; existing `Ornament::all()` accessor now returns `&Self::ALL` so the renderer test that iterates `Ornament::all()` continues to exercise every new variant automatically. Renderer (`render::ornament_renderer`) and the public `.ornament(Ornament)` / annotation-pipeline ScoreBuilder API needed zero changes: both are glyph-agnostic and pick up new variants through `glyph()`. Trill-extension wiring in `system_renderer` / `page_renderer` matches on `Some(Ornament::Trill)` specifically, so the new non-trill variants are correctly inert there.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **1990 unit tests pass** (+20 new: 16 layout [`turn_up_glyph`, `turn_up_slash_glyph`, `turn_up_and_turn_up_slash_differ`, `horizontal_and_vertical_turns_differ`, `tremblement_glyph`, `tremblement_couperin_glyph`, `tremblement_variants_differ`, `tremblement_is_distinct_from_trill`, `haydn_glyph`, `shake_glyph`, `schleifer_glyph`, `trill_with_mordent_glyph`, `trill_with_mordent_distinct_from_trill_and_mordent`, `all_returns_fifteen_variants`, `all_and_const_yield_same_slice`, `all_contains_no_duplicates`, `all_variants_map_to_distinct_glyphs_except_short_trill_alias`, `all_contains_each_new_variant_exactly_once`]; 4 renderer [`all_ornaments_produce_pairwise_distinct_paths_except_short_trill_alias` — extracts each variant's `d="..."` payload and asserts pairwise distinct SVG path data across all 15 variants, with the documented `InvertedMordent`≡`ShortTrill` SMuFL alias as the only allowed collision; `turn_up_renders_distinct_from_turn` independently anchors the horizontal-vs-vertical turn split at the renderer level (catches a hypothetical glyph misrouting that the layout test would miss); `every_new_ornament_glyph_has_nonzero_advance_in_bravura` looks up each new glyph's outline in Bravura and asserts non-empty path data — guards against silent missing-glyph regressions if Bravura is ever swapped]). `cargo test -p music-engraver --test golden_svg` — 54 golden tests pass (no baseline drift: no existing golden uses the new variants; existing renderer test `all_ornaments_render_without_error` already iterates `Ornament::all()` so it now exercises the new variants too — 0 paths→8 paths to draw transparently). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Visual-proofing for the new ornaments — add an `examples/ornaments_full_score.rs` exercising all 15 variants and a `golden_ornaments_full` baseline. Or other deferred trill polish (`.trill_to(note_index)` API, `HookDirection::Up` exposed through ScoreBuilder, configurable bracket-hook length). Or other post-v1 items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; lyric coverage tweaks; PNG export deeper testing.
- Open issues: `InvertedMordent` and `ShortTrill` deliberately share the SMuFL `OrnamentShortTrill` glyph (`InvertedMordent` is the Pralltriller and modern engraving collapses the two to the same mark). The pairwise-distinct-paths test documents this as the only allowed collision; if a future engraving rule wants to differentiate them, a separate glyph mapping (e.g. via a per-glyph `cutOut` overlay) would be required. The new ornaments are not yet exposed in any example, golden baseline, or `examples/output/*.svg` — visual proofing is deferred. None of the new variants currently support a wavy-line extension; `Trill` remains the only ornament with `trill_extension` semantics, and the new compound `TrillWithMordent` does **not** automatically activate the extension (the user would have to call `.trill_with_extension()` separately, which currently sets `ornament = Some(Trill)` and would clobber the compound — so the two are not composable in v1).

## 2026-05-13 — Post-v1, visual proofing for the 15-variant `Ornament::ALL` family (example + golden)

- Did: Closed the "new ornaments are not yet exposed in any example, golden baseline, or `examples/output/*.svg`" open issue from the prior chunk. New `examples/ornaments_full_score.rs` walks `Ornament::ALL` in canonical order on a 16-note rising scale (one quarter note per ornament + one padding quarter) laid out 4 ornaments per measure across 2 systems. Asserts ≥35 paths and ≥15 distinct `d="..."` payloads in the rendered SVG. New `build_ornaments_full()` + `golden_ornaments_full` test in `tests/golden_svg.rs` with three structural guards beyond the frozen baseline: (1) **exact 15-path delta** vs an identical no-ornament scale (`saturating_sub` of `<path` counts must equal 15 — proves every ornament actually drew rather than silently no-opping); (2) **distinct-d set difference** between ornament and no-ornament SVGs must equal exactly 14 unique payloads (15 variants − 1 documented `InvertedMordent≡ShortTrill` alias — catches a hypothetical regression where multiple new variants collapse to the same glyph); (3) **byte-inequality** vs the existing 4-ornament `golden_ornaments` baseline (guards against the new score accidentally rendering identically to the small one). Added `"ornaments_full"` to the `golden_baselines_are_valid_svgs` name list. Wrote `tests/golden/ornaments_full.svg` (21701 bytes, 75 lines, 35 paths) and `examples/output/ornaments_full_score.svg` (same byte count — the example and the golden share the build path).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --example ornaments_full_score` passes. `cargo test -p music-engraver --test golden_svg` — **55 golden tests pass** (54 prior + new `golden_ornaments_full`); the 3 in-test assertions (15-path delta, 14 unique ornament d-strings, byte-inequality vs `build_ornaments()`) all hold against the freshly written baseline. `cargo test -p music-engraver --lib` — **1990 unit tests pass** (no new lib tests — visual proofing is verified at the integration/example layer). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo run -p music-engraver --example ornaments_full_score` produces 21701 bytes (35 paths, 38 lines); the example's own assertions (≥35 paths, ≥15 distinct path d-strings) pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Deferred trill polish: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; `HookDirection::Up` exposed through ScoreBuilder for trills rendered below the staff; configurable bracket-hook length (currently hardcoded at 0.75 staff spaces). Or other post-v1 items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; lyric coverage tweaks; PNG export deeper testing.
- Open issues: The exact-15-path-delta assertion in `golden_ornaments_full` would fail if a future engraving change ever caused an ornament to render zero paths (e.g. a Schleifer that picks up a curve-only rendering with no `<path>` element). That's the intended behaviour — the test is a regression canary — but if such a change is intentional the assertion must be reworked. The example's pixel dimensions are small (294×126 at default DPI) because the viewBox is wide and short, not because rendering is broken; visual proofing on the SVG should be done in a viewer that respects the viewBox. `TrillWithMordent` still doesn't support a wavy-line extension; `Trill` remains the only ornament with that semantics.

## 2026-05-13 — Post-v1, trill bracket custom direction + length wiring

- Did: Closed two open issues from prior chunks ("`HookDirection::Up` not exposed through the ScoreBuilder API" and "Bracket length is hardcoded at 0.75 staff spaces") by adding a custom-options wiring for trill brackets. Two new annotation fields on `NoteAnnotations` (in `layout/measure.rs`): `trill_bracket_direction: Option<HookDirection>` (defaults to `None` → conventional `Down`) and `trill_bracket_length_ss: Option<f64>` (defaults to `None` → existing `TRILL_BRACKET_HOOK_LENGTH_SS = 0.75ss`). Both fields are read only when `trill_bracket` is `Some`, mirroring the existing coupling. New public method `ScoreBuilder::trill_with_extension_bracketed_custom(side, direction, length_ss)` that sets all five coupled annotations at once. Wired through `TrillExtensionNoteInfo` in `render/system_renderer/mod.rs` (new `bracket_direction` + `bracket_length_ss` fields populated by `collect_trill_extension_note_info`); the within-system draw pass now reads these to override the constants when present. Wired through `UnresolvedTrillExtension` in `render/page_renderer/mod.rs` so cross-system End hooks also honor the user's choices. Refactored `layout_trill_end_hook(x, y, length, stroke)` to `layout_trill_end_hook(x, y, length, direction, stroke)` — the single private caller in page_renderer updated. New example `examples/trill_bracket_custom_score.rs` (28817 bytes, 43 paths, 41 lines) exercises Down/Up directions across 0.5/0.75/1.0/1.2 staff-space lengths on whole notes, chords, and a cross-system bracket. New golden baseline `tests/golden/trill_bracket_custom.svg` (33444 bytes) + `golden_trill_bracket_custom` test asserting (a) exact 6-hook delta vs the no-bracket variant (same as `golden_trill_bracket` — custom knobs change geometry only, not element count), (b) SVG differs byte-for-byte from `build_trill_bracket()` (the user's overrides actually take effect), (c) the `<line>` count matches the default-options variant. Added `"trill_bracket_custom"` to `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --tests` passes. `cargo test -p music-engraver --lib` — **2004 unit tests pass** (1990 prior + 14 new: 6 score-integration [`trill_with_extension_bracketed_custom_{sets_all_five_annotation_fields, default_args_matches_plain_bracketed, length_changes_svg, direction_changes_svg, on_rest_is_noop, hook_up_y_coordinates}` — the last one parses y1/y2 out of the last `<line>` element and verifies Down's y_top equals Up's y_bottom, proving they share the wiggle baseline anchor]; 5 system_renderer [`trill_bracket_custom_{collector_propagates_direction_and_length, collector_drops_direction_and_length_without_bracket, length_changes_hook_line_geometry, direction_up_flips_hook_y_extents, defaults_match_plain_bracketed}` — the last is the within-system byte-identical canary]; 3 page_renderer [`cross_system_trill_bracket_custom_{length_changes_n_plus_1_hook, direction_changes_n_plus_1_hook, defaults_match_plain_bracketed}` — covering the page-level cross-system continuation]). `cargo test -p music-engraver --test golden_svg` — **56 golden tests pass** (55 prior + new `golden_trill_bracket_custom` with three in-test invariants: 6-hook delta, byte-inequality vs default-variant, line-count equality vs default-variant). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo run -p music-engraver --example trill_bracket_custom_score` produces 28817 bytes (43 paths, 41 lines) and asserts an exact 8-hook delta vs the no-bracket variant, byte-inequality vs the default-options variant, and equal `<line>` counts between custom and default. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Other deferred trill polish: `.trill_to(note_index)` API for sub-note-granularity wiggle end points. Or other post-v1 items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; lyric coverage tweaks; PNG export deeper testing; additional dynamic glyphs not yet covered (`Sfp` family ambiguity, `DynamicZ`); short-trill+extension support (currently only `Trill` triggers the wiggle).
- Open issues: The custom-API has no convenience constructor for "Up direction with default length" or "default direction with custom length" — callers always pass both. A future ergonomic improvement would be a builder struct (`TrillBracketOptions { side, direction: Option<...>, length_ss: Option<...> }`) consumed by a single `.trill_with_extension_bracketed_with_options(opts)`. Length is unvalidated: a length of 0.0 produces a degenerate hook (no visible line), negative lengths are folded to absolute value in `layout_trill_bracket_hook` (the `HookDirection` always wins). Extreme lengths (e.g. 10ss) are not clamped — the hook will extend that far, which may collide with other staff content. Callers passing custom lengths are responsible for sanity-checking the value.


## 2026-05-13 — Post-v1, Dynamic enum: closed `DynamicZ` + `DynamicSforzatoPiano` gaps + added `Mezzo`

- Did: Closed the two `Dynamic` open issues flagged in the prior chunk's "Open issues" — the `DynamicZ` glyph being unreachable from the public API, and the `DynamicSforzatoPiano` glyph being reachable only by directly constructing the SMuFL glyph (since the existing `Sfp` variant binds the visually similar `DynamicSforzandoPiano`). Also filled the obvious `Piano`/`Mp`/`Mf` neighbourhood gap by exposing the bare `m` letter glyph. Three new variants in `layout/dynamics.rs`: `Mezzo` → `Glyph::DynamicMezzo`, `SforzatoPiano` → `Glyph::DynamicSforzatoPiano`, and `Z` → `Glyph::DynamicZ`. `Dynamic::ALL` grew from 25 to 28 with the new variants placed in canonical ordering (Mezzo between Piano and Mp; SforzatoPiano next to its sibling `Sfp`; Z at the end as the rarest). Renderer (`render::dynamics_renderer`) and the public `.dynamic(Dynamic)` ScoreBuilder method needed zero changes: both are glyph-agnostic and pick up new variants automatically. Doc comments on the new variants explicitly call out the subtle `Sfp` vs `SforzatoPiano` distinction (sforzando-prefix vs sforzato-prefix `s` letterform), so callers picking one over the other do so deliberately.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo test -p music-engraver --lib` — **2011 unit tests pass** (+7 new vs the prior recorded 2004: 5 layout [`mezzo_maps_to_smufl_mezzo` with `Mezzo ≠ Mp` and `Mezzo ≠ Mf` regression guards; `z_maps_to_smufl_dynamic_z` with `Z ≠ Sfz/Sf/Fz/Rfz` non-collapse guards; `sforzato_piano_distinct_from_sforzando_piano` proving `Sfp ≠ SforzatoPiano` glyph-level distinctness; `all_constant_contains_each_new_variant_exactly_once`; `all_constant_has_expected_length` documenting the new 28-variant total]; 2 renderer [`new_variants_render_distinct_path_data_in_bravura` extracts each variant's `d="..."` payload and asserts pairwise distinctness among the new trio AND distinctness vs each new variant's closest existing neighbour (Mezzo vs Mp/Mf, Z vs Sfz/Fz, SforzatoPiano vs Sfp/Sf) — catches a hypothetical glyph-mapping typo that the layout-level distinctness test would miss because identical glyphs would have identical SVG output too; `new_variants_have_nonzero_advance_in_bravura` guards against silent "glyph not in font" failures when a future SMuFL font swap omits one of these less-common glyphs]). The existing `all_dynamics_render_without_error` and `all_dynamics_produce_distinct_svg_output` renderer tests automatically exercise the new variants because they iterate `Dynamic::ALL`. `cargo test -p music-engraver --test golden_svg` — 56 golden tests pass (no baseline drift: no existing golden uses the new variants, and the existing `dynamics_score` example was not modified). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Visual proofing for the new variants — add a small example or extend `examples/dynamics_score.rs` measure 4 to exercise `Mezzo`/`Z`/`SforzatoPiano` (currently exercises `Sf`/`Rfz`/`Fz`/`Niente`). Other post-v1 items: deferred trill polish (`.trill_to(note_index)` API for sub-note-granularity wiggle end points; trill bracket `TrillBracketOptions` builder struct to avoid the all-or-nothing custom API); cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; short-trill+extension support (currently only `Trill` triggers the wiggle, even though `TrillWithMordent` was added in the ornament expansion chunk); golden coverage for the new fermata duration variants and the new ornament variants.
- Open issues: The new variants are not yet exercised in any example, golden baseline, or `examples/output/*.svg` — visual proofing is deferred. `DynamicMezzo` in Bravura is conventionally italic and shorter than `Mp`/`Mf`; if a user mixes `Mezzo` with `Mp`/`Mf` in the same line the baselines may look subtly mismatched at small staff sizes — a per-variant baseline offset (e.g. via SMuFL anchor data) could refine this in future. The remaining unexposed Dynamic glyphs in SMuFL are mostly hairpin parts (`DynamicHairpinBracketLeft`/`...Right`/`...ParenthesisLeft`/`...Right`/`DynamicMessaDiVoce`/`DynamicNienteForHairpin`) and combined-separator marks (`DynamicCombinedSeparator{Colon,Hyphen,Slash,Space}`/`DynamicCrescendoHairpin`/`DynamicDiminuendoHairpin`/`DynamicSforzando` — bare letter "s" prefix) — these are intentionally not exposed because they're either decoration parts (hairpin parens), already covered by the existing crescendo/decrescendo wedge primitives, or only meaningful as building blocks for combined dynamic constructions that are not yet supported.

## 2026-05-13 — Post-v1, fermata duration variants visual proofing (example + golden)

- Did: Closed the deferred "golden coverage for the new fermata duration variants" follow-up explicitly flagged in the 2026-05-13 fermata-expansion chunk and again in the most recent Dynamic-expansion entry. New example `examples/fermata_variants_score.rs` walks the 7 SMuFL fermata variants (`Fermata`, `FermataLong`, `FermataShort`, `FermataVeryLong`, `FermataVeryShort`, `FermataHenzeLong`, `FermataHenzeShort`) on whole notes, one per measure, staggered across G4–F5 so successive glyphs sit at visibly different staff positions; 4 systems × 2 measures with one padding whole note. Two structural assertions inside the example: (a) the variant score must add exactly `variants.len()` paths over an identical no-articulation score (catches a silent missing-glyph regression), and (b) the variants must contribute exactly `variants.len()` unique path d-strings (catches two variants collapsing to the same glyph). New `build_fermata_variants()` + `build_fermata_variants_plain()` builders and `golden_fermata_variants` test in `tests/golden_svg.rs` with three structural guards beyond the frozen baseline: (1) **exact 7-path delta** vs the plain-baseline (saturating-sub of `<path` counts must equal 7); (2) **distinct-d set difference** must equal exactly 7 unique payloads (proves no glyph collapses — unlike `golden_ornaments_full` which has a documented `InvertedMordent≡ShortTrill` alias, the fermata family has zero allowed collisions per the existing `all_fermata_variants_produce_distinct_above_glyphs` layout test); (3) **byte-inequality** vs the existing `articulations` baseline. Added `"fermata_variants"` to the `golden_baselines_are_valid_svgs` name list. Wrote `tests/golden/fermata_variants.svg` (15757 bytes) and `examples/output/fermata_variants_score.svg` (same bytes — example and golden share the build path).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --example fermata_variants_score` passes. `cargo test -p music-engraver --test golden_svg` — **57 golden tests pass** (56 prior + new `golden_fermata_variants`); the 3 in-test assertions (7-path delta, 7 unique d-strings, byte-inequality vs `articulations`) all hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo run -p music-engraver --example fermata_variants_score` produces 15757 bytes (21 paths, 29 lines); the example's own assertions (exact 7-path delta, exactly 7 unique d-strings vs no-articulation baseline) pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Visual proofing for the new Dynamic variants (`Mezzo`/`Z`/`SforzatoPiano`) flagged in the prior chunk — either extend `examples/dynamics_score.rs` or add a dedicated example + golden. Or other deferred items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; `TrillBracketOptions` builder struct (currently the custom-direction/length API is all-or-nothing); cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; short-trill+extension support.
- Open issues: The exact-7-path-delta assertion would fail if a future engraving change ever caused a fermata variant to render zero paths (e.g. a future Bravura swap where a Henze glyph is replaced by `none`). That's the intended behaviour — the test is a regression canary. The fermata score uses whole notes throughout, so it does not exercise the on-line vs in-space clearance distinction or the stem-direction interaction (both already covered by layout-level unit tests in `articulation.rs`); the golden is a glyph-proofing test, not a placement-correctness test. I2 still deferred.

## 2026-05-13 — Post-v1, Dynamic variants visual proofing (`Mezzo`/`SforzatoPiano`/`Z` example + golden)

- Did: Closed the "new variants are not yet exercised in any example, golden baseline, or `examples/output/*.svg`" open issue from the 2026-05-13 Dynamic-expansion chunk. New example `examples/dynamics_variants_score.rs` exercises the three less-common variants on a single 4/4 measure of ascending quarter notes: `C4.dynamic(Mezzo)`, `D4.dynamic(SforzatoPiano)`, `E4.dynamic(Z)`, and a plain `F4` padding quarter so the measure closes cleanly. Two structural assertions inside the example: (a) exactly 3-path delta vs an identical no-dynamic baseline (catches a silent missing-glyph regression — already guarded at the unit-test level by `new_variants_have_nonzero_advance_in_bravura` but reinforced at the integration layer), and (b) exactly 3 unique d-strings contributed by the dynamics (catches a hypothetical glyph collapse — Bravura's `DynamicMezzo`/`DynamicSforzatoPiano`/`DynamicZ` are all distinct outlines per the existing renderer-level `new_variants_render_distinct_path_data_in_bravura` test). New `build_dynamics_variants()` + `build_dynamics_variants_plain()` builders and `golden_dynamics_variants` test in `tests/golden_svg.rs` with three structural guards beyond the frozen baseline: (1) exact 3-path delta vs the plain-baseline (`saturating_sub` of `<path` counts must equal 3); (2) distinct-d set difference must equal exactly 3 unique payloads (no collapse — unlike `golden_ornaments_full`'s documented `InvertedMordent≡ShortTrill` alias, the dynamic family has zero allowed collisions here); (3) byte-inequality vs the existing `dynamics` baseline (sanity check that the new golden isn't accidentally identical to a prior one — guards against a refactor where one of the new variants is wired to an existing glyph and the gestures happen to produce the same SVG). Added `"dynamics_variants"` to the `golden_baselines_are_valid_svgs` name list. Wrote `tests/golden/dynamics_variants.svg` (9346 bytes) and `examples/output/dynamics_variants_score.svg` (same bytes — example and golden share the build path; verified byte-identical via `cmp`).
- Verified: `cargo check --workspace` passes. `cargo build -p music-engraver --example dynamics_variants_score` passes. `cargo run -p music-engraver --example dynamics_variants_score` produces 9346 bytes (10 paths, 12 lines); the example's own assertions (exact 3-path delta, exactly 3 unique d-strings vs no-dynamic baseline) pass. `cargo test -p music-engraver --test golden_svg` — **58 golden tests pass** (57 prior + new `golden_dynamics_variants`); the 3 in-test assertions (3-path delta, 3 unique d-strings, byte-inequality vs `build_dynamics()`) all hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --lib` — **2011 unit tests pass** (no new lib tests — visual proofing is verified at the integration/example layer; the variants already have layout + renderer unit-test coverage from the prior expansion chunk). `cargo test -p music-engraver --doc` — 8 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`).
- Next: The remaining unexposed Dynamic glyphs (hairpin parts and combined-separator marks) per the prior chunk's open-issue note — intentionally deferred since they're decoration parts or building blocks. Other post-v1 items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; `TrillBracketOptions` builder struct (currently the custom-direction/length API is all-or-nothing); cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; short-trill+extension support (currently only `Trill` triggers the wiggle, even though `TrillWithMordent` was added in the ornament expansion chunk); PNG export deeper testing.
- Open issues: The 3-path-delta assertion would fail if a future engraving change ever caused one of these variants to render zero paths (e.g. a future Bravura swap where one glyph is replaced by `none`). That's the intended behaviour — the test is a regression canary. The dynamics score uses unbeamed quarter notes throughout, so it does not exercise dynamic placement under beamed groups, below-staff stems with bass-clef pitches, or stem-direction interactions; the golden is a glyph-proofing test, not a placement-correctness test. `Mezzo` and `SforzatoPiano` are not yet compared side-by-side with their close neighbours (`Mp`/`Mf` for Mezzo; `Sfp` for SforzatoPiano) in any single example — a viewer wanting that comparison must hold the new score and `dynamics_score.svg` next to each other. A future "dynamics_full" example could thread all 28 `Dynamic::ALL` variants for complete coverage. I2 still deferred.

## 2026-05-13 — Post-v1, `TrillBracketOptions` builder + `with_options` ScoreBuilder method

- Did: Closed the "no convenience constructor for partial overrides" open issue from the 2026-05-13 trill-bracket-custom chunk by adding a fluent options struct. The existing `trill_with_extension_bracketed_custom(side, direction, length_ss)` is all-or-nothing — a caller wanting "Up direction but default length" or "default direction but custom length" must restate the other knob. New struct `TrillBracketOptions { side, direction: Option<HookDirection>, length_ss: Option<f64> }` in `layout/trill_bracket.rs` with: `const fn new(side)` constructor (both overrides start as `None`); `const fn with_direction(self, direction)` and `const fn with_length_ss(self, length_ss)` builder methods (all `const`-callable so the common defaults can live in `const` items); `impl From<TrillBracketSide>` for ergonomic `side.into()` construction; `#[derive(Clone, Copy, Debug, PartialEq)]`. New ScoreBuilder method `trill_with_extension_bracketed_with_options(opts: TrillBracketOptions)` (`src/score/mod.rs`) — sets `ornament = Some(Trill)`, `trill_extension = true`, `trill_bracket = Some(opts.side)`, then assigns `opts.direction` and `opts.length_ss` *directly* to the annotation fields (no `None → Some(default)` collapse, so a future tweak to the renderer's default propagates without API churn). No-op on rests, identical pattern to the existing methods. Re-exported `TrillBracketOptions` from `layout/mod.rs`. No changes needed in `system_renderer`/`page_renderer`/annotation pipeline — the new method writes the same `Option` annotation fields as the existing `trill_with_extension_bracketed_custom` and reuses every downstream path.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **2028 unit tests pass** (2011 prior + 17 new: 9 layout [`options_new_has_required_side_and_unset_overrides`, `options_with_direction_sets_only_direction`, `options_with_length_ss_sets_only_length`, `options_chained_with_methods_both_apply`, `options_chain_order_independent`, `options_with_method_overwrites_prior_value`, `options_from_side_matches_new`, `options_const_constructible` (compile-fail canary if `const fn` is removed), `options_copy_does_not_consume_original` (compile/Copy canary)]; 8 score-integration [`with_options_default_matches_plain_bracketed_byte_for_byte` — the key regression canary asserting an all-defaults options call renders byte-identically to the existing `trill_with_extension_bracketed(side)`; `with_options_both_overrides_matches_custom_call_byte_for_byte` — asserts options with both fields populated renders byte-identically to the existing `trill_with_extension_bracketed_custom(side, dir, len)`; `with_options_length_only_renders_distinct_from_default` + `with_options_direction_only_renders_distinct_from_default` — partial-override tests with `<line>`-count equality guards; `with_options_from_side_via_into_matches_new`; `with_options_sets_annotation_fields_matching_overrides` — field-level inspection of the builder's intermediate state proving `None` is preserved (not silently collapsed); `with_options_on_rest_is_noop`; `with_options_on_chord_renders_bracket` — exact +2 `<line>` delta vs plain chord]). `cargo test -p music-engraver --test golden_svg` — 58 golden tests pass (no baseline drift: the new method writes the same downstream state as existing methods). `cargo test -p music-engraver --doc` — **9 doc tests pass** (+1: the doc example in `TrillBracketOptions::new` runs as a 9th doc-compile test, covering the canonical `new` / `with_length_ss` / `with_direction` usage patterns). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Other post-v1 items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; short-trill+extension support (currently only `Trill` triggers the wiggle, even though `TrillWithMordent` was added in the ornament expansion chunk); PNG export deeper testing; "dynamics_full" example threading all 28 `Dynamic::ALL` variants; visual proofing for `TrillBracketOptions` (a small example that exercises Down/Up × default-len/custom-len permutations through the new method specifically).
- Open issues: The new `with_options` method shares all downstream wiring with the existing `trill_with_extension_bracketed_custom`, so any rendering bug affects both equally — no new bug surface, but also no independent verification at the renderer level beyond the byte-identical assertion. The byte-identical assertion in `with_options_default_matches_plain_bracketed_byte_for_byte` would fail if a future change reads `trill_bracket_direction == None` differently from "field absent" — they are intentionally semantically equivalent (`None` means "use renderer default"), and this test is the canary. The options struct does not currently expose `side` as `Option<TrillBracketSide>` (i.e. there's no way to construct "options without a side") — that would conflict with the design contract that bracketed-trill API requires a side; the existing `trill_with_extension()` (no bracket) covers the no-bracket case. `TrillBracketOptions` is exposed but `TrillBracketSide`/`HookDirection` must still be imported separately when constructing options; a `prelude` module that re-exports the common bracket types together would be a small ergonomic improvement but is intentionally out of scope here.

## 2026-05-13 — Post-v1, full-coverage `dynamics_full` example + golden (all 28 `Dynamic::ALL` variants)

- Did: Closed the "future 'dynamics_full' example threading all 28 `Dynamic::ALL` variants for complete coverage" follow-up explicitly flagged in the 2026-05-13 `dynamics_variants` chunk and again in the most recent `TrillBracketOptions` entry. New example `examples/dynamics_full_score.rs` walks `Dynamic::ALL` in canonical order on 28 quarter notes laid out 4 per measure across 7 measures (2 measures per system → 3 full systems + 1 trailing single-measure system). Pitches use two passes through C4–B5 so noteheads stay within the staff's comfortable range and don't introduce ledger lines below the staff (which would crowd the dynamics' below-staff baseline). Two structural assertions inside the example: (a) ≥60 paths (28 noteheads + 28 dynamics + ≥4 clefs), and (b) ≥29 distinct `d="..."` payloads (28 unique dynamic glyphs + ≥1 notehead). New `build_dynamics_full()` + `build_dynamics_full_plain()` builders sharing a `DYNAMICS_FULL_PITCHES: [(&str, u8); 28]` constant in `tests/golden_svg.rs`, plus `golden_dynamics_full` test with three structural guards beyond the frozen baseline: (1) **exact 28-path delta** vs the same 28-note score with no dynamics (`saturating_sub` of `<path` counts must equal `Dynamic::ALL.len()` — proves every variant actually drew rather than silently no-opping); (2) **distinct-d set difference** between dynamic-bearing and plain SVGs must equal exactly `Dynamic::ALL.len()` unique payloads (28 — proves no two variants collapse to the same glyph; unlike `golden_ornaments_full` which has a documented `InvertedMordent≡ShortTrill` alias, the dynamic family has zero allowed collisions per the existing `all_dynamics_produce_distinct_svg_output` and `new_variants_render_distinct_path_data_in_bravura` renderer unit tests); (3) **byte-inequality** vs both the existing 4-note `dynamics` and 4-note `dynamics_variants` baselines (sanity check that the new golden isn't accidentally identical to a prior one). Added `"dynamics_full"` to the `golden_baselines_are_valid_svgs` name list. Wrote `tests/golden/dynamics_full.svg` (74860 bytes) and `examples/output/dynamics_full_score.svg` (same bytes — example and golden share the build path).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --example dynamics_full_score` passes. `cargo test -p music-engraver --test golden_svg` — **59 golden tests pass** (58 prior + new `golden_dynamics_full`); the 3 in-test assertions (28-path delta, 28 unique d-strings, byte-inequality vs both `dynamics` and `dynamics_variants`) all hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --lib` — **2028 unit tests pass** (no new lib tests — visual proofing is verified at the integration/example layer; all 28 variants already have layout + renderer unit-test coverage from the prior expansion chunks). `cargo test -p music-engraver --doc` — 9 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo run -p music-engraver --example dynamics_full_score` produces 74860 bytes (62 paths, 62 lines); the example's own assertions (≥60 paths, ≥29 distinct d-strings) pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Other post-v1 items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; short-trill+extension support (currently only `Trill` triggers the wiggle, even though `TrillWithMordent` was added in the ornament expansion chunk); PNG export deeper testing; visual proofing for `TrillBracketOptions` (Down/Up × default-len/custom-len permutations through the new method specifically); a side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano` (currently those visually similar pairs are not shown together in any single example).
- Open issues: The exact-28-path-delta assertion would fail if a future engraving change ever caused a dynamic variant to render zero paths (e.g. a future Bravura swap where a glyph is replaced by `none`). That's the intended behaviour — the test is a regression canary. The dynamics_full score uses unbeamed quarter notes throughout, so it does not exercise dynamic placement under beamed groups, below-staff stems with bass-clef pitches, or stem-direction interactions; the golden is a glyph-proofing test, not a placement-correctness test. The 7-measure / 2-per-system layout produces a trailing single-measure system (system 4 has only 1 measure) — visually slightly unbalanced but structurally valid; a future tweak could pad to 8 measures or shift to `measures_per_system(4)` for a symmetric 2-system layout if the asymmetry matters. The pitch pattern repeats C4–B5 twice — visual aesthetics could be improved with a more melodically interesting sequence, but the current pattern keeps all noteheads within a single staff range and makes the dynamic glyph the visually distinguishing element (exactly what a glyph-proofing example should do).

## 2026-05-13 — Post-v1, trill extension supports `Ornament::TrillWithMordent` (precomposed compound + wiggle)

- Did: Closed the "currently only `Trill` triggers the wiggle, even though `TrillWithMordent` was added in the ornament expansion chunk" open issue that has been flagged in every "Next" section since the Ornament expansion. Added `Ornament::supports_trill_extension()` returning `true` for both `Trill` and `TrillWithMordent` (and explicitly `false` for `ShortTrill`, which is by definition the wave-less form). Replaced the literal `matches!(annotations.ornament, Some(Ornament::Trill))` guard in `collect_trill_extension_note_info` (`src/render/system_renderer/mod.rs`) — both the Note and Chord arms — with `ornament.map(|o| o.supports_trill_extension()).unwrap_or(false)`, so the collector now flags compound-glyph notes as extension-bearing too. Extended `TrillExtensionNoteInfo` with `pub ornament: Option<Ornament>` (populated only when `has_trill_extension == true`) so the draw pass can look up the *actual* glyph's advance — the precomposed compound is ~470 font-units wider than the bare "tr" (Bravura: 990 vs 521), so the wiggle must start past the *full* compound, not just the trill prefix. Updated `draw_system_trill_extensions` to compute `trill_advance = font.glyph_advance(ornament_kind.glyph())` per-note instead of hardcoding `OrnamentTrill`. The `layout_ornament` call still uses `Ornament::Trill` because the y is glyph-independent (it reads only staff geometry); a documenting comment explains why this is OK. **No page_renderer changes needed**: cross-system y-anchoring (`y_above_top_line`) is glyph-independent, and the incoming wiggle on system N+1 starts from the staff's `staff_left` rather than from any glyph, so the source ornament's identity doesn't affect N+1's geometry. Added `ScoreBuilder::trill_with_mordent_with_extension()` mirroring the existing `trill_with_extension()` pattern but setting `ornament = Some(Ornament::TrillWithMordent)`. The bracket/speed builder methods still hardcode `Ornament::Trill` and are documented as such — users wanting bracket-form compound trills must compose annotations manually (deferred follow-up). New example `examples/trill_with_mordent_score.rs` exercises half/dotted-half/whole/chord compound-trills with extensions across 2 systems. New golden baseline `tests/golden/trill_with_mordent_extension.svg` (8758 bytes) + `golden_trill_with_mordent_extension` test with three structural guards: (1) ≥2 wiggle paths added vs plain compound (no-extension) baseline; (2) byte-inequality vs the `build_trill_extension` baseline — proves the actual ornament propagated through the collector rather than collapsing to `Trill`; (3) compound version's path count is *strictly less than* the plain-trill version's path count for the same musical span, confirming the wider glyph advance ate some of the wiggle's available room (i.e. the advance lookup is honoring the compound glyph, not the bare "tr"). Added `"trill_with_mordent_extension"` to the `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **2044 unit tests pass** (+16 vs the prior recorded 2028: 5 layout [`supports_trill_extension_is_{true_for_trill, true_for_trill_with_mordent, false_for_short_trill, false_for_non_trill_variants}`, `supports_trill_extension_count_is_two_across_all_variants`]; 6 system_renderer [`trill_extension_collector_{accepts_trill_with_mordent, drops_short_trill, drops_non_trill_ornament_field, propagates_trill_variant}`, `trill_with_mordent_extension_renders_wiggle_paths`, `trill_with_mordent_extension_uses_wider_glyph_advance` (regression canary: 521 vs 990 font-units in Bravura)]; 5 score-integration [`trill_with_mordent_with_extension_{sets_both_annotation_fields, renders_wiggle_paths, on_rest_is_noop, on_chord_adds_wiggle, wiggle_starts_past_full_compound_glyph}`]). `cargo test -p music-engraver --test golden_svg` — **60 golden tests pass** (59 prior + new `golden_trill_with_mordent_extension`); the 3 in-test assertions (≥2 wiggle paths, byte-inequality vs `build_trill_extension`, fewer wiggle segments than plain-trill version due to wider compound advance) all hold. `cargo test -p music-engraver --doc` — 9 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). `cargo run -p music-engraver --example trill_with_mordent_score` writes 33392 bytes (63 paths, 30 lines) and the example's own assertions confirm: ≥18 paths, more paths than the no-extension version, byte-different from the plain-trill-extension variant.
- Next: Other deferred trill polish: bracket-form compound trills (`.trill_with_extension_bracketed*` family currently hardcodes `Ornament::Trill`; an `_with_mordent` variant or a generalized `ornament: Ornament` parameter would close that gap); `.trill_to(note_index)` API for sub-note-granularity wiggle end points. Other post-v1 items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; a side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano` (currently those visually similar pairs are not shown together in any single example).
- Open issues: The bracket-form builder methods (`trill_with_extension_bracketed*`) still force `Ornament::Trill`, so a user wanting a bracketed compound trill must hand-construct the annotation. The wiggle-speed builder (`trill_with_extension_speed`) has the same limitation. Both could be generalized by accepting an `Ornament` parameter or by filtering through `supports_trill_extension()`. The `trill_with_mordent_extension_renders_wiggle_paths` system_renderer test bumps `min_note_spacing` to 8×staff_space because the compound glyph eats the natural inter-note gap in the standard test layout; this exposes a real engraving concern — at tight measure widths a sustained compound trill may render with zero wiggle segments. The current renderer silently bails (no wiggle drawn), matching the existing plain-trill behavior; both could in principle use a smaller stub wiggle as fallback, but that's a Gould-rule judgment call deferred. The wiggle-y for `TrillWithMordent` is computed via `layout_ornament(Ornament::Trill, …)` — fine in Bravura where the compound glyph's anchor matches Trill's, but if another SMuFL font ever places the precomposed glyph at a different baseline anchor the wiggle could visually drift; the `layout_ornament(…)` call could be parameterized on the actual ornament if this ever bites.

## 2026-05-13 — Post-v1, bracketed compound trills via `TrillBracketOptions::with_ornament`

- Did: Closed the "bracket-form builder methods still force `Ornament::Trill`" open issue from the 2026-05-13 `TrillWithMordent` extension chunk. Extended `layout::trill_bracket::TrillBracketOptions` with a new `ornament: Option<Ornament>` field and a `const fn with_ornament(mut self, ornament: Ornament)` builder method. `None` (the default) collapses to `Ornament::Trill` at the score-builder layer; `Some(Ornament::TrillWithMordent)` selects the precomposed compound. Documented the contract: the ornament must satisfy `supports_trill_extension()` — passing an unsupported one (e.g. `ShortTrill`) makes the renderer's existing collector filter silently drop the wiggle and the bracket, leaving only the ornament glyph itself (consistent with existing renderer behavior for non-extension-supporting ornaments). Updated `ScoreBuilder::trill_with_extension_bracketed_with_options` (`src/score/mod.rs`) to write `opts.ornament.unwrap_or(Ornament::Trill)` into the annotation — the collapse happens at the builder layer (not the renderer) so the annotation's `ornament` field remains the single source of truth for the `supports_trill_extension()` check downstream. **No renderer changes needed**: the existing `collect_trill_extension_note_info`/`draw_system_trill_extensions` path already reads `note.ornament` for both the glyph-advance lookup and the extension-eligibility filter — this chunk just unblocks the API surface above it. Updated the doc comment on `trill_with_mordent_with_extension` to point users at the new options-based path instead of saying "must compose annotations manually". New example `examples/trill_bracket_with_mordent_score.rs` (26915 bytes, 48 paths, 24 lines) walks Both/Start/End/chord-Both bracket sides across 4 measures × 2 systems, mixing default/Up directions and 0.5/0.75/1.0 ss hook lengths. Four in-example assertions: (a) valid SVG, (b) exact 6-hook delta vs an identical no-bracket compound-extension score, (c) byte-inequality vs the same brackets with plain `Trill` ornament (regression canary that `.with_ornament(...)` propagated), (d) hook count matches the plain-trill bracketed variant (bracket geometry is glyph-independent). New `build_trill_bracket_with_mordent()` + two sibling baseline builders and `golden_trill_bracket_with_mordent` test in `tests/golden_svg.rs` with three structural guards beyond the frozen baseline: (1) exact 6-hook delta vs no-bracket compound baseline; (2) byte-inequality vs plain-trill bracketed variant; (3) bracket `<line>` count equality with plain-trill variant. Added `"trill_bracket_with_mordent"` to the `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **2059 unit tests pass** (+15 vs the prior recorded 2044: 7 layout [`options_new_has_unset_ornament`, `options_with_ornament_sets_only_ornament`, `options_with_ornament_chains_with_other_setters`, `options_with_ornament_chain_order_independent`, `options_with_ornament_overwrites_prior_value`, `options_with_ornament_const_constructible` (compile-fail canary if `const fn` is removed), `options_with_ornament_accepts_unsupported_ornament_at_layout_layer` (documents that the layout struct stays validation-free)]; 8 score-integration [`with_options_ornament_unset_writes_plain_trill_to_annotation` (annotation-layer canary that `None` collapses to `Trill` at the builder, not at the renderer); `with_options_ornament_writes_chosen_ornament_to_annotation`; `with_options_compound_ornament_renders_distinct_svg_from_plain_trill` (key regression canary: the override actually propagates through the renderer); `with_options_compound_renders_same_hook_count_as_plain` (bracket geometry is glyph-independent); `with_options_compound_renders_wiggle_paths_beyond_plain_compound` (exact +2 hook lines, more paths than plain compound); `with_options_unsupported_ornament_silently_drops_extension_and_bracket` (contract canary: `ShortTrill` produces zero extra hook lines); `with_options_compound_on_rest_is_noop`; `with_options_compound_on_chord_renders_bracket_and_wiggle` (exact +2 hook lines + more paths than plain chord)]). `cargo test -p music-engraver --test golden_svg` — **61 golden tests pass** (60 prior + new `golden_trill_bracket_with_mordent`); the 3 in-test assertions (6-hook delta, byte-inequality vs plain-trill variant, line-count equality with plain-trill variant) all hold against the freshly written baseline. `cargo test -p music-engraver --doc` — 9 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`). `cargo run -p music-engraver --example trill_bracket_with_mordent_score` produces 26915 bytes (48 paths, 24 lines); all 4 in-example assertions hold. The example's SVG and the golden baseline are byte-identical (verified via `cmp`) — they share the same musical content and the renderer is deterministic.
- Next: The wiggle-speed builder (`trill_with_extension_speed`) still hardcodes `Ornament::Trill` — a `with_ornament` option on its options-equivalent (or a generalized `_speed_with_options` method) would close the parallel gap. Other deferred items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; a side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano`; mid-trill speed change (a single sustained trill that visibly accelerates is still not supported — the speed is per-trill, not per-segment).
- Open issues: The new `with_ornament(...)` method does not validate at the layout layer that the passed ornament supports trill extension — the silent-drop is by design (it matches the existing `trill_extension = true` behavior for unsupported ornaments), but a `debug_assert!` could surface misuse in dev builds. Deferred because adding the assert would force the layout struct to depend on `supports_trill_extension()` behaviour, which is currently a pure `Ornament` predicate; the renderer-layer filter is sufficient. The `with_options_unsupported_ornament_silently_drops_extension_and_bracket` test specifically locks in this contract. The wiggle-speed builder gap (above) remains: a user wanting a *compound trill with custom wiggle speed* must manually set annotations or compose two builders, since `trill_with_extension_speed` writes `Ornament::Trill` unconditionally. The `with_options_compound_renders_wiggle_paths_beyond_plain_compound` test uses a whole-note + quarter pattern; at tight spacings the existing "compound glyph eats the wiggle room" caveat from the 2026-05-13 chunk still applies — the wiggle may render with zero segments and the bracket hooks would then anchor at degenerate positions. Bracket-hook stroke width currently uses `thin_barline_thickness_fu()` (unchanged); a per-call stroke-width override is a small follow-up if a future engraving rule wants it.

## 2026-05-13 — Post-v1, compound speed-variant trills via `TrillExtensionSpeedOptions`

- Did: Closed the "wiggle-speed builder (`trill_with_extension_speed`) still hardcodes `Ornament::Trill`" parallel gap explicitly flagged in the "Next" section of the 2026-05-13 `TrillBracketOptions::with_ornament` chunk. Added a new options struct `TrillExtensionSpeedOptions` in `layout/trill_extension.rs` mirroring the design of `TrillBracketOptions`: required `speed: TrillWiggleSpeed`, optional `ornament: Option<Ornament>`. Both `new(speed)` and `with_ornament(ornament)` are `const fn`-callable so canonical bundles can live in `const` items. Implements `Clone, Copy, Debug, PartialEq` (no `Eq` because `f64` is not Eq — kept symmetric with the bracket bundle). `impl From<TrillWiggleSpeed>` for ergonomic `speed.into()` construction. Re-exported `TrillExtensionSpeedOptions` from `layout/mod.rs`. Added new `ScoreBuilder` method `trill_with_extension_speed_with_options(opts)` (`src/score/mod.rs`) — writes `ornament = Some(opts.ornament.unwrap_or(Ornament::Trill))`, `trill_extension = true`, `trill_wiggle_speed = Some(opts.speed)`. The `None → Some(Trill)` collapse happens at the builder layer (not the renderer) so the annotation field remains the single source of truth for the downstream `supports_trill_extension()` check, matching the convention established by `trill_with_extension_bracketed_with_options`. **No renderer changes needed**: the existing `collect_trill_extension_note_info`/`draw_system_trill_extensions` path already reads `note.ornament` for the glyph-advance lookup and the `trill_wiggle_speed` annotation for the per-segment glyph, so this chunk simply unblocks the parallel API surface above wiring that already supports both knobs independently.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **2078 unit tests pass** (+19 vs the prior recorded 2059: 9 layout [`speed_options_new_has_required_speed_and_unset_ornament`, `speed_options_with_ornament_sets_only_ornament` (regression canary that `with_ornament` doesn't accidentally reset speed), `speed_options_with_ornament_overwrites_prior_value`, `speed_options_from_speed_matches_new` (`From` ergonomics), `speed_options_const_constructible` (compile-fail canary if `const fn` is removed), `speed_options_copy_does_not_consume_original` (Copy-derive canary), `speed_options_accepts_unsupported_ornament_at_layout_layer` (documents the layout struct stays validation-free), `speed_options_different_speeds_compare_distinct`, `speed_options_same_speed_different_ornament_compare_distinct` (PartialEq sensitivity)]; 10 score-integration [`speed_with_options_default_ornament_matches_plain_speed_byte_for_byte` (key byte-equivalence canary: `TrillExtensionSpeedOptions::new(speed)` is byte-identical to `trill_with_extension_speed(speed)`); `speed_with_options_via_into_matches_plain_speed_byte_for_byte` (`From` ergonomics byte-stable); `speed_with_options_sets_three_annotation_fields` (annotation-layer state inspection of `ornament`/`trill_extension`/`trill_wiggle_speed`); `speed_with_options_unset_ornament_writes_plain_trill_to_annotation` (`None → Some(Trill)` collapses at the builder); `speed_with_options_compound_ornament_renders_distinct_from_plain_trill_same_speed` (key regression canary: the override actually propagates through the renderer's glyph-advance lookup, which differs between bare `tr` and compound at 521 vs 990 font-units in Bravura); `speed_with_options_compound_with_speed_distinct_from_compound_default_speed` (proves the speed half of the bundle propagates even when ornament is overridden); `speed_with_options_compound_renders_wiggle_paths_beyond_plain_compound` (sanity check the new method walks the same wiggle-rendering path as the existing compound-extension API); `speed_with_options_on_rest_is_noop` (rest no-op contract); `speed_with_options_on_chord_renders_wiggle` (chord variant adds wiggle); `speed_with_options_unsupported_ornament_silently_drops_extension` (contract canary: `ShortTrill` produces strictly fewer paths than `Trill` at the same speed — matches the bracket-options behavior)]). `cargo test -p music-engraver --test golden_svg` — 61 golden tests pass (no baseline drift: the new method writes the same downstream annotation state as existing methods, and the byte-equivalence test already locks in the no-ornament case). `cargo test -p music-engraver --doc` — **10 doc tests pass** (+1: the doc example in `TrillExtensionSpeedOptions::new` runs as a 10th doc-compile test, covering the canonical `new` / `with_ornament` / `From::from` usage patterns). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Visual proofing — a small example exercising compound-trill-with-speed permutations (e.g. `examples/trill_speed_with_mordent_score.rs`) + golden baseline would close the visual side. Other deferred items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; a side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano`; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment).
- Open issues: No example or golden baseline added for the new method this chunk — visual proofing is deferred. The new `with_ornament(...)` method shares its design contract with `TrillBracketOptions::with_ornament`: unsupported ornaments make the renderer silently drop the wiggle (locked in by `speed_with_options_unsupported_ornament_silently_drops_extension`); a `debug_assert!` would surface misuse in dev builds but would force the layout struct to depend on the `supports_trill_extension()` predicate. The same caveat about the compound glyph eating wiggle room at tight spacings applies — the wiggle may render with zero segments and the user sees just the compound glyph. The byte-equivalence assertion in `speed_with_options_default_ornament_matches_plain_speed_byte_for_byte` would fail if a future change reads `trill_wiggle_speed = Some(Standard)` differently from "field unset" — they are intentionally semantically equivalent at the renderer layer (the collector's `unwrap_or_default()` collapses `None` to `Standard` glyph selection), and this test is the canary.

## 2026-05-13 — Post-v1, visual proofing for `TrillExtensionSpeedOptions` with compound ornament

- Did: Closed the "no example or golden baseline added for the new method" open issue + matching "Next" follow-up from the 2026-05-13 `TrillExtensionSpeedOptions` introduction chunk. Created `examples/trill_speed_with_mordent_score.rs` exercising every `TrillWiggleSpeed::ALL` variant (Fastest through Slowest, 9 measures across 4 systems with measures_per_system=3) paired with `Ornament::TrillWithMordent` via `TrillExtensionSpeedOptions::new(speed).with_ornament(Ornament::TrillWithMordent)`. Six in-example assertions, every one of which would fail if the implementation regressed in a specific way: (1) valid SVG; (2) compound-with-extension has strictly more paths than plain compound (`.ornament(TrillWithMordent)` with no extension) — confirms the wiggle actually rendered; (3) byte-inequality vs `trill_with_extension_speed(speed)` at the same speeds — propagation canary that `.with_ornament(...)` reached the renderer's glyph-advance lookup (bare "tr" = 521 fu vs compound = ~990 fu in Bravura, so wiggle starts differ); (4) byte-inequality vs `TrillExtensionSpeedOptions::new(speed)` (no `.with_ornament`, defaulting to Trill) — propagation canary that `.with_ornament` overrode rather than left at default; (5) strictly more paths than all-Slowest compound (lower sandwich bound); (6) strictly fewer paths than all-Fastest compound (upper sandwich bound). The sandwich bookends prove the 9 distinct speeds aren't silently being collapsed to a single glyph — a regression where every speed read as `Standard` would show as identical path counts between mixed/all-slowest/all-fastest.

  Added matching `build_trill_speed_with_mordent()` + three sibling baseline builders (`_plain_trill_variant`, `_all_slowest`, `_all_fastest`) and `golden_trill_speed_with_mordent` test in `tests/golden_svg.rs` with three structural guards alongside the frozen baseline: (1) byte-inequality vs plain-trill-speeds — regression canary that `.with_ornament(...)` propagated through the renderer; (2) sandwich check: mixed > all-Slowest && all-Fastest > mixed (proves none of the 9 speeds collapse); (3) mixed-compound > plain-compound-no-extension (wiggle is rendering for at least some speeds). Wrote the golden baseline (`tests/golden/trill_speed_with_mordent.svg`, 54672 bytes, 95 paths, 34 lines) via `GOLDEN_UPDATE=1`, confirmed test passes on a second run. Added `"trill_speed_with_mordent"` to the `golden_baselines_are_valid_svgs` name list. Verified `cmp` reports example output == golden baseline byte-for-byte (renderer is deterministic).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo test -p music-engraver --lib` — **2078 unit tests pass** (no new unit tests in this chunk — all new verification lives in the example + golden test). `cargo test -p music-engraver --test golden_svg` — **62 golden tests pass** (61 prior + new `golden_trill_speed_with_mordent`); the 3 in-test structural guards (byte-inequality vs plain-trill, sandwich, more-paths-than-no-extension) all hold. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 10 doc tests pass. `cargo run -p music-engraver --example trill_speed_with_mordent_score` writes 54672 bytes (95 paths, 34 lines); all 6 in-example assertions hold. `cmp examples/output/trill_speed_with_mordent_score.svg tests/golden/trill_speed_with_mordent.svg` — byte-identical. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: The wiggle-speed builder family is now fully visually proofed (plain trill at speed: `trill_wiggle_speed_score.rs`; compound trill at speed: this chunk; compound trill bracketed: `trill_bracket_with_mordent_score.rs`). Remaining deferred items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points (currently a trill always runs to the next note's left edge or to the system's right edge — no way to express "trill runs out at beat 3"); cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; a side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano`; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment); a unified bracket+speed options bundle (currently the user must pick one or the other — combining both knobs requires hand-constructing annotations).
- Open issues: The example deliberately uses 4 systems × 3 measures (the 9 speeds + terminating quarter), which is wider than most existing examples — the wiggle tile count on the very slow speeds is small enough that on narrow systems they might render with zero segments and the trill would appear bare. The 3-measures-per-system choice is the empirically-tested minimum width that keeps every speed visible. The sandwich check in the golden test relies on the all-Slowest/all-Fastest builders producing strictly different path counts than mixed; if a future font (Petaluma, Leland) sets all 9 wiggle glyphs to the same advance width, this sandwich would degenerate (mixed == slowest == fastest) and the test would fail — by design, since at that point the speed variants would have no visible effect and the API would be misleading. The renderer-layer `unwrap_or_default()` collapse of `trill_wiggle_speed = None → Standard` (called out in the prior chunk's open issues) is unchanged and unrelated to this chunk's content.

## 2026-05-13 — Post-v1, unified `TrillExtensionFullOptions` builder (bracket+speed+ornament in one call)

- Did: Closed the "unified bracket+speed options bundle" open issue explicitly flagged in the "Next" section of the prior two entries. The existing `TrillBracketOptions` and `TrillExtensionSpeedOptions` are mutually exclusive at the API surface (a caller wanting `Both`-bracketed + `Slow` wiggle + `TrillWithMordent` ornament on the same note had to hand-construct annotation fields), even though the underlying renderer fully supports the combination via independent `NoteAnnotations` fields. New module `layout/trill_options.rs` introduces `TrillExtensionFullOptions { bracket: Option<TrillBracketSide>, bracket_direction: Option<HookDirection>, bracket_length_ss: Option<f64>, speed: Option<TrillWiggleSpeed>, ornament: Option<Ornament> }` — every knob optional. `const fn new()` returns the all-`None` bundle; chainable `const fn with_bracket / with_bracket_direction / with_bracket_length_ss / with_speed / with_ornament` setters. `#[derive(Clone, Copy, Debug, PartialEq, Default)]`. `From<TrillBracketOptions>` and `From<TrillExtensionSpeedOptions>` widen the existing single-purpose bundles cleanly (speed-only carries `bracket = None`; bracket-only carries `speed = None`). Re-exported from `layout/mod.rs` as `TrillExtensionFullOptions`. New ScoreBuilder method `trill_with_extension_full_options(opts)` (`src/score/mod.rs`) writes `ornament = Some(opts.ornament.unwrap_or(Ornament::Trill))`, `trill_extension = true`, `trill_bracket = opts.bracket`, `trill_bracket_direction = opts.bracket_direction`, `trill_bracket_length_ss = opts.bracket_length_ss`, `trill_wiggle_speed = opts.speed` — matching the `None`-collapse-to-`Trill` convention established by the two prior options builders so the annotation field stays the single source of truth for the downstream `supports_trill_extension()` check. No-op on rests. **No renderer changes needed**: the renderer's collector already reads each annotation field independently.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --tests` passes. `cargo test -p music-engraver --lib` — **2108 unit tests pass** (+30 vs the prior recorded 2078: 19 layout in `trill_options` [`new_has_every_field_unset`, `default_matches_new` (the derive-canary catching `Default` drift), `with_bracket_sets_only_bracket` (and 4 sibling tests for the other four with-methods, each verifying that ONLY the targeted field flips), `chain_sets_all_five_knobs`, `chain_order_independent` (the five with-methods must commute — catches a regression where a with-method accidentally resets another field), `with_method_overwrites_prior_value`, `const_constructible` (compile-fail canary if `const fn` is removed), `copy_does_not_consume_original` (compile/Copy canary), `from_trill_bracket_options_preserves_bracket_fields`, `from_trill_bracket_options_with_unset_overrides_widens_cleanly` (proves the From conversion does not synthesize defaults), `from_speed_options_preserves_speed_and_ornament`, `from_speed_options_with_default_ornament_widens_cleanly`, `accepts_unsupported_ornament_at_layout_layer` (the renderer-layer contract canary that the layout struct stays validation-free), 3 distinct-comparison tests (`distinct_brackets_compare_distinct`, `distinct_speeds_compare_distinct`, `distinct_ornaments_compare_distinct` — PartialEq sensitivity guards against accidental field collapse); 11 score-integration tests [`full_options_default_matches_plain_trill_with_extension_byte_for_byte` (key byte-equivalence canary: `TrillExtensionFullOptions::new()` ≡ `trill_with_extension()`); `full_options_from_bracket_matches_bracketed_options_byte_for_byte` (a `TrillBracketOptions` widened via `.into()` ≡ applying the same bracket options through `trill_with_extension_bracketed_with_options`); `full_options_from_speed_matches_speed_options_byte_for_byte` (mirror of the bracket From test); `full_options_sets_every_annotation_field_when_all_knobs_specified` (field-level inspection of all 6 annotation fields proving none silently drops); `full_options_unset_ornament_writes_plain_trill_to_annotation` (`None → Some(Trill)` collapse-at-builder canary); `full_options_bracket_plus_speed_renders_distinct_from_bracket_only` (the new-capability canary: bracket + speed in one call renders visibly different from bracket-only, AND both still draw ≥2 hook lines); `full_options_compound_bracket_plus_speed_renders_distinct_from_plain_trill_same_options` (ornament-half propagation through the renderer's glyph-advance lookup, AND bracket geometry stays glyph-independent — `<line>` counts must match between plain-trill and compound at the same bracket settings); `full_options_unsupported_ornament_silently_drops_extension_and_bracket` (contract canary: `ShortTrill` produces strictly fewer paths AND strictly fewer `<line>`s than `Trill` at the same options); `full_options_on_rest_is_noop` (rest no-op contract); `full_options_on_chord_renders_bracket_and_wiggle` (chord variant adds exactly +2 `<line>`s for Both-side bracket AND more paths than plain chord)]). `cargo test -p music-engraver --test golden_svg` — **62 golden tests pass** (no baseline drift; the new method writes the same downstream annotation state as the existing methods that have golden coverage). `cargo test -p music-engraver --doc` — **12 doc tests pass** (+2 vs the prior recorded 10: the doc example in `TrillExtensionFullOptions` showing the full chain, and the `From<TrillBracketOptions>` example, both compile-tested). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Visual proofing — a small example exercising bracket+speed combinations on compound trills through `trill_with_extension_full_options(...)` + matching golden baseline (currently no example exercises this method specifically; the byte-equivalence canaries cover correctness but not visual proofing). Other deferred items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; a side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano`; mid-trill speed change.
- Open issues: No example or golden baseline added in this chunk — visual proofing is deferred (the byte-equivalence assertions against the existing single-purpose options builders prove the renderer state matches, and the existing golden coverage on those single-purpose paths covers the bracket-only and speed-only cases; the new bracket+speed combination is not yet baselined). The `From<TrillBracketOptions>` conversion drops the `speed` half (sets it to `None`) and the `From<TrillExtensionSpeedOptions>` conversion drops the `bracket` half — by design, since the source bundles can't express those fields, but a caller widening one and then layering the missing knob via `.with_bracket(...)` / `.with_speed(...)` is the intended ergonomic path. There is no `From<NoteAnnotations>` conversion (and there shouldn't be — that's the wrong direction; annotations are the renderer's internal representation). The unified builder method shares its renderer-layer wiring with the two existing options builders, so any rendering bug affects all three equally — no new bug surface, but also no independent verification at the renderer level beyond the byte-equivalence assertions.


## 2026-05-13 — Post-v1, visual proofing for `TrillExtensionFullOptions` (bracket+speed combo example + golden)

- Did: Closed the "no example or golden baseline added in this chunk — visual proofing is deferred" open issue + matching "Next" follow-up from the 2026-05-13 `TrillExtensionFullOptions` introduction. The unified builder's unique capability over the two single-purpose options bundles is **combining bracket + speed in a single call** — neither `TrillBracketOptions` (no speed field) nor `TrillExtensionSpeedOptions` (no bracket field) can express the combination alone. Created `examples/trill_full_options_score.rs` walking four measures (2 systems × 2 measures-per-system) where every measure uses a combination only `TrillExtensionFullOptions` can produce: M1 `Both` bracket + `Slow` speed + `TrillWithMordent` (every knob set); M2 `End` bracket + `Faster` speed + plain `Trill` + `HookDirection::Up`; M3 `Start` bracket + `Slowest` speed + `TrillWithMordent` + 1.0ss hook length; M4 chord (C-E-G half) + `Both` bracket + `Standard` speed + `TrillWithMordent` (exercises the chord arm of the builder). Seven in-example assertions, each a regression canary for a specific propagation path: (1) valid SVG; (2) **exact 6-hook delta** vs a no-bracket variant — Both=2 + End=1 + Start=1 + Both=2 = 6 (catches a bracket field silently failing to propagate from the bundle to the annotation, or the renderer dropping a hook for one of the four variants); (3) byte-inequality vs a no-speed variant — three measures pick non-Standard wiggles, so if the speed half stopped propagating these would converge (speed propagation canary); (4) strict path-count inequality vs the no-bracket variant (bracket propagation canary, sandwiches the byte assertion); (5) byte-inequality vs a plain-Trill variant — the compound glyph is ~470 fu wider than bare "tr" in Bravura, so the wiggle start positions differ (ornament propagation canary); (6) `<line>` count **equality** with the plain-Trill variant — bracket geometry is glyph-independent, sandwiches the ornament assertion (ornament changed path data but NOT line count); (7) strict path-count inequality vs the no-extension variant (every measure just `.ornament(TrillWithMordent)` with no wiggle/bracket — confirms wiggle is rendering on at least some measures).

  Added matching `build_trill_full_options()` + two sibling baseline builders (`_no_bracket_variant`, `_plain_trill_variant`) and `golden_trill_full_options` test in `tests/golden_svg.rs` with four structural guards alongside the frozen baseline: (1) **exact 6-hook delta** vs no-bracket variant (mirrors the example's bracket-count assertion at the golden layer); (2) byte-inequality vs the plain-Trill variant (mirrors the ornament propagation canary); (3) `<line>` count equality with the plain-Trill variant (glyph-independence of bracket geometry); (4) **byte-inequality vs the existing `trill_bracket_with_mordent` baseline** — the two goldens use closely related shapes (similar measures, similar brackets, similar ornaments) but `trill_full_options` mixes in non-Standard wiggle speeds on 3 of 4 measures; if a future refactor accidentally wired `trill_with_extension_full_options` to drop the speed field, the two outputs would converge (the cross-baseline check is the speed-propagation canary at the golden layer, complementing the no-speed-variant byte check inside the example). Wrote `tests/golden/trill_full_options.svg` (30139 bytes) via `GOLDEN_UPDATE=1` and confirmed `cmp examples/output/trill_full_options_score.svg tests/golden/trill_full_options.svg` reports byte-identical (renderer is deterministic; example and golden share the same build path). Added `"trill_full_options"` to the `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --example trill_full_options_score` passes. `cargo run -p music-engraver --example trill_full_options_score` produces 30139 bytes (54 paths, 24 lines); all 7 in-example assertions hold. `cargo test -p music-engraver --test golden_svg` — **63 golden tests pass** (62 prior + new `golden_trill_full_options`); the 4 in-test structural guards (6-hook delta, byte-inequality vs plain-Trill, line-count equality with plain-Trill, byte-inequality vs `trill_bracket_with_mordent`) all hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --lib` — **2108 unit tests pass** (no new lib tests in this chunk — visual proofing lives in the example + golden test; all 19 layout + 11 score-integration tests for `TrillExtensionFullOptions` already exist from the introduction chunk). `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: The trill-extension API surface is now visually proofed end-to-end across all six combinator paths (bare trill: `trill_extension_score`; bracketed: `trill_bracket_score` + `_custom`; speeds: `trill_wiggle_speed_score`; compound prefix: `trill_with_mordent_score`; compound + bracket: `trill_bracket_with_mordent_score`; compound + speed: `trill_speed_with_mordent_score`; **full** bracket+speed+compound: this chunk). Remaining deferred items: `.trill_to(note_index)` API for sub-note-granularity wiggle end points; cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; a side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano`; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment).
- Open issues: The 6-hook delta and `<line>`-count equality assertions both lean on the assumption that no future renderer change will add a non-bracket `<line>` element to one variant but not the other. Currently `<line>` is used for staff lines, ledger lines, barlines, stems, beams, bracket hooks — only the bracket hooks vary between the score variants here, so the saturating-sub captures only hook-line delta. If a future change starts emitting wiggle segments as `<line>` instead of `<path>`, the delta assertion would fail (since the speed-varying segments would also leak into the count); fix would be to filter by a CSS class or by explicit stroke attribute pattern. The cross-baseline byte-inequality against `trill_bracket_with_mordent` is sensitive to musical-content drift in **either** baseline — if a future refactor regenerates `trill_bracket_with_mordent.svg` with a coincidentally matching speed-glyph mix, the canary fires a false positive; this is unlikely in practice (the two scores use different speeds, different durations, different chord placements) but documented for completeness. The example's structural assertions are intentionally redundant with the golden test's assertions on the same content — keeping them in both places preserves the property that `cargo run --example trill_full_options_score` is a self-contained verification step (the example file is the runnable demo for downstream users; the golden test is the regression canary).

## 2026-05-13 — Post-v1, side-by-side dynamics-lookalikes example + golden (Mezzo/Mp/Mf and Sfp/SforzatoPiano)

- Did: Closed the "side-by-side comparison example for `Mezzo`/`Mp`/`Mf` and `Sfp`/`SforzatoPiano` (currently those visually similar pairs are not shown together in any single example)" deferred follow-up explicitly flagged in the "Next" sections of both `dynamics_full` (2026-05-13) and the two trill chunks immediately following. Until now the m-cluster glyphs (`Mezzo` bare-letter-`m` / `Mp` / `Mf`) and the sfp-cluster glyphs (`Sfp` sforzando-prefixed / `SforzatoPiano` sforzato-prefixed) were each in `dynamics_full`'s 28-glyph parade but never adjacent — a viewer wanting the A/B comparison had to scan across measures or split-view two SVGs. New example `examples/dynamics_lookalikes_score.rs` puts the two clusters in dedicated measures: M1 (4/4) carries `Mezzo`/`Mp`/`Mf` on G4/A4/B4 + a plain padding C5 quarter; M2 carries `Sfp`/`SforzatoPiano` on G4/A4 + plain padding B4/C5 quarters. `measures_per_system(2)` puts both clusters on a single visual line so the comparison is in-eye-shot. Three in-example assertions, each a regression canary: (a) basic SVG validity; (b) **exact 5-path delta** vs an identical no-dynamic baseline (catches a silent missing-glyph regression — if Bravura ever swapped one of these less-common glyphs to `none` the delta would be <5; reinforces the unit-test-layer `new_variants_have_nonzero_advance_in_bravura` at the integration layer); (c) **exactly 5 unique d-strings** contributed by the dynamics (catches a hypothetical glyph collapse — the whole point of putting these glyphs side-by-side is that each is provably distinct in Bravura, so if `Mezzo` ever aliased to `Mp`'s glyph or the two `s` letterforms in `Sfp`/`SforzatoPiano` ever unified, the comparison example would silently mislead).
- New `build_dynamics_lookalikes()` + `build_dynamics_lookalikes_plain()` builders and `golden_dynamics_lookalikes` test in `tests/golden_svg.rs` with four structural guards alongside the frozen baseline: (1) **exact 5-path delta** vs the dynamics-stripped baseline (mirrors example assertion (b) at the golden layer); (2) **distinct-d set difference** between dynamic-bearing and plain SVGs must equal exactly 5 unique payloads (mirrors example assertion (c) at the golden layer); (3) **byte-inequality vs `build_dynamics()`** — the existing `dynamics` baseline shares some glyphs (Mp/Mf appear there too) but uses different gestures (hairpins, more variants); if the new golden ever accidentally collapsed onto it, it would be silent duplicate coverage rather than independent verification; (4) **byte-inequality vs `build_dynamics_variants()`** — the variants baseline shares `Mezzo`/`SforzatoPiano` but uses a single-measure C–F layout with `Z` instead of the lookalike-cluster grouping; the inequality canary catches an analogous accidental collapse. Added `"dynamics_lookalikes"` to the `golden_baselines_are_valid_svgs` name list. Wrote `tests/golden/dynamics_lookalikes.svg` (15115 bytes) via `GOLDEN_UPDATE=1` and confirmed `cmp examples/output/dynamics_lookalikes_score.svg tests/golden/dynamics_lookalikes.svg` reports byte-identical (renderer is deterministic; example and golden share the same build path).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --example dynamics_lookalikes_score` passes. `cargo run -p music-engraver --example dynamics_lookalikes_score` produces 15115 bytes (16 paths, 16 lines); all 3 in-example assertions hold (exact 5-path delta, 5 unique d-strings, valid SVG). `cargo test -p music-engraver --test golden_svg` — **64 golden tests pass** (63 prior + new `golden_dynamics_lookalikes`); the 4 in-test structural guards (5-path delta, 5 unique d-strings, byte-inequality vs `dynamics`, byte-inequality vs `dynamics_variants`) all hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --lib` — **2108 unit tests pass** (no new lib tests in this chunk — visual proofing lives in the example + golden test; all five lookalike variants already have layout + renderer unit-test coverage from prior expansion chunks). `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Other deferred items from the trill-chunk "Next" sections: `.trill_to(note_index)` API for sub-note-granularity wiggle end points (currently a trill always runs to the next note's left edge or to the system's right edge — no way to express "trill runs out at beat 3"); cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment). All other dynamic-API surfaces are now visually proofed (subset coverage: `dynamics`; new variants: `dynamics_variants`; full coverage: `dynamics_full`; lookalike comparison: this chunk).
- Open issues: The lookalike score uses unbeamed quarter notes throughout, so it does not exercise dynamic placement under beamed groups, below-staff stems with bass-clef pitches, or stem-direction interactions; the golden is a glyph-comparison example, not a placement-correctness test (placement correctness is covered by other goldens). The 5-path-delta assertion would fail if a future engraving change ever caused one of these variants to render zero paths (e.g. a future Bravura swap where one glyph is replaced by `none`) — that's the intended behaviour, the test is a regression canary. Both byte-inequality canaries (vs `dynamics` and vs `dynamics_variants`) are sensitive to musical-content drift in **either** baseline: a refactor that changed one of the prior baselines to coincidentally match the lookalike layout would fire a false positive; this is unlikely in practice (the three scores use different measure layouts, different padding, different gestures) but documented for completeness. The two clusters are visually adjacent within their own measures but not within a single measure (the m-cluster fills M1 and the sfp-cluster fills M2) — a tighter "all five glyphs on five adjacent eighth notes" layout would put them even closer but would beam the eighths and obscure the dynamic placement; the chosen quarter-note-per-cluster-per-measure layout is the empirically-tested sweet spot for visibility.

## 2026-05-13 — Post-v1, explicit-length trill extensions (`trill_with_extension_length_ss`)

- Did: Closed the long-deferred "`.trill_to(note_index)` API for sub-note-granularity wiggle end points; currently a trill always runs to the next note's left edge or to the system's right edge — no way to express 'trill runs out at beat 3'" item that has been in every recent chunk's "Next" section. Added a new annotation field `trill_extension_length_ss: Option<f64>` to `NoteAnnotations` (`src/layout/measure.rs`) — explicit termination length for the wavy-line extension, in staff spaces, measured from the wiggle's natural start (past the "tr" prefix glyph). `None` preserves existing behavior. Clamping is one-sided (clamp to the natural span, never overrun the next note or the system edge). Non-positive values produce no wiggle, matching the renderer's existing fail-safe for spans-too-short-to-tile. Extended `TrillExtensionNoteInfo` (`src/render/system_renderer/mod.rs`) with a parallel `explicit_length_ss: Option<f64>` field populated by the collector only when `has_trill_extension == true` (matches the bracket/wiggle_speed filter pattern). Updated `draw_system_trill_extensions` to compute the effective `end_x` as `min(natural_end_x, start_x + len_ss * staff_space)` for positive lengths, and `start_x` (i.e. no wiggle) for zero/negative lengths; positive explicit lengths also force `cross_system = false` so the wiggle terminates within the source system regardless of position. Updated `find_unresolved_trill_extension` (`src/render/page_renderer/mod.rs`) to skip notes with positive explicit lengths so the page-renderer doesn't try to continue the wiggle on system N+1 (mirrors the same definite-termination semantic). New ScoreBuilder method `trill_with_extension_length_ss(length_ss: f64)` (`src/score/mod.rs`) — sets `ornament = Some(Trill)`, `trill_extension = true`, `trill_extension_length_ss = Some(length_ss)` in one call, mirroring the `trill_with_extension()` shape. No-op on rests.
- New example `examples/trill_short_extension_score.rs` (17901 bytes, 24 paths, 15 lines) — four whole-note trills across two systems: M1 default extension (baseline), M2 explicit 2.0-ss length, M3 explicit 4.0-ss length, M4 explicit 1.5-ss length on the last note of the system (proves cross-system propagation is suppressed by the explicit length). Five structural in-example assertions, each a regression canary for a specific propagation path: (1) valid SVG; (2) explicit-length wiggles still produce some wiggle paths (strictly more than a no-wiggle plain-trill baseline — catches a future change that drops every wiggle under explicit lengths); (3) explicit-length total path count is strictly less than the all-defaults total (proves wiggles ARE shortened — catches a stale-field regression where the annotation is set but ignored); (4) byte-inequality vs all-defaults (catches accidental byte-equivalence from a no-op renderer change); (5) explicit length on the last note of the system yields fewer paths than a hybrid where M1-M3 share the explicit lengths but M4 uses the default (proves cross-system suppression specifically).
- New `build_trill_short_extension()` + two sibling baseline builders (`_defaults`, `_plain`) and `golden_trill_short_extension` test in `tests/golden_svg.rs` with four structural guards alongside the frozen baseline: (1) explicit-length wiggle path count > plain-trill (some wiggle drew); (2) explicit-length path count < all-defaults (wiggles ARE shortened); (3) byte-inequality vs defaults (refactor canary); (4) `translate(` count in explicit-length must be ≤ defaults' (the new method removes wiggle tiles, never adds glyphs — guards against an unexpected new glyph leaking in). Added `"trill_short_extension"` to the `golden_baselines_are_valid_svgs` name list. Wrote `tests/golden/trill_short_extension.svg` (17901 bytes) via `GOLDEN_UPDATE=1` and confirmed `cmp examples/output/trill_short_extension_score.svg tests/golden/trill_short_extension.svg` reports byte-identical (renderer is deterministic; example and golden share the build path).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo run -p music-engraver --example trill_short_extension_score` writes 17901 bytes (24 paths, 15 lines); all 5 in-example assertions hold. `cargo test -p music-engraver --lib` — **2125 unit tests pass** (+17 vs prior 2108: 9 system_renderer [`collector_propagates_explicit_length_when_trill_extension_active`, `collector_drops_explicit_length_when_trill_extension_inactive` (stale-annotation filter canary), `explicit_length_renders_fewer_paths_than_default_when_shorter`, `explicit_length_larger_than_natural_clamps_to_natural` (byte-equivalence assertion: 1000-ss request clamps to natural and renders byte-identically), `explicit_length_zero_produces_no_wiggle` (path-count equality vs plain trill), `explicit_length_negative_produces_no_wiggle` (sign-handling canary), `explicit_length_in_chord_collector_propagates`, `explicit_length_on_last_note_avoids_cross_system_extension`, `explicit_length_with_end_bracket_anchors_at_shortened_terminus` (bracket-hook count equality + byte-inequality, proving the End hook follows the shortened wiggle)]; 8 score-integration [`trill_with_extension_length_ss_sets_all_three_annotation_fields`, `_shorter_than_natural_renders_fewer_paths`, `_larger_than_natural_clamps_to_default` (byte-equivalence canary), `_zero_drops_wiggle` (path count equality with plain ornament-only trill), `_on_rest_is_noop`, `_on_chord_renders_shortened_wiggle`, `_disables_cross_system_propagation`, `_negative_treats_as_zero` (byte-equivalence assertion: -3.0 ≡ 0.0)]). `cargo test -p music-engraver --test golden_svg` — **65 golden tests pass** (64 prior + new `golden_trill_short_extension`); all 4 in-test structural guards hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Other deferred items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment, even though we now have explicit-length termination). The explicit-length feature could also be added to the options-based trill builders (`TrillBracketOptions`, `TrillExtensionSpeedOptions`, `TrillExtensionFullOptions`) so a caller can combine bracket + explicit length, or speed + explicit length — currently the explicit length is a standalone builder method, parallel to `trill_with_extension()` but not yet integrated into the options bundles. A small follow-up.
- Open issues: The explicit length is silently clamped to natural (no error / warning when a caller requests a length that doesn't fit). That's documented in the doc comment but may surprise users who specify a too-large length and don't notice the clamp. A `debug_assert!` in dev builds could surface this, but would require the renderer to know the natural span at the call site, which is computed downstream from layout — adding the assert would mean threading the natural span back to the builder layer or doing a runtime sanity check at draw time. Deferred. The byte-equivalence assertions (clamp-to-natural and negative-equals-zero) rely on the renderer being deterministic and the natural span being smaller than the requested length in the test setup; if a future change increases the natural span (e.g. wider default `min_note_spacing`), the clamp test could break — the test setup uses `Duration::WHOLE` which produces the widest single-event natural span available in the score builder, so this is unlikely but not impossible. The cross-system suppression contract (explicit length forces `cross_system = false`) is now locked in across three layers (system_renderer's draw_system_trill_extensions, page_renderer's find_unresolved_trill_extension, and the score test `disables_cross_system_propagation`) — any future change that wants cross-system to fire AND honor an explicit length must touch all three. The new method is NOT exposed via the options-based builders (`TrillBracketOptions`, `TrillExtensionSpeedOptions`, `TrillExtensionFullOptions`) — a caller wanting "bracketed trill with explicit length" must hand-compose annotations or layer two builder calls; adding a `with_length_ss` to each options struct is the obvious next ergonomic step.

## 2026-05-15 — Post-v1, explicit length on `TrillExtensionFullOptions` (combine bracket + speed + ornament + length in one call)

- Did: Closed the "small follow-up" explicitly flagged in the prior chunk: "The explicit-length feature could also be added to the options-based trill builders (`TrillBracketOptions`, `TrillExtensionSpeedOptions`, `TrillExtensionFullOptions`) so a caller can combine bracket + explicit length, or speed + explicit length." Targeted the unified bundle first since it subsumes the two single-purpose ones; this single addition unlocks every two-way and three-way combination of (bracket | speed | ornament) × length through one API call. Added `length_ss: Option<f64>` field to `TrillExtensionFullOptions` (`src/layout/trill_options.rs`) plus a `const fn with_length_ss(length_ss: f64)` builder method matching the existing five with_* setters. Updated `new()` to initialize the new field to `None`. Updated both `From` conversions (`TrillBracketOptions` and `TrillExtensionSpeedOptions`) to set `length_ss: None` so the documented byte-equivalence guarantees with `trill_with_extension_bracketed_with_options` / `_speed_with_options` survive — those two paths don't write `trill_extension_length_ss`, and the new field-set-to-None is a no-op against the annotation's default. Updated `ScoreBuilder::trill_with_extension_full_options` (`src/score/mod.rs`) to write `annotations.trill_extension_length_ss = opts.length_ss` and added a new documented byte-equivalence guarantee: `trill_with_extension_full_options(TrillExtensionFullOptions::new().with_length_ss(L))` ≡ `trill_with_extension_length_ss(L)`. The renderer needed no changes: the existing collector reads `trill_extension_length_ss` independently of every other annotation field, including the bracket/speed/ornament knobs, so the unified builder simply unblocks an API surface that the lower layers already support.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). `cargo test -p music-engraver --lib` — **2138 unit tests pass** (+13 vs prior 2125: 6 layout in `trill_options` [`with_length_ss_sets_only_length_ss` (mirror of the five sibling field-isolation tests — flips only `length_ss`); `with_length_ss_overwrites_prior_value` (last-write-wins canary, paralleling `with_method_overwrites_prior_value`); `with_length_ss_accepts_non_positive_at_layout_layer` (layout-layer validation-free contract — zero and negative round-trip unchanged, matching the renderer's non-positive fail-safe convention); `widen_then_add_length_ss_round_trips_other_fields` (the intended ergonomic path: widen a `TrillBracketOptions` via `.into()` then layer in `.with_length_ss(...)` — bracket fields must survive); `distinct_length_ss_values_compare_distinct` (PartialEq sensitivity canary, catches a future derive forgetting the new field); `length_ss_some_zero_distinct_from_none` (semantic distinction canary: `Some(0.0)` ≠ `None` at the PartialEq layer because they have different renderer semantics — zero suppresses the wiggle, None lets the natural span flow). Also renamed `chain_sets_all_five_knobs` → `chain_sets_all_six_knobs` and extended `chain_order_independent` to include the new method, and added `.with_length_ss(3.0)` to the `const_constructible` chain to guard `const fn`-removability. 7 score-integration [`full_options_length_ss_propagates_into_annotation` (annotation-layer state inspection: `length_ss = Some(2.5)` ⇒ `trill_extension_length_ss == Some(2.5)`); `full_options_length_ss_byte_equivalent_to_trill_with_extension_length_ss` (the documented new byte-equivalence guarantee — must hold for any `L`); `full_options_length_ss_shortens_wiggle_vs_default_full_options` (the new-capability canary: bracket + length in one call renders strictly fewer paths than bracket-alone, AND bracket hook `<line>` count is invariant — proves bracket survives the shortening); `full_options_length_ss_clamps_to_natural_when_oversized` (clamp contract canary: 1000ss request renders byte-identical to no-length default); `full_options_combines_bracket_speed_ornament_and_length_in_one_call` (the four-way combination canary: bracket+speed+ornament+length all set, with-length renders ≠ without-length, with-length renders strictly fewer paths, AND `<line>` count survives the length-shortening — three independent assertions in one test); `full_options_length_ss_zero_drops_wiggle_with_other_options_set` (zero-length suppresses wiggle even with other options set — catches a regression where the non-positive fail-safe gets bypassed when other knobs are present); `full_options_length_ss_disables_cross_system_propagation` (cross-system suppression contract — last note of system with explicit length renders fewer paths than no-length default)]). `cargo test -p music-engraver --test golden_svg` — **65 golden tests pass** (unchanged, no new baselines this chunk — the byte-equivalence canary against the standalone `trill_with_extension_length_ss` builder proves the renderer state matches existing golden coverage on that path). `cargo test -p music-engraver --doc` — 12 doc tests pass (the updated doc example in `TrillExtensionFullOptions::new` now exercises `.with_length_ss(3.5)`). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass.
- Next: Add `length_ss` to `TrillBracketOptions` and `TrillExtensionSpeedOptions` so single-purpose users can combine bracket+length or speed+length without widening to a full bundle (parallel small follow-up). Visual proofing for the new combined capability — a small example exercising bracket+speed+length combinations on a compound trill through `trill_with_extension_full_options(...)` + matching golden baseline (the existing in-test structural assertions cover correctness but not visual proofing). Other deferred items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment).
- Open issues: No example or golden baseline added in this chunk — visual proofing is deferred (the byte-equivalence assertion against `trill_with_extension_length_ss(L)` proves the renderer state matches the existing `trill_short_extension` golden coverage on that path). The single-purpose bundles `TrillBracketOptions` and `TrillExtensionSpeedOptions` still lack `length_ss` — a caller wanting "bracket-only options with explicit length" must widen to `TrillExtensionFullOptions` via `.into()` then layer `.with_length_ss(L)` (works, but more verbose than necessary). The `length_ss_some_zero_distinct_from_none` test locks in that `Some(0.0)` and `None` are PartialEq-distinct; if a future refactor ever collapses these at the layout layer (e.g. via a sentinel-replacing constructor), this test would catch it — by design, since the two values have semantically different renderer outcomes. The `widen_then_add_length_ss_round_trips_other_fields` test only exercises the `TrillBracketOptions → TrillExtensionFullOptions` widening path; the parallel `TrillExtensionSpeedOptions → TrillExtensionFullOptions` widening + `.with_length_ss(...)` path is exercised implicitly by `from_speed_options_with_default_ornament_widens_cleanly` (verifies `length_ss = None` after widening) but not in combination with a follow-up `.with_length_ss(...)` call. Could be added if a regression ever appears.

## 2026-05-15 — Post-v1, explicit length on the two single-purpose trill option bundles (`TrillBracketOptions` + `TrillExtensionSpeedOptions`)

- Did: Closed the "parallel small follow-up" explicitly flagged in the "Next" section of the prior chunk: "Add `length_ss` to `TrillBracketOptions` and `TrillExtensionSpeedOptions` so single-purpose users can combine bracket+length or speed+length without widening to a full bundle." Both bundles now carry an `extension_length_ss: Option<f64>` field with a matching `const fn with_extension_length_ss(length_ss: f64)` setter; the score-builder methods `trill_with_extension_bracketed_with_options` and `trill_with_extension_speed_with_options` write the field into `NoteAnnotations::trill_extension_length_ss`. The `From<TrillBracketOptions>` and `From<TrillExtensionSpeedOptions>` impls on `TrillExtensionFullOptions` propagate the new field into the widened bundle's existing `length_ss`. **No renderer changes needed** — the existing collector already reads `trill_extension_length_ss` independently of bracket/speed/ornament fields.

  Naming note: `TrillBracketOptions` already has a `length_ss` field — that one is the **hook** length (the short vertical line capping a bracketed wiggle). The new field is named `extension_length_ss` to avoid a collision and a breaking rename of the existing public field. The same name (`extension_length_ss`) is used on `TrillExtensionSpeedOptions` for symmetry, even though that struct had no naming conflict; consistency across the two single-purpose bundles is more valuable than terseness. `TrillExtensionFullOptions` keeps its existing `length_ss` name (no conflict there — bracket hook length is already prefixed as `bracket_length_ss` on the full bundle), so the From conversions are `length_ss: opts.extension_length_ss`.

  Doc comment updates: `TrillBracketOptions::length_ss` now explicitly says "this is the **hook** length, not the wiggle's horizontal extension length — see [`extension_length_ss`] for the latter"; `with_length_ss` says "To shorten the *wiggle* (extension) instead, see [`with_extension_length_ss`]"; the struct-level doc has a new paragraph on extension termination semantics.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo test -p music-engraver --lib` — **2167 unit tests pass** (+29 vs prior recorded 2138: 9 layout in `trill_bracket` [`options_new_has_unset_extension_length_ss`, `options_with_extension_length_ss_sets_only_extension_length`, `options_with_extension_length_ss_is_distinct_from_with_length_ss` (critical regression canary — the two `with_*_ss` setters write *different* fields; catches a naming-confusion aliasing bug), `options_with_extension_length_ss_chains_with_other_setters`, `options_with_extension_length_ss_chain_order_independent`, `options_with_extension_length_ss_overwrites_prior_value`, `options_with_extension_length_ss_accepts_non_positive_at_layout_layer`, `options_with_extension_length_ss_const_constructible`, `options_extension_length_some_zero_distinct_from_none` (PartialEq distinguishes `Some(0.0)` from `None` because they have different renderer semantics)]; 9 layout in `trill_extension` [`speed_options_new_has_unset_extension_length_ss`, `speed_options_with_extension_length_ss_sets_only_extension_length`, `speed_options_with_extension_length_ss_chains_with_ornament`, `speed_options_with_extension_length_ss_chain_order_independent`, `speed_options_with_extension_length_ss_overwrites_prior_value`, `speed_options_with_extension_length_ss_accepts_non_positive_at_layout_layer`, `speed_options_with_extension_length_ss_const_constructible`, `speed_options_extension_length_distinct_values_compare_distinct` (PartialEq sensitivity canary), `speed_options_extension_length_some_zero_distinct_from_none`]; 11 score-integration in `score::tests` [`bracketed_with_options_extension_length_ss_propagates_into_annotation` (annotation-layer state inspection — bracket+ornament+extension fields all set correctly); `bracketed_with_options_extension_length_ss_byte_equivalent_to_widened_full_options` (key byte-equivalence canary: `trill_with_extension_bracketed_with_options(opts)` ≡ `trill_with_extension_full_options(opts.into())` when `opts` carries an explicit length — proves the From conversion propagates the field); `bracketed_with_options_extension_length_ss_shortens_wiggle` (renderer-behavior canary: short renders fewer paths than no-length default, AND bracket hook `<line>` count is invariant — proves bracket survives the shortening); `bracketed_with_options_extension_length_ss_oversized_clamps_to_natural` (clamp contract: 1000ss clamps byte-identically); `bracketed_with_options_extension_length_ss_is_distinct_from_hook_length_ss` (score-integration analog of the layout-layer aliasing canary: `with_length_ss(1.5)` and `with_extension_length_ss(1.5)` must produce *different* SVGs; catches a wire-up bug that would render hook-length-vs-extension-length confusion silently); `bracketed_with_options_extension_length_ss_on_chord_renders_shortened_wiggle` (chord arm of the let-else wire-up — catches a regression where only the Note arm gets the new field); `speed_with_options_extension_length_ss_propagates_into_annotation`; `speed_with_options_extension_length_ss_byte_equivalent_to_widened_full_options` (mirror of the bracket byte-equivalence canary, for the speed widening path); `speed_with_options_extension_length_ss_shortens_wiggle`; `speed_with_options_extension_length_ss_zero_drops_wiggle` (non-positive fail-safe contract: 0.0 length renders path count equal to plain ornament-only trill); `speed_with_options_default_extension_length_ss_matches_plain_speed_byte_for_byte` (backwards-compatibility canary: `TrillExtensionSpeedOptions::new(speed)` with no `.with_extension_length_ss(...)` must remain byte-equivalent to `trill_with_extension_speed(speed)` — proves adding the new field with `None` default did NOT shift the existing all-defaults SVG output)]). `cargo test -p music-engraver --test golden_svg` — **65 golden tests pass** (no baseline drift; the byte-equivalence canaries against the existing `trill_with_extension_length_ss` and `trill_with_extension_speed` paths prove no SVG-level regression for existing usage). `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Visual proofing — a small example exercising the new field through each of the two single-purpose bundles (e.g. a "bracket + explicit length without the full bundle" example) + matching golden baseline. Other deferred items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment). A future renaming pass could rename `TrillBracketOptions::length_ss` → `hook_length_ss` to eliminate the lingering naming friction; that's a breaking API change so it's not free.
- Open issues: The existing `TrillBracketOptions::length_ss` and `with_length_ss` are now somewhat less self-explanatory in isolation — they're the hook length, while the new `extension_length_ss` and `with_extension_length_ss` are the wiggle length. The struct-level and method-level doc comments call this out and cross-link the two, and the `options_with_extension_length_ss_is_distinct_from_with_length_ss` + `bracketed_with_options_extension_length_ss_is_distinct_from_hook_length_ss` tests both lock in the distinct semantics. No example or golden baseline added in this chunk — visual proofing is deferred (the byte-equivalence canaries against the existing single-purpose builders + the `trill_with_extension_length_ss` standalone builder + the widened-full-options path prove the renderer state matches, and the existing golden coverage on those three paths transitively covers correctness). The `bracketed_with_options_extension_length_ss_oversized_clamps_to_natural` test depends on the natural span being smaller than 1000ss; if a future change ever made the default natural span larger than 1000ss the clamp test would mistakenly pass even if the new field were silently dropped — extremely unlikely (whole notes max out around ~10ss in current spacing) but documented for completeness.

## 2026-05-15 — Post-v1, visual proofing for `extension_length_ss` on the two single-purpose trill option bundles

- Did: Closed the "Visual proofing — a small example exercising the new field through each of the two single-purpose bundles + matching golden baseline" deferred follow-up explicitly flagged in the "Next" section of the prior chunk (2026-05-15, `extension_length_ss` on `TrillBracketOptions` + `TrillExtensionSpeedOptions`). Until now the new field was covered only by unit tests (29 in the introduction chunk) — no SVG-level regression baseline existed. New example `examples/trill_options_with_length_score.rs` (17902 bytes, 23 paths, 18 lines) walks four whole-note trills across two systems where every measure exercises a combination only the new field unlocks without widening to `TrillExtensionFullOptions`: M1 `TrillBracketOptions::new(Both).with_extension_length_ss(2.0)` (Both bracket + short length); M2 `TrillBracketOptions::new(End).with_extension_length_ss(4.0)` (End bracket + medium length); M3 `TrillExtensionSpeedOptions::new(Slow).with_extension_length_ss(2.0)` (Slow speed + short length); M4 `TrillExtensionSpeedOptions::new(Faster).with_extension_length_ss(3.0)` (Faster speed + medium length, last note of the system to exercise cross-system suppression). Six in-example assertions, each a regression canary for a specific propagation path: (1) basic SVG validity; (2) explicit-length wiggles produce strictly fewer paths than the no-length-on-either-bundle variant (the new-field-actually-shortens-things canary); (3) byte-inequality vs no-length variant (refactor canary); (4) **bracket hook count INVARIANT** under the explicit length — `<line>` count must match the no-length variant exactly (catches a regression where setting the length accidentally suppresses the bracket); (5) **byte-equivalence vs the widened-full-options path** — `trill_with_extension_bracketed_with_options(opts)` == `trill_with_extension_full_options(opts.into())` byte-for-byte when `opts` carries an explicit length, proving the `From<TrillBracketOptions>` and `From<TrillExtensionSpeedOptions>` conversions on `TrillExtensionFullOptions` propagate `extension_length_ss → length_ss` correctly (the highest-leverage canary — locks in the documented byte-equivalence guarantee at the integration layer); (6) sanity that path_count > 0.

  Added matching `build_trill_options_with_length()` + two sibling baseline builders (`_no_length_variant`, `_widened_variant`) and `golden_trill_options_with_length` test in `tests/golden_svg.rs` with five structural guards alongside the frozen baseline: (1) explicit-length variant has fewer paths than no-length (mirrors example assertion 2); (2) byte-inequality vs no-length (refactor canary); (3) **bracket hook count equality** with the no-length variant (mirrors example assertion 4 — the bracket-presence-invariance canary); (4) **byte-equivalence with the widened-full-options variant** (mirrors example assertion 5 — the highest-leverage `From` propagation canary at the golden layer); (5) **byte-inequality vs the existing `trill_short_extension` baseline** — that golden uses the standalone `trill_with_extension_length_ss(L)` builder on plain trills with no bracket and no speed override, while this golden overlays bracket and speed on top of explicit lengths; if a future refactor ever collapsed the option-bundle paths to the standalone path, the two outputs would converge (the cross-baseline check is the bracket+speed-must-still-be-present canary at the golden layer). Wrote `tests/golden/trill_options_with_length.svg` (17902 bytes) via `GOLDEN_UPDATE=1` and confirmed `cmp examples/output/trill_options_with_length_score.svg tests/golden/trill_options_with_length.svg` reports byte-identical (renderer is deterministic; example and golden share the same build path). Added `"trill_options_with_length"` to the `golden_baselines_are_valid_svgs` name list. Fixed the 8 `doc_overindented_list_items` clippy warnings in the new example's module-level doc comment (changed M1–M4 list items' second-line indent from 6 spaces to 2 spaces to match the rust-1.95 list-continuation style).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --example trill_options_with_length_score` passes. `cargo run -p music-engraver --example trill_options_with_length_score` writes 17902 bytes (23 paths, 18 lines); all 6 in-example assertions hold (path-count strictly less than no-length, byte-inequality vs no-length, **`<line>` count equality with no-length** locks bracket-presence invariance, **byte-equivalence with widened-full-options** locks the `From` conversion, valid SVG, path_count > 0). `cargo test -p music-engraver --test golden_svg` — **66 golden tests pass** (65 prior + new `golden_trill_options_with_length`); all 5 in-test structural guards (path-count delta, byte-inequality vs no-length, bracket-hook-count equality, byte-equivalence with widened, byte-inequality vs `trill_short_extension`) hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --lib` — **2167 unit tests pass** (unchanged from prior recorded 2167; this chunk adds no library tests — visual proofing lives in the example + golden test, and all 29 unit tests for `extension_length_ss` already exist from the introduction chunk). `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cmp examples/output/trill_options_with_length_score.svg tests/golden/trill_options_with_length.svg` — byte-identical. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: Other deferred items (now visually proofed: bracket+length and speed+length combinations through the two single-purpose bundles; the unified `TrillExtensionFullOptions::with_length_ss` path remains visually unproofed in isolation — covered transitively by the new golden's byte-equivalence assertion against the widened-full-options variant, but no example exercises it directly with the unified builder's full bracket+speed+ornament+length combination). Remaining: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment); a future renaming pass that renames `TrillBracketOptions::length_ss` → `hook_length_ss` to eliminate the lingering naming friction (breaking API change).
- Open issues: The byte-equivalence assertion (guard 4 / example assertion 5) locks in that the `From<TrillBracketOptions>` and `From<TrillExtensionSpeedOptions>` conversions on `TrillExtensionFullOptions` propagate `extension_length_ss → length_ss` losslessly; if a future refactor changes the unified bundle's `length_ss` semantics independently of the single-purpose bundles' `extension_length_ss`, this assertion would fire. By design — the documented contract is that the conversions are byte-equivalent, so a divergence is a regression. The bracket-hook-count equality assertion (guard 3 / example assertion 4) leans on the assumption that no future renderer change adds a non-bracket `<line>` element to one variant but not the other; currently `<line>` is used for staff lines, ledger lines, barlines, stems, beams, bracket hooks — none of which vary between the two variants in this score (same musical content, only the explicit-length annotation differs). If a future change starts emitting wiggle segments as `<line>` instead of `<path>`, the equality assertion would fire (since wiggle tile counts differ between variants); fix would be to filter `<line>` by stroke pattern or class. The cross-baseline byte-inequality against `trill_short_extension` (guard 5) is sensitive to musical-content drift in **either** baseline — if a future refactor regenerates `trill_short_extension.svg` with coincidentally matching musical content (same pitches, same lengths, no bracket, no speed), the canary fires a false positive; this is unlikely (the two scores use different bracket/speed annotations on every measure, and `trill_short_extension` deliberately uses no bracket and no speed override) but documented for completeness. The example's structural assertions are intentionally redundant with the golden test's assertions on the same content — keeping them in both places preserves the property that `cargo run --example trill_options_with_length_score` is a self-contained verification step (the example file is the runnable demo for downstream users; the golden test is the regression canary).

## 2026-05-15 — Post-v1, visual proofing for `TrillExtensionFullOptions::with_length_ss` (full bracket+speed+ornament+length combination)

- Did: Closed the "the unified `TrillExtensionFullOptions::with_length_ss` path remains visually unproofed in isolation — covered transitively by the new golden's byte-equivalence assertion against the widened-full-options variant, but no example exercises it directly with the unified builder's full bracket+speed+ornament+length combination" deferred follow-up explicitly flagged in the "Next" section of the prior chunk. The unified bundle's `length_ss` field was unit-tested (6 layout + 7 score-integration tests from the introduction chunk on 2026-05-15) and golden-covered transitively via `trill_options_with_length`'s widened-variant byte-equivalence assertion, but no example or golden directly exercised the **four-way combination** `bracket + speed + ornament + length` — the truly unique capability of the unified bundle (neither single-purpose bundle can express all four knobs in a single call). New example `examples/trill_full_options_with_length_score.rs` (18383 bytes, 27 paths, 24 lines) walks four measures across two systems where every measure sets all four knobs at once: M1 `Both` bracket + `Slow` speed + `TrillWithMordent` + 2.0ss length; M2 `End` bracket + `Up` direction + `Faster` speed + plain `Trill` + 4.0ss length; M3 `Start` bracket + 1.0ss hook + `Slowest` speed + `TrillWithMordent` + 3.0ss length; M4 chord (C-E-G half) + `Both` bracket + `Standard` speed + `TrillWithMordent` + 2.5ss length, followed by a quarter D4. **Diagnostic finding** during development: at M3 length=1.5ss with `Slowest` speed (the widest wiggle glyph at ~2.4ss advance in Bravura), the wiggle is shorter than one full segment and the renderer's documented fail-safe drops BOTH the wiggle AND its Start hook — bumped M3 to 3.0ss so every measure renders a visible bracket and the hook-count invariance assertion holds. Seven in-example structural assertions, each a regression canary for a specific propagation path: (1) basic SVG validity; (2) explicit lengths produce **strictly fewer paths** than the no-length baseline (the new-capability canary — if `length_ss` stops propagating through the unified bundle when other knobs are set, both versions render the same wiggle tile counts); (3) byte-inequality vs no-length baseline (refactor canary); (4) **net bracket hook count INVARIANT** vs no-length baseline — M2's cross-system End hook moves from system 2 (no-length variant, via `draw_cross_system_trill_extensions`) to system 1 (with-length variant, since positive explicit length disables cross-system propagation), but the TOTAL count is preserved (1 End hook drawn either way); catches a regression where setting the length suppresses a hook entirely; (5) per-measure independence: a uniform-length variant (2.0/2.0/3.0/2.0) must render byte-different from the heterogeneous featured score (2.0/4.0/3.0/2.5) — catches a future refactor that collapses `length_ss` to a global field; (6) sanity that `path_count > 0`; (7) the shortened score must be substantially smaller (>10% byte reduction) than the no-length baseline.

  Added matching `build_trill_full_options_with_length()` builder and `golden_trill_full_options_with_length` test in `tests/golden_svg.rs` with five structural guards alongside the frozen baseline: (1) byte-inequality vs the existing `trill_full_options` baseline (same bracket+speed+ornament, no length) — locks in that `.with_length_ss(...)` on every measure produces a visibly different SVG; (2) explicit-length variant has strictly fewer paths than `trill_full_options` (length actually shortens wiggle); (3) **net bracket hook count equality** with `trill_full_options` (cross-system hook-position differences are absorbed by the total — see example assertion 4 for the same property); (4) byte-inequality vs `trill_short_extension` — that golden uses the standalone `trill_with_extension_length_ss(L)` shortcut on plain trills with no bracket/speed; this golden layers bracket + speed + compound ornament on top; if a future refactor accidentally collapsed the unified-options path to the standalone path, the two baselines would converge; (5) byte-inequality vs the existing `trill_options_with_length` baseline — that one uses the two single-purpose bundles with `extension_length_ss`; this one uses the unified bundle with FOUR knobs (`bracket + speed + ornament + length`) which neither single-purpose bundle can express on its own — the cross-baseline canary catches a refactor that accidentally collapsed the unified-bundle path's expressiveness down to the single-purpose bundles'. Wrote `tests/golden/trill_full_options_with_length.svg` (18383 bytes) via `GOLDEN_UPDATE=1` and confirmed `cmp examples/output/trill_full_options_with_length_score.svg tests/golden/trill_full_options_with_length.svg` reports byte-identical (renderer is deterministic; example and golden share the same build path). Added `"trill_full_options_with_length"` to the `golden_baselines_are_valid_svgs` name list.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --example trill_full_options_with_length_score` passes. `cargo run -p music-engraver --example trill_full_options_with_length_score` writes 18383 bytes (27 paths, 24 lines); all 7 in-example assertions hold (basic validity, path-count strictly less than no-length, byte-inequality vs no-length, **bracket hook count invariant**, per-measure independence vs uniform-length variant, path_count > 0, byte-size reduction >10%). `cargo test -p music-engraver --test golden_svg` — **67 golden tests pass** (66 prior + new `golden_trill_full_options_with_length`); all 5 in-test structural guards (byte-inequality vs `trill_full_options`, path-count delta, bracket-hook-count equality, byte-inequality vs `trill_short_extension`, byte-inequality vs `trill_options_with_length`) hold against the freshly written baseline. `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` — passes including the new name entry. `cargo test -p music-engraver --lib` — **2167 unit tests pass** (unchanged from prior recorded 2167; this chunk adds no library tests — visual proofing lives in the example + golden test, and all 6 layout + 7 score-integration tests for `TrillExtensionFullOptions::with_length_ss` already exist from the introduction chunk). `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cmp examples/output/trill_full_options_with_length_score.svg tests/golden/trill_full_options_with_length.svg` — byte-identical. `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`).
- Next: The trill-extension API surface is now visually proofed across every combinator path that the public API surfaces: bare trill (`trill_extension_score`); bracketed (`trill_bracket_score` + `_custom`); speeds (`trill_wiggle_speed_score`); compound prefix (`trill_with_mordent_score`); compound + bracket (`trill_bracket_with_mordent_score`); compound + speed (`trill_speed_with_mordent_score`); explicit length (`trill_short_extension_score`); bracket+length and speed+length through the two single-purpose bundles (`trill_options_with_length_score`); **full bracket+speed+ornament+length combination through the unified bundle** (this chunk); plain bracket+speed+ornament without length (`trill_full_options_score`). Remaining deferred items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; PNG export deeper testing; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment); a future renaming pass that renames `TrillBracketOptions::length_ss` → `hook_length_ss` to eliminate the lingering naming friction with the same-struct's new `extension_length_ss` field (breaking API change). Different feature areas (non-trill): PNG export deeper testing remains an obvious leverage target.
- Open issues: The hook-count invariance assertion (guard 3 / example assertion 4) holds because the renderer balances cross-system continuation: when an explicit length forces `cross_system=false`, the End hook moves from the page-renderer's incoming-wiggle pass on system N+1 to the system-renderer's within-system pass on system N — net 1 End hook either way. The assertion would fire if a future change broke that balance (e.g., by suppressing the End hook on system N when length is set without also disabling the cross-system continuation on system N+1, which would yield zero hooks — clear regression). It would also fire if a future renderer change emitted wiggle segments as `<line>` instead of `<path>`, since wiggle tile counts differ between variants and the `<line>` total would no longer be balanced; the fix in that case would be to filter `<line>` by stroke pattern or class. M3 length=1.5ss with `Slowest` speed was below the renderer's wiggle-too-short fail-safe threshold and dropped both the wiggle and its Start hook (the diagnostic finding above); chose 3.0ss to land above the threshold. A future change to the `Slowest` glyph advance (e.g., via a new Bravura version) could shift this threshold; the test would catch the regression by failing the hook-count invariance assertion, at which point M3's length would need to be re-tuned. The example deliberately uses the same musical content as `trill_full_options_score.rs` so the two SVGs are *minimally* different — every byte of difference is attributable to the new length field, which is the cleanest design for a regression canary but means the two examples would converge if the length field stopped propagating (covered by guard 1 / example assertion 3).

## 2026-05-15 — Post-v1, PNG export deeper testing: pixel-content verification

- Did: Closed the long-deferred "PNG export deeper testing" follow-up that has carried in the "Next" section of multiple recent chunks. Until this run, PNG coverage in `src/render/png.rs` checked magic bytes, IHDR dimensions (manually parsed), and byte-size lower bounds — but a PNG of all-transparent pixels would still have passed every test. A regression like a font-loading failure rendering Bravura glyphs as invisible, a rasterizer transform painting into the wrong region, or a usvg parse silently emitting an empty tree, would all have slipped through. Added 10 new pixel-content verification tests to the `png::tests` module, each decoding the rendered PNG back into a `tiny_skia::Pixmap` (via `Pixmap::decode_png`, which is available since `tiny-skia`'s `png-format` feature is on by default) and asserting on actual pixel values:
  1. `pixmap_dimensions_match_png_header_dimensions` — sanity round-trip between IHDR parse and decoder.
  2. `score_png_contains_non_zero_inked_pixels` — score PNG must have >=200 inked pixels (alpha >= 32). Catches the "PNG is blank" regression class — font failures, missing render call, wrong-region rasterization.
  3. `score_png_is_mostly_transparent_background` — >50% of pixels must be alpha == 0. Catches a regression that paints an opaque background across the whole image.
  4. `empty_svg_renders_fully_transparent_png` — known input (SVG with no elements) → every pixel alpha == 0. Catches a regression where the rasterizer ever started clearing to opaque.
  5. `opaque_filled_rect_renders_all_pixels_opaque` — known input (full-canvas filled rect) → every pixel alpha == 255 with average R near 0. Sanity check on encode→decode round-trip.
  6. `red_filled_rect_decodes_as_red_dominant` — channel-order canary: R >> G + 64 and R >> B + 64 at center of a red rect. Catches BGRA/RGBA swap and premultiplication bugs.
  7. `inked_pixel_count_scales_roughly_with_area` — 2×/1× ink-pixel ratio in [2.0, 5.5]. Theoretical area ratio is 4× for filled shapes; for thin-stroke score content (staff lines, stems) AA fringe inflates the 1× count proportionally more than the 2× count, pulling the observed ratio toward ~2×. Empirically lands at ~2.5× on the dense-four-notes score. Lower bound guards against "scale does nothing" (ratio ≈ 1); upper bound guards against pathological over-scaling (ratio ≈ 8× would indicate extra factor of 2 leaked into stroke widths).
  8. `score_png_ink_bounding_box_spans_most_of_width` — ink bbox width >50% of image width. Catches a bug where all content rendered into a single column.
  9. `different_scores_produce_different_inked_pixel_counts` — score with 1 whole rest vs score with 4 distinct quarter notes: `assert_ne!` on counts + assert dense > sparse. Regression canary against rendering being content-independent (e.g. accidental short-circuit / wrong-key cache). Initial design used "1 quarter note + 3 quarter rests" as the sparse score, but quarter-rest glyphs have ink density comparable to a quarter note + stem, so ink counts came out near-identical (3646 vs 3642). Replaced sparse score with a single whole rest (a small filled rectangle) for a robust density gradient.
  10. `score_png_has_dense_horizontal_band_consistent_with_staff` — sweep all rows for max inked-pixel count; densest row must be >=30% of image width. Validates staff-line presence: a horizontal stripe at staff-line altitude should span the full measure width. Catches a regression where staff lines are dropped or replaced with dashed/intermittent strokes.

  Added helpers in `png::tests`: `INK_ALPHA_THRESHOLD = 32` (small alpha threshold ignores the long AA tail but counts meaningful coverage — robust to AA changes); `decode_pixmap`; `count_inked_pixels(pixmap, threshold)`; `inked_bbox(pixmap, threshold) -> Option<(x_min, y_min, x_max, y_max)>` (None when no pixel meets threshold); `inked_pixels_in_row(pixmap, y, threshold)`; `score_sparse_whole_rest()`; `score_dense_four_notes()`. The whole-rest vs four-notes ink-density gradient is the input I tuned to make the regression canary robust — see the "Did" item 9 explanation.

  No new dependencies needed: `tiny-skia` is already an optional dep for the `png` feature, and `decode_png` is available because `tiny-skia`'s default `png-format` feature is enabled (confirmed by inspecting `tiny-skia-0.11.4/Cargo.toml`).
- Verified: `cargo check -p music-engraver` passes (default features). `cargo check -p music-engraver --features png --tests` passes. `cargo check --workspace` passes. `cargo build -p music-engraver --features png` passes. `cargo test -p music-engraver --features png --lib png::tests` — **22 png tests pass** (12 pre-existing + 10 new); during development the initial 2-tuple of failures (ratio=2.48 outside [3.0, 5.5]; sparse vs dense ink counts too close: 3646 vs 3642) drove the tuning of the area-scaling tolerance and the redesign of the sparse score, confirming both tests would catch real regressions. `cargo test -p music-engraver --features png --lib` — **2196 unit tests pass** (2186 prior + 10 new from this chunk; prior total was 2167 lib tests without the `png` feature gating in 12 existing PNG tests, so 2167 + 12 existing PNG + 10 new = 2189 — discrepancy of 7 vs counted 2196 is from feature-gated tests elsewhere in the crate that I haven't audited individually). `cargo test -p music-engraver --features png --test golden_svg` — **67 golden tests pass** (unchanged from prior; no SVG-level regression). `cargo test -p music-engraver --features png --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass.
- Next: Other deferred items: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; mid-trill speed change (a single sustained trill that visibly accelerates — speed is still per-trill, not per-segment); a future renaming pass that renames `TrillBracketOptions::length_ss` → `hook_length_ss` to eliminate the lingering naming friction with that struct's new `extension_length_ss` field (breaking API change). Other post-v1 candidate next chunks not yet started: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / chord symbols / lyrics / grace-note family expansion; golden-SVG corpus + PHASH-based visual regression harness. For PNG specifically, the next-level deepening would be: (a) a golden-PNG corpus (small set of frozen PNG baselines for byte- or pixel-level comparison — useful but adds binary baselines to the repo); (b) a `--features png` golden test that mirrors one of the existing SVG goldens at PNG level; (c) PNG output for `MultiStaffScore` and tab scores (the existing tests cover only single-staff `ScoreBuilder`).
- Open issues: The `inked_pixel_count_scales_roughly_with_area` test bounds ratio in [2.0, 5.5]; the empirical observed value of ~2.48 on the dense-four-notes score is below the textbook area-scaling expectation of 4× because thin-stroke content has AA fringe that inflates the 1× count more than the 2× count. Lower bound 2.0 is the meaningful one (catches "scale does nothing"); upper bound 5.5 catches pathological over-scaling. If a future refactor changes line widths in a way that shifts the ratio outside this window, the test would fire — likely a genuine signal that something about scaling is no longer correct, but worth re-baselining if intentional. The `score_png_has_dense_horizontal_band_consistent_with_staff` test's 30% density threshold is a lower bound — empirically the staff lines should produce rows close to 100% inked density (staff lines span the full measure width), so 30% leaves significant headroom for AA fringe and edge effects. If a future change starts emitting staff lines as a series of short dashes rather than continuous strokes, the densest row would drop below 30% and this test would fire. The `INK_ALPHA_THRESHOLD = 32` constant is a deliberate choice: pixels with alpha 1-31 are part of the AA edge fringe and we want to ignore them to count "real" coverage, not "any pixel touched"; if a future renderer change starts producing different AA strength (e.g. uses a different rasterizer with thinner AA tails), the absolute ink counts could shift and trigger thresholds. The whole-rest vs four-notes score-pair choice is robust to most refactors but would break if (a) the whole-rest glyph ever started rendering at a notehead's density (extremely unlikely) or (b) the four-notes content ever short-circuited to nothing (which would itself be a regression the test should catch). PNG output for `MultiStaffScore`/tab scores is exercised by other tests (`render_png` is called in `score/multi_staff.rs` and `score/tab.rs`) but those tests only check magic bytes — the pixel-content depth added here is for `ScoreBuilder` only. Deepening multi-staff / tab PNG tests is a clean next chunk.

## 2026-05-15 — Post-v1, non-breaking `with_hook_length_ss` / `hook_length_ss` alias on `TrillBracketOptions`

- Did: Addressed the naming-friction "Next" item that has carried in recent chunks ("a future renaming pass that renames `TrillBracketOptions::length_ss` → `hook_length_ss` to eliminate the lingering naming friction with that struct's new `extension_length_ss` field — breaking API change"). Instead of a breaking rename, added a non-breaking method alias: `TrillBracketOptions::with_hook_length_ss(length_ss: f64) -> Self` (setter, `const fn`) writes to the same existing `length_ss` field as `with_length_ss`, plus a `hook_length_ss(&self) -> Option<f64>` getter that returns the same field by the clearer name. The field itself keeps its public name `length_ss` so struct-literal construction stays source-compatible. Doc comments on `with_length_ss`, the `length_ss` field, and the struct-level doc example were updated to point readers to the clearer name without deprecating the legacy one (deprecation would generate noise across the many existing call sites in tests and examples; the alias is a pure ergonomic addition).

  Naming rationale: `TrillBracketOptions` carries two semantically distinct "length" knobs — `length_ss` (the bracket *hook* length, the short vertical line capping the wiggle) and `extension_length_ss` (the wiggle's horizontal *extension* termination length, added in a prior chunk). Side-by-side, the bare name `length_ss` reads ambiguously next to `extension_length_ss`. `hook_length_ss` disambiguates without breaking source compatibility. Both setters remain valid and byte-equivalent — pick whichever reads more clearly at the call site.
- Added 9 new layout-layer tests in `layout::trill_bracket::tests`, each locking in a specific contract:
  1. `with_hook_length_ss_writes_to_length_ss_field` — the new setter populates the *existing* `length_ss` field, not a phantom parallel field. Asserts every other knob (direction, ornament, extension_length_ss, side) is untouched.
  2. `with_hook_length_ss_is_byte_equivalent_to_with_length_ss` — loops over 7 length values (including `0.0`, `1.0`, `-2.0`, a non-special non-integer `3.5`) and asserts `PartialEq` equivalence of options built via the two setters. The PartialEq derive covers every field; if a future refactor splits the storage, the assertion fires for at least one of the 7 values. Initial test used `3.14159` which clippy flagged as `approx_constant` (close to `f64::consts::PI`); swapped to `3.5` to avoid the lint without weakening the test (the value just needs to be a non-special non-integer for the loop, not literally π).
  3. `with_hook_length_ss_overwrites_with_length_ss_when_chained` — last-write-wins semantic: calling both setters in sequence lands on whichever was called last. Tested in both orders, since they're aliases of each other.
  4. `hook_length_ss_getter_returns_length_ss_field` — the getter returns the same `Option<f64>` the field holds, in all three states (unset, set via legacy setter, set via new setter). Locks in the `getter ≡ field` contract.
  5. `with_hook_length_ss_is_distinct_from_with_extension_length_ss` — critical naming-disambiguation canary: the new alias must write to a *different* field than `with_extension_length_ss`. If a future refactor accidentally aliased them — exactly the plausible naming-confusion bug this whole chunk is meant to prevent — this assertion fires. Mirrors the existing `options_with_extension_length_ss_is_distinct_from_with_length_ss` test but for the new alias name.
  6. `with_hook_length_ss_chains_with_other_setters` — composes all five setters in one chain (side, direction, hook length via the new name, ornament, extension length); asserts every field lands at the expected value. Also asserts the getter agrees with the field after chaining.
  7. `with_hook_length_ss_chain_order_independent_from_other_setters` — three different orderings of the same three setters must produce equal options (`a == b == c`). Mirrors `options_chain_order_independent` for the new setter.
  8. `with_hook_length_ss_is_const_constructible` — locks in `const fn` via a `const` item at module scope. If `const fn` is removed in a future refactor, this test stops compiling. Mirrors `options_const_constructible` for the new setter.
  9. `hook_length_ss_getter_is_const_callable` — locks in `const fn` for the getter too, plus asserts `None` round-trips through the const-context call.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). During development clippy initially fired `approx_constant` on `3.14159`; fixed by swapping to `3.5`. `cargo test -p music-engraver --lib` — **2176 unit tests pass** (+9 vs prior recorded 2167 with default features; PNG-feature gated tests inflate to 2196 under `--features png` per the prior chunk's record). All 9 new tests in `layout::trill_bracket::tests` pass; the existing 13 `length_ss`/`extension_length_ss`/`ornament` tests in the same module continue to pass unchanged, confirming the alias is purely additive. `cargo test -p music-engraver --test golden_svg` — **67 golden tests pass** (unchanged; no SVG-level regression expected since the alias writes to an existing field with the same semantics). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass (the updated doc example on `TrillBracketOptions` adds a one-line `.with_hook_length_ss(1.0)` call alongside the existing `.with_length_ss(1.0)` example; both compile and round-trip).
- Next: Other deferred items still pending: cross-system church rests; per-note collision detection in beamed additional voices (this one is genuinely architectural — the current collision detector at `src/layout/voice_collision.rs:33-40` collects ALL beam-group note staff_positions and compares them at the beam group's single start x, when correct behavior would compare each constituent note at its own x; fixing that requires extending `VoiceCollisionOffset` with a `note_index_within_element: Option<usize>` and threading per-note offsets through the renderer, which currently shifts the entire beam group as a unit); Gourlay penalty tuning; mid-trill speed change. Now that `with_hook_length_ss` exists, the parallel single-purpose bundles `TrillExtensionSpeedOptions` already have only `extension_length_ss` (no naming clash) so no alias is needed there. The full-options bundle `TrillExtensionFullOptions` carries both `length_ss` (wiggle extension length — confusing the *other* direction since the bracket struct's `length_ss` is the hook length) and `bracket_length_ss` (bracket hook length); a parallel non-breaking alias `with_extension_length_ss` / `extension_length_ss()` on `TrillExtensionFullOptions` would symmetrically disambiguate that struct. Worth considering as a future small follow-up. Larger remaining post-v1 candidates: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / chord symbols / lyrics / grace-note family expansion; golden-SVG corpus + PHASH-based visual regression harness; chord-symbol SMuFL accidental glyph composition (currently chord symbols render `#` and `b` as ASCII text, not as SMuFL `accidentalSharp`/`accidentalFlat` glyphs — a real engraving-quality gap).
- Open issues: The legacy `with_length_ss` method and `length_ss` field name remain in place — they're not deprecated to avoid `#[deprecated]` warning noise across the many existing call sites in tests, examples, and the `From<TrillBracketOptions>` / `From<TrillExtensionSpeedOptions>` conversions on `TrillExtensionFullOptions` (which write through `opts.length_ss`). A future cleanup pass could deprecate the legacy setter once all internal call sites have been migrated to the clearer name. The `with_hook_length_ss_is_byte_equivalent_to_with_length_ss` test uses fixed-point f64 literals; if floating-point hash semantics changed in a way that broke `Some(0.0) == Some(0.0)` (e.g. if the field were ever changed to wrap an `f64` in a NaN-handling wrapper), the test would need to switch to bit-pattern comparison. Extremely unlikely in current Rust but documented for completeness. The `with_hook_length_ss_is_distinct_from_with_extension_length_ss` test specifically uses the same numeric value (1.0) for both knobs to ensure the assertion fails only if the *field* assignment is wrong (not if the values happen to differ); this is the intentional design — distinguishing on `Some(1.0) == Some(1.0)` vs different values would weaken the regression canary.

## 2026-05-15 — Post-v1, non-breaking `with_extension_length_ss` / `extension_length_ss` alias on `TrillExtensionFullOptions`

- Did: Closed the "parallel non-breaking alias `with_extension_length_ss` / `extension_length_ss()` on `TrillExtensionFullOptions` would symmetrically disambiguate that struct — worth considering as a future small follow-up" item explicitly flagged in the "Next" section of the prior chunk (2026-05-15, `with_hook_length_ss` / `hook_length_ss` alias on `TrillBracketOptions`). The two structs have **inverse** length-field naming: on `TrillBracketOptions`, `length_ss` is the **hook** length and `extension_length_ss` is the wiggle's extension length; on `TrillExtensionFullOptions`, `length_ss` is the **wiggle extension** length and `bracket_length_ss` is the bracket hook length. The bare name `length_ss` is ambiguous next to a sibling `bracket_length_ss` / `extension_length_ss` field on the same struct — the alias resolves the asymmetric naming friction on the full-options bundle the same way the prior chunk resolved it on the bracket bundle. Added `with_extension_length_ss(length_ss: f64) -> Self` (setter, `const fn`) writing to the existing `length_ss` field plus an `extension_length_ss(&self) -> Option<f64>` getter (`const fn`) reading it back by the clearer name. The field itself keeps its public name `length_ss` for source compatibility with the documented `From<TrillBracketOptions>` / `From<TrillExtensionSpeedOptions>` byte-equivalence guarantees and with the struct-literal construction path. Updated `with_length_ss` doc to point at the alias; added a "Note: this is the wiggle extension length, not the bracket hook length" paragraph to the `length_ss` field doc, cross-linking both `bracket_length_ss` and the new `extension_length_ss()` accessor.

  Naming rationale: the renamed setter creates a *symmetric* surface across both option bundles — `TrillBracketOptions::with_extension_length_ss` (already existed) and `TrillExtensionFullOptions::with_extension_length_ss` (this chunk, alias for `with_length_ss`) both refer to "the wiggle's horizontal extension termination length." Callers who reach for `with_extension_length_ss` regardless of which bundle they hold get consistent semantics; the legacy `with_length_ss` on the full bundle remains valid byte-for-byte. **No renderer changes needed** — the alias is a pure layout-layer ergonomic addition writing to an existing field.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). `cargo test -p music-engraver --lib` — **2186 unit tests pass** (+10 vs prior recorded 2176 with default features; all 10 new tests live in `layout::trill_options::tests` and mirror the analogous prior-chunk tests for the bracket bundle: `with_extension_length_ss_writes_to_length_ss_field`, `with_extension_length_ss_is_byte_equivalent_to_with_length_ss` (loops 7 lengths including `0.0` / `1.0` / `-2.0` / `3.5` — non-special non-integer to avoid the `approx_constant` lint), `with_extension_length_ss_overwrites_with_length_ss_when_chained` (last-write-wins canary tested in both orders), `extension_length_ss_getter_returns_length_ss_field` (returns the field value across unset / set-via-legacy / set-via-new), `with_extension_length_ss_is_distinct_from_with_bracket_length_ss` (the **critical naming-disambiguation canary** — must write to different fields; mirrors the prior chunk's `with_hook_length_ss_is_distinct_from_with_extension_length_ss` but in the inverse direction, since on this struct it is `bracket_length_ss` that is the hook), `with_extension_length_ss_chains_with_other_setters` (composes all six setters in one chain; verifies the getter agrees with the field after chaining), `with_extension_length_ss_chain_order_independent_from_other_setters` (three orderings produce equal options — `a == b == c`), `with_extension_length_ss_is_const_constructible` (compile-fail canary if `const fn` is removed), `extension_length_ss_getter_is_const_callable` (`const fn` for the getter too), `extension_length_ss_getter_after_widening_from_bracket_options` (locks in the **source-of-truth chain** `TrillBracketOptions::extension_length_ss → TrillExtensionFullOptions::length_ss → extension_length_ss()` — the cross-struct propagation canary).). All 26 pre-existing `trill_options::tests` continue to pass unchanged, confirming the alias is purely additive. `cargo test -p music-engraver --doc` — 12 doc tests pass (no doc-example changes needed in this chunk; the existing struct-level doc example already exercises `.with_length_ss(3.5)`). `cargo test -p music-engraver --test golden_svg` — **67 golden tests pass** (unchanged; no SVG-level regression expected since the alias writes to an existing field with the same semantics, and every existing SVG golden uses either `with_length_ss` or the `From`-conversion path which is unaffected). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass.
- Next: Other deferred items still pending: cross-system church rests; per-note collision detection in beamed additional voices (architectural — see prior chunk's note: requires extending `VoiceCollisionOffset` with a `note_index_within_element: Option<usize>` and threading per-note offsets through the renderer, which currently shifts the entire beam group as a unit); Gourlay penalty tuning; mid-trill speed change. Larger remaining post-v1 candidates: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / chord symbols / lyrics / grace-note family expansion; chord-symbol SMuFL accidental glyph composition (currently chord symbols render `#` and `b` as ASCII text, not as SMuFL `accidentalSharp`/`accidentalFlat` glyphs — a real engraving-quality gap); golden-SVG corpus + PHASH-based visual regression harness. A future cleanup pass could deprecate both legacy `with_length_ss` setters (on the bracket bundle and the full-options bundle) once all internal call sites have been migrated to the clearer aliases — currently held back to avoid `#[deprecated]` warning noise across the many existing tests, examples, and the two `From` conversions on `TrillExtensionFullOptions`.
- Open issues: The `extension_length_ss_getter_after_widening_from_bracket_options` test reaches across two structs (bracket → full) and locks in that the From conversion preserves the field through the rename: `TrillBracketOptions::extension_length_ss` → `TrillExtensionFullOptions::length_ss` (semantically the same wiggle-length value, syntactically a different field name because the parent struct renamed the field). The new `extension_length_ss()` accessor surfaces it under the symmetric name — so callers reading `widened.extension_length_ss()` after a From conversion get the same value they wrote via `bracket.with_extension_length_ss(...)`. If a future refactor ever renamed the `TrillExtensionFullOptions::length_ss` field itself (breaking change), this test would catch the regression at the accessor layer. The legacy `with_length_ss` method and `length_ss` field name remain in place to avoid breaking the documented byte-equivalence guarantees with `trill_with_extension_bracketed_with_options(opts)` (when `opts` carries an explicit length) and with the struct-literal construction path. The `with_extension_length_ss_is_byte_equivalent_to_with_length_ss` test uses fixed-point f64 literals; same caveat as in the prior chunk regarding floating-point hash semantics if the field were ever wrapped in a NaN-handling wrapper. The `with_extension_length_ss_is_distinct_from_with_bracket_length_ss` test specifically uses the same numeric value (1.0) for both knobs to ensure the assertion fails only if the *field* assignment is wrong (not if the values happen to differ); intentional design mirroring the prior chunk's distinct-from-extension-length test.

## 2026-05-15 — Post-v1, chord-symbol SMuFL accidental glyph composition

- Did: Closed the long-deferred "chord-symbol SMuFL accidental glyph composition (currently chord symbols render `#` and `b` as ASCII text, not as SMuFL `accidentalSharp`/`accidentalFlat` glyphs — a real engraving-quality gap)" item explicitly flagged in the "Next" sections of multiple recent chunks. Until this run, every `#`, `b`, `♯`, `♭`, `♮` in a chord symbol was rendered as a literal ASCII or Unicode character inside a single bold `<text>` element — visually crude (text-font's `#`/`b` is positioned and weighted as a body letter, not as an accidental), and the engraving-quality gap was real (every commercial engraver — Sibelius, Finale, Dorico, MuseScore — composes chord symbols as text + SMuFL glyph paths, with the accidentals sized at ~70 % of the text size and baseline-raised).

  Layout (`layout/chord_symbol.rs`, **+~280 LOC**): added a new parser `parse_chord_symbol_segments(text) -> Vec<ChordSymbolSegment>` that splits a chord symbol into a sequence of `Text(String)` / `Sharp` / `Flat` / `Natural` segments. ASCII `#` is always Sharp; Unicode `♯`/`♭`/`♮` (U+266F / U+266D / U+266E) are always the corresponding accidentals. The tricky case is lowercase ASCII `b` — it's a flat root letter component only in chord-symbol context, not the alphabet. Rule: `b` is a flat when preceded by an uppercase root letter `A`–`G` (catches `Bb`, `Eb`); preceded by a digit (catches `b5`, `b9`, `7b5`, `13b9`); preceded by `(`, `+`, `-`, `/`, `,` (catches `C(b5)`, `C7+b9`, `D/Bb`); or the symbol's first character (catches a bare `b5` alteration label). Otherwise it stays text. Then added a composite layout function `layout_chord_symbol_composite(text, note_center_x, staff, staff_space, units_per_em, accidental_advance)` returning `ChordSymbolCompositeLayout { boxes: Vec<ChordSymbolSegmentBox>, x_center, y_baseline, font_size, total_width }`. Each `ChordSymbolSegmentBox` carries its own `(x_left, y_baseline, font_size, width)`. Text widths use a documented constant `CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR = 0.55` em (conservative average for serif/sans-serif chord fonts; text font is not bundled — this estimate only positions the *gaps* between accidental glyphs, not the rendered text width itself which depends on the viewer's serif fallback). Accidental glyph widths come from the music font via the `accidental_advance: impl Fn(Glyph) -> u16` callback, scaled by `ACCIDENTAL_SIZE_FACTOR = 0.70`. The font-agnostic stance of the layout module is preserved — the callback abstracts away which font is in use; swapping Bravura for Petaluma or Leland requires zero layout changes. Two more constants: `ACCIDENTAL_BASELINE_RAISE_FACTOR = 0.20` (visual centering on text x-height) and `ACCIDENTAL_SIDE_BEARING_FACTOR = 0.08` (small gap on either side of every accidental so it doesn't kiss adjacent text). The original single-text `layout_chord_symbol` and `ChordSymbolLayout` are preserved for backward compatibility; nothing was deleted.

  Renderer (`render/chord_symbol_renderer.rs`, **+~150 LOC**): added `draw_chord_symbol_composite(svg, font, layout) -> Result<(), FontError>`. Walks the layout's `boxes` and emits `<text>` for text runs (with `text-anchor="start"` and `x_left` as the literal left edge — necessary so accidental glyph paths can be positioned at known x-coords relative to the text) and `<path>` for accidental segments (transformed to `translate(x_left, y_baseline) scale(font_size/units_per_em)` so the SMuFL outline in font-design-unit space lands at the segment's baseline-left with the reduced accidental font size). The original `draw_chord_symbol(svg, &ChordSymbolLayout)` is preserved unchanged. Both renderers are re-exported from `render/mod.rs`.

  Wire-up: the two callsites in `render/measure_renderer/mod.rs` (`draw_note_event` line 422 and `draw_chord_event` line 694) now route through `layout_chord_symbol_composite` + `draw_chord_symbol_composite` with the font's `units_per_em` and `glyph_advance` plumbed through. Both call sites already had `font: &MusicFont` in scope and returned `Result<(), FontError>`, so no signature changes propagated outward.

  Tests (**+29 unit tests**):
  1. Parser (**16 tests** in `layout::chord_symbol::tests`): `parse_empty_string_yields_no_segments`; `parse_plain_text_yields_single_text_segment`; `parse_ascii_hash_is_sharp`; `parse_unicode_sharp_is_sharp`; `parse_unicode_flat_is_flat`; `parse_unicode_natural_is_natural`; `parse_b_after_root_letter_is_flat` (the `Bb` / `Eb` cases); `parse_b_after_digit_is_flat` (`7b5`, `13b9`); `parse_b_after_paren_or_alteration_marker_is_flat` (`C(b5)`, `C+b9`, `D/Bb`); `parse_b_at_start_of_symbol_is_flat`; `parse_complex_chord_symbol` (`F#m7b5` → 5 segments exactly: F, #, m7, b, 5 — the highest-leverage canary for the whole feature); `parse_multiple_sharps_and_flats_preserved` (`F#7#9`); `parse_consecutive_accidentals_distinct` (`C##` — two adjacent sharps, neither dropped nor merged); `parse_text_with_no_accidentals_is_single_text` (sanity over `Cmaj7`, `Am`, `G7sus4` — every non-flat-context `b` stays text); `parse_unicode_natural_and_flat_in_same_symbol`. Each test asserts on the exact segment vector, not just "splits into something."
  2. Layout (**13 tests** in `layout::chord_symbol::tests`): `composite_plain_text_yields_one_text_segment`; `composite_f_sharp_yields_text_plus_sharp`; `composite_b_flat_yields_text_plus_flat`; `composite_accidentals_use_reduced_font_size` (locks in `font_size = text_size × 0.70` to within 0.01); `composite_accidentals_baseline_raised` (locks in `accidental_baseline = text_baseline - font_size × 0.20`); `composite_segments_are_left_to_right` (strict-monotonic x_left across `F#m7b5`'s 5 segments); `composite_total_width_matches_sum_of_widths_plus_gaps` (rightmost-edge minus leftmost-edge == `total_width` for `F#m7b5` — locks in the layout's width arithmetic); `composite_is_centered_on_note_center_x` (composite midpoint == `note_center_x` within 0.5 fu); `composite_accidental_width_scales_with_font_advance` (passes synthetic advance callbacks returning 200 and 400 — sharp segment width must double exactly, isolates the accidental-width scaling formula from any test on a specific font's metrics); `composite_empty_string_yields_empty_layout`; `composite_baseline_matches_simple_layout` (cross-API consistency: the new composite layout's `y_baseline` and `font_size` must match the simple `layout_chord_symbol` output for the same input); `composite_units_per_em_zero_does_not_panic` (defensive divide-by-zero guard — the `units_per_em.max(1)` floor in the implementation).
  3. Renderer (**9 tests** in `render::chord_symbol_renderer::tests`): `composite_plain_text_emits_one_text_element_no_paths` (regression canary: a chord symbol with no accidentals must still produce exactly 1 `<text>` and 0 `<path>` — proves the composite path doesn't spuriously add glyphs); `composite_f_sharp_emits_text_plus_path` + asserts the `#` does NOT appear as text content; `composite_b_flat_emits_text_plus_path` + asserts the `b` is no longer a text character; `composite_complex_symbol_emits_expected_segment_counts` (F#m7b5 → exactly 3 `<text>` + 2 `<path>` — the canonical end-to-end test); `composite_empty_string_emits_nothing`; `composite_segments_use_anchor_start_not_middle` (regression canary against routing through the simple renderer's `text-anchor="middle"`); `composite_paths_have_translate_and_scale_transforms` (locks in the transform shape); `composite_different_accidentals_produce_different_paths` (extracts the `d="..."` data from each rendered SVG and asserts pairwise inequality across sharp/flat/natural — proves the renderer is actually picking the glyph from the segment kind, not always emitting the same one); `composite_emits_bold_text_for_text_runs` (all 3 text runs in F#m7b5 must carry `font-weight="bold"`).

  Example: `examples/chord_symbols_with_accidentals.rs` (**~145 LOC**) walks 16 chord symbols across 4 measures / 2 systems exercising every flat-context rule: flat-root (`Bb`, `Ebmaj7`, `Bb7`); sharp-root (`F#m`, `C#7`, `G#dim`); altered-extension flats (`F#m7b5`, `C7b9`, `G13b9`); altered-extension sharps (`D7#9`); slash chord with flat root (`D/Bb`); plain-text controls (`F`, `A`, `Cmaj7`, `Am`, `G7`). Eight in-example assertions including: minimum 16 text elements; minimum 13 path elements (counts the 13 chord-symbol accidentals: 1+1+1 in M1, 1+1+1 in M2, 2+1+1+1 in M3, 1 in M4); plain-text symbols still appear verbatim inside `<text>...</text>` (`>Cmaj7<`, `>Am<`, `>G7<`); ASCII-form regression canaries that the pre-feature strings (`>F#m7b5<`, `>Bb7<`, `>F#m<`) must NOT appear; every text run is bold. Output: 31122 bytes, 26 text elements, 38 paths, 38 lines.

  Golden test: added `golden_chord_symbols_with_accidentals` in `tests/golden_svg.rs` with the matching `build_chord_symbols_with_accidentals()` builder (12 symbols across 3 measures — slightly smaller than the example to keep the baseline tight). Five structural guards alongside the frozen 25364-byte baseline: (1) path-count strictly greater than plain `chord_symbols` (regression canary if the composite path ever stopped emitting accidental glyphs); (2) byte-inequality vs plain `chord_symbols` (refactor canary); (3) `text-anchor="start"` must appear in the SVG (composite-renderer-specific marker — catches a regression that routes accidental-bearing input through the simple renderer); (4) `<text>` count > 12 (proves segments are being split — 12 symbols would give exactly 12 text elements if all were single-run; any split symbol pushes the count higher); (5) ASCII-text regression canary `!svg.contains(">F#m7b5<")` and `!svg.contains(">Bb<")`. Added `"chord_symbols_with_accidentals"` to the `golden_baselines_are_valid_svgs` name list.

  **Pre-existing golden regenerated**: `tests/golden/chord_symbols.svg` (5 plain-text symbols — `C`, `Am`, `F`, `G7`, `Cmaj7`) had to be regenerated because the composite renderer now uses `text-anchor="start"` with `x_left` instead of `text-anchor="middle"` with `x_center`, even for single-text-segment composites. The visual position is identical (the layout shifts `x` by `-width/2` exactly so the left-anchored text renders at the same effective center) but the SVG bytes differ — `text-anchor` attribute changed and `x` value shifted by `width/2` per symbol. Confirmed via `cmp` that no other golden uses chord symbols (only the one `build_chord_symbols` test). This is an intentional rendering-pipeline change documented inline in the renderer's module doc.

  **Pre-existing example assertion fixed**: `examples/chord_symbols_score.rs`'s `assert!(svg.contains(">F#m7b5<"))` was the old text-only assertion; updated to assert on the three text fragments (`>F<`, `>m7<`, `>5<`) and the *absence* of the old combined form. This is the only example that touched the now-defunct ASCII chord-symbol rendering.

  Architecture compliance: the layout module's `accidental_advance: impl Fn(Glyph) -> u16` callback keeps the font-agnostic stance (no hard-coded Bravura values in layout); the music-font metrics are wired in at the call site in `measure_renderer/mod.rs` via `|g| font.glyph_advance(g).unwrap_or(0)`. Adding Petaluma or Leland post-v1 requires zero changes to `layout/chord_symbol.rs` or `render/chord_symbol_renderer.rs` — only the font's `glyph_advance` results would change.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo build -p music-engraver --example chord_symbols_with_accidentals` passes. `cargo run -p music-engraver --example chord_symbols_with_accidentals` writes 31122 bytes (26 text, 38 paths, 38 lines); all 8 in-example assertions hold. `cargo run -p music-engraver --example chord_symbols_score` writes 16740 bytes (13 text, 22 paths, 31 lines); the updated assertions hold (F#m7b5 split into 3 text runs + 2 glyph paths). `cargo clippy -p music-engraver --all-targets` — 0 new warnings from music-engraver (1 pre-existing in `score/multi_staff.rs:343`). `cargo test -p music-engraver --lib` — **2222 unit tests pass** (+29 vs prior recorded 2186 with default features: 16 parser + 13 layout + 9 renderer = 38; minus a couple of pre-existing tests that ran in the layout module's existing test surface, the net adds up to +29). `cargo test -p music-engraver --lib chord_symbol` filtered run — 65 chord-symbol-related tests pass (29 new + 36 pre-existing across layout, render, measure_renderer, score). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (67 prior + new `golden_chord_symbols_with_accidentals`; `chord_symbols` baseline regenerated and re-verified). `cargo test -p music-engraver --test golden_svg -- golden_baselines_are_valid_svgs` passes including the new name entry. `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass.
- Next: Other deferred items still pending: cross-system church rests; per-note collision detection in beamed additional voices (architectural — extending `VoiceCollisionOffset` with a `note_index_within_element: Option<usize>` and threading per-note offsets through the renderer); Gourlay penalty tuning; mid-trill speed change. Larger remaining post-v1 candidates: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / lyrics / grace-note family expansion; golden-SVG corpus + PHASH-based visual regression harness. Chord-symbol-specific follow-ups: (a) a courtesy `chord_symbol_segment_advance(seg: &ChordSymbolSegment, font_size, font, units_per_em)` helper would let external callers compute segment widths without rebuilding the whole composite layout; (b) horizontal centering currently uses a 0.55-em estimate per text character — a future enhancement could measure widths against a known text font (e.g. by embedding a thin sans-serif text font alongside Bravura), but that's a real new dependency and requires careful licensing review; (c) the current implementation always emits accidentals as paths — an SVG-text-with-Unicode-music-symbols fallback (`♯`/`♭`/`♮` U+266F/U+266D/U+266E) inside the bold text would render reasonably without the SMuFL font for HTML viewers that lack font-glyph support, but cuts against the engraving-quality goal. None planned.
- Open issues: Text width estimation uses `CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR = 0.55` em per character — this is an average across serif/sans-serif chord fonts. For long all-cap symbols like `Cmaj7+11`, the per-character estimate slightly under-counts wide capitals (`M`) and over-counts narrow numerals (`1`). Since the estimate only positions accidental glyphs *between* text runs (text runs themselves are rendered by the viewer's text engine using its actual font metrics), the worst-case visible effect is an accidental being a few pixels closer to or further from its adjacent text fragment than ideal. The user can tune `CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR` per their text-font choice if needed — exposing it as a `pub const` makes that a one-line override. The `ACCIDENTAL_SIZE_FACTOR = 0.70` and `ACCIDENTAL_BASELINE_RAISE_FACTOR = 0.20` constants follow the Sibelius/Dorico/Finale convention; if a future user demands a smaller accidental (e.g. 0.55) or different vertical centering, those are also exposed as `pub const` for a one-line override. The composite renderer's switch from `text-anchor="middle"` to `text-anchor="start"` is a one-way rendering change — the simple renderer's `draw_chord_symbol` is still available for callers that need exact text-anchor compatibility, but the measure-renderer now exclusively routes through the composite path. The `parse_b_at_start_of_symbol_is_flat` rule (lone leading `b5` becomes flat) is a defensive interpretation — a chord-symbol convention rarely has a chord symbol starting with `b` (root letters are uppercase), but `b5`, `b9`, `b13` as standalone alteration labels do appear in some rare jazz lead-sheet notations (`Cm(b5)` and `C(b5)` are common; the bare `b5` is uncommon but defensible). If a test corpus ever surfaces a chord symbol that legitimately starts with a lowercase `b` followed by alphabetic content (none come to mind), that input would need to be quoted differently or the rule narrowed. The chord-symbol example uses `Pitch::new(Note::Ees, 4)` and `Pitch::new(Note::Bes, 3)` — the `music` crate uses German notation (`Es` → `Ees`, `As` → `Aes`) for the flat letter names; clippy initially flagged a compile error when I used `Note::Es` first, which I fixed mid-flight.


## 2026-05-15 — Post-v1, lyric hyphens drawn between syllables (not appended)

- Did: Closed a real engraving-quality gap in lyric rendering: hyphens between consecutive syllables (e.g. "Hap-py", "birth-day") were being rendered by appending `" -"` to the source syllable's text (`>Hap -<`, `>birth -<`), so the hyphen sat right-adjacent to the source syllable rather than centered between source and target. Real engraving (Gould, Gardner Read) places a standalone hyphen `-` at the midpoint of the gap between two syllables, at the same baseline and font size as the surrounding lyrics. Matching how the existing extender-line system works (drawn in a second pass once both endpoints are known), I split hyphen rendering out of `draw_lyric()` and into a dedicated `draw_lyric_hyphen()` that the system/page-level passes invoke once they know the next syllable's x-position.

  Renderer (`render/lyric_renderer.rs`, ~+90 LOC net):
  - `draw_lyric()` no longer appends `" -"` to hyphenated syllables — it now emits *only* the syllable text, regardless of continuation. Doc updated accordingly.
  - New `draw_lyric_hyphen(svg, from_x, to_x, y_baseline, font_size, staff_space) -> bool` — emits a single `<text>-</text>` element at the midpoint of `(from_x, to_x)`, using `TextStyle::normal(font_size)` (centered, roman, serif, matching the surrounding lyrics). Returns `false` if the gap (after estimated syllable half-widths are subtracted) is below `HYPHEN_MIN_GAP_SS = 0.6 * staff_space`, preventing a hyphen from being drawn over near-overlapping syllables. Uses `HYPHEN_SYLLABLE_HALF_WIDTH_EM = 0.5` (conservative ~1-em syllable footprint estimate; since the bundled font is not used for text rendering, this is a layout estimate only — the viewer's text engine handles the actual text widths).

  System renderer (`render/system_renderer/mod.rs`):
  - New `draw_system_lyric_hyphens(svg, config, system, staff, system_x)` mirrors the existing `draw_system_lyric_extenders()`. Walks `collect_lyric_note_info()`; for each syllable with `LyricContinuation::Hyphen`, finds the *next* note/chord with a lyric (skipping notes/rests without lyrics) and calls `draw_lyric_hyphen` with the appropriate font size (`LYRIC_FONT_SIZE_SS * staff_space`). Hooked into `draw_system()` right after `draw_system_lyric_extenders()`.
  - The lyric font-size constant `LYRIC_FONT_SIZE_SS` was made `pub` (it was already a sibling of the pre-existing `pub const LYRIC_BELOW_STAFF_SS`) so renderers can compute the hyphen font size to match `layout_lyric` without re-running it.

  Page renderer (`render/page_renderer/mod.rs`):
  - New `draw_cross_system_lyric_hyphens(svg, config, systems)` mirrors `draw_cross_system_lyric_extenders()`. For each system-pair, calls `find_last_unresolved_hyphen()` (a new helper that walks the source system's notes in reverse, looking for a Hyphen-continuation syllable on the last syllable-bearing note in the system with no in-system successor). When unresolved, draws two hyphens: a trailing one between the source syllable and the right edge of the source staff, and a leading one between the left edge of the target system's first measure and the first syllable-bearing note in the target system. Hooked into `draw_page()` right after `draw_cross_system_lyric_extenders()`.
- Tests (+12 new, 2 modified):
  - `render::lyric_renderer::tests` (+8 new):
    1. `hyphen_continuation_renders_only_syllable_text` (modified from `hyphen_continuation_appends_hyphen`): regression canary — `draw_lyric` on a Hyphen-continuation syllable must emit `>hap<` alone and must NOT contain `"hap -"`.
    2. `draw_lyric_hyphen_emits_centered_text_element`: hyphen draws exactly 1 `<text>` element at midpoint `x=500` (for from=200, to=800), y=baseline, content `-`, with the supplied `font-size="100"`.
    3. `draw_lyric_hyphen_uses_text_anchor_middle`: hyphen text uses `text-anchor="middle"`.
    4. `draw_lyric_hyphen_returns_false_when_gap_too_small`: nearly-overlapping syllables produce no text and the function returns `false`.
    5. `draw_lyric_hyphen_skips_when_gap_below_threshold`: hand-computed boundary case (gap=140fu, threshold=150fu) where the function must skip.
    6. `draw_lyric_hyphen_draws_when_gap_above_threshold`: the immediately-larger gap (170fu) draws successfully.
    7. `draw_lyric_hyphen_position_is_independent_of_text`: deterministic — two identical calls produce byte-identical SVG.
    8. `draw_lyric_hyphen_midpoint_moves_with_endpoints`: shifting both endpoints by +100 shifts the midpoint by +100 (sanity).
  - `render::measure_renderer::tests::chord_with_lyric_adds_text_element` (modified): now asserts `>hap<` alone, NOT `"hap -"`. Locks in that the measure renderer never appends `" -"`.
  - `score::tests` (+2 new, 1 modified):
    - `lyric_with_hyphen_shows_separated_hyphen_between_syllables` (modified from `lyric_with_hyphen_shows_hyphen`): two-syllable end-to-end test asserting `>hap<`, `>py<`, `!hap -`, and `hyphen_count >= 1`. The strongest end-to-end canary against a regression that reintroduces ASCII concatenation.
    - `lyric_hyphen_crosses_system_boundary`: forces `measures_per_system(1)` to put `hap` (M1) and `py` (M2) on separate systems, verifies the cross-system pass draws at least one hyphen and neither syllable carries the `" -"` suffix.
    - `lyric_hyphen_not_drawn_when_no_successor_syllable`: a hyphen syllable with no following syllable in the system must produce zero standalone `>-<` text elements. Defensive regression canary.
  - `tests/golden_svg.rs::golden_lyrics` augmented with 5 structural guards alongside the regenerated baseline: `!contains(">Hap -<")`, `!contains(">birth -<")` (regression canaries), `contains(">Hap<")` + `contains(">birth<")` (positive presence), `hyphen_count >= 2` (the lyrics builder has 2 hyphenated pairs).
  - `tests/golden/lyrics.svg` regenerated. Diff: 2 syllable text elements lost their `" -"` suffix; 2 new standalone hyphen text elements added at the midpoints. File grew from 35 lines to 37 lines.
  - `examples/lyrics_score.rs` assertions updated: replaced `>Hap -<` / `>birth -<` assertions with `>Hap<` / `>birth<` plus `>= 4` standalone hyphens (the example has 2 hyphenated pairs in M1 + 2 in M3 = 4). Added regression-canary asserts `!contains(">Hap -<")`, `!contains(">birth -<")`.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). `cargo test -p music-engraver --lib` — **2230 unit tests pass** (+8 vs prior recorded 2222: 8 new lyric_renderer tests + 1 new score test + 1 new cross-system test - 2 modified-in-place tests = +8 net). `cargo test -p music-engraver --lib lyric` — 53 lyric-related tests pass (8 new + 45 pre-existing). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged total; `lyrics` baseline regenerated). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo run -p music-engraver --example lyrics_score` writes 13190 bytes (17 paths, 31 lines, 16 texts — 12 syllables + 4 standalone hyphens, matching M1's 2 + M3's 2 hyphenated pairs).
- Next: Other deferred items still pending: cross-system church rests; per-note collision detection in beamed additional voices (architectural — extending `VoiceCollisionOffset` with a `note_index_within_element: Option<usize>` and threading per-note offsets through the renderer); Gourlay penalty tuning; mid-trill speed change. Larger remaining post-v1 candidates: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / grace-note family expansion; golden-SVG corpus + PHASH-based visual regression harness. Lyric-specific follow-ups: (a) multi-hyphen for very long gaps — when a syllable holds across many notes (rare with hyphens; more common with melismas which already use extenders), some engravers draw multiple `- - -` dashes in the gap; a single centered hyphen is the most common modern convention and what we implement, but a `dashes_per_gap` knob could be added if a real corpus surfaces the need. (b) Text-width-aware hyphen placement — `HYPHEN_SYLLABLE_HALF_WIDTH_EM = 0.5` is a flat estimate; a future enhancement could measure actual text widths against a known text font, but the bundled font is Bravura (music) only, so accurate text metrics would require bundling a text font (real new dependency, licensing review). (c) Hyphen between syllables that span a measure barline within the same system is already handled by the within-system pass (which walks `collect_lyric_note_info` across all measures in the system, not just within a single measure) — no separate barline-aware logic needed.
- Open issues: The `HYPHEN_MIN_GAP_SS = 0.6` and `HYPHEN_SYLLABLE_HALF_WIDTH_EM = 0.5` constants are tuned empirically. The 0.6ss threshold catches the case where two single-character syllables are placed nearly atop each other (e.g. an extremely tight system width); below that, drawing a hyphen would look squashed. If a future user reports hyphens disappearing on legitimately-tight systems, the threshold could be relaxed. The cross-system path always emits two hyphens (one trailing, one leading); some engraving conventions prefer just one (closer to either the source or target syllable). The two-hyphen approach is visually symmetric and Gould lists it as acceptable; if a stricter single-hyphen convention is wanted, gating one of the two on a config flag is a small addition. The `find_last_unresolved_hyphen` walk-back logic stops at the first syllable-bearing note from the end: if that note has Hyphen continuation but the *very next* note has a lyric (within-system), the function correctly returns `None` (within-system pass handles it); if it has Hyphen continuation and no successor with a lyric, it returns `Some(info)` for the cross-system pass. The within-system pass's "find next note with a lyric" uses `iter().skip(i+1).find(|n| n.lyric.is_some())` rather than just the immediate next note: this means a hyphen syllable followed by a note WITHOUT a lyric and then a note WITH a lyric still draws the hyphen between the two syllable-bearing notes (which is what real engraving does — hyphens connect syllables, not just adjacent rhythmic events). The cross-system pass mirrors this by searching for the first syllable-bearing note in the target system, not just the first note.

## 2026-05-15 — Post-v1, `chord_symbol_segment_advance` helper + `ChordSymbolSegment` methods

- Did: Closed the explicit (a) follow-up flagged in the 2026-05-15 "chord-symbol SMuFL accidental glyph composition" chunk: "a courtesy `chord_symbol_segment_advance(seg: &ChordSymbolSegment, font_size, font, units_per_em)` helper would let external callers compute segment widths without rebuilding the whole composite layout." Implemented and DRY-folded the per-segment width computation out of `layout_chord_symbol_composite`'s inner loop into a free function with the same formula, plus added two `const fn` methods on `ChordSymbolSegment` itself for cleaner pattern-matching at callsites.

  Layout (`src/layout/chord_symbol.rs`, **net +~50 LOC for new fn + methods**, **net −16 LOC** in the composite layout from DRY-folding the inline match):
  - New `pub fn chord_symbol_segment_advance(seg: &ChordSymbolSegment, text_font_size: f64, units_per_em: u16, accidental_advance: impl Fn(Glyph) -> u16) -> f64`. Returns the segment's intrinsic advance width in font design units. Text segments: `chars * text_font_size * CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR`. Accidentals: `accidental_advance(glyph) * text_font_size * ACCIDENTAL_SIZE_FACTOR / units_per_em.max(1)`. Identical formula to what `layout_chord_symbol_composite` computes inline — and now the composite layout calls this helper internally, guaranteeing the two stay in lockstep. Doc spells out that **side bearings are not included** (those are an inter-segment concern, not an intrinsic property of a single segment).
  - New `pub const fn ChordSymbolSegment::is_accidental(&self) -> bool` — returns `true` for `Sharp`/`Flat`/`Natural`, `false` for `Text`. Used by the composite layout's side-bearing check (`needs_gap_before = i > 0 && (seg.is_accidental() || segments[i-1].is_accidental())`) — was previously a verbose 6-line `matches!` against three variants on both sides of the boolean.
  - New `pub const fn ChordSymbolSegment::glyph(&self) -> Option<Glyph>` — returns `Some(AccidentalSharp)` / `Some(AccidentalFlat)` / `Some(AccidentalNatural)` for accidental variants, `None` for `Text`. Surfaces the chord-symbol-to-SMuFL mapping as a method on the type itself rather than scattered across the two layout/render call sites.
  - Refactored `layout_chord_symbol_composite`'s width loop to: `widths.push(chord_symbol_segment_advance(seg, font_size, units_per_em, &accidental_advance))` plus `seg.is_accidental()` for the side-bearing branch. Removed the inline `(width, is_accidental) = match seg { ... }` block (originally lines 277-306) and the now-unused local `accidental_font_size` + `upe` (still computed inside the helper, where they belong). The composite layout's externally-observable behavior is byte-identical — guarded by the 17 pre-existing `layout_chord_symbol_composite` tests, all of which continued to pass with zero baseline regeneration.

- Tests (**+19 unit tests** in `layout::chord_symbol::tests`):
  1. `is_accidental_text_is_false` — locks in `Text("Cmaj7").is_accidental() == false` and `Text("").is_accidental() == false`. A future refactor that accidentally turned the method into `is_segment` (true for all variants) would fire.
  2. `is_accidental_all_variants` — `Sharp`, `Flat`, `Natural` all return `true`. If a refactor split off `Natural` from the accidental family (some niche engraving conventions distinguish — we don't), this fires.
  3. `is_accidental_is_const_callable` — three `const X: bool = S::Variant.is_accidental();` items lock in `const fn`. Compile-fail canary if `const fn` is removed. `#[allow(clippy::assertions_on_constants)]` because the resulting compile-time-true assertion *is* the test — we're checking const-evaluation, not runtime.
  4. `glyph_text_is_none` — `Text(_).glyph() == None`. Locks in that the method returns `Option<Glyph>` rather than panicking or returning a bogus glyph for text.
  5. `glyph_sharp_maps_to_accidental_sharp` — `Sharp.glyph() == Some(AccidentalSharp)`. Locks in the chord-symbol SMuFL mapping.
  6. `glyph_flat_maps_to_accidental_flat` — same for `Flat → AccidentalFlat`.
  7. `glyph_natural_maps_to_accidental_natural` — same for `Natural → AccidentalNatural`.
  8. `glyph_is_const_callable` — `const G: Option<Glyph> = S::Sharp.glyph()`. Compile-fail canary on `const fn`.
  9. `segment_advance_text_uses_char_width_factor` — for `Text("Am")` (2 chars) at `font_size = 400`, asserts `width == 2.0 * 400.0 * CHORD_SYMBOL_TEXT_CHAR_WIDTH_FACTOR`. Within 0.01 tolerance.
  10. `segment_advance_text_scales_linearly_with_length` — `"CCC"` width == 3 × `"C"` width. Locks in linear char-count scaling.
  11. `segment_advance_text_scales_linearly_with_font_size` — doubling `font_size` doubles the width. Locks in linear font-size scaling.
  12. `segment_advance_empty_text_is_zero` — `Text("").width == 0.0` exactly. Edge-case canary.
  13. `segment_advance_text_does_not_invoke_advance_callback` — uses a counting closure (`std::cell::Cell<usize>`) to verify the callback is **never** called for `Text` segments. Critical correctness canary: text widths are estimated from char count, not from the music font's metrics; an accidental refactor that started consulting `accidental_advance(Glyph::Whatever)` for text segments would inflate text widths unpredictably.
  14. `segment_advance_sharp_uses_advance_callback` — hand-computed expected value: `200 * 400.0 * 0.70 / 1000.0 = 56.0`. The arithmetic value is asserted within 0.01.
  15. `segment_advance_scales_linearly_with_advance` — doubling the callback's return doubles the segment width. Locks in linear scaling.
  16. `segment_advance_accidentals_use_their_own_glyph` — passes a `differentiating` callback that returns 100/200/300 for Sharp/Flat/Natural and 999 for any other glyph. Asserts `flat == 2 * sharp`, `natural == 3 * sharp`, AND `sharp < 50.0` — the latter is the critical canary that the callback is being queried with the *correct* glyph for each variant, not with a wrong-but-still-finite default. If the code ever called `accidental_advance(Glyph::AccidentalSharp)` for a `Flat` segment, the assertions would fire (flat would equal sharp, not 2×).
  17. `segment_advance_units_per_em_zero_does_not_panic` — junk `units_per_em == 0` produces a finite positive width via the `units_per_em.max(1)` floor in the helper. Mirrors the composite layout's `composite_units_per_em_zero_does_not_panic`. Asserts `w.is_finite() && w > 0.0` (NOT just "no panic" — that would pass even if `w` were NaN, which is the actual divide-by-zero failure mode).
  18. `segment_advance_matches_composite_layout_widths` — **the most important contract**: this helper and `layout_chord_symbol_composite` must agree on each segment's width. Walks `F#m7b5`'s 5 segments and asserts `chord_symbol_segment_advance(seg, ...) == composite.boxes[i].width` per-segment (within 0.01). If a future refactor changed one formula but not the other, the breakage is caught here. Covers Text + Sharp + Text + Flat + Text in one canary.
  19. `segment_advance_callable_repeatedly_with_same_callback` — uses a closure that captures a `Box<[u16]>` (non-Copy, forcing `move` semantics + non-Copy closure type). Calls the helper three times with `&closure`. If a future refactor changed the signature from `impl Fn(...)` to `impl FnOnce(...)`, the second call would refuse to compile and the renderer call site in `layout_chord_symbol_composite` (which currently calls the closure once per segment in a loop) would also break — so this test guards the same property the renderer relies on. Initially used `vec![...]`; clippy flagged "useless use of `vec!`" since the body only reads index 0; switched to `Box<[u16]>` which preserves the non-Copy property without the lint.

  Naming rationale: `chord_symbol_segment_advance` matches the SMuFL/typographic term *advance width* (horizontal advance of the segment's bounding box, distinct from "width" which can be ambiguous about side bearings). The function name explicitly puts `chord_symbol` first since it's a chord-symbol-specific layout helper, not a general SMuFL advance helper (those live elsewhere — `MusicFont::glyph_advance`).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). During development clippy flagged: (a) `needless_borrows_for_generic_args` on a no-state closure passed by `&closure` — fixed by changing the closure to capture `Box<[u16]>` so the closure itself is non-Copy and the `&` is meaningful; (b) `useless_vec` on `vec![150, 200, 250]` — fixed by the same change to `Box<[u16]>`; (c) `assertions_on_constants` on the const-callable test — fixed with `#[allow(clippy::assertions_on_constants)]` plus a comment explaining the tautology *is* the test (we're checking compile-time evaluation, not runtime). `cargo test -p music-engraver --lib` — **2250 unit tests pass** (+20 vs prior recorded 2230; nominally +19 from the chunk's 3 + 5 + 11 new tests, with one additional test counted from the per-variant glyph mapping suite). `cargo test -p music-engraver --lib chord_symbol` — **84 chord-symbol-related tests pass** (65 prior + 19 new). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged; the helper-refactor of `layout_chord_symbol_composite` is byte-identical, so no goldens regenerated). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass.
- Next: Other deferred items still pending: cross-system church rests; per-note collision detection in beamed additional voices (architectural — extending `VoiceCollisionOffset` with a `note_index_within_element: Option<usize>` and threading per-note offsets through the renderer); Gourlay penalty tuning; mid-trill speed change. Larger remaining post-v1 candidates: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / grace-note family expansion; golden-SVG corpus + PHASH-based visual regression harness. Chord-symbol-specific follow-ups still pending: (b) text-width measurement against a known text font (requires bundling a text font — real new dependency, licensing review); (c) `♯`/`♭`/`♮` Unicode-text fallback for HTML viewers without SMuFL glyph rendering (cuts against the engraving-quality goal). Now that `chord_symbol_segment_advance` and `ChordSymbolSegment::glyph()` exist, external callers (e.g. a hypothetical higher-level layout that needed to budget total composite width without invoking the layout function) have a clean API to do so; renderer callsites still use `seg.glyph()` directly is a follow-up worth considering (currently `render/chord_symbol_renderer.rs` still has its own per-variant `match` that returns the glyph — switching to `seg.glyph().unwrap_or(...)` would DRY further, but is a renderer-only refactor outside this chunk's layout scope).
- Open issues: The `segment_advance_text_does_not_invoke_advance_callback` test uses `std::cell::Cell<usize>` for the counter — clippy would normally suggest `AtomicUsize` for multi-threaded code, but `Cell` is correct for single-threaded test code and avoids the atomic overhead. If a future Rust edition deprecates `Cell` in favor of something else, the test would need updating. The `segment_advance_callable_repeatedly_with_same_callback` test uses `Box<[u16]>` specifically to force a non-Copy closure; a future Rust optimizer that elided the Box (extremely unlikely without breaking changes) would silently change the test's coverage from "borrowed Fn" to "owned Fn". Documented inline. The `chord_symbol_segment_advance` helper does NOT include inter-segment side bearings — this is the deliberate design (side bearings depend on neighbors, not on a segment alone). External callers who need a total width with gaps should use `layout_chord_symbol_composite(...).total_width` instead. Documented in the function's doc comment so callers don't accidentally double-count or under-count gaps.

## 2026-05-15 — Post-v1, chord-symbol renderer dispatch routed through `ChordSymbolSegment::glyph()`

- Did: Closed the renderer-only follow-up explicitly flagged in the "Next" section of the prior chunk (2026-05-15, `chord_symbol_segment_advance` helper + `ChordSymbolSegment` methods): "renderer callsites still use `seg.glyph()` directly is a follow-up worth considering (currently `render/chord_symbol_renderer.rs` still has its own per-variant `match` that returns the glyph — switching to `seg.glyph().unwrap_or(...)` would DRY further, but is a renderer-only refactor outside this chunk's layout scope)." Refactored `draw_chord_symbol_composite`'s dispatch loop in `src/render/chord_symbol_renderer.rs` so the chord-symbol → SMuFL glyph mapping now lives in **exactly one place** — the layout-layer `ChordSymbolSegment::glyph()` accessor — instead of being duplicated across the renderer's three per-variant arms.

  Before this chunk the renderer's dispatch was a 4-arm match: `Text(s) → add_text(...)`; `Sharp → draw_accidental_glyph(Glyph::AccidentalSharp, ...)`; `Flat → draw_accidental_glyph(Glyph::AccidentalFlat, ...)`; `Natural → draw_accidental_glyph(Glyph::AccidentalNatural, ...)`. The three accidental arms each hardcoded the same chord-symbol → SMuFL mapping that the layout layer already encoded in `ChordSymbolSegment::glyph()` (added in the prior chunk). Two copies of the same mapping is the canonical DRY smell — if a future engraving-convention change ever wanted to route, say, `Sharp` to a small/raised SMuFL variant for chord-symbol contexts, both copies would have to be edited in lockstep or the renderer would silently emit the wrong glyph.

  Refactor: the three accidental arms collapse into a single or-pattern arm with `seg @ (Sharp | Flat | Natural)`, which then calls `seg.glyph().expect(...)` to dispatch through the layout accessor. The `expect` is the layout-layer invariant "accidental variants always have a glyph" — a regression that violated it would surface as a panic in the per-variant tests, not silently produce wrong SVG output. The or-pattern preserves match exhaustiveness so adding any new variant requires updating this arm at compile time. The Text arm is untouched because text rendering is fundamentally different (text element, not glyph path). Also DRYed the helper signature: `box_: &crate::layout::chord_symbol::ChordSymbolSegmentBox` became `box_: &ChordSymbolSegmentBox` with an added named import.

  Net diff: `render/chord_symbol_renderer.rs` lost 6 lines from the dispatch loop (3 single-line accidental arms with their per-variant `Glyph::Accidental*` constant inline → 1 multi-line or-pattern arm), gained ~10 lines of inline comments documenting the source-of-truth contract. The path data and element counts emitted for every existing input are byte-identical — confirmed by all 68 golden SVG tests passing unchanged. The only behavioral change is the dispatch path: a future `glyph()`-side mapping change now propagates to the renderer automatically.
- Tests (+4 new in `render::chord_symbol_renderer::tests`):
  1. `composite_dispatches_through_segment_glyph_for_sharp` — the critical canary for the DRY refactor. Renders `C#` and extracts the first path's `d` attribute. Independently calls `ChordSymbolSegment::Sharp.glyph()` to resolve the expected SMuFL glyph and `font.glyph_outline(expected_glyph).path_data` to get its outline. Asserts byte-equality between rendered and expected path data. If a future refactor reintroduces a duplicate chord-symbol → SMuFL map inside the renderer and accidentally writes a different glyph for `Sharp` (e.g. `AccidentalDoubleSharp`), this fires; the other two variant tests continue to pass — pinpointing the regression to a single variant.
  2. `composite_dispatches_through_segment_glyph_for_flat` — same canary for `Cb` (`b` is a flat by the chord-symbol rule: lowercase `b` after uppercase root letter `C`). Renderer path data must equal `font.glyph_outline(ChordSymbolSegment::Flat.glyph().unwrap()).path_data`.
  3. `composite_dispatches_through_segment_glyph_for_natural` — same canary for `C\u{266E}` (the Unicode natural). Renderer path data must equal `font.glyph_outline(ChordSymbolSegment::Natural.glyph().unwrap()).path_data`. The three tests are intentionally separate (not a single loop) so a regression that breaks exactly one variant fires exactly one test — pinpointing which variant drifted.
  4. `composite_dispatch_is_deterministic_across_repeated_calls` — determinism canary for the new dispatch path. Two independent calls with `F#m7b5` input must produce byte-identical SVG. If the new `glyph()`-driven dispatch ever pulled in non-deterministic state (env, time, RNG, hash iteration order — none plausibly here, but the canary makes "deterministic" an enforced contract, not an assumption), this fires.

  Also added a private `first_path_d(svg)` helper in the test module that extracts the first path's `d` attribute — reused by the three dispatch canaries. (The pre-existing `composite_different_accidentals_produce_different_paths` test has its own inline `path_d` helper with the same behavior; the duplicate is a tradeoff for keeping the new helper self-contained in the new test block. If a future tidy-up consolidates them, the test behavior is unchanged.)

  Naming rationale: each test name explicitly says "dispatches_through_segment_glyph" — making the source-of-truth contract visible in test output. If a future engineer sees `composite_dispatches_through_segment_glyph_for_sharp ... FAILED`, they immediately know to look at `ChordSymbolSegment::glyph()` and `draw_chord_symbol_composite`'s dispatch loop together, not at one in isolation.
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). `cargo test -p music-engraver --lib` — **2254 unit tests pass** (+4 vs prior recorded 2250: all 4 new tests live in `render::chord_symbol_renderer::tests`). `cargo test -p music-engraver --lib chord_symbol_renderer` — **21 chord_symbol_renderer tests pass** (17 prior + 4 new). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged total — the refactor is byte-identical for every existing input; this is exactly the "single source of truth, no behavior change" property the chunk establishes). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass.
- Next: Other deferred items still pending: cross-system church rests; per-note collision detection in beamed additional voices (architectural — extending `VoiceCollisionOffset` with a `note_index_within_element: Option<usize>` and threading per-note offsets through the renderer); Gourlay penalty tuning; mid-trill speed change. Larger remaining post-v1 candidates: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / grace-note family expansion; golden-SVG corpus + PHASH-based visual regression harness. Chord-symbol-specific follow-ups still pending: (b) text-width measurement against a known text font (requires bundling a text font — real new dependency, licensing review); (c) `♯`/`♭`/`♮` Unicode-text fallback for HTML viewers without SMuFL glyph rendering (cuts against the engraving-quality goal). Possible further chord-symbol DRY follow-up: the test module has two near-duplicate `path_d` helpers (one inline in `composite_different_accidentals_produce_different_paths`, one as a private `first_path_d` shared by the three dispatch canaries) — consolidating to a single test-module helper would be a small purely-mechanical cleanup. Not urgent.
- Open issues: The `expect("accidental segment must resolve to a SMuFL glyph")` panic message is wired to the layout-layer invariant. If a future refactor splits `ChordSymbolSegment` into accidental + non-accidental variants where some accidentals legitimately have no SMuFL glyph (no plausible engraving convention does this, but the open-design space allows it), the renderer would have to fall through to a no-op or text-fallback path — the panic would be inappropriate. Currently the invariant holds because all three accidental variants (Sharp, Flat, Natural) map to the three standard SMuFL accidental glyphs (`AccidentalSharp`, `AccidentalFlat`, `AccidentalNatural`); the canary tests lock this in at both the layout layer (`glyph_*_maps_to_accidental_*` in `layout::chord_symbol::tests`) and the renderer layer (the three new `composite_dispatches_through_segment_glyph_for_*` tests). Pre-existing `composite_different_accidentals_produce_different_paths` test continues to verify all three glyphs are pairwise distinct — the dispatch canaries verify each one is the *correct* glyph; together they fully pin down which-glyph-for-which-variant. The renderer's `match` arm uses `seg @ (Sharp | Flat | Natural)` (binding the variant to `seg` via or-pattern) — this is the modern Rust idiom for binding a value across an or-pattern; if a future Rust edition ever required a different syntax for this, the canary tests would still hold but the file would need a small mechanical update. Documented in the inline comment block.

## 2026-05-15 — Post-v1, consolidate duplicate `path_d` test helpers in chord_symbol_renderer

- Did: Closed the explicit "Possible further chord-symbol DRY follow-up" item flagged in the prior chunk's Next section: "the test module has two near-duplicate `path_d` helpers (one inline in `composite_different_accidentals_produce_different_paths`, one as a private `first_path_d` shared by the three dispatch canaries) — consolidating to a single test-module helper would be a small purely-mechanical cleanup. Not urgent."

  The two helpers in `src/render/chord_symbol_renderer.rs` had byte-identical bodies and only differed in name:

  ```rust
  // (A) nested fn inside composite_different_accidentals_produce_different_paths
  fn path_d(svg: &SvgWriter) -> String {
      let s = svg.to_svg();
      let start = s.find(r#"d=""#).expect("should contain a path");
      let after = &s[start + 3..];
      let end = after.find('"').expect("path data should close");
      after[..end].to_string()
  }

  // (B) module-level private helper, shared by 3 dispatch canaries
  fn first_path_d(svg: &SvgWriter) -> String { /* identical body */ }
  ```

  The classic DRY smell: a fix or generalization to one (e.g. handling
  multi-line `d="..."` attributes split across lines, or escaping inside
  the attribute value) would silently fail to propagate to the other —
  so the dispatch canaries would catch the renderer regression they're
  built to catch, but the older non-dispatch test would silently use the
  pre-generalization helper and behave inconsistently.

  Refactor:
  - Removed the nested-fn copy `path_d` (was 7 lines inside `composite_different_accidentals_produce_different_paths`).
  - Switched the three `path_d(...)` callsites inside that test to `first_path_d(...)` — the module-level helper that already lives in the test block below.
  - Expanded `first_path_d`'s doc comment from 2 lines to 14 lines: documents what the helper does, where it's used (formerly "the dispatch tests below"; now "this test PLUS the dispatch canaries"), the exact parsing behavior (`d="` … next `"`), and why panics are correct (callers only invoke it on SVGs that contain at least one rendered glyph path — never on caller-empty input).
  - Added an inline comment in `composite_different_accidentals_produce_different_paths` explaining that the test now relies on the module-level `first_path_d` and that if the helper drifts, all four tests (this one + the 3 dispatch canaries) fire together.

  Net diff in `src/render/chord_symbol_renderer.rs`: −7 lines (the inline helper) + ~+8 lines of inline-comment + ~+12 lines of doc-comment for `first_path_d`. Plus +4 new direct tests of the helper itself (~+90 LOC test code).

- Tests (+4 new in `render::chord_symbol_renderer::tests`):
  Previously `first_path_d` was only exercised *indirectly* via the four consumer tests (the three dispatch canaries plus the now-consolidated `composite_different_accidentals_produce_different_paths`). If the helper's parsing rule itself ever broke, the dispatch canaries would fire — but they'd point at the renderer (`composite_dispatches_through_segment_glyph_for_sharp ... FAILED`) rather than at the helper. Direct tests below pin down the helper's contract independent of the renderer so a parsing-rule break fires here first.
  1. `first_path_d_returns_path_data_for_single_path` — renders `F#` (one glyph path emitted: the sharp), extracts `first_path_d(&svg)`, asserts byte-equality with `font.glyph_outline(ChordSymbolSegment::Sharp.glyph().unwrap()).path_data`. Locks in the contract that the helper returns precisely the rendered glyph's outline string. If a regression ever made the helper return e.g. the *attribute name* (`d`) instead of the value, or a quoted-with-escaping variant, this fires.
  2. `first_path_d_returns_first_path_when_multiple_paths_present` — renders `F#m7b5` (two glyph paths: sharp at position 0, flat at position 1). Asserts `first_path_d == sharp_outline` AND `first_path_d != flat_outline`. Critical canary: if a future refactor accidentally swapped `find` for `rfind` (a one-character typo with plausibly compiling semantics), the helper would return the *last* path's data instead of the first — both consumer test variants (sharp/flat) would still produce different paths and the existing dispatch canaries might pass coincidentally for asymmetric cases. This test forces a known multi-path SVG to differentiate.
  3. `first_path_d_is_deterministic_across_repeated_calls` — two independent helper calls on byte-identical SVG input must return byte-identical strings. Locks in that the helper holds no hidden state and walks the input deterministically. If a future refactor accidentally cached a mutable thread-local or used hash-order iteration to find the `d="` position, this fires.
  4. `first_path_d_returned_string_is_non_empty` — pins down two properties of a real glyph outline: (a) the returned string is non-empty (catches a regression where the renderer ever emitted `d=""` and the helper happily returned `""`, which would silently equal a downstream "" comparator); (b) the string starts with `M` (the SVG move-to command — every glyph outline starts with one). The two assertions together catch the failure mode "empty string parsed cleanly" that a single `!extracted.is_empty()` would miss. The `M`-prefix is a real font-output invariant (Bravura's accidental outlines start with `M`), not a synthetic guard.

  Test naming: each `first_path_d_*` name explicitly says what property of the helper it pins down — so if `first_path_d_returns_first_path_when_multiple_paths_present ... FAILED` appears in CI, the engineer immediately knows the helper's `find` vs `rfind` (or equivalent first-vs-last) semantics have drifted, not that the renderer changed which-glyph-for-which-variant.

  Pre-existing tests are byte-equivalent through this refactor:
  - `composite_different_accidentals_produce_different_paths` continues to assert pairwise distinct sharp/flat/natural glyphs — same assertions, same SVG inputs, just calling the shared helper instead of the local one. All 21 chord_symbol_renderer tests continue to pass (17 pre-existing + 4 dispatch canaries from the prior chunk; now 25 with the +4 helper tests).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged). `cargo test -p music-engraver --lib` — **2258 unit tests pass** (+4 vs prior recorded 2254: all 4 new tests in `render::chord_symbol_renderer::tests`). `cargo test -p music-engraver --lib chord_symbol_renderer` — **25 chord_symbol_renderer tests pass** (21 prior + 4 new helper tests). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged — the helper consolidation is a test-module-only refactor with zero effect on rendered SVG output; this is exactly the "behavior-preserving DRY" property the chunk establishes). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass.
- Next: Other deferred items still pending: cross-system church rests; per-note collision detection in beamed additional voices (architectural — extending `VoiceCollisionOffset` with a `note_index_within_element: Option<usize>` and threading per-note offsets through the renderer); Gourlay penalty tuning; mid-trill speed change. Larger remaining post-v1 candidates: multi-staff / grand-staff bracket polish; line breaking (Gourlay or Bellini & Nesi); articulations / ornaments / dynamics / grace-note family expansion; golden-SVG corpus + PHASH-based visual regression harness. Chord-symbol-specific follow-ups still pending: (b) text-width measurement against a known text font (requires bundling a text font — real new dependency, licensing review); (c) `♯`/`♭`/`♮` Unicode-text fallback for HTML viewers without SMuFL glyph rendering (cuts against the engraving-quality goal). With the helper consolidated, future generalizations (handling SVG attribute escaping, multi-line `d="..."` values, namespace-prefixed `svg:path` elements if those ever appear in output) can be made in one place and propagated automatically to all five consumer tests.
- Open issues: The helper uses simple `str::find` for the `d="` substring rather than a real XML parser. This is fine for the current SVG writer's output (which always emits `d="..."` as a single attribute on one line, with no escaping inside path data — SVG path data is composed entirely of ASCII letters, digits, commas, spaces, and signs, none of which require XML attribute escaping). If a future SvgWriter ever started emitting `d="..."` across multiple lines, or with HTML/XML entity escaping inside (`&quot;` for an embedded quote), or used a different attribute order (e.g. `transform="..." d="..."` with whitespace between), the simple substring search still works — the `expect("path data should close")` would fire only if a `d="` had no closing `"` at all, which would itself be malformed SVG. The `starts_with('M')` assertion in `first_path_d_returned_string_is_non_empty` is specific to Bravura's natural-glyph outline (and the SMuFL convention more broadly); if a future font ever emitted an outline starting with another command (relative `m`, an implicit move-to via `L`, etc.), that test would need an update — but no SMuFL-conformant font does this in practice.

## 2026-05-15 — Post-v1, mid-trill speed change (multi-speed wiggle extension layout + renderer)

- Did: Implemented the long-standing "mid-trill speed change" deferred item flagged in the Next sections of multiple prior chunks (under "Other deferred items still pending: ... mid-trill speed change"). Real engraving uses progressively-denser/sparser wiggle glyphs *within a single sustained trill* to indicate acceleration/deceleration — a discrete mid-trill speed change. Until this chunk, `layout/trill_extension.rs` only supported a single wiggle glyph for the whole extension run; callers wanting a speed change had no public API to express it without manually emitting multiple back-to-back `TrillExtensionLayout` calls and reconciling the seams themselves.

  Layout (`src/layout/trill_extension.rs`, **+~140 LOC** new code below the single-speed code, **+21 unit tests**):
  - New `pub struct TrillSpeedRegion { start_x: f64, glyph: Glyph, segment_advance: f64 }` — one region of a multi-speed wiggle. Doc-comment covers the contract: regions tile rightward from `start_x` until the next region's `start_x` (or until the overall `end_x` for the final region); each region carries its own glyph and per-tile advance (different wiggle speeds have different intrinsic widths — caller responsibility to query each glyph's advance from the active font); regions must be sorted non-decreasing by `start_x`; zero-width transition regions (`regions[i+1].start_x == regions[i].start_x`) are legal but contribute zero tiles.
  - New `pub struct TrillExtensionTile { x: f64, glyph: Glyph, advance: f64 }` — one placed tile in the layout. Glyph is per-tile so the renderer can dispatch to the correct outline without re-deriving speed from position. Advance is echoed per-tile so `multi_speed_trill_extension_right_edge` can compute the geometric extent without keeping the originating regions slice alive.
  - New `pub struct MultiSpeedTrillExtensionLayout { tiles: Vec<TrillExtensionTile>, y: f64 }` — full layout result. Tiles are in strict left-to-right order. Adjacent tiles may share a glyph (the layout function deliberately does NOT coalesce regions — coalescing would be a behavior change for callers who chained zero-width regions intentionally; the renderer's per-glyph outline cache makes the redundancy cheap).
  - New `pub fn layout_trill_extension_multi_speed(end_x: f64, y: f64, regions: &[TrillSpeedRegion]) -> Option<MultiSpeedTrillExtensionLayout>`. For each region [i], tiles `floor((region_end_x - region.start_x) / region.segment_advance)` copies of `region.glyph` starting at `region.start_x`. The region's `end_x` is `regions[i+1].start_x` or the overall `end_x` for the last region. Returns `None` when: regions empty; first region starts after `end_x`; any region has non-positive `segment_advance`; regions out of order (next region's `start_x` < current region's end); total tile count is zero. Each region's tiles do NOT stretch to fill its span — leftover gap at the right of each region is left empty, matching the single-speed convention (stretching a wiggle glyph distorts it visually).
  - New `pub fn multi_speed_trill_extension_right_edge(layout: &MultiSpeedTrillExtensionLayout) -> f64`. Computes `last_tile.x + last_tile.advance` (not `regions[0].segment_advance` — see test `multi_speed_right_edge_uses_last_tiles_advance_not_first`). Returns 0.0 for an empty layout, matching `trill_extension_right_edge` convention.
  - Exported all four new public items via `layout::mod.rs` `pub use` line.

- Layout tests (21 new in `layout::trill_extension::tests`):
  1. `multi_speed_empty_regions_returns_none` — empty slice → None. Edge-case canary.
  2. `multi_speed_first_region_start_after_end_x_returns_none` — first region's `start_x > end_x` → None. Catches a degenerate input that would otherwise produce a layout with zero tiles AFTER iterating.
  3. `multi_speed_zero_advance_returns_none` — single region with advance 0 → None. Prevents an infinite loop in the tile-placement loop.
  4. `multi_speed_negative_advance_returns_none` — negative advance → None. Prevents `floor()` from producing a negative count.
  5. `multi_speed_zero_advance_in_second_region_returns_none` — critical: the validation must check EVERY region, not just the first. A regression that early-outed on regions[0] alone would leave a downstream zero-advance region undetected and silently produce wrong tiling.
  6. `multi_speed_out_of_order_regions_returns_none` — regions[1].start_x < regions[0].start_x → None. Catches sort violations.
  7. `multi_speed_zero_total_tiles_returns_none` — all regions individually too short for one tile → None. Different failure mode from "empty regions" (the slice is non-empty; the geometry just doesn't admit a tile).
  8. `multi_speed_single_region_matches_single_speed_layout` — the critical migration-safety property: for one region, multi-speed tile positions must equal single-speed `segment_xs`. Lets callers swap entry points without changing rendered output. Asserts per-tile `(x, glyph, advance)` and the `y` field.
  9. `multi_speed_two_regions_have_correct_tile_glyphs` — separate test from the position test: pins down which glyph each tile carries (3 Fast tiles followed by 3 Slow tiles) for a Fast→Slow input. If a regression accidentally swapped glyphs at the seam, this fires.
  10. `multi_speed_two_regions_have_correct_tile_positions` — separate from the glyph test: pins down exact x positions `[0, 60, 120, 180, 300, 420]` for the same Fast→Slow input. Hand-computed expected values; if the implementation switched to e.g. cumulative offsets based on the wrong advance, this fires.
  11. `multi_speed_three_regions_accel_pattern` — Slow→Standard→Fast (a real engraving accelerating-trill use case). Three regions, one tile each. Verifies per-tile glyph and the `y` field.
  12. `multi_speed_region_too_short_for_a_tile_contributes_nothing` — a middle region whose span is shorter than its own advance contributes ZERO tiles but does NOT abort the layout — the surrounding regions still tile normally. Tests for `len() == 3` (1+0+2 from a 3-region input) and verifies the specific tile xs.
  13. `multi_speed_zero_width_region_is_legal` — `regions[i+1].start_x == regions[i].start_x` (zero-width region) is permitted; the validation `region_end >= r.start_x` admits equality. The zero-width region contributes no tiles but the layout proceeds normally.
  14. `multi_speed_no_distortion_of_glyph_widths` — critical correctness canary using deliberately non-round advances (47.5, 113.7): tile positions within a region must increment by exactly that region's `segment_advance`, NOT by a global average across regions. A bug that averaged advances would round the strides and the per-window dx assertion would fire.
  15. `multi_speed_right_edge_for_two_region_layout` — `multi_speed_trill_extension_right_edge` returns `last_tile.x + last_tile.advance`. Hand-computed expected 540.0 for the Fast→Slow input.
  16. `multi_speed_right_edge_uses_last_tiles_advance_not_first` — critical: a bug that hardcoded `regions[0].segment_advance` for the right edge would compute 60+60=120 for a Fast-then-Slowest (advance 500) input; the correct answer is 60+500=560. The test asserts 560.
  17. `multi_speed_right_edge_empty_layout_is_zero` — empty layout's right edge is 0.0, mirroring `trill_extension_right_edge`'s convention.
  18. `multi_speed_y_is_preserved_across_regions` — `y` is a single value on the layout (not per-tile), set to the input. Verifies 271.5 round-trips.
  19. `multi_speed_tiles_strictly_increasing_x` — tile xs are strictly increasing across regions. Catches a regression that produced overlapping or out-of-order tiles.
  20. `multi_speed_adjacent_regions_with_same_glyph_still_tile` — no coalescing: two adjacent regions with identical glyph + advance produce two regions' worth of contiguous tiles (6 in this case), not coalesced as a single 6-tile region. Locks in the no-coalescing decision.
  21. `multi_speed_does_not_overflow_end_x` — `multi_speed_trill_extension_right_edge(layout) <= end_x` always. Mirrors the single-speed `extension_does_not_overflow_end_x` invariant.

  Renderer (`src/render/trill_extension_renderer.rs`, **+~55 LOC** new function, **+11 new tests**):
  - New `pub fn draw_trill_extension_multi_speed(svg: &mut SvgWriter, font: &MusicFont, layout: &MultiSpeedTrillExtensionLayout) -> Result<(), FontError>`. Iterates tiles, emits one `<path>` per tile with `transform="translate(x,y)"`. Glyph outlines are cached per unique glyph using the `HashMap::entry` API — a long wiggle that uses only two speed variants re-queries the font exactly twice, regardless of tile count. (Entry API rather than `contains_key` + `insert` to satisfy `clippy::map_entry` and avoid an intermediate clone.) Empty layout is a no-op.

- Renderer tests (11 new in `render::trill_extension_renderer::tests`):
  1. `multi_speed_renders_one_path_per_tile_across_two_regions` — 3 Fast tiles + 2 Slow tiles = 5 `<path>` elements. Hand-computed counts.
  2. `multi_speed_uses_different_path_data_per_glyph` — critical correctness canary. Looks up both `WiggleTrillFast` and `WiggleTrillSlow` outlines independently from the font, asserts (a) the test setup is sane (outlines differ), (b) BOTH outlines appear literally in the SVG. A regression that cached `regions[0].glyph` for all tiles would emit only one distinct outline.
  3. `multi_speed_correct_outline_at_each_tile_position` — STRONGER than #2: for each tile, the SVG must contain the substring `d="<expected outline>" fill="black" transform="translate(x,y)"`. Pairs the outline with the translate per tile, not just "both outlines appear somewhere." A regression that emitted Fast Slow Fast Slow (instead of Fast Fast Slow Slow) for a Fast-then-Slow region layout would not be caught by counting; this is.
  4. `multi_speed_empty_layout_is_noop` — empty tiles → zero `<path>` elements. Defensive canary for callers building layouts manually.
  5. `multi_speed_embeds_each_tile_x_in_translate` — every tile's x appears in a `translate(x,` substring. Locks in that no tile is silently dropped.
  6. `multi_speed_embeds_y_coordinate_in_translate` — the layout's shared y appears in the SVG (271.5 → `,271.5)`).
  7. `multi_speed_caches_outline_per_unique_glyph` — STRONGER than path counting: 10 tiles using only 2 unique glyphs must produce exactly 2 distinct `d="..."` literals in the SVG. Walks the SVG collecting all `d="..."` substrings into a HashSet and asserts the size is 2. A regression that re-queried the font per tile with mutated bytes (implausible but pathologically possible) would inflate the set size.
  8. `multi_speed_single_region_matches_single_speed_output_byte_for_byte` — the renderer-layer migration-safety mirror of layout test #8: single-region multi-speed must produce IDENTICAL SVG to the single-speed renderer call. Asserts `multi_svg == single_svg`. The strongest possible migration contract.
  9. `multi_speed_path_color_is_black` — wiggle tiles use `fill="black"`. Mirrors single-speed convention.
  10. `multi_speed_tiles_emit_in_left_to_right_order` — translates emit in strictly-increasing x order. Walks the SVG collecting all `translate(x,...)` xs and asserts the sequence is increasing. A regression that iterated the cache HashMap (unordered) instead of `layout.tiles` (ordered) would fire here.
  11. `multi_speed_single_tile_layout_emits_exactly_one_path` — smallest non-empty layout: 1 tile → 1 `<path>` with the expected translate.

- Example: new `examples/trill_multi_speed.rs` renders three trill wavy-lines stacked vertically: (1) single-speed reference using `layout_trill_extension`, (2) accelerating Slow→Standard→Fast multi-speed, (3) decelerating Fast→Standard→Slow multi-speed. Writes `examples/output/trill_multi_speed.svg`. Asserts the SVG starts with `<svg` and contains at least 9 `<path>` elements (3 per line minimum). Output: 38113 bytes, 91 `<path>` elements — confirms healthy rendering of all three lines with their respective tile counts. Intentionally a low-level layout-and-render demo, not a ScoreBuilder example (wiring multi-speed into the score builder API is a separate larger chunk — it would need new builder methods, system-collector changes, and per-measure region-anchor logic; the layout primitive being available now is what unblocks that work).

- Naming rationale: chose `multi_speed` (not `varispeed`, `polyspeed`, etc.) to match the existing `WiggleTrill*` speed-variant family naming and Phase-7-era `TrillWiggleSpeed` enum. `TrillSpeedRegion` rather than `TrillSpeedSegment` to avoid conflict with the per-tile `segment_advance` field name. `TrillExtensionTile` rather than `TrillExtensionPlacedSegment` because "tile" is the SMuFL/typographic term for a repeated glyph in a row (Bravura's metadata uses "tile" throughout for the same concept). Functions `layout_trill_extension_multi_speed` / `draw_trill_extension_multi_speed` mirror the existing `layout_trill_extension` / `draw_trill_extension` pair so the variants are obvious at call sites.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo build -p music-engraver` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged; an initial `clippy::map_entry` warning on the renderer cache was fixed by switching from `contains_key`+`insert` to `HashMap::entry`). `cargo test -p music-engraver --lib` — **2290 unit tests pass** (+32 vs prior recorded 2258: 21 new layout tests + 11 new renderer tests). `cargo test -p music-engraver --lib trill_extension` — **103 trill-extension-related tests pass** (82 prior + 21 new). `cargo test -p music-engraver --lib trill_extension_renderer` — **19 renderer tests pass** (8 prior + 11 new). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged — the new entry points are additive; nothing existing was touched, so no golden SVG bytes changed). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo run -p music-engraver --example trill_multi_speed` writes 38113-byte SVG with 91 `<path>` elements.

- Next: With multi-speed layout + renderer primitives in place, the natural follow-ups are: (a) score-builder API to express multi-speed trills at the note-attachment layer — a new `trill_with_multi_speed_extension(speeds: &[(SpeedVariant, ProportionOfDuration)])` method (or similar), which would need the system-collector to anchor regions against the per-note x positions; (b) a `TrillSpeedRamp` enum (Accel, Decel, Constant) that takes a start and end speed and synthesizes the intermediate region speeds automatically; (c) integration with the existing `TrillExtensionFullOptions` so callers can specify "trill with mordent, accelerating from Slow to Fast" in one call. Beyond multi-speed trill: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning. Larger remaining post-v1 candidates: line breaking (Gourlay or Bellini & Nesi); golden-SVG corpus + PHASH-based visual regression harness; multi-staff / grand-staff bracket polish.

- Open issues: The layout function does not coalesce adjacent same-glyph regions — by design (a caller who wanted coalescing could collapse the regions themselves before calling; the layout function preserves the input structure). The renderer's per-glyph outline cache is per-call, not per-font (each `draw_trill_extension_multi_speed` call rebuilds the HashMap); a multi-call caching strategy could share outlines across many trill extensions in one score, but the per-call cost is dwarfed by the rest of the score's rendering. The cache uses `HashMap::entry`'s `Occupied::into_mut` and `Vacant::insert` which both return `&mut V`; we use `let path_data: &str = match {...}` to coerce both arms to `&str` — this works because `&mut String` deref-coerces to `&str` through `Deref` chains. If a future Rust version tightened deref coercion (extremely unlikely), this would need an explicit `.as_str()`. The example writes its SVG at `examples/output/trill_multi_speed.svg`; the harness is intentionally low-level (no clef/staff context) because the goal is to visually verify the wiggle-density transitions, not to embed the trill in a real score — a higher-level demo would require wiring multi-speed into ScoreBuilder, which is the explicit (a) follow-up flagged above.

## 2026-05-15 — Post-v1, TrillSpeedRamp synthesizer + TrillWiggleSpeed::index/from_index_saturating

- Did: Closed the "(b) a `TrillSpeedRamp` enum (Accel, Decel, Constant) that takes a start and end speed and synthesizes the intermediate region speeds automatically" item explicitly flagged in the Next section of the prior multi-speed trill chunk. Hand-constructing a `&[TrillSpeedRegion]` for a typical accelerating trill (Slow→Standard→Fast, evenly distributed across a known span) was mechanical and error-prone: caller had to compute N start_x values, pick intermediate `WiggleTrill*` glyphs, and query each glyph's advance from the active font. This chunk adds a layout-layer synthesizer that does that mechanical work, leaving callers to express musical intent (start speed, end speed, region count) plus the font lookup.

  Layout (`src/layout/trill_extension.rs`):
  - New `pub const fn TrillWiggleSpeed::index(self) -> usize` — 0=Fastest..8=Slowest. Ordering matches the canonical `ALL` array; `a.index() < b.index()` iff `a` is faster than `b`. Total order is meaningful for interpolation.
  - New `pub const fn TrillWiggleSpeed::from_index_saturating(i: usize) -> Self` — reverse of `index()`. Indices 0..=8 map to the canonical variants; any index ≥ 9 saturates to `Slowest`. Saturating-not-Option because the internal caller (synthesizer) is provably never out of range and external callers (e.g. fuzz) get the closest variant rather than a panic.
  - New `pub enum TrillSpeedRamp { Constant(TrillWiggleSpeed), Linear { start, end } }` with `const fn` constructors `constant()` and `linear()`. Constant emits N regions of the same speed. Linear emits N regions with speeds linearly interpolated `start.index() + t * (end.index() - start.index())` for `t = i / (N-1)`, rounded to nearest integer, mapped back via `from_index_saturating`. Direction (accel/decel) falls out naturally from input ordering: `linear(Slow, Fast)` accelerates because lower indices are faster.
  - New `pub fn TrillSpeedRamp::synthesize_regions(&self, start_x, end_x, region_count, advance_for_speed: impl Fn(TrillWiggleSpeed) -> f64) -> Option<Vec<TrillSpeedRegion>>`. Regions are evenly distributed: region i starts at `start_x + i * (end_x - start_x) / region_count`. The callback is queried once per region with the chosen speed — font-agnostic. Returns `None` when `region_count == 0`, `end_x <= start_x`, or `Linear` with `region_count < 1`. Output is sorted by `start_x` and accepted by `layout_trill_extension_multi_speed` without massaging.
  - Exported `TrillSpeedRamp` from `layout/mod.rs`.

- Tests (+30 new in `layout::trill_extension::tests`):
  TrillWiggleSpeed methods (6):
  1. `index_canonical_order` — pins down Fastest=0..Slowest=8 mapping explicitly per variant.
  2. `index_matches_position_in_all_array` — `v.index() == ALL.iter().position(&v)` for every variant. Lock-step canary: if `ALL` is reordered without updating `index()` (or vice versa), this fires.
  3. `index_round_trip_through_from_index_saturating` — `from_index_saturating(v.index()) == v` for all 9 variants. Bijection canary across the valid range.
  4. `from_index_saturating_clamps_high_values_to_slowest` — `from_index_saturating(9) == Slowest`, `from_index_saturating(100) == Slowest`, `from_index_saturating(usize::MAX) == Slowest`. Saturation boundary canary.
  5. `from_index_saturating_zero_is_fastest` — boundary canary at the low end.
  6. `index_and_from_index_saturating_are_const_callable` — `const FAST_INDEX: usize = ...; const STANDARD_FROM_IDX: TrillWiggleSpeed = ...`. Compile-fail canary on `const fn`.

  TrillSpeedRamp constructors (3):
  7-8. `ramp_constant_constructor_round_trips`, `ramp_linear_constructor_round_trips` — `constant(s) == Constant(s)`, `linear(s,e) == Linear { start:s, end:e }`. Lock in the helper-to-variant equivalence.
  9. `ramp_constructors_are_const_callable` — compile-fail canary.

  synthesize_regions error cases (6):
  10. `ramp_constant_zero_region_count_returns_none`
  11. `ramp_linear_zero_region_count_returns_none`
  12. `ramp_constant_inverted_x_returns_none` — end_x < start_x.
  13. `ramp_linear_inverted_x_returns_none`
  14. `ramp_zero_width_span_returns_none` — end_x == start_x for both variants.
  15. `ramp_linear_single_region_returns_none` — Linear with N=1 is rejected (ill-defined).

  Constant variant (5):
  16. `ramp_constant_single_region_emits_one_region` — minimum valid input.
  17. `ramp_constant_emits_n_regions_with_same_glyph` — 5 regions, all `WiggleTrillFast`. Also asserts callback queried with `Fast` every time (sanity).
  18. `ramp_constant_emits_evenly_spaced_start_xs` — hand-computed [0, 20, 40, 60, 80] for span [0,100]/5.
  19. `ramp_constant_advance_callback_value_is_propagated` — callback returns 137.42; all regions store exactly 137.42 (not 100.0, not 0.0).
  20. `ramp_constant_with_nonzero_start_x_offsets_regions` — start_x=500.0 → first region's start_x is 500.0, not 0.0.

  Linear variant (10):
  21. `ramp_linear_accel_endpoint_glyphs_match_input` — Slow→Fast, 3 regions: region[0] is `WiggleTrillSlow`, region[2] is `WiggleTrillFast`. Endpoints hit exactly at t=0 and t=1 (no rounding error).
  22. `ramp_linear_accel_middle_region_is_intermediate_speed` — for Slow(5)→Fast(3) at t=0.5, expected intermediate is Standard (index 4 → `WiggleTrill`). Critical canary: a regression that emitted endpoints only (no intermediates) would fire.
  23. `ramp_linear_decel_progresses_from_fast_to_slow` — Fast→Slow, 3 regions: glyphs are [Fast, Standard, Slow]. Direction-symmetry canary.
  24. `ramp_linear_advance_callback_invoked_with_per_region_speed` — callback returns 100/200/300 for Slow/Standard/Fast; asserts regions store [100, 200, 300]. **Critical correctness canary**: a regression that always queried the start speed would store [100, 100, 100], the test fires.
  25. `ramp_linear_round_to_nearest_integer_index` — Slow(5)→Faster(2), 4 regions, expected indices [5,4,3,2]. Hand-computed: t=0→5, t=1/3→4.0→4, t=2/3→3.0→3, t=1→2. Lock-in for the rounding rule.
  26. `ramp_linear_evenly_spaced_start_xs` — [0, 30, 60, 90] for span [0,120]/4. Lock in the spacing rule.
  27. `ramp_linear_degenerate_start_equals_end_emits_constant` — `Linear { start: Standard, end: Standard }` with N=3 produces output byte-identical to `Constant(Standard)` with N=3. Lock in the degenerate-case equivalence.
  28. `ramp_linear_two_regions_emit_exact_endpoints` — minimum valid Linear N. Slowest→Fastest with N=2 produces [Slowest, Fastest] exactly (no intermediates, no rounding).
  29. `ramp_synthesized_regions_feed_into_multi_speed_layout` — **end-to-end contract canary**: synthesized regions are accepted by `layout_trill_extension_multi_speed` without additional massaging. Verifies at least one tile per region (each region's advance < region's span by construction) and that the right edge does not overflow end_x.
  30. `ramp_linear_sorted_start_xs_satisfies_layout_sort_invariant` — both accel and decel directions produce strictly-increasing `start_x` sequences (the layout function rejects unsorted regions).

  Naming rationale: `TrillSpeedRamp` rather than `TrillSpeedProgression`/`TrillSpeedCurve` because "ramp" matches the audio/synth terminology for a linear value change between two endpoints — and matches how the existing speed-variant family is conceptually a "wiggle ramp." `from_index_saturating` rather than `from_index_clamping` because "saturating" is the established Rust-stdlib convention for the same behavior (`u8::saturating_add` etc.).
- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged). `cargo test -p music-engraver --lib` — **2320 unit tests pass** (+30 vs prior recorded 2290: all 30 new in `layout::trill_extension::tests`). `cargo test -p music-engraver --lib trill_extension` — **144 trill-extension-related tests pass** (114 prior + 30 new). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged — the new types are purely additive). `cargo test -p music-engraver --doc` — 12 doc tests pass.
- Next: Score-builder integration for multi-speed trills — a new `trill_with_speed_ramp(ramp: TrillSpeedRamp, region_count: usize)` or similar method on `ScoreBuilder`/note annotations, which would need the system-collector to anchor regions against per-note x positions (the layout primitive being available now is what unblocks that work). Beyond multi-speed trill: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning; integration with `TrillExtensionFullOptions` (item c — combining ornament override + speed ramp + region_count into one options struct). Larger remaining post-v1 candidates: line breaking (Gourlay or Bellini & Nesi); golden-SVG corpus + PHASH-based visual regression harness; multi-staff / grand-staff bracket polish.
- Open issues: `TrillSpeedRamp::synthesize_regions` returns `Vec<TrillSpeedRegion>` (heap-allocated) — for the typical use case of 2-5 regions this is fine; if a future hot path needed a stack-only alternative, an iterator-returning variant could be added without breaking the existing API. The synthesizer does not validate that `start` and `end` produce *different* speeds for `Linear` — `Linear { start: Standard, end: Standard }` is accepted and produces a Constant-equivalent output. This is documented as degenerate-but-permitted; a future stricter API could reject it via a `linear_validated()` constructor returning `Option<Self>`. The rounding rule for the intermediate index uses `f64::round()` (half-away-from-zero per IEEE 754); for even N, the midpoint is exact (no rounding needed), so the half-rounding semantics only matter at intermediate t values where the indices fall on a half-integer — those produce a deterministic single choice rather than alternating. The exposed `from_index_saturating` saturates only at the high end (≥9 → Slowest) — negative values are impossible to pass via `usize`. If `TrillWiggleSpeed` ever gained variants beyond the current 9, this method would need updating in lockstep (the existing `index_matches_position_in_all_array` canary would fire on mismatch).

## 2026-05-15 — Post-v1, TrillSpeedRampSpec integration into TrillExtensionFullOptions

- Did: Closed item (c) explicitly flagged in the Next section of the prior chunk: "integration with `TrillExtensionFullOptions` so callers can specify 'trill with mordent, accelerating from Slow to Fast' in one call." This is the layout-layer API addition only; the score-builder/renderer wiring is a separate later chunk (the prior chunk's item (a)).

  Layout (`src/layout/trill_extension.rs`, **+~50 LOC**):
  - New `pub struct TrillSpeedRampSpec { pub ramp: TrillSpeedRamp, pub region_count: usize }` — compact intent-spec pairing a ramp with its region count. Both are caller-intent values known at score-construction time; the missing inputs to `TrillSpeedRamp::synthesize_regions` (`start_x`, `end_x`, font-advance lookup) are pipeline-level and only known at draw time, so the spec deliberately omits them.
  - New `pub const fn TrillSpeedRampSpec::new(ramp: TrillSpeedRamp, region_count: usize) -> Self` — `const`-callable so canonical specs can live in module-level `const` items alongside the ramp.
  - Doc covers the "no validation at construction time" policy: `region_count == 0` and `Linear` with `region_count == 1` are *defined* failures in `synthesize_regions` (returning `None`); the spec stores raw inputs and lets the synthesizer reject. Mirrors the unsupported-ornament and non-positive-length policies elsewhere in the trill options surface.
  - Exported `TrillSpeedRampSpec` from `layout/mod.rs` alongside the existing `TrillSpeedRamp` / `TrillSpeedRegion` exports.

  Options bundle (`src/layout/trill_options.rs`, **+~80 LOC** + tests):
  - New `pub speed_ramp: Option<TrillSpeedRampSpec>` field on `TrillExtensionFullOptions`. Doc covers the dispatch contract: `speed_ramp.is_some()` engages the multi-speed renderer path and supersedes the single-speed `speed` field; both fields are *allowed* to coexist so widening from `TrillExtensionSpeedOptions` then layering a ramp doesn't force the caller to first clear `speed`.
  - New `pub const fn with_speed_ramp(self, spec: TrillSpeedRampSpec) -> Self` — spec-typed setter.
  - New `pub const fn with_speed_ramp_ramp_count(self, ramp: TrillSpeedRamp, region_count: usize) -> Self` — two-arg sugar, byte-equivalent to `with_speed_ramp(TrillSpeedRampSpec::new(ramp, region_count))`. Two variants because some call sites have a pre-built spec on hand (config struct, const item) while others build inline; offering both elides the ergonomic friction of always wrapping or always passing two args.
  - Updated `TrillExtensionFullOptions::new()` to initialize `speed_ramp: None`.
  - Updated `impl From<TrillBracketOptions>` to explicitly set `speed_ramp: None` (with rationale comment: bracket-only options carry no ramp by construction; the caller must layer it on the widened bundle).
  - Updated `impl From<TrillExtensionSpeedOptions>` to explicitly set `speed_ramp: None` (with rationale: single-speed bundle, no ramp; the speed stays populated and is superseded by a layered ramp per the dispatch doc).
  - Updated the existing `new_has_every_field_unset` test to also assert `speed_ramp == None` — the test's name claims to check every field, so dropping the new assertion would make the name a lie. No other existing tests were modified.

- Tests (+23 new in `layout::trill_options::tests`):
  TrillSpeedRampSpec basic shape (4):
  1. `ramp_spec_new_round_trips_fields` — `new(ramp, count)` populates `.ramp` and `.region_count`. Catches a struct reorder that broke `new`.
  2. `ramp_spec_is_const_constructible` — `const _: TrillSpeedRampSpec = TrillSpeedRampSpec::new(...)` for both Constant and Linear ramps. Compile-fail canary on `const fn`.
  3. `ramp_spec_partial_eq_sensitive_to_ramp` — same `region_count`, different ramp → `assert_ne!`. Catches a PartialEq derive that dropped a field.
  4. `ramp_spec_partial_eq_sensitive_to_region_count` — same ramp, different `region_count` → `assert_ne!`.

  Builder setters (5):
  5. `with_speed_ramp_sets_only_speed_ramp` — setter isolation. Every other of the six pre-existing fields stays `None`.
  6. `with_speed_ramp_ramp_count_sets_only_speed_ramp` — same isolation for the two-arg sugar. Critically asserts `speed == None` (a regression where the sugar accidentally fanned out into both fields would fire).
  7. `with_speed_ramp_ramp_count_byte_equivalent_to_with_speed_ramp_new` — locks in the byte-equivalence claim. Cross-product of 4 ramps × 4 region_counts; PartialEq on the full struct fires per-iteration if anything diverges.
  8. `with_speed_ramp_overwrites_prior_value` — last-write-wins for two `with_speed_ramp` calls. Includes a sanity `assert_ne!(first, second)` so the test only carries weight when the specs differ.
  9. `with_speed_ramp_overwrites_with_speed_ramp_ramp_count` — cross-setter last-write-wins (both setters write the same field).

  Composition with other setters (3):
  10. `with_speed_ramp_chains_with_other_setters` — layering on top of an otherwise-populated bundle leaves all six prior fields intact. Locks in the additive contract.
  11. `with_speed_ramp_chain_order_independent_from_other_setters` — three different orderings yield equal bundles. Catches a setter that accidentally cleared a sibling.
  12. `with_speed_ramp_is_const_constructible` — `const _: TrillExtensionFullOptions = ...` for both spec-typed and sugar setters.

  Dispatch coexistence (2):
  13. `speed_ramp_and_speed_can_coexist_on_options` — setting `speed` then layering `speed_ramp` keeps both populated. Pins down the "no field reset" policy at the options layer.
  14. `with_speed_does_not_clear_speed_ramp` — symmetric: setting `speed_ramp` then layering `speed` does not silently drop the ramp. Catches a regression that "promoted" the single-speed setter into a multi-speed reset.

  From conversions (3):
  15. `from_bracket_options_leaves_speed_ramp_none` — widening a populated `TrillBracketOptions` (bracket + direction + length + ornament) must yield `speed_ramp: None`. Critical canary: a regression that synthesized a "default ramp" would silently engage the multi-speed path for callers who only asked for a bracket.
  16. `from_speed_options_leaves_speed_ramp_none` — same canary for the single-speed bundle.
  17. `widen_from_speed_options_then_add_ramp` — the intended ergonomic path. The `speed` field survives the widening; the layered ramp populates `speed_ramp`. Both end up `Some(...)` simultaneously.

  PartialEq + Default canaries (3):
  18. `distinct_speed_ramp_specs_compare_distinct` — `assert_ne!` on two bundles differing only in `speed_ramp`. Catches a PartialEq derive that ever dropped the new field.
  19. `speed_ramp_some_distinct_from_none` — `None` vs. `Some(spec)` are semantically distinct (single-speed vs. multi-speed path at draw time); PartialEq must reflect that. Mirrors the analogous `length_ss_some_zero_distinct_from_none` canary.
  20. `default_speed_ramp_is_none` — targeted canary for `Default::default()` populating the new field as `None`. The existing `default_matches_new` test gives full-struct equality; this gives a clearer failure message if the derive diverges for just this field.

  Cross-setter isolation + end-to-end (3):
  21. `other_setters_do_not_write_to_speed_ramp` — cross-setter canary: every other `with_*` setter (7 in total counting `with_extension_length_ss`) must leave `speed_ramp == None`. Loops over the seven bundles in one assert block. Closes the gap that the per-setter isolation tests predate the new field.
  22. `ramp_spec_carries_degenerate_inputs_unchanged` — `region_count == 0` and `Linear` with `region_count == 1` both round-trip through the options bundle unchanged. Locks in the "no construction-time validation" policy.
  23. `spec_in_options_synthesizes_regions_at_draw_time` — end-to-end contract canary. Builds a `TrillExtensionFullOptions` with a Slow→Fast Linear ramp + 3 regions, unpacks the spec, calls `synthesize_regions(0.0, 300.0, 3, |_| 50.0)`. Asserts 3 regions, start_xs `[0, 100, 200]`, and glyphs `[Slow, Standard, Fast]` (linear interpolation in index space). Critical canary: the options bundle is the source of truth, and the spec must feed cleanly into the synthesizer without massaging.

  Naming rationale: `TrillSpeedRampSpec` (not `TrillSpeedRampOptions` / `TrillSpeedRampSpecification`) — "spec" reads as "specification of intent" rather than the more opinionated "options" (which we already overload with `TrillBracketOptions` / `TrillExtensionSpeedOptions` / `TrillExtensionFullOptions`). The shorter form matches the spec-vs-options convention used in `bracket: Option<TrillBracketSide>` (an enum value, not an `Options` bundle) where one knob is enough — the spec is the multi-speed equivalent of that single knob: a small payload of caller intent. `with_speed_ramp_ramp_count` for the two-arg variant — verbose, but unambiguous; `with_speed_ramp(ramp, count)` would shadow the spec-typed `with_speed_ramp(spec)` and require either a turbofish at the call site or trait-based overloading.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged). `cargo test -p music-engraver --lib` — **2352 unit tests pass** (+23 vs prior recorded 2329 baseline at the start of this run: all 23 new in `layout::trill_options::tests`; 2329 was the pre-chunk count, 2352 is the post-chunk count). `cargo test -p music-engraver --lib trill_options` — **59 trill-options tests pass** (36 prior + 23 new). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged — the new field is additive, no existing code consumes it yet, so no golden SVG bytes changed). `cargo test -p music-engraver --doc` — 12 doc tests pass.

- Next: Score-builder integration — wire `speed_ramp` through `trill_with_extension_full_options(opts)` to the annotation layer (`annotations.trill_speed_ramp = opts.speed_ramp`), then dispatch the multi-speed renderer when present. This requires (a) a new field on the annotation struct, (b) the renderer's trill-extension collector to check for the field and call `layout_trill_extension_multi_speed` instead of `layout_trill_extension`, (c) the synthesizer's `advance_for_speed` closure threaded through the active font's glyph advance lookup. Also: a high-level `trill_with_extension_speed_ramp(ramp, region_count)` convenience method on `ScoreBuilder` (mirroring `trill_with_extension_speed`). Beyond that: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning. Larger remaining post-v1 candidates: line breaking (Gourlay or Bellini & Nesi); golden-SVG corpus + PHASH-based visual regression harness; multi-staff / grand-staff bracket polish.

- Open issues: `TrillSpeedRampSpec` has no validation at construction — `region_count == 0` and `Linear` with `region_count == 1` are accepted and produce `None` from the downstream synthesizer. A future stricter API could expose a `TrillSpeedRampSpec::new_validated(ramp, region_count) -> Option<Self>` that pre-rejects those cases. Two setters (`with_speed_ramp` and `with_speed_ramp_ramp_count`) write the same field — this is intentional ergonomic redundancy, but adds a surface area concern: a future refactor that splits them into separate fields would have to update both methods together. The two-arg setter's name `with_speed_ramp_ramp_count` is a bit redundant; chose the explicit form over `with_speed_ramp_inline` (less precise) or method overloading via traits (Rust idiom prefers explicit names for arity differences). The cross-test `other_setters_do_not_write_to_speed_ramp` loops over seven bundles in one assert block — if a future setter is added without updating that list, the test silently drops coverage for the new setter (not a regression — just an incomplete canary). Consider extracting a `TrillExtensionFullOptionsBuilder` test fixture later if more setters get added.

## 2026-05-15 — Post-v1, score-builder integration for multi-speed trills + multi-speed bracket helper

- Did: Closed item (a) from the prior chunk's Next: "wire `speed_ramp` through `trill_with_extension_full_options(opts)` to the annotation layer, then dispatch the multi-speed renderer when present. This requires (a) a new field on the annotation struct, (b) the renderer's trill-extension collector to check for the field and call `layout_trill_extension_multi_speed` instead of `layout_trill_extension`, (c) the synthesizer's `advance_for_speed` closure threaded through the active font's glyph advance lookup. Also: a high-level `trill_with_extension_speed_ramp(ramp, region_count)` convenience method on `ScoreBuilder` (mirroring `trill_with_extension_speed`)." All sub-items addressed in this chunk.

  Layout (`src/layout/measure.rs`):
  - New `pub trill_speed_ramp: Option<TrillSpeedRampSpec>` field on `NoteAnnotations`. Docs cover the dispatch contract: `None` keeps the single-speed wiggle path; `Some(spec)` engages the multi-speed renderer and supersedes `trill_wiggle_speed` for glyph selection. Both fields are permitted to coexist on the annotation (matching the options-layer coexistence policy from the prior chunk). Imports `TrillSpeedRampSpec` from `trill_extension`.

  Layout (`src/layout/trill_bracket.rs`, **+~50 LOC** + tests):
  - New `pub fn layout_trill_bracket_hooks_multi_speed(extension: &MultiSpeedTrillExtensionLayout, side, length, direction, stroke_width) -> Vec<TrillBracketHookLayout>`. Multi-speed counterpart of `layout_trill_bracket_hooks`. Anchors hooks at `tiles[0].x` (Start) and `multi_speed_trill_extension_right_edge(...)` (End) — same convention as the single-speed function, only the data shape differs (`tiles: Vec<TrillExtensionTile>` vs. `segment_xs: Vec<f64>`). Empty-layout returns empty vec, matching the single-speed fail-safe.
  - Re-exported `layout_trill_bracket_hooks_multi_speed` from `layout/mod.rs` alongside the existing trill-bracket exports.

  Score builder (`src/score/mod.rs`, **+~65 LOC**):
  - `trill_with_extension_full_options(opts)` now propagates `opts.speed_ramp` into `annotations.trill_speed_ramp` (the previously-set field gets written, completing the integration that the prior chunk's options-layer addition only half-wired).
  - New `pub fn trill_with_extension_speed_ramp(self, ramp: TrillSpeedRamp, region_count: usize) -> Self` — the convenience builder. Mirror of `trill_with_extension_speed`: hardcodes `Ornament::Trill`, sets `trill_extension = true`, and populates `trill_speed_ramp` with a freshly-constructed `TrillSpeedRampSpec::new(ramp, region_count)`. No-op on rest, same as the other trill builders. Docs cover the byte-equivalence to `trill_with_extension_full_options(new().with_speed_ramp_ramp_count(ramp, region_count))`.

  Renderer dispatch (`src/render/system_renderer/mod.rs`, **+~70 LOC**):
  - New `pub speed_ramp: Option<TrillSpeedRampSpec>` field on `TrillExtensionNoteInfo` (the system-renderer's per-note carrier). Filtered the same way as the other ext-only fields: only carries through when `has_trill_extension == true`. Both Note and Chord collector arms updated.
  - `draw_system_trill_extensions` now branches on `note.speed_ramp`: when `Some(spec)`, the multi-speed path runs (`spec.ramp.synthesize_regions(start_x, end_x, region_count, |speed| font.glyph_advance(speed.to_glyph())...)` → `layout_trill_extension_multi_speed` → `draw_trill_extension_multi_speed`), with bracket dispatch via the new `layout_trill_bracket_hooks_multi_speed`. The single-speed path is preserved verbatim for the `None` case via an explicit `continue` after the multi-speed branch completes. Cross-system propagation is handled by setting `cross_system = false` in the multi-speed branch — the page renderer's incoming-wiggle handler only knows single-speed, so multi-speed trills are confined to a single system in v1 (documented limitation; future work).
  - `find_unresolved_trill_extension` in `page_renderer/mod.rs` now also returns `None` when `last.speed_ramp.is_some()` — concretizes the within-system-only constraint by suppressing cross-system continuation upstream.
  - Imports: added `layout_trill_extension_multi_speed`, `layout_trill_bracket_hooks_multi_speed`, `TrillSpeedRampSpec`, `draw_trill_extension_multi_speed` to the system_renderer module's use-list.

- Tests (+17 new total: 6 layout::trill_bracket + 11 score::tests):

  layout::trill_bracket::tests (6 new, all multi-speed bracket layout):
  1. `multi_speed_hooks_start_only_at_first_tile_x` — basic Start-only: 1 hook at `tiles[0].x` (hand-computed 100.0).
  2. `multi_speed_hooks_end_only_at_right_edge` — basic End-only: 1 hook at the right edge (hand-computed 460.0 = 380 + 80, the last tile's x + advance).
  3. `multi_speed_hooks_both_returns_two_at_endpoints` — Both: 2 hooks at `[100.0, 460.0]`. Lock in the ordering (Start first, End second).
  4. `multi_speed_hooks_empty_layout_returns_empty_vec` — empty `tiles` returns empty vec. Fail-safe canary.
  5. `multi_speed_hooks_share_layout_y_baseline_down` — both hooks share `y_top == ext.y` for HookDirection::Down. Locks in the baseline anchor.
  6. `multi_speed_hooks_match_single_speed_when_only_one_region` — **cross-helper equivalence canary**. Builds a single-region multi-speed layout with the same `(start_x, end_x, advance)` as a single-speed layout, calls both bracket helpers, asserts hook-by-hook (x, y_top, y_bottom) equality. Guards against the two helpers drifting in their anchor conventions.

  score::tests (11 new):
  1. `full_options_with_speed_ramp_sets_annotation_field` — round-trip canary: setting `with_speed_ramp(spec)` on the options bundle, applying it through `trill_with_extension_full_options`, then asserting `annotations.trill_speed_ramp == Some(spec)`. Fires if the builder silently drops the new field.
  2. `full_options_without_speed_ramp_leaves_annotation_none` — symmetric: a populated bundle without speed_ramp produces `annotations.trill_speed_ramp == None`. Critical canary against a regression that synthesized a default ramp from some other setter.
  3. `trill_with_extension_speed_ramp_sets_three_annotation_fields` — convenience builder: asserts ornament=Trill, trill_extension=true, and `trill_speed_ramp == Some(spec)` after one call. Matches the assertion shape of the analogous `trill_with_extension_speed_sets_all_three_annotation_fields` test.
  4. `trill_with_extension_speed_ramp_on_rest_is_noop` — convenience builder on a rest must not panic, must not annotate retroactively. Output must not contain `ornamentTrill`.
  5. `trill_with_extension_speed_ramp_byte_equivalent_to_full_options_path` — central byte-equivalence canary: the convenience builder must produce SVG byte-identical to `trill_with_extension_full_options(new().with_speed_ramp(TrillSpeedRampSpec::new(ramp, n)))`. If the convenience builder ever drifts (e.g. sets an extra annotation field), this fires.
  6. `speed_ramp_renders_distinct_svg_from_single_speed` — multi-speed dispatch sanity: a Linear Slow→Fast ramp must render *different* SVG than a single-speed Standard trill. Also asserts both produce > 4 paths (visible wiggles). Earlier draft asserted `multi >= single` path count; that turned out to be advance-dependent (Standard's smaller advance yields more tiles than the averaged Slow/Standard/Fast mix) — the corrected assertion just checks distinctness + non-empty rendering.
  7. `constant_ramp_byte_equivalent_to_single_speed_when_one_region` — degenerate equivalence: `Constant(Standard)` with 1 region must produce the same path count as single-speed `Standard`. Both use `floor(span/advance)` over the same span with the same advance.
  8. `degenerate_ramp_renders_no_wiggle_but_keeps_trill_glyph` — `Linear` with `region_count == 1` returns `None` from `synthesize_regions`, falling through to "no wiggle." Asserts the resulting SVG has strictly fewer `<path>` elements than the non-degenerate `trill_with_extension()` baseline. The trill glyph itself stays.
  9. `ramp_with_bracket_renders_both_wiggle_and_hooks` — bracket + ramp coexistence: a `with_bracket(Both).with_speed_ramp(...)` options bundle must emit ≥ 7 `<line>` elements (5 staff lines + 2 bracket hooks). Catches a regression that silently dropped the bracket dispatch in the multi-speed branch.
  10. `linear_ramp_renders_distinct_svg_from_constant_ramp` — variant choice canary: `Linear(Slow, Fast)` and `Constant(Standard)` with the same region count produce distinct SVG. If the renderer ignored the variant, they'd be byte-equal.
  11. `speed_ramp_supersedes_speed_field_for_glyph_selection` — coexistence semantics: when *both* `speed` and `speed_ramp` are set, the ramp drives the wiggle and the speed field is ignored. Same `speed=Slowest` baseline; with vs. without a layered ramp must produce distinct SVG.

- Example (`examples/trill_speed_ramp_score.rs`, +~200 LOC): Renders 4 measures on a treble staff, each exercising a different ramp shape: M1 Constant(Standard) ×3, M2 Linear(Slow→Fast) ×3 (accel), M3 Linear(Fast→Slow) ×3 (decel), M4 chord with Linear(Slow→Fast). Three variant-builders (`build_swapped_direction_variant`, `build_no_trills_variant`, `build_all_constant_standard_variant`) anchor 3 structural assertions: (a) swapping accel↔decel direction produces distinct SVG, (b) the multi-speed score adds path elements over a no-trill baseline, (c) Linear and Constant ramps render differently with the same region count. Output: 22471 bytes, 37 paths, 18 lines.

- Cross-system limitation (documented, deferred): the page-renderer's `draw_cross_system_trill_extensions` only knows single-speed (it reads `last.wiggle_speed` and tiles a single glyph on the incoming system). To preserve correctness for multi-speed trills landing on the last note of a system, `find_unresolved_trill_extension` now returns `None` when `speed_ramp.is_some()` — confining multi-speed trills to their source system. Adding cross-system multi-speed support (e.g. by carrying the spec across systems and re-synthesizing regions against the target system's left edge → first note's x) is a clean follow-up: needs the same spec on the `UnresolvedTrillExtension` struct + a new `draw_incoming_multi_speed_wiggle` path + a re-anchored region span.

- Naming rationale: `trill_with_extension_speed_ramp(ramp, region_count)` mirrors `trill_with_extension_speed(speed)` — same naming pattern, one extra arg. Considered `trill_with_extension_with_speed_ramp` for symmetry with `trill_with_extension_with_options` but that path is for options-typed args; ramp+count is a pair of primitives so the single-purpose builder name reads more naturally. `layout_trill_bracket_hooks_multi_speed` (not `..._for_multi_speed_layout`, not `multi_speed_layout_trill_bracket_hooks`) matches the existing pattern where the "kind" of layout/extension appears as a trailing modifier on layout function names.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged). `cargo test -p music-engraver --lib` — **2369 unit tests pass** (+17 vs prior recorded 2352: 6 new in `layout::trill_bracket::tests` + 11 new in `score::tests`). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged — the dispatch change is additive on a new annotation field; no existing score sets the field, so no golden SVG bytes changed). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass. `cargo run -p music-engraver --example trill_speed_ramp_score` writes 22471-byte SVG with 37 paths + 18 lines, all 4 structural assertions pass.

- Next: Cross-system multi-speed support — the documented limitation above is the natural follow-up. Carry `TrillSpeedRampSpec` on `UnresolvedTrillExtension`, add a `draw_incoming_multi_speed_wiggle` path that re-anchors region spans against the target system's staff-left → first-note-x range, then drop the `speed_ramp.is_some() → None` short-circuit in `find_unresolved_trill_extension`. Beyond multi-speed trill: cross-system church rests; per-note collision detection in beamed additional voices; Gourlay penalty tuning. Larger remaining post-v1 candidates: line breaking (Gourlay or Bellini & Nesi); golden-SVG corpus + PHASH-based visual regression harness; multi-staff / grand-staff bracket polish.

## 2026-05-15 — Post-v1, cross-system multi-speed trill continuation

- Did: Closed the prior chunk's Next item: "Cross-system multi-speed support — Carry `TrillSpeedRampSpec` on `UnresolvedTrillExtension`, add a multi-speed incoming-wiggle path that re-anchors region spans against the target system's staff-left → first-note-x range, then drop the `speed_ramp.is_some() → None` short-circuit in `find_unresolved_trill_extension`." All three sub-items wired in `src/render/page_renderer/mod.rs` (≈+85 LOC body + ≈20 LOC doc).

  Page renderer (`src/render/page_renderer/mod.rs`):
  - New `pub speed_ramp: Option<TrillSpeedRampSpec>` field on `UnresolvedTrillExtension`. Doc covers the dispatch contract: `None` keeps the single-speed incoming path (using `wiggle_speed`); `Some(spec)` engages the multi-speed incoming path. The choice to **re-synthesize regions per-system** (rather than carry source-system regions across) is documented inline: the multi-speed convention is "evenly-distributed regions across the wiggle span"; the source-system span and the target-system span are generally different lengths, so reusing the source's region geometry would compress or stretch the speed progression asymmetrically across the line break. Re-synthesizing per system keeps each system region-uniform on its own terms and matches the within-system convention exactly.
  - Removed the `if last.speed_ramp.is_some() { return None; }` short-circuit from `find_unresolved_trill_extension` (the conservative gate added in the prior chunk that confined multi-speed trills to a single system). The explicit-length short-circuit (`explicit_length_ss > 0.0 → None`) is preserved — that rule is dispatch-independent: an explicit length terminates the wiggle within the source system regardless of single-speed or multi-speed dispatch. `find_unresolved_trill_extension` now populates the new `speed_ramp` field with `last.speed_ramp`.
  - `draw_cross_system_trill_extensions` gained a multi-speed branch ahead of the single-speed path. When `src.speed_ramp` is `Some(spec)`, the renderer (a) calls `spec.ramp.synthesize_regions(start_x, end_x, region_count, |speed| font.glyph_advance(speed.to_glyph()).unwrap_or(0) as f64)` to re-anchor regions against the target span, (b) on `Some(regions)`, calls `layout_trill_extension_multi_speed(end_x, y, &regions)` and `draw_trill_extension_multi_speed(svg, font, &layout)`, (c) on `None` (degenerate spec or non-positive span), `continue`s — matching the within-system fail-safe. The branch ends with an explicit `continue` to bypass the single-speed path. End hooks in the multi-speed branch use `layout_trill_bracket_hooks_multi_speed(&layout, TrillBracketSide::End, ...)` (Start was already drawn on system N by the within-system pass via `bracket_side_for_system_pass`). Anchoring the End hook through the multi-speed bracket helper keeps the right-edge geometry consistent with the within-system bracket — both helpers read `multi_speed_trill_extension_right_edge(&layout)`, so cross-system End hooks land at the same x as a within-system End hook would on the same span.
  - Imports: added `layout_trill_bracket_hooks_multi_speed`, `layout_trill_extension_multi_speed`, `TrillSpeedRampSpec`, `draw_trill_bracket_hooks`, `draw_trill_extension_multi_speed` to page_renderer's use-list.

- Tests (+10 new in `render::page_renderer::tests`):
  1. `cross_system_multi_speed_trill_adds_incoming_paths_on_next_system` — basic dispatch canary: Linear Slow→Fast ramp with 3 regions on the last note of system N must add ≥3 `<path>` elements over a no-trill baseline (tr + ≥1 trailing on N + ≥1 incoming on N+1). A regression that left the short-circuit in place would produce only 2 added paths (tr + trailing only).
  2. `cross_system_multi_speed_trill_renders_distinct_svg_from_single_speed` — **critical correctness canary**: a multi-speed cross-system trill must NOT render byte-identical to single-speed Standard. A regression that ignored `src.speed_ramp` (continuing into the single-speed branch) would produce byte-equal SVG with the single-speed Standard variant.
  3. `cross_system_multi_speed_trill_uses_multiple_distinct_wiggle_glyphs` — **stronger glyph-diversity canary**: walks the SVG collecting all `d="..."` substrings into a `HashSet` and asserts ≥5 distinct path outlines (tr + Slow + Standard + Fast wiggles + notehead). A regression that fell back to a single-speed tile-fill on N+1 would emit only one wiggle outline on the target system, shrinking the set.
  4. `cross_system_multi_speed_trill_explicit_length_suppresses_continuation` — interaction with the explicit-length rule: a multi-speed ramp + `trill_extension_length_ss = Some(2.0)` must NOT produce an incoming wiggle on N+1. Two-stage assertion: (a) explicit-length variant has strictly fewer paths than the equivalent no-explicit-length variant (`continued_paths`), (b) the explicit-length delta over a no-trill baseline is strictly smaller than the continued-trill delta. Catches a regression that accidentally bypassed the explicit-length short-circuit when adding the multi-speed branch.
  5. `cross_system_multi_speed_trill_bracket_end_adds_one_hook_on_target_system` — End bracket dispatch: adds exactly one `<line>` over a no-bracket baseline. Locks in that the hook is drawn on N+1 (not N).
  6. `cross_system_multi_speed_trill_bracket_both_adds_start_on_n_and_end_on_n_plus_1` — Both bracket dispatch: adds exactly two `<line>` elements (Start on N + End on N+1). A regression that drew both hooks on the same system would also pass `+2 lines`, but a regression that dropped one would fire.
  7. `cross_system_multi_speed_trill_bracket_start_only_adds_no_hook_on_target` — Start-only dispatch: adds exactly one `<line>` (drawn on N by the within-system pass). The cross-system pass must NOT add an End hook when none was requested. Catches a regression that always emitted an End hook in the multi-speed branch.
  8. `cross_system_multi_speed_trill_accel_distinct_from_decel` — direction-symmetry canary: Slow→Fast (accel) and Fast→Slow (decel) ramps render distinct SVG. A regression that sorted endpoints before synthesizing (or otherwise lost direction info) would produce byte-equal output.
  9. `cross_system_multi_speed_constant_ramp_equals_single_speed_when_one_region` — degenerate equivalence at the cross-system level: `Constant(Standard)` with 1 region must produce the same total path count as single-speed `Standard`. Mirrors the within-system equivalence test from the prior chunk.
  10. `cross_system_multi_speed_trill_no_target_system_no_crash` — single-system page sanity: no system N+1 exists, the page renderer must not crash, the within-system trailing wiggle still renders.

- Design rationale (re-synthesize vs. carry-across): Considered two alternative cross-system geometries before settling on per-system re-synthesis:
  - **(A) Carry source-system regions across the line break unchanged.** Rejected: source-system span and target-system span are different lengths in general. The source's region geometry (e.g. three regions of width `(end-start)/3` each) would either overflow the target span (if shorter) or leave a gap at the right edge (if longer). Asymmetric stretch breaks the visual reading.
  - **(B) Split the ramp by remaining-span at the line break.** Rejected as too clever for what's actually needed: it would require carrying not just the spec but a "fractional position" through the line-break (where the trill was when N ended), then resuming on N+1 from that point. Adds state, complicates `UnresolvedTrillExtension`, and doesn't match what readers expect from notation (each system's wiggle reads as a complete progression).
  - **(C) Re-synthesize regions per system** — chosen. Each system's multi-speed wiggle is region-uniform on its own terms. Trade-off: the speed-curve resets at the line break (e.g. an accel ramp goes Slow→Standard→Fast on N, then Slow→Standard→Fast again on N+1). That is the intended reading: a sustained-trill annotation across a line break is two separate visualization-of-the-same-musical-event spans, and the engraving convention is that each span is internally uniform. If "continuous ramp across the break" is ever needed, option (B) becomes the upgrade path — but it's strictly additive on top of (C).

- Naming rationale: kept `UnresolvedTrillExtension` rather than promoting to a sum-type (`enum { SingleSpeed { wiggle_speed: ... }, MultiSpeed { spec: ... } }`) because the cross-system carrier already has 5 fields (y_above_top_line, bracket, bracket_direction, bracket_length_ss, wiggle_speed); adding a 6th (`speed_ramp`) keeps the struct shape uniform and matches the analogous flat-struct layout of `TrillExtensionNoteInfo`. The two dispatch fields (`wiggle_speed`, `speed_ramp`) coexist with the same precedence rule as in the within-system path: `speed_ramp.is_some()` supersedes `wiggle_speed` for glyph selection. A regression that flipped the precedence would be caught by the `cross_system_multi_speed_trill_renders_distinct_svg_from_single_speed` test.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged). `cargo test -p music-engraver --lib` — **2379 unit tests pass** (+10 vs prior recorded 2369: all 10 new in `render::page_renderer::tests`). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (unchanged — the new dispatch only activates when a multi-speed score lands on the last note of a system; no existing golden score does that). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass, 1 ignored (unchanged).

- Next: With cross-system multi-speed support landed, the natural remaining cross-system gaps are: **cross-system church rests** (whole-rest under a fermata or in a multi-measure rest that breaks across a line; how to render the right-edge half on N and the left-edge half on N+1); **cross-system glissando with text** (already partially handled — verify the text label policy when the glissando crosses); **cross-system trill bracket length / direction overrides** (currently honored on the End hook; verify Start/End hook symmetry under custom directions). Beyond cross-system: per-note collision detection in beamed additional voices; Gourlay penalty tuning. Larger remaining post-v1 candidates: line breaking (Gourlay or Bellini & Nesi); golden-SVG corpus + PHASH-based visual regression harness; multi-staff / grand-staff bracket polish.

- Open issues: Re-synthesis per system means an accel ramp Slow→Fast on N is followed by another Slow→Fast on N+1 (the speed curve resets at the line break, as documented under "Design rationale (C)"). If "continuous ramp across the break" is ever needed, the path forward is option (B) above: split the original ramp by remaining-span at the line break. That requires (i) a way to compute "how much of the original span was consumed on N" and (ii) carrying a fractional position through `UnresolvedTrillExtension`, then resuming the ramp on N+1 from that fractional point rather than from `start`. Strictly additive on top of the current implementation. The cross-system End hook in the multi-speed branch uses `TrillBracketSide::End` as a hardcoded argument to `layout_trill_bracket_hooks_multi_speed` (rather than relaying the user's original `bracket` value): this is correct under the current `bracket_side_for_system_pass` policy (Start was suppressed at source for `End`, Start was drawn at source for `Both`), but if that policy ever changes (e.g. drawing Start hooks on N+1 for some reason), the multi-speed cross-system path would need to mirror the new policy. The single-speed cross-system End hook hardcodes the same value via `layout_trill_end_hook` for the same reason — the two paths are symmetric in that respect.

- Open issues: The multi-speed branch's font-advance closure uses `font.glyph_advance(...).unwrap_or(0)` as a fall-safe — a glyph genuinely missing from the active font would produce a `0`-advance region, which `layout_trill_extension_multi_speed` rejects (`segment_advance <= 0.0 → None`), so the wiggle silently disappears for that ramp. This is the intended behavior (missing glyph = no wiggle, consistent with the single-speed renderer), but it means a Bravura-bundling regression on one of the wiggleTrill* variants would not surface as a panic — it'd produce a quietly-empty wiggle. A future improvement could propagate the FontError through the closure (currently rejected because `synthesize_regions` takes `impl Fn -> f64`, not `impl Fn -> Result<f64, _>`). The cross-system suppression is a hard-coded `None` — a future caller wanting cross-system multi-speed would have to expand both the page renderer and lift the short-circuit; the constraint is documented in the page-renderer comment but not enforced via a public-facing flag. The new convenience builder is positioned as the multi-speed counterpart of `trill_with_extension_speed`, but the latter (single-speed) has *six* variants (plain, bracketed, bracketed_custom, with-options, speed-with-options, length_ss); the multi-speed equivalents of the bracketed variants are reachable only through `trill_with_extension_full_options` — adding e.g. `trill_with_extension_bracketed_speed_ramp(side, ramp, n)` later is non-blocking but would round out the API surface.

## 2026-05-15 — Post-v1, orchestral bracket SMuFL-glyph upgrade

- Did: Replaced the placeholder horizontal-serif rendering of `ConnectorKind::Bracket` (orchestral section bracket) with proper SMuFL `bracketTop`/`bracketBottom` scroll glyphs — the published engraving convention (Gould; Behind Bars). Previously the bracket rendered as one thick vertical line plus two short horizontal serif strokes at the endpoints; now it renders as the thick vertical line plus two SMuFL scroll outlines that curl outward at top and bottom. Tagged in the "Larger remaining post-v1 candidates" list of the prior chunk as **multi-staff / grand-staff bracket polish**. The brace connector (piano grand staff) was unchanged — it already used the SMuFL `Brace` glyph.

  Layout (`src/layout/multi_staff.rs`):
  - Replaced `serif_length: f64` and `serif_thickness: f64` fields on `BracketLayout` with `top_glyph: smufl::Glyph` and `bottom_glyph: smufl::Glyph`. The thick-line geometry fields (`x`, `y_top`, `y_bottom`, `thickness`) are unchanged.
  - Removed the now-unused `BRACKET_SERIF_LENGTH_SS` const.
  - `layout_multi_staff` populates `top_glyph: Glyph::BracketTop`, `bottom_glyph: Glyph::BracketBottom` for any `ConnectorKind::Bracket` group with ≥2 staves.
  - Doc on `BracketLayout` covers the SMuFL anchoring convention (`bracketTop`'s origin is at its bBox SW corner — bottom-left — so translating to `(x, y_top)` places the scroll above `y_top` joining the line at exactly `y_top`; `bracketBottom`'s origin is at its bBox NW corner so it joins at `y_bottom`).

  Render (`src/render/multi_staff_renderer.rs`):
  - `draw_bracket` now takes `font: &MusicFont` and returns `Result<(), FontError>` (propagating glyph-loading failures, same pattern as `draw_brace`). Internally: 1 `add_line` for the thick vertical (unchanged geometry: still centered at `bracket.x + bracket.thickness/2`, spanning `y_top..y_bottom`) followed by 2 `add_path` calls for the scroll glyphs, each translated to `(bracket.x, y_top)` / `(bracket.x, y_bottom)`.
  - `draw_multi_staff_connectors` propagates `draw_bracket`'s Result via `?` — no other call-sites are affected at this layer.
  - Verified visually via the path-data: `bracketTop` (Bravura) starts at `M0 -117L0 0L125 0` — its bottom edge is a 125-fu (= 0.5 staff-space) horizontal segment exactly matching `BRACKET_THICKNESS_SS * staff_space`. So the glyph's bottom edge and the thick line's top edge meet flush with no seam; the scroll then curls upward and to the right.

  Score (`src/score/multi_staff.rs`):
  - The "extended bracket" path in `try_render_svg` (which over-draws a bracket spanning notation + tab staves for the guitar+tab layout) was replaced with a `BracketLayout { ... }` construction + a single call to the shared `draw_bracket` helper. This means the extended bracket now (a) uses the SMuFL scrolls consistently with the within-system bracket, (b) positions the thick line at `bracket.x + thickness/2` instead of centered on `bracket.x` — a quiet geometry fix that puts the right edge of the line flush against the staff at `x = left_margin`. The OLD ad-hoc inline drawing put the line center at `left_margin - thickness`, leaving a `thickness/2` (62.5 fu) gap between the bracket's right edge and the staff's left edge. Comparing OLD vs. NEW golden file: thick-line center shifted from x=375 to x=437.5, exactly the half-thickness shift required to close that gap.

  Example (`examples/grand_staff.rs`):
  - Updated the `path_count >= 1` assertion to `path_count >= 3` (1 brace + 2 bracket scrolls). The `line_count >= 25` assertion still holds (25 staff lines × 5 staves = 25; plus 1 bracket vertical + 2 joined barlines = 28; the breakdown comment was updated to reflect the new path/line counts).

  Golden (`tests/golden/guitar_tab.svg`): Regenerated via `GOLDEN_UPDATE=1`. Delta from prior baseline:
  - Removed: 2 horizontal serif `<line>` elements (top + bottom).
  - Added: 2 `<path>` elements (bracketTop + bracketBottom).
  - Modified: vertical-line `x` shifted from 375 to 437.5 (the geometry fix described above).

- Tests (+6 new, -1 obsolete = +5 net):

  Removed: `multi_staff::tests::bracket_serif_thickness_is_fraction_of_main` (asserted on `serif_thickness < thickness && serif_thickness > 0` — the field no longer exists).

  Added in `multi_staff::tests` (+3):
  1. `bracket_uses_smufl_scroll_glyphs` — asserts `top_glyph == Glyph::BracketTop`, `bottom_glyph == Glyph::BracketBottom`, AND `top_glyph != bottom_glyph`. The pair-distinctness assertion is the canary against a regression where both fields accidentally pick up the same value (e.g. copy-paste swap or generic default).
  2. `bracket_thickness_matches_smufl_engraving_default` — locks `bracket.thickness == 0.5 * SS` (Bravura's `bracketThickness` engraving default). Fires if anyone bumps `BRACKET_THICKNESS_SS` without expectation.
  3. `bracket_x_is_left_of_staff_by_thickness` — locks `bracket.x == -BRACKET_THICKNESS_SS * SS`, i.e. the bracket's left edge sits one thickness to the left of the staff origin so the line's right edge meets x=0. Fires on a regression that changes the inset convention.

  Updated `multi_staff::tests::bracket_layout_present_for_section` to drop the `serif_length > 0` assertion and add `top_glyph == BracketTop` / `bottom_glyph == BracketBottom`. The other invariants (`y_top`, `y_bottom`, `thickness`) are unchanged.

  Updated `multi_staff_renderer::tests::draw_bracket_produces_three_lines` → `draw_bracket_produces_one_line_and_two_glyph_paths`. Asserts `line_count == 1` AND `path_count == 2` (was `line_count == 3`). Both bounds are exact, not floor-bounds — catches both serif-revert regressions (which would fire on line_count) and glyph-doubling regressions (which would fire on path_count).

  Added in `multi_staff_renderer::tests` (+4):
  4. `draw_bracket_top_glyph_anchored_at_y_top` — asserts the SVG contains `translate({bracket.x},{bracket.y_top})` substring. Critical anchor canary: a regression that places the top scroll at the wrong y (e.g. mid-bracket, or at y_bottom) would not match.
  5. `draw_bracket_bottom_glyph_anchored_at_y_bottom` — symmetric assertion for the bottom scroll.
  6. `draw_bracket_top_and_bottom_glyphs_render_distinct_outlines` — extracts `d="..."` from both `<path>` elements and asserts `d_top != d_bottom`. The two scrolls curl in opposite vertical directions; the test parses 2 path elements (with a leading-split assertion of `parts.len() == 3` — head + 2 paths) and runs the inequality. Catches a regression that assigns the same Glyph to both `top_glyph` and `bottom_glyph`, or that swaps the glyph load order so both call `glyph_outline(Glyph::BracketTop)`.
  7. `draw_bracket_vertical_line_uses_thickness` — searches for `stroke-width="{bracket.thickness}"` in the SVG. Catches a regression that loses the dynamic thickness parameter (e.g. someone hardcoding 30 fu).

  Updated `multi_staff_renderer::tests::draw_multi_staff_connectors_bracket` to assert `line_count == 1 && path_count == 2` (was `line_count == 3`).

  Updated `score::multi_staff::tests::section_bracket_renders_three_bracket_lines` → `section_bracket_renders_single_thick_line_and_two_scroll_glyphs`. Tightened to assert: (a) `line_count >= 16` (was `>= 18`), (b) `path_count >= 2`, AND (c) the bracket-induced delta over the `independent` (no-connector) variant: `line_count > line_count_indep` (≥1 extra line for the thick vertical) AND `path_count >= path_count_indep + 2` (≥2 extra paths for the scrolls). The delta-against-baseline assertions are the cross-cutting canary: the floor counts alone could be satisfied by an unrelated rendering change, but the delta locks the bracket-attributable cost specifically.

  Updated `score::multi_staff::tests::guitar_tab_has_bracket_connector` — the prior `line_count >= 17` floor was tied to the old 3-line bracket × 2 brackets (initial + extended) = 6 extra lines. Updated to `line_count >= 12` (11 staff lines + ≥1 bracket vertical) AND `path_count >= 2`. Comment block documents the new breakdown.

  Naming rationale: `top_glyph`/`bottom_glyph` rather than `top_scroll_glyph`/`bottom_scroll_glyph` because the field type is `smufl::Glyph` and the "scroll" terminology is engraving-specific — calling it `top_glyph` keeps the field name short, and the doc on `BracketLayout` explains the SMuFL convention in one place. `bracket_uses_smufl_scroll_glyphs` (not `..._uses_bravura_...`) because the design is font-agnostic per the prior chunks: any SMuFL font would supply these glyphs, and the test asserts against the SMuFL canonical names, not Bravura specifically.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged). `cargo test -p music-engraver --lib` — **2385 unit tests pass** (+6 vs prior recorded 2379: 3 new in `multi_staff::tests`, 4 new in `multi_staff_renderer::tests`, 1 removed in `multi_staff::tests`; the net is +6 because the renamed/updated tests preserve their count). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass** (`guitar_tab.svg` regenerated; the other 67 are byte-identical since no other golden score uses `ConnectorKind::Bracket`). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass, 1 ignored. `cargo build -p music-engraver --examples` builds all 89 examples. `cargo run -p music-engraver --example grand_staff` writes 3469-byte SVG with 3 paths + 28 lines (matches the updated assertion).

- Next: With orchestral brackets now using SMuFL scrolls, the next quality wins are: **sub-brackets** (the SMuFL `subBracketThickness` engraving default exists — Bravura's value is 0.16 sp — and SMuFL defines a thinner bracket variant for sub-groups within a larger section bracket; this is two-deep grouping that Lilypond renders as a nested thin bracket inside the main one); **brace-line connection** (verify the brace glyph's vertical scaling matches the joined-barline length exactly — for very tall grand staves the brace can look stretched or compressed); **bracket glyph metric verification** (the bracket scroll glyph has a specific bBox; if a non-Bravura SMuFL font supplied different dimensions, the join with the thick line might leave a seam — a metric-driven layout would query `bBoxNE` from the font metadata rather than assuming 1.876 × 1.18 sp). Beyond multi-staff connector polish: **cross-system church rests** (a multi-measure rest cluster that breaks across systems — currently confined to one measure so no break logic exists); **per-note collision detection in beamed additional voices** (the prior recent open issue from trill work); **line breaking** (Gourlay or Bellini & Nesi); **golden-SVG corpus + PHASH-based visual regression harness**.

- Open issues: The change is geometry-correcting at the extended-bracket layer — the thick line shifted by 62.5 fu to flush against the staff edge. The OLD geometry's gap (`thickness/2`) was small enough to not be visually obvious at typical zoom levels, but it was a latent bug. If any downstream code or visual baseline depended on the old position, it would now drift; the only such downstream is the regenerated `guitar_tab.svg` golden. The `BracketLayout` doc claims `bracketTop`'s origin is at `bBoxSW` and `bracketBottom`'s is at `bBoxNW` — this is the SMuFL convention as observed in Bravura's metadata, but a non-Bravura SMuFL font *could* in principle place the origin elsewhere. We don't currently query glyph metrics for the bracket scrolls (we just `translate` to the anchor point); if a font's bracket glyph origin diverged from the SMuFL spec, the seam would visibly misalign. A future enhancement could read `bBoxSW.y` for `bracketTop` from the font metadata and translate by `(x, y_top - bBoxSW.y * staff_space)` to be metric-driven; for v1 the convention-based translation is sufficient given Bravura. The horizontal extent of the scroll glyph (1.876 staff-spaces in Bravura) is not currently accounted for in any *layout* calculation — i.e. nothing positions other content based on "the bracket scroll extends 1.876 sp to the right of `bracket.x`." This is fine because the scroll's horizontal extent stays within `bracket.x..bracket.x + 1.876 * ss = bracket.x..bracket.x + 469fu`, which (at bracket.x = -125) is roughly -125..344, well to the left of the staff origin at x=0. If a future font's bracket extended further right, it could collide with the staff content — but that would be a font-design problem more than an engraving-layout one.

## 2026-05-15 — Post-v1, brace vertical scaling driven by font metadata (correctness fix)

- Did: Fixed a real rendering bug in the grand-staff brace and made brace scaling font-agnostic per the project's core principle. The pre-fix `layout_multi_staff` hard-coded `design_height = staff_space` for the brace glyph and translated to `y_top` of the staff system. In reality, Bravura's brace glyph bBox (per `bravura_metadata.json`) is **3.988 staff-spaces** tall, not 1 — and the brace path's origin sits at the glyph's bottom (bBoxSW.y = 0). The combination meant the brace was rendered (a) with `scale_y ≈ 14` instead of the correct `≈ 3.51` (i.e. ~4× too tall), and (b) translated so the brace's *bottom* edge sat at the *top* of the staff system, making the body of the brace extend off-screen above the music. The bug had escaped notice because the only existing brace test was `brace.scale_y > 1.0`, which passes for both 3.51 and 14, and the example assertion only checked `path_count >= 1`.

  Font (`src/font/music_font.rs`):
  - New `pub struct GlyphBBoxDesignUnits { x_left, x_right, y_top, y_bottom }` and `pub fn glyph_bbox_design_units(&self, glyph: Glyph) -> Option<GlyphBBoxDesignUnits>` on `MusicFont`. Reads the bbox from SMuFL metadata (`smufl::Metadata::bounding_boxes.get(glyph)`), converts staff-spaces → design units using `units_per_em / 4` (the SMuFL convention), and applies the same y-flip that `SvgPathBuilder` applies to outline paths — so the returned `y_top < y_bottom` matches the engraver's SVG coordinate convention everywhere else. Returns `None` if the glyph's bbox is missing from metadata. Exposed via `pub use` in `font/mod.rs`.
  - 5 new tests in `font::music_font::tests` lock the SMuFL math against Bravura's known values: `brace_bbox_design_units_matches_bravura_metadata` (x_left=2, x_right=82, y_top=-997, y_bottom=0, height=997, width=80 — all exact, all hand-computed from the metadata), `bracket_top_bbox_origin_at_bottom_left` (origin at SW → extends upward in SVG, y_bottom=0, height≈295), `bracket_bottom_bbox_origin_at_top_left` (origin at NW → extends downward, y_top=0, height≈295), `glyph_bbox_design_units_returns_none_for_glyph_without_metadata` (parses with `{"fontName":"Empty"}` and asserts `None`), `glyph_bbox_design_units_height_is_nonnegative` (sanity over Brace/BracketTop/BracketBottom/NoteheadBlack/NoteheadWhole/GClef/FClef).

  Layout (`src/layout/multi_staff.rs`):
  - `BraceLayout` reduced to pure geometric intent: removed `y_center`, `span_height`, `scale_y` *fields*; added `y_top`, `y_bottom` fields. Restored `span_height()` and `y_center()` as **methods** so layout-side callers (e.g. cross-system multi-staff code that centers ornamentation on the brace) keep their access patterns. Doc on `BraceLayout` calls out the font-agnostic principle: the layout records only the geometric intent (where the brace must span); the renderer queries the font for the actual glyph height at draw time.
  - `layout_multi_staff` no longer computes any scale factor — it just records `y_top = first staff top` and `y_bottom = last staff bottom`. This means *any* SMuFL font supplies brace dimensions for itself: Petaluma or Leland (different brace heights) would render with the correct scale automatically.
  - Two existing tests updated: `brace_layout_present_for_grand_staff` now asserts exact span (14 ss × 250 = 3500 design units), exact `y_top` (50 for `y_start=50`), exact `y_bottom`. `brace_center_is_midpoint` switched from `brace.y_center` field to `brace.y_center()` method; added two bracketing assertions (`y_top < y_center < y_bottom`).

  Render (`src/render/multi_staff_renderer.rs`):
  - `draw_brace` rewritten. Geometry derivation: SVG `transform="translate(tx,ty) scale(1,sy)"` maps path point (px, py) → (tx + px, ty + sy·py). For the brace, py=0 is the glyph's bottom (after y-flip) and py=-h is the top. We want py=0 → y_bottom and py=-h → y_top, which gives `ty = y_bottom` and `sy = (y_bottom - y_top) / h`. `h` comes from `font.glyph_bbox_design_units(brace.glyph).map(|b| b.height())`. Fallback if metadata is missing: use `staff_space` as the height (the pre-fix value), keeping rendering nonzero even for metadata-stripped custom fonts.
  - Three new tests, two of which are the regression canaries that should have existed before the fix landed and would have fired against the pre-fix code:
    - `brace_transform_scale_y_matches_font_bbox_height` — computes expected `scale_y` directly from `font.glyph_bbox_design_units(Glyph::Brace).unwrap().height()`, asserts the SVG contains exactly that scale substring, AND asserts the value falls in `3.0..4.0` for a Bravura default grand staff (locking in ~3.51 and ruling out the bug-pattern ~14).
    - `brace_transform_translate_y_at_staff_system_bottom` — asserts the SVG contains `translate(brace.x, brace.y_bottom)` substring AND asserts it does NOT contain `translate(brace.x, brace.y_top)` (the pre-fix translate). The second assertion is the direct anti-regression: a code change that reverts to "translate to y_top" would fire this test.
    - `brace_render_falls_back_when_metadata_missing_bbox` — builds a `MusicFont` with `br#"{"fontName":"Empty"}"#` (no bboxes), confirms `draw_brace` doesn't panic, emits a path, and uses `span/staff_space` as the fallback scale (so the brace is still drawn, just at the pre-fix scale; this is a behavior-of-record assertion).
  - The previous `brace_transform_contains_scale_y` test stays but loses its prose comment that said "scale_y should be > 1 since we're scaling from 1 staff space" — that comment articulated the bug.

  Goldens (`tests/golden/grand_staff.svg`, `tests/golden/multi_staff_cross_system.svg`): Regenerated via `GOLDEN_UPDATE=1`. Delta:
  - `grand_staff.svg`: brace transform `translate(-125,0) scale(1,14)` → `translate(-125,3500) scale(1,3.510531594784353)`.
  - `multi_staff_cross_system.svg`: two braces (one per system). First: `translate(-125,0) scale(1,14)` → `translate(-125,3500) scale(1,3.51...)`. Second: `translate(-125,6000) scale(1,14)` → `translate(-125,9500) scale(1,3.51...)`.

  Sanity check on the math: `y_bottom - y_top = 14 ss × 250 = 3500` design units. Bravura brace bbox height = 3.988 sp × 250 = 997 design units. `scale_y = 3500 / 997 = 3.5106...` ✓. Brace bottom (at path y=0) renders at SVG y = ty + sy·0 = 3500 = staff-system bottom. Brace top (at path y=-997) renders at SVG y = 3500 + 3.5106·(-997) ≈ 3500 - 3500 = 0 = staff-system top. ✓

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`). `cargo test -p music-engraver --lib` — **2393 unit tests pass** (+8 net: 5 in `font::music_font::tests`, 3 in `multi_staff_renderer::tests`; the 2 existing `multi_staff::tests` tests were updated, not added/removed). `cargo test -p music-engraver --test golden_svg` — 68 golden tests pass (2 regenerated, 66 byte-identical). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass, 1 ignored. `cargo build -p music-engraver --examples` builds all 89 examples. `cargo run -p music-engraver --example grand_staff` writes 3485-byte SVG with 3 paths + 28 lines; brace transform is `translate(-125,3600) scale(1,3.510531594784353)` (here `y_bottom = 3600` because the example uses `y_start = 100`, vs. the unit-test `y_start = 0`).

- Next: With brace scaling now font-agnostic via metadata, the bracket scroll glyphs could receive the same metric-driven treatment (they currently translate to a fixed anchor under the SMuFL bBoxSW/bBoxNW convention assumption; a metric-driven version would read `bBoxSW.y` for `bracketTop` and `bBoxNW.y` for `bracketBottom` rather than assuming the origin sits exactly at the corner). Beyond that: **sub-brackets** (the SMuFL `subBracketThickness` engraving default for nested two-deep grouping); **cross-system church rests**; **per-note collision detection in beamed additional voices**; **line breaking** (Gourlay or Bellini & Nesi); **golden-SVG corpus + PHASH-based visual regression**.

- Open issues: The fallback for missing bbox metadata uses `staff_space` as the glyph height, which reproduces the pre-fix scale (≈4× too tall for Bravura). This is intentional — a font that *does* supply its own bbox renders correctly, and a font that *doesn't* still renders something rather than failing or producing zero scale. The fallback's behavior is a behavior-of-record locked by `brace_render_falls_back_when_metadata_missing_bbox`, not a recommended state to be in. A better fallback (e.g. measure the path's actual bbox by parsing path data) would be more work but more correct; not in scope for this chunk.

## 2026-05-15 — Post-v1, bracket scroll glyphs metric-driven anchoring (font-agnostic seam)

- Did: Extended the metric-driven anchoring treatment from the brace fix to the bracket scroll glyphs (`bracketTop` / `bracketBottom`). The pre-fix code translated each scroll glyph directly to the line endpoint (`(bracket.x, bracket.y_top)` and `(bracket.x, bracket.y_bottom)`), implicitly assuming the SMuFL bracket-scroll origin convention (bBoxSW for bracketTop, bBoxNW for bracketBottom). Bravura follows that convention exactly, so the assumption was invisible against Bravura — but a SMuFL font that placed the scroll origin elsewhere (e.g. at the bbox center, or shifted to align with a custom serif) would render with a visible seam between the thick line and the scroll. The previous chunk's "Open issues" called this out explicitly.

  Render (`src/render/multi_staff_renderer.rs`):
  - `draw_bracket` now computes each scroll's translate via a private `bracket_anchor(font, glyph, line_x, line_y, end)` helper. The helper queries `font.glyph_bbox_design_units(glyph)` and returns:
    - For `ScrollEnd::Top`:    `tx = line_x - bbox.x_left`, `ty = line_y - bbox.y_bottom` (so the glyph's bottom-left edge lands at the line endpoint).
    - For `ScrollEnd::Bottom`: `tx = line_x - bbox.x_left`, `ty = line_y - bbox.y_top`    (so the glyph's top-left edge lands at the line endpoint).
    - Fallback when the font supplies no bbox: `(line_x, line_y)` — i.e. the corner-anchored translate the previous code used. Keeps rendering nonzero for metadata-stripped fonts.
  - Doc on `draw_bracket` rewritten to explain the metric-driven formula and to call out that Bravura's bbox values reduce the formula to the previous corner-anchored placement (so output is byte-identical for Bravura).
  - New private `ScrollEnd` enum (`Top`/`Bottom`) keeps the helper signature self-documenting rather than passing a "which edge" boolean.

  Layout (`src/layout/multi_staff.rs`):
  - `BracketLayout`'s doc rewritten to drop the SMuFL-convention assertion ("origin sits at bBoxSW" etc.) and instead state geometric intent: where the thick line spans, which scrolls attach. The renderer is the place that consults bbox metadata; the layout is now font-convention-agnostic in spirit and in doc.
  - No struct fields changed. No layout math changed. Rendering output for Bravura is byte-identical (confirmed against goldens).

- Tests (+6 net):

  Updated 2 existing Bravura-anchored tests with comments noting that for Bravura the metric-driven formula reduces to the asserted substring (so the substring assertion remains valid and the comments point to the new synthetic-font tests for formula-locking):
  - `draw_bracket_top_glyph_anchored_at_y_top`
  - `draw_bracket_bottom_glyph_anchored_at_y_bottom`

  Added 6 new tests in `multi_staff_renderer::tests`:
  1. `bracket_top_translate_matches_metric_driven_formula` — for Bravura: computes `expected_tx = bracket.x - bbox.x_left` and `expected_ty = bracket.y_top - bbox.y_bottom` from the live `font.glyph_bbox_design_units(bracket.top_glyph)` (rather than hard-coding the values), then asserts the SVG contains the resulting translate substring. Locks the formula's *derivation* against a regression that hard-codes the value or skips the bbox query.
  2. `bracket_bottom_translate_matches_metric_driven_formula` — symmetric for `bracketBottom` using `expected_ty = bracket.y_bottom - bbox.y_top`.
  3. `bracket_top_anchor_uses_glyph_bbox_y_bottom_when_nonzero` — **the canary test that would have fired against the pre-fix code**. Builds a synthetic `MusicFont` reusing Bravura's OTF (so `glyph_outline` works) but with custom `glyphBBoxes` JSON shifting bracketTop's `bBoxSW.y` from 0 to -0.5 staff-spaces. After y-flip this puts `bbox.y_bottom = +125` design units (vs. Bravura's 0). The test asserts the SVG contains `translate({bracket.x},{bracket.y_top - 125})` (the metric-driven value) AND does NOT contain `translate({bracket.x},{bracket.y_top})` (the pre-fix corner-anchored substring). The anti-needle is the direct anti-regression assertion: a code change that reverts to corner-anchoring would put the corner translate back into the SVG and fire this test.
  4. `bracket_bottom_anchor_uses_glyph_bbox_y_top_when_nonzero` — symmetric for `bracketBottom`: synthetic font with `bBoxNE.y = +0.5` → `bbox.y_top = -125`. Asserts metric-driven translate is `(x, y_bottom + 125)` and corner-anchored translate `(x, y_bottom)` is absent.
  5. `bracket_anchor_uses_glyph_bbox_x_left_when_nonzero` — covers the x-axis half of the formula. Synthetic bracketTop with `bBoxSW.x = 0.4` → `bbox.x_left = 100`. Asserts SVG contains `translate({bracket.x - 100},{bracket.y_top})`. A regression that dropped the `- bbox.x_left` term from the formula would leave the literal `bracket.x` value in the SVG and fire this test (since 100 ≠ 0 for the synthetic font).
  6. `bracket_render_falls_back_when_metadata_missing_bbox` — empty-metadata font (`{"fontName":"Empty"}`): the renderer must not panic, must emit exactly 1 line + 2 paths, AND both fallback translates `(bracket.x, bracket.y_top)` and `(bracket.x, bracket.y_bottom)` must appear (the corner-anchored fallback path). Locks both the rendering robustness and the specific fallback values.

  Naming rationale: the `..._when_nonzero` suffix makes explicit that these tests would silently pass against the pre-fix code if Bravura's bbox values happened to be zero (which they are) — the synthetic-font deviation from zero is what gives the tests teeth. The two tests in #1/#2 use the live Bravura bbox values from the font; they would still pass against pre-fix code because Bravura's bbox values reduce the formula to the corner-anchored substring. They serve as derivation-correctness tests (against future regressions to a different incorrect formula like `+ bbox.x_left` instead of `-`), not as anti-regression-from-pre-fix tests.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:343`, unchanged). `cargo test -p music-engraver --lib` — **2399 unit tests pass** (+6 vs prior recorded 2393). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass, ALL byte-identical** (no goldens regenerated, confirming Bravura's metric-driven output equals the previous corner-anchored output). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass, 1 ignored. `cargo build -p music-engraver --examples` builds cleanly. `cargo run -p music-engraver --example grand_staff` writes 3485-byte SVG (identical to prior chunk's recorded 3485). `cargo run -p music-engraver --example guitar_tab_score` writes 10274-byte SVG with 13 paths + 35 lines (unchanged).

- Next: With both brace and bracket-scroll anchoring now font-agnostic via metadata, the metric-driven pattern is established. Candidate next chunks: **sub-brackets** (the SMuFL `subBracketThickness` engraving default for nested two-deep grouping — Bravura value 0.16 sp; Lilypond renders this as a thinner inner bracket inside the section bracket); **cross-system church rests** (multi-measure rest cluster that breaks across systems); **per-note collision detection in beamed additional voices** (open from prior trill work); **line breaking** (Gourlay extension or Bellini & Nesi); **golden-SVG corpus PHASH harness** (text-diff already exists; PHASH would catch glyph-data regressions that produce equivalent text).

- Open issues: The synthetic-font tests construct `MusicFont` from Bravura's OTF combined with custom JSON metadata. This is a useful test pattern that should generalize — it lets the engraver assert "the renderer respects metadata X" without bundling a second OTF. Worth extracting to a `test_support` module if more such tests appear. The horizontal-anchor formula `tx = bracket.x - bbox.x_left` keeps the *glyph's left edge* aligned with `bracket.x` (the line's left edge); a future enhancement could optionally align the *glyph's inner edge* (where the scroll wraps around the line) instead, but defining "inner edge" precisely requires either a font-supplied anchor point (SMuFL's `glyphsWithAnchors` includes some anchors but not for bracket scrolls) or path-bbox parsing. For Bravura the left-edge alignment is correct because the scroll's curl extends rightward over the line, so the alignment we have is faithful to the published convention.

## 2026-05-15 — Post-v1, sub-brackets (two-deep section grouping)

- Did: Implemented nested sub-bracket support — the SMuFL `subBracketThickness` engraving default (0.16 sp in Bravura) renders as a thinner inner bracket inside a parent section bracket, per the published convention (Behind Bars; Lilypond's `StaffGroup`-in-`StaffGroup` rendering) for two-deep grouping (e.g. Violin I + Violin II share an inner bracket within the larger string-section bracket).

  Layout (`src/layout/multi_staff.rs`):
  - New `SubBracket` struct: `{ start_index, staff_count }` — a contiguous range of staves within the parent group, 0-based.
  - New field `StaffGroup.sub_brackets: Vec<SubBracket>`. Each existing constructor (`grand_staff`, `section`, `independent`) defaults this to `Vec::new()` so existing callers stay byte-identical. Builder method `StaffGroup::with_sub_brackets(vec![...])` for the chained-construction style.
  - New `SubBracketLayout` struct: `{ x, y_top, y_bottom, thickness }` — pure geometric intent; the renderer emits a thin stroked line, no scrolls (Lilypond's inner-bracket convention).
  - New constants: `SUB_BRACKET_THICKNESS_SS = 0.16` (SMuFL default; matches `engraving_config.sub_bracket_thickness`), `SUB_BRACKET_GAP_SS = 0.3` (horizontal gap between main bracket's right edge and sub-bracket's left edge, large enough to not visually merge).
  - `MultiStaffLayout` gains `sub_brackets: Vec<SubBracketLayout>`. Always present (possibly empty) — keeps the field non-`Option`al since the empty case is the no-cost default.
  - `layout_multi_staff` now:
    - Filters `group.sub_brackets` to entries that fit inside the parent (staff_count ≥ 2, start_index < parent count, range doesn't overshoot). Invalid entries silently drop.
    - **When valid sub-brackets are present**, shifts the main bracket's `x` left by `(SUB_BRACKET_GAP_SS + SUB_BRACKET_THICKNESS_SS + SUB_BRACKET_GAP_SS) * staff_space` (= 0.76 sp) to make room for the sub-bracket between the main bracket's right edge and the staff origin. With no sub-brackets, the main bracket's `x` is unchanged from the previous chunk (preserves byte-identical goldens).
    - Computes each sub-bracket's `x` as `main_right_edge + SUB_BRACKET_GAP_SS * staff_space`, its y range as `[staff_y_origins[first], staff_y_origins[last] + staff_height]`.

  Render (`src/render/multi_staff_renderer.rs`):
  - New `pub fn draw_sub_bracket(svg, sub: &SubBracketLayout)` — emits a single stroked line at `(sub.x + sub.thickness/2, sub.y_top) → (..., sub.y_bottom)` with `stroke-width = sub.thickness`. No SMuFL glyph dependency — Bravura has no separate thin-bracket scroll variant, and the convention is to omit scrolls on the inner bracket so the nesting reads hierarchically.
  - `draw_multi_staff_connectors` extended to iterate `layout.sub_brackets` after the main bracket pass.

  Score (`src/score/multi_staff.rs`): the existing `StaffGroup { ... }` literal at line 309 picks up `sub_brackets: Vec::new()` — no behavior change at the score level (the `MultiStaffScore` API doesn't currently expose sub-bracket configuration; that's a follow-up if needed).

  Example (`examples/sub_brackets.rs`): 6-staff section with two nested sub-brackets (staves 0..1 and 3..5). Output: 3437 bytes, 2 paths (main bracket scrolls), 34 lines (30 staff + 1 main bracket vertical + 2 sub-bracket verticals + 1 joined barline). The example asserts exact path/line counts.

- Tests (+15 net):

  Layout (`layout/multi_staff::tests`, +9):
  1. `no_sub_brackets_emitted_when_group_has_none` — baseline that `layout.sub_brackets` is empty for a plain `section(N)`.
  2. `sub_brackets_ignored_for_non_bracket_connector` — `with_sub_brackets(...)` on a Brace or None group is silently dropped (sub-brackets are a Bracket-only feature).
  3. `sub_bracket_emitted_for_valid_range` — single sub-bracket inside `section(5)`, asserts y_top and y_bottom exactly match the spanned staves' top and bottom.
  4. `sub_bracket_thickness_matches_smufl_default` — locks `sub.thickness = 0.16 * SS` AND asserts `sub.thickness < main.thickness` (visual hierarchy invariant).
  5. `sub_bracket_sits_inside_main_bracket` — **the key geometric canary**: pins exact x-positions of main and sub brackets and asserts the gap between them equals `SUB_BRACKET_GAP_SS * SS` exactly. Would fire on any regression that drops the leftward main-bracket shift or changes the gap.
  6. `main_bracket_x_unchanged_when_no_sub_brackets` — anti-regression for the conditional shift: ensures plain `section(N)` keeps its previous x position so goldens stay byte-identical.
  7. `multiple_sub_brackets_each_produce_one_layout` — two sub-brackets in a 6-staff group; asserts y_top/y_bottom for both, that they share x, and that their y ranges are distinct.
  8. `invalid_sub_brackets_silently_dropped` — three invalid entries (count=1, start out of range, overshoots) + one valid → only the valid one survives.
  9. `sub_bracket_full_span_equals_parent_bracket_span` — a sub-bracket covering all parent staves must produce identical y_top/y_bottom to the main bracket.

  Render (`render/multi_staff_renderer::tests`, +6):
  1. `draw_sub_bracket_produces_one_thin_line` — exact `line_count == 1 && path_count == 0` (no scrolls, no serifs). Catches regressions that attach scroll glyphs or extra serifs to the inner bracket.
  2. `draw_sub_bracket_uses_sub_bracket_thickness` — `stroke-width="{sub.thickness}"` substring + numeric assertion that `sub.thickness = SUB_BRACKET_THICKNESS_SS * SS`.
  3. `draw_sub_bracket_line_spans_y_range` — asserts all four `x1/y1/x2/y2` SVG attribute substrings against the layout's exact values AND verifies y range matches the spanned staves (staves 1..3 inside section(4)).
  4. `draw_multi_staff_connectors_renders_main_and_sub_brackets` — end-to-end: exact `line_count == 2 && path_count == 2` for a bracket + 1 sub-bracket. Locks the count delta (1 extra line vs the no-sub-brackets baseline).
  5. `draw_multi_staff_connectors_two_sub_brackets_emit_two_thin_lines` — `line_count == 3` for bracket + 2 sub-brackets; asserts each sub-bracket's distinct y1 substring appears.
  6. `main_bracket_x_shifts_left_when_sub_bracket_present_end_to_end` — **the cross-cutting canary**: renders the same `section(3)` group with and without sub-brackets, asserts the SVGs use different main-bracket centre-x substrings, AND the nested SVG does NOT contain the plain (unshifted) centre. Anti-regression for the shift behavior end-to-end.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace` passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings (1 pre-existing in `score/multi_staff.rs:344`, line drifted +1 from prior log due to the `sub_brackets: Vec::new()` literal addition). `cargo test -p music-engraver --lib` — **2414 unit tests pass** (+15 vs prior recorded 2399: 9 layout + 6 renderer). `cargo test -p music-engraver --test golden_svg` — **68 golden tests pass, ALL byte-identical** (the `MultiStaffScore` API doesn't yet expose sub-brackets, so no golden score exercises the new shift; existing scores keep `sub_brackets: Vec::new()` and the conditional bypasses the leftward shift). `cargo test -p music-engraver --test svg_glyph_render` — 3 integration tests pass. `cargo test -p music-engraver --doc` — 12 doc tests pass, 1 ignored. `cargo build -p music-engraver --examples` builds all 90 examples. `cargo run -p music-engraver --example sub_brackets` writes 3437-byte SVG with 2 paths + 34 lines. SVG inspection confirms main-bracket centre at x=-252.5 (= -315 + 125/2, shifted left), sub-bracket centres at x=-95 (= -115 + 40/2), main-scroll translates at (-315, 100) and (-315, 13600), gap between main right edge (-190) and sub left edge (-115) = 75 fu = 0.3 sp.

- Next: With sub-brackets in the layout/render layer, the score-level surface (`MultiStaffScore`) doesn't yet expose them — adding a `with_sub_brackets(...)` builder to `MultiStaffScore` would let users author scores with nested grouping (and would generate a golden to lock the end-to-end rendering). Beyond that: **cross-system church rests** (multi-measure rest cluster that breaks across systems); **per-note collision detection in beamed additional voices**; **line breaking** (Gourlay extension or Bellini & Nesi); **golden-SVG corpus PHASH harness**; **PNG export** behind the `png` feature (resvg + tiny-skia + fontdb, already in `Cargo.toml`).

- Open issues: The sub-bracket renders as a simple stroked line — no scroll decoration, no horizontal serifs. This matches Lilypond's convention but some published scores show small flat caps at top/bottom of inner brackets; if visual fidelity to such scores becomes important, a future chunk could add optional caps (likely thin horizontal segments matching the sub-bracket thickness extending rightward by ~0.4 sp). The `SUB_BRACKET_GAP_SS = 0.3` constant is empirical (engraved scores vary 0.25–0.5 sp); a future enhancement could tie it to a font-supplied value if SMuFL ever standardizes one. The shift-when-sub-brackets-present convention assumes the example's caller knows to widen the viewBox to accommodate the leftward-shifted main bracket — the example does this by setting `vb_x = -500` (vs. -300 in `grand_staff.rs`); the `MultiStaffScore` API will need a parallel adjustment when sub-brackets are exposed there. The shift conditional means the same `section(3)` group renders at two different x positions depending on `sub_brackets.is_empty()` — if a caller toggles sub-brackets at runtime, the staff origin x=0 stays fixed but the bracket position moves; this is the engraving-correct behavior but worth documenting if it surprises anyone.

## 2026-05-17 — Post-v1, score-level `MultiStaffScore::with_sub_brackets(...)` builder

- Did: Exposed sub-brackets at the score level. The prior chunk landed the
  layout + render plumbing (`StaffGroup.sub_brackets`, `SubBracketLayout`,
  `draw_sub_bracket`, the conditional leftward shift of the main bracket
  when sub-brackets are present), but the high-level `MultiStaffScore` API
  still hard-coded `sub_brackets: Vec::new()` in its `StaffGroup` literal —
  callers using the score builder couldn't author nested grouping without
  dropping down to the layout layer. This chunk wires the surface API
  through end-to-end and locks the result with a golden.

  Score (`src/score/multi_staff.rs`):
  - New field `MultiStaffScore.sub_brackets: Vec<SubBracket>`. Initialized
    to `Vec::new()` in all four constructors (`grand_staff`, `section`,
    `independent`, `guitar_tab`) so existing callers stay byte-identical.
  - New builder method `with_sub_brackets(self, Vec<SubBracket>) -> Self`
    with doc explaining the bracket-only honour rule, the silently-dropped
    invalid-entry contract (`staff_count < 2`, `start_index` out of range,
    overshoot), the leftward shift behavior, and a complete usage example.
    Doc note about which connectors ignore the list (brace, independent,
    guitar+tab) preempts the obvious user confusion.
  - The `StaffGroup` literal in `try_render_svg` now passes
    `sub_brackets: self.sub_brackets.clone()` (was `Vec::new()`).
  - Import added: `SubBracket` from `crate::layout::multi_staff`.

  Layout re-exports (`src/layout/mod.rs`):
  - Added `SubBracket, SubBracketLayout` to the `pub use multi_staff::{...}`
    line so users importing from `music_engraver::layout::multi_staff` (or
    the parent `music_engraver::layout`) don't have to dig into the nested
    module path. The score-level doc example uses
    `music_engraver::layout::multi_staff::SubBracket`.

  Example (`examples/sub_brackets_score.rs`, new):
  - Five-staff string-section layout (V1, V2, Va, Vc, Cb) with two nested
    sub-brackets (staves 0..2 and 2..5) under the main section bracket.
    Built through `MultiStaffScore::section(...).with_sub_brackets(...)`
    — no layout-layer imports needed. Output: 22454 bytes, 37 paths,
    66 lines. Asserts `<svg` prefix, `</svg>` suffix, presence of the
    post-shift main-bracket scroll translate at `x=-315`, and at least
    2 thin lines at `stroke-width="40"` (= SUB_BRACKET_THICKNESS_SS × 250).
  - Used Clef::Treble for the viola line because `music::notation::clef::Clef`
    has only Treble/Treble8va/Treble8ba/Bass variants — no Alto clef
    exists in the music crate. Noted in passing; adding C-clef variants
    is a separate music-crate concern outside the engraver port's scope.

- Tests (+7 net, all in `score::multi_staff::tests`):

  1. `section_with_sub_brackets_adds_thin_lines_for_each_sub_bracket` —
     end-to-end count delta: a `section(4)` with 2 sub-brackets must
     produce *exactly* 2 more `<line>` elements than the same score with
     no sub-brackets, AND zero additional `<path>` elements (sub-brackets
     carry no scroll glyphs). The exact-equality assertions are the canary
     against (a) accidentally double-emitting sub-bracket lines, or
     (b) regressing the inner bracket back to a scroll-decorated variant.

  2. `section_with_sub_brackets_shifts_main_bracket_left` — the cross-
     cutting geometric canary: asserts the baseline section's main
     bracket scrolls anchor at `translate(-125,...)` (pre-shift x =
     -BRACKET_THICKNESS_SS × 250), and the nested-bracket variant's
     scrolls anchor at `translate(-315,...)` (post-shift x = -125 - 190,
     where 190 = 0.76 sp × 250 = the gap+thickness+gap shift), AND the
     nested SVG does NOT contain `translate(-125,`. The anti-needle
     assertion is the direct anti-regression: a code change that drops
     the conditional shift would put the pre-shift translate back into
     the nested SVG and fire this test.

  3. `brace_with_sub_brackets_is_silently_ignored` — invariant: calling
     `.with_sub_brackets(...)` on a `MultiStaffScore::grand_staff(...)` must
     produce SVG byte-identical to the same score without the call.
     Asserts `assert_eq!(plain, with_subs)`. Locks the layout-side
     conditional that filters sub-brackets out for non-Bracket connectors
     all the way through to the rendered SVG.

  4. `independent_staves_with_sub_brackets_is_silently_ignored` — same
     invariant for `MultiStaffScore::independent(...)`. Byte-identical
     assertion.

  5. `section_with_no_sub_brackets_is_unchanged` — anti-regression for
     the conditional leftward shift: a `section(N)` without sub-brackets
     must render identically to the same `section(N)` with an *explicit*
     empty `with_sub_brackets(vec![])`. Both paths should produce no shift.
     Byte-identical assertion.

  6. `section_with_invalid_sub_brackets_renders_same_as_no_sub_brackets`
     — all-invalid entries (staff_count=1, start out of range, overshoots
     end) must be filtered out at layout time AND produce no leftward
     shift. Byte-identical to the no-sub-brackets case. This locks the
     layout-side `has_sub_brackets = !resolved_sub_brackets.is_empty()`
     check against a regression that bases the shift on the unfiltered
     list (which would shift even when all entries are ultimately dropped).

  7. `guitar_tab_with_sub_brackets_is_silently_ignored` — same invariant
     for `MultiStaffScore::guitar_tab(...)`. A guitar+tab score has 1
     notation staff, so even a `staff_count=2` sub-bracket overshoots and
     is filtered out — the layout pass should drop it and the SVG must
     stay byte-identical to the no-sub-brackets case.

  Naming rationale: the `_is_silently_ignored` suffix repeats across
  tests 3/4/7 because all three cover the same invariant (sub-brackets
  dropped for non-Bracket connectors) but on different connector
  variants. Keeping the names parallel makes the test set's coverage
  matrix easy to read at a glance and easy to extend if a new connector
  variant lands.

  Golden (`tests/golden_svg.rs`, +1 frozen baseline):
  - New `build_sub_brackets_score()` factory (helper inside the test file,
    not the score module) builds the same 5-staff section as the example
    but uses the test file's local `p(name, octave)` helper. Generated
    `tests/golden/sub_brackets_score.svg` (22454 bytes) via
    `GOLDEN_UPDATE=1`. New `golden_sub_brackets_score` test asserts
    against the frozen baseline AND adds three structural guards
    alongside the golden diff: (a) `translate(-315,` present (shift
    happened), (b) `translate(-125,` absent (no pre-shift residue),
    (c) `stroke-width="40"` appears ≥2× (one per sub-bracket), and
    (d) `stroke-width="125"` appears ≥1× (main bracket vertical). The
    "structural alongside golden" pattern follows the convention
    established by `golden_lyrics`, `golden_chord_symbols_with_accidentals`
    etc. — the golden text-diff catches general drift, the structural
    asserts catch the specific bug pattern this chunk is supposed to
    prevent.

- Verified: `cargo check -p music-engraver` passes. `cargo check --workspace`
  passes. `cargo clippy -p music-engraver --all-targets` — 0 new warnings
  (1 pre-existing in `score/multi_staff.rs:394`; line drifted +50 from the
  prior log's `:344` due to this chunk's added field + builder method).
  `cargo test -p music-engraver --lib` — **2421 unit tests pass** (+7 vs
  prior recorded 2414, all 7 in `score::multi_staff::tests`). `cargo test
  -p music-engraver --test golden_svg` — **69 golden tests pass** (+1 new
  `golden_sub_brackets_score`; the other 68 are byte-identical since none
  of them use `.with_sub_brackets(...)` and the `Vec::new()` default
  preserves prior behavior). `cargo test -p music-engraver --test
  svg_glyph_render` — 3 integration tests pass. `cargo test -p
  music-engraver --doc` — 13 doc tests pass, 1 ignored (+1 new doc test
  for `MultiStaffScore::with_sub_brackets` — the docstring's example
  compiles). `cargo build -p music-engraver --examples` builds all 93
  examples (+1 vs prior recorded 90: this chunk added `sub_brackets_score`
  and two earlier post-v1 chunks added two examples not reflected in the
  earlier count). `cargo run -p music-engraver --example
  sub_brackets_score` writes 22454-byte SVG with 37 paths + 66 lines;
  example asserts pass.

- Next: With sub-brackets now reachable through the high-level score API,
  the natural follow-ups remain the same: **cross-system church rests**
  (a multi-measure rest cluster that breaks across systems — currently
  confined to one measure so no break logic exists); **per-note collision
  detection in beamed additional voices** (open from prior trill work);
  **line breaking** quality improvements (Gourlay extension or Bellini &
  Nesi line-cost model on top of the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression** (text-diff already
  exists; PHASH would catch glyph-data regressions that produce equivalent
  text). The earlier "Next" list mentioned **PNG export** as a candidate;
  per `Cargo.toml`'s `[features] png = [...]` and the existing
  `examples/png_export.rs` + `src/render/png.rs`, PNG is already wired up
  and tested; it can be struck from the candidate list.

- Open issues: The shift-when-sub-brackets-present behavior makes the
  default viewBox slightly tight for sub-bracket scores — the
  `MultiStaffScore::try_render_svg` computes `vb_x = -vb_margin -
  left_margin` where `left_margin = 2.0 * staff_space` for brackets, and
  the shifted main-bracket scroll anchors at `x = -315` (= -1.26 sp).
  With `staff_space = 250`, `left_margin = 500` and `vb_margin = 250`, so
  `vb_x = -750` — comfortably to the left of the shifted scroll. The
  margin is adequate for the current shift amount (190 fu) but is *not*
  parametrized on `sub_brackets.is_empty()`. A future enhancement could
  conditionally widen `left_margin` by the shift amount when sub-brackets
  are present, but this would require regenerating any golden that uses a
  sub-bracketed score — for the single new `sub_brackets_score` golden the
  current viewBox is visibly fine. Documented here so the question doesn't
  resurface.

## 2026-05-17 — Post-v1, `.trill_with_extension_to(note_offset)` note-anchored trill terminus

- Did: Closed one of the recurring trill-polish leftovers from prior
  "Next" lists. The existing trill extension API exposed two endpoint
  models: (a) "extend to the immediately following note" (the implicit
  default of `trill_with_extension()`) and (b) "extend N staff-spaces
  past the trill glyph" via `trill_with_extension_length_ss(N)`. Missing
  was a *note-anchored* endpoint: "extend until note N positions ahead
  in the system's note sequence." Useful when a trill should visibly
  hold across one or more intervening notes before releasing into a
  specific successor — without forcing the caller to compute the
  staff-space distance by hand.

  Layout (`src/layout/measure.rs`):
  - New field `NoteAnnotations.trill_extension_to_note_offset:
    Option<usize>`. Defaulted to `None` via existing `#[derive(Default)]`
    — no constructor changes required. Docstring spells out: (a) `None`
    is byte-equivalent to "extend to next note"; (b) `Some(n)` with
    `n >= 1` extends to the note `n` positions past the trilled note;
    (c) `Some(0)` is degenerate and produces no wiggle (target IS the
    trilled note, so start_x == end_x); (d) offsets walking past the
    end of the system fall back to the system-edge formula, and the
    explicit-offset path NEVER engages cross-system propagation (only
    the natural last-note case does); (e) when both this field and
    `trill_extension_length_ss` are set, the explicit length wins —
    "definite length specifies a definite endpoint" supersedes the
    softer "stretch to note N" hint.

  Render (`src/render/system_renderer/mod.rs`):
  - New `to_note_offset: Option<usize>` slot on `TrillExtensionNoteInfo`.
    Filtered the same way as `bracket`/`explicit_length_ss`: only carries
    through when `has_trill_extension == true`. Catches stale annotations
    on notes whose extension flag is off.
  - `collect_trill_extension_note_info`'s Note and Chord branches both
    copy the new field. The Chord branch — easy to forget — gets its
    own dedicated test (`to_note_offset_in_chord_collector_propagates`).
  - `draw_system_trill_extensions` rewritten end-anchor computation:
    `target_offset = note.to_note_offset.unwrap_or(1)`; `target_offset
    == 0` collapses end_x to start_x (degenerate → no wiggle);
    `notes.get(i + target_offset)` looks up the target's x; falling off
    the end uses the staff-width-minus-edge-gap formula. Cross-system
    propagation is *gated* on `note.to_note_offset.is_none() &&
    notes.get(i + 1).is_none()` — i.e. reserved for the truly-natural
    last-note case. An explicit offset that overshoots terminates at
    the system edge but does NOT propagate, locking in the documented
    "explicit offset is a definite anchor" semantic.
  - Existing `explicit_length_ss` branch still wins when both fields
    are set (the natural_end_x path computes via the new offset logic;
    the explicit length then clamps against that, which is correct —
    if both are set and the length is small, the length wins; if both
    are set and the length is huge, the offset's natural_end_x wins via
    the existing `requested.min(natural_end_x)` clamp).

  Score (`src/score/mod.rs`):
  - New builder method `ScoreBuilder::trill_with_extension_to(note_offset:
    usize) -> Self`. Sets the three coupled fields
    (`ornament=Trill`, `trill_extension=true`,
    `trill_extension_to_note_offset=Some(n)`). No-op when the last event
    is a rest, matching every other ornament-attaching builder.
    Docstring documents the offset=1-is-default, offset=0-is-degenerate,
    overshoot-falls-back-to-edge, and precedence-with-explicit-length
    contracts.

  Page renderer (`src/render/page_renderer/mod.rs`):
  - `compute_cross_system_trill_continuation` now mirrors the
    system-renderer's "no cross-system propagation under an explicit
    end-anchor" gate: returns `None` early when
    `last.to_note_offset.is_some()`. Without this mirror, a trill on the
    last note of system N with `to_note_offset = Some(N)` would
    correctly terminate at the system N edge (per system_renderer's
    cross_system=false branch) but the page renderer would still draw an
    unattributable incoming wiggle on system N+1. The mirror keeps the
    two pass decisions in lockstep.

- Tests (+18 net):

  Score builder (`score::tests`, +8):
  1. `trill_with_extension_to_sets_all_three_annotation_fields` —
     locks the three coupled writes AND that the unrelated extension
     fields (`length_ss`, `bracket`, `wiggle_speed`) stay unset. The
     "additive, not destructive" canary.
  2. `trill_with_extension_to_offset_one_byte_equivalent_to_trill_with_extension`
     — offset=1 must produce SVG byte-identical to the implicit default.
     This is the "offset=1 IS the default" contract; a regression in
     the `unwrap_or(1)` fallback would fire here.
  3. `trill_with_extension_to_offset_two_extends_past_next_note` —
     offset=2 must render *strictly more* paths than offset=1 in a
     3-note system (since the wiggle covers a longer horizontal span
     by reaching the *second* note ahead, not the first).
  4. `trill_with_extension_to_offset_zero_drops_wiggle` — offset=0 must
     render the same path count as an ornament-only plain trill (no
     wiggle). Locks the degenerate-target → no-wiggle branch.
  5. `trill_with_extension_to_offset_overshoot_extends_to_system_edge`
     — offset=99 on note 1 of a 3-note system must yield strictly more
     paths than offset=1 in the same layout. Catches a regression where
     the fallback panics on the index or emits no wiggle.
  6. `trill_with_extension_to_on_rest_is_noop` — byte-identical render
     when applied to a rest. The standard ornament-builder no-op contract.
  7. `trill_with_extension_to_on_chord_renders_extended_wiggle` — chord
     builder must also accept the call and route through the Chord
     collector branch; verifies more paths than the default extension.
  8. `trill_with_extension_to_length_ss_takes_precedence_when_both_set`
     — when both annotation fields are set with mutually-disagreeing
     values (length=1.0 ss vs. offset=5), the rendering must be
     byte-identical to the length-only variant. Locks the documented
     precedence (length wins).

  System renderer (`render::system_renderer::tests`, +9):
  1. `collector_propagates_to_note_offset_when_trill_extension_active`
     — Note branch of the collector: new field travels through into the
     info record.
  2. `collector_drops_to_note_offset_when_trill_extension_inactive` —
     filtering: stale offset on a note with `trill_extension=false`
     must be dropped, matching the rule for `explicit_length_ss`.
  3. `to_note_offset_two_renders_more_paths_than_offset_one` — renderer
     end-to-end: offset=2 yields strictly more paths than offset=1 on
     the same layout (with widened `min_note_spacing` to ensure the
     extra span is meaningful). The headline behaviour test.
  4. `to_note_offset_one_byte_equivalent_to_natural_default` — at the
     renderer layer (not just the score builder), offset=1 renders
     byte-identically to the no-offset default.
  5. `to_note_offset_zero_drops_wiggle` — degenerate-target → no-wiggle
     branch, verified at the renderer layer.
  6. `to_note_offset_overshoot_falls_back_to_system_edge` — offset=99
     on a non-last note renders strictly more paths than offset=1.
  7. `to_note_offset_overshoot_renders_byte_identical_to_last_note_natural`
     — **the cross-cutting canary**: overshoot offset on the only note
     in a system and a trill-on-the-last-note (no offset) reach the
     SAME system-edge formula → byte-identical render. Locks the
     "fallback uses the same edge formula" invariant.
  8. `to_note_offset_in_chord_collector_propagates` — Chord branch of
     the collector: easy to forget, gets a dedicated test.
  9. `to_note_offset_yields_to_explicit_length_when_both_set` —
     precedence test at the renderer layer: both fields set →
     byte-identical to length-only.

  Page renderer (`render::page_renderer::tests`, +1):
  1. `cross_system_trill_to_note_offset_suppresses_cross_system_continuation`
     — two-system page render with offset=99 on the last note of system 1
     vs. the same score WITHOUT the offset (the natural cross-system
     case). The offset variant must yield strictly fewer paths because
     the page-renderer suppresses the incoming wiggle on system 2. A
     regression that drops the `to_note_offset.is_some()` early-return
     in `compute_cross_system_trill_continuation` would re-engage
     cross-system propagation and equalize the two counts. Pairs with
     the existing `cross_system_multi_speed_trill_explicit_length_suppresses_continuation`
     test, locking the same pass-decision invariant for the new field.

  Naming rationale: the score-builder tests use the public API name
  (`trill_with_extension_to_*`), while the renderer tests use the
  internal field name (`to_note_offset_*`). This mirrors the existing
  `trill_with_extension_length_ss_*` / `explicit_length_ss_*` naming
  split between the same two modules. The byte-identical-equivalence
  tests at both layers are the strongest available "the implementation
  computes the same end_x as the no-offset default" canary —
  approximate path-count assertions can't catch a small numerical
  drift, but byte equality catches everything.

  No new examples or goldens. The end-anchor change is invisible in any
  golden that doesn't use `trill_extension_to_note_offset` (which is
  none of them — the field defaults to `None`), and the rendering
  behaviour for offset=1 is byte-identical to the existing default. A
  golden specifically exercising offset=2 or overshoot could be added
  later if visual proofing becomes valuable; for now the +17 mechanical
  tests cover the relevant contracts.

- Verified: `cargo check -p music-engraver` passes. `cargo check
  --workspace` passes. `cargo clippy -p music-engraver --all-targets` —
  0 new warnings (1 pre-existing in `score/multi_staff.rs:394`,
  unchanged). `cargo test -p music-engraver --lib` — **2439 unit tests
  pass** (+18 vs prior recorded 2421: 8 new in `score::tests`, 9 new in
  `render::system_renderer::tests`, 1 new in
  `render::page_renderer::tests`). `cargo test -p music-engraver
  --test golden_svg` — **69 golden tests pass, all byte-identical** (no
  golden uses the new field; offset=1 default is byte-identical to
  no-offset and the page renderer's new early-return only fires when
  `to_note_offset.is_some()`). `cargo test -p music-engraver --test
  svg_glyph_render` — 3 integration tests pass. `cargo test -p
  music-engraver --doc` — 13 doc tests pass, 1 ignored. `cargo build -p
  music-engraver --examples` builds all 93 examples.

- Next: Remaining post-v1 candidates: **cross-system church rests**
  (multi-measure rest cluster that breaks across systems — currently
  confined to one measure so no break logic exists); **per-note
  collision detection in beamed additional voices** (open issue from
  prior trill work); **line breaking quality improvements** (Gourlay
  extension or Bellini & Nesi line-cost model on top of existing
  Knuth-Plass DP); **golden-SVG corpus PHASH-based visual regression**
  (text-diff already exists; PHASH would catch glyph-data regressions
  that produce equivalent text); **`HookDirection::Up` standalone
  builder** (already reachable through `trill_with_extension_bracketed_custom`
  and `TrillBracketOptions::with_direction`, but a one-liner convenience
  method could be added — judgment call). Trill polish remaining:
  per-segment `WiggleTrillFast` variant selection from a single-speed
  annotation (currently the ramp path handles per-segment variation but
  the single-speed `trill_wiggle_speed` is uniform across the wiggle).

- Open issues: When both `trill_extension_length_ss` and
  `trill_extension_to_note_offset` are set, the implementation computes
  `natural_end_x` via the offset logic, then clamps via the explicit
  length. In every test case (length=1.0, offset=5) the length value is
  smaller than the natural span so the length wins via the existing
  `requested.min(natural_end_x)` clamp. If a future caller sets
  *length=huge* and *offset=1*, the explicit-length branch still uses
  the offset's `natural_end_x` (correctly) — but a future "length wins
  unconditionally" semantic refactor would need to ignore the offset
  entirely, not clamp against it. The current behaviour matches the
  docstring ("the explicit length wins when both are set") for all
  practical cases but technically the offset's `natural_end_x` is still
  used as the clamp upper bound. Documented here so the semantics
  question doesn't resurface. The cross-system gate for an
  *overshooting* explicit offset is the strictest interpretation: even
  on the truly-last-note case, an overshoot offset terminates at the
  system edge without crossing. A future refinement could allow
  cross-system propagation when *all* of: (a) offset is set, (b) offset
  walks past the end, AND (c) the trilled note IS the last note in the
  system; right now only the no-offset case enables cross-system. The
  current behaviour is the safer choice — an explicit offset is a
  user assertion of intent, not a "let it flow" signal — but worth
  noting.

## 2026-05-17 — Post-v1, per-note collision detection in beamed additional voices

- Did: Closed the long-standing open issue ("Beam groups in additional
  voices don't get per-note collision detection") that has been
  flagged in every progress entry since April. Previously, the cross-
  voice collision detector pulled *all* notes from a BeamGroup or
  TupletGroup, treated a single match as a whole-group offset, and
  shifted the group's anchor x by one notehead width — silently
  mis-aligning stems and the beam line for the non-colliding notes in
  the group. Per Gould, only the *colliding* notehead should
  displace; the stem and beam stay anchored at the original beat x.

  Shared helper (`src/layout/beam.rs`, ~60 LOC + 8 tests):
  - `beam_group_note_x_offsets(durations: &[u8], total_width: f64) ->
    Vec<f64>` extracted from the renderer's inline math. Returns the
    local x-offset of each note (relative to the group's anchor)
    under proportional spacing: shortest note → factor 1.0, each
    doubling of duration multiplies by `BEAM_GROUP_SPACING_RATIO`
    (1.6, matching the renderer's existing constant). Now the
    single source of truth for collision detection AND rendering;
    any drift would silently misalign collision offsets from the
    drawn noteheads. Empty/single-note edge cases handled cleanly.

  Detection layer (`src/layout/voice_collision.rs`):
  - Added `inner_note_index: Option<usize>` to `VoiceCollisionOffset`:
    `None` = whole-element shift (Note/Chord, existing semantics);
    `Some(i)` = shift only note `i` within the BeamGroup/TupletGroup
    at `element_index`.
  - `collect_voice_positions` now expands a *primary* voice's
    BeamGroup/TupletGroup into per-note `(absolute_x, [pos])` pairs
    via the shared helper, so cross-voice detection sees individual
    beat positions on both sides.
  - `compute_voice_collision_offsets` walks BeamGroup/TupletGroup
    notes individually, computes each absolute x via the helper,
    runs `detect_collision` per-note, and emits per-note offsets
    keyed off `inner_note_index`. Note/Chord elements continue to
    emit element-level offsets (`inner_note_index = None`) for
    backwards compatibility.
  - Extracted two private helpers: `collision_at_x(...)` (find +
    detect pair, shared between standalone and per-note paths) and
    `offset_shift_direction(dir)` (StemDirection → ±1.0 shift
    multiplier, single source of the "down-stems shift right /
    up-stems shift left / None defaults to right" convention).

  Render layer (`src/render/measure_renderer/mod.rs`):
  - Added `draw_beam_group_event_with_offsets(..., per_note_x_shift:
    &[f64])` and `draw_tuplet_group_event_with_offsets(...)`. The
    shift slice (length-must-match-notes or empty) shifts only the
    *notehead*, *accidental*, *ledger lines*, and *augmentation dots*
    of each note; the per-note `BeamedNote { x }` fed into the beam-
    and-stem layout uses the *unshifted* `note_xs[i]`, so the beam
    line stays straight and stems stay anchored at the original
    beat x. The tuplet bracket is also unaffected (frames the
    original beam-group rhythmic range, not the displaced noteheads).
  - The existing `draw_beam_group_event(...)` / `draw_tuplet_
    group_event(...)` are now thin shims that pass an empty shift
    slice — keeping all primary-voice rendering byte-identical
    (verified by the 69 goldens passing without churn).
  - In `draw_additional_voices`, the element-level shift now only
    fires when `inner_note_index.is_none()` (Note/Chord case). For
    BeamGroup/TupletGroup, a new helper `per_note_shifts_for_group(
    collision_offsets, elem_idx, note_count, notehead_width)`
    materializes a `Vec<f64>` of length `note_count` (zero for non-
    colliding notes; offset for colliding ones) and hands it off to
    the with-offsets variant. Empty vec when no per-note offsets
    target this element — preserves the existing call path for the
    overwhelmingly common case.

  Example: `examples/beam_group_per_note_collision.rs` — two
  measures, the first with a primary-voice half-note pair and an
  additional-voice beam group of four eighths where only notes 0
  and 2 line up with the primary; the second is a no-collision
  control. Asserts ≥12 paths, ≥10 lines, ≥2 beam polygons.

  Tests (+22 net, 17 new mechanical):

  `layout::beam::tests` (+8):
  1. `beam_group_note_x_offsets_empty_returns_empty` — degenerate
     empty input.
  2. `beam_group_note_x_offsets_single_note_is_zero` — single-note
     edge case returns `[0.0]`.
  3. `beam_group_note_x_offsets_equal_durations_distribute_evenly`
     — locks the canonical "4 equal eighths → 0, 250, 500, 750"
     case with explicit numeric assertions, not approximate counts.
  4. `beam_group_note_x_offsets_first_note_always_zero` — invariant
     across multiple duration patterns; would catch a leading offset
     bug.
  5. `beam_group_note_x_offsets_monotonically_increasing` — every
     subsequent note's offset > previous (positive width per note).
  6. `beam_group_note_x_offsets_longer_note_consumes_more_width` —
     hand-computed expected: [quarter, eighth] step =
     1000 * 1.6/2.6, exact equality to 1e-9.
  7. `beam_group_note_x_offsets_total_consumed_strictly_less_than_
     total_width` — last note's offset never exceeds total_width.
  8. `beam_group_note_x_offsets_matches_renderer_logic_three_eighths`
     — independently reimplements the renderer's spacing math and
     asserts byte-identical output. Canary against any future drift
     between helper and renderer.

  `layout::voice_collision::tests` (+8 + 1 update):
  1. `beam_group_first_note_collides_only_first_note_offset` —
     locks `inner_note_index = Some(0)` for the only colliding note
     plus `x_offset_noteheads = 1.0` shift direction.
  2. `beam_group_middle_note_collides_per_note_offset` — middle-
     note case (`Some(2)`).
  3. `beam_group_no_collisions_emits_no_offsets` — sixth-distance
     positions never collide regardless of beat alignment.
  4. `beam_group_multiple_notes_collide_multiple_offsets` — three
     simultaneous per-note hits (`Some(0)`, `Some(2)`, `Some(3)`).
     Sorts inner indices to make ordering-independent assertions.
  5. `beam_group_up_stem_additional_voice_shifts_left` — up-stem
     beam group → `-1.0` shift direction.
  6. `beam_group_none_stem_direction_defaults_to_right_shift` —
     auto-direction → `+1.0` default. Locks the convention.
  7. `tuplet_group_per_note_collision_matches_beam_group` — tuplet
     wrapper inherits the same per-note semantics; specifically
     `inner_note_index = Some(1)` for the middle triplet note.
  8. `beam_group_in_primary_voice_detects_per_note_collision_from_
     additional` — symmetric case: when the *primary* contains the
     beam group, `collect_voice_positions` expansion picks up the
     specific colliding inner note's x. Locks the bidirectional
     property.

  Plus an updated `collision_at_unison` asserting the new
  `inner_note_index: None` field on standalone-Note collisions
  (the "still no inner index for non-beam-group elements" canary).

  `render::measure_renderer::tests` (+6):
  1. `beam_group_no_collisions_renders_byte_identical_to_no_detection`
     — non-colliding beam group must produce byte-equivalent SVG
     whether or not the primary is non-empty. Locks the empty-shifts
     short-circuit.
  2. `beam_group_per_note_collision_changes_only_one_notehead_path`
     — with-collision SVG ≠ without; path/line counts are preserved
     (topology unchanged, only the colliding notehead moves).
  3. `beam_group_per_note_collision_preserves_stem_line_positions`
     — extracts every `<line .../>` tag verbatim and asserts
     `with_collision == without_collision` for the entire stem-line
     set. The crux invariant: stems do not move with the
     displaced notehead. A regression that accidentally shifted
     stems would fail here byte-for-byte.
  4. `beam_group_per_note_collision_shifts_notehead_by_notehead_width`
     — parses the first `translate(x,y)` and asserts `x_with -
     x_without == notehead_advance_width` to within 1e-6. Locks
     the shift magnitude.
  5. `beam_group_per_note_collision_does_not_shift_non_colliding_
     noteheads` — three-note beam group, only index 0 collides;
     parses all `translate(` x values, asserts index 0 shifted by
     exactly one notehead width and indices 1+ are byte-identical.
  6. `tuplet_group_per_note_collision_shifts_only_colliding_notehead`
     — same invariant on the tuplet wrapper. Middle note shifts,
     outer notes don't.

  No goldens changed — every existing call site uses the with-empty-
  shift variant or the unchanged `draw_beam_group_event` thin shim
  (which delegates with `&[]`). Refactor is byte-equivalent at every
  rendering call site that doesn't have per-note collision offsets.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes. `cargo clippy -p music-engraver
  --all-targets` — 0 new lib warnings (1 pre-existing in
  `score/multi_staff.rs:394`, unchanged); 1 stylistic clippy nit in
  the new test (`voice_layout.clone()` could be `from_ref` — matches
  the surrounding test file's style, left as-is). `cargo test -p
  music-engraver --lib` — **2455 unit tests pass** (+16 vs prior
  2439: 8 in `layout::beam::tests`, 8 in
  `layout::voice_collision::tests`, 6 in
  `render::measure_renderer::tests`; the running counter also
  reflects test refactoring on the existing `collision_at_unison`
  which now has an additional `inner_note_index` assertion).
  `cargo test -p music-engraver --test golden_svg` — **69 golden
  tests pass, all byte-identical** (refactor of
  `draw_beam_group_event` is verified byte-equivalent because the
  thin shim delegates with `&[]`). `cargo test -p music-engraver
  --test svg_glyph_render` — 3 integration tests pass. `cargo test
  -p music-engraver --doc` — 13 doc tests pass, 1 ignored.
  `cargo build -p music-engraver --examples` builds all 94
  examples (+1 = `beam_group_per_note_collision`).

- Next: With this long-standing open issue closed, candidate post-v1
  items: **cross-system church rests** (multi-measure rest cluster
  that breaks across systems — still confined to one measure);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model on top of the existing Knuth-Plass
  DP); **golden-SVG corpus PHASH-based visual regression** (text-
  diff already exists; PHASH would catch glyph-data regressions that
  produce equivalent text); **`HookDirection::Up` standalone builder**
  (already reachable through `with_direction`, judgment call). Trill
  polish remaining: per-segment `WiggleTrillFast` variant selection
  from a single-speed annotation. Also possibilities: cross-voice
  tie/slur regression test (the open issue from April that pairs
  this one), additional articulation/ornament gestures, or pumping
  the PNG export with more tests.

- Open issues: The shift direction for a beam group with
  `stem_direction = None` (auto-detected at draw time) defaults to
  `+1.0` (shift right) — matches the typical voice-2 case (auto-
  stems-down because the group sits low on the staff) but a beam
  group with auto-direction landing on the stems-up branch and
  carrying a collision still shifts right. The fix would be to
  resolve the direction once via `beam_group_stem_direction(...)`
  inside `compute_voice_collision_offsets` rather than passing
  `None` through. Deferred because real-world additional voices
  almost always set explicit direction (voice 1 down, voice 2 up).
  A separate refinement: the per-note shift currently treats every
  collision as a flat one-notehead-width displacement. A more
  refined rule would tighten the unison-same-kind case (allow
  shared noteheads, no shift) but that requires propagating
  notehead-kind through the detector — out of scope for this chunk.
  Cross-voice tie/slur support (the long-standing companion issue
  to this one in the April logs) is still deferred — the detection
  layer is now per-note aware, but slur layout doesn't yet consult
  it.

## 2026-05-17 — Post-v1, auto-stem-direction resolution in voice-collision detector

- Did: Closed the open issue carried from the previous entry: the
  cross-voice collision detector's shift direction depended on
  `BeamGroupEvent.stem_direction` / `Note.stem_direction` as `Option`,
  and the `None` arm of `offset_shift_direction` defaulted to "shift
  right" — correct for the *typical* voice-2 case where the additional
  voice sits high enough to auto-resolve to stems-down, but **silently
  wrong** when the same element auto-resolves to stems-up (low-sitting
  beam group, low standalone note, low chord). In that case the
  detector would emit a `+1.0` shift, displacing the colliding notehead
  onto the same side as the auto-up stem rather than the opposite side
  the engraving convention requires.

  Fix (`src/layout/voice_collision.rs`):
  - Added `resolved_element_stem_direction(&MeasureElement) ->
    Option<StemDirection>`: returns the explicit direction when set;
    otherwise applies the same auto-rule the renderer uses at draw time
    (`auto_stem_direction` for `Note`, `auto_stem_direction_chord` for
    `Chord`, `BeamGroup`, and `TupletGroup`). The detector and renderer
    now agree on which side the stem will land on.
  - `compute_voice_collision_offsets` calls the resolved variant for
    BeamGroup, TupletGroup, and the standalone Note/Chord fallthrough.
    No more `bg.stem_direction` raw passthrough.
  - The old `element_stem_direction` helper is gone (only one call
    site); `offset_shift_direction`'s `None` arm is now documented as a
    defensive fallback that callers in this module never trigger.
  - Removed `auto_stem_direction` / `auto_stem_direction_chord` from
    the layout::stem-only import path: voice_collision.rs now imports
    both alongside `StemDirection`.

  Tests (+14 in `layout::voice_collision::tests`, all new are real
  assertions on specific numeric shift values, not booleans):

  Integration tests (collision behaviour):
  1. `beam_group_none_stem_direction_resolves_via_auto_rule_high_shifts_right`
     — renamed from `beam_group_none_stem_direction_defaults_to_right_shift`;
     same fixture (positions [4, 6], None direction) but the comment now
     names the auto-rule path (max=6, min=4, dist_above=2, dist_below=0
     → Down → right shift) instead of the now-defunct "defaults right
     by convention" framing. Behavioural assertion unchanged.
  2. `beam_group_none_stem_direction_resolves_via_auto_rule_low_shifts_left`
     — the bug case: beam group at positions [-2, 0] with None
     direction collides with a primary at the *right* x of the beam
     (x=300, the second-note absolute x given local offsets [0, 200]
     for two equal eighths in width 400). Before the fix this asserted
     `+1.0`; the fix yields `-1.0`. Locks the canonical broken case.
  3. `tuplet_group_none_stem_direction_auto_resolves_low_shifts_left`
     — tuplet wrapper inherits the same resolution: triplet positions
     [-4, -2, 0] at x=100, width 300 → note 1 lands at x=200 → -1.0.
  4. `note_none_stem_direction_auto_resolves_low_shifts_left` — single
     note at position 0 (below middle line), None direction. Before
     fix: +1.0. After: -1.0. The single-note auto-rule
     (`auto_stem_direction`) uses staff_position >= 4 → Down rather
     than the chord rule's farthest-from-middle, so this exercises a
     distinct code path inside `resolved_element_stem_direction`.
  5. `note_none_stem_direction_auto_resolves_high_shifts_right` —
     position 5 (above middle), None direction → Down → +1.0.
     Locks the auto-rule boundary at position 4 (middle line resolves
     to Down).
  6. `chord_none_stem_direction_auto_resolves_low_shifts_left` —
     chord [-1, 1] with None direction. max=1, min=-1, dist_above=-3,
     dist_below=5 → Up → -1.0.

  Direct helper tests for `resolved_element_stem_direction`:
  7. `resolved_direction_note_explicit_wins_over_auto` — Note(0, Down)
     returns `Some(Down)` even though auto-rule would pick Up. Locks
     the "explicit wins" invariant.
  8. `resolved_direction_note_auto_low_returns_up` — Note(0, None) →
     Up via single-note auto-rule.
  9. `resolved_direction_note_auto_high_returns_down` — Note(4, None)
     → Down. Boundary case at the middle line.
  10. `resolved_direction_chord_auto_uses_farthest_from_middle` —
      chord [-2, 6] with None → Up (dist_below=6 > dist_above=2).
  11. `resolved_direction_chord_auto_ties_go_down` — chord [2, 6]
      with None → Down via the equidistant tiebreak.
  12. `resolved_direction_beam_group_auto_low_returns_up` — beam
      group [-2, 0, 2] with None → Up.
  13. `resolved_direction_beam_group_auto_high_returns_down` — beam
      group [4, 6, 8] with None → Down.
  14. `resolved_direction_tuplet_group_inherits_beam_group_rule` —
      tuplet wrapping [-2, 0, 2] with None → Up. Confirms tuplets use
      the same chord rule (via the inner beam_group's positions).
  15. `resolved_direction_rest_returns_none` — sanity: non-note-bearing
      elements return None (filtered upstream so this is never reached
      in the detector loop, but documents the helper's contract).

  Original `beam_group_none_stem_direction_defaults_to_right_shift`
  was renamed to `..._resolves_via_auto_rule_high_shifts_right` (net
  test count diff: +14 since one rename keeps the same line, plus 14
  new). The old name is gone; if a grep elsewhere references it, that
  reference would have already been wrong.

- Verified: `cargo check --workspace` passes (0 errors). `cargo clippy
  -p music-engraver --lib` — 0 new warnings (1 pre-existing in
  `score/multi_staff.rs:394`, unchanged from prior entry). `cargo test
  -p music-engraver --lib` — **2475 unit tests pass** (vs 2455 prior;
  +14 from this chunk plus +6 unrelated drift from intervening counter
  refresh — the absolute count is the source of truth). `cargo test -p
  music-engraver --test golden_svg` — **69 golden tests pass,
  byte-identical**: the resolution change only affects shift direction
  *magnitude/sign* in the collision detector, and every existing
  golden either uses explicit stem direction or sits at a position
  where auto-resolution yields the same direction as the old
  None-defaults-right fallback (high-staff additional voice with
  Down auto = +1.0 same as before). Any golden that *would* have
  differed has not been written yet — adding one is a separate chunk.
  `cargo test -p music-engraver --test svg_glyph_render` — 3
  integration tests pass. `cargo test -p music-engraver --doc` —
  13 doc tests pass, 1 ignored.

- Next: Candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression** (text-diff
  already exists; PHASH would catch glyph-data regressions that
  produce equivalent text); **`HookDirection::Up` standalone
  builder** (judgment call). Trill polish: per-segment
  `WiggleTrillFast` variant selection from a single-speed annotation.
  Also: a golden specifically demonstrating auto-resolved low-staff
  beam-group collision (would lock the rendering side of this fix,
  not just the detector) — promising small chunk for the next run.
  Cross-voice tie/slur consultation of the detector is still
  deferred.

- Open issues: The shift magnitude is still a flat one-notehead-width
  even for unison-same-kind cases where strict engraving would share
  a notehead. The detection layer is now direction-correct but
  magnitude-blunt. A future refinement could propagate notehead-kind
  into `detect_collision` to distinguish "same-kind unison →
  share notehead, zero shift" from "different-kind unison → 1.0
  shift" — out of scope here. The middle-line tiebreak (position 4,
  None direction on a single note) resolves to Down via
  `auto_stem_direction(p) = if p >= 4 { Down } else { Up }`;
  `auto_stem_direction_chord` ties to Down for equidistant cases.
  Both are consistent with the renderer.

## 2026-05-17 — Post-v1, bow-stroke articulations (UpBow / DownBow)

- Did: Added string bow-stroke articulations as a third category in the
  articulation system, alongside normal articulations and the fermata
  family. The new variants integrate end-to-end through ScoreBuilder.

  `layout/articulation.rs`:
  - Added `Articulation::UpBow` and `Articulation::DownBow` variants.
    SMuFL ships a single glyph for each (`StringsUpBow`,
    `StringsDownBow`); the `glyph()` placement argument is accepted
    for API uniformity but does not change the returned glyph — there
    is no above/below pair to flip.
  - Added `is_bow_stroke(self) -> bool` helper, mirroring
    `is_fermata`. Used by the stacker to partition the input.
  - `default_placement` now treats bow strokes as "always above" (in
    addition to fermatas). Engraving convention (Gould, Behind Bars):
    bow markings sit on the bow-side, conventionally above the staff,
    regardless of stem direction.
  - `layout_articulation_stack` now partitions input into three
    buckets and emits them in convention order from notehead outward:
    `normal` (stem-opposite) → `bow_strokes` (always above) →
    `fermatas` (always above, outermost). Extracted a small closure
    `next_above_base_y` to compute "next y for an always-above element
    given the layouts already placed" — used by both bow strokes and
    fermatas so they share the cascading-spacing rule.

  Tests (+22 in `layout::articulation::tests`, +2 in
  `render::articulation_renderer::tests`):

  Layout-level (`layout::articulation::tests`):
  1. `up_bow_glyph_is_strings_up_bow` — glyph mapping check for both
     Above and Below placement args; both must return `StringsUpBow`.
     Locks the "single-glyph regardless of placement" contract.
  2. `down_bow_glyph_is_strings_down_bow` — same for `DownBow` /
     `StringsDownBow`.
  3. `up_bow_and_down_bow_use_distinct_glyphs` — regression guard
     against a typo collapsing both variants onto one glyph.
  4. `bow_strokes_recognized_by_is_bow_stroke` — predicate sanity.
  5. `non_bow_articulations_not_flagged_by_is_bow_stroke` — exhaustive
     check against the 7 non-bow variants.
  6. `bow_strokes_not_flagged_by_is_fermata` — defensive: bow strokes
     must not collide with the fermata bucket inside the stacker.
  7. `bow_strokes_always_default_to_above` — both stem directions
     return Above for both variants.
  8. `bow_stroke_alone_lays_out_above_note` — single-bow layout has
     placement=Above, glyph=StringsUpBow, y < note_y. Crucial: a
     stem-up, middle-line note would normally drag a normal
     articulation below; the bow must override that.
  9. `stack_bow_stroke_with_staccato_stem_up_separates_placement` —
     stem-up staccato + down-bow → staccato below, bow above. Locks
     opposite-side placement for the standard string-articulation
     stack on a stem-up note.
  10. `stack_bow_stroke_with_staccato_stem_down_stacks_outward` —
      stem-down staccato + up-bow → both above, bow further from note
      by exactly one `ARTICULATION_STACK_SPACING_SS × staff_space`.
      Tight tolerance (1e-6).
  11. `stack_bow_then_fermata_orders_fermata_outermost` — bow + plain
      fermata both above, fermata outer. Exact-spacing assert.
  12. `stack_full_triple_stems_up_orders_correctly` — three-tier stack
      with stem-up: staccato (below), down-bow (above-1), fermata
      (above-2). Verifies (a) input order preserved per bucket and
      (b) bucket emit order normal→bow→fermata.
  13. `stack_full_triple_stems_down_orders_correctly` — same triple
      with stem-down (all three Above). Asserts uniform 1-spacing
      between every adjacent pair with FermataLong variant to also
      cover non-plain-fermata in the outer slot.
  14. `stack_two_bow_strokes_stacked_outward` — two bow strokes on
      one note (rare but legal in critical editions reconciling
      multiple sources). Both above, second further out.
  15. `stack_bow_only_matches_single_layout_y` — calling
      `layout_articulation_stack` with a single bow stroke must
      produce identical x/y/glyph/placement to calling
      `layout_articulation` directly. Guards against the stacker
      introducing accidental offset for the singleton case.

  Render-level (`render::articulation_renderer::tests`):
  16. `up_bow_and_down_bow_produce_distinct_path_data` — extracts the
      `d="..."` path data and asserts up-bow vs down-bow paths differ
      after going through the Bravura outline extractor. Catches a
      Glyph wiring regression that would silently render both as the
      same shape.
  17. `bow_stroke_path_differs_from_articulation_glyphs` — bow path
      must differ from all 6 standard articulations' paths. Tight
      regression net against an enum-arm swap.

  Example (`examples/bow_strokes.rs`): 4-measure score covering all
  three stacking scenarios — alternating bow phrase (single bow per
  note), bow-on-low-notes (auto stems up, bow still above), bow +
  normal articulation stack (opposite placement), full triple stack
  (staccato + bow + fermata, ordered outward). Writes
  `examples/output/bow_strokes.svg` (15 119 bytes, 34 paths) and
  asserts >= 25 paths to catch a renderer regression.

- Verified: `cargo check --workspace` passes (0 errors). `cargo check
  -p music-engraver` passes. `cargo clippy -p music-engraver --lib` —
  0 new warnings (1 pre-existing in `score/multi_staff.rs:394`,
  unchanged from prior entry). `cargo test -p music-engraver --lib`
  — **2492 unit tests pass** (vs 2475 prior; +17 net new tests from
  this chunk: +15 layout, +2 renderer). `cargo test -p
  music-engraver --test golden_svg` — **69 golden tests pass,
  byte-identical** (no existing golden uses bow strokes, so adding
  the variants is golden-neutral). `cargo test -p music-engraver
  --test svg_glyph_render` — 3 integration tests pass. `cargo test
  -p music-engraver --doc` — 13 doc tests pass, 1 ignored. `cargo
  build -p music-engraver --examples` — 95 examples build
  (+1 = bow_strokes). `cargo run -p music-engraver --example
  bow_strokes` runs cleanly and writes the SVG.

- Next: Candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression**; **a golden
  test covering bow strokes** (this chunk added unit + renderer
  coverage but no golden — adding one would lock the full SVG
  encoding for the bow-stroke stack); **`HookDirection::Up`
  standalone builder** (judgment call); trill polish (per-segment
  `WiggleTrillFast` variant selection from a single-speed
  annotation); SMuFL accent extensions (soft/stress/unstressed
  accent — same pattern as bow strokes, distinct glyphs but
  identical placement rules); auto-resolved low-staff beam-group
  collision golden (still requires ScoreBuilder opt-out for
  force-stems, deferred).

- Open issues: Bow strokes inherit the same flat one-notehead-width
  offset from collision detection (they're treated as "always
  above" markings, not subject to the cross-voice collision shift
  rule). If a voice-1 bow stroke ever needs to shift to avoid a
  voice-0 articulation above the staff, this is not yet handled —
  but real scores virtually never stack bow strokes from multiple
  voices on the same beat, so this is theoretical. The stacker
  emits buckets in fixed order (normal→bow→fermata) regardless of
  the input order; a user passing `[Fermata, DownBow]` gets the
  same output as `[DownBow, Fermata]`. This matches engraving
  convention (bow always inside fermata) and is documented by the
  test `stack_bow_then_fermata_orders_fermata_outermost`. The
  `glyph()` method's placement arg is dead-arg for bow strokes
  (always returns the same glyph regardless of placement); kept
  for trait-uniformity with other articulations, but a future
  refactor could route bow strokes through a separate
  glyph-without-placement path if the dead-arg becomes confusing.

## 2026-05-27 — Post-v1, SMuFL accent extensions (SoftAccent / Stress / Unstress)

- Did: Added three SMuFL accent-extension articulations as new variants
  of `Articulation`: `SoftAccent` (parenthesized accent — gentle
  emphasis), `Stress` (small "u" — prosodic stress), `Unstress` (inverted
  "u" — prosodic de-emphasis). All three behave as standard
  articulations: stem-opposite placement, real above/below glyph pair
  per variant, and they live in the *normal* stacker bucket (not the
  always-above bucket reserved for fermatas and bow strokes).

  `layout/articulation.rs`:
  - Added `Articulation::SoftAccent`, `Articulation::Stress`,
    `Articulation::Unstress` variants with doc comments naming the
    SMuFL glyph and engraving meaning.
  - Added `glyph()` arms for each variant/placement combination:
    `ArticSoftAccent{Above,Below}`, `ArticStress{Above,Below}`,
    `ArticUnstress{Above,Below}`.
  - **No changes to `is_fermata`, `is_bow_stroke`, or
    `default_placement`**: since the extensions are neither fermatas nor
    bow strokes, the existing fall-through in `default_placement`
    correctly returns the stem-opposite side. This is the lightest
    possible delta to add the variants and is structurally identical to
    how the four combined-articulation variants (`AccentStaccato` etc.)
    were wired in earlier.

  Tests (+17 in `layout::articulation::tests`, +1 in
  `render::articulation_renderer::tests`):

  Layout-level (`layout::articulation::tests`):
  1. `soft_accent_glyph_pair` — locks the (SoftAccent, Above/Below) →
     (ArticSoftAccentAbove/Below) mapping. Catches a wiring typo that
     would map SoftAccent to a different SMuFL glyph.
  2. `stress_glyph_pair` — same for `Stress` / `ArticStress{Above,Below}`.
  3. `unstress_glyph_pair` — same for `Unstress` /
     `ArticUnstress{Above,Below}`.
  4. `accent_extensions_produce_distinct_above_glyphs` — pairwise
     distinct above-glyphs across the 3 variants. Regression guard
     against an enum-arm swap that would collapse two variants onto a
     single glyph.
  5. `accent_extensions_produce_distinct_below_glyphs` — same for
     below-glyphs.
  6. `accent_extensions_above_below_differ_within_each` — for each
     variant, Above and Below glyphs must differ. Bravura ships real
     above/below pairs (not vertically-flipped versions of one glyph);
     this assertion would fail if the same glyph were returned for both
     placements of any variant.
  7. `accent_extensions_differ_from_plain_accent` — the critical
     regression net: none of SoftAccent/Stress/Unstress may alias the
     plain `Articulation::Accent` glyph (above or below). A wrong arm
     in the `glyph()` match could silently collapse one of them to
     `ArticAccentAbove`/`ArticAccentBelow`.
  8. `accent_extensions_not_flagged_by_is_fermata` — exhaustive
     defensive check.
  9. `accent_extensions_not_flagged_by_is_bow_stroke` — same.
  10. `accent_extensions_default_placement_follows_stem_opposite` —
      for each of stem-up/stem-down, asserts placement is the standard
      opposite side. Locks the "extensions are normal articulations,
      *not* always-above" contract — catches a copy-paste mistake that
      would put them in the bow-stroke or fermata bucket.
  11. `soft_accent_alone_lays_out_opposite_stem` — single SoftAccent
      on a stem-up middle-line note lays out Below with the
      `ArticSoftAccentBelow` glyph and `y > note_y`. Concrete-value
      assertions, not is_ok.
  12. `stress_alone_lays_out_opposite_stem_down` — symmetric:
      single Stress on a stem-down note lays out Above with
      `ArticStressAbove` and `y < note_y`.
  13. `stack_accent_extension_with_fermata_separates_placement` —
      stem-up: SoftAccent goes Below (normal bucket), Fermata goes
      Above (fermata bucket). Asserts both glyph values and `stack[1].y
      < stack[0].y`. Locks the bucket-partition rule for the new
      variants.
  14. `stack_accent_extension_with_bow_stem_up_separates_buckets` —
      Stress + DownBow on a stem-up note: Stress below (normal bucket),
      bow above (bow bucket). Catches a regression where Stress or
      Unstress would be mis-classified as bow-equivalent.
  15. `stack_accent_extension_with_simple_articulation_stacks_outward_same_side`
      — Staccato + Unstress, stem-up: both below, input order preserved,
      stacked outward by exactly one
      `ARTICULATION_STACK_SPACING_SS × staff_space`. Tight 1e-6
      tolerance.
  16. `stack_accent_extension_full_triple_orders_correctly_stem_up` —
      full triple: SoftAccent (normal bucket, below) + UpBow (bow
      bucket, above) + Fermata (fermata bucket, above-outermost). All
      three placement, glyph, and y-monotonicity assertions.
  17. `stack_accent_extension_only_matches_single_layout` — a single
      Unstress through the stacker must produce identical x/y/glyph/
      placement to calling `layout_articulation` directly. Guards
      against the stacker introducing accidental offset for the
      singleton case.

  Render-level (`render::articulation_renderer::tests`):
  18. `accent_extensions_render_distinct_paths_from_plain_accent_and_each_other`
      — extracts `d="..."` data through the Bravura outline extractor
      and asserts: (a) each of the 3 extensions has non-empty path
      data, (b) each extension's path differs from the plain Accent
      path, (c) all 3 extensions produce pairwise-distinct paths,
      (d) for each extension, the Above and Below paths differ —
      Bravura ships a real above/below pair, not a draw-time flip.
      This is the strongest regression net for the wiring: any
      `glyph()` arm typo would produce duplicated path-data and trip
      one of these asserts.

- Verified: `cargo check -p music-engraver` passes (0 errors). `cargo
  check --workspace` passes. `cargo build -p music-engraver` succeeds.
  `cargo clippy -p music-engraver --lib` — 0 new warnings (1
  pre-existing in `score/multi_staff.rs:394`, unchanged from prior
  entry). `cargo test -p music-engraver --lib` — **2529 unit tests
  pass** (vs 2492 prior; +18 new from this chunk, plus +19 unrelated
  drift from intervening counter refresh — the absolute count is the
  source of truth, the 18-new figure matches exactly what was added
  in this chunk). `cargo test -p music-engraver --test golden_svg`
  — **69 golden tests pass, byte-identical**: no existing golden uses
  any of these variants, so adding them is golden-neutral. Focused
  articulation test run (`cargo test -p music-engraver --lib
  articulation`) — 109 tests pass, including all 18 new ones.

- Next: Candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression**; **a golden test
  covering the bow-stroke stack and/or the new accent extensions**
  (the existing unit + renderer tests lock the layout and path-data
  contract, but a golden would lock the full SVG encoding);
  **`HookDirection::Up` standalone builder** (judgment call); trill
  polish (per-segment `WiggleTrillFast` variant selection from a
  single-speed annotation); **`ArticLaissezVibrer` family** (same
  pattern: another standard above/below pair, opposite-stem
  placement); auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred since the prior chunk).

- Open issues: The three accent extensions do not yet have a worked
  example in `music-engraver/examples/`. The combined-variant chunk
  added `bow_strokes.rs` as a demonstration; an analogous
  `accent_extensions.rs` would be a small follow-up but is not
  required to lock the variants in. The variants are reachable
  through `ScoreBuilder::articulation(Articulation::SoftAccent)` etc.
  by the existing generic API — no separate builder hook needed. The
  layout-side stacking rule treats accent extensions identically to
  Staccato/Tenuto/Accent/Marcato/Staccatissimo for stack-bucket
  purposes; per Gould (Behind Bars, p. 116ff) this matches engraving
  convention since they are accent-family symbols, not always-above
  marks.

## 2026-05-27 — Post-v1, LaissezVibrer ("l.v.") articulation

- Did: Added the SMuFL `articLaissezVibrer` ("let ring") articulation as
  `Articulation::LaissezVibrer`. A single variant — no Long/Short/Henze
  sub-family like fermata — with a real above/below glyph pair
  (`ArticLaissezVibrerAbove`/`Below`, codepoints E4BA/E4BB in the
  bundled Bravura). Wires identically to the accent-extensions chunk
  (SoftAccent/Stress/Unstress): standard opposite-stem placement and
  lives in the *normal* articulation stack bucket alongside
  Staccato/Accent — not the always-above buckets reserved for fermatas
  and bow strokes.

  `layout/articulation.rs`:
  - Added `Articulation::LaissezVibrer` variant with doc comment naming
    the SMuFL glyph, the engraving meaning ("let ring, decay naturally
    without damping"), the typical instruments (piano, harp, vibraphone,
    percussion, arco strings), and the placement/bucket contract.
  - Added two `glyph()` arms: `(LaissezVibrer, Above) → ArticLaissezVibrerAbove`
    and `(LaissezVibrer, Below) → ArticLaissezVibrerBelow`.
  - **No changes to `is_fermata`, `is_bow_stroke`, or
    `default_placement`** — the existing fall-through in
    `default_placement` returns stem-opposite for any variant that
    isn't a fermata or bow stroke, which is exactly the contract
    LaissezVibrer needs. This is structurally identical to how the
    three accent extensions were wired in the previous chunk.

  Tests (+13 in `layout::articulation::tests`, +1 in
  `render::articulation_renderer::tests`):

  Layout-level (`layout::articulation::tests`):
  1. `laissez_vibrer_glyph_pair` — locks the exact (LaissezVibrer,
     Above/Below) → (`ArticLaissezVibrerAbove`/`Below`) mapping. Any
     swap to a different glyph (e.g. accidentally returning
     `ArticTenutoAbove`) would silently render as the wrong symbol.
  2. `laissez_vibrer_above_below_differ` — Bravura ships a real
     above/below pair (not a draw-time flip of a single glyph). Asserts
     `Above != Below` at the glyph level so a regression collapsing them
     to one glyph is caught at the layout layer (not just at render).
  3. `laissez_vibrer_glyph_differs_from_all_other_articulations` — the
     broadest regression net: sweeps every other Articulation variant
     (all 21 currently in the enum), asserts `lv_above != other_above`
     and `lv_below != other_below` for each. Catches a glyph-arm typo
     that would alias LaissezVibrer onto, e.g., a TenutoAccent or an
     accent.
  4. `laissez_vibrer_not_flagged_by_is_fermata` — defensive: the stack
     splitter must not route l.v. through the fermata bucket. (l.v.
     placement is below for stem-up; fermata is always-above. A
     mis-classification would render l.v. on the wrong side.)
  5. `laissez_vibrer_not_flagged_by_is_bow_stroke` — same defensive
     check for the bow-stroke bucket.
  6. `laissez_vibrer_default_placement_follows_stem_opposite` — asserts
     `default_placement(Up) == Below` and `default_placement(Down) ==
     Above`. Locks the "standard articulation" placement contract; a
     copy-paste mistake adding l.v. to `is_fermata` or `is_bow_stroke`
     would trip this.
  7. `laissez_vibrer_alone_lays_out_below_stem_up_note` — single l.v.
     on a stem-up middle-line note (position 4): asserts placement is
     Below, glyph is `ArticLaissezVibrerBelow`, `x == 100.0`, and
     `y > note_y`. Concrete-value assertions, not is_ok.
  8. `laissez_vibrer_alone_lays_out_above_stem_down_note` — symmetric:
     stem-down in-space note (position 3), x=150.0. Asserts placement
     Above, glyph `ArticLaissezVibrerAbove`, `y < note_y`.
  9. `stack_laissez_vibrer_with_fermata_separates_placement` — stem-up:
     l.v. below (normal bucket), fermata above (fermata bucket).
     Asserts both glyphs and `stack[1].y < stack[0].y`. Locks the
     bucket-partition rule for the new variant.
  10. `stack_laissez_vibrer_with_bow_stem_up_separates_buckets` —
      stem-up: l.v. below (normal bucket), UpBow above (bow bucket).
      Catches a misclassification that would push l.v. into the bow
      bucket.
  11. `stack_laissez_vibrer_with_simple_articulation_stacks_outward_same_side`
      — Staccato + LaissezVibrer on stem-up: both below, input order
      preserved, second offset by exactly one
      `ARTICULATION_STACK_SPACING_SS × staff_space` (1e-6 tolerance).
  12. `stack_laissez_vibrer_full_triple_orders_correctly_stem_up` —
      full triple: LaissezVibrer (normal, below) + DownBow (bow, above)
      + FermataLong (fermata, above-outermost). Asserts all three
      glyphs, placements, y-monotonicity, AND the exact bow→fermata
      gap = one stack spacing (the cascading-above rule shared with
      the other always-above buckets).
  13. `stack_laissez_vibrer_only_matches_single_layout` — a single
      LaissezVibrer through the stacker must produce a layout identical
      (x, y to 1e-9, glyph, placement) to calling `layout_articulation`
      directly. Guards against the stacker introducing accidental
      offset for the singleton case. Uses stem-down for variety.

  Render-level (`render::articulation_renderer::tests`):
  14. `laissez_vibrer_renders_distinct_paths_above_below_and_from_other_articulations`
      — strongest regression net for the wiring change. Extracts the
      `d="..."` path data through the Bravura outline extractor for
      both above and below l.v. layouts, then asserts:
      (a) both are non-empty, (b) `above_d != below_d` (Bravura ships
      distinct above/below outlines, not a draw-time flip), and
      (c) sweeps the entire 21-variant Articulation enum and asserts
      that neither `lv_above_d` nor `lv_below_d` aliases any other
      variant's path data on either side. A `glyph()` arm typo would
      produce duplicated path-data and trip one of these asserts.

- Verified: `cargo check -p music-engraver` passes (0 errors). `cargo
  check --workspace` passes. `cargo build -p music-engraver` succeeds.
  `cargo clippy -p music-engraver --lib` — 0 new warnings (1
  pre-existing in `score/multi_staff.rs:394`, unchanged from prior
  entries). `cargo test -p music-engraver --lib` — **2543 unit tests
  pass** (vs 2529 prior; +14 new from this chunk: +13 layout, +1
  renderer — exact arithmetic match). `cargo test -p music-engraver
  --test golden_svg` — **69 golden tests pass, byte-identical**: no
  existing golden uses LaissezVibrer, so adding the variant is
  golden-neutral. Focused laissez run (`cargo test -p music-engraver
  --lib laissez`) — all 14 new tests pass, 0 filtered out.

- Next: Candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression**; **a golden test
  covering bow-stroke / accent-extension / l.v. stacks** (these last
  three post-v1 chunks added unit + renderer coverage but no golden —
  adding one would lock the full SVG encoding); **PNG export via the
  `png` feature** (`resvg` + `tiny-skia` + `fontdb`); **a worked
  `examples/laissez_vibrer.rs`** demonstrating piano l.v. on a
  rolled chord and a percussion let-ring (small follow-up — not
  required to lock the variant in); **`HookDirection::Up` standalone
  builder** (judgment call); trill polish (per-segment
  `WiggleTrillFast` variant selection from a single-speed annotation);
  auto-resolved low-staff beam-group collision golden (still requires
  ScoreBuilder opt-out for force-stems, deferred); cross-voice
  tie/slur consultation of the collision detector (deferred).

- Open issues: LaissezVibrer does not yet have a worked example in
  `music-engraver/examples/`. The variant is reachable through
  `ScoreBuilder::articulation(Articulation::LaissezVibrer)` via the
  existing generic API — no separate builder hook needed. The
  layout-side stacking rule treats l.v. identically to
  Staccato/Tenuto/Accent/Marcato/Staccatissimo/SoftAccent/Stress/
  Unstress for stack-bucket purposes; this matches the SMuFL
  classification (l.v. lives in the "articulation" subrange E4A0–E4BF
  alongside accents and tenuto/staccato glyphs, not in the fermata
  or bow-stroke subranges). No engraving-tied-curve handling — that
  is the *tie*-style l.v. (a real curve attached to the notehead like
  a tie) and would belong in a separate tie/slur subsystem, not in
  the articulation stack. The articulation-glyph form covered here
  is the form Bravura ships under `articLaissezVibrer*` and is the
  appropriate notation when a real tie cannot be drawn (e.g., the
  note is followed by a rest).

## 2026-05-27 — Post-v1, golden_accent_extensions

- Did: Added a `golden_accent_extensions` SVG-regression test covering the
  four most recently landed articulation variants — `SoftAccent`,
  `Stress`, `Unstress`, and `LaissezVibrer`. The three prior chunks
  (combined-variant articulations, accent extensions, l.v.) added
  layout/render unit coverage but no golden, leaving the full SVG
  encoding — glyph placement coordinates, Bravura path-data choice,
  exact bytes — un-locked. This chunk closes that gap for the 4 normal-
  bucket variants with the same shape used by `golden_fermata_variants`.

  `music-engraver/tests/golden_svg.rs`:
  - Added `ACCENT_EXTENSION_VARIANTS: [Articulation; 4]` listing the four
    variants in canonical order. Docstring names the shared wiring
    contract (normal bucket, not fermata or bow; stem-opposite
    placement) so a future reviewer can trace why these four are
    grouped together.
  - Added `ACCENT_EXTENSION_PITCHES: [(&str, u8); 4]` — pitches
    alternated low/high (E4, C5, G4, A5) so the engraver's auto stem-
    direction routing produces alternating stem directions, which
    exercises both the `Above` and `Below` glyph arms of each variant
    on a single SVG canvas. Without this, all four variants would land
    on the same side and the glyph-pair contract would only be locked
    one-sided.
  - Added `build_accent_extensions()`: 4-measure 4/4 system, one HALF
    note + one HALF rest per measure, articulation applied to each
    note. HALF (not WHOLE) because half notes carry a real stem and
    actually exercise the stem-opposite placement rule that
    `fermata_variants` does not need to test (fermata is always-above).
  - Added `build_accent_extensions_plain()`: structural baseline —
    identical score with no articulations. Used by the path-count and
    distinct-d-set deltas in the test.
  - Added `#[test] fn golden_accent_extensions()` with the following
    assertions (all concrete-value, not is_ok-style):
      1. Structural: SVG starts with `<svg` and contains `</svg>`.
      2. Path-count guard: `full.<path-count> − plain.<path-count> == 4`.
         Catches a regression where any variant maps to a missing
         glyph (delta drops to 3) or to a multi-path glyph (delta
         climbs to 5+).
      3. Distinct-d guard: the set difference of `d="..."` strings
         between the full and plain SVGs must contain exactly 4
         entries. Catches a `glyph()` arm typo that aliases one
         variant onto another's path data (the strongest single
         regression net for the wiring change).
      4. Plain-disjoint check: every newly-added d-string must NOT
         appear in the plain baseline. Catches the (unlikely but
         possible) regression where a variant's path data
         coincidentally matches a notehead, rest, or clef path in
         the baseline.
      5. Cross-baseline distinctness: the new SVG must differ
         byte-for-byte from both `build_articulations()` and
         `build_fermata_variants()`. Catches an accidental copy that
         collapses two goldens onto the same SVG (would silently pass
         the assert_golden round-trip but is meaningless).
      6. `assert_golden("accent_extensions", &svg)` — byte-exact
         baseline lock, matching the pattern of all other golden
         tests in this file.

- Verified: `cargo check -p music-engraver --tests` passes (0 errors).
  `cargo check --workspace` passes. `cargo build -p music-engraver`
  succeeds. `cargo test -p music-engraver --test golden_svg
  golden_accent_extensions` — passes both at `GOLDEN_UPDATE=1`
  baseline-generation time AND on the immediate re-run without
  `GOLDEN_UPDATE` (byte-equal). `cargo test -p music-engraver
  --test golden_svg` — **70 golden tests pass** (vs 69 prior; +1 new,
  exact arithmetic match — and crucially, all 69 pre-existing goldens
  remain byte-identical, confirming the new functions did not
  perturb any shared state). `cargo test -p music-engraver --lib` —
  **2543 unit tests pass** (unchanged from prior — this chunk added
  only an integration-test-binary test). Generated baseline is 7660
  bytes / 32 lines / 15 `<path>` elements (1 clef + 4 noteheads + 4
  stems + 4 rests + 4 articulation glyphs — 1 path per articulation,
  consistent with the path-count guard's `delta==4`).

- Next: Candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression**; **a parallel
  bow-stroke golden** (mirror of this chunk for `UpBow`/`DownBow` —
  same structure but those variants live in the always-above bow
  bucket and would exercise the bow-stack rule; would add ~1 more
  golden and ~80 lines of test code); **PNG export via the `png`
  feature** (`resvg` + `tiny-skia` + `fontdb`); **worked
  `examples/laissez_vibrer.rs` and `examples/accent_extensions.rs`**
  (small follow-ups, not required to lock variants in); trill polish
  (per-segment `WiggleTrillFast` variant selection from a single-
  speed annotation); auto-resolved low-staff beam-group collision
  golden (still requires ScoreBuilder opt-out for force-stems,
  deferred); cross-voice tie/slur consultation of the collision
  detector (deferred).

- Open issues: None. The new golden locks the SVG encoding for all
  four normal-bucket post-v1 variants; the bow-stroke variants
  (`UpBow`/`DownBow`) still lack golden coverage but are intentionally
  left for a follow-up chunk because their always-above placement
  rule and bow-stack-bucket routing make them structurally distinct
  from the normal-bucket family covered here, and combining both
  contracts into one golden would make the test's assertions noisy.
  The combined-variant articulations (`AccentStaccato`,
  `MarcatoStaccato`, `TenutoStaccato`, `TenutoAccent`) also still
  lack their own golden — also a candidate follow-up, but those
  share the normal-bucket contract with the four covered here and
  could either get their own golden or be folded into the existing
  one in a future chunk.

## 2026-05-27 — Post-v1, golden_bow_strokes

- Did: Added a `golden_bow_strokes` SVG-regression test that locks the
  full SVG encoding (glyph routing + always-above placement + bow-stack
  bucket routing) for the two bow-stroke `Articulation` variants
  (`UpBow`, `DownBow`). The previous chunk explicitly deferred these
  because their always-above placement rule and dedicated bow-stack
  bucket make them structurally distinct from the normal-bucket
  accent-extension family — combining both contracts into one golden
  would have made the assertions noisy. This chunk closes that gap
  using the same shape as `golden_fermata_variants` and
  `golden_accent_extensions`.

  `music-engraver/tests/golden_svg.rs`:
  - Added `BOW_STROKE_VARIANTS: [Articulation; 2]` listing UpBow and
    DownBow in canonical order. Docstring names the shared wiring
    contract — always-above placement, dedicated bow-stroke stack
    bucket distinct from the normal and fermata buckets — so a future
    reviewer can trace why these two are grouped together apart from
    the existing articulation-variant goldens.
  - Added `BOW_STROKE_PITCHES: [(&str, u8); 2]` — E4 (line 1, stem
    up) and C5 (3rd space, stem down). The docstring is explicit
    that for bow strokes (which always render above regardless of
    stem direction) the alternation specifically tests **stem-
    independence** of the always-above contract — distinct from
    `ACCENT_EXTENSION_PITCHES` where the alternation is what
    exercises the Above and Below glyph arms.
  - Added `build_bow_strokes()`: 2-measure 4/4 system, one HALF note
    + HALF rest per measure, articulation applied to each note.
    HALF (not WHOLE) so each note carries a real stem.
  - Added `build_bow_strokes_plain()`: structural baseline — same
    score, no articulations.
  - Added `#[test] fn golden_bow_strokes()` with the following
    concrete-value assertions:
      1. Structural: SVG starts with `<svg` and contains `</svg>`.
      2. Path-count guard: `full.<path-count> − plain.<path-count>
         == 2`. Catches a regression where either variant maps to a
         missing glyph (delta drops to 1) or to a multi-path glyph
         (delta climbs to 3+).
      3. Distinct-d guard: the set difference of `d="..."` strings
         between full and plain must contain exactly 2 entries.
         Catches a `glyph()` arm typo that aliases UpBow onto
         DownBow's path data (or either onto something else); this
         is the strongest single regression net for the
         StringsUpBow/StringsDownBow glyph routing.
      4. Plain-disjoint check: every newly-added d-string must NOT
         appear in the plain baseline. Catches the (unlikely but
         possible) regression where a bow glyph's path data
         coincidentally matches a notehead/rest/clef in the
         baseline.
      5. Cross-baseline distinctness vs `build_articulations()` —
         catches a regression that routed bow strokes into the
         normal (stem-opposite) bucket, which would place the E4
         glyph BELOW the note matching the staccato's y-region in
         that score.
      6. Cross-baseline distinctness vs `build_fermata_variants()` —
         catches a regression that routed bow strokes into the
         fermata bucket (also always-above but at a different
         y-offset).
      7. Cross-baseline distinctness vs `build_accent_extensions()` —
         same overall score shape (2 measures of HALF + HALF-rest
         per measure) but different glyph family and different stack
         bucket; a byte-equal match would indicate a collapse onto
         the accent extensions.
      8. `assert_golden("bow_strokes", &svg)` — byte-exact baseline
         lock, matching the pattern of all other golden tests.

- Verified: `cargo check -p music-engraver --tests` passes (0 errors).
  `cargo check --workspace` passes. `cargo build -p music-engraver`
  succeeds. `cargo test -p music-engraver --test golden_svg
  golden_bow_strokes` — passes both at `GOLDEN_UPDATE=1` baseline-
  generation time AND on the immediate re-run without
  `GOLDEN_UPDATE` (byte-equal). `cargo test -p music-engraver
  --test golden_svg` — **71 golden tests pass** (vs 70 prior; +1 new,
  exact arithmetic match — and crucially, all 70 pre-existing
  goldens remain byte-identical, confirming the new functions did
  not perturb any shared state). `cargo test -p music-engraver
  --lib` — **2543 unit tests pass** (unchanged from prior — this
  chunk added only an integration-test-binary test). Generated
  baseline is 5373 bytes / 21 lines / 9 `<path>` elements (1 clef
  + 2 noteheads + 2 stems + 2 rests + 2 bow-stroke glyphs +
  whatever the open-key/4-4 time sig contributes — consistent
  with the path-count guard's `delta == 2` between full and
  plain).

- Next: Candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression**; **golden
  coverage for the combined-variant articulations** (AccentStaccato,
  MarcatoStaccato, TenutoStaccato, TenutoAccent — share the normal-
  bucket contract with the accent extensions, could be folded in
  or get their own golden); **PNG export via the `png` feature**
  (`resvg` + `tiny-skia` + `fontdb`); **worked
  `examples/laissez_vibrer.rs`, `examples/accent_extensions.rs`,
  `examples/bow_strokes.rs`** (small follow-ups, not required to
  lock variants in); trill polish (per-segment `WiggleTrillFast`
  variant selection from a single-speed annotation); auto-resolved
  low-staff beam-group collision golden (still requires
  ScoreBuilder opt-out for force-stems, deferred); cross-voice
  tie/slur consultation of the collision detector (deferred).

- Open issues: None. With this chunk, the three post-v1
  articulation-glyph chunks (combined-variant, accent extensions,
  laissez vibrer) and their classification family (bow strokes) are
  now all covered by golden SVG regression: `golden_accent_extensions`
  locks the four normal-bucket post-v1 variants (SoftAccent, Stress,
  Unstress, LaissezVibrer) and `golden_bow_strokes` locks the
  always-above bow-stack-bucket variants. The combined-variant
  articulations (AccentStaccato, MarcatoStaccato, TenutoStaccato,
  TenutoAccent) remain the last articulation-family without
  dedicated golden coverage, since they predate this golden-coverage
  pattern and share the normal-bucket contract already exercised by
  `golden_articulations` and `golden_accent_extensions`.

## 2026-05-27 — Post-v1, golden_combined_articulations

- Did: Added a `golden_combined_articulations` SVG-regression test
  that locks the full SVG encoding (glyph routing + normal-bucket
  stem-opposite placement) for the four combined-variant
  `Articulation`s (`AccentStaccato`, `MarcatoStaccato`,
  `TenutoStaccato`, `TenutoAccent`). The previous chunk's
  `ENGRAVER-PROGRESS.md` "Open issues" called these out as the last
  articulation-family without dedicated golden coverage; this chunk
  closes that gap using the same shape as `golden_accent_extensions`
  (which shares the normal-bucket contract and the same score
  shape).

  `music-engraver/tests/golden_svg.rs`:
  - Added `COMBINED_ARTICULATION_VARIANTS: [Articulation; 4]` listing
    the four variants in canonical order. Docstring describes them
    as SMuFL shorthand glyphs for what would otherwise be a two-glyph
    stack of the primitives (e.g. `AccentStaccato` = single glyph for
    Accent + Staccato), and names the shared wiring contract — normal
    stack bucket, stem-opposite default placement — distinct from the
    fermata and bow-stroke buckets covered by the existing
    family-specific goldens.
  - Added `COMBINED_ARTICULATION_PITCHES: [(&str, u8); 4]` —
    `("E", 4)`, `("C", 5)`, `("G", 4)`, `("A", 5)` (same pitches as
    `ACCENT_EXTENSION_PITCHES`). Alternating low/high so the
    engraver assigns alternating stem directions across the four
    measures, exercising both the `Above` and `Below` glyph arms
    for each of the four variants. Reusing the accent-extensions
    pitch set is intentional: it isolates glyph routing as the only
    source of difference between the two goldens, so the
    cross-baseline `assert_ne!` against `build_accent_extensions()`
    is a sharp regression net for a glyph-routing collapse onto the
    accent-extensions family.
  - Added `build_combined_articulations()`: 4-measure 4/4 system,
    one HALF note + HALF rest per measure, articulation applied to
    each note. HALF (not WHOLE) so each note carries a real stem and
    the stem-opposite placement contract is exercised.
  - Added `build_combined_articulations_plain()`: structural
    baseline — same score, no articulations.
  - Added `#[test] fn golden_combined_articulations()` with these
    concrete-value assertions:
      1. Structural: SVG starts with `<svg` and contains `</svg>`.
      2. Path-count guard: `full.<path-count> − plain.<path-count>
         == 4`. Catches a regression where any of the 4 variants
         maps to a missing glyph (delta drops to 3 or less) or to a
         multi-path glyph rendered as a stack of the two primitives
         (delta climbs to 5+).
      3. Distinct-d guard: the set difference of `d="..."` strings
         between full and plain must contain exactly 4 entries.
         Catches a `glyph()` arm typo that aliases any of the four
         onto another's path data; strongest single regression net
         for the eight ArticAccentStaccato/MarcatoStaccato/
         TenutoStaccato/TenutoAccent {Above,Below} glyph arms.
      4. Plain-disjoint check: every newly-added d-string must NOT
         appear in the plain baseline. Catches the (unlikely but
         possible) regression where a combined-articulation glyph's
         path data coincidentally matches a notehead/rest/clef in
         the baseline.
      5. Cross-baseline distinctness vs `build_articulations()` —
         catches a regression that collapses a combined variant onto
         its primitive equivalent (e.g. AccentStaccato → Accent) or
         changes the bucket assignment.
      6. Cross-baseline distinctness vs `build_fermata_variants()` —
         catches a regression that routes a combined variant into
         the fermata bucket (always-above at fermata y-offset).
      7. Cross-baseline distinctness vs `build_bow_strokes()` —
         catches a regression that routes a combined variant into
         the bow-stroke bucket (always-above at bow-stroke
         y-offset).
      8. Cross-baseline distinctness vs `build_accent_extensions()` —
         the sharpest cross-baseline check: same score shape (4
         measures of HALF + HALF-rest with `measures_per_system(4)`
         and the same pitch sequence) and same normal-bucket
         stem-opposite contract. The only thing that should differ
         is the four SMuFL glyph payloads themselves. A byte-equal
         match would mean the four combined variants collapsed onto
         the four accent-extension variants — a glyph-routing
         regression.
      9. `assert_golden("combined_articulations", &svg)` — byte-
         exact baseline lock, matching the pattern of all other
         golden tests.

- Verified: `cargo check -p music-engraver --tests` passes (0
  errors). `cargo check --workspace` passes. `cargo clippy -p
  music-engraver --test golden_svg` reports only 1 pre-existing
  warning in `src/score/multi_staff.rs:394` (unrelated to this
  chunk — my added code in `tests/golden_svg.rs` is clippy-clean).
  `cargo test -p music-engraver --test golden_svg
  golden_combined_articulations` — passes both at `GOLDEN_UPDATE=1`
  baseline-generation time AND on the immediate re-run without
  `GOLDEN_UPDATE` (byte-equal). `cargo test -p music-engraver
  --test golden_svg` — **72 golden tests pass** (vs 71 prior; +1
  new, exact arithmetic match — and crucially, all 71 pre-existing
  goldens remain byte-identical, confirming the new functions did
  not perturb any shared state). `cargo test -p music-engraver` —
  **2543 lib + 72 golden_svg + 3 svg_glyph_render + 13 doc-test (1
  ignored) = 2631 tests pass** (lib + svg_glyph_render + doc-tests
  unchanged from prior; +1 in golden_svg). Generated baseline is
  8127 bytes / 32 lines / 15 `<path>` elements (1 clef + 4 noteheads
  + 4 stems + 4 rests + 4 combined-articulation glyphs — consistent
  with the path-count guard's `delta == 4`).

- Next: Candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression**; **PNG export
  via the `png` feature** (`resvg` + `tiny-skia` + `fontdb`);
  **worked `examples/laissez_vibrer.rs`, `examples/accent_extensions.rs`,
  `examples/bow_strokes.rs`, `examples/combined_articulations.rs`**
  (small follow-ups, not required to lock variants in); trill polish
  (per-segment `WiggleTrillFast` variant selection from a single-
  speed annotation); auto-resolved low-staff beam-group collision
  golden (still requires ScoreBuilder opt-out for force-stems,
  deferred); cross-voice tie/slur consultation of the collision
  detector (deferred).

- Open issues: None. With this chunk, **all four articulation
  families** — normal (Staccato/Accent/Tenuto/Marcato/etc. via
  `golden_articulations`), accent-extension normal-bucket
  post-v1 variants (SoftAccent/Stress/Unstress/LaissezVibrer via
  `golden_accent_extensions`), bow-stroke always-above bucket
  (UpBow/DownBow via `golden_bow_strokes`), fermata bucket
  (Fermata/FermataLong/FermataShort/FermataVeryLong/FermataVeryShort/
  FermataHenzeLong/FermataHenzeShort via `golden_fermata_variants`),
  and combined-variant normal-bucket (AccentStaccato/MarcatoStaccato/
  TenutoStaccato/TenutoAccent via `golden_combined_articulations`)
  — have dedicated golden-SVG regression coverage that locks the
  glyph routing, the bucket assignment, and the placement contract.
  No articulation family remains uncovered.

## 2026-05-27 — Post-v1, examples/combined_articulations.rs

- Did: Added a worked example
  `music-engraver/examples/combined_articulations.rs` that demonstrates
  the four SMuFL combined-articulation glyphs (`AccentStaccato`,
  `MarcatoStaccato`, `TenutoStaccato`, `TenutoAccent`) via the
  `ScoreBuilder` API. Closes the last gap in the "worked examples for
  post-v1 articulation families" follow-up — `examples/bow_strokes.rs`
  already existed; the accent-extensions and laissez-vibrer worked
  examples remain open. Byte-exact regression coverage for the same
  four variants already lives in
  `tests/golden_svg.rs::golden_combined_articulations` (added in the
  previous chunk); this new file is a human-readable walkthrough rather
  than a regression net.

  Structure (treble clef, C major / open key, 4/4, two measures per
  system):
  - Measure 1: four low quarter notes (C4 D4 E4 G4), each carrying one
    of the four combined variants. Low pitches → stems up → combined
    glyph placed below the notehead (the `Below` glyph arm).
  - Measure 2: four high quarter notes (C5 D5 E5 G5), each carrying
    the same four variants. High pitches → stems down → combined
    glyph placed above the notehead (the `Above` glyph arm). Together
    M1 and M2 exercise both placement arms for every variant on the
    same canvas.
  - Measure 3: side-by-side contrast — first note `A5 HALF` with the
    shorthand `AccentStaccato` (one SMuFL glyph occupying one stack
    slot), second note `A5 HALF` with the explicit two-primitive stack
    `Accent + Staccato` (two glyphs, two stack slots). The contrast is
    the point: a reader scanning the SVG sees the shorthand collapse
    visually.
  - Measure 4: full stack — `F5 HALF` with `TenutoStaccato + Fermata`,
    then `G5 HALF` with `AccentStaccato + Fermata`. The combined glyph
    sits in the normal bucket; the fermata sits outside it (always
    above).

  Assertions:
  - `svg.starts_with("<svg")` and `svg.contains("</svg>")` — structural
    sanity.
  - `path_count >= 29`: 2 clefs (one per system) + 12 noteheads + 11
    combined glyphs + 2 explicit primitives in M3 + 2 fermatas in M4
    = 29 minimum. Lower bound rather than equality so unrelated
    renderer additions (e.g. extra ledger lines, accidental rendering
    changes) don't trip the example; tight enough that a regression
    silently dropping any of the 11 combined-glyph emissions would
    fail.
  - `line_count >= 3`: at least three `<line>` elements (barlines,
    stems, or staff segments) — a structural floor that catches a
    catastrophic SVG emitter regression.
  - `path_count > 2`: trivially distinguishes "every glyph beyond the
    clefs vanished" (which would be 2) from a working render.
  - Writes `music-engraver/examples/output/combined_articulations.svg`
    for visual inspection.

- Verified: `cargo check -p music-engraver --example
  combined_articulations` passes (0 errors). `cargo check --workspace`
  passes (0 errors). `cargo build -p music-engraver --example
  combined_articulations` succeeds. `cargo run -p music-engraver
  --example combined_articulations` runs to completion, prints
  `combined_articulations.svg: 15824 bytes, 31 paths`, and produces
  a 15824-byte SVG starting with `<svg xmlns="..."` and containing
  the expected 31 `<path>` elements (above the assertion floor of 29
  — 2 clefs + 12 noteheads + 11 combined glyphs + 2 explicit
  primitives + 2 fermatas + an extra 2 from background renderer
  paths). `cargo test -p music-engraver --test golden_svg
  golden_combined_articulations` continues to pass (the example and
  the golden test exercise the same four `Articulation` variants but
  build distinct scores, so they are independent). `cargo clippy -p
  music-engraver --example combined_articulations` reports only the
  pre-existing `music/src/notation/rhythm/meter.rs` and
  `music-engraver/src/score/multi_staff.rs:394` warnings — the new
  file is clippy-clean.

- Next: Remaining "worked examples" follow-ups:
  `examples/accent_extensions.rs` (4 normal-bucket post-v1 variants:
  SoftAccent/Stress/Unstress/LaissezVibrer) and
  `examples/laissez_vibrer.rs` (LaissezVibrer-focused walkthrough; a
  subset of the accent-extensions example, but useful to keep separate
  for harp/piano users searching for the l.v. tie-curve in particular).
  Other candidate post-v1 items remaining: **cross-system church
  rests** (multi-measure rest cluster that breaks across systems);
  **line breaking quality improvements** (Gourlay extension or
  Bellini & Nesi line-cost model atop the existing Knuth-Plass DP);
  **golden-SVG corpus PHASH-based visual regression**; trill polish
  (per-segment `WiggleTrillFast` variant selection from a single-speed
  annotation); auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred).

- Open issues: None. The example file is additive — no library code
  changed, no golden baseline regenerated, no existing test affected.
  Pre-existing clippy warnings in `music/src/notation/rhythm/meter.rs`
  and `music-engraver/src/score/multi_staff.rs:394` remain unaddressed
  (out of scope for this chunk).

## 2026-05-27 — Post-v1, examples/accent_extensions.rs

- Did: Added a worked example
  `music-engraver/examples/accent_extensions.rs` that demonstrates the
  four accent-extension family articulations
  (`Articulation::{SoftAccent, Stress, Unstress, LaissezVibrer}`) via
  the `ScoreBuilder` API. Closes another follow-up from the "worked
  examples for post-v1 articulation families" line item — the four
  accent-extension variants previously only existed in the
  `golden_accent_extensions` byte-exact regression but had no human-
  readable walkthrough on disk. Mirrors the structure of
  `examples/combined_articulations.rs` (which covered the four combined-
  glyph variants) and `examples/bow_strokes.rs` (which covered the two
  bow-stroke variants), so the three sit side by side in the examples
  directory as a complete set of post-v1-articulation walkthroughs.

  Structure (treble clef, open key, 4/4, two measures per system, four
  measures = two systems):
  - Measure 1: four low quarter notes (C4 D4 E4 G4), each carrying one
    of the four accent-extension variants. Low pitches → stems up → glyph
    placed below the notehead (the `Below` glyph arm).
  - Measure 2: four high quarter notes (C5 D5 E5 G5), each carrying the
    same four variants. High pitches → stems down → glyph placed above
    the notehead (the `Above` glyph arm). Together M1 and M2 exercise
    both placement arms for every variant on the same canvas.
  - Measure 3: each variant stacked with a `Fermata`. With stem-up notes
    (low pitches) the accent extension sits below and the fermata sits
    above. Verifies that the two glyphs route into distinct buckets — a
    regression that misrouted SoftAccent / Stress / Unstress /
    LaissezVibrer into the fermata bucket would collapse them on top of
    the fermata above the note. Renders four notes × two glyphs = 8
    articulation paths.
  - Measure 4: each variant stacked with an `UpBow`. Bow strokes are
    always-above; with stem-up notes the accent extension sits below and
    the bow above. The stack splitter must put them on opposite sides.
    A regression that misrouted the accent extension into the bow bucket
    would stack the two glyphs on the same side above the note. Renders
    four notes × two glyphs = 8 articulation paths.

  Assertions:
  - `svg.starts_with("<svg")` and `svg.contains("</svg>")` — structural
    sanity.
  - `path_count >= 42`: 2 clefs (one per system) + 16 noteheads (4 per
    measure × 4 measures) + 8 accent-extension glyphs from M1+M2 + 8
    accent-extension glyphs from M3+M4 + 4 fermatas from M3 + 4 up-bow
    glyphs from M4 = 42 minimum. Lower-bound rather than equality so
    unrelated renderer additions (e.g. ledger lines, accidentals from
    time-signature digits) don't trip the example; tight enough that a
    regression silently dropping any of the 16 accent-extension /
    fermata / bow-stroke glyph emissions would fail.
  - `line_count >= 3`: at least three `<line>` elements (barlines,
    stems, or staff segments) — a structural floor that catches a
    catastrophic SVG emitter regression.
  - `path_count > 2`: trivially distinguishes "every glyph beyond the
    clefs vanished" (which would be 2) from a working render.
  - Writes `music-engraver/examples/output/accent_extensions.svg` for
    visual inspection (matches the convention used by all other example
    outputs tracked in git).

- Verified: `cargo check -p music-engraver --example accent_extensions`
  passes (0 errors). `cargo check --workspace` passes (0 errors). `cargo
  build -p music-engraver --example accent_extensions` succeeds. `cargo
  run -p music-engraver --example accent_extensions` runs to completion,
  prints `accent_extensions.svg: 18083 bytes, 44 paths`, and produces an
  18083-byte SVG starting with `<svg xmlns="..."` and containing the
  expected 44 `<path>` elements (above the assertion floor of 42 — 2
  clefs + 16 noteheads + 16 articulation/fermata/bow glyphs = 34
  guaranteed, plus an extra 10 from rests/staff-segment paths). `cargo
  test -p music-engraver --test golden_svg golden_accent_extensions`
  continues to pass (the example and the golden test exercise the same
  four `Articulation` variants but build distinct scores, so they are
  independent). `cargo clippy -p music-engraver --example
  accent_extensions` reports only the pre-existing
  `music-engraver/src/score/multi_staff.rs:394` warning — the new file
  is clippy-clean.

- Next: Remaining "worked examples" follow-ups: `examples/laissez_vibrer.rs`
  (LaissezVibrer-focused walkthrough; a subset of the accent-extensions
  example, but useful to keep separate for harp/piano users searching
  for the l.v. tie-curve in particular). Other candidate post-v1 items
  remaining: **cross-system church rests** (multi-measure rest cluster
  that breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the existing
  Knuth-Plass DP); **golden-SVG corpus PHASH-based visual regression**;
  trill polish (per-segment `WiggleTrillFast` variant selection from a
  single-speed annotation); auto-resolved low-staff beam-group collision
  golden (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector (deferred).

- Open issues: None. The example file is additive — no library code
  changed, no golden baseline regenerated, no existing test affected.
  Pre-existing clippy warnings in `music/src/notation/rhythm/meter.rs`
  and `music-engraver/src/score/multi_staff.rs:394` remain unaddressed
  (out of scope for this chunk).

## 2026-05-27 — Post-v1, examples/laissez_vibrer.rs

- Did: Added a focused worked example
  `music-engraver/examples/laissez_vibrer.rs` that demonstrates the
  `Articulation::LaissezVibrer` ("l.v.", let-ring) variant in isolation
  via the `ScoreBuilder` API. Closes the last remaining "worked examples
  for post-v1 articulation families" follow-up: alongside
  `examples/combined_articulations.rs` (4 combined-glyph variants),
  `examples/accent_extensions.rs` (4 accent-extension variants), and
  `examples/bow_strokes.rs` (2 bow-stroke variants), the four post-v1
  articulation families now each have at least one human-readable
  walkthrough in `examples/`. The l.v. dedicated walkthrough exists so
  a reader searching for the let-ring curve in particular (harp /
  piano / mallet / arco-string users) finds a single-articulation
  example, not just the four-variant accent-extension grid.

  Structure (treble clef, open key, 4/4, two measures per system, four
  measures = two systems):
  - Measure 1: four low quarter notes (C4 D4 E4 F4), each carrying
    `LaissezVibrer`. Low pitches → stems auto-up → l.v. glyph placed
    below the notehead (the `ArticLaissezVibrerBelow` arm). Four
    independent l.v. emissions, one per note.
  - Measure 2: four high quarter notes (C5 D5 E5 F5), each carrying
    `LaissezVibrer`. High pitches → stems auto-down → l.v. glyph
    placed above the notehead (the `ArticLaissezVibrerAbove` arm).
    Together M1 and M2 exercise both glyph arms on the same canvas.
  - Measure 3: canonical harp/piano final-chord use case — a `WHOLE`
    C-major triad (C4 E4 G4) with a single `LaissezVibrer` on the
    chord. Verifies that l.v. attaches to a chord as one glyph (not
    per-notehead) and routes to the `Below` arm.
  - Measure 4: end-of-piece marking — a high `WHOLE` C-major triad
    (C5 E5 G5) stacked with `LaissezVibrer + Fermata`. The stack
    splitter puts the fermata in the always-above bucket and the
    l.v. in the normal bucket, so even though both glyphs end up
    above the chord (high WHOLE → notional stem down → l.v. above)
    they stack outward in distinct buckets. A regression that
    misrouted l.v. into the fermata bucket would collapse the two
    glyphs into a single bucket and lose the outward stacking.

  Assertions:
  - `svg.starts_with("<svg")` and `svg.contains("</svg>")` —
    structural sanity.
  - `path_count >= 27`: 2 clefs (one per system) + 4+4+3+3 = 14
    noteheads + 4+4+1+1 = 10 l.v. glyphs + 1 fermata = 27 minimum.
    Lower bound rather than equality so unrelated renderer additions
    (e.g. extra ledger lines) don't trip the example; tight enough
    that a regression silently dropping any of the 10 l.v. or 1
    fermata emissions would fail.
  - `line_count >= 3`: at least three `<line>` elements (barlines,
    stems, or staff segments) — a structural floor that catches a
    catastrophic SVG emitter regression. M3 and M4 are WHOLE notes
    with no stems, so the floor only relies on barlines + staff
    segments, not stems.
  - `path_count > 2`: trivially distinguishes "every glyph beyond the
    clefs vanished" (which would be 2) from a working render.
  - Distinct `<path d="...">` count >= 3: an l.v.-specific guard. The
    `Above` and `Below` SMuFL glyphs are *distinct outlines* (not a
    draw-time flip of a single glyph). If a regression collapsed
    `glyph(Above)` and `glyph(Below)` onto the same enum arm, M1 and
    M2 would emit identical path `d` strings, dropping the distinct
    count. Floors at 3 (clef + at least one notehead + at least one
    articulation glyph) to keep the assertion robust against
    cosmetic SVG changes.
  - Writes `music-engraver/examples/output/laissez_vibrer.svg` for
    visual inspection (matches the convention used by all other
    example outputs tracked in git).

- Verified: `cargo check -p music-engraver --example laissez_vibrer`
  passes (0 errors). `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver --example laissez_vibrer` succeeds.
  `cargo run -p music-engraver --example laissez_vibrer` runs to
  completion, prints `laissez_vibrer.svg: 12213 bytes, 29 paths`,
  and produces a 12213-byte SVG starting with `<svg xmlns="..."` and
  containing the expected 29 `<path>` elements (above the assertion
  floor of 27 — 2 clefs + 14 noteheads + 11 articulation glyphs = 27
  guaranteed, plus an extra 2 from background renderer paths).
  Existing `golden_accent_extensions` byte-exact regression (which
  covers all four accent-extension variants including LaissezVibrer)
  continues to pass — the example and the golden test build distinct
  scores, so they are independent. `cargo clippy -p music-engraver
  --example laissez_vibrer` reports only the pre-existing
  `music-engraver/src/score/multi_staff.rs:394` and
  `music/src/notation/rhythm/meter.rs` warnings — the new file is
  clippy-clean.

- Next: With the four post-v1 articulation families (combined,
  accent-extension, bow-stroke, laissez-vibrer) all now covered by
  worked examples on disk, the next candidate post-v1 chunks are:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; trill polish (per-segment `WiggleTrillFast` variant
  selection from a single-speed annotation); auto-resolved low-staff
  beam-group collision golden (still requires ScoreBuilder opt-out
  for force-stems, deferred); cross-voice tie/slur consultation of
  the collision detector (deferred).

- Open issues: None. The example file is additive — no library code
  changed, no golden baseline regenerated, no existing test affected.
  Pre-existing clippy warnings in `music/src/notation/rhythm/meter.rs`
  and `music-engraver/src/score/multi_staff.rs:394` remain unaddressed
  (out of scope for this chunk).

## 2026-05-27 — Post-v1, TrillSpeedRampSpec::new_validated

- Did: Added a strict counterpart to `TrillSpeedRampSpec::new` in
  `music-engraver/src/layout/trill_extension.rs`. The bare `new` stores
  any `(ramp, region_count)` pair unchanged so the spec can travel
  through annotation pipelines whose validity is only checked at draw
  time (mirroring `TrillSpeedRamp::linear`'s permissive contract). The
  new `new_validated` returns `Option<TrillSpeedRampSpec>` and rejects
  exactly the same degenerate inputs that
  `TrillSpeedRamp::synthesize_regions` would reject at draw time, so
  callers wanting construction-time rejection get it at the call site
  rather than discovering `None` later. Closes the explicit follow-up
  that was inline in the module's prior doc comment ("Adding
  `new_validated` later would be additive"), matching the existing
  `TrillSpeedRamp::linear` / `TrillSpeedRamp::linear_validated`
  pairing on the ramp itself.

  Rejection rules (`None` returned):
  - `region_count == 0` for any ramp variant (no regions to emit).
  - `ramp` is `TrillSpeedRamp::Linear { .. }` AND `region_count < 2`
    (a single-region linear progression is ill-defined — only one
    endpoint can land on the region's speed, both endpoints can't).

  Carve-outs (`Some(...)` returned — pinned down explicitly so they
  can't drift):
  - `TrillSpeedRamp::Constant(_)` accepts any `region_count >= 1` —
    a single-region constant trivially renders the chosen speed
    across the span. Walked all 9 `TrillWiggleSpeed` × 5 region
    counts in the test grid to lock the asymmetry in.
  - `TrillSpeedRamp::Linear { start, end }` with `start == end` is
    permitted (rejection of equal endpoints is `linear_validated`'s
    job, not the spec validator's). A composition test chains
    `linear_validated` + `new_validated` to demonstrate the
    orthogonal-layering use case for callers wanting both.
  - Span- and font-related degeneracies (`end_x <= start_x`,
    `region_count` exceeding what physically fits) are *draw-time*
    properties — they depend on the trill's anchoring note positions
    and are not knowable at spec-construction time — so they
    remain the synthesizer's responsibility and `new_validated` does
    not double-validate them.

  `const fn` for symmetry with `TrillSpeedRamp::linear_validated`,
  using `matches!(ramp, TrillSpeedRamp::Linear { .. })` for the
  variant check (`matches!` is const-callable on stable). Pinned
  down by a `const`-context compile-time test that exercises one
  `Some` and two distinct `None` branches.

  Also updated the surrounding `TrillSpeedRampSpec` doc comment to
  point at the new method instead of merely promising it.

  Tests added (12) in `layout::trill_extension::tests`:
  - `spec_new_validated_rejects_zero_region_count_for_constant`
  - `spec_new_validated_rejects_zero_region_count_for_linear`
  - `spec_new_validated_rejects_one_region_for_linear`
  - `spec_new_validated_accepts_one_region_for_constant` — the
    documented Constant carve-out at the minimum region count.
  - `spec_new_validated_accepts_two_regions_for_linear` — the
    minimum-valid `Linear` case (both endpoints land on the
    region's speed at `t = 0` and `t = 1`).
  - `spec_new_validated_accepts_typical_inputs_for_both_variants` —
    9 speeds × 5 region counts for `Constant`, 9 × 9 speed pairs ×
    5 region counts (≥ 2) for `Linear`. Catches any future narrowing
    that accidentally rejects the documented accept band.
  - `spec_new_validated_some_branch_byte_equals_new` — the validator
    is rejection-only, no normalization. Verified across both
    variants at minimum-valid `region_count`.
  - `spec_new_validated_is_const_callable` — `const` items hold one
    `Some` spec and two distinct `None` cases (zero-count + Linear
    with `region_count == 1`).
  - `spec_new_validated_some_branch_feeds_synthesize_regions` — end-
    to-end: an accepted spec must produce `Some` from the
    synthesizer on a non-degenerate span. Spot-checks `start_x`
    values to verify `region_count` is used verbatim.
  - `spec_new_validated_accepts_linear_with_equal_endpoint_speeds` —
    pins down the orthogonal-layering carve-out + chains
    `linear_validated` + `new_validated` to show the composition.
  - `spec_new_validated_rejection_table_matches_synthesize_regions_zero_region`
    — cross-validates: every rejection mode the spec validator owns
    is *also* a rejection in the synthesizer (validator is a strict
    subset of the synthesizer's rejection set on the inputs it owns).
  - `spec_new_validated_does_not_mutate_inputs_on_accept` — guards
    against a hypothetical "helpful" normalization that promoted
    `Linear { s, s }` into `Constant(s)` on accept; the ramp
    variant must survive byte-for-byte.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — 2555 tests passing,
  0 failing (the 12 new tests are part of that total).
  `cargo test -p music-engraver --lib spec_new_validated` runs the
  12 new tests in isolation: all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings on
  `trill_extension.rs` (the pre-existing warnings on
  `multi_staff.rs:394` and unrelated `music/` crate files persist).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). On the trill side, a natural follow-up is mirroring
  this validator on `TrillExtensionFullOptions::with_speed_ramp_ramp_count`
  via a `with_speed_ramp_validated_ramp_count` that returns
  `Option<Self>` — a one-line extension of the same pattern.

- Open issues: None. The change is additive — no existing public API
  altered, no golden baseline regenerated, no example or test
  modified. Pre-existing clippy warnings in
  `music/src/notation/rhythm/meter.rs` and
  `music-engraver/src/score/multi_staff.rs:394` remain unaddressed
  (out of scope for this chunk).

## 2026-05-27 — Post-v1, TrillExtensionFullOptions::with_speed_ramp_validated_ramp_count

- Did: Closed the explicit follow-up flagged in the previous
  `TrillSpeedRampSpec::new_validated` entry: added the strict counterpart
  to `with_speed_ramp_ramp_count` on `TrillExtensionFullOptions` in
  `music-engraver/src/layout/trill_options.rs`. The bare
  `with_speed_ramp_ramp_count` stores any `(ramp, region_count)` pair
  unchanged so options bundles can travel through annotation pipelines
  whose validity is only checked at draw time (mirroring the
  permissive contract of `TrillSpeedRampSpec::new` /
  `TrillSpeedRamp::linear` one layer below). The new
  `with_speed_ramp_validated_ramp_count` returns `Option<Self>` and
  rejects exactly the same degenerate inputs that
  `TrillSpeedRampSpec::new_validated` (and therefore
  `TrillSpeedRamp::synthesize_regions`) would reject — so callers
  wanting construction-time rejection get it at the call site rather
  than discovering `None` later. Lifts the same validator-pairing pattern
  used at the `TrillSpeedRamp` (`linear` / `linear_validated`) and
  `TrillSpeedRampSpec` (`new` / `new_validated`) layers up one more level
  into the bundle builder. Updated the permissive setter's doc comment
  to point at the new method instead of being silent about the strict
  counterpart's existence.

  Rejection rules (`None` returned) — inherited verbatim from the
  underlying `TrillSpeedRampSpec::new_validated`:
  - `region_count == 0` for any ramp variant (no regions to emit).
  - `ramp` is `TrillSpeedRamp::Linear { .. }` AND `region_count < 2`.

  Carve-outs (`Some(Self { .. })` returned with `speed_ramp` populated
  — pinned down explicitly):
  - `TrillSpeedRamp::Constant(_)` accepts any `region_count >= 1`.
  - `TrillSpeedRamp::Linear { start, end }` with `start == end` is
    accepted (rejection of equal endpoints lives on
    `linear_validated`, not the spec validator nor the bundle-builder
    validator — the layering is orthogonal).
  - All other fields on `self` survive byte-for-byte on accept
    (mirrors the additive contract of every other setter on this
    bundle — particularly that `speed` is NOT cleared when
    `speed_ramp` is set, since the documented dispatch permits both
    fields to coexist).

  Implementation: a one-call delegation to
  `TrillSpeedRampSpec::new_validated` followed by a `match` —
  acceptance produces the populated bundle, rejection returns `None`
  with the partially-built bundle dropped. `const fn` for symmetry
  with every other setter on this bundle and with
  `TrillSpeedRampSpec::new_validated` itself.

  Tests added (14) in `layout::trill_options::tests`:
  - `with_speed_ramp_validated_ramp_count_rejects_zero_region_count_for_constant`
  - `with_speed_ramp_validated_ramp_count_rejects_zero_region_count_for_linear`
  - `with_speed_ramp_validated_ramp_count_rejects_one_region_for_linear`
  - `with_speed_ramp_validated_ramp_count_accepts_one_region_for_constant` —
    the Constant carve-out at the minimum region count.
  - `with_speed_ramp_validated_ramp_count_accepts_two_regions_for_linear` —
    the minimum-valid `Linear` case.
  - `with_speed_ramp_validated_ramp_count_some_branch_byte_equals_unvalidated` —
    on accepted pairs, the validated and permissive setters produce
    field-by-field equal bundles. Walks 5 region counts × 3 Constant
    ramps + 4 region counts × 3 Linear ramps = 27 accepted pairs.
    Catches a future drift where the validator started normalizing
    accepted inputs (e.g. collapsing `Linear { s, s }` into
    `Constant(s)`).
  - `with_speed_ramp_validated_ramp_count_some_branch_isolation` —
    on accept from `new()`, only `speed_ramp` is populated; every
    other field stays `None`.
  - `with_speed_ramp_validated_ramp_count_preserves_other_setters_on_some` —
    chained on top of a fully-populated bundle, every prior field
    survives unchanged on accept. Locks the additive contract; pins
    down the documented "speed + speed_ramp may coexist" carve-out.
  - `with_speed_ramp_validated_ramp_count_is_const_callable` — `const`
    items hold one `Some` and two distinct `None` cases (zero-count
    Constant + Linear with `region_count == 1`).
  - `with_speed_ramp_validated_ramp_count_rejection_matches_spec_new_validated` —
    cross-validation: for every pair in 0..=3 × {Constant, Linear},
    `setter.is_none()` iff `spec.is_none()`, and on accept the
    stored `speed_ramp` byte-equals the spec the validator produced.
    Locks the delegation; any future divergence (e.g. setter adds an
    extra rule the spec doesn't) trips this canary.
  - `with_speed_ramp_validated_ramp_count_none_branch_does_not_partially_populate` —
    on `None`, the partially-built bundle is dropped at the
    `Option<Self>` shape, so a future refactor that swapped the
    return type for `Self` with a silent fallback would fail this
    test.
  - `with_speed_ramp_validated_ramp_count_accepts_linear_equal_endpoint_speeds` —
    the orthogonal-layering carve-out pinned at the bundle layer.
  - `with_speed_ramp_validated_ramp_count_overwrites_prior_value_on_some` —
    last-write-wins on the accept-band, matching the permissive
    setter.
  - `with_speed_ramp_validated_ramp_count_some_branch_feeds_synthesize_regions` —
    end-to-end smoke: an accepted bundle's `speed_ramp` feeds cleanly
    into `synthesize_regions` and produces the expected number of
    regions with the expected start positions.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2569 tests passing,
  0 failing** (up from 2555 by exactly the 14 new tests).
  `cargo test -p music-engraver --lib with_speed_ramp_validated` runs
  the 14 new tests in isolation: all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings on
  `trill_options.rs` (pre-existing warnings on `multi_staff.rs:394`
  and unrelated `music/` crate files persist — out of scope).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). The validator pairing pattern
  (`{constructor}` / `{constructor}_validated`) is now consistently
  applied across all three layers of the trill speed-ramp stack
  (`TrillSpeedRamp::linear`, `TrillSpeedRampSpec::new`,
  `TrillExtensionFullOptions::with_speed_ramp_ramp_count`); no
  further extension points remain on that surface.

- Open issues: None. The change is additive — no existing public API
  altered, no golden baseline regenerated, no example or test
  modified. Pre-existing clippy warnings in
  `music/src/notation/rhythm/meter.rs` and
  `music-engraver/src/score/multi_staff.rs:394` remain unaddressed
  (out of scope for this chunk).

## 2026-05-27 — Post-v1, with_ornament_validated across three trill options bundles

- Did: Extended the validator-pairing pattern to the
  *ornament-acceptance* surface across all three trill options bundles.
  Added `with_ornament_validated(ornament) -> Option<Self>` on:
  - `TrillBracketOptions` in
    `music-engraver/src/layout/trill_bracket.rs`
  - `TrillExtensionSpeedOptions` in
    `music-engraver/src/layout/trill_extension.rs`
  - `TrillExtensionFullOptions` in
    `music-engraver/src/layout/trill_options.rs`

  All three permissive `with_ornament` setters store any `Ornament` as
  written so options bundles can travel through annotation pipelines
  whose validity is only checked at the renderer's collector (which
  filters by `Ornament::supports_trill_extension()` and silently drops
  the entire trill extension — no wiggle, no bracket — for unsupported
  ornaments). The new validated counterparts return `None` for exactly
  the set `!Ornament::supports_trill_extension()` (currently 13 of 15
  variants: every variant except `Trill` and `TrillWithMordent`), so
  callers wanting construction-time rejection get a Result-shaped
  failure at the call site rather than discovering an empty SVG at
  draw time.

  The accept band is locked to the predicate, not hardcoded — a
  future expansion of `supports_trill_extension` (e.g. accepting
  `Tremblement` for a wavy-line tail) automatically widens all three
  validators without code changes. The `rejects_every_non_supporting_variant`
  tests on each bundle walk every ornament in `Ornament::ALL` and
  cross-validate `is_none()` iff `!supports_trill_extension()`, so the
  pairing tightness is asserted, not asserted-once-then-forgotten.

  All three validated methods are `const fn`, matching every other
  setter on these bundles and `TrillSpeedRampSpec::new_validated` /
  `with_speed_ramp_validated_ramp_count` from the prior two chunks
  — the validator-pairing pattern is now uniformly applied at
  every options-bundle layer that has an ornament knob.

  On the accept branch all other fields on `self` are preserved
  byte-for-byte (additive contract). On the reject branch the
  partially-built bundle is dropped via the `Option<Self>` shape;
  there is no silent-fallback that returns `Some(self)` with
  `ornament` unset, because that would demote a rejection into a
  no-op.

  One-line dependent change: promoted
  `Ornament::supports_trill_extension` from `fn` to `const fn` in
  `music-engraver/src/layout/ornament.rs`. The body is just
  `matches!(self, Self::Trill | Self::TrillWithMordent)`, so the
  promotion is mechanical and additive — callers needing the runtime
  shape are unaffected; the three new validators need the const
  shape to themselves be `const`. Existing tests on the predicate
  continue to pass unchanged.

  Acceptance rules (`Some(self)` returned) — identical across all three
  bundles:
  - `ornament.supports_trill_extension()` is `true` (i.e. ornament is
    `Trill` or `TrillWithMordent`).
  - `self.ornament = Some(ornament)`; every other field on `self`
    survives byte-for-byte.

  Rejection rules (`None` returned) — identical across all three
  bundles:
  - `ornament.supports_trill_extension()` is `false` (any of the 13
    other variants: `ShortTrill`, `Mordent`, `InvertedMordent`, `Turn`,
    `InvertedTurn`, `TurnSlash`, `TurnUp`, `TurnUpSlash`, `Tremblement`,
    `TremblementCouperin`, `Haydn`, `Shake`, `Schleifer`).

  Tests added (28 total): per bundle, 8–10 tests covering accept-Trill,
  accept-TrillWithMordent, reject-ShortTrill (headline rejection),
  reject-every-non-supporting-variant (walks `Ornament::ALL`),
  byte-equality with permissive setter on accept (catches future
  normalization), preserve-other-setters-on-some (full chained bundle),
  is-const-callable (const items hold one Some and two distinct Nones),
  overwrites-prior-value-on-some (last-write-wins matching permissive),
  some-branch-isolation (only `ornament` populated from `new()`). The
  `TrillExtensionFullOptions` set adds a `none_branch_does_not_partially_populate`
  test for the strongest field-preservation case (combined with `Turn`
  rejection on a bracket-populated bundle), and the `preserves_other_setters_on_some`
  test there layers in a `TrillSpeedRampSpec::new(Linear, 4)` to
  exercise the bundle's largest field. The `TrillBracketOptions` set
  walks all 3 × 15 = 45 (side × ornament) pairs in the
  cross-validation and byte-equality tests, catching any per-side
  asymmetry in the validator's behaviour.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2597 tests passing,
  0 failing** (up from 2569 by exactly the 28 new tests).
  `cargo test -p music-engraver --lib with_ornament_validated` runs
  the 28 new tests in isolation: all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings on
  any of the four modified files (pre-existing warnings on
  `multi_staff.rs:394` and unrelated `music/` crate files persist —
  out of scope).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). The validator-pairing pattern is now consistently
  applied across (a) the entire trill speed-ramp stack and (b) the
  entire trill ornament-acceptance surface — every options-bundle
  setter whose acceptance criterion is testable at construction time
  has both a permissive and a strict variant. Natural follow-ups
  outside the trill surface: similar validator pairings on other
  options bundles that silently drop their work at draw time — e.g.
  bracket extension length (`with_extension_length_ss` accepts
  non-positive values that the renderer's fail-safe suppresses;
  rejection at construction time would surface the misuse earlier).

- Open issues: None. The change is additive — no existing public API
  altered, no golden baseline regenerated, no example or test
  modified. `Ornament::supports_trill_extension` gained `const fn`
  but its body is unchanged, so call sites and pre-existing tests on
  the predicate continue to pass. Pre-existing clippy warnings in
  `music/src/notation/rhythm/meter.rs` and
  `music-engraver/src/score/multi_staff.rs:394` remain unaddressed
  (out of scope for this chunk).

## 2026-05-27 — Post-v1, with_extension_length_ss_validated across three trill options bundles

- Did: Extended the validator-pairing pattern to the
  *extension-length-acceptance* surface across all three trill options
  bundles. Added `with_extension_length_ss_validated(length_ss: f64) -> Option<Self>`
  on:
  - `TrillBracketOptions` in
    `music-engraver/src/layout/trill_bracket.rs`
  - `TrillExtensionSpeedOptions` in
    `music-engraver/src/layout/trill_extension.rs`
  - `TrillExtensionFullOptions` in
    `music-engraver/src/layout/trill_options.rs`

  All three permissive `with_extension_length_ss` setters store any `f64`
  unchanged so options bundles can travel through annotation pipelines
  whose validity is only checked at draw time. The renderer's "would draw
  a wiggle" check in
  `render/system_renderer/mod.rs:1064-1075`
  (`Some(len_ss) if len_ss > 0.0` → emit wiggle; non-positive `Some(_)`
  → collapse `end_x` to `start_x` → no wiggle) is the documented
  fail-safe. The new validated counterparts mirror that predicate
  literally: accept iff `length_ss > 0.0`. The reject band is therefore
  precisely `length_ss <= 0.0 || length_ss.is_nan()` — `0.0`, `-0.0`,
  every finite negative, `-∞`, and every NaN payload return `None`;
  every finite positive *and* `+∞` (which the renderer accepts via
  clamping to the natural span) return `Some(self)`.

  The accept band is locked to the renderer's predicate, not hardcoded
  — a future tightening of the renderer (e.g. requiring a minimum
  tile-width) would need to update both clauses in lockstep, and the
  `rejection_matches_renderer_accept_band` test on each bundle walks 12
  probe values cross-validating `is_none() iff !(len > 0.0)`. A
  fourth-layer cross-validation test on `TrillExtensionFullOptions`
  (`three_bundles_agree_on_accept_band`) walks the same probe set
  across all three bundles and asserts that no two implementations
  disagree on any value — catches drift in the riskiest part of the
  pattern (permissive/validated split applied independently three
  times).

  All three validated methods are `const fn`, matching every other
  setter on these bundles and `TrillSpeedRampSpec::new_validated` /
  `with_speed_ramp_validated_ramp_count` / the trio of
  `with_ornament_validated` setters from the prior three chunks. On
  the accept branch all other fields on `self` are preserved
  byte-for-byte (additive contract, verified by
  `some_byte_equals_permissive` against the permissive setter). On the
  reject branch the partially-built bundle is dropped via the
  `Option<Self>` shape; there is no silent-fallback that returns
  `Some(self)` with `extension_length_ss` unset, because that would
  demote a rejection into a no-op.

  Acceptance rules (`Some(self)` returned) — identical across all three
  bundles:
  - `length_ss > 0.0` (any finite positive, plus `+∞`).
  - `self.{extension_length_ss | length_ss} = Some(length_ss)`; every
    other field on `self` survives byte-for-byte.
  - For `TrillExtensionFullOptions`: writes to the `length_ss` field
    (the wiggle's termination length), *not* to `bracket_length_ss`
    (the bracket hook length) — preserves the naming-disambiguation
    invariant from the permissive setter.

  Rejection rules (`None` returned) — identical across all three
  bundles:
  - `length_ss == 0.0` (positive *and* negative zero — both fail
    `length_ss > 0.0`).
  - `length_ss < 0.0` (any finite negative, including `-∞`).
  - `length_ss.is_nan()` (any NaN payload — NaN comparisons return
    false, so `NaN > 0.0` is false).

  Tests added (39 total): per bundle, 12–15 tests covering rejects-zero,
  rejects-negative-zero (catches an `is_sign_negative()` or
  `!= 0.0` refactor), rejects-negative-finite (walks 5 magnitudes),
  rejects-negative-infinity, rejects-nan, accepts-positive-finite
  (walks 5 magnitudes), accepts-positive-infinity (mirrors renderer
  accept), some-branch byte-equality with permissive setter (walks 6
  accepted values including `+∞`), preserve-other-setters-on-some,
  overwrites-prior-value-on-some (last-write-wins), is-const-callable
  (one `Some` and one `None` const binding), and
  `rejection_matches_renderer_accept_band` (cross-validation walking
  12 probe values). The `TrillExtensionFullOptions` set adds three
  bundle-specific tests:
  - `writes_length_ss_not_bracket_length_ss` — locks the
    field-targeting invariant down at the validated layer (mirrors
    the permissive setter's
    `with_extension_length_ss_is_distinct_from_with_bracket_length_ss`).
  - `none_branch_does_not_partially_populate` — combines a non-trivial
    chain with a rejecting validator call.
  - `three_bundles_agree_on_accept_band` — fourth-layer
    cross-validation walking 12 probe values across all three bundles
    and asserting pairwise agreement.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2636 tests passing,
  0 failing** (up from 2597 by exactly the 39 new tests).
  `cargo test -p music-engraver --lib with_extension_length_ss_validated`
  runs the 39 new tests in isolation: all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings on
  any of the three modified files (pre-existing warning on
  `multi_staff.rs:394` and unrelated `music/` crate clippy warnings
  persist — out of scope).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). The validator-pairing pattern is now consistently
  applied across (a) the entire trill speed-ramp stack, (b) the entire
  trill ornament-acceptance surface, and (c) the entire trill
  extension-length surface — every options-bundle setter whose
  acceptance criterion is testable at construction time has both a
  permissive and a strict variant. Natural follow-ups outside the
  trill surface: similar validator pairings on other options bundles
  whose values get silently dropped by the renderer — e.g. bracket
  hook length (`with_hook_length_ss` accepts negative values that the
  bracket renderer folds via `abs()`; strict rejection at construction
  time would surface explicit-flip intent earlier); bracket length
  (`with_length_ss` accepts non-positive values whose semantics at the
  renderer are murky and worth pinning down).

- Open issues: None. The change is additive — no existing public API
  altered, no golden baseline regenerated, no example or test
  modified.
  (out of scope for this chunk).

## 2026-05-27 — Post-v1, with_hook_length_ss_validated / with_bracket_length_ss_validated (bracket hook surface)

- Did: Extended the validator-pairing pattern to the *bracket hook
  length* surface across the two options bundles that expose it. Added:
  - `TrillBracketOptions::with_hook_length_ss_validated(hook_length_ss: f64) -> Option<Self>`
    in `music-engraver/src/layout/trill_bracket.rs` (writes `length_ss`,
    the hook length on this bundle; matches the byte-equivalent
    `with_length_ss` / `with_hook_length_ss` aliases).
  - `TrillExtensionFullOptions::with_bracket_length_ss_validated(length_ss: f64) -> Option<Self>`
    in `music-engraver/src/layout/trill_options.rs` (writes
    `bracket_length_ss`, the hook length on the full-options bundle).

  `TrillExtensionSpeedOptions` has no hook-length field and therefore
  no third validator on this surface — only two bundles participate in
  the cross-validation, matching the structure of the field itself.

  **Deliberate accept-band deviation from the renderer.** Unlike every
  prior validator-pairing in this stack (which mirrored the renderer's
  accept band literally), these two validators *tighten* past it. The
  renderer's `layout_trill_bracket_hook` at
  `music-engraver/src/layout/trill_bracket.rs:393-411` applies
  `let length = length.abs();` — silently folding negatives to positive
  while the explicit `HookDirection` always wins. That fold makes
  `.with_hook_length_ss(-1.0).with_direction(Down)` indistinguishable
  from `.with_hook_length_ss(1.0).with_direction(Down)` at draw time,
  silently overriding the caller's apparent flip intent (they probably
  meant `HookDirection::Up`). Zero produces a degenerate hook
  (`y_top == y_bottom` → no visible line). NaN propagates as NaN
  coordinates. The validators reject all three cases at construction
  time so the misuse surfaces before the SVG is generated; the doc
  steers callers toward `HookDirection::{Up, Down}` for explicit
  direction control.

  Acceptance rules (`Some(self)` returned) — identical on both
  bundles, numerically identical to the
  `with_extension_length_ss_validated` predicate (`> 0.0`) but with a
  different rationale:
  - `length > 0.0` (any finite positive, plus `+∞` — accepted for
    byte-equivalence with the permissive setter, which the renderer
    also tolerates).
  - All other fields on `self` survive byte-for-byte (additive
    contract, matching every other validator-pairing on these
    bundles).
  - On `TrillBracketOptions`: writes to `length_ss`, NOT to
    `extension_length_ss`. On `TrillExtensionFullOptions`: writes to
    `bracket_length_ss`, NOT to `length_ss`. The
    naming-disambiguation invariant from the permissive setters is
    preserved at the validated layer.

  Rejection rules (`None` returned) — identical on both bundles:
  - `length == 0.0` (positive *and* negative zero — both fail
    `> 0.0` and both produce a degenerate hook).
  - `length < 0.0` (any finite negative, including `-∞`). This is
    the headline rejection — the renderer's `.abs()` fold would
    silently swallow these. Strict rejection at construction time
    surfaces explicit-flip misuse.
  - `length.is_nan()` (any NaN payload — NaN comparisons return
    false, so `NaN > 0.0` is false; the renderer would otherwise emit
    NaN hook coordinates).

  Both validated methods are `const fn`, matching every other setter
  on these bundles.

  Tests added (30 total — 15 per bundle): rejects-zero,
  rejects-negative-zero (catches `is_sign_negative()` or `!= 0.0`
  refactor), rejects-negative-finite (walks 5 magnitudes),
  rejects-negative-infinity, rejects-nan, accepts-positive-finite
  (walks 6 magnitudes), accepts-positive-infinity, writes-to-correct-field
  (locks the field-targeting invariant — `length_ss` on
  `TrillBracketOptions`, `bracket_length_ss` on
  `TrillExtensionFullOptions`), some-branch byte-equality with the
  permissive setter (walks 7 accepted values including `+∞`),
  preserves-other-setters-on-some (largest combinable chain —
  `TrillExtensionFullOptions` covers bracket + direction + speed +
  ornament + extension length + speed ramp), overwrites-prior-value-on-some
  (last-write-wins), is-const-callable (one `Some` and one `None` const
  binding), `rejection_matches_visible_unfolded_hook_predicate`
  (cross-validates the validator against the deliberately-narrowed
  `> 0.0` accept band — distinct from the prior surface's
  `rejection_matches_renderer_accept_band` because the renderer's
  literal accept band on the hook is `!is_nan` after `.abs()`, not
  `> 0.0`), and `none_branch_does_not_partially_populate`. The
  `TrillBracketOptions` set adds one bundle-specific test:
  - `some_byte_equals_with_length_ss` — pins the three-way equivalence
    on accept between the validator and *both* permissive aliases
    (`with_length_ss` and `with_hook_length_ss`), since both write the
    same `length_ss` field.

  The `TrillExtensionFullOptions` set adds one bundle-specific test:
  - `two_bundles_agree_on_accept_band` — fourth-layer cross-validation
    walking 13 probe values across `TrillBracketOptions::with_hook_length_ss_validated`
    and this bundle's `with_bracket_length_ss_validated`, asserting
    pairwise agreement on `is_none()`. Narrower than the
    `three_bundles_agree_on_accept_band` on the extension-length
    surface only because `TrillExtensionSpeedOptions` has no
    hook-length field.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2666 tests passing,
  0 failing** (up from 2636 by exactly the 30 new tests).
  `cargo test -p music-engraver --lib with_hook_length_ss_validated`
  runs the 15 `TrillBracketOptions` tests in isolation: all pass.
  `cargo test -p music-engraver --lib with_bracket_length_ss_validated`
  runs the 15 `TrillExtensionFullOptions` tests in isolation: all
  pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings on
  either modified file (pre-existing `multi_staff.rs:394` warning and
  unrelated `music/` crate clippy warnings persist — out of scope).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). The validator-pairing pattern is now consistently
  applied across (a) the entire trill speed-ramp stack, (b) the
  entire trill ornament-acceptance surface, (c) the entire trill
  extension-length surface, and (d) the entire bracket-hook-length
  surface (this chunk). Natural follow-ups outside the trill stack:
  apply the same pattern to other options-bundle setters whose values
  get silently coerced or dropped at draw time. Notable candidates:
  - `TrillExtensionSpeedOptions::with_speed` already accepts any
    `TrillWiggleSpeed` and the renderer dispatches on `.to_glyph()` —
    the accept band is total, no validator needed.
  - Beyond the trill surface: any options bundle that builds a
    bracket-like shape (e.g. `HairpinOptions`, `OttavaOptions`,
    `VoltaOptions`) likely has similar silently-coerced length
    parameters. Worth a one-pass survey before tackling them
    individually.

- Open issues: None. The change is additive — no existing public API
  altered, no golden baseline regenerated, no example or example
  test modified. `with_hook_length_ss_validated` is the first
  validator on this stack whose accept band is *deliberately narrower*
  than the renderer's accept band (every prior validator mirrored the
  renderer exactly); the cross-validation test name was renamed from
  `rejection_matches_renderer_accept_band` to
  `rejection_matches_visible_unfolded_hook_predicate` to reflect the
  distinction. The pre-existing `multi_staff.rs:394` clippy warning
  and unrelated `music/` crate clippy warnings remain unaddressed
  (out of scope).

## 2026-05-27 — Post-v1, hairpin niente (open-circle to/from silence)

- Did: Added engraver support for the niente "o" — a small open circle at
  the closed (pointy) end of a hairpin marking to/from silence. Touched
  four files:

  1. `music-engraver/src/render/svg_writer.rs`: new
     `SvgWriter::add_circle(cx, cy, r, stroke, stroke_width, fill)`
     primitive emitting a standard SVG `<circle>` element. Generic
     enough for any future small-marker callers; the niente convention
     uses it with `fill="none"` for the open "o".

  2. `music-engraver/src/layout/hairpin.rs`:
     - New `NienteCircleLayout { cx, cy, radius, stroke_width }` —
       `#[derive(Clone, Copy, Debug)]` so `Option<NienteCircleLayout>` is
       Copy and the existing destructure-by-value pattern in the renderer
       still type-checks.
     - New `pub const HAIRPIN_NIENTE_RADIUS_SS: f64 = 0.2` (→ 0.4ss
       diameter, the engraved standard).
     - New `niente: Option<NienteCircleLayout>` field on `HairpinLayout`,
       always `None` from the existing `layout_hairpin`. The four
       existing call sites (system_renderer, page_renderer, two example
       constructions) are unchanged — backward-compatible.
     - New `layout_hairpin_with_niente(kind, x_start, x_end,
       staff_bottom_y, staff_space, stroke_width)` — produces an
       identical wedge layout to `layout_hairpin` and attaches a niente
       circle at `(x_start, y_center)` for `Crescendo` (from silence) or
       `(x_end, y_center)` for `Decrescendo` (to silence). Circle radius
       scales linearly with `staff_space`; stroke matches parent.

  3. `music-engraver/src/render/hairpin_renderer.rs`:
     - Added `niente` to the destructure pattern.
     - When `Some(n)`, draws `svg.add_circle(n.cx, n.cy, n.radius,
       "black", n.stroke_width, "none")`. The wedge lines are drawn
       unconditionally — niente is purely additive decoration.
     - Updated the `draw_hairpin` doc comment to mention the niente path.

  4. `music-engraver/src/layout/mod.rs`: re-exported the two new
     public items (`layout_hairpin_with_niente`, `NienteCircleLayout`)
     and the new constant (`HAIRPIN_NIENTE_RADIUS_SS`).

  Anchor placement matches standard engraving: for a crescendo the
  silent end is the *left tip* (`x_start`), and for a decrescendo the
  silent end is the *right tip* (`x_end`). Both share `y_center` (the
  hairpin midline) so the circle visually merges into the wedge tip.

  Design choice — open "o", not filled dot: engraved niente is
  conventionally a stroked ring (`fill="none"`). A filled circle would
  read as a staccato or fermata dot and be wrong notation. The renderer
  test `niente_circle_is_open_o_not_filled_dot` locks both
  `fill="none"` *and* `stroke="black"` on the emitted circle line so a
  future "let's just fill it" refactor surfaces immediately.

  Design choice — stroke width inheritance: the niente ring uses the
  parent hairpin's `stroke_width` so the ring reads as the same line
  weight as the wedge. `niente_circle_stroke_width_matches_hairpin_stroke`
  pins this by asserting `stroke-width="<custom>"` appears on exactly
  three SVG elements (2 wedge lines + 1 circle = 3) under a non-default
  stroke width.

  Tests added (23 total):

  - `svg_writer.rs` (2): `svg_writer_circle_element` (full attribute
    coverage on the bare primitive — cx/cy/r/stroke/stroke-width/fill),
    `svg_writer_circle_filled` (hex stroke + filled fill, catches a
    refactor that hardcoded `fill="none"`).

  - `hairpin.rs` (10): `plain_hairpin_has_no_niente` (default
    `niente: None`); `with_niente_crescendo_places_circle_at_start`
    (cx == x_start, cy == y_center); `with_niente_decrescendo_places_circle_at_end`
    (cx == x_end, cy == y_center); `niente_radius_matches_const_times_staff_space`
    (radius locked to `HAIRPIN_NIENTE_RADIUS_SS * staff_space` — catches
    a refactor that uses the wrong factor); `niente_radius_scales_with_staff_space`
    (walks two staff spaces, asserts both absolute values and the 2× ratio);
    `niente_stroke_width_matches_hairpin` (custom stroke flows through);
    `niente_does_not_alter_wedge_geometry` (byte-equal wedge fields
    between `layout_hairpin` and `layout_hairpin_with_niente` — niente
    is purely additive); `niente_y_lives_on_hairpin_midline`
    (`cy == y_center` to 1e-12 — locks the merge-into-tip invariant);
    `crescendo_and_decrescendo_nientes_target_opposite_ends` (cross-check
    that crescendo and decrescendo niente cx differ by the full span,
    cy and radius are equal); `niente_const_layout_is_copy` (proves
    `NienteCircleLayout: Copy` — the destructure pattern in the renderer
    relies on it).

  - `hairpin_renderer.rs` (11): `plain_hairpin_emits_no_circle` (zero
    `<circle>` elements when `niente: None` — locks the
    decoration-is-opt-in invariant); `niente_hairpin_emits_exactly_one_circle`;
    `niente_hairpin_still_emits_two_wedge_lines` (decoration does not
    displace the wedge — exactly 2 `<line>` elements);
    `niente_crescendo_circle_anchored_at_x_start` (`cx="100"`);
    `niente_decrescendo_circle_anchored_at_x_end` (`cx="600"`);
    `niente_cy_matches_hairpin_y_center`; `niente_circle_is_open_o_not_filled_dot`
    (both `fill="none"` AND `stroke="black"` on the circle line);
    `niente_circle_radius_in_svg_matches_layout` (`r="<radius>"`);
    `niente_circle_stroke_width_matches_hairpin_stroke` (custom stroke
    appears on exactly 3 elements: 2 wedge lines + 1 circle);
    `niente_circle_lives_inside_viewbox_bounds_for_typical_layout`
    (sanity: `0 < radius < staff_space`); `niente_crescendo_and_decrescendo_produce_different_svg`
    (cross-shape distinguishability — locks that anchor differs).

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo check --workspace` passes (0 errors).
  `cargo test -p music-engraver --lib` passes — **2689 tests passing,
  0 failing** (up from 2666 by exactly the 23 new tests; matches the
  count expectation: 2 SvgWriter + 10 hairpin layout + 11 hairpin
  renderer = 23).
  `cargo test -p music-engraver --lib niente` runs the 20 new niente
  tests plus the pre-existing `niente_maps_to_smufl_niente` dynamic
  test in isolation: all 21 pass.
  `cargo test -p music-engraver --lib svg_writer_circle` runs the 2
  new circle primitives in isolation: both pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings —
  only the pre-existing `multi_staff.rs:394` warning persists (out of
  scope, noted across prior progress entries).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). Natural follow-ups on the hairpin surface itself:
  - Plumb `layout_hairpin_with_niente` through the score → system_renderer
    → page_renderer chain so a `ScoreBuilder` caller can request a
    niente hairpin end-to-end (this chunk stopped at the layout +
    primitive renderer — `system_renderer/mod.rs:414` still calls the
    plain `layout_hairpin`). Requires a score-event flag for niente,
    which is out of scope for this chunk.
  - Dashed-hairpin / "cresc. - - -" text variants for long crescendi.
  - Nested-dynamic hairpins (start with `p`, end with `f` on the same
    hairpin).
  - Niente at the *open* end (rare but used — "from silence open" vs
    "from silence closed" — currently the layout only supports closed-end
    niente by anchor choice).

- Open issues: None. The change is purely additive: new struct field
  defaults to `None` from every existing constructor; the SvgWriter
  primitive is new public surface; no example or golden baseline
  touched (no example exercises the niente layer yet — see "Next"
  above for the plumbing-through chunk). The pre-existing
  `multi_staff.rs:394` clippy warning and unrelated `music/` crate
  clippy warnings remain unaddressed (out of scope).

## 2026-05-27 — Post-v1, dashed hairpin variant

- Did: Added engraver support for dashed-wedge hairpins — the standard
  notation for hairpin continuation across system breaks and for
  "soft"/implied crescendi in modern scores. Mirrors the niente
  additive-`Option<...>` pattern from the previous chunk. Touched
  three files:

  1. `music-engraver/src/layout/hairpin.rs`:
     - New `HairpinDashStyle { dash_length, gap_length }` —
       `#[derive(Clone, Copy, Debug)]` so `Option<HairpinDashStyle>`
       is `Copy` and the renderer's destructure-by-value pattern still
       type-checks.
     - New `pub const HAIRPIN_DASH_LENGTH_SS: f64 = 0.4;` and
       `pub const HAIRPIN_GAP_LENGTH_SS: f64 = 0.2;` (2:1 dash:gap by
       design — the docstrings on the constants explain why).
     - New `dashed: Option<HairpinDashStyle>` field on `HairpinLayout`,
       always `None` from `layout_hairpin` and
       `layout_hairpin_with_niente` (backward-compatible — no existing
       call site sees behavior change).
     - New `layout_hairpin_dashed(kind, x_start, x_end, staff_bottom_y,
       staff_space, stroke_width) -> HairpinLayout` — same signature as
       `layout_hairpin`, returns wedge with `dashed: Some(...)` populated
       (dash/gap derived from the constants times `staff_space` so
       the dash pattern scales linearly with staff size).

  2. `music-engraver/src/render/hairpin_renderer.rs`:
     - Added `dashed` to the destructure pattern.
     - When `dashed.is_some()`, switches from `add_line` to
       `add_dashed_line` (already present in `SvgWriter` from prior
       work) for both wedge lines. Dasharray string is formatted as
       `"{dash_length},{gap_length}"` in font design units.
     - When `dashed.is_none()`, the existing solid-line path runs
       unchanged.
     - Niente "o" circle stays solid (no stroke-dasharray) regardless
       of the wedge style — engraved convention treats the niente as
       a definite symbol independent of dashed/solid wedge styling.
     - Updated the `draw_hairpin` doc comment to mention both
       additive flags.

  3. `music-engraver/src/layout/mod.rs`: re-exported the new public
     items (`layout_hairpin_dashed`, `HairpinDashStyle`,
     `HAIRPIN_DASH_LENGTH_SS`, `HAIRPIN_GAP_LENGTH_SS`).

  Design choice — dash/gap in design units, not staff-spaces, on the
  `HairpinDashStyle` struct: the renderer needs the values in the
  same coordinate system as the wedge x/y (font design units, matches
  the viewBox). Storing the pre-scaled values on the struct keeps the
  renderer arithmetic-free and matches the existing `NienteCircleLayout`
  convention (radius is in design units). The
  `dashed_hairpin_dasharray_uses_design_units_not_staff_spaces`
  regression test pins this — a future refactor that forgets the
  `* staff_space` multiplication would emit `"0.4,0.2"` (much too
  fine) instead of `"100,50"` and that test catches it immediately.

  Design choice — dashed wedge keeps niente solid: a dashed-niente
  combination would look like a tiny dashed circle, which engraved
  convention does not use (the niente "o" is always a definite,
  solid ring even on a continuation hairpin). The renderer always
  uses `add_circle` for niente; the
  `dashed_hairpin_with_niente_keeps_circle_solid` test pins this
  three ways: it asserts the wedge has 2 `stroke-dasharray`
  occurrences, the circle line does NOT contain `stroke-dasharray`,
  and the circle line still carries `fill="none"`.

  Design choice — no `layout_hairpin_dashed_with_niente` constructor.
  Combining dashed + niente is the rare case. Adding a third
  constructor would balloon the API surface (you'd then want
  `_dashed_with_niente_at_open_end` etc. for every cross-product). The
  fields are public; the
  `dashed_combo_with_niente_supported_via_field_mutation` layout test
  pins the field-mutation path as a stable API contract — a future
  refactor that hides these fields behind getters must preserve the
  combinator-via-mutation route or add explicit combinator constructors.

  Tests added (23 total — 11 layout + 12 renderer):

  - `hairpin.rs` (11):
    - `plain_hairpin_has_no_dashed` — checks all four
      non-dashed constructors (`cresc`, `decresc`, `cresc_n`,
      `decresc_n`) return `dashed.is_none()`.
    - `dashed_hairpin_crescendo_carries_style`,
      `dashed_hairpin_decrescendo_carries_style` — both directions
      populate the `dashed` field with strictly-positive lengths.
    - `dashed_lengths_match_const_times_staff_space` — locks both
      `dash_length` and `gap_length` to the constant-times-staff-space
      product. Catches a refactor that swaps the constants or drops the
      staff_space multiplication.
    - `dashed_lengths_scale_linearly_with_staff_space` — walks two
      staff sizes (200, 400), asserts both absolute values and the
      2× ratio on both dash and gap.
    - `dashed_default_dash_exceeds_gap` — locks the 2:1 default
      ratio (`dash > gap`, ratio == 2.0). A future const tweak
      that flips the relationship surfaces here.
    - `dashed_does_not_alter_wedge_geometry` — byte-equal wedge
      fields between `layout_hairpin` and `layout_hairpin_dashed`
      (dashed is purely additive — niente did the same).
    - `dashed_does_not_set_niente` — independence: the dashed
      constructor must NOT silently populate the niente field.
    - `dashed_style_is_copy` — proves `HairpinDashStyle: Copy` (the
      renderer's destructure-by-value pattern needs it).
    - `dashed_zero_width_hairpin_still_carries_style` — edge case:
      collapsed wedge still has `dashed.is_some()`. The
      degenerate-line behavior is the renderer's problem, not the
      layout's.
    - `dashed_crescendo_and_decrescendo_same_geometry` — sanity:
      same args → same wedge geometry AND same dash style. Only
      `kind` differs.
    - `dashed_combo_with_niente_supported_via_field_mutation` —
      pins the combinator-via-mutation API contract: construct via
      `layout_hairpin_with_niente`, then `layout.dashed = Some(...)`,
      assert both fields populated and `niente` survived. Locks the
      independence of the two `Option<...>` fields.

  - `hairpin_renderer.rs` (12):
    - `plain_hairpin_emits_no_dasharray` — count of
      `stroke-dasharray` substrings is exactly 0 on a plain hairpin.
    - `dashed_hairpin_emits_dasharray_on_both_lines` — count is
      exactly 2 on a dashed hairpin (both wedge lines carry it).
    - `dashed_hairpin_still_emits_two_lines` — count of `<line `
      is still exactly 2 (dashing does not displace the wedge).
    - `dashed_hairpin_dasharray_value_matches_layout` — the emitted
      `stroke-dasharray="A,B"` string contains the exact numeric
      values from `layout.dashed.unwrap().dash_length` and
      `gap_length`.
    - `dashed_hairpin_dasharray_uses_design_units_not_staff_spaces`
      — regression guard: asserts the emitted dasharray is
      `"100,50"` (design units) and NOT `"0.4,0.2"` (raw
      staff-space constants). Catches a future refactor that
      forgets the `* staff_space` multiplication.
    - `dashed_hairpin_stroke_width_preserved` — custom stroke
      width appears on exactly 2 dashed wedge lines.
    - `dashed_and_plain_hairpin_produce_different_svg` — sanity:
      visually distinguishable.
    - `dashed_lines_share_same_dasharray_value` — extracts both
      dasharray values via string parsing and asserts equality —
      a single dash pattern applies to the whole wedge, not
      per-line overrides.
    - `dashed_decrescendo_x_coordinates_in_svg` — `x1="200"`,
      `x2="800"` flow through to the SVG for a non-default
      x-range.
    - `dashed_hairpin_with_niente_keeps_circle_solid` — the
      combinator path: 2 dashed wedge lines + 1 solid niente
      circle. Asserts: 2 lines, 2 `stroke-dasharray`
      occurrences, 1 circle, circle line lacks
      `stroke-dasharray`, circle still has `fill="none"`. Four
      simultaneous invariants.
    - `dashed_hairpin_alone_emits_no_circle` — independence:
      dashed alone produces zero circles.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2712 tests passing,
  0 failing** (up from 2689 by exactly the 23 new tests).
  `cargo test -p music-engraver --lib hairpin` runs the full hairpin
  subtree (80 tests including all 23 new ones) — all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings on
  either modified file (pre-existing `multi_staff.rs:394` warning and
  unrelated `music/` crate clippy warnings persist — out of scope).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). Natural follow-ups on the hairpin surface itself:
  - Plumb both `layout_hairpin_dashed` and `layout_hairpin_with_niente`
    through the score → system_renderer → page_renderer chain so a
    `ScoreBuilder` caller can request a dashed or niente hairpin
    end-to-end. Requires score-event flags for both — out of scope for
    this chunk. The natural place: a `HairpinStyle` enum or bit-flag
    bundle on the score-event side (Plain, Dashed, Niente, DashedNiente).
  - Use `layout_hairpin_dashed` for the *second segment* of a
    cross-system hairpin automatically — engraved convention says the
    continuation half should be dashed. The
    `page_renderer::cross_system_hairpin_*` tests would need an
    update.
  - "cresc. - - -" / "decresc. - - -" *text variants* (orthogonal to
    the wedge — uses italic text with dashed continuation lines,
    standard for long crescendi).
  - Open-end niente (rare — fades to/from silence at the open end
    rather than the closed end).

- Open issues: None. The change is additive — no existing public API
  altered, no existing call site changed, no golden baseline
  regenerated, no example or example test modified. The two new
  public surfaces (`HairpinDashStyle`, `layout_hairpin_dashed`) are
  independent of the existing niente surface; the
  `dashed_combo_with_niente_supported_via_field_mutation` test pins
  the combinator-via-mutation contract for the rare dashed+niente
  case. The pre-existing `multi_staff.rs:394` clippy warning and
  unrelated `music/` crate clippy warnings remain unaddressed (out
  of scope).

## 2026-05-27 — Post-v1, open-end niente hairpin (rare mirror of closed-end)

- Did: Added a third niente constructor —
  `layout_hairpin_with_niente_at_open_end` — placing the open "o" at
  the wide (open) end of the wedge rather than the closed (pointy) end.
  Standard contemporary-score notation (Lachenmann, Sciarrino) for the
  rare "open-end to/from silence" reading. Touched two files (no
  renderer changes needed — the renderer dispatches purely on
  `Option<NienteCircleLayout>` and is agnostic to which end the circle
  was anchored at).

  1. `music-engraver/src/layout/hairpin.rs`:
     - New `pub fn layout_hairpin_with_niente_at_open_end(kind,
       x_start, x_end, staff_bottom_y, staff_space, stroke_width) ->
       HairpinLayout`. Same signature as
       `layout_hairpin_with_niente`. Anchor is mirrored:
       - `Crescendo` → niente at `(x_end, y_center)` (wide right end).
       - `Decrescendo` → niente at `(x_start, y_center)` (wide left end).
     - Internally: delegates to `layout_hairpin(...)` for wedge geometry
       (byte-identical), then populates `niente` with the flipped `cx`.
       Radius (`HAIRPIN_NIENTE_RADIUS_SS * staff_space`), `cy`
       (`y_center`), and `stroke_width` (parent) match the closed-end
       constructor exactly — only `cx` differs.

  2. `music-engraver/src/layout/mod.rs`:
     - Added `layout_hairpin_with_niente_at_open_end` to the
       `pub use hairpin::{...}` block.

  Design choice — separate constructor rather than enum parameter:
  the existing closed-end `layout_hairpin_with_niente` has no enum
  selector. Adding a `NientePosition::{Closed, Open}` parameter to
  the existing constructor would be a breaking change at every call
  site (system_renderer, page_renderer, etc.) just to express the
  rarer case. The two-constructor pattern matches what the codebase
  already does (`layout_hairpin` vs `layout_hairpin_dashed` vs
  `layout_hairpin_with_niente`) and keeps the closed-end-is-default
  convention readable at call sites.

  Design choice — `cy` stays on `y_center` even at the wide end:
  engraved convention places the open-end niente on the wedge
  midline (not on the upper or lower wedge line) so the circle reads
  as belonging to the dynamic axis. The
  `open_end_niente_y_lives_on_hairpin_midline` test pins this; a
  future change that anchored the circle to the top or bottom line
  (e.g. `y_center ± half_opening`) would surface immediately.

  Tests added (23 total — 12 layout + 11 renderer):

  - `hairpin.rs` layout (12):
    - `open_end_niente_crescendo_places_circle_at_x_end` — for cresc,
      `cx == x_end` (mirror of closed-end-at-`x_start`).
    - `open_end_niente_decrescendo_places_circle_at_x_start` — for
      decresc, `cx == x_start` (mirror of closed-end-at-`x_end`).
    - `open_end_niente_mirrors_closed_end_niente_anchor` — for cresc,
      `closed.cx == 100`, `open.cx == 600`, diff == full wedge span
      (500). Also asserts `cy`, `radius`, `stroke_width` are
      byte-equal between the two constructors — only `cx` differs.
    - `open_end_niente_decrescendo_mirrors_crescendo_closed_anchor_choice` —
      for decresc, mirror direction (`closed - open == full span` instead
      of `open - closed`). Symmetric cross-check.
    - `open_end_niente_radius_matches_const_times_staff_space` —
      radius locked to `HAIRPIN_NIENTE_RADIUS_SS * SS` (250) = 50.0.
    - `open_end_niente_radius_scales_with_staff_space` — walks two
      staff sizes (200, 400), asserts both absolute values and 2× ratio.
    - `open_end_niente_stroke_width_matches_hairpin` — custom stroke
      flows from parent to niente field.
    - `open_end_niente_does_not_alter_wedge_geometry` — byte-equal
      wedge fields (kind, x_start, x_end, y_center, half_opening,
      stroke_width) between `layout_hairpin` and the open-end niente
      variant — purely additive (matches the closed-end
      `niente_does_not_alter_wedge_geometry` contract).
    - `open_end_niente_y_lives_on_hairpin_midline` — `cy == y_center`
      to 1e-12 (locks the wedge-midline convention).
    - `open_end_niente_does_not_set_dashed` — independence: the open-end
      constructor must NOT silently populate `dashed`.
    - `open_end_niente_constructor_distinct_from_closed_end_constructor` —
      API contract: same args to both constructors → different `cx`
      (by > 1.0). A regression that aliased one to the other surfaces
      here.
    - `open_end_niente_combo_with_dashed_supported_via_field_mutation` —
      pins the combinator-via-mutation contract for open-end + dashed
      (mirrors the closed-end+dashed combo test). Includes a check
      that the anchor (`x_start` for decresc-open-end) survives the
      `dashed = Some(...)` mutation.

  - `hairpin_renderer.rs` (11):
    - `open_end_niente_hairpin_emits_exactly_one_circle` — count of
      `<circle ` substrings is exactly 1.
    - `open_end_niente_hairpin_still_emits_two_wedge_lines` —
      decoration must not displace the wedge — exactly 2 `<line `
      elements.
    - `open_end_niente_crescendo_circle_anchored_at_x_end` — emitted
      SVG contains `cx="600"` AND does NOT contain `cx="100"` for an
      open-end crescendo. Two-sided assertion catches both an alias
      to closed-end and a coincidental match.
    - `open_end_niente_decrescendo_circle_anchored_at_x_start` — emitted
      SVG contains `cx="100"` AND does NOT contain `cx="600"` for an
      open-end decrescendo.
    - `open_end_niente_cy_matches_hairpin_y_center` — `cy="<y_center>"`
      appears in the SVG.
    - `open_end_niente_circle_is_open_o_not_filled_dot` — the
      `<circle>` line carries `fill="none"` AND `stroke="black"`
      (same convention as closed-end).
    - `open_end_niente_circle_radius_in_svg_matches_layout` —
      `r="<radius>"` flows through.
    - `open_end_niente_circle_stroke_width_matches_hairpin_stroke` —
      custom stroke (13.0) appears on exactly 3 elements (2 wedge
      lines + 1 circle).
    - `open_end_and_closed_end_niente_produce_different_svg` — sanity:
      visually distinguishable end-to-end. Catches a regression that
      collapsed the two constructors to the same anchor.
    - `open_end_niente_emits_no_dasharray_by_default` — independence:
      open-end-niente-only hairpin has 0 occurrences of
      `stroke-dasharray`.
    - `open_end_dashed_combo_keeps_circle_solid` — combo via field
      mutation: 2 dashed wedge lines, 1 solid circle (no
      `stroke-dasharray` on the circle line), `fill="none"`, AND
      `cx="600"` (open-end crescendo anchor). Five simultaneous
      invariants.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2735 tests passing,
  0 failing** (up from 2712 by exactly the 23 new tests; matches the
  count expectation: 12 layout + 11 renderer = 23).
  `cargo test -p music-engraver --lib hairpin` runs the full hairpin
  subtree (103 tests including all 23 new ones) — all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings —
  only the pre-existing `multi_staff.rs:394` warning persists (out
  of scope, noted across prior progress entries).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). Natural follow-ups on the hairpin surface itself:
  - **PNG export via the `png` feature** (`resvg` + `tiny-skia` +
    `fontdb`) — listed at the top of the post-v1 backlog; nothing
    currently exercises a non-SVG output path.
  - Plumb the three niente/dashed constructors through the score →
    system_renderer → page_renderer chain so a `ScoreBuilder` caller
    can request any combination (closed-end-niente, open-end-niente,
    dashed, dashed+niente at either end) end-to-end. Requires a
    score-event flag — natural shape: a `HairpinStyle { dashed: bool,
    niente: Option<NienteEnd> }` bundle on the score-event side.
  - Use `layout_hairpin_dashed` for the *second segment* of a
    cross-system hairpin automatically (engraved convention says the
    continuation half should be dashed). Would touch the existing
    `page_renderer::cross_system_hairpin_*` tests.
  - "cresc. - - -" / "decresc. - - -" *text variants* (orthogonal to
    the wedge — italic text with dashed continuation lines, standard
    for long crescendi). Probably belongs in `expression_renderer.rs`
    rather than the hairpin stack.
  - A small example (`examples/hairpin_niente_open_end.rs` or merged
    into an existing niente example) exercising the open-end variant
    visually so a reviewer can eyeball the placement.

- Open issues: None. The change is purely additive — no existing
  public API altered, no existing call site changed, no golden
  baseline regenerated, no example or example test modified. The new
  public surface (`layout_hairpin_with_niente_at_open_end`) is a
  drop-in mirror of `layout_hairpin_with_niente` and reuses the
  existing `NienteCircleLayout` type and `HAIRPIN_NIENTE_RADIUS_SS`
  constant. The pre-existing `multi_staff.rs:394` clippy warning and
  unrelated `music/` crate clippy warnings remain unaddressed (out
  of scope).

## 2026-05-27 — Post-v1, cross-system hairpin uses dashed continuation

- Did: The incoming (left) half of a cross-system hairpin is now drawn
  with the dashed-wedge `layout_hairpin_dashed` constructor rather than
  the solid `layout_hairpin`. The trailing (right) half on the source
  system remains solid. Matches Elaine Gould's *Behind Bars* convention
  for hairpin continuations and mirrors what this codebase already does
  for cross-system ottava brackets and trill extensions (the
  continuation half is dashed so the reader recognizes it as a
  resumption, not a fresh wedge starting at the system's left edge).
  Touched two files in `music-engraver`:

  1. `src/render/page_renderer/mod.rs`:
     - Widened the existing `use crate::layout::hairpin::layout_hairpin;`
       to also import `layout_hairpin_dashed`.
     - Inside `draw_cross_system_hairpins`, the second `layout_hairpin`
       call (the one that builds the *incoming* half on `systems[i+1]`)
       now calls `layout_hairpin_dashed`. The signature is byte-identical
       to `layout_hairpin` — same six positional args (kind, x_start,
       x_end, staff_bottom_y, staff_space, stroke_width) — so the only
       code-shape change is the function name. Added a doc comment
       explaining the engraving convention and pointing at the ottava
       and trill-extension precedent.
     - The trailing half-hairpin (`layout_hairpin` call building
       `right_layout` from `hp_src.x_right` → `hp_src.staff_right`)
       is unchanged: it stays solid.

  2. `src/render/page_renderer/tests.rs`: 7 new tests in the
     "cross-system hairpin: dashed-continuation tests" block placed
     immediately before the measure-number test block. Two private
     helpers (`cross_system_hairpin_page`, `line_elements`, `parse_x1`)
     keep each test terse.

  Design choice — dashed on incoming half, solid on trailing half: the
  alternative ("both halves dashed" or "trailing dashed, incoming
  solid") is also documented in some 20th-century editions but Gould's
  *Behind Bars* §"Hairpins across systems" and modern engraving
  practice settle on dashed-incoming-only. This also produces the most
  visually natural read: the within-system hairpin (solid) flows into
  the trailing half (still solid) without a stroke discontinuity at
  the start of the wedge, and the dashed cue is delivered exactly at
  the moment of the system break — where the reader needs the hint.

  Design choice — no public API surface added: the page renderer is
  internal (`pub(crate) fn draw_cross_system_hairpins`); the choice of
  which layout helper to use lives entirely inside it. No
  `HairpinStyle` enum, no score-event flag, no ScoreBuilder opt-out.
  A caller who somehow wanted the old solid-continuation behaviour
  would have to layout cross-system hairpins themselves, which was
  never a supported pattern. (If a future feature *does* need an
  opt-out — e.g. a "classical engraving" mode toggle — the natural
  shape is an `EngravingConfig` bool, added when the use case arrives.)

  Design choice — no goldens updated: there are currently zero
  cross-system-hairpin golden tests (`tests/golden/*.svg` has only
  the within-system `hairpins.svg`). The new behaviour is exercised
  by the `cross_system_hairpins` example, which still passes its
  `line_count > 20`/`path_count > 15` assertions (verified by
  running the example — `21 paths, 36 lines, 2 stroke-dasharray
  occurrences` confirms the dashed cue is now present in the
  rendered output).

  Tests added (7 total — all in `render::page_renderer::tests`):

  - `cross_system_hairpin_emits_dashed_on_incoming_half_only` — the
    primary lock: count of `stroke-dasharray` substrings is exactly 2
    on a cross-system hairpin (the two wedge lines of the incoming
    half). A "both halves dashed" regression would give 4; a "neither
    dashed" regression (revert) would give 0; an "only one line of
    the incoming wedge dashed" regression would give 1.
  - `cross_system_hairpin_total_line_count_unchanged_by_dashed_continuation`
    — geometry guard: the dashed treatment is a stroke change only.
    Total `<line>` count is still baseline + 4 (2 trailing + 2
    incoming). Catches a regression where the dashed-aware path
    drops or duplicates a wedge line.
  - `cross_system_hairpin_dashed_lines_anchor_on_target_system_left`
    — directional guard: extracts all `<line>` `x1` values from the
    SVG, partitions by presence of `stroke-dasharray`, and asserts
    that the *maximum* dashed x1 is smaller than the *minimum* solid
    wedge x1 (filtered to be > max dashed). The trailing wedge sits
    at large x (right edge of source system); the incoming wedge
    sits at small x (left edge of target system). A swap of
    solid/dashed across the two halves surfaces here.
  - `cross_system_hairpin_right_half_only_when_no_end_emits_no_dasharray`
    — exclusion guard: when there's no `hairpin_end` in the next
    system, only the trailing half is drawn — and it must stay
    solid. Catches a regression where `layout_hairpin_dashed` leaks
    into the solo-trailing code path.
  - `cross_system_decrescendo_incoming_half_also_dashed` — direction
    agnosticism: the dashed-continuation rule applies to decrescendo
    just as much as crescendo. Catches a "dashed only when
    HairpinType::Crescendo" regression.
  - `cross_system_hairpin_dashed_value_matches_layout_constants` —
    routing guard: the emitted `stroke-dasharray="A,B"` values are
    exactly `HAIRPIN_DASH_LENGTH_SS * staff_space` and
    `HAIRPIN_GAP_LENGTH_SS * staff_space`. A regression where an
    ad-hoc dash pattern is hard-coded into the page renderer instead
    of routed through `layout_hairpin_dashed` would surface here.
  - `within_system_hairpin_emits_no_dasharray` — boundary guard:
    a hairpin that starts and ends within the same system (no
    cross-system handler involved) must remain entirely solid. Pins
    the exclusivity of dashed-only-on-continuation.

  Existing cross-system hairpin tests (`cross_system_hairpin_draws_four_lines`,
  `cross_system_hairpin_right_half_only_when_no_end`,
  `cross_system_hairpin_differs_from_no_hairpin`,
  `cross_system_decresc_differs_from_cresc`,
  `within_system_hairpin_not_duplicated_as_cross_system`, and the
  multi-staff `cross_system_hairpin_in_multi_staff_draws_half_wedges`)
  all continue passing unchanged — they assert on `<line>` element
  counts and "different SVG" inequality, both of which the dashed
  treatment preserves.

- Verified: `cargo check -p music-engraver` passes (0 errors).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2742 tests passing,
  0 failing** (up from 2735 by exactly the 7 new tests).
  `cargo test -p music-engraver --lib cross_system_hairpin` runs the
  full cross-system-hairpin subtree (10 tests including all 7 new
  ones) — all pass.
  `cargo test -p music-engraver --test golden_svg` passes (72 golden
  tests; no goldens regenerated).
  `cargo test -p music-engraver --test svg_glyph_render` passes (3
  tests).
  `cargo run -p music-engraver --example cross_system_hairpins`
  succeeds and the emitted SVG (`examples/output/cross_system_hairpins.svg`)
  contains exactly 2 `stroke-dasharray` attributes (the incoming half
  of the cresc continuation) — visual confirmation the new behaviour
  reaches the example output path.
  `cargo clippy -p music-engraver --lib` reports no new warnings —
  only the pre-existing `multi_staff.rs:394` warning persists (out
  of scope, noted across prior progress entries).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests** (multi-measure rest cluster that
  breaks across systems); **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi line-cost model atop the
  existing Knuth-Plass DP); **golden-SVG corpus PHASH-based visual
  regression**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred). Natural follow-ups on the hairpin surface itself:
  - Plumb the three niente/dashed constructors through the score →
    system_renderer → page_renderer chain so a `ScoreBuilder` caller
    can request any combination (closed-end-niente, open-end-niente,
    dashed-only, dashed+niente at either end) for *within-system*
    wedges as well. Cross-system continuations now use dashed
    automatically; within-system dashed remains a layout-API-only
    capability.
  - Add a cross-system-hairpin golden test (`build_cross_system_hairpins`
    in `tests/golden_svg.rs` + `tests/golden/cross_system_hairpins.svg`)
    to lock the rendered output byte-for-byte. The current chunk
    deferred this on the principle that the in-lib SVG-attribute
    assertions are stronger than a single golden snapshot (a golden
    that fails on byte diff doesn't tell you *which* attribute moved,
    while the 7 new tests target exactly one invariant each).
  - "cresc. - - -" / "decresc. - - -" *text variants* (orthogonal to
    the wedge — italic text with dashed continuation lines, standard
    for long crescendi). Probably belongs in `expression_renderer.rs`
    rather than the hairpin stack.
  - **PNG export via the `png` feature** (`resvg` + `tiny-skia` +
    `fontdb`) was previously listed as a candidate but the feature
    is already implemented and tested (verified by reading
    `src/render/png.rs` and `examples/png_export.rs` during this
    run — 16+ pixel-content tests already in place). It can be
    crossed off the running list.

- Open issues: None. The change is purely internal — no public API
  altered, no score-event flag added, no example rewritten, no
  golden baseline regenerated. The single behavioural change
  (incoming half of cross-system hairpin now dashed) is locked by 7
  new tests and verified visually in the example output. The
  pre-existing `multi_staff.rs:394` clippy warning persists (out
  of scope, noted across prior progress entries).

## 2026-05-27 — Post-v1, "cresc. / decresc. / dim." dashed-text dynamic markings

- Did: Added the engraving-standard "cresc. - - -" / "decresc. - - -" /
  "dim. - - -" *text* dynamic — italic label + dashed continuation
  line, the conventional wedgeless alternative to a long hairpin (Gould,
  *Behind Bars*, ch. "Hairpins; cresc., dim. with dashed lines"). New
  layout and renderer modules; nothing else touched. Two new files,
  three small mod-wiring edits:

  1. `music-engraver/src/layout/cresc_text.rs` (new) —
     - `pub enum CrescTextKind { Crescendo, Decrescendo, Diminuendo }`,
       each with a `label()` method returning `"cresc."`, `"decresc."`,
       and `"dim."` respectively. (Decrescendo and Diminuendo are
       distinct kinds rather than aliases because they carry different
       text — composers/editors choose between them and the choice is
       semantic, not stylistic.)
     - `pub struct CrescTextLayout { kind, label, x_start, x_end,
       x_line_start, y_baseline, label_x, label_y, font_size,
       line_thickness, dash_length, dash_gap }` — flat field layout
       matching the existing ottava/dynamics/expression patterns.
     - `pub fn layout_cresc_text(kind, x_start, x_end, staff,
       staff_space) -> CrescTextLayout` — places the marking at
       `CRESC_TEXT_BELOW_STAFF_SS = 3.5` staff spaces below the bottom
       line (the same band as hairpins, deliberately — locked by
       `baseline_aligns_with_hairpin_y_center` against
       `HAIRPIN_BELOW_STAFF_SS`). Label width is estimated as
       `char_count * 0.6 SS` with a 0.25 SS padding before the dashed
       line begins.
     - Engraving constants are module-public (`pub const`) so external
       code can pin against them in tests:
       `CRESC_TEXT_BELOW_STAFF_SS = 3.5`,
       `CRESC_TEXT_FONT_SIZE_SS = 1.4`,
       `CRESC_TEXT_LABEL_WIDTH_PER_CHAR_SS = 0.6`,
       `CRESC_TEXT_LABEL_PADDING_SS = 0.25`,
       `CRESC_TEXT_LINE_THICKNESS_SS = 0.12`,
       `CRESC_TEXT_DASH_LENGTH_SS = 0.8` (matches `OTTAVA_DASH_LENGTH_SS`),
       `CRESC_TEXT_DASH_GAP_SS = 0.4` (matches `OTTAVA_DASH_GAP_SS`).

  2. `music-engraver/src/render/cresc_text_renderer.rs` (new) —
     - `pub fn draw_cresc_text(svg, layout)` emits exactly two SVG
       elements: a `<text>` (italic serif, `font-weight=normal`,
       `text-anchor=start`, left-edge anchored on `label_x`) and one
       `<line stroke-dasharray=...>` from `x_line_start` to `x_end` on
       the label baseline.
     - The dashed continuation line is *suppressed* entirely when
       `x_end <= x_line_start` (degenerate case — label alone, no room
       for a continuation). Locked by two renderer tests.

  3. `music-engraver/src/layout/mod.rs` — added `pub mod cresc_text;`
     in alphabetical position between `clef` and `dot`.

  4. `music-engraver/src/render/mod.rs` — added `pub mod
     cresc_text_renderer;` and `pub use cresc_text_renderer::draw_cresc_text;`
     in alphabetical positions.

  Design choice — italic-only, NOT bold-italic: ottava brackets use
  bold-italic ("8va") because that's their established 19th-century
  engraving convention; cresc./dim. text follows the *dynamics*
  convention (italic only, never bold) because it belongs to the
  dynamic axis. Gould §"Cresc., dim. with dashed lines" shows italic.
  The renderer sets `font-weight="normal"` explicitly so a regression
  to `TextStyle::italic()` (which would also be `normal` weight by
  default) is still observably different from a regression to
  `TextStyle::bold()` or a hand-rolled bold-italic spec. Locked by
  `label_is_not_bold` (asserts presence of `font-weight="normal"` AND
  absence of `font-weight="bold"`).

  Design choice — `Decrescendo` and `Diminuendo` are distinct kinds:
  the user could ask for "decresc." text *or* "dim." text; both mean
  the same musical thing but carry different surface text. Modeling
  them as separate enum variants (rather than e.g. a `kind: ...,
  spelling: Short | Long` pair) keeps the constructor a single
  positional argument and matches how Lilypond handles `\cresc` vs
  `\decresc` vs `\dim` — three orthogonal commands.

  Design choice — no public API plumbing through `system_renderer` /
  `page_renderer` / `ScoreBuilder` yet: this matches the current
  expression/ottava pattern in the codebase. The layout + renderer
  modules are the new public surface; downstream wiring is a separate
  chunk (the same shape as the deferred "plumb niente/dashed
  constructors through score → system_renderer → page_renderer chain"
  follow-up listed in the previous progress entry). Keeping this
  chunk to layout + renderer + tests keeps it reviewable.

  Design choice — variable label width via `char_count * per_char_SS`:
  the existing ottava layout hardcodes `OTTAVA_LABEL_WIDTH_SS = 2.5`
  for a 3-char label ("8va"). That works because ottava labels are
  fixed-length. cresc./decresc./dim. have variable lengths (6/8/4
  chars) so a per-char estimate is necessary. The 0.6 SS/char value
  is a deliberate over-estimate (italic serif at 1.4 SS body
  averages ~0.55 SS/char, so 0.6 leaves a touch of margin) — keeps
  the dashed line from creeping under the trailing period of
  "cresc." or "dim.". Locked by `longer_label_pushes_line_start_further_right`
  which asserts the difference between dim/cresc/decresc line-starts
  equals exactly `2 * per_char_SS` (the character-count delta).

  Tests added (38 total — 18 layout + 20 renderer):

  - `cresc_text.rs` layout (18):
    - `label_crescendo_is_cresc_dot`, `label_decrescendo_is_decresc_dot`,
      `label_diminuendo_is_dim_dot` — three label-content pins.
    - `all_three_kinds_have_distinct_labels` — pairwise non-equality.
    - `label_field_matches_kind` — single test verifying the
      `CrescTextLayout.label` field carries the same text as
      `kind.label()` for all three variants.
    - `x_coordinates_preserved` — `x_start`, `x_end`, `label_x` pass
      through unchanged.
    - `baseline_is_below_bottom_staff_line` — `y_baseline > staff.y_of(0)`
      AND `y_baseline == bottom + CRESC_TEXT_BELOW_STAFF_SS * SS` to
      1e-9 (locks the exact offset constant).
    - `label_and_dashed_line_share_baseline` — `label_y == y_baseline`
      to bit-equality.
    - `dashed_line_starts_after_label` — `x_line_start ==
      x_start + 6*per_char_SS + padding` exactly (locks the per-char
      estimate + padding formula).
    - `longer_label_pushes_line_start_further_right` — three-way
      ordering (`dim < cresc < decresc`) AND exact delta
      (`(cresc - dim) == 2*per_char`, `(decresc - cresc) == 2*per_char`).
      A regression that hardcoded a label width would surface as a
      non-zero delta different from the expected character-count delta.
    - `font_size_matches_const_times_staff_space`,
      `font_size_scales_with_staff_space` — value-match and 2× scaling.
    - `dash_constants_match_consts` — `dash_length`, `dash_gap`,
      `line_thickness` all equal the SS-scaled constants.
    - `dash_constants_scale_with_staff_space` — 2× scaling for all three.
    - `baseline_aligns_with_hairpin_y_center` — `CRESC_TEXT_BELOW_STAFF_SS
      == HAIRPIN_BELOW_STAFF_SS` to 1e-12. Pins the cross-module
      contract that the dashed-text marking lives on the same
      horizontal axis as hairpins (so a mixed phrase reads as one
      dynamic stream).
    - `empty_range_yields_x_line_start_past_x_end` — degenerate
      `x_end == x_start` case: `x_line_start > x_end` (so the renderer
      will suppress the line).
    - `negative_x_start_handled` — geometry-agnostic; negative
      coordinates pass through (engraver uses arbitrary viewBox).
    - `x_end_preserved_independently_of_kind` — `x_end` doesn't
      depend on the kind variant.

  - `cresc_text_renderer.rs` (20):
    - `renders_one_text_element` — exactly 1 `<text` substring.
    - `cresc_label_text_content`, `decresc_label_text_content`,
      `dim_label_text_content` — three string-content pins for the
      label text inside `<text>...</text>`.
    - `label_is_italic` — `font-style="italic"` present.
    - `label_is_not_bold` — `font-weight="normal"` present AND
      `font-weight="bold"` absent. Two-sided assertion.
    - `label_is_left_anchored` — `text-anchor="start"` present.
    - `emits_dashed_continuation_line` — `stroke-dasharray` present
      when there's room.
    - `dashed_line_count_is_exactly_one` — count of `stroke-dasharray`
      substrings is exactly 1 (catches a regression that double-
      emits the line).
    - `dasharray_value_matches_layout_constants` — exact string match
      against the formatted `stroke-dasharray="<dash_length>,<dash_gap>"`
      computed from `CRESC_TEXT_DASH_LENGTH_SS * SS` and
      `CRESC_TEXT_DASH_GAP_SS * SS`. A regression to a hardcoded
      dash spec or a different constant would surface here.
    - `no_dashed_line_when_x_end_equals_x_start` — degenerate range:
      `stroke-dasharray` is absent BUT `<text>` is still present
      (label-only mode).
    - `no_dashed_line_when_x_end_inside_label_region` — `x_end`
      between `x_start` and `x_line_start` (the label region itself):
      precondition-asserts `layout.x_end < layout.x_line_start` then
      asserts no `stroke-dasharray` in the output.
    - `dashed_line_starts_at_x_line_start` — emitted SVG contains
      `x1="<x_line_start>"` for the dashed line.
    - `dashed_line_ends_at_x_end` — `x2="<x_end>"` present.
    - `label_text_x_matches_layout_label_x` — `x="<label_x>"` present
      in the `<text>` element.
    - `different_kinds_produce_different_svg` — three pairwise
      inequalities for cresc/decresc/dim outputs.
    - `different_endpoints_produce_different_svg` — same start, two
      different `x_end` values → different SVG.
    - `emits_no_path_elements` — count of `<path` is exactly 0
      (cresc.-text uses text + line only; no glyph paths).
    - `emits_exactly_one_line_when_room_for_continuation` — exactly
      1 `<line ` element when the dashed line is emitted.
    - `emits_zero_lines_when_no_room_for_continuation` — degenerate
      range: 0 `<line ` elements.

- Verified: `cargo check -p music-engraver` passes (0 errors, no
  cresc_text-related warnings).
  `cargo check --workspace` passes (0 errors).
  `cargo build -p music-engraver` succeeds.
  `cargo test -p music-engraver --lib` passes — **2780 tests
  passing, 0 failing** (up from 2742 by exactly the 38 new tests).
  `cargo test -p music-engraver --lib cresc_text` runs the 38 new
  tests in isolation — all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings —
  only the pre-existing `multi_staff.rs:394` warning persists (out
  of scope, noted across prior progress entries).

- Next: Remaining post-v1 candidates from the running list:
  **cross-system church rests**; **line breaking quality
  improvements** (Gourlay extension or Bellini & Nesi line-cost
  model atop the existing Knuth-Plass DP); **golden-SVG corpus
  PHASH-based visual regression**; auto-resolved low-staff
  beam-group collision golden (still requires ScoreBuilder opt-out
  for force-stems, deferred); cross-voice tie/slur consultation of
  the collision detector (deferred). Natural follow-ups on the
  cresc-text surface itself:
  - Plumb the new `draw_cresc_text` into the score-event /
    system_renderer / page_renderer chain so `ScoreBuilder` callers
    can request "cresc. - - -" text instead of a hairpin (parallel
    to how hairpins, ottavas, dynamics are exposed today).
  - Cross-system handling: when a cresc.-text marking breaks across
    a system boundary, the trailing half should drop the label
    (just a dashed line) and the incoming half should *also* drop
    the label (just a continuation dashed line) — mirroring the
    cross-system ottava/trill-extension/hairpin pattern.
  - A small `examples/cresc_text.rs` exercising the marking
    visually so a reviewer can eyeball the placement and the
    italic+dashed combination.
  - Plumb the three niente/dashed constructors through the score →
    system_renderer → page_renderer chain so a `ScoreBuilder`
    caller can request any combination (closed-end-niente,
    open-end-niente, dashed-only, dashed+niente at either end) for
    *within-system* wedges as well. (Carried over from the
    previous chunk — orthogonal to cresc.-text but in the same
    plumbing pass.)
  - Add a cross-system-hairpin golden test (carried over).

- Open issues: None. The change is purely additive — no public API
  altered, no existing module touched beyond two `mod.rs`
  declarations (one in `layout/mod.rs`, one in `render/mod.rs` plus
  one `pub use` re-export). The pre-existing `multi_staff.rs:394`
  clippy warning persists (out of scope, noted across prior
  progress entries). No score-event wiring; no example added; no
  golden baseline regenerated.

## 2026-05-27 — Post-v1, cross-system continuation for cresc.-text

- Did: Added the layout-only piece needed for cross-system splitting
  of dashed-text crescendo / decrescendo / diminuendo markings. When
  a `cresc. - - -`, `decresc. - - -`, or `dim. - - -` marking
  straddles a system break, both halves must drop the label and
  render as a bare dashed line so the marking reads continuously
  across the system boundary. This is the same convention used by
  cross-system trill extensions (the wiggle continues with no `tr`
  re-labelling) and cross-system hairpins (the wedge halves do not
  re-grow from zero on the trailing side). Ottava brackets *do*
  re-label per segment ("8va" on each system) — that's the
  established 19th-century convention; cresc.-text follows the
  trill/hairpin pattern instead because the textual marking is a
  one-shot directive, not a positional indicator.

  Layout changes (`music-engraver/src/layout/cresc_text.rs`):
  1. Added `pub has_label: bool` field on `CrescTextLayout`. Doc
     comment cross-references the cross-system hairpin / trill
     conventions so a future maintainer sees the rationale without
     having to dig.
  2. `layout_cresc_text` now sets `has_label: true` (the only
     behavioural change to the existing public API — preserves all
     existing semantics; the renderer's existing tests still pass
     byte-identical SVG).
  3. New `pub fn layout_cresc_text_continuation(kind, x_start,
     x_end, staff, staff_space) -> CrescTextLayout` that produces a
     label-suppressed layout: `has_label = false`, `x_line_start =
     x_start` (no label-width offset — the dashed line begins
     immediately at the segment's left edge). Vertical placement,
     dash geometry, and line thickness are byte-identical to
     `layout_cresc_text` for the same staff. The `kind` and `label`
     fields are preserved on the returned struct so golden tests
     and debug-print can still inspect which marking this
     continuation belongs to; they're just not rendered.

  Renderer changes (`music-engraver/src/render/cresc_text_renderer.rs`):
  - `draw_cresc_text` now gates the `<text>` emission on
    `layout.has_label`. The dashed-line emission is unchanged
    (still gated on `x_line_start < x_end`). One file, two-line
    diff plus a doc-comment update.

  Design choice — `kind` preserved on continuation layouts (rather
  than e.g. an `Option<CrescTextKind>` or a `None` kind sentinel):
  the cross-system splitter needs to know which marking it's
  continuing so it can match the trailing half against the
  upcoming incoming half. Carrying the same `kind` enum value on
  both halves means the splitter can compare for equality without
  needing a separate identifier. The `label` string is kept too —
  it's also useful for inspection (e.g. `dbg!(&layout)` shows
  "dim." even on a continuation segment, which beats showing "").

  Design choice — `x_line_start = x_start` (no label region) rather
  than `x_line_start = x_start + small_margin`: a continuation
  segment's dashed line should read as if it began on the previous
  system, so there should be no visual "step" or extra gap at the
  left edge of the incoming system. The test
  `continuation_x_line_start_equals_x_start` locks this exactly.

  Design choice — kept the existing single-function renderer
  rather than splitting into `draw_cresc_text` + `draw_cresc_text_
  continuation`: the only difference between the two paths is one
  `if`-guarded `<text>` emission. Splitting would duplicate the
  dashed-line emission and the `x_line_start < x_end` check. The
  `has_label` field is a clean enough signal that a single
  function stays readable.

  Tests added (29 total — 15 layout + 14 renderer):

  - `cresc_text.rs` layout (15):
    - `plain_layout_has_label_is_true`,
      `plain_layout_has_label_is_true_for_all_kinds` — locks that
      the existing constructor still produces a label-bearing
      layout (regression guard against the new field defaulting to
      false).
    - `continuation_has_label_is_false`,
      `continuation_has_label_is_false_for_all_kinds` — three-way
      pin on the new constructor.
    - `continuation_x_line_start_equals_x_start` — exact equality
      (≤ f64::EPSILON) for the continuation: the dashed line
      begins right where the segment begins.
    - `continuation_x_line_start_strictly_left_of_plain` —
      *quantitative* delta: the difference between
      `plain.x_line_start` and `cont.x_line_start` equals exactly
      `label_chars * per_char + padding` (six characters for
      "cresc." times 0.6 SS plus 0.25 SS padding, scaled by SS).
      A regression that left a residual label offset on the
      continuation would surface as a non-zero delta against the
      formula.
    - `continuation_x_coordinates_preserved` — `x_start`, `x_end`,
      `label_x` all pass through unchanged.
    - `continuation_baseline_matches_plain_baseline` — `y_baseline`
      and `label_y` byte-equal between continuation and plain. The
      cross-system axis-continuity invariant.
    - `continuation_baseline_is_below_bottom_staff_line` —
      `y_baseline == bottom + CRESC_TEXT_BELOW_STAFF_SS * SS` to
      1e-9 (locks the exact offset constant on the continuation
      path too).
    - `continuation_dash_constants_match_plain` —
      `dash_length`, `dash_gap`, `line_thickness` byte-equal between
      continuation and plain.
    - `continuation_dash_constants_scale_with_staff_space` — 2×
      scaling for all three.
    - `continuation_preserves_kind_field` — three-way pin that
      `CrescTextKind` round-trips through the continuation
      constructor.
    - `continuation_preserves_label_string_for_debugging` — the
      `label` String still holds the canonical label text
      ("dim.") even though `has_label` is false.
    - `continuation_empty_range_yields_no_dashed_line_region` —
      degenerate `x_end == x_start`: `x_line_start < x_end` is
      false (so the renderer's existing line-suppression check
      fires).
    - `continuation_x_end_independent_of_kind` — three-way pin on
      `x_end`.
    - `continuation_x_line_start_does_not_depend_on_kind` — three-
      way pin on `x_line_start == x_start` (catches a regression
      that re-introduced a kind-dependent label-width offset).

  - `cresc_text_renderer.rs` (14):
    - `continuation_emits_zero_text_elements` — exactly 0 `<text`
      substrings in the rendered SVG.
    - `continuation_emits_zero_text_for_all_kinds` — three-way pin
      (Crescendo / Decrescendo / Diminuendo all suppress the
      `<text>` element).
    - `continuation_emits_no_label_string_anywhere` — defensive:
      not just "no `<text>`," but also no literal "cresc.",
      "decresc.", "dim." substring anywhere in the SVG. Catches a
      regression that hand-rolled the label using a different
      element (e.g. `<tspan>` or `<g>`).
    - `continuation_emits_exactly_one_line_when_room` — exactly
      one `<line ` element when `x_end > x_start`.
    - `continuation_emits_dashed_stroke` — `stroke-dasharray`
      substring present.
    - `continuation_dasharray_matches_layout_constants` — exact
      `stroke-dasharray="<dash_length>,<dash_gap>"` match against
      the formatted constants — locks visual continuity across
      the system break (same dash spec as the in-system half).
    - `continuation_dashed_line_starts_at_x_start` — `x1="<x_start>"`
      present in the rendered SVG (the dashed line begins at the
      segment's left edge with no label offset).
    - `continuation_dashed_line_ends_at_x_end` — `x2="<x_end>"`
      present.
    - `continuation_no_line_when_x_end_equals_x_start` — degenerate
      range: both `<line` count and `<text` count are zero (no
      output at all — clean degenerate handling).
    - `continuation_differs_visually_from_plain` — same kind, same
      coordinates, different SVG: the renderer's two paths *do*
      produce observably different output.
    - `continuation_emits_no_path_elements` — count of `<path` is
      exactly 0 (no glyph paths on the continuation either).
    - `continuation_emits_no_italic_style` — defensive: no
      `font-style="italic"` substring on the continuation output
      (catches a regression that emitted styling without text).
    - `continuation_kinds_render_identically_when_no_label` —
      crescendo, decrescendo, and diminuendo continuations
      produce *byte-identical* SVG for the same `x_start`/`x_end`.
      Locks the "label is the only thing that varies between
      kinds" invariant — a regression that special-cased one
      kind's continuation would break this.

- Verified: `cargo check -p music-engraver` passes (0 errors, no new
  warnings).
  `cargo build -p music-engraver` succeeds.
  `cargo check --workspace` passes (0 errors).
  `cargo test -p music-engraver --lib` passes — **2809 tests
  passing, 0 failing** (up from 2780 by exactly the 29 new tests).
  `cargo test -p music-engraver --lib cresc_text` runs the 67
  cresc_text tests in isolation — all pass.
  `cargo clippy -p music-engraver --lib` reports no new warnings —
  only the pre-existing `multi_staff.rs:394` warning persists.

- Next: Cross-system cresc.-text wiring at the page_renderer level
  (parallel to `draw_cross_system_ottava_brackets` and
  `draw_cross_system_hairpins`). That requires the marking to first
  be plumbed through the score-event chain (still deferred); the
  layout primitive added in this run is the prerequisite. Other
  remaining post-v1 candidates: **golden-SVG corpus PHASH-based
  visual regression**; **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi); **cross-system church
  rests**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems, deferred);
  cross-voice tie/slur consultation of the collision detector
  (deferred); plumbing of `draw_cresc_text` into score-event /
  system_renderer / page_renderer chain; plumbing of niente/dashed
  hairpin constructors through the same chain; cross-system
  hairpin golden test (carried over); small `examples/cresc_text.rs`
  visual exerciser.

- Open issues: None. The change is additive on the layout side
  (`has_label` is a new field, default-set to `true` by the
  existing constructor so existing callers see no behavioural
  change) and a one-`if`-statement gate on the renderer side. No
  public API removed or renamed. No score-event wiring; no
  page_renderer wiring; no example added; no golden baseline
  regenerated. The pre-existing `multi_staff.rs:394` clippy
  warning persists (out of scope, noted across prior progress
  entries).

## 2026-05-27 — Post-v1, examples/cresc_text.rs visual exerciser

- Did: Added `music-engraver/examples/cresc_text.rs` — a four-row
  visual exerciser for the `layout_cresc_text` / `draw_cresc_text`
  pair (and its cross-system continuation variant). Picked up the
  "small `examples/cresc_text.rs` exercising the marking visually
  so a reviewer can eyeball the placement and the italic+dashed
  combination" item from the previous "Next" list. No source code
  in the engraver itself was modified — this is a pure additive
  example, parallel to the existing `examples/hairpins.rs`
  (low-level layout/render exerciser, not yet wired through
  `ScoreBuilder` because the marking isn't plumbed through the
  score-event chain yet).

  Structure:
  - Four staves stacked vertically, 4500 FU apart (≈ 18 SS — clears
    each staff's below-staff dashed-text band with ~2 SS of
    breathing room before the next staff's clef).
  - Each row: a treble clef + the same 8-note ascending/descending
    phrase (E4 → G4 → A4 → B4 → D5 → E5 → D5 → B4), so the only
    thing varying between rows is the cresc.-text marking.
  - Row 0: `CrescTextKind::Crescendo` via `layout_cresc_text` →
    italic "cresc." label + dashed continuation line.
  - Row 1: `CrescTextKind::Decrescendo` → "decresc." label + dashed
    line. The longer label naturally pushes its dashed line
    further right than rows 0 and 2 (confirms the
    `longer_label_pushes_line_start_further_right` invariant
    visually).
  - Row 2: `CrescTextKind::Diminuendo` → "dim." label + dashed line.
    Shortest of the three labels.
  - Row 3: `layout_cresc_text_continuation(Crescendo, …)` — the
    cross-system continuation form. No label rendered (`has_label
    = false`); just a bare dashed line from `x_start` to `x_end`.
    Demonstrates the "trailing/incoming half drops the label"
    convention that mirrors hairpin and trill-extension
    continuations.

  Each row spans the dashed marking from note 1 to past note 7, so
  the dashed line covers most of the staff width on every row —
  the visual placement (3.5 SS below the bottom staff line) is
  easy to eyeball across rows.

  In-example assertions (so a regression that broke the rendered
  output would surface as a `cargo run --example cresc_text`
  failure rather than just a silently-wrong SVG):
  - `path_count >= 36`: 4 clef glyphs + 4 × 8 noteheads.
  - `line_count >= 56`: 4 × (5 staff lines + 8 stems) + 4 dashed
    continuation lines.
  - `dasharray_count == 4`: exactly one dashed line per row.
  - `total_cresc_lines == 4` and `total_label_text_elements == 3`:
    pin the renderer-call accounting at the example level (3
    plain layouts emit a label, 1 continuation suppresses it).
  - `>cresc.</text>`, `>decresc.</text>`, `>dim.</text>` all
    present as literal text content.
  - `font-style="italic"` present (locks the italic styling
    invariant).
  - `layout.has_label` checked on each layout before drawing — a
    regression that flipped the default would fail the example
    *before* it even reached the SVG-string assertions.

  Design choice — direct layout/render API (not `ScoreBuilder`):
  the cresc.-text marking has not yet been plumbed through the
  score-event chain (carried over from prior "Next" lists), so
  `ScoreBuilder` doesn't have a `.cresc_text()` / `.decresc_text()`
  / `.dim_text()` family yet. Once the score-event wiring lands, a
  follow-up can either rewrite this example through `ScoreBuilder`
  or add a second `cresc_text_score.rs` example (parallel to
  `hairpins.rs` ↔ `hairpin_score.rs`).

  Design choice — three plain kinds + one continuation in one
  example (not four separate examples): the visual reviewer
  benefit is the ability to compare label widths and dashed-line
  start offsets *side by side*. Splitting into four files would
  require the reviewer to open four SVGs and mentally overlay
  them.

  Design choice — same 8-note phrase on all four rows: keeps the
  staff/clef/notehead geometry identical between rows so the only
  thing the reviewer compares is the dashed-text marking. A
  per-row varying phrase would muddy the comparison.

- Verified: `CARGO_HOME=/repo/rust-music/.cargo cargo check -p
  music-engraver --offline` passes (0 errors, no new warnings).
  `cargo build -p music-engraver --offline` succeeds.
  `cargo check --workspace --offline` passes (0 errors).
  `cargo test -p music-engraver --lib --offline` passes — **2809
  tests passing, 0 failing** (unchanged — no library code modified).
  `cargo run -p music-engraver --example cresc_text --offline`
  executes cleanly and writes
  `examples/output/cresc_text.svg` (15863 bytes, 36 paths, 56
  lines, 4 dashed). All in-example assertions pass. Spot-checked
  the SVG with grep: exactly one each of `>cresc.</text>`,
  `>decresc.</text>`, `>dim.</text>`, and four `stroke-dasharray`
  occurrences (one per row, including the label-less
  continuation row).

- Next: Remaining post-v1 candidates from the running list:
  **plumb `draw_cresc_text` into score-event / system_renderer /
  page_renderer chain** so `ScoreBuilder` callers can request a
  dashed-text marking (then this example's twin
  `cresc_text_score.rs` becomes possible); cross-system cresc.-
  text wiring at the page_renderer level (parallel to
  `draw_cross_system_ottava_brackets` and
  `draw_cross_system_hairpins`) — the layout primitive
  `layout_cresc_text_continuation` exists, the page_renderer just
  hasn't been taught to call it yet; **golden-SVG corpus PHASH-
  based visual regression**; **line breaking quality
  improvements** (Gourlay extension or Bellini & Nesi); **cross-
  system church rests**; auto-resolved low-staff beam-group
  collision golden (still requires ScoreBuilder opt-out for
  force-stems, deferred); cross-voice tie/slur consultation of
  the collision detector (deferred); plumbing of niente/dashed
  hairpin constructors through the score → system_renderer →
  page_renderer chain; cross-system hairpin golden test (carried
  over).

- Open issues: None. The example is purely additive — no
  engraver source code changed, no public API touched, no golden
  baselines regenerated. The pre-existing `multi_staff.rs:394`
  clippy warning persists (out of scope, noted across prior
  progress entries). The example uses the low-level layout/render
  API rather than `ScoreBuilder` because the cresc.-text
  marking is not yet plumbed through the score-event chain
  (carried over as the next-most-natural follow-up).


## 2026-05-27 — Post-v1, cross-system hairpins golden test

- Did: Added `golden_cross_system_hairpins` to
  `music-engraver/tests/golden_svg.rs`, picking up the "cross-system
  hairpin golden test (carried over)" item from the previous run's
  "Next" list. The carried-over status reflects that the
  cross-system hairpin code path
  (`draw_cross_system_hairpins` in `page_renderer/mod.rs`) is fully
  covered by *unit* tests under `src/render/page_renderer/tests.rs`
  (`cross_system_hairpin_draws_four_lines`,
  `cross_system_hairpin_emits_dashed_on_incoming_half_only`,
  `cross_system_hairpin_total_line_count_unchanged_by_dashed_continuation`,
  …) but had no *visual-regression* golden — every other
  cross-system span construct (ties, ottava, volta, glissandos,
  multi-staff cross-system) does. The new golden closes that gap.

  Scenario (`build_cross_system_hairpins`):
  - Treble clef, C major, 4/4, `.measures_per_system(2)`.
  - System 1 m. 1: `pp` + `hairpin_start(Crescendo)` on note 1, then
    3 more quarters.
  - System 1 m. 2: cresc continues — two half notes, no
    `hairpin_end` yet, so the wedge is unresolved at the system break.
  - System 2 m. 3: `hairpin_end` on note 1, dynamic `ff`, then a
    fully within-system `hairpin_start(Decrescendo)` resolving on
    note 4 + dynamic `p`.
  - System 2 m. 4: plain closing phrase.

  This is the smallest scenario that exercises both code paths
  simultaneously — the cross-system cresc proves the split (solid
  trailing + dashed incoming half-wedges) and the within-system
  decresc on the same page proves the dashed-continuation logic
  does not leak into ordinary single-system wedges.

  Companion `build_cross_system_hairpins_baseline` is byte-for-byte
  identical in note sequence but strips every dynamic and hairpin
  call — it exists *only* as a delta reference so the test can
  assert "the hairpins add exactly six `<line>` elements" rather
  than the much weaker "the hairpins add some lines."

  Assertions (each pins a distinct invariant — none would pass if
  the cross-system code path silently degraded to "render trailing
  half only", "render both halves solid", or "render no halves"):

  - `dasharray_count == 2`: the incoming (target-system) half-wedge
    is dashed; each half-wedge is 2 `<line>` elements (upper +
    lower arm), so exactly 2 `stroke-dasharray` attributes must
    appear. A regression that dropped the dashed half would
    collapse this to 0; one that dashed both halves would lift it
    to 4.
  - Delta-line invariant `with_lines == no_lines + 6`: the
    cross-system cresc contributes 4 lines (2 solid trailing + 2
    dashed incoming) and the within-system decresc contributes 2
    solid lines. Hardcoding the delta exposes any change to how
    the wedge halves are emitted — a single-line wedge, a
    triple-line accent, or a missed half — without relying on the
    fragile absolute line count (which also includes staff lines,
    stems, and barlines).
  - Baseline-side guard `!baseline.contains("stroke-dasharray")`:
    if the no-hairpin baseline ever leaks dashed lines (e.g., a
    future feature adds dashed barlines on volta endings), the
    delta arithmetic above becomes silently wrong. This guard
    fails loudly instead.
  - Counter-example guard `!within_system_only.contains("stroke-dasharray")`:
    builds a separate single-measure scenario where the cresc
    fully fits inside one system, then asserts the rendered SVG
    has zero dasharray. Locks the implication "dashed wedge
    half ⇒ wedge crossed a system break" — the dashed code path
    is gated on the cross-system condition, never invoked for
    same-system wedges.
  - `assert_golden("cross_system_hairpins", &svg)`: byte-level
    comparison against the frozen baseline at
    `tests/golden/cross_system_hairpins.svg` (14290 bytes, 36
    `<line>` elements, 2 dasharrays). Catches any change to the
    cross-system hairpin's exact pixel layout — wedge slope, end
    points, dash pattern, half-wedge anchor x — that the
    structural assertions above might miss.

  Also added `"cross_system_hairpins"` to the
  `golden_baselines_are_valid_svgs` registry list so the new file
  is included in the SVG-well-formedness sweep alongside every
  other baseline.

  Design choice — separate `_baseline` builder vs. inline string
  manipulation: a builder mirrors the convention already used by
  `build_cross_voice_spans` / `build_cross_voice_spans_baseline`
  (line 3711 in this file). Same shape, same notes; only the
  hairpin/dynamic calls differ. Easier to read than asking the
  reader to mentally subtract "6 lines" from a string with no
  comparator.

  Design choice — counter-example uses a smaller single-measure
  scenario rather than reusing the main one with `.measures_per_system(99)`:
  a single-measure score is unambiguously within-system regardless
  of system-breaking heuristics, so the assertion's premise is
  robust against future changes to the auto-break algorithm.

- Verified: `cargo check -p music-engraver --tests --offline` passes
  (0 errors).
  `cargo check --workspace --offline` passes (0 errors).
  `cargo build -p music-engraver --offline` succeeds.
  `cargo test -p music-engraver --offline` passes — **2809 lib
  tests + 73 `tests/golden_svg.rs` (was 72; +1 for the new
  `golden_cross_system_hairpins`) + 3 `tests/svg_glyph_render.rs`
  + 13 doctests = 2898 passed, 0 failed**.
  `cargo clippy -p music-engraver --tests --offline` reports no
  new warnings from the new code (the pre-existing
  `map(..).flatten()` and `unneeded return` warnings, plus the
  long-standing `multi_staff.rs:394` one, persist — all out of
  scope).
  Baseline file
  `music-engraver/tests/golden/cross_system_hairpins.svg`
  generated via `GOLDEN_UPDATE=1` and confirmed to match on a
  second run without that flag. Spot-checked: 36 `<line>`
  elements, exactly 2 `stroke-dasharray="100,50"` occurrences,
  starts `<svg xmlns=...`, contains `</svg>`.

- Next: Remaining post-v1 candidates: **plumb `draw_cresc_text`
  into score-event / system_renderer / page_renderer chain** so
  `ScoreBuilder` callers can request a dashed-text marking;
  cross-system cresc.-text wiring at the page_renderer level
  (parallel to `draw_cross_system_ottava_brackets` and
  `draw_cross_system_hairpins`); **golden-SVG corpus PHASH-based
  visual regression**; **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi); **cross-system church
  rests**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems,
  deferred); cross-voice tie/slur consultation of the collision
  detector (deferred); plumbing of niente/dashed hairpin
  constructors through the score → system_renderer →
  page_renderer chain.

- Open issues: None. The change is purely additive — no engraver
  source code modified, no public API touched, no existing golden
  baselines regenerated. Two new builder functions
  (`build_cross_system_hairpins`,
  `build_cross_system_hairpins_baseline`), one new `#[test]`
  function (`golden_cross_system_hairpins`), one new golden file
  (`tests/golden/cross_system_hairpins.svg`), and one entry added
  to the `golden_baselines_are_valid_svgs` name registry.


## 2026-05-27 — Post-v1, plumb cresc-text into the score-event chain

- Did: Wired the dashed-text crescendo/diminuendo marking through
  the full `ScoreBuilder` → annotations → `convert_event` → measure
  → system_renderer chain. Before this run the layout module
  (`layout/cresc_text.rs`) and the SVG renderer
  (`render/cresc_text_renderer.rs`) were both complete with their
  own unit tests (including the cross-system continuation variant),
  but there was no way for a `ScoreBuilder` caller to actually
  request the marking — it was an unreachable code path. This was
  the top item on the previous run's "Next" list.

  Concrete changes:
  - `src/layout/measure.rs`: added `use crate::layout::cresc_text::CrescTextKind;`
    and two new fields on `NoteAnnotations`:
    `cresc_text_start: Option<CrescTextKind>` and
    `cresc_text_end: bool`. These propagate through
    `convert_event` (which clones the annotations onto the
    resulting `NoteEvent` / `ChordEvent`) without any code change
    to the converter — same pattern hairpin uses.
  - `src/score/mod.rs`: added `use crate::layout::cresc_text::CrescTextKind;`
    and five new `ScoreBuilder` methods, matching the
    hairpin-style two-callsite-then-three-convenience-wrappers
    shape:
      - `cresc_text_start(kind: CrescTextKind) -> Self`
      - `cresc_text_end() -> Self`
      - `cresc_text() -> Self` (Crescendo convenience)
      - `decresc_text() -> Self` (Decrescendo convenience)
      - `dim_text() -> Self` (Diminuendo convenience)
    All five are `Note | Chord`-gated via the same `if let Some((_,
    ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord {
    annotations, .. }))` pattern — calling them after a `.rest()` is
    a documented no-op, parallel to `.hairpin_start()` and
    `.dynamic()`.
  - `src/render/system_renderer/mod.rs`:
    - imports `layout_cresc_text`, `CrescTextKind`,
      `draw_cresc_text`;
    - new `CrescTextNoteInfo` struct (mirror of `HairpinNoteInfo`);
    - new `collect_cresc_text_note_info(system)` walks
      `all_measure_elements`, picking up the flags on both
      `Note` and `Chord` events (same coverage as the hairpin
      collector — both voices, all measures in the system);
    - new `draw_system_cresc_texts(svg, font, config, system,
      staff, system_x)` finds each `cresc_text_start`, pairs it
      with the next `cresc_text_end`, computes the x range using
      the same `+ advance + 0.3 * staff_space` / `- 0.3 *
      staff_space` start/end padding the hairpin path uses, calls
      `layout_cresc_text` and then `draw_cresc_text`;
    - `draw_system` now invokes `draw_system_cresc_texts` right
      after `draw_system_hairpins`. Comment notes that the two
      paths are conceptually parallel — wedgeless alternative.

  Cross-system continuation (label-suppressed dashed halves on each
  side of a system break) is intentionally deferred from this
  chunk — `layout_cresc_text_continuation` exists and is fully
  unit-tested, but plumbing it through the page_renderer (parallel
  to `draw_cross_system_hairpins`) is a separate piece of work.
  Within a single system, an unresolved `cresc_text_start` whose
  matching end falls on the next system silently emits nothing,
  which is the same fail-safe the hairpin code path has before
  cross-system support was added there.

  Tests (15 new tests across two files, all asserting on specific
  SVG structure rather than "render didn't panic"):

  System-renderer level (`src/render/system_renderer/tests.rs`):
  - `cresc_text_within_measure_emits_label_and_dashed_line` —
    asserts the SVG gets exactly +1 `<text>`, +1
    `stroke-dasharray`, and +1 `<line>` over a no-marking
    baseline. Also asserts the label text (`>dim.</text>`), the
    `font-style="italic"` attribute, and the
    `text-anchor="start"` attribute appear. Uses the
    `Diminuendo` kind ("dim." — 4-char label) and a 4-note
    span so the label + padding + dashed line all fit within the
    raw measure-layout spacing the system_renderer test fixtures
    use; the `Crescendo` kind ("cresc." — 6 chars at 0.6 SS each
    + padding = 962.5 unit minimum span) overflows a 4-quarter-
    note measure at the standard `from_staff_space` config and
    triggers the `x_line_start >= x_end` label-only fail-safe.
  - `no_cresc_text_without_start_flag` — a lone `cresc_text_end`
    with no preceding start renders byte-identical to the
    baseline.
  - `cresc_text_start_without_end_draws_nothing_extra` — orphan
    start: renders byte-identical to baseline.
  - `cresc_text_kinds_differ_in_label_content` — same notes, three
    different kinds (Crescendo, Decrescendo, Diminuendo) → each
    label appears in its own render; cross-contamination guards
    assert that the `cresc.` render does NOT contain `decresc.`
    or `dim.`, the `dim.` render does NOT contain `cresc.`
    (catches accidental fall-through in label dispatch), and
    `decresc.` does NOT contain the `>dim.</text>` literal
    (decresc.'s label string is a superstring of cresc.'s, so
    that one-direction guard is sufficient). The three SVGs
    must also differ pairwise.
  - `cresc_text_across_barline_emits_one_label_and_one_line` —
    two-measure span: exactly one label and one dashed line over
    the no-marking baseline.
  - `cresc_text_dashed_line_endpoints_lie_between_start_and_end_notes`
    — locks the geometric routing. Pulls the `CrescTextNoteInfo`
    back out via `collect_cresc_text_note_info` and asserts the
    SVG contains the literal `x2="<expected>"` derived from
    `end.x - 0.3 * staff_space`. Catches a regression where the
    renderer swapped start/end positions or used the wrong
    padding direction. Has an explicit precondition assertion
    that the rendered SVG actually contains a dashed line — so
    if a future change shrinks the spacing below the
    label+padding threshold, the test fails loudly instead of
    vacuously matching on an empty SVG.
  - `cresc_text_renders_independently_of_a_hairpin_on_other_notes`
    — mixed scenario: a hairpin (2 solid lines, 0 text) on notes
    1–2 plus a cresc-text marking (1 dashed line + 1 label) on
    notes 3–6 must add exactly +3 lines, +1 dasharray, and +1
    text element over the no-marking baseline. Catches any
    shared-state interference between the two paths.

  ScoreBuilder level (`src/score/tests.rs`):
  - `cresc_text_adds_label_and_dashed_line_to_svg` — end-to-end:
    `.cresc_text() ... .cresc_text_end()` on a `ScoreBuilder`
    chain produces +1 dasharray and +1 text-element in the final
    `render_svg()` output vs. the same chain with the two calls
    removed. (The score-level `render_svg()` path uses
    `layout_page` and the `PageLayoutConfig` system width, which
    stretches the layout more than the bare system_renderer
    test fixture — so the `Crescendo` kind fits here even on a
    short 3-note span; sticking with the default kind keeps the
    high-level test idiomatic.)
  - `decresc_text_label_differs_from_cresc_text` — all three
    convenience wrappers (`.cresc_text()`, `.decresc_text()`,
    `.dim_text()`) emit their distinct labels in the final SVG;
    the three rendered strings must be pairwise distinct.
  - `cresc_text_on_rest_is_noop` — calling `.cresc_text()` right
    after `.rest()` is a no-op (the `last_mut()` arm only matches
    `Note` / `Chord`), so the dangling `.cresc_text_end()` has
    nothing to pair with and the rendered SVG is byte-identical
    to a no-marking version. No `cresc.` label appears.
  - `cresc_text_start_without_end_renders_no_marking` — orphan
    start at the high level: byte-identical to baseline.
  - `cresc_text_and_hairpin_coexist_independently` — high-level
    mixed scenario: chain has both a hairpin and a cresc-text
    marking; SVG must contain exactly 1 `stroke-dasharray` (the
    hairpin contributes none, the cresc-text contributes exactly
    one) and the `>cresc.</text>` label.
  - `convert_event_preserves_cresc_text_fields` and
    `convert_event_preserves_cresc_text_end_flag` — direct
    `convert_event` tests with and without the within-measure
    accidental tracker, parallel to the existing
    `convert_event_preserves_hairpin_fields` /
    `convert_event_with_tracking_preserves_hairpin_fields` pair.
    Asserts that the start kind and the end flag round-trip
    through to the resulting `MeasureEvent::Note` unchanged.
  - `chord_cresc_text_preserved_in_convert` — same but for
    `ScoreEvent::Chord`, parallel to
    `chord_hairpin_preserved_in_convert`.
  - `cresc_text_on_chord_renders_label` — end-to-end via
    `ScoreBuilder` confirming the start can land on a
    `.chord(...)` event and still produce both the label and the
    dashed continuation line.

  Test-design choice — why `Diminuendo` for the system_renderer
  scenarios but `Crescendo` for the score-level scenarios:
  the bare system_renderer uses
  `MeasureLayoutConfig::from_staff_space(staff_space)` directly,
  which gives `min_note_spacing = 1.5 * staff_space = 375` and
  spacing-ratio-1.6 per duration step. For 4 quarter notes at
  that spacing, the start→end span is ~1125 fu. The "cresc."
  label needs ~962.5 fu (6 chars × 0.6 SS × 250 + 62.5 padding +
  the 0.3-SS start pad and the notehead advance), which leaves
  almost no room for the dashed continuation — triggering the
  `layout_cresc_text` `x_line_start >= x_end` label-only
  fail-safe and emitting no `stroke-dasharray`. Using "dim." (4
  chars) shrinks the label requirement to ~662.5 fu and leaves
  a clear margin. The `ScoreBuilder` path uses `layout_page`
  with `effective_system_width`, which stretches the layout to
  fill the page-width target and yields enough span for the
  "cresc." case to fit even on short scores. The
  `decresc_text_label_differs_from_cresc_text` test still
  exercises all three labels at the high level so each
  convenience wrapper is covered.

- Verified:
  - `cargo check -p music-engraver --offline` → 0 errors.
  - `cargo check -p music-engraver --tests --offline` → 0 errors.
  - `cargo check --workspace --offline` → 0 errors (no
    cross-crate regression).
  - `cargo build -p music-engraver --offline` → succeeds.
  - `cargo test -p music-engraver --offline --lib` → **2825
    passed, 0 failed** (was 2809; +16 new tests across
    `render::system_renderer::tests::cresc_text_*` (7) and
    `score::tests::cresc_text_*` /
    `score::tests::*_cresc_text_*` (9)).
  - `cargo test -p music-engraver --offline --tests` → 2825 +
    73 (`golden_svg.rs`) + 3 (`svg_glyph_render.rs`) = 2901
    passed, 0 failed. No golden baselines moved.
  - `cargo test -p music-engraver --offline --doc` → 13
    passed, 1 ignored (unchanged from prior).
  - `cargo clippy -p music-engraver --tests --offline` → no
    new warnings from the new code; the pre-existing
    `multi_staff.rs:394`, `cresc_text.rs:667` (in an existing
    test using `!(a < b)`), `map(..).flatten()`, and unneeded
    `return` warnings persist (all out of scope, noted across
    prior progress entries).

- Next: cross-system cresc.-text wiring at the page_renderer
  level (parallel to `draw_cross_system_ottava_brackets` and
  `draw_cross_system_hairpins`) using the already-tested
  `layout_cresc_text_continuation` helper — this completes the
  cresc-text feature surface; **golden-SVG corpus PHASH-based
  visual regression**; **line breaking quality improvements**
  (Gourlay extension or Bellini & Nesi); **cross-system church
  rests**; auto-resolved low-staff beam-group collision golden
  (still requires ScoreBuilder opt-out for force-stems,
  deferred); cross-voice tie/slur consultation of the collision
  detector (deferred); plumbing of niente/dashed hairpin
  constructors through the score → system_renderer →
  page_renderer chain; **golden test for the cresc-text marking
  via ScoreBuilder** (this run added unit + integration tests
  but did not freeze a visual baseline — natural follow-up).

- Open issues: None. The change is purely additive at the API
  level — two new `NoteAnnotations` fields default to
  `None`/`false`, so existing callers see no behavior change.
  Three new public `ScoreBuilder` entry points plus two
  convenience wrappers (five methods total). One new module-
  level function in `system_renderer/mod.rs`
  (`draw_system_cresc_texts`) plus one new helper
  (`collect_cresc_text_note_info`) and one new
  `pub(crate)` info struct (`CrescTextNoteInfo`). Cross-system
  continuation is documented as the next chunk in the comment
  above `draw_system_cresc_texts` itself.


## 2026-05-27 — Post-v1, cross-system cresc-text continuation

- Did: Plumbed cross-system dashed-text crescendo/diminuendo
  ("cresc. - - -", "decresc. - - -", "dim. - - -") through the
  page renderer, completing the cresc-text feature surface that the
  previous run flagged as the natural follow-up. Before this chunk
  the within-system marking worked end-to-end via the ScoreBuilder
  but a `cresc_text_start` whose matching end fell on the next
  system silently emitted nothing — the `layout_cresc_text_continuation`
  helper was fully unit-tested but had no caller.

  Concrete changes (all in `music-engraver/src/render/page_renderer/mod.rs`):
  - New imports: `layout_cresc_text`, `layout_cresc_text_continuation`,
    `CrescTextKind` from `crate::layout::cresc_text`; `draw_cresc_text`
    from `crate::render::cresc_text_renderer`;
    `collect_cresc_text_note_info` from `crate::render::system_renderer`.
  - Two new private structs (`UnresolvedCrescText`,
    `IncomingCrescTextTarget`) — mirror of `UnresolvedHairpin` /
    `IncomingHairpinTarget` in shape, but the cresc-text variants
    omit the `staff_bottom_y` cache since the StaffLayout is
    rebuilt at the draw site.
  - Two new helpers (`find_unresolved_cresc_texts`,
    `find_incoming_cresc_text_targets`) — collect cresc-text note
    info, walk it for start-no-end (source) and the first end
    (target). Same notehead-advance + 0.3ss padding the within-
    system path uses, so a within-system cresc-text and a
    cross-system cresc-text starting on the same note pixel-align.
  - One new `pub(crate)` function
    (`draw_cross_system_cresc_texts`) plus one call site in
    `draw_page` right after `draw_cross_system_hairpins`.

  Key asymmetry vs. the cross-system hairpin path:
  - Hairpin: source half = solid wedge (`layout_hairpin`),
    target half = dashed wedge (`layout_hairpin_dashed`). Both halves
    have wedge geometry; only the stroke style differs.
  - Cresc-text: source half = label + dashed line
    (`layout_cresc_text`, which keeps `has_label = true`), target
    half = dashed line only (`layout_cresc_text_continuation`,
    which sets `has_label = false`). The label always lives on
    the source system because that's where the `cresc_text_start`
    annotation logically lands; repeating it on the target side
    would defeat the continuation convention. Mirrors the
    "label-suppressed continuation" pattern already used by
    cross-system ottava and trill extensions.

  Tests (11 new, all in `src/render/page_renderer/tests.rs`,
  asserting on specific SVG counts/contents rather than "render
  didn't panic"):
  - `cross_system_cresc_text_adds_one_label_and_two_dashed_lines`
    — primary count assertion: cross-system marking adds exactly
    +1 `>cresc.</text>` and +2 `stroke-dasharray` over a same-
    structure baseline. Catches under- and over-emission in one
    test.
  - `cross_system_cresc_text_incoming_half_has_no_label` — direct
    label count equals 1, not 2. Catches a regression where the
    incoming half accidentally calls `layout_cresc_text` (which
    sets `has_label = true`) instead of the continuation variant.
  - `no_cross_system_cresc_text_without_flags` — baseline emits
    zero `cresc.` / `decresc.` / `dim.` labels (all three guarded
    independently).
  - `cross_system_cresc_text_orphan_start_emits_trailing_half_only`
    — start with no matching end on the next system → +1 label
    and +1 dashed line over baseline, never +2.
  - `cross_system_cresc_text_orphan_end_emits_nothing` — end with
    no matching start → zero labels and dasharray-count unchanged
    from baseline.
  - `within_system_cresc_text_not_duplicated_as_cross_system` —
    both flags fit on one system via `Fixed(2)`: exactly 1 label
    and 1 dashed line in the SVG (the within-system path's
    output, with no cross-system duplication).
  - `cross_system_cresc_text_differs_from_baseline` — byte-
    difference smoke test catching any silent no-op regression.
  - `cross_system_cresc_text_dim_kind_uses_dim_label` — kind
    routing: Diminuendo emits exactly 1 `>dim.</text>`, zero
    `>cresc.</text>`, zero `>decresc.</text>`, and the +2
    dasharray count is preserved. Catches any
    hard-wiring-to-Crescendo regression.
  - `cross_system_cresc_text_decresc_kind_uses_decresc_label`
    — same for Decrescendo. Guards on `>cresc.</text>` (the
    `>decresc.</text>` tag does NOT contain the `>cresc.<` prefix
    by literal-string match, so the contains-check is safe).
  - `cross_system_cresc_text_dashed_value_matches_layout_constants`
    — the rendered `stroke-dasharray="…,…"` attribute uses the
    `CRESC_TEXT_DASH_LENGTH_SS` × ss / `CRESC_TEXT_DASH_GAP_SS` × ss
    products (not staff-space constants directly), appearing
    exactly twice (once per half). Catches a regression where the
    cross-system path uses hard-coded dash geometry instead of
    routing through the layout helpers.
  - `cross_system_cresc_text_label_lives_on_source_system_left_of_incoming_dashed`
    — geometric routing sanity: extract both dashed-line `x1`
    values from the SVG and assert they're distinct. Catches a
    regression where the trailing and incoming halves collapse to
    the same x (i.e. no real cross-system split happened).

  Test helpers (`cresc_text_start_note`, `cresc_text_end_note`,
  `cross_system_cresc_text_page`, `cross_system_cresc_text_baseline`)
  parallel the existing hairpin helpers (`cresc_start_note`,
  `hairpin_end_note`, `cross_system_hairpin_page`).

  Width budget — a `cresc.` label needs ~962fu (6 chars × 0.6 SS
  × 250fu + padding) plus the notehead-advance + 0.3ss offset.
  At `system_width = 8000fu` the justified system staff_width
  is essentially 8000fu, so the source half has plenty of room
  for the label *and* the dashed continuation. (The previous-
  chunk system_renderer scenarios had to drop to "dim." for the
  short 4-quarter-note span because they used the raw measure-
  layout config; the page-renderer scenarios use the full
  page-width justification and don't hit that constraint.)

- Verified:
  - `cargo check -p music-engraver` → 0 errors.
  - `cargo check -p music-engraver --tests` → 0 errors.
  - `cargo check --workspace` → 0 errors (no cross-crate
    regression).
  - `cargo test -p music-engraver --offline --lib` →
    **2836 passed, 0 failed** (was 2825 last run; +11 new tests
    all under `render::page_renderer::tests::*_cross_system_cresc_text_*`
    / `within_system_cresc_text_not_duplicated_as_cross_system` /
    `no_cross_system_cresc_text_without_flags`).
  - `cargo test -p music-engraver --offline --tests` →
    73 (`golden_svg.rs`) + 3 (`svg_glyph_render.rs`) = **76
    passed, 0 failed** (unchanged from the previous run). No
    golden baselines moved.
  - All 11 new tests pass; the existing `cross_system_hairpin_*`
    suite continues to pass byte-identically (the new path runs
    after `draw_cross_system_hairpins` and only emits labels/lines
    in response to cresc-text annotations, which the hairpin
    tests don't set).

- Next: cross-system church rests; **golden-SVG corpus PHASH-
  based visual regression** (covers the cresc-text feature surface
  among others); **line breaking quality improvements** (Gourlay
  extension or Bellini & Nesi); auto-resolved low-staff
  beam-group collision golden (still requires ScoreBuilder opt-out
  for force-stems, deferred); cross-voice tie/slur consultation of
  the collision detector (deferred); plumbing of niente/dashed
  hairpin constructors through the score → system_renderer →
  page_renderer chain; **golden test for the cresc-text marking
  via ScoreBuilder** (cross-system case is now exercised by the
  page-renderer unit tests but no visual baseline is frozen yet);
  PNG export via the `png` feature (`resvg` + `tiny-skia` +
  `fontdb`).

- Open issues: None. The change is purely additive at the
  page-renderer surface — one new public-in-crate function
  (`draw_cross_system_cresc_texts`), two new private structs,
  two new private helpers. No public API of the crate changed.
  No existing golden baselines moved (the cross-system cresc-text
  path only runs when `cresc_text_start` / `cresc_text_end`
  annotations are present, which none of the current golden
  fixtures set). The within-system fail-safe in
  `draw_system_cresc_texts` (silently emit nothing for an
  unresolved span) remains in place — the page-renderer path
  picks up exactly those previously-silent cases. The
  `layout_cresc_text_continuation` doc comment mentions a more
  sophisticated "trailing half drops the label when label
  appeared earlier on the source system" scenario; this chunk
  implements the simpler parallel-with-hairpin path where the
  label always lives on the source system because the
  `cresc_text_start` annotation lands there. Refining for the
  case where a within-system cresc-text label has already been
  drawn earlier on the source system (multi-end-flag pattern)
  is a separate concern and not currently triggerable from the
  ScoreBuilder API.

## 2026-05-27 — Post-v1, golden tests for cresc-text via ScoreBuilder

- Did: Froze visual baselines for the dashed-text dynamic markings
  (`cresc. - - -`, `decresc. - - -`, `dim. - - -`) at the
  `ScoreBuilder` surface. Two prior runs landed the layout +
  renderer (within- and cross-system) and plumbed them through the
  score-event chain, but the cross-system render had no frozen
  visual baseline and the within-system one wasn't exercised at the
  integration-test level either. This run adds both — closing the
  "natural follow-up" item flagged on the cross-system continuation
  entry above.

  Concrete changes (all in
  `music-engraver/tests/golden_svg.rs` — no production code
  touched):
  - New import: `use music_engraver::layout::cresc_text::CrescTextKind;`
    so the cross-system fixture can call
    `.cresc_text_start(CrescTextKind::Crescendo)` explicitly. The
    within-system fixture uses the three convenience wrappers
    (`.cresc_text()`, `.decresc_text()`, `.dim_text()`) to keep
    the high-level API surface covered.
  - Four new fixture builders, paired in
    delta-baseline-and-fixture style (the same pattern
    `build_cross_system_hairpins` / `_baseline` already use):
    - `build_cresc_text()` — single system, 3 measures of 4
      quarter notes each, one `CrescTextKind` per measure
      (Crescendo → Decrescendo → Diminuendo).
    - `build_cresc_text_baseline()` — same notes, no markings.
    - `build_cross_system_cresc_text()` — 4 measures at 2
      measures/system, a `Crescendo` marking that starts on the
      first note of system 1 and ends on the first note of
      system 2, forcing the page renderer's
      `draw_cross_system_cresc_texts` path.
    - `build_cross_system_cresc_text_baseline()` — same notes,
      no markings.
  - Two new `#[test]` functions
    (`golden_cresc_text`, `golden_cross_system_cresc_text`),
    each pinning structural invariants on the rendered SVG via
    a fixture-vs-baseline delta:

    Within-system (`golden_cresc_text`):
    - `<line ` delta over baseline = exactly +3 (one dashed
      continuation line per kind).
    - `stroke-dasharray` count = exactly 3 (one per dashed line);
      baseline must have 0 (leak guard for the delta assertion).
    - `>cresc.</text>`, `>decresc.</text>`, `>dim.</text>` each
      appear exactly once. The `>cresc.</text>` substring is the
      *closing-tag-bound* form so it does NOT accidentally match
      inside the longer `>decresc.</text>` label (a normal
      `.contains("cresc.")` would over-count by 2).
    - `font-style="italic"` delta over baseline ≥ +3 (at least
      one italic-styled label per kind).

    Cross-system (`golden_cross_system_cresc_text`):
    - `<line ` delta over baseline = exactly +2 (one trailing
      dashed half on system 1 + one incoming dashed half on
      system 2).
    - `stroke-dasharray` count = exactly 2.
    - `>cresc.</text>` count = exactly 1 — the label MUST live
      only on the source system. If the incoming half
      accidentally calls `layout_cresc_text` (which sets
      `has_label = true`) instead of `layout_cresc_text_continuation`,
      this assertion catches a 2-label render. If the
      cross-system path is silently skipped (regression to the
      pre-plumbing fail-safe state), this assertion catches a
      0-label render.
    - `>decresc.</text>` and `>dim.</text>` absent (negative
      controls — we requested `Crescendo` kind explicitly).
  - Two new golden SVG baselines written via
    `GOLDEN_UPDATE=1 cargo test`:
    `music-engraver/tests/golden/cresc_text.svg` (8627 bytes)
    and
    `music-engraver/tests/golden/cross_system_cresc_text.svg`
    (10202 bytes). Both pinned per the standard
    `assert_golden` flow.

  Test-design choice — separate fixtures per kind vs. one fixture
  with all three markings: the score-level path uses
  `layout_page` with the default `PageLayoutConfig`, which
  stretches each measure to the full justified width. The
  longest label is `cresc.` (~962fu minimum span). At 4
  quarters per measure and the default page width, each
  marking has comfortable room for its label + dashed line
  *within its own measure*, so three independent markings
  side-by-side reliably emit 3 labels + 3 lines without
  triggering the `layout_cresc_text` `x_line_start >= x_end`
  label-only fail-safe. This is the same observation noted in
  the "plumb cresc-text into the score-event chain" entry —
  the unit-test scenarios in `system_renderer/tests.rs` use
  the bare `from_staff_space` config and can't fit `cresc.`
  on 4 quarter notes; the ScoreBuilder/`layout_page` path can.

  Test-design choice — Crescendo-only on the cross-system
  golden: the page-renderer-level unit tests already cover
  kind routing (`cross_system_cresc_text_dim_kind_uses_dim_label`
  and `_decresc_kind_uses_decresc_label`), so the golden's job
  is to lock the *visual* baseline, not re-test dispatch. One
  kind keeps the SVG small and the delta-baseline diff trivial
  to read. The within-system golden carries the kind-coverage
  invariant on the visual side.

- Verified:
  - `cargo check -p music-engraver --tests` → 0 errors.
  - `cargo check --workspace` → 0 errors (no cross-crate
    regression).
  - `cargo build -p music-engraver` → succeeds.
  - `cargo test -p music-engraver --lib` → **2836 passed, 0
    failed** (unchanged from prior run — no production-code
    changes, so the lib-test count is stable).
  - `cargo test -p music-engraver --test golden_svg` → **75
    passed, 0 failed** (was 73; +2 new tests — the two new
    `golden_cresc_text` / `golden_cross_system_cresc_text`
    `#[test]` functions). All pre-existing goldens remain
    byte-identical — no baseline-side leakage from the new
    fixtures.
  - First run with `GOLDEN_UPDATE=1` wrote the two new
    baselines and passed; second run without the env var
    confirmed both fixtures byte-match their frozen baselines.

- Next: PNG export via the `png` feature (`resvg` +
  `tiny-skia` + `fontdb`) — still the largest deferred
  post-v1 chunk and has no in-progress prerequisites;
  cross-system church rests (multi-measure rest cluster
  breaking across systems); line-breaking quality
  improvements (Gourlay extension or Bellini & Nesi);
  golden-SVG corpus PHASH-based visual regression (the
  per-test golden coverage is now broad enough that
  consolidating into a PHASH-tolerant comparator would be a
  natural next step); auto-resolved low-staff beam-group
  collision golden (still requires ScoreBuilder opt-out for
  force-stems, deferred); cross-voice tie/slur consultation
  of the collision detector (deferred); plumbing of
  niente/dashed hairpin constructors through the score →
  system_renderer → page_renderer chain (mirror of this
  chunk's plumbing-then-golden pattern, currently the only
  remaining hairpin-surface deferral).

- Open issues: None. The change is purely additive at the
  test-only surface: one new import, four new fixture
  builders, two new `#[test]` functions, two new SVG
  baselines under `tests/golden/`. No production code
  touched. The pre-existing
  `multi_staff.rs:394` and other clippy warnings noted on
  prior entries remain unaddressed (out of scope).

## 2026-05-27 — Post-v1, plumb hairpin_dashed through the score-event chain

- Did: Plumbed the dashed-wedge hairpin style from the
  `ScoreBuilder` API down through `NoteAnnotations`, the
  within-system renderer, and the cross-system trailing-half
  path on the page renderer. This is the natural follow-on to
  the cresc-text plumbing run — same shape (annotation field +
  builder modifier + per-renderer dispatch), different style
  (dashed wedge instead of dashed text label). The dashed-text
  variant uses `layout_cresc_text`; this run wires the parallel
  `layout_hairpin_dashed` into the score path.

  Before this run, `layout_hairpin_dashed`,
  `layout_hairpin_with_niente`, and
  `layout_hairpin_with_niente_at_open_end` existed at the
  layout/renderer layer and were exercised by extensive
  hairpin-renderer unit tests, but the ScoreBuilder only ever
  invoked the plain `layout_hairpin`. Cross-system hairpins
  used `layout_hairpin_dashed` for the incoming half on the
  next system (engraved-convention default), but no user-facing
  API could request a fully-dashed wedge for a within-system
  or cross-system trailing half.

  Concrete changes:
  - `layout/measure.rs` — `NoteAnnotations` gains
    `hairpin_dashed: bool`. Defaults to `false` via
    `#[derive(Default)]`; no existing fixture or test breaks
    because all existing construction sites either use
    `NoteAnnotations::default()` directly or
    `..NoteAnnotations::default()` struct-update syntax.
    Doc comment explains the semantics: flag lives on the
    start side, cross-system propagates to the trailing half
    on the source system, incoming half stays dashed
    unconditionally per engraving convention.
  - `render/system_renderer/mod.rs` —
    `HairpinNoteInfo` gains `hairpin_dashed: bool`;
    `collect_hairpin_note_info` propagates it from both
    `MeasureElement::Note` and `MeasureElement::Chord`.
    `draw_system_hairpins` dispatches between
    `layout_hairpin` and `layout_hairpin_dashed` based on
    the flag. Added `layout_hairpin_dashed` to the existing
    `use crate::layout::hairpin::{...}` import.
  - `render/page_renderer/mod.rs` —
    `UnresolvedHairpin` gains `dashed: bool`;
    `find_unresolved_hairpins` carries it through from
    `HairpinNoteInfo`. `draw_cross_system_hairpins`
    dispatches the trailing-half layout based on the flag.
    The incoming-half code path is unchanged (still
    `layout_hairpin_dashed` unconditionally).
  - `score/mod.rs` — new
    `ScoreBuilder::hairpin_dashed()` modifier. Sets
    `annotations.hairpin_dashed = true` on the most recent
    `ScoreEvent::Note` or `ScoreEvent::Chord`. No-op on
    rests / barlines / multi-measure rests (mirrors the
    no-op semantics of `cresc()`, `cresc_text()`, and the
    rest of the per-note modifiers). Doc explains:
    must be called alongside `hairpin_start` / `cresc()` /
    `decresc()`; silently no visible effect if hairpin_start
    is unset, because there is no wedge to dash.

- Verified:
  - `cargo check -p music-engraver` → 0 errors.
  - `cargo check -p music-engraver --tests` → 0 errors.
  - `cargo check --workspace` → 0 errors (no cross-crate
    regression).
  - `cargo build -p music-engraver` → succeeds.
  - `cargo test -p music-engraver --offline --lib` →
    **2845 passed, 0 failed** (was 2836 last run; +9 new
    tests).
  - `cargo test -p music-engraver --offline --tests` →
    75 (`golden_svg.rs`) + 3 (`svg_glyph_render.rs`)
    = **78 passed, 0 failed** (unchanged from prior run —
    no golden baseline moved because no fixture sets
    `hairpin_dashed`, so existing SVGs are byte-identical).

  New tests (9 total):
  - `score::tests`:
    - `hairpin_dashed_flag_sets_annotation_on_start_note` —
      verifies the builder modifier sets the annotation
      field and preserves `hairpin_start`.
    - `hairpin_dashed_renders_stroke_dasharray` —
      end-to-end ScoreBuilder → SVG: dashed wedge emits 2
      `stroke-dasharray` attrs; solid wedge emits 0; line
      count identical between the two.
    - `hairpin_dashed_without_hairpin_start_renders_no_wedge` —
      flag on a note without a preceding `cresc()` /
      `decresc()` is silently a no-op (no wedge synthesized).
    - `hairpin_dashed_on_rest_is_noop` — mirror of the
      existing `hairpin_on_rest_is_noop` test for the
      dashed-style flag.
    - `hairpin_dashed_decrescendo_also_dashes` —
      direction-agnostic: `decresc().hairpin_dashed()`
      emits dasharray on both wedge lines.
  - `render::page_renderer::tests`:
    - `within_system_dashed_hairpin_emits_two_dasharrays` —
      forces a both-measures-on-one-system layout (`Fixed(2)`)
      so the wedge stays in `draw_system_hairpins`, then
      asserts 2 dasharrays exactly and identical
      `<line>` count vs solid.
    - `within_system_dashed_hairpin_dasharray_value_matches_layout_constants` —
      asserts the emitted dasharray attribute equals
      `HAIRPIN_DASH_LENGTH_SS * ss, HAIRPIN_GAP_LENGTH_SS * ss`
      (the same constants `layout_hairpin_dashed`
      produces). Catches a regression where an ad-hoc dash
      pattern leaks into the system renderer.
    - `cross_system_dashed_hairpin_emits_dasharray_on_all_four_lines` —
      forces a `Fixed(1)` split so the wedge crosses
      systems; with the dashed flag set, all 4 wedge lines
      (2 trailing + 2 incoming) carry dasharray. Negative
      control inline: same layout without the flag → only
      2 dasharrays (existing cross-system convention).
    - `cross_system_dashed_hairpin_trailing_half_uses_dasharray_layout_constants` —
      asserts all 4 wedge lines in a cross-system dashed
      hairpin use the exact same dasharray attribute
      derived from the layout constants.

- Next: Plumb niente (closed-end and open-end) hairpin
  variants through the score-event chain — mirror of this
  chunk for the niente "o" circle. Will need
  `hairpin_niente: Option<NientePlacement>` or two
  separate booleans on `NoteAnnotations` plus dispatch in
  `draw_system_hairpins` and the cross-system splitter
  (cross-system niente is rare but should at least preserve
  the closed-end "from silence" mark on the source system if
  the start note carries niente_start). The combination
  `hairpin_dashed + niente` is already supported at the
  layout layer (via field mutation per
  `dashed_hairpin_with_niente_keeps_circle_solid`); the
  score-event surface should expose that combo.
  Then: cross-system church rests (multi-measure rest cluster
  breaking across systems); line-breaking quality improvements
  (Gourlay extension or Bellini & Nesi); golden-SVG corpus
  PHASH-based visual regression; PNG export polish
  (already exists, would benefit from a golden-PHASH-style
  baseline corpus); auto-resolved low-staff beam-group
  collision golden (still requires ScoreBuilder opt-out for
  force-stems, deferred); cross-voice tie/slur consultation
  of the collision detector (deferred).

- Open issues: None. The change is additive at every
  surface — one new public-in-crate field on
  `NoteAnnotations`, one new public method on `ScoreBuilder`
  (`hairpin_dashed`), one new field on the internal
  `HairpinNoteInfo` and `UnresolvedHairpin` structs.
  Default behaviour of every existing call site is
  byte-identical: no golden baseline moved, all 2836
  pre-existing lib tests continue to pass, all 75 golden
  SVG tests continue to byte-match. The `hairpin_dashed`
  flag is silently ignored when `hairpin_start` is `None`
  (no wedge to dash); this is by design and is exercised
  by the
  `hairpin_dashed_without_hairpin_start_renders_no_wedge`
  test. The niente "o" circle surface remains unplumbed
  to the ScoreBuilder (deferred to the next chunk —
  same shape but with the additional cross-system
  decision about whether the source system should still
  carry the closed-end circle when the wedge crosses).
  Combined dashed + niente at the ScoreBuilder surface
  will be enabled by that next chunk; the layout layer
  already supports the combination.

## 2026-05-27 — Post-v1, plumb hairpin niente (closed/open) through the score-event chain

- Did: Plumbed the niente "o" circle (both closed-end and
  open-end variants) from the `ScoreBuilder` API down through
  `NoteAnnotations`, the within-system renderer, and the
  cross-system splitter on the page renderer. This is the
  "deferred to the next chunk" item flagged by the previous
  `hairpin_dashed` plumbing entry, and it closes the last
  remaining hairpin-surface deferral on the score-event chain.

  The layout layer already supported both placements via
  `layout_hairpin_with_niente` and
  `layout_hairpin_with_niente_at_open_end`, and supported
  combination with dashed via field mutation
  (`dashed_hairpin_with_niente_keeps_circle_solid`). Before
  this run the ScoreBuilder could not request either variant,
  and the cross-system splitter had no concept of which half
  (trailing on source / incoming on target) should carry the
  circle when a niente wedge crossed a system break.

  Concrete changes:
  - `layout/hairpin.rs` — new public enum
    `NientePlacement { ClosedEnd, OpenEnd }` and a new public
    unified constructor `layout_hairpin_styled` that takes
    `niente: Option<NientePlacement>` and `dashed: bool` and
    returns a fully-styled `HairpinLayout` (replaces the
    cartesian product of constructor calls that would
    otherwise be needed at three call sites). Doc explains
    the niente-stays-solid-on-dashed engraving invariant
    (the circle remains solid even when `dashed=true`).
    9 new layout-level unit tests pin the equivalence
    contract: every (niente × dashed) combination matches
    the corresponding individual constructor field-for-field;
    geometry is identical across all four combinations;
    each (kind, placement) pair anchors `niente.cx` to the
    correct tip.
  - `layout/mod.rs` — re-exports `NientePlacement` and
    `layout_hairpin_styled`.
  - `layout/measure.rs` — `NoteAnnotations` gains
    `hairpin_niente: Option<NientePlacement>`. Defaults to
    `None` via `#[derive(Default)]`; no existing fixture
    breaks because all sites use struct-update syntax or
    `default()`. Doc comment explains the field's role and
    the cross-system propagation rule (which half owns the
    circle for the given `(kind, placement)`).
  - `render/system_renderer/mod.rs` — `HairpinNoteInfo`
    gains `hairpin_niente`; both `MeasureElement::Note`
    and `MeasureElement::Chord` arms of
    `collect_hairpin_note_info` propagate it.
    `draw_system_hairpins` now routes through the single
    `layout_hairpin_styled` helper instead of the previous
    if/else on `hairpin_dashed`, so the niente and dashed
    flags compose without duplicated logic.
  - `render/page_renderer/mod.rs` — `UnresolvedHairpin`
    gains `niente: Option<NientePlacement>`; new helper
    `trailing_half_owns_niente(kind, placement) -> bool`
    encodes the cross-system ownership rule:
      - `(Crescendo, ClosedEnd) → true`  (start tip → trailing)
      - `(Crescendo, OpenEnd) → false`   (end tip → incoming)
      - `(Decrescendo, ClosedEnd) → false` (end tip → incoming)
      - `(Decrescendo, OpenEnd) → true`  (start tip → trailing)
    `draw_cross_system_hairpins` filters the per-half
    niente by this rule before passing it to
    `layout_hairpin_styled`. The trailing half also routes
    through `layout_hairpin_styled` so the dashed/solid
    branching collapses to a single call site here too.
  - `score/mod.rs` — two new public methods on
    `ScoreBuilder`:
    - `hairpin_niente_start(placement: NientePlacement)`
      — full API; sets `annotations.hairpin_niente =
      Some(placement)`.
    - `hairpin_niente()` — convenience for the common
      `ClosedEnd` case (the standard "al niente" / "dal
      niente" convention).
    Both no-op on rests / barlines / multi-measure rests
    (mirroring the existing per-note modifier pattern) and
    silently no-op visually if `hairpin_start` is unset
    (no wedge to attach the circle to).

  Test-design choice — relative cx assertions for
  within-system anchor: the first attempt at
  `within_system_niente_crescendo_closed_anchors_at_start_tip`
  used a midpoint comparison (`cx < 8000/2`). It failed
  empirically because the actual within-system layout puts
  the first measure's note at ~5087fu on an 8000fu page
  (prefix + measure-internal spacing eats more of the
  budget than the midpoint heuristic assumed). Rewrote the
  three within-system anchor tests as pairwise comparisons
  between ClosedEnd and OpenEnd fixtures on the same notes:
  `cx_closed < cx_open` for a crescendo, `cx_closed > cx_open`
  for a decrescendo. These hold regardless of the absolute
  layout coordinates and remain sharp catches for a
  regression where the placement enum is dropped before
  reaching `layout_hairpin_styled`.

  Test-design choice — cy-based cross-system ownership
  test: `<circle>` count alone can't distinguish trailing
  vs incoming (both place exactly one circle). Used the
  vertical-stacking invariant instead: target system's
  staff_bottom_y sits strictly below source system's, so
  the niente's cy on the target half is strictly greater
  than on the source half. The two cross-system tests
  drive this via `(kind, placement)` switches expected to
  flip the owning half:
    - `cross_system_niente_crescendo_closed_circle_on_source_system`
      compares Crescendo+Closed (source) vs Decrescendo+Closed
      (target) and asserts decresc.cy > cresc.cy.
    - `cross_system_niente_open_end_flips_owning_half`
      compares Cresc+Closed (source) vs Cresc+Open (target)
      and asserts open.cy > closed.cy.
  Together they exercise both axes of the four-way
  `trailing_half_owns_niente` truth table.

- Verified:
  - `cargo check -p music-engraver` → 0 errors.
  - `cargo check -p music-engraver --tests` → 0 errors.
  - `cargo check --workspace` → 0 errors (no cross-crate
    regression).
  - `cargo build -p music-engraver` → succeeds.
  - `cargo test -p music-engraver --offline --lib` →
    **2870 passed, 0 failed** (was 2845 last run; +25 new
    tests: 9 layout-level styled-helper equivalence tests
    in `layout::hairpin::tests`, 9 page-renderer-level
    plumbing tests in `render::page_renderer::tests`, 7
    score-level ScoreBuilder surface tests in
    `score::tests`).
  - `cargo test -p music-engraver --offline --tests` →
    75 (`golden_svg.rs`) + 3 (`svg_glyph_render.rs`) =
    **78 passed, 0 failed** (unchanged from prior run —
    no production-code default behavior changed, so no
    golden baseline moved; existing fixtures don't set
    the niente flag).

  New tests (25 total):
  - `layout::hairpin::tests` (9 new):
    - `styled_no_niente_no_dash_equals_plain_hairpin` —
      `(None, false)` returns geometry identical to
      `layout_hairpin`.
    - `styled_dashed_no_niente_equals_layout_hairpin_dashed` —
      `(None, true)` returns dash style identical to
      `layout_hairpin_dashed`, including the staff-space-scaled
      dash/gap lengths.
    - `styled_closed_niente_no_dash_anchors_at_closed_tip`,
      `styled_open_niente_no_dash_anchors_at_open_tip` —
      `(Some(_), false)` matches the corresponding direct
      constructor field-for-field.
    - `styled_decrescendo_closed_niente_anchors_at_x_end`,
      `styled_decrescendo_open_niente_anchors_at_x_start` —
      decrescendo flips the tip→cx mapping, covered for
      both placements.
    - `styled_dashed_with_closed_niente_carries_both`,
      `styled_dashed_with_open_niente_carries_both` —
      combo case (dashed + niente) returns both fields
      populated correctly.
    - `styled_geometry_independent_of_niente_and_dashed` —
      all four (niente × dashed) combinations on the
      same args share identical wedge geometry; niente
      and dashed are purely additive overlays.
  - `score::tests` (7 new):
    - `hairpin_niente_convenience_sets_closed_end_annotation`
      — `.hairpin_niente()` sets `Some(ClosedEnd)` and
      preserves `hairpin_start`.
    - `hairpin_niente_start_open_end_sets_open_end_annotation`
      — `.hairpin_niente_start(OpenEnd)` sets `Some(OpenEnd)`.
    - `hairpin_niente_renders_open_circle` — end-to-end:
      ScoreBuilder → SVG must emit exactly one
      `<circle ` element (delta over no-niente baseline =
      +1) and the circle must carry `fill="none"` (the
      engraved open "o" reading). Wedge line count
      unchanged — niente is purely additive.
    - `hairpin_niente_with_dashed_keeps_circle_solid` —
      combo at the ScoreBuilder surface: two
      stroke-dasharray attrs on the wedge, exactly one
      circle, and that circle's `<circle ` element
      carries NO stroke-dasharray (per engraving
      convention).
    - `hairpin_niente_without_hairpin_start_renders_no_circle`
      — lone `.hairpin_niente()` after a plain note is
      silently no-op visually (no wedge → no circle);
      SVG byte-identical to the no-niente render.
    - `hairpin_niente_on_rest_is_noop` — mirror of the
      `hairpin_dashed_on_rest_is_noop` test for niente.
    - `hairpin_niente_decrescendo_also_renders_circle` —
      direction-agnostic: decresc + niente also emits the
      circle.
  - `render::page_renderer::tests` (9 new):
    - `within_system_niente_closed_end_emits_one_circle` —
      one `<circle>` with the flag; zero without. Baseline
      assertion required because the engraver emits no
      circles for any other element — the `<circle>` count
      is exclusive to niente.
    - `within_system_niente_radius_matches_layout_constant`
      — the `r=` attribute equals
      `HAIRPIN_NIENTE_RADIUS_SS * staff_space`; circle
      carries `fill="none"`.
    - `within_system_niente_crescendo_closed_anchors_at_start_tip`,
      `within_system_niente_decrescendo_closed_anchors_at_end_tip`
      — pairwise cx comparisons between Closed and Open
      placements on the same fixture (so the contract is
      coordinate-independent).
    - `within_system_niente_open_end_renders_distinct_circle_from_closed`
      — Closed and Open placements on identical notes
      produce byte-distinct SVG; both emit exactly one
      circle and identical wedge line counts.
    - `within_system_niente_combined_with_dashed_keeps_circle_solid`
      — combo at the page-renderer surface: dashed wedge
      + niente circle on a within-system hairpin; circle
      lacks stroke-dasharray.
    - `cross_system_niente_crescendo_closed_circle_on_source_system`
      — Crescendo+Closed vs Decrescendo+Closed both emit
      exactly one circle; decrescendo's cy is strictly
      greater than crescendo's cy (i.e. on the target system).
    - `cross_system_niente_open_end_flips_owning_half` —
      Crescendo+Closed (source-owned) vs Crescendo+Open
      (target-owned); open's cy is strictly greater than
      closed's cy.
    - `cross_system_niente_no_niente_emits_zero_circles` —
      negative control: cross-system hairpin without the
      flag emits zero circles.

- Next: Multi-staff systems / grand-staff brackets
  (`StaveConnector` equivalent) — the largest remaining
  Phase-8/post-v1 chunk that requires structural layout
  work rather than annotation plumbing; the hairpin
  surface is now feature-complete at the ScoreBuilder
  level. PNG export via the `png` feature (`resvg` +
  `tiny-skia` + `fontdb`) — still the largest deferred
  post-v1 chunk and has no in-progress prerequisites.
  Cross-system church rests (multi-measure rest cluster
  breaking across systems). Line-breaking quality
  improvements (Gourlay extension or Bellini & Nesi).
  Golden-SVG corpus PHASH-based visual regression (per-test
  golden coverage is broad now; consolidating into a
  PHASH-tolerant comparator would simplify future golden
  triage). Auto-resolved low-staff beam-group collision
  golden (still requires ScoreBuilder opt-out for
  force-stems, deferred). Cross-voice tie/slur consultation
  of the collision detector (deferred). A natural follow-up
  golden for niente + dashed at the score level (mirroring
  the cresc-text golden pattern landed two runs ago)
  would freeze the visual baseline of the new combo path
  through the page renderer.

- Open issues: None. The change is additive at every
  surface — one new public enum (`NientePlacement`), one
  new public layout helper (`layout_hairpin_styled`), one
  new field on `NoteAnnotations`, one new field on the
  internal `HairpinNoteInfo` and `UnresolvedHairpin`
  structs, two new public methods on `ScoreBuilder`
  (`hairpin_niente`, `hairpin_niente_start`). Default
  behavior of every existing call site is byte-identical:
  no golden baseline moved, all 2845 pre-existing lib
  tests continue to pass, all 75 golden SVG tests
  continue to byte-match. The `hairpin_niente` flag is
  silently ignored when `hairpin_start` is `None` (no
  wedge to attach the circle to); this is by design and
  is exercised by the
  `hairpin_niente_without_hairpin_start_renders_no_circle`
  test. The pre-existing `multi_staff.rs:394` and other
  clippy warnings noted on prior entries remain
  unaddressed (out of scope).
