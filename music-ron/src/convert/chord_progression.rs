use music::note::note::Note;
use music::note_collections::chord_name::parsing::parse_chord_name;
use music::note_collections::pc_set::PcShape;
use music::notation::rhythm::duration::DurationKind;

use crate::ast::OwnedChordProgression;
use crate::ast::common::{OwnedDuration, OwnedKey, OwnedMeter};
use crate::error::MusicRonError;
use crate::visitor::duration::parse_duration_shorthand;

/// A validated chord entry: root + shape + optional resolved duration.
#[derive(Debug)]
pub struct ResolvedChordEntry {
    pub root: Note,
    pub shape: PcShape,
    pub symbol: String,
    pub duration: Option<(DurationKind, u8)>,
}

/// A validated chord progression with all symbols parsed.
#[derive(Debug)]
pub struct ResolvedChordProgression {
    pub chords: Vec<ResolvedChordEntry>,
    pub meter: Option<OwnedMeter>,
    pub key: Option<OwnedKey>,
}

/// Resolve an OwnedDuration to (DurationKind, dots).
fn resolve_duration(d: &OwnedDuration, path: &str) -> Result<(DurationKind, u8), MusicRonError> {
    match d {
        OwnedDuration::Shorthand(s) => parse_duration_shorthand(s).map_err(|()| {
            MusicRonError::InvalidDuration {
                input: s.clone(),
                path: path.to_string(),
            }
        }),
        OwnedDuration::Long { kind, dots } => Ok((*kind, *dots)),
    }
}

/// Convert an owned chord progression AST node into validated runtime types.
///
/// Each chord symbol is delegated to `parse_chord_name` (REQ-O20);
/// unknown symbols surface as `UnknownChordSymbol` (REQ-O21).
pub fn convert_chord_progression(
    owned: &OwnedChordProgression,
) -> Result<ResolvedChordProgression, MusicRonError> {
    let mut chords = Vec::with_capacity(owned.chords.len());

    for (i, entry) in owned.chords.iter().enumerate() {
        let (root, shape) = parse_chord_name(&entry.symbol).map_err(|e| {
            MusicRonError::UnknownChordSymbol {
                input: entry.symbol.clone(),
                inner: e.to_string(),
                path: format!("chords[{}].symbol", i),
            }
        })?;

        let duration = match &entry.duration {
            Some(d) => Some(resolve_duration(d, &format!("chords[{}].duration", i))?),
            None => None,
        };

        chords.push(ResolvedChordEntry {
            root,
            shape,
            symbol: entry.symbol.clone(),
            duration,
        });
    }

    Ok(ResolvedChordProgression {
        chords,
        meter: owned.meter.clone(),
        key: owned.key.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::common::OwnedChordEntry;

    fn make_progression(symbols: &[&str]) -> OwnedChordProgression {
        OwnedChordProgression {
            meta: None,
            version: None,
            chords: symbols
                .iter()
                .map(|s| OwnedChordEntry {
                    symbol: s.to_string(),
                    duration: None,
                })
                .collect(),
            meter: None,
            key: None,
        }
    }

    #[test]
    fn basic_progression_resolves() {
        let owned = make_progression(&["CMaj7", "Am7", "Dm7", "G7"]);
        let resolved = convert_chord_progression(&owned).unwrap();
        assert_eq!(resolved.chords.len(), 4);
        assert_eq!(resolved.chords[0].root, Note::C);
        assert_eq!(resolved.chords[1].root, Note::A);
        assert_eq!(resolved.chords[2].root, Note::D);
        assert_eq!(resolved.chords[3].root, Note::G);
    }

    #[test]
    fn unknown_chord_symbol_errors() {
        let owned = make_progression(&["CMaj7", "Xfoo"]);
        let err = convert_chord_progression(&owned).unwrap_err();
        match err {
            MusicRonError::UnknownChordSymbol { input, path, .. } => {
                assert_eq!(input, "Xfoo");
                assert_eq!(path, "chords[1].symbol");
            }
            other => panic!("expected UnknownChordSymbol, got: {other}"),
        }
    }

    #[test]
    fn chord_with_duration_shorthand() {
        let owned = OwnedChordProgression {
            meta: None,
            version: None,
            chords: vec![OwnedChordEntry {
                symbol: "Am".to_string(),
                duration: Some(OwnedDuration::Shorthand("4.".to_string())),
            }],
            meter: None,
            key: None,
        };
        let resolved = convert_chord_progression(&owned).unwrap();
        let (kind, dots) = resolved.chords[0].duration.unwrap();
        assert_eq!(kind, DurationKind::Qtr);
        assert_eq!(dots, 1);
    }

    #[test]
    fn chord_with_duration_long_form() {
        let owned = OwnedChordProgression {
            meta: None,
            version: None,
            chords: vec![OwnedChordEntry {
                symbol: "G".to_string(),
                duration: Some(OwnedDuration::Long {
                    kind: DurationKind::Half,
                    dots: 0,
                }),
            }],
            meter: None,
            key: None,
        };
        let resolved = convert_chord_progression(&owned).unwrap();
        let (kind, dots) = resolved.chords[0].duration.unwrap();
        assert_eq!(kind, DurationKind::Half);
        assert_eq!(dots, 0);
    }

    #[test]
    fn meter_and_key_pass_through() {
        let owned = OwnedChordProgression {
            meta: None,
            version: None,
            chords: vec![OwnedChordEntry {
                symbol: "C".to_string(),
                duration: None,
            }],
            meter: Some(OwnedMeter {
                beats: 4,
                unit: DurationKind::Qtr,
            }),
            key: Some(OwnedKey {
                tonic: Note::C,
                mode: "major".to_string(),
            }),
        };
        let resolved = convert_chord_progression(&owned).unwrap();
        assert!(resolved.meter.is_some());
        assert_eq!(resolved.meter.as_ref().unwrap().beats, 4);
        assert!(resolved.key.is_some());
        assert_eq!(resolved.key.as_ref().unwrap().tonic, Note::C);
    }
}
