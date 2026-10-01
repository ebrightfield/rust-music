use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("slonimsky").unwrap()
}

#[test]
fn sequence_generates_nested_pattern_and_cycles_rhythm() {
    cmd()
        .args([
            "sequence",
            "C,D,E,F,G,A,B",
            "--pattern",
            "1,1/2",
            "--rhythm",
            "8,q",
            "--start",
            "C4",
            "--low",
            "C4",
            "--high",
            "C5",
            "--length",
            "6",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "1\tC4\t60\t8\t16\tfalse\n2\tD4\t62\tq\t32\tfalse",
        ))
        .stdout(predicate::str::contains("6\tG4\t67\tq\t32\tfalse"));
}

#[test]
fn sequence_json_is_machine_readable_and_complete() {
    let assert = cmd()
        .args([
            "sequence",
            "C,D,E,F,G,A,B",
            "--length",
            "3",
            "--format",
            "json",
        ])
        .assert()
        .success();
    let value: Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    let events = value.as_array().unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["pitch"], "C4");
    assert_eq!(events[0]["midi"], 60);
    assert_eq!(events[0]["duration"], "8");
    assert_eq!(events[0]["ticks"], 16);
    assert_eq!(events[0]["tied"], false);
}

#[test]
fn sequence_infers_json_from_output_extension() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("sequence.json");
    cmd()
        .args([
            "sequence",
            "C,E,G",
            "--length",
            "2",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout("");
    let value: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 2);
}

#[test]
fn sequence_accepts_timed_harmonic_progressions() {
    cmd()
        .args([
            "sequence",
            "C,E,G",
            "D,F,A",
            "--chord-durations",
            "q,q",
            "--rhythm",
            "q",
            "--length",
            "3",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2\tF4"))
        .stdout(predicate::str::contains("3\tG4"));
}

#[test]
fn sequence_applies_boundary_turnaround_modes() {
    cmd()
        .args([
            "sequence",
            "C,D,E,F,G,A,B",
            "--pattern",
            "2",
            "--start",
            "B4",
            "--low",
            "C4",
            "--high",
            "C5",
            "--turnaround",
            "wrap",
            "--length",
            "2",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2\tC4\t60"));
}

#[test]
fn sequence_rejects_invalid_bounds_and_duration_cardinality() {
    cmd()
        .args([
            "sequence", "C,E,G", "--start", "C2", "--low", "C3", "--high", "C5",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("outside bounds"));

    cmd()
        .args(["sequence", "C,E,G", "D,F,A", "--chord-durations", "q,h,w"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("one value or one per harmony"));
}

#[cfg(feature = "midi")]
#[test]
fn sequence_writes_parseable_midi_with_tempo_and_ppq_controls() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("sequence.mid");
    cmd()
        .args([
            "sequence",
            "C,D,E,F,G,A,B",
            "--length",
            "4",
            "--rhythm",
            "q",
            "--bpm",
            "90",
            "--ppq",
            "480",
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let bytes = std::fs::read(output).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert_eq!(smf.tracks.len(), 2);
}

#[cfg(not(feature = "midi"))]
#[test]
fn sequence_explains_how_to_enable_midi_output() {
    cmd()
        .args([
            "sequence",
            "C,D,E,F,G,A,B",
            "--length",
            "1",
            "--format",
            "midi",
            "--output",
            "unused.mid",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "MIDI, WAV, and playback output require --features midi",
        ));
}
