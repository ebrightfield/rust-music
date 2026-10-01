// REQ-O3, O6, O16: LilypondScore -> MIDI events via flatten::iter_events
#![cfg(feature = "smf")]

use music::notation::lilypond::document::score::{LilypondScore, LilypondStaffGroup};
use music::notation::lilypond::document::staff::LilypondStaff;
use music::notation::lilypond::staff_elements::LilypondVoiceElement;
use music::notation::rhythm::{
    duration::{Duration, DurationKind},
    RhythmicNotatedEvent,
};
use music::note::pitch::Pitch;
use music_midi::{
    smf::SmfBuilder, tempo::StaticTempoMap, ConvertCtx, MidiConversionError, MidiMessage,
    ToMidiEvents, VelocityPolicy,
};

fn make_score_with_single_note(midi_note: u8) -> LilypondScore<'static> {
    // NOTE: This function leaks a Pitch for 'static lifetime to avoid lifetime complexity.
    // Safe in tests — leaking a single small value is acceptable.
    let pitch = Pitch::from_midi(midi_note).unwrap();
    let event = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Qtr, 0));
    let voice = vec![LilypondVoiceElement::Common(event)];
    let staff = LilypondStaff::new().add_voice(voice);
    let group = LilypondStaffGroup::new(vec![staff]);
    LilypondScore::new().staff_group(group)
}

/// Smoke test: a score with a single quarter note emits NoteOn + NoteOff.
#[test]
fn score_single_note_emits_note_on_and_off() {
    let score = make_score_with_single_note(60);
    let tm = StaticTempoMap::constant(120.0);
    let instr = |_: &str| 0u8;
    let ctx = ConvertCtx::new(480, &tm, VelocityPolicy::Fixed(80), None, &instr).unwrap();
    let mut out = Vec::new();
    score.append_midi(0, 0, &ctx, &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert!(matches!(
        out[0].message,
        MidiMessage::NoteOn { key: 60, .. }
    ));
    assert!(matches!(
        out[1].message,
        MidiMessage::NoteOff { key: 60, .. }
    ));
}

/// Score with 2 voices gets channels 0 and 1.
#[test]
fn score_two_voices_get_distinct_channels() {
    let pitch_a = Pitch::from_midi(60).unwrap();
    let pitch_b = Pitch::from_midi(64).unwrap();

    let event_a = RhythmicNotatedEvent::pitch(pitch_a, Duration::new(DurationKind::Qtr, 0));
    let event_b = RhythmicNotatedEvent::pitch(pitch_b, Duration::new(DurationKind::Qtr, 0));
    let voice_a = vec![LilypondVoiceElement::Common(event_a)];
    let voice_b = vec![LilypondVoiceElement::Common(event_b)];
    let staff = LilypondStaff::new().add_voice(voice_a).add_voice(voice_b);
    let group = LilypondStaffGroup::new(vec![staff]);
    let score = LilypondScore::new().staff_group(group);

    let tm = StaticTempoMap::constant(120.0);
    let instr = |_: &str| 0u8;
    let ctx = ConvertCtx::new(480, &tm, VelocityPolicy::Fixed(80), None, &instr).unwrap();
    let mut out = Vec::new();
    score.append_midi(0, 0, &ctx, &mut out).unwrap();

    // 4 events: 2 NoteOns + 2 NoteOffs, across channels 0 and 1
    assert_eq!(out.len(), 4);
    let channels: std::collections::HashSet<u8> = out.iter().map(|e| e.channel).collect();
    assert_eq!(channels.len(), 2, "expected 2 distinct channels");
}

/// Score→SMF round-trip via SmfBuilder::add_track.
#[test]
fn score_to_smf_round_trip() {
    let score = make_score_with_single_note(60);
    let smf = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("score", 0, &score)
        .unwrap()
        .build()
        .unwrap();
    let mut bytes = Vec::new();
    smf.write(&mut bytes).unwrap();
    let parsed = midly::Smf::parse(&bytes).unwrap();
    assert_eq!(parsed.tracks.len(), 2, "conductor + score track");
}
