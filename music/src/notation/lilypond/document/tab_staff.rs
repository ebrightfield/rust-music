use tera::Context;
use crate::notation::lilypond::staff_elements::LilypondVoiceElement;
use crate::notation::lilypond::templates::{NO_AUTOMATIC_BAR_LINES, OMIT_BAR_NUMBER, OMIT_STRING_NUMBER, TEMPLATE_ENGINE};
use crate::notation::lilypond::ToLilypondString;

/// For engraving tablature. This is meant to be populated with
/// `SoundedNote`s, but Lilypond is technically capable of inferring fretboard
/// information when it's not there. The result is just usually not desirable.
///
/// Therefore, users of this struct are encouraged to make sure they design
/// their Lilypond documents with `Fretboard`s in mind.
pub struct LilypondTabStaff<'a> {
    modern_tab_clef: bool,
    show_bar_numbers: bool,
    show_string_numbers: bool,
    automatic_bar_lines: bool,
    voices: Vec<Vec<LilypondVoiceElement<'a>>>,
}

impl<'a> LilypondTabStaff<'a> {
    pub fn new() -> Self {
        Self {
            modern_tab_clef: true,
            show_bar_numbers: false,
            show_string_numbers: false,
            automatic_bar_lines: true,
            voices: vec![]
        }
    }

    pub fn add_voice(mut self, voice: Vec<LilypondVoiceElement<'a>>) -> Self {
        self.voices.push(voice);
        self
    }

    /// A clean, non-cursive, non-serif vertical "TAB" across the beginning of the system.
    pub fn use_modern_tab_clef(mut self, use_modern_tab: bool) -> Self {
        self.modern_tab_clef = use_modern_tab;
        self
    }

    /// Show or hide bar numbers above the staff's barlines.
    pub fn bar_numbers(mut self, show: bool) -> Self {
        self.show_bar_numbers = show;
        self
    }

    // TODO I'm not sure this flag is relevant for tab staff
    /// Show or hide string numbers.
    pub fn string_numbers(mut self, show: bool) -> Self {
        self.show_string_numbers = show;
        self
    }

    /// Toggle the automatic drawing of bar-lines. When this is off,
    /// you can manually engrave them at places you prefer.
    pub fn automatic_bars(mut self, draw_bar_lines: bool) -> Self {
        self.automatic_bar_lines = draw_bar_lines;
        self
    }
}

impl<'a> ToLilypondString for LilypondTabStaff<'a> {
    fn to_lilypond_string(&self) -> String {
        let mut ctx = Context::new();
        let mut statements = vec![];
        if self.modern_tab_clef {
            statements.push("\\clef moderntab");
        }
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
            .map(|voice| voice.to_lilypond_string().replace("Voice", "TabVoice"))
            .collect::<Vec<String>>();
        ctx.insert("voices", &voices);
        (*TEMPLATE_ENGINE).render("tab_staff", &ctx).unwrap()
    }
}
