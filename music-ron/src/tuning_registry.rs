//! Named tuning resolution (REQ-O24, O42).
//!
//! Maps tuning name strings (`"standard"`, `"drop_d"`) to static
//! [`Fretboard`] references. Returns pointer-identical values on
//! repeated calls for the same name.

use music::fretboard::{Fretboard, STD_6STR_GTR, DROP_D};
use crate::error::MusicRonError;

/// Resolve a named tuning string to its canonical `&'static Fretboard` instance.
///
/// Returns the exact same reference as `music::fretboard::STD_6STR_GTR` /
/// `DROP_D` etc., preserving pointer identity (REQ-O42).
pub fn resolve(name: &str, path: &str) -> Result<&'static Fretboard, MusicRonError> {
    match name {
        "standard" => Ok(&*STD_6STR_GTR),
        "drop_d" => Ok(&*DROP_D),
        other => Err(MusicRonError::UnknownTuning {
            name: other.into(),
            path: path.into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_resolves() {
        let fb = resolve("standard", "").unwrap();
        assert_eq!(fb.open_strings.len(), 6);
    }

    #[test]
    fn drop_d_resolves() {
        let fb = resolve("drop_d", "").unwrap();
        assert_eq!(fb.open_strings.len(), 6);
    }

    #[test]
    fn named_tuning_resolves_to_same_static() {
        // REQ-O42: pointer identity preserved
        let std_ref = resolve("standard", "").unwrap();
        assert!(std::ptr::eq(std_ref, &*STD_6STR_GTR));

        let drop_ref = resolve("drop_d", "").unwrap();
        assert!(std::ptr::eq(drop_ref, &*DROP_D));
    }

    #[test]
    fn unknown_tuning_is_error() {
        let err = resolve("open_c", "tuning").unwrap_err();
        match err {
            MusicRonError::UnknownTuning { name, path } => {
                assert_eq!(name, "open_c");
                assert_eq!(path, "tuning");
            }
            other => panic!("expected UnknownTuning, got: {other:?}"),
        }
    }
}
