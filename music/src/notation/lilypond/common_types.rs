use itertools::Itertools;
use crate::notation::lilypond::ToLilypondString;
use crate::{Note, Pitch, Spelling, Voicing};
use crate::notation::clef::Clef;
use crate::notation::rhythm::duration::{Duration, DurationKind};
use crate::notation::rhythm::{NotatedEvent, RhythmicNotatedEvent, SingleEvent};
use crate::notation::rhythm::meter::Meter;
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
        match &self.event {
            NotatedEvent::SingleEvent(event, duration) => {
                let pitches = match event {
                    SingleEvent::Pitch(p) => {
                        p.to_lilypond_string()
                    }
                    SingleEvent::Voicing(v) => {
                        v.to_lilypond_string()
                    }
                    SingleEvent::Rest => {
                        "r".to_string()
                    }
                };
                let duration = duration.to_lilypond_string();
                format!("{}{}", pitches, duration)
            }
            NotatedEvent::Tuplet(_) => todo!()
        }
    }
}