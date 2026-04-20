// REQ-O1..O6: ToMidiEvents trait + ConvertCtx

use crate::{event::{AbsoluteTicks, MidiEvent}, error::MidiConversionError,
            tempo::TempoSource, dynamics::VelocityPolicy};
use music::notation::rhythm::meter::Meter;

/// Pitch and fretboard converters.
pub mod pitch;
/// Chord voicing converters.
pub mod voicing;
/// Rhythmic notation converters (tied notes, tuplets).
pub mod rhythm;
/// Melodic event sequence converters.
pub mod melody;

// [AMEND-A] `score` requires the `music/lilypond` feature; gate at submodule decl.
/// LilypondScore converter. Requires the `score` feature (implies `music/lilypond`).
#[cfg(feature = "score")] pub mod score;

/// Shared context for all MIDI conversion operations.
///
/// `ConvertCtx` carries the PPQ resolution, tempo source, velocity policy,
/// optional meter, and instrument-name lookup needed to convert any
/// [`ToMidiEvents`] implementor into a stream of [`MidiEvent`]s.
///
/// Construct via [`ConvertCtx::new`]; it validates that `ppq` is a non-zero
/// multiple of 32 before allowing use.
pub struct ConvertCtx<'a> {
    /// Pulses-per-quarter-note (must be a non-zero multiple of 32).
    pub ppq: u16,
    /// Tempo source for tick-to-seconds mapping.
    pub tempo: &'a dyn TempoSource,
    /// Velocity assignment policy for NoteOn events.
    pub velocity: VelocityPolicy,
    /// Optional time-signature context (used for `TimeSignature` meta events).
    pub meter: Option<&'a Meter>,
    /// Maps instrument name strings to General MIDI program numbers (0–127).
    pub instrument_lookup: &'a dyn Fn(&str) -> u8,
}

impl<'a> ConvertCtx<'a> {
    /// Construct a new conversion context.
    ///
    /// Returns [`MidiConversionError::InvalidPpq`] if `ppq` is 0 or not a
    /// multiple of 32.  Valid values include 32, 64, 96, 128, 160, 192, 224,
    /// 256, 288, 320, 352, 384, 416, 448, 480, 512, 960, etc.
    pub fn new(
        ppq: u16,
        tempo: &'a dyn TempoSource,
        velocity: VelocityPolicy,
        meter: Option<&'a Meter>,
        instrument_lookup: &'a dyn Fn(&str) -> u8,
    ) -> Result<Self, MidiConversionError> {
        // REQ-O9: multiple-of-32 invariant
        if ppq == 0 || ppq % 32 != 0 {
            return Err(MidiConversionError::InvalidPpq(ppq));   // REQ-X7
        }
        Ok(Self { ppq, tempo, velocity, meter, instrument_lookup })
    }

    /// Rescale a `music` internal tick count to MIDI ticks at the current PPQ.
    ///
    /// The `music` crate uses 128 internal ticks per whole note (32 per quarter).
    /// At PPQ=480, one quarter note is 32 internal ticks → 480 MIDI ticks.
    #[inline]
    pub(crate) fn rescale_ticks(&self, music_ticks: u32) -> u64 {
        (music_ticks as u64) * (self.ppq as u64) / 32
    }
}

/// Convert a `music` type into a stream of [`MidiEvent`]s.
///
/// Implementations exist for:
/// - [`Pitch`](music::note::pitch::Pitch) — one NoteOn/NoteOff pair
/// - [`Voicing`](music::note_collections::voicing::Voicing) — parallel NoteOn/NoteOff pairs
/// - [`[MelodicEvent]`](music::melody::sequencer::MelodicEvent) — note sequence with ties
/// - [`[RhythmicNotatedEvent]`](music::notation::rhythm::RhythmicNotatedEvent) — rhythm + tuplets
/// - [`LilypondScore`](music::notation::lilypond::document::score::LilypondScore) — full score
///   (requires the `score` feature)
pub trait ToMidiEvents {
    /// Append MIDI events for `self` starting at `base_tick` on `channel`.
    ///
    /// Returns the absolute tick of the last event emitted (the "end tick").
    /// Implementations must push [`MidiEvent`]s in ascending tick order.
    fn append_midi(
        &self,
        base_tick: AbsoluteTicks,
        channel: u8,
        ctx: &ConvertCtx<'_>,
        out: &mut Vec<MidiEvent>,
    ) -> Result<AbsoluteTicks, MidiConversionError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dynamics::VelocityPolicy, tempo::StaticTempoMap};

    fn instr(_s: &str) -> u8 { 0 }

    #[test]
    fn rescale_quarter_note_at_480() {
        let t = StaticTempoMap::constant(120.0);
        let ctx = ConvertCtx::new(480, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();
        assert_eq!(ctx.rescale_ticks(32), 480);   // quarter note = 32 music ticks = 480 MIDI ticks
        assert_eq!(ctx.rescale_ticks(128), 1920); // whole note
    }
}
