use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== common-tones subcommand ====================

#[test]
fn common_tones_c_major_a_minor_shares_c_e() {
    let out = slonimsky()
        .args(["common-tones", "C,E,G", "A,C,E"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should find C and E as common tones
    assert!(stdout.contains("C"), "should contain C; got:\n{stdout}");
    assert!(stdout.contains("E"), "should contain E; got:\n{stdout}");
    assert!(
        stdout.contains("Count: 2"),
        "should have 2 common tones; got:\n{stdout}"
    );
}

#[test]
fn common_tones_c_major_d_minor_empty() {
    let out = slonimsky()
        .args(["common-tones", "C,E,G", "D,F,A"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("(empty)"),
        "C major and D minor share no tones; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Count: 0"),
        "count should be 0; got:\n{stdout}"
    );
}

#[test]
fn common_tones_three_sets_intersection() {
    // C major {C,E,G}, A minor {A,C,E}, Cmaj7 {C,E,G,B}
    // Three-way intersection = {C, E}
    let out = slonimsky()
        .args(["common-tones", "C,E,G", "A,C,E", "C,E,G,B"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Count: 2"),
        "three-way intersection should have 2; got:\n{stdout}"
    );
    // Should label all three input sets
    assert!(
        stdout.contains("Set 1:"),
        "should show Set 1; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Set 2:"),
        "should show Set 2; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Set 3:"),
        "should show Set 3; got:\n{stdout}"
    );
}

#[test]
fn common_tones_integer_input() {
    // {0,4,7} ∩ {0,4,9} = {0,4} = {C,E}
    let out = slonimsky()
        .args(["common-tones", "0,4,7", "0,4,9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Count: 2"),
        "should have 2 common tones; got:\n{stdout}"
    );
}

#[test]
fn common_tones_verbose_pairwise_with_three_sets() {
    let out = slonimsky()
        .args(["-v", "common-tones", "C,E,G", "A,C,E", "C,E,G,B"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Verbose mode with 3+ sets should show pairwise intersections on stderr
    assert!(
        stderr.contains("Pairwise"),
        "verbose with 3 sets should show pairwise; got stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("Set 1") && stderr.contains("Set 2"),
        "should show set pair labels; got stderr:\n{stderr}"
    );
}

#[test]
fn common_tones_identical_sets_full_overlap() {
    let out = slonimsky()
        .args(["common-tones", "C,E,G", "C,E,G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Count: 3"),
        "identical sets should share all 3; got:\n{stdout}"
    );
}

#[test]
fn common_tones_single_set_fails() {
    slonimsky()
        .args(["common-tones", "C,E,G"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 2 sets"));
}

#[test]
fn common_tones_no_input_fails() {
    slonimsky()
        .args(["common-tones"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ==================== orbits subcommand ====================

#[test]
fn orbits_dim7_shows_t3_t6() {
    let out = slonimsky()
        .args(["orbits", "0", "3", "6", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("T3"),
        "dim7 should have T3 symmetry; got:\n{stdout}"
    );
    assert!(
        stdout.contains("T6"),
        "dim7 should have T6 symmetry; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Inversionally symmetric: yes"),
        "dim7 should be inversionally symmetric; got:\n{stdout}"
    );
}

#[test]
fn orbits_major_triad_no_symmetry() {
    let out = slonimsky()
        .args(["orbits", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("none"),
        "major triad should have no T-symmetry; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Inversionally symmetric: no"),
        "major triad should not be I-symmetric; got:\n{stdout}"
    );
    // Should show inverted form
    assert!(
        stdout.contains("Inverted form:"),
        "should show the inverted form; got:\n{stdout}"
    );
}

#[test]
fn orbits_augmented_triad_t4() {
    let out = slonimsky()
        .args(["orbits", "C", "E", "Ab"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("T4"),
        "augmented triad should have T4 symmetry; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Inversionally symmetric: yes"),
        "augmented triad should be I-symmetric; got:\n{stdout}"
    );
}

#[test]
fn orbits_whole_tone_t2_t4_t6() {
    let out = slonimsky()
        .args(["orbits", "0", "2", "4", "6", "8", "10"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("T2"),
        "whole-tone should have T2; got:\n{stdout}"
    );
    assert!(
        stdout.contains("T4"),
        "whole-tone should have T4; got:\n{stdout}"
    );
    assert!(
        stdout.contains("T6"),
        "whole-tone should have T6; got:\n{stdout}"
    );
}

#[test]
fn orbits_type_transpositional_only() {
    let out = slonimsky()
        .args(["orbits", "0", "3", "6", "9", "--type", "transpositional"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Transpositional symmetries:"),
        "should show T-symmetry section; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("Inversionally symmetric"),
        "should NOT show I-symmetry section when --type transpositional; got:\n{stdout}"
    );
}

#[test]
fn orbits_type_inversional_only() {
    let out = slonimsky()
        .args(["orbits", "0", "3", "6", "9", "--type", "inversional"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("Transpositional symmetries:"),
        "should NOT show T-symmetry section when --type inversional; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Inversionally symmetric"),
        "should show I-symmetry section; got:\n{stdout}"
    );
}

#[test]
fn orbits_verbose_per_pc_detail() {
    let out = slonimsky()
        .args(["-v", "orbits", "0", "3", "6", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Verbose should show per-PC detail lines like "  Pc0: {T3, T6}"
    assert!(
        stdout.contains("Pc0:"),
        "verbose should show per-PC detail; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Pc3:"),
        "verbose should show Pc3; got:\n{stdout}"
    );
}

#[test]
fn orbits_bad_type_fails() {
    slonimsky()
        .args(["orbits", "0", "3", "6", "9", "--type", "garbage"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown symmetry type"));
}

#[test]
fn orbits_no_input_fails() {
    slonimsky()
        .args(["orbits"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn orbits_shows_pcset_header() {
    let out = slonimsky()
        .args(["orbits", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("PcSet:"),
        "output should begin with PcSet label; got:\n{stdout}"
    );
}
