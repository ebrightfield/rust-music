use music::note::pitch::Pitch;
use music::note_collections::voicing::Voicing;

use crate::ast::common::OwnedPitch;
use crate::error::MusicRonError;
use crate::visitor::pitch::parse_pitch_shorthand;

/// Resolve an `OwnedPitch` (shorthand string or long-form struct) into a
/// `music::note::pitch::Pitch`, surfacing octave-range or parse errors.
pub(crate) fn resolve(owned: &OwnedPitch, path: &str) -> Result<Pitch, MusicRonError> {
    let (note, octave_i32) = match owned {
        OwnedPitch::Shorthand(s) => parse_pitch_shorthand(s).map_err(|(input, _)| {
            MusicRonError::InvalidPitch {
                input,
                path: path.into(),
            }
        })?,
        OwnedPitch::Long { note, octave } => (*note, *octave),
    };
    let octave = i8::try_from(octave_i32).map_err(|_| MusicRonError::OctaveOutOfRange {
        got: octave_i32,
        path: path.into(),
    })?;
    Pitch::try_new(note, octave).map_err(|_| MusicRonError::OctaveOutOfRange {
        got: octave_i32,
        path: path.into(),
    })
}

/// Resolve a slice of `OwnedPitch` into a `Voicing` (sorted pitch collection).
pub(crate) fn resolve_voicing(
    pitches: &[OwnedPitch],
    path: &str,
) -> Result<Voicing, MusicRonError> {
    let resolved: Vec<Pitch> = pitches
        .iter()
        .enumerate()
        .map(|(i, p)| resolve(p, &format!("{path}[{i}]")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Voicing::new(resolved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::note::note::Note;

    #[test]
    fn shorthand_c4() {
        let owned = OwnedPitch::Shorthand("c4".into());
        let pitch = resolve(&owned, "test").unwrap();
        assert_eq!(pitch, Pitch::new(Note::C, 4));
    }

    #[test]
    fn long_form_des3() {
        let owned = OwnedPitch::Long {
            note: Note::Des,
            octave: 3,
        };
        let pitch = resolve(&owned, "test").unwrap();
        assert_eq!(pitch, Pitch::new(Note::Des, 3));
    }

    #[test]
    fn octave_out_of_range_high() {
        let owned = OwnedPitch::Long {
            note: Note::C,
            octave: 200,
        };
        let err = resolve(&owned, "oct_path").unwrap_err();
        match err {
            MusicRonError::OctaveOutOfRange { got, path } => {
                assert_eq!(got, 200);
                assert_eq!(path, "oct_path");
            }
            other => panic!("expected OctaveOutOfRange, got: {other:?}"),
        }
    }

    #[test]
    fn octave_out_of_range_negative() {
        let owned = OwnedPitch::Long {
            note: Note::C,
            octave: -5,
        };
        let err = resolve(&owned, "neg").unwrap_err();
        match err {
            MusicRonError::OctaveOutOfRange { got, .. } => assert_eq!(got, -5),
            other => panic!("expected OctaveOutOfRange, got: {other:?}"),
        }
    }

    #[test]
    fn invalid_shorthand_is_error() {
        let owned = OwnedPitch::Shorthand("xyz".into());
        let err = resolve(&owned, "p").unwrap_err();
        match err {
            MusicRonError::InvalidPitch { input, path } => {
                assert_eq!(input, "xyz");
                assert_eq!(path, "p");
            }
            other => panic!("expected InvalidPitch, got: {other:?}"),
        }
    }

    #[test]
    fn resolve_voicing_two_pitches() {
        let pitches = vec![
            OwnedPitch::Shorthand("c4".into()),
            OwnedPitch::Shorthand("e4".into()),
            OwnedPitch::Shorthand("g4".into()),
        ];
        let voicing = resolve_voicing(&pitches, "chord").unwrap();
        assert_eq!(voicing.len(), 3);
    }

    #[test]
    fn resolve_voicing_one_bad_pitch_errors() {
        let pitches = vec![
            OwnedPitch::Shorthand("c4".into()),
            OwnedPitch::Shorthand("bad".into()),
        ];
        let err = resolve_voicing(&pitches, "chord").unwrap_err();
        assert!(matches!(err, MusicRonError::InvalidPitch { .. }));
    }
}
