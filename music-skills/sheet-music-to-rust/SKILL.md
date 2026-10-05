---
name: sheet-music-to-rust
description: Transcribe sheet music images (PDF/PNG/JPG) into LilyPond source and native Rust types from the `music` crate in this repo. Use when the user shares an image or PDF of a score and asks to reproduce it as `.ly`, as `RhythmicNotatedEvent`/`Voicing`/`Pitch` literals, or as a `LilypondBuilder` document. Handles ImageMagick preprocessing for images too large or dense for direct vision reading, measure-by-measure transcription, and round-trip QA against the source.
argument-hint: [image-path-or-url] [output-target?]
---

# Sheet Music → LilyPond / Rust

Transcribe an image of sheet music into LilyPond and/or this repo's native Rust music types. The skill assumes the user has supplied (or will supply) a score image and wants a faithful reproduction.

## Composition

This skill **composes** the existing `lilypond` skill rather than replacing it:

- For LilyPond syntax (note names, durations, articulations, key signatures, polyphonic voices, repeats), defer to the `lilypond` skill's main reference and `references/sheet-music-transcription.md`.
- This skill adds: **image ingestion**, **ImageMagick preprocessing**, **mapping to native `music`-crate types**, and a **round-trip QA loop** specific to this repo.

If the user only wants `.ly` output and the image is already legible, much of this skill collapses into the `lilypond` workflow plus a final compile step.

## When to apply

- User shares a sheet-music image (PNG/JPG) or PDF and asks for LilyPond or Rust code.
- User says "transcribe this score" / "reproduce this in code" / "turn this into `RhythmicNotatedEvent`s".
- User wants to verify existing LilyPond/Rust against a source image (QA mode).

Do **not** invoke for: generating fresh music from a description (no source image), pure LilyPond syntax questions (use the `lilypond` skill), or audio-to-score (out of scope).

## Output targets

The user may ask for either or both. If unclear, ask once which they want before transcribing.

1. **LilyPond `.ly` source** — A complete, compilable file. Must round-trip through `lilypond-parser` (the repo's parser supports a useful subset; see `lilypond-parser/src/lib.rs`).
2. **Native Rust** — `RhythmicNotatedEvent` / `Voicing` / `Pitch` literals built with the `pitch!`, `voicing!`, `pc!` macros, assembled into a `LilypondBuilder` document. See `references/rust-native-output.md`.

The native-Rust path produces a runnable example modeled on `music/examples/generate_lilypond.rs`. Place new examples under `music/examples/` only if the user explicitly asks for a file; otherwise return code inline.

## Workflow

### 1. Acquire the image

If the user gave a path, use it as-is. If they gave a URL or PDF, download/convert first:

```bash
# PDF → per-page PNG (uses ImageMagick; needs Ghostscript installed)
magick -density 300 score.pdf score-page-%02d.png

# URL → local file
curl -L -o /tmp/score.png "<url>"
```

### 2. Preprocess with ImageMagick (only if needed)

Claude Code can read images directly via the Read tool, but very large, very dense, or low-contrast scans degrade transcription accuracy. **First, try reading the image as-is.** If you can clearly identify clef, key signature, time signature, and individual noteheads, skip preprocessing.

If the image is too large/dense to read reliably, preprocess. See `references/image-preprocessing.md` for the full decision table. Quick defaults:

```bash
# Standard cleanup: cap longest side, normalize contrast, sharpen lightly
magick input.png -resize 'x2000>' -normalize -unsharp 0x1 /tmp/score-clean.png

# For multi-page or wide images, slice into systems (horizontal strips)
magick input.png -crop x400 +repage /tmp/system-%02d.png
```

After preprocessing, **read the cleaned file** with the Read tool. If it's still illegible, escalate (ask the user for a higher-resolution source rather than guessing).

### 3. Analyze the score

Before writing any code, read the (possibly preprocessed) image and report:

- Clef(s), key signature, time signature, tempo marking
- Staff layout (single, grand staff, multi-staff)
- Measure count
- Polyphonic voices on any staff
- Pickup / anacrusis
- Repeats, voltas, D.C./D.S./Coda
- Unusual notation (tuplets, grace notes, cross-staff beaming, ottava)

Reuse the analysis checklist from the `lilypond` skill's `references/sheet-music-transcription.md` Phase 1.

### 4. Transcribe

Work measure-by-measure with bar checks (`|`) at every barline. See the `lilypond` skill for syntax. For **multi-page** or **multi-system** scores, finish one system fully (analysis → transcription → bar-check pass) before moving to the next.

For native Rust output, see `references/rust-native-output.md` — it documents the available macros, the `RhythmicNotatedEvent`/`Tuplet`/`Voicing` types, available `Duration` constants, and how to assemble a `LilypondBuilder` document. The Rust path is intentionally narrower than full LilyPond: it covers pitches, voicings, durations (including dots and tuplets), rests, and multi-voice staves. Articulations, dynamics, lyrics, and complex repeats are still best expressed in `.ly` source.

### 5. Verify (round-trip QA)

Two complementary checks:

**A. Parser round-trip** (LilyPond output only) — confirms the `.ly` is well-formed in the subset the repo's parser accepts:

```bash
cd /home/eric/zooanthid/rust-music && \
  cargo test -p lilypond-parser -- --nocapture
# Or, for a one-off file, add it under lilypond-parser/tests/fixtures/
# and let the existing fixture_roundtrip test pick it up.
```

The parser is **absolute-mode only** (no `\relative`). If you wrote `\relative`, either rewrite in absolute form for the round-trip check, or skip this step and rely on `lilypond` compilation only.

**B. LilyPond render-back** — compile the `.ly` to PNG and diff against the source visually:

```bash
lilypond --png -o /tmp/qa /tmp/transcription.ly
# Then read /tmp/qa/transcription.png and compare to the source image
```

Report discrepancies using the QA findings format from the `lilypond` skill's transcription reference (measure / beat / staff / type / current / should-be / reason).

## Anti-patterns

- **Don't guess at illegible pitches.** If a notehead is genuinely unreadable after preprocessing, stop and ask for a clearer source. A confident wrong answer is worse than a flagged uncertainty.
- **Don't use `\relative` in output destined for the parser round-trip.** The parser is absolute-mode only.
- **Don't write new files for `.ly` output by default.** Show the source inline first; only write to disk if the user wants compilation or persistence.
- **Don't add features the source doesn't have** (extra dynamics, articulations, "stylistic improvements"). Reproduce the source faithfully.

## Files in this skill

- `references/image-preprocessing.md` — ImageMagick recipes, when to apply each, troubleshooting illegible scans.
- `references/rust-native-output.md` — Mapping from notation to `music`-crate types, available macros and constants, full worked example.
