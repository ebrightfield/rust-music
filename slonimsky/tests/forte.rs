use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== forte subcommand ====================

#[test]
fn forte_major_triad_is_3_11() {
    let out = slonimsky()
        .args(["forte", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("3-11"),
        "major triad should be Forte 3-11; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Prime form:"),
        "should show prime form label; got:\n{stdout}"
    );
    assert!(
        stdout.contains("[0, 3, 7]"),
        "major triad prime form is [0,3,7]; got:\n{stdout}"
    );
}

#[test]
fn forte_minor_triad_same_class_as_major() {
    let major = slonimsky()
        .args(["forte", "C", "E", "G"])
        .output()
        .expect("command should run");
    let minor = slonimsky()
        .args(["forte", "D", "F", "A"])
        .output()
        .expect("command should run");

    let major_out = String::from_utf8_lossy(&major.stdout);
    let minor_out = String::from_utf8_lossy(&minor.stdout);

    // Both should be 3-11
    assert!(major_out.contains("3-11"));
    assert!(minor_out.contains("3-11"));
    // Both should have the same prime form
    assert!(major_out.contains("[0, 3, 7]"));
    assert!(minor_out.contains("[0, 3, 7]"));
}

#[test]
fn forte_dim7_is_4_28() {
    let out = slonimsky()
        .args(["forte", "0", "3", "6", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("4-28"),
        "dim7 should be Forte 4-28; got:\n{stdout}"
    );
    assert!(
        stdout.contains("[0, 3, 6, 9]"),
        "dim7 prime form is [0,3,6,9]; got:\n{stdout}"
    );
}

#[test]
fn forte_augmented_is_3_12() {
    let out = slonimsky()
        .args(["forte", "C", "E", "Ab"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("3-12"),
        "augmented triad should be Forte 3-12; got:\n{stdout}"
    );
    assert!(stdout.contains("[0, 4, 8]"));
}

#[test]
fn forte_dom7_is_4_27() {
    let out = slonimsky()
        .args(["forte", "0", "4", "7", "10"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("4-27"),
        "dom7 should be Forte 4-27; got:\n{stdout}"
    );
}

#[test]
fn forte_shows_pcset_label() {
    let out = slonimsky()
        .args(["forte", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("PcSet:"),
        "should show PcSet header; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Forte:"),
        "should show Forte header; got:\n{stdout}"
    );
}

#[test]
fn forte_verbose_shows_iv_and_symmetry() {
    let out = slonimsky()
        .args(["forte", "0", "3", "6", "9", "-v"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Verbose should show interval vector
    assert!(
        stdout.contains("IV:"),
        "verbose should show interval vector; got:\n{stdout}"
    );
    // Dim7 has T3 and T6 symmetry
    assert!(
        stdout.contains("T3"),
        "dim7 verbose should show T3 symmetry; got:\n{stdout}"
    );
    assert!(
        stdout.contains("T6"),
        "dim7 verbose should show T6 symmetry; got:\n{stdout}"
    );
    // Dim7 is inversionally symmetric
    assert!(
        stdout.contains("I-symmetry: yes"),
        "dim7 should be inversionally symmetric; got:\n{stdout}"
    );
}

#[test]
fn forte_verbose_z_relation_shown() {
    // 4-Z15: {0,1,4,6}
    let out = slonimsky()
        .args(["forte", "0", "1", "4", "6", "-v"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("4-Z15"),
        "should show Forte 4-Z15; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Z-relation"),
        "Z-related set should show Z-relation line; got:\n{stdout}"
    );
    assert!(
        stdout.contains("4-Z29"),
        "4-Z15's Z-partner is 4-Z29; got:\n{stdout}"
    );
}

#[test]
fn forte_pentatonic_is_5_35() {
    // Pentatonic now in table (cardinality 5 added)
    let out = slonimsky()
        .args(["forte", "0", "2", "4", "7", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("5-35"),
        "pentatonic should be 5-35; got:\n{stdout}"
    );
}

#[test]
fn forte_major_scale_is_7_35() {
    // Major scale {0,2,4,5,7,9,11} → 7-35
    let out = slonimsky()
        .args(["forte", "0", "2", "4", "5", "7", "9", "11"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("7-35"),
        "major scale should be 7-35; got:\n{stdout}"
    );
}

#[test]
fn forte_harmonic_minor_is_7_32() {
    // Harmonic minor {0,2,3,5,7,8,11} → 7-32
    let out = slonimsky()
        .args(["forte", "0", "2", "3", "5", "7", "8", "11"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("7-32"),
        "harmonic minor should be 7-32; got:\n{stdout}"
    );
}

#[test]
fn forte_octatonic_is_8_28() {
    // Octatonic scale {0,1,3,4,6,7,9,10} → 8-28
    let out = slonimsky()
        .args(["forte", "0", "1", "3", "4", "6", "7", "9", "10"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("8-28"),
        "octatonic should be 8-28; got:\n{stdout}"
    );
}

#[test]
fn forte_nonachord_is_9_12() {
    // {0,1,2,4,5,6,8,9,10} → 9-12
    let out = slonimsky()
        .args(["forte", "0", "1", "2", "4", "5", "6", "8", "9", "10"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("9-12"),
        "should be 9-12; got:\n{stdout}"
    );
}

#[test]
fn forte_decachord_is_10_1() {
    let out = slonimsky()
        .args(["forte", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("10-1"),
        "decachord should be Forte 10-1; got:\n{stdout}"
    );
}

#[test]
fn forte_chromatic_aggregate_is_12_1() {
    let out = slonimsky()
        .args(["forte", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("12-1"),
        "chromatic aggregate should be Forte 12-1; got:\n{stdout}"
    );
}

#[test]
fn forte_chromatic_hexachord_is_6_1() {
    let out = slonimsky()
        .args(["forte", "0", "1", "2", "3", "4", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("6-1"),
        "chromatic hexachord should be 6-1; got:\n{stdout}"
    );
}

#[test]
fn forte_whole_tone_scale_is_6_35() {
    let out = slonimsky()
        .args(["forte", "0", "2", "4", "6", "8", "10"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("6-35"),
        "whole-tone scale should be 6-35; got:\n{stdout}"
    );
}

#[test]
fn forte_tritone_is_2_6() {
    let out = slonimsky()
        .args(["forte", "0", "6"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("2-6"),
        "tritone should be Forte 2-6; got:\n{stdout}"
    );
}

#[test]
fn forte_single_pc_is_1_1() {
    let out = slonimsky()
        .args(["forte", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("1-1"),
        "single PC should be Forte 1-1; got:\n{stdout}"
    );
}

#[test]
fn forte_enharmonic_equivalence() {
    // C Eb Gb A (enharmonic dim7) should match 0 3 6 9
    let enharmonic = slonimsky()
        .args(["forte", "C", "Eb", "Gb", "A"])
        .output()
        .expect("command should run");
    let integer = slonimsky()
        .args(["forte", "0", "3", "6", "9"])
        .output()
        .expect("command should run");

    let enh_out = String::from_utf8_lossy(&enharmonic.stdout);
    let int_out = String::from_utf8_lossy(&integer.stdout);

    // Both should be 4-28
    assert!(enh_out.contains("4-28"), "enharmonic dim7 should be 4-28; got:\n{enh_out}");
    assert!(int_out.contains("4-28"), "integer dim7 should be 4-28; got:\n{int_out}");
}

#[test]
fn forte_no_input_fails() {
    slonimsky()
        .arg("forte")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn forte_verbose_major_triad_no_t_symmetry() {
    let out = slonimsky()
        .args(["forte", "C", "E", "G", "-v"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("T-symmetry: none"),
        "major triad has no T-symmetry; got:\n{stdout}"
    );
    assert!(
        stdout.contains("I-symmetry: no"),
        "major triad is not inversionally symmetric; got:\n{stdout}"
    );
}
