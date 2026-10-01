// REQ-O1, O3: MelodicEvent stream uses its Duration directly
use super::{ConvertCtx, ToMidiEvents};
use crate::{
    error::MidiConversionError,
    event::{AbsoluteTicks, MidiEvent, MidiMessage},
};
use music::melody::sequencer::MelodicEvent;

impl ToMidiEvents for [MelodicEvent] {
    fn append_midi(
        &self,
        mut base_tick: AbsoluteTicks,
        channel: u8,
        ctx: &ConvertCtx<'_>,
        out: &mut Vec<MidiEvent>,
    ) -> Result<AbsoluteTicks, MidiConversionError> {
        let mut i = 0;
        while i < self.len() {
            let cur = &self[i];
            let mut j = i + 1;
            // [AMEND-E] explicit `as u32`; usize -> u32 has no Into impl.
            let mut dur_ticks: u64 = ctx.rescale_ticks(cur.duration.ticks() as u32);
            // REQ-O4, X8: extend across tied MelodicEvents of the same pitch
            while j < self.len() && self[j].tied && self[j].pitch.midi_note == cur.pitch.midi_note {
                dur_ticks += ctx.rescale_ticks(self[j].duration.ticks() as u32);
                j += 1;
            }
            if cur.pitch.midi_note > 127 {
                return Err(MidiConversionError::PitchOutOfRange(cur.pitch.midi_note));
            }
            let vel = ctx.velocity.velocity_no_event();
            out.push(MidiEvent {
                time: base_tick,
                channel,
                message: MidiMessage::NoteOn {
                    key: cur.pitch.midi_note,
                    velocity: vel,
                },
            });
            out.push(MidiEvent {
                time: base_tick + dur_ticks,
                channel,
                message: MidiMessage::NoteOff {
                    key: cur.pitch.midi_note,
                    velocity: 64,
                },
            });
            base_tick += dur_ticks;
            i = j;
        }
        Ok(base_tick)
    }
}
