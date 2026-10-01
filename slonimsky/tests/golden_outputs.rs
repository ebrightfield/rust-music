//! Golden-output regression tests.
//!
//! These tests compare the exact stdout of selected subcommands against
//! frozen known-good outputs under `tests/golden/`. Any byte-level change
//! in output will cause a test failure, making unintentional regressions
//! immediately visible.
//!
//! To update a golden file after a deliberate output format change, re-run
//! the subcommand and overwrite the file, e.g.:
//!   cargo run -p slonimsky -- forte 0 4 7 > slonimsky/tests/golden/forte_c_major_triad.txt

use assert_cmd::Command;
use std::path::PathBuf;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name)
}

fn read_golden(name: &str) -> String {
    let path = golden_path(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read golden file {}: {e}", path.display()))
}

// ==================== Text subcommands ====================

#[test]
fn golden_interval_vector_c_major_triad() {
    let expected = read_golden("interval_vector_c_major_triad.txt");
    let out = slonimsky()
        .args(["interval-vector", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "interval-vector output differs from golden file"
    );
}

#[test]
fn golden_forte_c_major_triad() {
    let expected = read_golden("forte_c_major_triad.txt");
    let out = slonimsky()
        .args(["forte", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "forte output differs from golden file"
    );
}

#[test]
fn golden_prime_form_c_major_triad() {
    let expected = read_golden("prime_form_c_major_triad.txt");
    let out = slonimsky()
        .args(["prime-form", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "prime-form output differs from golden file"
    );
}

#[test]
fn golden_common_tones_cmaj_amin() {
    let expected = read_golden("common_tones_cmaj_amin.txt");
    let out = slonimsky()
        .args(["common-tones", "C,E,G", "A,C,E"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "common-tones output differs from golden file"
    );
}

#[test]
fn golden_orbits_dim7() {
    let expected = read_golden("orbits_dim7.txt");
    let out = slonimsky()
        .args(["orbits", "0", "3", "6", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "orbits output differs from golden file"
    );
}

// ==================== spell ====================

#[test]
fn golden_spell_cmaj7_all() {
    let expected = read_golden("spell_cmaj7_all.txt");
    let out = slonimsky()
        .args(["spell", "Cmaj7", "--format", "all"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "spell --format all output differs from golden file"
    );
    // Structural: all three labeled lines present
    assert!(stdout.contains("Notes:"), "should have Notes line");
    assert!(stdout.contains("PCs:"), "should have PCs line");
    assert!(stdout.contains("Intervals:"), "should have Intervals line");
}

// ==================== voice-leading ====================

#[test]
fn golden_voice_leading_cmaj_to_fmaj() {
    let expected = read_golden("voice_leading_cmaj_to_fmaj.txt");
    let out = slonimsky()
        .args([
            "voice-leading",
            "--from",
            "C4,E4,G4",
            "--to",
            "F,A,C",
            "--limit",
            "5",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "voice-leading output differs from golden file"
    );
    // Structural: header present, results ranked by distance
    assert!(stdout.contains("Voice-leading:"), "should have header");
    assert!(stdout.contains("dist=3"), "first result should have dist=3");
    assert!(stdout.contains("Total: 24"), "should report total count");
}

// ==================== subchords ====================

#[test]
fn golden_subchords_cmaj7_size3() {
    let expected = read_golden("subchords_cmaj7_size3.txt");
    let out = slonimsky()
        .args(["subchords", "0", "4", "7", "11", "--size", "3", "--name"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "subchords output differs from golden file"
    );
    // Structural: expected named chords present
    assert!(stdout.contains("CMaj"), "should contain CMaj triad");
    assert!(stdout.contains("Emin"), "should contain Emin triad");
    assert!(
        stdout.contains("Total: 4"),
        "should have 4 subchords of a 4-note set choose 3"
    );
}

// ==================== progression ====================

#[test]
fn golden_progression_cmaj_fmaj_gmaj() {
    let expected = read_golden("progression_cmaj_fmaj_gmaj.txt");
    let out = slonimsky()
        .args(["progression", "C,E,G", "F,A,C", "G,B,D"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "progression output differs from golden file"
    );
    // Structural: header, steps, and total present
    assert!(
        stdout.contains("Progression: C,E,G"),
        "should have progression header"
    );
    assert!(stdout.contains("Step 1:"), "should have Step 1");
    assert!(stdout.contains("Step 2:"), "should have Step 2");
    assert!(
        stdout.contains("Total voice-leading cost: 9"),
        "total cost should be 9"
    );
}

// ==================== contains ====================

#[test]
fn golden_contains_cmaj_super_10() {
    let expected = read_golden("contains_cmaj_super_10.txt");
    let out = slonimsky()
        .args([
            "contains",
            "0",
            "4",
            "7",
            "--direction",
            "super",
            "--limit",
            "10",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "contains output differs from golden file"
    );
    // Structural: section headers and known supersets
    assert!(
        stdout.contains("4-note chords"),
        "should have 4-note chords section"
    );
    assert!(
        stdout.contains("7-note scales"),
        "should have 7-note scales section"
    );
    assert!(stdout.contains("C Maj7"), "should contain C Maj7 superset");
    assert!(
        stdout.contains("C Major"),
        "should contain C Major scale superset"
    );
    assert!(
        stdout.contains("Total: 10"),
        "should report 10 total results"
    );
}

// ==================== closest ====================

#[test]
fn golden_closest_cmaj_chords_5() {
    let expected = read_golden("closest_cmaj_chords_5.txt");
    let out = slonimsky()
        .args(["closest", "0", "4", "7", "--pool", "chords", "--limit", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "closest output differs from golden file"
    );
    // Structural: all results at dist=1 (single PC addition from major triad)
    assert!(
        stdout.contains("dist=1"),
        "all results should be at distance 1"
    );
    assert!(
        stdout.contains("3 common tones"),
        "should show 3 common tones (the full triad)"
    );
    assert!(stdout.contains("C Maj7"), "should include C Maj7");
    assert!(stdout.contains("C Dom7"), "should include C Dom7");
    assert!(stdout.contains("Total: 5"), "should report 5 total results");
}

// ==================== voicings ====================

#[test]
fn golden_voicings_c_major_triad() {
    let expected = read_golden("voicings_c_major_triad.txt");
    let out = slonimsky()
        .args(["voicings", "C", "E", "G"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "voicings output differs from golden file"
    );
    // Structural: 2 families, 6 voicings, correct intervals
    assert!(
        stdout.contains("2 families, 6 voicings total"),
        "should have 2 families of 6 voicings"
    );
    assert!(
        stdout.contains("Family 1") && stdout.contains("Family 2"),
        "should have both family headers"
    );
    assert!(
        stdout.contains("[4, 3]"),
        "root position close voicing should have [4, 3] (M3 + m3)"
    );
    assert!(
        stdout.contains("[3, 5]"),
        "first inversion close voicing should have [3, 5] (m3 + P4)"
    );
    assert!(
        stdout.contains("Quality: Major"),
        "should identify quality as Major"
    );
}

#[test]
fn golden_voicings_dim7_limit6() {
    let expected = read_golden("voicings_dim7_limit6.txt");
    let out = slonimsky()
        .args(["voicings", "0", "3", "6", "9", "--limit", "6"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "voicings dim7 output differs from golden file"
    );
    // Structural: dim7 has 6 families × 4 inversions = 24 total, limited to 6
    assert!(
        stdout.contains("6 families, 24 voicings total"),
        "dim7 should have 6 families, 24 total"
    );
    assert!(
        stdout.contains("Quality: Dim7"),
        "should identify quality as Dim7"
    );
    // Dim7 close position: all intervals are [3, 3, 3] (symmetric!)
    assert!(
        stdout.contains("[3, 3, 3]"),
        "close-position dim7 should have uniform minor-third intervals"
    );
    assert!(
        stdout.contains("showing 6/24"),
        "should indicate limit is active"
    );
}

// ==================== SVG subcommand ====================

#[test]
fn golden_pitch_circle_c_major_triad_svg() {
    let expected = read_golden("pitch_circle_c_major_triad.svg");
    let out = slonimsky()
        .args(["pitch-circle", "C", "E", "G", "--title", "C Major Triad"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);

    // SVG golden comparison: exact byte match
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "pitch-circle SVG output differs from golden file"
    );

    // Structural sanity (redundant with exact match, but documents intent)
    assert!(stdout.starts_with("<svg"), "SVG should start with <svg");
    assert!(
        stdout.trim_end().ends_with("</svg>"),
        "SVG should end with </svg>"
    );
    assert!(
        stdout.contains("C Major Triad"),
        "SVG should contain the title"
    );
}

// ==================== name ====================

#[test]
fn golden_name_cmaj_triad() {
    let expected = read_golden("name_cmaj_triad.txt");
    let out = slonimsky()
        .args(["name", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "name output differs from golden file"
    );
    // Structural: C major triad should be named CMaj
    assert!(stdout.contains("CMaj"), "should identify as CMaj");
}

// ==================== interval-matrix ====================

#[test]
fn golden_interval_matrix_cmaj_triad() {
    let expected = read_golden("interval_matrix_cmaj_triad.txt");
    let out = slonimsky()
        .args(["interval-matrix", "0", "4", "7"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "interval-matrix output differs from golden file"
    );
    // Structural: 3×3 matrix with correct diagonal (zeros)
    assert!(
        stdout.contains("0   4   7"),
        "header row should show PCs 0, 4, 7"
    );
    assert!(
        stdout.contains("Interval vector: <0, 0, 1, 1, 1, 0>"),
        "major triad interval vector should be <0,0,1,1,1,0>"
    );
}

#[test]
fn golden_interval_matrix_dim7() {
    let expected = read_golden("interval_matrix_dim7.txt");
    let out = slonimsky()
        .args(["interval-matrix", "0", "3", "6", "9"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "interval-matrix dim7 output differs from golden file"
    );
    // Structural: dim7 is maximally symmetric — all off-diagonal entries are 3, 6, or 9
    assert!(
        stdout.contains("Interval vector: <0, 0, 4, 0, 0, 2>"),
        "dim7 interval vector should be <0,0,4,0,0,2>"
    );
    // The matrix should be circulant (each row is a rotation)
    assert!(
        stdout.contains("0   3   6   9"),
        "first row should be 0 3 6 9"
    );
    assert!(
        stdout.contains("9   0   3   6"),
        "second row should be rotated"
    );
}

// ==================== sight-reading (deterministic via seed) ====================

#[test]
fn golden_sight_reading_c_major_seed42() {
    let expected = read_golden("sight_reading_c_major_seed42.txt");
    let out = slonimsky()
        .args([
            "sight-reading",
            "--key",
            "C",
            "--scale",
            "major",
            "--difficulty",
            "1",
            "--measures",
            "2",
            "--seed",
            "42",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "sight-reading output differs from golden file"
    );
    // Structural: header and measures present
    assert!(stdout.contains("Key: C major"), "should show key in header");
    assert!(
        stdout.contains("Difficulty: 1/5"),
        "should show difficulty level"
    );
    assert!(stdout.contains("m1:"), "should have measure 1");
    assert!(stdout.contains("m2:"), "should have measure 2");
    // Difficulty 1 = steps only, so all notes should be in C major scale
    assert!(
        !stdout.contains('#') && !stdout.contains('b'),
        "difficulty 1 in C major should have no accidentals"
    );
}

// ==================== analyze ====================

#[test]
fn golden_analyze_i_vi_ii_v_text() {
    let expected = read_golden("analyze_i_vi_ii_v.txt");
    let out = slonimsky()
        .args(["analyze", "C,E,G", "A,C,E", "D,F,A", "G,B,D"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "analyze text output differs from golden file"
    );
    // Structural: key estimation, Roman numerals, transitions
    assert!(
        stdout.contains("C major (estimated, confidence: 7/7)"),
        "should estimate C major with full confidence"
    );
    assert!(stdout.contains("I"), "should have I for C major");
    assert!(stdout.contains("vi"), "should have vi for A minor");
    assert!(stdout.contains("ii"), "should have ii for D minor");
    assert!(stdout.contains("V"), "should have V for G major");
    assert!(
        stdout.contains("total: 9"),
        "total voice-leading cost should be 9"
    );
}

#[test]
fn golden_analyze_i_vi_ii_v_json() {
    let expected = read_golden("analyze_i_vi_ii_v.json");
    let out = slonimsky()
        .args([
            "analyze", "C,E,G", "A,C,E", "D,F,A", "G,B,D", "--format", "json",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "analyze JSON output differs from golden file"
    );
    // Structural: valid JSON with expected fields
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert_eq!(json["key"]["root"], "C");
    assert_eq!(json["key"]["scale"], "major");
    assert_eq!(json["key"]["confidence"], 7);
    assert_eq!(json["chords"].as_array().unwrap().len(), 4);
    assert_eq!(json["chords"][0]["roman"], "I");
    assert_eq!(json["chords"][1]["roman"], "vi");
    assert_eq!(json["chords"][2]["roman"], "ii");
    assert_eq!(json["chords"][3]["roman"], "V");
    assert_eq!(json["transitions"].as_array().unwrap().len(), 3);
}

// ==================== annotate ====================

#[test]
fn golden_annotate_cmaj_fmaj_gmaj() {
    let expected = read_golden("annotate_cmaj_fmaj_gmaj.txt");
    let out = slonimsky()
        .args(["annotate", "C4,E4,G4", "C4,F4,A4", "B3,D4,G4"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "annotate output differs from golden file"
    );
    // Structural: steps with voice-leading detail
    assert!(stdout.contains("Step 1:"), "should have Step 1");
    assert!(stdout.contains("Step 2:"), "should have Step 2");
    assert!(stdout.contains("L1 cost: 3"), "step 1 L1 cost should be 3");
    assert!(stdout.contains("L1 cost: 6"), "step 2 L1 cost should be 6");
    assert!(
        stdout.contains("common tone"),
        "step 1 should identify C4 as common tone"
    );
    assert!(
        stdout.contains("Crossings: none"),
        "no voice crossings expected"
    );
    assert!(stdout.contains("Total L1 cost: 9"), "total L1 should be 9");
    assert!(
        stdout.contains("Smoothness rating: excellent"),
        "overall smoothness should be excellent"
    );
}

// ==================== practice-sheet ====================

#[test]
fn golden_practice_sheet_c_major() {
    let expected = read_golden("practice_sheet_c_major.txt");
    let out = slonimsky()
        .args(["practice-sheet", "--key", "C", "--scale", "major"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "practice-sheet output differs from golden file"
    );
    // Structural: all 5 sections present
    assert!(
        stdout.contains("Practice Sheet: C Major"),
        "should have header"
    );
    assert!(stdout.contains("SCALE"), "should have SCALE section");
    assert!(stdout.contains("MODES"), "should have MODES section");
    assert!(
        stdout.contains("3-NOTE SUBCHORDS (35 total)"),
        "should have 35 3-note subchords (C(7,3)=35)"
    );
    assert!(
        stdout.contains("4-NOTE SUBCHORDS (35 total)"),
        "should have 35 4-note subchords (C(7,4)=35)"
    );
    assert!(
        stdout.contains("PRACTICE SUGGESTIONS"),
        "should have practice suggestions section"
    );
    // Content checks
    assert!(
        stdout.contains("Notes: C D E F G A B"),
        "C major scale should have all natural notes"
    );
    assert!(
        stdout.contains("Interval Vector: <2, 5, 4, 3, 6, 1>"),
        "C major IV should be <2,5,4,3,6,1>"
    );
    assert!(
        stdout.contains("Ionian") && stdout.contains("Dorian") && stdout.contains("Locrian"),
        "should list all 7 mode names"
    );
    assert!(
        stdout.contains("ii-V-I"),
        "practice suggestions should include ii-V-I"
    );
}

// ==================== scale-book ====================

#[test]
fn golden_scale_book_major_c() {
    let expected = read_golden("scale_book_major_c.txt");
    let out = slonimsky()
        .args(["scale-book", "major", "--keys", "C"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "scale-book output differs from golden file"
    );
    // Structural: header, all 7 modes, total
    assert!(
        stdout.contains("Scale Book: major (7 modes × 1 keys = 7 entries)"),
        "should have correct header"
    );
    assert!(
        stdout.contains("Total: 7 entries"),
        "should show 7 total entries"
    );
    // All 7 mode names present
    assert!(stdout.contains("Ionian"), "should have Ionian");
    assert!(stdout.contains("Dorian"), "should have Dorian");
    assert!(stdout.contains("Phrygian"), "should have Phrygian");
    assert!(stdout.contains("Lydian"), "should have Lydian");
    assert!(stdout.contains("Mixolydian"), "should have Mixolydian");
    assert!(stdout.contains("Aeolian"), "should have Aeolian");
    assert!(stdout.contains("Locrian"), "should have Locrian");
    // Content: C Ionian should have all natural notes
    assert!(
        stdout.contains("C Ionian: C D E F G A B"),
        "C Ionian should have natural notes"
    );
    // Content: Lydian has raised 4th
    assert!(
        stdout.contains("C Lydian: C D E F#/Gb G A B"),
        "C Lydian should have raised 4th"
    );
}

// ==================== superchords ====================

#[test]
fn golden_superchords_cmaj_size4() {
    let expected = read_golden("superchords_cmaj_size4.txt");
    let out = slonimsky()
        .args([
            "superchords",
            "C",
            "E",
            "G",
            "--min-size",
            "4",
            "--max-size",
            "4",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "superchords output differs from golden file"
    );
    // Structural checks: must contain known superchords of C major triad
    assert!(stdout.contains("C Maj7"), "should contain CMaj7");
    assert!(stdout.contains("C Dom7"), "should contain CDom7");
    assert!(
        stdout.contains("A Min7"),
        "should contain Am7 (shares C,E,G subset)"
    );
    assert!(
        stdout.contains("Total: 9 superchords"),
        "should have exactly 9 size-4 superchords"
    );
}

// ==================== arpeggio-dictionary (structural, not exact) ====================
// Shape ordering from find_chord_shapes() is non-deterministic, so we test
// structure rather than exact golden output.

#[test]
fn golden_arpeggio_dict_cmaj_structure() {
    let out = slonimsky()
        .args([
            "arpeggio-dictionary",
            "C",
            "E",
            "G",
            "--keys",
            "C",
            "--positions",
            "3",
            "--max-span",
            "4",
        ])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Arpeggio Dictionary: [C, E, G]"),
        "header should show input PcSet as notes"
    );
    assert!(
        stdout.contains("Tuning: standard"),
        "should show default tuning"
    );
    assert!(stdout.contains("Key: C"), "should have C key section");
    assert!(
        stdout.contains("Total: 3 shapes across 1 keys"),
        "should show correct totals"
    );
    // Each shape line has a numbered prefix, fret notation, and a "(frets N-M)" range
    let shape_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()) && l.contains("frets"))
        .collect();
    assert_eq!(shape_lines.len(), 3, "should have exactly 3 shape lines");
    // All shape lines should have dash-separated fret notation
    for line in &shape_lines {
        let fret_part = line.trim().split_whitespace().nth(1).unwrap_or("");
        let segments: Vec<&str> = fret_part.split('-').collect();
        assert_eq!(
            segments.len(),
            6,
            "standard tuning = 6 strings, got: {fret_part}"
        );
        for seg in &segments {
            assert!(
                *seg == "x" || seg.parse::<u8>().is_ok(),
                "each fret segment should be 'x' or a number, got: {seg}"
            );
        }
    }
}

// ==================== fretboard (exact SVG golden) ====================

#[test]
fn golden_fretboard_c_chord_svg() {
    let expected = read_golden("fretboard_c_chord.svg");
    let out = slonimsky()
        .args(["fretboard", "x-3-2-0-1-0"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.as_ref(),
        expected.as_str(),
        "fretboard SVG output differs from golden file"
    );
    // Structural checks on the golden SVG itself
    assert!(stdout.starts_with("<svg"), "output should be SVG");
    assert!(stdout.trim_end().ends_with("</svg>"), "SVG should close");
    assert!(
        stdout.contains("<circle"),
        "fretboard SVG should contain note circles"
    );
    assert!(
        stdout.contains("<line"),
        "fretboard SVG should contain fret/string lines"
    );
}

// ==================== chord-dictionary (structural, non-deterministic ordering) ====================

#[test]
fn golden_chord_dictionary_cmaj_structure() {
    let out = slonimsky()
        .args(["chord-dictionary", "C", "E", "G", "--max-results", "5"])
        .output()
        .expect("command should run");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Header
    assert!(
        stdout.contains("Chord dictionary: [C, E, G]"),
        "header should identify the chord, got:\n{stdout}"
    );
    assert!(
        stdout.contains("Tuning: standard"),
        "default tuning should be standard"
    );
    assert!(
        stdout.contains("Max span: 4"),
        "default max span should be 4"
    );
    // Should have at least one shape
    let shape_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| {
            let trimmed = l.trim();
            trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains("frets")
        })
        .collect();
    assert!(
        !shape_lines.is_empty(),
        "should find at least one chord shape"
    );
    assert!(
        shape_lines.len() <= 5,
        "should respect --max-results 5, got {}",
        shape_lines.len()
    );
    // Each shape has valid fret notation (6 dash-separated segments for standard tuning)
    for line in &shape_lines {
        let fret_part = line.trim().split_whitespace().nth(1).unwrap_or("");
        let segments: Vec<&str> = fret_part.split('-').collect();
        assert_eq!(
            segments.len(),
            6,
            "standard tuning = 6 strings in fret notation, got: {fret_part}"
        );
        // At least 3 segments must be non-x (a triad needs at least 3 sounded strings)
        let sounded = segments.iter().filter(|s| **s != "x").count();
        assert!(
            sounded >= 3,
            "C major triad needs at least 3 sounded strings, got {sounded} in: {fret_part}"
        );
    }
    // Total line
    assert!(
        stdout.contains("Total:"),
        "output should include a Total line"
    );
}

#[test]
fn golden_chord_dictionary_cmaj_svg_structure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let svg_path = dir.path().join("chordex.svg");
    let out = slonimsky()
        .args([
            "chord-dictionary",
            "C",
            "E",
            "G",
            "--max-results",
            "3",
            "-o",
            svg_path.to_str().unwrap(),
        ])
        .output()
        .expect("command should run");

    assert!(
        out.status.success(),
        "exit 0, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let svg = std::fs::read_to_string(&svg_path).expect("SVG file should exist");
    assert!(svg.starts_with("<svg"), "output should be SVG");
    assert!(svg.trim_end().ends_with("</svg>"), "SVG should close");
    // A multi-shape grid should have multiple <svg elements (nested viewboxes) or
    // multiple fretboard groups. Check for at least one <circle (note dot) and <line.
    assert!(
        svg.contains("<circle"),
        "chord-dictionary SVG should contain note circles"
    );
    assert!(
        svg.contains("<line"),
        "chord-dictionary SVG should contain lines"
    );
    // Should have at least 200 bytes (a single fretboard SVG is ~2KB)
    assert!(
        svg.len() > 200,
        "SVG should be non-trivial, got {} bytes",
        svg.len()
    );
}
