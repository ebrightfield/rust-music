use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== superchords subcommand ====================

#[test]
fn superchords_c_major_triad_finds_4note_chords() {
    let out = slonimsky()
        .args(["superchords", "C", "E", "G", "--max-size", "4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C major triad should produce several 4-note superchords
    assert!(
        stdout.contains("4-note chords"),
        "should show 4-note chord section; got:\n{stdout}"
    );
    // Must include Cmaj7 (C Major + B = {0,4,7,11})
    assert!(
        stdout.contains("C Maj7"),
        "C major triad should have CMaj7 as superchord; got:\n{stdout}"
    );
    // Must include Cdom7 (C Major + Bb = {0,4,7,10})
    assert!(
        stdout.contains("C Dom7"),
        "C major triad should have CDom7 as superchord; got:\n{stdout}"
    );
}

#[test]
fn superchords_shows_total_count() {
    let out = slonimsky()
        .args(["superchords", "C", "E", "G", "--max-size", "4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Total:"),
        "output should show total count; got:\n{stdout}"
    );
    // Parse the total — C major triad should have multiple 4-note superchords
    let total_line = stdout.lines().find(|l| l.contains("Total:")).unwrap();
    let count: usize = total_line
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .expect("total should be a number");
    assert!(
        count >= 2,
        "C major triad should have at least 2 four-note superchords, got {count}"
    );
}

#[test]
fn superchords_header_shows_input_set() {
    let out = slonimsky()
        .args(["superchords", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Superchords of {0,4,7}"),
        "header should show the input PcSet; got:\n{stdout}"
    );
}

#[test]
fn superchords_scales_for_c_major() {
    let out = slonimsky()
        .args([
            "superchords",
            "C",
            "E",
            "G",
            "--min-size",
            "7",
            "--max-size",
            "7",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("7-note scales"),
        "should show 7-note scale section; got:\n{stdout}"
    );
    // C major triad is in C Major scale
    assert!(
        stdout.contains("C Major"),
        "C major triad should be in C Major scale; got:\n{stdout}"
    );
}

#[test]
fn superchords_none_found_for_impossible_size() {
    // Ask for 5-note superchords of a triad — catalog has no 5-note types
    let out = slonimsky()
        .args([
            "superchords",
            "C",
            "E",
            "G",
            "--min-size",
            "5",
            "--max-size",
            "5",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("(none found)"),
        "should report none found for size 5; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Total: 0"),
        "total should be 0; got:\n{stdout}"
    );
}

#[test]
fn superchords_min_size_equals_input_fails() {
    // min-size must be > input size; input has 3 PCs so min-size=3 should fail
    slonimsky()
        .args(["superchords", "C", "E", "G", "--min-size", "3"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be greater than input size"));
}

#[test]
fn superchords_no_input_fails() {
    slonimsky()
        .args(["superchords"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ==================== prime-form subcommand ====================

#[test]
fn prime_form_c_major_triad() {
    let out = slonimsky()
        .args(["prime-form", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[0, 3, 7]"),
        "C major prime form should be [0, 3, 7]; got:\n{stdout}"
    );
}

#[test]
fn prime_form_d_minor_same_class_as_major() {
    // D minor {2,5,9} is the same set class as major triad → [0, 3, 7]
    let out = slonimsky()
        .args(["prime-form", "D", "F", "A"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[0, 3, 7]"),
        "D minor prime form should be [0, 3, 7]; got:\n{stdout}"
    );
}

#[test]
fn prime_form_shows_pc_set() {
    let out = slonimsky()
        .args(["prime-form", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("PcSet:"),
        "output should show PcSet line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Prime form:"),
        "output should show Prime form line; got:\n{stdout}"
    );
}

#[test]
fn prime_form_dim7_symmetric() {
    let out = slonimsky()
        .args(["prime-form", "0", "3", "6", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[0, 3, 6, 9]"),
        "dim7 prime form should be [0, 3, 6, 9]; got:\n{stdout}"
    );
}

#[test]
fn prime_form_verbose_shows_symmetry() {
    let out = slonimsky()
        .args(["prime-form", "0", "3", "6", "9", "-v"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Verbose mode shows IV
    assert!(
        stdout.contains("IV:"),
        "verbose should show interval vector; got:\n{stdout}"
    );
    // Dim7 has transpositional symmetry (T3, T6, T9)
    assert!(
        stdout.contains("T-symmetry:"),
        "verbose should show T-symmetry; got:\n{stdout}"
    );
    // Dim7 is inversionally symmetric
    assert!(
        stdout.contains("I-symmetry: yes"),
        "dim7 should be inversionally symmetric; got:\n{stdout}"
    );
}

#[test]
fn prime_form_verbose_no_symmetry_for_major_triad() {
    let out = slonimsky()
        .args(["prime-form", "C", "E", "G", "-v"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Major triad has no transpositional symmetry
    assert!(
        stdout.contains("T-symmetry: none"),
        "major triad should have no T-symmetry; got:\n{stdout}"
    );
}

#[test]
fn prime_form_augmented_triad_symmetric() {
    // Augmented triad {0,4,8} — has T4 and T8 symmetry, inversionally symmetric
    let out = slonimsky()
        .args(["prime-form", "0", "4", "8", "-v"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[0, 4, 8]"),
        "augmented triad prime form should be [0, 4, 8]; got:\n{stdout}"
    );
    // Has transpositional symmetry
    assert!(
        !stdout.contains("T-symmetry: none"),
        "augmented triad should have T-symmetry; got:\n{stdout}"
    );
}

#[test]
fn prime_form_no_input_fails() {
    slonimsky()
        .args(["prime-form"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}
