# Advanced LilyPond Notation

Extended reference for complex notation features. Load this when working with chord symbols, percussion, transposition, figured bass, or layout customization.

## Chord Mode

Chord mode provides shorthand for chord symbols above a staff:

```lilypond
\chordmode {
  c1           % C major
  c:m          % C minor
  c:7          % C dominant 7th
  c:maj7       % C major 7th
  c:m7         % C minor 7th
  c:dim        % C diminished
  c:dim7       % C diminished 7th (fully diminished)
  c:m7.5-      % C half-diminished (m7♭5)
  c:aug        % C augmented
  c:sus4       % C sus4
  c:sus2       % C sus2
  c:9          % C dominant 9th
  c:m9         % C minor 9th
  c:13         % C dominant 13th
  c:6          % C6 (major with added 6th)
  c:m6         % Cm6
  c:7.5+       % C7#5 (augmented dominant)
  c:7.5-       % C7♭5
}
```

### Inversions and Bass Notes

```lilypond
\chordmode {
  c/e          % C/E (first inversion)
  c/g          % C/G (second inversion)
  c/+bes       % C with added bass B♭ (slash chord)
}
```

### Specific Interval Notation

```lilypond
\chordmode {
  c:3.5.6      % C with specific intervals (add6)
  c:7^5        % C7 omit 5
  c:3-.5.7     % Cm7 (minor 3rd, perfect 5th, minor 7th)
}
```

### Using Chord Names in a Score

```lilypond
chords = \chordmode { c2 f | g:7 c | }

\score {
  <<
    \new ChordNames \chords
    \new Staff \melody
  >>
}
```

---

## Percussion Notation

### Unpitched Percussion

```lilypond
\new DrumStaff {
  \drummode {
    hihat4 hihat hihat hihat |
    << { hihat4 hihat hihat hihat } \\ { bassdrum4 snare bassdrum snare } >> |
  }
}
```

### Common Drum Names

| Name | Code | Name | Code |
|------|------|------|------|
| Bass drum | `bassdrum` or `bd` | Snare | `snare` or `sn` |
| Hi-hat | `hihat` or `hh` | Open hi-hat | `openhihat` or `hho` |
| Crash | `crashcymbal` or `cymc` | Ride | `ridecymbal` or `cymr` |
| High tom | `tomhi` or `tomh` | Mid tom | `tommh` |
| Low tom | `tomlo` or `toml` | Pedal hi-hat | `pedalhihat` or `hhp` |
| Rim shot | `snare` with override | Cross stick | `sidestick` or `ss` |

### Custom Drum Style

```lilypond
#(define mydrums '(
  (bassdrum   default #f -1)
  (snare      default #f  1)
  (hihat      cross   #f  3)
  (pedalhihat cross   #f -3)
))
```

---

## Transposition

```lilypond
% Transpose from C to D (up a major 2nd)
\transpose c d { c'4 e' g' c'' }

% Transpose for B♭ instrument (concert pitch → written pitch)
\transpose bes c' { \concertPitchMusic }

% Transpose for E♭ instrument
\transpose ees c' { \concertPitchMusic }
```

**Key principle**: `\transpose from to { music }` shifts every pitch by the interval from `from` to `to`.

### Transposing Instruments

```lilypond
% B♭ clarinet (sounds a major 2nd lower than written)
clarinetMusic = \transpose bes c' {
  \relative c' { c4 d e f | g2 g | }
}
```

---

## Figured Bass

```lilypond
\figures {
  <6>4 <6 4>2 <_+>4 |
  <7>2 <6 4 3>2 |
}
```

| Symbol | Code | Meaning |
|--------|------|---------|
| 6 | `<6>` | First inversion |
| 6/4 | `<6 4>` | Second inversion |
| 7 | `<7>` | Seventh chord |
| ♯ on figure | `<6+>` or `<6\+>` | Raised figure |
| ♭ on figure | `<6->` | Lowered figure |
| ♮ on figure | `<6!>` | Natural figure |

---

## Ossia Staves

```lilypond
\new Staff \with {
  \remove "Time_signature_engraver"
  alignAboveContext = "main"
  fontSize = #-3
  \override StaffSymbol.staff-space = #(magstep -3)
  \override StaffSymbol.thickness = #(magstep -3)
  firstClef = ##f
} {
  s1*3 |  % skip measures before ossia
  c''4 d'' e'' f'' |
}
```

---

## Page and Score Layout

### Paper Block

```lilypond
\paper {
  #(set-paper-size "letter")
  top-margin = 15\mm
  bottom-margin = 15\mm
  left-margin = 20\mm
  right-margin = 20\mm
  indent = 15\mm            % first system indentation
  short-indent = 5\mm       % subsequent systems
  system-system-spacing.basic-distance = #14
  ragged-last-bottom = ##f  % fill last page
}
```

### Layout Block

```lilypond
\layout {
  \context {
    \Score
    \override SpacingSpanner.common-shortest-duration = #(ly:make-moment 1/8)
  }
  \context {
    \Staff
    \override TimeSignature.break-visibility = ##(#f #t #t)
  }
}
```

### Line and Page Breaks

```lilypond
\break         % force line break
\noBreak       % prevent line break
\pageBreak     % force page break
\noPageBreak   % prevent page break
```

### Spacing Between Systems

```lilypond
\paper {
  system-system-spacing = #'((basic-distance . 12)
                              (minimum-distance . 8)
                              (padding . 1)
                              (stretchability . 60))
}
```

---

## Multi-Movement Works

```lilypond
\book {
  \header { title = "Sonata" }

  \score {
    \header { piece = "I. Allegro" }
    \new Staff { c'1 }
    \layout { }
  }

  \score {
    \header { piece = "II. Andante" }
    \new Staff { c'1 }
    \layout { }
  }
}
```

---

## Scheme Customization

LilyPond uses GNU Guile (Scheme). Inline Scheme is delimited by `#`:

```lilypond
% Boolean
##t    % true
##f    % false

% Number
#5
#-3

% String
#"hello"

% List
#'(1 2 3)

% Scheme expression in context
\override NoteHead.color = #(rgb-color 0.8 0.2 0.2)

% Define a Scheme function
#(define (my-func x) (* x 2))

% Moment (duration as a fraction)
#(ly:make-moment 1/4)    % quarter note
```

**Important**: Use `;` for comments inside Scheme blocks, not `%`.

---

## Cue Notes

```lilypond
\cueDuring "otherVoice" #UP {
  R1 | R1 |
}
```

---

## MIDI Output

```lilypond
\score {
  \new Staff { c'4 d e f }
  \layout { }    % produces PDF
  \midi {
    \tempo 4 = 120
  }
}
```

To produce MIDI only (no PDF), omit `\layout { }`.

### Articulate for better MIDI

Verbal markings like `\tempo "Allegro"`, slurs (legato), staccatos, and "swing 8ths" do not affect the default MIDI export — LilyPond emits straight on-the-grid notes. `\articulate` (a stock include) rewrites the MIDI stream to honor these markings:

```lilypond
\include "articulate.ly"

\score {
  \unfoldRepeats \articulate \scoreBody
  \midi { \tempo 4 = 92 }
}
```

What it does:

- **Slurs → legato** (notes overlap slightly).
- **Staccato / staccatissimo → detached** (notes shortened).
- **Equal-length consecutive 8th-notes → swung** (long-short pair, ~2:1 ratio). This is the killer feature for jazz charts.
- **Trills, mordents, ornaments → expanded** into the actual MIDI notes.
- **Tempo changes (`\tempo`, `\accel`, `\rit`) → applied** to the MIDI clock.

`\unfoldRepeats` is usually paired with it so `\repeat volta` blocks actually repeat in MIDI.

`\articulate` only affects MIDI — the PDF rendering is untouched. See the two-`\score` pattern below for using one source for both a clean PDF and an articulated MIDI.

---

## Jazz Idioms

Lessons from real-world jazz transcription/composition. These come up often enough that they belong in your default toolkit.

### Swing notation: three approaches, three trade-offs

Verbal "swing" is not honored by MIDI export. Three ways to handle it:

| Approach | Score looks | MIDI sounds | Use when |
|---|---|---|---|
| **A. Straight 8ths + verbal "Swing"** | Clean jazz chart | Mechanical straight 8ths | Human readers only; no MIDI playback needed |
| **B. Triplet notation** (`\tuplet 3/2 { x4 y8 }` for each pair) | Bracket-cluttered | Swung correctly | You need MIDI playback and can't / don't want to use `\articulate` |
| **C. Straight 8ths + `\articulate`** | Clean jazz chart | Swung correctly | **Best of both** — the usual answer |

Approach C requires the two-`\score` pattern (see below). Approach B is the fallback when `\articulate` isn't available or you want a different swing ratio.

A wrinkle: bars that contain *real* 8th-note triplets (a triplet flurry, a 3:4 polyrhythm) should stay as `\tuplet 3/2 { x8 y8 z8 }` regardless of approach — those are not swing notation, they're actual triplets. Players read them as triplets.

### The two-`\score` pattern (clean chart + swung MIDI)

One source produces both a clean PDF (straight 8ths) and a swung MIDI (via `\articulate`):

```lilypond
\include "articulate.ly"

scoreBody = <<
  \new ChordNames \harmony
  \new Staff \melody
>>

% PDF: straight 8ths as written
\score {
  \scoreBody
  \layout {}
}

% MIDI: 8ths swung by articulate
\score {
  \unfoldRepeats \articulate \scoreBody
  \midi { \tempo 4 = 92 }
}
```

Define the music as a variable (`scoreBody = << ... >>`) so both `\score` blocks read the same source. Otherwise you'll drift the chart and the audio out of sync.

### `\bendAfter` — fall-off / scoop tails

```lilypond
c''2 \bendAfter #-6      % fall-off (downward bend after the note)
c''2 \bendAfter #+4      % scoop / bend up after the note
```

The number is the size of the bend in semitones; sign is direction. Visual only — does not render to MIDI.

### Grace-note bends (bend INTO a target)

The Scofield / blues idiom of bending into a chord tone:

```lilypond
\grace { c'8\glissando } cis'4       % bend up into c#'
\grace { e''8\glissando } d''4       % bend down into d''
```

This *does* render to MIDI as a fast grace note, but not as a true pitch-bend event. Visually it shows the grace-note + slur, which is what readers expect.

### `\clef "treble_8"` — the guitar-clef octave-shift gotcha

Treble clef with a small "8" below sounds **one octave lower than written**. Standard for guitar and tenor voice charts.

The trap: if you're converting an existing piece from `\clef treble` to `\clef "treble_8"` and want the *sounding* pitches to stay the same, **do not also shift the written pitches up an octave**. The clef does that for you. If you do both, you double-shift and end up an octave too high.

| Goal | Written | Clef | Sounds |
|---|---|---|---|
| Keep sounding pitches unchanged | unchanged | `treble_8` | unchanged |
| Make notes "sit higher on the staff" for readability | shifted up one octave | `treble` | one octave higher |
| ✗ Common mistake | shifted up one octave | `treble_8` | unchanged (but visually higher than it needs to be) |

Guitar music convention: keep written pitches at concert pitch and use `treble_8`. Notes will commonly drop below the staff with ledger lines — that's normal; guitarists read it without effort.

### `\chordmode` + `\new ChordNames` for chart-style harmony

Already covered in the Chord Mode section above. Quick reminder for jazz use:

```lilypond
harmony = \chordmode {
  \repeat unfold 8 { c1:7 | }       % 8 bars of C7 vamp
}

\score {
  <<
    \new ChordNames \harmony
    \new Staff \melody
  >>
}
```

Place `\new ChordNames` *first* inside the `<<...>>` so the chord track appears above the staff.

### Multi-staff jazz: melody + walking bass

A solo over a bass line. Bass on its own staff with bass clef:

```lilypond
\score {
  <<
    \new ChordNames \harmony
    \new Staff \with { instrumentName = "Gt." midiInstrument = "electric guitar (jazz)" } {
      \clef "treble_8" \melody
    }
    \new Staff \with { instrumentName = "Bs." midiInstrument = "acoustic bass" } {
      \clef bass \bassLine
    }
  >>
  \layout {}
  \midi { \tempo 4 = 92 }
}
```

`midiInstrument` is a GM patch name — `"acoustic bass"`, `"electric guitar (jazz)"`, `"electric piano 1"`, etc. (See `scm/midi.scm` in your LilyPond install for the full list, or use any GM patch name.)

### Hearing the MIDI

LilyPond produces a `.midi` file but no audio. To listen:

```bash
# fluidsynth + a SoundFont is the cleanest path
fluidsynth -i -g 2.0 -a pipewire ~/soundfonts/MuseScore_General.sf3 phrase.midi
```

Common gotchas:

- **Silent playback** usually means the synth has no soundfont mapped to the GM patch. Pass a `.sf2` / `.sf3` file explicitly.
- **Too quiet:** fluidsynth's default `-g` (gain) is 0.2 on a 0–10 scale. Push to `-g 2.0` or higher. Also check the PipeWire stream slider — fresh streams often start at ~40%.
- **No swing in playback:** add `\articulate` (above) or notate as triplets.

To render to a WAV file (e.g. for editing in a DAW or playing at boosted volume):

```bash
fluidsynth -F phrase.wav -g 3.0 ~/soundfonts/MuseScore_General.sf3 phrase.midi
mpv --volume=130 phrase.wav    # mpv goes above 100%
```

---

## Include and Modular Scores

```lilypond
% In main.ly
\include "violin.ly"
\include "cello.ly"

\score {
  <<
    \new Staff \violinPart
    \new Staff \celloPart
  >>
}
```

Each included file defines variables that the main file references. This keeps large scores manageable.
