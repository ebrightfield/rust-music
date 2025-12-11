//! Multi-level interval pattern system for melodic sequence generation.
//!
//! This module implements a hierarchical pattern system where multiple levels
//! of interval sequences can be combined. When a level is exhausted, it resets
//! and the next level advances.
//!
//! # Example Pattern
//!
//! ```ignore
//! // Pattern: [[1, 1], [2]] with master_step 1
//! // Yields: (1,0), (1,0), (2,1), (1,0), (1,0), (master=1,2), ...
//! let pattern = IntervalPattern::new(vec![vec![1, 1], vec![2]], 1);
//! ```

/// A single level of an interval pattern.
///
/// Each level contains a sequence of intervals that are yielded in order.
/// When exhausted, the level resets and signals the next level to advance.
#[derive(Debug, Clone)]
pub struct PatternLevel {
    intervals: Vec<i8>,
    position: usize,
}

impl PatternLevel {
    /// Create a new pattern level from a sequence of intervals.
    pub fn new(intervals: Vec<i8>) -> Self {
        Self {
            intervals,
            position: 0,
        }
    }

    /// Get the next interval from this level.
    /// Returns None if the level is exhausted (needs reset).
    pub fn next(&mut self) -> Option<i8> {
        if self.position >= self.intervals.len() {
            return None;
        }
        let val = self.intervals[self.position];
        self.position += 1;
        Some(val)
    }

    /// Reset this level to its starting position.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Check if this level has been exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.intervals.len()
    }

    /// Get the length of this pattern level.
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    /// Check if this pattern level is empty.
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }
}

/// Multi-level interval pattern for melodic sequence generation.
///
/// This implements the Python `interval_sequence` concept, where patterns
/// can have multiple nested levels. When all intervals at level 0 are consumed,
/// it resets and level 1 advances by one, and so on.
///
/// When all levels complete a full cycle, the master_step is yielded.
///
/// # Pattern Behavior
///
/// Given `[[9, 8], [7, 6, 5], [4]]` with master_step=1:
///
/// ```text
/// (9, 0) (8, 0) (7, 1)
/// (9, 0) (8, 0) (6, 1)
/// (9, 0) (8, 0) (5, 1)
/// (9, 0) (8, 0) (4, 2)
/// (9, 0) (8, 0) (7, 1)
/// (9, 0) (8, 0) (6, 1)
/// (9, 0) (8, 0) (5, 1)
/// (9, 0) (8, 0) (master=1, 3)
/// ...
/// ```
#[derive(Debug, Clone)]
pub struct IntervalPattern {
    levels: Vec<PatternLevel>,
    master_step: i8,
}

impl IntervalPattern {
    /// Create a new multi-level interval pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A vector of interval vectors, each representing a level
    /// * `master_step` - The interval to use when all levels complete a cycle
    pub fn new(pattern: Vec<Vec<i8>>, master_step: i8) -> Self {
        let levels = pattern
            .into_iter()
            .map(PatternLevel::new)
            .collect();
        Self { levels, master_step }
    }

    /// Create a simple single-level pattern.
    pub fn simple(intervals: Vec<i8>, master_step: i8) -> Self {
        Self::new(vec![intervals], master_step)
    }

    /// Get the next interval and the level it came from.
    ///
    /// Returns a tuple of (interval, level_index) where level_index indicates
    /// which pattern level produced the interval. When the master_step is
    /// returned, the level index equals the number of levels.
    pub fn next_interval(&mut self) -> (i8, usize) {
        if self.levels.is_empty() {
            return (self.master_step, 0);
        }

        for level in 0..self.levels.len() {
            if let Some(interval) = self.levels[level].next() {
                return (interval, level);
            }
            // This level exhausted - reset and try next level
            self.levels[level].reset();
        }

        // All levels completed one cycle - return master step
        (self.master_step, self.levels.len())
    }

    /// Reset all levels to their starting positions.
    pub fn reset(&mut self) {
        for level in &mut self.levels {
            level.reset();
        }
    }

    /// Get the number of levels in this pattern.
    pub fn num_levels(&self) -> usize {
        self.levels.len()
    }

    /// Get the master step value.
    pub fn master_step(&self) -> i8 {
        self.master_step
    }
}

impl Iterator for IntervalPattern {
    type Item = (i8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // This iterator is infinite - always returns Some
        Some(self.next_interval())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_level_basic() {
        let mut level = PatternLevel::new(vec![1, 2, 3]);
        assert_eq!(level.next(), Some(1));
        assert_eq!(level.next(), Some(2));
        assert_eq!(level.next(), Some(3));
        assert_eq!(level.next(), None);
        assert!(level.is_exhausted());

        level.reset();
        assert!(!level.is_exhausted());
        assert_eq!(level.next(), Some(1));
    }

    #[test]
    fn test_simple_pattern() {
        let mut pattern = IntervalPattern::simple(vec![1, 2], 3);

        // First cycle
        assert_eq!(pattern.next_interval(), (1, 0));
        assert_eq!(pattern.next_interval(), (2, 0));
        // Master step after completing level
        assert_eq!(pattern.next_interval(), (3, 1));
        // Repeats
        assert_eq!(pattern.next_interval(), (1, 0));
        assert_eq!(pattern.next_interval(), (2, 0));
        assert_eq!(pattern.next_interval(), (3, 1));
    }

    #[test]
    fn test_multi_level_pattern() {
        // Pattern: [[1, 1], [2]] - step step, skip, step step, skip, ...
        let mut pattern = IntervalPattern::new(vec![vec![1, 1], vec![2]], 1);

        // Level 0 twice, then level 1
        assert_eq!(pattern.next_interval(), (1, 0));
        assert_eq!(pattern.next_interval(), (1, 0));
        assert_eq!(pattern.next_interval(), (2, 1)); // Level 0 exhausted, level 1 advances

        // Level 0 resets, level 1 exhausted -> master step
        assert_eq!(pattern.next_interval(), (1, 0));
        assert_eq!(pattern.next_interval(), (1, 0));
        assert_eq!(pattern.next_interval(), (1, 2)); // Master step, both levels completed
    }

    #[test]
    fn test_three_level_pattern() {
        // Pattern from Python docs: [[9, 8], [7, 6, 5], [4]]
        let mut pattern = IntervalPattern::new(
            vec![vec![9, 8], vec![7, 6, 5], vec![4]],
            1,
        );

        // First full cycle
        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));
        assert_eq!(pattern.next_interval(), (7, 1)); // Level 0 exhausted

        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));
        assert_eq!(pattern.next_interval(), (6, 1));

        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));
        assert_eq!(pattern.next_interval(), (5, 1));

        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));
        assert_eq!(pattern.next_interval(), (4, 2)); // Level 0 and 1 exhausted, level 2

        // Next cycle of level 1 (level 2 is reset now)
        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));
        assert_eq!(pattern.next_interval(), (7, 1));

        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));
        assert_eq!(pattern.next_interval(), (6, 1));

        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));
        assert_eq!(pattern.next_interval(), (5, 1));

        // Level 1 exhausted, level 2 advances but is also exhausted
        // Still need to consume level 0 once more before master step
        assert_eq!(pattern.next_interval(), (9, 0));
        assert_eq!(pattern.next_interval(), (8, 0));

        // Now ALL levels are exhausted -> master step
        assert_eq!(pattern.next_interval(), (1, 3)); // Master step
    }

    #[test]
    fn test_empty_pattern() {
        let mut pattern = IntervalPattern::new(vec![], 5);
        // Empty pattern always returns master step
        assert_eq!(pattern.next_interval(), (5, 0));
        assert_eq!(pattern.next_interval(), (5, 0));
    }

    #[test]
    fn test_pattern_as_iterator() {
        let pattern = IntervalPattern::simple(vec![1, 2], 3);
        let intervals: Vec<_> = pattern.take(6).collect();

        assert_eq!(
            intervals,
            vec![(1, 0), (2, 0), (3, 1), (1, 0), (2, 0), (3, 1)]
        );
    }

    #[test]
    fn test_pattern_reset() {
        let mut pattern = IntervalPattern::simple(vec![1, 2, 3], 0);

        // Consume some
        pattern.next_interval();
        pattern.next_interval();

        // Reset
        pattern.reset();

        // Should start from beginning
        assert_eq!(pattern.next_interval(), (1, 0));
    }
}
