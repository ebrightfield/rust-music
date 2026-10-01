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

// ==================== range and spacing controls ====================

#[test]
fn voicings_range_expands_and_bounds_register_placements() {
    slonimsky()
        .args(["voicings", "C", "E", "G", "--range", "C4..C5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Range: C4..C5"))
        .stdout(predicate::str::contains("1 families, 2 voicings total"))
        .stdout(predicate::str::contains("C4 E4 G4"))
        .stdout(predicate::str::contains("E4 G4 C5"))
        .stdout(predicate::str::contains("C3").not())
        .stdout(predicate::str::contains("E5").not());
}

#[test]
fn voicings_spacing_filters_adjacent_intervals() {
    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--min-spacing",
            "5",
            "--max-spacing",
            "8",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adjacent spacing: 5..8 semitones"))
        .stdout(predicate::str::contains("1 families, 1 voicings total"))
        .stdout(predicate::str::contains("intervals: [8, 7]"));
}

#[test]
fn voicings_combines_range_and_spacing_filters() {
    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--range",
            "C3..C6",
            "--min-spacing",
            "7",
            "--max-spacing",
            "9",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 families, 6 voicings total"))
        .stdout(predicate::str::contains("6 placements"));
}

#[test]
fn voicings_rejects_invalid_constraints() {
    slonimsky()
        .args(["voicings", "C", "E", "G", "--range", "C6..C3"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "lower bound C6 exceeds upper bound C3",
        ));

    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--min-spacing",
            "9",
            "--max-spacing",
            "4",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--min-spacing (9) cannot exceed --max-spacing (4)",
        ));
}

// ==================== instrument controls ====================

#[test]
fn voicings_strings_and_tuning_generate_playable_placements() {
    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--tuning",
            "standard",
            "--strings",
            "3",
            "--range",
            "E2..E5",
            "--limit",
            "3",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("10 instrument voicings total"))
        .stdout(predicate::str::contains(
            "Tuning: standard, sounded strings: 3, doubling: forbid",
        ))
        .stdout(predicate::str::contains("Instrument placements"));
}

#[test]
fn voicings_required_doubling_uses_extra_strings() {
    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--tuning",
            "bass-4",
            "--strings",
            "4",
            "--doubling",
            "require",
            "--range",
            "E1..G4",
            "--limit",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("71 instrument voicings total"))
        .stdout(predicate::str::contains(
            "Tuning: bass-4, sounded strings: 4, doubling: require",
        ))
        .stdout(predicate::str::contains("E2 C3 E3 G3"));
}

#[test]
fn voicings_allow_doubling_accepts_extra_strings() {
    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--strings",
            "4",
            "--doubling",
            "allow",
            "--range",
            "E2..E5",
            "--limit",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Tuning: standard, sounded strings: 4, doubling: allow",
        ));
}

#[test]
fn voicings_rejects_incompatible_string_and_doubling_controls() {
    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--strings",
            "4",
            "--doubling",
            "forbid",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--doubling forbid requires --strings to equal the 3 chord tones",
        ));

    slonimsky()
        .args([
            "voicings",
            "C",
            "E",
            "G",
            "--tuning",
            "bass-4",
            "--strings",
            "5",
            "--doubling",
            "allow",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--strings (5) exceeds tuning 'bass-4' string count (4)",
        ));
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
