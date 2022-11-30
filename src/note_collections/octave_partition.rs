use crate::note_collections::pc_set::PcSet;
use crate::note::pc::Pc;
use anyhow::anyhow;

/// The twelve possible interval distances inside of an octave.
/// We keep this as a distinct class from [Pc] because while they're
/// both mod-12 values, they represent different things musically,
/// and they have certain relational properties that we want to be able to express
/// in code.
///
/// Technically, this is called an unordered Pitch Interval Class.
#[derive(Debug, Clone, PartialEq)]
pub enum IntervalClass {
    Ic0,
    Ic1,
    Ic2,
    Ic3,
    Ic4,
    Ic5,
    Ic6,
    Ic7,
    Ic8,
    Ic9,
    Ic10,
    Ic11,
}

impl From<&IntervalClass> for Pc {
    fn from(interval: &IntervalClass) -> Pc {
        match interval {
            IntervalClass::Ic0 => Pc::Pc0,
            IntervalClass::Ic1 => Pc::Pc1,
            IntervalClass::Ic2 => Pc::Pc2,
            IntervalClass::Ic3 => Pc::Pc3,
            IntervalClass::Ic4 => Pc::Pc4,
            IntervalClass::Ic5 => Pc::Pc5,
            IntervalClass::Ic6 => Pc::Pc6,
            IntervalClass::Ic7 => Pc::Pc7,
            IntervalClass::Ic8 => Pc::Pc8,
            IntervalClass::Ic9 => Pc::Pc9,
            IntervalClass::Ic10 => Pc::Pc10,
            IntervalClass::Ic11 => Pc::Pc11,
        }
    }
}


impl From<IntervalClass> for Pc {
    fn from(interval: IntervalClass) -> Pc {
        Pc::from(&interval)
    }
}

impl From<&IntervalClass> for i32 {
    fn from(interval: &IntervalClass) -> Self {
        match interval {
            IntervalClass::Ic0 => 0,
            IntervalClass::Ic1 => 1,
            IntervalClass::Ic2 => 2,
            IntervalClass::Ic3 => 3,
            IntervalClass::Ic4 => 4,
            IntervalClass::Ic5 => 5,
            IntervalClass::Ic6 => 6,
            IntervalClass::Ic7 => 7,
            IntervalClass::Ic8 => 8,
            IntervalClass::Ic9 => 9,
            IntervalClass::Ic10 => 10,
            IntervalClass::Ic11 => 11,
        }
    }
}

impl From<IntervalClass> for i32 {
    fn from(interval: IntervalClass) -> Self {
        i32::from(&interval)
    }
}

impl From<&i32> for IntervalClass {
    fn from(pc: &i32) -> Self {
        let pc = pc.rem_euclid(12);
        match pc {
            0 => IntervalClass::Ic0,
            1 => IntervalClass::Ic1,
            2 => IntervalClass::Ic2,
            3 => IntervalClass::Ic3,
            4 => IntervalClass::Ic4,
            5 => IntervalClass::Ic5,
            6 => IntervalClass::Ic6,
            7 => IntervalClass::Ic7,
            8 => IntervalClass::Ic8,
            9 => IntervalClass::Ic9,
            10 => IntervalClass::Ic10,
            11 => IntervalClass::Ic11,
            _ => unreachable!(),
        }
    }
}

impl From<i32> for IntervalClass {
    fn from(pc: i32) -> Self {
        IntervalClass::from(&pc)
    }
}

impl From<&u8> for IntervalClass {
    fn from(pc: &u8) -> Self {
        let pc = pc.rem_euclid(12);
        match pc {
            0 => IntervalClass::Ic0,
            1 => IntervalClass::Ic1,
            2 => IntervalClass::Ic2,
            3 => IntervalClass::Ic3,
            4 => IntervalClass::Ic4,
            5 => IntervalClass::Ic5,
            6 => IntervalClass::Ic6,
            7 => IntervalClass::Ic7,
            8 => IntervalClass::Ic8,
            9 => IntervalClass::Ic9,
            10 => IntervalClass::Ic10,
            11 => IntervalClass::Ic11,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for IntervalClass {
    fn from(pc: u8) -> Self {
        IntervalClass::from(&pc)
    }
}


impl Into<u8> for IntervalClass {
    fn into(self) -> u8 {
        (&self).into()
    }
}

impl Into<u8> for &IntervalClass {
    fn into(self) -> u8 {
        match self {
            IntervalClass::Ic0 => 0,
            IntervalClass::Ic1 => 1,
            IntervalClass::Ic2 => 2,
            IntervalClass::Ic3 => 3,
            IntervalClass::Ic4 => 4,
            IntervalClass::Ic5 => 5,
            IntervalClass::Ic6 => 6,
            IntervalClass::Ic7 => 7,
            IntervalClass::Ic8 => 8,
            IntervalClass::Ic9 => 9,
            IntervalClass::Ic10 => 10,
            IntervalClass::Ic11 => 11,
        }
    }
}

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
