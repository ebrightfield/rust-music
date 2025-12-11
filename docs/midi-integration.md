# MIDI File Integration

This crate doesn't include MIDI file parsing, but you can integrate it with existing MIDI libraries. This document shows how to read MIDI files and convert them to rust-music types.

## Dependencies

Add [midly](https://lib.rs/crates/midly) to your `Cargo.toml`:

```toml
[dependencies]
midly = "0.5"
music = { path = "./music" }
```

**Why midly?** It's the most actively maintained and widely-used Rust MIDI library (~6,700 downloads/month, used by 62 crates). It provides:

- Feature-complete SMF reading and writing with proper handling of edge cases
- Zero-copy parsing for performance
- Optional `no_std` and `no_alloc` support for embedded use
- Support for both file-based and real-time MIDI packets

For real-time MIDI I/O (connecting to hardware devices), add [midir](https://lib.rs/crates/midir) alongside midly.

## Type Mappings

| MIDI Concept | rust-music Type | Conversion |
|--------------|-----------------|------------|
| Note number (0-127) | `Pitch` | `Pitch::from(midi_num: u8)` |
| Note number mod 12 | `Pc` | `Pc::from(&pitch)` or `Pc::from(midi_num % 12)` |
| Spelled note | `Note` | Requires spelling heuristic (see below) |

## Basic Conversion

```rust
use midly::{Smf, TrackEventKind, MidiMessage};
use music::note::{Pitch, Pc, Note};

// Parse a MIDI file
let data = std::fs::read("song.mid").unwrap();
let smf = Smf::parse(&data).unwrap();

// Convert MIDI note numbers to Pitch
for track in smf.tracks {
    for event in track {
        if let TrackEventKind::Midi { message: MidiMessage::NoteOn { key, vel }, .. } = event.kind {
            // MIDI note number directly maps to Pitch
            let pitch = Pitch::from(key.as_int()); // Pitch implements From<u8>

            // Extract pitch class (octave-agnostic)
            let pc: Pc = Pc::from(&pitch);

            // Get possible spelled notes for this pitch class
            let possible_notes: Vec<Note> = pc.notes();
        }
    }
}
```

## Spelling Heuristics

MIDI doesn't encode enharmonic spelling (C# vs Db). You'll need heuristics to determine spelling:

```rust
use music::note::{Pc, Note, Pitch};
use music::note_collections::PcSet;

// Option 1: Use key signature context
fn spell_in_key(pc: Pc, key_root: Note) -> Note {
    // Choose spelling based on key signature
    pc.notes().into_iter()
        .min_by_key(|n| /* prefer notes in key */)
        .unwrap()
}

// Option 2: Build a PcSet and use chord naming
fn spell_chord(midi_notes: &[u8]) -> Option<ChordName> {
    let pcs: Vec<Pc> = midi_notes.iter()
        .map(|&n| Pc::from(n % 12))
        .collect();
    let pc_set = PcSet::new(pcs);
    // Use naming_heuristics module to infer chord name
    pc_set.try_into().ok()
}
```

## Building Collections from MIDI

### PcSet (for harmonic analysis)

```rust
use music::note_collections::PcSet;

fn midi_chord_to_pcset(simultaneous_notes: &[u8]) -> PcSet {
    let pcs: Vec<Pc> = simultaneous_notes.iter()
        .map(|&n| Pc::from(n % 12))
        .collect();
    PcSet::new(pcs)
}
```

### Voicing (preserves octave and ordering)

```rust
use music::note_collections::Voicing;

fn midi_chord_to_voicing(simultaneous_notes: &[u8]) -> Voicing {
    let pitches: Vec<Pitch> = simultaneous_notes.iter()
        .map(|&n| Pitch::from(n))
        .collect();
    Voicing::new(pitches)
}
```

## Complete Example

This example parses a MIDI file and extracts frames of active pitches:

```rust
use midly::{Smf, TrackEventKind, MidiMessage};
use music::note::Pitch;
use music::note_collections::PcSet;
use std::collections::HashMap;

struct MidiFrame {
    tick: u32,
    pitches: Vec<Pitch>,
}

fn parse_midi_to_frames(path: &str) -> Vec<MidiFrame> {
    let data = std::fs::read(path).unwrap();
    let smf = Smf::parse(&data).unwrap();

    let mut active_notes: HashMap<u8, Pitch> = HashMap::new();
    let mut frames = Vec::new();
    let mut current_tick = 0u32;

    for event in &smf.tracks[0] {
        current_tick += event.delta.as_int();

        match event.kind {
            TrackEventKind::Midi { message: MidiMessage::NoteOn { key, vel }, .. } if vel > 0 => {
                active_notes.insert(key.as_int(), Pitch::from(key.as_int()));
            }
            TrackEventKind::Midi { message: MidiMessage::NoteOff { key, .. }, .. } => {
                active_notes.remove(&key.as_int());
            }
            _ => {}
        }

        frames.push(MidiFrame {
            tick: current_tick,
            pitches: active_notes.values().cloned().collect(),
        });
    }

    frames
}
```

## Further Analysis

Once you have frames of `Pitch` values, you can:

- Convert to `PcSet` for set-theoretic analysis
- Build `Voicing` objects to analyze voice leading
- Use the `chord_name/naming_heuristics` module to infer chord names
- Apply `geometry/symmetry` analysis to find transpositional patterns
