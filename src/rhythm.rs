use crate::pitch::Pitch;

pub type DurationIn32ndNotes = u8;

/// Creates a [Vec<DurationIn32ndNotes>] marking
/// which 32nd note "ticks" are metrically prominent, or made prominent by choice.
/// e.g. This would convert 6/8 time signature to [vec![0, 12]],
/// as the first and fourth 8th notes in that signature are the strong beats.
fn get_big_beats(
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
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl MeterDenominator {
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

/// A time signature, accompanied with "big beat"/"groove" information.
pub struct Meter {
    /// Numerator of a time signature, as is.
    num_beats: u8,
    /// Demoninator of a time signature,
    /// but represented as a duration of 32nd notes.
    beat_duration: DurationIn32ndNotes,
    /// Vec of durations between the "big beats" in a time signature or groove pattern.
    beat_pattern: Vec<DurationIn32ndNotes>,
}

/// Any duration denominated in quarter-note "beats". We arbitrarily use a quarter-note
/// grid, even though we still account for e.g. meters like 4/8.
type NumBeats = u8;

impl Meter {
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
            beat_duration,
            beat_pattern,
        }
    }
}

pub struct RhythmicNotatedEvent {
    duration: u8, // Maximum duration is a double-whole-note
    tied: bool,
    event: NotatedEvent,
}

pub enum NotatedEvent {
    SingleEvent(SingleEvent),
    Tuple(Vec<SingleEvent>),
}

pub enum SingleEvent {
    Pitch(Pitch),
    Voicing,
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
