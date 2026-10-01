use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

#[test]
fn extracts_directions_intervals_and_transformations() {
    slonimsky()
        .args(["contour", "C4,E4,D4,D4,G4"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Directions: ascending descending repeat ascending",
        ))
        .stdout(predicate::str::contains("Numeric: 1 -1 0 1"))
        .stdout(predicate::str::contains("Intervals: +4 -2 +0 +5"))
        .stdout(predicate::str::contains(
            "Interval changes: contracting contracting expanding",
        ))
        .stdout(predicate::str::contains("Retrograde-inversion: -1 0 1 -1"));
}

#[test]
fn applies_selected_transformation_and_accepts_midi_input() {
    slonimsky()
        .args(["contour", "60", "64", "62", "--transform", "inversion"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformation: inversion"))
        .stdout(predicate::str::contains("Transformed numeric: -1 1"));
}

#[test]
fn compares_contours_across_transformations_as_json() {
    let output = slonimsky()
        .args([
            "contour",
            "C4,D4,C4",
            "--compare",
            "G4,F4,G4",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["source"]["numeric"], serde_json::json!([1, -1]));
    assert_eq!(
        value["comparison"]["contour"]["numeric"],
        serde_json::json!([-1, 1])
    );
    assert_eq!(value["comparison"]["similarity"], 0.0);
    assert_eq!(value["comparison"]["max_similarity"], 1.0);
    assert_eq!(value["comparison"]["equivalent"], true);
}

#[test]
fn rejects_unpitched_and_single_note_inputs() {
    slonimsky()
        .args(["contour", "C", "D4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires an octave"));

    slonimsky()
        .args(["contour", "C4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires at least two pitches"));
}
