use music::notation::rhythm::duration::Duration;

use crate::ast::common::OwnedDuration;
use crate::error::MusicRonError;
use crate::visitor::duration::parse_duration_shorthand;

/// Resolve an `OwnedDuration` (shorthand string or long-form struct) into a
/// `music::notation::rhythm::duration::Duration`.
pub(crate) fn resolve(owned: &OwnedDuration, path: &str) -> Result<Duration, MusicRonError> {
    let (kind, dots) = match owned {
        OwnedDuration::Shorthand(s) => {
            parse_duration_shorthand(s).map_err(|()| MusicRonError::InvalidDuration {
                input: s.clone(),
                path: path.into(),
            })?
        }
        OwnedDuration::Long { kind, dots } => (*kind, *dots),
    };
    Ok(Duration::new(kind, dots))
}

#[cfg(test)]
mod tests {
    use super::*;
    use music::notation::rhythm::duration::DurationKind;

    #[test]
    fn shorthand_quarter() {
        let owned = OwnedDuration::Shorthand("4".into());
        let dur = resolve(&owned, "test").unwrap();
        assert_eq!(dur.kind(), DurationKind::Qtr);
        assert_eq!(dur.num_dots(), 0);
    }

    #[test]
    fn shorthand_dotted_eighth() {
        let owned = OwnedDuration::Shorthand("8.".into());
        let dur = resolve(&owned, "test").unwrap();
        assert_eq!(dur.kind(), DurationKind::Eighth);
        assert_eq!(dur.num_dots(), 1);
    }

    #[test]
    fn long_form() {
        let owned = OwnedDuration::Long {
            kind: DurationKind::Half,
            dots: 2,
        };
        let dur = resolve(&owned, "test").unwrap();
        assert_eq!(dur.kind(), DurationKind::Half);
        assert_eq!(dur.num_dots(), 2);
    }

    #[test]
    fn invalid_shorthand_is_error() {
        let owned = OwnedDuration::Shorthand("7".into());
        let err = resolve(&owned, "dur_path").unwrap_err();
        match err {
            MusicRonError::InvalidDuration { input, path } => {
                assert_eq!(input, "7");
                assert_eq!(path, "dur_path");
            }
            other => panic!("expected InvalidDuration, got: {other:?}"),
        }
    }
}
