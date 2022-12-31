use duration::Duration;
use crate::notation::rhythm::duration::{DurationKind, DurationTicks};
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::Voicing;

pub mod duration;
pub mod meter;

/// A pitch or voicing with a rhythmic duration.
pub struct RhythmicNotatedEvent {
    /// Whether the event is tied to a previous event, and thus
    /// would not be articulated.
    pub tied: bool,
    /// The data representing the notated event.
    pub event: NotatedEvent,
}

impl RhythmicNotatedEvent {
    pub fn pitch(pitch: Pitch, duration: Duration) -> Self {
        Self {
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::Pitch(pitch), duration)
        }
    }

    pub fn pitch_tied(pitch: Pitch, duration: Duration) -> Self {
        Self {
            tied: true,
            event: NotatedEvent::SingleEvent(SingleEvent::Pitch(pitch), duration)
        }
    }

    pub fn voicing(voicing: Voicing, duration: Duration) -> Self {
        Self {
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::Voicing(voicing), duration)
        }
    }

    pub fn voicing_tied(voicing: Voicing, duration: Duration) -> Self {
        Self {
            tied: true,
            event: NotatedEvent::SingleEvent(SingleEvent::Voicing(voicing), duration)
        }
    }

    pub fn rest(duration: Duration) -> Self {
        Self {
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::Rest, duration)
        }
    }

    pub fn duration(&self) -> DurationTicks {
        match &self.event {
            NotatedEvent::SingleEvent(_, duration) => duration.ticks(),
            NotatedEvent::Tuplet(tuplet) => tuplet.real_duration(),
        }
    }
}

pub enum NotatedEvent {
    SingleEvent(SingleEvent, Duration),
    Tuplet(Tuplet),
}

pub enum SingleEvent {
    Pitch(Pitch),
    Voicing(Voicing),
    Rest,
}

/// Tuples satisfy the need to represent divisions of time in ratios other than
/// the usual "nested halvings" of whole, half, quarter, eighth notes, etc.
///
/// In general, a tuple has two properties -- a ratio, and a `base_unit` magnitude
/// (written as a [DurationKind]).
/// They can be understood roughly as, "putting a `numerator * base_unit`
/// worth of time in the space of an actual `denominator * base_unit` worth of time."
///
/// Usually the ratio is implied for the most common tuplets. Triplets are a 3/2 ratio,
/// and we speak of "eighth note triplets" to denote the magnitude. Similarly,
/// quintuplets are a 5/4 ratio, and we speak of "quarter-note quintuplets" and so forth.
pub struct Tuplet {
    /// A series of rhythmic events that reside inside the tuplet.
    /// Tuplets can be nested.
    pub events: Vec<RhythmicNotatedEvent>,
    /// The number of virtual `base_unit`.
    pub numerator: usize,
    /// The number of actual `base_unit`.
    pub denominator: usize,
    /// The "magnitude" of a tuplet. e.g. Eighth-note triplets are
    /// twice as short as quarter-note triplets.
    pub base_unit: DurationKind,
}

impl Tuplet {
    /// Constructor for dynamically populating a tuplet with its elements.
    pub fn empty(numerator: usize, denominator: usize, base_unit: DurationKind) -> Self {
        Self {
            events: vec![],
            numerator,
            denominator,
            base_unit
        }
    }

    /// Constructor when you have pre-existing events.
    pub fn new(
        events: Vec<RhythmicNotatedEvent>,
        numerator: usize,
        denominator: usize,
        base_unit: DurationKind
    ) -> Self {
        Self {
            events,
            numerator,
            denominator,
            base_unit
        }
    }

    /// Push a new event into the tuplet
    pub fn push(&mut self, event: RhythmicNotatedEvent) {
        self.events.push(event);
    }

    /// The total duration of the tuplet's events. If a tuplet is complete,
    /// this value will be equal to `self.virtual_duration()`.
    pub fn events_duration(&self) -> DurationTicks {
        self.events.iter().map(|event| event.duration()).sum()
    }

    /// "Inside" of the tuplet's space, there is this virtual duration
    pub fn virtual_duration(&self) -> DurationTicks {
        let base_ticks: DurationTicks = self.base_unit.into();
        base_ticks * self.numerator
    }

    /// "Outside" the tuplet's space, the tuplet spans the same duration
    /// as its denominator.
    pub fn real_duration(&self) -> DurationTicks {
        let base_ticks: DurationTicks = self.base_unit.into();
        base_ticks * self.denominator
    }

    /// If a tuplet is complete, all of its events occupy the required amount of
    /// virtual time.
    pub fn is_complete(&self) -> bool {
        self.events_duration() == self.virtual_duration()
    }
}

impl Into<RhythmicNotatedEvent> for Tuplet {
    fn into(self) -> RhythmicNotatedEvent {
        RhythmicNotatedEvent {
            tied: false,
            event: NotatedEvent::Tuplet(self)
        }
    }
}