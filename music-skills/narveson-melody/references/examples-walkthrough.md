# Worked examples

Three end-to-end walkthroughs showing exactly what compose, analyze, and revise modes look like in practice.

---

## Example 1 — Compose: "stating phrase, classical, C major, 8 bars"

### User input

> Write me a classical stating phrase in C major, 8 bars.

### What the skill does internally

- **Style profile**: Classical preset (see `style-profiles.md`).
- **Harmony**: default tonic-prolongation in C major (I–V–I–V | I–IV–V–I).
- **Paragraph**: single stating phrase, 8 bars.
- **Shape**: Compound (2 curves) — classical's bread and butter.
- **Subsurface design**: Flexible part-to-whole — core stated then varied.
- **Core**: `C E G` (do mi sol — classical's prominent scale tones, opens the first arch).
- **Rhythm template**: `common_time_anacrustic` + `common_time_quarters` mixed.
- **Extraneant**: one interrupting-repetitive pattern at the cadence (typical classical).
- **MTC**: central-tone, MTC = C (do).
- **Quality (after revision)**: Strong.

### Output to the user

```lilypond
\version "2.24.0"
\header { title = "Stating phrase — Classical, C major" }

\score {
  \new Staff {
    \clef treble  \key c \major  \time 4/4  \tempo 4 = 92

    % bars 1-4: first arch — core stated, ascends to peak g'
    c'4 e'   g'2          |
    a'4 g'   f'2          |
    e'4 d'   c' d'        |
    e'2     ~ e'4 r4      |

    % bars 5-8: second arch — core varied, descends to MTC
    g'4 e'   c' d'        |
    e'4 f'   g'2          |
    f'4 e'   d' d'        |     % interrupting-repetitive on d'
    c'1                   \bar "|."
  }
  \layout {}
  \midi { \tempo 4 = 92 }
}
```

**One-line summary above the artifact:** "Classical stating phrase, Compound shape, do-mi-sol core stated then varied, ending on tonic via cadential repeat."

---

## Example 2 — Analyze: "what's the shape of c'4 d' e' f' g'2 e' f'4 d' e' c' g'1"

### User input

> Analyze this phrase Narveson-style: `c'4 d' e' f' g'2 e' f'4 d' e' c' g'1`

### What the skill does internally

1. Parse the LilyPond into pitches + durations.
2. Walk the pitch stream:
   - `c' d' e' f'` — 4 pitches stepwise ascending → **scale element 1** (sc., direction up, range P4).
   - linking note: f'.
   - `f' g'` — wait, f' g' is also stepwise; reconsider: the run is `c d e f g`, 5 pitches stepwise up. → **scale element 1 extends through g'**.
   - `g' e'` — descending m3 → **skip element 1**.
   - `e' f'` — stepwise → **appoggiatura 1**.
   - linking note: f'.
   - `f' d'` — descending m3 → **skip element 2**.
   - `d' e'` — stepwise up → **appoggiatura 2**.
   - linking note: e'.
   - `e' c'` — descending M3 → **skip element 3**.
   - linking note: c'.
   - `c' g'` — ascending P5 → **skip element 4**.
3. Curvature: c→g (up), g→e→f (down-up), f→d→e (down-up), e→c→g (down-up). Counting direction changes: 4 changes. Curvature ≈ 2½ curves.
4. With 2½ curves, this fits a **Compound shape** (expanded ceiling). No extraneants seem to apply cleanly — the deviating pitch rule fails (every note is on a beat), and there's no x-y-x return pattern.
5. Range: c'–g' = P5. Pitch level: roughly e' (halfway).
6. MTC: first pitch c', last pitch g' — both diatonic in C major, **skip apart (P5)** → two-indicative-tones chart row 1 → MTC = **g'** (last note).
7. Phrase type: with clear cadential gesture and prominent material, this is a **stating phrase**, possibly Stating-Concluding hybrid (ends on V, not I — so still wants more).
8. Interest features: Variety (moderate — rhythmic variety only from the half-note g'2); Irregularity (low); no DelayedCompletion or SurpriseCompletion observable. Quality: **Standard**.

### Output to the user

```
Elements (5):
  sc.1   c' d' e' f' g'           up, range P4
  s.1    g' e'                    skip down m3, linking on g'
  a.1    e' f'                    step up,  linking on e'
  s.2    f' d'                    skip down m3, linking on f'
  a.2    d' e'                    step up,  linking on d'
  s.3    e' c'                    skip down M3, linking on e'
  s.4    c' g'                    skip up  P5,  linking on c'

Curvature: 2½ curves (4 direction changes)
Primary shape:   Compound (expanded, ceiling 2½)
Range:           P5 (c'–g')
Pitch level:     e'
Secondary shape: Skeletal (pitches on principal accents: c' g' f' e' g')

MTC:    g'    (two-indicative-tones chart, skip-apart row, last note rules)
Type:   Stating (or Stating-Concluding given the ends-on-V close)

Interest:
  Variety:        moderate (rhythm only)
  Irregularity:   low
  DelayedCompl:   —
  SurpriseCompl:  —
  Compensation:   respected
  Subordination:  respected
Quality: Standard

Style fit: closest to Baroque (element ranges ≤ octave; stepwise primary; do-and-sol prominent),
           but no chromaticism or modulation observed.
```

If the analysis is ambiguous (e.g. could-be-Compound-or-modified-Simple), state both with reasoning rather than picking arbitrarily.

---

## Example 3 — Revise: "improve this phrase"

### User input

> Revise this Narveson-style: `c'4 d' e' f' g'2 e' f'4 d' e' c' g'1` — make it strong quality.

### What the skill does internally

Re-run analyze (see example 2). Then apply the five passes:

**Pass 1 — Warmup**. No gross errors found. No change.

**Pass 2 — Design**. Shape is Compound at ceiling (2½ curves). Marginally too busy. Considered reducing to 2 curves cleanly. Decision: keep curvature; the extra half-curve is intentional motion to V.

**Pass 3 — Interest**. Variety is moderate; irregularity is low. Add interest:
- Replace the bare `f'4` in bar 3 with `f'8 e'8 f'4` — adds an irregularity (rhythmic surprise + subtle neighbor figure).
- Delay the resolution: change the final `c' g'1` to `c' b1 c''2` — a tendency-tone delay that creates DelayedCompletion, with `c''` as the satisfying arrival. (But this changes the MTC reading — note that to the user.)

**Pass 4 — Hidden-core**. Original core looks like `c d e f` (the opening scale element). It only appears once. Plant a reuse: change `f' d' e' c'` near the end to `f' e' d' c'` (descending scale element) so the original ascending core has a descending answer (Flexible design — core varied, not literal).

**Pass 5 — Final-touch**. Add a slur over the opening scale element. Add `\f` dynamic at start, `\p` at the climactic g'2, then `\f` again at the answering line. Suggest `\tempo 4 = 88`.

### Output to the user

```lilypond
\version "2.24.0"
\header { title = "Revised stating phrase" }

\score {
  \new Staff {
    \clef treble  \key c \major  \time 4/4  \tempo 4 = 88

    \f
    c'4( d' e' f') g'2 \p e' |
    f'8 e' f'4 d' e' c'      |
    f' e' d' c' \f           |
    c'4 b c''2               \bar "|."
  }
  \layout {}
  \midi { \tempo 4 = 88 }
}
```

**Diff log:**
- Pass 1: no change.
- Pass 2: no change.
- Pass 3: replaced `f'4` with `f'8 e' f'4` (variety + irregularity); changed cadence `c' g'1` → `c' b c''2` (DelayedCompletion: `b` is tendency-tone `ti` resolving to `c''`).
- Pass 4: replaced second-half scale fragment with descending scale element to plant core reuse.
- Pass 5: added phrase-opening slur; dynamics `\f \p \f`; tempo `quarter = 88`.

**New analysis (delta):**
- MTC has shifted from g' to c'' (central-tone formation: first c', last c'' — same pitch octave apart → MTC = c'). Closes more conclusively now.
- Interest: Variety high, Irregularity moderate, DelayedCompletion present. **Quality: Strong.**

---

## Notes for the skill author (you)

- The textual outputs above are aspirational — they show the *target format*. When you don't have the bandwidth in a turn to produce all of this, prioritize: artifact first (the LilyPond or analysis table), then a one-line summary, then the diff log if revising.
- LilyPond rhythm syntax: `c'4` is C above middle-C, quarter; subsequent notes inherit the duration until a new one is given. `c''` is one octave higher. Common rhythmic suffixes: `1` whole, `2` half, `4` quarter, `8` eighth, `16` sixteenth; dot `.` lengthens. `r` is a rest.
- When showing elements in the analysis table, use Narveson's abbreviations (sc., a., c., s., n., p.) and the form (direct vs background) only when ambiguous.
- When in doubt about a shape, **say so** and present alternatives. Narveson himself says some phrases are deliberately ambiguous.
