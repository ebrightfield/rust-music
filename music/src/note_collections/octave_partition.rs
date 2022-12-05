use crate::note_collections::pc_set::PcSet;
use crate::note::pc::Pc;
use anyhow::anyhow;
use crate::note_collections::interval_class::IntervalClass;

/// An ordered, cyclic series of intervals that sum to an octave.
/// Represents a way to "slice" an octave into n intervals.
#[derive(Debug, PartialEq)]
pub struct OctavePartition(pub Vec<IntervalClass>);

impl OctavePartition {
    /// Sanitized to ensure that it's valid
    pub fn new(intervals: Vec<IntervalClass>) -> anyhow::Result<Self> {
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
            return Self(vec![IntervalClass::Ic0]);
        }
        let vals = pc_set.0.iter().map(|pc| i32::from(pc));
        let next_vals = pc_set.0.iter().skip(1).map(|pc| i32::from(pc));

        let mut diffs: Vec<i32> = vals.zip(next_vals).map(|(cur, next)| next - cur).collect();
        diffs.push(i32::from(pc_set.0.first().unwrap()) - i32::from(pc_set.0.last().unwrap()));
        let diffs = diffs
            .iter()
            .map(|i| IntervalClass::from(i))
            .collect();
        OctavePartition::new(diffs).unwrap()
    }
}

impl From<PcSet> for OctavePartition {
    fn from(pc_set: PcSet) -> Self {
        OctavePartition::from(&pc_set)
    }
}

impl From<&OctavePartition> for PcSet {
    fn from(value: &OctavePartition) -> Self {
        let mut i: u8 = 0;
        let mut pcs = vec![];
        for part in &value.0 {
            let distance: u8 = part.into();
            i += distance;
            pcs.push(i);
        }
        PcSet::new(pcs.iter().map(|pc| pc.into()).collect())
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
                IntervalClass::Ic4,
                IntervalClass::Ic3,
                IntervalClass::Ic5,
            ])
            .unwrap()
        );
    }

    #[test]
    fn test_invalid_octave_partition() {
        let intervals = vec![
            IntervalClass::Ic4,
            IntervalClass::Ic3,
            IntervalClass::Ic6,
        ];
        let result = OctavePartition::new(intervals.clone());
        match result {
            Ok(_) => panic!("octave partition should have failed"),
            Err(_) => {}
        }
    }
}
