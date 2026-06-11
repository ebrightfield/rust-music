//! Realtime MIDI playback via `midir` or a custom [`RealtimeSink`].
//!
//! Requires the `playback` feature (`music-midi = { features = ["playback"] }`).
//!
//! # Panic-safety
//! [`MidiPlayer`] sends `AllNotesOff` (CC 123) on all 16 channels in its `Drop`
//! impl. To ensure notes are silenced even on hard kill (e.g. `SIGKILL`), install
//! a panic hook that calls [`MidiPlayer::connect_default`] and drops it, or embed
//! the player in a `ctrlc`/`signal-hook` handler. See `docs/midi-playback.md`.
//!
//! # Stop resolution
//! [`PlaybackHandle::stop`] polls the stop channel every 10 ms during sleep intervals.
//! Worst-case latency from `stop()` call to silence is approximately 10 ms.
// REQ availability-security: AllNotesOff on Drop; dynamic tempo polling (P4)
// Feature: playback — realtime MIDI output via midir or custom RealtimeSink.

use crate::{error::MidiConversionError, tempo::StaticTempoMap};
use std::{
    sync::mpsc,
    time::{Duration as StdDuration, Instant},
};

// ─── Seam traits ─────────────────────────────────────────────────────────────

/// A writable MIDI output sink. Implement this to integrate a non-midir backend
/// (OSC, network, testing, null-sink).
pub trait RealtimeSink {
    fn send(&mut self, bytes: &[u8]) -> Result<(), MidiConversionError>;
}

/// A monotonic wall-clock with sleep capability. Implement this to provide a
/// deterministic fake clock in tests.
pub trait Clock {
    fn now(&self) -> Instant;
    fn sleep_until(&mut self, t: Instant);
}

// ─── SystemClock ─────────────────────────────────────────────────────────────

/// Real system clock using `std::time::Instant` and `std::thread::sleep`.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep_until(&mut self, t: Instant) {
        let now = Instant::now();
        if t > now {
            std::thread::sleep(t - now);
        }
    }
}

// ─── MidirSink ───────────────────────────────────────────────────────────────

/// A [`RealtimeSink`] backed by a live `midir::MidiOutputConnection`.
pub struct MidirSink(pub(crate) midir::MidiOutputConnection);

impl RealtimeSink for MidirSink {
    fn send(&mut self, bytes: &[u8]) -> Result<(), MidiConversionError> {
        self.0.send(bytes).map_err(|e| {
            MidiConversionError::Io(std::io::Error::other(e.to_string()))
        })
    }
}

// ─── MidiPlayer ──────────────────────────────────────────────────────────────

/// Realtime MIDI player. Sends MIDI bytes to a [`RealtimeSink`] timed by a [`Clock`].
///
/// On `Drop`, sends `AllNotesOff` (CC 123) on all 16 channels.
pub struct MidiPlayer {
    sink: Box<dyn RealtimeSink + Send>,
    clock: Box<dyn Clock + Send>,
    tempo: Option<Box<dyn crate::TempoSource>>,
}

impl MidiPlayer {
    /// Construct a player from a custom sink and clock. Use this to integrate a
    /// non-midir backend (OSC, network, testing, null-sink).
    pub fn from_sink(
        sink: Box<dyn RealtimeSink + Send>,
        clock: Box<dyn Clock + Send>,
    ) -> Self {
        Self { sink, clock, tempo: None }
    }

    /// Open the first available system MIDI output port and bind a player to it.
    ///
    /// Returns [`MidiConversionError::NoPlaybackPort`] if no ports are present.
    pub fn connect_default() -> Result<Self, MidiConversionError> {
        let out = midir::MidiOutput::new("music-midi").map_err(|e| {
            MidiConversionError::Io(std::io::Error::other(e.to_string()))
        })?;
        let ports = out.ports();
        let port = ports.first().ok_or(MidiConversionError::NoPlaybackPort)?;
        let conn = out.connect(port, "music-midi").map_err(|e| {
            MidiConversionError::Io(std::io::Error::other(e.to_string()))
        })?;
        Ok(Self::from_sink(
            Box::new(MidirSink(conn)),
            Box::new(SystemClock),
        ))
    }

    /// Install a user-supplied [`TempoSource`](crate::TempoSource) for subsequent playback.
    ///
    /// When wired, `play_blocking` queries `self.tempo.bpm_at(tick)` per event
    /// and IGNORES `Tempo` meta events in the SMF conductor track. Without this
    /// call, `play_blocking` falls back to building a [`StaticTempoMap`] from the
    /// SMF conductor track — the prior behavior.
    ///
    /// # Precedence
    /// User-supplied `TempoSource` always wins over SMF-baked tempo meta events.
    pub fn with_tempo(
        mut self,
        tempo: impl crate::TempoSource + 'static,
    ) -> Self {
        // REQ-O9, O12
        self.tempo = Some(Box::new(tempo));
        self
    }

    /// Play an SMF synchronously (blocks until playback is complete or an error occurs).
    ///
    /// When a user-supplied [`TempoSource`](crate::TempoSource) is installed via
    /// [`with_tempo`](Self::with_tempo), it is queried per-event and SMF conductor
    /// tempo metas are ignored. Otherwise, a [`StaticTempoMap`] is built from the
    /// conductor track (prior behavior).
    ///
    /// Polling the stop channel between events is not done here; use
    /// [`MidiPlayer::play_background`] if you need interruptible playback.
    pub fn play_blocking(&mut self, smf: &midly::Smf) -> Result<(), MidiConversionError> {
        self.play_blocking_interruptible(smf, None)
    }

    /// Play an SMF in a background thread. Returns a [`PlaybackHandle`] that can
    /// stop playback and join the thread.
    ///
    /// The player is moved into the spawned thread; its `Drop` impl will send
    /// `AllNotesOff` when playback ends (normally or via `stop()`).
    pub fn play_background(
        mut self,
        smf: &midly::Smf,
    ) -> Result<PlaybackHandle, MidiConversionError> {
        // We need an owned `Smf<'static>` to move into the thread.
        // `Smf::to_static` strips bytestring lifetimes (safe: see midly docs).
        let smf_owned: midly::Smf<'static> = smf.to_static();

        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (done_tx, done_rx) = mpsc::channel::<()>();

        std::thread::spawn(move || {
            // Run the blocking loop; check the stop channel before each sleep.
            let result = self.play_blocking_interruptible(&smf_owned, Some(&stop_rx));
            // Suppress result — errors during background playback are ignored
            // (no caller context to propagate to).
            let _ = result;
            // `self` drops here → AllNotesOff sent in Drop impl.
            drop(done_tx); // signal that the thread finished
        });

        Ok(PlaybackHandle { stop_tx, done_rx })
    }

    /// Internal: plays the SMF, polling an optional stop channel between events.
    fn play_blocking_interruptible(
        &mut self,
        smf: &midly::Smf<'_>,
        stop_rx: Option<&mpsc::Receiver<()>>,
    ) -> Result<(), MidiConversionError> {
        let ppq = match smf.header.timing {
            midly::Timing::Metrical(tpb) => tpb.as_int(),
            midly::Timing::Timecode(_, _) => 480,
        };

        // REQ-O10, O11: user TempoSource (if installed) wins over SMF conductor
        // tempo metas. The fallback StaticTempoMap is built only when no user
        // source is present — routing happens at the tick-to-seconds site below.

        let mut events: Vec<(u64, Vec<u8>)> = Vec::new();
        for track in smf.tracks.iter() {
            let mut tick: u64 = 0;
            for ev in track.iter() {
                tick += u64::from(ev.delta.as_int());
                if let Some(live) = ev.kind.as_live_event() {
                    let mut buf = Vec::with_capacity(4);
                    live.write_std(&mut buf).map_err(|e| {
                        MidiConversionError::Io(std::io::Error::other(e.to_string()))
                    })?;
                    events.push((tick, buf));
                }
            }
        }
        events.sort_by_key(|(t, _)| *t);

        let start = self.clock.now();
        for (tick, bytes) in events {
            if let Some(rx) = stop_rx {
                if rx.try_recv().is_ok() { return Ok(()); }
            }
            // REQ-O10: user tempo wins; REQ-O11: fallback to SMF conductor map.
            let secs = match &self.tempo {
                Some(t) => ticks_to_seconds_dynamic(t.as_ref(), tick, ppq),
                None => static_map_from_conductor(smf, ppq).ticks_to_seconds(tick),
            };
            let deadline = start + StdDuration::from_secs_f64(secs);

            loop {
                let now = self.clock.now();
                if now >= deadline { break; }
                if let Some(rx) = stop_rx {
                    if rx.try_recv().is_ok() { return Ok(()); }
                }
                let remaining = deadline - now;
                let slice = remaining.min(StdDuration::from_millis(10));
                self.clock.sleep_until(now + slice);
            }
            self.sink.send(&bytes)?;
        }
        Ok(())
    }
}

/// Integrate a dynamic TempoSource up to `tick` to get wall-clock seconds.
/// Called only when `self.tempo.is_some()` (REQ-O10).
fn ticks_to_seconds_dynamic(src: &dyn crate::TempoSource, tick: u64, ppq: u16) -> f64 {
    let cps = src.change_points();
    if cps.is_empty() {
        let bpm = src.bpm_at(0) as f64;
        return (tick as f64 / ppq as f64) * 60.0 / bpm;
    }
    let mut seconds = 0.0;
    let mut last_tick = 0u64;
    let mut last_bpm = src.bpm_at(0) as f64;
    for &(t, bpm) in cps {
        if t >= tick {
            seconds += ((tick - last_tick) as f64 / ppq as f64) * 60.0 / last_bpm;
            return seconds;
        }
        seconds += ((t - last_tick) as f64 / ppq as f64) * 60.0 / last_bpm;
        last_tick = t;
        last_bpm = bpm as f64;
    }
    seconds += ((tick - last_tick) as f64 / ppq as f64) * 60.0 / last_bpm;
    seconds
}

/// Build a StaticTempoMap from the SMF's conductor track (track 0).
/// Used by the fallback arm when no user TempoSource is installed (REQ-O11).
fn static_map_from_conductor(smf: &midly::Smf<'_>, ppq: u16) -> StaticTempoMap {
    let mut map = StaticTempoMap { entries: vec![(0, 120.0)], ppq };
    if let Some(conductor) = smf.tracks.first() {
        let mut tick: u64 = 0;
        for event in conductor.iter() {
            tick += u64::from(event.delta.as_int());
            if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(us_per_beat)) = event.kind {
                // Malformed SMFs may declare 0 µs/beat; clamp to 1 to avoid +inf BPM
                // which propagates to Duration::from_secs_f64(+inf) and panics downstream.
                let us = us_per_beat.as_int().max(1);
                let bpm = 60_000_000.0 / us as f32;
                map.push(tick, bpm);
            }
        }
    }
    map
}

impl Drop for MidiPlayer {
    fn drop(&mut self) {
        // REQ availability-security: AllNotesOff CC 123 on all 16 channels.
        // Must execute even during panic unwind.
        for ch in 0u8..16 {
            let _ = self.sink.send(&[0xB0 | ch, 123, 0]);
        }
    }
}

// ─── PlaybackHandle ───────────────────────────────────────────────────────────

/// A handle to a background MIDI playback thread. Call [`stop()`](PlaybackHandle::stop)
/// to interrupt playback and wait for the thread to clean up (AllNotesOff is sent by
/// the player's `Drop` impl on the background thread).
pub struct PlaybackHandle {
    stop_tx: mpsc::Sender<()>,
    done_rx: mpsc::Receiver<()>,
}

impl PlaybackHandle {
    /// Send a stop signal and block until the background thread has finished.
    /// After this returns, all notes have been silenced.
    pub fn stop(self) {
        // Best-effort: ignore SendError (thread may have already finished).
        let _ = self.stop_tx.send(());
        // Wait for the thread to drop the player (which sends AllNotesOff).
        // Ignore RecvError — the channel closes when the thread exits.
        let _ = self.done_rx.recv();
    }
}
