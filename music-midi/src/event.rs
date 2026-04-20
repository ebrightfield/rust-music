// REQ-O1, O2, O10: event types, default PPQ

/// Absolute tick position within an SMF track at the current PPQ resolution.
///
/// Tick 0 is the beginning of the piece. Use `ConvertCtx::rescale_ticks` to
/// convert from `music` internal ticks (128 per whole note) to MIDI ticks.
pub type AbsoluteTicks = u64;

/// Default pulses-per-quarter-note used by `SmfBuilder` when none is specified.
///
/// 480 PPQ is a common DAW standard and divides cleanly by all standard note
/// durations (whole through 128th) when the internal resolution is 32 per quarter.
pub const DEFAULT_PPQ: u16 = 480;

/// A single MIDI event with an absolute tick position and channel.
#[derive(Clone, Debug, PartialEq)]
pub struct MidiEvent {
    /// Absolute tick position of this event within its track.
    pub time: AbsoluteTicks,
    /// MIDI channel (0–15).
    pub channel: u8,
    /// The MIDI message payload.
    pub message: MidiMessage,
}

/// The payload of a [`MidiEvent`].
///
/// Variants map 1-to-1 to MIDI wire messages. The internal variants
/// `TempoBpm`, `TimeSignature`, `TrackName`, and `EndOfTrack` are meta
/// events emitted into the conductor track by `SmfBuilder::build` and
/// stripped from per-instrument tracks.
#[derive(Clone, Debug, PartialEq)]
pub enum MidiMessage {
    /// Note on (key = MIDI note 0–127, velocity = 0–127).
    NoteOn  { key: u8, velocity: u8 },
    /// Note off (key = MIDI note 0–127, velocity = release velocity).
    NoteOff { key: u8, velocity: u8 },
    /// Program / instrument change (0–127).
    ProgramChange(u8),
    /// General-purpose control change (controller 0–127, value 0–127).
    ControlChange { controller: u8, value: u8 },
    /// Internal: tempo change in beats-per-minute. Converted to a MIDI
    /// `Tempo` meta event (µs/beat) in the `SmfBuilder` conductor track.
    TempoBpm(f32),
    /// Internal: time signature. `denominator_pow2` is log₂ of the denominator
    /// (e.g. `2` for 4/4, `3` for 6/8).
    TimeSignature { numerator: u8, denominator_pow2: u8 },
    /// Internal: track name meta event.
    TrackName(String),
    /// Internal: end-of-track meta event.
    EndOfTrack,
}
