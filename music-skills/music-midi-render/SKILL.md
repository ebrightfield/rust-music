---
name: music-midi-render
description: Turn melodies (from the `music` crate or anywhere) into expressive MIDI and listenable audio using the workspace's `music-midi` crate. Use when the user wants to render a melody to WAV, build multi-track SMF (melody + bass + drums + comp), apply expressive transforms (velocity policies, tempo automation, dynamics), arrange a phrase with rhythm-section parts, or convert a LilyPond MIDI file into a Rust `music-midi` program. Composes the workspace `music-midi` crate (`SmfBuilder`, `AudioRenderer`, `SoundFont`, `MidiPlayer`) and inspects its actual API before emitting code. For swing feel, defers to the `lilypond` skill's `\articulate.ly` pattern. Triggers on phrases like "render this to audio", "play this with proper swing", "add a walking bass to my melody", "make a multi-track MIDI", "build a WAV from this Rust melody".
argument-hint: [intent] (e.g. "render /tmp/foo.midi to wav", "build SMF from these notes with bass", "play this Rust melody")
---

# music-midi-render — Rust-native MIDI rendering & arrangement

This skill operates the workspace's `music-midi` crate to turn `music`-crate types (`MelodicEvent`, `Pitch`, `RhythmicNotatedEvent`, `LilypondScore`) into MIDI and audio. It's the *audio output* counterpart to the `narveson-melody` skill (which produces the notes) and complements the `lilypond` skill (which handles notation and `\articulate`-driven swing).

The crate is feature-rich: `smf` (write SMF files), `playback` (live `midir`), `render` (WAV via `oxisynth` + SoundFont). The skill knows the feature flags and emits code that compiles.

## When to apply

Invoke when the user wants to:

- **Render a melody to audio** — `.midi` → `.wav`, ideally with a real SoundFont and no separate fluidsynth invocation.
- **Build a multi-track SMF** — melody + walking bass + drums + comp, each as its own track with its own GM patch.
- **Apply expressive transforms** — velocity from dynamics, tempo automation (accel/rit), per-track timing offset.
- **Live-play a melody** through `midir` to a connected MIDI device or DAW.
- **Convert** a LilyPond `.midi` into a Rust `music-midi` program (so it can be parameterized / programmatically modified).
- **Arrange** a melody by adding parts (bassline, drum pattern, comping voicings) from `music` primitives.

Do **not** invoke for:
- LilyPond syntax or visual notation (use `lilypond`).
- Composing the melody itself (use `narveson-melody`).
- VST hosting, DAW automation, audio plugin chains, live performance — explicitly out of scope.
- Image-to-Rust transcription (use `sheet-music-to-rust`).
- Swing notation choices in printed scores (use `lilypond`'s "Jazz Idioms" reference).

## Out of scope (explicit)

- **No VST or LV2 plugin hosting.** If the user wants amp sims, real guitar tone, or commercial sample libraries, point them at a DAW (Ardour/Reaper) and stop.
- **No DAW automation.** Writing OSC, sending MIDI clock to a DAW, controlling Ardour from Rust — out.
- **No live performance.** Latency tuning, monitoring, real-time audio routing — out. `midir` playback for sketch/test purposes is in; rigging it as a stage instrument is not.

If the user asks for any of these, say so plainly and recommend a tool that fits.

## The five operating modes

| Mode | Trigger phrases | Output |
|------|-----------------|--------|
| **render** | "render to audio/wav", "play this with proper sound" | Rust program that writes `.wav` via `AudioRenderer` |
| **smf** | "build an SMF / .midi", "save this as MIDI" | Rust program that writes a `.midi` file via `SmfBuilder` |
| **arrange** | "add bass / drums / comp", "make this multi-track" | Multi-track `SmfBuilder` program with one track per part |
| **play** | "play this through MIDI", "send to my synth" | Rust program using `MidiPlayer::play_blocking` |
| **convert** | "turn this LilyPond MIDI into Rust", "make a music-midi program from this .ly" | Hand-translated Rust source |

For ad-hoc playback of a `.midi` file the user already has (e.g. fresh from `lilypond foo.ly`), default to `fluidsynth` rather than writing Rust — see "When NOT to use `music-midi`" below.

## Composition with other skills

This skill **consumes** rather than replaces:

- **`lilypond`** — for any `.ly` source; in particular, the "Jazz Idioms" reference covers swing notation, `\articulate`, `treble_8`, and the two-`\score` pattern. **For swing feel: defer to the lilypond skill's `\articulate.ly` approach.** This skill does not implement a swing transform; if the user's input is a non-LilyPond MIDI file that they want swung, escalate to the user ("the swing path goes through LilyPond + `\articulate` — do you want to regenerate the source through LilyPond, or shall we accept the straight playback?").
- **`narveson-melody`** — for generating the melodic material. This skill takes that material and arranges/renders it.
- **`sheet-music-to-rust`** — for ingesting a printed source. Once it's Rust literals, this skill can render and arrange them.

## Always-inspect rule

The `music-midi` crate is evolving. Before emitting any Rust:

1. **Read the relevant source files** in `/home/eric/Documents/rust-music/music-midi/src/` to confirm current API. The key entry points are listed in `references/crate-api.md`.
2. **Check the active features** by reading `music-midi/Cargo.toml`. `smf` is default; `playback`/`render` are opt-in. Tell the user which `--features` they'll need.
3. **Confirm constructor signatures** by reading the relevant module (`smf.rs`, `render.rs`, `soundfont.rs`, etc.). Don't assume an old function name — the workspace's debt log (`docs/spec-music-midi-debt.md`) lists known-evolving areas.

The `lilypond-parser` and `music` crates are co-evolving too; check `music/src/prelude.rs` and `music-midi/src/lib.rs` for current exports.

## Workflow (compose / render mode)

The canonical flow when the user gives you a melody and wants audio:

1. **Identify input form.** Is it `Vec<MelodicEvent>`? A `LilypondScore`? Raw pitch+duration pairs the user typed? A `.midi` file on disk? Each takes a different entry point (see `references/crate-api.md` § "Entry points by input type").

2. **Confirm rendering target.** `.midi` (SmfBuilder), `.wav` (AudioRenderer), live playback (MidiPlayer). Multiple targets is fine — build the SMF once, then render and/or play.

3. **Inspect the crate.** Read `music-midi/src/lib.rs` for current re-exports; read the specific module you'll call into (smf.rs, render.rs, soundfont.rs); spot-check Cargo.toml for the right `--features` flags.

4. **Pick a SoundFont.** Three choices:
   - `SoundFont::general_user_gs()` — auto-downloads GeneralUser GS if not cached. Default choice.
   - `SoundFont::from_path("~/soundfonts/MuseScore_General.sf3")` — user's local file (this is the one we already have for fluidsynth).
   - `SoundFont::from_bytes(...)` — embedded sf2 for tests.

5. **Emit the Rust program.** Use the templates in `references/crate-api.md`. Bake in the right `cargo run --features ...` invocation in a comment at the top.

6. **Run it (if asked).** The user may want you to actually execute `cargo run`; ask before doing so since it builds the workspace.

7. **Verify the output.** If WAV, check the file size is reasonable (~150–300 KB/sec at 44.1 kHz mono); if SMF, check `MThd` magic bytes; if playback, check the synth port connected.

## Workflow (arrange mode)

When adding bass / drums / comp to an existing melody:

1. Start from the existing melody as `Vec<MelodicEvent>` (or `Vec<RhythmicNotatedEvent>`).
2. Build each accompaniment part as its own `Vec` (see `references/arrangements.md` for templates).
3. Combine via `SmfBuilder::add_track(name, channel, events)` — one track per part, different channels (0 = melody, 1 = bass, 9 = GM drum channel, etc.).
4. Set per-track `midiInstrument` via a Program Change event in the track (or let GM patch defaults work).
5. Build, render, optionally play.

Arrangement style matches the source melody's idiom — Scofield-style → walking bass + drum-kit ride + comping piano; Bach-style → continuo bass + harpsichord; etc. The user's prompt should disambiguate; ask if not.

## Workflow (convert mode)

When converting LilyPond MIDI to a `music-midi` Rust program:

1. The LilyPond `.ly` source is the better starting point than the `.midi` (preserves articulation hints, dynamics).
2. Walk the `.ly` measure-by-measure; map each note to a `MelodicEvent` or `RhythmicNotatedEvent`.
3. Map `\tempo` markings to a `StaticTempoMap`; mid-piece `\tempo` changes become `.push(tick, bpm)` calls.
4. Map dynamics (`\f`, `\p`, hairpins) to `Dynamic` events + a `VelocityPolicy`.
5. **`\articulate` swing does NOT convert.** If the source relies on `\articulate.ly` for swing playback, the converted program will play *straight*. Two responses: (a) tell the user and accept; (b) ask whether they want a triplet-rewrite for the swung sections. See `references/recipes.md` recipe 5 for a worked Scofield-vamp conversion.

## When NOT to use `music-midi`

For pure ad-hoc playback of a `.midi` file the user already has — especially one freshly produced by LilyPond — **don't write Rust**. Run `fluidsynth` directly:

```bash
fluidsynth -i -g 2.0 -a pipewire ~/soundfonts/MuseScore_General.sf3 phrase.midi
```

`music-midi` earns its keep when you need:
- Programmatic note generation (Rust as the source).
- Multi-track arrangement built from `music`-crate types.
- Embedding rendering in a larger Rust program (CLI, server, automated test fixture).
- Deterministic / reproducible audio output (the WAV bytes are bit-identical run-to-run for the same soundfont).

For "I have a .midi, let me hear it," fluidsynth is faster. See the `lilypond` skill's "Hearing the MIDI" subsection.

## Reference index — load on demand

| File | When to read it |
|------|-----------------|
| `references/crate-api.md` | Building any Rust output. The entry-point map (`MelodicEvent` → `SmfBuilder` → `AudioRenderer` → file). Updated by inspecting the actual crate at use time. |
| `references/arrangements.md` | Arrange mode: walking-bass templates, drum patterns, comping voicings, multi-track channel assignments. |
| `references/recipes.md` | Worked end-to-end examples. Recipe 1 = Scofield C7 vamp render. Recipe 2 = multi-track arrangement. Recipe 5 = LilyPond `.midi` → Rust conversion. Start here for unfamiliar tasks. |

Load references **only when needed**. Most render-mode tasks need only `crate-api.md`; arrangement adds `arrangements.md`.

## Default output behavior

- **Default target**: a runnable Rust file at `/tmp/music-midi-render-<descriptor>.rs` that the user can copy into `music-midi/examples/` if they want it committed.
- **Default cargo invocation**: print it as the first line comment in the Rust file. Don't run `cargo` yourself unless asked.
- **Default SoundFont**: `~/soundfonts/MuseScore_General.sf3` if it exists; otherwise `SoundFont::general_user_gs()` (which auto-downloads).
- **Default sample rate**: 44.1 kHz mono (the AudioRenderer default).
- **Default tempo**: 120 BPM if unspecified.

Don't write files unless the user asks. Default is inline output.
