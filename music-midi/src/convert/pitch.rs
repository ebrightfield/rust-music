// REQ-O1: Pitch -> one NoteOn/NoteOff pair

use super::{ConvertCtx, ToMidiEvents};
use crate::{event::{AbsoluteTicks, MidiEvent, MidiMessage}, error::MidiConversionError,
            dynamics::VelocityPolicy};
use music::note::pitch::Pitch;
use music::fretboard::fretted_note::SoundedNote;

/// Duration in MIDI ticks to apply when a Pitch is emitted without a
/// wrapping rhythm event. Spec does not mandate a default; we pick a
/// quarter at the ctx's PPQ.
fn default_gate_ticks(ctx: &ConvertCtx<'_>) -> u64 {
    ctx.ppq as u64     // one quarter note
}

fn velocity_for(ctx: &ConvertCtx<'_>) -> u8 {
    // REQ-O12, P1
    match &ctx.velocity {
        VelocityPolicy::Fixed(v) => *v,
        VelocityPolicy::FromDynamic(d) => d.velocity(),
        VelocityPolicy::PerEvent(_) => 80,  // no RhythmicNotatedEvent in scope
    }
}

impl ToMidiEvents for Pitch {
    fn append_midi(
        &self,
        base_tick: AbsoluteTicks,
        channel: u8,
        ctx: &ConvertCtx<'_>,
        out: &mut Vec<MidiEvent>,
    ) -> Result<AbsoluteTicks, MidiConversionError> {
        // REQ-O1: validate range
        // SAFETY: Pitch::from_midi rejects >= 108 today; guard anticipates future range expansion
        if self.midi_note > 127 {
            return Err(MidiConversionError::PitchOutOfRange(self.midi_note));
        }
        let key = self.midi_note;
        let vel = velocity_for(ctx);
        let gate = default_gate_ticks(ctx);
        out.push(MidiEvent { time: base_tick, channel,
            message: MidiMessage::NoteOn { key, velocity: vel } });
        out.push(MidiEvent { time: base_tick + gate, channel,
            message: MidiMessage::NoteOff { key, velocity: 64 } });
        Ok(base_tick + gate)
    }
}

impl<'a> ToMidiEvents for SoundedNote<'a> {
    fn append_midi(
        &self,
        base_tick: AbsoluteTicks,
        channel: u8,
        ctx: &ConvertCtx<'_>,
        out: &mut Vec<MidiEvent>,
    ) -> Result<AbsoluteTicks, MidiConversionError> {
        // REQ-O1, P2: string info currently discarded
        self.pitch.append_midi(base_tick, channel, ctx, out)
    }
}
