//! Conversion from owned AST types to validated runtime types.
//!
//! Each `convert_<variant>` function validates an owned AST node and
//! produces a `Resolved<Variant>` struct whose fields are borrowed
//! `music` crate types (REQ-O6–O29). Shared helpers live in internal
//! `pitch` and `duration` sub-modules, plus the crate-level
//! `tuning_registry`.

pub(crate) mod duration;
pub(crate) mod pitch;

use crate::error::MusicRonError;

/// Enforces XOR identity constraint: exactly one of the named fields
/// must be present. Returns the name of the single present field, or
/// `AmbiguousIdentity` if zero or more than one are set.
pub(crate) fn xor_identity(
    fields: &[(&'static str, bool)],
    path: &str,
) -> Result<&'static str, MusicRonError> {
    let present: Vec<String> = fields
        .iter()
        .filter(|(_, set)| *set)
        .map(|(name, _)| (*name).to_string())
        .collect();
    if present.len() != 1 {
        return Err(MusicRonError::AmbiguousIdentity {
            path: path.into(),
            fields: present,
        });
    }
    // Safe: present.len() == 1 guaranteed by guard above, so exactly one (_, true) exists.
    Ok(fields.iter().find(|(_, set)| *set).map(|(name, _)| *name).unwrap_or(""))

}

pub mod snippet;
pub mod tab;
pub mod fretboard_shape;
pub mod pitch_circle;
pub mod chord_progression;
pub mod scale_diagram;
pub mod interval_matrix;

pub use snippet::{convert_snippet, ResolvedSnippet};
pub use tab::convert_tab;
pub use fretboard_shape::{convert_fretboard_shape, FretValue, ResolvedBarre, ResolvedFretboardShape};
pub use pitch_circle::{convert_pitch_circle, PitchCircleIdentity, ResolvedPitchCircle};
pub use chord_progression::{convert_chord_progression, ResolvedChordEntry, ResolvedChordProgression};
pub use scale_diagram::{convert_scale_diagram, ScaleDiagramIdentity, Orientation, ResolvedScaleDiagram};
pub use interval_matrix::{convert_interval_matrix, IntervalStyle, IntervalMatrixIdentity, ResolvedIntervalMatrix};
