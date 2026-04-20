use crate::token::Span;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{kind} at {}..{}", span.start, span.end)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseErrorKind {
    #[error("unexpected character {0:?}")]
    UnexpectedChar(char),
    #[error("unterminated block comment")]
    UnterminatedBlockComment,
    #[error("unexpected end of input; expected {0}")]
    UnexpectedEof(&'static str),
    #[error("expected {expected}, found {found}")]
    Expected {
        expected: &'static str,
        found: String,
    },
    #[error("unknown pitch name {0:?}")]
    UnknownPitch(String),
    #[error("invalid duration {0}")]
    InvalidDuration(u32),
    #[error("octave {0} out of supported range 0..=8")]
    OctaveOutOfRange(i32),
    #[error("no duration given and no previous duration to carry")]
    MissingDuration,
    #[error("unsupported command {0:?}")]
    UnsupportedCommand(String),
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}
