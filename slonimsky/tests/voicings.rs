use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== triad voicings ====================

#[test]
fn voicings_c_major_triad_has_2_families_6_voicings() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("2 families, 6 voicings total"),
        "triad should have 2 families × 3 inversions = 6; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Total: 6 voicings"),
        "total line should show 6; got:\n{stdout}"
    );
}

#[test]
fn voicings_header_shows_input_notes_and_pcset() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Voicings for:"),
        "header should show 'Voicings for:'; got:\n{stdout}"
    );
    assert!(
        stdout.contains("0, 4, 7"),
        "header should show PcSet {{0, 4, 7}}; got:\n{stdout}"
    );
}

#[test]
fn voicings_triad_shows_intervals() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Close-position triad in root position has intervals [4, 3]
    assert!(
        stdout.contains("[4, 3]"),
        "root position close voicing should have [4, 3]; got:\n{stdout}"
    );
    assert!(
        stdout.contains("intervals:"),
        "output should label intervals; got:\n{stdout}"
    );
}

// ==================== seventh chord voicings ====================

#[test]
fn voicings_cmaj7_has_6_families_24_voicings() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G", "B"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("6 families, 24 voicings total"),
        "7th chord should have 6 families × 4 inversions = 24; got:\n{stdout}"
    );
}

#[test]
fn voicings_cmaj7_shows_quality() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G", "B"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Quality:"),
        "should show chord quality; got:\n{stdout}"
    );
}

// ==================== limit flag ====================

#[test]
fn voicings_limit_caps_output() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G", "--limit", "2"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("showing 2/6"),
        "limit=2 should show 2 of 6; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Total: 2 voicings"),
        "total should be capped at 2; got:\n{stdout}"
    );
}

#[test]
fn voicings_without_limit_shows_all() {
    let limited = slonimsky()
        .args(["voicings", "C", "E", "G", "--limit", "2"])
        .output()
        .expect("command should run");
    let unlimited = slonimsky()
        .args(["voicings", "C", "E", "G"])
        .output()
        .expect("command should run");

    let limited_out = String::from_utf8_lossy(&limited.stdout);
    let unlimited_out = String::from_utf8_lossy(&unlimited.stdout);
    assert!(
        unlimited_out.len() > limited_out.len(),
        "unlimited output should be longer than limited"
    );
}

// ==================== integer input ====================

#[test]
fn voicings_integer_input_works() {
    let out = slonimsky()
        .args(["voicings", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("2 families, 6 voicings total"),
        "integer input 0 4 7 should produce same result as C E G; got:\n{stdout}"
    );
}

// ==================== verbose flag ====================

#[test]
fn voicings_verbose_shows_span_and_inversion() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G", "-v"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("span:"),
        "verbose should show span info; got:\n{stdout}"
    );
    assert!(
        stdout.contains("inversion"),
        "verbose should show inversion number; got:\n{stdout}"
    );
}

// ==================== family structure ====================

#[test]
fn voicings_triad_shows_family_headers() {
    let out = slonimsky()
        .args(["voicings", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Family 1") && stdout.contains("Family 2"),
        "should show Family 1 and Family 2 headers; got:\n{stdout}"
    );
    assert!(
        stdout.contains("3 inversions"),
        "each triad family should show 3 inversions; got:\n{stdout}"
    );
}

// ==================== error cases ====================

#[test]
fn voicings_two_notes_fails() {
    slonimsky()
        .args(["voicings", "C", "E"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("3- or 4-note"));
}

#[test]
fn voicings_five_notes_fails() {
    slonimsky()
        .args(["voicings", "C", "D", "E", "G", "A"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("3- or 4-note"));
}

#[test]
fn voicings_no_input_fails() {
    slonimsky()
        .args(["voicings"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}
