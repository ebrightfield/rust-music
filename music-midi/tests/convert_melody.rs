// REQ-O1, O3, O4
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::Duration;
use music::melody::sequencer::MelodicEvent;
use music_midi::{ConvertCtx, DEFAULT_PPQ, MidiMessage, ToMidiEvents,
                 dynamics::VelocityPolicy, tempo::StaticTempoMap};

fn instr(_s: &str) -> u8 { 0 }

/// Two quarter notes should emit NoteOn at tick 0 and 480, with delta = 480.
#[test]
fn two_quarter_notes_delta_times_correct() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(DEFAULT_PPQ, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();

    let events = vec![
        MelodicEvent::new(Pitch::from_midi(60).unwrap(), Duration::QTR),
        MelodicEvent::new(Pitch::from_midi(62).unwrap(), Duration::QTR),
    ];
    let mut out = Vec::new();
    let end = events.append_midi(0, 0, &ctx, &mut out).unwrap();

    // Each quarter = 480 MIDI ticks at PPQ=480; two quarters = end at 960.
    assert_eq!(end, 960);
    // 2 notes × (NoteOn + NoteOff) = 4 events.
    assert_eq!(out.len(), 4);

    let ons: Vec<_> = out.iter().filter(|e| matches!(e.message, MidiMessage::NoteOn { .. })).collect();
    assert_eq!(ons[0].time, 0,   "first note starts at tick 0");
    assert_eq!(ons[1].time, 480, "second note starts at tick 480");

    // Delta between consecutive NoteOns = 480 = QTR ticks * ppq / 32
    let delta = ons[1].time - ons[0].time;
    assert_eq!(delta, (Duration::QTR.ticks() as u64) * (DEFAULT_PPQ as u64) / 32);
}

/// A tied pair of quarter notes produces a single held note spanning a half note's duration.
#[test]
fn two_tied_melodic_quarters_merge_into_half_note() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(DEFAULT_PPQ, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();

    let p = Pitch::from_midi(60).unwrap();
    let events = vec![
        MelodicEvent::new(p, Duration::QTR),
        MelodicEvent::tied(p, Duration::QTR),
    ];
    let mut out = Vec::new();
    let end = events.append_midi(0, 0, &ctx, &mut out).unwrap();

    // Two tied quarters = half note = 960 ticks.
    assert_eq!(end, 960);
    // Merged into single NoteOn + NoteOff.
    assert_eq!(out.len(), 2);
    assert!(matches!(out[0].message, MidiMessage::NoteOn  { key: 60, .. }));
    assert!(matches!(out[1].message, MidiMessage::NoteOff { key: 60, .. }));
    assert_eq!(out[0].time, 0);
    assert_eq!(out[1].time, 960);
}

/// A single quarter note emits the correct NoteOn/NoteOff pair.
#[test]
fn single_melodic_event_emits_noteon_noteoff() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(DEFAULT_PPQ, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();

    let events = vec![
        MelodicEvent::new(Pitch::from_midi(69).unwrap(), Duration::QTR),
    ];
    let mut out = Vec::new();
    let end = events.append_midi(0, 0, &ctx, &mut out).unwrap();

    assert_eq!(out.len(), 2);
    assert!(matches!(out[0].message, MidiMessage::NoteOn  { key: 69, velocity: 80 }));
    assert!(matches!(out[1].message, MidiMessage::NoteOff { key: 69, .. }));
    assert_eq!(end, 480);
}
