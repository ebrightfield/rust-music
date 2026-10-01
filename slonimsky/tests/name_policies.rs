use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

#[test]
fn presets_and_explicit_inference_overrides_change_names() {
    slonimsky()
        .args(["name", "C,D,E,G", "--naming-style", "pop"])
        .assert()
        .success()
        .stdout("Cadd9\n");

    slonimsky()
        .args(["name", "C,E,Bb", "--show-omissions", "false"])
        .assert()
        .success()
        .stdout("C7\n");

    slonimsky()
        .args(["name", "C,E,G,A", "--distinguish-sixth", "false"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(13)"));
}

#[test]
fn extension_suspension_and_symbol_policies_change_display() {
    slonimsky()
        .args(["name", "C,E,G,Bb,D,F", "--extension-style", "highest"])
        .assert()
        .success()
        .stdout("C11 (9)\n");

    slonimsky()
        .args(["name", "C,F,G,Bb", "--explicit-sus4"])
        .assert()
        .success()
        .stdout("C7sus4\n");

    slonimsky()
        .args([
            "name",
            "C,E,G,B",
            "--major-symbol",
            "delta",
            "--root-spacing",
            "1",
        ])
        .assert()
        .success()
        .stdout("C Δ7\n");
}

#[test]
fn accidental_policy_selects_ascii_or_unicode() {
    slonimsky()
        .args(["name", "C,E,Gb,Bb", "--accidentals", "ascii"])
        .assert()
        .success()
        .stdout("C7 (b5)\n");

    slonimsky()
        .args(["name", "C,E,Gb,Bb", "--accidentals", "unicode"])
        .assert()
        .success()
        .stdout("C7 (♭5)\n");
}

#[test]
fn ambiguity_reporting_is_structured_in_json() {
    let output = slonimsky()
        .args([
            "name",
            "C,Eb,E,G",
            "--report-ambiguities",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["ambiguities"][0]["kind"], "duplicate-scale-degree");
    assert_eq!(value["ambiguities"][0]["degree"], 3);
    assert_eq!(value["config"]["naming"]["report_ambiguities"], true);
}

#[test]
fn output_extension_infers_json_and_invalid_policies_fail() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("name.json");
    slonimsky()
        .args(["name", "C,E,G", "--output", output.to_str().unwrap()])
        .assert()
        .success();
    let value: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(value["name"], "CMaj");

    slonimsky()
        .args(["name", "C,E,G", "--major-symbol", "triangle"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported major symbol"));
}

#[test]
fn bass_aware_inference_names_inversions() {
    slonimsky()
        .args(["name", "E,G,B,C", "--bass", "E"])
        .assert()
        .success()
        .stdout("CMaj7/E\n");

    slonimsky()
        .args(["name", "E,G,C", "--bass", "E"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "could not infer a bass-aware chord name",
        ));

    slonimsky()
        .args(["name", "E,G,C", "--bass", "E", "--naming-style", "jazz"])
        .assert()
        .success()
        .stdout("CMaj/E\n");
}

#[test]
fn explicit_root_and_slash_spacing_are_authoritative() {
    slonimsky()
        .args([
            "name",
            "A,C,E,G",
            "--root",
            "C",
            "--bass",
            "E",
            "--quality-slash-spacing",
            "1",
            "--slash-spacing",
            "1",
        ])
        .assert()
        .success()
        .stdout("CMaj6 / E\n");
}

#[test]
fn bass_aware_json_exposes_resolved_tonality() {
    let output = slonimsky()
        .args(["name", "E,G,B,C", "--bass", "E", "--format", "json"])
        .output()
        .expect("name command should run");
    assert!(output.status.success());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(value["name"], "CMaj7/E");
    assert_eq!(value["root"], "C");
    assert_eq!(value["root_pc"], 0);
    assert_eq!(value["bass"], "E");
    assert_eq!(value["bass_pc"], 4);
    assert_eq!(value["inversion"], true);
    assert_eq!(value["config"]["naming"]["slash_chord_threshold"], 4);
}
