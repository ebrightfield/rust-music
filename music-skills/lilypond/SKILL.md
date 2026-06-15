---
name: lilypond
description: LilyPond music notation engraving - syntax reference, sheet music transcription from images, compilation debugging, and QA verification of pitches, rhythms, and registers against source scores. Also covers jazz idioms (swung MIDI via \articulate, chord-symbol tracks, treble_8 guitar clef, bends and fall-offs) and MIDI playback via fluidsynth + SoundFont.
---

# LilyPond Music Notation

LilyPond is a text-based music engraving system that produces publication-quality scores. This skill covers writing, debugging, and verifying LilyPond code.

## Quick Reference

### Installation & Compilation

```bash
# macOS
brew install lilypond

# Arch Linux
pacman -S lilypond

# Compile to PDF
lilypond score.ly

# Compile with PNG output
lilypond --png score.ly

# Specify output directory
lilypond -o output/ score.ly
```

### Minimal File Template

```lilypond
\version "2.24.4"

\header {
  title = "Title"
  composer = "Composer"
  tagline = ##f
}

\score {
  \relative c' {
    \clef treble
    \key c \major
    \time 4/4
    c4 d e f | g2 g |
  }
  \layout { }
  \midi { }
}
```

**Always include `\version`** — syntax changes between versions.

---

## Notes and Pitches

### Note Names (Dutch Convention)

| Note | Name | Sharp | Flat |
|------|------|-------|------|
| C | `c` | `cis` | `ces` |
| D | `d` | `dis` | `des` |
| E | `e` | `eis` | `ees` (or `es`) |
| F | `f` | `fis` | `fes` |
| G | `g` | `gis` | `ges` |
| A | `a` | `ais` | `aes` (or `as`) |
| B | `b` | `bis` | `bes` |

Double sharp: `isis` (e.g., `cisis`). Double flat: `eses` (e.g., `ceses`).

### Octave Marks

| Mark | Meaning | Example |
|------|---------|---------|
| (none) | Small octave (C3 region) | `c` |
| `'` | Up one octave | `c'` = middle C (C4) |
| `''` | Up two octaves | `c''` = C5 |
| `,` | Down one octave | `c,` = C2 |
| `,,` | Down two octaves | `c,,` = C1 |

### Relative vs Absolute Mode

**Relative mode** (recommended for hand-written scores): each note is placed nearest to the previous note. Only add `'` or `,` when the interval exceeds a fourth.

```lilypond
\relative c' {
  c d e f | g a b c |   % stepwise — no marks needed
  c g c, g              % large jumps need marks
}
```

**Absolute mode**: every note needs full octave specification.

```lilypond
{ c'4 d' e' f' | g' a' b' c'' | }
```

**Critical gotcha**: In relative mode, one wrong octave mark shifts ALL subsequent notes. Use bar checks (`|`) to catch errors early.

### Durations

| Duration | Code | Duration | Code |
|----------|------|----------|------|
| Whole | `1` | Eighth | `8` |
| Half | `2` | Sixteenth | `16` |
| Quarter | `4` | Thirty-second | `32` |

- Dotted: append `.` → `c4.` (dotted quarter)
- Double-dotted: `c4..`
- **Duration carries forward**: if omitted, the previous duration is reused

### Rests

| Type | Syntax | Use |
|------|--------|-----|
| Regular rest | `r4` | Visible rest with duration |
| Whole-measure rest | `R1` | Full-measure rest |
| Multi-measure rest | `R1*4` | 4 measures of rest |
| Spacer rest | `s4` | Invisible (for padding voices) |

### Chords

```lilypond
<c e g>4        % C major chord, quarter note
<c f a>2        % F major chord, half note
```

Duration goes AFTER the closing bracket. **Not** `<c4 e4 g4>`.

In relative mode, the first note of a chord is relative to the previous note; subsequent chord tones are relative to the first note of that chord.

---

## Key, Time, Clef, Tempo

```lilypond
\key g \major           % or \minor
\time 4/4               % any fraction: 3/4, 6/8, 5/4, 7/8
\clef treble            % bass, alto, tenor, "treble_8"
\tempo "Allegro" 4 = 120
\partial 4              % pickup measure (anacrusis)
```

---

## Articulations and Dynamics

### Articulation Shorthand

Attach after the note with a direction prefix: `-` (auto), `^` (above), `_` (below).

| Shorthand | Meaning |
|-----------|---------|
| `-.` | Staccato |
| `->` | Accent |
| `-^` | Marcato |
| `--` | Tenuto |
| `-!` | Staccatissimo |
| `-_` | Portato |

```lilypond
c'4-. d-> e-^ f--
c'4^.          % staccato forced above
```

### Ornaments

```lilypond
c'4\trill  c\mordent  c\turn  c\prall
c'1\fermata
```

### Dynamics

```lilypond
c'4\pp  d\p  e\mp  f\mf
c'4\f   d\ff  e\fff
c'4\<  d e f | g1\f       % crescendo hairpin
c'4\>  d e f | g1\p       % decrescendo hairpin
c'4\<  d e f\!            % end hairpin explicitly
```

---

## Ties, Slurs, Tuplets, Grace Notes

### Ties (same pitch, extend duration)

```lilypond
g'4~ g c2~ | c4 r r2
```

### Slurs (different pitches, phrasing)

```lilypond
d''4( c16) cis( d e c cis d) e( d4)
```

### Phrasing Slurs (longer arcs, can nest with regular slurs)

```lilypond
g'4\( g8( a) b( c) b4\)
```

### Tuplets

```lilypond
\tuplet 3/2 { c'8 d e }         % triplet
\tuplet 5/4 { c16 d e f g }     % quintuplet
```

The fraction means "actual / normal" — 3/2 = 3 notes in the time of 2.

### Grace Notes

```lilypond
\grace c'16 d4             % plain grace note
\acciaccatura d'8 c4       % slashed grace with slur
\appoggiatura e'8 d4       % unslashed grace
```

---

## Staff Structure

### Single Staff

```lilypond
\new Staff { c'4 d e f }
```

### Multiple Staves

```lilypond
<<
  \new Staff { \clef treble c''4 d e f }
  \new Staff { \clef bass c4 d e f }
>>
```

### Staff Groups

| Type | Use |
|------|-----|
| `\new GrandStaff << >>` | Piano brace |
| `\new PianoStaff << >>` | Piano with brace |
| `\new ChoirStaff << >>` | Choir bracket |
| `\new StaffGroup << >>` | Orchestra bracket |

### Polyphonic Voices

```lilypond
\relative {
  << { g'4 fis8( g) a4 g } \\ { d4 d d d } >>
}
```

Voice 1 (before `\\`): stems up. Voice 2 (after `\\`): stems down.

Explicit voice commands: `\voiceOne`, `\voiceTwo`, `\voiceThree`, `\voiceFour`, `\oneVoice`.

**Key rule**: Slurs and ties cannot cross voice boundaries.

---

## Repeats and Bar Lines

### Volta Repeats

```lilypond
\repeat volta 2 {
  c'4 d e f
  \alternative {
    \volta 1 { g2 g }
    \volta 2 { a2 a }
  }
}
```

### Unfolded Repeats (written out, useful for MIDI)

```lilypond
\repeat unfold 4 { c'8 d e f }
```

### Tremolo

```lilypond
\repeat tremolo 8 { c'16 d' }    % alternating tremolo
c'2:32                             % single-note tremolo
```

### Bar Lines

```lilypond
\bar "|"        % standard
\bar "||"       % double
\bar "|."       % final
\bar ".|:"      % start repeat
\bar ":|."      % end repeat
\bar ":|.|:"    % end-start repeat
```

### Bar Checks

Place `|` at measure boundaries to catch duration errors:

```lilypond
c'4 d e f | g a b c |
```

---

## Lyrics

```lilypond
<<
  \new Voice = "melody" {
    \autoBeamOff
    c''4 c g' g | a a g2
  }
  \new Lyrics \lyricsto "melody" {
    Twin -- kle twin -- kle lit -- tle star
  }
>>
```

- Syllable hyphens: `--` (two dashes)
- Extender lines (melisma): `__` (two underscores)
- Skip a note: `_` (single underscore)
- **Use `\autoBeamOff`** in vocal music so beaming matches syllables

---

## Variables and Structure

Separate content from structure for maintainability:

```lilypond
melody = \relative c'' {
  \clef treble \key g \major \time 4/4
  g4 a b c | d2 g, |
}

bass = \relative c {
  \clef bass \key g \major \time 4/4
  g2 d | g1 |
}

\score {
  <<
    \new Staff \melody
    \new Staff \bass
  >>
  \layout { }
  \midi { }
}
```

---

## Overrides and Tweaks

```lilypond
\override NoteHead.color = "red"       % all subsequent
\once \override NoteHead.font-size = -3 % next only
\revert NoteHead.color                  % undo

\set Staff.fontSize = -2               % context property

% Single-element tweak (e.g., one note in a chord):
<c \tweak color "red" e g>4
```

---

## Markup

```lilypond
c'4^\markup { \italic "solo" }        % text above
c'4_\markup { \bold "pizz." }         % text below
\markup { \bold \italic "molto espr." }
```

---

## Extended Documentation

For specialized workflows, see:

- **[references/sheet-music-transcription.md](./references/sheet-music-transcription.md)** — Transcribing sheet music images to LilyPond, QA verification of pitches/rhythms/registers, and systematic error checking
- **[references/advanced-notation.md](./references/advanced-notation.md)** — Chord mode, figured bass, percussion, transposition, ossia staves, spacing/layout control, Scheme customization, and **jazz idioms** (swing notation tradeoffs, the two-`\score` pattern for clean charts + swung MIDI via `\articulate`, `\bendAfter` fall-offs, grace-note bends, `treble_8` octave-shift gotcha, multi-staff jazz scores, MIDI playback via fluidsynth + SoundFont)

---

## Common Errors and Fixes

| Error / Symptom | Cause | Fix |
|----------------|-------|-----|
| Music runs off the page | Duration error extends past barline | Add bar checks (`\|`), find the bad duration |
| Extra staff appears | `\override` before `\new Staff` | Move override inside `\new Staff { }` |
| Unexpected `\new` | Multiple staves without `<< >>` | Wrap in `<< \new Staff { } \new Staff { } >>` |
| Unbound variable % | `%` comment in Scheme code | Use `;` for Scheme comments |
| Voice needs \voiceXx | Two voices with same stem direction | Use explicit `\voiceOne`, `\voiceTwo` |
| Bar check warnings | Duration doesn't fill the measure | Count beats; fix the offending measure |

### Debugging Strategy

1. Always compile frequently — don't write 50 measures then compile
2. Use `|` bar checks at every measure boundary
3. Comment out sections with `%` (line) or `%{ %}` (block) to isolate errors
4. Error line numbers may be off by 1-2 lines — check nearby
5. Use `\barNumberCheck #N` at rehearsal marks to verify alignment

---

## Best Practices

1. **Always include `\version`** at the top
2. **One measure per line** for readability
3. **Use bar checks** (`|`) at every barline
4. **Use variables** to separate parts from score structure
5. **Indent with 2 spaces** (no tabs)
6. **Use `\include`** to split large scores across files
7. **Set `tagline = ##f`** in `\header` to remove the default LilyPond footer
8. **Don't name variables** with reserved words (`key`, `time`, `score`, `staff`)

---

## Resources

- [LilyPond Notation Reference](https://lilypond.org/doc/v2.24/Documentation/notation/)
- [LilyPond Learning Manual](https://lilypond.org/doc/v2.24/Documentation/learning/)
- [LilyPond Snippet Repository](https://lsr.di.unimi.it/)
- [Mutopia Project (free scores)](https://www.mutopiaproject.org/)
