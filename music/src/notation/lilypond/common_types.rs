use crate::notation::clef::Clef;
use crate::notation::lilypond::ToLilypondString;
use crate::notation::rhythm::duration::{Duration, DurationKind};
use crate::notation::rhythm::meter::Meter;
use crate::notation::rhythm::{NotatedEvent, RhythmicNotatedEvent, SingleEvent};
use crate::note::spelling::Accidental;
use crate::{Note, Pitch, Spelling, Voicing};
use itertools::Itertools;

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
        }
        .to_string()
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
            Clef::Alto => "alto",
            Clef::Tenor => "tenor",
        }
        .to_string()
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

/// Does not use relative pitch.
///
/// LilyPond octave marks name the octave of the written letter, exactly like
/// [`Pitch::octave`]: `c'` is C4, so `ces'` is C♭4 (MIDI 59) and `bis` is
/// B♯3 (MIDI 60). No per-spelling adjustment is needed.
impl ToLilypondString for Pitch {
    fn to_lilypond_string(&self) -> String {
        let note = self.note.to_lilypond_string();
        let octave = self.octave as i32;
        let marks = if octave < 3 {
            ",".repeat((3 - octave) as usize)
        } else {
            "'".repeat((octave - 3) as usize)
        };
        format!("{}{}", note, marks)
    }
}

/// Space separated interior elements, surrounded by `<` `>` angle brackets.
impl ToLilypondString for Voicing {
    fn to_lilypond_string(&self) -> String {
        let inner: String = self.iter().map(|p| p.to_lilypond_string()).join(" ");
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
                    }
                    SingleEvent::Fretted(s) => {
                        let pitch = s.pitch.to_lilypond_string();
                        format!("{}{}\\{}", pitch, duration, s.string)
                    }
                    SingleEvent::FrettedMany(notes) => {
                        let inner: String = notes
                            .iter()
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
                let content = tuplet
                    .events
                    .iter()
                    .map(|event| event.to_lilypond_string())
                    .join(" ");
                // Notate the tuplet
                format!("\\tuplet {} {{ {} }}", ratio, content)
            }
        }
    }
}

#[cfg(test)]
mod pitch_to_lilypond_tests {
    use super::*;

    fn render(note: Note, octave: i8) -> String {
        Pitch::new(note, octave).to_lilypond_string()
    }

    #[test]
    fn natural_pitches() {
        assert_eq!(render(Note::C, 0), "c,,,");
        assert_eq!(render(Note::C, 3), "c");
        assert_eq!(render(Note::C, 4), "c'");
        assert_eq!(render(Note::C, 5), "c''");
        assert_eq!(render(Note::A, 4), "a'");
    }

    #[test]
    fn ces_marks_name_the_written_octave() {
        // C♭4 is MIDI 59 (sounds B3) and is written `ces'`, like C4 = `c'`.
        assert_eq!(Pitch::new(Note::Ces, 4).midi_note, 59);
        assert_eq!(render(Note::Ces, 4), "ces'");
        assert_eq!(render(Note::Ces, 5), "ces''");
        assert_eq!(render(Note::Ces, 3), "ces");
        assert_eq!(render(Note::Ces, 2), "ces,");
        assert_eq!(render(Note::Ces, 0), "ces,,,");
        // Spelling MIDI 71 as C♭ yields C♭5, written `ces''`.
        let respelled = Pitch::from_midi_spelled_as(71, &vec![Note::Ces]).unwrap();
        assert_eq!(respelled.to_lilypond_string(), "ces''");
    }

    #[test]
    fn bis_marks_name_the_written_octave() {
        // B♯3 is MIDI 60 (sounds C4) and is written `bis`, like B3 = `b`.
        assert_eq!(Pitch::new(Note::Bis, 3).midi_note, 60);
        assert_eq!(render(Note::Bis, 3), "bis");
        assert_eq!(render(Note::Bis, 4), "bis'");
        assert_eq!(render(Note::Bis, 2), "bis,");
        assert_eq!(render(Note::Bis, 0), "bis,,,");
        // Spelling MIDI 60 as B♯ yields B♯3, written `bis`.
        let respelled = Pitch::from_midi_spelled_as(60, &vec![Note::Bis]).unwrap();
        assert_eq!(respelled.to_lilypond_string(), "bis");
    }

    #[test]
    fn no_overflow_across_full_range() {
        // Every valid (Note, octave) must render without panicking.
        use crate::note::note::Note::*;
        let notes = [
            C, Cis, Cisis, Ces, Deses, D, Dis, Disis, Des, Eeses, E, Eis, Ees, Fes, F, Fis, Fisis,
            Geses, Ges, G, Gis, Gisis, Aeses, Aes, A, Ais, Aisis, Beses, Bes, B, Bis,
        ];
        for n in notes {
            for oct in -1i8..=9i8 {
                if let Ok(p) = Pitch::try_new(n, oct) {
                    let _ = p.to_lilypond_string();
                }
            }
        }
    }
}
