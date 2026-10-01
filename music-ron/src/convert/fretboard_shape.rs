use music::fretboard::StringConvention;

use crate::ast::common::OwnedBarre;
use crate::ast::fretboard_shape::{OwnedFretValue, OwnedFretboardShape};
use crate::error::MusicRonError;

/// Validated per-string fret assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FretValue {
    Muted,
    Open,
    Fingered(u8),
}

/// Validated barre descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBarre {
    pub fret: u8,
    pub from_string: u8,
    pub to_string: u8,
}

/// A validated fretboard-shape ready for rendering (REQ-O26).
#[derive(Debug)]
pub struct ResolvedFretboardShape {
    pub frets: Vec<FretValue>,
    pub fingers: Option<Vec<Option<u8>>>,
    pub barre: Option<ResolvedBarre>,
    pub tuning_name: Option<String>,
    pub string_convention: Option<StringConvention>,
}

/// Convert an owned fretboard-shape AST node into validated runtime types.
///
/// Validates:
/// - Each fret value: "x"/"X" → Muted, 0 → Open, 1..=35 → Fingered
/// - Barre string range within frets.len()
/// - Barre from_string < to_string
/// - Fingers length matches frets length (if present)
pub fn convert_fretboard_shape(
    owned: &OwnedFretboardShape,
) -> Result<ResolvedFretboardShape, MusicRonError> {
    let num_strings = owned.frets.len();

    // Validate each fret value.
    let mut frets = Vec::with_capacity(num_strings);
    for (i, fv) in owned.frets.iter().enumerate() {
        let resolved = match fv {
            OwnedFretValue::Muted(s) => {
                if s.eq_ignore_ascii_case("x") {
                    FretValue::Muted
                } else {
                    return Err(MusicRonError::InvalidPitch {
                        input: s.clone(),
                        path: format!("FretboardShape.frets[{}]", i),
                    });
                }
            }
            OwnedFretValue::Fret(0) => FretValue::Open,
            OwnedFretValue::Fret(n) if *n <= 35 => FretValue::Fingered(*n),
            OwnedFretValue::Fret(n) => {
                return Err(MusicRonError::InvalidPitch {
                    input: format!("fret {}", n),
                    path: format!("FretboardShape.frets[{}]", i),
                });
            }
        };
        frets.push(resolved);
    }

    // Validate fingers length if present.
    if let Some(fingers) = &owned.fingers {
        if fingers.len() != num_strings {
            return Err(MusicRonError::InvalidPitch {
                input: format!(
                    "fingers length {} != frets length {}",
                    fingers.len(),
                    num_strings
                ),
                path: "FretboardShape.fingers".into(),
            });
        }
    }

    // Validate barre if present.
    let barre = if let Some(b) = &owned.barre {
        validate_barre(b, num_strings)?;
        Some(ResolvedBarre {
            fret: b.fret,
            from_string: b.from_string,
            to_string: b.to_string,
        })
    } else {
        None
    };

    let tuning_name = match &owned.tuning {
        crate::ast::common::OwnedTuning::Named(name) => Some(name.clone()),
        crate::ast::common::OwnedTuning::Inline { .. } => None,
    };

    Ok(ResolvedFretboardShape {
        frets,
        fingers: owned.fingers.clone(),
        barre,
        tuning_name,
        string_convention: owned.string_convention,
    })
}

fn validate_barre(b: &OwnedBarre, num_strings: usize) -> Result<(), MusicRonError> {
    if b.fret == 0 {
        return Err(MusicRonError::InvalidPitch {
            input: "barre fret 0".into(),
            path: "FretboardShape.barre.fret".into(),
        });
    }
    if b.from_string >= b.to_string {
        return Err(MusicRonError::InvalidPitch {
            input: format!(
                "barre from_string ({}) >= to_string ({})",
                b.from_string, b.to_string
            ),
            path: "FretboardShape.barre".into(),
        });
    }
    if b.to_string as usize >= num_strings {
        return Err(MusicRonError::InvalidPitch {
            input: format!(
                "barre to_string ({}) >= num_strings ({})",
                b.to_string, num_strings
            ),
            path: "FretboardShape.barre.to_string".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::common::{OwnedBarre, OwnedTuning};
    use crate::ast::fretboard_shape::OwnedFretValue;

    fn base_shape(frets: Vec<OwnedFretValue>) -> OwnedFretboardShape {
        OwnedFretboardShape {
            meta: None,
            version: None,
            tuning: OwnedTuning::Named("standard".into()),
            string_convention: None,
            frets,
            fingers: None,
            barre: None,
        }
    }

    #[test]
    fn open_chord_resolves() {
        // E-major open: 0 2 2 1 0 0
        let owned = base_shape(vec![
            OwnedFretValue::Fret(0),
            OwnedFretValue::Fret(2),
            OwnedFretValue::Fret(2),
            OwnedFretValue::Fret(1),
            OwnedFretValue::Fret(0),
            OwnedFretValue::Fret(0),
        ]);
        let resolved = convert_fretboard_shape(&owned).unwrap();
        assert_eq!(resolved.frets[0], FretValue::Open);
        assert_eq!(resolved.frets[1], FretValue::Fingered(2));
        assert_eq!(resolved.frets[3], FretValue::Fingered(1));
        assert_eq!(resolved.tuning_name, Some("standard".into()));
    }

    #[test]
    fn muted_string_x() {
        let owned = base_shape(vec![
            OwnedFretValue::Muted("x".into()),
            OwnedFretValue::Fret(3),
            OwnedFretValue::Fret(2),
            OwnedFretValue::Fret(0),
            OwnedFretValue::Fret(1),
            OwnedFretValue::Fret(0),
        ]);
        let resolved = convert_fretboard_shape(&owned).unwrap();
        assert_eq!(resolved.frets[0], FretValue::Muted);
    }

    #[test]
    fn muted_string_uppercase_x() {
        let owned = base_shape(vec![
            OwnedFretValue::Muted("X".into()),
            OwnedFretValue::Fret(0),
        ]);
        let resolved = convert_fretboard_shape(&owned).unwrap();
        assert_eq!(resolved.frets[0], FretValue::Muted);
    }

    #[test]
    fn invalid_muted_string_rejected() {
        let owned = base_shape(vec![OwnedFretValue::Muted("mute".into())]);
        let err = convert_fretboard_shape(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { path, .. } => {
                assert_eq!(path, "FretboardShape.frets[0]");
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn fret_over_35_rejected() {
        let owned = base_shape(vec![OwnedFretValue::Fret(36)]);
        let err = convert_fretboard_shape(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { input, .. } => {
                assert!(input.contains("36"));
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn barre_valid() {
        let mut owned = base_shape(vec![
            OwnedFretValue::Muted("x".into()),
            OwnedFretValue::Fret(3),
            OwnedFretValue::Fret(3),
            OwnedFretValue::Fret(3),
            OwnedFretValue::Fret(3),
            OwnedFretValue::Fret(3),
        ]);
        owned.barre = Some(OwnedBarre {
            fret: 3,
            from_string: 1,
            to_string: 5,
        });
        let resolved = convert_fretboard_shape(&owned).unwrap();
        let b = resolved.barre.unwrap();
        assert_eq!(b.fret, 3);
        assert_eq!(b.from_string, 1);
        assert_eq!(b.to_string, 5);
    }

    #[test]
    fn barre_fret_zero_rejected() {
        let mut owned = base_shape(vec![OwnedFretValue::Fret(0); 6]);
        owned.barre = Some(OwnedBarre {
            fret: 0,
            from_string: 0,
            to_string: 5,
        });
        let err = convert_fretboard_shape(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { input, .. } => {
                assert!(input.contains("barre fret 0"));
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn barre_from_ge_to_rejected() {
        let mut owned = base_shape(vec![OwnedFretValue::Fret(1); 6]);
        owned.barre = Some(OwnedBarre {
            fret: 1,
            from_string: 3,
            to_string: 3,
        });
        let err = convert_fretboard_shape(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { path, .. } => {
                assert_eq!(path, "FretboardShape.barre");
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn barre_out_of_range_rejected() {
        let mut owned = base_shape(vec![OwnedFretValue::Fret(1); 4]);
        owned.barre = Some(OwnedBarre {
            fret: 1,
            from_string: 0,
            to_string: 4, // only 4 strings (indices 0..3)
        });
        let err = convert_fretboard_shape(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { path, .. } => {
                assert_eq!(path, "FretboardShape.barre.to_string");
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn fingers_length_mismatch_rejected() {
        let mut owned = base_shape(vec![OwnedFretValue::Fret(0); 6]);
        owned.fingers = Some(vec![None, Some(1), Some(2)]); // 3 != 6
        let err = convert_fretboard_shape(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { path, .. } => {
                assert_eq!(path, "FretboardShape.fingers");
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn fingers_valid_passthrough() {
        let mut owned = base_shape(vec![
            OwnedFretValue::Fret(0),
            OwnedFretValue::Fret(2),
            OwnedFretValue::Fret(2),
            OwnedFretValue::Fret(1),
            OwnedFretValue::Fret(0),
            OwnedFretValue::Fret(0),
        ]);
        owned.fingers = Some(vec![None, Some(2), Some(3), Some(1), None, None]);
        let resolved = convert_fretboard_shape(&owned).unwrap();
        let f = resolved.fingers.unwrap();
        assert_eq!(f[1], Some(2));
        assert_eq!(f[0], None);
    }
}
