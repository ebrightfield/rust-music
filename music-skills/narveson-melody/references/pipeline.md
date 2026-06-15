# Pipeline reference

The per-phrase compose pipeline and the five-pass revision pipeline, in operational detail.

Load this in compose and revise modes.

---

## Per-phrase compose pipeline (steps 5a–5h)

Run for each phrase in the paragraph. Carry the previous phrase's analysis in mind so transitions and reuses are coherent.

### 5a. Choose primary phrase shape

Pick consistent with the phrase's role:

| Role | Default shape preference |
|---|---|
| Introducing | Simple or Compound (direct shapes; introducing material should be clear and graspable) |
| Stating | Compound (2 curves) — most "song-like"; Simple when the paragraph is short |
| Extending | Whatever the thesis used (continuity); often simpler |
| Transisting | Skeletal or Cross-Wave (reduced; less melodically prominent — function dominates) |
| Concluding | Simple or modified Simple with a cadential interrupting-repetitive extraneant |

Override with style: Romantic likes Compound + Cross-Wave; Contemporary likes Cross-Wave or modified-Compound with deviating pitches; Baroque likes Simple/Compound + secondary skeletal.

### 5b. Pick subsurface design

| Subsurface design | When to use |
|---|---|
| **Flexible** (part-to-whole) | Core appears with variation between uses — good for stating phrases where the listener should hear "the idea, transformed". |
| **Attachment** (part-to-whole) | Core uses joined by connective material — good for extending phrases ("we're stretching the same idea"). |
| **Shape-elaborated** (whole-to-part) | Shape is decided first, then filled — good for transisting phrases (function over motive). |
| **Background-elaborated** (whole-to-part) | Background elements (rbs) first, surface second — good for the Tchaikovsky quirk and for tonal-orientation phrases. |

### 5c. Generate the core

A core is 3–5 pitches and/or a short rhythm cell. Three approaches:

1. **Stylistic core** — match the style's prominent scale tones (e.g. Baroque: `do mi sol`). Default for stating phrases.
2. **Inversion / contour core** — pick a contour (up-up-down, up-down-up) and instantiate it in the active scale. Default for transisting and extending.
3. **Quirk-driven core** — for composer quirks: Hindemith `do fa sol`, Brahms `do mi sol mi`, Bartók `do fa fi do`, Tchaikovsky `do re mi fa sol` (literal scalar).

Record the core somewhere mentally — you'll need it for the hidden-core revision pass.

### 5d. Lay the core onto the shape skeleton

- **Direct shapes**: place the core at the phrase's pitch-level (the halfway pitch in the range), then extend the half-curves outward from there.
- **Concealed shapes**: place core notes on principal accents (skeletal) or on the chosen pulse weight (cross-wave). Filler notes go between.

Plan 1–3 core reuses depending on phrase length:
- 4–6 measures: 2 reuses (statement + answer pair).
- 7–10 measures: 3 reuses (statement + sequence + answer).
- For Flexible design, vary each reuse; for Attachment, keep reuses identical.

### 5e. Fill the surface with elements

For each gap between core reuses, choose elements that obey:

- **Chain-of-elements rule**: each non-final element shares its last pitch with the next element's first pitch.
- **Style range policy**: don't blow past the era's element-range thresholds.
- **Tendency-tone treatment**: resolve or nonresolve according to style.
- **Chord membership**: surface pitches on principal accents should be chord tones of the current harmony; off-accent pitches may be passing/neighbor/appoggiatura.
- **Pitch-set policy**: hard reject in Strict mode; pay deviation cost in Soft modes.

Common element choices:

| Want to connect... | Use... |
|---|---|
| Two close pitches (≤ M2) | Appoggiatura (single step), or scale element if it's part of a run |
| Two pitches ≥ m3 apart | Skip element, or chord element if there are intervening chord tones |
| A pitch back to itself with ornament | Neighbor element |
| A pitch back to itself across distance | Pedal element |
| Three or more in a row in one direction | Scale element |

### 5f. Apply extraneants (direct shapes only)

Decision flow:

1. Does the style call for extraneants? Baroque rarely; Romantic often; Contemporary sometimes (deviating pitch especially).
2. If yes, pick: dangling return / interrupting repetition / deviating pitch.
3. Respect count limits (digest §4.1). Place legally.
4. Re-check curvature — extraneants shouldn't push curvature past the shape's ceiling.

### 5g. Snap to rhythmic template

Pick a template from the library below (or one the user supplied). Walk through slot by slot, assigning each filled-in pitch its duration. Subdivide or tie as needed (per `RhythmFlex`).

**Swing decision.** If the style is jazz (bebop_eighths, swing-feel ballad, any jazz composer quirk), the rendered output needs to *sound* swung even though the score should *look* straight. Default to the **two-`\score` + `\articulate` pattern** documented in `references/output-formats.md` → "Jazz-chart template". Notation guidance:

- Notate the **score as straight 8ths**. Don't litter it with `\tuplet 3/2 { x4 y8 }` brackets unless the player needs to see actual triplets.
- Add `\include "articulate.ly"` and a second `\score` block wrapping the body in `\unfoldRepeats \articulate { ... }` for MIDI.
- Real triplets (a triplet flurry, a 3:4 polyrhythm) **stay as triplets** — those aren't swing notation, they're actual triplets.

For non-jazz styles (classical, romantic, folk, modal), skip the pattern — a single `\score` block with `\layout` + `\midi` is enough.

### 5h. Tag the analysis

Record alongside the phrase:
- Elements (ordered, with linking notes marked)
- Primary + secondary shapes
- Subsurface design
- Core + reuse locations
- MTC analysis
- Style profile in effect

This analysis is consumed by the revision pipeline. If you skip it, you'll need to re-derive it during revision.

---

## Rhythm template library

Quick presets to apply at step 5g. All assume 4/4 unless noted.

```
common_time_quarters:        Q Q Q Q (1 bar) — baroque/classical defaults
common_time_anacrustic:      e | Q Q Q e (pickup eighth + 1 bar)
common_time_with_sixteenths: e s s Q e s s | Q Q Q Q
common_time_dotted:          d. e Q Q (one bar; dotted-quarter + eighth + 2 quarters)

triple_meter_3_4:            Q Q Q (one bar 3/4 — minuet/scherzo default)
compound_6_8:                e e e e e e (one bar 6/8 baseline)
sicilienne_6_8:              d. e e e | d. d. (lilting; pairs of bars)
gigue_12_8:                  d Q e (rep) | (one bar 12/8 — bouncy compound)

bebop_eighths:               e e e e e e e e (uniform eighths; jazz/contemporary)
ballad:                      h Q e e | h. e e (slow; long-short)
march:                       Q Q e e Q | Q Q Q Q (military / processional)
hemiola_2_against_3:         in 6/8: Q Q Q (treats 6/8 as 3/4 for a bar)

contemporary_irregular_5_8:  e e e Q (one bar 5/8)
contemporary_irregular_7_8:  e e e e e Q (one bar 7/8)
```

Notation key: `s` = sixteenth, `e` = eighth, `Q` = quarter, `d` = dotted-quarter, `h` = half, `d.` = dotted-quarter (alt), `h.` = dotted-half.

When the user supplies their own template, treat it as the override.

---

## Five-pass revision pipeline

Run each pass once per invocation. Mutate the phrase locally; track what changed for the diff log.

### Pass 1 — Warmup revision

**Looks at:** gross errors and obvious infelicities.

Checklist:
- All pitches inside the bounds (default C3–C6, or user-specified).
- No broken chain-of-elements (every non-final element has a linking note to the next).
- No illegal A2 placements (digest §1 notation gotchas).
- No false chord elements (chord-element pitches that include a stepwise pair internally).
- Phrase length sane (3–10 elements; revisit if outside).
- Time signature notations consistent (durations sum to a whole number of beats per measure).

**Fix moves:** clip out-of-range pitches by octave-shifting; insert linking notes by repeating the prior pitch one beat earlier; reclassify a stepwise pair inside a chord element as appoggiatura + chord.

### Pass 2 — Design revision

**Looks at:** shape integrity and extraneant rules.

Checklist:
- Curvature count matches claimed shape (½/1 for Simple; 1½/2/2½ for Compound; lesser/greater multicurve for concealed).
- Extraneants legal: dangling return location, interrupting repetitive count, deviating pitch off-beat + ≥3-prior-notes rule.
- Skeletal shape: every principal accent has a pitch (or echoes preceding pitch on rest).
- Cross-wave: pulse-weight consistency holds across the phrase.
- Linking notes correctly identify the last repeated note when an element ends in repetition.

**Fix moves:** remove a directional change that broke curvature count; move a deviating pitch to the next off-beat; collapse two appoggiaturas into a scale element when three pitches step in one direction.

### Pass 3 — Interest revision

**Looks at:** interest features and the two laws.

Checklist for *features* (count and magnitude):
- Variety: rhythmic + intervallic variety across phrase. Score low if uniform.
- Irregularity: at least one rhythmic or intervallic surprise.
- Subtlety: small refinements (passing chromaticism, agogic accents).
- DelayedCompletion: tendency tone resolves later than expected — at least one in any phrase aspiring above Standard.
- SurpriseCompletion: an unexpected target pitch — sparingly.

Checklist for *laws*:
- **Compensation**: every tension followed by a compensating relaxation (or carried over to the next phrase deliberately).
- **Subordination**: no interest feature so prominent it upstages the core or the primary shape.

**Fix moves:** add a chromatic passing tone (variety + subtlety); delay a tendency-tone resolution by an eighth (DelayedCompletion); replace a predictable cadential pitch with the next-most-likely (SurpriseCompletion); reduce an interrupting-repetitive count from 3 to 2 (subordination).

Re-score; assign a quality grade: Inferior < Imperfect < Standard < Strong < Superior. Most phrases should hit Standard after this pass; revise again or accept.

### Pass 4 — Hidden-core revision

**Looks at:** the core and its reuses.

Checklist:
- Core appears at least twice in the phrase (literal, transposed, or varied per the subsurface design).
- At least one reuse lands on a structurally important position (downbeat of a strong measure, or first/last pitch of the phrase).
- For Flexible design: reuses differ enough to feel like variation, not repetition.
- For Attachment design: reuses are close enough to feel like the same idea.
- Master core (paragraph-level) appears at least once if this phrase is the stating or concluding phrase.

**Fix moves:** retrofit a missing reuse by replacing a generic scale-element segment with the core; transpose an existing reuse to make it more recognizable; tighten core spans by removing intervening notes.

### Pass 5 — Final-touch revision

**Looks at:** surface polish.

Checklist:
- Contour smoothness: no isolated leaps > octave unless style asks for it.
- Articulation: choose slurs (legato), staccatos (detached), accents per style.
- Last-note resolution: phrase ends on a stable pitch consistent with MTC.
- Dynamics suggestion: an opening dynamic + a phrase-end dynamic per style (Baroque: terraced f/p; Romantic: gradual cresc/dim; Contemporary: explicit).
- Tempo marking suggestion if not given.

**Fix moves:** add a slur over a step-element; convert two same-pitch notes to a tie; add a final fermata in concluding phrases for a stronger close.

---

## Quality grade scoring (heuristic)

Apply after the interest pass:

| Grade | Roughly |
|---|---|
| Inferior | Missing features; law violations uncorrected |
| Imperfect | Features present but laws partially violated |
| Standard | Features present + laws respected; baseline good melody |
| Strong | Multiple notable features; clean laws; clear core; tasteful extraneants |
| Superior | Strong + a single unifying gesture; cross-phrase coherence; standout phrase |

Don't be afraid to grade Standard. Most of the world's good melodies are Standard.

---

## Determinism note

When the user wants reproducibility, hash their seed string into your local decisions: at each choice point, pick the first viable option ordered alphabetically by name (element kind, shape kind, etc.). Tell them you're doing it so they can verify by re-running.
