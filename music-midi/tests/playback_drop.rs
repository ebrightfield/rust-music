// REQ availability-security: AllNotesOff CC 123 is sent for every channel on Drop.
// Uses MidiPlayer::from_sink to avoid opening a real MIDI port during CI.
#![cfg(feature = "playback")]

use music_midi::error::MidiConversionError;
use music_midi::playback::{Clock, MidiPlayer, RealtimeSink};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// ─── CapturingSink ────────────────────────────────────────────────────────────

/// A `RealtimeSink` that records every sent byte vector for later inspection.
#[derive(Clone, Default)]
struct CapturingSink(Arc<Mutex<Vec<Vec<u8>>>>);

impl RealtimeSink for CapturingSink {
    fn send(&mut self, b: &[u8]) -> Result<(), MidiConversionError> {
        self.0.lock().unwrap().push(b.to_vec());
        Ok(())
    }
}

// ─── FakeClock ───────────────────────────────────────────────────────────────

/// A deterministic clock for testing — advances `now` to the requested deadline.
struct FakeClock {
    now: Instant,
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        self.now
    }

    fn sleep_until(&mut self, t: Instant) {
        if t > self.now {
            self.now = t;
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn drop_sends_all_notes_off_on_all_16_channels() {
    let cap = CapturingSink::default();
    {
        let _player = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(FakeClock { now: Instant::now() }));
        // implicit drop here — AllNotesOff should fire for channels 0..=15
    }
    let sent = cap.0.lock().unwrap();
    assert_eq!(sent.len(), 16, "expected 16 AllNotesOff messages, one per channel");

    for (i, bytes) in sent.iter().enumerate() {
        assert_eq!(
            bytes.len(),
            3,
            "channel {i} message should be 3 bytes"
        );
        assert_eq!(
            bytes[0] & 0xF0,
            0xB0,
            "channel {i}: status nibble should be 0xB (CC)"
        );
        assert_eq!(
            bytes[0] & 0x0F,
            i as u8,
            "channel {i}: channel nibble should match index"
        );
        assert_eq!(
            bytes[1], 123,
            "channel {i}: controller number should be 123 (AllNotesOff)"
        );
        assert_eq!(bytes[2], 0, "channel {i}: value should be 0");
    }
}

#[test]
fn drop_sends_all_notes_off_in_order_channel_0_first() {
    let cap = CapturingSink::default();
    {
        let _p = MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(FakeClock { now: Instant::now() }));
    }
    let sent = cap.0.lock().unwrap();
    // First message must be channel 0, last must be channel 15.
    assert_eq!(sent.first().unwrap()[0], 0xB0);
    assert_eq!(sent.last().unwrap()[0], 0xBF);
}
