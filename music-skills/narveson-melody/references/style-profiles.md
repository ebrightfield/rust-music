# Style profiles

Each profile is a compact record. Load this file when picking or sketching a style. Combine an era preset with at most one composer quirk for most realistic results.

For the underlying theory, see `narveson-digest.md` §6 and §7.

---

## How to read a profile

```
Era:                     Baroque | Classical | Romantic | Contemporary | Custom
Element ranges:          (scale / chord / skip behavior)
Chord guise bias:        (which guises dominate chord elements)
Background scale (rbs):  (variety policy across phrases)
Tendency-tone treatment: (respect / nonrespect rate; resolution timing)
Prominent scale tones:   (top-2 tones in concealed shapes)
Overall bg scale span:   (range of overall-background-scale element)
MTC approach:            (central-tone | two-indicative-tones | avoidance)
Composer quirk:          (optional one-off bias)
Interest weight bias:    (which interest features the style rewards)
```

---

## Era presets

### Baroque

```
Era:                     Baroque
Element ranges:          infrequently exceed an octave (sc/c/s)
Chord guise bias:        FamiliarChord, IncompleteChord
Background scale (rbs):  stays in one major/minor scale across the phrase (occasionally one transition)
Tendency-tone treatment: nearly always respects resolution (immediate or delayed); stepwise resolutions inside small rbs elements
Prominent scale tones:   do, sol  (both major and minor)
Overall bg scale span:   do→sol or do→do
MTC approach:            central-tone, MTC=do; or two-indicative-tones with skip-apart constituency
Composer quirk:          —
Interest weight bias:    DelayedCompletion (moderate), Subtlety (high); compensation expected swiftly
```

### Classical

```
Era:                     Classical
Element ranges:          significantly often well exceed an octave
Chord guise bias:        FamiliarChord (V7, fundamental triads dominate); IncompleteChord
Background scale (rbs):  stays in one major/minor scale; clear modulations between phrases
Tendency-tone treatment: respects resolution; clearer cadences than baroque
Prominent scale tones:   do, sol  (both scales)
Overall bg scale span:   do→sol or do→do
MTC approach:            central-tone, MTC=do; two-indicative-tones (skip-apart) common in transisting phrases
Composer quirk:          —
Interest weight bias:    Variety (high), Irregularity (moderate); strong compensation; phrase symmetry valued
```

### Romantic

```
Era:                     Romantic
Element ranges:          chromatic scale element often well exceeds P4, up to 2+ octaves
Chord guise bias:        FamiliarChord with chromatic appoggiaturas; ExtendedNonchord on emotional peaks
Background scale (rbs):  one major/minor primarily, with chromatic detours
Tendency-tone treatment: respects but emphasizes the prolonged tendency over the resolution — tension lingers
Prominent scale tones:   mi, sol  (both scales)
Overall bg scale span:   mi→do / mi→mi (pseudo-phrygian) or sol→mi / sol→sol (pseudo-mixolydian)
MTC approach:            central-tone with MTC=mi or sol; two-indicative-tones (skip-apart) common
Composer quirk:          —
Interest weight bias:    DelayedCompletion (very high), SurpriseCompletion (moderate); compensation can be delayed across phrases
```

### Contemporary

```
Era:                     Contemporary
Element ranges:          rarely exceed P4 (compressed)
Chord guise bias:        ModernChord, Polychordal, ExtendedNonchord
Background scale (rbs):  varied traditional scales (free or systematic); church modes common
Tendency-tone treatment: significantly often nonrespects resolution — even strong tendencies may not resolve
Prominent scale tones:   do + 1–2 non-do/non-sol tones, optionally sol; depends on chosen mode
Overall bg scale span:   either do→sol/do in the few that use overall bg scale, or deliberate scalar confusion
MTC approach:            central-tone (single scale); two-indicative-tones (step-apart preferred); or avoidance
Composer quirk:          —
Interest weight bias:    Irregularity (high), SurpriseCompletion (high); compensation often replaced by recontextualization
```

---

## Composer quirks (apply on top of an era preset)

### Hindemith

```
Quirk:                   Predominant melodic skip = P4
Implementation hint:     When choosing a skip element, P4 weights ≈ 5x normal; tritones and P5 next; thirds suppressed
Best paired with:        Contemporary
```

### Brahms

```
Quirk:                   Predominant intervals = M3 and m3 (melodic AND background)
Implementation hint:     Skip element bias toward 3rds; rbs elements often outline 3rd-stacks; cores favor 3rd-leaps
Best paired with:        Romantic (or late Classical)
```

### Bartók

```
Quirk:                   Mixture of P4 and tritone (melodic and background)
Implementation hint:     Skip element ≈ 50% P4 / 30% tritone / 20% other; modes drift toward acoustic / Lydian-Mixolydian
Best paired with:        Contemporary
```

### Tchaikovsky

```
Quirk:                   Themes are lightly elaborated background scale elements
Implementation hint:     Stating phrase's surface closely tracks its overall background scale — minimal embellishment over a clear scalar arc; concluding phrases extend with sequential extending phrases
Best paired with:        Romantic
```

---

## Sketching a custom profile

When the user describes a style verbally rather than naming a preset, fill in each axis using their description as hints. Reasonable defaults if they don't address an axis:

| If user says... | Profile axis biased toward... |
|---|---|
| "modal", "folksy", "ethereal" | Church modes; tendency-tone nonrespects; MTC=do; church-mode element ranges |
| "jazzy" | ExtendedNonchord guise; tendency-tone respects but with delayed resolution; chord element range > octave; prominent extensions |
| "minimal" | Compressed ranges; pedal element prominent; flexible subsurface design with literal core reuse |
| "intense", "dramatic" | Romantic preset + DelayedCompletion weight pushed higher |
| "playful", "light" | Classical preset + element ranges slightly compressed |
| "atonal", "12-tone" | Contemporary preset; MTC avoidance; no prominent scale tones; pitch-set strict over a row |
| "Eastern" / "raga-like" | Pitch-set strict over chosen scale; ornaments prominent (neighbor + appoggiatura); pedal element common |

A custom profile is a perfectly valid output of style-fit mode — record it inline so it can be reused later in the conversation.

---

## Style-fit shortcuts

When the user gives a snippet for style-fit mode, look for these tells:

| Tell | Implies |
|---|---|
| Element ranges all ≤ octave, lots of stepwise | Baroque |
| Wide leaps + clear V→I cadences | Classical |
| Chromatic appoggiaturas, prolonged tension | Romantic |
| Compressed range + nonresolved tendencies | Contemporary |
| Repeated P4 leaps | Hindemith quirk |
| Stacked 3rds in skip element | Brahms quirk |
| P4 + tritone alternation | Bartók quirk |
| Surface closely tracks a scale arc | Tchaikovsky quirk |
| First and last notes match | central-tone MTC |
| First and last differ by skip (not tritone/M7/m7/9) | two-indicative-tones MTC, last = MTC |
| First and last differ by tritone/M7/m7/9 | MTC avoidance |

A snippet of < 2 phrases yields low confidence — say so when reporting the fit.
