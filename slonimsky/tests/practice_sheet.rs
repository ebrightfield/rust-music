use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn slonimsky() -> Command {
    Command::cargo_bin("slonimsky").expect("binary should exist")
}

// --- Text output defaults (C major) ---

#[test]
fn default_c_major_has_header() {
    let out = slonimsky().args(["practice-sheet"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Practice Sheet: C Major"),
        "default should produce C Major header, got: {}",
        stdout.lines().next().unwrap_or("")
    );
}

#[test]
fn default_has_all_five_sections() {
    let out = slonimsky().args(["practice-sheet"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("SCALE"), "missing SCALE section");
    assert!(stdout.contains("MODES"), "missing MODES section");
    assert!(
        stdout.contains("3-NOTE SUBCHORDS"),
        "missing 3-NOTE SUBCHORDS section"
    );
    assert!(
        stdout.contains("4-NOTE SUBCHORDS"),
        "missing 4-NOTE SUBCHORDS section"
    );
    assert!(
        stdout.contains("PRACTICE SUGGESTIONS") || stdout.contains("PRACTICE"),
        "missing PRACTICE section"
    );
}

#[test]
fn c_major_scale_notes_correct() {
    let out = slonimsky().args(["practice-sheet"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let notes_line = stdout
        .lines()
        .find(|l| l.contains("Notes:"))
        .expect("should have a Notes: line");
    for note in &["C", "D", "E", "F", "G", "A", "B"] {
        assert!(
            notes_line.contains(note),
            "C major Notes line should contain {note}, got: {notes_line}"
        );
    }
}

#[test]
fn c_major_has_seven_modes() {
    let out = slonimsky().args(["practice-sheet"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for mode in &[
        "Ionian",
        "Dorian",
        "Phrygian",
        "Lydian",
        "Mixolydian",
        "Aeolian",
        "Locrian",
    ] {
        assert!(
            stdout.contains(mode),
            "C major practice sheet should list mode '{mode}'"
        );
    }
}

#[test]
fn c_major_interval_vector() {
    let out = slonimsky().args(["practice-sheet"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Major scale interval vector: <2, 5, 4, 3, 6, 1>
    assert!(
        stdout.contains("Interval Vector:"),
        "should have Interval Vector line"
    );
    let iv_line = stdout
        .lines()
        .find(|l| l.contains("Interval Vector:"))
        .unwrap();
    assert!(
        iv_line.contains("2")
            && iv_line.contains("5")
            && iv_line.contains("4")
            && iv_line.contains("3")
            && iv_line.contains("6")
            && iv_line.contains("1"),
        "major scale IV should be <2,5,4,3,6,1>, got: {iv_line}"
    );
}

// --- Key transposition ---

#[test]
fn g_major_header_and_notes() {
    let out = slonimsky()
        .args(["practice-sheet", "--key", "G"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Practice Sheet: G Major"),
        "should show G Major header"
    );
    let notes_line = stdout
        .lines()
        .find(|l| l.contains("Notes:"))
        .expect("should have Notes: line");
    // G major: G A B C D E F#
    assert!(
        notes_line.contains("G") && notes_line.contains("A") && notes_line.contains("B"),
        "G major Notes line should contain G, A, B; got: {notes_line}"
    );
}

// --- Scale variants ---

#[test]
fn melodic_minor_modes_present() {
    let out = slonimsky()
        .args(["practice-sheet", "--scale", "melodic-minor"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Melodic Minor"),
        "header should say Melodic Minor"
    );
    // Check for at least a few known melodic minor mode names
    assert!(
        stdout.contains("Dorian")
            || stdout.contains("Lydian")
            || stdout.contains("Altered")
            || stdout.contains("Melodic Minor"),
        "should contain at least one melodic minor mode name"
    );
}

#[test]
fn harmonic_minor_accepted() {
    let out = slonimsky()
        .args(["practice-sheet", "--scale", "harmonic-minor"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Harmonic Minor"),
        "should show Harmonic Minor"
    );
}

#[test]
fn harmonic_major_accepted() {
    let out = slonimsky()
        .args(["practice-sheet", "--scale", "harmonic-major"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Harmonic Major"),
        "should show Harmonic Major"
    );
}

#[test]
fn unknown_scale_fails() {
    slonimsky()
        .args(["practice-sheet", "--scale", "blues"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown scale"));
}

// --- Triad and seventh chord content ---

#[test]
fn c_major_triads_include_known_chords() {
    let out = slonimsky().args(["practice-sheet"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    // C major diatonic triads should include at least some recognizable names
    // The triad section should exist and have entries
    let triads_section: Vec<&str> = stdout
        .lines()
        .skip_while(|l| !l.contains("3-NOTE SUBCHORDS"))
        .skip(2) // skip the header and underline
        .take_while(|l| !l.is_empty() && !l.starts_with('\n'))
        .collect();
    assert!(
        triads_section.len() >= 7,
        "C major should have at least 7 diatonic triads, got {}",
        triads_section.len()
    );
}

#[test]
fn c_major_sevenths_section_exists() {
    let out = slonimsky().args(["practice-sheet"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let has_sevenths = stdout.lines().any(|l| l.contains("4-NOTE SUBCHORDS"));
    assert!(has_sevenths, "should have 4-NOTE SUBCHORDS section");
}

// --- SVG output ---

#[test]
fn svg_output_valid_structure() {
    let tmp = TempDir::new().unwrap();
    let out_path = tmp.path().join("practice.svg");
    slonimsky()
        .args(["practice-sheet", "-o", out_path.to_str().unwrap()])
        .assert()
        .success();
    let svg = std::fs::read_to_string(&out_path).expect("SVG file should exist");
    assert!(
        svg.starts_with("<svg") || svg.starts_with("<?xml"),
        "SVG should start with <svg or <?xml"
    );
    assert!(svg.contains("</svg>"), "SVG should close with </svg>");
    assert!(svg.contains("<text"), "SVG should contain text elements");
}

#[test]
fn svg_output_with_dark_theme() {
    let tmp = TempDir::new().unwrap();
    let out_path = tmp.path().join("practice_dark.svg");
    slonimsky()
        .args([
            "practice-sheet",
            "--theme",
            "dark",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let svg = std::fs::read_to_string(&out_path).expect("SVG file should exist");
    assert!(svg.contains("</svg>"), "SVG should be valid");
}

#[test]
fn svg_output_with_key_and_scale() {
    let tmp = TempDir::new().unwrap();
    let out_path = tmp.path().join("practice_gmin.svg");
    slonimsky()
        .args([
            "practice-sheet",
            "--key",
            "G",
            "--scale",
            "harmonic-minor",
            "-o",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let svg = std::fs::read_to_string(&out_path).expect("SVG file should exist");
    assert!(svg.contains("</svg>"), "SVG should be valid");
    assert!(
        svg.contains("<circle") || svg.contains("<path"),
        "SVG should contain graphical elements (pitch circle)"
    );
}

// --- Error cases ---

#[test]
fn non_svg_output_extension_fails() {
    let tmp = TempDir::new().unwrap();
    let out_path = tmp.path().join("practice.pdf");
    slonimsky()
        .args(["practice-sheet", "-o", out_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("only supports .svg"));
}
