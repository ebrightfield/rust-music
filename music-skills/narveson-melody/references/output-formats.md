# Output formats

How to render a `Phrase` or `MelodicParagraph` for this workspace. Pick the format the user asked for (or default to LilyPond).

---

## LilyPond (default)

The workspace's primary visual target. Lean on the `lilypond` skill for syntax detail; the templates below cover the common cases.

### Minimal phrase

```lilypond
\version "2.24.0"
\header {
  title    = "Narveson study — stating phrase"
  composer = "Generated"
}

\score {
  \new Staff {
    \clef treble
    \key c \major
    \time 4/4
    \tempo 4 = 92

    c'4 d' e' f' | g'2 e' | f'4 d' e' c' | g'1
  }
  \layout {}
  \midi { \tempo 4 = 92 }
}
```

### Multi-phrase paragraph with structural barlines

```lilypond
\score {
  \new Staff {
    \clef treble
    \key g \minor
    \time 6/8
    \tempo "Andante con moto" 4. = 60

    % stating phrase
    d'8. ees'16 f'8 g'4 f'8   | bes'4. a'8 g' f' \bar "||"
    % extending phrase
    ees'8 d' c' bes4 c'8       | d'4. f'                 \bar "||"
    % concluding phrase
    g'8 a' bes' c''4 bes'8     | a'2.                    \bar "|."
  }
}
```

### Annotated phrase (analysis overlay)

For analytical PDFs — uses LilyPond's text-markup to label elements and shapes inline.

```lilypond
\new Staff {
  \clef treble \key c \major \time 4/4
  c'4^\markup { \bold "sc." }
  d' e' f'^\markup { "linking" }
  g'2^\markup { \italic "core reuse 1" } e'
  f'4 d' e' c'
  g'1^\markup { \bold "MTC: do" }
}
```

To compile (if user wants the PDF):

```bash
cd /tmp
lilypond out.ly         # produces out.pdf + out.midi
```

The `lilypond` skill has the full guide if you need polyphony, voices, or unusual notation. In particular, see its "Jazz Idioms" section in `references/advanced-notation.md` for swing notation, `\bendAfter`, `treble_8` clef gotchas, and chord-symbol tracks.

### Jazz-chart template (clean PDF + swung MIDI)

For Scofield-, Coltrane-, or any swing-feel output, use the **two-`\score` pattern**: write straight 8ths so the chart reads cleanly, and let `\articulate` swing the MIDI. This avoids the triplet-bracket clutter that makes jazz charts unreadable.

```lilypond
\version "2.24.0"
\include "articulate.ly"

harmony = \chordmode { \repeat unfold 8 { c1:7 | } }

melody = {
  % straight 8ths — articulate will swing them
  r4 e8 ees8 d8 e8 g8 e8 |
  c8 d8 e8 g8 bes8 c'8 bes8 g8 |
  % ...
}

scoreBody = <<
  \new ChordNames \harmony
  \new Staff \with {
    instrumentName = "Guitar"
    midiInstrument = "electric guitar (jazz)"
  } {
    \clef "treble_8"
    \key c \major
    \time 4/4
    \tempo "Medium swing" 4 = 92
    \melody
  }
>>

% PDF: straight 8ths as written
\score { \scoreBody  \layout {} }

% MIDI: swung by articulate
\score { \unfoldRepeats \articulate \scoreBody  \midi { \tempo 4 = 92 } }
```

When to use vs. skip the pattern:

| Output style | Use jazz template? |
|---|---|
| Baroque, Classical, Romantic, Contemporary classical | No — single `\score` block with `\layout` + `\midi` is enough |
| Swing-feel jazz (bebop, hard bop, swing) | **Yes** — `\articulate` makes the playback work |
| Latin, funk, rock | No — those are straight-8th feels; no swing needed |
| Ballad with swing 8ths (slow swing) | Yes — same pattern; `\articulate` still swings |

A bar that contains *real* 8th-note triplets (a triplet flurry, a polyrhythm) stays as `\tuplet 3/2 { x8 y8 z8 }` regardless — those aren't swing notation, they're actual triplets, and players read them as such.

### Guitar clef

For guitar parts, use `\clef "treble_8"` (sounds an octave below written) and keep pitches at concert pitch. **Do not** shift pitches up an octave to "compensate" — the clef does that for you. Double-shifting puts the music an octave too high.

---

## MIDI via `music-midi`

The workspace has a working MIDI path. Two ways to deliver:

### Path A: LilyPond MIDI (easiest)

The `\midi {}` block in the LilyPond template above produces a `.midi` file as a side effect of `lilypond out.ly`. Use this when the user just wants to hear it.

For **swung playback**, the source must use either the jazz-chart template above (recommended) or notate every 8th-note pair as `\tuplet 3/2 { x4 y8 }`. Verbal "Medium swing" tempo markings are not honored by LilyPond's MIDI export.

### Path B: Native via `music-midi`

For programmatic use. Build a small Rust example file:

```rust
// /tmp/narveson_play.rs — paste into music-midi/examples/
use music::prelude::*;
use music::melody::*;
use music::notation::rhythm::duration::Duration;
use music_midi::convert::melody::melody_to_midi;
use music_midi::playback::play_smf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let events = vec![
        MelodicEvent::new(pitch!(C, 4), Duration::QTR),
        MelodicEvent::new(pitch!(D, 4), Duration::QTR),
        MelodicEvent::new(pitch!(E, 4), Duration::QTR),
        MelodicEvent::new(pitch!(F, 4), Duration::QTR),
        MelodicEvent::new(pitch!(G, 4), Duration::HALF),
        // ...
    ];

    let smf = melody_to_midi(&events, 92 /* tempo bpm */)?;
    play_smf(&smf)?;
    Ok(())
}
```

Run with:

```bash
cargo run --example narveson_play -p music-midi
```

(The exact `melody_to_midi` / `play_smf` function names may differ slightly — check `music-midi/src/convert/melody.rs` and `music-midi/src/playback.rs` before pasting. Adjust to whatever's there.)

### Playing the MIDI

LilyPond emits a `.midi` file but no audio. To hear it:

```bash
# Live playback (PipeWire backend; -g raises gain from default 0.2 to 2.0)
fluidsynth -i -g 2.0 -a pipewire ~/soundfonts/MuseScore_General.sf3 phrase.midi

# Render to WAV (boostable in any player)
fluidsynth -F phrase.wav -g 3.0 ~/soundfonts/MuseScore_General.sf3 phrase.midi
mpv --volume=130 phrase.wav   # mpv accepts volumes > 100%
```

The `lilypond` skill's "Jazz Idioms" section covers fluidsynth gotchas (silent playback = no soundfont mapped to GM patch; too quiet = default gain is 0.2 on 0–10 scale; PipeWire per-stream volume sliders).

### When to recommend post-processing through `music-midi`

LilyPond's `\articulate` swing is good but fixed at ≈2:1. Recommend the `music-midi` path when the user wants:

- A **specific swing ratio** (1.5:1 light swing for medium-up tempos; 2.5:1 heavy for slow blues).
- **Real pitch-bend events** on bend notation (LilyPond's `\bendAfter` is visual-only).
- **Humanization** (timing/velocity jitter) — Scofield doesn't play on the grid even when not swinging.
- **Drum machine / accompaniment generation** layered on the melody.

These all live in the future `music-midi` post-processing pipeline. For now, LilyPond + `\articulate` covers ~80% of cases.

---

## Rust literals from `music` crate primitives

For users prototyping in code. Default style:

```rust
use music::prelude::*;
use music::melody::*;
use music::notation::rhythm::duration::Duration;

fn stating_phrase() -> Vec<MelodicEvent> {
    vec![
        MelodicEvent::new(pitch!(C, 4), Duration::QTR),
        MelodicEvent::new(pitch!(D, 4), Duration::QTR),
        MelodicEvent::new(pitch!(E, 4), Duration::QTR),
        MelodicEvent::new(pitch!(F, 4), Duration::QTR),
        MelodicEvent::new(pitch!(G, 4), Duration::HALF),
        MelodicEvent::new(pitch!(E, 4), Duration::HALF),
        MelodicEvent::new(pitch!(F, 4), Duration::QTR),
        MelodicEvent::new(pitch!(D, 4), Duration::QTR),
        MelodicEvent::new(pitch!(E, 4), Duration::QTR),
        MelodicEvent::new(pitch!(C, 4), Duration::QTR),
        MelodicEvent::new(pitch!(G, 4), Duration::new(DurationKind::Whole, 0)),
    ]
}
```

When the user wants harmonic context attached:

```rust
fn paragraph_harmony() -> ChordProgression {
    ChordProgression::new(vec![
        TimedChord::new(
            NoteSet::new(vec![Note::C, Note::E, Note::G]),
            Duration::HALF,
        ),
        TimedChord::new(
            NoteSet::new(vec![Note::G, Note::B, Note::D]),
            Duration::HALF,
        ),
    ])
}
```

Check `music/src/prelude.rs` and `music/src/melody/mod.rs` for exact type names; the workspace re-exports `pitch!`, `voicing!`, `pc!`, `pc_shape!`, `content!` macros from the crate root.

---

## Combined output

Many user requests want both LilyPond (for inspection) and MIDI (for hearing). Default behavior:

1. Print the LilyPond block inline in the response.
2. Mention the user can compile it with `lilypond out.ly` to get both PDF and MIDI.
3. Only emit Rust literals when the user explicitly asks for code.

---

## Where artifacts go

If the user asks you to write files:

- LilyPond drafts → `/tmp/narveson_<descriptor>.ly` by default; user can move them.
- LilyPond examples for the repo → `music/examples/<name>.rs` (only if the user explicitly wants a committed example).
- Analytical PDFs → `docs/narveson-experiments/<descriptor>.pdf` (create the folder if it doesn't exist; ask first).

Don't write files unless asked. Default is inline output.
