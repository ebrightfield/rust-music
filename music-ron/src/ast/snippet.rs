use super::common::{OwnedEvent, OwnedMeta};
use crate::visitor::bounded_vec::deserialize_bounded_vec;
use serde::{Deserialize, Serialize};

/// A notated music snippet with clef and events (REQ-O9, O10, O17).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedSnippet {
    #[serde(default)]
    pub meta: Option<OwnedMeta>,
    #[serde(default)]
    pub version: Option<u32>,
    /// Clef as a string; unknown values route through UnknownClef at convert-time.
    pub clef: String,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub events: Vec<OwnedEvent>,
}
