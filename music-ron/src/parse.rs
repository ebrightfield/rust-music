//! RON deserialization with preflight validation.
//!
//! Enforces input-size limits ([`MAX_INPUT_BYTES`](crate::limits::MAX_INPUT_BYTES)),
//! recursion depth ([`MAX_NESTING_DEPTH`](crate::limits::MAX_NESTING_DEPTH)),
//! version guard (REQ-O5), and BOM stripping (REQ-G8) before delegating
//! to `ron::Options` for the actual parse.

use std::path::PathBuf;

use crate::ast::{Document, DocumentKind};
use crate::error::MusicRonError;
use crate::limits::{MAX_INPUT_BYTES, MAX_NESTING_DEPTH};

pub(crate) const MAX_SUPPORTED_VERSION: u32 = 1;

pub fn parse(input: &str) -> Result<Document, MusicRonError> {
    parse_internal(input, None)
}

pub(crate) fn parse_internal(
    input: &str,
    source_path: Option<PathBuf>,
) -> Result<Document, MusicRonError> {
    // REQ-G8: strip UTF-8 BOM if present (idempotent no-op when absent).
    // Applied centrally so every entry point (parse/parse_as/parse_path)
    // handles BOM-prefixed input uniformly.
    let input = input.strip_prefix('\u{FEFF}').unwrap_or(input);

    // REQ-O40: byte-length preflight
    if input.len() > MAX_INPUT_BYTES {
        return Err(MusicRonError::SyntaxError {
            line: 0,
            col: 0,
            message: format!(
                "input exceeds max length {} bytes (got {})",
                MAX_INPUT_BYTES,
                input.len()
            ),
            source_path: source_path.map(|p| p.display().to_string()),
            snippet: String::new(),
        });
    }

    // REQ-O40: recursion-limit via ron options
    let opts = ron::Options::default().with_recursion_limit(MAX_NESTING_DEPTH);
    let doc: Document = opts
        .from_str(input)
        .map_err(|e| ron_err_to_music_ron_err(e, input, source_path.clone()))?;

    // REQ-O10: version guard
    validate_version(&doc, source_path)?;

    Ok(doc)
}

fn validate_version(doc: &Document, source_path: Option<PathBuf>) -> Result<(), MusicRonError> {
    let v = match doc {
        Document::Snippet(d) => d.version,
        Document::Tab(d) => d.version,
        Document::FretboardShape(d) => d.version,
        Document::PitchCircle(d) => d.version,
        Document::ChordProgression(d) => d.version,
        Document::ScaleDiagram(d) => d.version,
        Document::IntervalMatrix(d) => d.version,
    };
    match v {
        None | Some(1) => Ok(()),
        Some(got) => Err(MusicRonError::UnsupportedVersion {
            got,
            max_supported: MAX_SUPPORTED_VERSION,
            source_path: source_path.map(|p| p.display().to_string()),
        }),
    }
}

/// Parse as a specific variant `T`. Returns `SyntaxError` if the document's
/// `kind` tag does not match `T::KIND`.
pub fn parse_as<T: DocumentKind>(input: &str) -> Result<T, MusicRonError> {
    let doc = parse_internal(input, None)?;
    let actual = doc.kind_str();
    T::from_document(doc).ok_or_else(|| MusicRonError::SyntaxError {
        line: 0,
        col: 0,
        message: format!(
            "kind mismatch: expected \"{}\", got \"{}\"",
            T::KIND,
            actual,
        ),
        source_path: None,
        snippet: String::new(),
    })
}

fn ron_err_to_music_ron_err(
    e: ron::error::SpannedError,
    input: &str,
    source_path: Option<PathBuf>,
) -> MusicRonError {
    let line = e.span.start.line;
    let col = e.span.start.col;
    let snippet = input
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .to_string();
    MusicRonError::SyntaxError {
        line,
        col,
        message: e.code.to_string(),
        source_path: source_path.map(|p| p.display().to_string()),
        snippet,
    }
}
