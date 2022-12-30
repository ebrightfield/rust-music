use itertools::Itertools;
use tera::Context;
use crate::notation::lilypond::templates::TEMPLATE_ENGINE;
use crate::notation::lilypond::ToLilypondString;
use crate::notation::rhythm::RhythmicNotatedEvent;

// pub struct LilypondVoice(Vec<LilypondVoiceElement>);
//
// impl ToLilypondString for LilypondVoice {
//     fn to_lilypond_string(&self) -> String {
//         self.0.iter()
//             .map(|item| item.to_lilypond_string())
//             .join(" ")
//     }
// }

impl ToLilypondString for Vec<LilypondVoiceElement> {
    fn to_lilypond_string(&self) -> String {
        let content = self.iter()
            .map(|item| item.to_lilypond_string())
            .join(" ");
        let mut ctx = Context::new();
        ctx.insert("content", &content);
        (*TEMPLATE_ENGINE).render("voice", &ctx).unwrap()
    }
}

/// Abstraction over common elements (things that other engraving systems
/// should definitely have, like notes and rests),
/// with the addition of other elements that may be unique to Lilypond.
pub enum LilypondVoiceElement {
    /// Note, chord, rest.
    Common(RhythmicNotatedEvent),
    // TODO Replace this with definite types
    /// This enum is meant to be used inside of Voice contexts.
    /// Therefore, any `impl ToLilypondString` that is not valid inside
    /// of a Voice context will fail to compile.
    Other(Box<dyn ToLilypondString>),
}

impl Into<LilypondVoiceElement> for RhythmicNotatedEvent {
    fn into(self) -> LilypondVoiceElement {
        LilypondVoiceElement::Common(self)
    }
}

impl ToLilypondString for LilypondVoiceElement {
    fn to_lilypond_string(&self) -> String {
        match &self {
            LilypondVoiceElement::Common(rhythmic_notated_event) => {
                rhythmic_notated_event.to_lilypond_string()
            },
            LilypondVoiceElement::Other(ly) => ly.to_lilypond_string()
        }
    }
}