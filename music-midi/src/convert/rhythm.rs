// REQ-O3, O4, O17: RhythmicNotatedEvent incl. ties + tuplets + Fretted/FrettedMany/Rest coverage
use super::{ConvertCtx, ToMidiEvents};
use crate::{
    error::MidiConversionError,
    event::{AbsoluteTicks, MidiEvent, MidiMessage},
};
use music::notation::rhythm::duration::Duration;
use music::notation::rhythm::{NotatedEvent, RhythmicNotatedEvent, SingleEvent, Tuplet};

fn single_event_pitches<'a>(s: &'a SingleEvent<'a>) -> Vec<u8> {
    match s {
        SingleEvent::Pitch(p) => vec![p.midi_note],
        SingleEvent::Voicing(v) => v.iter().map(|p| p.midi_note).collect(),
        SingleEvent::Fretted(sn) => vec![sn.pitch.midi_note],
        SingleEvent::FrettedMany(v) => v.iter().map(|sn| sn.pitch.midi_note).collect(),
        SingleEvent::Rest => vec![],
    }
}

fn emit_single(
    rne: &RhythmicNotatedEvent<'_>,
    duration: &Duration,
    base_tick: AbsoluteTicks,
    channel: u8,
    ctx: &ConvertCtx<'_>,
    out: &mut Vec<MidiEvent>,
) -> Result<AbsoluteTicks, MidiConversionError> {
    // [AMEND-E] Duration::ticks() returns usize (DurationTicks); no Into<u32>.
    // Tick values are bounded by the largest authored duration (breve = 256), well under u32::MAX.
    let music_ticks: u32 = duration.ticks() as u32;
    let dt = ctx.rescale_ticks(music_ticks);
    let keys = match &rne.event {
        NotatedEvent::SingleEvent(s, _) => single_event_pitches(s),
        NotatedEvent::Tuplet(_) => unreachable!(),
    };
    if keys.is_empty() {
        // REQ-O3: Rest advances time with no events
        return Ok(base_tick + dt);
    }
    for k in &keys {
        if *k > 127 {
            return Err(MidiConversionError::PitchOutOfRange(*k));
        }
    }
    let vel = ctx.velocity.velocity_for(rne);
    for k in &keys {
        out.push(MidiEvent {
            time: base_tick,
            channel,
            message: MidiMessage::NoteOn {
                key: *k,
                velocity: vel,
            },
        });
    }
    for k in &keys {
        out.push(MidiEvent {
            time: base_tick + dt,
            channel,
            message: MidiMessage::NoteOff {
                key: *k,
                velocity: 64,
            },
        });
    }
    Ok(base_tick + dt)
}

impl<'a> ToMidiEvents for RhythmicNotatedEvent<'a> {
    fn append_midi(
        &self,
        base_tick: AbsoluteTicks,
        channel: u8,
        ctx: &ConvertCtx<'_>,
        out: &mut Vec<MidiEvent>,
    ) -> Result<AbsoluteTicks, MidiConversionError> {
        match &self.event {
            NotatedEvent::SingleEvent(_, dur) => {
                emit_single(self, dur, base_tick, channel, ctx, out)
            }
            NotatedEvent::Tuplet(t) => emit_tuplet(t, base_tick, channel, ctx, out),
        }
    }
}

// [AMEND-W3] Nested tuplets compound ratios: if outer is num_o:den_o and inner is num_i:den_i,
// each innermost child's effective scale is (den_o * den_i) / (num_o * num_i).
// We flatten via an explicit recursion that carries the accumulated (num_acc, den_acc) ratio,
// instead of recursing into a fresh `emit_tuplet` that only sees the inner ratio.
fn emit_tuplet<'a>(
    t: &Tuplet<'a>,
    base_tick: AbsoluteTicks,
    channel: u8,
    ctx: &ConvertCtx<'_>,
    out: &mut Vec<MidiEvent>,
) -> Result<AbsoluteTicks, MidiConversionError> {
    emit_tuplet_scaled(t, base_tick, channel, ctx, out, 1, 1)
}

/// `(num_acc, den_acc)` is the accumulated outer ratio. The effective ratio applied
/// to a child's `child_music_ticks` is `(num_acc * t.numerator) / (den_acc * t.denominator)`
/// inverted (i.e., we shrink the child by the ratio). Concretely the per-child MIDI
/// duration is computed by `tuplet_child_ticks_compound`.
fn emit_tuplet_scaled<'a>(
    t: &Tuplet<'a>,
    base_tick: AbsoluteTicks,
    channel: u8,
    ctx: &ConvertCtx<'_>,
    out: &mut Vec<MidiEvent>,
    num_acc: usize,
    den_acc: usize,
) -> Result<AbsoluteTicks, MidiConversionError> {
    let mut tick = base_tick;
    let cum_num = num_acc
        .checked_mul(t.numerator)
        .ok_or_else(|| arith_overflow(t))?;
    let cum_den = den_acc
        .checked_mul(t.denominator)
        .ok_or_else(|| arith_overflow(t))?;
    for child in &t.events {
        let child_music_ticks: u32 = match &child.event {
            NotatedEvent::SingleEvent(_, dur) => dur.ticks() as u32,
            NotatedEvent::Tuplet(nested) => nested.real_duration() as u32,
        };
        let scaled = tuplet_child_ticks_compound(child_music_ticks, cum_num, cum_den, ctx.ppq)?;
        match &child.event {
            NotatedEvent::SingleEvent(s, _) => {
                let keys = single_event_pitches(s);
                if !keys.is_empty() {
                    let vel = ctx.velocity.velocity_for(child);
                    for k in &keys {
                        out.push(MidiEvent {
                            time: tick,
                            channel,
                            message: MidiMessage::NoteOn {
                                key: *k,
                                velocity: vel,
                            },
                        });
                    }
                    for k in &keys {
                        out.push(MidiEvent {
                            time: tick + scaled,
                            channel,
                            message: MidiMessage::NoteOff {
                                key: *k,
                                velocity: 64,
                            },
                        });
                    }
                }
            }
            NotatedEvent::Tuplet(nested) => {
                // [AMEND-W3] recurse carrying the accumulated ratio so events land at
                // the compound-scaled positions, not at the inner-only-scaled positions.
                emit_tuplet_scaled(nested, tick, channel, ctx, out, cum_num, cum_den)?;
            }
        }
        tick += scaled;
    }
    Ok(tick)
}

fn arith_overflow<'a>(t: &Tuplet<'a>) -> MidiConversionError {
    MidiConversionError::TupletInexact {
        num: t.numerator as u32,
        den: t.denominator as u32,
        ppq: 0,
    }
}

/// [AMEND-W3] compound-ratio variant of `tuplet_child_ticks`. Replaces the per-call
/// (num, den) with an accumulated (cum_num, cum_den) so nested tuplets are correct.
fn tuplet_child_ticks_compound(
    base_music_ticks: u32,
    cum_num: usize,
    cum_den: usize,
    ppq: u16,
) -> Result<u64, MidiConversionError> {
    let n = cum_num as u128;
    let d = cum_den as u128;
    let midi = (base_music_ticks as u128) * (ppq as u128) * d / (32 * n);
    let remainder = ((base_music_ticks as u128) * (ppq as u128) * d) % (32 * n);
    if remainder != 0 {
        return Err(MidiConversionError::TupletInexact {
            num: cum_num as u32,
            den: cum_den as u32,
            ppq,
        });
    }
    Ok(midi as u64)
}

impl<'a> ToMidiEvents for [RhythmicNotatedEvent<'a>] {
    fn append_midi(
        &self,
        mut base_tick: AbsoluteTicks,
        channel: u8,
        ctx: &ConvertCtx<'_>,
        out: &mut Vec<MidiEvent>,
    ) -> Result<AbsoluteTicks, MidiConversionError> {
        // REQ-O4, X8: merge tied events by suppressing NoteOn and extending NoteOff
        let mut i = 0;
        while i < self.len() {
            let start = i;
            // Absorb any following events marked `tied` that share the pitch set.
            let mut j = i + 1;
            while j < self.len() && self[j].tied && same_pitches(&self[i], &self[j]) {
                j += 1;
            }
            // Sum durations from i..j into a synthetic emission.
            // [AMEND-E] explicit `as u32`.
            let mut merged_ticks: u64 = 0;
            for ev in &self[i..j] {
                let mt: u32 = match &ev.event {
                    NotatedEvent::SingleEvent(_, d) => d.ticks() as u32,
                    NotatedEvent::Tuplet(t) => t.real_duration() as u32,
                };
                merged_ticks += ctx.rescale_ticks(mt);
            }
            // Emit NoteOn at base_tick for first event's pitches, NoteOff at base + merged_ticks.
            let keys = match &self[i].event {
                NotatedEvent::SingleEvent(s, _) => single_event_pitches(s),
                NotatedEvent::Tuplet(_) => {
                    // REQ-O3: tied-through-tuplet is out of scope; fall through per-event
                    self[i].append_midi(base_tick, channel, ctx, out)?;
                    base_tick += merged_ticks;
                    i = j;
                    continue;
                }
            };
            if !keys.is_empty() {
                let vel = ctx.velocity.velocity_for(&self[i]);
                for k in &keys {
                    out.push(MidiEvent {
                        time: base_tick,
                        channel,
                        message: MidiMessage::NoteOn {
                            key: *k,
                            velocity: vel,
                        },
                    });
                }
                for k in &keys {
                    out.push(MidiEvent {
                        time: base_tick + merged_ticks,
                        channel,
                        message: MidiMessage::NoteOff {
                            key: *k,
                            velocity: 64,
                        },
                    });
                }
            }
            base_tick += merged_ticks;
            let _ = start; // retained for future debug assertions
            i = j;
        }
        Ok(base_tick)
    }
}

fn same_pitches(a: &RhythmicNotatedEvent<'_>, b: &RhythmicNotatedEvent<'_>) -> bool {
    let (NotatedEvent::SingleEvent(sa, _), NotatedEvent::SingleEvent(sb, _)) = (&a.event, &b.event)
    else {
        return false;
    };
    single_event_pitches(sa) == single_event_pitches(sb)
}
