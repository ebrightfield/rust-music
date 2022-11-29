use crate::note::note::Note;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

struct PcIter {
    curr: Pc,
    until: Pc,
    is_first: bool,
}
impl PcIter {
    pub fn starting_on(start: &Pc) -> Self {
        Self {
            curr: start.clone(),
            until: start.clone(),
            is_first: true,
        }
    }
    pub fn section(start: &Pc, until: &Pc) -> Self {
        Self {
            curr: start.clone(),
            until: until.clone(),
            is_first: true,
        }
    }
}

impl Default for PcIter {
    fn default() -> Self {
        Self {
            curr: Pc::Pc0,
            until: Pc::Pc0,
            is_first: true,
        }
    }
}

impl Iterator for PcIter {
    type Item = Pc;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.until && !self.is_first {
            return None;
        }
        self.is_first = false;
        let next = self.curr.next();
        let curr = self.curr.clone();
        self.curr = next;
        Some(curr)
    }
}

/// Pitch-class -- a representation of musical notes that is
/// agnostic to both letter and octave information.
/// Every note of the chromatic scale is mapped to mod-12
/// space of integers.
/// Which note is chosen to be zero is arbitrary and situational,
/// although by far the most common choice is usually the note "C".
/// This convention is built into the [impl From<Note> for Pc].
/// [Pc] converts to [i32] for arithmetic operations, and to [u8]
/// for MIDI note representation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Pc {
    Pc0,
    Pc1,
    Pc2,
    Pc3,
    Pc4,
    Pc5,
    Pc6,
    Pc7,
    Pc8,
    Pc9,
    Pc10,
    Pc11,
}

impl Pc {
    pub fn next(&self) -> Self {
        Self::from(u8::from(self) + 1)
    }

    pub fn previous(&self) -> Self {
        Self::from(u8::from(self) - 1)
    }
}

impl PartialOrd<Self> for Pc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let other: i32 = other.into();
        let this: i32 = self.into();
        Some(this.cmp(&other))
    }
}

impl Ord for Pc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Hash for Pc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        i32::from(self).hash(state)
    }
}

impl Pc {
    pub fn notes(&self) -> Vec<Note> {
        match self {
            Pc::Pc0 => vec![Note::C, Note::Bis, Note::Deses],
            Pc::Pc1 => vec![Note::Cis, Note::Des],
            Pc::Pc2 => vec![Note::D, Note::Cisis, Note::Eeses],
            Pc::Pc3 => vec![Note::Dis, Note::Ees],
            Pc::Pc4 => vec![Note::E, Note::Disis, Note::Fes],
            Pc::Pc5 => vec![Note::F, Note::Eis, Note::Geses],
            Pc::Pc6 => vec![Note::Fis, Note::Ges],
            Pc::Pc7 => vec![Note::G, Note::Fisis, Note::Aeses],
            Pc::Pc8 => vec![Note::Gis, Note::Aes],
            Pc::Pc9 => vec![Note::A, Note::Gisis, Note::Beses],
            Pc::Pc10 => vec![Note::Ais, Note::Bes],
            Pc::Pc11 => vec![Note::B, Note::Aisis, Note::Ces],
        }
    }
}

impl From<Pc> for i32 {
    fn from(pc: Pc) -> Self {
        Self::from(&pc)
    }
}

impl From<&Pc> for i32 {
    fn from(pc: &Pc) -> Self {
        match pc {
            Pc::Pc0 => 0,
            Pc::Pc1 => 1,
            Pc::Pc2 => 2,
            Pc::Pc3 => 3,
            Pc::Pc4 => 4,
            Pc::Pc5 => 5,
            Pc::Pc6 => 6,
            Pc::Pc7 => 7,
            Pc::Pc8 => 8,
            Pc::Pc9 => 9,
            Pc::Pc10 => 10,
            Pc::Pc11 => 11,
        }
    }
}

impl From<Pc> for u8 {
    fn from(pc: Pc) -> Self {
        Self::from(&pc)
    }
}

impl From<&Pc> for u8 {
    fn from(pc: &Pc) -> Self {
        match pc {
            Pc::Pc0 => 0,
            Pc::Pc1 => 1,
            Pc::Pc2 => 2,
            Pc::Pc3 => 3,
            Pc::Pc4 => 4,
            Pc::Pc5 => 5,
            Pc::Pc6 => 6,
            Pc::Pc7 => 7,
            Pc::Pc8 => 8,
            Pc::Pc9 => 9,
            Pc::Pc10 => 10,
            Pc::Pc11 => 11,
        }
    }
}

impl From<u8> for Pc {
    fn from(pc: u8) -> Self {
        Pc::from(&pc)
    }
}

impl From<&u8> for Pc {
    fn from(pc: &u8) -> Self {
        let pc = pc.rem_euclid(12);
        match pc {
            0 => Pc::Pc0,
            1 => Pc::Pc1,
            2 => Pc::Pc2,
            3 => Pc::Pc3,
            4 => Pc::Pc4,
            5 => Pc::Pc5,
            6 => Pc::Pc6,
            7 => Pc::Pc7,
            8 => Pc::Pc8,
            9 => Pc::Pc9,
            10 => Pc::Pc10,
            11 => Pc::Pc11,
            _ => unreachable!(),
        }
    }
}

impl From<i32> for Pc {
    fn from(pc: i32) -> Self {
        let pc = pc.rem_euclid(12);
        match pc {
            0 => Pc::Pc0,
            1 => Pc::Pc1,
            2 => Pc::Pc2,
            3 => Pc::Pc3,
            4 => Pc::Pc4,
            5 => Pc::Pc5,
            6 => Pc::Pc6,
            7 => Pc::Pc7,
            8 => Pc::Pc8,
            9 => Pc::Pc9,
            10 => Pc::Pc10,
            11 => Pc::Pc11,
            _ => unreachable!(),
        }
    }
}

impl From<Note> for Pc {
    fn from(note: Note) -> Self {
        Pc::from(&note)
    }
}

impl From<&mut Note> for Pc {
    fn from(note: &mut Note) -> Self {
        Pc::from(note.deref())
    }
}

impl From<&Note> for Pc {
    fn from(note: &Note) -> Self {
        match note {
            Note::C => Pc::Pc0,
            Note::Deses => Pc::Pc0,
            Note::Cis => Pc::Pc1,
            Note::Des => Pc::Pc1,
            Note::Cisis => Pc::Pc2,
            Note::D => Pc::Pc2,
            Note::Eeses => Pc::Pc2,
            Note::Dis => Pc::Pc3,
            Note::Ees => Pc::Pc3,
            Note::Disis => Pc::Pc4,
            Note::E => Pc::Pc4,
            Note::Fes => Pc::Pc4,
            Note::Eis => Pc::Pc5,
            Note::F => Pc::Pc5,
            Note::Geses => Pc::Pc5,
            Note::Fis => Pc::Pc6,
            Note::Ges => Pc::Pc6,
            Note::Fisis => Pc::Pc7,
            Note::G => Pc::Pc7,
            Note::Aeses => Pc::Pc7,
            Note::Gis => Pc::Pc8,
            Note::Aes => Pc::Pc8,
            Note::Gisis => Pc::Pc9,
            Note::A => Pc::Pc9,
            Note::Beses => Pc::Pc9,
            Note::Ais => Pc::Pc10,
            Note::Bes => Pc::Pc10,
            Note::Aisis => Pc::Pc11,
            Note::B => Pc::Pc11,
            Note::Ces => Pc::Pc11,
            Note::Bis => Pc::Pc0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iteration_works() {
        let all_pcs: Vec<Pc> = PcIter::default().into_iter().collect::<Vec<Pc>>();
        assert_eq!(all_pcs.len(), 12);
        assert_eq!(all_pcs.first().cloned(), Some(Pc::Pc0));
        let all_pcs: Vec<Pc> = PcIter::starting_on(&Pc::Pc4)
            .into_iter()
            .collect::<Vec<Pc>>();
        assert_eq!(all_pcs.len(), 12);
        assert_eq!(all_pcs.first().cloned(), Some(Pc::Pc4));
        let all_pcs: Vec<Pc> = PcIter::section(&Pc::Pc4, &Pc::Pc3)
            .into_iter()
            .collect::<Vec<Pc>>();
        assert_eq!(all_pcs.len(), 11);
        assert_eq!(all_pcs.first().cloned(), Some(Pc::Pc4));
        assert_eq!(all_pcs.last().cloned(), Some(Pc::Pc2));
    }
}
