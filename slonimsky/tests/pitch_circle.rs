use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Success cases ---

#[test]
fn pitch_circle_stdout_produces_valid_svg() {
    let out = slonimsky()
        .args(["pitch-circle", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"), "output must start with <svg");
    assert!(svg.contains("</svg>"), "output must close </svg>");
    // C major triad has 3 pitch classes → at least 3 circle/text markers
    let circle_count = svg.matches("<circle").count();
    assert!(
        circle_count >= 3,
        "expected ≥3 <circle> elements for 3 PCs, got {circle_count}"
    );
}

#[test]
fn pitch_circle_integer_input() {
    let out = slonimsky()
        .args(["pitch-circle", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"));
    // Same triad as C E G — should produce identical structure
    assert!(svg.matches("<circle").count() >= 3);
}

#[test]
fn pitch_circle_comma_separated() {
    let out = slonimsky()
        .args(["pitch-circle", "0,4,7,11"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.matches("<circle").count() >= 4);
}

#[test]
fn pitch_circle_show_intervals_adds_lines() {
    let without = slonimsky()
        .args(["pitch-circle", "C", "E", "G"])
        .output()
        .unwrap();
    let with = slonimsky()
        .args(["pitch-circle", "C", "E", "G", "--show-intervals"])
        .output()
        .unwrap();

    let svg_without = String::from_utf8_lossy(&without.stdout);
    let svg_with = String::from_utf8_lossy(&with.stdout);

    // --show-intervals should produce more SVG content (lines between PCs)
    assert!(
        svg_with.len() > svg_without.len(),
        "show-intervals SVG ({} bytes) should be larger than plain ({} bytes)",
        svg_with.len(),
        svg_without.len()
    );
}

#[test]
fn pitch_circle_title_appears_in_svg() {
    let out = slonimsky()
        .args(["pitch-circle", "C", "E", "G", "--title", "C Major Triad"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(
        svg.contains("C Major Triad"),
        "title text should appear in SVG output"
    );
}

#[test]
fn pitch_circle_dark_theme_changes_colors() {
    let default_out = slonimsky()
        .args(["pitch-circle", "C", "E", "G"])
        .output()
        .unwrap();
    let dark_out = slonimsky()
        .args(["pitch-circle", "C", "E", "G", "--theme", "dark"])
        .output()
        .unwrap();

    assert!(dark_out.status.success());
    let svg_default = String::from_utf8_lossy(&default_out.stdout);
    let svg_dark = String::from_utf8_lossy(&dark_out.stdout);

    // Dark and default themes must produce different SVG
    assert_ne!(
        svg_default.as_ref(),
        svg_dark.as_ref(),
        "dark theme should differ from default"
    );
}

#[test]
fn pitch_circle_write_to_file() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("triad.svg");

    slonimsky()
        .args([
            "pitch-circle",
            "C",
            "E",
            "G",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let content = fs::read_to_string(&out_path).expect("output file should exist");
    assert!(content.starts_with("<svg"));
    assert!(content.contains("</svg>"));
    assert!(content.matches("<circle").count() >= 3);
}

#[test]
fn pitch_circle_sharps_and_flats() {
    slonimsky()
        .args(["pitch-circle", "C#", "Eb", "F#", "Bb"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("<svg"));
}

#[test]
fn pitch_circle_root_flag() {
    let out = slonimsky()
        .args(["pitch-circle", "C", "E", "G", "--root", "E"])
        .output()
        .unwrap();

    assert!(out.status.success());
    let svg = String::from_utf8_lossy(&out.stdout);
    assert!(svg.starts_with("<svg"));
}

// --- Error cases ---

#[test]
fn pitch_circle_no_input_fails() {
    slonimsky()
        .args(["pitch-circle"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn pitch_circle_bad_input_fails() {
    slonimsky()
        .args(["pitch-circle", "xyz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized pitch class"));
}

#[test]
fn pitch_circle_bad_theme_fails() {
    slonimsky()
        .args(["pitch-circle", "C", "E", "G", "--theme", "neon"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown theme"));
}

#[test]
fn pitch_circle_non_svg_extension_fails() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("triad.pdf");

    slonimsky()
        .args([
            "pitch-circle",
            "C",
            "E",
            "G",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("only supports .svg"));
}

// --- Global flags ---

#[test]
fn version_flag() {
    slonimsky()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn help_flag() {
    slonimsky()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("pitch-circle"));
}

#[test]
fn no_subcommand_shows_help() {
    slonimsky()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}
