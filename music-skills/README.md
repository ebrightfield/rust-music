# music-skills

Packaged contents of the music-related Claude Code skills used with this
workspace. Each subdirectory is a complete skill (`SKILL.md` plus its
`references/`), copied here as a self-contained, version-controllable snapshot.

| Skill | Purpose |
|-------|---------|
| `narveson-melody` | Compose, analyze, and revise melodies in the style of Paul Narveson's *Theory of Melody* (era/style profiles, five-pass revision, jazz lines). Emits LilyPond / MIDI / `music`-crate Rust artifacts. |
| `lilypond` | LilyPond engraving — syntax reference, sheet-music transcription from images, compilation debugging, QA against a source score, jazz idioms, and MIDI playback via fluidsynth + SoundFont. |
| `music-midi-render` | Turn melodies into expressive MIDI and audio with the workspace's `music-midi` crate (`SmfBuilder`, `AudioRenderer`, `SoundFont`, `MidiPlayer`) — multi-track SMF, velocity/tempo/dynamics transforms, WAV render. |
| `sheet-music-to-rust` | Transcribe sheet-music images/PDFs into LilyPond source and native `music`-crate Rust types (`Voicing`, `Pitch`, `RhythmicNotatedEvent`, `LilypondBuilder`), with ImageMagick preprocessing and round-trip QA. |

## Note on the live skills

This is a **snapshot**, not the source the Claude Code harness loads at runtime.
The active copies live at:

- `.claude/skills/{narveson-melody,music-midi-render,sheet-music-to-rust}` — in-repo
- `~/.claude/skills/lilypond` — user-global

Editing files here does **not** change skill behavior; update the live copies for
that.
