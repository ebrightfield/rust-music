use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Success: basic text output ---

#[test]
fn annotate_two_chords_text_output() {
    let out = slonimsky()
        .args(["annotate", "C4,E4,G4", "C4,F4,A4"])
        .output()
        .expect("command should run");

    assert!(out.status.success(), "exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Step header with voice-leading direction
    assert!(
        stdout.contains("Step 1:"),
        "should show step header, got:\n{stdout}"
    );
    // L1 cost for C4,E4,G4 → C4,F4,A4: voice1=0, voice2=+1, voice3=+2 → L1=3
    assert!(
        stdout.contains("L1 cost: 3"),
        "L1 cost should be 3 (0+1+2), got:\n{stdout}"
    );
    // L∞ cost should be 2 (max of 0,1,2)
    assert!(
        stdout.contains("L∞ cost: 2"),
        "L∞ cost should be 2, got:\n{stdout}"
    );
    // No crossings for this smooth leading
    assert!(
        stdout.contains("Crossings: none"),
        "should have no crossings, got:\n{stdout}"
    );
}

#[test]
fn annotate_voice_motion_labels() {
    let out = slonimsky()
        .args(["annotate", "C4,E4,G4", "C4,F4,A4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Voice 1: C4→C4 is a common tone
    assert!(
        stdout.contains("common tone"),
        "C4→C4 should be labeled 'common tone', got:\n{stdout}"
    );
    // Voice 2: E4→F4 (+1 st) is a step
    assert!(
        stdout.contains("step"),
        "E4→F4 should be labeled 'step', got:\n{stdout}"
    );
}

#[test]
fn annotate_smoothness_rating_excellent() {
    let out = slonimsky()
        .args(["annotate", "C4,E4,G4", "C4,F4,A4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // L1=3 for 3 voices → excellent (threshold: ≤4)
    assert!(
        stdout.contains("excellent"),
        "L1=3 with 3 voices should rate 'excellent', got:\n{stdout}"
    );
}

#[test]
fn annotate_summary_section() {
    let out = slonimsky()
        .args(["annotate", "C4,E4,G4", "C4,F4,A4", "D4,F4,A4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Summary:"),
        "should have a summary section"
    );
    assert!(
        stdout.contains("Total L1 cost:"),
        "summary should show total L1"
    );
    assert!(
        stdout.contains("Average L1 per step:"),
        "summary should show average L1"
    );
    assert!(
        stdout.contains("Voice crossings:"),
        "summary should show crossing count"
    );
    assert!(
        stdout.contains("Smoothness rating:"),
        "summary should show overall rating"
    );
}

#[test]
fn annotate_detects_voice_crossings() {
    // Voice 1: C4→A4 (up), Voice 2: G4→D4 (down) → they cross
    let out = slonimsky()
        .args(["annotate", "C4,G4", "A4,D4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should report at least 1 crossing
    assert!(
        !stdout.contains("Crossings: none"),
        "should detect crossing when voices swap registers, got:\n{stdout}"
    );
    // Crossing count should appear as a positive number
    assert!(
        stdout.contains("Crossings: 1"),
        "should detect exactly 1 crossing, got:\n{stdout}"
    );
}

#[test]
fn annotate_no_crossings_flag_adds_warning() {
    let out = slonimsky()
        .args(["annotate", "--no-crossings", "C4,G4", "A4,D4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("crossing"),
        "should mention crossing with --no-crossings flag, got:\n{stdout}"
    );
}

#[test]
fn annotate_multi_step_progression() {
    // 3 chords = 2 steps
    let out = slonimsky()
        .args(["annotate", "C4,E4,G4", "C4,F4,A4", "B3,D4,G4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Step 1:"), "should show step 1");
    assert!(stdout.contains("Step 2:"), "should show step 2");
}

// --- Success: JSON output ---

#[test]
fn annotate_json_via_format_flag() {
    let out = slonimsky()
        .args(["annotate", "--format", "json", "C4,E4,G4", "C4,F4,A4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output must be valid JSON");

    assert_eq!(parsed["voices"], 3, "should have 3 voices");
    assert!(parsed["steps"].is_array(), "steps must be an array");
    assert_eq!(
        parsed["steps"].as_array().unwrap().len(),
        1,
        "2 chords → 1 step"
    );
}

#[test]
fn annotate_json_step_fields() {
    let out = slonimsky()
        .args(["annotate", "--format", "json", "C4,E4,G4", "C4,F4,A4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout))
            .expect("valid JSON");

    let step0 = &parsed["steps"][0];
    assert!(step0["from"].is_array(), "step should have 'from' array");
    assert!(step0["to"].is_array(), "step should have 'to' array");
    assert!(step0["paths"].is_array(), "step should have 'paths' array");
    assert_eq!(step0["l1_cost"], 3, "L1 should be 3");
    assert_eq!(step0["linf_cost"], 2, "L∞ should be 2");
    assert_eq!(step0["crossings"], 0, "no crossings expected");
    assert_eq!(step0["smoothness"], "excellent");
}

#[test]
fn annotate_json_summary() {
    let out = slonimsky()
        .args([
            "annotate", "--format", "json",
            "C4,E4,G4", "C4,F4,A4", "D4,F4,A4",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout))
            .expect("valid JSON");

    assert!(parsed["summary"]["total_l1"].is_number());
    assert!(parsed["summary"]["average_l1"].is_number());
    assert!(parsed["summary"]["total_crossings"].is_number());
    assert!(parsed["summary"]["smoothness"].is_string());
}

// --- File output ---

#[test]
fn annotate_writes_json_to_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let outpath = dir.path().join("annotate.json");

    let out = slonimsky()
        .args([
            "annotate",
            "C4,E4,G4",
            "C4,F4,A4",
            "-o",
            outpath.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success(), "exit 0");
    assert!(outpath.exists(), "output file should exist");
    let content = std::fs::read_to_string(&outpath).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("file must contain valid JSON");
    assert_eq!(parsed["voices"], 3);
}

#[test]
fn annotate_writes_text_to_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let outpath = dir.path().join("annotate.txt");

    let out = slonimsky()
        .args([
            "annotate",
            "C4,E4,G4",
            "C4,F4,A4",
            "-o",
            outpath.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success(), "exit 0");
    assert!(outpath.exists(), "output file should exist");
    let content = std::fs::read_to_string(&outpath).unwrap();
    assert!(
        content.contains("Step 1:"),
        "text file should contain step info"
    );
    assert!(
        content.contains("L1 cost: 3"),
        "text file should contain L1 cost"
    );
}

// --- Error cases ---

#[test]
fn annotate_rejects_single_chord() {
    slonimsky()
        .args(["annotate", "C4,E4,G4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 2 chords"));
}

#[test]
fn annotate_rejects_mismatched_voices() {
    slonimsky()
        .args(["annotate", "C4,E4,G4", "F4,A4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("same number of voices"));
}

#[test]
fn annotate_rejects_invalid_pitch() {
    slonimsky()
        .args(["annotate", "C4,E4,G4", "X4,Y4,Z4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized note"));
}

#[test]
fn annotate_rejects_missing_octave() {
    slonimsky()
        .args(["annotate", "C,E,G", "F,A,C"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("octave"));
}

#[test]
fn annotate_rejects_unknown_format() {
    slonimsky()
        .args(["annotate", "--format", "xml", "C4,E4,G4", "C4,F4,A4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown format"));
}
