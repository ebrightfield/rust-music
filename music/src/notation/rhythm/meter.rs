use crate::notation::rhythm::duration::{Duration, DurationTicks};

/// Creates a Vector of [DurationTicks] marking
/// which ticks are metrically prominent, or made prominent by choice.
/// e.g. This would convert 6/8 time signature to [vec![0, 12]],
/// as the first and fourth 8th notes in that signature are the strong beats.
pub fn get_big_beats(
    num_beats: usize,
    base_unit_duration: DurationTicks,
) -> Vec<DurationTicks> {
    // Compound meters and 4/4
    for divisor in [7, 5, 3, 2] {
        if num_beats.rem_euclid(divisor) == 0 && num_beats != divisor {
            let divided = num_beats / divisor;
            return (0..divided)
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

/// Returns a Vec of [DurationTicks] representing the
/// amount of time between temporally adjacent elements.
pub fn big_beats_to_durations(
    big_beats: Vec<DurationTicks>,
    total_duration: DurationTicks,
) -> Vec<DurationTicks> {
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

impl MeterDenominator {
    /// Converts the associated rhythmic value into a [Duration
    pub fn ticks(&self) -> DurationTicks {
        match &self {
            MeterDenominator::One => 32,
            MeterDenominator::Two => 16,
            MeterDenominator::Four => 8,
            MeterDenominator::Eight => 4,
            MeterDenominator::Sixteen => 2,
        }
    }
}

impl Into<Duration> for &MeterDenominator {
    fn into(self) -> Duration {
        match self {
            MeterDenominator::One => Duration::WHOLE,
            MeterDenominator::Two => Duration::HALF,
            MeterDenominator::Four => Duration::QTR,
            MeterDenominator::Eight => Duration::EIGHTH,
            MeterDenominator::Sixteen => Duration::SIXTEENTH,
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
    pub num_beats: usize,
    /// Denominator of a time signature.
    pub denominator: MeterDenominator,
    /// Vec of durations between the "big beats" in a time signature or groove pattern.
    pub beat_pattern: Vec<DurationTicks>,
}

impl Meter {
    /// Takes the numerator and demoninator of a typical non-additive meter,
    /// and optionally, an accent pattern. A default accent pattern is inferred
    /// for various meters.
    pub fn new(
        numerator: usize,
        denominator: MeterDenominator,
        beat_pattern: Option<Vec<DurationTicks>>,
    ) -> Self {
        let beat_duration: DurationTicks = denominator.ticks();
        let big_beats = if let Some(pattern) = beat_pattern {
            pattern
        } else {
            get_big_beats(numerator, beat_duration)
        };
        let total_duration = beat_duration * (numerator as usize);
        let beat_pattern = big_beats_to_durations(big_beats, total_duration);
        Self {
            num_beats: numerator,
            denominator,
            beat_pattern,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_big_beats() {
        // 4/4
        let result = get_big_beats(4, 8);
        assert_eq!(result, vec![0, 16]);
        // 3/4
        let result = get_big_beats(3, 8);
        assert_eq!(result, vec![0, 8, 16]);
        // 5/4
        let result = get_big_beats(5, 8);
        assert_eq!(result, vec![0, 24]);
    }
}
