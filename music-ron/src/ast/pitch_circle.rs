use super::common::OwnedMeta;
use music::note::note::Note;
use serde::{Deserialize, Serialize};

/// Pitch-class circle diagram with 3-way identity XOR (REQ-O27).
/// The XOR guard (exactly one of chord/pcs/scale) is enforced at convert-time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedPitchCircle {
    #[serde(default)]
    pub meta: Option<OwnedMeta>,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub chord: Option<String>,
    #[serde(default)]
    pub pcs: Option<Vec<u8>>,
    #[serde(default)]
    pub scale: Option<String>,
    #[serde(default)]
    pub root: Option<Note>,
    #[serde(default)]
    pub theme: Option<String>,
}
