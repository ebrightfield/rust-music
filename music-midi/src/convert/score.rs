// REQ-O3, O6: LilypondScore -> multi-track events via iter_events
// [AMEND-A] entire file is gated by `cfg(feature = "score")` at submodule declaration.

use super::{ConvertCtx, ToMidiEvents};
use crate::{event::{AbsoluteTicks, MidiEvent}, error::MidiConversionError};
use music::notation::lilypond::document::score::LilypondScore;
use music::notation::rhythm::flatten::iter_events;

/// Channel allocation strategy: globally unique (staff, voice) index → MIDI channel 0..=15.
///
/// We collect all (staff, voice) pairs in encounter order (same order as iter_events),
/// assign each a unique global index, and derive the MIDI channel from that index.
/// If more than 16 unique (staff, voice) pairs exist, we return `TooManyVoices` early.
fn derive_channel(global_voice_idx: usize) -> u8 {
    global_voice_idx as u8
}

impl<'a> ToMidiEvents for LilypondScore<'a> {
    fn append_midi(&self, _base_tick: AbsoluteTicks, _channel: u8, ctx: &ConvertCtx<'_>,
                   out: &mut Vec<MidiEvent>) -> Result<AbsoluteTicks, MidiConversionError> {
        // REQ-O18: count distinct (staff, voice) pairs; fail if > 16.
        // We need a two-pass or one-pass with known max. Use one pass, tracking max global idx.
        // Build a (staff, voice) -> global_idx mapping from the flat events.
        // We collect first, then emit — or we can compute the global_idx inline since
        // iter_events is ordered by (staff, voice).
        //
        // Strategy: iter_events yields events in stable (staff, voice) order within a pass.
        // We assign a global voice index by tracking the unique (staff, voice) pairs seen.

        // First, collect all flat events to count voice pairs (avoids double-scoring iterator).
        let flat_events: Vec<_> = iter_events(self).collect();

        // Determine global voice assignment: (staff, voice) → u8 channel.
        // Pairs appear in encounter order; assign incrementally.
        let mut voice_map: Vec<(usize, usize)> = Vec::new(); // ordered unique (staff, voice) pairs
        for fe in &flat_events {
            let pair = (fe.staff, fe.voice);
            if !voice_map.contains(&pair) {
                voice_map.push(pair);
            }
        }

        // REQ-O18: cap at 16 voices
        if voice_map.len() > 16 {
            return Err(MidiConversionError::TooManyVoices(voice_map.len()));
        }

        let mut end_tick: AbsoluteTicks = 0;
        for fe in &flat_events {
            let global_idx = voice_map.iter().position(|&p| p == (fe.staff, fe.voice))
                .unwrap_or_else(|| unreachable!("voice_map invariant: pair must be in map"));
            let ch = derive_channel(global_idx);
            let tick_end = fe.event.append_midi(fe.tick, ch, ctx, out)?;
            if tick_end > end_tick {
                end_tick = tick_end;
            }
        }

        Ok(end_tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dynamics::VelocityPolicy, tempo::StaticTempoMap};
    use music::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
    use music::notation::lilypond::document::staff::LilypondStaff;
    use music::notation::lilypond::staff_elements::LilypondVoiceElement;
    use music::notation::rhythm::{RhythmicNotatedEvent, duration::{Duration, DurationKind}};
    use music::note::pitch::Pitch;

    fn ctx() -> StaticTempoMap { StaticTempoMap::constant(120.0) }
    fn instr(_s: &str) -> u8 { 0 }

    #[test]
    fn score_single_note_emits_two_events() {
        let pitch = Pitch::from_midi(60).unwrap();
        let event = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Qtr, 0));
        let voice = vec![LilypondVoiceElement::Common(event)];
        let staff = LilypondStaff::new().add_voice(voice);
        let group = LilypondStaffGroup::new(vec![staff]);
        let score = LilypondScore::new().staff_group(group);

        let tm = ctx();
        let c = ConvertCtx::new(480, &tm, VelocityPolicy::Fixed(80), None, &instr).unwrap();
        let mut out = Vec::new();
        score.append_midi(0, 0, &c, &mut out).unwrap();
        // NoteOn + NoteOff = 2 events
        assert_eq!(out.len(), 2);
    }
}
