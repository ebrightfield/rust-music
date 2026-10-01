use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::parsing::parse_chord_name;
use music::note_collections::pc_set::PcShape;

use super::xor_identity;
use crate::ast::OwnedPitchCircle;
use crate::error::MusicRonError;

/// Identity source resolved from exactly one of chord/pcs/scale (REQ-O27).
#[derive(Debug)]
pub enum PitchCircleIdentity {
    /// Chord symbol parsed into root + shape (REQ-O20).
    Chord {
        root: Note,
        shape: PcShape,
        symbol: String,
    },
    /// Raw pitch-class integers validated to 0..=11.
    Pcs(Vec<Pc>),
    /// Scale name (opaque string for downstream consumers).
    Scale(String),
}

/// A validated pitch-circle configuration ready for rendering.
#[derive(Debug)]
pub struct ResolvedPitchCircle {
    pub identity: PitchCircleIdentity,
    pub root: Option<Note>,
    pub theme: Option<String>,
}

/// Convert an owned pitch-circle AST node into validated runtime types.
///
/// Enforces the XOR identity constraint (REQ-O27): exactly one of
/// `chord`, `pcs`, or `scale` must be present. Chord symbols are
/// delegated to `parse_chord_name` (REQ-O20).
pub fn convert_pitch_circle(
    owned: &OwnedPitchCircle,
) -> Result<ResolvedPitchCircle, MusicRonError> {
    let winner = xor_identity(
        &[
            ("chord", owned.chord.is_some()),
            ("pcs", owned.pcs.is_some()),
            ("scale", owned.scale.is_some()),
        ],
        "PitchCircle",
    )?;

    let identity = if let Some(chord) = &owned.chord {
        debug_assert_eq!(winner, "chord");
        let (root, shape) =
            parse_chord_name(chord).map_err(|e| MusicRonError::UnknownChordSymbol {
                input: chord.clone(),
                inner: e.to_string(),
                path: "PitchCircle.chord".into(),
            })?;
        PitchCircleIdentity::Chord {
            root,
            shape,
            symbol: chord.clone(),
        }
    } else if let Some(pcs) = &owned.pcs {
        let mut validated = Vec::with_capacity(pcs.len());
        for (i, &val) in pcs.iter().enumerate() {
            if val > 11 {
                return Err(MusicRonError::InvalidPitch {
                    input: format!("pc value {} at index {}", val, i),
                    path: format!("PitchCircle.pcs[{}]", i),
                });
            }
            validated.push(Pc::from(val));
        }
        PitchCircleIdentity::Pcs(validated)
    } else if let Some(scale) = &owned.scale {
        PitchCircleIdentity::Scale(scale.clone())
    } else {
        unreachable!("XOR guard ensures exactly one field is present")
    };

    Ok(ResolvedPitchCircle {
        identity,
        root: owned.root,
        theme: owned.theme.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_with_chord(symbol: &str) -> OwnedPitchCircle {
        OwnedPitchCircle {
            meta: None,
            version: None,
            chord: Some(symbol.to_string()),
            pcs: None,
            scale: None,
            root: None,
            theme: None,
        }
    }

    fn make_with_pcs(pcs: Vec<u8>) -> OwnedPitchCircle {
        OwnedPitchCircle {
            meta: None,
            version: None,
            chord: None,
            pcs: Some(pcs),
            scale: None,
            root: None,
            theme: None,
        }
    }

    fn make_with_scale(name: &str) -> OwnedPitchCircle {
        OwnedPitchCircle {
            meta: None,
            version: None,
            chord: None,
            pcs: None,
            scale: Some(name.to_string()),
            root: None,
            theme: None,
        }
    }

    #[test]
    fn chord_identity_resolves() {
        let owned = make_with_chord("CMaj7");
        let resolved = convert_pitch_circle(&owned).unwrap();
        match &resolved.identity {
            PitchCircleIdentity::Chord { root, symbol, .. } => {
                assert_eq!(*root, Note::C);
                assert_eq!(symbol, "CMaj7");
            }
            other => panic!("expected Chord identity, got: {other:?}"),
        }
    }

    #[test]
    fn pcs_identity_resolves() {
        let owned = make_with_pcs(vec![0, 4, 7]);
        let resolved = convert_pitch_circle(&owned).unwrap();
        match &resolved.identity {
            PitchCircleIdentity::Pcs(pcs) => {
                assert_eq!(pcs.len(), 3);
                assert_eq!(pcs[0], Pc::Pc0);
                assert_eq!(pcs[1], Pc::Pc4);
                assert_eq!(pcs[2], Pc::Pc7);
            }
            other => panic!("expected Pcs identity, got: {other:?}"),
        }
    }

    #[test]
    fn scale_identity_resolves() {
        let owned = make_with_scale("major");
        let resolved = convert_pitch_circle(&owned).unwrap();
        match &resolved.identity {
            PitchCircleIdentity::Scale(name) => assert_eq!(name, "major"),
            other => panic!("expected Scale identity, got: {other:?}"),
        }
    }

    #[test]
    fn ambiguous_chord_and_pcs_errors() {
        let owned = OwnedPitchCircle {
            meta: None,
            version: None,
            chord: Some("C".to_string()),
            pcs: Some(vec![0, 4, 7]),
            scale: None,
            root: None,
            theme: None,
        };
        let err = convert_pitch_circle(&owned).unwrap_err();
        match err {
            MusicRonError::AmbiguousIdentity { fields, .. } => {
                assert_eq!(fields, vec!["chord", "pcs"]);
            }
            other => panic!("expected AmbiguousIdentity, got: {other}"),
        }
    }

    #[test]
    fn no_identity_errors() {
        let owned = OwnedPitchCircle {
            meta: None,
            version: None,
            chord: None,
            pcs: None,
            scale: None,
            root: None,
            theme: None,
        };
        let err = convert_pitch_circle(&owned).unwrap_err();
        match err {
            MusicRonError::AmbiguousIdentity { fields, .. } => {
                assert!(fields.is_empty());
            }
            other => panic!("expected AmbiguousIdentity, got: {other}"),
        }
    }

    #[test]
    fn unknown_chord_symbol_errors() {
        let owned = make_with_chord("Xfoo");
        let err = convert_pitch_circle(&owned).unwrap_err();
        match err {
            MusicRonError::UnknownChordSymbol { input, .. } => {
                assert_eq!(input, "Xfoo");
            }
            other => panic!("expected UnknownChordSymbol, got: {other}"),
        }
    }

    #[test]
    fn pc_out_of_range_errors() {
        let owned = make_with_pcs(vec![0, 13, 7]);
        let err = convert_pitch_circle(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { path, .. } => {
                assert_eq!(path, "PitchCircle.pcs[1]");
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn root_and_theme_pass_through() {
        let mut owned = make_with_pcs(vec![0, 3, 7]);
        owned.root = Some(Note::C);
        owned.theme = Some("dark".to_string());
        let resolved = convert_pitch_circle(&owned).unwrap();
        assert_eq!(resolved.root, Some(Note::C));
        assert_eq!(resolved.theme.as_deref(), Some("dark"));
    }
}
