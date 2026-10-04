//! Centralized error types for the music-engraver crate.
//!
//! [`EngraverError`] is the top-level error type returned by public APIs
//! like [`ScoreBuilder::try_render_svg`](crate::score::ScoreBuilder::try_render_svg).
//! Internal modules may use more specific error types (e.g. [`FontError`])
//! that convert into `EngraverError` via `From` impls.

use crate::font::FontError;

/// Top-level error type for the music-engraver crate.
///
/// Wraps domain-specific error variants so that the public API returns a
/// single error type. New variant families (layout validation, I/O, etc.)
/// can be added here without changing function signatures.
#[derive(Debug, thiserror::Error)]
pub enum EngraverError {
    /// A font operation failed (glyph lookup, outline extraction, parsing).
    #[error(transparent)]
    Font(#[from] FontError),

    /// A shared guitar timeline or physical realization was invalid.
    #[error(transparent)]
    Guitar(#[from] crate::score::guitar::GuitarScoreError),

    /// The score's bar structure cannot be engraved as written.
    #[error(transparent)]
    Structure(#[from] crate::score::ScoreStructureError),

    /// PNG rasterization failed (requires the `png` feature).
    #[cfg(feature = "png")]
    #[error(transparent)]
    Png(#[from] crate::render::png::PngError),

    /// Writing rendered output to disk failed (requires the `png` feature,
    /// used by the `save_png` convenience methods).
    #[cfg(feature = "png")]
    #[error("failed to write PNG to disk: {0}")]
    Io(#[from] std::io::Error),
}
