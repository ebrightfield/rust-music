use tera::Context;
use crate::notation::clef::Clef;
use crate::notation::lilypond::staff_elements::LilypondVoiceElement;
use crate::notation::lilypond::ToLilypondString;
use crate::notation::lilypond::templates::{NO_AUTOMATIC_BAR_LINES, OMIT_BAR_NUMBER, OMIT_CLEF, OMIT_STRING_NUMBER, OMIT_TIME_SIGNATURE, TEMPLATE_ENGINE};
use crate::notation::rhythm::Meter;

pub struct LilypondStaff {
    clef: Option<Clef>,
    time_signature: Option<Meter>,
    show_bar_numbers: bool,
    show_string_numbers: bool,
    automatic_bar_lines: bool,
    voices: Vec<Vec<LilypondVoiceElement>>,
}

impl LilypondStaff {
    pub fn new() -> Self {
        Self {
            clef: None,
            time_signature: None,
            show_bar_numbers: false,
            show_string_numbers: false,
            automatic_bar_lines: true,
            voices: vec![]
        }
    }

    pub fn add_voice(mut self, voice: Vec<LilypondVoiceElement>) -> Self {
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
}

impl ToLilypondString for LilypondStaff {
    fn to_lilypond_string(&self) -> String {
        let mut ctx = Context::new();
        let mut statements = vec![];
        let clef = self.clef
            .as_ref()
            .map_or(
                OMIT_CLEF.to_string(),
                |clef| format!("\\clef {}", clef.to_lilypond_string()),
            );
        statements.push(clef.as_str());
        let time_sig = self.time_signature
            .as_ref()
            .map_or(
                OMIT_TIME_SIGNATURE.to_string(),
                |meter| format!("\\time {}", meter.to_lilypond_string()),
            );
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
        let voices = self.voices.iter()
            .map(|voice| voice.to_lilypond_string())
            .collect::<Vec<String>>();
        ctx.insert("voices", &voices);
        (*TEMPLATE_ENGINE).render("staff", &ctx).unwrap()
    }
}
