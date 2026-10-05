use super::common::{OwnedChordEntry, OwnedEvent, OwnedKey, OwnedMeta, OwnedMeter};
use crate::visitor::bounded_vec::deserialize_bounded_vec;
use serde::{Deserialize, Serialize};

/// A provenance-rich, measure-addressable complete score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedScore {
    #[serde(default)]
    pub meta: Option<OwnedMeta>,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_bounded_vec")]
    pub sources: Vec<OwnedScoreSource>,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub parts: Vec<OwnedPartDefinition>,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub measures: Vec<OwnedMeasure>,
}

/// Immutable identity and location of evidence used for a transcription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedScoreSource {
    pub id: String,
    pub path: String,
    pub sha256: String,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub work_id: Option<String>,
    #[serde(default)]
    pub edition: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Stable score-wide definition of a part or staff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedPartDefinition {
    pub id: String,
    pub name: String,
    pub clef: String,
}

/// One written measure. Meter and key are changes effective at its start.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedMeasure {
    pub number: String,
    #[serde(default)]
    pub meter: Option<OwnedMeter>,
    #[serde(default)]
    pub key: Option<OwnedKey>,
    #[serde(default, deserialize_with = "deserialize_bounded_vec")]
    pub parts: Vec<OwnedMeasurePart>,
    #[serde(default, deserialize_with = "deserialize_bounded_vec")]
    pub harmony: Vec<OwnedChordEntry>,
    #[serde(default, deserialize_with = "deserialize_bounded_vec")]
    pub readings: Vec<OwnedReading>,
}

/// Material belonging to one score-defined part in a measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedMeasurePart {
    pub part: String,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub voices: Vec<OwnedVoice>,
}

/// A sequential notated voice within one measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedVoice {
    pub id: String,
    #[serde(deserialize_with = "deserialize_bounded_vec")]
    pub events: Vec<OwnedEvent>,
}

/// Explicitly records uncertainty without inventing a score fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedReading {
    #[serde(default)]
    pub beat: Option<String>,
    pub subject: String,
    pub status: String,
    pub confidence: String,
    pub note: String,
}
