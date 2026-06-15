# Recipes

End-to-end worked examples. Read these before unfamiliar tasks — most real requests are variations on one of these recipes.

Recipe 1 is the canonical session walk-through (Scofield C7 vamp, rendered as Rust). It captures the full pipeline; the others are variations.

---

## Recipe 1 — Render a melody to WAV

**Task:** "I have this melody as `Vec<MelodicEvent>`. Render it to audio."

**Output:** A Rust program at `/tmp/music-midi-render-melody.rs` that, when run, produces `out.midi` + `out.wav`.

```rust
// Run with:
//   cargo run -p music-midi --features full --example render-melody -- /tmp/out
//
// Produces /tmp/out.midi and /tmp/out.wav.

use std::{env, fs};
use std::path::PathBuf;

use music_midi::smf::SmfBuilder;
use music_midi::render::AudioRenderer;
use music_midi::soundfont::SoundFont;
use music_midi::{StaticTempoMap, MidiConversionError};

use music::melody::sequencer::MelodicEvent;
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::{Duration, DurationKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let out_stem = args.get(1).cloned().unwrap_or_else(|| "/tmp/out".into());
    let midi_path = PathBuf::from(format!("{out_stem}.midi"));
    let wav_path  = PathBuf::from(format!("{out_stem}.wav"));

    // --- the melody ---
    let melody = build_melody();

    // --- build SMF ---
    let smf = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("melody", 0, melody.as_slice())?
        .build()?;

    // Write .midi
    let mut bytes = Vec::new();
    smf.write(&mut bytes)
        .map_err(|e| MidiConversionError::Smf(e.to_string()))?;
    fs::write(&midi_path, &bytes)?;
    println!("Wrote {}", midi_path.display());

    // --- render to WAV ---
    let sf_path = "/home/eric/soundfonts/MuseScore_General.sf3";
    let sf = SoundFont::from_path(sf_path)
        .or_else(|_| SoundFont::general_user_gs())?;
    AudioRenderer::new(sf)?
        .sample_rate(44_100)
        .tail_seconds(2.0)
        .render_to_wav(&smf.as_smf(), &wav_path)?;
    println!("Wrote {}", wav_path.display());

    Ok(())
}

fn build_melody() -> Vec<MelodicEvent> {
    let p = |m: u8| Pitch::from_midi(m).unwrap();
    let q  = Duration::QTR;
    let h  = Duration::HALF;
    vec![
        // C major scale + a little flourish — replace with the user's actual melody
        MelodicEvent::new(p(60), q),   // C4
        MelodicEvent::new(p(62), q),   // D4
        MelodicEvent::new(p(64), q),   // E4
        MelodicEvent::new(p(65), q),   // F4
        MelodicEvent::new(p(67), h),   // G4
        MelodicEvent::new(p(69), q),   // A4
        MelodicEvent::new(p(71), q),   // B4
        MelodicEvent::new(p(72), h),   // C5
    ]
}
```

### Where it lives

For one-off use, save to `/tmp/music-midi-render-melody.rs` and compile inside the workspace:

```bash
# As an ad-hoc example inside the workspace:
cp /tmp/music-midi-render-melody.rs music-midi/examples/render-melody.rs
cargo run -p music-midi --features full --example render-melody -- /tmp/out
```

To commit the example permanently, place it under `music-midi/examples/` with a meaningful name (and add brief doc-comments at the top per the workspace's convention — see `music-midi/examples/score_to_mid.rs`).

---

## Recipe 2 — Multi-track Scofield arrangement (the session output, in Rust)

**Task:** "Take the Scofield C7 vamp we did in LilyPond and turn it into a 4-track `music-midi` arrangement: guitar, bass, drums, piano comp."

**Output:** Rust program building a 4-track SMF + WAV.

```rust
// Run with:
//   cargo run -p music-midi --features full --example scofield-c7-vamp
//
// Produces /tmp/scofield-c7-vamp.{midi,wav}.

use music_midi::smf::SmfBuilder;
use music_midi::render::AudioRenderer;
use music_midi::soundfont::SoundFont;
use music_midi::{StaticTempoMap, MidiConversionError};

use music::melody::sequencer::MelodicEvent;
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::{Duration, DurationKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let melody = guitar_melody();
    let bass   = walking_bass();
    let ride   = swing_ride();
    let snare  = snare_pattern();

    let smf = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(92.0))
        .add_track("guitar", 0, melody.as_slice())?
        .add_track("bass",   1, bass.as_slice())?
        .add_track("ride",   9, ride.as_slice())?
        .add_track("snare",  9, snare.as_slice())?
        .build()?;

    let mut bytes = Vec::new();
    smf.write(&mut bytes)
        .map_err(|e| MidiConversionError::Smf(e.to_string()))?;
    std::fs::write("/tmp/scofield-c7-vamp.midi", &bytes)?;

    let sf = SoundFont::from_path("/home/eric/soundfonts/MuseScore_General.sf3")?;
    AudioRenderer::new(sf)?
        .sample_rate(44_100)
        .tail_seconds(3.0)
        .render_to_wav(&smf.as_smf(), "/tmp/scofield-c7-vamp.wav")?;

    println!("Wrote /tmp/scofield-c7-vamp.{{midi,wav}}");
    Ok(())
}

fn guitar_melody() -> Vec<MelodicEvent> {
    let p = |m: u8| Pitch::from_midi(m).unwrap();
    // Translate the LilyPond melody — same pitches, straight 8ths.
    // (The full transcription is long; abbreviated here.)
    let e  = Duration::EIGHTH;
    let q  = Duration::QTR;
    vec![
        // bar 1: r4 e8 ees8 d8 e8 g8 e8 — rests not representable in MelodicEvent
        // so we start at the first sounding note. (To keep timing right, prepend a
        // rest-track or use RhythmicNotatedEvent::rest in a Vec<RhythmicNotatedEvent>.)
        MelodicEvent::new(p(64), e),  // e
        MelodicEvent::new(p(63), e),  // ees
        MelodicEvent::new(p(62), e),  // d
        MelodicEvent::new(p(64), e),  // e
        MelodicEvent::new(p(67), e),  // g
        MelodicEvent::new(p(64), e),  // e
        // ... continue for 8 bars ...
    ]
}

fn walking_bass() -> Vec<MelodicEvent> {
    let p = |m: u8| Pitch::from_midi(m).unwrap();
    let q = Duration::QTR;
    // 8 bars of C7 walking bass: C-E-G-A pattern.
    let mut v = Vec::new();
    for _bar in 0..8 {
        v.push(MelodicEvent::new(p(36), q));  // C2
        v.push(MelodicEvent::new(p(40), q));  // E2
        v.push(MelodicEvent::new(p(43), q));  // G2
        v.push(MelodicEvent::new(p(45), q));  // A2 — chromatic approach back to C
    }
    v
}

fn swing_ride() -> Vec<MelodicEvent> {
    let p = |m: u8| Pitch::from_midi(m).unwrap();
    let q = Duration::QTR;
    let e = Duration::EIGHTH;
    // Ride pattern: 1, 2-and, 3, 4-and (8 bars)
    let mut v = Vec::new();
    for _ in 0..8 {
        v.push(MelodicEvent::new(p(51), q));  // 1 - ride
        v.push(MelodicEvent::new(p(51), e));  // 2 - ride
        v.push(MelodicEvent::new(p(51), e));  // and-2 - ride
        v.push(MelodicEvent::new(p(51), q));  // 3 - ride
        v.push(MelodicEvent::new(p(51), e));  // 4 - ride
        v.push(MelodicEvent::new(p(51), e));  // and-4 - ride
    }
    v
}

fn snare_pattern() -> Vec<MelodicEvent> {
    let p = |m: u8| Pitch::from_midi(m).unwrap();
    let q = Duration::QTR;
    let h = Duration::HALF;
    // Snare cross-stick on 2 and 4 (8 bars)
    let mut v = Vec::new();
    for _ in 0..8 {
        // The "rest" before beat 2 is a duration before the snare hit, but
        // MelodicEvent has no rests. For accurate timing of multi-track drums,
        // prefer Vec<RhythmicNotatedEvent> with ::rest entries (see Recipe 4),
        // OR put each drum on its own track and let the channel-9 mixer handle it.
        v.push(MelodicEvent::new(p(37), q));  // cross-stick (note 37)
        v.push(MelodicEvent::new(p(37), q));
    }
    v
}
```

### Notes on this recipe

- **Rests are required for accurate drum timing.** `MelodicEvent` has no rest variant. The melody and drums above will start sounding immediately on tick 0 — the snare won't actually land on beat 2 unless either (a) you use `RhythmicNotatedEvent` (which supports rests), or (b) you put each drum voice on its own track and the tracks naturally align at tick 0.
- **For real fidelity to the LilyPond source**, convert the melody as `Vec<RhythmicNotatedEvent>` instead, since the source contains rests, ties, and dotted durations. See Recipe 4.
- **`\articulate`-derived swing in the source LilyPond** does not carry over to this conversion. The rendered output will play *straight*. To add swing, post-process the rendered SMF or notate the swing as triplets in the Rust source.

---

## Recipe 3 — Play live through `midir`

**Task:** "Send this melody to my connected MIDI device / DAW."

**Output:** Rust program that opens the first available MIDI output and plays the SMF.

```rust
// Run with:
//   cargo run -p music-midi --features full --example play-live
//
// Connects to the first available MIDI output port and plays.

use music_midi::smf::SmfBuilder;
use music_midi::playback::MidiPlayer;
use music_midi::{StaticTempoMap, MidiConversionError};

use music::melody::sequencer::MelodicEvent;
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let melody: Vec<MelodicEvent> = build_melody();

    let smf = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("melody", 0, melody.as_slice())?
        .build()?;

    let mut player = MidiPlayer::connect_default()?;
    println!("Connected to default MIDI output. Playing...");
    player.play_blocking(&smf.as_smf())?;
    println!("Done.");
    Ok(())
}

fn build_melody() -> Vec<MelodicEvent> {
    /* ... */
    vec![]
}
```

If the user wants to pick a specific port (e.g. their DAW's virtual input), enumerate `midir::MidiOutput::ports()` first and print them; then accept a `--port N` argument.

---

## Recipe 4 — Use `RhythmicNotatedEvent` for rests/ties/dynamics

**Task:** "I have a phrase with rests and ties (e.g. a LilyPond fragment). Render it accurately."

`MelodicEvent` is pitch+duration only. For anything with rests, ties, dynamics, or voicings, use `RhythmicNotatedEvent`.

```rust
use music::prelude::*;
use music::notation::rhythm::{RhythmicNotatedEvent};
use music::notation::rhythm::duration::{Duration, DurationKind};

// LilyPond: r8 e'8 d'8 e'8 ~ e'2  -- rest, three eighths, tied half
let phrase: Vec<RhythmicNotatedEvent> = vec![
    RhythmicNotatedEvent::rest(Duration::EIGHTH),
    RhythmicNotatedEvent::pitch(pitch!(e, 4),  Duration::EIGHTH),
    RhythmicNotatedEvent::pitch(pitch!(d, 4),  Duration::EIGHTH),
    RhythmicNotatedEvent::pitch_tied(pitch!(e, 4), Duration::EIGHTH),
    RhythmicNotatedEvent::pitch(pitch!(e, 4),  Duration::HALF),
];

let smf = SmfBuilder::new()
    .ppq(480)
    .tempo(StaticTempoMap::constant(92.0))
    .add_track("melody", 0, phrase.as_slice())?  // [RhythmicNotatedEvent] impls ToMidiEvents
    .build()?;
```

For dynamics (`\f`, `\p` etc.) and per-note velocity control, build a custom `VelocityPolicy` and pass it through `ConvertCtx`. See `convert/mod.rs` for the current ctx API.

---

## Recipe 5 — Convert a LilyPond `.midi` → `music-midi` Rust program

**Task:** "I have a LilyPond `.ly` (and the `.midi` it produced). Make a `music-midi` program that does the same thing."

**Why bother?** Because the Rust version is *programmatic* — you can parameterize the tempo, transpose, generate variations, layer it with other parts, etc.

**Workflow:**

1. **Don't parse the `.midi`** — `midly::Smf::parse` works but throws away articulation information. Read the `.ly` source instead.
2. Walk the source measure-by-measure. Each note becomes a `RhythmicNotatedEvent` (use `::pitch`, `::pitch_tied`, `::rest`, `::voicing` as appropriate).
3. Map `\tempo` markings to `StaticTempoMap`.
4. Map `\time` to `Meter` (passed to `SmfBuilder::meter(...)`).
5. **Swing handling** (this is the hard part):
   - If the source uses `\include "articulate.ly"` + a separate `\articulate`d MIDI score, **the converted Rust program will play straight**. There's no `\articulate` equivalent in `music-midi`.
   - Workarounds: (a) notate every 8th-pair as a triplet (`\tuplet 3/2 { x4 y8 }` → an event sequence representing 2/3 + 1/3 of the beat); (b) accept the straight playback; (c) escalate to the user and ask which they want.
6. Map dynamics if present to `Dynamic` events + a `VelocityPolicy`.
7. Render to verify against the LilyPond MIDI's pitches and rhythms (modulo swing).

If the source is large (>32 bars), this is a manual chore. Worth doing once for a "headline" piece, but for ad-hoc playback, `fluidsynth` on the LilyPond-produced `.midi` is faster.

---

## Recipe 6 — Render a `.midi` file directly (skip the Rust)

**Task:** "I just want to render this `.midi` file to WAV with the same SoundFont my LilyPond uses."

**Decision: don't write Rust for this.** Use the workspace's `music-midi` CLI-friendly path *if* one exists, or fall back to fluidsynth.

If you do want it in Rust (for batch processing, scripting, deterministic output):

```rust
use std::fs;
use music_midi::render::AudioRenderer;
use music_midi::soundfont::SoundFont;
use midly::Smf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read("/tmp/phrase.midi")?;
    let sf = SoundFont::from_path("/home/eric/soundfonts/MuseScore_General.sf3")?;
    AudioRenderer::new(sf)?
        .sample_rate(44_100)
        .tail_seconds(2.0)
        .render_bytes_to_wav(&bytes, "/tmp/phrase.wav")?;
    Ok(())
}
```

Otherwise, the one-liner:

```bash
fluidsynth -F /tmp/phrase.wav -g 3.0 ~/soundfonts/MuseScore_General.sf3 /tmp/phrase.midi
```

The two paths produce slightly different WAVs (different synth engines: `oxisynth` vs `fluidsynth`). For deterministic / committed test fixtures, prefer the Rust path. For ad-hoc listening, fluidsynth is fine.

---

## When the user is asking the wrong question

Common cases where the user says "render this in `music-midi`" but a different tool fits better:

- **"Make it sound real" / "Use a real guitar tone"** → soundfont GM patches are limited. Recommend a DAW + sample library. Out of scope for this skill.
- **"Apply Scofield-quality swing"** → `\articulate` swing is the best you'll get without a custom transform. If they want a specific ratio, that's a *new* tool (rule of three).
- **"Generate the melody for me"** → that's `narveson-melody`.
- **"Make a chart from this"** → that's the `lilypond` skill.
- **"Loop this for 30 minutes"** → use the `repeat` parameter or just duplicate the events; not a `music-midi` feature.

When in doubt, ask. Don't write Rust that does the wrong thing well.
