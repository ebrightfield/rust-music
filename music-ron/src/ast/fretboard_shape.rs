use super::common::{OwnedBarre, OwnedMeta, OwnedStringConvention, OwnedTuning};
use crate::visitor::bounded_vec::deserialize_bounded_vec;
use serde::{Deserialize, Serialize};

/// Per-string fret value: `"x"` (muted), `0` (open), or fingered `1..=N` (REQ-O26).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OwnedFretValue {
    Muted(String),
    Fret(u8),
}

/// Fretboard chord shape with per-string frets and optional barre (REQ-O26).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedFretboardShape {
    #[serde(default)]
    pub meta: Option<OwnedMeta>,
    #[serde(default)]
    pub version: Option<u32>,
    pub tuning: OwnedTuning,
    #[serde(default)]
    pub string_convention: Option<OwnedStringConvention>,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub frets: Vec<OwnedFretValue>,
    #[serde(default)]
    pub fingers: Option<Vec<Option<u8>>>,
    #[serde(default)]
    pub barre: Option<OwnedBarre>,
}
