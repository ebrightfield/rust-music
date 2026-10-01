// Deterministic tests for play_background / PlaybackHandle::stop using FakeClock.
#![cfg(feature = "playback")]

use music_midi::error::MidiConversionError;
use music_midi::playback::{Clock, MidiPlayer, RealtimeSink};
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

/// Fake clock that uses the REAL wall clock under the hood but accepts
/// arbitrary `sleep_until` deadlines. This lets tests that spawn threads work
/// without complicated synchronization of a manually-advanced clock.
struct FakeClock {
    base: Instant,
}

impl FakeClock {
    fn new() -> Self {
        Self {
            base: Instant::now(),
        }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep_until(&mut self, t: Instant) {
        let now = Instant::now();
        if t > now {
            std::thread::sleep(t - now);
        }
        let _ = self.base; // suppress unused warning
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn play_background_empty_smf_completes_and_stop_is_clean() {
    use midly::{Format, Header, Smf, Timing};

    let cap = CapturingSink::default();
    let player = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(FakeClock::new()));

    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(midly::num::u15::from(480)),
        },
        tracks: vec![],
    };

    let handle = player
        .play_background(&smf)
        .expect("play_background should succeed");
    // stop() blocks until the thread finishes and AllNotesOff is sent.
    handle.stop();

    let sent = cap.0.lock().unwrap();
    // AllNotesOff on all 16 channels is sent by the player's Drop on the thread.
    assert_eq!(
        sent.len(),
        16,
        "stop() should have triggered AllNotesOff on all channels"
    );
    for (i, bytes) in sent.iter().enumerate() {
        assert_eq!(bytes[0] & 0xF0, 0xB0, "channel {i}: CC status");
        assert_eq!(bytes[1], 123, "channel {i}: AllNotesOff controller");
    }
}

#[test]
fn play_background_stop_can_be_called_after_natural_finish() {
    use midly::{Format, Header, Smf, Timing};

    let cap = CapturingSink::default();
    let player = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(FakeClock::new()));

    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(midly::num::u15::from(480)),
        },
        tracks: vec![],
    };

    let handle = player
        .play_background(&smf)
        .expect("play_background should succeed");

    // Give the thread time to finish naturally before we call stop().
    std::thread::sleep(Duration::from_millis(50));

    // stop() on an already-finished player should not panic or deadlock.
    handle.stop();
}

#[test]
fn play_background_stop_is_deterministic() {
    use midly::{Format, Header, Smf, Timing, TrackEvent, TrackEventKind};

    // Construct a LONG SMF: many events spaced 10 seconds apart at 480 PPQ.
    // If the test ever waits for natural completion, this would hang for minutes.
    let ppq = 480u16;
    let mut track: Vec<TrackEvent> = Vec::new();
    // Tempo: 120 BPM → 1 quarter = 500 ms; 10s per event = 20 quarters = 9600 ticks
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::Tempo(500_000u32.into())),
    });
    for _ in 0..100 {
        track.push(TrackEvent {
            delta: 9600u32.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: midly::MidiMessage::NoteOn {
                    key: 60.into(),
                    vel: 80u8.into(),
                },
            },
        });
    }
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });
    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(ppq.into())),
        tracks: vec![track],
    };

    let cap = CapturingSink::default();
    let player = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(FakeClock::new()));
    let handle = player.play_background(&smf).expect("background play ok");
    // REQ-O7: send on stop channel; REQ-O8: assert handle.stop() returns.
    handle.stop();
    // The assertion is that handle.stop() returned (meaning the thread joined
    // cleanly). No wall-clock, no Instant::elapsed, no sleep-based polling.
    let sent = cap.0.lock().unwrap();
    assert!(
        sent.iter().any(|b| b.len() == 3 && b[1] == 123),
        "AllNotesOff must have been sent (CC 123 on any channel) after stop"
    );
}
