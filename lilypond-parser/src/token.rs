use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Bare identifier: `cis`, `treble`, `major`, `r`.
    Ident(String),
    /// Unsigned integer literal: `4`, `16`.
    Number(u32),
    /// `.` (duration dot).
    Dot,
    /// `'` (octave up).
    Apostrophe,
    /// `,` (octave down).
    Comma,
    /// `<`
    LAngle,
    /// `>`
    RAngle,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `/` (used in `\tuplet 3/2` and `\time 4/4`).
    Slash,
    /// Backslash command, stored without the leading `\`: `\clef` -> `"clef"`.
    Command(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
