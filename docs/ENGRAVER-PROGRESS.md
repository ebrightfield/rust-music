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
