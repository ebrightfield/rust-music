//! Minimal parser for a useful subset of LilyPond source.
//!
//! Accepts absolute-mode pitches (no `\relative`), durations with dots,
//! sticky duration carry, rests, chords `<c e g>`, and brace-delimited
//! voice blocks. Produces events that correspond to the types used by
//! `music::notation::lilypond` on the output side, so output -> parse
//! round-trips cleanly.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

pub use ast::{Event, Item};
pub use error::{ParseError, ParseErrorKind};
pub use parser::parse;
