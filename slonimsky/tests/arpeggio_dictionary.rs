use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Basic text output ---

#[test]
fn major_triad_default_text_output() {
    let out = slonimsky()
        .args(["arpeggio-dictionary", "C", "E", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Arpeggio Dictionary:"),
        "should have header"
    );
    assert!(
        stdout.contains("Tuning: standard"),
        "default tuning should be standard"
    );
    assert!(
        stdout.contains("12 keys"),
        "default should show all 12 keys, got:\n{}",
        stdout.lines().take(5).collect::<Vec<_>>().join("\n")
    );
    // Should have a Key line for at least C
    assert!(stdout.contains("Key: C"), "should contain Key: C section");
    assert!(stdout.contains("Total:"), "should have total summary line");
}

#[test]
fn text_output_contains_fret_positions() {
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "--keys",
            "C",
            "--positions",
            "3",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Shapes should show fret ranges like "(frets X-Y)"
    assert!(
        stdout.contains("frets"),
        "shape entries should show fret ranges"
    );
    // Key C section
    assert!(stdout.contains("Key: C"));
    assert!(stdout.contains("1 keys"), "should show 1 key when --keys C");
}

// --- Key filtering ---

#[test]
fn specific_keys_filter() {
    let out = slonimsky()
        .args(["arpeggio-dictionary", "C", "E", "G", "--keys", "C,G,D"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("3 keys"), "should show 3 keys");
    assert!(stdout.contains("Key: C"));
    assert!(stdout.contains("Key: G"));
    assert!(stdout.contains("Key: D"));
    // Should NOT have keys outside the filter
    assert!(
        !stdout.contains("Key: Bb"),
        "should not contain unselected key Bb"
    );
}

#[test]
fn single_key() {
    let out = slonimsky()
        .args(["arpeggio-dictionary", "C", "E", "G", "--keys", "A"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1 keys"));
    assert!(stdout.contains("Key: A"));
}

// --- Tuning options ---

#[test]
fn drop_d_tuning() {
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "G",
            "--tuning",
            "drop-d",
            "--keys",
            "D",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Tuning: drop-d"),
        "should show drop-d tuning"
    );
    assert!(stdout.contains("Key: D"));
}

#[test]
fn seven_string_tuning() {
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "B",
            "--tuning",
            "7-string",
            "--keys",
            "C",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Tuning: 7-string"));
}

#[test]
fn unknown_tuning_fails() {
    slonimsky()
        .args(["arpeggio-dictionary", "C", "E", "G", "--tuning", "banjo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid tuning"));
}

// --- Positions and span ---

#[test]
fn positions_limits_shapes_per_key() {
    // With 1 position, we should get at most 1 shape per key
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "--keys",
            "C",
            "--positions",
            "1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Count numbered shape lines ("  N. ...")
    let shape_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.trim_start().starts_with("1."))
        .collect();
    assert!(
        shape_lines.len() <= 1,
        "with --positions 1, should have at most 1 shape, got {}",
        shape_lines.len()
    );
}

#[test]
fn max_span_constrains_shapes() {
    // Very tight span should potentially yield fewer shapes
    let tight = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "--keys",
            "C",
            "--max-span",
            "2",
            "--positions",
            "10",
        ])
        .output()
        .unwrap();
    let wide = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "--keys",
            "C",
            "--max-span",
            "6",
            "--positions",
            "10",
        ])
        .output()
        .unwrap();
    assert!(tight.status.success());
    assert!(wide.status.success());
    let tight_stdout = String::from_utf8_lossy(&tight.stdout);
    let wide_stdout = String::from_utf8_lossy(&wide.stdout);
    // Count shape lines in each
    let count_shapes = |s: &str| -> usize {
        s.lines()
            .filter(|l| {
                let trimmed = l.trim();
                trimmed.len() > 2
                    && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && trimmed.contains("frets")
            })
            .count()
    };
    let tight_n = count_shapes(&tight_stdout);
    let wide_n = count_shapes(&wide_stdout);
    assert!(
        wide_n >= tight_n,
        "wider span should yield >= shapes: wide={wide_n} tight={tight_n}"
    );
}

// --- SVG output ---

#[test]
fn svg_output_to_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_arpeggio.svg");
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "--keys",
            "C,G",
            "--positions",
            "2",
            "-o",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "SVG generation should succeed");
    let svg = fs::read_to_string(&path).expect("SVG file should exist");
    assert!(svg.starts_with("<svg"), "should start with <svg tag");
    assert!(svg.contains("</svg>"), "should close with </svg>");
    assert!(
        svg.contains("Arpeggio Dictionary"),
        "SVG should contain title text"
    );
    // Grid should have key labels
    assert!(
        svg.contains(">C<") || svg.contains("C</text>"),
        "SVG should contain key C label"
    );
    assert!(
        svg.contains(">G<") || svg.contains("G</text>"),
        "SVG should contain key G label"
    );
}

#[test]
fn svg_dark_theme() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dark_arpeggio.svg");
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "Eb",
            "G",
            "--keys",
            "C",
            "--positions",
            "2",
            "--theme",
            "dark",
            "-o",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let svg = fs::read_to_string(&path).expect("SVG file should exist");
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

// --- Error cases ---

#[test]
fn no_input_fails() {
    slonimsky().args(["arpeggio-dictionary"]).assert().failure();
}

#[test]
fn invalid_key_fails() {
    slonimsky()
        .args(["arpeggio-dictionary", "C", "E", "G", "--keys", "Z"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn unsupported_output_format_fails() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.pdf");
    slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "-o",
            path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".svg or .txt"));
}

// --- Verbose mode ---

#[test]
fn verbose_prints_generation_info() {
    let out = slonimsky()
        .args(["arpeggio-dictionary", "C", "E", "G", "--keys", "C", "-v"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("arpeggio-dictionary"),
        "verbose stderr should mention arpeggio-dictionary"
    );
    assert!(
        stderr.contains("positions"),
        "verbose stderr should mention positions"
    );
}

// --- Different chord types ---

#[test]
fn minor_seventh_chord() {
    let out = slonimsky()
        .args(["arpeggio-dictionary", "C", "Eb", "G", "Bb", "--keys", "A"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Key: A"));
    assert!(
        stdout.contains("Arpeggio Dictionary:"),
        "should have header for 4-note chord"
    );
}

#[test]
fn power_chord_dyad() {
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "G",
            "--keys",
            "E",
            "--positions",
            "3",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Key: E"));
    // A dyad should find shapes
    assert!(
        stdout.contains("frets"),
        "power chord dyad should produce shapes"
    );
}

#[test]
fn arpeggio_dictionary_accepts_tuning_file() {
    let dir = TempDir::new().unwrap();
    let tuning = dir.path().join("tuning.txt");
    std::fs::write(&tuning, "E3 A3 D4 G4 B4 E5\n").unwrap();
    slonimsky()
        .args([
            "arpeggio-dictionary",
            "C,E,G",
            "--tuning",
            &format!("@{}", tuning.display()),
            "--keys",
            "C",
            "--positions",
            "1",
        ])
        .assert()
        .success();
}
