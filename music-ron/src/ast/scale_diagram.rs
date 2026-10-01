use super::common::{OwnedMeta, OwnedStringConvention, OwnedTuning};
use music::note::note::Note;
use serde::{Deserialize, Serialize};

/// Fretboard orientation for scale diagrams (REQ-O29).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnedOrientation {
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "vertical")]
    Vertical,
}

/// Scale overlay on a fretboard region (REQ-O29).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedScaleDiagram {
    #[serde(default)]
    pub meta: Option<OwnedMeta>,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub scale: Option<String>,
    #[serde(default)]
    pub pcs: Option<Vec<u8>>,
    #[serde(default)]
    pub root: Option<Note>,
    pub tuning: OwnedTuning,
    #[serde(default)]
    pub string_convention: Option<OwnedStringConvention>,
    pub start_fret: u8,
    pub num_frets: u8,
    pub orientation: OwnedOrientation,
    #[serde(default)]
    pub show_degrees: bool,
    #[serde(default)]
    pub highlight_root: bool,
    #[serde(default)]
    pub theme: Option<String>,
}
