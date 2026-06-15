---
name: narveson-melody
description: Compose, analyze, and revise melodies in the style of Paul Narveson's "Theory of Melody" using Claude Code directly (no Rust runtime required). Use when the user wants to generate a phrase, paragraph, or motivic study with a chosen era/style (classical, jazz, modal, atonal, etc.); analyze an existing phrase into Narveson's element/shape/phrase-type vocabulary; revise a draft through Narveson's five-pass intensive revision; fit a quick "style profile" from a small corpus snippet; produce LilyPond / MIDI / Rust artifacts that drop into the rust-music workspace; or compose jazz lines (Scofield-, Coltrane-, bebop-style) with chord-symbol tracks and swung MIDI playback. Triggers on phrases like "write me a Bach-like stating phrase", "make a Romantic melody over ii-V-I", "analyze this melody Narveson-style", "revise this phrase", "what would this look like in Bartok's style", "write me a Scofield-style solo over a C7 vamp".
argument-hint: [intent] (e.g. "stating phrase in C minor, classical, 8 bars" or "analyze: c4 d e f g2")
---

# Narveson melody — Claude-Code-native composer / analyzer / reviser

Treat this skill as a **manual implementation** of the Rust system proposed in `docs/narveson-melody-system.md`. The architecture, vocabulary, and laws live there. This skill replays the same pipeline in plain reasoning so the user can prototype outputs without building the crate yet.

Outputs are designed to drop straight into this workspace's tooling: LilyPond source consumable by `music::notation::lilypond`, MIDI commands consumable by `music-midi`, or Rust literals built from `music`-crate primitives.

## When to apply

Invoke when the user asks to:

- **Compose** a phrase or paragraph with a stated style (era / composer / mood) and optional harmonic / pitch-set / rhythm constraints.
- **Analyze** an existing phrase into Narveson's vocabulary (elements, shape, phrase type, interest features, MTC, quality grade).
- **Revise** a draft via the five-pass intensive revision (warmup → design → interest → hidden-core → final-touch).
- **Sketch a style profile** from a short corpus example or verbal description ("write something in the style of the first 8 bars of …").
- **Switch styles** on existing material ("now in Bartók's style", "now Romantic").
- **Render** any of the above to LilyPond, MIDI command lines, or `music`-crate Rust literals.

Do **not** invoke for: pure LilyPond syntax (use the `lilypond` skill), audio-to-score (out of scope), or general music-theory questions unrelated to melody construction.

## Composition with other skills

This skill **consumes** rather than replaces:

- `lilypond` skill for LilyPond syntax details and compile-to-PDF (`lilypond foo.ly`).
- `sheet-music-to-rust` for ingesting a printed phrase before analysis.
- The `music` crate types (`Pitch`, `NoteSet`, `MelodicEvent`, `ChordProgression`) for Rust output — read `music/src/prelude.rs` and `music/src/melody/` to stay aligned.

## Operating modes (pick one per invocation)

| Mode | Trigger phrases | Output |
|------|-----------------|--------|
| **compose** | "write me…", "generate…", "compose a…" | annotated phrase(s) + chosen output format |
| **analyze** | "analyze this…", "classify…", "what's the shape of…" | `PhraseAnalysis` table |
| **revise** | "revise…", "improve…", "fix the…" | before / after + diff of choices |
| **style-fit** | "in the style of <snippet>", "what's the style of…" | informal `StyleProfile` record |
| **render** | "render that as LilyPond", "give me Rust code for that" | format-specific artifact |

If the user's intent spans multiple modes (the common case: "write a Bach-like phrase, then analyze it"), run them in order and label each section.

## The composition pipeline (compose mode)

This is the canonical workflow. Run it in your head; surface only the artifacts the user asked for, not the intermediate scaffolding (unless they asked for it).

1. **Clarify what's underspecified.** Ask at most one or two questions, and only when a missing field would substantively change the output. Sensible defaults: classical style, C major, 4/4, 8-bar paragraph with stating + concluding phrases, mezzo register (C3–C6 bounds), seed = stable. Don't pepper the user.
2. **Pick a style profile.** Pull from `references/style-profiles.md`. If the user named a composer or era, use that preset; if they described it verbally, sketch a custom profile (range policy, tendency-tone treatment, prominent scale tones, MTC approach, optional composer quirk).
3. **Set harmonic context.** If the user gave a chord progression or Roman numerals, use that. Otherwise default to a tonic-prolongation appropriate to the style. For pitch-set constraints (modal/synthetic scales), record the allowed `PcSet` and the deviation policy (strict / soft / soft-with-resolution).
4. **Plan the paragraph.** Decide phrase count (3–9), ordering (default: stating + concluding; add transisting / introducing only when style or length asks for it), and which phrase carries the **master subsurface design**.
5. **For each phrase, run the per-phrase pipeline.** Steps 5a–5h are spelled out in `references/pipeline.md`. The compressed version:
   - Choose primary phrase shape (simple / compound / skeletal / cross-wave) consistent with the phrase's role.
   - Pick a subsurface design (flexible / attachment / shape-elaborated / background-elaborated).
   - Generate a small **core** (3–5 pitches and/or a rhythm cell) sized to the design.
   - Lay the core onto the shape skeleton; place reuses along the curvature contour.
   - Fill the surface with melodic elements obeying the chain-of-elements rule, the style's element-range policy, tendency-tone treatment, chord membership, and pitch-set policy.
   - Apply chosen extraneants (direct shapes only): dangling return / interrupting repetition / deviating pitch, within Narveson's count rules.
   - Snap to a rhythmic template (see `references/pipeline.md` §rhythm) and emit the note stream.
   - Tag the phrase with its analysis (you'll need it for the revision pass).
6. **Run the five-pass revision.** See `references/pipeline.md` §revision. Each pass mutates the phrase to fix a specific class of problem:
   - Warmup — gross errors (out-of-range, broken chain-of-elements).
   - Design — shape integrity, extraneant rules, curvature counts.
   - Interest — variety / irregularity / subtlety; laws of compensation + subordination.
   - Hidden-core — verify master core reuses are present and audible.
   - Final touch — contour smoothing, articulation, last-note resolution.
7. **Render** to the requested format(s). See `references/output-formats.md`.

## Analyze mode

Given a phrase as ABC, LilyPond, scientific pitch (`C4 D4 E4 …`), or a parsed `Vec<MelodicEvent>`:

1. Parse to a pitch + duration stream. If LilyPond, lean on the user's existing parser conventions (see `lilypond-parser/`).
2. Segment into elements (scale / appoggiatura / chord / skip / neighbor / pedal). The chain-of-elements rule does most of the work: linking notes are where one element ends and the next begins.
3. Compute curvature (count direction changes). Apply the shape decision tree from `references/narveson-digest.md` §02.02 to pick primary shape; check for extraneants if direct-shape candidates fail.
4. Identify secondary shapes (skeletal + cross-wave; only one secondary direct, and only when concealed is primary).
5. Identify MTC via the central-tone or two-indicative-tones chart.
6. Score interest features and assign a quality grade (inferior / imperfect / standard / strong / superior).
7. Emit the analysis as a labeled table — see `references/examples-walkthrough.md` for the format.

If any classification is ambiguous (e.g. multiple competing shapes within 1 curvature unit), say so explicitly and offer the alternatives.

## Revise mode

The five passes act on whichever IR the user gave you (post-analyze):

1. Run analyze mode first (silently if you already have the phrase from a prior turn).
2. Apply each pass in order; show the diff before / after each pass only if the user asked for verbose mode. Default is to show only the before-and-after final pair plus a one-line note per pass.
3. Re-analyze and report the new quality grade.

If the user said "revise" without saying which passes, default to all five. If they named specific passes, run only those.

## Style-fit mode

When the user supplies a corpus snippet ("write something like this" + a phrase or two):

1. Analyze each phrase in the snippet.
2. Aggregate the analyses into informal counters: element-range histogram, guise distribution, tendency-tone treatment frequency, prominent scale tones (top-2 of concealed-shape pitches), MTC approach.
3. Map the counters to the closest preset in `references/style-profiles.md`; if no preset fits, sketch a custom profile by filling in each axis from the data.
4. Note the *confidence* of the fit (low if snippet is < 2 phrases or all phrases were the same shape).
5. Use that profile for any subsequent compose calls in the conversation, unless the user overrides it.

## Output

Default to a brief textual summary + the artifact the user asked for. **Do not over-explain.** A composer rarely wants a paragraph of meta-commentary above their melody.

Default artifact format: LilyPond, since this workspace's primary visual output is LilyPond. Switch to MIDI command lines if the user mentions playback, or to Rust literals if they're prototyping in code.

See `references/output-formats.md` for the exact templates and a few worked snippets.

## Reference index — load on demand

| File | When to read it |
|------|-----------------|
| `references/narveson-digest.md` | Need the precise rules for elements, shapes, extraneants, MTC; any time analyze mode is invoked. |
| `references/style-profiles.md` | Picking or sketching a `StyleProfile`. Has the four era presets + composer quirks (Hindemith P4, Brahms 3rds, Bartók P4+tritone, Tchaikovsky). |
| `references/pipeline.md` | Composing: the detailed per-phrase pipeline, the rhythm template library, the five revision passes. |
| `references/output-formats.md` | Rendering: LilyPond, MIDI, Rust literal recipes; what to import from the `music` crate. |
| `references/examples-walkthrough.md` | Three end-to-end worked examples (compose, analyze, revise) showing exact input/output formats. |
| `references/prompts.md` | The decision-point checklist used when you're playing the role of the `DecisionOracle` (shape / element / pitch / duration / extraneant choices). Mostly useful for transparency when the user asks "why did you choose that?". |

Read references **only when needed**. The digest is large; for compose mode you usually need only `style-profiles.md` and `pipeline.md`.

## Determinism

Outputs will vary turn-to-turn. If the user wants a stable result, ask them for a "seed phrase" (a short slogan like `seed=BachI`); commit to that seed by hashing it into your decisions (e.g., always pick the first viable option for that conversation, breaking ties alphabetically by element name). The user can then say "regenerate with the same seed" and get the same phrase.

## What this skill is not

- It is not the Rust crate. It cannot generate 10,000 phrases per second. It is an interactive thinking aid that produces a few phrases of high-fidelity Narvesonian melody per turn.
- It is not a substitute for `analyze` once the crate exists. Where the future crate would do statistical fitting, this skill does pattern-matching against named presets.
- It is not a general MIDI composer. It is opinionated about *melody* in Narveson's specific sense (modern epoch, monophonic line over harmonic context).
