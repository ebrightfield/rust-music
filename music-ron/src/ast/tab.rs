use serde::{Deserialize, Serialize};
use super::common::{OwnedDuration, OwnedMeta, OwnedStringConvention, OwnedTuning};
use crate::visitor::bounded_vec::deserialize_bounded_vec;

/// A single tablature event on a specific string/fret (REQ-O22).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedTabEvent {
    pub string: u8,
    pub fret: u8,
    pub duration: OwnedDuration,
}

/// Tablature document with tuning and string events (REQ-O22).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedTab {
    #[serde(default)]
    pub meta: Option<OwnedMeta>,
    #[serde(default)]
    pub version: Option<u32>,
    pub tuning: OwnedTuning,
    /// Defaults to OneIndexedFromHigh; routing is at convert-time (REQ-O25).
    #[serde(default)]
    pub string_convention: Option<OwnedStringConvention>,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub events: Vec<OwnedTabEvent>,
}
