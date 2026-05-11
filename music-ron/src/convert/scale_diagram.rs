use music::note::note::Note;
use music::note::pitch_class::Pc;

use crate::ast::scale_diagram::OwnedOrientation;
use crate::ast::OwnedScaleDiagram;
use crate::error::MusicRonError;
use super::xor_identity;

/// Identity source resolved from exactly one of pcs/scale (REQ-O27 XOR).
#[derive(Debug)]
pub enum ScaleDiagramIdentity {
    /// Raw pitch-class integers validated to 0..=11.
    Pcs(Vec<Pc>),
    /// Scale name (opaque string for downstream consumers).
    Scale(String),
}

/// Orientation for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// A validated scale-diagram configuration ready for rendering (REQ-O29).
#[derive(Debug)]
pub struct ResolvedScaleDiagram {
    pub identity: ScaleDiagramIdentity,
    pub root: Option<Note>,
    pub orientation: Orientation,
    pub start_fret: u8,
    pub num_frets: u8,
    pub show_degrees: bool,
    pub highlight_root: bool,
    pub theme: Option<String>,
}

/// Convert an owned scale-diagram AST node into validated runtime types.
///
/// Enforces the XOR identity constraint: exactly one of `pcs` or `scale`
/// must be present. Unknown scale names are passed through (downstream
/// consumers raise `UnknownScale` if they can't resolve).
pub fn convert_scale_diagram(
    owned: &OwnedScaleDiagram,
) -> Result<ResolvedScaleDiagram, MusicRonError> {
    let _winner = xor_identity(
        &[("pcs", owned.pcs.is_some()), ("scale", owned.scale.is_some())],
        "ScaleDiagram",
    )?;

    let identity = if let Some(pcs) = &owned.pcs {
        let mut validated = Vec::with_capacity(pcs.len());
        for (i, &val) in pcs.iter().enumerate() {
            if val > 11 {
                return Err(MusicRonError::InvalidPitch {
                    input: format!("pc value {} at index {}", val, i),
                    path: format!("ScaleDiagram.pcs[{}]", i),
                });
            }
            validated.push(Pc::from(val));
        }
        ScaleDiagramIdentity::Pcs(validated)
    } else if let Some(scale) = &owned.scale {
        ScaleDiagramIdentity::Scale(scale.clone())
    } else {
        unreachable!("XOR guard ensures exactly one field is present")
    };

    let orientation = match owned.orientation {
        OwnedOrientation::Horizontal => Orientation::Horizontal,
        OwnedOrientation::Vertical => Orientation::Vertical,
    };

    Ok(ResolvedScaleDiagram {
        identity,
        root: owned.root,
        orientation,
        start_fret: owned.start_fret,
        num_frets: owned.num_frets,
        show_degrees: owned.show_degrees,
        highlight_root: owned.highlight_root,
        theme: owned.theme.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::common::OwnedTuning;

    fn base(pcs: Option<Vec<u8>>, scale: Option<&str>) -> OwnedScaleDiagram {
        OwnedScaleDiagram {
            meta: None,
            version: None,
            pcs,
            scale: scale.map(String::from),
            root: None,
            tuning: OwnedTuning::Named("standard".into()),
            string_convention: None,
            start_fret: 0,
            num_frets: 5,
            orientation: OwnedOrientation::Vertical,
            show_degrees: false,
            highlight_root: false,
            theme: None,
        }
    }

    #[test]
    fn pcs_identity_resolves() {
        let owned = base(Some(vec![0, 2, 4, 5, 7, 9, 11]), None);
        let resolved = convert_scale_diagram(&owned).unwrap();
        match &resolved.identity {
            ScaleDiagramIdentity::Pcs(pcs) => {
                assert_eq!(pcs.len(), 7);
                assert_eq!(pcs[0], Pc::Pc0);
                assert_eq!(pcs[6], Pc::Pc11);
            }
            other => panic!("expected Pcs identity, got: {other:?}"),
        }
    }

    #[test]
    fn scale_identity_resolves() {
        let owned = base(None, Some("major_pentatonic"));
        let resolved = convert_scale_diagram(&owned).unwrap();
        match &resolved.identity {
            ScaleDiagramIdentity::Scale(name) => assert_eq!(name, "major_pentatonic"),
            other => panic!("expected Scale identity, got: {other:?}"),
        }
    }

    #[test]
    fn ambiguous_pcs_and_scale_errors() {
        let owned = base(Some(vec![0, 4, 7]), Some("major"));
        let err = convert_scale_diagram(&owned).unwrap_err();
        match err {
            MusicRonError::AmbiguousIdentity { fields, .. } => {
                assert_eq!(fields, vec!["pcs", "scale"]);
            }
            other => panic!("expected AmbiguousIdentity, got: {other}"),
        }
    }

    #[test]
    fn no_identity_errors() {
        let owned = base(None, None);
        let err = convert_scale_diagram(&owned).unwrap_err();
        match err {
            MusicRonError::AmbiguousIdentity { fields, .. } => {
                assert!(fields.is_empty());
            }
            other => panic!("expected AmbiguousIdentity, got: {other}"),
        }
    }

    #[test]
    fn pc_out_of_range_errors() {
        let owned = base(Some(vec![0, 12, 7]), None);
        let err = convert_scale_diagram(&owned).unwrap_err();
        match err {
            MusicRonError::InvalidPitch { path, .. } => {
                assert_eq!(path, "ScaleDiagram.pcs[1]");
            }
            other => panic!("expected InvalidPitch, got: {other}"),
        }
    }

    #[test]
    fn horizontal_orientation() {
        let mut owned = base(None, Some("minor"));
        owned.orientation = OwnedOrientation::Horizontal;
        let resolved = convert_scale_diagram(&owned).unwrap();
        assert_eq!(resolved.orientation, Orientation::Horizontal);
    }

    #[test]
    fn options_pass_through() {
        let mut owned = base(None, Some("minor"));
        owned.root = Some(Note::E);
        owned.start_fret = 5;
        owned.num_frets = 4;
        owned.show_degrees = true;
        owned.highlight_root = true;
        owned.theme = Some("dark".to_string());

        let resolved = convert_scale_diagram(&owned).unwrap();
        assert_eq!(resolved.root, Some(Note::E));
        assert_eq!(resolved.start_fret, 5);
        assert_eq!(resolved.num_frets, 4);
        assert!(resolved.show_degrees);
        assert!(resolved.highlight_root);
        assert_eq!(resolved.theme.as_deref(), Some("dark"));
    }
}
