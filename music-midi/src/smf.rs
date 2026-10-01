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

use crate::{
    convert::{ConvertCtx, ToMidiEvents},
    dynamics::VelocityPolicy,
    error::MidiConversionError,
    event::{AbsoluteTicks, MidiEvent, MidiMessage, DEFAULT_PPQ},
    tempo::{StaticTempoMap, TempoSource},
};
use midly::{
    num::{u15, u24, u28, u4, u7},
    Format, Header, MetaMessage, MidiMessage as MM, Smf, Timing, Track, TrackEvent, TrackEventKind,
};
use music::notation::rhythm::meter::{Meter, MeterDenominator};

/// Owning, leak-free wrapper produced by [`SmfBuilder::build`].
///
/// Stores an event blueprint plus a `Vec<Vec<u8>>` of owned track-name
/// buffers. On [`OwnedSmf::as_smf`], [`OwnedSmf::write`], and
/// [`OwnedSmf::to_bytes`], a short-lived `midly::Smf<'a>` is reconstructed
/// whose `TrackName` slices borrow from `self.track_names`.
///
/// No `&'static [u8]` is ever handed out; the previous `.leak()` path is gone.
#[derive(Debug)]
pub struct OwnedSmf {
    header: Header,
    /// Per-instrument track names, owned. Index i corresponds to instrument
    /// track i; the conductor track has no TrackName meta in our output.
    track_names: Vec<Vec<u8>>,
    /// Blueprint conductor track (tempo + time-signature metas).
    conductor_events: Vec<OwnedTrackEvent>,
    /// Blueprint per-instrument track bodies.
    instrument_events: Vec<Vec<OwnedTrackEvent>>,
}

/// Owned, lifetime-free mirror of `midly::TrackEvent` payload.
/// Every variant uses copy-only data; no borrows.
#[derive(Clone, Debug)]
enum OwnedTrackEvent {
    TempoMeta {
        delta: u32,
        usec_per_qn: u32,
    },
    TimeSignatureMeta {
        delta: u32,
        num: u8,
        denom_log2: u8,
        cpc: u8,
        tpq: u8,
    },
    EndOfTrack {
        delta: u32,
    },
    MidiMsg {
        delta: u32,
        channel: u8,
        msg: OwnedMidiMsg,
    },
}

#[derive(Clone, Debug)]
enum OwnedMidiMsg {
    NoteOn { key: u8, vel: u8 },
    NoteOff { key: u8, vel: u8 },
    ProgramChange(u8),
    Controller { controller: u8, value: u8 },
}

impl OwnedTrackEvent {
    /// Reconstruct a short-lived `TrackEvent<'a>`.
    fn as_track_event<'a>(&self) -> TrackEvent<'a> {
        match *self {
            OwnedTrackEvent::TempoMeta { delta, usec_per_qn } => TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::new(usec_per_qn))),
            },
            OwnedTrackEvent::TimeSignatureMeta {
                delta,
                num,
                denom_log2,
                cpc,
                tpq,
            } => TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Meta(MetaMessage::TimeSignature(num, denom_log2, cpc, tpq)),
            },
            OwnedTrackEvent::EndOfTrack { delta } => TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            },
            OwnedTrackEvent::MidiMsg {
                delta,
                channel,
                ref msg,
            } => {
                let message = match *msg {
                    OwnedMidiMsg::NoteOn { key, vel } => MM::NoteOn {
                        key: u7::new(key),
                        vel: u7::new(vel),
                    },
                    OwnedMidiMsg::NoteOff { key, vel } => MM::NoteOff {
                        key: u7::new(key),
                        vel: u7::new(vel),
                    },
                    OwnedMidiMsg::ProgramChange(p) => MM::ProgramChange {
                        program: u7::new(p),
                    },
                    OwnedMidiMsg::Controller { controller, value } => MM::Controller {
                        controller: u7::new(controller),
                        value: u7::new(value),
                    },
                };
                TrackEvent {
                    delta: u28::from(delta),
                    kind: TrackEventKind::Midi {
                        channel: u4::from(channel),
                        message,
                    },
                }
            }
        }
    }
}

impl OwnedSmf {
    /// Build a short-lived `midly::Smf<'a>` whose `TrackName` slices borrow
    /// from `self.track_names`.
    ///
    /// REQ-O2: no reallocation of the backing buffers — only the thin
    /// `Vec<TrackEvent<'a>>` wrappers are assembled.
    pub fn as_smf<'a>(&'a self) -> Smf<'a> {
        let mut tracks: Vec<Track<'a>> = Vec::with_capacity(self.instrument_events.len() + 1);

        // Conductor track (index 0) — no TrackName spliced.
        let conductor: Vec<TrackEvent<'a>> = self
            .conductor_events
            .iter()
            .map(OwnedTrackEvent::as_track_event)
            .collect();
        tracks.push(conductor);

        // Instrument tracks — prepend a TrackName meta that borrows from self.
        for (i, body) in self.instrument_events.iter().enumerate() {
            let mut track: Vec<TrackEvent<'a>> = Vec::with_capacity(body.len() + 1);
            track.push(TrackEvent {
                delta: u28::from(0u32),
                kind: TrackEventKind::Meta(MetaMessage::TrackName(&self.track_names[i])),
            });
            for ev in body {
                track.push(ev.as_track_event());
            }
            tracks.push(track);
        }

        Smf {
            header: self.header,
            tracks,
        }
    }

    /// Number of owned track-name buffers. Used only by the leak-regression test.
    #[doc(hidden)]
    pub fn track_name_count(&self) -> usize {
        self.track_names.len()
    }

    /// Serialize the SMF to `out`. REQ-O1: no leaks, no 'static slices.
    pub fn write(&self, out: &mut Vec<u8>) -> Result<(), MidiConversionError> {
        self.as_smf()
            .write(out)
            .map_err(|e| MidiConversionError::Smf(e.to_string()))
    }

    /// Convenience: serialize to a fresh `Vec<u8>`.
    pub fn to_bytes(&self) -> Result<Vec<u8>, MidiConversionError> {
        let mut v = Vec::new();
        self.write(&mut v)?;
        Ok(v)
    }
}

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
    meter: Option<Meter>,
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
            meter: None,
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

    /// Set the time signature for the conductor track. If not called, no
    /// `TimeSignature` meta is emitted (current behavior preserved).
    pub fn meter(mut self, meter: Meter) -> Self {
        self.meter = Some(meter);
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
        // `effective_ppq` is either DEFAULT_PPQ (a constant known-valid value) or a
        // previously validated `self.ppq`. Propagate any `InvalidPpq` via `?` rather
        // than panicking, so a future regression in that calculation cannot turn into
        // a release-build panic.
        let ctx = ConvertCtx::new(
            effective_ppq,
            tempo_ref,
            VelocityPolicy::Fixed(80),
            None,
            &instr,
        )?;
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
    /// Returns an [`OwnedSmf`] that owns all buffers — no `.leak()`.
    pub fn build(self) -> Result<OwnedSmf, MidiConversionError> {
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
        // REQ-O18: enforce ≤ 16 instrument tracks
        if self.tracks.len() > 16 {
            return Err(MidiConversionError::TooManyVoices(self.tracks.len()));
        }

        let header = Header {
            format: Format::Parallel,
            timing: Timing::Metrical(u15::new(self.ppq)),
        };

        // Build conductor blueprint: tempo metas → [optional TimeSignature] → EOT.
        let mut conductor_events: Vec<OwnedTrackEvent> = Vec::new();
        let mut last_tick: u64 = 0;
        for (tick, bpm) in cps {
            // W8: guard against tick-underflow (unordered change points) and
            // oversize deltas (u28 max is 0x0FFF_FFFF). midly's u28::from is
            // lossy-truncating, so we validate explicitly and surface a Smf
            // error instead of silently corrupting the stream.
            let delta_u64 = tick.checked_sub(last_tick).ok_or_else(|| {
                MidiConversionError::Smf(format!(
                    "tempo change points are not monotonically ordered: \
                     tick {} follows tick {}",
                    tick, last_tick
                ))
            })?;
            let delta = u32::try_from(delta_u64)
                .ok()
                .and_then(u28::try_from)
                .ok_or_else(|| {
                    MidiConversionError::Smf(format!(
                        "tempo tick delta {} exceeds SMF u28 max ({}); \
                         input PPQ or tick positions are out of range",
                        delta_u64,
                        u28::max_value().as_int()
                    ))
                })?
                .as_int();
            last_tick = *tick;
            let usec_per_qn = (60_000_000.0 / *bpm as f64).round() as u32;
            conductor_events.push(OwnedTrackEvent::TempoMeta { delta, usec_per_qn });
        }
        // REQ-O15 (D2): splice TimeSignature AFTER tempo metas, BEFORE EOT.
        if let Some(m) = &self.meter {
            conductor_events.push(OwnedTrackEvent::TimeSignatureMeta {
                delta: 0,
                num: m.num_beats as u8,
                denom_log2: meter_denom_to_log2(m.denominator),
                cpc: 24,
                tpq: 8,
            });
        }
        conductor_events.push(OwnedTrackEvent::EndOfTrack { delta: 0 });

        // Build instrument blueprints, in parallel with track_names.
        let mut track_names: Vec<Vec<u8>> = Vec::with_capacity(self.tracks.len());
        let mut instrument_events: Vec<Vec<OwnedTrackEvent>> =
            Vec::with_capacity(self.tracks.len());

        for pt in self.tracks {
            track_names.push(pt.name.into_bytes());
            instrument_events.push(build_instrument_blueprint(pt.channel, pt.events)?);
        }

        Ok(OwnedSmf {
            header,
            track_names,
            conductor_events,
            instrument_events,
        })
    }
}

impl Default for SmfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Translate a `MidiEvent` stream into an `OwnedTrackEvent` blueprint body.
/// Meta events (TrackName, TimeSignature, TempoBpm, EndOfTrack) that reach
/// here are dropped — they belong in the conductor track.
fn build_instrument_blueprint(
    channel: u8,
    mut events: Vec<MidiEvent>,
) -> Result<Vec<OwnedTrackEvent>, MidiConversionError> {
    events.sort_by_key(|e| e.time);
    let mut out: Vec<OwnedTrackEvent> = Vec::with_capacity(events.len() + 1);
    let mut last: AbsoluteTicks = 0;
    for e in events {
        // W8: guard against tick-underflow (defensive — events are sorted
        // above, so this should be unreachable) and oversize deltas (u28 max
        // is 0x0FFF_FFFF). midly's u28::from is lossy-truncating, so we
        // validate explicitly rather than silently corrupt the stream.
        let delta_u64 = e.time.checked_sub(last).ok_or_else(|| {
            MidiConversionError::Smf(format!(
                "instrument event ticks are not monotonically ordered after \
                 sort: tick {} follows tick {}",
                e.time, last
            ))
        })?;
        let delta = u32::try_from(delta_u64)
            .ok()
            .and_then(u28::try_from)
            .ok_or_else(|| {
                MidiConversionError::Smf(format!(
                    "instrument tick delta {} exceeds SMF u28 max ({}); \
                     input PPQ or event positions are out of range",
                    delta_u64,
                    u28::max_value().as_int()
                ))
            })?
            .as_int();
        last = e.time;
        let msg = match e.message {
            MidiMessage::NoteOn { key, velocity } => OwnedMidiMsg::NoteOn { key, vel: velocity },
            MidiMessage::NoteOff { key, velocity } => OwnedMidiMsg::NoteOff { key, vel: velocity },
            MidiMessage::ProgramChange(p) => OwnedMidiMsg::ProgramChange(p),
            MidiMessage::ControlChange { controller, value } => {
                OwnedMidiMsg::Controller { controller, value }
            }
            // Meta variants belong in the conductor track only.
            MidiMessage::TimeSignature { .. }
            | MidiMessage::TrackName(_)
            | MidiMessage::TempoBpm(_)
            | MidiMessage::EndOfTrack => continue,
        };
        out.push(OwnedTrackEvent::MidiMsg {
            delta,
            channel,
            msg,
        });
    }
    out.push(OwnedTrackEvent::EndOfTrack { delta: 0 });
    Ok(out)
}

/// Map the `music` crate's `MeterDenominator` enum to MIDI's log₂(denominator) byte.
///
/// MIDI `MetaMessage::TimeSignature` encodes its second field as `log₂(denominator)`.
/// Valid denominators are powers of two: 1, 2, 4, 8, 16 → 0, 1, 2, 3, 4.
fn meter_denom_to_log2(d: MeterDenominator) -> u8 {
    match d {
        MeterDenominator::One => 0,
        MeterDenominator::Two => 1,
        MeterDenominator::Four => 2,
        MeterDenominator::Eight => 3,
        MeterDenominator::Sixteen => 4,
    }
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
    let owned = SmfBuilder::new()
        .ppq(ppq)
        .tempo(StaticTempoMap::constant(bpm))
        .add_track("pitch", 0, pitch)?
        .build()?;
    owned.to_bytes()
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
    let owned = SmfBuilder::new()
        .ppq(ppq)
        .tempo(StaticTempoMap::constant(bpm))
        .add_track("melody", 0, melody)?
        .build()?;
    owned.to_bytes()
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
    let owned = SmfBuilder::new()
        .ppq(ppq)
        .tempo(StaticTempoMap::constant(bpm))
        .add_track("score", 0, score)?
        .build()?;
    owned.to_bytes()
}
