use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("slonimsky").unwrap()
}

// --- Basic triadic progression ---

#[test]
fn progression_two_chords_shows_header_and_steps() {
    let assert = cmd()
        .args(["progression", "C,E,G", "F,A,C"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        out.contains("Progression:"),
        "should show Progression header"
    );
    assert!(out.contains("Voices: 3"), "should show voice count");
    assert!(out.contains("Start:"), "should show starting voicing");
    assert!(out.contains("Step 1:"), "should show at least one step");
    assert!(out.contains("dist="), "should show distance for step");
    assert!(
        out.contains("Total voice-leading cost:"),
        "should show total cost"
    );
}

#[test]
fn progression_three_chords_has_two_steps() {
    let assert = cmd()
        .args(["progression", "C,E,G", "F,A,C", "G,B,D"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains("Step 1:"), "should have step 1");
    assert!(out.contains("Step 2:"), "should have step 2");
    assert!(out.contains("2 steps"), "total should mention 2 steps");
}

// --- Header content ---

#[test]
fn progression_header_shows_chord_labels() {
    let assert = cmd()
        .args(["progression", "C,E,G", "D,F,A"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Header should show the chord note names with arrow separator
    assert!(out.contains("→"), "should show arrow between chords");
    assert!(out.contains("C"), "should show C in progression label");
    assert!(out.contains("D"), "should show D in progression label");
}

// --- No crossings ---

#[test]
fn progression_no_crossings_shows_rule() {
    let assert = cmd()
        .args(["progression", "C,E,G", "F,A,C", "--no-crossings"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        out.contains("no voice crossings"),
        "should show no-crossings rule"
    );
}

// --- Verbose mode ---

#[test]
fn progression_verbose_shows_paths() {
    let assert = cmd()
        .args(["progression", "C,E,G", "F,A,C", "-v"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Verbose shows per-voice semitone paths
    assert!(out.contains("paths=["), "verbose should show paths");
}

#[test]
fn progression_verbose_shows_avg_cost_on_stderr() {
    let assert = cmd()
        .args(["progression", "C,E,G", "F,A,C", "G,B,D", "-v"])
        .assert()
        .success();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        err.contains("Average cost per step"),
        "verbose should print avg cost to stderr"
    );
}

// --- Four-voice progression (ii-V-I) ---

#[test]
fn progression_four_voice_ii_v_i() {
    let assert = cmd()
        .args([
            "progression",
            "D,F,A,C",
            "G,B,D,F",
            "C,E,G,B",
            "--no-crossings",
        ])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains("Voices: 4"), "should have 4 voices");
    assert!(out.contains("Step 1:"), "should have step 1");
    assert!(out.contains("Step 2:"), "should have step 2");
    // Total cost should be a positive number
    assert!(
        out.contains("Total voice-leading cost:"),
        "should show total cost"
    );
    // Extract cost value — it should not be 0 for different chords
    let cost_line = out
        .lines()
        .find(|l| l.contains("Total voice-leading cost:"))
        .unwrap();
    assert!(
        !cost_line.contains("cost: 0"),
        "cost should be > 0 for different chords"
    );
}

// --- Integer input ---

#[test]
fn progression_integer_input() {
    let assert = cmd()
        .args(["progression", "0,4,7", "5,9,0"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains("Voices: 3"), "integer input should work");
    assert!(out.contains("Step 1:"), "should have a step");
}

// --- Distance is positive ---

#[test]
fn progression_distance_positive_for_different_chords() {
    let assert = cmd()
        .args(["progression", "C,E,G", "D,F,A"])
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Parse the dist= value from Step 1
    let step_line = out.lines().find(|l| l.contains("Step 1:")).unwrap();
    assert!(step_line.contains("dist="), "step should show distance");
    // dist should not be 0
    assert!(
        !step_line.contains("dist=0"),
        "distance should be > 0 for different chords"
    );
}

// --- Weighted metric ---

#[test]
fn progression_weighted_metric_scores_each_voice() {
    cmd()
        .args([
            "progression",
            "C,E,G",
            "F,A,C",
            "G,B,D",
            "--metric",
            "weighted",
            "--weights",
            "10,1,1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Metric: weighted L1"))
        .stdout(predicate::str::contains("Weights: 10,1,1"))
        .stdout(predicate::str::contains(
            "Total voice-leading cost: 18 (weighted L1, 2 steps)",
        ));
}

#[test]
fn progression_weights_require_weighted_metric() {
    cmd()
        .args(["progression", "C,E,G", "F,A,C", "--weights", "1,1,1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--weights requires --metric weighted",
        ));
}

// --- Error cases ---

#[test]
fn progression_single_chord_fails() {
    cmd()
        .args(["progression", "C,E,G"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 2 chords"));
}

#[test]
fn progression_mismatched_cardinality_fails() {
    cmd()
        .args(["progression", "C,E,G", "D,F,A,C"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("same number of notes"));
}

#[test]
fn progression_no_input_fails() {
    cmd()
        .arg("progression")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn progression_bad_note_fails() {
    cmd()
        .args(["progression", "X,Y,Z", "A,B,C"])
        .assert()
        .failure();
}

// ==================== MIDI output (requires `midi` feature) ====================

#[cfg(feature = "midi")]
mod midi_integration {
    use assert_cmd::Command;
    use tempfile::TempDir;

    fn cmd() -> Command {
        Command::cargo_bin("slonimsky").unwrap()
    }

    #[test]
    fn progression_midi_produces_valid_smf() {
        let dir = TempDir::new().unwrap();
        let midi_path = dir.path().join("prog.mid");

        let out = cmd()
            .args([
                "progression",
                "C,E,G",
                "F,A,C",
                "G,B,D",
                "-o",
                midi_path.to_str().unwrap(),
            ])
            .output()
            .expect("command should run");

        assert!(
            out.status.success(),
            "exit 0: {:?}",
            String::from_utf8_lossy(&out.stderr)
        );

        let bytes = std::fs::read(&midi_path).expect("MIDI file should exist");
        // MThd magic bytes
        assert_eq!(&bytes[0..4], b"MThd", "MIDI starts with MThd");
        // Header chunk length is always 6
        assert_eq!(
            u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            6,
            "MThd chunk length should be 6"
        );
        // Format 1 (parallel tracks)
        assert_eq!(
            u16::from_be_bytes([bytes[8], bytes[9]]),
            1,
            "SMF format should be 1 (parallel)"
        );
        // 2 tracks: conductor + instrument
        assert_eq!(
            u16::from_be_bytes([bytes[10], bytes[11]]),
            2,
            "should have 2 tracks"
        );
        // PPQ = 480
        assert_eq!(
            u16::from_be_bytes([bytes[12], bytes[13]]),
            480,
            "PPQ should be 480"
        );
        // Verify both MTrk markers
        let mtrk_count = bytes.windows(4).filter(|w| *w == b"MTrk").count();
        assert_eq!(mtrk_count, 2, "should contain 2 MTrk chunks");
    }

    #[test]
    fn progression_midi_text_still_printed_to_stdout() {
        let dir = TempDir::new().unwrap();
        let midi_path = dir.path().join("prog.mid");

        let out = cmd()
            .args([
                "progression",
                "C,E,G",
                "F,A,C",
                "-o",
                midi_path.to_str().unwrap(),
            ])
            .output()
            .expect("command should run");

        assert!(out.status.success());
        let stdout = String::from_utf8_lossy(&out.stdout);
        // Text summary should still appear even when writing MIDI
        assert!(
            stdout.contains("Progression:"),
            "stdout should still contain text summary when writing MIDI"
        );
        assert!(stdout.contains("Step 1:"), "stdout should still show steps");
    }

    #[test]
    fn progression_midi_verbose_reports_file_info_on_stderr() {
        let dir = TempDir::new().unwrap();
        let midi_path = dir.path().join("prog.mid");

        let out = cmd()
            .args([
                "progression",
                "C,E,G",
                "D,F,A",
                "G,B,D",
                "-o",
                midi_path.to_str().unwrap(),
                "-v",
            ])
            .output()
            .expect("command should run");

        assert!(out.status.success());
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("Wrote MIDI"),
            "verbose stderr should report MIDI write: {}",
            stderr
        );
        assert!(
            stderr.contains("bytes"),
            "verbose stderr should report byte count"
        );
        assert!(
            stderr.contains("3 chords"),
            "verbose stderr should report chord count"
        );
    }

    #[test]
    fn progression_midi_four_voice_ii_v_i() {
        let dir = TempDir::new().unwrap();
        let midi_path = dir.path().join("iivi.mid");

        cmd()
            .args([
                "progression",
                "D,F,A,C",
                "G,B,D,F",
                "C,E,G,B",
                "--no-crossings",
                "-o",
                midi_path.to_str().unwrap(),
            ])
            .assert()
            .success();

        let bytes = std::fs::read(&midi_path).expect("MIDI file should exist");
        assert_eq!(&bytes[0..4], b"MThd");
        // 2 tracks (conductor + instrument)
        assert_eq!(
            u16::from_be_bytes([bytes[10], bytes[11]]),
            2,
            "should have 2 tracks"
        );
        // 4-voice progression produces a substantially larger file than a
        // 3-voice 2-chord progression due to more note events per chord
        // and more chords. Just verify the file is non-trivially large.
        assert!(
            bytes.len() > 100,
            "4-voice ii-V-I MIDI should be substantial (got {} bytes)",
            bytes.len()
        );
    }

    #[test]
    fn progression_midi_larger_has_more_data() {
        let dir = TempDir::new().unwrap();
        let path_short = dir.path().join("short.mid");
        let path_long = dir.path().join("long.mid");

        cmd()
            .args([
                "progression",
                "C,E,G",
                "F,A,C",
                "-o",
                path_short.to_str().unwrap(),
            ])
            .assert()
            .success();

        cmd()
            .args([
                "progression",
                "C,E,G",
                "F,A,C",
                "G,B,D",
                "C,E,G",
                "A,C,E",
                "-o",
                path_long.to_str().unwrap(),
            ])
            .assert()
            .success();

        let short_bytes = std::fs::read(&path_short).unwrap();
        let long_bytes = std::fs::read(&path_long).unwrap();
        assert!(
            long_bytes.len() > short_bytes.len(),
            "longer progression should produce larger MIDI ({} vs {} bytes)",
            long_bytes.len(),
            short_bytes.len()
        );
    }

    #[test]
    fn progression_midi_with_linf_metric() {
        let dir = TempDir::new().unwrap();
        let midi_path = dir.path().join("linf.mid");

        let out = cmd()
            .args([
                "progression",
                "C,E,G",
                "D,F,A",
                "--metric",
                "linf",
                "-o",
                midi_path.to_str().unwrap(),
            ])
            .output()
            .expect("command should run");

        assert!(out.status.success(), "linf metric with MIDI should succeed");
        let bytes = std::fs::read(&midi_path).expect("MIDI file should exist");
        assert_eq!(&bytes[0..4], b"MThd");
        // Text output should mention linf metric
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("L∞") || stdout.contains("Linf"),
            "output should mention linf metric"
        );
    }
}
