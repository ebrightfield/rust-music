# `music-midi` crate API

The entry points and call shapes you need to emit Rust against the workspace's `music-midi` crate. **Verify each shape against the live source** before pasting — the crate is evolving (see `docs/spec-music-midi-debt.md`).

Source root: `/home/eric/zooanthid/rust-music/music-midi/`.

---

## Feature flags

`music-midi/Cargo.toml`:

| Feature | What it pulls in | When to enable |
|---|---|---|
| `smf` (default) | `midly` — writing Standard MIDI Files. Also implies `score`. | Almost always; needed for any `.midi` output. |
| `score` | `music/lilypond` — `LilypondScore` conversion. | When the input is a `LilypondScore`. |
| `playback` | `midir` — live MIDI via OS port. | Live `MidiPlayer` playback. |
| `render` | `oxisynth`, `hound`, `ureq`, `dirs`, `sha2` — synthesis + WAV + SoundFont download. | WAV output via `AudioRenderer`. |
| `full` | `smf + playback + render`. | Render + play + write. The usual choice for a worked example. |

**Cargo invocation pattern:**

```bash
cargo run -p music-midi --features full --example my-render
# or, when adapting a one-off into music-midi/examples/:
cargo run -p music-midi --features smf,render --example my-render
```

Always print the right invocation as the first line comment in emitted Rust.

---

## Top-level re-exports (`lib.rs`)

Confirm by reading `music-midi/src/lib.rs`. As of this writing:

```rust
pub use error::MidiConversionError;
pub use event::{MidiEvent, MidiMessage, AbsoluteTicks, DEFAULT_PPQ};
pub use tempo::{TempoSource, StaticTempoMap, BoxedTempoSource};
pub use dynamics::{Dynamic, VelocityPolicy};
pub use convert::{ToMidiEvents, ConvertCtx};

#[cfg(feature = "smf")]
pub use smf::{OwnedSmf, pitch_to_smf_bytes, melody_to_smf_bytes};

#[cfg(feature = "score")]
pub use smf::score_to_smf_bytes;
```

Modules: `error`, `event`, `dynamics`, `tempo`, `convert`. Feature-gated: `smf`, `playback`, `render`, `soundfont`.

---

## Entry points by input type

| You have... | First call | Then... |
|---|---|---|
| `Vec<MelodicEvent>` from `music::melody` | `melody_to_smf_bytes(&melody, bpm)` or `SmfBuilder::new().add_track(name, ch, melody.as_slice())` | Save bytes or pass `OwnedSmf` to `AudioRenderer` |
| Single `Pitch` (e.g. a test tone) | `pitch_to_smf_bytes(&pitch, duration, bpm)` | Save or render |
| `Vec<RhythmicNotatedEvent<'a>>` from `music::notation::rhythm` | `SmfBuilder::new().add_track(...)` — `&[RhythmicNotatedEvent]` impls `ToMidiEvents` | Build → render or write |
| `LilypondScore` (needs `score` feature) | `score_to_smf_bytes(&score, bpm)` | Save or render |
| Existing `.midi` file on disk | `midly::Smf::parse(&fs::read(path)?)` then `AudioRenderer::render_to_wav(&smf, "out.wav")` | Audio out |
| Raw `Vec<MidiEvent>` (hand-built) | Implement `ToMidiEvents` for your container, or use the `MidiEvent` slice directly with `SmfBuilder` | Build |

---

## The `ToMidiEvents` trait

`convert/mod.rs`:

```rust
pub trait ToMidiEvents {
    fn to_midi_events(&self, ctx: &mut ConvertCtx) -> Result<Vec<MidiEvent>, MidiConversionError>;
}
```

Implementations live in `convert/{melody,pitch,rhythm,voicing,score}.rs`. The relevant ones:

- `impl ToMidiEvents for [MelodicEvent]` — see `convert/melody.rs`.
- `impl ToMidiEvents for [RhythmicNotatedEvent<'_>]` — see `convert/rhythm.rs`.
- `impl ToMidiEvents for LilypondScore<'_>` — see `convert/score.rs` (feature `score`).

This means you pass `melody.as_slice()` (note: `&[MelodicEvent]`, not `&Vec<MelodicEvent>`) to `add_track`.

`ConvertCtx::new(...)` — read `convert/mod.rs` for the current parameter list (it controls PPQ, tempo source, velocity policy, etc.).

---

## `SmfBuilder` — the canonical multi-track entry point

`smf.rs`:

```rust
use music_midi::smf::SmfBuilder;
use music_midi::StaticTempoMap;

let smf: OwnedSmf = SmfBuilder::new()
    .ppq(480)                                    // default DEFAULT_PPQ if omitted
    .tempo(StaticTempoMap::constant(120.0))      // BPM
    .meter(Meter::common_time())                 // optional; for time-signature track meta-event
    .add_track("melody", 0, melody.as_slice())?  // channel 0
    .add_track("bass",   1, bass.as_slice())?    // channel 1
    .add_track("drums",  9, drums.as_slice())?   // channel 9 = GM drum kit
    .build()?;

let mut bytes = Vec::new();
smf.write(&mut bytes)
    .map_err(|e| MidiConversionError::Smf(e.to_string()))?;
std::fs::write("out.midi", &bytes)?;
```

Key call shapes to verify before pasting:

- `add_track<T: ToMidiEvents + ?Sized>(self, name: &str, channel: u8, events: &T) -> Result<Self, _>` — verify exact name and signature in `smf.rs`.
- `build(self) -> Result<OwnedSmf, _>`.
- `OwnedSmf::as_smf<'a>(&'a self) -> Smf<'a>` — for passing to `AudioRenderer`.
- `OwnedSmf::to_bytes(&self) -> Result<Vec<u8>, _>` — direct bytes shortcut.

---

## One-shot helpers (no `SmfBuilder` needed)

For single-track jobs:

```rust
// All in music_midi::smf, re-exported at crate root when `smf` feature is on.
let bytes = melody_to_smf_bytes(&melody, 120.0 /* bpm */)?;     // -> Vec<u8>
let bytes = pitch_to_smf_bytes(&pitch, Duration::QTR, 120.0)?;  // single note test
let bytes = score_to_smf_bytes(&lilypond_score, 120.0)?;        // with `score` feature
```

Verify signatures: `grep -n "pub fn .*_to_smf_bytes" music-midi/src/smf.rs`.

---

## `AudioRenderer` — SMF → WAV

`render.rs`:

```rust
use music_midi::render::AudioRenderer;
use music_midi::soundfont::SoundFont;

let sf = SoundFont::from_path("/home/eric/soundfonts/MuseScore_General.sf3")?;
// OR: SoundFont::general_user_gs()?  -- auto-downloads & caches

let renderer = AudioRenderer::new(sf)?
    .sample_rate(44_100)
    .tail_seconds(2.0);   // silent decay after last note-off

// Path A: from a built OwnedSmf
let smf_view = owned_smf.as_smf();
renderer.render_to_wav(&smf_view, "out.wav")?;

// Path B: from raw bytes (e.g. a .midi file on disk, including LilyPond-produced)
let bytes = std::fs::read("phrase.midi")?;
renderer.render_bytes_to_wav(&bytes, "out.wav")?;
```

Key methods (verify in `render.rs`):

- `AudioRenderer::new(sf: SoundFont) -> Result<Self, _>`.
- Builder-style: `.sample_rate(u32) -> Self`, `.tail_seconds(f32) -> Self`.
- `render_to_wav(&self, smf: &midly::Smf, path: impl AsRef<Path>) -> Result<(), _>`.
- `render_bytes_to_wav(&self, bytes: &[u8], path: impl AsRef<Path>) -> Result<(), _>`.

Mono 16-bit WAV at the chosen sample rate. Roughly 88 KB/sec at 44.1 kHz.

---

## `SoundFont` — three ways to load

`soundfont.rs`:

```rust
let sf = SoundFont::from_path("path/to/file.sf2")?;     // local file (sf2 or sf3)
let sf = SoundFont::from_bytes(&include_bytes!(...))?;  // embedded
let sf = SoundFont::general_user_gs()?;                 // online; downloads + caches
let sf = SoundFont::general_user_gs_offline()?;         // offline; requires cached file
```

The `general_user_gs()` cache lives under `dirs::cache_dir()` — on Linux that's typically `~/.cache/music-midi/soundfonts/`. First call may take a few seconds; subsequent calls are instant.

In this workspace, **prefer `from_path("/home/eric/soundfonts/MuseScore_General.sf3")`** — the user already downloaded it during the earlier session. Fall back to `general_user_gs()` if asked for a self-contained example.

---

## `MidiPlayer` — live playback via `midir`

`playback.rs` (needs `playback` feature):

```rust
use music_midi::playback::MidiPlayer;

let mut player = MidiPlayer::connect_default()?;                       // first MIDI port
// OR: MidiPlayer::from_sink(...) -- inject a specific MidiOutputConnection

player.play_blocking(&smf_view)?;  // blocks until done; respects tempo
// OR:
let handle = player.play_background(smf_view.clone(), tempo)?;
// ...do other work...
handle.stop();
```

Verify in `playback.rs`. Background playback returns a `PlaybackHandle` whose `.stop()` joins the thread.

`connect_default()` picks the first available output port. The user may want a specific port (e.g. their DAW's virtual MIDI input); fall back to enumerating with `midir::MidiOutput::ports()` if needed.

---

## Tempo control

`tempo.rs`:

```rust
use music_midi::{StaticTempoMap, TempoSource};

// Constant tempo throughout
let tempo = StaticTempoMap::constant(120.0);

// Tempo changes mid-piece
let mut tempo = StaticTempoMap::constant(120.0);
tempo.push(absolute_ticks_at_bar_5, 90.0);    // rit at bar 5
tempo.push(absolute_ticks_at_bar_9, 120.0);   // back to tempo

// Pass to SmfBuilder
SmfBuilder::new().tempo(tempo)...
```

`AbsoluteTicks` is just an alias for `u64` (or similar — verify in `event.rs`). Use `bar * beats_per_bar * ppq` for bar-aligned changes.

`BoxedTempoSource` exists for dyn dispatch when you need to swap tempo strategies at runtime.

---

## Dynamics & velocity

`dynamics.rs`:

```rust
use music_midi::dynamics::{Dynamic, VelocityPolicy};

let policy = VelocityPolicy::default();        // verify variants in dynamics.rs
let velocity = policy.velocity_for(&rne);      // u8 in [0, 127]
let baseline = policy.velocity_no_event();     // u8 for unmarked notes
```

`Dynamic` enum covers `pp`, `p`, `mp`, `mf`, `f`, `ff`, etc. `VelocityPolicy` decides how a `Dynamic` notation maps to a 0–127 velocity. Hand-roll a custom `VelocityPolicy` if you need humanization.

---

## A complete worked example

`music-midi/examples/score_to_mid.rs` is the canonical worked source. Read it once before emitting; copy its shape.

Highlights:

```rust
use music_midi::{smf::SmfBuilder, StaticTempoMap};
use music::melody::sequencer::MelodicEvent;
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::{Duration, DurationKind};

let melody: Vec<MelodicEvent> = vec![ /* ... */ ];

let smf = SmfBuilder::new()
    .ppq(480)
    .tempo(StaticTempoMap::constant(120.0))
    .add_track("melody", 0, melody.as_slice())?
    .build()?;

let mut bytes = Vec::new();
smf.write(&mut bytes)
    .map_err(|e| MidiConversionError::Smf(e.to_string()))?;
fs::write(output_path, &bytes)?;
```

Run with: `cargo run -p music-midi --example score_to_mid --features smf -- /tmp/out.mid`.

---

## Common pitfalls

- **Forgetting `--features`**: the default is just `smf`. Add `render` to call `AudioRenderer`. Add `playback` to call `MidiPlayer`. `full` covers everything.
- **`melody` vs `melody.as_slice()`**: `add_track` takes `&T: ToMidiEvents`. The `impl` is for `[MelodicEvent]` (the slice), not `Vec<MelodicEvent>`. Always call `.as_slice()` (or pass `&melody[..]`).
- **`Pitch::from_midi`** returns `Option<Pitch>`; unwrap with `.expect("in MIDI range")` for hard-coded note numbers, or `?` after `.ok_or(...)` for fallible inputs.
- **GM patch numbers** start at 0, not 1. "Electric guitar (jazz)" = program 26, not 27. The `general_user_gs` SoundFont uses GM numbering.
- **Drum channel is 9** (zero-indexed). Track-level program changes don't apply to channel 9 — the GM drum kit is always there.
- **Tempo via `\tempo` in LilyPond ≠ tempo in the SMF**. LilyPond's `\midi { \tempo 4 = 92 }` emits the tempo meta-event correctly; verbal `\tempo "Medium swing" 4 = 92` does too. Both are honored by `AudioRenderer`.
- **`articulate.ly`-derived swing in LilyPond MIDI**: when you load a LilyPond-produced `.midi` and render it with `AudioRenderer`, the swing baked in by `\articulate` is preserved — it's just notes with timing offsets, no special handling needed.
