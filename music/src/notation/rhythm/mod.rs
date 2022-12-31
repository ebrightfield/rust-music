use duration::Duration;
use crate::notation::rhythm::duration::{DurationKind, DurationTicks};
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::Voicing;
use crate::SoundedNote;

pub mod duration;
pub mod meter;

/// A pitch or voicing with a rhythmic duration.
pub struct RhythmicNotatedEvent<'a> {
    /// Whether the event is tied to a previous event, and thus
    /// would not be articulated.
    pub tied: bool,
    /// The data representing the notated event.
    pub event: NotatedEvent<'a>,
}

impl<'a> RhythmicNotatedEvent<'a> {
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

    pub fn fretted(sounded_note: SoundedNote<'a>, duration: Duration) -> Self {
        Self {
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::Fretted(sounded_note), duration)
        }
    }

    pub fn fretted_tied(sounded_note: SoundedNote<'a>, duration: Duration) -> Self {
        Self {
            tied: true,
            event: NotatedEvent::SingleEvent(SingleEvent::Fretted(sounded_note), duration)
        }
    }

    pub fn fretted_many(notes: Vec<SoundedNote<'a>>, duration: Duration) -> Self {
        Self {
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::FrettedMany(notes), duration)
        }
    }

    pub fn fretted_many_tied(notes: Vec<SoundedNote<'a>>, duration: Duration) -> Self {
        Self {
            tied: true,
            event: NotatedEvent::SingleEvent(SingleEvent::FrettedMany(notes), duration)
        }
    }

    /// The total duration of the event. In the case of a tuplet, this returns
    /// the real duration (i.e. quarter-note triplets would return 2 beats of ticks).
    pub fn duration(&self) -> DurationTicks {
        match &self.event {
            NotatedEvent::SingleEvent(_, duration) => duration.ticks(),
            NotatedEvent::Tuplet(tuplet) => tuplet.real_duration(),
        }
    }
}

/// A composition over single events and tuplets. You should never need to interact
/// with this type directly.
pub enum NotatedEvent<'a> {
    SingleEvent(SingleEvent<'a>, Duration),
    Tuplet(Tuplet<'a>),
}

/// A wrapper over the various musical events that can be engraved
/// after pairing with a duration.
pub enum SingleEvent<'a> {
    /// Single note, no fretboard information
    Pitch(Pitch),
    /// Multiple notes, no fretboard information
    Voicing(Voicing),
    /// Single note, with fretboard information
    Fretted(SoundedNote<'a>),
    /// Multiple notes, with fretboard information
    FrettedMany(Vec<SoundedNote<'a>>),
    /// Musical silence
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
pub struct Tuplet<'a> {
    /// A series of rhythmic events that reside inside the tuplet.
    /// Tuplets can be nested.
    pub events: Vec<RhythmicNotatedEvent<'a>>,
    /// The number of virtual `base_unit`.
    pub numerator: usize,
    /// The number of actual `base_unit`.
    pub denominator: usize,
    /// The "magnitude" of a tuplet. e.g. Eighth-note triplets are
    /// twice as short as quarter-note triplets.
    pub base_unit: DurationKind,
}

impl<'a> Tuplet<'a> {
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
        events: Vec<RhythmicNotatedEvent<'a>>,
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
    pub fn push(&mut self, event: RhythmicNotatedEvent<'a>) {
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

impl<'a> Into<RhythmicNotatedEvent<'a>> for Tuplet<'a> {
    fn into(self) -> RhythmicNotatedEvent<'a> {
        RhythmicNotatedEvent {
            tied: false,
            event: NotatedEvent::Tuplet(self)
        }
    }
}