use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== interval-matrix subcommand ====================

#[test]
fn interval_matrix_text_contains_interval_vector() {
    let out = slonimsky()
        .args(["interval-matrix", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Interval vector: <"),
        "text output should contain reduced IV; got:\n{stdout}"
    );
    // Extract and validate 6-element vector
    let iv_line = stdout
        .lines()
        .find(|l| l.contains("Interval vector:"))
        .unwrap();
    let inner = iv_line
        .split('<')
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    let vals: Vec<u32> = inner
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    assert_eq!(vals.len(), 6, "reduced IV should have 6 elements");
}

#[test]
fn interval_matrix_text_shows_matrix_grid() {
    let out = slonimsky()
        .args(["interval-matrix", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The matrix grid should contain the PCs as row/column headers
    assert!(stdout.contains(" 0"), "matrix should show pc 0");
    assert!(stdout.contains(" 4"), "matrix should show pc 4");
    assert!(stdout.contains(" 7"), "matrix should show pc 7");
    // Should have separator dashes
    assert!(stdout.contains("----"), "matrix should have grid separator");
}

#[test]
fn interval_matrix_full_flag() {
    let out = slonimsky()
        .args(["interval-matrix", "C", "E", "G", "--full"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Full interval vector:"),
        "--full should show full 12-element vector; got:\n{stdout}"
    );
    // Full vector has 12 comma-separated values in brackets
    let fv_line = stdout
        .lines()
        .find(|l| l.contains("Full interval vector:"))
        .unwrap();
    let inner = fv_line
        .split('[')
        .nth(1)
        .unwrap()
        .split(']')
        .next()
        .unwrap();
    let vals: Vec<u32> = inner
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    assert_eq!(vals.len(), 12, "full IV should have 12 elements");
}

#[test]
fn interval_matrix_title_flag() {
    let out = slonimsky()
        .args(["interval-matrix", "C", "E", "G", "--title", "C Major"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("C Major"),
        "title should appear at start of output; got:\n{stdout}"
    );
}

#[test]
fn interval_matrix_svg_output() {
    let dir = tempfile::tempdir().expect("temp dir");
    let out_path = dir.path().join("matrix.svg");

    slonimsky()
        .args([
            "interval-matrix",
            "0",
            "3",
            "6",
            "9",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.starts_with("<svg"), "output should be SVG");
    assert!(content.contains("</svg>"), "SVG should close properly");
    assert!(
        content.contains("<text") || content.contains("<rect"),
        "SVG should contain matrix elements"
    );
}

#[test]
fn interval_matrix_rejects_bad_extension() {
    let dir = tempfile::tempdir().expect("temp dir");
    let out_path = dir.path().join("matrix.pdf");

    slonimsky()
        .args([
            "interval-matrix",
            "C",
            "E",
            "G",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("supports .svg and .txt"));
}

#[test]
fn interval_matrix_no_input_fails() {
    slonimsky()
        .args(["interval-matrix"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ==================== interval-vector subcommand ====================

#[test]
fn interval_vector_text_shows_pcset_and_iv() {
    let out = slonimsky()
        .args(["interval-vector", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("PcSet: {0, 4, 7}"),
        "should display PcSet; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Interval vector: <"),
        "should show reduced IV; got:\n{stdout}"
    );
}

#[test]
fn interval_vector_labeled_breakdown() {
    let out = slonimsky()
        .args(["interval-vector", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ic1:"), "should have ic breakdown labels");
    assert!(stdout.contains("m2/M7"), "should show interval class names");
    assert!(stdout.contains("P4/P5"), "should show P4/P5 label");
    assert!(stdout.contains("tritone"), "should show tritone label");
}

#[test]
fn interval_vector_full_flag() {
    let out = slonimsky()
        .args(["interval-vector", "C", "E", "G", "--full"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Full interval vector"),
        "--full should show full vector; got:\n{stdout}"
    );
    // Should NOT show reduced vector in full mode
    assert!(
        !stdout.contains("Interval vector: <"),
        "full mode should not show reduced vector"
    );
}

#[test]
fn interval_vector_title_flag() {
    let out = slonimsky()
        .args(["interval-vector", "0", "3", "6", "9", "--title", "Dim7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("Dim7"),
        "title should appear at start; got:\n{stdout}"
    );
}

#[test]
fn interval_vector_svg_output() {
    let dir = tempfile::tempdir().expect("temp dir");
    let out_path = dir.path().join("vector.svg");

    slonimsky()
        .args([
            "interval-vector",
            "C",
            "E",
            "G",
            "B",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.starts_with("<svg"), "output should be SVG");
    assert!(content.contains("</svg>"), "SVG should close properly");
    assert!(
        content.contains("<rect") || content.contains("<text"),
        "SVG should contain bar chart elements"
    );
}

#[test]
fn interval_vector_rejects_bad_extension() {
    let dir = tempfile::tempdir().expect("temp dir");
    let out_path = dir.path().join("vector.mid");

    slonimsky()
        .args([
            "interval-vector",
            "C",
            "E",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("supports .svg and .txt"));
}

#[test]
fn interval_vector_no_input_fails() {
    slonimsky()
        .args(["interval-vector"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ==================== subchords subcommand ====================

#[test]
fn subchords_major_scale_size_3_has_35_subsets() {
    let out = slonimsky()
        .args([
            "subchords",
            "C",
            "D",
            "E",
            "F",
            "G",
            "A",
            "B",
            "--size",
            "3",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C(7,3) = 35
    assert!(
        stdout.contains("Total: 35 subchords"),
        "7-note set should have 35 size-3 subsets; got:\n{stdout}"
    );
}

#[test]
fn subchords_dom7_size_3_has_4_subsets() {
    let out = slonimsky()
        .args(["subchords", "C", "E", "G", "Bb", "--size", "3"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C(4,3) = 4
    assert!(
        stdout.contains("Total: 4 subchords"),
        "4-note set should have 4 size-3 subsets; got:\n{stdout}"
    );
}

#[test]
fn subchords_name_flag_labels_subsets() {
    let out = slonimsky()
        .args(["subchords", "C", "E", "G", "Bb", "--size", "3", "--name"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // At least one subset should get a name (CMaj from {0,4,7})
    // Names appear after the set notation like {0,4,7}  CMaj
    let named_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains('{') && !l.contains("Subchords of"))
        .collect();
    assert!(!named_lines.is_empty(), "should have subset lines");
    // At least one line should have a name (not just "?")
    let has_real_name = named_lines.iter().any(|l| {
        let after_brace = l.split('}').nth(1).unwrap_or("");
        let trimmed = after_brace.trim();
        !trimmed.is_empty() && trimmed != "?"
    });
    assert!(
        has_real_name,
        "at least one subset should be named; lines: {named_lines:?}"
    );
}

#[test]
fn subchords_name_flag_uses_set_analysis_instead_of_question_marks() {
    slonimsky()
        .args(["subchords", "0,1,2,3", "--size", "3", "--name"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "{0,1,2}  set 3-1 pf=[0,1,2] iv=<2,1,0,0,0,0>",
        ))
        .stdout(predicate::str::contains("  ?").not());
}

#[test]
fn subchords_size_too_small_fails() {
    slonimsky()
        .args(["subchords", "C", "E", "G", "B", "--size", "2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 3"));
}

#[test]
fn subchords_size_equals_input_fails() {
    slonimsky()
        .args(["subchords", "C", "E", "G", "--size", "3"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("less than"));
}

#[test]
fn subchords_no_input_fails() {
    slonimsky()
        .args(["subchords"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn subchords_header_shows_input_set() {
    let out = slonimsky()
        .args(["subchords", "0", "4", "7", "11", "--size", "3"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Subchords of {"),
        "header should show input set; got:\n{stdout}"
    );
    assert!(
        stdout.contains("(size 3)"),
        "header should show size; got:\n{stdout}"
    );
}

// ==================== subchords: root-relative labeling ====================
// Regression tests for docs/slonimsky-cli-bugs.md §4 — subsets and their names
// were reported against the zero-anchored prime form, so a G-major query came
// back labeled in C.

#[test]
fn subchords_names_use_the_queried_root() {
    let out = slonimsky()
        .args(["subchords", "G,A,B,C,D,E,F#", "--size", "3", "--name"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("GMaj"),
        "G-major query should yield G-rooted names; got:\n{stdout}"
    );
    // The header lists the queried pitch classes, not {0,2,4,6,7,9,11}.
    assert!(
        stdout.contains("Subchords of {1,2,4,6,7,9,11}"),
        "header should show the queried set; got:\n{stdout}"
    );
}

#[test]
fn subchords_bb_major_labels_are_bb_rooted() {
    let out = slonimsky()
        .args(["subchords", "Bb,C,D,Eb,F,G,A", "--size", "5", "--name"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // pc 10 renders as "A#/Bb". The reported bug was that the *first* subset —
    // which contains the queried root — came back as "Cmin (9, 11)".
    assert!(
        stdout.contains("A#/Bbmin"),
        "Bb-major query should yield Bb-rooted names; got:\n{stdout}"
    );
    let first_subset = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("1. {"))
        .expect("should list a first subset");
    assert!(
        first_subset.contains("A#/Bbmin"),
        "the Bb-containing subset must be named from Bb, not C; got: {first_subset}"
    );
}

#[test]
fn subchords_relative_flag_restores_prime_form() {
    let out = slonimsky()
        .args([
            "subchords",
            "G,A,B,C,D,E,F#",
            "--size",
            "3",
            "--name",
            "--relative",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Subchords of {0,2,4,6,7,9,11}") && stdout.contains("prime-form relative"),
        "--relative should report prime form and say so; got:\n{stdout}"
    );
}

#[test]
fn superchords_header_matches_the_queried_key() {
    // Absolute roots are reported in the body (G Maj7 for G,B,D), so the header
    // must not disagree by printing the zeroed shape.
    let out = slonimsky()
        .args(["superchords", "G,B,D", "--min-size", "4", "--max-size", "4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Superchords of {2,7,11}"),
        "header should show the queried set; got:\n{stdout}"
    );
    assert!(
        stdout.contains("G Maj7"),
        "should find G Maj7 above G,B,D; got:\n{stdout}"
    );
}
