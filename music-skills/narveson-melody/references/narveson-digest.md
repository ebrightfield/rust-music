# Narveson digest — the rules in operational form

This is the compressed rulebook used by the skill. Source: Paul Narveson, *Theory of Melody*. Use it during analyze and compose modes when you need to apply or check a specific rule.

For the full design context, see `docs/narveson-melody-system.md` at the workspace root.

---

## 1. Six melodic elements (the atomic vocabulary)

Every phrase consists entirely of elements drawn from this list. Each (except pedal) has a **direct form** and a **background form**.

| Element | Abbr | Essence | Count rule | Direction rule | Notation gotchas |
|---|---|---|---|---|---|
| **Scale** | sc. | Stepwise motion | ≥3 pitches | One direction only | A2 belongs here only as a "gap" along an established scale (pentatonic, upper harmonic minor) |
| **Appoggiatura** | a. | Single step | exactly 2 pitches | Either | Never written-A2; any A2 belongs to scale-as-gap or to a skipwise element |
| **Chord** | c. | Skipwise (possibly mixed shake) | ≥3 pitches | May freely mix | May include written A2's (read as skips). False chord elements: any stepwise motion between chord members → break into smaller elements |
| **Skip** | s. | Single skip OR skipwise-isolated shake | 2 (skip) or ≥3 (shake) | One pair / shake | Written-A2 reads as a skip here too. Skipwise-isolated shake is *the skip element*, not the chord element |
| **Neighbor** | n. | Trillwise (alternation across one step) | ≥3 pitches | Alternates | Basic interval is m2 / M2 / rarely d3, never A2 |
| **Pedal** | p. | One pitch appearing ≥2× with rhythmic consistency | ≥2 occurrences, non-consecutive | static | **Only element with no background form**. Layout kinds: regular, agogic, asorhythmic, agogic-with-regular, combined |

Chord guises (for chord elements only): `FamiliarChord`, `ModernChord`, `Polychordal`, `IncompleteChord`, `ShortNonchord`, `ExtendedNonchord`.

## 2. The chain-of-elements rule

When two or more direct elements appear in a phrase:

1. Each non-final direct element joins the next on **one common note** — the **linking note**.
2. If a non-final element ends with repeated notes, the **last** repeated note is the linking note.
3. A change of element may or may not entail a change of direction (independent decisions).

Pedal elements always coincide with notes of the direct portion (their member pitches are also notes of some direct element).

Typical phrase has **5–10 elements** total.

## 3. Time factors (apply to any element)

Optional variables, selectively used or omitted: rests, caesura / breathing point, accent placement (highlight or de-emphasize), repeated notes, time-value choices, combinations. Repeated notes never change the underlying pitches.

## 4. The four phrase shapes

Curvature = number of arches (one direction change per arch). Counted in half-curves: ½, 1, 1½, 2, 2½, etc.

| Shape | Family | Allowed curvature | Determination |
|---|---|---|---|
| **Simple** | Direct (nonreduced) | 1 curve (also ½ as floor) | Total config = 1 arch |
| **Compound** | Direct (nonreduced) | 2 curves (1½ contracted, 2½ expanded ceiling) | Total config = 2 arches |
| **Skeletal** | Concealed (reduced) | Lesser multicurve only (≈ ½ to 1½ between principal accents) | Pitches on principal accents form the shape |
| **Cross-wave** | Concealed (reduced) | Greater multicurve (≈ ≥ 1½ between principal accents) | Pitches on consistent-weight pulses (usually beats or main beats) |

For each shape:
- **Range** = interval between extreme pitches.
- **Pitch level** = pitch halfway in the range (lower of two when tied — gravity).
- **Proportions** = ratio of half-curves' ranges (varies by shape; see Narveson §02.02).

### 4.1 Modified direct shapes — extraneants

A direct shape can have up to three extraneants (non-shape leftovers):

| Extraneant | Constraint | Count limit |
|---|---|---|
| **Dangling return** (`x-y-x` pattern fastened from outside one end) | Adjacent motion segments must be contrary; shape pitch nearest the attaching note must differ from all return-pattern pitches | 1 per end (at most 2 total) |
| **Interrupting repetitive pattern** (repeats an adjacent segment any number of times) | Repeat segment must be immediately adjacent to where extraneant interrupts; case 2 (segment-after) only valid when segment is final | Modified simple: ≥1 allowed. Modified compound: exactly 1 |
| **Deviating pitch** (single pitch participating in a 3-pitch motion that begins contrary then over-extends the half-curve) | Off-beat (rule 1); preceded by ≥3 prior notes of the half-curve, ≥2 distinct in pitch (rule 2) | At most 1 per shape |

Co-use of extraneants is allowed (any 2 or all 3), each within its own quantity limit; locations must not nest inside another extraneant.

### 4.2 Concealed-shape selection rule

If a phrase, after removing potential extraneants, has ≥3 curves OR has inadmissible extraneants → it gets a concealed shape, not a modified direct one.

Lesser multicurve (≈ ½–1½ curves between principal accents) → skeletal. Greater multicurve (≥ 1½) → cross-wave.

### 4.3 Skeletal pitch position rules

1. Principal accents = downbeats (default) or other locations elevated to downbeat-equivalence (notated accents, agogics, contextual).
2. If a rest falls on a non-first principal accent, the preceding pitch echoes there and counts as the skeletal pitch.
3. Skeletal pitches are normally non-consecutive in the parent phrase; sometimes consecutive (repeated, tied, or different).

### 4.4 Cross-wave position rules

1. All pitches on either the beats or main beats (context picks which).
2. Occasional pitches may be on a different pulse via accent-shifting.
3. Rest on a required pulse → preceding pitch counts.
4. Some pitches may be consecutive.

### 4.5 Simultaneous secondary shapes (every phrase has ≥1)

- Phrase with a **direct** primary → secondary skeletal (always) + theoretical secondary cross-wave (accept only if it differs significantly from primary and other secondary).
- Phrase with **skeletal** primary → secondary cross-wave (always) + theoretical secondary cross-wave on next-smaller pulse (same accept rule).
- Phrase with **cross-wave** primary → secondary skeletal (always) + theoretical secondary cross-wave (same).
- A **secondary direct** shape can exist only when the primary is concealed; at most one per phrase; rarely useful in practice.

### 4.6 Successive shape combinations

Practically relevant only across a period (typically antecedent + consequent). Rule of thumb: combinations grand-totaling **≤ 2½ curves**.

## 5. The eight phrase types

Three accompanying + five melody-line.

### 5.1 Accompanying (all share essential regularity)

Listed from most to least superficially complex:

1. **Elaborate** — fast, nondescript note patterns; near-uniform durations; roles: arpeggiated series, secondary melody (countersubject/obbligato), heterophonic device.
2. **Semi-melodious** — fixed formula: main motive series with simplified lead-in / tail; if both attached, durations near-uniform; roles: secondary melody mostly.
3. **Rudimentary** — nondescript, simple. **By far the most common**; often 2–4 simultaneously; roles: emphasis device, doubling line, fundamental bass, quasi-ostinato.

Any accompanying phrase can switch role about halfway through and keep identity.

### 5.2 Melody-line (essential irregularity)

The natural order in a paragraph: **introducing → stating → transisting → concluding**, with extending phrases attaching to any thesis to form *areas*. Fragmentary additions are short tails (2–3 measures) appended to a complete phrase.

| Type | Function | Frequency |
|---|---|---|
| **Introducing** | Opens the paragraph; own meter/tempo possible; material distinct | "Basically a luxury"; commonly omitted |
| **Stating** | Presents the main melodic idea; most fully melodious; establishes initial MTC if tonal | **Always used** (only indispensable melody-line phrase) |
| **Extending** | Dependent. Immediately or soon follows a thesis; expands on it. Creates areas | Commonly ≥1; often 3–4 per paragraph |
| **Transisting** | Between dissimilar phrases/areas; effects an orderly change. Often takes form of elaborate accompanying ("elaborate transisting") | Used in about half of paragraphs |
| **Concluding** | Closes; emphasizes a cadence (extended cadence OR repetition of cadence-bearing pattern) | About as often omitted as not |

Substitutions are common and labeled (`stating-concluding`, `transisting-introducing`); reprises are unlabeled.

A paragraph is usually 3–9 melody-line phrases.

## 6. Melodic style factors

Three axes: range/guise (minor), background elements (minor + major), MTC (major).

### 6.1 Minor — element ranges (scale / chord / skip)

| Era | Behavior |
|---|---|
| **Minor baroque** | Ranges infrequently exceed the octave |
| **Minor classical** | Significantly often well exceed the octave |
| **Minor romantic** | With chromatic scale element, often well exceeds P4 (up to 2+ octaves) |
| **Minor contemporary** | Rarely exceed P4 |

Plus: classical biases chord element to fundamental triads / V7; contemporary biases chord element to novel chords (polychordal, modern).

### 6.2 Minor — background elements (rbs = rearranged background scale)

- **Minor baroque**: rbs series usually stays in one major/minor scale, or transitions once.
- **Minor contemporary**: rbs series across varied traditional scales (free or systematic).

### 6.3 Minor — tendency tones

Tendency tones (non-do/mi/sol in non-locrian scales; non-do/mi/la in locrian) presume stepwise resolution. Strong tendency tones (half-step resolution): `fa`, `ti` (major); unraised `la` (minor); `re`, `la` (phrygian).

- **Earlier styles**: almost always respect resolution (immediate, delayed, or one of two least-violating non-resolutions). Stepwise respecting outside a familiar scale element usually happens inside a small-ranged rbs element.
- **Contemporary**: in any heavy-major-scale usage, significantly often *nonrespects* the resolution. Nonrespects still tend to occur inside a small rbs element.

Background chord elements have parallel minor factors.

### 6.4 Major — phrase shape × tendency tones

- **Major romantic**: significantly often a tendency tone appears prolonged in a phrase shape and resolves only later — style emphasizes tendency over resolution.
- **Major contemporary**: same, stronger.

### 6.5 Major — prominent scale tones (top of concealed phrase shapes)

| Era | Top tones |
|---|---|
| **Baroque** | do, sol (both scales) |
| **Classical** | do, sol (both scales) |
| **Romantic** | mi, sol (both scales) |
| **Contemporary** | do + 1–2 non-do/non-sol tones, optionally sol (often church-mode) |

### 6.6 Major — overall background scale element

| Era | Span |
|---|---|
| **Baroque / Classical** | do→sol or do→do |
| **Romantic** | mi→do or mi→mi (pseudo-phrygian); or sol→mi or sol→sol (pseudo-mixolydian) |
| **Contemporary** | Either do→sol/do (the few phrases that use overall bg scale), or deliberate scalar confusion (most phrases) |

### 6.7 Composer trademarks

| Composer | Trademark |
|---|---|
| **Hindemith** | Predominant melodic skip = P4 |
| **Brahms** | Predominantly M3 / m3 (melodic and background) |
| **Bartók** | Mixture of P4 and tritone |
| **Tchaikovsky** | Themes are lightly elaborated background scale elements |

## 7. Melodic tonal center (MTC)

Two approaches; otherwise MTC is avoided.

### 7.1 Central-tone formation

First and last notes of the phrase shape are the **same pitch** (or differ by octave). The single pitch is the MTC.

Style biases:
- **Baroque / Classical** → MTC most often **do** (both scales).
- **Romantic** → MTC most often **mi** or **sol**.
- **Contemporary** → MTC = **do** (one scale) or just a pitch (no scale / mixed scales).

### 7.2 Two-indicative-tones formation

First and last differ. Chart:

| First/last constituency | MTC determination | Reason |
|---|---|---|
| Two diatonic / chromatic / mixed tones any skip apart **except** tritone, M7, m7, 9 | Whichever is **last** | Goal of motion inside a stable triad |
| Two **diatonic** tones a step apart | Whichever is **in the tonic chord** (else `ti`) | Tone of rest in a tendency/resolution situation |
| Two **mixed** tones a step apart (not: two chromatic a whole step apart; not: tritone) | The **diatonic** tone | Same as above |

Style bias: earlier styles favor the skip-apart row; contemporary favors the step-apart row.

### 7.3 Avoidance

Phrases that fall outside the chart (e.g. tritone / M7 / m7 / 9 ends) achieve MTC avoidance. Cadence and MTC are independent — a non-tonal-centered phrase can still have a cadence.

## 8. Interest features (Layer 4 of the system)

**Revised-form features:** Variety, Irregularity, Subtlety.
**Basic features:** DelayedCompletion, SurpriseCompletion.
**Two laws:** Compensation (every tension answered), Subordination (interest never upstages primary material).
**Quality grades:** Inferior, Imperfect, Standard, Strong, Superior.

When scoring interest, weight by the active style profile — Romantic tolerates more prolonged tension; Baroque expects swift compensation.

## 9. Short-range design (Layer 6)

**Core** — a small recurring kernel (pitch pattern, rhythm pattern, or combined) that the phrase is built around. The core appears multiple times in a phrase as **reuses** (literal, transposed, varied).

**Four subsurface designs:**

| Design | Group | Idea |
|---|---|---|
| **Flexible** | Part-to-whole | Core appears with significant variation between uses |
| **Attachment** | Part-to-whole | Core uses joined by attached connective material |
| **Shape-elaborated** | Whole-to-part | Shape is built first, then filled with elements |
| **Background-elaborated** | Whole-to-part | Background elements / rbs are built first, then surface |

## 10. Long-range design (Layer 7)

**Master subsurface design** — one core acts as DNA across the whole paragraph; cross-phrase master-core reuses anchor structural points.
**Submaster subsurface design** — a secondary cross-phrase core, optional.
**Overall inter-component aspects** — how phrases relate by shape, role, key/MTC, tempo, register.

## 11. Five-pass revision (Layer 8)

| Pass | Looks at | Fixes |
|---|---|---|
| **Warmup** | Gross errors | Out-of-range pitches, broken chain-of-elements, illegal A2 placements, false chord elements |
| **Design** | Shape integrity | Wrong curvature count, illegal extraneants, missing linking notes, accent misalignment for concealed shapes |
| **Interest** | Interest features + laws | Score variety / irregularity / subtlety; enforce compensation and subordination |
| **Hidden-core** | Core reuses | Verify master core reuses are present at the right structural points; add or strengthen |
| **Final-touch** | Surface polish | Contour smoothing, articulation, last-note resolution, tempo/dynamics suggestions |
