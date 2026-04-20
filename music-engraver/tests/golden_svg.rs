//! Golden-SVG visual regression tests.
//!
//! Each test function builds a deterministic score via ScoreBuilder,
//! renders it to SVG, and compares the result against a frozen baseline
//! stored in `tests/golden/*.svg`. If the baseline does not exist, the
//! test creates it and fails with a message to review and commit. If the
//! baseline exists but differs, the test fails with a line-level diff.
//!
//! To regenerate all baselines (e.g. after an intentional rendering change):
//!   GOLDEN_UPDATE=1 cargo test -p music-engraver --test golden_svg

use std::path::PathBuf;

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::articulation::Articulation;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::grace::GraceNoteKind;
use music_engraver::layout::hairpin::HairpinType;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::layout::rehearsal::RehearsalStyle;
use music_engraver::layout::lyric::LyricSyllable;
use music_engraver::layout::tempo::{MetronomeNoteKind, TempoMark};
use music_engraver::score::multi_staff::MultiStaffScore;
use music_engraver::score::ScoreBuilder;

fn p(name: &str, octave: u8) -> Pitch {
    let note = match name {
        "C" => Note::C,
        "D" => Note::D,
        "E" => Note::E,
        "F" => Note::F,
        "G" => Note::G,
        "A" => Note::A,
        "B" => Note::B,
        "F#" => Note::Fis,
        "Bb" => Note::Bes,
        "C#" => Note::Cis,
        "Eb" => Note::Ees,
        "Ab" => Note::Aes,
        _ => panic!("unknown note: {name}"),
    };
    Pitch::new(note, octave).expect("valid pitch")
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("golden")
}

/// Compare SVG output against a frozen baseline. If GOLDEN_UPDATE=1,
/// write the new output without failing.
fn assert_golden(name: &str, actual_svg: &str) {
    let path = golden_dir().join(format!("{name}.svg"));
    let update = std::env::var("GOLDEN_UPDATE").is_ok_and(|v| v == "1");

    if update || !path.exists() {
        std::fs::write(&path, actual_svg)
            .unwrap_or_else(|e| panic!("failed to write golden file {}: {e}", path.display()));
        if !update {
            panic!(
                "Golden baseline created at {}. Review the SVG and commit it. \
                 Re-run the test without GOLDEN_UPDATE to verify.",
                path.display()
            );
        }
        return;
    }

    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read golden file {}: {e}", path.display()));

    if actual_svg == expected {
        return;
    }

    // Produce a readable line-level diff
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual_svg.lines().collect();
    let max_lines = expected_lines.len().max(actual_lines.len());

    let mut diff = String::new();
    diff.push_str(&format!(
        "Golden SVG mismatch for '{name}' ({}):\n",
        path.display()
    ));
    diff.push_str(&format!(
        "Expected {} lines, got {} lines.\n\n",
        expected_lines.len(),
        actual_lines.len()
    ));

    let mut diff_count = 0;
    for i in 0..max_lines {
        let exp = expected_lines.get(i).copied().unwrap_or("<EOF>");
        let act = actual_lines.get(i).copied().unwrap_or("<EOF>");
        if exp != act {
            diff_count += 1;
            diff.push_str(&format!("Line {i}: \n  expected: {exp}\n  actual:   {act}\n"));
            if diff_count >= 20 {
                diff.push_str("... (truncated, more differences follow)\n");
                break;
            }
        }
    }

    diff.push_str(&format!(
        "\nTo update the baseline: GOLDEN_UPDATE=1 cargo test -p music-engraver --test golden_svg -- {name}"
    ));

    panic!("{diff}");
}

// ---------------------------------------------------------------------------
// Golden test cases
// ---------------------------------------------------------------------------

/// Single measure: clef + key sig + time sig + 4 quarter notes + barline.
fn build_simple_scale() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F#", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Multi-system: 4 measures across 2 systems with mixed note values.
fn build_multi_system() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("C", 4), Duration::HALF)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .barline()
        .note(p("A", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .rest(Duration::QTR)
        .barline()
        .rest(Duration::HALF)
        .note(p("G", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Chords with accidentals and second-avoidance.
fn build_chords() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::HALF)
        .chord(vec![p("D", 4), p("F", 4), p("A", 4)], Duration::HALF)
        .barline()
        .chord(vec![p("E", 4), p("F", 4)], Duration::QTR)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4), p("C", 5)], Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Beam groups: eighths and sixteenths.
fn build_beams() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .beam_group(vec![
            (p("C", 4), Duration::EIGHTH),
            (p("D", 4), Duration::EIGHTH),
            (p("E", 4), Duration::EIGHTH),
            (p("F", 4), Duration::EIGHTH),
        ])
        .beam_group(vec![
            (p("G", 4), Duration::SIXTEENTH),
            (p("A", 4), Duration::SIXTEENTH),
            (p("B", 4), Duration::SIXTEENTH),
            (p("C", 5), Duration::SIXTEENTH),
        ])
        .end_barline()
        .render_svg()
}

/// Ties within and across barlines.
fn build_ties() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("E", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .tie()
        .barline()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::HALF)
        .note(p("G", 4), Duration::QTR)
        .tie()
        .end_barline()
        .render_svg()
}

/// Dynamics and hairpins.
fn build_dynamics() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .cresc()
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .dynamic(Dynamic::Forte)
        .hairpin_end()
        .end_barline()
        .render_svg()
}

/// Tuplet bracket (triplet).
fn build_tuplet() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .tuplet(
            3,
            vec![
                (p("C", 4), Duration::EIGHTH),
                (p("E", 4), Duration::EIGHTH),
                (p("G", 4), Duration::EIGHTH),
            ],
        )
        .note(p("C", 5), Duration::QTR)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Slurs spanning notes.
fn build_slurs() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .slur_start()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .slur_end()
        .note(p("C", 5), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Articulations on notes.
fn build_articulations() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .note(p("F", 4), Duration::QTR)
        .articulation(Articulation::Tenuto)
        .note(p("G", 4), Duration::QTR)
        .articulation(Articulation::Accent)
        .note(p("A", 4), Duration::QTR)
        .articulation(Articulation::Marcato)
        .end_barline()
        .render_svg()
}

/// Grace note before a principal note.
fn build_grace_notes() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::HALF)
        .grace_note(p("D", 4), GraceNoteKind::Acciaccatura)
        .note(p("C", 4), Duration::QTR)
        .grace_note(p("B", 3), GraceNoteKind::Appoggiatura)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Rehearsal marks and tempo marks.
fn build_annotations() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .rehearsal_mark("A".to_string(), RehearsalStyle::Boxed)
        .tempo(TempoMark::TextWithMetronome {
            text: "Allegro".to_string(),
            note_kind: MetronomeNoteKind::Quarter,
            dotted: false,
            bpm: 120,
        })
        .note(p("D", 4), Duration::QTR)
        .expression("dolce".to_string())
        .note(p("E", 4), Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Bass clef with flats key signature.
fn build_bass_clef() -> String {
    ScoreBuilder::new()
        .clef(Clef::Bass)
        .key_signature(KeySignature::Flats(3))
        .time_signature(4, 4)
        .note(p("C", 3), Duration::QTR)
        .note(p("Eb", 3), Duration::QTR)
        .note(p("G", 3), Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Auto line breaking with mixed-density measures.
fn build_auto_breaks() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(1))
        .time_signature(4, 4)
        .auto_line_breaks()
        // Measure 1: whole note (sparse)
        .note(p("G", 4), Duration::WHOLE)
        .barline()
        // Measure 2: 4 quarters
        .note(p("A", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .barline()
        // Measure 3: 8 eighths (dense)
        .beam_group(vec![
            (p("E", 5), Duration::EIGHTH),
            (p("D", 5), Duration::EIGHTH),
            (p("C", 5), Duration::EIGHTH),
            (p("B", 4), Duration::EIGHTH),
        ])
        .beam_group(vec![
            (p("A", 4), Duration::EIGHTH),
            (p("G", 4), Duration::EIGHTH),
            (p("F#", 4), Duration::EIGHTH),
            (p("E", 4), Duration::EIGHTH),
        ])
        .barline()
        // Measure 4: half + half
        .note(p("D", 4), Duration::HALF)
        .note(p("G", 4), Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Grand staff (piano): treble + bass clef with brace connector.
fn build_grand_staff() -> String {
    let treble = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("F#", 5), Duration::QTR)
        .note(p("G", 5), Duration::QTR)
        .barline()
        .note(p("A", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .end_barline();

    let bass = ScoreBuilder::new()
        .clef(Clef::Bass)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .note(p("D", 3), Duration::WHOLE)
        .barline()
        .note(p("A", 2), Duration::HALF)
        .note(p("D", 3), Duration::HALF)
        .end_barline();

    MultiStaffScore::grand_staff(treble, bass).render_svg()
}

/// Lyrics: syllables with hyphens and extenders under notes.
fn build_lyrics() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("Hap"))
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::word("py"))
        .note(p("D", 4), Duration::QTR)
        .lyric(LyricSyllable::with_hyphen("birth"))
        .note(p("C", 4), Duration::QTR)
        .lyric(LyricSyllable::word("day"))
        .barline()
        .note(p("F", 4), Duration::HALF)
        .lyric(LyricSyllable::with_extender("to"))
        .note(p("E", 4), Duration::HALF)
        .lyric(LyricSyllable::word("you!"))
        .end_barline()
        .render_svg()
}

/// Ornaments above notes: trill, mordent, inverted mordent, turn.
fn build_ornaments() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .ornament(Ornament::Trill)
        .note(p("F", 4), Duration::QTR)
        .ornament(Ornament::Mordent)
        .note(p("G", 4), Duration::QTR)
        .ornament(Ornament::InvertedMordent)
        .note(p("A", 4), Duration::QTR)
        .ornament(Ornament::Turn)
        .end_barline()
        .render_svg()
}

/// Hairpins (crescendo + decrescendo) spanning notes.
fn build_hairpins() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .hairpin_start(HairpinType::Crescendo)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Forte)
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Cross-system ties: tie from end of system 1 to start of system 2.
fn build_cross_system_ties() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::HALF)
        .barline()
        .note(p("G", 4), Duration::HALF)
        .note(p("G", 4), Duration::HALF)
        .tie()
        .barline()
        .note(p("G", 4), Duration::WHOLE)
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

/// Expression text (italic) below the staff.
fn build_expression_text() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .expression("dolce".to_string())
        .note(p("F", 4), Duration::QTR)
        .note(p("G", 4), Duration::HALF)
        .expression("cantabile".to_string())
        .end_barline()
        .render_svg()
}

/// Chord symbols above notes.
fn build_chord_symbols() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .chord_symbol("C")
        .note(p("A", 3), Duration::QTR)
        .chord_symbol("Am")
        .note(p("F", 4), Duration::QTR)
        .chord_symbol("F")
        .note(p("G", 4), Duration::QTR)
        .chord_symbol("G7")
        .barline()
        .note(p("C", 4), Duration::WHOLE)
        .chord_symbol("Cmaj7")
        .end_barline()
        .render_svg()
}

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

#[test]
fn golden_simple_scale() {
    assert_golden("simple_scale", &build_simple_scale());
}

#[test]
fn golden_multi_system() {
    assert_golden("multi_system", &build_multi_system());
}

#[test]
fn golden_chords() {
    assert_golden("chords", &build_chords());
}

#[test]
fn golden_beams() {
    assert_golden("beams", &build_beams());
}

#[test]
fn golden_ties() {
    assert_golden("ties", &build_ties());
}

#[test]
fn golden_dynamics() {
    assert_golden("dynamics", &build_dynamics());
}

#[test]
fn golden_tuplet() {
    assert_golden("tuplet", &build_tuplet());
}

#[test]
fn golden_slurs() {
    assert_golden("slurs", &build_slurs());
}

#[test]
fn golden_articulations() {
    assert_golden("articulations", &build_articulations());
}

#[test]
fn golden_grace_notes() {
    assert_golden("grace_notes", &build_grace_notes());
}

#[test]
fn golden_annotations() {
    assert_golden("annotations", &build_annotations());
}

#[test]
fn golden_bass_clef() {
    assert_golden("bass_clef", &build_bass_clef());
}

#[test]
fn golden_auto_breaks() {
    assert_golden("auto_breaks", &build_auto_breaks());
}

#[test]
fn golden_grand_staff() {
    assert_golden("grand_staff", &build_grand_staff());
}

#[test]
fn golden_lyrics() {
    assert_golden("lyrics", &build_lyrics());
}

#[test]
fn golden_chord_symbols() {
    assert_golden("chord_symbols", &build_chord_symbols());
}

#[test]
fn golden_ornaments() {
    assert_golden("ornaments", &build_ornaments());
}

#[test]
fn golden_hairpins() {
    assert_golden("hairpins", &build_hairpins());
}

#[test]
fn golden_cross_system_ties() {
    assert_golden("cross_system_ties", &build_cross_system_ties());
}

#[test]
fn golden_expression_text() {
    assert_golden("expression_text", &build_expression_text());
}

/// Verify all golden baselines are valid SVGs with expected structure.
#[test]
fn golden_baselines_are_valid_svgs() {
    let names = [
        "simple_scale",
        "multi_system",
        "chords",
        "beams",
        "ties",
        "dynamics",
        "tuplet",
        "slurs",
        "articulations",
        "grace_notes",
        "annotations",
        "bass_clef",
        "auto_breaks",
        "grand_staff",
        "lyrics",
        "chord_symbols",
        "ornaments",
        "hairpins",
        "cross_system_ties",
        "expression_text",
    ];

    for name in &names {
        let path = golden_dir().join(format!("{name}.svg"));
        if !path.exists() {
            continue; // Will be caught by the individual test
        }
        let svg = std::fs::read_to_string(&path).unwrap();
        assert!(
            svg.starts_with("<svg"),
            "Golden file {name}.svg should start with <svg"
        );
        assert!(
            svg.contains("</svg>"),
            "Golden file {name}.svg should contain closing </svg>"
        );
        assert!(
            svg.contains("xmlns"),
            "Golden file {name}.svg should contain xmlns attribute"
        );
    }
}
