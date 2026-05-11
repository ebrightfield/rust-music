use serde::{Deserialize, Serialize};
use super::common::{OwnedChordEntry, OwnedKey, OwnedMeta, OwnedMeter};
use crate::visitor::bounded_vec::deserialize_bounded_vec;

/// Ordered chord symbols with optional meter and key (REQ-O30).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedChordProgression {
    #[serde(default)]
    pub meta: Option<OwnedMeta>,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub chords: Vec<OwnedChordEntry>,
    #[serde(default)]
    pub meter: Option<OwnedMeter>,
    #[serde(default)]
    pub key: Option<OwnedKey>,
}
