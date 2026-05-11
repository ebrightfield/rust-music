use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Success: text output ---

#[test]
fn analyze_c_major_i_vi_ii_v_estimates_key() {
    let out = slonimsky()
        .args(["analyze", "C,E,G", "A,C,E", "D,F,A", "G,B,D"])
        .output()
        .expect("command should run");

    assert!(out.status.success(), "exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Key should be estimated as C major with full confidence
    assert!(
        stdout.contains("C major"),
        "should identify C major, got:\n{stdout}"
    );
    assert!(
        stdout.contains("estimated"),
        "should say 'estimated' when --key not given"
    );
    assert!(
        stdout.contains("7/7"),
        "all 7 diatonic PCs present → confidence 7/7"
    );
}

#[test]
fn analyze_shows_roman_numerals() {
    let out = slonimsky()
        .args(["analyze", "C,E,G", "A,C,E", "D,F,A", "G,B,D"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Roman numeral column should contain these for I-vi-ii-V
    assert!(stdout.contains("I"), "should show I for C major triad");
    assert!(stdout.contains("vi"), "should show vi for A minor triad");
    assert!(stdout.contains("ii"), "should show ii for D minor triad");
    assert!(stdout.contains("V"), "should show V for G major triad");
}

#[test]
fn analyze_shows_common_tones() {
    let out = slonimsky()
        .args(["analyze", "C,E,G", "A,C,E"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C and E are common tones between Cmaj and Am
    assert!(
        stdout.contains("Common tones"),
        "should include common tones section"
    );
}

#[test]
fn analyze_shows_voice_leading_cost() {
    let out = slonimsky()
        .args(["analyze", "C,E,G", "F,A,C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Voice-leading cost"),
        "should include L1 cost section"
    );
    assert!(
        stdout.contains("total:"),
        "should show total cost"
    );
}

#[test]
fn analyze_explicit_key() {
    let out = slonimsky()
        .args(["analyze", "--key", "G", "--scale", "major", "G,B,D", "C,E,G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should show G major without "estimated"
    assert!(stdout.contains("G major"), "should show G major key");
    assert!(
        !stdout.contains("estimated"),
        "should NOT say 'estimated' when --key is explicit"
    );
}

#[test]
fn analyze_explicit_key_minor() {
    let out = slonimsky()
        .args([
            "analyze",
            "--key", "A",
            "--scale", "natural-minor",
            "A,C,E", "D,F,A",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("A") && stdout.contains("natural"),
        "should reflect A natural minor key: {stdout}"
    );
}

// --- Success: JSON output ---

#[test]
fn analyze_json_is_valid() {
    let out = slonimsky()
        .args(["analyze", "--format", "json", "C,E,G", "F,A,C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Validate JSON structure
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output must be valid JSON");

    assert!(parsed["key"]["root"].is_string(), "key.root must be a string");
    assert!(parsed["key"]["scale"].is_string(), "key.scale must be a string");
    assert!(parsed["key"]["confidence"].is_number(), "key.confidence must be a number");
    assert!(parsed["chords"].is_array(), "chords must be an array");
    assert_eq!(parsed["chords"].as_array().unwrap().len(), 2, "should have 2 chords");
    assert!(parsed["transitions"].is_array(), "transitions must be an array");
    assert_eq!(parsed["transitions"].as_array().unwrap().len(), 1, "should have 1 transition");
}

#[test]
fn analyze_json_chord_fields() {
    let out = slonimsky()
        .args(["analyze", "--format", "json", "--key", "C", "C,E,G", "F,A,C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout))
            .expect("valid JSON");

    let chord0 = &parsed["chords"][0];
    assert!(chord0["input"].is_string());
    assert!(chord0["pcs"].is_array());
    assert!(chord0["root"].is_string());
    assert!(chord0["quality"].is_string());
    // degree should be a number for a diatonic chord
    assert!(chord0["degree"].is_number(), "diatonic chord should have numeric degree");
    assert!(chord0["roman"].is_string());
    assert!(chord0["diatonic"].is_boolean());
}

#[test]
fn analyze_json_transition_has_l1_cost() {
    let out = slonimsky()
        .args(["analyze", "--format", "json", "C,E,G", "F,A,C"])
        .output()
        .expect("command should run");

    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout))
            .expect("valid JSON");

    let t0 = &parsed["transitions"][0];
    assert_eq!(t0["from"], 0);
    assert_eq!(t0["to"], 1);
    assert!(t0["common_tones"].is_array());
    assert!(t0["l1_cost"].is_number());
    // L1 cost between Cmaj and Fmaj should be small but non-zero
    let cost = t0["l1_cost"].as_i64().unwrap();
    assert!(cost > 0, "Cmaj→Fmaj should have nonzero L1 cost");
    assert!(cost <= 12, "L1 cost for 3-voice step should be ≤12");
}

// --- Error cases ---

#[test]
fn analyze_rejects_single_chord() {
    slonimsky()
        .args(["analyze", "C,E,G"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 2 chords"));
}

#[test]
fn analyze_rejects_unknown_scale() {
    slonimsky()
        .args(["analyze", "--key", "C", "--scale", "phrygian", "C,E,G", "F,A,C"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown scale"));
}

#[test]
fn analyze_rejects_unknown_format() {
    slonimsky()
        .args(["analyze", "--format", "xml", "C,E,G", "F,A,C"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown format"));
}
