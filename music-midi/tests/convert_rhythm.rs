// REQ-O3, O4, O17, X8
use music::note::pitch::Pitch;
use music::notation::rhythm::duration::Duration;
use music::notation::rhythm::{RhythmicNotatedEvent, NotatedEvent, Tuplet};
use music::notation::rhythm::duration::DurationKind;
use music_midi::{ConvertCtx, DEFAULT_PPQ, MidiConversionError, MidiMessage, ToMidiEvents,
                 dynamics::VelocityPolicy, tempo::StaticTempoMap};

fn instr(_s: &str) -> u8 { 0 }
fn ctx<'a>(t: &'a StaticTempoMap) -> ConvertCtx<'a> {
    ConvertCtx::new(DEFAULT_PPQ, t, VelocityPolicy::Fixed(80), None, &instr).unwrap()
}

#[test]
fn simple_quarter_note() {
    let t = StaticTempoMap::constant(120.0); let c = ctx(&t);
    let e = RhythmicNotatedEvent::pitch(Pitch::from_midi(60).unwrap(), Duration::QTR);
    let mut out = Vec::new();
    let end = e.append_midi(0, 0, &c, &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(end, 480);
}

#[test]
fn rest_advances_time_emits_no_events() {
    let t = StaticTempoMap::constant(120.0); let c = ctx(&t);
    let e = RhythmicNotatedEvent::rest(Duration::HALF);
    let mut out = Vec::new();
    let end = e.append_midi(0, 0, &c, &mut out).unwrap();
    assert!(out.is_empty());
    assert_eq!(end, 960);
}

#[test]
fn two_tied_quarters_merge_into_one_held_note() {
    let t = StaticTempoMap::constant(120.0); let c = ctx(&t);
    let p = Pitch::from_midi(60).unwrap();
    let es = vec![
        RhythmicNotatedEvent::pitch(p, Duration::QTR),
        RhythmicNotatedEvent::pitch_tied(p, Duration::QTR),
    ];
    let mut out = Vec::new();
    es.append_midi(0, 0, &c, &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert!(matches!(out[0].message, MidiMessage::NoteOn { key: 60, .. }));
    assert!(matches!(out[1].message, MidiMessage::NoteOff { key: 60, .. }));
    assert_eq!(out[1].time, 960);  // two quarters = half note
}

#[test]
fn quarter_triplet_at_ppq_480_is_exact() {
    let t = StaticTempoMap::constant(120.0); let c = ctx(&t);
    let tup = Tuplet::new(
        vec![
            RhythmicNotatedEvent::pitch(Pitch::from_midi(60).unwrap(), Duration::QTR),
            RhythmicNotatedEvent::pitch(Pitch::from_midi(62).unwrap(), Duration::QTR),
            RhythmicNotatedEvent::pitch(Pitch::from_midi(64).unwrap(), Duration::QTR),
        ],
        3, 2, DurationKind::Qtr,
    );
    let e = RhythmicNotatedEvent { tied: false,
        event: NotatedEvent::Tuplet(tup) };
    let mut out = Vec::new();
    let end = e.append_midi(0, 0, &c, &mut out).unwrap();
    // three quarter-triplets span 2 beats = 960 MIDI ticks
    assert_eq!(end, 960);
    assert_eq!(out.len(), 6);  // 3 ons + 3 offs
}

#[test]
fn septuplet_at_ppq_480_errors_or_rounds() {
    let t = StaticTempoMap::constant(120.0); let c = ctx(&t);
    let tup = Tuplet::new(
        (0..7).map(|_| RhythmicNotatedEvent::pitch(Pitch::from_midi(60).unwrap(), Duration::QTR)).collect(),
        7, 4, DurationKind::Qtr,
    );
    let e = RhythmicNotatedEvent { tied: false,
        event: NotatedEvent::Tuplet(tup) };
    let mut out = Vec::new();
    let r = e.append_midi(0, 0, &c, &mut out);
    assert!(matches!(r, Err(MidiConversionError::TupletInexact { num: 7, den: 4, .. })));
}

/// [AMEND-W3] regression test: outer 3:2 triplet containing one quarter note + one inner 3:2
/// triplet of three quarter notes. At PPQ=288 the math is exact.
///
/// With PPQ=288, Quarter = 32 music ticks = 288 MIDI ticks.
/// Outer cum ratio after fold: num=3, den=2.
/// - Outer child 0 (single QTR, 32 music ticks):
///   scaled = 32 * 288 * 2 / (32 * 3) = 192 MIDI ticks.
/// - Outer child 1 (inner 3:2 tuplet, real_duration = 2*QTR = 64 music ticks):
///   scaled = 64 * 288 * 2 / (32 * 3) = 384 MIDI ticks (total span of inner group).
///   Inside, cum ratio = 3*3 / 2*2 = 9/4. Each inner QTR child:
///   scaled = 32 * 288 * 4 / (32 * 9) = 128 MIDI ticks.
///   3 * 128 = 384. Correct.
/// Total: 192 + 384 = 576 = 2 * 288 = outer real_duration. Correct.
///
/// Key assertion: inner note durations = 128, NOT 192 (which would be wrong: inner-only scale
/// of 2/3 on a QTR would give 32 * 288 * 2 / (32 * 3) = 192).
#[test]
fn nested_triplet_in_triplet_compound_ratio() {
    // Use PPQ=288 for exact compound-ratio arithmetic on 3:2 * 3:2.
    let t = StaticTempoMap::constant(120.0);
    let ctx = ConvertCtx::new(288, &t, VelocityPolicy::Fixed(80), None, &instr).unwrap();

    let inner_tup = Tuplet::new(
        vec![
            RhythmicNotatedEvent::pitch(Pitch::from_midi(64).unwrap(), Duration::QTR),
            RhythmicNotatedEvent::pitch(Pitch::from_midi(65).unwrap(), Duration::QTR),
            RhythmicNotatedEvent::pitch(Pitch::from_midi(67).unwrap(), Duration::QTR),
        ],
        3, 2, DurationKind::Qtr,
    );

    let outer_tup = Tuplet::new(
        vec![
            RhythmicNotatedEvent::pitch(Pitch::from_midi(60).unwrap(), Duration::QTR),
            RhythmicNotatedEvent { tied: false, event: NotatedEvent::Tuplet(inner_tup) },
        ],
        3, 2, DurationKind::Qtr,
    );

    let e = RhythmicNotatedEvent { tied: false,
        event: NotatedEvent::Tuplet(outer_tup) };
    let mut out = Vec::new();
    let end = e.append_midi(0, 0, &ctx, &mut out).unwrap();

    // Total span = outer real_duration = 2 QTR at PPQ=288 = 576 MIDI ticks.
    assert_eq!(end, 576, "outer triplet must span exactly 2 beats = 576 ticks at PPQ=288");

    // 8 events: outer note (on+off) + 3 inner notes (on+off each) = 2 + 6 = 8.
    assert_eq!(out.len(), 8);

    // The outer single note starts at tick 0 and is 192 ticks long.
    let outer_on = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOn { key: 60, .. })).unwrap();
    let outer_off = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOff { key: 60, .. })).unwrap();
    assert_eq!(outer_on.time, 0);
    assert_eq!(outer_off.time, 192, "outer QTR in 3:2 at PPQ=288 = 192 ticks");

    // The inner notes start at tick 192 (after the outer note).
    // Each inner note under compound ratio 9:4 at PPQ=288 spans 128 ticks.
    // Inner note 0 (key=64): on at 192, off at 192+128=320.
    // Inner note 1 (key=65): on at 320, off at 320+128=448.
    // Inner note 2 (key=67): on at 448, off at 448+128=576.

    let inner_on_0 = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOn { key: 64, .. })).unwrap();
    assert_eq!(inner_on_0.time, 192,
        "inner note 0 must start at 192, NOT at 0+192 = 192 under outer-only scale \
         (should have been 192 here, but must also be 128 ticks long, not 192)");

    let inner_off_0 = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOff { key: 64, .. })).unwrap();
    // The key assertion: inner note duration = 128 (compound 9:4), NOT 192 (inner-only 3:2).
    assert_eq!(inner_off_0.time - inner_on_0.time, 128,
        "compound ratio 9:4 gives 128 ticks/note, NOT the wrong inner-only 192");

    // Verify second and third inner notes are also 128 ticks each.
    let inner_on_1 = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOn { key: 65, .. })).unwrap();
    let inner_off_1 = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOff { key: 65, .. })).unwrap();
    assert_eq!(inner_off_1.time - inner_on_1.time, 128);

    let inner_on_2 = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOn { key: 67, .. })).unwrap();
    let inner_off_2 = out.iter().find(|e| matches!(e.message, MidiMessage::NoteOff { key: 67, .. })).unwrap();
    assert_eq!(inner_off_2.time - inner_on_2.time, 128);
    assert_eq!(inner_on_2.time, 448);
    assert_eq!(inner_off_2.time, 576);
}
