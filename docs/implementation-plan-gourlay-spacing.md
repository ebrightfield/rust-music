# Implementation Plan — Gourlay Spring-Rod Spacing

**Status:** proposed
**Scope decision:** full model (measure + system), per port-plan §6
**Compat decision:** replace the power-of-ratio model; keep the 78 golden baselines
green (re-baseline deliberately via `GOLDEN_UPDATE=1` only if shifts are real and
correct).

## 1. Problem

The engraver's horizontal spacing deviates from the port plan in two places:

- `layout/measure.rs::duration_spacing_factor` — pure **power-of-ratio**:
  `width = min_note_spacing · spacing_ratio^(shortest_log2 − duration_log2)`.
  The *entire* event width is treated as one compressible quantity. There is no
  incompressible **rod** (notehead + accidental cluster + dot cluster + min pad),
  so dense passages can compress until noteheads/accidentals collide.

- `layout/system.rs` (target-width branch, ~L223–243) — when fitting a system to
  `target_width`, it scales **everything uniformly** (`elem.x *= scale`,
  `elem.width *= scale`), including the clef/key/time prefix. Gourlay requires that
  **rods stay fixed and only springs scale**.

The port plan (§6) specifies:

1. **Rod** per event = `notehead + accidental_cluster + dot_cluster + min_padding`
   — incompressible.
2. **Spring** between adjacent events at rest length `k · duration^c`, `c ≈ 0.6`.
3. **Compression/extension** (one-line analytic): find scale `s` such that
   `Σ(rod_i + s · spring_i) = target_width`. Only springs scale.
4. **Multi-voice:** spring length = max across voices at each shared tick position.

## 2. Design decisions (resolved)

- **Rods in staff-space units, not real glyph metrics.** `layout_measure` today
  has no font handle and works purely in `ss`-scaled constants. Threading a font
  through would be a large, orthogonal change. Instead, rod components are
  `ss`-multiple estimates in `MeasureLayoutConfig`, consistent with existing code
  (`glissando.rs:70` already uses `ss * 1.18` for notehead width). Accidental and
  dot contributions are estimated per-event from the event's own fields
  (`accidental.is_some()`, `dots`). This keeps the rod *correct in structure*
  (incompressible, per-event) without a metrics refactor. A follow-up can swap in
  real advance widths behind the same rod API.

- **Spring rest length** `spring_i = k · duration^c` where `duration` is the note's
  duration as a multiple of the shortest note in the measure (so the shortest note
  has `duration = 1`, a note twice as long has `duration = 2`, etc.). This is the
  natural reading of Gourlay and keeps `c` dimensionless. `k` is a config constant
  in `ss` units; `c = spacing_exponent` (default 0.6, range 0.5–0.7 per §6).

- **`spacing_ratio` is removed**, replaced by `spacing_exponent` (c) and a spring
  constant `k`. `duration_spacing_factor` is deleted; its tests are replaced.

- **Compression solves at the system level**, where `target_width` lives. The
  measure layer produces, per element, *both* a rod and a spring contribution so
  the system layer can scale springs only. `MeasureLayout` therefore needs to
  expose per-element rod/spring, not just a single `width`.

## 3. Data model changes

### `PositionedElement` (layout/measure.rs)
Add the rod/spring decomposition so the system layer can scale correctly:

```rust
pub struct PositionedElement {
    pub x: f64,
    pub element: MeasureElement,
    pub width: f64,      // = rod + spring at natural (s = 1) layout
    pub rod: f64,        // incompressible
    pub spring: f64,     // compressible; width == rod + spring
}
```

`width` stays as the natural-layout convenience (`rod + spring`) so existing
renderers that read `width` keep working. Invariant asserted in tests:
`width == rod + spring` for every element; prefix/barline elements have
`spring == 0` (fully incompressible).

### `MeasureLayout`
Add aggregate rod/spring so the system layer doesn't have to re-sum:

```rust
pub struct MeasureLayout {
    pub elements: Vec<PositionedElement>,
    pub total_width: f64,   // natural width (s = 1)
    pub total_rod: f64,     // Σ rod
    pub total_spring: f64,  // Σ spring  (total_width == total_rod + total_spring)
}
```

### `MeasureLayoutConfig`
Remove `spacing_ratio`. Add:

```rust
pub spacing_exponent: f64,   // Gourlay c; default 0.6
pub spring_constant: f64,    // k, in font units; rest length = k · duration^c
pub notehead_rod: f64,       // ss-estimate of notehead advance
pub accidental_rod: f64,     // added when event has an accidental
pub dot_rod: f64,            // added per augmentation dot
pub min_rod_padding: f64,    // floor padding inside every rhythmic rod
```

`from_staff_space` populates these (notehead_rod ≈ 1.18·ss to match glissando.rs;
others tuned in Phase 4 calibration).

## 4. Algorithm

### Per rhythmic event (measure.rs)
```
rod    = min_rod_padding
       + notehead_rod
       + (accidental present ? accidental_rod : 0)
       + dots · dot_rod
dur    = 2^(shortest_log2 − duration_log2)        // shortest note = 1.0
spring = spring_constant · dur^spacing_exponent
width  = rod + spring
```
- **Chords:** rod uses the widest accidental contribution (chords stack
  accidentals leftward); one notehead_rod (shared stem column). `dots` from event.
- **Beam/Tuplet groups:** each inner note contributes its own rod+spring; the group
  element's rod/spring are the sums (preserves current "sum of inner spacing"
  behavior but now decomposed). Inner note x-offsets continue to come from
  `beam_group_note_x_offsets`, which we feed the group's `total_width`.
- **Prefix (clef/key/time), barline, MultiMeasureRest:** `rod = width`,
  `spring = 0`. (MultiMeasureRest is incompressible-as-a-block; it already uses a
  fixed allocation.)

### System-level compression (system.rs)
Replace the uniform-scale branch. Given `target_width`:
```
total_rod    = Σ measure.total_rod
total_spring = Σ measure.total_spring
s = if total_spring > 0 { (target_width − total_rod) / total_spring } else { 1.0 }
s = s.max(MIN_SPRING_SCALE)      // clamp: never collapse springs below a floor
```
Then for every element: `new_spring = s · spring`, `new_width = rod + new_spring`,
and re-flow `x` left-to-right (do **not** scale `x` directly — recompute by
accumulation, since x is now a function of preceding widths).

**Clamp rationale:** if `target_width < total_rod`, the content genuinely doesn't
fit; springs go to the floor and the system overflows (caller's line-breaker must
handle — out of scope here, but the clamp prevents negative/zero widths). `log`-free;
just a documented floor constant.

### Multi-voice (system.rs)
At the time of this plan, additional voices were laid out independently and
scaled to the primary voice's total width. The first three phases below kept
that endpoint-only alignment while converting to spring-only scaling.

**Implemented 2026-10-04:** `layout/rhythm_grid.rs` now merges exact performed
onsets (including tuplets) across voices and staves, taking the maximum rod
and spring requirement per shared column. `layout/system.rs::layout_staves_followed_by`
assigns one horizontal grid; `MultiStaffScore` uses it for all staves and
joined barlines. Voice-collision ink offsets apply after temporal alignment.
The former endpoint-only strategy and its deferred-work note no longer
describe the current renderer.

## 5. Phasing

### Phase 1 — Rod/spring data model (measure.rs)
- Add `rod`/`spring` to `PositionedElement`, `total_rod`/`total_spring` to
  `MeasureLayout`.
- Replace `MeasureLayoutConfig` fields (`spacing_ratio` → `spacing_exponent`,
  `spring_constant`, rod estimates). Update `from_staff_space`.
- Rewrite `layout_measure` to compute rod + spring per element; delete
  `duration_spacing_factor`.
- Update existing measure tests: replace the three `duration_spacing_factor_*`
  unit tests with `spring_rest_length_*` tests; keep the behavioral tests
  (`proportional_spacing_*`, `equal_durations_*`) but assert on the new model
  (longer note → longer spring; rods equal for same-glyph events). Add invariant
  tests: `width == rod + spring`, prefix/barline `spring == 0`.
- **Exit:** `cargo test -p music-engraver --lib layout::measure` green; natural
  widths within a few percent of old model for the calibration corpus (Phase 4
  tunes `k`).

### Phase 2 — System-level spring-only compression (system.rs)
- Replace the uniform-scale branch with the analytic `s` solve (springs only).
- Re-flow `x` by accumulation after scaling.
- Apply spring-only scaling to additional voices (scale to primary width).
- Add `MIN_SPRING_SCALE` const + doc comment on the overflow case.
- **Exit:** system tests green; a `target_width` larger than natural stretches
  springs but leaves the clef/key/time prefix byte-for-byte fixed (new test).

### Phase 3 — Downstream + golden
- Audit every reader of `PositionedElement.width` / `MeasureLayout.total_width`
  (renderers, beam offset feeder, page layout) — these keep working since `width`
  is preserved as `rod + spring`. Confirm no reader assumed uniform scaling.
- Run full golden suite. Triage each diff: a shifted note x is *expected* where the
  old uniform model was wrong (prefix-in-dense-systems). Re-baseline only confirmed-
  correct diffs via `GOLDEN_UPDATE=1`, eyeballing rasterized output.
- **Exit:** `cargo test -p music-engraver` fully green; `cargo clippy` clean; any
  re-baselined goldens documented in the progress entry.

### Phase 4 — Calibration
- Build/extend a small corpus (C-major scale, Twinkle, dotted rhythms, dense 16th
  runs, mixed whole+eighth) — several already exist as examples/goldens.
- Sweep `spacing_exponent ∈ {0.5, 0.6, 0.7}` and tune `spring_constant` so a
  typical measure's natural width ≈ the old model's (minimizes baseline churn) and
  dense passages no longer collide. Lock defaults.
- **Exit:** defaults committed with a one-paragraph rationale in the progress log.

## 6. Risks / notes

- **Baseline churn.** Mitigated by tuning `k` so natural widths match the old model
  in the common case; only genuinely-wrong-before layouts shift.
- **`x` is no longer scalable directly.** Phase 2 must re-flow by accumulation;
  scaling `x` in place (as today) would be wrong once springs and rods scale
  differently. Explicit test: prefix x unchanged under stretch.
- **Multi-voice max-spring-per-tick deferred** — documented deviation, follow-up.
- **Additive-where-possible.** New struct fields are additive; the breaking change
  is `MeasureLayoutConfig` (`spacing_ratio` removed) and `duration_spacing_factor`
  deletion. Both are `pub` in a pre-1.0 internal crate; grep confirms no external
  consumer beyond the engraver itself.

## 7. Verification checklist (per phase)
- `cargo build -p music-engraver` clean
- `cargo test -p music-engraver --lib --test golden_svg`
- `cargo clippy -p music-engraver --tests`
- `GOLDEN_UPDATE=1` rasterize + eyeball any re-baselined goldens
- Progress-log entry in `docs/ENGRAVER-PROGRESS.md` (Did / Verified / Next / Open
  issues) matching the existing cadence.
