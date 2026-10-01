use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

#[test]
fn bmaj7_exhaustive_json_exposes_all_shapes() {
    let output = slonimsky()
        .args([
            "melodic-shapes",
            "B,D#,F#,A#",
            "--category",
            "exhaustive",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["count"], 44);
    assert_eq!(
        value["pitch_set"],
        serde_json::json!(["B", "D#", "F#", "A#"])
    );
    assert!(value["shapes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|shape| shape["complete"] == true));
}

#[test]
fn content_controls_select_start_score_span_and_limit() {
    let output = slonimsky()
        .args([
            "melodic-shapes",
            "B,D#,F#,A#",
            "--category",
            "exhaustive",
            "--starting-note",
            "B",
            "--max-score",
            "1",
            "--max-span",
            "5",
            "--sort",
            "score",
            "--limit",
            "3",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let shapes = value["shapes"].as_array().unwrap();
    assert_eq!(shapes.len(), 3);
    assert!(shapes.iter().all(|shape| shape["starting_note"] == "B"));
    assert!(shapes
        .iter()
        .all(|shape| shape["score"].as_u64().unwrap() <= 1));
    assert!(shapes
        .iter()
        .all(|shape| shape["span"]["width"].as_u64().unwrap() <= 5));
}

#[test]
fn rendering_controls_produce_svg_grid() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bmaj7.svg");
    slonimsky()
        .args([
            "melodic-shapes",
            "B,D#,F#,A#",
            "--category",
            "simple,2nps",
            "--orientation",
            "vertical",
            "--columns",
            "2",
            "--tile-width",
            "280",
            "--tile-height",
            "300",
            "--gap",
            "12",
            "--num-frets",
            "8",
            "--no-string-names",
            "--title",
            "BMaj7 positions",
            "-o",
            path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    let svg = fs::read_to_string(path).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("BMaj7 positions"));
    assert!(svg.matches("<g transform=").count() >= 2);
}

#[test]
fn png_output_has_png_signature() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("shape.png");
    slonimsky()
        .args([
            "melodic-shapes",
            "B,D#,F#,A#",
            "--category",
            "2nps",
            "--starting-note",
            "B",
            "--dpi",
            "96",
            "-o",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let png = fs::read(path).unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn pdf_output_has_pdf_signature() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("shape.pdf");
    slonimsky()
        .args([
            "melodic-shapes",
            "B,D#,F#,A#",
            "--category",
            "simple",
            "--limit",
            "2",
            "-o",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let pdf = fs::read(path).unwrap();
    assert_eq!(&pdf[..5], b"%PDF-");
}

#[test]
fn rejects_unknown_category_and_non_member_start() {
    slonimsky()
        .args(["melodic-shapes", "C,E,G", "--category", "imaginary"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown category"));
    slonimsky()
        .args(["melodic-shapes", "C,E,G", "--starting-note", "D"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not in the input set"));
}

#[test]
fn accepts_inline_arbitrary_tuning() {
    let output = slonimsky()
        .args([
            "melodic-shapes",
            "C,E,G",
            "--tuning",
            "E3,A3,D4,G4,B4,E5",
            "--category",
            "simple",
            "--limit",
            "1",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
