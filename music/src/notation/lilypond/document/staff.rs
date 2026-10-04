use crate::notation::clef::Clef;
use crate::notation::lilypond::staff_elements::LilypondVoiceElement;
use crate::notation::lilypond::templates::{
    NO_AUTOMATIC_BAR_LINES, OMIT_BAR_NUMBER, OMIT_CLEF, OMIT_STRING_NUMBER, OMIT_TIME_SIGNATURE,
    TEMPLATE_ENGINE,
};
use crate::notation::lilypond::ToLilypondString;
use crate::notation::rhythm::meter::Meter;
use tera::Context;

pub struct LilypondStaff<'a> {
    clef: Option<Clef>,
    time_signature: Option<Meter>,
    show_bar_numbers: bool,
    show_string_numbers: bool,
    automatic_bar_lines: bool,
    /// Each voice is simply a `Vec<LilypondVoiceElement>`
    voices: Vec<Vec<LilypondVoiceElement<'a>>>,
}

impl<'a> LilypondStaff<'a> {
    pub fn new() -> Self {
        Self {
            clef: None,
            time_signature: None,
            show_bar_numbers: false,
            show_string_numbers: false,
            automatic_bar_lines: true,
            voices: vec![],
        }
    }

    pub fn add_voice(mut self, voice: Vec<LilypondVoiceElement<'a>>) -> Self {
        self.voices.push(voice);
        self
    }

    pub fn clef(mut self, clef: Option<Clef>) -> Self {
        self.clef = clef;
        self
    }

    pub fn meter(mut self, time_signature: Option<Meter>) -> Self {
        self.time_signature = time_signature;
        self
    }

    pub fn bar_numbers(mut self, show: bool) -> Self {
        self.show_bar_numbers = show;
        self
    }

    pub fn string_numbers(mut self, show: bool) -> Self {
        self.show_string_numbers = show;
        self
    }

    pub fn automatic_bars(mut self, draw_bar_lines: bool) -> Self {
        self.automatic_bar_lines = draw_bar_lines;
        self
    }

    /// Returns all voices in this staff, each as a slice of `LilypondVoiceElement`.
    /// Used by `music::notation::rhythm::flatten::iter_events` (REQ-O16).
    pub fn voices(&self) -> &[Vec<LilypondVoiceElement<'_>>] {
        &self.voices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::lilypond::staff_elements::LilypondVoiceElement;
    use crate::notation::rhythm::{
        duration::Duration, duration::DurationKind, RhythmicNotatedEvent,
    };
    use crate::note::pitch::Pitch;

    #[test]
    fn voices_returns_slice() {
        let pitch = Pitch::from_midi(60).unwrap();
        let event = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Qtr, 0));
        let voice = vec![LilypondVoiceElement::Common(event)];
        let staff = LilypondStaff::new().add_voice(voice);
        assert_eq!(staff.voices().len(), 1);
    }

    #[test]
    fn c_clefs_emit_lilypond_clef_commands() {
        use crate::notation::clef::Clef;
        for (clef, expected) in [(Clef::Alto, "\\clef alto"), (Clef::Tenor, "\\clef tenor")] {
            let pitch = Pitch::from_midi(60).unwrap();
            let event = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Qtr, 0));
            let staff = LilypondStaff::new()
                .clef(Some(clef))
                .add_voice(vec![LilypondVoiceElement::Common(event)]);
            let out = staff.to_lilypond_string();
            assert!(out.contains(expected), "{clef:?}: {out}");
        }
    }
}

impl<'a> ToLilypondString for LilypondStaff<'a> {
    fn to_lilypond_string(&self) -> String {
        let mut ctx = Context::new();
        let mut statements = vec![];
        let clef = self.clef.as_ref().map_or(OMIT_CLEF.to_string(), |clef| {
            format!("\\clef {}", clef.to_lilypond_string())
        });
        statements.push(clef.as_str());
        let time_sig = self
            .time_signature
            .as_ref()
            .map_or(OMIT_TIME_SIGNATURE.to_string(), |meter| {
                format!("\\time {}", meter.to_lilypond_string())
            });
        statements.push(time_sig.as_str());
        if !self.show_bar_numbers {
            statements.push(OMIT_BAR_NUMBER)
        }
        if !self.show_string_numbers {
            statements.push(OMIT_STRING_NUMBER)
        }
        if !self.automatic_bar_lines {
            statements.push(NO_AUTOMATIC_BAR_LINES)
        }
        ctx.insert("statements", &statements);
        let voices = self
            .voices
            .iter()
            .map(|voice| voice.to_lilypond_string())
            .collect::<Vec<String>>();
        ctx.insert("voices", &voices);
        (*TEMPLATE_ENGINE).render("staff", &ctx).unwrap()
    }
}
