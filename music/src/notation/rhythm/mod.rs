use duration::Duration;
use crate::error::MusicSemanticsError;
use crate::notation::rhythm::duration::{DurationKind, DurationTicks};
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::Voicing;
use crate::SoundedNote;

pub mod beat_grid;
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

    /// Create a triplet (3 notes in the time of 2).
    ///
    /// # Arguments
    /// * `events` - Exactly 3 rhythmic events
    /// * `base_unit` - The duration kind for each note (e.g., `Eighth` for eighth-note triplets)
    ///
    /// # Example
    /// ```ignore
    /// // Eighth-note triplet of three C4 pitches
    /// let events = vec![
    ///     RhythmicNotatedEvent::pitch(c4.clone(), Duration::new(DurationKind::Eighth, 0)),
    ///     RhythmicNotatedEvent::pitch(c4.clone(), Duration::new(DurationKind::Eighth, 0)),
    ///     RhythmicNotatedEvent::pitch(c4.clone(), Duration::new(DurationKind::Eighth, 0)),
    /// ];
    /// let triplet = Tuplet::triplet(events, DurationKind::Eighth)?;
    /// ```
    pub fn triplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base_unit: DurationKind,
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 3 {
            return Err(MusicSemanticsError::InvalidTuplet(
                format!("Triplet must have exactly 3 events, got {}", events.len())
            ));
        }
        Ok(Self::new(events, 3, 2, base_unit))
    }

    /// Create a duplet (2 notes in the time of 3).
    /// Common in compound meters (e.g., 6/8, 9/8).
    ///
    /// # Arguments
    /// * `events` - Exactly 2 rhythmic events
    /// * `base_unit` - The duration kind for each note
    pub fn duplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base_unit: DurationKind,
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 2 {
            return Err(MusicSemanticsError::InvalidTuplet(
                format!("Duplet must have exactly 2 events, got {}", events.len())
            ));
        }
        Ok(Self::new(events, 2, 3, base_unit))
    }

    /// Create a quintuplet (5 notes in the time of 4).
    ///
    /// # Arguments
    /// * `events` - Exactly 5 rhythmic events
    /// * `base_unit` - The duration kind for each note
    pub fn quintuplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base_unit: DurationKind,
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 5 {
            return Err(MusicSemanticsError::InvalidTuplet(
                format!("Quintuplet must have exactly 5 events, got {}", events.len())
            ));
        }
        Ok(Self::new(events, 5, 4, base_unit))
    }

    /// Create a sextuplet (6 notes in the time of 4).
    ///
    /// # Arguments
    /// * `events` - Exactly 6 rhythmic events
    /// * `base_unit` - The duration kind for each note
    pub fn sextuplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base_unit: DurationKind,
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 6 {
            return Err(MusicSemanticsError::InvalidTuplet(
                format!("Sextuplet must have exactly 6 events, got {}", events.len())
            ));
        }
        Ok(Self::new(events, 6, 4, base_unit))
    }

    /// Create a septuplet (7 notes in the time of 4).
    ///
    /// # Arguments
    /// * `events` - Exactly 7 rhythmic events
    /// * `base_unit` - The duration kind for each note
    pub fn septuplet(
        events: Vec<RhythmicNotatedEvent<'a>>,
        base_unit: DurationKind,
    ) -> Result<Self, MusicSemanticsError> {
        if events.len() != 7 {
            return Err(MusicSemanticsError::InvalidTuplet(
                format!("Septuplet must have exactly 7 events, got {}", events.len())
            ));
        }
        Ok(Self::new(events, 7, 4, base_unit))
    }

    /// Validate that the tuplet's events fill the expected virtual duration.
    /// Returns an error if the events don't sum to the correct duration.
    pub fn validate(&self) -> Result<(), MusicSemanticsError> {
        let events_dur = self.events_duration();
        let expected_dur = self.virtual_duration();
        if events_dur != expected_dur {
            return Err(MusicSemanticsError::InvalidTuplet(
                format!(
                    "Tuplet events sum to {} ticks, expected {} ticks",
                    events_dur, expected_dur
                )
            ));
        }
        Ok(())
    }
}

impl<'a> From<Tuplet<'a>> for RhythmicNotatedEvent<'a> {
    fn from(tuplet: Tuplet<'a>) -> Self {
        RhythmicNotatedEvent {
            tied: false,
            event: NotatedEvent::Tuplet(tuplet)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::note::Note;

    fn make_pitch_event(duration_kind: DurationKind) -> RhythmicNotatedEvent<'static> {
        let pitch = Pitch::new(Note::C, 4).unwrap();
        RhythmicNotatedEvent::pitch(pitch, Duration::new(duration_kind, 0))
    }

    fn make_rest_event(duration_kind: DurationKind) -> RhythmicNotatedEvent<'static> {
        RhythmicNotatedEvent::rest(Duration::new(duration_kind, 0))
    }

    #[test]
    fn test_triplet_creation() {
        let events = vec![
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
        ];
        let triplet = Tuplet::triplet(events, DurationKind::Eighth).unwrap();

        assert_eq!(triplet.numerator, 3);
        assert_eq!(triplet.denominator, 2);
        assert_eq!(triplet.events.len(), 3);
        assert!(triplet.is_complete());

        // Real duration is 2 eighth notes (32 ticks)
        assert_eq!(triplet.real_duration(), 32);
        // Virtual duration is 3 eighth notes (48 ticks)
        assert_eq!(triplet.virtual_duration(), 48);
    }

    #[test]
    fn test_triplet_wrong_count() {
        let events = vec![
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
        ];
        let result = Tuplet::triplet(events, DurationKind::Eighth);
        assert!(result.is_err());
        assert!(matches!(result, Err(MusicSemanticsError::InvalidTuplet(_))));
    }

    #[test]
    fn test_duplet_creation() {
        let events = vec![
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
        ];
        let duplet = Tuplet::duplet(events, DurationKind::Eighth).unwrap();

        assert_eq!(duplet.numerator, 2);
        assert_eq!(duplet.denominator, 3);
        assert_eq!(duplet.events.len(), 2);

        // Real duration is 3 eighth notes (48 ticks)
        assert_eq!(duplet.real_duration(), 48);
        // Virtual duration is 2 eighth notes (32 ticks)
        assert_eq!(duplet.virtual_duration(), 32);
    }

    #[test]
    fn test_duplet_wrong_count() {
        let events = vec![
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
        ];
        let result = Tuplet::duplet(events, DurationKind::Eighth);
        assert!(result.is_err());
    }

    #[test]
    fn test_quintuplet_creation() {
        let events = vec![
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
        ];
        let quintuplet = Tuplet::quintuplet(events, DurationKind::Sixteenth).unwrap();

        assert_eq!(quintuplet.numerator, 5);
        assert_eq!(quintuplet.denominator, 4);
        assert_eq!(quintuplet.events.len(), 5);

        // Real duration is 4 sixteenth notes (32 ticks)
        assert_eq!(quintuplet.real_duration(), 32);
        // Virtual duration is 5 sixteenth notes (40 ticks)
        assert_eq!(quintuplet.virtual_duration(), 40);
    }

    #[test]
    fn test_sextuplet_creation() {
        let events = vec![
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
        ];
        let sextuplet = Tuplet::sextuplet(events, DurationKind::Sixteenth).unwrap();

        assert_eq!(sextuplet.numerator, 6);
        assert_eq!(sextuplet.denominator, 4);
        assert_eq!(sextuplet.events.len(), 6);
    }

    #[test]
    fn test_septuplet_creation() {
        let events = vec![
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
            make_pitch_event(DurationKind::Sixteenth),
        ];
        let septuplet = Tuplet::septuplet(events, DurationKind::Sixteenth).unwrap();

        assert_eq!(septuplet.numerator, 7);
        assert_eq!(septuplet.denominator, 4);
        assert_eq!(septuplet.events.len(), 7);
    }

    #[test]
    fn test_tuplet_validate_complete() {
        let events = vec![
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
        ];
        let triplet = Tuplet::triplet(events, DurationKind::Eighth).unwrap();
        assert!(triplet.validate().is_ok());
    }

    #[test]
    fn test_tuplet_validate_incomplete() {
        // Create a triplet with only 2 eighth notes of content
        let mut triplet = Tuplet::empty(3, 2, DurationKind::Eighth);
        triplet.push(make_pitch_event(DurationKind::Eighth));
        triplet.push(make_pitch_event(DurationKind::Eighth));

        assert!(!triplet.is_complete());
        assert!(triplet.validate().is_err());
    }

    #[test]
    fn test_quarter_note_triplet() {
        // Quarter note triplets - 3 quarters in time of 2
        let events = vec![
            make_pitch_event(DurationKind::Qtr),
            make_pitch_event(DurationKind::Qtr),
            make_pitch_event(DurationKind::Qtr),
        ];
        let triplet = Tuplet::triplet(events, DurationKind::Qtr).unwrap();

        // Real duration is 2 quarter notes (64 ticks)
        assert_eq!(triplet.real_duration(), 64);
        // Virtual duration is 3 quarter notes (96 ticks)
        assert_eq!(triplet.virtual_duration(), 96);
    }

    #[test]
    fn test_triplet_with_rest() {
        // Triplet with a rest in the middle
        let events = vec![
            make_pitch_event(DurationKind::Eighth),
            make_rest_event(DurationKind::Eighth),
            make_pitch_event(DurationKind::Eighth),
        ];
        let triplet = Tuplet::triplet(events, DurationKind::Eighth).unwrap();

        assert!(triplet.is_complete());
        assert!(triplet.validate().is_ok());
    }
}