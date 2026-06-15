# Arrangements

Adding rhythm-section parts (bass, drums, comp) to a melody. Each accompaniment part is a `Vec<MelodicEvent>` (or `Vec<RhythmicNotatedEvent>` if you need polyphony / rests / ties).

Load this in arrange mode. The crate-api.md `SmfBuilder` section covers how to *attach* the parts; this file is about *generating* them stylistically.

---

## Track/channel conventions

| Part | Channel | GM patch (program number) | Notes |
|---|---|---|---|
| Melody | 0 | varies — guitar (24–31), piano (0–7), sax (64–71), etc. | First track ("melody" name) |
| Bass | 1 | 32 acoustic bass, 33 electric bass (finger), 34 electric bass (pick), 35 fretless | Walks in low octaves |
| Comp (chordal) | 2 | 0 grand piano, 4 Rhodes EP, 16 organ, 24 acoustic guitar | Block chords or comping voicings |
| Drums | **9** (always) | n/a — GM drum kit fixed | Channel 9 is the GM drum channel; program changes ignored |
| Secondary melody | 3 | varies | Countermelody, descant, harmony line |

GM patch list is in `oxisynth`'s internals; common values are listed in any GM reference. Channel 9 is hard-coded — every GM-compatible synth treats note numbers there as drums.

For drum programming, MIDI note numbers map to drum kit pieces:

| MIDI note | Drum | MIDI note | Drum |
|---|---|---|---|
| 35 | acoustic bass drum | 42 | closed hi-hat |
| 36 | bass drum 1 | 44 | pedal hi-hat |
| 38 | acoustic snare | 46 | open hi-hat |
| 40 | electric snare | 49 | crash cymbal 1 |
| 41 | low floor tom | 51 | ride cymbal 1 |
| 43 | high floor tom | 53 | ride bell |
| 45 | low tom | 56 | cowbell |
| 47 | low-mid tom | 57 | crash cymbal 2 |
| 48 | hi-mid tom | 59 | ride cymbal 2 |

---

## Walking bass — Scofield/jazz idiom

A walking bass is a quarter-note line that outlines chord tones with chromatic approach notes between. For a C7 vamp:

```rust
use music::melody::sequencer::MelodicEvent;
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::Duration;

// Walking bass over C7 vamp — 1 bar = 4 quarters.
// Pattern: root - 3rd - 5th - chromatic-approach-to-next-root
fn walking_bass_c7_one_bar(start_root_midi: u8) -> Vec<MelodicEvent> {
    let p = |m: u8| Pitch::from_midi(m).unwrap();
    vec![
        MelodicEvent::new(p(start_root_midi),       Duration::QTR),  // C2 = 36
        MelodicEvent::new(p(start_root_midi + 4),   Duration::QTR),  // E2 = 40 (the 3)
        MelodicEvent::new(p(start_root_midi + 7),   Duration::QTR),  // G2 = 43 (the 5)
        MelodicEvent::new(p(start_root_midi + 9),   Duration::QTR),  // A2 = 45 (approach back to C)
    ]
}
```

Bass note range: roughly `Pitch::from_midi(28)` (E1) to `Pitch::from_midi(55)` (G3). Real upright/electric basses live mostly in C2–C4.

**Pattern variations** for a 32-bar form:

- Chorus 1 (stating): 1-3-5-6 walk (the template above).
- Chorus 2 (extending): 1-2-3-4 chromatic walk-up, 1-7-6-5 chromatic walk-down.
- Chorus 3 (going outside): drop to a **root pedal** (4 quarters of the same root) — frees the soloist to play out without losing the harmonic ground.
- Chorus 4 (concluding): half-step chromatic walk-down from b7 to root in the final 2 bars.

---

## Drum patterns

### Medium swing ride pattern

The classic Scofield/post-bop drum part: ride cymbal carries the swing pulse, snare on 2 and 4 (cross-stick or rim), bass drum sparse "feathering."

```rust
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::Duration;

// One bar of 4/4 medium swing.
// Ride: 1, 2-and, 3, 4-and  (the "spang-a-lang" pattern)
// Snare: 2 and 4 (cross-stick = note 37)
// Kick: feathered on 1 and 3 (very quiet)
fn medium_swing_one_bar() -> Vec<MelodicEvent> {
    let p = |m: u8| Pitch::from_midi(m).unwrap();
    let q = Duration::QTR;
    let e = Duration::EIGHTH;

    // For drum tracks, MelodicEvent's pitch is the GM drum mapping.
    // Note: this is a monophonic per-channel-9 part; for true polyrhythm
    // (kick + ride + snare simultaneously) you'd need multiple tracks
    // or RhythmicNotatedEvent with chord-style sounding.
    vec![
        // Beat 1: kick + ride (use ride; kick goes in a separate track)
        MelodicEvent::new(p(51), q),    // ride on 1
        // Beat 2: snare (cross-stick) + ride 2-and
        MelodicEvent::new(p(51), e),    // ride on 2
        MelodicEvent::new(p(51), e),    // ride on and-of-2
        // Beat 3: ride
        MelodicEvent::new(p(51), q),    // ride on 3
        // Beat 4: snare (cross-stick) + ride 4-and
        MelodicEvent::new(p(51), e),    // ride on 4
        MelodicEvent::new(p(51), e),    // ride on and-of-4
    ]
}
```

**Important**: a real drum part has *simultaneous* sounding events (ride + snare on beat 2 at the same time), which `MelodicEvent` (monophonic) can't represent on a single track. Two options:

1. **Multiple drum tracks**, all on channel 9, each carrying one drum voice (one track for ride, one for snare, one for kick). The mixer in `oxisynth` overlays them correctly. This is the simpler approach.

2. **Use `RhythmicNotatedEvent::voicing(...)`** to sound multiple drum pitches at the same beat. The `Voicing` is a stack of pitches sounded together; on channel 9, each pitch is a separate drum.

Recommend approach (1) for first-pass arrangements — easier to write, easier to mix.

### Brushes / quieter swing

For ballad or quieter passages, replace ride with **closed hi-hat** (note 42) on every beat, and replace snare cross-stick with snare **brush sweep** (no GM equivalent; use note 38 with very low velocity).

### Latin / straight 8ths

For non-swing styles, replace the ride pattern with straight 8ths on the hi-hat:

```rust
// Hi-hat 8ths
vec![p(42); 8].into_iter().map(|p| MelodicEvent::new(p, Duration::EIGHTH))
```

Plus a bass-drum pattern on 1 and 3, snare on 2 and 4 — the classic backbeat.

---

## Comping voicings

Piano or guitar comping provides harmonic backdrop. Use `Voicing` from `music::prelude` for stacked-pitch chords, then `RhythmicNotatedEvent::voicing(voicing, duration)`.

### Block-chord comping (simple)

```rust
use music::prelude::*;
use music::notation::rhythm::{RhythmicNotatedEvent};
use music::notation::rhythm::duration::Duration;

// C7 voicing — shell voicing (root, 3rd, 7th) common in jazz comping
let c7_shell = voicing![pitch!(c, 3), pitch!(e, 3), pitch!(bes, 3)];

// One bar of 4/4: stab on beat 2 + beat 4 ("Freddie Green" style — but block)
let comp: Vec<RhythmicNotatedEvent> = vec![
    RhythmicNotatedEvent::rest(Duration::QTR),
    RhythmicNotatedEvent::voicing(c7_shell.clone(), Duration::QTR),
    RhythmicNotatedEvent::rest(Duration::QTR),
    RhythmicNotatedEvent::voicing(c7_shell.clone(), Duration::QTR),
];
```

### Syncopated comping (Scofield-style)

Real jazz comping hits on "and of 2" and "4" with short chord stabs, leaving space:

```rust
// One bar: rest 1, stab on 2-and, rest 3, stab on 4, rest 4-and
let comp: Vec<RhythmicNotatedEvent> = vec![
    RhythmicNotatedEvent::rest(Duration::new(DurationKind::Qtr, 1)),    // 1 + and of 1 + 2 = dotted-qtr rest
    RhythmicNotatedEvent::voicing(c7_voicing.clone(), Duration::EIGHTH), // and of 2
    RhythmicNotatedEvent::rest(Duration::QTR),                           // 3
    RhythmicNotatedEvent::voicing(c7_voicing.clone(), Duration::QTR),    // 4
    RhythmicNotatedEvent::rest(Duration::EIGHTH),                        // and of 4
];
```

Range: comp voicings sit *under* the melody — typically C3–C5 for guitar/piano, leaving the melody up high.

### Voicing styles

| Style | Voicing pattern (for Cmaj7 example) | Use |
|---|---|---|
| Shell (3rd + 7th) | `<e bes>` | Bare-bones jazz; lots of room for melody |
| Rootless 4-voice (3-5-7-9 or 3-6-9) | `<e g bes d'>` for C7, `<e a d'>` for C6/9 | Standard jazz piano L.H. |
| Drop-2 | bottom voice of a closed chord dropped an octave | Big-band guitar / piano |
| Quartal (stacked 4ths) | `<f bes ees'>` over C — McCoy Tyner | Modal / contemporary |
| Cluster (close 2nds) | `<d ees f>` | Contemporary / outside |

For Scofield-style work, **shell + rootless** is the default. Quartal voicings show up in modal sections.

---

## Multi-track wiring

Pulling it together. With `melody`, `bass`, `ride`, `kick`, `snare`, `comp` all built as `Vec<MelodicEvent>` (or `Vec<RhythmicNotatedEvent>`):

```rust
let smf = SmfBuilder::new()
    .ppq(480)
    .tempo(StaticTempoMap::constant(92.0))
    .add_track("melody", 0, melody.as_slice())?     // ch 0, guitar
    .add_track("bass",   1, bass.as_slice())?       // ch 1, bass
    .add_track("comp",   2, comp.as_slice())?       // ch 2, piano
    .add_track("ride",   9, ride.as_slice())?       // ch 9, drums
    .add_track("kick",   9, kick.as_slice())?       // ch 9, drums
    .add_track("snare",  9, snare.as_slice())?      // ch 9, drums
    .build()?;
```

**Program change events** for melody/bass/comp must be inserted somehow — check `convert/mod.rs` `ConvertCtx` for how the crate handles initial program changes. If there's no automatic mechanism, you'll need a manual `MidiMessage::ProgramChange { program }` event at tick 0 of each non-drum track.

---

## Arrangement templates by style

### Jazz combo (melody + bass + drums + comp)

- Melody: ch 0, guitar / sax / piano R.H.
- Bass: ch 1, walking quarter notes outlining chord tones
- Comp: ch 2, syncopated piano voicings on "and of 2" and "4"
- Drums: ch 9, swing ride + snare 2/4 + sparse kick

### Solo guitar (melody only, but with chord stabs interleaved)

- One track, ch 0
- Use `RhythmicNotatedEvent::voicing` for the chord stabs, `RhythmicNotatedEvent::pitch` for the melody notes
- Sound thin but legible

### Big band shout chorus (heavy)

- Multiple horn tracks: trumpets (ch 0), tenor sax (ch 1), trombones (ch 2)
- Walking bass (ch 3)
- Comping piano (ch 4) — block chords on the off-beats
- Drums (ch 9) — heavier kit, more cymbal crashes

### Classical chamber ensemble

- Violin 1 (ch 0), Violin 2 (ch 1), Viola (ch 2), Cello (ch 3)
- No drums
- Tempo automation (`StaticTempoMap.push(...)`) for rubato

---

## Sanity-checking arrangements

After building, verify:

- **Track count** matches expectations (`grep -c "\.add_track" /tmp/your-example.rs`).
- **Bass doesn't collide with melody** — peek at the lowest melody note vs. the highest bass note. They should be at least an octave apart usually.
- **Drum velocities** are reasonable — too quiet (< 40) drums get lost; too loud (> 110) drown the melody.
- **Tempo matches the style** — swing rides at 80–140 BPM; ballads at 50–80; up-tempo at 200+. Setting the wrong tempo on a good arrangement still sounds wrong.

Render to WAV and listen *before* claiming the arrangement is good. A correct-looking SMF can still sound terrible.
