use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Text output: correctness ---

#[test]
fn text_output_has_header_with_input_pcs() {
    let out = slonimsky()
        .args(["chord-dictionary", "C", "E", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Chord dictionary:"),
        "should have header line"
    );
    // Header should show pitch class labels
    assert!(stdout.contains("C"), "header should contain C");
}

#[test]
fn text_output_has_total_line() {
    let out = slonimsky()
        .args(["chord-dictionary", "C", "E", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Total:"), "should have total line");
    // C major triad should have many shapes on standard guitar
    assert!(
        stdout.contains("Total: ") && !stdout.contains("Total: 0"),
        "should find at least 1 shape for C major"
    );
}

#[test]
fn text_output_shows_fret_ranges() {
    let out = slonimsky()
        .args(["chord-dictionary", "C", "E", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Each shape line includes "(frets N-M)"
    assert!(
        stdout.contains("(frets "),
        "shape lines should include fret range"
    );
}

#[test]
fn text_output_shapes_are_numbered() {
    let out = slonimsky()
        .args(["chord-dictionary", "C", "E", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // First shape should be numbered "1."
    assert!(
        stdout.contains("  1."),
        "shapes should be numbered starting at 1"
    );
}

#[test]
fn integer_input_works() {
    let out = slonimsky()
        .args(["chord-dictionary", "0", "4", "7"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Total:"));
    assert!(!stdout.contains("Total: 0"));
}

// --- Filtering ---

#[test]
fn max_span_zero_reduces_results() {
    let wide = slonimsky()
        .args(["chord-dictionary", "C", "E", "G", "--max-span", "4"])
        .output()
        .unwrap();
    let narrow = slonimsky()
        .args(["chord-dictionary", "C", "E", "G", "--max-span", "0"])
        .output()
        .unwrap();
    assert!(wide.status.success());
    assert!(narrow.status.success());

    let wide_text = String::from_utf8_lossy(&wide.stdout);
    let narrow_text = String::from_utf8_lossy(&narrow.stdout);

    // Extract total counts
    let wide_total = extract_total(&wide_text);
    let narrow_total = extract_total(&narrow_text);
    assert!(
        wide_total >= narrow_total,
        "max-span=4 ({wide_total}) should find >= max-span=0 ({narrow_total}) shapes"
    );
}

#[test]
fn max_results_caps_output() {
    let out = slonimsky()
        .args(["chord-dictionary", "C", "E", "G", "--max-results", "3"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let total = extract_total(&stdout);
    assert!(total <= 3, "max-results=3 should cap to <=3, got {total}");
}

#[test]
fn tuning_drop_d_works() {
    let out = slonimsky()
        .args(["chord-dictionary", "D", "A", "D", "--tuning", "drop-d"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Total:"));
}

// --- SVG output ---

#[test]
fn svg_output_to_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("chords.svg");
    let out = slonimsky()
        .args([
            "chord-dictionary",
            "C",
            "E",
            "G",
            "-o",
            path.to_str().unwrap(),
            "--max-results",
            "4",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let svg = std::fs::read_to_string(&path).unwrap();
    assert!(svg.starts_with("<svg"), "should start with <svg");
    assert!(svg.contains("</svg>"), "should close with </svg>");
    // Grid layout: should have <g transform="translate(...)"> groups
    assert!(
        svg.contains("<g transform"),
        "should have positioned <g> groups"
    );
    // With 4 shapes, expect 4 groups
    let group_count = svg.matches("<g transform=\"translate(").count();
    assert!(
        (1..=4).contains(&group_count),
        "expected 1-4 positioned groups, got {group_count}"
    );
}

#[test]
fn svg_output_with_theme() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("chords_dark.svg");
    slonimsky()
        .args([
            "chord-dictionary",
            "A",
            "C",
            "E",
            "-o",
            path.to_str().unwrap(),
            "--theme",
            "dark",
            "--max-results",
            "2",
        ])
        .assert()
        .success();

    let svg = std::fs::read_to_string(&path).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

// --- Verbose ---

#[test]
fn verbose_prints_to_stderr() {
    let out = slonimsky()
        .args(["chord-dictionary", "C", "E", "G", "-v"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("chord-dictionary:"),
        "verbose should print diagnostics to stderr"
    );
    assert!(
        stderr.contains("found"),
        "verbose should mention shape count"
    );
}

// --- Error cases ---

#[test]
fn no_input_fails() {
    slonimsky()
        .arg("chord-dictionary")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn bad_tuning_fails() {
    slonimsky()
        .args(["chord-dictionary", "C", "E", "G", "--tuning", "banjo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid tuning"));
}

#[test]
fn bad_extension_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("out.pdf");
    slonimsky()
        .args([
            "chord-dictionary",
            "C",
            "E",
            "G",
            "-o",
            path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("supports text, json, or svg"));
}

// --- Helper ---

fn extract_total(text: &str) -> usize {
    for line in text.lines() {
        if line.starts_with("Total:") {
            // "Total: 15 shapes"
            let num_str = line
                .trim_start_matches("Total:")
                .split_whitespace()
                .next()
                .unwrap_or("0");
            return num_str.parse().unwrap_or(0);
        }
    }
    0
}

#[test]
fn chord_dictionary_accepts_midi_tuning() {
    slonimsky()
        .args([
            "chord-dictionary",
            "C,E,G",
            "--tuning",
            "40,45,50,55,59,64",
            "--max-results",
            "1",
        ])
        .assert()
        .success();
}

#[test]
fn json_exposes_all_shape_metadata() {
    let out = slonimsky()
        .args([
            "chord-dictionary",
            "C,E,G",
            "--classification",
            "playable",
            "--max-results",
            "1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let shape = &value["shapes"][0];
    assert_eq!(shape["classification"], "playable");
    assert!(shape["frets"].is_array());
    assert!(shape["family"].is_array());
    assert!(shape["voicing"].is_array());
    assert!(shape["bass"].is_string());
    assert!(shape["contains_open_strings"].is_boolean());
}

#[test]
fn classification_and_open_string_filters_compose() {
    slonimsky()
        .args([
            "chord-dictionary",
            "D,F#,A",
            "--classification",
            "nontransposable",
            "--open-strings",
            "required",
            "--max-span",
            "12",
            "--max-results",
            "2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("nontransposable"));
}

#[test]
fn fret_range_filter_bounds_every_result() {
    let out = slonimsky()
        .args([
            "chord-dictionary",
            "C,E,G",
            "--min-fret",
            "5",
            "--max-fret",
            "8",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    for shape in value["shapes"].as_array().unwrap() {
        assert!(shape["min_fret"].as_u64().unwrap() >= 5);
        assert!(shape["max_fret"].as_u64().unwrap() <= 8);
    }
}

#[test]
fn family_and_bass_filters_select_an_inversion() {
    slonimsky()
        .args([
            "chord-dictionary",
            "C,E,G",
            "--family",
            "E,G,C",
            "--bass",
            "E",
            "--max-results",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("bass E, family E,G,C"));
}

#[test]
fn exact_voicing_filter_matches_register() {
    slonimsky()
        .args([
            "chord-dictionary",
            "C,E,G",
            "--voicing",
            "G4,C5,E5",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#""voicing": [
        "G4",
        "C5",
        "E5""#,
        ));
}

#[test]
fn unknown_classification_is_rejected() {
    slonimsky()
        .args([
            "chord-dictionary",
            "C,E,G",
            "--classification",
            "comfortable",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown classification"));
}
