// REQ-O16: shared flatten::iter_events for music-midi and future music-engraver
// This module is gated behind the `lilypond` feature because it references LilypondScore.
#![cfg(feature = "lilypond")]

use crate::notation::lilypond::document::score::LilypondScore;
use crate::notation::rhythm::{RhythmicNotatedEvent, NotatedEvent};

pub type AbsoluteTicks = u64;
pub type StaffIdx = usize;
pub type VoiceIdx = usize;

pub struct FlatEvent<'a> {
    pub tick: AbsoluteTicks,
    pub event: &'a RhythmicNotatedEvent<'a>,
    pub staff: StaffIdx,
    pub voice: VoiceIdx,
}

impl<'a> std::fmt::Debug for FlatEvent<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatEvent")
            .field("tick", &self.tick)
            .field("staff", &self.staff)
            .field("voice", &self.voice)
            .finish_non_exhaustive()
    }
}

/// Iterate all `RhythmicNotatedEvent`s in the score in (staff, voice) order
/// with accumulated absolute ticks.
///
/// Non-`Common` voice elements (e.g., `LilypondVoiceElement::Other`) are skipped;
/// their duration is encoded in the surrounding events.
///
/// [AMEND-C] consumes accessors `staff_groups`, `staves`, `voices`, `as_rhythmic_event`
/// added in Phase 3 task 3a-i.
pub fn iter_events<'a>(score: &'a LilypondScore<'a>) -> impl Iterator<Item = FlatEvent<'a>> + 'a {
    let mut out: Vec<FlatEvent<'a>> = Vec::new();
    let mut staff_idx: StaffIdx = 0;
    for group in score.staff_groups() {
        for staff in group.staves() {
            for (voice_idx, voice) in staff.voices().iter().enumerate() {
                let mut tick: AbsoluteTicks = 0;
                for elem in voice {
                    // [AMEND-C] `as_rhythmic_event` returns `Option<&_>` so non-Common
                    // voice elements (clef changes, articulations) are skipped without
                    // advancing tick (their duration is encoded in the surrounding event).
                    let Some(e) = elem.as_rhythmic_event() else { continue };
                    // [AMEND-E] explicit `as u32` cast on usize-typed DurationTicks value.
                    let dt: u32 = match &e.event {
                        NotatedEvent::SingleEvent(_, d) => d.ticks() as u32,
                        NotatedEvent::Tuplet(t) => t.real_duration() as u32,
                    };
                    out.push(FlatEvent { tick, event: e, staff: staff_idx, voice: voice_idx });
                    tick += dt as u64;
                }
            }
            staff_idx += 1;
        }
    }
    out.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
    use crate::notation::lilypond::document::staff::LilypondStaff;
    use crate::notation::lilypond::staff_elements::LilypondVoiceElement;
    use crate::notation::rhythm::duration::{Duration, DurationKind};
    use crate::note::pitch::Pitch;

    #[test]
    fn flatten_single_quarter_note_at_tick_zero() {
        let pitch = Pitch::from_midi(60).unwrap();
        let event = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Qtr, 0));
        let voice = vec![LilypondVoiceElement::Common(event)];
        let staff = LilypondStaff::new().add_voice(voice);
        let group = LilypondStaffGroup::new(vec![staff]);
        let score = LilypondScore::new().staff_group(group);

        let events: Vec<_> = iter_events(&score).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tick, 0);
        assert_eq!(events[0].staff, 0);
        assert_eq!(events[0].voice, 0);
    }

    #[test]
    fn flatten_two_notes_ticks_accumulate() {
        let pitch = Pitch::from_midi(60).unwrap();
        let e1 = RhythmicNotatedEvent::pitch(pitch.clone(), Duration::new(DurationKind::Qtr, 0));
        let e2 = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Half, 0));
        let voice = vec![
            LilypondVoiceElement::Common(e1),
            LilypondVoiceElement::Common(e2),
        ];
        let staff = LilypondStaff::new().add_voice(voice);
        let group = LilypondStaffGroup::new(vec![staff]);
        let score = LilypondScore::new().staff_group(group);

        let events: Vec<_> = iter_events(&score).collect();
        assert_eq!(events.len(), 2);
        // First at tick 0, second at tick 32 (quarter note = 32 music ticks)
        assert_eq!(events[0].tick, 0);
        assert_eq!(events[1].tick, 32);
    }

    #[test]
    fn flatten_other_elements_skipped_tick_unchanged() {
        struct DummyLy;
        impl crate::notation::lilypond::ToLilypondString for DummyLy {
            fn to_lilypond_string(&self) -> String { "".to_string() }
        }
        let pitch = Pitch::from_midi(60).unwrap();
        let e1 = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Qtr, 0));
        let voice = vec![
            LilypondVoiceElement::Other(Box::new(DummyLy)),
            LilypondVoiceElement::Common(e1),
        ];
        let staff = LilypondStaff::new().add_voice(voice);
        let group = LilypondStaffGroup::new(vec![staff]);
        let score = LilypondScore::new().staff_group(group);

        let events: Vec<_> = iter_events(&score).collect();
        // Only the Common event is yielded; Other is skipped, tick unchanged
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tick, 0);
    }
}
