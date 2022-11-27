use crate::chord::pc_set::PcSet;
use crate::fretboard::{Fretboard, FrettedNote, FrettedNoteKind};
use crate::pitch::Pitch;
use itertools::Itertools;
use crate::note::note::Note;

#[derive(Debug)]
pub struct FretboardShape<'a> {
    pub strings: Vec<FrettedNote<'a>>,
    pub fretboard: &'a Fretboard,
}

impl<'a> FretboardShape<'a> {
    pub fn span(&self) -> (u8, u8) {
        let mut lowest: u8 = 0;
        let mut highest: u8  = 0;
        for fretted_note in &self.strings {
            match &fretted_note.kind {
                FrettedNoteKind::Fretted { fret, .. } => {
                    if *fret < lowest {
                        lowest = *fret;
                    }
                    if *fret > highest {
                        highest = *fret;
                    }
                },
                FrettedNoteKind::Open(_) => {
                    lowest = 0;
                },
                _ => {}
            }
        }
        (lowest, highest)
    }

    /// Since frets are spelling-agnostic, we compare by MIDI note value
    /// to equivocate over accidentals but not octaves.
    pub fn contains(&self, member: &Pitch) -> bool {
        for fretted_note in &self.strings {
            match &fretted_note.kind {
                FrettedNoteKind::Fretted { pitch, .. } => {
                    if pitch.midi_note == member.midi_note {
                        return true;
                    }
                },
                FrettedNoteKind::Open(pitch) => {
                    if pitch.midi_note == member.midi_note {
                        return true;
                    }
                },
                _ => {}
            }
        }
        false
    }
}


pub fn find_chord_shapes<'a>(chord: Vec<Note>, fretboard: &'a Fretboard) -> anyhow::Result<Vec<FretboardShape<'a>>> {
    let chord_len = chord.len();
    let num_strings = fretboard.num_strings();
    let note_permutations = chord.iter().permutations(chord_len);
    let string_groupings = (0..num_strings).combinations(chord_len);
    let strings = vec![FrettedNote {
        kind: FrettedNoteKind::Muted,
        string: 0,
        fretboard
    }];
    let shape = FretboardShape {
        strings,
        fretboard,
    };
    Ok(vec![shape])
}