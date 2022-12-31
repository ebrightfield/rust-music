use crate::fretboard::fretboard_shape::FretboardShape;
use crate::fretboard::fretted_note::{FrettedNote, SoundedNote};
use crate::notation::rhythm::duration::{Duration, DurationKind};
use crate::notation::rhythm::{NotatedEvent, RhythmicNotatedEvent, SingleEvent};
use crate::note::pitch::Pitch;
use crate::note_collections::voicing::Voicing;

pub trait ToVexTab {
    fn to_vextab(&self) -> String;
}

impl ToVexTab for Pitch {
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

impl ToVexTab for Voicing {
    fn to_vextab(&self) -> String {
        format!("({})", self.iter().map(|pitch| {
            pitch.to_vextab()
        }).collect::<Vec<_>>().join("."))
    }
}

impl ToVexTab for DurationKind {
    fn to_vextab(&self) -> String {
        match self {
            DurationKind::Whole => ":w",
            DurationKind::Half => ":h",
            DurationKind::Qtr => ":q",
            DurationKind::Eighth => ":8",
            DurationKind::Sixteenth => ":16",
            DurationKind::ThirtySecond => ":32",
            _ => panic!("Unsupported rhythmic duration for Vextab")
        }.to_string()
    }
}

impl ToVexTab for Duration {
    fn to_vextab(&self) -> String {
        let dots = "d".repeat(self.num_dots() as usize);
        let kind = self.kind().to_vextab();
        format!("{}{}", kind, dots)
    }
}

impl ToVexTab for RhythmicNotatedEvent {
    fn to_vextab(&self) -> String {
        let pitch_content = match &self.event {
            NotatedEvent::SingleEvent(e, d) => {
                let pitch_content = e.to_vextab();
                let duration = d.to_vextab();
                return format!("{}{}", duration, pitch_content)
            }
            NotatedEvent::Tuplet(_) => todo!()
        };
    }
}

impl ToVexTab for SingleEvent {
    fn to_vextab(&self) -> String {
        match self {
            SingleEvent::Pitch(p) => p.to_vextab(),
            SingleEvent::Voicing(v) => v.to_vextab(),
            SingleEvent::Rest => "##".to_string(),
        }
    }
}

pub mod barline {
    pub const BAR: &str = "|";
    pub const DOUBLE_BAR: &str = "=||";
    pub const REPEAT_BEGIN: &str = "=|:";
    pub const REPEAT_END: &str = "=:|";
    pub const DOUBLE_REPEAT: &str = "=::";
    pub const END_BAR: &str = "=|=";
}