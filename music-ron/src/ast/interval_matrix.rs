use music::note::note::Note;
use serde::{Deserialize, Serialize};
use super::common::OwnedMeta;

/// Interval vector/matrix visualization (REQ-O28).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedIntervalMatrix {
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
    /// Required style; unknown values routed to UnknownStyle at convert-time.
    pub style: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub bar_size: Option<f32>,
    #[serde(default)]
    pub cell_size: Option<f32>,
}
