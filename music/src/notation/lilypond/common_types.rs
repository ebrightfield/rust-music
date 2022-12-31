use itertools::Itertools;
use crate::notation::lilypond::ToLilypondString;
use crate::{Note, Pitch, Spelling, Voicing};
use crate::notation::clef::Clef;
use crate::notation::duration::{Duration, DurationKind};
use crate::notation::rhythm::{DurationIn32ndNotes, Meter, NotatedEvent, RhythmicNotatedEvent, SingleEvent};
use crate::note::pitch::MIDDLE_C;
use crate::note::spelling::Accidental;

impl ToLilypondString for Meter {
    fn to_lilypond_string(&self) -> String {
        format!("{}/{}", self.num_beats, self.denominator.to_string())
    }
}

impl ToLilypondString for DurationKind {
    fn to_lilypond_string(&self) -> String {
        match &self {
            DurationKind::Breve => "\\breve",
            DurationKind::Whole => "1",
            DurationKind::Half => "2",
            DurationKind::Qtr => "4",
            DurationKind::Eighth => "8",
            DurationKind::Sixteenth => "16",
            DurationKind::ThirtySecond => "32",
            DurationKind::SixtyFourth => "64",
            DurationKind::OneTwentyEighth => "128",
        }.to_string()
    }
}

impl ToLilypondString for Duration {
    fn to_lilypond_string(&self) -> String {
        let kind = self.kind().to_lilypond_string();
        let dots = ".".repeat(self.num_dots() as usize);
        format!("{}{}", kind, dots)
    }
}

// TODO There could almost be an enum for valid, "atomically engravable" durations
// TODO Perform automatic tied conversions for rhythmic durations that need to be
//    expressed as tied values.
pub fn ly_duration(duration: DurationIn32ndNotes) -> Option<String> {
    Some(match duration {
        1 => "32",
        2 => "16",
        3 => "16.",
        4 => "8",
        6 => "8.",
        7 => "8..",
        8 => "4",
        12 => "4.",
        14 => "4..",
        15 => "4...",
        16 => "2",
        24 => "2.",
        28 => "2..",
        30 => "2...",
        31 => "2....",
        32 => "1",
        48 => "1.",
        56 => "1..",
        60 => "1...",
        62 => "1....",
        63 => "1.....",
        64 => "\\breve",
        0 => {
            return None;
        },
        _ => {
            return None;
        },
    }.to_string())
}

impl ToLilypondString for Clef {
    fn to_lilypond_string(&self) -> String {
        let clef_name = match &self {
            Clef::Treble => "treble",
        };
        format!("\\clef {}\n", clef_name)
    }
}

impl ToLilypondString for Note {
    fn to_lilypond_string(&self) -> String {
        let spelling = Spelling::from(self);
        let letter = spelling.letter.to_string().to_lowercase();
        let acc = match spelling.acc {
            Accidental::Natural => "",
            Accidental::Sharp => "is",
            Accidental::Flat => "es",
            Accidental::DoubleSharp => "isis",
            Accidental::DoubleFlat => "eses",
        };
        format!("{}{}", letter, acc)
    }
}

/// Does not use relative pitch
impl ToLilypondString for Pitch {
    fn to_lilypond_string(&self) -> String {
        let note = self.note.to_lilypond_string();
        let mut octave = self.octave;
        if self.note == Note::Ces {
            octave += 1;
        } else if self.note == Note::Bis {
            octave -= 1;
        }
        let octave = if self.midi_note < MIDDLE_C {
            // Octave will always be <= 3 here
            ",".repeat((3 - octave) as usize)
        } else {
            "'".repeat((octave - 3) as usize)
        };
        format!("{}{}", note, octave)
    }
}

impl ToLilypondString for Voicing {
    fn to_lilypond_string(&self) -> String {
        let inner: String = self.iter()
            .map(|p| p.to_lilypond_string())
            .join(" ");
        format!("<{}>", inner)
    }
}

// TODO You need to account for meter and offset, and convert
//    some durations into tied composite notated events.
impl ToLilypondString for RhythmicNotatedEvent {
    fn to_lilypond_string(&self) -> String {
        let pitches = match &self.event {
            NotatedEvent::SingleEvent(event) => {
                match event {
                    SingleEvent::Pitch(p) => {
                        p.to_lilypond_string()
                    }
                    SingleEvent::Voicing(v) => {
                        v.to_lilypond_string()
                    }
                    SingleEvent::Rest => {
                        "r".to_string()
                    }
                }
            }
            NotatedEvent::Tuple(_) => todo!()
        };
        let duration = self.duration.to_lilypond_string();
        format!("{}{}", pitches, duration)
    }
}