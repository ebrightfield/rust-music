use music::note::note::Note;
use music::note::pitch_class::Pc;
use music::note_collections::chord_name::parsing::parse_chord_name;
use music::note_collections::pc_set::PcShape;

use crate::ast::OwnedIntervalMatrix;
use crate::error::MusicRonError;
use super::xor_identity;

/// Known interval visualization styles, mapping 1:1 to
/// `IntervalBuilder::build_*` methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalStyle {
    Vector,
    FullVector,
    Matrix,
    Linear,
}

/// Identity source resolved from exactly one of chord/pcs/scale.
#[derive(Debug)]
pub enum IntervalMatrixIdentity {
    Chord { root: Note, shape: PcShape, symbol: String },
    Pcs(Vec<Pc>),
    Scale(String),
}

/// A validated interval-matrix configuration ready for rendering.
#[derive(Debug)]
pub struct ResolvedIntervalMatrix {
    pub identity: IntervalMatrixIdentity,
    pub style: IntervalStyle,
    pub root: Option<Note>,
    pub title: Option<String>,
    pub theme: Option<String>,
    pub bar_size: Option<f32>,
    pub cell_size: Option<f32>,
}

fn parse_style(s: &str) -> Result<IntervalStyle, MusicRonError> {
    match s {
        "vector" => Ok(IntervalStyle::Vector),
        "full_vector" => Ok(IntervalStyle::FullVector),
        "matrix" => Ok(IntervalStyle::Matrix),
        "linear" => Ok(IntervalStyle::Linear),
        _ => Err(MusicRonError::UnknownStyle {
            got: s.to_string(),
            path: "IntervalMatrix.style".into(),
        }),
    }
}

/// Convert an owned interval-matrix AST node into validated runtime types.
///
/// Enforces: XOR identity (chord/pcs/scale), known style (REQ-O28),
/// chord-symbol delegation via `parse_chord_name` (REQ-O20).
pub fn convert_interval_matrix(
    owned: &OwnedIntervalMatrix,
) -> Result<ResolvedIntervalMatrix, MusicRonError> {
    let style = parse_style(&owned.style)?;

    let _winner = xor_identity(
        &[("chord", owned.chord.is_some()), ("pcs", owned.pcs.is_some()), ("scale", owned.scale.is_some())],
        "IntervalMatrix",
    )?;

    let identity = if let Some(chord) = &owned.chord {
        let (root, shape) = parse_chord_name(chord).map_err(|e| {
            MusicRonError::UnknownChordSymbol {
                input: chord.clone(),
                inner: e.to_string(),
                path: "IntervalMatrix.chord".into(),
            }
        })?;
        IntervalMatrixIdentity::Chord { root, shape, symbol: chord.clone() }
    } else if let Some(pcs) = &owned.pcs {
        let mut validated = Vec::with_capacity(pcs.len());
        for (i, &val) in pcs.iter().enumerate() {
            if val > 11 {
                return Err(MusicRonError::InvalidPitch {
                    input: format!("pc value {} at index {}", val, i),
                    path: format!("IntervalMatrix.pcs[{}]", i),
                });
            }
            validated.push(Pc::from(val));
        }
        IntervalMatrixIdentity::Pcs(validated)
    } else if let Some(scale) = &owned.scale {
        IntervalMatrixIdentity::Scale(scale.clone())
    } else {
        unreachable!("XOR guard ensures exactly one field is present")
    };

    Ok(ResolvedIntervalMatrix {
        identity,
        style,
        root: owned.root,
        title: owned.title.clone(),
        theme: owned.theme.clone(),
        bar_size: owned.bar_size,
        cell_size: owned.cell_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_with_chord(symbol: &str, style: &str) -> OwnedIntervalMatrix {
        OwnedIntervalMatrix {
            meta: None, version: None,
            chord: Some(symbol.to_string()),
            pcs: None, scale: None,
            root: None, style: style.to_string(),
            title: None, theme: None,
            bar_size: None, cell_size: None,
        }
    }

    fn make_with_pcs(pcs: Vec<u8>, style: &str) -> OwnedIntervalMatrix {
        OwnedIntervalMatrix {
            meta: None, version: None,
            chord: None,
            pcs: Some(pcs),
            scale: None,
            root: None, style: style.to_string(),
            title: None, theme: None,
            bar_size: None, cell_size: None,
        }
    }

    fn make_with_scale(name: &str, style: &str) -> OwnedIntervalMatrix {
        OwnedIntervalMatrix {
            meta: None, version: None,
            chord: None, pcs: None,
            scale: Some(name.to_string()),
            root: None, style: style.to_string(),
            title: None, theme: None,
            bar_size: None, cell_size: None,
        }
    }

    #[test]
    fn chord_with_vector_style() {
        let owned = make_with_chord("CMaj7", "vector");
        let res = convert_interval_matrix(&owned).unwrap();
        assert_eq!(res.style, IntervalStyle::Vector);
        match &res.identity {
            IntervalMatrixIdentity::Chord { root, symbol, .. } => {
                assert_eq!(*root, Note::C);
                assert_eq!(symbol, "CMaj7");
            }
            other => panic!("expected Chord, got: {other:?}"),
        }
    }

    #[test]
    fn pcs_with_matrix_style() {
        let owned = make_with_pcs(vec![0, 3, 7], "matrix");
        let res = convert_interval_matrix(&owned).unwrap();
        assert_eq!(res.style, IntervalStyle::Matrix);
        match &res.identity {
            IntervalMatrixIdentity::Pcs(pcs) => {
                assert_eq!(pcs.len(), 3);
            }
            other => panic!("expected Pcs, got: {other:?}"),
        }
    }

    #[test]
    fn scale_with_full_vector_style() {
        let owned = make_with_scale("major", "full_vector");
        let res = convert_interval_matrix(&owned).unwrap();
        assert_eq!(res.style, IntervalStyle::FullVector);
        match &res.identity {
            IntervalMatrixIdentity::Scale(name) => assert_eq!(name, "major"),
            other => panic!("expected Scale, got: {other:?}"),
        }
    }

    #[test]
    fn linear_style_parses() {
        let owned = make_with_pcs(vec![0, 4, 7], "linear");
        let res = convert_interval_matrix(&owned).unwrap();
        assert_eq!(res.style, IntervalStyle::Linear);
    }

    #[test]
    fn unknown_style_errors() {
        let owned = make_with_chord("C", "heatmap");
        let err = convert_interval_matrix(&owned).unwrap_err();
        match err {
            MusicRonError::UnknownStyle { got, .. } => assert_eq!(got, "heatmap"),
            other => panic!("expected UnknownStyle, got: {other}"),
        }
    }

    #[test]
    fn ambiguous_identity_errors() {
        let owned = OwnedIntervalMatrix {
            meta: None, version: None,
            chord: Some("C".to_string()),
            pcs: Some(vec![0, 4, 7]),
            scale: None,
            root: None, style: "vector".to_string(),
            title: None, theme: None,
            bar_size: None, cell_size: None,
        };
        let err = convert_interval_matrix(&owned).unwrap_err();
        match err {
            MusicRonError::AmbiguousIdentity { fields, .. } => {
                assert_eq!(fields, vec!["chord", "pcs"]);
            }
            other => panic!("expected AmbiguousIdentity, got: {other}"),
        }
    }

    #[test]
    fn no_identity_errors() {
        let owned = OwnedIntervalMatrix {
            meta: None, version: None,
            chord: None, pcs: None, scale: None,
            root: None, style: "vector".to_string(),
            title: None, theme: None,
            bar_size: None, cell_size: None,
        };
        let err = convert_interval_matrix(&owned).unwrap_err();
        match err {
            MusicRonError::AmbiguousIdentity { fields, .. } => {
                assert!(fields.is_empty());
            }
            other => panic!("expected AmbiguousIdentity, got: {other}"),
        }
    }

    #[test]
    fn display_options_pass_through() {
        let mut owned = make_with_pcs(vec![0, 3, 7], "matrix");
        owned.root = Some(Note::C);
        owned.title = Some("My Matrix".to_string());
        owned.theme = Some("dark".to_string());
        owned.bar_size = Some(20.0);
        owned.cell_size = Some(30.0);
        let res = convert_interval_matrix(&owned).unwrap();
        assert_eq!(res.root, Some(Note::C));
        assert_eq!(res.title.as_deref(), Some("My Matrix"));
        assert_eq!(res.theme.as_deref(), Some("dark"));
        assert_eq!(res.bar_size, Some(20.0));
        assert_eq!(res.cell_size, Some(30.0));
    }
}
