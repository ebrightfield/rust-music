use crate::fretboard::fretboard_shape::FretboardShape;
use crate::fretboard::fretted_note::{FrettedNote, SoundedNote};
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::Voicing;

pub trait ToVexTab {
    fn to_vextab(&self) -> String;
}

impl<'a> ToVexTab for Pitch {
    fn to_vextab(&self) -> String {
        format!("{}/{}", self.note, self.octave).replacen("b", "@", 2)
    }
}

impl<'a> ToVexTab for SoundedNote<'a> {
    fn to_vextab(&self) -> String {
        let string = (self.fretboard.num_strings() - self.string).to_string();
        let fret = self.fret.to_string();
        format!("{}/{}", fret, string)
    }
}

impl<'a> ToVexTab for FrettedNote<'a> {
    fn to_vextab(&self) -> String {
        match &self {
            FrettedNote::Sounded(sounded_note) => sounded_note.to_vextab(),
            FrettedNote::Muted { .. } => "".to_string()
        }
    }
}

impl<'a> ToVexTab for FretboardShape<'a> {
    fn to_vextab(&self) -> String {
        let notes = self.fretted_notes.iter()
            .filter(|item| match &item {
                FrettedNote::Sounded(_) => true,
                FrettedNote::Muted { .. } => false,
            })
            .map(|fretted_note| {
                fretted_note.to_vextab()
            })
            .collect::<Vec<String>>()
            .join(".");
        format!("({notes})")
    }
}

impl<'a> ToVexTab for Voicing {
    fn to_vextab(&self) -> String {
        format!("({})", self.iter().map(|pitch| {
            pitch.to_vextab()
        }).collect::<Vec<_>>().join("."))
    }
}
