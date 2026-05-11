use std::io::Write;
use music_ron::{parse_path, MusicRonError};
use tempfile::NamedTempFile;

#[test]
fn parse_path_attaches_source_path_on_syntax_error() {
    let mut f = NamedTempFile::new().unwrap();
    // Unterminated string triggers a syntax error
    writeln!(f, "(kind: \"Snippet\", clef: \"treble\", events: [Note(pitch: \"c4\", duration: \"").unwrap();
    let err = parse_path(f.path()).unwrap_err();
    match err {
        MusicRonError::SyntaxError { source_path: Some(p), .. } => {
            assert_eq!(p, f.path().display().to_string());
        }
        other => panic!("expected SyntaxError with source_path, got: {other:?}"),
    }
}

#[test]
fn parse_path_strips_bom() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all("\u{FEFF}".as_bytes()).unwrap();
    f.write_all(b"(kind: \"Snippet\", clef: \"treble\", events: [])").unwrap();
    parse_path(f.path()).unwrap();
}

#[test]
fn parse_path_io_error_on_missing_file() {
    let p = std::path::Path::new("/nonexistent/deliberately-missing.ron");
    let err = parse_path(p).unwrap_err();
    match err {
        MusicRonError::Io { source_path, .. } => {
            assert_eq!(source_path, p);
        }
        other => panic!("expected Io error, got: {other:?}"),
    }
}

#[test]
fn parse_path_unsupported_version_carries_source_path() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "(kind: \"Snippet\", version: 99, clef: \"treble\", events: [])").unwrap();
    let err = parse_path(f.path()).unwrap_err();
    match err {
        MusicRonError::UnsupportedVersion { got: 99, max_supported: 1, source_path: Some(p) } => {
            assert_eq!(p, f.path().display().to_string());
        }
        other => panic!("expected UnsupportedVersion, got: {other:?}"),
    }
}
