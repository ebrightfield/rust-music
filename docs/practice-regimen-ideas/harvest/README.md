# Harvest Library

Externally-sourced melodic/mechanical/compositional ideas, converted into notatable practice
material. This is the **non-systematic** input to the daily generator.

## Why this exists

Everything else in the regimen is *systematically derived* — pitch-set combinatorics, scale-pattern
tuples, strum enumerations. Systematic material has a characteristic flavor, and there are whole
categories it will never produce: a Towner open-string voicing, a Lage rhythmic displacement, a
McLaughlin open-string connector. Those come from listening to and reading about players.

The harvest library supplies the day's **"idea of the day"**, which then interlocks with the
systematic elements per §1 Rule 0d and §4 (elements of the day) of `../exercise-suite.md`.

## What counts as a harvest

Anything that can become notated material or a terse cue:

| Kind | Example |
|---|---|
| **Mechanical device** | McLaughlin: chromatic tetrachords with an open string as connector |
| **Voicing concept** | Towner: open strings woven into fretted voicings; avoid low close intervals |
| **Melodic procedure** | chromatic approach-and-resolve to a target tone |
| **Compositional technique** | top-note-constant harmonization (Lage, via Bach) |
| **Rhythmic device** | rhythmic displacement of a fixed motif |
| **Stylistic trait** | pseudo-polyphony: sustain one voice while others move |
| **Mental cue** | Lage: practice extremely quietly; use a "reset chord" |
| **Transcription fragment** | 2–4 bars of an actual line, cited |

## Screening rule (learned the hard way)

Harvest quality varies enormously by artist and source. A search for **John McLaughlin** returned
hard, notatable procedures (tetrachord groupings, semitone-descent fingerings, open-string
connectors, diminished/harmonic-minor fingerings). The same search shape for **Pedro Martins**
returned only biography and album reviews — "intricate harmonies, rhythmic complexity" — with no
technique detail at all.

**So: a harvest is only accepted if it survives this test —**

1. Can it be written as notation, a fingering, a voicing, or a one-line cue? If it only yields
   adjectives ("lyrical", "intricate"), **reject it**.
2. Is it specific enough that two different readers would notate it the same way?
3. Does it interlock with at least one systematic element (a set, position, articulation, meter)?
4. Is the source cited, so the claim is checkable and attribution is honest?

Reject → try a different source (instructional books, transcription sites, interview
technique-talk) or a different artist. Do **not** invent plausible-sounding technique and
attribute it to a player. If the material is generic, present it as generic.

## Attribution honesty

- Cite the source URL for every entry.
- Distinguish **"documented device"** (the player demonstrably does this, per a cited source)
  from **"in the spirit of"** (our own material inspired by a documented trait).
- Never present a generated line as a transcription of an actual solo.
- On the sheet, one short attribution is enough: *"open-string connector — McLaughlin device"*.
  Keep Rule 0b's prose budget; the notation still carries the assignment.

## Artists of interest

Tim Reynolds · John McLaughlin · Julian Lage · John Scofield · Ralph Towner · Robin Katz ·
Pedro Martins

Expand freely — but screen every entry through the test above.

## Files

- `library.yaml` — the accepted-harvest store, one record per idea, with source + status.
- `../examples/harv1.ly` — worked example: the McLaughlin tetrachord/open-string device, notated.
