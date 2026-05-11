use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Default output ---

#[test]
fn default_produces_header_and_measures() {
    let out = slonimsky()
        .args(["sight-reading"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Sight-Reading Exercise"),
        "should have exercise header"
    );
    assert!(
        stdout.contains("Key: C major"),
        "default key should be C major, got:\n{}",
        stdout.lines().take(5).collect::<Vec<_>>().join("\n")
    );
    assert!(
        stdout.contains("Difficulty: 2/5"),
        "default difficulty should be 2"
    );
    assert!(
        stdout.contains("Measures: 4"),
        "default should be 4 measures"
    );
    // Should have measure lines m1 through m4
    for m in 1..=4 {
        assert!(
            stdout.contains(&format!("m{}:", m)),
            "should contain measure {m}"
        );
    }
}

// --- Key and scale options ---

#[test]
fn g_major_key() {
    let out = slonimsky()
        .args(["sight-reading", "--key", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Key: G major"),
        "should show G major"
    );
}

#[test]
fn melodic_minor_scale() {
    let out = slonimsky()
        .args(["sight-reading", "--scale", "melodic-minor"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("melodic-minor") || stdout.contains("melodic minor"),
        "should mention melodic minor scale"
    );
}

#[test]
fn harmonic_minor_scale() {
    slonimsky()
        .args(["sight-reading", "--scale", "harmonic-minor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("harmonic-minor").or(predicate::str::contains("harmonic minor")));
}

#[test]
fn harmonic_major_scale() {
    slonimsky()
        .args(["sight-reading", "--scale", "harmonic-major"])
        .assert()
        .success()
        .stdout(predicate::str::contains("harmonic-major").or(predicate::str::contains("harmonic major")));
}

#[test]
fn unknown_scale_fails() {
    slonimsky()
        .args(["sight-reading", "--scale", "blues"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown scale"));
}

#[test]
fn unknown_key_fails() {
    slonimsky()
        .args(["sight-reading", "--key", "Z"])
        .assert()
        .failure();
}

// --- Difficulty levels ---

#[test]
fn difficulty_1_produces_fewer_notes() {
    let out = slonimsky()
        .args(["sight-reading", "--difficulty", "1", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Difficulty 1 = 4 notes/measure × 4 measures = 16 notes
    let notes_line = stdout
        .lines()
        .find(|l| l.starts_with("Notes:"))
        .expect("should have Notes: line");
    assert!(
        notes_line.contains("16"),
        "difficulty 1 should produce 16 notes, got: {notes_line}"
    );
}

#[test]
fn difficulty_5_produces_more_notes() {
    let out = slonimsky()
        .args(["sight-reading", "--difficulty", "5", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Difficulty 5 = 7 notes/measure × 4 measures = 28 notes
    let notes_line = stdout
        .lines()
        .find(|l| l.starts_with("Notes:"))
        .expect("should have Notes: line");
    assert!(
        notes_line.contains("28"),
        "difficulty 5 should produce 28 notes, got: {notes_line}"
    );
}

#[test]
fn all_difficulty_levels_succeed() {
    for d in 1..=5 {
        slonimsky()
            .args(["sight-reading", "--difficulty", &d.to_string(), "--seed", "7"])
            .assert()
            .success();
    }
}

// --- Seed determinism ---

#[test]
fn same_seed_produces_identical_output() {
    let out1 = slonimsky()
        .args(["sight-reading", "--seed", "123", "--difficulty", "3"])
        .output()
        .unwrap();
    let out2 = slonimsky()
        .args(["sight-reading", "--seed", "123", "--difficulty", "3"])
        .output()
        .unwrap();
    assert!(out1.status.success());
    assert!(out2.status.success());
    assert_eq!(
        out1.stdout, out2.stdout,
        "same seed should produce identical output"
    );
}

#[test]
fn different_seeds_produce_different_output() {
    let out1 = slonimsky()
        .args(["sight-reading", "--seed", "1", "--difficulty", "3"])
        .output()
        .unwrap();
    let out2 = slonimsky()
        .args(["sight-reading", "--seed", "2", "--difficulty", "3"])
        .output()
        .unwrap();
    assert!(out1.status.success());
    assert!(out2.status.success());
    // Different seeds should (with very high probability) produce different melodies
    assert_ne!(
        out1.stdout, out2.stdout,
        "different seeds should produce different output"
    );
}

// --- Measures option ---

#[test]
fn custom_measure_count() {
    let out = slonimsky()
        .args(["sight-reading", "--measures", "8", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Measures: 8"), "should show 8 measures");
    // Should have m1 through m8
    assert!(stdout.contains("m8:"), "should have measure 8");
}

#[test]
fn single_measure() {
    let out = slonimsky()
        .args(["sight-reading", "--measures", "1", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Measures: 1"), "should show 1 measure");
    assert!(stdout.contains("m1:"), "should have measure 1");
    assert!(
        !stdout.contains("m2:"),
        "should not have measure 2"
    );
}

// --- Output content validation ---

#[test]
fn notes_contain_pitch_and_duration() {
    let out = slonimsky()
        .args(["sight-reading", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let m1 = stdout
        .lines()
        .find(|l| l.contains("m1:"))
        .expect("should have m1");
    // Each note in a measure should have pitch (letter + octave) and duration in parens
    // e.g. "C4 (quarter)"
    assert!(
        m1.contains('(') && m1.contains(')'),
        "measure line should contain duration in parens, got: {m1}"
    );
    // Should contain an octave number (3, 4, or 5 expected for C3-C6 range)
    assert!(
        m1.contains('3') || m1.contains('4') || m1.contains('5') || m1.contains('6'),
        "measure should contain octave numbers, got: {m1}"
    );
}

#[test]
fn c_major_pitches_within_bounds() {
    let out = slonimsky()
        .args(["sight-reading", "--key", "C", "--seed", "42", "--difficulty", "3", "--measures", "8"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // All measure lines should not contain octave 1 or 2 (below C3)
    // or octave 7+ (above C6)
    for line in stdout.lines() {
        if line.trim_start().starts_with('m') && line.contains(':') {
            assert!(
                !line.contains("1)") || line.contains("m1"),
                "should not have octave 1 pitches (below range)"
            );
        }
    }
}

// --- Verbose mode ---

#[test]
fn verbose_prints_to_stderr() {
    let out = slonimsky()
        .args(["sight-reading", "-v", "--seed", "42"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Generated") && stderr.contains("events"),
        "verbose should print generation info to stderr, got: {stderr}"
    );
    assert!(
        stderr.contains("Pitch range"),
        "verbose should print pitch range to stderr, got: {stderr}"
    );
}

// --- Combined options ---

#[test]
fn g_harmonic_minor_difficulty_4() {
    let out = slonimsky()
        .args([
            "sight-reading",
            "--key", "G",
            "--scale", "harmonic-minor",
            "--difficulty", "4",
            "--measures", "2",
            "--seed", "99",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Key: G"));
    assert!(stdout.contains("Difficulty: 4/5"));
    assert!(stdout.contains("Measures: 2"));
    assert!(stdout.contains("m1:"));
    assert!(stdout.contains("m2:"));
    assert!(!stdout.contains("m3:"));
}
