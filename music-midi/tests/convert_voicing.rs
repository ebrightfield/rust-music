// REQ-O2
use music::note::pitch::Pitch;
use music::note_collections::voicing::Voicing;
use music_midi::{ConvertCtx, DEFAULT_PPQ, MidiMessage, ToMidiEvents,
                 dynamics::VelocityPolicy, tempo::StaticTempoMap};

fn instr(_s: &str) -> u8 { 0 }

#[test]
fn c_major_triad_emits_three_noteons_then_three_noteoffs() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(DEFAULT_PPQ, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();
    let v = Voicing::new(vec![
        Pitch::from_midi(60).unwrap(),
        Pitch::from_midi(64).unwrap(),
        Pitch::from_midi(67).unwrap(),
    ]);
    let mut out = Vec::new();
    let end = v.append_midi(0, 0, &ctx, &mut out).unwrap();
    assert_eq!(out.len(), 6);
    let ons: Vec<_> = out.iter().filter(|e| matches!(e.message, MidiMessage::NoteOn { .. })).collect();
    let offs: Vec<_> = out.iter().filter(|e| matches!(e.message, MidiMessage::NoteOff { .. })).collect();
    assert_eq!(ons.len(), 3);
    assert_eq!(offs.len(), 3);
    for o in &ons { assert_eq!(o.time, 0); }
    for o in &offs { assert_eq!(o.time, DEFAULT_PPQ as u64); }
    assert_eq!(end, DEFAULT_PPQ as u64);
}
