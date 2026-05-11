use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Success cases ---

#[test]
fn fretboard_stdout_produces_valid_svg() {
    let out = slonimsky()
        .args(["fretboard", "x-3-2-0-1-0"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"), "output must start with <svg");
    assert!(svg.contains("</svg>"), "output must close </svg>");
    // 6-string chord should produce fret-position markers
    let marker_count = svg.matches("<circle").count() + svg.matches("<rect").count();
    assert!(
        marker_count >= 4,
        "expected ≥4 marker elements for a 6-string shape, got {marker_count}"
    );
}

#[test]
fn fretboard_open_chord_e_minor() {
    let out = slonimsky()
        .args(["fretboard", "0-2-2-0-0-0"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn fretboard_with_muted_strings() {
    // Barre-ish shape with muted low E
    let out = slonimsky()
        .args(["fretboard", "x-1-3-3-2-1"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"));
    // Muted string should have an X marker (typically rendered as text "X" or a path)
    assert!(
        svg.contains("<text") || svg.contains("<line") || svg.contains("<path"),
        "SVG should contain muted-string markers"
    );
}

#[test]
fn fretboard_title_appears_in_svg() {
    let out = slonimsky()
        .args(["fretboard", "x-3-2-0-1-0", "--title", "C Major"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(
        svg.contains("C Major"),
        "title text should appear in SVG output"
    );
}

#[test]
fn fretboard_dark_theme_differs_from_default() {
    let default_out = slonimsky()
        .args(["fretboard", "x-3-2-0-1-0"])
        .output()
        .unwrap();
    let dark_out = slonimsky()
        .args(["fretboard", "x-3-2-0-1-0", "--theme", "dark"])
        .output()
        .unwrap();

    assert!(dark_out.status.success());
    let svg_default = String::from_utf8_lossy(&default_out.stdout);
    let svg_dark = String::from_utf8_lossy(&dark_out.stdout);

    assert_ne!(
        svg_default.as_ref(),
        svg_dark.as_ref(),
        "dark theme should produce different SVG than default"
    );
}

#[test]
fn fretboard_write_to_file() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("chord.svg");

    slonimsky()
        .args([
            "fretboard",
            "x-3-2-0-1-0",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let content = fs::read_to_string(&out_path).expect("output file should exist");
    assert!(content.starts_with("<svg"));
    assert!(content.contains("</svg>"));
}

#[test]
fn fretboard_horizontal_orientation() {
    let vert = slonimsky()
        .args(["fretboard", "0-2-2-1-0-0"])
        .output()
        .unwrap();
    let horiz = slonimsky()
        .args(["fretboard", "0-2-2-1-0-0", "--orientation", "horizontal"])
        .output()
        .unwrap();

    assert!(horiz.status.success());
    let svg_vert = String::from_utf8_lossy(&vert.stdout);
    let svg_horiz = String::from_utf8_lossy(&horiz.stdout);

    // Horizontal layout produces a different SVG viewBox / dimensions
    assert_ne!(
        svg_vert.as_ref(),
        svg_horiz.as_ref(),
        "horizontal orientation should differ from vertical"
    );
}

#[test]
fn fretboard_drop_d_tuning() {
    let out = slonimsky()
        .args(["fretboard", "0-0-0-2-3-2", "--tuning", "drop-d"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"));
}

#[test]
fn fretboard_7string_tuning() {
    // 7-string shape needs 7 fret values
    let out = slonimsky()
        .args(["fretboard", "0-2-2-1-0-0-0", "--tuning", "7-string"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn fretboard_bass_4_tuning() {
    let out = slonimsky()
        .args(["fretboard", "0-2-2-0", "--tuning", "bass-4"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"));
}

#[test]
fn fretboard_num_frets_flag() {
    let default_out = slonimsky()
        .args(["fretboard", "x-3-2-0-1-0"])
        .output()
        .unwrap();
    let wide_out = slonimsky()
        .args(["fretboard", "x-3-2-0-1-0", "--num-frets", "12"])
        .output()
        .unwrap();

    assert!(wide_out.status.success());
    let svg_default = String::from_utf8_lossy(&default_out.stdout);
    let svg_wide = String::from_utf8_lossy(&wide_out.stdout);

    // More frets → larger SVG
    assert!(
        svg_wide.len() > svg_default.len(),
        "12-fret SVG ({} bytes) should be larger than default ({} bytes)",
        svg_wide.len(),
        svg_default.len()
    );
}

#[test]
fn fretboard_all_themes_valid() {
    for theme in &["default", "dark", "print", "colorful"] {
        let out = slonimsky()
            .args(["fretboard", "x-3-2-0-1-0", "--theme", theme])
            .output()
            .unwrap_or_else(|e| panic!("theme '{theme}' failed: {e}"));

        assert!(
            out.status.success(),
            "theme '{theme}' should succeed, got: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let svg = String::from_utf8_lossy(&out.stdout);
        assert!(svg.starts_with("<svg"), "theme '{theme}' should produce SVG");
    }
}

// --- Error cases ---

#[test]
fn fretboard_no_input_fails() {
    slonimsky()
        .args(["fretboard"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn fretboard_bad_fret_notation_fails() {
    slonimsky()
        .args(["fretboard", "not-valid-frets"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid fret notation"));
}

#[test]
fn fretboard_unknown_tuning_fails() {
    slonimsky()
        .args(["fretboard", "x-3-2-0-1-0", "--tuning", "ukulele"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown tuning"));
}

#[test]
fn fretboard_bad_theme_fails() {
    slonimsky()
        .args(["fretboard", "x-3-2-0-1-0", "--theme", "neon"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown theme"));
}

#[test]
fn fretboard_non_svg_extension_fails() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("chord.pdf");

    slonimsky()
        .args([
            "fretboard",
            "x-3-2-0-1-0",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("only supports .svg"));
}
