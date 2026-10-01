pub type DurationTicks = usize;

/// Not every tick value can be represented as a singly notated durational symbol.
/// For example, a five eighth-note duration (i.e. two-and-a-half beats) can't be represented
/// except for two or more tied rhythmic values. In contrast, durations of four, six,
/// or seven eighth notes can all be represented using dotted quarter,
/// and double-dotted quarter.
///
/// For that reason, it is sensible to have a means of working with only those values
/// which can be represented "atomically" in notation.
/// These are the "durational building blocks" from which we can construct any other value,
/// such as the five eighth note duration previously mentioned.
///
/// The longest duration here is a double-whole-note, and the shortest duration
/// is a 128th note. This range of values was chosen because it covers what Lilypond
/// is capable of.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DurationKind {
    Breve,
    Whole,
    Half,
    Qtr, // 32 "ticks"
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    OneTwentyEighth, // one "tick"
}

impl From<DurationKind> for DurationTicks {
    fn from(kind: DurationKind) -> Self {
        match kind {
            DurationKind::Breve => 256,
            DurationKind::Whole => 128,
            DurationKind::Half => 64,
            DurationKind::Qtr => 32,
            DurationKind::Eighth => 16,
            DurationKind::Sixteenth => 8,
            DurationKind::ThirtySecond => 4,
            DurationKind::SixtyFourth => 2,
            DurationKind::OneTwentyEighth => 1,
        }
    }
}

impl From<DurationKind> for u32 {
    fn from(kind: DurationKind) -> Self {
        match kind {
            DurationKind::Breve => 256,
            DurationKind::Whole => 128,
            DurationKind::Half => 64,
            DurationKind::Qtr => 32,
            DurationKind::Eighth => 16,
            DurationKind::Sixteenth => 8,
            DurationKind::ThirtySecond => 4,
            DurationKind::SixtyFourth => 2,
            DurationKind::OneTwentyEighth => 1,
        }
    }
}

impl DurationKind {
    /// Returns all duration kinds from longest to shortest.
    pub const fn all() -> [DurationKind; 9] {
        [
            DurationKind::Breve,
            DurationKind::Whole,
            DurationKind::Half,
            DurationKind::Qtr,
            DurationKind::Eighth,
            DurationKind::Sixteenth,
            DurationKind::ThirtySecond,
            DurationKind::SixtyFourth,
            DurationKind::OneTwentyEighth,
        ]
    }

    /// Returns the tick count for this duration kind.
    pub fn ticks(self) -> DurationTicks {
        self.into()
    }
}

/// A [DurationKind] potentially lengthened with zero to five dots.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Duration {
    /// Zero to five dots, each of which augment the base value `d`
    /// according to the normal "geometric series" of `d/2 + d/4 + d/8 + ...`
    ///
    /// Five is chosen as a maximum on the grounds
    /// that asking the reader to read more than that is unreasonable, and
    /// other durations can easily be represented with ties.
    /// On the very shortest [DurationKind] variants, the practical bound is lower,
    /// since taking 1/16 of 2 ticks floors to zero.
    dot: u8,
    /// The basic category of duration.
    dur: DurationKind,
}

impl Duration {
    pub const WHOLE: Self = Self {
        dot: 0,
        dur: DurationKind::Whole,
    };
    pub const HALF: Self = Self {
        dot: 0,
        dur: DurationKind::Half,
    };
    pub const QTR: Self = Self {
        dot: 0,
        dur: DurationKind::Qtr,
    };
    pub const EIGHTH: Self = Self {
        dot: 0,
        dur: DurationKind::Eighth,
    };
    pub const SIXTEENTH: Self = Self {
        dot: 0,
        dur: DurationKind::Sixteenth,
    };

    /// A "maybe" constructor.
    /// If the passed number of ticks can be represented as an "atomic" musical duration,
    /// then this constructor returns such an instance.
    pub fn try_from_ticks(ticks: DurationTicks) -> Option<Self> {
        for dur in [
            DurationKind::Breve,
            DurationKind::Whole,
            DurationKind::Half,
            DurationKind::Qtr,
            DurationKind::Eighth,
            DurationKind::Sixteenth,
            DurationKind::ThirtySecond,
            DurationKind::SixtyFourth,
            DurationKind::OneTwentyEighth,
        ] {
            let dur_ticks: usize = dur.into();
            if dur_ticks <= ticks {
                for dot in 0..6 {
                    let instance = Self { dot, dur };
                    if instance.ticks() == ticks {
                        return Some(instance);
                    }
                }
                // We only need to iterate dots on
                // the first denomination < the ticks
                // You can't e.g. dot an eighth to get n beats where 1 < n < 2.
                return None;
            }
        }
        None
    }

    pub fn new(dur: DurationKind, dot: u8) -> Self {
        Self { dot, dur }
    }

    pub fn kind(&self) -> DurationKind {
        self.dur
    }

    pub fn num_dots(&self) -> u8 {
        self.dot
    }

    /// Returns the duration in beat "ticks", where one tick = a 128th note.
    /// This entails that 1 beat = 32 ticks.
    pub fn ticks(&self) -> usize {
        if self.dot == 0 {
            return self.dur.into();
        }
        let base_dur: u32 = self.dur.into();
        (0u32..self.dot as u32).fold(base_dur, |acc, n| acc + base_dur / 2u32.pow(n + 1)) as usize
    }

    /// Try to add two durations, returning None if result isn't representable
    /// as a single (possibly dotted) duration.
    pub fn try_add(&self, other: &Duration) -> Option<Duration> {
        let total_ticks = self.ticks() + other.ticks();
        Duration::try_from_ticks(total_ticks)
    }

    /// Split a tick count into tied notes that sum to that duration.
    /// Uses the largest possible durations first (greedy algorithm).
    /// Returns durations from largest to smallest.
    pub fn split_ticks_for_ties(mut ticks: DurationTicks) -> Vec<Duration> {
        let mut result = Vec::new();

        for kind in DurationKind::all() {
            let kind_ticks = kind.ticks();
            while kind_ticks <= ticks {
                result.push(Duration::new(kind, 0));
                ticks -= kind_ticks;
            }
        }

        result
    }

    /// Split this duration into tied notes that sum to the same total.
    /// For durations that can be represented atomically (e.g., dotted quarter),
    /// this returns a single-element vector.
    /// For durations requiring ties, returns multiple durations.
    pub fn split_for_ties(&self) -> Vec<Duration> {
        Duration::split_ticks_for_ties(self.ticks())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotted_durations() {
        // Dots accumulate duration appropriately
        assert_eq!(32, Duration::new(DurationKind::Qtr, 0).ticks());
        assert_eq!(32 + 16, Duration::new(DurationKind::Qtr, 1).ticks());
        assert_eq!(32 + 16 + 8, Duration::new(DurationKind::Qtr, 2).ticks());
        assert_eq!(32 + 16 + 8 + 4, Duration::new(DurationKind::Qtr, 3).ticks());
        // At the extremes, dots don't add anything
        assert_eq!(3, Duration::new(DurationKind::SixtyFourth, 1).ticks());
        assert_eq!(3, Duration::new(DurationKind::SixtyFourth, 2).ticks());
    }

    #[test]
    fn durations_from_ticks() {
        let d = Duration::try_from_ticks(32);
        assert_eq!(d, Some(Duration::new(DurationKind::Qtr, 0)));
        let d = Duration::try_from_ticks(32 + 16);
        assert_eq!(d, Some(Duration::new(DurationKind::Qtr, 1)));
        let d = Duration::try_from_ticks(32 + 16 + 8);
        assert_eq!(d, Some(Duration::new(DurationKind::Qtr, 2)));
        let d = Duration::try_from_ticks(64 + 32 + 16 + 8);
        assert_eq!(d, Some(Duration::new(DurationKind::Half, 3)));
    }

    #[test]
    fn test_try_add() {
        // Quarter + quarter = half
        let qtr = Duration::QTR;
        let half = qtr.try_add(&qtr);
        assert_eq!(half, Some(Duration::HALF));

        // Quarter + eighth = dotted quarter
        let eighth = Duration::EIGHTH;
        let dotted_qtr = qtr.try_add(&eighth);
        assert_eq!(dotted_qtr, Some(Duration::new(DurationKind::Qtr, 1)));

        // Two eighths = quarter
        let result = eighth.try_add(&eighth);
        assert_eq!(result, Some(Duration::QTR));

        // 5 eighths (80 ticks) can't be represented as single duration
        // 80 = 64 + 16 (half + eighth) - not a standard notation
        let five_eighths = Duration::try_from_ticks(80);
        assert_eq!(five_eighths, None);
    }

    #[test]
    fn test_split_for_ties() {
        // A quarter note splits to just itself
        let qtr = Duration::QTR;
        let split = qtr.split_for_ties();
        assert_eq!(split.len(), 1);
        assert_eq!(split[0], Duration::QTR);

        // 5 eighth notes (80 ticks) = half + eighth
        let split = Duration::split_ticks_for_ties(80);
        assert_eq!(split.len(), 2);
        assert_eq!(split[0], Duration::HALF);
        assert_eq!(split[1], Duration::EIGHTH);

        // 3 quarters (96 ticks) = half + quarter
        let split = Duration::split_ticks_for_ties(96);
        assert_eq!(split.len(), 2);
        assert_eq!(split[0], Duration::HALF);
        assert_eq!(split[1], Duration::QTR);

        // Whole + quarter (160 ticks) = whole + quarter
        let split = Duration::split_ticks_for_ties(160);
        assert_eq!(split.len(), 2);
        assert_eq!(split[0], Duration::WHOLE);
        assert_eq!(split[1], Duration::QTR);
    }
}
