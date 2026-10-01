use music::notation::rhythm::duration::DurationKind;
use music::note::note::Note;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::visitor::bounded_vec::deserialize_bounded_vec;
use crate::visitor::duration::parse_duration_shorthand;
use crate::visitor::pitch::parse_pitch_shorthand;

/// Optional metadata shared by all document variants (REQ-O9).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnedMeta {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Pitch with shorthand string or long-form struct.
/// Octave is `i32` so out-of-range values surface as `OctaveOutOfRange` at
/// convert-time rather than as serde deserialization errors (REQ-O12, O16).
#[derive(Debug, Clone)]
pub enum OwnedPitch {
    Shorthand(String),
    Long { note: Note, octave: i32 },
}

impl Serialize for OwnedPitch {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            OwnedPitch::Shorthand(s) => serializer.serialize_str(s),
            OwnedPitch::Long { note, octave } => {
                let mut st = serializer.serialize_struct("OwnedPitch", 2)?;
                st.serialize_field("note", note)?;
                st.serialize_field("octave", octave)?;
                st.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for OwnedPitch {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OwnedPitchVisitor;

        impl<'de> Visitor<'de> for OwnedPitchVisitor {
            type Value = OwnedPitch;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a pitch shorthand string like \"c4\" or a struct { note, octave }")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<OwnedPitch, E> {
                // REQ-O11: validate shorthand at parse time so RON position info is available.
                parse_pitch_shorthand(v)
                    .map(|_| OwnedPitch::Shorthand(v.to_owned()))
                    .map_err(|(input, _)| {
                        de::Error::custom(format!("invalid pitch shorthand: \"{input}\""))
                    })
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<OwnedPitch, A::Error> {
                let mut note: Option<Note> = None;
                let mut octave: Option<i32> = None;
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "note" => {
                            if note.is_some() {
                                return Err(de::Error::duplicate_field("note"));
                            }
                            note = Some(map.next_value()?);
                        }
                        "octave" => {
                            if octave.is_some() {
                                return Err(de::Error::duplicate_field("octave"));
                            }
                            octave = Some(map.next_value()?);
                        }
                        other => {
                            return Err(de::Error::unknown_field(other, &["note", "octave"]));
                        }
                    }
                }
                let note = note.ok_or_else(|| de::Error::missing_field("note"))?;
                let octave = octave.ok_or_else(|| de::Error::missing_field("octave"))?;
                Ok(OwnedPitch::Long { note, octave })
            }
        }

        deserializer.deserialize_any(OwnedPitchVisitor)
    }
}

/// Duration with shorthand string or long-form struct (REQ-O13, O14).
#[derive(Debug, Clone)]
pub enum OwnedDuration {
    Shorthand(String),
    Long { kind: DurationKind, dots: u8 },
}

impl Serialize for OwnedDuration {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            OwnedDuration::Shorthand(s) => serializer.serialize_str(s),
            OwnedDuration::Long { kind, dots } => {
                let mut st = serializer.serialize_struct("OwnedDuration", 2)?;
                st.serialize_field("kind", kind)?;
                st.serialize_field("dots", dots)?;
                st.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for OwnedDuration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OwnedDurationVisitor;

        impl<'de> Visitor<'de> for OwnedDurationVisitor {
            type Value = OwnedDuration;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(
                    "a duration shorthand string like \"4\" or \"8.\" or a struct { kind, dots }",
                )
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<OwnedDuration, E> {
                // REQ-O13: validate shorthand at parse time for early error reporting.
                parse_duration_shorthand(v)
                    .map(|_| OwnedDuration::Shorthand(v.to_owned()))
                    .map_err(|()| de::Error::custom(format!("invalid duration shorthand: \"{v}\"")))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<OwnedDuration, A::Error> {
                let mut kind: Option<DurationKind> = None;
                let mut dots: Option<u8> = None;
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "kind" => {
                            if kind.is_some() {
                                return Err(de::Error::duplicate_field("kind"));
                            }
                            kind = Some(map.next_value()?);
                        }
                        "dots" => {
                            if dots.is_some() {
                                return Err(de::Error::duplicate_field("dots"));
                            }
                            dots = Some(map.next_value()?);
                        }
                        other => {
                            return Err(de::Error::unknown_field(other, &["kind", "dots"]));
                        }
                    }
                }
                let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
                let dots = dots.ok_or_else(|| de::Error::missing_field("dots"))?;
                Ok(OwnedDuration::Long { kind, dots })
            }
        }

        deserializer.deserialize_any(OwnedDurationVisitor)
    }
}

/// Struct variants for snippet events (REQ-O17).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnedEvent {
    Note {
        pitch: OwnedPitch,
        duration: OwnedDuration,
    },
    Chord {
        #[serde(deserialize_with = "deserialize_bounded_vec")]
        pitches: Vec<OwnedPitch>,
        duration: OwnedDuration,
    },
    Rest {
        duration: OwnedDuration,
    },
    Tie,
    Tuplet {
        numerator: u8,
        denominator: u8,
        base: DurationKind,
        #[serde(deserialize_with = "deserialize_bounded_vec")]
        children: Vec<OwnedEvent>,
    },
}

/// Named or inline tuning (REQ-O24).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OwnedTuning {
    Named(String),
    Inline {
        #[serde(deserialize_with = "deserialize_bounded_vec")]
        pitches: Vec<OwnedPitch>,
    },
}

/// Barre chord descriptor (REQ-O26).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedBarre {
    pub fret: u8,
    pub from_string: u8,
    pub to_string: u8,
}

/// Chord entry in a progression (REQ-O30).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedChordEntry {
    pub symbol: String,
    #[serde(default)]
    pub duration: Option<OwnedDuration>,
}

/// Meter as beats + duration unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedMeter {
    pub beats: u8,
    pub unit: DurationKind,
}

/// Key with tonic note + mode name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedKey {
    pub tonic: Note,
    pub mode: String,
}

/// Re-export so variant modules can use a consistent name.
pub use music::fretboard::StringConvention as OwnedStringConvention;

#[cfg(test)]
mod tests {
    use super::*;

    // --- OwnedPitch custom deser tests ---

    #[test]
    fn pitch_shorthand_valid_ron() {
        let p: OwnedPitch = ron::from_str("\"c4\"").unwrap();
        assert!(matches!(p, OwnedPitch::Shorthand(s) if s == "c4"));
    }

    #[test]
    fn pitch_shorthand_sharp_ron() {
        let p: OwnedPitch = ron::from_str("\"cs4\"").unwrap();
        assert!(matches!(p, OwnedPitch::Shorthand(s) if s == "cs4"));
    }

    #[test]
    fn pitch_shorthand_invalid_rejected() {
        let err = ron::from_str::<OwnedPitch>("\"z9\"");
        assert!(err.is_err());
    }

    #[test]
    fn pitch_long_form_ron() {
        let p: OwnedPitch = ron::from_str("(note: Cis, octave: 5)").unwrap();
        assert!(matches!(
            p,
            OwnedPitch::Long {
                note: Note::Cis,
                octave: 5
            }
        ));
    }

    #[test]
    fn pitch_long_form_negative_octave() {
        let p: OwnedPitch = ron::from_str("(note: C, octave: -1)").unwrap();
        assert!(matches!(
            p,
            OwnedPitch::Long {
                note: Note::C,
                octave: -1
            }
        ));
    }

    // --- OwnedDuration custom deser tests ---

    #[test]
    fn duration_shorthand_quarter() {
        let d: OwnedDuration = ron::from_str("\"4\"").unwrap();
        assert!(matches!(d, OwnedDuration::Shorthand(s) if s == "4"));
    }

    #[test]
    fn duration_shorthand_dotted_eighth() {
        let d: OwnedDuration = ron::from_str("\"8.\"").unwrap();
        assert!(matches!(d, OwnedDuration::Shorthand(s) if s == "8."));
    }

    #[test]
    fn duration_shorthand_invalid_rejected() {
        let err = ron::from_str::<OwnedDuration>("\"7\"");
        assert!(err.is_err());
    }

    #[test]
    fn duration_long_form_ron() {
        let d: OwnedDuration = ron::from_str("(kind: Qtr, dots: 1)").unwrap();
        assert!(matches!(
            d,
            OwnedDuration::Long {
                kind: DurationKind::Qtr,
                dots: 1
            }
        ));
    }
}
