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
        format!("{}/{}", self.fret, string)
    }
}

impl<'a> ToVexTab for FrettedNote<'a> {
    fn to_vextab(&self) -> String {
        match &self {
            FrettedNote::Sounded(sounded_note) => sounded_note.to_vextab(),
            FrettedNote::Muted { .. } => "".to_string(),
        }
    }
}

impl<'a> ToVexTab for FretboardShape<'a> {
    fn to_vextab(&self) -> String {
        self.fretted_notes
            .iter()
            .filter_map(|item| match &item {
                FrettedNote::Sounded(note) => Some(note),
                FrettedNote::Muted { .. } => None,
            })
            .collect::<Vec<_>>()
            .to_vextab()
    }
}

impl<'a> ToVexTab for Vec<&SoundedNote<'a>> {
    fn to_vextab(&self) -> String {
        let notes = self
            .iter()
            .map(|fretted_note| fretted_note.to_vextab())
            .collect::<Vec<String>>()
            .join(".");
        format!("({notes})")
    }
}

impl ToVexTab for Voicing {
    fn to_vextab(&self) -> String {
        format!(
            "({})",
            self.iter()
                .map(|pitch| { pitch.to_vextab() })
                .collect::<Vec<_>>()
                .join(".")
        )
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
            _ => panic!("Unsupported rhythmic duration for Vextab"),
        }
        .to_string()
    }
}

impl ToVexTab for Duration {
    fn to_vextab(&self) -> String {
        let dots = "d".repeat(self.num_dots() as usize);
        let kind = self.kind().to_vextab();
        format!("{}{}", kind, dots)
    }
}

impl<'a> ToVexTab for RhythmicNotatedEvent<'a> {
    fn to_vextab(&self) -> String {
        match &self.event {
            NotatedEvent::SingleEvent(e, d) => {
                let pitch_content = e.to_vextab();
                let duration = d.to_vextab();
                format!("{}{}", duration, pitch_content)
            }
            NotatedEvent::Tuplet(_) => todo!(),
        }
    }
}

impl<'a> ToVexTab for SingleEvent<'a> {
    fn to_vextab(&self) -> String {
        match self {
            SingleEvent::Pitch(p) => p.to_vextab(),
            SingleEvent::Voicing(v) => v.to_vextab(),
            SingleEvent::Fretted(s) => s.to_vextab(),
            SingleEvent::FrettedMany(notes) => {
                let notes = notes
                    .iter()
                    .map(|fretted_note| fretted_note.to_vextab())
                    .collect::<Vec<String>>()
                    .join(".");
                format!("({notes})")
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::note::Note;

    /// VexTab keys name the written letter's octave, so a MIDI note spelled
    /// C♭ or B♯ carries the octave of that spelling, not of its sounding C..B
    /// block.
    #[test]
    fn pitch_keys_use_the_written_octave() {
        let c_flat_5 = Pitch::from_midi_spelled_as(71, &vec![Note::Ces]).unwrap();
        assert_eq!(c_flat_5.to_vextab(), "C@/5");
        let b_sharp_3 = Pitch::from_midi_spelled_as(60, &vec![Note::Bis]).unwrap();
        assert_eq!(b_sharp_3.to_vextab(), "B#/3");
        assert_eq!(Pitch::new(Note::Bes, 4).to_vextab(), "B@/4");
    }
}
