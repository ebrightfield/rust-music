# Decision-point checklists

When the user wants to understand *why* a particular note, shape, or extraneant was chosen, walk through the relevant checklist below. These are also useful as self-prompts when you're stuck mid-compose — answer the questions in order and the next note falls out.

The checklists mirror the `DecisionOracle` methods in `docs/narveson-melody-system.md` §15.4.

---

## Shape choice

For the next phrase to write, in order:

1. **Role.** What's this phrase's job in the paragraph (introducing / stating / extending / transisting / concluding)?
2. **Style preference.** Which shape does the active style favor for that role? (See `pipeline.md` §5a.)
3. **Length budget.** A 2-bar phrase usually wants Simple; 4 bars Simple-or-Compound; 6–8 bars Compound or Concealed.
4. **Continuity.** If this is an extending phrase, what shape did the thesis use? Default to matching.
5. **Variety budget.** Has every phrase in this paragraph used the same shape? Force a different one for the next.

Output: one of `Simple` / `Compound` / `Skeletal` / `CrossWave`, with a modifier (`modified-` if extraneants are planned).

---

## Element choice (gap between two pitches)

For the gap between pitch A and pitch B, with current chord harmony C:

1. **Distance.** What's the interval A→B?
   - = unison: pedal element (if rhythmically isolated) or repeated note (if consecutive).
   - = m2 / M2: appoggiatura, OR scale element (if extending a stepwise run).
   - = m3 / M3 / P4 / P5 / P8: skip element, OR chord element (if both A and B are in C).
   - ≥ m6: skip element. If style is Contemporary, possibly chord element with intermediate chord tones.
2. **Direction continuity.** Is the prior element moving in the same direction? Extend it as scale or chord element. Different direction? End the prior, start a new one.
3. **Style range policy.** Does the chosen element's range exceed the style's threshold (Baroque ≤ octave; Contemporary ≤ P4)? Compress or split.
4. **Repetition check.** Could a neighbor element fit here instead (i.e. should we ornament rather than connect)? Useful in stating phrases for variety.

Output: one of the six element kinds, plus its form (direct/background) and pitch list.

---

## Pitch choice (within a chord/scale)

For the next pitch given current chord C, current element E, and direction D:

1. **Candidate set.** Start with chord tones of C. Add scale tones from active mode. Add tendency tones (chromatic neighbors of chord tones) if style allows.
2. **Pitch-set policy.** Intersect with the active `PitchSetConstraint`:
   - Strict: drop any candidate outside the allowed `PcSet`.
   - Soft: keep all, but penalize out-of-set candidates by `penalty_per_note`.
   - SoftWithResolution: out-of-set candidates allowed only if they can resolve to in-set within `resolution_window`.
3. **Bound check.** Drop candidates outside the user's `PitchBounds`.
4. **Style biasing.** Boost candidates matching the style's prominent scale tones (Baroque: do/sol; Romantic: mi/sol).
5. **Composer quirk.** If active, apply: Hindemith boosts P4 jumps; Brahms boosts thirds; Bartók boosts P4 and tritone.
6. **Element membership.** The element kind constrains which candidates remain: scale element needs stepwise; chord element needs chord tones; etc.
7. **Tie-break.** When multiple candidates remain, pick the one closest to the contour the shape is calling for. If still tied, pick alphabetically (deterministic mode) or randomly (default).

Output: one `Pitch`.

---

## Rhythm choice (next slot)

For the next slot in the active `RhythmTemplate`:

1. **Slot kind.** Fixed (Note/Rest)? Use as-is. Choice? Pick from the choice list. Fill? Pick a duration that fits the remaining budget.
2. **Element pacing.** Scale elements want shorter durations (eighths / sixteenths) so the stepwise motion is audible; chord/skip elements want longer durations so the arrival is heard.
3. **Style.** Baroque favors regular subdivisions; Romantic favors mixed; Contemporary may favor irregular meters.
4. **Syncopation budget.** Has this measure already used its allotted off-beat attacks? If yes, pick an on-beat duration.
5. **Phrase position.** Approach to cadence wants a longer duration; mid-phrase wants flow.

Output: one `Duration`.

---

## Extraneant choice (direct shape only)

After the surface is filled, decide whether to add an extraneant:

1. **Style.** Baroque: skip (rare). Classical: 0–1 (especially cadential interrupting-repetitive). Romantic: 1–2 (dangling returns common). Contemporary: 0–1 (deviating pitch).
2. **Shape capacity.** Modified Simple can take ≥1 interrupting-repetitive + 1 deviating + 1 dangling-return per end. Modified Compound can take exactly 1 interrupting-repetitive + 1 deviating + 1 per end.
3. **Curvature budget.** After adding, will the phrase still fit the shape's ceiling (1 for Simple, 2½ for Compound)?
4. **Cadence emphasis.** A cadential interrupting-repetitive is a strong cliché — use it consciously.

Output: 0–3 extraneants with locations.

---

## Quality grade (final pass)

After the five revision passes, score:

1. Count interest features detected. ≥3 ⇒ at least Standard.
2. Are both laws (compensation + subordination) respected? Required for Strong.
3. Is the core audible and reused at ≥1 structural point? Required for Strong.
4. Is there a unifying gesture across the paragraph (master core reuse)? Required for Superior.

Output: `Inferior` / `Imperfect` / `Standard` / `Strong` / `Superior`. Don't inflate. Standard is honorable.
