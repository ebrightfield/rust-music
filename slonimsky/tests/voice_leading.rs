use assert_cmd::Command;
use predicates::prelude::*;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// ==================== basic voice-leading ====================

#[test]
fn voice_leading_c_to_f_finds_results() {
    let out = slonimsky()
        .args(["voice-leading", "--from", "C4,E4,G4", "--to", "F,A,C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Voice-leading:"),
        "should show header; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Voices: 3"),
        "should show 3 voices; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Total:"),
        "should show total line; got:\n{stdout}"
    );
}

#[test]
fn voice_leading_header_shows_from_and_to() {
    let out = slonimsky()
        .args(["voice-leading", "--from", "C4,E4,G4", "--to", "F,A,C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Header should contain the starting pitches and target notes
    assert!(
        stdout.contains("C4") || stdout.contains("c4"),
        "header should mention C4; got:\n{stdout}"
    );
}

#[test]
fn voice_leading_results_show_dist() {
    let out = slonimsky()
        .args(["voice-leading", "--from", "C4,E4,G4", "--to", "F,A,C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("dist="),
        "results should show distance; got:\n{stdout}"
    );
}

#[test]
fn voice_leading_results_sorted_by_distance() {
    let out = slonimsky()
        .args(["voice-leading", "--from", "C4,E4,G4", "--to", "F,A,C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Extract distances from lines containing "dist="
    let dists: Vec<i32> = stdout
        .lines()
        .filter_map(|line| {
            let idx = line.find("dist=")?;
            let after = &line[idx + 5..];
            let end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
            after[..end].parse().ok()
        })
        .collect();
    assert!(dists.len() >= 2, "should find at least 2 results with distances");
    for w in dists.windows(2) {
        assert!(
            w[0] <= w[1],
            "results should be sorted by distance: {} > {}",
            w[0],
            w[1]
        );
    }
}

// ==================== no-crossings rule ====================

#[test]
fn voice_leading_no_crossings_shows_rule() {
    let out = slonimsky()
        .args([
            "voice-leading",
            "--from", "C4,E4,G4",
            "--to", "F,A,C",
            "--no-crossings",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("no voice crossings"),
        "should show crossing rule; got:\n{stdout}"
    );
}

#[test]
fn voice_leading_no_crossings_reduces_count() {
    let all = slonimsky()
        .args(["voice-leading", "--from", "C4,E4,G4", "--to", "F,A,C"])
        .output()
        .expect("command should run");
    let constrained = slonimsky()
        .args([
            "voice-leading",
            "--from", "C4,E4,G4",
            "--to", "F,A,C",
            "--no-crossings",
        ])
        .output()
        .expect("command should run");

    let all_out = String::from_utf8_lossy(&all.stdout);
    let con_out = String::from_utf8_lossy(&constrained.stdout);

    // Extract total count from "Total: N voice-leadings"
    let extract_total = |s: &str| -> usize {
        s.lines()
            .find(|l| l.starts_with("Total:"))
            .and_then(|l| {
                let after = l.strip_prefix("Total: ")?;
                let end = after.find(' ').unwrap_or(after.len());
                after[..end].parse().ok()
            })
            .unwrap_or(0)
    };

    let all_total = extract_total(&all_out);
    let con_total = extract_total(&con_out);
    assert!(
        con_total <= all_total,
        "no-crossings should produce ≤ results: {} vs {}",
        con_total,
        all_total
    );
    assert!(con_total > 0, "no-crossings should still find some results");
}

// ==================== limit ====================

#[test]
fn voice_leading_limit_caps_output() {
    let limited = slonimsky()
        .args([
            "voice-leading",
            "--from", "C4,E4,G4",
            "--to", "F,A,C",
            "--limit", "2",
        ])
        .output()
        .expect("command should run");

    assert!(limited.status.success());
    let stdout = String::from_utf8_lossy(&limited.stdout);
    assert!(
        stdout.contains("showing 2/"),
        "should show 'showing 2/N'; got:\n{stdout}"
    );
    // Count numbered results (lines starting with whitespace + digit + ".")
    let result_lines = stdout
        .lines()
        .filter(|l| {
            let trimmed = l.trim_start();
            trimmed.starts_with("1.") || trimmed.starts_with("2.") || trimmed.starts_with("3.")
        })
        .count();
    assert_eq!(result_lines, 2, "should show exactly 2 results; got:\n{stdout}");
}

// ==================== verbose ====================

#[test]
fn voice_leading_verbose_shows_paths() {
    let out = slonimsky()
        .args([
            "voice-leading",
            "--from", "C4,E4,G4",
            "--to", "F,A,C",
            "-v",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("paths=["),
        "verbose should show paths; got:\n{stdout}"
    );
}

// ==================== four-voice ====================

#[test]
fn voice_leading_four_voices_works() {
    let out = slonimsky()
        .args([
            "voice-leading",
            "--from", "C4,E4,G4,B4",
            "--to", "D,F,A,C",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Voices: 4"),
        "should show 4 voices; got:\n{stdout}"
    );
    assert!(
        stdout.contains("Total:"),
        "should show total; got:\n{stdout}"
    );
}

// ==================== error cases ====================

#[test]
fn voice_leading_missing_from_fails() {
    slonimsky()
        .args(["voice-leading", "--to", "F,A,C"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--from"));
}

#[test]
fn voice_leading_missing_to_fails() {
    slonimsky()
        .args(["voice-leading", "--from", "C4,E4,G4"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--to"));
}

#[test]
fn voice_leading_voice_count_mismatch_fails() {
    let out = slonimsky()
        .args([
            "voice-leading",
            "--from", "C4,E4,G4",
            "--to", "F,A",
        ])
        .output()
        .expect("command should run");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("voice count mismatch"),
        "should report voice count mismatch; got:\n{stderr}"
    );
}

#[test]
fn voice_leading_bad_pitch_fails() {
    let out = slonimsky()
        .args([
            "voice-leading",
            "--from", "X4,Y4,Z4",
            "--to", "F,A,C",
        ])
        .output()
        .expect("command should run");

    assert!(!out.status.success());
}
