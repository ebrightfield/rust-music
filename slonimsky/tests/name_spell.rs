use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== name subcommand ====================

#[test]
fn name_c_major_triad_outputs_cmaj() {
    let out = slonimsky()
        .args(["name", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Maj"),
        "C E G should be named as major; got: {stdout}"
    );
    assert!(stdout.contains("C"), "root should be C; got: {stdout}");
}

#[test]
fn name_d_minor_triad() {
    let out = slonimsky()
        .args(["name", "D", "F", "A"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("min") || stdout.contains("Min") || stdout.contains("m"),
        "D F A should be named as minor; got: {stdout}"
    );
    assert!(stdout.contains("D"), "root should be D; got: {stdout}");
}

#[test]
fn name_integer_input() {
    let out = slonimsky()
        .args(["name", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 0,4,7 = C major triad
    assert!(
        stdout.contains("Maj"),
        "0 4 7 should be major; got: {stdout}"
    );
}

#[test]
fn name_dominant_seventh() {
    let out = slonimsky()
        .args(["name", "G", "B", "D", "F"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // G B D F = G7 (dominant seventh)
    assert!(stdout.contains("G"), "root should be G; got: {stdout}");
    assert!(
        stdout.contains("7"),
        "G B D F should contain '7' in name; got: {stdout}"
    );
}

#[test]
fn name_with_explicit_root() {
    let out = slonimsky()
        .args(["name", "A", "C", "E", "--root", "A"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("A"), "root should be A; got: {stdout}");
    // A C E = Am
    assert!(
        stdout.contains("min") || stdout.contains("m"),
        "A C E should be minor; got: {stdout}"
    );
}

#[test]
fn name_verbose_prints_detail_to_stderr() {
    let out = slonimsky()
        .args(["-v", "name", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Verbose mode prints pitch classes and intervals to stderr
    assert!(
        stderr.contains("pitch classes") || stderr.contains("pcs") || stderr.contains("name:"),
        "verbose mode should print detail to stderr; got: {stderr}"
    );
}

#[test]
fn name_no_input_fails() {
    slonimsky()
        .arg("name")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn name_single_pc_produces_output() {
    // Edge case: a single pitch class — should still produce something
    // (may be "unclassified" or error, but should not panic)
    let out = slonimsky()
        .args(["name", "C"])
        .output()
        .expect("command should run");

    // Single PC might fail with "could not identify" — that's acceptable
    // The key assertion: it doesn't panic and either succeeds or exits 1
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "should exit cleanly with 0 or 1, not crash"
    );
}

// ==================== spell subcommand ====================

#[test]
fn spell_c_major_outputs_notes() {
    let out = slonimsky()
        .args(["spell", "C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C major triad = C E G
    assert!(stdout.contains("C"), "should contain C; got: {stdout}");
    assert!(stdout.contains("E"), "should contain E; got: {stdout}");
    assert!(stdout.contains("G"), "should contain G; got: {stdout}");
}

#[test]
fn spell_cmaj7_format_all_shows_three_lines() {
    let out = slonimsky()
        .args(["spell", "Cmaj7", "--format", "all"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Notes:"),
        "all format should have Notes: line; got: {stdout}"
    );
    assert!(
        stdout.contains("PCs:"),
        "all format should have PCs: line; got: {stdout}"
    );
    assert!(
        stdout.contains("Intervals:"),
        "all format should have Intervals: line; got: {stdout}"
    );
}

#[test]
fn spell_cmaj7_notes_has_four_notes() {
    let out = slonimsky()
        .args(["spell", "Cmaj7", "--format", "notes"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let words: Vec<&str> = stdout.split_whitespace().collect();
    assert_eq!(
        words.len(),
        4,
        "Cmaj7 should have 4 notes, got {}: {:?}",
        words.len(),
        words
    );
    assert_eq!(words[0], "C", "root should be C");
}

#[test]
fn spell_am_outputs_a_c_e() {
    let out = slonimsky()
        .args(["spell", "Am"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("A"), "Am should include A; got: {stdout}");
    assert!(stdout.contains("C"), "Am should include C; got: {stdout}");
    assert!(stdout.contains("E"), "Am should include E; got: {stdout}");
}

#[test]
fn spell_dm7_intervals_shows_flat3_flat7() {
    let out = slonimsky()
        .args(["spell", "Dm7", "--format", "intervals"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("R"),
        "should start with root R; got: {stdout}"
    );
    assert!(stdout.contains("b3"), "Dm7 should have b3; got: {stdout}");
    assert!(stdout.contains("b7"), "Dm7 should have b7; got: {stdout}");
    assert!(stdout.contains("5"), "Dm7 should have 5; got: {stdout}");
}

#[test]
fn spell_pcs_format_outputs_integers() {
    let out = slonimsky()
        .args(["spell", "Cmaj7", "--format", "pcs"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Cmaj7 intervals from root: 0 4 7 11
    assert!(stdout.contains("0"), "pcs should include 0; got: {stdout}");
    assert!(stdout.contains("4"), "pcs should include 4; got: {stdout}");
    assert!(stdout.contains("7"), "pcs should include 7; got: {stdout}");
    assert!(
        stdout.contains("11"),
        "pcs should include 11; got: {stdout}"
    );
}

#[test]
fn spell_invalid_symbol_fails() {
    slonimsky()
        .args(["spell", "ZZZZZ"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse chord symbol"));
}

#[test]
fn spell_bad_format_fails() {
    slonimsky()
        .args(["spell", "Cmaj7", "--format", "garbage"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown format"));
}

#[test]
fn spell_verbose_prints_to_stderr() {
    let out = slonimsky()
        .args(["-v", "spell", "Cmaj7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("spell:"),
        "verbose should print 'spell:' to stderr; got: {stderr}"
    );
}

// ==================== spell: tertian interval spelling ====================
// Regression tests for docs/slonimsky-cli-bugs.md §1 — chord tones were chosen
// by "prefer a natural, else match the root's accidental", which spelled Cm7's
// third as D# (an augmented second) instead of Eb.

/// Run `spell <symbol>` and return its trimmed stdout.
fn spell(symbol: &str) -> String {
    let out = slonimsky()
        .args(["spell", symbol])
        .output()
        .expect("command should run");
    assert!(out.status.success(), "spell {symbol} should succeed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn spell_minor_sevenths_are_tertian() {
    assert_eq!(spell("Cm7"), "C Eb G Bb");
    assert_eq!(spell("Abm7"), "Ab Cb Eb Gb");
    assert_eq!(spell("Dbm7"), "Db Fb Ab Cb");
    // A double flat is the correct third here, and preferable to A.
    assert_eq!(spell("Gbm7"), "Gb Bbb Db Fb");
}

#[test]
fn spell_previously_correct_roots_unchanged() {
    assert_eq!(spell("Bbmaj7"), "Bb D F A");
    assert_eq!(spell("Ebmaj7"), "Eb G Bb D");
    assert_eq!(spell("F7"), "F A C Eb");
    assert_eq!(spell("Ebm7"), "Eb Gb Bb Db");
    assert_eq!(spell("Fm7"), "F Ab C Eb");
}

#[test]
fn spell_resolves_ambiguous_intervals_from_context() {
    // Fully-diminished seventh: the top note is a seventh (Bbb), not a sixth.
    assert_eq!(spell("Cdim7"), "C Eb Gb Bbb");
    // Half-diminished keeps a natural b7.
    assert_eq!(spell("Cm7b5"), "C Eb Gb Bb");
    // With a perfect fifth present the tritone is a #11 (F#), not a b5.
    assert_eq!(spell("Cmaj7#11"), "C E F# G B");
}

#[test]
fn spell_enharmonic_roots_differ() {
    assert_eq!(spell("C#m7"), "C# E G# B");
    assert_eq!(spell("Dbm7"), "Db Fb Ab Cb");
}
