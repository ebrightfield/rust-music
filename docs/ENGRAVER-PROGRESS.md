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
