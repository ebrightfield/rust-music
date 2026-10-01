use crate::note::pitch_class::Pc;
use std::fmt::{Display, Formatter};

// TODO Need to be able to derive these from two Pc instances

/// The twelve possible interval distances inside of an octave.
/// We keep this as a distinct class from [Pc] because while they're
/// both mod-12 values, they represent different things musically,
/// and they have certain relational properties that we want to be able to express
/// in code.
///
/// Technically, in musical set theory, this is called an unordered Pitch Interval Class.
/// We're being a bit loose with names here for brevity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntervalClass {
    /// Unison, Octave
    Ic0,
    /// Halfstep
    Ic1,
    /// Wholestep, diminished third
    Ic2,
    /// Minor third, Augmented second
    Ic3,
    /// Major third, diminished fourth
    Ic4,
    /// Perfect fourth
    Ic5,
    /// Augmented fourth, diminished fifth
    Ic6,
    /// Perfect fifth
    Ic7,
    /// Minor sixth, Augmented fifth
    Ic8,
    /// Major sixth, diminished seventh
    Ic9,
    /// Minor seventh, Augmented sixth
    Ic10,
    /// Major seventh
    Ic11,
}

impl Display for IntervalClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            IntervalClass::Ic0 => "unison",
            IntervalClass::Ic1 => "m2",
            IntervalClass::Ic2 => "M2",
            IntervalClass::Ic3 => "m3",
            IntervalClass::Ic4 => "M3",
            IntervalClass::Ic5 => "P4",
            IntervalClass::Ic6 => "TT",
            IntervalClass::Ic7 => "P5",
            IntervalClass::Ic8 => "m6",
            IntervalClass::Ic9 => "M6",
            IntervalClass::Ic10 => "m7",
            IntervalClass::Ic11 => "M7",
        };
        f.write_str(label)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_produces_proper_interval_labels() {
        assert_eq!(IntervalClass::Ic0.to_string(), "unison");
        assert_eq!(IntervalClass::Ic1.to_string(), "m2");
        assert_eq!(IntervalClass::Ic2.to_string(), "M2");
        assert_eq!(IntervalClass::Ic3.to_string(), "m3");
        assert_eq!(IntervalClass::Ic4.to_string(), "M3");
        assert_eq!(IntervalClass::Ic5.to_string(), "P4");
        assert_eq!(IntervalClass::Ic6.to_string(), "TT");
        assert_eq!(IntervalClass::Ic7.to_string(), "P5");
        assert_eq!(IntervalClass::Ic8.to_string(), "m6");
        assert_eq!(IntervalClass::Ic9.to_string(), "M6");
        assert_eq!(IntervalClass::Ic10.to_string(), "m7");
        assert_eq!(IntervalClass::Ic11.to_string(), "M7");
    }
}
