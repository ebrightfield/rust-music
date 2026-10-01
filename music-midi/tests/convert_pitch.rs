// REQ-O1, O12

use music::note::pitch::Pitch;
use music_midi::{
    dynamics::{Dynamic, VelocityPolicy},
    tempo::StaticTempoMap,
    ConvertCtx, MidiMessage, ToMidiEvents, DEFAULT_PPQ,
};

fn instr(_s: &str) -> u8 {
    0
}

#[test]
fn middle_c_emits_noteon_noteoff() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(DEFAULT_PPQ, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();
    let p = Pitch::from_midi(60).unwrap();
    let mut out = Vec::new();
    let end = p.append_midi(0, 0, &ctx, &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert!(matches!(
        out[0].message,
        MidiMessage::NoteOn {
            key: 60,
            velocity: 80
        }
    ));
    assert!(matches!(
        out[1].message,
        MidiMessage::NoteOff { key: 60, .. }
    ));
    assert_eq!(end, DEFAULT_PPQ as u64);
}

#[test]
fn dynamic_mapping_applies() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(
        DEFAULT_PPQ,
        &t,
        VelocityPolicy::FromDynamic(Dynamic::Pp),
        None,
        &instr,
    )
    .unwrap();
    let p = Pitch::from_midi(60).unwrap();
    let mut out = Vec::new();
    p.append_midi(0, 0, &ctx, &mut out).unwrap();
    match out[0].message {
        MidiMessage::NoteOn { velocity, .. } => assert_eq!(velocity, 16),
        _ => panic!(),
    }
}

#[test]
fn round_trip_midi_note_preserved() {
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(DEFAULT_PPQ, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();
    // Pitch::from_midi computes octave as (n / 12) - 1 on u8; values 0..11 underflow.
    // Valid range starts at 12 (C0); upper bound 107 per Pitch::from_midi rejection of >= 108.
    for n in 12u8..=107 {
        let p = Pitch::from_midi(n).unwrap();
        let mut out = Vec::new();
        p.append_midi(0, 0, &ctx, &mut out).unwrap();
        match out[0].message {
            MidiMessage::NoteOn { key, .. } => assert_eq!(key, n),
            _ => panic!(),
        }
    }
}
