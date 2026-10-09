use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::tempdir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../music-ron/fixtures/happy")
        .join(name)
}

#[test]
fn every_music_ron_document_kind_validates_and_emits_json() {
    for name in [
        "snippet.ron",
        "tab.ron",
        "fretboard_shape.ron",
        "pitch_circle.ron",
        "chord_progression.ron",
        "scale_diagram.ron",
        "interval_matrix.ron",
    ] {
        Command::cargo_bin("slonimsky")
            .unwrap()
            .args([
                "render",
                fixture(name).to_str().unwrap(),
                "--format",
                "json",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"kind\""));
    }
}

#[test]
fn score_is_rejected_instead_of_claiming_semantic_validation() {
    Command::cargo_bin("slonimsky")
        .unwrap()
        .args([
            "render",
            fixture("score.ron").to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Score documents are not supported by slonimsky render",
        ));
}

#[test]
fn pitch_circle_renders_svg_inferred_from_output_extension() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("circle.svg");
    Command::cargo_bin("slonimsky")
        .unwrap()
        .args([
            "render",
            fixture("pitch_circle.ron").to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn render_reads_stdin_and_writes_png() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("circle.png");
    let input = std::fs::read_to_string(fixture("pitch_circle.ron")).unwrap();
    Command::cargo_bin("slonimsky")
        .unwrap()
        .args(["render", "-", "--output", output.to_str().unwrap()])
        .write_stdin(input)
        .assert()
        .success();
    let png = std::fs::read(output).unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn binary_output_requires_a_path() {
    Command::cargo_bin("slonimsky")
        .unwrap()
        .args([
            "render",
            fixture("pitch_circle.ron").to_str().unwrap(),
            "--format",
            "png",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PNG/PDF output requires --output"));
}
