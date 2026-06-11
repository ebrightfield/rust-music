use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Text output: correctness ---

#[test]
fn text_output_has_header_with_mode_and_key_counts() {
    let out = slonimsky()
        .args(["scale-book", "major", "--keys", "C"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Scale Book:"),
        "should have 'Scale Book:' header"
    );
    assert!(
        stdout.contains("7 modes"),
        "should mention 7 modes"
    );
    assert!(
        stdout.contains("1 keys"),
        "should mention 1 key"
    );
    assert!(
        stdout.contains("7 entries"),
        "7 modes × 1 key = 7 entries"
    );
}

#[test]
fn text_output_contains_all_major_mode_names() {
    let out = slonimsky()
        .args(["scale-book", "major", "--keys", "C"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for mode in &["Ionian", "Dorian", "Phrygian", "Lydian", "Mixolydian", "Aeolian", "Locrian"] {
        assert!(
            stdout.contains(mode),
            "should contain mode name '{mode}'"
        );
    }
}

#[test]
fn text_output_c_ionian_has_correct_notes() {
    let out = slonimsky()
        .args(["scale-book", "major", "--keys", "C"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C Ionian should contain all natural notes
    // Find the Ionian line for key C
    let ionian_line = stdout.lines()
        .find(|l| l.contains("Ionian") && l.trim().starts_with("C"))
        .expect("should have C Ionian line");
    for note in &["C", "D", "E", "F", "G", "A", "B"] {
        assert!(
            ionian_line.contains(note),
            "C Ionian should contain note {note}, got: {ionian_line}"
        );
    }
}

#[test]
fn text_output_multiple_keys_multiplies_entries() {
    let out = slonimsky()
        .args(["scale-book", "major", "--keys", "C,G,D"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("21 entries"),
        "7 modes × 3 keys = 21 entries"
    );
}

#[test]
fn text_output_total_line_at_end() {
    let out = slonimsky()
        .args(["scale-book", "major", "--keys", "C"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Total: 7 entries"),
        "should have total line"
    );
}

// --- Scale families ---

#[test]
fn melodic_minor_has_correct_mode_names() {
    let out = slonimsky()
        .args(["scale-book", "melodic-minor", "--keys", "C"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Melodic Minor"), "should have Melodic Minor");
    assert!(stdout.contains("Altered"), "should have Altered mode");
    assert!(stdout.contains("Lydian Dominant"), "should have Lydian Dominant");
}

#[test]
fn harmonic_minor_has_correct_mode_names() {
    let out = slonimsky()
        .args(["scale-book", "harmonic-minor", "--keys", "C"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Harmonic Minor"), "should have Harmonic Minor");
    assert!(stdout.contains("Phrygian Dominant"), "should have Phrygian Dominant");
}

#[test]
fn harmonic_major_accepted() {
    let out = slonimsky()
        .args(["scale-book", "harmonic-major", "--keys", "C"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Harmonic Major"), "should have Harmonic Major");
}

// --- SVG output ---

#[test]
fn svg_output_valid_structure() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("book.svg");
    slonimsky()
        .args([
            "scale-book", "major", "--keys", "C,G",
            "-o", out_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.starts_with("<svg"), "should start with <svg");
    assert!(content.contains("</svg>"), "should close SVG");
    assert!(content.contains("Scale Book"), "should contain title");
    assert!(content.contains("Ionian"), "should contain mode name");
    assert!(content.contains("Locrian"), "should contain last mode name");
}

#[test]
fn svg_output_has_pitch_circle_groups() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("book.svg");
    slonimsky()
        .args([
            "scale-book", "major", "--keys", "C",
            "-o", out_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let content = std::fs::read_to_string(&out_path).unwrap();
    // Should have 7 <g transform="translate...scale(0.45)"> groups (one per mode)
    let group_count = content.matches(r#"scale(0.45)"#).count();
    assert_eq!(group_count, 7, "7 modes × 1 key = 7 circle groups");
}

#[test]
fn svg_output_dark_theme_accepted() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("dark.svg");
    slonimsky()
        .args([
            "scale-book", "major", "--keys", "C",
            "-t", "dark",
            "-o", out_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.starts_with("<svg"), "dark theme should produce valid SVG");
}

// --- Verbose ---

#[test]
fn verbose_prints_diagnostics() {
    let out = slonimsky()
        .args(["scale-book", "major", "--keys", "C", "-v"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("scale-book:"),
        "verbose should print diagnostics to stderr"
    );
    assert!(stderr.contains("modes=7"), "should report mode count");
    assert!(stderr.contains("keys=1"), "should report key count");
}

// --- Error cases ---

#[test]
fn no_scale_arg_fails() {
    slonimsky()
        .args(["scale-book"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn unknown_scale_fails() {
    slonimsky()
        .args(["scale-book", "pentatonic"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown scale"));
}

#[test]
fn bad_extension_fails() {
    let dir = TempDir::new().unwrap();
    let out_path = dir.path().join("out.pdf");
    slonimsky()
        .args([
            "scale-book", "major",
            "-o", out_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".svg"));
}
