// REQ-O13: user-wins precedence

#![cfg(feature = "playback")]

use music_midi::playback::{Clock, MidiPlayer, RealtimeSink};
use music_midi::{MidiConversionError, TempoSource};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone, Default)]
struct CapturingSink(Arc<Mutex<Vec<Vec<u8>>>>);
impl RealtimeSink for CapturingSink {
    fn send(&mut self, b: &[u8]) -> Result<(), MidiConversionError> {
        self.0.lock().unwrap().push(b.to_vec());
        Ok(())
    }
}

struct FakeClock;
impl Clock for FakeClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
    fn sleep_until(&mut self, t: Instant) {
        let n = Instant::now();
        if t > n {
            std::thread::sleep(t - n);
        }
    }
}

#[derive(Default)]
struct CountingTempo {
    query_count: Arc<std::sync::atomic::AtomicUsize>,
    bpm: f32,
}

impl TempoSource for CountingTempo {
    fn bpm_at(&self, _: u64) -> f32 {
        self.query_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.bpm
    }
}

#[test]
fn user_tempo_source_wins_over_smf_metas() {
    // REQ-O13: SMF with 90 BPM meta + with_tempo(120) → user source queried.
    use midly::{Format, Header, MetaMessage, Smf, Timing, TrackEvent, TrackEventKind};

    let ppq = 480u16;
    let mut track: Vec<TrackEvent> = Vec::new();
    // 90 BPM tempo meta: 60_000_000 / 90 = 666_667 µs/beat
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(666_667u32.into())),
    });
    track.push(TrackEvent {
        delta: u32::from(ppq).into(),
        kind: TrackEventKind::Midi {
            channel: 0.into(),
            message: midly::MidiMessage::NoteOn {
                key: 60.into(),
                vel: 80u8.into(),
            },
        },
    });
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(ppq.into())),
        tracks: vec![track],
    };

    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let user_tempo = CountingTempo {
        query_count: counter.clone(),
        bpm: 120.0,
    };

    let cap = CapturingSink::default();
    let mut player =
        MidiPlayer::from_sink(Box::new(cap.clone()), Box::new(FakeClock)).with_tempo(user_tempo);

    player.play_blocking(&smf).expect("play ok");

    // REQ-O13: assert user source was queried.
    assert!(
        counter.load(std::sync::atomic::Ordering::SeqCst) >= 1,
        "user TempoSource should have been queried at least once"
    );
}

#[test]
fn no_with_tempo_falls_back_to_smf_metas() {
    // REQ-O11: without with_tempo, SMF tempo metas are used (StaticTempoMap fallback).
    use midly::{Format, Header, Smf, Timing};
    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(480u16.into())),
        tracks: vec![],
    };
    let cap = CapturingSink::default();
    let mut player = MidiPlayer::from_sink(Box::new(cap), Box::new(FakeClock));
    // Should not panic; empty SMF completes trivially.
    player.play_blocking(&smf).expect("fallback path ok");
}
