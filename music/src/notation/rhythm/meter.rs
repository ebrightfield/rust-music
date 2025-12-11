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
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Converts the associated rhythmic value into ticks.
    /// Note: Uses the same tick scale as Duration (128 ticks = whole note).
    pub fn ticks(&self) -> DurationTicks {
        match &self {
            MeterDenominator::One => 128,
            MeterDenominator::Two => 64,
            MeterDenominator::Four => 32,
            MeterDenominator::Eight => 16,
            MeterDenominator::Sixteen => 8,
        }
    }
}

impl From<&MeterDenominator> for Duration {
    fn from(denom: &MeterDenominator) -> Self {
        match denom {
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
#[derive(Debug, Clone, PartialEq)]
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
        let total_duration = beat_duration * numerator;
        let beat_pattern = big_beats_to_durations(big_beats, total_duration);
        Self {
            num_beats: numerator,
            denominator,
            beat_pattern,
        }
    }

    /// Get the duration of one base unit (denominator) in ticks.
    pub fn base_unit_ticks(&self) -> DurationTicks {
        self.denominator.ticks()
    }

    /// Total measure duration in ticks.
    pub fn measure_ticks(&self) -> DurationTicks {
        self.num_beats * self.base_unit_ticks()
    }

    /// Get the tick positions of the "big beats" (primary rhythmic groupings).
    /// Returns cumulative positions starting from 0.
    pub fn big_beat_positions(&self) -> Vec<DurationTicks> {
        let mut positions = vec![0];
        let mut cumulative = 0;
        for &duration in &self.beat_pattern {
            cumulative += duration;
            if cumulative < self.measure_ticks() {
                positions.push(cumulative);
            }
        }
        positions
    }
}

/// Tracks position within a meter for meter-aware duration splitting.
///
/// This is used to determine when durations need to be split (tied)
/// to respect barlines and big beat boundaries.
#[derive(Debug, Clone)]
pub struct MeterContext {
    /// The meter being used.
    pub meter: Meter,
    /// Current position in ticks within the measure (0 = start of measure).
    pub position: DurationTicks,
}

impl MeterContext {
    /// Create a new meter context at the start of a measure.
    pub fn new(meter: Meter) -> Self {
        Self { meter, position: 0 }
    }

    /// Create a new meter context at a specific position.
    pub fn at_position(meter: Meter, position: DurationTicks) -> Self {
        let measure_len = meter.measure_ticks();
        Self {
            meter,
            position: position % measure_len,
        }
    }

    /// Advance position by the given number of ticks, wrapping at measure boundary.
    pub fn advance(&mut self, ticks: DurationTicks) {
        self.position = (self.position + ticks) % self.meter.measure_ticks();
    }

    /// Check if a duration at the current position would cross a barline.
    /// Returns the number of ticks until the barline if it would cross.
    fn barline_crossing(&self, duration_ticks: DurationTicks) -> Option<DurationTicks> {
        let end_pos = self.position + duration_ticks;
        let measure_end = self.meter.measure_ticks();

        if end_pos > measure_end {
            Some(measure_end - self.position)
        } else {
            None
        }
    }

    /// Check if a duration at current position would cross a big beat boundary.
    /// Returns the number of ticks until the first big beat if it would cross.
    fn big_beat_crossing(&self, duration_ticks: DurationTicks) -> Option<DurationTicks> {
        let end_pos = self.position + duration_ticks;
        let big_beats = self.meter.big_beat_positions();

        for beat_pos in big_beats {
            if self.position < beat_pos && beat_pos < end_pos {
                return Some(beat_pos - self.position);
            }
        }

        None
    }

    /// Check if a duration at current position needs splitting.
    /// Returns the number of ticks at which to split, if splitting is needed.
    /// Barline crossings take priority over big beat crossings.
    pub fn needs_split(&self, duration_ticks: DurationTicks) -> Option<DurationTicks> {
        // Check barline crossing first (takes priority)
        if let Some(split_at) = self.barline_crossing(duration_ticks) {
            return Some(split_at);
        }

        // Check big beat crossing
        self.big_beat_crossing(duration_ticks)
    }

    /// Split a duration respecting meter, returning tied durations.
    ///
    /// Each element in the result is a tuple of (Duration, is_tied).
    /// The `is_tied` flag indicates whether the note should be tied to the next.
    /// The last note in the sequence will have `is_tied = false`.
    pub fn split_for_meter(&self, duration: &Duration) -> Vec<(Duration, bool)> {
        let mut result = Vec::new();
        let mut remaining = duration.ticks();
        let mut pos = self.position;

        while remaining > 0 {
            // Create temporary context at current position
            let temp_ctx = MeterContext::at_position(self.meter.clone(), pos);

            // Check if we need to split
            if let Some(split_at) = temp_ctx.needs_split(remaining) {
                // Split here - add duration(s) for the first part
                let first_durations = Duration::split_ticks_for_ties(split_at);
                let num_first = first_durations.len();
                for (i, d) in first_durations.into_iter().enumerate() {
                    // All but the last of the first part are tied to each other
                    // The last one is tied to the next segment
                    let is_last_of_first = i == num_first - 1;
                    result.push((d, !is_last_of_first || remaining > split_at));
                }

                remaining -= split_at;
                pos = (pos + split_at) % self.meter.measure_ticks();
            } else {
                // No split needed - add final duration(s)
                let final_durations = Duration::split_ticks_for_ties(remaining);
                let num_final = final_durations.len();
                for (i, d) in final_durations.into_iter().enumerate() {
                    // All but the last are tied
                    result.push((d, i < num_final - 1));
                }
                remaining = 0;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::rhythm::duration::DurationKind;

    #[test]
    fn test_get_big_beats() {
        // 4/4: quarter note = 32 ticks, total = 128 ticks, big beats at 0 and 64
        let result = get_big_beats(4, 32);
        assert_eq!(result, vec![0, 64]);
        // 3/4: quarter note = 32 ticks, total = 96 ticks, big beats at every quarter
        let result = get_big_beats(3, 32);
        assert_eq!(result, vec![0, 32, 64]);
        // 5/4: quarter note = 32 ticks, big beats follow 3+2 pattern (0 and 96)
        let result = get_big_beats(5, 32);
        assert_eq!(result, vec![0, 96]);
    }

    #[test]
    fn test_meter_helper_methods() {
        let meter_4_4 = Meter::new(4, MeterDenominator::Four, None);
        assert_eq!(meter_4_4.base_unit_ticks(), 32);
        assert_eq!(meter_4_4.measure_ticks(), 128);
        // 4/4 big beats at 0 and 64
        assert_eq!(meter_4_4.big_beat_positions(), vec![0, 64]);

        let meter_6_8 = Meter::new(6, MeterDenominator::Eight, None);
        assert_eq!(meter_6_8.base_unit_ticks(), 16);
        assert_eq!(meter_6_8.measure_ticks(), 96);
        // 6/8 big beats at 0 and 48 (compound duple)
        assert_eq!(meter_6_8.big_beat_positions(), vec![0, 48]);
    }

    #[test]
    fn test_meter_context_no_split_needed() {
        // A quarter note at the start of 4/4 doesn't need splitting
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let ctx = MeterContext::new(meter);
        let qtr = Duration::QTR;

        let result = ctx.split_for_meter(&qtr);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], (Duration::QTR, false));
    }

    #[test]
    fn test_meter_context_barline_split() {
        // A half note starting at beat 3 of 4/4 should split at barline
        // Position = beat 3 = 2 quarters = 64 ticks
        // Half note = 64 ticks, would end at 128, which is the barline
        // Actually this doesn't cross, let's use a bigger duration

        let meter = Meter::new(4, MeterDenominator::Four, None);
        let ctx = MeterContext::at_position(meter, 96); // beat 4 (3 quarters in)

        // A half note (64 ticks) starting at tick 96 would end at 160
        // That crosses the barline at 128
        let half = Duration::HALF;
        let result = ctx.split_for_meter(&half);

        // Should split: 32 ticks before barline, 32 ticks after
        // Both parts are quarters
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, Duration::QTR);
        assert!(result[0].1); // tied
        assert_eq!(result[1].0, Duration::QTR);
        assert!(!result[1].1); // not tied (last note)
    }

    #[test]
    fn test_meter_context_big_beat_split() {
        // In 4/4, a half note starting on beat 2 should split at the half-bar
        // Position = beat 2 = 32 ticks
        // Big beats are at 0 and 64
        // Half note = 64 ticks, would go from 32 to 96, crossing the big beat at 64

        let meter = Meter::new(4, MeterDenominator::Four, None);
        let ctx = MeterContext::at_position(meter, 32); // beat 2

        let half = Duration::HALF;
        let result = ctx.split_for_meter(&half);

        // Should split at tick 64: first quarter (32 ticks) + second quarter (32 ticks)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, Duration::QTR);
        assert!(result[0].1); // tied
        assert_eq!(result[1].0, Duration::QTR);
        assert!(!result[1].1); // not tied (last note)
    }

    #[test]
    fn test_meter_context_whole_note_in_4_4() {
        // A whole note at the start of 4/4 should split at the half-bar
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let ctx = MeterContext::new(meter);

        let whole = Duration::WHOLE;
        let result = ctx.split_for_meter(&whole);

        // Should split at tick 64: two half notes
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, Duration::HALF);
        assert!(result[0].1); // tied
        assert_eq!(result[1].0, Duration::HALF);
        assert!(!result[1].1); // not tied (last note)
    }

    #[test]
    fn test_meter_context_dotted_half_from_beat_2() {
        // Dotted half (96 ticks) starting at beat 2 (32 ticks) in 4/4
        // End position would be 128, at barline
        // But it also crosses big beat at 64
        // Should split at 64: quarter + half

        let meter = Meter::new(4, MeterDenominator::Four, None);
        let ctx = MeterContext::at_position(meter, 32);

        let dotted_half = Duration::new(DurationKind::Half, 1); // 96 ticks
        let result = ctx.split_for_meter(&dotted_half);

        // Split at big beat (64): first part = 32 ticks (quarter)
        // Remaining = 64 ticks (half), no more splits needed
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, Duration::QTR);
        assert!(result[0].1);
        assert_eq!(result[1].0, Duration::HALF);
        assert!(!result[1].1);
    }

    #[test]
    fn test_meter_context_advance() {
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let mut ctx = MeterContext::new(meter);

        assert_eq!(ctx.position, 0);

        ctx.advance(32); // advance one quarter
        assert_eq!(ctx.position, 32);

        ctx.advance(128); // advance a full measure
        assert_eq!(ctx.position, 32); // should wrap

        ctx.advance(96); // advance to end of measure
        assert_eq!(ctx.position, 0); // should wrap to start
    }
}
