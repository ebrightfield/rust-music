use crate::notation::duration::Duration;
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::Voicing;

// TODO A better way would be to enumerate the possible durations
//    that are atomically engravable, then make a special constructor
//    that potentially returns more than one value, tied, and metrically
//    sanitized.
/// A temporal duration, denoted in units of 32nd notes.
/// This library operates at a 32nd note "resolution",
/// but can go shorter when factoring in tuplets.
pub type DurationIn32ndNotes = u8;

/// Creates a Vector of [DurationIn32ndNotes] marking
/// which 32nd note "ticks" are metrically prominent, or made prominent by choice.
/// e.g. This would convert 6/8 time signature to [vec![0, 12]],
/// as the first and fourth 8th notes in that signature are the strong beats.
pub fn get_big_beats(
    num_beats: NumBeats,
    base_unit_duration: DurationIn32ndNotes,
    beat_pattern: Option<Vec<DurationIn32ndNotes>>,
) -> Vec<DurationIn32ndNotes> {
    if let Some(pattern) = beat_pattern {
        return pattern;
    }
    // Compound meters and 4/4
    for divisor in [7, 5, 3, 2] {
        if num_beats.rem_euclid(divisor) == 0 && num_beats != divisor {
            let divided = num_beats / divisor;
            return (0u8..divided)
                .map(|i| i * divisor * base_unit_duration)
                .collect();
        }
    }
    // Some default patterns for 7/X, 7/X, 11/X, 13/X.
    if num_beats == 7 {
        return vec![0, 4 * base_unit_duration];
    }
    if num_beats == 5 {
        return vec![0, 3 * base_unit_duration];
    }
    if num_beats == 11 {
        return vec![
            0,
            3 * base_unit_duration,
            6 * base_unit_duration,
            9 * base_unit_duration,
        ];
    }
    if num_beats == 13 {
        return vec![
            0,
            3 * base_unit_duration,
            6 * base_unit_duration,
            9 * base_unit_duration,
            11 * base_unit_duration,
        ];
    }
    // Remaining patterns mark strong beats with the denominator
    // of qtr, half, or whole note
    if vec![8, 16, 32].contains(&base_unit_duration) {
        return (0..num_beats).map(|i| i * base_unit_duration).collect();
    }
    // Remaining <=8th note meters patterns, leave empty
    vec![]
}

/// Returns a Vec of [DurationIn32ndNotes] representing the
/// amount of time between temporally adjacent elements.
pub fn big_beats_to_durations(
    big_beats: Vec<DurationIn32ndNotes>,
    total_duration: u8,
) -> Vec<DurationIn32ndNotes> {
    let mut beats = big_beats.clone();
    beats.push(total_duration);
    beats.as_slice().windows(2).map(|w| w[1] - w[0]).collect()
}

/// The only valid units in the denominator of a time signature.
pub enum MeterDenominator {
    /// Whole-note gets the beat.
    One,
    /// Half-note gets the beat.
    Two,
    /// Quarter-note gets the beat.
    Four,
    /// Eighth-note gets the beat.
    Eight,
    /// Sixteenth-note gets the beat.
    Sixteen,
}

impl ToString for MeterDenominator {
    fn to_string(&self) -> String {
        match &self {
            MeterDenominator::One => "1".to_string(),
            MeterDenominator::Two => "2".to_string(),
            MeterDenominator::Four => "4".to_string(),
            MeterDenominator::Eight => "8".to_string(),
            MeterDenominator::Sixteen => "16".to_string(),
        }
    }
}

impl From<&MeterDenominator> for DurationIn32ndNotes {
    fn from(value: &MeterDenominator) -> DurationIn32ndNotes {
        match &value {
            MeterDenominator::One => 32,
            MeterDenominator::Two => 16,
            MeterDenominator::Four => 8,
            MeterDenominator::Eight => 4,
            MeterDenominator::Sixteen => 2,
        }
    }
}

impl MeterDenominator {
    /// Converts the associated rhythmic value into a [Duration
    pub fn duration_in_32nd_notes(&self) -> DurationIn32ndNotes {
        match &self {
            MeterDenominator::One => 32,
            MeterDenominator::Two => 16,
            MeterDenominator::Four => 8,
            MeterDenominator::Eight => 4,
            MeterDenominator::Sixteen => 2,
        }
    }
}

/// A time signature, accompanied with an accent pattern/"big beats"/"groove".
///
/// Meter subdivisions take a natural heirarchy of psychological salience,
/// with a bias toward the wider and more evenly spaced beats in the heirarchy.
/// This is the origin of the term "big beat", and it can be thought of as a kind of
/// rhythmic middle-ground between that of the measure as a whole, and the beat grid.
pub struct Meter {
    /// Numerator of a time signature, as is.
    pub num_beats: u8,
    /// Denominator of a time signature,
    /// but represented as a duration of 32nd notes.
    pub denominator: MeterDenominator,
    // TODO DurationTicks
    /// Vec of durations between the "big beats" in a time signature or groove pattern.
    pub beat_pattern: Vec<DurationIn32ndNotes>,
}

/// Any duration denominated in quarter-note "beats". We arbitrarily use a quarter-note
/// grid, even though we still account for e.g. meters like 4/8.
pub type NumBeats = u8;

impl Meter {
    /// Takes the numerator and demoninator of a typical non-additive meter,
    /// and optionally, an accent pattern. A default accent pattern is inferred
    /// for various meters.
    pub fn new(
        numerator: NumBeats,
        denominator: MeterDenominator,
        beat_pattern: Option<Vec<DurationIn32ndNotes>>,
    ) -> Self {
        let beat_duration: DurationIn32ndNotes = denominator.duration_in_32nd_notes();
        let big_beats = get_big_beats(numerator, beat_duration, beat_pattern.clone());
        let total_duration = beat_duration * numerator;
        let beat_pattern = big_beats_to_durations(big_beats, total_duration);
        Self {
            num_beats: numerator,
            denominator,
            beat_pattern,
        }
    }
}

/// A pitch or voicing with a rhythmic duration.
pub struct RhythmicNotatedEvent {
    /// Maximum duration is a double-whole-note
    pub duration: Duration,
    /// Whether the event is tied to a previous event, and thus
    /// would not be articulated.
    pub tied: bool,
    /// The data representing the notated event.
    pub event: NotatedEvent,
}

impl RhythmicNotatedEvent {
    pub fn pitch(pitch: Pitch, duration: Duration) -> Self {
        Self {
            duration,
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::Pitch(pitch))
        }
    }

    pub fn pitch_tied(pitch: Pitch, duration: Duration) -> Self {
        Self {
            duration,
            tied: true,
            event: NotatedEvent::SingleEvent(SingleEvent::Pitch(pitch))
        }
    }

    pub fn voicing(voicing: Voicing, duration: Duration) -> Self {
        Self {
            duration,
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::Voicing(voicing))
        }
    }

    pub fn voicing_tied(voicing: Voicing, duration: Duration) -> Self {
        Self {
            duration,
            tied: true,
            event: NotatedEvent::SingleEvent(SingleEvent::Voicing(voicing))
        }
    }

    pub fn rest(duration: Duration) -> Self {
        Self {
            duration,
            tied: false,
            event: NotatedEvent::SingleEvent(SingleEvent::Rest)
        }
    }
}

pub enum NotatedEvent {
    SingleEvent(SingleEvent),
    Tuple(Vec<SingleEvent>),
}

pub enum SingleEvent {
    Pitch(Pitch),
    Voicing(Voicing),
    Rest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_big_beats() {
        // 4/4
        let result = get_big_beats(4, 8, None);
        assert_eq!(result, vec![0, 16]);
        // 3/4
        let result = get_big_beats(3, 8, None);
        assert_eq!(result, vec![0, 8, 16]);
        // 5/4
        let result = get_big_beats(5, 8, None);
        assert_eq!(result, vec![0, 24]);
        // 5/4 - 2 + 3 beat pattern
        let result = get_big_beats(5, 8, Some(vec![0, 16]));
        assert_eq!(result, vec![0, 16]);
    }
}
