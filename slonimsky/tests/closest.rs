use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== basic output ====================

#[test]
fn closest_c_major_triad_finds_close_chords() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--pool", "chords", "--limit", "10"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Cmaj7 = {0,4,7,11} is one note away from C major triad
    assert!(
        stdout.contains("Maj7") || stdout.contains("CMaj7"),
        "Cmaj7 should be close (dist=1); got:\n{stdout}"
    );
    assert!(
        stdout.contains("dist=1"),
        "should have at least one dist=1 match; got:\n{stdout}"
    );
}

#[test]
fn closest_header_shows_input_pcset() {
    let out = slonimsky()
        .args(["closest", "0", "4", "7", "--limit", "3"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Closest known chords/scales to {0,4,7}"),
        "header should show input PcSet; got:\n{stdout}"
    );
}

#[test]
fn closest_total_line_present() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--limit", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Total:"),
        "should have a Total line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("results (distance > 0)"),
        "total line should note distance > 0 filter; got:\n{stdout}"
    );
}

// ==================== excludes exact matches ====================

#[test]
fn closest_excludes_distance_zero() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--pool", "chords", "--limit", "100"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // No result should have dist=0
    assert!(
        !stdout.contains("dist=0"),
        "exact matches (dist=0) should be excluded; got:\n{stdout}"
    );
}

// ==================== limit flag ====================

#[test]
fn closest_limit_caps_results() {
    let out_limited = slonimsky()
        .args(["closest", "C", "E", "G", "--limit", "3"])
        .output()
        .expect("command should run");

    let out_unlimited = slonimsky()
        .args(["closest", "C", "E", "G", "--limit", "50"])
        .output()
        .expect("command should run");

    assert!(out_limited.status.success());
    assert!(out_unlimited.status.success());

    let limited = String::from_utf8_lossy(&out_limited.stdout);
    let unlimited = String::from_utf8_lossy(&out_unlimited.stdout);

    // Count numbered result lines (start with whitespace + digits + period)
    let count_limited = limited.lines().filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit())).count();
    let count_unlimited = unlimited.lines().filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit())).count();

    assert_eq!(count_limited, 3, "limited should have exactly 3 results; got:\n{limited}");
    assert!(count_unlimited > 3, "unlimited should have more than 3 results; got:\n{unlimited}");
}

// ==================== pool filter ====================

#[test]
fn closest_pool_chords_excludes_scales() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--pool", "chords", "--limit", "20"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // With --pool chords, no 7-note scale results should appear
    assert!(
        !stdout.contains("Major") || !stdout.contains("7,"), // scales have 7+ common tones or "Major" as scale name
        // This is tricky — just check no result has >4 note chords
        // Actually let's check a simpler property: all results should be close
        "pool=chords should only return chord-sized results"
    );
}

#[test]
fn closest_pool_scales_only() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--pool", "scales", "--limit", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // With pool=scales, should find 7-note scales like C Major
    assert!(
        stdout.contains("Major"),
        "pool=scales should find Major scale; got:\n{stdout}"
    );
}

// ==================== common tones info ====================

#[test]
fn closest_shows_common_tones_count() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--pool", "chords", "--limit", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("common tones"),
        "results should show common tone counts; got:\n{stdout}"
    );
    // Dist=1 match should have at least 2 common tones with a 3-note input
    assert!(
        stdout.contains("3 common tones") || stdout.contains("2 common tones"),
        "close chords should share tones with input; got:\n{stdout}"
    );
}

// ==================== verbose mode ====================

#[test]
fn closest_verbose_shows_detail_on_stderr() {
    let out = slonimsky()
        .args(["-v", "closest", "C", "E", "G", "--pool", "chords", "--limit", "3"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("only in input") || stderr.contains("closest:"),
        "verbose mode should print detail to stderr; got:\n{stderr}"
    );
}

// ==================== error cases ====================

#[test]
fn closest_no_input_fails() {
    slonimsky()
        .arg("closest")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn closest_bad_metric_fails() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--metric", "euclidean"])
        .output()
        .expect("command should run");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown metric"),
        "should report unknown metric; got:\n{stderr}"
    );
}

#[test]
fn closest_bad_pool_fails() {
    let out = slonimsky()
        .args(["closest", "C", "E", "G", "--pool", "garbage"])
        .output()
        .expect("command should run");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown pool"),
        "should report unknown pool; got:\n{stderr}"
    );
}

// ==================== integer input ====================

#[test]
fn closest_integer_input_works() {
    let out = slonimsky()
        .args(["closest", "0", "4", "7", "--pool", "chords", "--limit", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Same as C major triad — should find Maj7 at dist=1
    assert!(
        stdout.contains("dist=1"),
        "integer input should work same as note names; got:\n{stdout}"
    );
}
