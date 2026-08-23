//! Beat grid for rhythmic analysis and metric structure representation.
//!
//! A beat grid provides a hierarchical view of beat positions within a meter,
//! with associated strength values for each position.

use crate::notation::rhythm::duration::DurationTicks;
use crate::notation::rhythm::meter::Meter;

/// Represents the metric strength of a beat position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BeatStrength {
    /// The strongest beat, typically the first beat of a measure.
    Downbeat = 4,
    /// Strong beats, typically the "big beats" in compound or simple meters.
    Strong = 3,
    /// Medium beats, e.g., beats 2 and 4 in 4/4 time.
    Medium = 2,
    /// Weak beats, typically off-beats or subdivisions.
    Weak = 1,
}

impl BeatStrength {
    /// Returns the numeric weight of this beat strength.
    pub fn weight(&self) -> u8 {
        match self {
            BeatStrength::Downbeat => 4,
            BeatStrength::Strong => 3,
            BeatStrength::Medium => 2,
            BeatStrength::Weak => 1,
        }
    }
}

/// A position in the beat grid with its associated strength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridPosition {
    /// Tick position from the start of the measure.
    pub position: DurationTicks,
    /// Metric strength at this position.
    pub strength: BeatStrength,
}

impl GridPosition {
    pub fn new(position: DurationTicks, strength: BeatStrength) -> Self {
        Self { position, strength }
    }
}

/// A hierarchical beat grid representing metric structure.
///
/// The beat grid provides a complete picture of all beat positions in a measure,
/// each with an associated strength value. This is useful for:
/// - Quantizing rhythms to metric positions
/// - Analyzing metric weight of events
/// - Generating rhythmic accompaniments
/// - Understanding compound vs. simple meter subdivisions
#[derive(Debug, Clone, PartialEq)]
pub struct BeatGrid {
    /// All grid positions, sorted by tick position.
    positions: Vec<GridPosition>,
    /// Total measure duration in ticks.
    measure_ticks: DurationTicks,
}

impl BeatGrid {
    /// Create a beat grid from a meter.
    ///
    /// The grid is populated with positions at the beat level (denominator unit)
    /// and assigns strengths based on metric hierarchy.
    pub fn from_meter(meter: &Meter) -> Self {
        let base_ticks = meter.base_unit_ticks();
        let measure_ticks = meter.measure_ticks();
        let big_beat_positions = meter.big_beat_positions();

        let mut positions = Vec::new();

        for beat_num in 0..meter.num_beats {
            let tick_pos = beat_num * base_ticks;
            let strength = Self::determine_strength(tick_pos, &big_beat_positions, measure_ticks);
            positions.push(GridPosition::new(tick_pos, strength));
        }

        Self {
            positions,
            measure_ticks,
        }
    }

    /// Create a beat grid with subdivision to a finer resolution.
    ///
    /// `subdivision` is how many parts to divide each beat into.
    /// For example, subdivision=2 creates eighth-note positions in 4/4.
    pub fn from_meter_subdivided(meter: &Meter, subdivision: usize) -> Self {
        if subdivision == 0 {
            return Self::from_meter(meter);
        }

        let base_ticks = meter.base_unit_ticks();
        let measure_ticks = meter.measure_ticks();
        let big_beat_positions = meter.big_beat_positions();
        let sub_ticks = base_ticks / subdivision;

        let mut positions = Vec::new();
        let num_positions = meter.num_beats * subdivision;

        for i in 0..num_positions {
            let tick_pos = i * sub_ticks;
            let is_on_beat = tick_pos.is_multiple_of(base_ticks);

            let strength = if is_on_beat {
                Self::determine_strength(tick_pos, &big_beat_positions, measure_ticks)
            } else {
                BeatStrength::Weak
            };

            positions.push(GridPosition::new(tick_pos, strength));
        }

        Self {
            positions,
            measure_ticks,
        }
    }

    /// Determine the strength of a beat at the given tick position.
    fn determine_strength(
        tick_pos: DurationTicks,
        big_beat_positions: &[DurationTicks],
        measure_ticks: DurationTicks,
    ) -> BeatStrength {
        // First beat of measure is always downbeat
        if tick_pos == 0 {
            return BeatStrength::Downbeat;
        }

        // Check if this is a big beat position
        if big_beat_positions.contains(&tick_pos) {
            return BeatStrength::Strong;
        }

        // Check if position is at the half-measure point (for simple meters)
        let half_measure = measure_ticks / 2;
        if tick_pos == half_measure && !big_beat_positions.contains(&half_measure) {
            return BeatStrength::Medium;
        }

        // All other beats
        BeatStrength::Medium
    }

    /// Get all positions in the grid.
    pub fn positions(&self) -> &[GridPosition] {
        &self.positions
    }

    /// Get the total measure duration in ticks.
    pub fn measure_ticks(&self) -> DurationTicks {
        self.measure_ticks
    }

    /// Get the number of positions in the grid.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if the grid is empty.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Find the strength at a given tick position.
    /// Returns None if the position is not on a grid point.
    pub fn strength_at(&self, tick_pos: DurationTicks) -> Option<BeatStrength> {
        self.positions
            .iter()
            .find(|p| p.position == tick_pos)
            .map(|p| p.strength)
    }

    /// Find the nearest grid position to a given tick value.
    /// Returns the grid position that is closest to the input.
    pub fn nearest_position(&self, tick_pos: DurationTicks) -> Option<&GridPosition> {
        self.positions.iter().min_by_key(|p| {
            
            tick_pos.abs_diff(p.position)
        })
    }

    /// Quantize a tick position to the nearest grid position.
    /// Returns the quantized tick position.
    pub fn quantize(&self, tick_pos: DurationTicks) -> DurationTicks {
        self.nearest_position(tick_pos)
            .map(|p| p.position)
            .unwrap_or(0)
    }

    /// Get all positions with a given minimum strength.
    pub fn positions_at_least(&self, min_strength: BeatStrength) -> Vec<&GridPosition> {
        self.positions
            .iter()
            .filter(|p| p.strength >= min_strength)
            .collect()
    }

    /// Get positions between two tick values (inclusive).
    pub fn positions_in_range(
        &self,
        start: DurationTicks,
        end: DurationTicks,
    ) -> Vec<&GridPosition> {
        self.positions
            .iter()
            .filter(|p| p.position >= start && p.position <= end)
            .collect()
    }

    /// Calculate the metric weight of an event at a given position.
    /// Returns a value from 1-4 based on beat strength.
    pub fn metric_weight_at(&self, tick_pos: DurationTicks) -> u8 {
        self.strength_at(tick_pos)
            .map(|s| s.weight())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::rhythm::meter::MeterDenominator;

    #[test]
    fn test_beat_grid_4_4() {
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let grid = BeatGrid::from_meter(&meter);

        assert_eq!(grid.len(), 4);
        assert_eq!(grid.measure_ticks(), 128);

        // Beat 1 (tick 0) is downbeat
        assert_eq!(grid.strength_at(0), Some(BeatStrength::Downbeat));
        // Beat 2 (tick 32) is medium
        assert_eq!(grid.strength_at(32), Some(BeatStrength::Medium));
        // Beat 3 (tick 64) is strong (big beat)
        assert_eq!(grid.strength_at(64), Some(BeatStrength::Strong));
        // Beat 4 (tick 96) is medium
        assert_eq!(grid.strength_at(96), Some(BeatStrength::Medium));
    }

    #[test]
    fn test_beat_grid_6_8() {
        let meter = Meter::new(6, MeterDenominator::Eight, None);
        let grid = BeatGrid::from_meter(&meter);

        assert_eq!(grid.len(), 6);
        assert_eq!(grid.measure_ticks(), 96);

        // Beat 1 is downbeat
        assert_eq!(grid.strength_at(0), Some(BeatStrength::Downbeat));
        // Beat 4 (tick 48) is strong (big beat in compound duple)
        assert_eq!(grid.strength_at(48), Some(BeatStrength::Strong));
    }

    #[test]
    fn test_beat_grid_3_4() {
        let meter = Meter::new(3, MeterDenominator::Four, None);
        let grid = BeatGrid::from_meter(&meter);

        assert_eq!(grid.len(), 3);
        // Beat 1 is downbeat
        assert_eq!(grid.strength_at(0), Some(BeatStrength::Downbeat));
        // Beats 2 and 3 should be strong (since 3/4 has big beats on each quarter)
        assert_eq!(grid.strength_at(32), Some(BeatStrength::Strong));
        assert_eq!(grid.strength_at(64), Some(BeatStrength::Strong));
    }

    #[test]
    fn test_subdivided_grid() {
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let grid = BeatGrid::from_meter_subdivided(&meter, 2); // eighth note subdivisions

        assert_eq!(grid.len(), 8);

        // On-beat positions retain their strength
        assert_eq!(grid.strength_at(0), Some(BeatStrength::Downbeat));
        assert_eq!(grid.strength_at(64), Some(BeatStrength::Strong));

        // Off-beat positions are weak
        assert_eq!(grid.strength_at(16), Some(BeatStrength::Weak)); // "and" of 1
        assert_eq!(grid.strength_at(48), Some(BeatStrength::Weak)); // "and" of 2
    }

    #[test]
    fn test_quantize() {
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let grid = BeatGrid::from_meter(&meter);

        // Position 10 is closer to 0 than to 32
        assert_eq!(grid.quantize(10), 0);
        // Position 20 is closer to 32 than to 0
        assert_eq!(grid.quantize(20), 32);
        // Position 50 is closer to 64 than to 32
        assert_eq!(grid.quantize(50), 64);
    }

    #[test]
    fn test_positions_at_least_strength() {
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let grid = BeatGrid::from_meter(&meter);

        let strong_or_better = grid.positions_at_least(BeatStrength::Strong);
        assert_eq!(strong_or_better.len(), 2); // downbeat and beat 3

        let downbeats = grid.positions_at_least(BeatStrength::Downbeat);
        assert_eq!(downbeats.len(), 1); // only beat 1
    }

    #[test]
    fn test_metric_weight() {
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let grid = BeatGrid::from_meter(&meter);

        assert_eq!(grid.metric_weight_at(0), 4);  // downbeat
        assert_eq!(grid.metric_weight_at(64), 3); // strong
        assert_eq!(grid.metric_weight_at(32), 2); // medium
        assert_eq!(grid.metric_weight_at(15), 0); // not on grid
    }

    #[test]
    fn test_positions_in_range() {
        let meter = Meter::new(4, MeterDenominator::Four, None);
        let grid = BeatGrid::from_meter(&meter);

        let range = grid.positions_in_range(30, 70);
        assert_eq!(range.len(), 2); // positions at 32 and 64
    }
}
