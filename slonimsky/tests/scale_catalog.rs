use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

#[test]
fn catalog_lists_all_twenty_two_families_as_json() {
    let output = slonimsky()
        .args(["scale-catalog", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let scales = value["scales"].as_array().unwrap();
    assert_eq!(scales.len(), 22);
    assert_eq!(scales[0]["slug"], "major");
    assert!(scales
        .iter()
        .any(|scale| scale["slug"] == "major-sharp9-b13"));
}

#[test]
fn selected_family_expands_to_seven_modes() {
    let output = slonimsky()
        .args([
            "scale-catalog",
            "melodic-minor",
            "--modes",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let modes = value["scales"][0]["modes"].as_array().unwrap();
    assert_eq!(modes.len(), 7);
    assert_eq!(modes[0]["name"], "Melodic Minor");
    assert_eq!(modes[6]["name"], "Altered");
    assert_eq!(
        modes[6]["pitch_classes"],
        serde_json::json!([0, 1, 3, 4, 6, 8, 10])
    );
}

#[test]
fn tonic_first_scale_identification_finds_mode_and_family() {
    slonimsky()
        .args(["scale-catalog", "--identify", "D,E,F,G,A,B,C"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Tonic: D (2)"))
        .stdout(predicate::str::contains("Major [major], mode 2: Dorian"));
}

#[test]
fn scale_book_accepts_every_catalog_family() {
    slonimsky()
        .args(["scale-book", "major-sharp9-b13", "--keys", "C"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mode 1: Major #9 b13 mode 1"))
        .stdout(predicate::str::contains("Total: 7 entries"));
}

#[test]
fn identification_rejects_non_catalog_scales() {
    slonimsky()
        .args(["scale-catalog", "--identify", "C,Db,D,Eb,E,F,Gb"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in the catalog"));
}
