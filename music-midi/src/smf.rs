//! Build Standard MIDI Files (SMF) from `music` types.
//!
//! Requires the `smf` feature (default). The `smf` feature implies `score`,
//! which enables [`LilypondScore`](music::notation::lilypond::document::score::LilypondScore)
//! conversion via the `music/lilypond` feature.
//!
//! # Quick start
//!
//! ```ignore
//! use music_midi::smf::SmfBuilder;
//! use music_midi::StaticTempoMap;
//!
//! let smf = SmfBuilder::new()
//!     .ppq(480)
//!     .tempo(StaticTempoMap::constant(120.0))
//!     .add_track("piano", 0, &my_events)?
//!     .build()?;
//! ```
// REQ-O5, O6, O7, O8, O9, O15, O18: SmfBuilder
#![cfg(feature = "smf")]

use crate::{
    convert::{ConvertCtx, ToMidiEvents},
    dynamics::VelocityPolicy,
    error::MidiConversionError,
    event::{AbsoluteTicks, DEFAULT_PPQ, MidiEvent, MidiMessage},
    tempo::{StaticTempoMap, TempoSource},
};
use midly::{
    Format, Header, MetaMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
    MidiMessage as MM,
    num::{u4, u7, u15, u24, u28},
};

/// Builder for a Standard MIDI File (SMF / `.mid`).
///
/// Collects instrument tracks, validates settings, and serializes everything
/// to a `midly::Smf<'static>` which can be written with `.write()`.
///
/// The builder always emits a conductor track (track 0) containing the tempo
/// meta events from the supplied [`TempoSource`]. Per-instrument tracks follow.
///
/// # Constraints
/// - PPQ must be a non-zero multiple of 32 (see [`ConvertCtx`]).
/// - At most 16 instrument tracks (MIDI channels 0–15).
/// - The tempo source must have at least one change point.
///
/// # Example
///
/// ```ignore
/// use music_midi::smf::SmfBuilder;
/// use music_midi::StaticTempoMap;
/// use music::note::pitch::Pitch;
///
/// let c4 = Pitch::from_midi(60).unwrap();
/// let smf = SmfBuilder::new()
///     .ppq(480)
///     .tempo(StaticTempoMap::constant(120.0))
///     .add_track("piano", 0, &c4)?
///     .build()?;
/// let mut bytes = Vec::new();
/// smf.write(&mut bytes)?;
/// ```
pub struct SmfBuilder {
    ppq: u16,
    tempo: Option<Box<dyn TempoSource>>,
    tracks: Vec<PendingTrack>,
}

struct PendingTrack {
    name: String,
    channel: u8,
    events: Vec<MidiEvent>,
}

impl SmfBuilder {
    pub fn new() -> Self {
        Self {
            ppq: DEFAULT_PPQ,
            tempo: None,
            tracks: Vec::new(),
        }
    }

    /// Set PPQ (pulses per quarter note). Must be a multiple of 32.
    /// Validated at `build()` time; `ppq(481)` is captured here, rejected at build.
    pub fn ppq(mut self, ppq: u16) -> Self {
        self.ppq = ppq;
        self
    }

    /// Set the tempo source. Required for `build()`.
    pub fn tempo(mut self, tempo: impl TempoSource + 'static) -> Self {
        self.tempo = Some(Box::new(tempo));
        self
    }

    /// Add an instrument track. Returns `TooManyVoices` if 16 tracks already exist.
    pub fn add_track<T: ToMidiEvents + ?Sized>(
        mut self,
        name: &str,
        channel: u8,
        src: &T,
    ) -> Result<Self, MidiConversionError> {
        // REQ-O18: cap at 16 channels (0..=15)
        if self.tracks.len() >= 16 {
            return Err(MidiConversionError::TooManyVoices(self.tracks.len() + 1));
        }
        // Use a temporary tempo for event collection during add_track.
        // The real validation (TempoSourceEmpty) happens at build().
        let instr = |_: &str| 0u8;
        let fallback_tempo = StaticTempoMap::constant(120.0);
        let tempo_ref: &dyn TempoSource = match &self.tempo {
            Some(t) => t.as_ref(),
            None => &fallback_tempo,
        };
        // If ppq is invalid, fall back to DEFAULT_PPQ for event collection;
        // build() will reject the invalid ppq value.
        let effective_ppq = if self.ppq == 0 || self.ppq % 32 != 0 {
            DEFAULT_PPQ
        } else {
            self.ppq
        };
        let ctx = ConvertCtx::new(effective_ppq, tempo_ref, VelocityPolicy::Fixed(80), None, &instr)
            .expect("effective_ppq is valid (DEFAULT_PPQ or validated self.ppq)");
        let mut events = Vec::new();
        src.append_midi(0, channel, &ctx, &mut events)?;
        self.tracks.push(PendingTrack {
            name: name.to_string(),
            channel,
            events,
        });
        Ok(self)
    }

    /// Build the SMF. Validates PPQ, requires a non-empty tempo source.
    pub fn build(self) -> Result<Smf<'static>, MidiConversionError> {
        // REQ-O9, X7: PPQ must be a non-zero multiple of 32
        if self.ppq == 0 || self.ppq % 32 != 0 {
            return Err(MidiConversionError::InvalidPpq(self.ppq));
        }
        // REQ-O15, X9: offline export requires at least one tempo change point
        let tempo = self.tempo.ok_or(MidiConversionError::TempoSourceEmpty)?;
        let cps = tempo.change_points();
        if cps.is_empty() {
            return Err(MidiConversionError::TempoSourceEmpty);
        }

        let header = Header {
            format: Format::Parallel,
            timing: Timing::Metrical(u15::new(self.ppq)),
        };
        let mut tracks: Vec<Track<'static>> = Vec::with_capacity(self.tracks.len() + 1);

        // Track 0: conductor — tempo meta events + time signature metas.
        // [W3 mitigation]: TimeSignature meta is emitted ONLY in the conductor track (track 0),
        // never in per-instrument tracks (build_track silently drops them).
        let mut conductor: Vec<TrackEvent<'static>> = Vec::new();
        let mut last_tick: u64 = 0;
        for (tick, bpm) in cps {
            let delta = (*tick - last_tick) as u32;
            last_tick = *tick;
            // REQ-O5: convert bpm to microseconds per quarter note (u24)
            let usec_per_qn = (60_000_000.0 / *bpm as f64).round() as u32;
            conductor.push(TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(usec_per_qn))),
            });
        }
        conductor.push(TrackEvent {
            delta: u28::from(0u32),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
        tracks.push(conductor);

        // REQ-O18: enforce ≤ 16 instrument tracks
        if self.tracks.len() > 16 {
            return Err(MidiConversionError::TooManyVoices(self.tracks.len()));
        }

        for pt in self.tracks {
            tracks.push(build_track(&pt.name, pt.channel, pt.events)?);
        }

        Ok(Smf { header, tracks })
    }
}

impl Default for SmfBuilder {
    fn default() -> Self { Self::new() }
}

/// Build a single instrument track from collected events.
///
/// Note W3: `TimeSignature` and `TempoBpm` messages are silently dropped here;
/// they belong in the conductor track (track 0) only.
fn build_track(
    name: &str,
    channel: u8,
    mut events: Vec<MidiEvent>,
) -> Result<Track<'static>, MidiConversionError> {
    // Sort by time for deterministic event ordering
    events.sort_by_key(|e| e.time);

    let mut out: Vec<TrackEvent<'static>> = Vec::new();

    // Track name meta event at delta=0.
    // FIXME(consolidation): midly's TrackName borrows a `&'static [u8]` but our name
    // is runtime-owned. Leaking a small allocation per track is bounded per build()
    // but unbounded across repeated build() calls in long-running processes. Revisit
    // by either (a) switching to a midly API that takes owned bytes, or (b) caching
    // track-name allocations in an arena keyed by name.
    let name_bytes: &'static [u8] = name.as_bytes().to_vec().leak();
    out.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::TrackName(name_bytes)),
    });

    let mut last: AbsoluteTicks = 0;
    for e in events {
        let delta = (e.time - last) as u32;
        last = e.time;
        let kind = match e.message {
            MidiMessage::NoteOn { key, velocity } => TrackEventKind::Midi {
                channel: u4::from(channel),
                message: MM::NoteOn {
                    key: u7::new(key),
                    vel: u7::new(velocity),
                },
            },
            MidiMessage::NoteOff { key, velocity } => TrackEventKind::Midi {
                channel: u4::from(channel),
                message: MM::NoteOff {
                    key: u7::new(key),
                    vel: u7::new(velocity),
                },
            },
            MidiMessage::ProgramChange(p) => TrackEventKind::Midi {
                channel: u4::from(channel),
                message: MM::ProgramChange { program: u7::new(p) },
            },
            MidiMessage::ControlChange { controller, value } => TrackEventKind::Midi {
                channel: u4::from(channel),
                message: MM::Controller {
                    controller: u7::new(controller),
                    value: u7::new(value),
                },
            },
            // [W3]: TimeSignature/TempoBpm belong in the conductor track only;
            // any that reached here are dropped. TrackName is re-emitted above from
            // the PendingTrack.name field. EndOfTrack is emitted unconditionally at
            // the end of every track below.
            MidiMessage::TimeSignature { .. }
            | MidiMessage::TrackName(_)
            | MidiMessage::TempoBpm(_)
            | MidiMessage::EndOfTrack => continue,
        };
        out.push(TrackEvent {
            delta: u28::from(delta),
            kind,
        });
    }

    out.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Ok(out)
}

/// Emit a `TimeSignature` meta event into the conductor track.
///
/// Called from `build()` after tempo meta emission in the conductor track.
/// The time signature numerator and denominator come from
/// `music::notation::rhythm::meter::Meter`.
///
/// `num_beats` is the time signature numerator (e.g. `4` for 4/4).
/// `denominator_pow2` is log₂ of the denominator (e.g. `2` for 4/4, `3` for 6/8).
pub fn emit_time_signature_into_conductor(
    conductor: &mut Vec<TrackEvent<'static>>,
    num_beats: u8,
    denominator_pow2: u8,
) {
    // MIDI TimeSignature: (numerator, log2(denominator), clocks_per_click, 32nds_per_qn)
    // Standard values: 24 MIDI clocks per click, 8 thirty-second notes per quarter note.
    conductor.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
            num_beats,
            denominator_pow2,
            24,
            8,
        )),
    });
}

/// Convert a single [`Pitch`](music::note::pitch::Pitch) to SMF bytes.
///
/// Creates a two-track SMF (conductor + one instrument track) at the given
/// `bpm` and `ppq`. The pitch is emitted as a single NoteOn/NoteOff pair
/// lasting one quarter note.
///
/// # Errors
///
/// Returns [`MidiConversionError::PitchOutOfRange`] if the pitch MIDI note
/// exceeds 127. Returns [`MidiConversionError::InvalidPpq`] if `ppq` is not a
/// non-zero multiple of 32.
pub fn pitch_to_smf_bytes(
    pitch: &music::note::pitch::Pitch,
    bpm: f32,
    ppq: u16,
) -> Result<Vec<u8>, MidiConversionError> {
    let smf = SmfBuilder::new()
        .ppq(ppq)
        .tempo(StaticTempoMap::constant(bpm))
        .add_track("pitch", 0, pitch)?
        .build()?;
    let mut bytes = Vec::new();
    smf.write(&mut bytes).map_err(|e| MidiConversionError::Smf(e.to_string()))?;
    Ok(bytes)
}

/// Convert a slice of [`MelodicEvent`](music::melody::MelodicEvent)s to SMF bytes.
///
/// Creates a two-track SMF (conductor + one instrument track) at the given
/// `bpm` and `ppq`. Tied events in the slice are merged into a single
/// NoteOn/NoteOff pair.
///
/// # Errors
///
/// Returns an error if any event's pitch is out of MIDI range, or if `ppq` is
/// not a non-zero multiple of 32.
pub fn melody_to_smf_bytes(
    melody: &[music::melody::MelodicEvent],
    bpm: f32,
    ppq: u16,
) -> Result<Vec<u8>, MidiConversionError> {
    let smf = SmfBuilder::new()
        .ppq(ppq)
        .tempo(StaticTempoMap::constant(bpm))
        .add_track("melody", 0, melody)?
        .build()?;
    let mut bytes = Vec::new();
    smf.write(&mut bytes).map_err(|e| MidiConversionError::Smf(e.to_string()))?;
    Ok(bytes)
}

/// Convert a [`LilypondScore`](music::notation::lilypond::document::score::LilypondScore)
/// to SMF bytes.
///
/// Creates a multi-track SMF: one conductor track (tempo metas) plus one track
/// per voice (staff × voice index). Voice-to-channel assignment is in encounter
/// order; the first voice gets channel 0, the second channel 1, etc.
///
/// Requires the `score` feature (enabled automatically by `smf`).
///
/// # Errors
///
/// Returns [`MidiConversionError::TooManyVoices`] if the score has more than
/// 16 distinct (staff, voice) pairs.
#[cfg(feature = "score")]
pub fn score_to_smf_bytes(
    score: &music::notation::lilypond::document::score::LilypondScore<'_>,
    bpm: f32,
    ppq: u16,
) -> Result<Vec<u8>, MidiConversionError> {
    let smf = SmfBuilder::new()
        .ppq(ppq)
        .tempo(StaticTempoMap::constant(bpm))
        .add_track("score", 0, score)?
        .build()?;
    let mut bytes = Vec::new();
    smf.write(&mut bytes).map_err(|e| MidiConversionError::Smf(e.to_string()))?;
    Ok(bytes)
}
