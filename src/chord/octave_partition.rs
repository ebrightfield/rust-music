use crate::chord::pc_set::PcSet;
use crate::note::pc::Pc;
use anyhow::anyhow;

/// The twelve possible interval distances inside of an octave.
#[derive(Debug, Clone, PartialEq)]
pub enum BaseChromaticInterval {
    Interval0,
    Interval1,
    Interval2,
    Interval3,
    Interval4,
    Interval5,
    Interval6,
    Interval7,
    Interval8,
    Interval9,
    Interval10,
    Interval11,
}

impl From<&BaseChromaticInterval> for i32 {
    fn from(interval: &BaseChromaticInterval) -> Self {
        match interval {
            BaseChromaticInterval::Interval0 => 0,
            BaseChromaticInterval::Interval1 => 1,
            BaseChromaticInterval::Interval2 => 2,
            BaseChromaticInterval::Interval3 => 3,
            BaseChromaticInterval::Interval4 => 4,
            BaseChromaticInterval::Interval5 => 5,
            BaseChromaticInterval::Interval6 => 6,
            BaseChromaticInterval::Interval7 => 7,
            BaseChromaticInterval::Interval8 => 8,
            BaseChromaticInterval::Interval9 => 9,
            BaseChromaticInterval::Interval10 => 10,
            BaseChromaticInterval::Interval11 => 11,
        }
    }
}

impl From<BaseChromaticInterval> for i32 {
    fn from(interval: BaseChromaticInterval) -> Self {
        i32::from(&interval)
    }
}

impl From<&i32> for BaseChromaticInterval {
    fn from(pc: &i32) -> Self {
        let pc = pc.rem_euclid(12);
        match pc {
            0 => BaseChromaticInterval::Interval0,
            1 => BaseChromaticInterval::Interval1,
            2 => BaseChromaticInterval::Interval2,
            3 => BaseChromaticInterval::Interval3,
            4 => BaseChromaticInterval::Interval4,
            5 => BaseChromaticInterval::Interval5,
            6 => BaseChromaticInterval::Interval6,
            7 => BaseChromaticInterval::Interval7,
            8 => BaseChromaticInterval::Interval8,
            9 => BaseChromaticInterval::Interval9,
            10 => BaseChromaticInterval::Interval10,
            11 => BaseChromaticInterval::Interval11,
            _ => unreachable!(),
        }
    }
}

impl From<i32> for BaseChromaticInterval {
    fn from(pc: i32) -> Self {
        BaseChromaticInterval::from(&pc)
    }
}

/// An ordered, cyclic series of intervals that sum to an octave.
/// Represents a way to "slice" an octave into n intervals.
#[derive(Debug, PartialEq)]
pub struct OctavePartition(pub Vec<BaseChromaticInterval>);

impl OctavePartition {
    /// Sanitized to ensure that it's valid
    pub fn new(intervals: Vec<BaseChromaticInterval>) -> anyhow::Result<Self> {
        let sum: i32 = intervals.iter().map(|interval| i32::from(interval)).sum();
        if sum != 12 {
            return Err(anyhow!("Invalid octave partition: {:?}", intervals));
        }
        Ok(Self(intervals))
    }
}

impl From<&[Pc]> for OctavePartition {
    fn from(pcs: &[Pc]) -> Self {
        OctavePartition::from(PcSet(pcs.to_vec()))
    }
}

impl From<&PcSet> for OctavePartition {
    fn from(pc_set: &PcSet) -> Self {
        if pc_set.is_empty() {
            return Self(vec![BaseChromaticInterval::Interval0]);
        }
        let vals = pc_set.0.iter().map(|pc| i32::from(pc));
        let next_vals = pc_set.0.iter().skip(1).map(|pc| i32::from(pc));

        let mut diffs: Vec<i32> = vals.zip(next_vals).map(|(cur, next)| next - cur).collect();
        diffs.push(i32::from(pc_set.0.first().unwrap()) - i32::from(pc_set.0.last().unwrap()));
        let diffs = diffs
            .iter()
            .map(|i| BaseChromaticInterval::from(i))
            .collect();
        OctavePartition::new(diffs).unwrap()
    }
}

impl From<PcSet> for OctavePartition {
    fn from(pc_set: PcSet) -> Self {
        OctavePartition::from(&pc_set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::pc::Pc;

    #[test]
    fn test_from_pc_set() {
        assert_eq!(
            {
                let pc_set = PcSet::new(vec![Pc::Pc0, Pc::Pc4, Pc::Pc7]);
                OctavePartition::from(pc_set)
            },
            OctavePartition::new(vec![
                BaseChromaticInterval::Interval4,
                BaseChromaticInterval::Interval3,
                BaseChromaticInterval::Interval5,
            ])
            .unwrap()
        );
    }

    #[test]
    fn test_invalid_octave_partition() {
        let intervals = vec![
            BaseChromaticInterval::Interval4,
            BaseChromaticInterval::Interval3,
            BaseChromaticInterval::Interval6,
        ];
        let result = OctavePartition::new(intervals.clone());
        match result {
            Ok(_) => panic!("octave partition should have failed"),
            Err(_) => {}
        }
    }
}
