#![cfg(feature = "midi")]
/// Integration tests for the `ear-training` subcommand.
///
/// These tests require `--features midi` to build and run.
/// Run: `cargo test -p slonimsky --features midi --test ear_training`
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== MIDI output ====================

#[test]
fn ear_training_produces_midi_with_seed() {
    let dir = TempDir::new().unwrap();
    let midi_path = dir.path().join("quiz.mid");

    let out = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "42",
            "--count",
            "5",
            "-o",
            midi_path.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success(), "exit 0");

    // MIDI file exists and has MThd magic bytes
    let bytes = std::fs::read(&midi_path).expect("MIDI file should exist");
    assert!(bytes.len() > 14, "MIDI file should be non-trivial");
    assert_eq!(&bytes[0..4], b"MThd", "MIDI starts with MThd header");

    // Format 1 (parallel) has 2+ tracks
    let num_tracks = u16::from_be_bytes([bytes[10], bytes[11]]);
    assert_eq!(
        num_tracks, 2,
        "SMF should have 2 tracks (conductor + notes)"
    );
}

#[test]
fn ear_training_answer_key_on_stdout() {
    let dir = TempDir::new().unwrap();
    let midi_path = dir.path().join("quiz.mid");

    let out = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "42",
            "--count",
            "3",
            "-o",
            midi_path.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Header
    assert!(
        stdout.contains("Ear Training"),
        "should show ear training header"
    );
    assert!(
        stdout.contains("3 items"),
        "should show item count: {}",
        stdout
    );

    // Answer entries with numbered items
    assert!(stdout.contains("#"), "should show numbered items");
    assert!(
        stdout.contains("semitones"),
        "should show semitone distances"
    );
}

#[test]
fn ear_training_seed_produces_deterministic_output() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let path1 = dir1.path().join("a.mid");
    let path2 = dir2.path().join("b.mid");

    let out1 = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "123",
            "--count",
            "5",
            "-o",
            path1.to_str().unwrap(),
        ])
        .output()
        .expect("run 1");

    let out2 = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "123",
            "--count",
            "5",
            "-o",
            path2.to_str().unwrap(),
        ])
        .output()
        .expect("run 2");

    assert!(out1.status.success());
    assert!(out2.status.success());

    // Same seed → same MIDI bytes
    let bytes1 = std::fs::read(&path1).unwrap();
    let bytes2 = std::fs::read(&path2).unwrap();
    assert_eq!(bytes1, bytes2, "same seed should produce identical MIDI");

    // Same seed → same answer key text
    assert_eq!(
        out1.stdout, out2.stdout,
        "same seed should produce identical answers"
    );
}

#[test]
fn ear_training_different_seeds_differ() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let path1 = dir1.path().join("a.mid");
    let path2 = dir2.path().join("b.mid");

    let out1 = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "1",
            "--count",
            "10",
            "-o",
            path1.to_str().unwrap(),
        ])
        .output()
        .expect("run 1");

    let out2 = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "999",
            "--count",
            "10",
            "-o",
            path2.to_str().unwrap(),
        ])
        .output()
        .expect("run 2");

    assert!(out1.status.success());
    assert!(out2.status.success());

    let bytes1 = std::fs::read(&path1).unwrap();
    let bytes2 = std::fs::read(&path2).unwrap();
    assert_ne!(
        bytes1, bytes2,
        "different seeds should produce different MIDI"
    );
}

// ==================== JSON output ====================

#[test]
fn ear_training_json_output_is_valid() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("answers.json");

    let out = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "42",
            "--count",
            "4",
            "-o",
            json_path.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());

    let json_str = std::fs::read_to_string(&json_path).expect("JSON file should exist");
    assert!(json_str.starts_with('['), "JSON should be an array");
    assert!(
        json_str.trim_end().ends_with(']'),
        "JSON array should close"
    );

    // Check structural fields in each entry
    assert!(json_str.contains("\"item\""), "should have item field");
    assert!(json_str.contains("\"root\""), "should have root field");
    assert!(json_str.contains("\"target\""), "should have target field");
    assert!(
        json_str.contains("\"semitones\""),
        "should have semitones field"
    );
    assert!(
        json_str.contains("\"interval\""),
        "should have interval field"
    );
}

#[test]
fn ear_training_json_has_correct_item_count() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("answers.json");

    slonimsky()
        .args([
            "ear-training",
            "--seed",
            "42",
            "--count",
            "6",
            "-o",
            json_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let json_str = std::fs::read_to_string(&json_path).unwrap();
    let item_count = json_str.matches("\"item\"").count();
    // count may be slightly less than 6 if target_midi > 108 skip happens,
    // but with seed 42 and reasonable range it should be close
    assert!(
        (4..=6).contains(&item_count),
        "should have 4–6 items (got {})",
        item_count
    );
}

// ==================== verbose mode ====================

#[test]
fn ear_training_verbose_prints_to_stderr() {
    let dir = TempDir::new().unwrap();
    let midi_path = dir.path().join("quiz.mid");

    let out = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "42",
            "--count",
            "3",
            "-o",
            midi_path.to_str().unwrap(),
            "-v",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Wrote") && stderr.contains("quiz items"),
        "verbose should report items written: {}",
        stderr
    );
    assert!(stderr.contains("bytes"), "verbose should report byte count");
}

// ==================== count parameter ====================

#[test]
fn ear_training_count_1_produces_single_item() {
    let dir = TempDir::new().unwrap();
    let midi_path = dir.path().join("quiz.mid");

    let out = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "42",
            "--count",
            "1",
            "-o",
            midi_path.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1 items"), "should show 1 item");
    // Only one numbered answer
    assert!(stdout.contains("# 1:"), "should have item #1: {}", stdout);
}

#[test]
fn ear_training_larger_count_produces_more_midi_data() {
    let dir = TempDir::new().unwrap();
    let small_path = dir.path().join("small.mid");
    let large_path = dir.path().join("large.mid");

    slonimsky()
        .args([
            "ear-training",
            "--seed",
            "10",
            "--count",
            "2",
            "-o",
            small_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    slonimsky()
        .args([
            "ear-training",
            "--seed",
            "10",
            "--count",
            "20",
            "-o",
            large_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let small = std::fs::read(&small_path).unwrap();
    let large = std::fs::read(&large_path).unwrap();
    assert!(
        large.len() > small.len(),
        "more items should produce larger MIDI ({} vs {})",
        large.len(),
        small.len()
    );
}

// ==================== interval names in output ====================

#[test]
fn ear_training_answer_key_contains_interval_names() {
    let dir = TempDir::new().unwrap();
    let midi_path = dir.path().join("quiz.mid");

    // Run with enough items and a known seed to hit various intervals
    let out = slonimsky()
        .args([
            "ear-training",
            "--seed",
            "7",
            "--count",
            "20",
            "-o",
            midi_path.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);

    // With 20 items we should hit at least one of these common intervals
    let has_named_interval = stdout.contains("(P1)")
        || stdout.contains("(m2)")
        || stdout.contains("(M2)")
        || stdout.contains("(m3)")
        || stdout.contains("(M3)")
        || stdout.contains("(P4)")
        || stdout.contains("(TT)")
        || stdout.contains("(P5)")
        || stdout.contains("(P8)");

    assert!(
        has_named_interval,
        "answer key should contain named intervals: {}",
        stdout
    );
}

// ==================== error cases ====================

#[test]
fn ear_training_bad_quiz_type_fails() {
    slonimsky()
        .args(["ear-training", "--type", "nonsense"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown quiz type"));
}

#[test]
fn ear_training_help_shows_flags() {
    slonimsky()
        .args(["ear-training", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--type"))
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("--seed"));
}
