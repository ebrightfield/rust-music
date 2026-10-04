# Slonimsky CLI — Bug Report

Tested: 2026-08-03 · binary built from workspace HEAD (`cargo build -p slonimsky`)
Context: found while prototyping the daily guitar-practice generator
(`docs/practice-regimen-ideas/exercise-suite.md`). Every item below was reproduced against the
current build, not carried over from notes.

Supersedes the status claims in `slonimsky-cli-gaps.md` (2026-05-12) — several gaps recorded
there are now **fixed**; see §5. That file has since been marked retired.

---

## Resolution status (fixed 2026-08-03)

| Item | Status | Fix |
|---|---|---|
| §1 `spell` non-tertian spellings | **fixed** | `spell.rs` now spells by interval-from-root letter arithmetic, emitting double accidentals (`Gbm7 → Gb Bbb Db Fb`) where they are correct |
| §2 SVG not publication-quality | **fixed** | Rests and beaming wired through a new `NotationEvent`; two genuine engraver bugs fixed (see below) |
| §3 `--clef treble-8` ignored | **partly misdiagnosed → fixed via §7** | The clef *glyph* was already correct (`GClef8Vb`, `8` renders). The note *placement* was genuinely wrong — see §7 |
| §7 `treble-8` notes an octave too high | **fixed** | `note_placement.rs` gave the transposing clefs a shifted reference pitch, double-applying the octave. Written pitches now sit exactly where plain treble puts them. Guard: `docs/verify-clef-octave.sh` |
| §4 subchord labels rooted on pc 0 | **fixed** | `subchords` transposes results back into the queried key; `--relative` keeps the old prime-form output |
| §6 `ear-training` discoverability | **fixed** | `--help` now lists the subcommand and the `--features midi` requirement |
| §9 clef errors exit 0 | **fixed** | The clef was parsed only inside the `-o` branch, so `--clef alto` printed a normal sheet and exited 0. Both `sight-reading` and `rhythm-drill` now validate it up front; bad values exit 1 with the same messages |

Two engraver defects were found and fixed while addressing §2 — both affected any
multi-system score, not just the CLI:

- **Beams ignored the system y-offset.** `layout_beam_group` returns staff-relative
  stem tips, but `beam_renderer` mixed them with absolute notehead positions, so every
  beam on the 2nd and later systems was drawn at the first system's height.
- **The viewBox clipped notes above the staff.** It was derived from the staff box
  alone, so ledger-line passages (e.g. a `treble-8` guitar part) were cut off.

`spell` also gained context-sensitive handling for the two ambiguous intervals: the
tritone spells as a `#11` when a perfect fifth is present (`Cmaj7#11 → C E F# G B`,
previously the absurd `Gb G`), and a dim7's top note as a diminished seventh
(`Cdim7 → C Eb Gb Bbb`, previously `A`).

---

## 1. `spell` produces non-tertian and wrong-direction spellings — HIGH · FIXED

A chord's third must be spelled as a *third* (two letter-names up), its seventh as a seventh,
etc. Several roots violate this, and flat-key minors come back with sharps.

```
$ slonimsky spell Cm7
C D# G A#          # WRONG — should be C Eb G Bb
                   # D# is an augmented 2nd above C, not a minor 3rd; A# likewise

$ slonimsky spell Abm7
Ab B Eb Gb         # WRONG — should be Ab Cb Eb Gb
                   # B is an augmented 2nd above Ab, not a minor 3rd

$ slonimsky spell Dbm7
Db E Ab B          # WRONG — should be Db Fb Ab Cb

$ slonimsky spell Gbm7
Gb A Db E          # WRONG — should be Gb Bbb Db Fb
```

Correct today (so the fix is a spelling-selection issue, not a chord-tone issue):

```
$ slonimsky spell Bbmaj7   ->  Bb D F A     ✓
$ slonimsky spell Ebmaj7   ->  Eb G Bb D    ✓
$ slonimsky spell F7       ->  F A C Eb     ✓
$ slonimsky spell Ebm7     ->  Eb Gb Bb Db  ✓
$ slonimsky spell Fm7      ->  F Ab C Eb    ✓
```

**Pattern:** minor chords on roots whose minor 3rd requires a flat-of-a-flat (or a letter the
current code won't pick) fall back to the sharp enharmonic. `Cm7` is the most visible case
because it is extremely common.

**Expected:** spell by *interval-from-root letter arithmetic* — root letter + 2 letters = third,
+ 4 = fifth, + 6 = seventh — then choose the accidental (including double-flats) that lands on
the required pitch class. Emitting `Bbb` is correct and preferable to `A`.

**Impact:** any generated sheet that prints chord tones. Currently the generator must
re-spell chords itself, which defeats the purpose of the command.

---

## 2. Notation SVG output is not publication-quality — HIGH · FIXED

Affects `rhythm-drill -o x.svg` and `sight-reading -o x.svg` (both now *do* write a file — see
§5 — but the engraving is unusable).

```
$ slonimsky rhythm-drill --measures 4 --seed 7 -o r.svg
$ slonimsky sight-reading --key Bb --scale major --measures 4 -o sr.svg
```

Rendered output shows:

- **No beaming.** Every eighth/sixteenth carries its own flag; nothing is beamed into groups.
- **No interior barlines.** Measures run together; only the final double bar is drawn.
  Systems break arbitrarily rather than at measure boundaries.
- **Rests dropped.** `rhythm-drill`'s *text* output correctly shows `𝄾♪` rests, but the SVG
  omits them, so the notated rhythm does not match the printed text for the same seed.
- **Inconsistent measure grouping** across systems.

The text output for the same commands is correct, so this is purely the SVG engraver.

**Expected:** beamed subdivisions, barlines at every measure, rests rendered, systems broken at
measure boundaries.

**Workaround in use:** all staff notation is routed through LilyPond; Slonimsky is used for
content decisions and for the *diagram* outputs (`fretboard`, `pitch-circle`,
`interval-matrix`), which are fine.

**Note:** `music-engraver` is in progress and is presumably the intended fix.

---

## 3. `--clef treble-8` is accepted but ignored — MEDIUM · MISDIAGNOSED (see §7)

```
$ slonimsky sight-reading --key Bb --scale major --measures 2 --clef treble-8 -o sr.svg
```

The flag parses (no error) but the output renders a plain treble clef with no `8` below.
For guitar this is a pitch-level error of one octave, not a cosmetic one.

**Expected:** `treble-8` renders the octave-treble clef (guitar standard); `bass`/`alto`
likewise. Alternatively reject unimplemented values instead of silently ignoring them.

**Re-test 2026-08-03 — the clef *glyph* was already correct, the placement was not.**
The flag is wired end to end (`ClefChoice::Treble8` → `Clef::Treble8ba` →
`Glyph::GClef8Vb`) and does render the `8` below the G-clef, so "accepted but ignored"
was not the right diagnosis. But the notes *were* wrong: they shifted down an octave in
staff position, which for a transposing clef is a real bug — see §7, where it is
diagnosed and fixed. Two contributing render bugs (a clipped viewBox and misplaced
beams) are fixed under §2. `alto`/`tenor` were rejected with an explicit error until
2026-10-04 (RM-MN-003), when `music::Clef` gained `Alto`/`Tenor`; both now render a
C clef (`Glyph::CClef`) with C4 on the middle / fourth line.

---

## 4. `subchords` / `superchords` label sets relative to pitch-class 0, not the queried root — MEDIUM · FIXED

```
$ slonimsky subchords G,A,B,C,D,E,F# --size 3 --name
Subchords of {0,2,4,6,7,9,11} (size 3):
    1. {0,2,4}  CMaj (9)        # this is the G-rooted set; "C" is wrong
    3. {0,2,7}  Csus2
    4. {0,2,9}  CMaj6 (9)

$ slonimsky subchords Bb,C,D,Eb,F,G,A --size 5 --name
    1. {0,2,3,5,7}  Cmin (9, 11)   # queried Bb major; labels come back C-rooted
```

The input set is normalized to start at pitch-class 0 and the *labels are generated against
that transposition*, so every name is transposed by `-root`. The `{…}` sets are internally
consistent; only the human-readable names are wrong.

**Expected:** either report names in the key of the queried root, or add an explicit
`--relative-to <note>` / `--absolute` flag. At minimum, document that names are prime-form
relative so callers know to transpose.

**Impact:** a generator that prints these labels verbatim mislabels every chord. Currently
worked around by recomputing names downstream.

**Fixed 2026-08-03.** The cause was `PcShape::new`, which sorts *and* zero-anchors, so the
queried root was gone before naming ran. `subchords` now shifts the enumerated subsets back
into the queried key and prefers the queried root when a subset contains it, so
`subchords G,A,B,C,D,E,F#` labels the G-rooted triad `GMaj` and the Bb query yields `Bbmin`.
Set listings are sorted ascending, and `--relative` restores the prime-form output for
callers that want it.

Note that `superchords` was **not** affected: it already searched all 12 transpositions and
reported real absolute roots (`superchords G,B,D` → `G Maj7`). Only its header line printed
the zeroed shape, which now prints in the queried key as well.

---

## 5. Previously-reported gaps that are now FIXED

Recorded as open in `slonimsky-cli-gaps.md` (2026-05-12); verified working today. That file
should be updated or retired.

| Gap (as filed) | Status | Evidence |
|---|---|---|
| Enharmonic spelling in `sight-reading` always sharp | **fixed** | `--key Bb --scale major` yields `Bb4 C5 D5 Eb5 F5 G5`, correct flats |
| No staff-notation output from `sight-reading` (`-o` silently ignored) | **fixed** | `-o sr.svg` writes the file (quality issues → §2) |
| No `--clef` flag | **partly fixed** | flag now exists on `sight-reading`; value ignored → §3 |
| No rhythm-drill generation | **fixed** | `rhythm-drill` subcommand exists with `--time-sig/--style/--syncopation/--measures/--seed` |
| `spell Bbmaj7` returns `B Dbb Fb G` | **fixed for this case** | now `Bb D F A`; other roots still wrong → §1 |

Also added since that report and working: `--seed` on `sight-reading` and `rhythm-drill`
(reproducible days), `practice-sheet -o` writes SVG.

---

## 6. Minor / lower priority

- **`ear-training` is feature-gated and therefore invisible in a default build.** *Not a bug —
  a discoverability issue.* `slonimsky ear-training` → `unrecognized subcommand` after a plain
  `cargo build -p slonimsky`, because the subcommand sits behind `#[cfg(feature = "midi")]`
  (`main.rs:296`). It works correctly after `cargo build -p slonimsky --features midi`, offering
  `--type/--count/--seed/-o` and generating MIDI interval quizzes. Worth noting in `--help` or
  the README so it isn't mistaken for missing; useful for the ear-training exercise families.
- **`subchords --name` prints `?` for many valid sets** (e.g. `{0,1,6,7}`, `{0,1,6,8}` from
  Lydian b9 b13). Expected for genuinely unnamed sets, but a Forte number or interval-vector
  fallback would be more useful than `?` for a practice sheet.

---

## 7. `--clef treble-8` renders notes an octave too high — HIGH (regression) · FIXED

Found 2026-08-03 while re-testing the §1/§2/§4 fixes. Those fixes are real — beaming,
barlines and rests now render, and the `8` appears under the G-clef. But **note placement**
for `treble-8` is off by an octave.

### Reproduce

```
$ slonimsky sight-reading --key C --scale major --measures 1 --seed 5 --clef treble   -o t.svg
$ slonimsky sight-reading --key C --scale major --measures 1 --seed 5 --clef treble-8 -o t8.svg
```

Same key, same seed, same written pitches — only the clef differs. The text output is
correct in both cases; only the SVG differs.

### Measured offset (this is the acceptance test)

Extracting the `translate(x, y)` values in document order from both files:

| | y values |
|---|---|
| `treble` | 750, 250, 750, **1250, 1125, 1250, 1500, 1375, 1250** |
| `treble-8` | 750, 250, 750, **375, 250, 375, 625, 500, 375** |
| delta | 0, 0, 0, **875, 875, 875, 875, 875, 875** |

The first three are staff/clef furniture and are correctly unshifted. Every **notehead** is
shifted by a uniform **875 units = 7 × 125-unit steps = exactly one octave** (7 diatonic
steps). The viewBox also grows 750 units taller (`-250 -250 35500 3000` →
`-250 -1000 35500 2875`) as the engraver makes room for glyphs it placed too high.

Visually: Bb4–G5 sits *on* a treble staff with the default clef, and floats 3–5 ledger lines
*above* it with `treble-8`.

### Cause

`treble-8` (`treble_8` / `8vb`) is an octave-**transposing** clef: written pitch *sounds* an
octave lower. Staff placement must therefore be **byte-identical to plain treble** — only the
clef glyph changes, plus the sounding pitch for MIDI. The engraver is applying the octave
shift to glyph positions as well as to the sounding pitch, i.e. double-applying it.

### Acceptance criteria for the fix

1. For identical written pitches, `--clef treble-8` and `--clef treble` produce the **same
   notehead y values** — the delta array above must be all zeros.
2. The `treble-8` viewBox should match the `treble` viewBox for the same content (no extra
   headroom), unless genuine ledger lines require it.
3. The clef glyph still differs (`GClef8Vb` vs `GClef`).
4. Any MIDI/sounding-pitch output for `treble-8` stays an octave below written.
5. Regression guard: `bass` is unaffected — its y-range is `(-375, 750)` for this input and
   should not change.

Reference behaviour: LilyPond's `\clef "treble_8"`, which the practice generator already uses.

**Impact:** guitar is the primary use case for this flag, so every `treble-8` sheet is wrong —
and wrong in the *pitches-look-misplaced* way, which is worse than the feature being absent.

### Verified FIXED — 2026-08-03

`docs/verify-clef-octave.sh` passes against the current build:

```
treble   viewBox = -250 -250 35500 3000
treble-8 viewBox = -250 -250 35500 3000        # was -250 -1000 35500 2875
deltas     = [0, 0, 0, 0, 0, 0, 0, 0, 0]       # was [0,0,0, 875 × 6]
PASS — treble-8 places noteheads identically to treble.
```

All five acceptance criteria met: placement identical, viewBoxes match, the clef glyph still
differs (13418 vs 13914 bytes — the extra `8`), and `bass` is unaffected. Rendering confirms
notes on the staff with the `8` under the G-clef.

---

## 8. Remaining: recommendation for the practice generator

With §1–§4 and §7 all fixed, Slonimsky's SVG is now genuinely viable for **rhythm drills and
single-line melody**, including the guitar clef. The practice generator can start using it for
those.

One reason it still routes the rest through LilyPond: **no tab staff.** The sheets need staff +
TAB together, with string/fret assignments, fingerings, slurs, glissandi, ghost/parenthesised
notes, and chord symbols above. That is a feature gap, not a bug.

Suggested split until tab exists:
- **Slonimsky** — `rhythm-drill` and `sight-reading` blocks (ear/rhythm), plus all diagram
  output (`fretboard`, `pitch-circle`, `interval-matrix`).
- **LilyPond** — anything with tab: warmups, vocabulary patterns, voicings, improv models.

---

## 9. NEW — clef errors print `Error:` but exit 0 — LOW · FIXED

Found 2026-08-03 during a post-fix sweep. The clef validation added for §3 reports
unsupported and unknown values with good messages, but the process still exits **successfully**:

```
$ slonimsky sight-reading --measures 1 --clef alto  >/dev/null 2>&1 ; echo $?
0        # prints "Error: clef 'alto' is not supported (…)" but exits 0

$ slonimsky sight-reading --measures 1 --clef bogus >/dev/null 2>&1 ; echo $?
0        # prints "Error: unknown clef: 'bogus' (…)" but exits 0

$ slonimsky spell NotAChord                        >/dev/null 2>&1 ; echo $?
1        # correct — other commands do exit non-zero
```

So `spell` is right and the clef path is not; the behaviour is inconsistent within the CLI.

**Why it matters:** the practice generator shells out to these commands and checks exit status
to decide whether a step succeeded. An exit-0 error means a bad `--clef` silently yields a
sheet with the wrong clef instead of failing the run. This is exactly the class of silent
failure that produced several of the notation bugs recorded in
`practice-regimen-ideas/exercise-suite.md` ("verify by rendering, never by exit code").

**Expected:** any path that prints `Error:` returns a non-zero exit code.

**Note:** the error *messages* themselves are good and worth keeping — they name the supported
values (`treble, treble-8, bass`) and distinguish "not supported by `music::Clef`" from
"unknown value". Only the exit code needs changing.

**Fixed 2026-08-03.** The root cause was narrower than "exit code": `ClefChoice::from_str_opt`
was called *inside* the `if let Some(output)` branch, so without `-o` the clef was never
validated at all — hence a full text sheet and exit 0. Both `sight-reading` and `rhythm-drill`
now parse the clef with their other arguments, before generating anything, so an invalid value
fails identically with or without `-o`:

```
$ slonimsky sight-reading --measures 1 --clef alto  >/dev/null 2>&1 ; echo $?
1
$ slonimsky sight-reading --measures 1 --clef bogus >/dev/null 2>&1 ; echo $?
1
$ slonimsky sight-reading --measures 1 --clef treble-8 >/dev/null 2>&1 ; echo $?
0
```

Guarded by `bad_clef_exits_nonzero_without_output_file` and `supported_clefs_succeed` in
`slonimsky/tests/sight_reading.rs`. The messages are unchanged.

---

## 10. Pre-existing — annotation glyphs above/below the staff can still be clipped — LOW

Found 2026-08-03 while auditing the golden-SVG baselines for the §2 viewBox fix. Not a
regression and not part of the original report; recorded so it isn't rediscovered.

The §2 fix sizes the viewBox from **notehead** staff positions, so ledger-line passages are
no longer cut off. It does not measure *annotation* glyphs, which are placed at their own
offsets from the staff. Two goldens still have a glyph outside the box — identically before
and after the fix:

| golden | box | glyph extent | outside |
|---|---|---|---|
| `annotations.svg` | `[-250, 2500]` | `[-700, 1250]` | rehearsal mark above the staff |
| `pedal_marks.svg` | `[-250, 2500]` | `[250, 2750]` | pedal mark below the staff |

**Expected:** the extent calculation should also cover dynamics, pedal/ottava brackets,
rehearsal marks, lyrics, and chord symbols — i.e. anything the renderers draw — rather than
noteheads alone.

**Impact for the practice generator: none today.** `sight-reading` and `rhythm-drill` emit
none of these annotation types. It would matter if chord symbols or dynamics were added to
generated sheets, so it should be fixed before that.

---

## Priority for the practice-generator use case

**Status 2026-08-03 — everything originally reported is fixed and verified.**

| Item | Was | Now |
|---|---|---|
| §1 `spell` non-tertian spellings | HIGH | fixed — `Cm7 → C Eb G Bb`, `Gbm7 → Gb Bbb Db Fb` |
| §2 SVG beaming / barlines / rests | HIGH | fixed (plus 2 engraver defects) |
| §3 `--clef treble-8` ignored | MEDIUM | misdiagnosed — clef glyph was correct, but placement was an octave off → fixed in §7 |
| §4 subchord labels rooted on pc 0 | MEDIUM | fixed — `subchords` and `superchords` both report in the queried key |
| §7 `treble-8` octave regression | HIGH | fixed — guard: `docs/verify-clef-octave.sh` |
| §9 clef errors exit 0 | LOW | fixed — clef is validated up front, bad values exit 1 |
| §10 annotation glyphs clipped | LOW | **open** — pre-existing, not a regression; no impact on current output (see §10) |

Remaining, in order:

1. **§10 (low, pre-existing defect)** — extend the viewBox extent calculation past noteheads to
   annotation glyphs. No effect on today's output; needed before adding chord symbols or dynamics.
2. **§8 (feature, not a bug)** — tab-staff output (staff + TAB, string/fret, fingerings, slurs,
   glissandi, chord symbols). This is the only thing keeping the practice generator on LilyPond
   for its guitar-notation blocks; rhythm and single-line melody can move to Slonimsky now.
3. **§6 (cosmetic)** — a Forte number or interval-vector fallback instead of `?` for unnamed
   sets, which show up often in irregular 7NS subsets.

Verification helper: `docs/verify-clef-octave.sh` (exit 0 = fixed, 1 = regressed). It is a thin
wrapper around the regression tests that now live with the code, so there is one source of truth:

- `music-engraver/src/layout/note_placement.rs` — placement invariant at the unit level
- `music-engraver/tests/golden_svg.rs` — `treble8ba_clef` / `treble8va_clef` frozen baselines,
  plus a whole-SVG invariant asserting the transposing clefs match treble in everything but the
  clef glyph (this covers beams, stems, ledger lines, and the viewBox — which an SVG-scraping
  script cannot)
- `slonimsky/tests/sight_reading.rs` — end-to-end through the CLI

Before this, all 77 goldens used plain treble (plus one bass), so no baseline exercised a
transposing clef — which is why the §7 octave shift survived in them unnoticed.
