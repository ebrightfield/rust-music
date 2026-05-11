//! `music-ron` — a RON (Rusty Object Notation) parser for music documents.
//!
//! Parses `.ron` files into a tagged [`Document`] enum (REQ-O4/O5),
//! validates owned AST nodes, and converts them into borrowed runtime
//! types from the [`music`] crate via [`convert`] functions (REQ-O6–O29).
//!
//! Entry points: [`parse`], [`parse_path`], [`parse_as`].

pub mod ast;
pub mod convert;
pub mod error;
pub mod limits;
mod parse;
pub mod tuning_registry;
pub(crate) mod visitor;

pub use ast::{Document, DocumentKind};
pub use error::MusicRonError;

use std::path::Path;

/// Parse a RON string into the tagged `Document` enum.
/// REQ-O4 (R1)
pub fn parse(input: &str) -> Result<Document, MusicRonError> {
    parse::parse(input)
}

/// Parse a RON file; attaches source path to any error.
/// REQ-O5 (R2), G8 (BOM strip handled centrally in `parse_internal`)
pub fn parse_path(path: &Path) -> Result<Document, MusicRonError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|source| MusicRonError::Io { source_path: path.to_path_buf(), source })?;
    parse::parse_internal(&raw, Some(path.to_path_buf()))
}

/// Parse as a specific variant. The caller asserts the expected `KIND`.
/// REQ-O6 (R3)
pub fn parse_as<T: DocumentKind>(input: &str) -> Result<T, MusicRonError> {
    parse::parse_as(input)
}
