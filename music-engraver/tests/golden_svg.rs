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

/// The three less-common `Dynamic` variants exposed in the 2026-05-13
/// Dynamic-expansion chunk: `Mezzo` (bare letter-`m`, distinct from
/// `Mp`/`Mf`), `SforzatoPiano` (sforzato-prefixed sfp, distinct from
/// the sforzando-prefixed `Sfp`), and `Z` (the rare single-letter sudden
/// accent). One quarter note per variant + a plain padding quarter so the
/// 4/4 measure closes and the structural delta in
/// `golden_dynamics_variants` is exactly 3.
fn build_dynamics_variants() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .dynamic(Dynamic::Mezzo)
        .note(p("D", 4), Duration::QTR)
        .dynamic(Dynamic::SforzatoPiano)
        .note(p("E", 4), Duration::QTR)
        .dynamic(Dynamic::Z)
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// The same four-note measure as `build_dynamics_variants` with no
/// dynamics attached. The golden test uses this as the structural baseline
/// so the variant score must differ by exactly one path and one unique
/// d-string per variant.
fn build_dynamics_variants_plain() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// 28 pitches threading every variant in `Dynamic::ALL`. Two passes through
/// the diatonic C scale (C4-B4-C5-B5) so the noteheads occupy a consistent
/// staff-position range and don't introduce ledger lines below the staff
/// (which would crowd the dynamics' below-staff baseline). Shared between
/// `build_dynamics_full`, its no-dynamic sibling, and the golden test so
/// all three see the same list.
const DYNAMICS_FULL_PITCHES: [(&str, u8); 28] = [
    ("C", 4), ("D", 4), ("E", 4), ("F", 4),
    ("G", 4), ("A", 4), ("B", 4), ("C", 5),
    ("D", 5), ("E", 5), ("F", 5), ("G", 5),
    ("A", 5), ("B", 5), ("C", 4), ("D", 4),
    ("E", 4), ("F", 4), ("G", 4), ("A", 4),
    ("B", 4), ("C", 5), ("D", 5), ("E", 5),
    ("F", 5), ("G", 5), ("A", 5), ("B", 5),
];

/// Every variant in `Dynamic::ALL` (28 dynamics) — one per quarter note,
/// laid out 4 per measure across 7 measures (2 measures per system). Visual
/// proofing companion to the unit-test glyph-distinctness assertions
/// covering the full set (e.g. `all_dynamics_produce_distinct_svg_output`
/// in `render::dynamics_renderer`).
fn build_dynamics_full() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    for (i, dynamic) in Dynamic::ALL.iter().enumerate() {
        let (n, oct) = DYNAMICS_FULL_PITCHES[i];
        b = b.note(p(n, oct), Duration::QTR).dynamic(*dynamic);
        if (i + 1) % 4 == 0 && (i + 1) < DYNAMICS_FULL_PITCHES.len() {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
}

/// Same 28-note pattern as `build_dynamics_full` with no dynamics attached.
/// The golden test uses this as the structural baseline so the full-coverage
/// score must differ by exactly one path and one unique d-string per
/// variant.
fn build_dynamics_full_plain() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    for (i, (n, oct)) in DYNAMICS_FULL_PITCHES.iter().enumerate() {
        b = b.note(p(n, *oct), Duration::QTR);
        if (i + 1) % 4 == 0 && (i + 1) < DYNAMICS_FULL_PITCHES.len() {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
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

/// Canonical ordering of the 7 fermata variants. Shared between
/// `build_fermata_variants`, its no-fermata sibling, and the golden test
/// so all three see the same list.
const FERMATA_VARIANTS: [Articulation; 7] = [
    Articulation::Fermata,
    Articulation::FermataLong,
    Articulation::FermataShort,
    Articulation::FermataVeryLong,
    Articulation::FermataVeryShort,
    Articulation::FermataHenzeLong,
    Articulation::FermataHenzeShort,
];

/// Pitches for the fermata-variant score. Staggered so successive variants
/// don't all sit at the same staff position — keeps the visual gap between
/// glyphs honest at golden-comparison time.
const FERMATA_PITCHES: [(&str, u8); 7] = [
    ("G", 4),
    ("A", 4),
    ("B", 4),
    ("C", 5),
    ("D", 5),
    ("E", 5),
    ("F", 5),
];

/// Each of the 7 fermata variants from the SMuFL duration-coded family on
/// a whole note in its own measure, plus a padding whole note to close
/// the 4-system layout neatly.
fn build_fermata_variants() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    let count = FERMATA_VARIANTS.len();
    for (i, variant) in FERMATA_VARIANTS.iter().enumerate() {
        let (n, oct) = FERMATA_PITCHES[i];
        b = b.note(p(n, oct), Duration::WHOLE).articulation(*variant);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.barline()
        .note(p("G", 4), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

/// The same layout as `build_fermata_variants` with no articulations.
/// Used as the structural baseline for the fermata-variants golden test:
/// the variant score must differ from this by exactly one path per
/// variant, and the variant score's d-string set must contribute exactly
/// `FERMATA_VARIANTS.len()` unique d-strings.
fn build_fermata_variants_plain() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    let count = FERMATA_VARIANTS.len();
    for (i, _variant) in FERMATA_VARIANTS.iter().enumerate() {
        let (n, oct) = FERMATA_PITCHES[i];
        b = b.note(p(n, oct), Duration::WHOLE);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.barline()
        .note(p("G", 4), Duration::WHOLE)
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

/// Every variant in `Ornament::ALL` (15 ornaments) — one per note, laid
/// out 4 ornaments per measure across 2 systems. Visual proofing
/// companion to the unit-test glyph-distinctness assertions added when
/// `Ornament::ALL` was introduced.
fn build_ornaments_full() -> String {
    let pitches = [
        ("C", 4), ("D", 4), ("E", 4), ("F", 4),
        ("G", 4), ("A", 4), ("B", 4), ("C", 5),
        ("D", 5), ("E", 5), ("F", 5), ("G", 5),
        ("A", 5), ("B", 5), ("C", 6),
    ];
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    for (i, ornament) in Ornament::ALL.iter().enumerate() {
        let (n, oct) = pitches[i];
        b = b.note(p(n, oct), Duration::QTR).ornament(*ornament);
        if (i + 1) % 4 == 0 {
            b = b.barline();
        }
    }
    // Pad measure 4 to 4/4 with one ornamentless quarter.
    b.note(p("D", 6), Duration::QTR).end_barline().render_svg()
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
fn golden_dynamics_variants() {
    let svg = build_dynamics_variants();
    let plain = build_dynamics_variants_plain();

    // Structural validity
    assert!(svg.starts_with("<svg"), "dynamics_variants should be SVG");
    assert!(svg.contains("</svg>"), "dynamics_variants should close SVG");

    // Path-count guard: each new variant must add exactly one path on top
    // of the same notes with no dynamics. Catches a silent regression
    // where a variant maps to a missing glyph and renders zero paths
    // (e.g. a future SMuFL font swap that omits one of these less-common
    // glyphs — already guarded at the unit-test level by
    // `new_variants_have_nonzero_advance_in_bravura`, but reinforced
    // here at the integration layer).
    let full_paths = svg.matches("<path").count();
    let plain_paths = plain.matches("<path").count();
    let added_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        added_paths, 3,
        "each new dynamic variant must add exactly one path: full={full_paths}, \
         plain={plain_paths}, delta={added_paths}, expected=3"
    );

    // Distinct-d guard: each variant must contribute a unique SMuFL path
    // payload. `Mezzo`, `SforzatoPiano`, and `Z` are visually similar to
    // their neighbours (`Mp`/`Mf`, `Sfp`, `Sfz`/`Fz` respectively) but
    // must produce distinct path data in Bravura — see
    // `new_variants_render_distinct_path_data_in_bravura` in
    // `render::dynamics_renderer` for the per-glyph proof.
    use std::collections::HashSet;
    fn distinct_d(svg: &str) -> HashSet<String> {
        let mut out = HashSet::new();
        for chunk in svg.split("d=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                out.insert(chunk[..end].to_string());
            }
        }
        out
    }
    let added: HashSet<_> = distinct_d(&svg)
        .difference(&distinct_d(&plain))
        .cloned()
        .collect();
    assert_eq!(
        added.len(),
        3,
        "the 3 new dynamic variants must contribute 3 unique path d-strings \
         (no glyph collapses), got {}",
        added.len()
    );

    // Byte-equal regression: this score must differ from the existing
    // `dynamics` baseline (different gestures — no hairpins, different
    // variants — sanity check that the new golden isn't accidentally
    // identical to a prior one).
    assert_ne!(
        svg,
        build_dynamics(),
        "dynamics_variants must differ from the existing dynamics baseline"
    );

    assert_golden("dynamics_variants", &svg);
}

#[test]
fn golden_dynamics_full() {
    let svg = build_dynamics_full();
    let plain = build_dynamics_full_plain();

    // Structural validity
    assert!(svg.starts_with("<svg"), "dynamics_full should be SVG");
    assert!(svg.contains("</svg>"), "dynamics_full should close SVG");

    // Path-count guard: every variant in `Dynamic::ALL` must add exactly
    // one path on top of the same 28-note score with no dynamics. Catches
    // a silent regression where a variant maps to a missing glyph and
    // renders zero paths.
    let full_paths = svg.matches("<path").count();
    let plain_paths = plain.matches("<path").count();
    let added_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        added_paths,
        Dynamic::ALL.len(),
        "each dynamic variant must add exactly one path: full={full_paths}, \
         plain={plain_paths}, delta={added_paths}, expected={}",
        Dynamic::ALL.len()
    );

    // Distinct-d guard: all 28 variants must contribute distinct SMuFL
    // path payloads (Bravura provides a dedicated composite glyph for each
    // — proven at the renderer-unit-test layer by
    // `all_dynamics_produce_distinct_svg_output` and the per-variant
    // distinctness tests; this guard reinforces it at the integration layer).
    use std::collections::HashSet;
    fn distinct_d(svg: &str) -> HashSet<String> {
        let mut out = HashSet::new();
        for chunk in svg.split("d=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                out.insert(chunk[..end].to_string());
            }
        }
        out
    }
    let added: HashSet<_> = distinct_d(&svg)
        .difference(&distinct_d(&plain))
        .cloned()
        .collect();
    assert_eq!(
        added.len(),
        Dynamic::ALL.len(),
        "all 28 dynamic variants must contribute unique path d-strings \
         (no glyph collapses), got {}",
        added.len()
    );

    // Byte-inequality vs smaller dynamics baselines: the full-coverage
    // score must differ from both the 4-note `dynamics` and 4-note
    // `dynamics_variants` baselines. Sanity check that the new golden
    // isn't accidentally identical to a prior one.
    assert_ne!(
        svg,
        build_dynamics(),
        "dynamics_full must differ from the 4-note `dynamics` baseline"
    );
    assert_ne!(
        svg,
        build_dynamics_variants(),
        "dynamics_full must differ from the `dynamics_variants` baseline"
    );

    assert_golden("dynamics_full", &svg);
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
fn golden_fermata_variants() {
    let svg = build_fermata_variants();
    let plain = build_fermata_variants_plain();

    // Structural validity
    assert!(svg.starts_with("<svg"), "fermata_variants should be SVG");
    assert!(svg.contains("</svg>"), "fermata_variants should close SVG");

    // Path-count guard: each variant must add exactly one path on top of
    // the same score with no articulations. Catches a silent regression
    // where a variant maps to a missing glyph and renders zero paths.
    let full_paths = svg.matches("<path").count();
    let plain_paths = plain.matches("<path").count();
    let added_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        added_paths,
        FERMATA_VARIANTS.len(),
        "each fermata variant must add exactly one path: full={full_paths}, \
         plain={plain_paths}, delta={added_paths}, expected={}",
        FERMATA_VARIANTS.len()
    );

    // Distinct-d guard: each variant must contribute a unique SMuFL path
    // payload. Bravura's plain Fermata, FermataLong, FermataShort,
    // FermataVeryLong, FermataVeryShort, FermataLongHenze, and
    // FermataShortHenze are all distinct shapes — if any two glyphs
    // accidentally collapse to the same d-string, this assertion fails.
    use std::collections::HashSet;
    fn distinct_d(svg: &str) -> HashSet<String> {
        let mut out = HashSet::new();
        for chunk in svg.split("d=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                out.insert(chunk[..end].to_string());
            }
        }
        out
    }
    let added: HashSet<_> = distinct_d(&svg)
        .difference(&distinct_d(&plain))
        .cloned()
        .collect();
    assert_eq!(
        added.len(),
        FERMATA_VARIANTS.len(),
        "fermata variants must contribute exactly {} unique path d-strings \
         (no glyph collapses), got {}",
        FERMATA_VARIANTS.len(),
        added.len()
    );

    // Byte-equal regression: this score must differ from the existing
    // 4-articulation `articulations` baseline (different gestures, different
    // layout — sanity check that the new golden isn't accidentally
    // identical to a prior one).
    assert_ne!(
        svg,
        build_articulations(),
        "fermata_variants must differ from the existing articulations baseline"
    );

    assert_golden("fermata_variants", &svg);
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
fn golden_ornaments_full() {
    let svg = build_ornaments_full();

    // Structural validity
    assert!(svg.starts_with("<svg"), "ornaments_full should be SVG");
    assert!(svg.contains("</svg>"), "ornaments_full should close SVG");

    // Path-count guard: must strictly exceed the same score without any
    // ornaments (identical 16-note scale + key sig + time sig + clef).
    // This proves every ornament call actually drew a path, rather than
    // silently no-oping.
    let pitches = [
        ("C", 4), ("D", 4), ("E", 4), ("F", 4),
        ("G", 4), ("A", 4), ("B", 4), ("C", 5),
        ("D", 5), ("E", 5), ("F", 5), ("G", 5),
        ("A", 5), ("B", 5), ("C", 6),
    ];
    let mut plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    for (i, (n, oct)) in pitches.iter().enumerate() {
        plain = plain.note(p(n, *oct), Duration::QTR);
        if (i + 1) % 4 == 0 {
            plain = plain.barline();
        }
    }
    let plain_svg = plain
        .note(p("D", 6), Duration::QTR)
        .end_barline()
        .render_svg();

    let full_paths = svg.matches("<path").count();
    let plain_paths = plain_svg.matches("<path").count();
    let ornament_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        ornament_paths, 15,
        "ornaments_full must add exactly one path per ornament \
         (15 ornaments → 15 paths). full={full_paths}, plain={plain_paths}, \
         delta={ornament_paths}"
    );

    // Distinct-d guard: the 15 ornament glyphs reduce to 14 unique SMuFL
    // paths because `InvertedMordent` and `ShortTrill` deliberately share
    // `OrnamentShortTrill` (documented in `layout::ornament`). Compute
    // the set of d="..." values in `svg` vs `plain_svg`; the difference
    // must be exactly the unique ornament glyphs.
    use std::collections::HashSet;
    fn distinct_d(svg: &str) -> HashSet<String> {
        let mut out = HashSet::new();
        for chunk in svg.split("d=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                out.insert(chunk[..end].to_string());
            }
        }
        out
    }
    let added: HashSet<_> = distinct_d(&svg).difference(&distinct_d(&plain_svg)).cloned().collect();
    assert_eq!(
        added.len(),
        14,
        "ornaments must contribute exactly 14 unique path d-strings \
         (15 variants − 1 InvertedMordent≡ShortTrill alias), got {}",
        added.len()
    );

    // Byte-equal regression: ornaments_full SVG must differ from the
    // existing 4-ornament `ornaments` baseline.
    assert_ne!(
        svg,
        build_ornaments(),
        "ornaments_full must differ from the 4-ornament baseline"
    );

    assert_golden("ornaments_full", &svg);
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

/// A short score that exercises trill_with_extension on long-duration notes
/// (whole + dotted-half + chord) so the wavy line has room to tile.
fn build_trill_extension() -> String {
    use music::notation::rhythm::duration::DurationKind;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("G", 4), Duration::HALF)
        .trill_with_extension()
        .note(p("A", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .barline()
        .note(p("E", 5), Duration::new(DurationKind::Half, 1))
        .trill_with_extension()
        .note(p("D", 5), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// The same score as `build_trill_extension` but with plain `.ornament(Trill)`
/// instead — used to compare path counts and confirm the extension adds
/// tangible wiggle content.
fn build_trill_no_extension() -> String {
    use music::notation::rhythm::duration::DurationKind;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("G", 4), Duration::HALF)
        .ornament(Ornament::Trill)
        .note(p("A", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .barline()
        .note(p("E", 5), Duration::new(DurationKind::Half, 1))
        .ornament(Ornament::Trill)
        .note(p("D", 5), Duration::QTR)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_extension() {
    let svg = build_trill_extension();
    let baseline = build_trill_no_extension();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // The extension must add wiggle paths. Both scores have two "tr" glyphs;
    // only the extension version has any tiled wiggle segments.
    let ext_paths = svg.matches("<path").count();
    let base_paths = baseline.matches("<path").count();
    assert!(
        ext_paths > base_paths,
        "trill_with_extension must add wiggle paths beyond plain trill: \
         ext={ext_paths}, base={base_paths}"
    );

    // The extra paths are wiggle segments. Each tile shares its translate y
    // with the trill glyph. We at least expect more than 2 extra paths since
    // any reasonable wiggle should tile several segments across a half note
    // span, even at conservative measure widths.
    let delta = ext_paths - base_paths;
    assert!(
        delta >= 2,
        "expected at least 2 wiggle segments added by extension, got {delta}"
    );

    // The two scores must not be byte-identical.
    assert_ne!(svg, baseline, "extension must change the rendered SVG");

    assert_golden("trill_extension", &svg);
}

/// Build a short score exercising all three bracket forms (Start/End/Both)
/// plus a chord-trill bracket. Uses 2 measures per system so each measure
/// is wide enough for the wiggle to tile.
fn build_trill_bracket() -> String {
    use music_engraver::layout::trill_bracket::TrillBracketSide;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Both bracket on whole note — frames the trill's full range.
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .barline()
        // Note the bracketed trill resolves to in M2.
        .note(p("A", 4), Duration::WHOLE)
        .barline()
        // Start bracket only on whole note — marks unambiguous start.
        .note(p("E", 5), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Start)
        .barline()
        .note(p("D", 5), Duration::WHOLE)
        .barline()
        // End bracket only on whole note — marks unambiguous end.
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::End)
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .barline()
        // Both bracket on chord (top note 7).
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Both)
        .end_barline()
        .note(p("F", 4), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_bracket() {
    use music_engraver::layout::trill_bracket::TrillBracketSide;
    let svg = build_trill_bracket();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Build a no-bracket baseline using only trill_with_extension. The
    // bracketed version must add hook <line> elements: 2 (M1 Both) +
    // 1 (M3 Start) + 1 (M5 End) + 2 (M7 chord Both) = 6.
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .barline()
        .note(p("E", 5), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("D", 5), Duration::WHOLE)
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .barline()
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let bracket_lines = svg.matches("<line ").count();
    let plain_lines = baseline.matches("<line ").count();
    let delta = bracket_lines - plain_lines;
    assert_eq!(
        delta, 6,
        "expected exactly 6 hook lines (Both:2 + Start:1 + End:1 + chord Both:2). \
         Got delta={delta} (bracketed={bracket_lines}, plain={plain_lines})"
    );

    // Verify a single-side bracket adds only 1 hook (a regression guard
    // against accidentally double-rendering hooks).
    let only_start = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed(TrillBracketSide::Start)
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let only_start_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(
        only_start.matches("<line ").count() - only_start_plain.matches("<line ").count(),
        1,
        "Start-only bracket must add exactly 1 hook line"
    );

    // SVG must differ from no-bracket baseline byte-for-byte.
    assert_ne!(svg, baseline, "brackets must change the rendered SVG");

    assert_golden("trill_bracket", &svg);
}

/// Build a score exercising the custom-options bracket API with non-default
/// directions and lengths. Validates the wiring of `HookDirection::Up` and
/// arbitrary length-in-staff-spaces through the system + page renderers.
fn build_trill_bracket_custom() -> String {
    use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Both, Down direction, longer-than-default 1.2 ss hook.
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Both,
            HookDirection::Down,
            1.2,
        )
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .barline()
        // Start, Up direction, default-length 0.75 ss.
        .note(p("E", 5), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Start,
            HookDirection::Up,
            0.75,
        )
        .barline()
        .note(p("D", 5), Duration::WHOLE)
        .barline()
        // End, Up direction, short 0.5 ss hook.
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::End,
            HookDirection::Up,
            0.5,
        )
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .barline()
        // Chord trill: Both, Up direction, 1.0 ss hook.
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Both,
            HookDirection::Up,
            1.0,
        )
        .end_barline()
        .note(p("F", 4), Duration::WHOLE)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_bracket_custom() {
    use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
    let svg = build_trill_bracket_custom();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Same hook-count delta as `golden_trill_bracket` — 6 hooks total
    // (M1 Both:2 + M3 Start:1 + M5 End:1 + M7 chord Both:2) vs the plain
    // no-bracket variant. The custom knobs only change geometry, not count.
    let baseline = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .barline()
        .note(p("E", 5), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("D", 5), Duration::WHOLE)
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .barline()
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let bracket_lines = svg.matches("<line ").count();
    let plain_lines = baseline.matches("<line ").count();
    let delta = bracket_lines - plain_lines;
    assert_eq!(
        delta, 6,
        "expected 6 hook lines for custom variant (same as default bracket variant). \
         Got delta={delta} (bracketed={bracket_lines}, plain={plain_lines})"
    );

    // The custom variant must differ from the default-options variant
    // (build_trill_bracket) — the user's overrides actually take effect.
    let default_variant = build_trill_bracket();
    assert_ne!(
        svg, default_variant,
        "custom overrides (length+direction) must produce different SVG than defaults"
    );
    // ...but the count of <line> elements must be the same — overrides only
    // change geometry, never count.
    assert_eq!(
        svg.matches("<line ").count(),
        default_variant.matches("<line ").count(),
        "custom overrides must not add/remove <line> elements"
    );

    // Single-side custom: a Start-only bracket with Up direction must still
    // add exactly 1 hook line vs the same score with no bracket.
    let only_start_up = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(
            TrillBracketSide::Start,
            HookDirection::Up,
            0.9,
        )
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    let only_start_plain = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("E", 4), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .note(p("F", 4), Duration::QTR)
        .end_barline()
        .render_svg();
    assert_eq!(
        only_start_up.matches("<line ").count() - only_start_plain.matches("<line ").count(),
        1,
        "Start+Up custom bracket must add exactly 1 hook line"
    );

    assert_golden("trill_bracket_custom", &svg);
}

/// Build a short score exercising all 9 SMuFL wiggleTrill* speed variants
/// through `.trill_with_extension_speed`. Each measure picks a different
/// speed; the wide whole-note spans give every wiggle enough room for
/// several tiles so density differences are visible.
fn build_trill_wiggle_speed() -> String {
    use music_engraver::layout::trill_extension::TrillWiggleSpeed;
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    // Walk through every variant in canonical order. The terminating
    // quarter is added at the end so the last whole-note trill has a
    // following note to anchor its within-system wiggle.
    let pitches = [
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
    ];
    for (i, speed) in TrillWiggleSpeed::ALL.iter().enumerate() {
        let (n, oct) = pitches[i];
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed(*speed)
            .barline();
    }
    b.note(p("B", 5), Duration::QTR).end_barline().render_svg()
}

#[test]
fn golden_trill_wiggle_speed() {
    use music_engraver::layout::trill_extension::TrillWiggleSpeed;
    let svg = build_trill_wiggle_speed();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Every measure has exactly one "tr" glyph + tiled wiggle. The total
    // path count must dominate the same score where every wiggle is at
    // Slowest (the sparsest density) — confirming individual speed choices
    // actually affect tile counts in the final SVG.
    let mut all_slowest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);
    let pitches = [
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
    ];
    for (n, oct) in pitches {
        all_slowest = all_slowest
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
            .barline();
    }
    let baseline = all_slowest
        .note(p("B", 5), Duration::QTR)
        .end_barline()
        .render_svg();

    let mixed_paths = svg.matches("<path").count();
    let slowest_paths = baseline.matches("<path").count();
    assert!(
        mixed_paths > slowest_paths,
        "Mixed-speed score must have more tile paths than all-Slowest baseline: \
         mixed={mixed_paths}, slowest={slowest_paths}"
    );

    // A score with all-Fastest must in turn produce strictly more paths than
    // mixed (and even more than all-Slowest). This sandwiches the mixed
    // value between the two extremes — a structural guard that none of the
    // 9 speeds are silently treated as a single glyph.
    let mut all_fastest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);
    for (n, oct) in pitches {
        all_fastest = all_fastest
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed(TrillWiggleSpeed::Fastest)
            .barline();
    }
    let fastest = all_fastest
        .note(p("B", 5), Duration::QTR)
        .end_barline()
        .render_svg();
    let fastest_paths = fastest.matches("<path").count();
    assert!(
        fastest_paths > mixed_paths && fastest_paths > slowest_paths,
        "All-Fastest must produce the most tile paths: \
         fastest={fastest_paths}, mixed={mixed_paths}, slowest={slowest_paths}"
    );

    // SVG must not be byte-equal to either bookend.
    assert_ne!(svg, baseline, "mixed-speed must differ from all-Slowest");
    assert_ne!(svg, fastest, "mixed-speed must differ from all-Fastest");

    assert_golden("trill_wiggle_speed", &svg);
}

/// Build a short score exercising the precomposed trill-with-mordent
/// compound ornament + extension via
/// `ScoreBuilder::trill_with_mordent_with_extension`. The compound glyph
/// is wider than the bare "tr" — the wiggle must start past the *full*
/// compound, not just the trill prefix.
fn build_trill_with_mordent_extension() -> String {
    use music::notation::rhythm::duration::DurationKind;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("G", 4), Duration::HALF)
        .trill_with_mordent_with_extension()
        .note(p("A", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .barline()
        .note(p("E", 5), Duration::new(DurationKind::Half, 1))
        .trill_with_mordent_with_extension()
        .note(p("D", 5), Duration::QTR)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_with_mordent_extension() {
    let svg = build_trill_with_mordent_extension();
    let plain_compound = {
        use music::notation::rhythm::duration::DurationKind;
        ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Open)
            .time_signature(4, 4)
            .note(p("G", 4), Duration::HALF)
            .ornament(Ornament::TrillWithMordent)
            .note(p("A", 4), Duration::QTR)
            .note(p("G", 4), Duration::QTR)
            .barline()
            .note(p("E", 5), Duration::new(DurationKind::Half, 1))
            .ornament(Ornament::TrillWithMordent)
            .note(p("D", 5), Duration::QTR)
            .end_barline()
            .render_svg()
    };

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // The extension must add wiggle paths beyond plain compound rendering.
    let ext_paths = svg.matches("<path").count();
    let base_paths = plain_compound.matches("<path").count();
    assert!(
        ext_paths > base_paths,
        "trill_with_mordent_with_extension must add wiggle paths beyond plain compound: \
         ext={ext_paths}, base={base_paths}"
    );

    // At least 2 wiggle segments — guards against the renderer drawing a
    // single phantom path while skipping the actual wiggle tiles.
    let delta = ext_paths - base_paths;
    assert!(
        delta >= 2,
        "expected at least 2 wiggle segments added by compound-extension, got {delta}"
    );

    // The compound version must differ byte-for-byte from the equivalent
    // plain-trill extension score. This is the regression canary that
    // confirms `collect_trill_extension_note_info` propagated the actual
    // ornament rather than collapsing TrillWithMordent to Trill, and that
    // `draw_system_trill_extensions` used the actual glyph's advance for
    // the wiggle-start position.
    let trill_ext = build_trill_extension();
    assert_ne!(
        svg, trill_ext,
        "trill-with-mordent + extension must differ from trill + extension: \
         different prefix glyph, different wiggle start"
    );

    assert_golden("trill_with_mordent_extension", &svg);
}

/// Build the bracketed compound-trill score.
///
/// Four whole-note measures, 2 per system → 2 systems total:
/// - M1: compound trill with `Both` brackets, all defaults (Down/0.75ss).
/// - M2: compound trill with `Start`-only bracket, `Up` direction, 1.0ss length.
/// - M3: compound trill with `End`-only bracket, default direction, 0.5ss length.
/// - M4: chord compound trill with `Both` brackets, all defaults.
///
/// Exercises every combination of `TrillBracketOptions` knobs in the
/// presence of the `ornament` override. Hook geometry should be
/// glyph-independent (same hook count as the parallel plain-trill scores)
/// while the prefix glyph + wiggle start should differ.
fn build_trill_bracket_with_mordent() -> String {
    use music::notation::rhythm::duration::DurationKind;
    use music_engraver::layout::trill_bracket::{
        HookDirection, TrillBracketOptions, TrillBracketSide,
    };
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Both, all defaults, compound override.
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        // M2: Start only, Up direction, 1.0ss length.
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Start)
                .with_ornament(Ornament::TrillWithMordent)
                .with_direction(HookDirection::Up)
                .with_length_ss(1.0),
        )
        .barline()
        // M3: End only, default direction, 0.5ss length.
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::End)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(0.5),
        )
        .barline()
        // M4: chord compound-trill, Both, all defaults.
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .note(p("D", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same shape as `build_trill_bracket_with_mordent` but with plain `Trill`
/// brackets (no `.with_ornament(...)`). Used to prove the compound override
/// actually propagates a different glyph through the renderer.
fn build_trill_bracket_with_mordent_plain_variant() -> String {
    use music::notation::rhythm::duration::DurationKind;
    use music_engraver::layout::trill_bracket::{
        HookDirection, TrillBracketOptions, TrillBracketSide,
    };
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Start)
                .with_direction(HookDirection::Up)
                .with_length_ss(1.0),
        )
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::End).with_length_ss(0.5),
        )
        .barline()
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .note(p("D", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same shape as `build_trill_bracket_with_mordent` but with plain compound
/// `.trill_with_mordent_with_extension()` (no brackets). Used to count the
/// hook-line delta the brackets contribute.
fn build_trill_bracket_with_mordent_no_bracket_variant() -> String {
    use music::notation::rhythm::duration::DurationKind;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .barline()
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_mordent_with_extension()
        .note(p("D", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_bracket_with_mordent() {
    let svg = build_trill_bracket_with_mordent();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Structural guard 1: exact 6-hook delta vs the no-bracket variant.
    // The example exercises Both+Start+End+Both = 2+1+1+2 = 6 hooks.
    let no_bracket = build_trill_bracket_with_mordent_no_bracket_variant();
    let bracket_lines = svg.matches("<line ").count();
    let no_bracket_lines = no_bracket.matches("<line ").count();
    let delta = bracket_lines.saturating_sub(no_bracket_lines);
    assert_eq!(
        delta, 6,
        "expected 6 hook lines added by bracketed compound (M1 Both+M2 Start+M3 End+M4 chord Both); \
         got delta={delta} (bracketed={bracket_lines}, no_bracket={no_bracket_lines})"
    );

    // Structural guard 2: the compound override must produce a different
    // SVG than the same brackets on plain `Ornament::Trill`. This is the
    // regression canary that the `.with_ornament(...)` actually propagated
    // through the renderer — different prefix glyph + wider wiggle start.
    let plain_variant = build_trill_bracket_with_mordent_plain_variant();
    assert_ne!(
        svg, plain_variant,
        "bracketed compound must differ from bracketed plain trill: \
         different prefix glyph, wider wiggle start"
    );

    // Structural guard 3: hook geometry is glyph-independent, so the
    // bracketed compound and bracketed plain variants must emit the same
    // number of <line> elements. (The prefix glyph affects path data, not
    // line count.)
    assert_eq!(
        bracket_lines,
        plain_variant.matches("<line ").count(),
        "bracket hook count must be glyph-independent — plain and compound \
         brackets emit the same <line> count"
    );

    assert_golden("trill_bracket_with_mordent", &svg);
}

/// Build the canonical mixed-speed compound-trill score: every variant of
/// `TrillWiggleSpeed` paired with `Ornament::TrillWithMordent` via
/// `TrillExtensionSpeedOptions::new(speed).with_ornament(...)`. Three
/// measures per system × 9 speeds + 1 terminating measure = 4 systems.
///
/// Closes the visual-proofing gap from the 2026-05-13
/// `TrillExtensionSpeedOptions` introduction chunk by freezing a baseline
/// for "compound at every speed."
fn build_trill_speed_with_mordent() -> String {
    use music_engraver::layout::trill_extension::{
        TrillExtensionSpeedOptions, TrillWiggleSpeed,
    };
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
    ];
    for (i, speed) in TrillWiggleSpeed::ALL.iter().enumerate() {
        let (n, oct) = pitches[i];
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed_with_options(
                TrillExtensionSpeedOptions::new(*speed)
                    .with_ornament(Ornament::TrillWithMordent),
            )
            .barline();
    }
    b.note(p("B", 5), Duration::QTR).end_barline().render_svg()
}

/// Same musical content + speeds, but using plain
/// `trill_with_extension_speed(speed)` (`Ornament::Trill`, no compound).
/// Regression-canary baseline for the golden test.
fn build_trill_speed_with_mordent_plain_trill_variant() -> String {
    use music_engraver::layout::trill_extension::TrillWiggleSpeed;
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
    ];
    for (i, speed) in TrillWiggleSpeed::ALL.iter().enumerate() {
        let (n, oct) = pitches[i];
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed(*speed)
            .barline();
    }
    b.note(p("B", 5), Duration::QTR).end_barline().render_svg()
}

/// Same musical content, every measure uses the compound ornament at
/// `Slowest` speed. Lower bookend for the sandwich check that the mixed
/// speeds aren't being collapsed to a single glyph.
fn build_trill_speed_with_mordent_all_slowest() -> String {
    use music_engraver::layout::trill_extension::{
        TrillExtensionSpeedOptions, TrillWiggleSpeed,
    };
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
    ];
    for (n, oct) in pitches {
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed_with_options(
                TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slowest)
                    .with_ornament(Ornament::TrillWithMordent),
            )
            .barline();
    }
    b.note(p("B", 5), Duration::QTR).end_barline().render_svg()
}

/// Same musical content, every measure uses the compound ornament at
/// `Fastest` speed. Upper bookend.
fn build_trill_speed_with_mordent_all_fastest() -> String {
    use music_engraver::layout::trill_extension::{
        TrillExtensionSpeedOptions, TrillWiggleSpeed,
    };
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
    ];
    for (n, oct) in pitches {
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed_with_options(
                TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fastest)
                    .with_ornament(Ornament::TrillWithMordent),
            )
            .barline();
    }
    b.note(p("B", 5), Duration::QTR).end_barline().render_svg()
}

#[test]
fn golden_trill_speed_with_mordent() {
    let svg = build_trill_speed_with_mordent();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Structural guard 1: regression canary that the `.with_ornament(...)`
    // on the speed options actually propagated through the renderer. The
    // same speeds with plain `Trill` ornament must produce a byte-different
    // SVG — different prefix glyph (bare "tr" vs precomposed compound), and
    // different wiggle start positions (compound is ~470 fu wider).
    let plain_trill = build_trill_speed_with_mordent_plain_trill_variant();
    assert_ne!(
        svg, plain_trill,
        "compound-speed mixed score must differ from plain-trill-speed mixed score: \
         different prefix glyph + wider wiggle start"
    );

    // Structural guard 2: the mixed-speed score must have strictly more
    // tile paths than the all-Slowest compound bookend, and strictly
    // fewer than the all-Fastest compound bookend. Sandwich check that
    // none of the 9 speeds are being silently collapsed to a single glyph.
    let all_slowest = build_trill_speed_with_mordent_all_slowest();
    let all_fastest = build_trill_speed_with_mordent_all_fastest();
    let mixed_paths = svg.matches("<path").count();
    let slowest_paths = all_slowest.matches("<path").count();
    let fastest_paths = all_fastest.matches("<path").count();
    assert!(
        mixed_paths > slowest_paths,
        "mixed-speed compound must produce more tile paths than all-Slowest: \
         mixed={mixed_paths}, slowest={slowest_paths}"
    );
    assert!(
        fastest_paths > mixed_paths,
        "all-Fastest compound must produce more tile paths than mixed: \
         fastest={fastest_paths}, mixed={mixed_paths}"
    );

    // Structural guard 3: mixed compound must produce more paths than
    // a plain compound-only baseline (no extension at all). Confirms the
    // wiggle is rendering for at least some of the speeds.
    let no_extension = {
        let mut b = ScoreBuilder::new()
            .clef(Clef::Treble)
            .key_signature(KeySignature::Open)
            .time_signature(4, 4)
            .measures_per_system(3);
        let pitches = [
            ("G", 4),
            ("A", 4),
            ("B", 4),
            ("C", 5),
            ("D", 5),
            ("E", 5),
            ("F", 5),
            ("G", 5),
            ("A", 5),
        ];
        for (n, oct) in pitches {
            b = b
                .note(p(n, oct), Duration::WHOLE)
                .ornament(Ornament::TrillWithMordent)
                .barline();
        }
        b.note(p("B", 5), Duration::QTR).end_barline().render_svg()
    };
    let no_ext_paths = no_extension.matches("<path").count();
    assert!(
        mixed_paths > no_ext_paths,
        "compound-speed with extension must add wiggle paths beyond plain compound: \
         mixed_ext={mixed_paths}, no_ext={no_ext_paths}"
    );

    assert_golden("trill_speed_with_mordent", &svg);
}

/// Build the canonical `TrillExtensionFullOptions` proofing score: 4 measures
/// across 2 systems, each picking a bracket+speed+ornament combination that
/// neither single-purpose options bundle can express on its own. Mirrors the
/// content of `examples/trill_full_options_score.rs`.
///
/// Closes the visual-proofing follow-up explicitly flagged in the "Next" /
/// "Open issues" section of the 2026-05-13 `TrillExtensionFullOptions`
/// introduction chunk.
fn build_trill_full_options() -> String {
    use music::notation::rhythm::duration::DurationKind;
    use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
    use music_engraver::layout::trill_extension::TrillWiggleSpeed;
    use music_engraver::layout::trill_options::TrillExtensionFullOptions;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Both bracket + Slow speed + TrillWithMordent.
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        // M2: End bracket + Faster speed + plain Trill + Up direction.
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up)
                .with_speed(TrillWiggleSpeed::Faster),
        )
        .barline()
        // M3: Start bracket + Slowest speed + TrillWithMordent + 1.0ss length.
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Start)
                .with_bracket_length_ss(1.0)
                .with_speed(TrillWiggleSpeed::Slowest)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        // M4: chord + Both bracket + Standard speed + TrillWithMordent.
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .note(p("D", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same musical content + same ornaments + same speeds + same extension flag,
/// but every measure's `.bracket = None` (no hooks). Bracket-isolation
/// baseline for the golden test's hook-line delta check.
fn build_trill_full_options_no_bracket_variant() -> String {
    use music::notation::rhythm::duration::DurationKind;
    use music_engraver::layout::trill_extension::TrillWiggleSpeed;
    use music_engraver::layout::trill_options::TrillExtensionFullOptions;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new().with_speed(TrillWiggleSpeed::Faster),
        )
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_speed(TrillWiggleSpeed::Slowest)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .note(p("D", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same brackets + same speeds + same extension flag, but every measure's
/// `.ornament = None` (collapses to `Trill` at the builder). Ornament-
/// propagation baseline for the golden test's plain-vs-compound canary.
fn build_trill_full_options_plain_trill_variant() -> String {
    use music::notation::rhythm::duration::DurationKind;
    use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
    use music_engraver::layout::trill_extension::TrillWiggleSpeed;
    use music_engraver::layout::trill_options::TrillExtensionFullOptions;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow),
        )
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up)
                .with_speed(TrillWiggleSpeed::Faster),
        )
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Start)
                .with_bracket_length_ss(1.0)
                .with_speed(TrillWiggleSpeed::Slowest),
        )
        .barline()
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard),
        )
        .note(p("D", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_full_options() {
    let svg = build_trill_full_options();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Structural guard 1: exact 6-hook delta vs the no-bracket variant.
    // M1 Both=2 + M2 End=1 + M3 Start=1 + M4 chord Both=2 = 6 hook lines.
    // Catches a regression where a bracket field stops propagating from the
    // full-options bundle to the annotation, or where the renderer drops a
    // hook for one of the four bracket variants.
    let no_bracket = build_trill_full_options_no_bracket_variant();
    let full_lines = svg.matches("<line ").count();
    let no_bracket_lines = no_bracket.matches("<line ").count();
    let delta = full_lines.saturating_sub(no_bracket_lines);
    assert_eq!(
        delta, 6,
        "expected exactly 6 bracket hook lines (M1 Both=2 + M2 End=1 + M3 Start=1 + M4 chord Both=2); \
         got delta={delta} (full={full_lines}, no_bracket={no_bracket_lines})"
    );

    // Structural guard 2: ornament propagation — the same bracket+speed
    // settings on plain `Trill` must produce a byte-different SVG. The
    // compound glyph is ~470 fu wider than bare "tr" in Bravura, so the
    // wiggle start positions differ. Three of the four measures use
    // `TrillWithMordent`, so this must hold.
    let plain_trill = build_trill_full_options_plain_trill_variant();
    assert_ne!(
        svg, plain_trill,
        "full-options compound score must differ from same options with ornament=None: \
         different prefix glyph, different wiggle start positions"
    );

    // Structural guard 3: bracket geometry is glyph-independent — the plain
    // and compound variants must emit the same `<line>` count. Sandwiches
    // the previous assertion: ornament changed path data, but it did NOT
    // change the bracket hook count.
    assert_eq!(
        full_lines,
        plain_trill.matches("<line ").count(),
        "bracket hook count must be glyph-independent — plain and compound \
         brackets emit the same <line> count"
    );

    // Structural guard 4: byte-inequality vs the existing
    // `trill_bracket_with_mordent` baseline. The two golden baselines use
    // closely related shapes (similar measures, similar brackets, similar
    // ornaments), but `trill_full_options` mixes in non-Standard wiggle
    // speeds — that wiggle-speed mix must produce visibly different SVG.
    // If a future refactor accidentally wired `trill_with_extension_full_options`
    // to drop the speed field, the two outputs would converge.
    let bracket_with_mordent = build_trill_bracket_with_mordent();
    assert_ne!(
        svg, bracket_with_mordent,
        "trill_full_options must differ from trill_bracket_with_mordent: \
         the full-options score adds non-Standard speed overrides on 3 of 4 measures"
    );

    assert_golden("trill_full_options", &svg);
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
        "dynamics_variants",
        "dynamics_full",
        "tuplet",
        "slurs",
        "articulations",
        "fermata_variants",
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
        "ornaments_full",
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
        "trill_extension",
        "trill_bracket",
        "trill_bracket_custom",
        "trill_wiggle_speed",
        "trill_with_mordent_extension",
        "trill_bracket_with_mordent",
        "trill_speed_with_mordent",
        "trill_full_options",
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
