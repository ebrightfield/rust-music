use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

#[test]
fn linear_interval_diagram_writes_svg() {
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("linear.svg");
    slonimsky()
        .args([
            "interval-linear",
            "C,E,G",
            "--title",
            "C major intervals",
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("C major intervals"));
    assert!(svg.contains("<path"));
    assert!(svg.contains("<circle"));
}

#[test]
fn interval_pairs_lists_every_unordered_pair() {
    slonimsky()
        .args(["interval-pairs", "C,E,G"])
        .assert()
        .success()
        .stdout(predicate::str::contains("C (0) -> E (4): 4 semitones, ic4"))
        .stdout(predicate::str::contains("C (0) -> G (7): 7 semitones, ic5"))
        .stdout(predicate::str::contains("E (4) -> G (7): 3 semitones, ic3"));
}

#[test]
fn interval_pair_query_is_directed_json() {
    let output = slonimsky()
        .args([
            "interval-pairs",
            "C,E,G",
            "--pair",
            "G,C",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
    assert_eq!(value[0]["ascending_semitones"], 5);
    assert_eq!(value[0]["interval_class"], 5);
}

#[test]
fn scale_rotation_rebases_major_to_dorian() {
    slonimsky()
        .args(["scale-rotate", "C,D,E,F,G,A,B", "--steps", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Root: D (2)"))
        .stdout(predicate::str::contains("Notes: D E F G A B C"))
        .stdout(predicate::str::contains("Shape: 0 2 3 5 7 9 10"));
}

#[test]
fn negative_scale_rotation_wraps_and_emits_json() {
    let output = slonimsky()
        .args([
            "scale-rotate",
            "C,D,E,F,G,A,B",
            "--steps",
            "-1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["normalized_rotation"], 6);
    assert_eq!(value["root"], "B");
    assert_eq!(value["shape"], serde_json::json!([0, 1, 3, 5, 6, 8, 10]));
}
