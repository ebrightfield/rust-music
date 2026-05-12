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
use music_engraver::layout::arpeggio::ArpeggioDirection;
use music_engraver::layout::breath::BreathMark;
use music_engraver::layout::articulation::Articulation;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::glissando::GlissandoStyle;
use music_engraver::layout::grace::GraceNoteKind;
use music_engraver::layout::hairpin::HairpinType;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::navigation::NavigationSign;
use music_engraver::layout::ornament::Ornament;
use music_engraver::layout::ottava::OttavaKind;
use music_engraver::layout::tremolo::TremoloCount;
use music_engraver::layout::rehearsal::RehearsalStyle;
use music_engraver::layout::lyric::LyricSyllable;
use music_engraver::layout::tempo::{MetronomeNoteKind, TempoMark};
use music_engraver::score::multi_staff::MultiStaffScore;
use music_engraver::layout::tab_bend::BendAmount;
use music_engraver::score::tab::TabScoreBuilder;
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
        .articulation(Articulation::Staccato)
        .articulation(Articulation::Fermata)
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

fn build_grace_note_slur() -> String {
    // Mirrors the gestures in `build_grace_notes` but uses `grace_note_slur`
    // so the rendered SVG must include slur crescents in addition to grace
    // glyphs. Adds a chord with a slurred acciaccatura to exercise the
    // "attach to closest chord note" rule.
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::HALF)
        .grace_note_slur(p("D", 4), GraceNoteKind::Acciaccatura)
        .note(p("C", 4), Duration::QTR)
        .grace_note_slur(p("B", 3), GraceNoteKind::Appoggiatura)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::QTR)
        .grace_note_slur(p("B", 3), GraceNoteKind::Acciaccatura)
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

/// Optimal line breaking: same content as auto_breaks but uses DP-optimal.
fn build_optimal_breaks() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(1))
        .time_signature(4, 4)
        .optimal_line_breaks()
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

/// Multi-staff (grand staff) with cross-system ties and slurs.
/// 4 measures across 2 systems, treble has tie across system break,
/// bass has slur across system break.
fn build_multi_staff_cross_system() -> String {
    let treble = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("F#", 5), Duration::QTR)
        .note(p("G", 5), Duration::QTR)
        .barline()
        // Last note of system 1 — tie forward across system break
        .note(p("A", 5), Duration::HALF)
        .note(p("A", 5), Duration::HALF)
        .tie()
        .barline()
        // First note of system 2 — tie target
        .note(p("A", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .barline()
        .note(p("E", 5), Duration::WHOLE)
        .end_barline();

    let bass = ScoreBuilder::new()
        .clef(Clef::Bass)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("D", 3), Duration::WHOLE)
        .barline()
        // Slur across system break
        .note(p("A", 2), Duration::HALF)
        .slur_start()
        .note(p("B", 2), Duration::HALF)
        .barline()
        .note(p("D", 3), Duration::HALF)
        .slur_end()
        .note(p("A", 2), Duration::HALF)
        .barline()
        .note(p("D", 3), Duration::WHOLE)
        .end_barline();

    MultiStaffScore::grand_staff(treble, bass).render_svg()
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
fn golden_grace_note_slur() {
    let svg = build_grace_note_slur();
    assert_golden("grace_note_slur", &svg);

    // Structural assertions independent of the byte-for-byte baseline:
    // three grace_note_slur calls should each emit one slur crescent
    // (stroke="none" filled path) — exactly 3 slur paths in this score.
    let slur_path_count = svg.matches(r#"stroke="none""#).count();
    assert_eq!(
        slur_path_count, 3,
        "expected exactly 3 grace-slur paths, got {slur_path_count}"
    );
    // And the result must differ from the plain grace_notes baseline.
    let plain = build_grace_notes();
    assert_ne!(svg, plain, "slurred grace score must differ from plain");
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
fn golden_optimal_breaks() {
    assert_golden("optimal_breaks", &build_optimal_breaks());
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

#[test]
fn golden_multi_staff_cross_system() {
    assert_golden(
        "multi_staff_cross_system",
        &build_multi_staff_cross_system(),
    );
}

fn build_tab_score() -> String {
    TabScoreBuilder::guitar()
        .measures_per_system(2)
        // Measure 1: E minor arpeggio
        .fret(6, 0).next()
        .fret(5, 2).next()
        .fret(4, 2).next()
        .fret(3, 0)
        .barline()
        // Measure 2: scale on string 1
        .fret(1, 0).next()
        .fret(1, 3).next()
        .fret(1, 5).next()
        .fret(1, 7)
        .barline()
        // Measure 3: power chord
        .fret(6, 0)
        .fret(5, 2)
        .fret(4, 2)
        .next()
        .fret(6, 3)
        .fret(5, 5)
        .fret(4, 5)
        .barline()
        // Measure 4: high frets
        .fret(1, 12).next()
        .fret(2, 12).next()
        .fret(1, 15).next()
        .fret(1, 17)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_score() {
    assert_golden("tab_score", &build_tab_score());
}

fn build_tab_slides() -> String {
    TabScoreBuilder::guitar()
        .measures_per_system(2)
        .system_width_fu(12000.0)
        // Measure 1: ascending slide on string 1
        .fret(1, 5).slide().next()
        .fret(1, 7).next()
        .fret(1, 3).next()
        .fret(1, 0)
        .barline()
        // Measure 2: descending slide on string 2
        .fret(2, 12).slide().next()
        .fret(2, 9).next()
        .fret(2, 7).next()
        .fret(1, 5)
        .barline()
        // Measure 3: chord slide (power chord shift)
        .fret(6, 3).fret(5, 5).fret(4, 5).slide().next()
        .fret(6, 5).fret(5, 7).fret(4, 7).next()
        .rest().next()
        .fret(6, 0)
        .barline()
        // Measure 4: consecutive slides (chain)
        .fret(1, 5).slide().next()
        .fret(1, 7).slide().next()
        .fret(1, 9).slide().next()
        .fret(1, 12)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_slides() {
    assert_golden("tab_slides", &build_tab_slides());
}

fn build_tab_hammer_pull() -> String {
    TabScoreBuilder::guitar()
        .measures_per_system(2)
        // Measure 1: hammer-on 5→7
        .fret(1, 5).hammer().next().fret(1, 7).next().rest().next().rest()
        .barline()
        // Measure 2: pull-off 7→5
        .fret(1, 7).pull().next().fret(1, 5).next().rest().next().rest()
        .barline()
        // Measure 3: chain 5→7→5
        .fret(2, 5).hammer().next().fret(2, 7).pull().next().fret(2, 5).next().rest()
        .barline()
        // Measure 4: multi-string chord hammer
        .fret(5, 5).fret(4, 7).fret(3, 7).hammer().next()
        .fret(5, 7).fret(4, 9).fret(3, 9)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_hammer_pull() {
    assert_golden("tab_hammer_pull", &build_tab_hammer_pull());
}

fn build_tab_bends() -> String {
    TabScoreBuilder::guitar()
        .measures_per_system(2)
        .system_width_fu(10000.0)
        // Measure 1: full bend, half bend
        .fret(2, 8)
        .bend(BendAmount::Full)
        .next()
        .fret(1, 7)
        .bend(BendAmount::Half)
        .next()
        .fret(3, 9)
        .next()
        .fret(1, 5)
        .barline()
        // Measure 2: quarter bend, chord bend
        .fret(3, 7)
        .bend(BendAmount::Quarter)
        .next()
        .fret(1, 10)
        .fret(2, 10)
        .bend(BendAmount::Full)
        .next()
        .fret(1, 12)
        .next()
        .fret(1, 7)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_bends() {
    assert_golden("tab_bends", &build_tab_bends());
}

fn build_volta_brackets() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(0))
        .time_signature(4, 4)
        .measures_per_system(3)
        // Measure 1: plain opening
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // Measure 2: single-measure 1st ending (both hooks)
        .volta_start("1.")
        .note(p("G", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .volta_end()
        .barline()
        // Measure 3: single-measure 2nd ending (both hooks)
        .volta_start("2.")
        .note(p("A", 4), Duration::WHOLE)
        .volta_end()
        .barline()
        // Measure 4 (system 2): multi-measure 3rd ending start
        .volta_start("3.")
        .note(p("B", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .barline()
        // Measure 5: multi-measure continuation + end
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .volta_end()
        .barline()
        // Measure 6: plain closing
        .note(p("C", 4), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_volta_brackets() {
    assert_golden("volta_brackets", &build_volta_brackets());
}

fn build_cross_system_volta() -> String {
    // A multi-measure volta "1." that spans the system break (measures 2-4,
    // system break after measure 2). Demonstrates trailing bracket on system 1
    // and continuation bracket on system 2.
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(0))
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: plain opening
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // Measure 2: start of multi-measure volta "1." (left hook + text)
        // This is the LAST measure of system 1 — the volta must continue
        // across the system break to system 2.
        .volta_start("1.")
        .note(p("G", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .barline()
        // --- system break here (measures_per_system=2) ---
        // Measure 3: volta continuation (no hooks, no text — top line only)
        .note(p("B", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .barline()
        // Measure 4: volta end (right hook — closes the bracket)
        .note(p("G", 4), Duration::WHOLE)
        .volta_end()
        .barline()
        // Measure 5-6 (system 3): second ending
        .volta_start("2.")
        .note(p("C", 5), Duration::WHOLE)
        .volta_end()
        .barline()
        .note(p("C", 4), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_cross_system_volta() {
    let svg = build_cross_system_volta();

    // Verify structural properties before freezing as golden baseline:

    // "1." label should appear on system 1 (measure 2, the volta start)
    assert!(svg.contains(">1.<"), "should contain volta label '1.'");
    // "2." label should appear on system 3 (measure 5)
    assert!(svg.contains(">2.<"), "should contain volta label '2.'");

    // The multi-measure volta spans measures 2-4 across system 1→2.
    // System 1 has measure 2 with LeftOnly (left hook + text + top line = 2 lines).
    // System 2 has measure 3 with Neither (top line only = 1 line) and
    //          measure 4 with RightOnly (top line + right hook = 2 lines).
    // System 3 has measure 5 with Both (left hook + text + top line + right hook = 3 lines).
    // Total volta bracket lines: 2 + 1 + 2 + 3 = 8 extra lines beyond staff/stem.

    // Count total lines — staff lines (5 per system × 3 systems = 15) + stems + volta lines
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 23,
        "expected at least 23 lines (15 staff + stems + volta brackets), got {line_count}"
    );

    assert_golden("cross_system_volta", &svg);
}

fn build_guitar_tab() -> String {
    let notation = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .barline()
        .note(p("E", 5), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("G", 4), Duration::HALF)
        .end_barline();

    let tab = TabScoreBuilder::guitar()
        .quarter().fret(1, 0).next()
        .quarter().fret(3, 0).next()
        .quarter().fret(2, 0).next()
        .quarter().fret(1, 5)
        .barline()
        .quarter().fret(1, 5).next()
        .quarter().fret(2, 0).next()
        .half().fret(3, 0)
        .end_barline();

    MultiStaffScore::guitar_tab(notation, tab)
        .system_width_fu(10000.0)
        .render_svg()
}

#[test]
fn golden_guitar_tab() {
    let svg = build_guitar_tab();

    // Notation staff has 5 lines; tab staff has 6 lines
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 11,
        "should have at least 11 staff lines (5+6), got {line_count}"
    );

    // Tab should show fret numbers
    assert!(svg.contains(">0</text>"), "should show fret 0");
    assert!(svg.contains(">5</text>"), "should show fret 5");

    // Should have both notation and tab content
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 2,
        "should have treble clef + TAB clef paths, got {path_count}"
    );

    // Fret numbers have dominant-baseline="central" (tab fret centering)
    assert!(
        svg.contains("dominant-baseline=\"central\""),
        "tab fret numbers should use central baseline"
    );

    assert_golden("guitar_tab", &svg);
}

/// Navigation signs (segno, coda, coda square) above staff.
fn build_navigation_signs() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        // Segno on first note, coda on last
        .note(p("C", 4), Duration::QTR)
        .navigation_sign(NavigationSign::Segno)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .navigation_sign(NavigationSign::Coda)
        .note(p("F", 4), Duration::QTR)
        .navigation_sign(NavigationSign::CodaSquare)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_navigation_signs() {
    assert_golden("navigation_signs", &build_navigation_signs());
}

fn build_ottava_brackets() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: 8va over high notes
        .note(p("C", 6), Duration::QTR)
        .ottava_start(OttavaKind::Ottava8va)
        .note(p("D", 6), Duration::QTR)
        .note(p("E", 6), Duration::QTR)
        .ottava_end()
        .note(p("F", 5), Duration::QTR)
        .barline()
        // Measure 2: 8vb under low notes
        .note(p("C", 3), Duration::HALF)
        .ottava_start(OttavaKind::Ottava8vb)
        .note(p("B", 2), Duration::HALF)
        .ottava_end()
        .end_barline()
        .render_svg()
}

#[test]
fn golden_ottava_brackets() {
    let svg = build_ottava_brackets();
    assert!(svg.contains(">8va</text>"), "should contain 8va label");
    assert!(svg.contains(">8vb</text>"), "should contain 8vb label");
    assert!(svg.contains("stroke-dasharray"), "should contain dashed lines");
    assert_golden("ottava_brackets", &svg);
}

fn build_cross_system_ottava() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: normal opening
        .note(p("C", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        // Start 8va near end of system 1
        .note(p("C", 6), Duration::QTR)
        .ottava_start(OttavaKind::Ottava8va)
        .barline()
        // Measure 2: still in system 1
        .note(p("D", 6), Duration::HALF)
        .note(p("E", 6), Duration::HALF)
        .barline()
        // Measure 3: system 2 — end 8va
        .note(p("F", 6), Duration::QTR)
        .note(p("G", 6), Duration::QTR)
        .ottava_end()
        .note(p("A", 5), Duration::HALF)
        .barline()
        // Measure 4: normal closing
        .note(p("C", 5), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_cross_system_ottava() {
    let svg = build_cross_system_ottava();
    // Cross-system ottava: trailing half-bracket in system 1 + incoming in system 2
    let count_8va = svg.matches("8va").count();
    assert!(
        count_8va >= 2,
        "cross-system ottava should produce at least 2 '8va' labels, got {count_8va}"
    );
    assert!(svg.contains("stroke-dasharray"), "should contain dashed lines");
    assert_golden("cross_system_ottava", &svg);
}

fn build_pedal_marks() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        // Pedal down on first note, up on last
        .note(p("C", 4), Duration::QTR)
        .pedal_down()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .pedal_up()
        .end_barline()
        .render_svg()
}

#[test]
fn golden_pedal_marks() {
    let svg = build_pedal_marks();
    // Pedal down (Ped.) and pedal up (*) produce 2 extra paths
    let base_svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .end_barline()
        .render_svg();
    let pedal_paths = svg.matches("<path").count();
    let base_paths = base_svg.matches("<path").count();
    assert_eq!(
        pedal_paths,
        base_paths + 2,
        "pedal marks should add exactly 2 paths (Ped. + *)"
    );
    assert_golden("pedal_marks", &svg);
}

fn build_tab_pre_bends() -> String {
    TabScoreBuilder::guitar()
        .measures_per_system(2)
        .system_width_fu(10000.0)
        // Measure 1: full pre-bend, half pre-bend
        .fret(2, 8)
        .pre_bend(BendAmount::Full)
        .next()
        .fret(1, 7)
        .pre_bend(BendAmount::Half)
        .next()
        .fret(3, 9)
        .next()
        .fret(1, 5)
        .barline()
        // Measure 2: pre-bend→release sequence, chord pre-bend
        .fret(1, 7)
        .pre_bend(BendAmount::Full)
        .next()
        .fret(1, 5)
        .release()
        .next()
        .fret(1, 10)
        .fret(2, 10)
        .pre_bend(BendAmount::Half)
        .next()
        .fret(1, 12)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_pre_bends() {
    let svg = build_tab_pre_bends();
    // Pre-bends produce straight vertical arrows (lines), not curves
    assert!(svg.contains(">full</text>"), "should have 'full' pre-bend label");
    assert!(svg.contains(">1/2</text>"), "should have '1/2' pre-bend label");
    // Release bends produce downward curve paths with arrowheads
    let filled_paths = svg.matches("fill=\"black\"").count();
    assert!(
        filled_paths >= 4,
        "should have at least 4 filled arrowheads (pre-bends + release), got {filled_paths}"
    );
    assert_golden("tab_pre_bends", &svg);
}

/// Tremolo slashes on stems: single, double, triple across notes.
fn build_tremolo() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .tremolo(TremoloCount::Single)
        .note(p("E", 4), Duration::QTR)
        .tremolo(TremoloCount::Double)
        .note(p("G", 4), Duration::QTR)
        .tremolo(TremoloCount::Triple)
        .note(p("B", 4), Duration::QTR)
        .tremolo(TremoloCount::Single)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tremolo() {
    let svg = build_tremolo();
    // 4 notes + 1 clef + 4 tremolo glyphs = at least 9 paths
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 9,
        "expected at least 9 paths (notes + clef + tremolo), got {path_count}"
    );
    // Each tremolo glyph is a distinct path; verify all 3 glyph types differ
    // by checking that the SVG contains more paths than just notes+clef
    let svg_without_tremolo = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let paths_without = svg_without_tremolo.matches("<path").count();
    assert!(
        path_count > paths_without,
        "tremolo version ({path_count} paths) should have more paths than plain ({paths_without})"
    );
    assert_golden("tremolo", &svg);
}

fn build_tab_vibrato() -> String {
    TabScoreBuilder::guitar()
        // Measure 1: normal vibrato on single notes
        .quarter()
        .fret(1, 5)
        .vibrato()
        .next()
        .fret(1, 7)
        .vibrato()
        .next()
        .fret(2, 5)
        .vibrato()
        .next()
        .fret(1, 5)
        .barline()
        // Measure 2: wide vibrato + chord vibrato
        .fret(1, 12)
        .wide_vibrato()
        .next()
        .fret(3, 9)
        .wide_vibrato()
        .next()
        .fret(1, 7)
        .fret(2, 7)
        .vibrato()
        .next()
        .fret(1, 5)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_vibrato() {
    let svg = build_tab_vibrato();
    // Should have vibrato wave paths (unfilled strokes with Q commands)
    let wave_count = svg.matches("fill=\"none\" stroke=\"black\"").count();
    assert!(
        wave_count >= 5,
        "should have at least 5 vibrato waves, got {wave_count}"
    );
    assert_golden("tab_vibrato", &svg);
}

fn build_tab_harmonics() -> String {
    TabScoreBuilder::guitar()
        // Measure 1: natural harmonics at fret 12
        .fret(1, 12)
        .harmonic()
        .next()
        .fret(2, 12)
        .harmonic()
        .next()
        .fret(3, 12)
        .harmonic()
        .next()
        .fret(4, 12)
        .harmonic()
        .barline()
        // Measure 2: chord harmonic at fret 7 + single harmonics
        .fret(1, 7)
        .fret(2, 7)
        .fret(3, 7)
        .harmonic()
        .next()
        .fret(1, 5)
        .harmonic()
        .next()
        .fret(6, 12)
        .harmonic()
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_harmonics() {
    let svg = build_tab_harmonics();
    // Should have harmonic glyph paths (filled black with scale(0.6))
    let scale_count = svg.matches("scale(0.6)").count();
    assert!(
        scale_count >= 8,
        "should have at least 8 harmonic indicators, got {scale_count}"
    );
    assert_golden("tab_harmonics", &svg);
}

fn build_tab_palm_mute() -> String {
    TabScoreBuilder::guitar()
        // Measure 1: two palm-muted + two normal
        .quarter()
        .fret(6, 0)
        .fret(5, 2)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .fret(5, 2)
        .palm_mute()
        .next()
        .quarter()
        .fret(1, 3)
        .next()
        .quarter()
        .fret(1, 5)
        .barline()
        // Measure 2: four consecutive palm-muted (long dashed line)
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .end_barline()
        .measures_per_system(2)
        .render_svg()
}

#[test]
fn golden_tab_palm_mute() {
    let svg = build_tab_palm_mute();
    // Should have P.M. text elements
    let pm_count = svg.matches("P.M.").count();
    assert!(
        pm_count >= 6,
        "should have at least 6 P.M. text annotations, got {pm_count}"
    );
    // Should have dashed continuation lines for consecutive palm mutes
    assert!(
        svg.contains("stroke-dasharray"),
        "consecutive palm mutes should produce dashed lines"
    );
    assert_golden("tab_palm_mute", &svg);
}

#[test]
fn golden_tab_muted_strings() {
    let svg = TabScoreBuilder::guitar()
        // All-mute percussive strum
        .mute(1)
        .mute(2)
        .mute(3)
        .mute(4)
        .mute(5)
        .mute(6)
        .next()
        // Power chord with muted high strings
        .fret(6, 0)
        .fret(5, 2)
        .mute(3)
        .mute(2)
        .mute(1)
        .next()
        // Normal frets for contrast
        .fret(1, 5)
        .fret(2, 3)
        .barline()
        // Individual mutes in sequence
        .mute(6)
        .next()
        .mute(5)
        .next()
        // Mixed fret + mute
        .fret(6, 3)
        .mute(1)
        .mute(2)
        .next()
        .fret(1, 12)
        .mute(6)
        .end_barline()
        .measures_per_system(2)
        .render_svg();

    // Verify muted string "x" markers
    let x_count = svg.matches(">x</text>").count();
    assert!(
        x_count >= 12,
        "should have at least 12 'x' markers, got {x_count}"
    );

    // Verify fret numbers coexist with mutes
    assert!(svg.contains(">0</text>"), "fret 0 should be present");
    assert!(svg.contains(">12</text>"), "fret 12 should be present");

    // Each x marker gets a white background rect
    let rect_count = svg.matches("<rect ").count();
    assert!(
        rect_count >= x_count,
        "each 'x' marker should have a background rect"
    );

    // Verify muted score differs from unmuted
    let no_mute = TabScoreBuilder::guitar()
        .fret(6, 0)
        .fret(5, 2)
        .end_barline()
        .render_svg();
    assert_ne!(svg, no_mute, "muted should differ from unmuted");

    assert_golden("tab_muted_strings", &svg);
}

#[test]
fn golden_tab_let_ring() {
    let svg = TabScoreBuilder::guitar()
        // Measure 1: Arpeggio with let ring (dashed continuation)
        .quarter()
        .fret(6, 0)
        .let_ring()
        .next()
        .quarter()
        .fret(5, 2)
        .let_ring()
        .next()
        .quarter()
        .fret(4, 2)
        .let_ring()
        .next()
        .quarter()
        .fret(3, 1)
        .let_ring()
        .barline()
        // Measure 2: Single let ring on chord, then normal
        .half()
        .fret(6, 0)
        .fret(5, 2)
        .fret(4, 2)
        .let_ring()
        .next()
        .quarter()
        .fret(1, 3)
        .next()
        .quarter()
        .fret(1, 5)
        .end_barline()
        .measures_per_system(2)
        .render_svg();

    // Should have "let ring" italic text elements
    let lr_count = svg.matches("let ring").count();
    assert!(
        lr_count >= 4,
        "should have at least 4 'let ring' annotations, got {lr_count}"
    );

    // Should be italic
    assert!(
        svg.contains("italic"),
        "let ring text should be rendered in italic"
    );

    // Should have dashed continuation lines for consecutive let ring events
    assert!(
        svg.contains("stroke-dasharray"),
        "consecutive let ring events should produce dashed lines"
    );

    // Verify let ring score differs from plain score
    let no_lr = TabScoreBuilder::guitar()
        .fret(6, 0)
        .next()
        .fret(5, 2)
        .end_barline()
        .render_svg();
    assert_ne!(svg, no_lr, "let ring should differ from plain score");

    assert_golden("tab_let_ring", &svg);
}

fn build_arpeggios() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: upward arpeggio on C major triad + plain quarter
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Up)
        .note(p("C", 5), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .barline()
        // Measure 2: downward arpeggio on D minor triad + single-note arpeggio
        .chord(
            vec![p("D", 4), p("F", 4), p("A", 4)],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Down)
        .note(p("E", 5), Duration::QTR)
        .arpeggio(ArpeggioDirection::Up)
        .rest(Duration::QTR)
        .barline()
        // Measure 3: wide voicing + downward arpeggio on two-note chord
        .chord(
            vec![p("C", 4), p("G", 4), p("E", 5)],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Up)
        .chord(
            vec![p("B", 4), p("D", 5)],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Down)
        .barline()
        // Measure 4: whole-note 4-note chord with arpeggio
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4), p("B", 4)],
            Duration::WHOLE,
        )
        .arpeggio(ArpeggioDirection::Up)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_arpeggios() {
    let svg = build_arpeggios();

    // Structural assertions: arpeggios use scale(1,...) transforms
    let scale_count = svg.matches("scale(1,").count();
    assert!(scale_count >= 5, "should have ≥5 arpeggio scale transforms: {scale_count}");

    // More paths than a plain chord score (arpeggio glyphs add paths)
    let paths = svg.matches("<path ").count();
    assert!(paths >= 10, "should have ≥10 paths: {paths}");

    // Verify up vs down produce different SVG
    let up_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .arpeggio(ArpeggioDirection::Up)
        .end_barline()
        .render_svg();
    let down_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .arpeggio(ArpeggioDirection::Down)
        .end_barline()
        .render_svg();
    assert_ne!(up_only, down_only, "up and down arpeggios should differ");

    assert_golden("arpeggios", &svg);
}

// --- Breath marks ---------------------------------------------------------

fn build_breath_marks() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: comma breaths between ascending notes
        .note(p("C", 4), Duration::QTR)
        .breath_mark(BreathMark::Comma)
        .note(p("E", 4), Duration::QTR)
        .breath_mark(BreathMark::Comma)
        .note(p("G", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .barline()
        // Measure 2: tick + caesura
        .note(p("D", 5), Duration::QTR)
        .breath_mark(BreathMark::Tick)
        .note(p("C", 5), Duration::QTR)
        .note(p("A", 4), Duration::HALF)
        .breath_mark(BreathMark::Caesura)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_breath_marks() {
    let svg = build_breath_marks();

    // Structural assertions: breath marks add path elements
    let path_count = svg.matches("<path ").count();
    // 8 notes + 1 clef + 5 breath marks = at least 14 paths
    assert!(
        path_count >= 14,
        "should have ≥14 paths (notes + clef + breath marks): {path_count}"
    );

    // All 3 breath mark types should produce distinct glyphs
    let comma_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::WHOLE)
        .breath_mark(BreathMark::Comma)
        .end_barline()
        .render_svg();
    let tick_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::WHOLE)
        .breath_mark(BreathMark::Tick)
        .end_barline()
        .render_svg();
    let caesura_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::WHOLE)
        .breath_mark(BreathMark::Caesura)
        .end_barline()
        .render_svg();

    assert_ne!(comma_only, tick_only, "comma and tick should differ");
    assert_ne!(tick_only, caesura_only, "tick and caesura should differ");
    assert_ne!(comma_only, caesura_only, "comma and caesura should differ");

    // Breath marks should add paths compared to a score without them
    let no_breath = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();
    let with_breath = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::WHOLE)
        .breath_mark(BreathMark::Comma)
        .end_barline()
        .render_svg();
    let no_breath_paths = no_breath.matches("<path ").count();
    let with_breath_paths = with_breath.matches("<path ").count();
    assert_eq!(
        with_breath_paths,
        no_breath_paths + 1,
        "breath mark should add exactly 1 path: {} vs {}",
        with_breath_paths,
        no_breath_paths
    );

    assert_golden("breath_marks", &svg);
}

// --- Glissandos -----------------------------------------------------------

fn build_glissandos() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: ascending plain line C4→G4, descending plain line E5→B4
        .note(p("C", 4), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .note(p("G", 4), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .note(p("B", 4), Duration::QTR)
        .barline()
        // Measure 2: ascending with "gliss." text, wide interval
        .note(p("C", 4), Duration::HALF)
        .glissando(GlissandoStyle::LineWithText)
        .note(p("A", 5), Duration::HALF)
        .barline()
        // Measure 3: descending with text label G5→C4
        .note(p("G", 5), Duration::QTR)
        .glissando(GlissandoStyle::LineWithText)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::HALF)
        .barline()
        // Measure 4: same-pitch horizontal glissando + plain quarter
        .note(p("D", 5), Duration::HALF)
        .glissando(GlissandoStyle::Line)
        .note(p("D", 5), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_glissandos() {
    let svg = build_glissandos();

    // Structural assertions: glissando adds lines beyond staff+stems
    let line_count = svg.matches("<line ").count();
    assert!(
        line_count >= 15,
        "should have ≥15 lines (5 staff + stems + glissando lines): {line_count}"
    );

    // "gliss." text should appear for LineWithText style
    let gliss_text_count = svg.matches("gliss.").count();
    assert!(
        gliss_text_count >= 2,
        "should have ≥2 'gliss.' labels: {gliss_text_count}"
    );

    // Line style vs LineWithText should differ
    let line_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .glissando(GlissandoStyle::Line)
        .note(p("G", 5), Duration::HALF)
        .end_barline()
        .render_svg();
    let with_text = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .glissando(GlissandoStyle::LineWithText)
        .note(p("G", 5), Duration::HALF)
        .end_barline()
        .render_svg();
    assert_ne!(line_only, with_text, "Line and LineWithText should differ");

    // Glissando should add lines compared to no-glissando version
    let no_gliss = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .note(p("G", 5), Duration::HALF)
        .end_barline()
        .render_svg();
    let with_gliss = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::HALF)
        .glissando(GlissandoStyle::Line)
        .note(p("G", 5), Duration::HALF)
        .end_barline()
        .render_svg();
    let no_gliss_lines = no_gliss.matches("<line ").count();
    let with_gliss_lines = with_gliss.matches("<line ").count();
    assert!(
        with_gliss_lines > no_gliss_lines,
        "glissando should add at least one line: {} vs {}",
        with_gliss_lines,
        no_gliss_lines
    );

    assert_golden("glissandos", &svg);
}

fn build_cross_system_glissandos() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: ascending quarter notes
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .barline()
        // Measure 2: target of cross-system glissando, then descending with text gliss
        .note(p("D", 5), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .glissando(GlissandoStyle::LineWithText)
        .barline()
        // Measure 3: target of text glissando across system break, then plain notes
        .note(p("C", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .barline()
        // Measure 4: within-system glissando for contrast
        .note(p("G", 4), Duration::HALF)
        .glissando(GlissandoStyle::Line)
        .note(p("C", 5), Duration::HALF)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_cross_system_glissandos() {
    let svg = build_cross_system_glissandos();

    // Cross-system glissando should produce extra lines
    let line_count = svg.matches("<line ").count();
    assert!(
        line_count >= 15,
        "should have ≥15 lines (10 staff + stems + glissando lines): {line_count}"
    );

    // LineWithText cross-system glissando should show "gliss." label
    let gliss_text_count = svg.matches("gliss.").count();
    assert!(
        gliss_text_count >= 1,
        "cross-system LineWithText should show at least 1 'gliss.' label: {gliss_text_count}"
    );

    // Compare with version without glissandos
    let without_gliss = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .barline()
        .note(p("D", 5), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        .note(p("C", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .barline()
        .note(p("G", 4), Duration::HALF)
        .note(p("C", 5), Duration::HALF)
        .end_barline()
        .render_svg();

    let without_lines = without_gliss.matches("<line ").count();
    assert!(
        line_count > without_lines,
        "glissando version should have more lines than plain: {line_count} vs {without_lines}"
    );

    assert_golden("cross_system_glissandos", &svg);
}

/// Multi-voice writing: two voices on one staff with forced stem directions.
fn build_voices() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: voice 0 = melody (stems up), voice 1 = bass (stems down)
        .note(p("E", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::WHOLE)
        .barline()
        // Measure 2: voice 0 = quarter notes up, voice 1 = half notes down
        .note(p("G", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .voice(1)
        .note(p("B", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_voices() {
    let svg = build_voices();

    // Must be valid SVG
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Multi-voice forces stem directions — both up and down stems should be present.
    // Voice 0 stems up (right side of notehead), voice 1 stems down (left side).
    // Count paths (noteheads + clef glyph) — at minimum: 1 clef + 6 voice-0 notes + 3 voice-1 notes = 10
    let path_count = svg.matches("<path ").count();
    assert!(
        path_count >= 10,
        "should have at least 10 paths (clef + noteheads), got {path_count}"
    );

    // Count lines — staff lines + stems + barlines.
    // 5 staff lines + at least 9 stems + barlines = ≥16
    let line_count = svg.matches("<line ").count();
    assert!(
        line_count >= 16,
        "should have at least 16 lines (staff + stems + barlines), got {line_count}"
    );

    // Compare against single-voice rendering — multi-voice should differ
    let single_voice = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("E", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .barline()
        .note(p("G", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .end_barline()
        .render_svg();

    assert_ne!(
        svg, single_voice,
        "multi-voice output should differ from single-voice"
    );

    assert_golden("voices", &svg);
}

fn build_voice_collision() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: unison collisions — both voices on C5 then D5
        .note(p("C", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 5), Duration::HALF)
        .note(p("D", 5), Duration::HALF)
        .barline()
        // Measure 2: second-apart collisions — voice 1 one step below voice 0
        .note(p("E", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .note(p("G", 5), Duration::HALF)
        .voice(1)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("F", 5), Duration::HALF)
        .barline()
        // Measure 3: mixed — unison on beat 1 & 3, far apart on beat 2 & 4
        .note(p("C", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("G", 5), Duration::QTR)
        .note(p("C", 6), Duration::QTR)
        .voice(1)
        .note(p("C", 5), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .note(p("G", 5), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .barline()
        // Measure 4: well-separated voices — no collision
        .note(p("A", 5), Duration::WHOLE)
        .voice(1)
        .note(p("C", 4), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_voice_collision() {
    let svg = build_voice_collision();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    let path_count = svg.matches("<path ").count();
    assert!(
        path_count >= 15,
        "should have at least 15 paths (clef + noteheads from both voices), got {path_count}"
    );

    let line_count = svg.matches("<line ").count();
    assert!(
        line_count >= 20,
        "should have at least 20 lines (staff + stems + barlines), got {line_count}"
    );

    // The collision-heavy score should differ from the basic voices golden
    // (which has well-separated voices and no collisions).
    let basic_voices = build_voices();
    assert_ne!(
        svg, basic_voices,
        "collision example should differ from basic voices"
    );

    assert_golden("voice_collision", &svg);
}

fn build_multi_measure_rest() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4)
        // Measure 1: notated opening
        .note(p("C", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .barline()
        // Multi-measure rest representing 8 measures of tacet
        .multi_measure_rest(8)
        .barline()
        // Notated re-entry
        .note(p("G", 5), Duration::HALF)
        .note(p("E", 5), Duration::HALF)
        .barline()
        // Multi-measure rest representing 16 measures of tacet
        .multi_measure_rest(16)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_multi_measure_rest() {
    let svg = build_multi_measure_rest();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Both count numbers must be present
    assert!(svg.contains(">8</text>"), "should contain count '8'");
    assert!(svg.contains(">16</text>"), "should contain count '16'");

    // Two H-bars = 2 × (2 serifs + crossbar) = 6 rects minimum.
    // No other element in this score uses <rect>.
    let rect_count = svg.matches("<rect").count();
    assert_eq!(
        rect_count, 6,
        "expected exactly 6 rects (2 H-bars × 3 components), got {rect_count}",
    );

    // Count text is bold
    assert!(
        svg.contains("font-weight=\"bold\""),
        "H-bar count text should be bold",
    );

    // Compare against a score with the same notated measures but no
    // multi-measure rests — the H-bar version must add elements.
    let without_mmr = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4)
        .note(p("C", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .barline()
        .note(p("G", 5), Duration::HALF)
        .note(p("E", 5), Duration::HALF)
        .end_barline()
        .render_svg();

    let without_rects = without_mmr.matches("<rect").count();
    assert!(
        rect_count > without_rects,
        "MMR version must have more rects than plain notated version: \
         with={rect_count}, without={without_rects}",
    );

    assert_golden("multi_measure_rest", &svg);
}

fn build_church_rest() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4)
        // Bar 1: opening melody so the part doesn't start on a rest.
        .note(p("C", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .barline()
        // Bar 2: church rest of 1 (whole rest hanging from line 4).
        .multi_measure_rest_church(1)
        .barline()
        // Bar 3: church rest of 2 (breve sitting on line 3).
        .multi_measure_rest_church(2)
        .barline()
        // Bar 4: church rest of 3 (breve + whole).
        .multi_measure_rest_church(3)
        .barline()
        // Bar 5: church rest of 4 (two breves).
        .multi_measure_rest_church(4)
        .barline()
        // Bar 6: closing melodic figure.
        .note(p("G", 5), Duration::HALF)
        .note(p("E", 5), Duration::HALF)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_church_rest() {
    let svg = build_church_rest();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Every supported count (1–4) must appear as bold count text above
    // the staff.
    for n in &["1", "2", "3", "4"] {
        let needle = format!(">{n}</text>");
        assert!(
            svg.contains(&needle),
            "expected count text {needle:?} in SVG",
        );
    }

    // Compare against an H-bar version of the same counts — the church-rest
    // form draws rest glyph <path>s instead of H-bar <rect>s, so the path
    // count must be strictly greater than the H-bar version's path count
    // for these measures.
    let hbar_version = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4)
        .note(p("C", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .barline()
        .multi_measure_rest(1)
        .barline()
        .multi_measure_rest(2)
        .barline()
        .multi_measure_rest(3)
        .barline()
        .multi_measure_rest(4)
        .barline()
        .note(p("G", 5), Duration::HALF)
        .note(p("E", 5), Duration::HALF)
        .end_barline()
        .render_svg();

    // Church-rest measures collectively draw 6 rest glyph paths:
    //   count 1 → 1 (whole)
    //   count 2 → 1 (breve)
    //   count 3 → 2 (breve + whole)
    //   count 4 → 2 (breve + breve)
    let church_paths = svg.matches("<path ").count();
    let hbar_paths = hbar_version.matches("<path ").count();
    assert_eq!(
        church_paths - hbar_paths,
        6,
        "church-rest must add 6 extra rest glyph paths vs H-bar form; \
         church={church_paths}, hbar={hbar_paths}",
    );

    // The H-bar version has 4 × 3 = 12 rects (one per multi-measure rest
    // measure); the church version draws no rects (no other notation in this
    // score emits <rect>).
    let church_rects = svg.matches("<rect").count();
    let hbar_rects = hbar_version.matches("<rect").count();
    assert_eq!(
        hbar_rects - church_rects,
        12,
        "H-bar version must have 12 more rects than church version (4 mmrs × 3 rects); \
         church={church_rects}, hbar={hbar_rects}",
    );

    // Count text must be bold (matches the H-bar convention so a reader
    // doesn't see two different count-text weights in the same part).
    assert!(
        svg.contains("font-weight=\"bold\""),
        "church-rest count text should be bold",
    );

    // The two renderings must differ.
    assert_ne!(svg, hbar_version, "church-rest and H-bar must render differently");

    assert_golden("church_rest", &svg);
}

/// Score where each of the two voices independently carries ties, slurs,
/// and a hairpin. Exercises the cross-voice span machinery: span endpoints
/// must be resolved against notes in their own voice, not the primary one.
fn build_cross_voice_spans() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // m1: voice 0 tie (E5–E5), voice 1 slur (C4–E4–G4–C5)
        .note(p("E", 5), Duration::HALF)
        .tie()
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .slur_start()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .slur_end()
        .voice(0)
        .barline()
        // m2: voice 0 slur (G5–F5–E5–D5), voice 1 tie (C4–C4)
        .note(p("G", 5), Duration::QTR)
        .slur_start()
        .note(p("F", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .slur_end()
        .voice(1)
        .note(p("C", 4), Duration::HALF)
        .tie()
        .note(p("C", 4), Duration::HALF)
        .voice(0)
        .barline()
        // m3: voice 0 crescendo, voice 1 decrescendo (simultaneous, opposing)
        .note(p("E", 5), Duration::QTR)
        .hairpin_start(HairpinType::Crescendo)
        .note(p("F", 5), Duration::QTR)
        .note(p("G", 5), Duration::QTR)
        .note(p("A", 5), Duration::QTR)
        .hairpin_end()
        .voice(1)
        .note(p("C", 5), Duration::QTR)
        .hairpin_start(HairpinType::Decrescendo)
        .note(p("A", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .hairpin_end()
        .voice(0)
        .barline()
        // m4: voice 0 tie (G5–G5), voice 1 slur (C4–D4–E4–F4)
        .note(p("G", 5), Duration::HALF)
        .tie()
        .note(p("G", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .slur_start()
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .slur_end()
        .voice(0)
        .end_barline()
        .render_svg()
}

/// Parallel score with identical pitches and rhythms but no span markers.
/// Used for delta assertions — every span in `build_cross_voice_spans`
/// should be an element this baseline doesn't have.
fn build_cross_voice_spans_baseline() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // m1
        .note(p("E", 5), Duration::HALF)
        .note(p("E", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .voice(0)
        .barline()
        // m2
        .note(p("G", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .note(p("E", 5), Duration::QTR)
        .note(p("D", 5), Duration::QTR)
        .voice(1)
        .note(p("C", 4), Duration::HALF)
        .note(p("C", 4), Duration::HALF)
        .voice(0)
        .barline()
        // m3
        .note(p("E", 5), Duration::QTR)
        .note(p("F", 5), Duration::QTR)
        .note(p("G", 5), Duration::QTR)
        .note(p("A", 5), Duration::QTR)
        .voice(1)
        .note(p("C", 5), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .voice(0)
        .barline()
        // m4
        .note(p("G", 5), Duration::HALF)
        .note(p("G", 5), Duration::HALF)
        .voice(1)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .voice(0)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_cross_voice_spans() {
    let svg = build_cross_voice_spans();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    let baseline = build_cross_voice_spans_baseline();

    // 3 ties + 3 slurs are each filled paths with stroke="none".
    let fills_with = svg.matches(r#"stroke="none""#).count();
    let fills_base = baseline.matches(r#"stroke="none""#).count();
    assert_eq!(
        fills_with - fills_base,
        6,
        "expected exactly 6 extra filled curves (3 ties + 3 slurs); \
         with={fills_with}, base={fills_base}",
    );

    // 2 hairpins (one per voice) contribute exactly 4 lines (2 sides each).
    let lines_with = svg.matches("<line ").count();
    let lines_base = baseline.matches("<line ").count();
    assert_eq!(
        lines_with - lines_base,
        4,
        "expected exactly 4 extra lines from 2 hairpins; \
         with={lines_with}, base={lines_base}",
    );

    // The whole rendering must differ from the no-spans version.
    assert_ne!(svg, baseline, "cross-voice spans must change the output");

    // Sanity: must have stems in both directions (multi-voice forces them).
    assert!(svg.contains("<line "), "should have stem lines");

    assert_golden("cross_voice_spans", &svg);
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
        "grace_note_slur",
        "annotations",
        "bass_clef",
        "auto_breaks",
        "optimal_breaks",
        "grand_staff",
        "lyrics",
        "chord_symbols",
        "ornaments",
        "hairpins",
        "cross_system_ties",
        "expression_text",
        "multi_staff_cross_system",
        "tab_score",
        "tab_slides",
        "tab_hammer_pull",
        "tab_bends",
        "volta_brackets",
        "cross_system_volta",
        "guitar_tab",
        "navigation_signs",
        "ottava_brackets",
        "cross_system_ottava",
        "pedal_marks",
        "tab_pre_bends",
        "tremolo",
        "tab_vibrato",
        "tab_harmonics",
        "tab_palm_mute",
        "tab_muted_strings",
        "tab_let_ring",
        "arpeggios",
        "breath_marks",
        "glissandos",
        "cross_system_glissandos",
        "voices",
        "voice_collision",
        "multi_measure_rest",
        "church_rest",
        "cross_voice_spans",
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
