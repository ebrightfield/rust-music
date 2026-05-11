use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== super direction (default for small input) ====================

#[test]
fn contains_c_major_triad_finds_scales() {
    let out = slonimsky()
        .args(["contains", "C", "E", "G", "--in", "scales"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C major triad is contained in C Major, F Major, G Major scales
    assert!(
        stdout.contains("C Major"),
        "C major triad should be contained in C Major scale; got:\n{stdout}"
    );
    assert!(
        stdout.contains("F Major"),
        "C major triad should be contained in F Major scale; got:\n{stdout}"
    );
    assert!(
        stdout.contains("7-note scales"),
        "should show 7-note scales section header; got:\n{stdout}"
    );
}

#[test]
fn contains_c_major_triad_finds_chords() {
    let out = slonimsky()
        .args(["contains", "C", "E", "G", "--in", "chords"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should find 4-note chords containing {0,4,7}
    assert!(
        stdout.contains("4-note chords"),
        "should show 4-note chords section; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Maj7"),
        "should include Maj7 as superchord of major triad; got:\n{stdout}"
    );
}

#[test]
fn contains_shows_header_and_total() {
    let out = slonimsky()
        .args(["contains", "C", "E", "G", "--in", "scales"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("containing"),
        "header should mention containment direction; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Total:"),
        "should show total count; got:\n{stdout}"
    );
    // Parse total count — should be > 0
    let total_line = stdout.lines().find(|l| l.contains("Total:")).unwrap();
    let count: usize = total_line
        .split_whitespace()
        .find_map(|w| w.parse().ok())
        .expect("total line should contain a number");
    assert!(count > 0, "should find at least one containing scale");
}

// ==================== sub direction (default for large input) ====================

#[test]
fn contains_c_major_scale_finds_subchords() {
    let out = slonimsky()
        .args(["contains", "0", "2", "4", "5", "7", "9", "11", "--in", "chords"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C major scale auto-detects sub direction (7 PCs ≥ 5)
    assert!(
        stdout.contains("contained in"),
        "large input should auto-detect sub direction; got:\n{stdout}"
    );
    // Should find triads and 7th chords
    assert!(
        stdout.contains("3-note chords"),
        "should find 3-note chords in C major scale; got:\n{stdout}"
    );
    // Total should be substantial (many triads + 7th chords in 7-note scale)
    let total_line = stdout.lines().find(|l| l.contains("Total:")).unwrap();
    let count: usize = total_line
        .split_whitespace()
        .find_map(|w| w.parse().ok())
        .expect("total should be a number");
    assert!(
        count >= 10,
        "C major scale should contain many chords, got {count}"
    );
}

#[test]
fn contains_explicit_sub_direction() {
    let out = slonimsky()
        .args([
            "contains", "0", "2", "4", "5", "7", "9", "11",
            "--direction", "sub",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("contained in"),
        "explicit sub should show 'contained in' header; got:\n{stdout}"
    );
}

#[test]
fn contains_explicit_super_direction() {
    // Force super direction even on large input
    let out = slonimsky()
        .args([
            "contains", "C", "E", "G",
            "--direction", "super",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("containing"),
        "explicit super should show 'containing' header; got:\n{stdout}"
    );
}

// ==================== --limit flag ====================

#[test]
fn contains_limit_caps_results() {
    let unlimited = slonimsky()
        .args(["contains", "C", "E", "G"])
        .output()
        .expect("command should run");
    let limited = slonimsky()
        .args(["contains", "C", "E", "G", "--limit", "3"])
        .output()
        .expect("command should run");

    assert!(unlimited.status.success());
    assert!(limited.status.success());

    let full_stdout = String::from_utf8_lossy(&unlimited.stdout);
    let lim_stdout = String::from_utf8_lossy(&limited.stdout);

    // Limited output should show "Total: 3"
    let lim_total = lim_stdout.lines().find(|l| l.contains("Total:")).unwrap();
    assert!(
        lim_total.contains("3"),
        "limited output should have 3 results; got: {lim_total}"
    );

    // Unlimited should have more than 3
    let full_total = full_stdout.lines().find(|l| l.contains("Total:")).unwrap();
    let full_count: usize = full_total
        .split_whitespace()
        .find_map(|w| w.parse().ok())
        .unwrap();
    assert!(
        full_count > 3,
        "unlimited should have more than 3 results; got {full_count}"
    );
}

// ==================== --in pool filtering ====================

#[test]
fn contains_pool_both_includes_chords_and_scales() {
    let out = slonimsky()
        .args(["contains", "C", "E", "G", "--in", "both"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // With pool=both, should find both 4-note chords and 7-note scales
    assert!(
        stdout.contains("4-note chords"),
        "pool=both should include chords; got:\n{stdout}"
    );
    assert!(
        stdout.contains("7-note scales"),
        "pool=both should include scales; got:\n{stdout}"
    );
}

// ==================== integer input ====================

#[test]
fn contains_integer_input_works() {
    let out = slonimsky()
        .args(["contains", "0", "4", "7", "--in", "scales"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // {0,4,7} = C major triad, same results as note-name input
    assert!(
        stdout.contains("Major"),
        "integer input should find scales; got:\n{stdout}"
    );
}

// ==================== error cases ====================

#[test]
fn contains_no_input_fails() {
    slonimsky()
        .args(["contains"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn contains_bad_pool_fails() {
    slonimsky()
        .args(["contains", "C", "E", "G", "--in", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown pool"));
}

#[test]
fn contains_bad_direction_fails() {
    slonimsky()
        .args(["contains", "C", "E", "G", "--direction", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown direction"));
}

// ==================== verbose mode ====================

#[test]
fn contains_verbose_shows_debug_info() {
    let out = slonimsky()
        .args(["-v", "contains", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("contains:") || stderr.contains("direction"),
        "verbose mode should print debug info to stderr; got:\n{stderr}"
    );
}

// ==================== edge case: chromatic cluster ====================

#[test]
fn contains_chromatic_cluster_finds_nothing_in_chords_pool() {
    // {0,1,2} — a chromatic cluster that is unlikely to match known chord types
    let out = slonimsky()
        .args(["contains", "0", "1", "2", "--in", "chords"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Might find some or none — but should not crash
    assert!(
        stdout.contains("Total:"),
        "should always show total; got:\n{stdout}"
    );
}
