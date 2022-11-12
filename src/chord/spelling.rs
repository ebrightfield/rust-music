use anyhow::{anyhow, Result};
use crate::chord::pc_set::PcSet;
use crate::note::note::*;
use crate::note::pc::Pc;
use crate::note::spelling::Spelling;

pub fn spell_pc_set(root: &Note, pc_set: &PcSet) -> Result<Vec<Note>> {
    if Spelling::from(root).acc.is_double() {
        return Err(anyhow!("Double accidentals are not valid roots for spelling. \
        Use a different note and rotate it instead."))
    }
    Ok(pc_set.0
        .iter()
        .map(|pc| {
            // Unwraps are safe here because we screened out double-accidentals
            let default_spelling = default_spelling(root, pc).unwrap();
            let rules = spell_rules(root).unwrap();
            for rule in rules {
                if rule.applied(*pc, pc_set) {
                    return default_spelling.enharmonic_flip_bcef();
                }
            }
            default_spelling
        })
        .collect())
}

pub struct SpellingRule {
    pc: Pc,
    incl: Vec<Pc>,
    excl: Vec<Pc>,
    not_all: Vec<Pc>,
}

impl SpellingRule {
    // Returns true when a rule is flagged,
    // meaning a note needs to be flipped enharmonically.
    pub fn applied(&self, pc: Pc, pc_set: &PcSet) -> bool {
        // We pass if the rule does not pertain to the given pc.
        if self.pc != pc {
            return false;
        }
        // We pass if not all the pcs in self.incl are in pc_set
        if !self.incl.iter().all(|incl_pc| pc_set.0.contains(incl_pc)) {
            return false;
        }
        // We pass if pc_set contains anything that the rule indicates should be excluded
        if self.excl.iter().any(|excl_pc| pc_set.0.contains(excl_pc)) {
            return false;
        }
        // We pass if self.not_all is not empty, and pc_set contains all of them.
        if !self.not_all.is_empty() && self.not_all.iter().all(|not_all_pc| pc_set.0.contains(not_all_pc)) {
            return false;
        }
        // Otherwise, the rule is flagging the spelling, and suggests an aggressive
        // enharmonic flip.
        true
    }
}

pub fn spell_rules(note: &Note) -> Option<Vec<SpellingRule>> {
    match *note {
        Note::C => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc4, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc10, Pc::Pc4],
                excl: vec![Pc::Pc8, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc3, Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::Cis => Some(vec![
            SpellingRule {
                pc: Pc::Pc4,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc4,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc9,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc10, Pc::Pc11, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc9,
                incl: vec![Pc::Pc8],
                excl: vec![Pc::Pc10, Pc::Pc11],
                not_all: vec![],
            },
        ]),
        Note::Des => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc7],
                excl: vec![Pc::Pc9],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc10,
                incl: vec![Pc::Pc9],
                excl: vec![],
                not_all: vec![],
            },
        ]),
        Note::D => Some(vec![
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc4, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc10, Pc::Pc4],
                excl: vec![Pc::Pc8, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::Dis => Some(vec![SpellingRule {
            pc: Pc::Pc2,
            incl: vec![],
            excl: vec![Pc::Pc3, Pc::Pc4],
            not_all: vec![],
        }]),
        Note::Ees => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc4, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
        ]),
        Note::E => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc2, Pc::Pc6],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc4, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc10, Pc::Pc4],
                excl: vec![Pc::Pc8, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc3, Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::F => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc4, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc10, Pc::Pc4],
                excl: vec![Pc::Pc8, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc3, Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::Fis => Some(vec![
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc7],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5],
                not_all: vec![],
            },
        ]),
        Note::Ges => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc7],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc10,
                incl: vec![Pc::Pc11],
                excl: vec![],
                not_all: vec![],
            },
        ]),
        Note::G => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc4, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc10, Pc::Pc4],
                excl: vec![Pc::Pc8, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc3, Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::Gis => Some(vec![
            SpellingRule {
                pc: Pc::Pc4,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc4,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc9,
                incl: vec![Pc::Pc8],
                excl: vec![Pc::Pc10, Pc::Pc11],
                not_all: vec![],
            },
        ]),
        Note::Aes => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::A => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc4, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc10, Pc::Pc4],
                excl: vec![Pc::Pc8, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::Ais => Some(vec![
            SpellingRule {
                pc: Pc::Pc2,
                incl: vec![Pc::Pc1],
                excl: vec![Pc::Pc3],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc4,
                incl: vec![Pc::Pc3],
                excl: vec![Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc4,
                incl: vec![Pc::Pc6],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc9,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::Bes => Some(vec![
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc2],
            },
            SpellingRule {
                pc: Pc::Pc3,
                incl: vec![Pc::Pc4],
                excl: vec![],
                not_all: vec![Pc::Pc1, Pc::Pc5],
            },
            SpellingRule {
                pc: Pc::Pc6,
                incl: vec![Pc::Pc5],
                excl: vec![],
                not_all: vec![],
            },
            SpellingRule {
                pc: Pc::Pc8,
                incl: vec![Pc::Pc4],
                excl: vec![Pc::Pc5, Pc::Pc7],
                not_all: vec![],
            },
        ]),
        Note::B => Some(vec![]),
        Note::Bis => Some(vec![]),
        Note::Ces => Some(vec![]),
        Note::Eis => Some(vec![]),
        Note::Fes => Some(vec![]),

        _ => None,
    }
}

pub fn default_spelling(root: &Note, pc: &Pc) -> Option<Note> {
    match &root {
        Note::C => match *pc {
            Pc::Pc0 => Some(Note::C),
            Pc::Pc1 => Some(Note::Des),
            Pc::Pc2 => Some(Note::D),
            Pc::Pc3 => Some(Note::Ees),
            Pc::Pc4 => Some(Note::E),
            Pc::Pc5 => Some(Note::F),
            Pc::Pc6 => Some(Note::Fis),
            Pc::Pc7 => Some(Note::G),
            Pc::Pc8 => Some(Note::Aes),
            Pc::Pc9 => Some(Note::A),
            Pc::Pc10 => Some(Note::Bes),
            Pc::Pc11 => Some(Note::B),
        },
        Note::Cis => match *pc {
            Pc::Pc0 => Some(Note::Cis),
            Pc::Pc1 => Some(Note::D),
            Pc::Pc2 => Some(Note::Dis),
            Pc::Pc3 => Some(Note::E),
            Pc::Pc4 => Some(Note::Eis),
            Pc::Pc5 => Some(Note::Fis),
            Pc::Pc6 => Some(Note::G),
            Pc::Pc7 => Some(Note::Gis),
            Pc::Pc8 => Some(Note::A),
            Pc::Pc9 => Some(Note::Ais),
            Pc::Pc10 => Some(Note::B),
            Pc::Pc11 => Some(Note::Bis),
        },
        Note::Des => match *pc {
            Pc::Pc0 => Some(Note::Des),
            Pc::Pc1 => Some(Note::Eeses),
            Pc::Pc2 => Some(Note::Ees),
            Pc::Pc3 => Some(Note::Fes),
            Pc::Pc4 => Some(Note::F),
            Pc::Pc5 => Some(Note::Ges),
            Pc::Pc6 => Some(Note::G),
            Pc::Pc7 => Some(Note::Aes),
            Pc::Pc8 => Some(Note::A),
            Pc::Pc9 => Some(Note::Bes),
            Pc::Pc10 => Some(Note::B),
            Pc::Pc11 => Some(Note::C),
        },
        Note::D => match *pc {
            Pc::Pc0 => Some(Note::D),
            Pc::Pc1 => Some(Note::Ees),
            Pc::Pc2 => Some(Note::E),
            Pc::Pc3 => Some(Note::F),
            Pc::Pc4 => Some(Note::Fis),
            Pc::Pc5 => Some(Note::G),
            Pc::Pc6 => Some(Note::Gis),
            Pc::Pc7 => Some(Note::A),
            Pc::Pc8 => Some(Note::Bes),
            Pc::Pc9 => Some(Note::B),
            Pc::Pc10 => Some(Note::C),
            Pc::Pc11 => Some(Note::Cis),
        },
        Note::Dis => match *pc {
            Pc::Pc0 => Some(Note::Dis),
            Pc::Pc1 => Some(Note::E),
            Pc::Pc2 => Some(Note::Eis),
            Pc::Pc3 => Some(Note::Fis),
            Pc::Pc4 => Some(Note::Fisis),
            Pc::Pc5 => Some(Note::Gis),
            Pc::Pc6 => Some(Note::A),
            Pc::Pc7 => Some(Note::Ais),
            Pc::Pc8 => Some(Note::B),
            Pc::Pc9 => Some(Note::C),
            Pc::Pc10 => Some(Note::Cis),
            Pc::Pc11 => Some(Note::Cisis),
        },
        Note::Ees => match *pc {
            Pc::Pc0 => Some(Note::Ees),
            Pc::Pc1 => Some(Note::Fes),
            Pc::Pc2 => Some(Note::F),
            Pc::Pc3 => Some(Note::Ges),
            Pc::Pc4 => Some(Note::G),
            Pc::Pc5 => Some(Note::Aes),
            Pc::Pc6 => Some(Note::A),
            Pc::Pc7 => Some(Note::Bes),
            Pc::Pc8 => Some(Note::B),
            Pc::Pc9 => Some(Note::C),
            Pc::Pc10 => Some(Note::Des),
            Pc::Pc11 => Some(Note::D),
        },
        Note::E => match *pc {
            Pc::Pc0 => Some(Note::E),
            Pc::Pc1 => Some(Note::F),
            Pc::Pc2 => Some(Note::Fis),
            Pc::Pc3 => Some(Note::G),
            Pc::Pc4 => Some(Note::Gis),
            Pc::Pc5 => Some(Note::A),
            Pc::Pc6 => Some(Note::Ais),
            Pc::Pc7 => Some(Note::B),
            Pc::Pc8 => Some(Note::C),
            Pc::Pc9 => Some(Note::Cis),
            Pc::Pc10 => Some(Note::D),
            Pc::Pc11 => Some(Note::Dis),
        },
        Note::F => match *pc {
            Pc::Pc0 => Some(Note::F),
            Pc::Pc1 => Some(Note::Ges),
            Pc::Pc2 => Some(Note::G),
            Pc::Pc3 => Some(Note::Aes),
            Pc::Pc4 => Some(Note::A),
            Pc::Pc5 => Some(Note::Bes),
            Pc::Pc6 => Some(Note::B),
            Pc::Pc7 => Some(Note::C),
            Pc::Pc8 => Some(Note::Des),
            Pc::Pc9 => Some(Note::D),
            Pc::Pc10 => Some(Note::Ees),
            Pc::Pc11 => Some(Note::E),
        },
        Note::Fis => match *pc {
            Pc::Pc0 => Some(Note::Fis),
            Pc::Pc1 => Some(Note::G),
            Pc::Pc2 => Some(Note::Gis),
            Pc::Pc3 => Some(Note::A),
            Pc::Pc4 => Some(Note::Ais),
            Pc::Pc5 => Some(Note::B),
            Pc::Pc6 => Some(Note::C),
            Pc::Pc7 => Some(Note::Cis),
            Pc::Pc8 => Some(Note::D),
            Pc::Pc9 => Some(Note::Dis),
            Pc::Pc10 => Some(Note::E),
            Pc::Pc11 => Some(Note::Eis),
        },
        Note::Ges => match *pc {
            Pc::Pc0 => Some(Note::Ges),
            Pc::Pc1 => Some(Note::Aeses),
            Pc::Pc2 => Some(Note::Aes),
            Pc::Pc3 => Some(Note::Beses),
            Pc::Pc4 => Some(Note::Bes),
            Pc::Pc5 => Some(Note::Ces),
            Pc::Pc6 => Some(Note::C),
            Pc::Pc7 => Some(Note::Des),
            Pc::Pc8 => Some(Note::D),
            Pc::Pc9 => Some(Note::Ees),
            Pc::Pc10 => Some(Note::Fes),
            Pc::Pc11 => Some(Note::F),
        },
        Note::G => match *pc {
            Pc::Pc0 => Some(Note::G),
            Pc::Pc1 => Some(Note::Aes),
            Pc::Pc2 => Some(Note::A),
            Pc::Pc3 => Some(Note::Bes),
            Pc::Pc4 => Some(Note::B),
            Pc::Pc5 => Some(Note::C),
            Pc::Pc6 => Some(Note::Cis),
            Pc::Pc7 => Some(Note::D),
            Pc::Pc8 => Some(Note::Ees),
            Pc::Pc9 => Some(Note::E),
            Pc::Pc10 => Some(Note::F),
            Pc::Pc11 => Some(Note::Fis),
        },
        Note::Gis => match *pc {
            Pc::Pc0 => Some(Note::Gis),
            Pc::Pc1 => Some(Note::A),
            Pc::Pc2 => Some(Note::Ais),
            Pc::Pc3 => Some(Note::B),
            Pc::Pc4 => Some(Note::Bis),
            Pc::Pc5 => Some(Note::Cis),
            Pc::Pc6 => Some(Note::D),
            Pc::Pc7 => Some(Note::Dis),
            Pc::Pc8 => Some(Note::E),
            Pc::Pc9 => Some(Note::Eis),
            Pc::Pc10 => Some(Note::Fis),
            Pc::Pc11 => Some(Note::Fisis),
        },
        Note::Aes => match *pc {
            Pc::Pc0 => Some(Note::Aes),
            Pc::Pc1 => Some(Note::Beses),
            Pc::Pc2 => Some(Note::Bes),
            Pc::Pc3 => Some(Note::Ces),
            Pc::Pc4 => Some(Note::C),
            Pc::Pc5 => Some(Note::Des),
            Pc::Pc6 => Some(Note::D),
            Pc::Pc7 => Some(Note::Ees),
            Pc::Pc8 => Some(Note::Fes),
            Pc::Pc9 => Some(Note::F),
            Pc::Pc10 => Some(Note::Ges),
            Pc::Pc11 => Some(Note::G),
        },
        Note::A => match *pc {
            Pc::Pc0 => Some(Note::A),
            Pc::Pc1 => Some(Note::Bes),
            Pc::Pc2 => Some(Note::B),
            Pc::Pc3 => Some(Note::C),
            Pc::Pc4 => Some(Note::Cis),
            Pc::Pc5 => Some(Note::D),
            Pc::Pc6 => Some(Note::Dis),
            Pc::Pc7 => Some(Note::E),
            Pc::Pc8 => Some(Note::F),
            Pc::Pc9 => Some(Note::Fis),
            Pc::Pc10 => Some(Note::G),
            Pc::Pc11 => Some(Note::Gis),
        },
        Note::Ais => match *pc {
            Pc::Pc0 => Some(Note::Ais),
            Pc::Pc1 => Some(Note::B),
            Pc::Pc2 => Some(Note::Bis),
            Pc::Pc3 => Some(Note::Cis),
            Pc::Pc4 => Some(Note::Cisis),
            Pc::Pc5 => Some(Note::Dis),
            Pc::Pc6 => Some(Note::E),
            Pc::Pc7 => Some(Note::Eis),
            Pc::Pc8 => Some(Note::Fis),
            Pc::Pc9 => Some(Note::Fisis),
            Pc::Pc10 => Some(Note::Gis),
            Pc::Pc11 => Some(Note::Gisis),
        },
        Note::Bes => match *pc {
            Pc::Pc0 => Some(Note::Bes),
            Pc::Pc1 => Some(Note::Ces),
            Pc::Pc2 => Some(Note::C),
            Pc::Pc3 => Some(Note::Des),
            Pc::Pc4 => Some(Note::D),
            Pc::Pc5 => Some(Note::Ees),
            Pc::Pc6 => Some(Note::E),
            Pc::Pc7 => Some(Note::F),
            Pc::Pc8 => Some(Note::Ges),
            Pc::Pc9 => Some(Note::G),
            Pc::Pc10 => Some(Note::Aes),
            Pc::Pc11 => Some(Note::A),
        },
        Note::B => match *pc {
            Pc::Pc0 => Some(Note::B),
            Pc::Pc1 => Some(Note::C),
            Pc::Pc2 => Some(Note::Cis),
            Pc::Pc3 => Some(Note::D),
            Pc::Pc4 => Some(Note::Dis),
            Pc::Pc5 => Some(Note::E),
            Pc::Pc6 => Some(Note::F),
            Pc::Pc7 => Some(Note::Fis),
            Pc::Pc8 => Some(Note::G),
            Pc::Pc9 => Some(Note::Gis),
            Pc::Pc10 => Some(Note::A),
            Pc::Pc11 => Some(Note::Ais),
        },
        Note::Bis => match *pc {
            Pc::Pc0 => Some(Note::Bis),
            Pc::Pc1 => Some(Note::Cis),
            Pc::Pc2 => Some(Note::D),
            Pc::Pc3 => Some(Note::Dis),
            Pc::Pc4 => Some(Note::E),
            Pc::Pc5 => Some(Note::F),
            Pc::Pc6 => Some(Note::Fis),
            Pc::Pc7 => Some(Note::G),
            Pc::Pc8 => Some(Note::Gis),
            Pc::Pc9 => Some(Note::A),
            Pc::Pc10 => Some(Note::Ais),
            Pc::Pc11 => Some(Note::Aisis),
        },
        Note::Ces => match *pc {
            Pc::Pc0 => Some(Note::Ces),
            Pc::Pc1 => Some(Note::Deses),
            Pc::Pc2 => Some(Note::Des),
            Pc::Pc3 => Some(Note::Eeses),
            Pc::Pc4 => Some(Note::Ees),
            Pc::Pc5 => Some(Note::Fes),
            Pc::Pc6 => Some(Note::F),
            Pc::Pc7 => Some(Note::Ges),
            Pc::Pc8 => Some(Note::Aeses),
            Pc::Pc9 => Some(Note::Aes),
            Pc::Pc10 => Some(Note::Beses),
            Pc::Pc11 => Some(Note::Bes),
        },
        Note::Eis => match *pc {
            Pc::Pc0 => Some(Note::Eis),
            Pc::Pc1 => Some(Note::Fis),
            Pc::Pc2 => Some(Note::G),
            Pc::Pc3 => Some(Note::Gis),
            Pc::Pc4 => Some(Note::A),
            Pc::Pc5 => Some(Note::Ais),
            Pc::Pc6 => Some(Note::B),
            Pc::Pc7 => Some(Note::C),
            Pc::Pc8 => Some(Note::Cis),
            Pc::Pc9 => Some(Note::D),
            Pc::Pc10 => Some(Note::Dis),
            Pc::Pc11 => Some(Note::Disis),
        },
        Note::Fes => match *pc {
            Pc::Pc0 => Some(Note::Fes),
            Pc::Pc1 => Some(Note::Geses),
            Pc::Pc2 => Some(Note::Ges),
            Pc::Pc3 => Some(Note::Aeses),
            Pc::Pc4 => Some(Note::Aes),
            Pc::Pc5 => Some(Note::Beses),
            Pc::Pc6 => Some(Note::Bes),
            Pc::Pc7 => Some(Note::Ces),
            Pc::Pc8 => Some(Note::Deses),
            Pc::Pc9 => Some(Note::Des),
            Pc::Pc10 => Some(Note::Eeses),
            Pc::Pc11 => Some(Note::Ees),
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_basic_spelling() {
        let pc_set = PcSet(vec![Pc::Pc0, Pc::Pc4, Pc::Pc7, Pc::Pc11]);
        let root = Note::C;
        let spelling = spell_pc_set(&root, &pc_set).unwrap();
        assert_eq!(
            spelling,
            vec![Note::C, Note::E, Note::G, Note::B],
        )
    }
}