use itertools::Itertools;
use crate::notation::lilypond::ToLilypondString;
use crate::{Note, Pitch, Spelling, Voicing};
use crate::notation::clef::Clef;
use crate::notation::rhythm::duration::{Duration, DurationKind};
use crate::notation::rhythm::{NotatedEvent, RhythmicNotatedEvent, SingleEvent};
use crate::notation::rhythm::meter::Meter;
use crate::note::pitch::MIDDLE_C;
use crate::note::spelling::Accidental;

/// Lilypond represents time signatures as simple fractions
impl ToLilypondString for Meter {
    fn to_lilypond_string(&self) -> String {
        format!("{}/{}", self.num_beats, self.denominator.to_string())
    }
}

/// The only tricky conversion here is the double-whole note `\breve`,
/// otherwise everything converts to the integer string value you'd expect.
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

/// The actual complete clef declaration is in the [LilypondStaff].
impl ToLilypondString for Clef {
    fn to_lilypond_string(&self) -> String {
        match &self {
            Clef::Treble => "treble",
            Clef::Treble8va => "treble^8",
            Clef::Treble8ba => "treble_8",
            Clef::Bass => "bass",
        }.to_string()
    }
}

/// Lilypond uses the solfege-style "is" (pronounced "ees") for "sharp", and "es" for "flat".
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

/// Space separated interior elements, surrounded by `<` `>` angle brackets.
impl ToLilypondString for Voicing {
    fn to_lilypond_string(&self) -> String {
        let inner: String = self.iter()
            .map(|p| p.to_lilypond_string())
            .join(" ");
        format!("<{}>", inner)
    }
}

/// This is where the duration and content are combined into an element
/// that can be rendered by Lilypond.
impl<'a> ToLilypondString for RhythmicNotatedEvent<'a> {
    fn to_lilypond_string(&self) -> String {
        match &self.event {
            NotatedEvent::SingleEvent(event, duration) => {
                let duration = duration.to_lilypond_string();
                match event {
                    SingleEvent::Pitch(p) => {
                        format!("{}{}", p.to_lilypond_string(), duration)
                    }
                    SingleEvent::Voicing(v) => {
                        format!("{}{}", v.to_lilypond_string(), duration)
                    },
                    SingleEvent::Fretted(s) => {
                        let pitch = s.pitch.to_lilypond_string();
                        format!("{}{}\\{}", pitch, duration, s.string)
                    },
                    SingleEvent::FrettedMany(notes) => {
                        let inner: String = notes.iter()
                            .map(|f| {
                                let pitch = f.pitch.to_lilypond_string();
                                format!("{}{}\\{}", pitch, duration, f.string)
                            })
                            .join(" ");
                        format!("<{}>{}", inner, duration)
                    }
                    SingleEvent::Rest => {
                        format!("r{}", duration)
                    }
                }
            }
            NotatedEvent::Tuplet(tuplet) => {
                let ratio = format!("{}/{}", tuplet.numerator, tuplet.denominator);
                let content = tuplet.events.iter()
                    .map(|event| event.to_lilypond_string())
                    .join(" ");
                // Notate the tuplet
                format!("\\tuplet {} {{ {} }}", ratio, content)
            }
        }
    }
}