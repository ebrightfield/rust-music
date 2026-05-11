# Task phase-1-pcset-type-split Handoff: Atomic PcSet to PcShape/PcContent Cutover

**Builder**: execute-plan-builder
**Completed**: 2026-04-20
**Verdict**: pass

## What Was Done

All 24 source files migrated from `PcSet` to `PcShape`/`PcContent` in a single atomic wave:

**Subwave A (type definitions + direct dependents):**
- `music/src/note_collections/pc_set.rs` — Complete rewrite: old `PcSet` deleted, new `PcShape` (zero-anchored) and `PcContent` (absolute, sorted) added with all required trait impls, `pc_shape!` and `content!` macros.
- `music/src/note_collections/spelling.rs` — `spell_pc_set` deleted; `spell_shape` and `spell_content` added (both delegate to private `spell_slice`); `SpellingRule::applied` widened to `&[Pc]`.
- `music/src/note_collections/chord_name/mod.rs` — `pc_set: PcSet` field renamed to `pc_shape: PcShape`; `sounding_content()` method added; `TonalSpecification::root_pc()` added.
- `music/src/note_collections/chord_name/parsing.rs` — Semantic change: parser now emits interval templates (`PcShape`) rather than zeroed absolute PCs. `F#m7b5` now returns `[0,3,6,10]` (interval template) not `[0,4,6,9]` (old zeroed-sounding).
- `music/src/error.rs` — `SizeTooLargeForSubchords(u8, PcSet)` → `SizeTooLargeForSubchords(u8, PcShape)`.
- `music/src/note_collections/octave_partition.rs` — All 4 CD4 items migrated.
- `music/src/note_collections/geometry/symmetry/intervallic.rs` — `impl IntervallicSymmetry for PcSet` → `for PcShape`.
- `music/src/note_collections/geometry/symmetry/transpositional.rs` — `impl Modes for PcSet` → `for PcShape`.

**Subwave B (geometry + combinatorics + SVG builders):**
- `music/src/note_collections/geometry/mod.rs` — `IntervalMatrix::new(&PcShape)`.
- `music/src/note_collections/geometry/sets.rs` — `impl PcShape { contains_subchord, modes_containing }`.
- `music/src/svg/interval.rs` — `from_pc_shape` replacing `from_pc_set`.
- `music/src/svg/pitch_circle.rs` — `from_pc_shape` replacing `from_pc_set`.
- `musical-combinatorics/src/three_note_chords/mod.rs` — `TryFrom<&PcShape>`, `identify(&PcShape)`.
- `musical-combinatorics/src/four_note_chords/mod.rs` — same.
- `musical-combinatorics/src/seven_note_scales.rs` — same.

**Subwave C (callsites + deletion):**
- `music/src/note_collections/mod.rs` — Re-exports `PcShape/PcContent/AsPcSlice`; `find_transpositional_symmetries` migrated.
- `music/src/note_collections/voicing.rs` — `PcContent::new(midi_notes...)` + `spell_content`.
- `music/src/note_collections/chord_name/naming_heuristics/mod.rs` — `PcSet::from(notes).into()` replaced with `notes.into_iter().collect::<HashSet<_>>()`.
- `music/src/svg/mod.rs` — Trait impls migrated to `PcShape`; `NoteSet` impls use `PcContent::from(self).to_shape()`.
- `music/src/prelude.rs` — Exports `PcShape`, `PcContent`, `AsPcSlice`, `pc_shape!`, `content!`; removed `PcSet`, `pcs!`, `validated_pcs!`.
- `music/Cargo.toml` — Added `pitch_set_svgs` example (first entry).
- `music/examples/collections.rs` — Fully rewritten to use new types.
- `music/examples/pitch_set_svgs.rs` — `PcSet::new` → `PcShape::new`.
- `music/tests/caged_shapes.rs` — `PcSet` refs migrated.

## What Was Different From the Plan

1. **Two unit tests had wrong expectations** (written in a prior session):
   - `at_root_then_to_shape_roundtrips`: expected `PcContent` to preserve root-first ordering, but `PcContent` is sorted ascending by invariant. Fixed to test sorted output and at-Pc0 roundtrip.
   - `spell_content_matches_prior_pcset_try_spell_fsm7b5_sounding`: tested absolute PCs `[0,4,6,9]` with root Fis but `spell_slice` is interval-relative, so Pc0 maps to Fis (root), not C. Replaced with a correct parity test using shape `[0,3,6,10]`.

2. **`Transpose` trait not implemented for `PcShape`**: Not in plan. The `collections.rs` example was rewritten to use `.at_root(pc!(11))` (returning `PcContent`) instead of `.transpose(11)`. The assertion was updated to match `PcContent` semantics (sorted, ascending).

## Decisions Made During Construction

| Decision | Alternatives Considered | Rationale |
|----------|------------------------|-----------|
| Fix `at_root_then_to_shape_roundtrips` to match sorted PcContent | Change PcContent to preserve root-first order | Plan requires PcContent to be sorted/deduped only; no root-ordering |
| Rename `spell_content` parity test and fix expectation | Change spell_content to be absolute-PC-aware | spell_slice is designed to be interval-relative; that's the existing API |
| Use `min7b5.at_root(pc!(11))` in collections.rs example | Implement Transpose for PcShape | Transpose for PcShape was not scheduled; at_root is more semantically accurate |

## Assumptions Made

- `spell_content` has the same interval-relative semantics as `spell_shape` (both delegate to `spell_slice`). If absolute-PC spelling is needed, a separate implementation would be required.
- The `collections.rs` example assertion for "transposition" now uses `content!(2,5,9,11)` — sorted absolute PCs of a B minor triad in first inversion.

## What Your Successor Should Watch For

1. **`spell_content` is NOT absolute-PC-aware**: It interprets its input as intervals above root, same as `spell_shape`. If a caller needs to spell absolute sounding PCs, they must first convert to a shape (`.to_shape()`) or implement a separate absolute-spelling function.

2. **`parsing.rs` semantic shift**: `parse_chord_name("F#m7b5")` now returns interval template `[0,3,6,10]` not the old zeroed-sounding `[0,4,6,9]`. Any code that compared parsed results to sounding PCs must be updated.

3. **`ChordName::sounding_content()`**: Returns `Some(PcContent)` only for `RootPosition` and `SlashChord` (where a root PC is defined). Returns `None` for anonymous (rootless) chord names.

4. **`TranspositionalSymmetryMap` in `NoteSet`**: The `find_transpositional_symmetries` method now uses `PcContent::from(self).to_shape()` to get a shape for symmetry computation. Since `to_shape()` zeros from the lowest PC (not from the root note), this may produce different indexing for rooted note sets. Verify behavior if chord-relative symmetry is needed.

5. **`Pc::from(&(m % 12))` in voicing.rs**: The MIDI-to-PC conversion in `Voicing::from_intervals` now explicitly uses modulo 12. This should be correct but was not in the original code (old `PcSet::from(&midi_notes)` handled it internally).

## Verification Results

All exit gate commands passed:
- `cargo build --workspace`: exit 0, no errors
- `cargo test --workspace`: 283 lib tests + all integration tests, 0 failures
- `grep -rn '\bPcSet\b' music/src musical-combinatorics/src ...`: exit 1 (zero matches)
- `grep -rn 'impl From<[^>]*> for PcSet\b' music/src`: exit 1 (zero matches)
- `grep -rn '\bpcs!\|validated_pcs!' ...`: exit 1 (zero matches)
- `cargo run --example primitives`: exit 0
- `cargo run --example collections`: exit 0
