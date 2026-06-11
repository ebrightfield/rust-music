// REQ-O12: local Dynamic enum and VelocityPolicy
// [AMEND-B] uses real music::notation::rhythm::RhythmicNotatedEvent import.

use music::notation::rhythm::RhythmicNotatedEvent;

/// Standard musical dynamics mapped to MIDI velocity values.
///
/// The mapping follows the General MIDI convention used by most DAWs:
/// `pp`=16, `p`=33, `mp`=49, `mf`=64, `f`=80, `ff`=96, `fff`=112.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dynamic {
    /// Pianississimo — very very soft (velocity 8).
    Ppp,
    /// Pianissimo — very soft (velocity 16).
    Pp,
    /// Piano — soft (velocity 33).
    P,
    /// Mezzo-piano — moderately soft (velocity 49).
    Mp,
    /// Mezzo-forte — moderately loud (velocity 64).
    Mf,
    /// Forte — loud (velocity 80).
    F,
    /// Fortissimo — very loud (velocity 96).
    Ff,
    /// Fortississimo — very very loud (velocity 112).
    Fff,
}

impl Dynamic {
    /// Return the MIDI velocity (0–127) that corresponds to this dynamic level.
    pub const fn velocity(self) -> u8 {
        // REQ-O12: mapping pp=16, p=33, mp=49, mf=64, f=80, ff=96, fff=112
        match self {
            Dynamic::Ppp => 8,
            Dynamic::Pp  => 16,
            Dynamic::P   => 33,
            Dynamic::Mp  => 49,
            Dynamic::Mf  => 64,
            Dynamic::F   => 80,
            Dynamic::Ff  => 96,
            Dynamic::Fff => 112,
        }
    }
}

/// Controls how MIDI NoteOn velocity is assigned during conversion.
pub enum VelocityPolicy {
    /// Use a fixed velocity for every note (0–127).
    Fixed(u8),
    /// Map a [`Dynamic`] level to its standard MIDI velocity.
    FromDynamic(Dynamic),
    /// Compute velocity per [`RhythmicNotatedEvent`] via a closure.
    ///
    /// The closure receives the event being converted and returns a velocity 0–127.
    PerEvent(Box<dyn Fn(&RhythmicNotatedEvent<'_>) -> u8 + Send + Sync>),
}

/// Default velocity used by call sites that evaluate [`VelocityPolicy`] outside
/// a [`RhythmicNotatedEvent`] context (matches the PerEvent fallback).
pub const DEFAULT_VELOCITY: u8 = 80;

impl VelocityPolicy {
    /// Resolve a velocity without a [`RhythmicNotatedEvent`] in scope.
    ///
    /// `PerEvent` policies fall back to [`DEFAULT_VELOCITY`] since there is no
    /// event to pass to the closure.
    pub fn velocity_no_event(&self) -> u8 {
        match self {
            VelocityPolicy::Fixed(v) => *v,
            VelocityPolicy::FromDynamic(d) => d.velocity(),
            VelocityPolicy::PerEvent(_) => DEFAULT_VELOCITY,
        }
    }

    /// Resolve a velocity with a [`RhythmicNotatedEvent`] in scope.
    pub fn velocity_for(&self, rne: &RhythmicNotatedEvent<'_>) -> u8 {
        match self {
            VelocityPolicy::Fixed(v) => *v,
            VelocityPolicy::FromDynamic(d) => d.velocity(),
            VelocityPolicy::PerEvent(f) => f(rne),
        }
    }
}
