//! Error types for RON parsing and conversion.
//!
//! [`MusicRonError`] is the single error enum surfaced by all public entry
//! points ([`parse`](crate::parse), [`parse_path`](crate::parse_path),
//! [`parse_as`](crate::parse_as)) and `convert_*` functions (REQ-O1).
//! Each variant carries contextual fields (`path`, `input`) so callers
//! can produce actionable diagnostics.

use std::fmt;

/// All user-visible errors from `music-ron` parsing and conversion.
#[derive(Debug)]
pub enum MusicRonError {
    SyntaxError {
        line: usize,
        col: usize,
        message: String,
        source_path: Option<String>,
        snippet: String,
    },
    Io {
        source_path: std::path::PathBuf,
        source: std::io::Error,
    },
    OctaveOutOfRange {
        got: i32,
        path: String,
    },
    InvalidTuplet {
        path: String,
        reason: String,
    },
    InvalidTie {
        path: String,
        reason: String,
    },
    UnknownTuning {
        name: String,
        path: String,
    },
    UnknownChordSymbol {
        input: String,
        inner: String,
        path: String,
    },
    UnknownClef {
        name: String,
        path: String,
    },
    UnknownScale {
        name: String,
        path: String,
    },
    InvalidDuration {
        input: String,
        path: String,
    },
    InvalidPitch {
        input: String,
        path: String,
    },
    UnsupportedVersion {
        got: u32,
        max_supported: u32,
        source_path: Option<String>,
    },
    AmbiguousIdentity {
        path: String,
        fields: Vec<String>,
    },
    UnknownStyle {
        got: String,
        path: String,
    },
}

impl std::error::Error for MusicRonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MusicRonError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for MusicRonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicRonError::SyntaxError { line, col, message, source_path, snippet } => {
                if let Some(p) = source_path {
                    write!(f, "{}:{}:{}: {}", p, line, col, message)?;
                } else {
                    write!(f, "{}:{}: {}", line, col, message)?;
                }
                if !snippet.is_empty() {
                    write!(f, "\n  | {}", snippet)?;
                    write!(f, "\n  | {:>width$}", "^", width = *col.max(&1))?;
                }
                Ok(())
            }
            MusicRonError::Io { source_path, source } => {
                write!(f, "I/O error in {}: {}", source_path.display(), source)
            }
            MusicRonError::OctaveOutOfRange { got, path } => {
                write!(f, "octave out of range: {} (at {})", got, path)
            }
            MusicRonError::InvalidTuplet { path, reason } => {
                write!(f, "invalid tuplet at {}: {}", path, reason)
            }
            MusicRonError::InvalidTie { path, reason } => {
                write!(f, "invalid tie at {}: {}", path, reason)
            }
            MusicRonError::UnknownTuning { name, path } => {
                write!(f, "unknown tuning {:?} at {}", name, path)
            }
            MusicRonError::UnknownChordSymbol { input, inner, path } => {
                write!(f, "unknown chord symbol {:?} at {}: {}", input, path, inner)
            }
            MusicRonError::UnknownClef { name, path } => {
                write!(f, "unknown clef {:?} at {}", name, path)
            }
            MusicRonError::UnknownScale { name, path } => {
                write!(f, "unknown scale {:?} at {}", name, path)
            }
            MusicRonError::InvalidDuration { input, path } => {
                write!(f, "invalid duration {:?} at {}", input, path)
            }
            MusicRonError::InvalidPitch { input, path } => {
                write!(f, "invalid pitch {:?} at {}", input, path)
            }
            MusicRonError::UnsupportedVersion { got, max_supported, source_path } => {
                if let Some(p) = source_path {
                    write!(f, "unsupported version {} (max: {}) in {}", got, max_supported, p)
                } else {
                    write!(f, "unsupported version {} (max: {})", got, max_supported)
                }
            }
            MusicRonError::AmbiguousIdentity { path, fields } => {
                write!(f, "ambiguous identity at {}: conflicting fields {:?}", path, fields)
            }
            MusicRonError::UnknownStyle { got, path } => {
                write!(f, "unknown style {:?} at {}", got, path)
            }
        }
    }
}

