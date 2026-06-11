//! Convert [`music`] crate types to MIDI.
//!
//! `music-midi` provides three backends, each gated by a Cargo feature:
//!
//! | Feature    | What it unlocks |
//! |------------|-----------------|
//! | `smf`      | [`smf::SmfBuilder`] — build Standard MIDI Files (SMF) from `music` types |
//! | `playback` | [`playback::MidiPlayer`] — realtime MIDI output via `midir` |
//! | `render`   | [`render::AudioRenderer`] — offline WAV rendering via `oxisynth` + SF2 |
//!
//! The default feature set is `["smf"]`.
//!
//! ## Quick start — single pitch to SMF bytes
//!
//! ```ignore
//! use music_midi::pitch_to_smf_bytes;
//! use music::note::{Note, pitch::Pitch};
//!
//! // Middle C, octave 4
//! let c4 = Pitch::try_new(Note::C, 4).unwrap();
//! let smf_bytes = pitch_to_smf_bytes(&c4, 120.0, 480).unwrap();
//! assert_eq!(&smf_bytes[..4], b"MThd");
//! ```
//!
//! ## Quick start — melody to SMF
//!
//! ```ignore
//! use music_midi::melody_to_smf_bytes;
//! use music::melody::sequencer::MelodicEvent;
//! use music::note::pitch::Pitch;
//! use music::notation::rhythm::duration::{Duration, DurationKind};
//!
//! let c4 = Pitch::from_midi(60).unwrap();
//! let events = vec![MelodicEvent::new(c4, Duration::QTR)];
//! let smf_bytes = melody_to_smf_bytes(&events, 120.0, 480).unwrap();
//! assert_eq!(&smf_bytes[..4], b"MThd");
//! ```

pub mod error;
pub mod event;
pub mod dynamics;
pub mod tempo;
pub mod convert;

// [AMEND-A] `convert::score` is the only converter that requires the lilypond
// feature on the `music` crate; gate it behind `score`.
#[cfg(feature = "smf")]      pub mod smf;
#[cfg(feature = "playback")] pub mod playback;
#[cfg(feature = "render")]   pub mod render;
#[cfg(feature = "render")]   pub mod soundfont;

pub use error::MidiConversionError;
pub use event::{MidiEvent, MidiMessage, AbsoluteTicks, DEFAULT_PPQ};
pub use tempo::{TempoSource, StaticTempoMap, BoxedTempoSource};
pub use dynamics::{Dynamic, VelocityPolicy};
pub use convert::{ToMidiEvents, ConvertCtx};

// REQ-O1: OwnedSmf + top-level SMF helper functions.
#[cfg(feature = "smf")]
pub use smf::{OwnedSmf, pitch_to_smf_bytes, melody_to_smf_bytes};

#[cfg(feature = "smf")]
#[cfg(feature = "score")]
pub use smf::score_to_smf_bytes;
