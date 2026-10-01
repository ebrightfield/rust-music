// REQ-O2: Voicing -> parallel NoteOns + NoteOffs
use super::{ConvertCtx, ToMidiEvents};
use crate::{
    error::MidiConversionError,
    event::{AbsoluteTicks, MidiEvent, MidiMessage},
};
use music::note_collections::voicing::Voicing;

impl ToMidiEvents for Voicing {
    fn append_midi(
        &self,
        base_tick: AbsoluteTicks,
        channel: u8,
        ctx: &ConvertCtx<'_>,
        out: &mut Vec<MidiEvent>,
    ) -> Result<AbsoluteTicks, MidiConversionError> {
        // REQ-O2: parallel NoteOns, then parallel NoteOffs at base + quarter
        let gate = ctx.ppq as u64;
        let vel = ctx.velocity.velocity_no_event();
        // Voicing derefs to Vec<Pitch>
        for pitch in self.iter() {
            if pitch.midi_note > 127 {
                return Err(MidiConversionError::PitchOutOfRange(pitch.midi_note));
            }
            out.push(MidiEvent {
                time: base_tick,
                channel,
                message: MidiMessage::NoteOn {
                    key: pitch.midi_note,
                    velocity: vel,
                },
            });
        }
        for pitch in self.iter() {
            out.push(MidiEvent {
                time: base_tick + gate,
                channel,
                message: MidiMessage::NoteOff {
                    key: pitch.midi_note,
                    velocity: 64,
                },
            });
        }
        Ok(base_tick + gate)
    }
}
