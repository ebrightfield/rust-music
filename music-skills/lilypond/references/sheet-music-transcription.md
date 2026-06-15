# Sheet Music Transcription and QA

This reference covers workflows for transcribing sheet music from images into LilyPond, and for verifying existing LilyPond code against source images.

## Transcription Workflow

When given an image of sheet music, follow this systematic process:

### Phase 1: Analyze the Image

Before writing any code, extract these elements from the image:

1. **Global attributes**: clef, key signature, time signature, tempo marking
2. **Staff structure**: number of staves, staff grouping (piano grand staff, etc.)
3. **Measure count**: count all measures visible
4. **Voices**: identify polyphonic passages (multiple simultaneous voices per staff)
5. **Special elements**: repeats, volta brackets, coda/segno, dynamics, text markings
6. **Pickup measure**: check if the piece starts with an anacrusis

Report these observations before writing code.

### Phase 2: Transcribe Measure by Measure

Work through the score systematically:

1. **Set up the file skeleton first**: version, header, key/time/clef, empty variables
2. **Transcribe one measure at a time**: write each measure, then verify beat count matches time signature
3. **Use bar checks** (`|`) at every barline — this is non-negotiable
4. **Handle each staff independently**: complete the treble staff, then the bass staff (for piano)
5. **Add articulations and dynamics in a second pass** after notes and rhythms are correct

### Phase 3: Verify

After transcription, verify against the source image:

1. Count measures in the code vs. the image
2. Verify first and last notes of each line/system
3. Check key signature and accidentals
4. Verify time signature changes
5. Check for ties that cross barlines
6. Verify repeat structures

---

## QA Verification Checklist

When asked to check existing LilyPond code against a source image, systematically verify each dimension:

### Pitch Accuracy

- [ ] Every note letter name matches the score
- [ ] Accidentals (sharps/flats) match — remember LilyPond shows accidentals in the output even if not in the key signature when written as `cis`/`bes`
- [ ] In `\relative` mode: verify octave placement is correct by tracing the relative chain from the starting pitch
- [ ] Chord tones are complete — no missing or extra notes
- [ ] Grace note pitches are correct

**Common pitch errors**:
- Wrong octave in relative mode (cascading error — one mistake shifts everything)
- Confusing `b` (B natural) with `bes` (B flat) — especially in flat keys
- Missing accidentals that are present in the source as courtesy accidentals
- Enharmonic substitution (e.g., `dis` vs `ees`) — functionally equivalent but check context

### Rhythm Accuracy

- [ ] Every note duration matches the source
- [ ] Dotted rhythms are correctly notated (`.` after duration number)
- [ ] Ties (`~`) connect the right notes for the right durations
- [ ] Tuplets use correct fraction (`\tuplet 3/2` for triplets, etc.)
- [ ] Rest durations and placements match
- [ ] Each measure's total beats equal the time signature
- [ ] Grace notes don't steal time from the wrong beat

**Common rhythm errors**:
- Forgetting that duration carries forward (writing `c8 d e` when `d` and `e` should be quarters)
- Missing dots on dotted rhythms
- Wrong tuplet fraction
- Tie vs slur confusion (`~` vs `()`)

### Register (Octave) Accuracy

- [ ] Starting pitch of each staff is in the correct octave
- [ ] In relative mode: trace through large intervals (>4th) to verify octave marks
- [ ] Ledger line notes are in the correct octave
- [ ] Octave changes at clef changes are handled correctly
- [ ] 8va/8vb markings are reflected in the code

**Systematic register check for relative mode**:
1. Note the starting reference pitch (e.g., `\relative c'`)
2. For each note, calculate: is the nearest instance of that letter name the correct one?
3. If a `'` or `,` is present, verify the intended octave displacement
4. Pay special attention after large leaps — the next note's octave depends on where the leap landed

### Structural Accuracy

- [ ] Correct number of measures
- [ ] Repeat signs (`\repeat volta`) match the source
- [ ] Volta brackets have correct endings
- [ ] D.C., D.S., Coda, and Fine markings are present
- [ ] Key signature changes at the right measure
- [ ] Time signature changes at the right measure
- [ ] Double barlines, final barlines placed correctly

### Expression and Articulation

- [ ] Dynamics (p, f, mf, etc.) on the correct notes
- [ ] Hairpins (crescendo/decrescendo) start and end at the right places
- [ ] Slurs connect the correct note ranges
- [ ] Staccato, accent, tenuto, marcato on the correct notes
- [ ] Tempo markings present
- [ ] Text expressions (dolce, legato, etc.) present

---

## Transcription Patterns

### Piano Score

```lilypond
\version "2.24.4"

\header {
  title = "Piece Title"
  composer = "Composer Name"
  tagline = ##f
}

treble = \relative c'' {
  \clef treble
  \key c \major
  \time 4/4
  % Measure 1
  c4 e g c |
  % Measure 2
  b4 d f b |
}

bass = \relative c {
  \clef bass
  \key c \major
  \time 4/4
  % Measure 1
  c2 e |
  % Measure 2
  g,2 g |
}

\score {
  \new PianoStaff <<
    \new Staff = "treble" \treble
    \new Staff = "bass" \bass
  >>
  \layout { }
}
```

### Vocal Score with Piano

```lilypond
\version "2.24.4"

melody = \relative c'' {
  \clef treble
  \key f \major
  \time 3/4
  \autoBeamOff
  c4 a f | g2. |
}

words = \lyricmode {
  Sing a song __
}

upper = \relative c'' {
  \clef treble
  \key f \major
  \time 3/4
  <c f a>4 <c f a> <c f a> |
  <bes e g>2. |
}

lower = \relative c {
  \clef bass
  \key f \major
  \time 3/4
  f2 f4 | c2. |
}

\score {
  <<
    \new Staff \new Voice = "mel" \melody
    \new Lyrics \lyricsto "mel" \words
    \new PianoStaff <<
      \new Staff \upper
      \new Staff \lower
    >>
  >>
  \layout { }
}
```

### Polyphonic Passage (Two Voices on One Staff)

```lilypond
\relative c'' {
  \clef treble
  \key g \major
  \time 4/4
  % Single voice
  g4 a b c |
  % Two voices
  << { b4 a g fis } \\ { d4 d d d } >> |
  % Back to single voice
  g1 |
}
```

### Lead Sheet (Melody + Chord Symbols)

```lilypond
\version "2.24.4"

melody = \relative c'' {
  \clef treble
  \key c \major
  \time 4/4
  c4 e g e | f2 e |
}

chords = \chordmode {
  c2 c | f c |
}

\score {
  <<
    \new ChordNames \chords
    \new Staff \melody
  >>
  \layout { }
}
```

---

## Working with Difficult Passages

### Dense Chords

For passages with many chord tones, transcribe one chord at a time and count the notes against the image:

```lilypond
% Source shows: C major 7 in close position, root position
<c' e' g' b'>4
% Count: 4 notes. Check: C E G B. ✓
```

### Cross-Staff Notation

```lilypond
\new PianoStaff <<
  \new Staff = "up" { s1 }
  \new Staff = "down" {
    \clef bass
    c8 e g \change Staff = "up" c' e' g' c'' e'' |
    \change Staff = "down"
  }
>>
```

### Complex Rhythms

For complex rhythms, write out the math:
- In 4/4: each measure = 4 quarter note beats
- `c8 d8 e4 f4.` = 0.5 + 0.5 + 1 + 1.5 = 3.5 beats ← WRONG, missing 0.5 beat
- Fix: `c8 d8 e4 f4. g8` = 0.5 + 0.5 + 1 + 1.5 + 0.5 = 4 beats ✓

---

## Error Reporting Format

When reporting QA findings, use this format:

```
## QA Report: [Piece Name]

### Summary
- Measures checked: N
- Errors found: N
- Severity: [minor/moderate/critical]

### Findings

#### Measure 5, Beat 3 (treble staff)
- **Type**: Pitch error
- **Current**: `e'4`
- **Should be**: `ees'4`
- **Reason**: Source shows E-flat (flat in key signature)

#### Measure 12, Beat 1 (bass staff)
- **Type**: Rhythm error
- **Current**: `c2 d4`
- **Should be**: `c2 d4 r4`
- **Reason**: Measure only has 3 beats, needs rest on beat 4
```
