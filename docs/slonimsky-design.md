# Slonimsky CLI — Design Document

> Authoritative live design for the `slonimsky` binary crate.
> Evolved incrementally; supersedes the original `cli-design.md` sketch
> (which is retained as design history).

---

## Global Interface

```
slonimsky [OPTIONS] <COMMAND>
```

| Flag | Short | Description |
|------|-------|-------------|
| `--output <PATH>` | `-o` | Output file; format inferred from extension (`.svg`, `.pdf`, `.png`, `.ly`, `.mid`, `.txt`, `.ron`, `.json`). Default: stdout for text, `./<name>` for binary. |
| `--verbose` | `-v` | Verbose / debug output on stderr. |
| `--theme <THEME>` | `-t` | SVG color theme: `default`, `dark`, `print`, `colorful`. Maps to `SvgTheme`. |
| `--help` | `-h` | Help text. |
| `--version` | `-V` | Version string. |

### Output format inference

Extension → format:

| Extension | Format | Backend |
|-----------|--------|---------|
| `.svg` | SVG | `music::svg::*Builder` |
| `.pdf` | PDF | SVG piped through `resvg` or LilyPond |
| `.png` | PNG | SVG piped through `resvg` |
| `.ly` | LilyPond source | `music::notation::lilypond` (feature-gated) |
| `.mid` | MIDI SMF | `music_midi::SmfBuilder` |
| `.txt` | Plain text table | Built-in formatters |
| `.ron` | RON round-trip | `serde` + `ron` |
| `.json` | Machine-readable | `serde_json` |

No extension or `-` → stdout as text.

### Input conventions

Where semantics allow, these are interchangeable:

- Positional args: chord symbol (`Cmaj7`), note list (`C E G B`), pc integers (`0 4 7 11`).
- `--pcs 0,4,7,11` — explicit pitch-class set.
- `--notes C,E,G,B` — explicit note names.
- `--scale "C major"` — scale name.
- `--ron <path>` — load input from a `.ron` document.

---

## Subcommands — Baseline (from cli-design.md)

### `pitch-circle`

Draw a pitch-class circle diagram.

```
slonimsky pitch-circle <INPUT>... [-o out.svg] [--theme dark]
    [--root <NOTE>] [--show-intervals] [--title <TITLE>]
```

Implementation: parse input → `PcSet`, build via `PitchCircleBuilder::from_pc_set()`.
Honors `--root`, `--show-intervals`, `--title`, `--theme`.

### `fretboard`

Draw a fretboard shape diagram.

```
slonimsky fretboard <INPUT>... [-o out.svg] [--theme dark]
    [--tuning <TUNING>] [--orientation <up-down|right-left>]
    [--title <TITLE>] [--frets <SHAPE_STR>]
```

`--tuning`: one of `standard`, `drop-d`, `dadgad`, `open-g`, `7-string`,
`bass-4`, `bass-5`, or a comma-separated semitone list.
`--frets`: direct fret string like `x-3-2-0-1-0`.

Implementation: if `--frets` given, `FretboardShape::from_string()`; else
`find_chord_shapes()` from input PcSet, pick first playable, render via
`FretboardBuilder`.

### `spell`

Spell a chord symbol into constituent notes.

```
slonimsky spell <SYMBOL> [--format <notes|pcs|intervals|all>]
```

Implementation: `ChordName::from_symbol()` → display notes, pitch classes,
or interval structure depending on `--format`. `all` prints all three on
labeled lines (Notes: / PCs: / Intervals:).

### `name`

Name a chord from pitch classes.

```
slonimsky name <PCS>... [--root <NOTE>] [--config <strict|jazz|pop>]
```

`--root` overrides root inference (default: first PC after normalization).
`--config` selects a `NamingConfig` preset — **planned, not yet implemented**.

Implementation: parse PCS → `PcSet`, run `infer_chord_quality()` with
`NamingConfig` preset. Verbose (`-v`) prints PCs, intervals, and spellings.
Output: chord name(s), ranked by confidence.

### `superchords`

Find chords that contain a given PcSet as a subset.

```
slonimsky superchords <INPUT>... [--min-size 4] [--max-size 7]
```

Implementation: iterate known chord/scale types from
`musical-combinatorics`, filter by superset relation.

### `subchords`

Find all N-note subsets of a chord/scale.

```
slonimsky subchords <INPUT>... [--size 3] [--name]
```

Implementation: `get_subchords()` from `geometry::sets`. If `--name`,
run chord-naming on each subset.

### `interval-matrix`

Print or draw the interval matrix for a PcSet.

```
slonimsky interval-matrix <INPUT>... [-o out.svg] [--title <TITLE>] [--full]
```

`--full`: show the full 12-element interval vector instead of the reduced
6-element interval-class vector.

Implementation: `IntervalMatrix::new()`, text table to stdout or SVG via
`IntervalBuilder::build_matrix()`. Output format inferred from `-o`
extension (`.svg` → SVG, `.txt` or absent → text to stdout).

### `interval-vector`

Print or draw the interval-class vector.

```
slonimsky interval-vector <INPUT>... [-o out.svg] [--title <TITLE>] [--full]
```

`--title`: label for the diagram (text mode: printed as first line; SVG
mode: rendered in the SVG header).
`--full`: show the full 12-element vector instead of the reduced
6-element interval-class vector.

Implementation: `IntervalMatrix::interval_vector()` /
`reduced_interval_vector()`, text or SVG via `IntervalBuilder::build_vector()`
/ `build_full_vector()`. Text mode includes a labeled ic breakdown
showing interval class names (m2/M7, M2/m7, m3/M6, M3/m6, P4/P5, TT).

### `chord-dictionary`

Enumerate chord shapes for a given chord across the fretboard.

```
slonimsky chord-dictionary <INPUT>...
    [--tuning <TUNING>] [--max-span 4] [--max-results 20]
    [-o out.svg]
```

Implementation: `find_chord_shapes()` → render each playable shape via
`FretboardBuilder`. Multi-shape SVG grid for file output.

Extended (beyond baseline, **planned**): `--voicing-type <drop-2|drop-3|drop-2+4|quartal|cluster>`
to filter by voicing family. Not yet implemented.

---

## Subcommands — Combinatoric Voicing & Voice-Leading

### `voicings`

Enumerate voicings of a chord under constraints.

```
slonimsky voicings <INPUT>...
    [--range <LOW>..<HIGH>]           # (planned)
    [--min-spacing <N>] [--max-spacing <N>]  # (planned)
    [--strings <N>] [--tuning <TUNING>]      # (planned)
    [--doubling <allow|forbid|require>]      # (planned)
    [--limit <N>]
    [-o out.txt|out.json]
```

**Current implementation:** `CanonicalVoicings` from `musical-combinatorics`
for canonical (register-normalized) voicings. Only `--limit` is implemented.
The `--range`, `--min-spacing`, `--max-spacing`, `--strings`, `--tuning`,
and `--doubling` flags are planned but require register-placed voicing
enumeration (not just canonical forms), which is a prerequisite BUILD task.

**Target implementation:** `Voicing` from `music::note_collections::voicing`
for register-placed variants, filtered by range/spacing constraints.

### `voice-leading`

Enumerate voice-leadings between two voicings.

```
slonimsky voice-leading --from <PITCH,PITCH,...> --to <NOTE,NOTE,...>
    [--no-crossings]
    [--metric <l1|linf|weighted>]    # l1 and linf implemented; weighted planned
    [--limit <N>]
    [-o out.txt|out.svg|out.mid]
```

`--from`: comma-separated pitches with octaves (e.g. `C4,E4,G4`).
`--to`: comma-separated note names (target chord, e.g. `F,A,C`).
`--no-crossings`: apply `NoVoxCrossings` rule (no voice-part inversions).
`--metric`: distance metric for sorting results. `l1` (default) = sum of
absolute semitone motions across all voices. `linf` (also `l_inf`, `max`)
= maximum single-voice absolute semitone motion. `weighted` (configurable
per-voice weights) is planned.

Results sorted by the chosen metric (ascending). Verbose mode (`-v`)
shows both L1 and L∞ values plus per-voice semitone paths (signed
integers) for each result, regardless of which metric is used for sorting.

Output: ranked table (text), or staff/tab diagram (SVG — planned), or
MIDI realization of each candidate leading (planned).

### `progression`

Plan smoothest voice-leadings through a chord sequence.

```
slonimsky progression <CHORD1> <CHORD2> [<CHORD3>...]
    [--no-crossings]
    [--metric <L1|Linf|weighted>]    # L1 (default) and Linf implemented; weighted planned
    [--range <LOW>..<HIGH>]           # (planned)
    [-o out.mid|out.txt]              # MIDI requires `midi` feature; SVG/LilyPond planned
    [-o out.svg|out.ly]               # (planned)
```

Each `<CHORD>` is a comma-separated group of PCs or note names (e.g.
`C,E,G F,A,C G,B,D`). Spaces delimit chords; commas delimit notes within
a chord. All chords must have the same number of notes (equal voice count).

`--no-crossings`: apply `NoVoxCrossings` rule (no voice-part inversions).

> **Design decision:** Comma-group input is the same convention as
> `common-tones` and `voice-leading --from`/`--to`, avoiding the clap `--`
> end-of-options conflict.

**Current implementation:** Greedy step-by-step optimization: places
the first chord as a close-position voicing in octave 4, then at each
step picks the lowest-cost voice-leading to the next chord via
`Voiceleading::find_all()`, scored by the chosen `--metric` (L1 default,
L∞ available). Text output to stdout by default. MIDI output via `-o
file.mid` (requires `midi` feature): writes an SMF file with block
chords using `midly`, one chord per beat at 120 BPM. Output header shows
the active metric. Verbose mode (`-v`) shows per-voice semitone paths,
the complementary metric (L∞ when sorting by L1, and vice versa), and
average cost per step.

**Planned extensions:**
- Global optimization (search across all steps simultaneously, not greedy).
- Weighted metric (same as `voice-leading`).
- Staff notation via LilyPond (feature-gated).
- Fretboard sheet via `FretboardBuilder` grid.

### `orbits`

Compute transpositional and inversional symmetry properties of a PcSet.

```
slonimsky orbits <INPUT>... [--type <transpositional|inversional|both>]
```

Default `--type both`. Output is a text report:

```
PcSet: {0, 3, 6, 9}
Transpositional symmetries: T3, T6
  Pc0: {T3, T6}
  Pc3: {T3, T6}
  Pc6: {T3, T6}
  Pc9: {T3, T6}
Inversionally symmetric: yes
Inverted form: (same — palindromic interval content)
```

**Library API mapping:**

- `PcSet::transpositional_symmetry()` → `TranspositionalSymmetryMap`
  (a `HashMap<Pc, HashSet<TranspositionalSymmetry>>`). The map is non-empty
  only when the set has T2/T3/T4/T6 symmetry. An empty map means
  "no transpositional symmetry beyond T0 (identity)."
- `TranspositionalSymmetry` enum: `T2`, `T3`, `T4`, `T6`.
- `PcSet::is_inversionally_symmetric()` (via `IntervallicSymmetry` trait)
  — returns `true` when `invert_intervals()` returns `None` (i.e. the
  `OctavePartition` is a palindrome).
- `PcSet::invert_intervals()` → `Option<PcSet>`. `None` = symmetric;
  `Some(inverted)` = the TnI-related partner.

**Implementation plan:**

1. Parse input → `PcSet` (via shared `parse_input_to_pcs`).
2. Call `transpositional_symmetry()`. Format the map: list unique
   symmetry types, then per-PC detail if verbose.
3. Call `is_inversionally_symmetric()` / `invert_intervals()`.
4. If `--type transpositional`, skip inversional section; if `--type
   inversional`, skip transpositional. Default shows both.
5. Text to stdout. No SVG mode planned initially.

### `prime-form` / `forte`

Classify a PcSet by prime form and (optionally) Forte number.

```
slonimsky prime-form <INPUT>...
slonimsky forte <INPUT>...
```

**Prime form algorithm** (no existing library function — implement in
`cmd/prime_form.rs`):

1. Compute all rotations: for each of 12 transpositions Tn, store the
   sorted PcSet.
2. Compute all inversions: for each transposition, apply TnI (invert
   then transpose). The inversional form of PcSet S at Tn is
   `{(n - pc) mod 12 | pc ∈ S}`, sorted.
3. Among all 24 candidates, pick the one that is lexicographically
   smallest (Rahn's algorithm: most compact from the left).
4. The result is the **prime form** (e.g. `[0,1,3,7]`).

The `forte` subcommand extends this by looking up the prime form in a
Forte number table (a static `&[(PrimeForm, &str)]` mapping, e.g.
`[0,3,7] → "3-11"`, `[0,3,6,9] → "4-28"`). The table covers all 223 set classes for cardinalities 1–12 (the
complete Forte catalog).

> **No A/B suffix.** Rahn's algorithm always picks the single
> lexicographically smallest prime form, so both C major (`{0,4,7}`) and
> C minor (`{0,3,7}`) map to `[0,3,7]` → "3-11". The A/B suffix that
> some references use to distinguish a set from its inversion is not
> emitted. Sets with the same prime form share the same Forte number.

**Output:**

```
$ slonimsky prime-form C E G
PcSet:      {0, 4, 7}
Prime form: [0, 3, 7]

$ slonimsky forte C E G
PcSet:      {0, 4, 7}
Prime form: [0, 3, 7]
Forte:      3-11
```

Verbose (`-v`) adds: interval vector, transpositional symmetries,
inversional symmetry status, Z-relation partner (if any).

**Implementation plan:**

1. `prime-form`: implement Rahn's algorithm (pure function on `PcSet` →
   `Vec<u8>`). This is new logic but small (~30 lines) and PcSet-specific;
   not a reimplementation of existing library code.
2. `forte`: embed the Forte table as a static lookup. Currently covers
   cardinalities 1–4 (48 entries); extend to full 220+ in later BUILD
   runs.
3. Both share the prime form computation; `forte` just adds the lookup.

---

## Subcommands — Commonality & Set Relations

### `common-tones`

Find common pitch classes between two or more chords/scales.

```
slonimsky common-tones <SET1> <SET2> [<SET3>...]
```

Each `<SET>` is a comma-separated group of PCs or note names (e.g.
`C,E,G D,F,A`). Spaces delimit sets; commas delimit PCs within a set.

> **Design decision:** The original `--` separator conflicts with clap's
> end-of-options convention. Comma-groups are unambiguous and compose
> well with shell quoting.

Output: intersection PcSet with PC count and note-name labels.
Verbose (`-v`) prints pairwise intersections when 3+ sets are provided.

### `contains`

Query containment relationships.

```
slonimsky contains <INPUT>... [--in <scales|chords|both>]
    [--direction <super|sub>] [--limit <N>]
```

`--in`: pool to search — `chords`, `scales`, or `both` (default).
`--direction`: `super` finds known types that contain the input PcSet;
`sub` finds known types contained within the input. Auto-detected from
input size when omitted (≤4 PCs → super, ≥5 PCs → sub).
`--limit`: cap the number of results.

> **Design decision:** Auto-detection by size covers the common use case
> ("which scales contain this triad?" vs "which chords are in this
> scale?") without requiring the user to think about direction. Explicit
> `--direction` overrides when the heuristic is wrong.

Results are sorted by size then root, grouped with section headers
(e.g. "4-note chords:", "7-note scales:"). Total count printed at end.

### `closest`

Rank known chords/scales by distance to a given PcSet.

```
slonimsky closest <INPUT>...
    [--metric <symmetric-diff|voice-leading>]
    [--pool <chords|scales|both>]
    [--limit <N>]
```

Output: ranked table of (name, distance, common tones, difference tones).

---

## Subcommands — Practice Material Generators

> These are the highest-value user-facing outputs.

### `practice-sheet`

Single-page practice reference for a key and scale.

```
slonimsky practice-sheet [--key <KEY>] [--scale <SCALE>]
    [-o out.svg|out.txt] [--theme <THEME>]
```

`--key`: root note (default C). Accepts note names (C, G, Bb, F#).
`--scale`: one of `major`, `melodic-minor`, `harmonic-minor`,
`harmonic-major` (default `major`).

**Current implementation:** Text output includes 5 sections: (1) scale
notes with interval vector, (2) all 7 modes transposed to the key,
(3) all 3-note subchords with chord names (via `get_subchords` +
`infer_chord_quality`), (4) all 4-note subchords with names, (5)
practice suggestions including a ii-V-I progression for the key. SVG
output embeds a pitch-circle diagram alongside text sections. Reuses
mode name constants from `scale_book.rs`.

**Planned extensions:**
- `--plan <PLAN.ron>` — `.ron` plan file for multi-page practice
  material (deferred until `music-ron` lands).
- Fretboard position diagrams (requires multi-page layout + `FretboardBuilder` grid).
- Arpeggio patterns and key-of-day cycle sections.
- PDF output via SVG → `resvg`.

### `scale-book`

All modes of a parent scale, all keys, as pitch-circle grid or text.

```
slonimsky scale-book <SCALE>
    [--keys <all|C,G,D,...>]
    [-o out.svg|out.txt] [--theme <THEME>]
```

`<SCALE>`: one of `major`, `melodic-minor`, `harmonic-minor`,
`harmonic-major`.
`--keys`: comma-separated list of root notes, or `all` (default all 12).

**Current implementation:** Uses `SevenNoteScaleQuality` →
`OctavePartition` → `PcSet` → `.modes()` to enumerate all 7 modes,
then transposes each to each key. Text output shows all modes × keys
with note names. SVG output produces a grid of `PitchCircleBuilder`
diagrams (modes as rows, keys as columns) with mode names and key
labels. Named mode tables for 4 scale families (28 mode names total:
Ionian/Dorian/Phrygian/etc. for major, plus Melodic Minor modes,
Harmonic Minor modes, Harmonic Major modes).

**Planned extensions:**
- `--tuning <TUNING>` — fretboard position diagrams alongside pitch circles.
- Staff notation output via LilyPond (feature-gated).
- PDF output via SVG → `resvg`.
- SVG `<defs>` sharing to reduce file size for large grids (~300KB for 7×12=84 diagrams).

### `arpeggio-dictionary`

Per-key chord shape sheets across fretboard positions.

```
slonimsky arpeggio-dictionary <INPUT>...
    [--keys <all|C,G,D,...>]
    [--positions 5] [--max-span 4]
    [--tuning <TUNING>]
    [-o out.svg|out.txt] [--theme <THEME>]
```

`<INPUT>`: PcSet input (note names or integers) defining the chord type.
`--keys`: comma-separated root notes, or `all` (default all 12).
`--positions`: shapes per key (default 5).
`--max-span`: maximum fret span per shape (default 4).
`--tuning`: all 7 presets (standard, drop-d, dadgad, open-g, 7-string,
bass-4, bass-5).

**Current implementation:** Transposes the input PcSet to each requested
key, finds shapes via `find_chord_shapes()` from the fretboard module.
Text output lists shapes per key with fret notation and position range.
SVG output produces a grid (keys as rows, positions as columns) of
`FretboardBuilder` diagrams.

> **Note:** Shapes are chord voicings (playable grips), not sequential
> single-note arpeggio fingerings. A future BUILD could use
> `ScaleShapeSearchResult` / `n_note_per_string_shape()` for true
> melodic arpeggio patterns.

**Planned extensions:**
- Linear arpeggio fingering patterns (single-note-per-string).
- PDF output via SVG → `resvg`.

### `sight-reading`

Generate melodic exercises for sight-reading practice.

```
slonimsky sight-reading
    [--key <KEY>] [--scale <SCALE>]
    [--difficulty <1-5>]
    [--measures <N>]
    [--seed <N>]
```

`--key`: root note (default C). Accepts note names (C, G, Bb, F#).
`--scale`: one of `major`, `melodic-minor`, `harmonic-minor`,
`harmonic-major` (default `major`).
`--difficulty`: integer 1–5 (default 2). Controls interval vocabulary,
rhythm complexity, and note density per measure:

| Level | Intervals | Rhythms | Notes/measure |
|-------|-----------|---------|---------------|
| 1 | Steps only (±1) | Quarter notes | 4 |
| 2 | Steps + one skip (±1, ±2) | Quarter + eighth | 6 |
| 3 | Steps + skips + thirds (±1–3) | Quarter + eighth + half | 5 |
| 4 | Wide intervals incl. fourths (±1–3) | Syncopated (eighth + quarter + half) | 6 |
| 5 | Wide leaps incl. fifths (±1–4) | Complex (dotted quarter, mixed) | 7 |

`--measures`: number of measures to generate (default 4). Assumes 4/4 time.
`--seed`: RNG seed for deterministic, reproducible exercises. The seed
rotates the base interval pattern to produce variety; different seeds
yield different melodic contours for the same difficulty level.

**Current implementation:** Uses `MelodicSequencer` with
`IntervalPattern` configs per difficulty level. The sequencer operates
within `PitchBounds` of C3–C6, starting on the key root in octave 4,
with `Direction::Up` and `TurnaroundMode::Reflect` (bounces off range
boundaries). Output is text only (to stdout): a header block (key,
scale, difficulty, measure count, note count) followed by measures
formatted as `m1: C4 (quarter)  D4 (quarter) ...`.

Verbose (`-v`) prints pitch range statistics to stderr.

**Planned extensions:**
- LilyPond output (feature-gated) for printable sheet music.
- MIDI output via `SmfBuilder` for audio playback.
- SVG staff notation via the native engraver (post-engraver integration).
- Time signature options beyond 4/4.
- `--range <LOW>..<HIGH>` to constrain pitch bounds.

### `ear-training`

Generate MIDI quiz clips for interval/chord ear training.

```
slonimsky ear-training
    [--type <intervals>]                     # only `intervals` implemented; chords/progressions planned
    [--count <N>]                            # number of quiz items (default 10)
    [--seed <N>]                             # RNG seed for reproducibility
    [-o out.mid|out.json]                    # .mid → MIDI file, .json → JSON answer key
```

**Requires** the `midi` cargo feature: `cargo build -p slonimsky --features midi`.

`--type intervals`: each quiz item is a random root note (C3–C5) followed
by the root transposed by a random interval (0–12 semitones). Notes are
separated by rests in the MIDI file. The answer key (printed to stdout in
text mode, or written to `.json` if `-o answer.json`) lists each item with
root, target, semitone distance, and named interval (P1, m2, M2, m3, M3,
P4, TT, P5, m6, M6, m7, M7, P8).

**Current implementation:** Uses `midly` for SMF writing (standard MIDI
format 1, 2 tracks: tempo + notes). Random root selection via `rand` crate
with optional `--seed` for deterministic output. JSON output via `serde_json`.
Verbose mode (`-v`) prints byte count and item count to stderr.

**Planned extensions:**
- `--type chords` — random chord identification (play chord, name the quality).
- `--type progressions` — random ii-V-I / IV-V-I progressions for key identification.
- `--range <LOW>..<HIGH>` — constrain root note range (currently hardcoded C3–C5).
- Difficulty levels (e.g. beginner = P4/P5/P8 only, advanced = all intervals).

---

## Subcommands — Analysis & Pedagogy

### `analyze`

Harmonic/structural analysis of a chord progression.

```
slonimsky analyze <CHORD1> <CHORD2> [<CHORD3>...]
    [--key <KEY>]
    [--scale <SCALE>]
    [--format <text|json>]
```

Each `<CHORD>` is a comma-separated group of PCs or note names
(same convention as `progression` and `common-tones`).

`--key`: specify the key for analysis (e.g. `C`, `Bb`, `F#`). When
omitted, the key is estimated (see algorithm below).
`--scale`: one of `major`, `natural-minor`, `melodic-minor`,
`harmonic-minor`, `harmonic-major` (default `major`). Determines the
parent scale for Roman numeral assignment. `natural-minor` is also
accepted as `minor`.
`--format`: `text` (default) or `json` for machine-readable output.

**Output (text mode):**

```
Key: C major (estimated, confidence: 5/7)

  Chord    PcSet        Quality      Degree  Roman
  ───────  ───────────  ───────────  ──────  ─────
  C,E,G    {0, 4, 7}   Major        1       I
  A,C,E    {0, 4, 9}   Minor        6       vi
  D,F,A    {2, 5, 9}   Minor        2       ii
  G,B,D    {2, 7, 11}  Major        5       V

Common tones: C,E,G→A,C,E: {0, 4}  A,C,E→D,F,A: {9}  D,F,A→G,B,D: {2}
Voice-leading cost (L1): 5 → 5 → 5  total: 15
Symmetry: none detected
```

**Output (JSON mode):**

```json
{
  "key": { "root": "C", "scale": "major", "confidence": 5 },
  "chords": [
    {
      "input": "C,E,G",
      "pcs": [0, 4, 7],
      "quality": "Major",
      "degree": 1,
      "roman": "I",
      "diatonic": true
    }
  ],
  "transitions": [
    { "from": 0, "to": 1, "common_tones": [0, 4], "l1_cost": 5 }
  ]
}
```

**Key estimation algorithm** (implemented in `cmd/analyze.rs`, not in
the `music` library — this is CLI-level analysis logic, not a reusable
primitive):

1. Collect the union of all pitch classes across all chords in the
   progression.
2. For each of the 12 possible roots × each supported scale quality,
   generate the parent scale PcSet.
3. Score each candidate key by counting how many of the union PCs are
   diatonic (contained in the parent scale). Ties broken by: (a) prefer
   the root of the first chord, (b) prefer major over minor modes.
4. The top-scoring candidate is the estimated key. Confidence is the
   count of diatonic PCs out of total unique PCs.

When `--key` is given, skip estimation and use the provided key.

**Roman numeral assignment algorithm:**

1. For each chord, find which scale degree it's built on: iterate the
   parent scale's PCs and check if the chord root matches a scale degree.
   The degree is the 1-indexed position in the scale.
2. Determine the Roman numeral case from chord quality:
   - Major / Dominant / Augmented → uppercase (I, IV, V, etc.)
   - Minor / Diminished → lowercase (ii, iii, vi, vii, etc.)
3. Add quality suffixes: `°` for diminished, `+` for augmented, `7` for
   seventh chords, `maj7` for major sevenths.
4. Non-diatonic chords (root not in scale) are marked with `♯` / `♭`
   prefix (e.g. `♭VII`, `♯IV`) and flagged `"diatonic": false` in JSON.

**Library APIs used:**

- `infer_chord_quality()` — chord quality per chord.
- `infer_scale_quality()` — to validate the `--scale` parameter.
- `PcSet` set operations — for subset/superset testing (diatonicism).
- `IntervalMatrix` — for common-tone computation between adjacent chords.
- `Voiceleading::find_all()` — for L1 cost between adjacent chords.
- `PcSet::transpositional_symmetry()` — symmetry detection on the
  overall PC content.

**Implementation plan:**

1. Parse comma-group chords → `Vec<PcSet>` (reuse `parse_comma_groups`
   from `input.rs`).
2. If `--key` absent, run key estimation. If present, build parent scale
   from root + scale quality.
3. For each chord: infer quality, find scale degree, assign Roman numeral.
4. Compute pairwise common tones and L1 voice-leading cost.
5. Format as text table or JSON.

**Deferred extensions:**

- LilyPond input (via `lilypond-parser`, feature-gated) — parse a
  `.ly` fragment, extract pitch content, and analyze.
- `.ron` input — when `music-ron` lands, accept a `ChordProgression`
  document and analyze.
- Modulation detection — scan for pivot chords where Roman numeral
  assignment improves by switching keys mid-progression.
- Functional harmony labels (T/S/D) — map Roman numerals to tonic /
  subdominant / dominant function.

### `annotate`

Annotate a voiced progression with voice-leading quality metrics and
per-note classification.

```
slonimsky annotate <CHORD1> <CHORD2> [<CHORD3>...]
    [--key <KEY>]
    [--scale <SCALE>]
    [--no-crossings]
    [--format <text|json>]
    [-o out.txt|out.json]
```

Each `<CHORD>` is a comma-separated group of pitch names with octaves
(e.g. `C4,E4,G4 C4,F4,A4`). Unlike `analyze` (which works on abstract
PcSets), `annotate` requires concrete voicings with register information
so it can assess voice-leading quality.

`--key`, `--scale`: same as `analyze` — establishes the diatonic context
for non-chord-tone classification.
`--no-crossings`: flag voice crossings as warnings.

**Output (text mode):**

```
Key: C major

Step 1: C4,E4,G4 → C4,F4,A4
  Voice 1: C4 → C4  (0 st, common tone)
  Voice 2: E4 → F4  (+1 st, step)
  Voice 3: G4 → A4  (+2 st, step)
  L1 cost: 3   L∞ cost: 2   Crossings: none
  Smoothness: excellent (L1 ≤ 4)

Summary:
  Total L1 cost: 3
  Average L1 per step: 3.0
  Voice crossings: 0
  Smoothness rating: excellent
```

**Smoothness rating thresholds** (per step, for 3-voice textures):
- Excellent: L1 ≤ 4
- Good: L1 ≤ 7
- Fair: L1 ≤ 10
- Poor: L1 > 10

(Thresholds scale linearly with voice count: multiply by `voices/3`.)

**Non-chord-tone classification** (deferred — requires melody input,
not just chord-to-chord):

When a melody line is provided alongside the progression (future
`--melody` flag), each melody note is classified as:
- **Chord tone**: member of the current chord's PcSet.
- **Passing tone**: stepwise between two chord tones.
- **Neighbor tone**: step away from and back to a chord tone.
- **Suspension**: held from previous chord, resolves down by step.
- **Anticipation**: arrives early (member of the next chord).

This classification requires sequential pitch context and is deferred
until the `annotate` subcommand has a way to accept melody input
alongside the chord progression.

**Library APIs used:**

- `Voiceleading::find_all()` — enumerate voice-leadings.
- `Pitch::from_str()` / pitch parsing — for register-specific input.
- Same key estimation and Roman numeral logic as `analyze` (shared
  module or function).

**Implementation plan:**

1. Parse comma-group pitched chords → `Vec<Vec<Pitch>>`.
2. For each adjacent pair, compute the voice-leading (1:1 mapping by
   voice index — no permutation search, since voices are given in order).
3. Compute L1 (sum of absolute semitone motion), L∞ (max single-voice
   motion), and check for voice crossings.
4. Rate smoothness per step and overall.
5. If `--key` given, also run `analyze`-style Roman numeral assignment
   on the underlying PcSets.
6. Format as text or JSON.

**Deferred extensions:**

- `--melody <PITCHES>` — melody line for non-chord-tone classification.
- SVG staff diagram showing voice-leading lines and annotations.
- LilyPond output with annotations as markup.
- Symmetry hit detection (flag when a transition exploits transpositional
  or inversional symmetry of the chord pair).

---

## `.ron` Input Integration

> **Status: DEFERRED.** The `music-ron` crate does not yet exist in the
> workspace. When it lands, integrate as described here.

### `render` (or `from-ron`)

Top-level subcommand that takes a `.ron` file, dispatches on document
kind, and emits the appropriate output.

```
slonimsky render <FILE.ron> [-o out.svg|out.mid|out.ly]
```

**Planned document kinds** (to be defined in `music-ron`):

| Kind | `.ron` type tag | Output |
|------|----------------|--------|
| Pitch circle | `PitchCircle(...)` | SVG |
| Fretboard shape | `FretboardShape(...)` | SVG |
| Scale diagram | `ScaleDiagram(...)` | SVG |
| Interval matrix | `IntervalMatrix(...)` | SVG / text |
| Chord progression | `ChordProgression(...)` | MIDI / LilyPond / SVG |
| Snippet | `Snippet(...)` | LilyPond / MIDI |
| Tab | `Tab(...)` | LilyPond / SVG |

Each `.ron` file is a single tagged enum value. The `render` subcommand
deserializes, matches on tag, and delegates to the corresponding builder.

---

## Post-Engraver Integration Plan

> **Status: DEFERRED.** The `music-engraver` crate is being built in a
> sibling worktree on branch `task/music-engraver-build`. When types land
> on the base branch, integrate as described here.

### Feature flag: `engraver`

```toml
[features]
engraver = ["music-engraver"]
```

### `engrave` subcommand

Direct SVG/PNG output via the native Rust engraver, no external LilyPond.

```
slonimsky engrave <INPUT> [-o out.svg|out.png]
    [--font <bravura|...>]
```

### Routing strategy

When `--engraver` is set (or inferred from feature availability):

- Staff notation goes through `music-engraver` instead of LilyPond.
- Fretboard/pitch-circle diagrams continue using `music::svg::*Builder`
  (engraver is for staff notation, not diagrams).
- PDF output: engraver SVG → `resvg` → PDF, rather than LilyPond → PDF.

### Integration checklist (for future BUILD runs)

- [ ] Add `music-engraver` optional dep when types are available.
- [ ] Create `src/engraver.rs` dispatch module.
- [ ] Wire `engrave` subcommand.
- [ ] Add `--engraver` flag to staff-producing subcommands as alternative
      to `--ly` / LilyPond path.
- [ ] Test with Bravura font bundled in `music-engraver/fonts/`.

---

## Module Layout

```
slonimsky/
  Cargo.toml
  src/
    main.rs          — CLI entry, clap definitions, global arg handling
    cmd/
      mod.rs
      input.rs         — Shared input parsing (PcSet literals, note names, theme)
      pitch_circle.rs
      fretboard.rs
      spell.rs
      name.rs
      superchords.rs
      subchords.rs
      interval_matrix.rs
      interval_vector.rs
      chord_dictionary.rs
      voicings.rs
      voice_leading.rs
      closest.rs
      progression.rs
      orbits.rs
      prime_form.rs
      forte.rs
      common_tones.rs
      contains.rs
      practice_sheet.rs
      scale_book.rs
      arpeggio_dictionary.rs
      sight_reading.rs     — melody generation via MelodicSequencer
      ear_training.rs      — requires `midi` feature
      analyze.rs           — key estimation + Roman numeral analysis
      annotate.rs          — voice-leading quality metrics (L1, L∞, crossings, smoothness)
      render.rs            — .ron dispatch (planned, when music-ron lands)
      engrave.rs           — native engraver (planned, when music-engraver lands)
  examples/
    fixtures/          — .ron input fixtures
    scripts/           — shell scripts for realistic invocations
    output/            — generated artifacts (gitignored)
  tests/
    golden/            — frozen known-good outputs for regression
    *.rs               — assert_cmd integration tests (one per subcommand group)
```

Rationale: one file per subcommand keeps diffs small and merge-friendly
for the incremental build loop. `input.rs` lives inside `cmd/` because
it's shared only among subcommand handlers. Output format inference is
handled inline in each subcommand (no separate `output.rs` module) —
the pattern is simple enough that centralization adds indirection without
value.

---

## Design Decisions Log

| Decision | Rationale |
|----------|-----------|
| One file per subcommand in `cmd/` | Keeps incremental runs isolated; merge conflicts are rare. |
| `input.rs` shared parser | All subcommands accept the same input forms; DRY. |
| No `output.rs` module | Output format inference is simple (match on file extension); centralizing it would add indirection without value. Each subcommand handles its own output inline. |
| Feature-gate LilyPond behind `lilypond` | Matches workspace convention; avoids Tera dep for users who don't need `.ly`. |
| Feature-gate engraver behind `engraver` | Engraver not yet available; clean opt-in when it lands. |
| `music-ron` integration deferred | Crate doesn't exist yet; design is ready, code waits. |
| `--theme` is global | All SVG subcommands benefit; simpler than per-subcommand flag. |
| Comma-group separator for multi-set inputs | `common-tones C,E,G D,F,A` — commas delimit PCs within a set, spaces delimit sets. The `--` convention conflicts with clap's end-of-options behavior. |
| Prime form via Rahn's algorithm | Standard in set theory; no existing library function, but ~30 lines and PcSet-specific — not a reimplementation of SVG/MIDI/LilyPond. |
| Forte table as static const | Avoids runtime file I/O; complete catalog (223 entries × 2 fields, cardinalities 1–12). |
| `spell --format all` | Added beyond original design; prints all three formats on labeled lines. Useful for quick reference. |
| `name --config` deferred | Planned in design, not yet implemented. Default naming config is adequate for MVP. |
| `closest` symmetric-diff only | Voice-leading distance metric deferred; symmetric difference is simpler (set operation, not per-PC matching) and covers the primary use case. |
| `chord-dictionary` sorts by lowest fret | Gives open-position shapes first, which is the most natural ordering for guitarists exploring a chord. |
| `contains` auto-detects direction | ≤4 PCs → "which scales contain this?" (super); ≥5 PCs → "which chords are in this?" (sub). Covers the common case without requiring the user to think about set containment direction. |
| No A/B suffix on Forte numbers | Rahn's algorithm picks a single canonical prime form; both a set and its inversion map to the same Forte number. Simplifies output without losing information. |
| `voice-leading` uses `--from`/`--to` flags | The original `<FROM> -- <TO>` positional design conflicts with clap's `--` end-of-options convention (same reason `common-tones` uses comma-groups). Named flags are unambiguous and self-documenting. |
| `progression` uses comma-group positional args | Same convention as `common-tones`: `C,E,G F,A,C G,B,D` — commas delimit notes within a chord, spaces delimit chords. |
| `progression` greedy algorithm | Greedy step-by-step (pick lowest L1 at each step) rather than global optimization. Simple, fast, and produces musically reasonable results. Global search is deferred — combinatorial explosion for long progressions. |
| Feature-gate MIDI behind `midi` | `ear-training` (and future MIDI-producing subcommands) require `music-midi`, `midly`, and `rand` — non-trivial deps that most users don't need. Opt-in via `--features midi` keeps the default binary lean. |
| `ear-training --seed` for determinism | Reproducible quiz generation is essential for testing and for sharing specific quiz sets between students/teachers. |
| `ear-training` JSON output via `-o .json` | Machine-readable answer keys enable integration with external quiz/grading tools. Inferred from extension, consistent with the global output format convention. |
| Key estimation in `analyze` is CLI-level logic, not a library primitive | Key detection is heuristic and use-case-dependent (brute-force scoring over 12 roots × N scales). Not general enough for `music` crate; belongs in the CLI where the specific scoring/tiebreaking policy can evolve without API stability concerns. |
| `analyze` uses comma-group input (same as `progression`, `common-tones`) | Consistent input convention across all multi-chord subcommands. |
| `annotate` requires pitched input (with octaves), not abstract PcSets | Voice-leading quality metrics (L1, L∞, crossings) are meaningless without register information. This distinguishes `annotate` (concrete voicings) from `analyze` (abstract harmony). |
| Smoothness rating thresholds scale with voice count | A 3-voice L1 of 6 is "good"; a 6-voice L1 of 6 is "excellent". Linear scaling (`threshold × voices/3`) is simple and musically reasonable. |
| Non-chord-tone classification deferred from `annotate` MVP | Requires melody input alongside chords — a second input modality that complicates the argument surface. MVP covers voice-leading quality; NCT classification is a follow-up. |
| `analyze --scale` includes `natural-minor` | The Aeolian mode (natural minor) is fundamental enough to warrant a dedicated `--scale` value alongside the four parent scale families. Also accepted as `minor` for convenience. |
| `practice-sheet` sections labeled "subchords" not "diatonic triads" | `get_subchords()` returns all C(7,k) subsets of the scale, not just thirds-stacked diatonic chords. The label "3-NOTE SUBCHORDS" / "4-NOTE SUBCHORDS" accurately describes the content. |
