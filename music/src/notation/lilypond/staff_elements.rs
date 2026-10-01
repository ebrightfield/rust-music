use crate::notation::lilypond::templates::TEMPLATE_ENGINE;
use crate::notation::lilypond::ToLilypondString;
use crate::notation::rhythm::RhythmicNotatedEvent;
use itertools::Itertools;
use tera::Context;

impl<'a> ToLilypondString for Vec<LilypondVoiceElement<'a>> {
    fn to_lilypond_string(&self) -> String {
        let content = self.iter().map(|item| item.to_lilypond_string()).join(" ");
        let mut ctx = Context::new();
        ctx.insert("content", &content);
        (*TEMPLATE_ENGINE).render("voice", &ctx).unwrap()
    }
}

/// Abstraction over common elements (things that other engraving systems
/// should definitely have, like notes and rests),
/// with the addition of other elements that may be unique to Lilypond.
pub enum LilypondVoiceElement<'a> {
    /// Notes, chords (fretted or otherwise), and rests.
    Common(RhythmicNotatedEvent<'a>),
    // TODO Repeat block
    // TODO \break
    // TODO barline
    // TODO Replace this with definite types
    /// This enum is meant to be used inside of Voice contexts.
    /// Therefore, any `impl ToLilypondString` that is not valid inside
    /// of a Voice context will fail to compile.
    Other(Box<dyn ToLilypondString>),
}

impl<'a> Into<LilypondVoiceElement<'a>> for RhythmicNotatedEvent<'a> {
    fn into(self) -> LilypondVoiceElement<'a> {
        LilypondVoiceElement::Common(self)
    }
}

impl<'a> ToLilypondString for LilypondVoiceElement<'a> {
    fn to_lilypond_string(&self) -> String {
        match &self {
            LilypondVoiceElement::Common(rhythmic_notated_event) => {
                rhythmic_notated_event.to_lilypond_string()
            }
            LilypondVoiceElement::Other(ly) => ly.to_lilypond_string(),
        }
    }
}

impl<'a> LilypondVoiceElement<'a> {
    /// Returns a reference to the inner `RhythmicNotatedEvent` if this is a `Common` variant.
    /// Returns `None` for `Other` variants (clef changes, articulations, etc.).
    /// Used by `music::notation::rhythm::flatten::iter_events` (REQ-O16).
    pub fn as_rhythmic_event(&self) -> Option<&RhythmicNotatedEvent<'_>> {
        match self {
            LilypondVoiceElement::Common(e) => Some(e),
            LilypondVoiceElement::Other(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::rhythm::{
        duration::Duration, duration::DurationKind, RhythmicNotatedEvent,
    };
    use crate::note::pitch::Pitch;

    #[test]
    fn as_rhythmic_event_returns_some_for_common() {
        let pitch = Pitch::from_midi(60).unwrap();
        let event = RhythmicNotatedEvent::pitch(pitch, Duration::new(DurationKind::Qtr, 0));
        let elem = LilypondVoiceElement::Common(event);
        assert!(elem.as_rhythmic_event().is_some());
    }

    #[test]
    fn as_rhythmic_event_returns_none_for_other() {
        struct DummyLy;
        impl ToLilypondString for DummyLy {
            fn to_lilypond_string(&self) -> String {
                "dummy".to_string()
            }
        }
        let elem: LilypondVoiceElement = LilypondVoiceElement::Other(Box::new(DummyLy));
        assert!(elem.as_rhythmic_event().is_none());
    }
}
