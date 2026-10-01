// Smoke tests for RealtimeSink and Clock trait impls.
// Uses MidiPlayer::from_sink to avoid opening a real MIDI port during CI.
#![cfg(feature = "playback")]

use music_midi::error::MidiConversionError;
use music_midi::playback::{Clock, MidiPlayer, RealtimeSink, SystemClock};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ─── CapturingSink ────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct CapturingSink(Arc<Mutex<Vec<Vec<u8>>>>);

impl RealtimeSink for CapturingSink {
    fn send(&mut self, b: &[u8]) -> Result<(), MidiConversionError> {
        self.0.lock().unwrap().push(b.to_vec());
        Ok(())
    }
}

// ─── FakeClock ───────────────────────────────────────────────────────────────

struct FakeClock {
    now: Instant,
    advances: Vec<Instant>,
}

impl FakeClock {
    fn new() -> Self {
        Self {
            now: Instant::now(),
            advances: Vec::new(),
        }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.now
    }

    fn sleep_until(&mut self, t: Instant) {
        self.advances.push(t);
        if t > self.now {
            self.now = t;
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn capturing_sink_records_sent_bytes() {
    let cap = CapturingSink::default();
    let mut sink = cap.clone();

    sink.send(&[0x90, 60, 100]).unwrap();
    sink.send(&[0x80, 60, 0]).unwrap();

    let recorded = cap.0.lock().unwrap();
    assert_eq!(recorded.len(), 2);
    assert_eq!(recorded[0], vec![0x90, 60, 100]);
    assert_eq!(recorded[1], vec![0x80, 60, 0]);
}

#[test]
fn fake_clock_advances_on_sleep_until() {
    let mut clock = FakeClock::new();
    let t0 = clock.now();
    let future = t0 + Duration::from_millis(500);

    clock.sleep_until(future);

    assert_eq!(clock.now(), future);
    assert_eq!(clock.advances, vec![future]);
}

#[test]
fn fake_clock_does_not_go_backward() {
    let mut clock = FakeClock::new();
    let t0 = clock.now();
    // Sleep to a past instant — clock should not regress.
    clock.sleep_until(t0 - Duration::from_millis(1));
    assert_eq!(
        clock.now(),
        t0,
        "clock should not regress when target is in the past"
    );
}

#[test]
fn system_clock_now_increases_monotonically() {
    let clock = SystemClock;
    let t0 = clock.now();
    let t1 = clock.now();
    // Monotonic: t1 >= t0
    assert!(t1 >= t0);
}

#[test]
fn system_clock_sleep_until_past_is_noop() {
    let mut clock = SystemClock;
    let past = Instant::now() - Duration::from_secs(1);
    // Should not block or panic — sleeping to a past deadline is a no-op.
    clock.sleep_until(past);
}

#[test]
fn midi_player_from_sink_round_trip() {
    // A player constructed with from_sink can be created and dropped cleanly.
    let cap = CapturingSink::default();
    let fake_clock = FakeClock::new();
    let _player = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(fake_clock));
    // Player drops here; AllNotesOff on 16 channels is sent.
    drop(_player);
    let sent = cap.0.lock().unwrap();
    // Drop sends 16 AllNotesOff messages.
    assert_eq!(sent.len(), 16);
}

#[test]
fn play_blocking_empty_smf_completes_without_error() {
    use midly::{Format, Header, Smf, Timing};

    let cap = CapturingSink::default();
    let fake_clock = FakeClock::new();
    let mut player = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(fake_clock));

    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(midly::num::u15::from(480)),
        },
        tracks: vec![],
    };

    let result = player.play_blocking(&smf);
    assert!(result.is_ok(), "play_blocking on empty SMF should succeed");

    // No MIDI events were sent (only the AllNotesOff on drop, which hasn't happened yet).
    let sent_during_play = cap.0.lock().unwrap().len();
    assert_eq!(
        sent_during_play, 0,
        "no MIDI events should be sent for an empty SMF"
    );
}

#[test]
fn play_blocking_single_note_sends_noteon_and_noteoff() {
    use midly::{
        num::{u15, u28, u4, u7},
        Format, Header, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
    };

    let cap = CapturingSink::default();
    let fake_clock = FakeClock::new();
    let mut player = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(fake_clock));

    // Build a minimal single-track SMF: NoteOn(C4, vel=100) at tick 0, NoteOff at tick 480.
    let track = vec![
        TrackEvent {
            delta: u28::from(0),
            kind: TrackEventKind::Midi {
                channel: u4::from(0),
                message: MidiMessage::NoteOn {
                    key: u7::from(60),
                    vel: u7::from(100),
                },
            },
        },
        TrackEvent {
            delta: u28::from(480),
            kind: TrackEventKind::Midi {
                channel: u4::from(0),
                message: MidiMessage::NoteOff {
                    key: u7::from(60),
                    vel: u7::from(0),
                },
            },
        },
    ];

    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(u15::from(480)),
        },
        tracks: vec![track],
    };

    player
        .play_blocking(&smf)
        .expect("play_blocking should succeed");

    let sent = cap.0.lock().unwrap();
    // Expect exactly 2 MIDI events (NoteOn + NoteOff), before the drop fires AllNotesOff.
    assert_eq!(sent.len(), 2, "expected NoteOn + NoteOff");
    // First event: NoteOn on channel 0
    assert_eq!(sent[0][0], 0x90, "first event should be NoteOn (0x90)");
    assert_eq!(sent[0][1], 60, "key should be C4 (60)");
    assert_eq!(sent[0][2], 100, "velocity should be 100");
    // Second event: NoteOff on channel 0
    assert_eq!(
        sent[1][0] & 0xF0,
        0x80,
        "second event should be NoteOff (0x8n)"
    );
    assert_eq!(sent[1][1], 60);
}
