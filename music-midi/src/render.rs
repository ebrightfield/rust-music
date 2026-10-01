//! Offline audio rendering via `oxisynth` and `hound`.
//!
//! Requires the `render` feature (`music-midi = { features = ["render"] }`).
//!
//! Use [`AudioRenderer`] to convert an SMF (from [`SmfBuilder`](crate::smf::SmfBuilder))
//! into a WAV file using an SF2 SoundFont. The render loop is block-by-block
//! at the requested sample rate; a 1-second tail is appended to let notes decay.
// REQ-O8, O11, O13: AudioRenderer — drives oxisynth block-by-block, writes WAV via hound.
// Phase 5 (5e, 5f, 5g)

use crate::{
    error::MidiConversionError,
    soundfont::{HoundWav, OxiSynthAdapter, SoundFont, Synthesizer, WavSink},
    tempo::StaticTempoMap,
    AbsoluteTicks, DEFAULT_PPQ,
};
use midly::{MetaMessage, TrackEventKind};
use std::path::Path;

// ---------------------------------------------------------------------------
// AudioRenderer
// ---------------------------------------------------------------------------

/// Renders MIDI data to a WAV file using an SF2 SoundFont.
///
/// ```ignore
/// let sf = SoundFont::general_user_gs()?;
/// AudioRenderer::new(sf)?.sample_rate(48_000).render_to_wav(&smf, "out.wav")?;
/// ```
pub struct AudioRenderer {
    sf_bytes: Vec<u8>,
    sample_rate: u32,
    tail_seconds: f32,
}

impl AudioRenderer {
    /// Construct a renderer from a loaded `SoundFont`.
    pub fn new(sf: SoundFont) -> Result<Self, MidiConversionError> {
        Ok(Self {
            sf_bytes: sf.bytes,
            sample_rate: 48_000,
            tail_seconds: 1.0,
        })
    }

    /// Override the output sample rate (default: 48 000 Hz).
    pub fn sample_rate(mut self, hz: u32) -> Self {
        self.sample_rate = hz;
        self
    }

    /// Override the tail duration appended after the last MIDI event (default: 1.0 s).
    ///
    /// The tail allows envelope releases to decay without clipping. Pass `0.0`
    /// for a hard stop; pass a larger value for long reverb tails.
    ///
    /// # Known Limitations (Tier C debt — D5)
    /// The tail is applied uniformly; there is no per-note envelope modeling.
    /// Values < 0.0 are clamped to 0.0 and NaN is treated as 0.0; no error is
    /// returned for invalid input. See `docs/spec-music-midi-debt.md` §4 R20-R22
    /// and §6 D5 for context.
    pub fn tail_seconds(mut self, secs: f32) -> Self {
        self.tail_seconds = secs;
        self
    }

    /// Render a parsed SMF to a WAV file at `path`.
    ///
    /// On success the WAV file is finalized. On any error the partial file is
    /// deleted so incomplete WAVs never survive.
    pub fn render_to_wav(
        &mut self,
        smf: &midly::Smf,
        path: impl AsRef<Path>,
    ) -> Result<(), MidiConversionError> {
        let path = path.as_ref();
        let synth = OxiSynthAdapter::new(&self.sf_bytes, self.sample_rate as f32)?;
        let wav = HoundWav::create(path, self.sample_rate)?;
        let result = render_smf(smf, synth, wav, self.sample_rate, self.tail_seconds);
        if result.is_err() {
            let _ = std::fs::remove_file(path);
        }
        result
    }

    /// Parse raw MIDI bytes and render to WAV.
    pub fn render_bytes_to_wav(
        &mut self,
        bytes: &[u8],
        path: impl AsRef<Path>,
    ) -> Result<(), MidiConversionError> {
        let smf = midly::Smf::parse(bytes).map_err(|e| MidiConversionError::Smf(e.to_string()))?;
        self.render_to_wav(&smf, path)
    }
}

// ---------------------------------------------------------------------------
// Core render loop — seam-injectable for tests
// ---------------------------------------------------------------------------

/// Represents a flat MIDI event with an absolute tick timestamp.
#[derive(Debug)]
struct AbsEvent {
    tick: AbsoluteTicks,
    kind: FlatKind,
}

#[derive(Debug)]
enum FlatKind {
    Note {
        channel: u8,
        key: u8,
        vel: u8,
        on: bool,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ControlChange {
        channel: u8,
        ctrl: u8,
        value: u8,
    },
    Tempo {
        micros_per_beat: u32,
    },
}

/// Collect absolute-tick events from all tracks, merge-sort by tick.
fn collect_events(smf: &midly::Smf) -> Vec<AbsEvent> {
    let mut events: Vec<AbsEvent> = Vec::new();
    for track in &smf.tracks {
        let mut tick: AbsoluteTicks = 0;
        for ev in track.iter() {
            tick += ev.delta.as_int() as AbsoluteTicks;
            let kind = match &ev.kind {
                TrackEventKind::Midi { channel, message } => {
                    let ch = u8::from(*channel);
                    match message {
                        midly::MidiMessage::NoteOn { key, vel } => {
                            let k = key.as_int();
                            let v = vel.as_int();
                            // Note-on with velocity 0 is treated as note-off per MIDI spec.
                            Some(FlatKind::Note {
                                channel: ch,
                                key: k,
                                vel: v,
                                on: v > 0,
                            })
                        }
                        midly::MidiMessage::NoteOff { key, .. } => {
                            let k = key.as_int();
                            Some(FlatKind::Note {
                                channel: ch,
                                key: k,
                                vel: 0,
                                on: false,
                            })
                        }
                        midly::MidiMessage::ProgramChange { program } => {
                            Some(FlatKind::ProgramChange {
                                channel: ch,
                                program: program.as_int(),
                            })
                        }
                        midly::MidiMessage::Controller { controller, value } => {
                            Some(FlatKind::ControlChange {
                                channel: ch,
                                ctrl: controller.as_int(),
                                value: value.as_int(),
                            })
                        }
                        _ => None,
                    }
                }
                TrackEventKind::Meta(MetaMessage::Tempo(micros)) => Some(FlatKind::Tempo {
                    micros_per_beat: micros.as_int(),
                }),
                _ => None,
            };
            if let Some(k) = kind {
                events.push(AbsEvent { tick, kind: k });
            }
        }
    }
    // Stable sort: preserve relative order within the same tick.
    events.sort_by_key(|e| e.tick);
    events
}

/// Build a `StaticTempoMap` from tempo events.
/// The map begins at 120 BPM (MIDI default) from tick 0.
fn build_tempo_map(events: &[AbsEvent], ppq: u16) -> StaticTempoMap {
    let mut map = StaticTempoMap {
        entries: vec![(0, 120.0)],
        ppq,
    };
    for ev in events {
        if let FlatKind::Tempo { micros_per_beat } = ev.kind {
            // Malformed SMFs may declare 0 µs/beat; clamp to 1 to avoid +inf BPM.
            let us = micros_per_beat.max(1);
            let bpm = 60_000_000.0 / us as f32;
            map.push(ev.tick, bpm);
        }
    }
    map
}

/// Core render: walk events, render audio between them, push to synth + WAV.
fn render_smf<S: Synthesizer, W: WavSink>(
    smf: &midly::Smf,
    mut synth: S,
    wav: W,
    sample_rate: u32,
    tail_seconds: f32,
) -> Result<(), MidiConversionError> {
    // Determine PPQ from the SMF header.
    let ppq = match smf.header.timing {
        midly::Timing::Metrical(t) => t.as_int(),
        midly::Timing::Timecode(_, _) => DEFAULT_PPQ,
    };

    let events = collect_events(smf);
    let tempo_map = build_tempo_map(&events, ppq);

    // Render audio block-by-block. Between each pair of adjacent events we
    // compute the wall-clock duration and render exactly that many frames.
    let mut current_tick: AbsoluteTicks = 0;
    let mut wav = Box::new(wav) as Box<dyn WavSink>;

    // Buffer for interleaved stereo f32 samples (BLOCK_FRAMES * 2 channels).
    const BLOCK_FRAMES: usize = 64;
    let mut buf = vec![0.0f32; BLOCK_FRAMES * 2];

    for ev in &events {
        let next_tick = ev.tick;
        if next_tick > current_tick {
            // Compute how many frames to render for [current_tick, next_tick).
            let t0 = tempo_map.ticks_to_seconds(current_tick);
            let t1 = tempo_map.ticks_to_seconds(next_tick);
            let frames_needed = ((t1 - t0) * sample_rate as f64).round() as usize;
            render_frames(
                &mut synth,
                wav.as_mut(),
                &mut buf,
                frames_needed,
                BLOCK_FRAMES,
            )?;
            current_tick = next_tick;
        }

        // Dispatch the event to the synthesizer.
        match &ev.kind {
            FlatKind::Note {
                channel,
                key,
                vel,
                on,
            } => {
                synth.handle_event(crate::MidiEvent {
                    time: ev.tick,
                    channel: *channel,
                    message: if *on {
                        crate::MidiMessage::NoteOn {
                            key: *key,
                            velocity: *vel,
                        }
                    } else {
                        crate::MidiMessage::NoteOff {
                            key: *key,
                            velocity: 0,
                        }
                    },
                });
            }
            FlatKind::ProgramChange { channel, program } => {
                synth.handle_event(crate::MidiEvent {
                    time: ev.tick,
                    channel: *channel,
                    message: crate::MidiMessage::ProgramChange(*program),
                });
            }
            FlatKind::ControlChange {
                channel,
                ctrl,
                value,
            } => {
                synth.handle_event(crate::MidiEvent {
                    time: ev.tick,
                    channel: *channel,
                    message: crate::MidiMessage::ControlChange {
                        controller: *ctrl,
                        value: *value,
                    },
                });
            }
            FlatKind::Tempo { .. } => {
                // Already consumed in tempo_map; no synth action needed.
            }
        }
    }

    // Render a tail after the last event to let notes decay.
    // Clamp negative/NaN to zero — no error, just a hard stop.
    let clamped = if tail_seconds.is_nan() || tail_seconds < 0.0 {
        0.0
    } else {
        tail_seconds as f64
    };
    let tail_frames = (clamped * sample_rate as f64).round() as usize;
    render_frames(
        &mut synth,
        wav.as_mut(),
        &mut buf,
        tail_frames,
        BLOCK_FRAMES,
    )?;

    wav.finalize()
}

/// Render exactly `total_frames` interleaved stereo f32 frames into `wav`,
/// processing in chunks of `block` frames at a time.
fn render_frames<S: Synthesizer + ?Sized, W: WavSink + ?Sized>(
    synth: &mut S,
    wav: &mut W,
    buf: &mut Vec<f32>,
    total_frames: usize,
    block: usize,
) -> Result<(), MidiConversionError> {
    let mut remaining = total_frames;
    while remaining > 0 {
        let n = remaining.min(block);
        let needed = n * 2;
        if buf.len() < needed {
            buf.resize(needed, 0.0);
        }
        // Zero before each block to avoid stale data in final partial block.
        for x in buf[..needed].iter_mut() {
            *x = 0.0;
        }
        synth.write_block(&mut buf[..needed]);
        for i in 0..n {
            let l_f = buf[i * 2].clamp(-1.0, 1.0);
            let r_f = buf[i * 2 + 1].clamp(-1.0, 1.0);
            let l = (l_f * i16::MAX as f32) as i16;
            let r = (r_f * i16::MAX as f32) as i16;
            wav.write_frame(l, r)?;
        }
        remaining -= n;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Test-facing helpers (seam injection)
// These are compiled whenever the `render` feature is active so that both
// in-crate unit tests and the external integration tests can use them.
// ---------------------------------------------------------------------------

/// Render using injected seam impls — used by tests that don't need a real
/// SoundFont or real disk I/O.
pub fn render_smf_with_seams<S: Synthesizer, W: WavSink>(
    smf: &midly::Smf,
    synth: S,
    wav: W,
    sample_rate: u32,
) -> Result<(), MidiConversionError> {
    render_smf(smf, synth, wav, sample_rate, 1.0)
}

/// Like [`render_smf_with_seams`] but with an explicit tail duration.
pub fn render_smf_with_seams_tail<S: Synthesizer, W: WavSink>(
    smf: &midly::Smf,
    synth: S,
    wav: W,
    sample_rate: u32,
    tail_seconds: f32,
) -> Result<(), MidiConversionError> {
    render_smf(smf, synth, wav, sample_rate, tail_seconds)
}

/// Build a minimal SMF in memory containing a single-channel C-major triad
/// held for half a second at 120 BPM, 480 PPQ. Useful in tests and examples.
///
/// Returns [`MidiConversionError::Smf`] if the underlying `midly` writer fails.
pub fn c_triad_smf_bytes() -> Result<Vec<u8>, MidiConversionError> {
    use midly::{Format, Header, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
    use std::io::Cursor;

    let ppq = 480u16;
    // At 120 BPM, 480 PPQ → half second = 240 ticks.
    let half_sec_ticks = 240u32;

    let mut track: Vec<TrackEvent> = Vec::new();

    // Tempo: 500 000 µs/beat (120 BPM)
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(500_000u32.into())),
    });

    // Note-on: C4 (60), E4 (64), G4 (67)
    for &key in &[60u8, 64, 67] {
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::NoteOn {
                    key: key.into(),
                    vel: 80u8.into(),
                },
            },
        });
    }

    // Note-off after half_sec_ticks
    let mut first = true;
    for &key in &[60u8, 64, 67] {
        track.push(TrackEvent {
            delta: if first {
                half_sec_ticks.into()
            } else {
                0.into()
            },
            kind: TrackEventKind::Midi {
                channel: 0.into(),
                message: MidiMessage::NoteOff {
                    key: key.into(),
                    vel: 0u8.into(),
                },
            },
        });
        first = false;
    }

    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(ppq.into())),
        tracks: vec![track],
    };
    let mut out = Vec::new();
    smf.write_std(Cursor::new(&mut out))
        .map_err(|e| MidiConversionError::Smf(e.to_string()))?;
    Ok(out)
}
