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
use music_engraver::layout::articulation::Articulation;
use music_engraver::layout::breath::BreathMark;
use music_engraver::layout::cresc_text::CrescTextKind;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::glissando::GlissandoStyle;
use music_engraver::layout::hairpin::{HairpinType, NientePlacement};
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::lyric::LyricSyllable;
use music_engraver::layout::measure::NoteheadStyle;
use music_engraver::layout::multi_staff::SubBracket;
use music_engraver::layout::navigation::NavigationSign;
use music_engraver::layout::ornament::Ornament;
use music_engraver::layout::ottava::OttavaKind;
use music_engraver::layout::rehearsal::RehearsalStyle;
use music_engraver::layout::tempo::{MetronomeNoteKind, TempoMark};
use music_engraver::layout::tremolo::TremoloCount;
use music_engraver::score::guitar::{
    BendGesture, BendPitch, BendRelease, GuitarMoment, GuitarScore,
};
use music_engraver::score::multi_staff::MultiStaffScore;
use music_engraver::score::tab::TabScoreBuilder;
use music_engraver::score::ScoreBuilder;
use smufl::Glyph;

#[path = "support/advanced_guitar.rs"]
mod advanced_guitar_fixture;

fn p(name: &str, octave: i8) -> Pitch {
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
    Pitch::new(note, octave)
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
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
            diff.push_str(&format!(
                "Line {i}: \n  expected: {exp}\n  actual:   {act}\n"
            ));
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
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4), p("C", 5)],
            Duration::QTR,
        )
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
const DYNAMICS_FULL_PITCHES: [(&str, i8); 28] = [
    ("C", 4),
    ("D", 4),
    ("E", 4),
    ("F", 4),
    ("G", 4),
    ("A", 4),
    ("B", 4),
    ("C", 5),
    ("D", 5),
    ("E", 5),
    ("F", 5),
    ("G", 5),
    ("A", 5),
    ("B", 5),
    ("C", 4),
    ("D", 4),
    ("E", 4),
    ("F", 4),
    ("G", 4),
    ("A", 4),
    ("B", 4),
    ("C", 5),
    ("D", 5),
    ("E", 5),
    ("F", 5),
    ("G", 5),
    ("A", 5),
    ("B", 5),
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

/// Visually-similar `Dynamic` clusters laid out side-by-side so a viewer
/// can A/B compare the letterforms in a single glance.
///
/// - **m-cluster** (M1, 3 dynamics): `Mezzo` / `Mp` / `Mf` — three
///   superficially similar "m"-prefixed glyphs that Bravura draws with
///   distinct shapes.
/// - **sfp-cluster** (M2, 2 dynamics): `Sfp` (sforzando-prefixed) /
///   `SforzatoPiano` (sforzato-prefixed) — both spell "sfp" but use a
///   different `s` letterform.
///
/// 5 dynamics across 2 measures of 4 quarters each. The unattached
/// quarters are plain padding so the structural delta vs the
/// dynamics-stripped baseline is exactly 5.
fn build_dynamics_lookalikes() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: m-cluster
        .note(p("G", 4), Duration::QTR)
        .dynamic(Dynamic::Mezzo)
        .note(p("A", 4), Duration::QTR)
        .dynamic(Dynamic::Mp)
        .note(p("B", 4), Duration::QTR)
        .dynamic(Dynamic::Mf)
        .note(p("C", 5), Duration::QTR)
        .barline()
        // M2: sfp-cluster
        .note(p("G", 4), Duration::QTR)
        .dynamic(Dynamic::Sfp)
        .note(p("A", 4), Duration::QTR)
        .dynamic(Dynamic::SforzatoPiano)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same two-measure layout as `build_dynamics_lookalikes` with no dynamics
/// attached. The golden test uses this as the structural baseline so the
/// lookalike score must differ by exactly 5 paths and 5 unique d-strings.
fn build_dynamics_lookalikes_plain() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .barline()
        .note(p("G", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
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
const FERMATA_PITCHES: [(&str, i8); 7] = [
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

/// Canonical ordering of the 3 SMuFL accent extensions plus LaissezVibrer.
/// All four share the same wiring: the *normal* articulation stack bucket
/// (not fermata, not bow-stroke) and stem-opposite default placement.
/// Locked here so a future regression that reclassifies any of these into
/// the fermata or bow bucket would change the rendered placement and trip
/// the byte-exact baseline.
const ACCENT_EXTENSION_VARIANTS: [Articulation; 4] = [
    Articulation::SoftAccent,
    Articulation::Stress,
    Articulation::Unstress,
    Articulation::LaissezVibrer,
];

/// Pitches for the accent-extension score. Alternating low/high so the
/// engraver assigns alternating stem directions: positions below the
/// middle line stem up (articulation below), positions above the middle
/// line stem down (articulation above). Exercises both the `Above` and
/// `Below` glyph arms for each variant on the same canvas.
const ACCENT_EXTENSION_PITCHES: [(&str, i8); 4] = [
    ("E", 4), // line 1 → stem up → glyph below
    ("C", 5), // 3rd space → stem down → glyph above
    ("G", 4), // line 2 → stem up → glyph below
    ("A", 5), // above staff → stem down → glyph above
];

/// Each of the 4 accent-extension family variants on a half note in its
/// own measure across one 4/4 system. Half notes (not whole notes) so
/// each note carries a real stem and the placement contract is exercised.
fn build_accent_extensions() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4);
    let count = ACCENT_EXTENSION_VARIANTS.len();
    for (i, variant) in ACCENT_EXTENSION_VARIANTS.iter().enumerate() {
        let (n, oct) = ACCENT_EXTENSION_PITCHES[i];
        b = b
            .note(p(n, oct), Duration::HALF)
            .articulation(*variant)
            .rest(Duration::HALF);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
}

/// Same layout as `build_accent_extensions` with no articulations.
/// Structural baseline for the variants test: the variant score must
/// add exactly one path per variant.
fn build_accent_extensions_plain() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4);
    let count = ACCENT_EXTENSION_VARIANTS.len();
    for (i, _variant) in ACCENT_EXTENSION_VARIANTS.iter().enumerate() {
        let (n, oct) = ACCENT_EXTENSION_PITCHES[i];
        b = b.note(p(n, oct), Duration::HALF).rest(Duration::HALF);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
}

/// Canonical ordering of the 2 SMuFL bow-stroke variants. Both share the
/// always-above placement contract and live in the dedicated bow-stroke
/// stack bucket — distinct from the normal (stem-opposite) bucket covered
/// by the accent-extensions golden, and distinct from the fermata bucket
/// covered by the fermata-variants golden. Locked here so a future
/// regression that reclassifies either of these into the normal bucket
/// (placing them stem-opposite) or the fermata bucket (placing them
/// outside any fermatas) would change the rendered placement and trip
/// the byte-exact baseline.
const BOW_STROKE_VARIANTS: [Articulation; 2] = [Articulation::UpBow, Articulation::DownBow];

/// Pitches for the bow-stroke score. Alternating low/high so the engraver
/// assigns alternating stem directions across the two measures: position
/// 1 (E4) stems up, position 7 (C5) stems down. Because bow strokes
/// **always** render above regardless of stem direction, the score on
/// the page must still place both glyphs above — the alternation
/// specifically tests that the always-above contract is independent of
/// stem direction. (Compare with `ACCENT_EXTENSION_PITCHES`, where
/// alternation is what *causes* both Above and Below arms to be
/// exercised.)
const BOW_STROKE_PITCHES: [(&str, i8); 2] = [
    ("E", 4), // line 1 → stem up; bow still above
    ("C", 5), // 3rd space → stem down; bow still above
];

/// Each of the 2 bow-stroke variants on a half note in its own measure
/// across one 2-measure system. HALF (not WHOLE) so each note carries a
/// real stem, exercising the stem-independence of the always-above
/// placement rule.
fn build_bow_strokes() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    let count = BOW_STROKE_VARIANTS.len();
    for (i, variant) in BOW_STROKE_VARIANTS.iter().enumerate() {
        let (n, oct) = BOW_STROKE_PITCHES[i];
        b = b
            .note(p(n, oct), Duration::HALF)
            .articulation(*variant)
            .rest(Duration::HALF);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
}

/// Same layout as `build_bow_strokes` with no articulations. Used as the
/// structural baseline for the bow-stroke golden test: the variant score
/// must differ from this by exactly one path per variant, and the
/// variant score's d-string set must contribute exactly
/// `BOW_STROKE_VARIANTS.len()` unique d-strings.
fn build_bow_strokes_plain() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2);
    let count = BOW_STROKE_VARIANTS.len();
    for (i, _variant) in BOW_STROKE_VARIANTS.iter().enumerate() {
        let (n, oct) = BOW_STROKE_PITCHES[i];
        b = b.note(p(n, oct), Duration::HALF).rest(Duration::HALF);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
}

/// Canonical ordering of the 4 SMuFL combined-articulation variants —
/// AccentStaccato, MarcatoStaccato, TenutoStaccato, TenutoAccent. These
/// are SMuFL shorthand glyphs for what would otherwise be a two-glyph
/// stack of the simpler primitives (e.g. `AccentStaccato` is the single
/// glyph for what would normally be Accent + Staccato stacked). All four
/// share the same wiring as `ACCENT_EXTENSION_VARIANTS`: the *normal*
/// articulation stack bucket (not fermata, not bow-stroke) and
/// stem-opposite default placement. They predate the golden-coverage
/// pattern introduced by `golden_fermata_variants` /
/// `golden_accent_extensions` / `golden_bow_strokes`, so this list locks
/// them in too — guarding against a regression that reclassifies any
/// of them into the fermata bucket (placing them outside any fermatas)
/// or the bow-stroke bucket (placing them at the wrong y-offset), or a
/// `glyph()` arm typo that collapses any two onto the same SMuFL outline.
const COMBINED_ARTICULATION_VARIANTS: [Articulation; 4] = [
    Articulation::AccentStaccato,
    Articulation::MarcatoStaccato,
    Articulation::TenutoStaccato,
    Articulation::TenutoAccent,
];

/// Pitches for the combined-articulation score. Alternating low/high so
/// the engraver assigns alternating stem directions across the four
/// measures: positions below the middle line stem up (articulation
/// placed below the notehead), positions above the middle line stem
/// down (articulation placed above). Exercises both the `Above` and
/// `Below` glyph arms for each of the four variants on the same canvas.
/// Same pitch set as `ACCENT_EXTENSION_PITCHES` by design — the
/// byte-exact baseline must still differ from the accent-extensions
/// baseline because the glyphs themselves are distinct, so reusing the
/// same pitches isolates glyph routing as the only source of difference.
const COMBINED_ARTICULATION_PITCHES: [(&str, i8); 4] = [
    ("E", 4), // line 1 → stem up → glyph below
    ("C", 5), // 3rd space → stem down → glyph above
    ("G", 4), // line 2 → stem up → glyph below
    ("A", 5), // above staff → stem down → glyph above
];

/// Each of the 4 combined-articulation variants on a half note in its
/// own measure across one 4/4 system. Half notes (not whole notes) so
/// each note carries a real stem and the stem-opposite placement
/// contract is exercised. Mirror of `build_accent_extensions` with the
/// variant list and pitch list swapped out.
fn build_combined_articulations() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4);
    let count = COMBINED_ARTICULATION_VARIANTS.len();
    for (i, variant) in COMBINED_ARTICULATION_VARIANTS.iter().enumerate() {
        let (n, oct) = COMBINED_ARTICULATION_PITCHES[i];
        b = b
            .note(p(n, oct), Duration::HALF)
            .articulation(*variant)
            .rest(Duration::HALF);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
}

/// Same layout as `build_combined_articulations` with no articulations.
/// Structural baseline for the combined-articulation golden test: the
/// variant score must add exactly one path per variant on top of this
/// baseline.
fn build_combined_articulations_plain() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(4);
    let count = COMBINED_ARTICULATION_VARIANTS.len();
    for (i, _variant) in COMBINED_ARTICULATION_VARIANTS.iter().enumerate() {
        let (n, oct) = COMBINED_ARTICULATION_PITCHES[i];
        b = b.note(p(n, oct), Duration::HALF).rest(Duration::HALF);
        if i + 1 < count {
            b = b.barline();
        }
    }
    b.end_barline().render_svg()
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

/// Octave-down treble clef (the guitar clef) over a typical guitar-part range.
///
/// A transposing clef changes the *sounding* pitch, not staff placement, so
/// these written pitches must land on exactly the same lines as they do under
/// plain treble — see `build_treble_for_octave_clef_comparison` and
/// `transposing_clefs_place_notes_like_treble`. Regression for
/// docs/slonimsky-cli-bugs.md §7, where `treble8ba` shifted noteheads an octave
/// up and pushed an ordinary phrase onto ledger lines.
fn build_treble8ba_clef() -> String {
    build_octave_clef_case(Clef::Treble8ba)
}

/// Octave-up treble clef, same content — the mirror of the `8vb` case.
fn build_treble8va_clef() -> String {
    build_octave_clef_case(Clef::Treble8va)
}

/// Plain treble with the same content, as the placement reference.
fn build_treble_for_octave_clef_comparison() -> String {
    build_octave_clef_case(Clef::Treble)
}

/// Shared body for the octave-clef goldens: a Bb-major phrase spanning the
/// staff, with a beamed pair so beam geometry is frozen alongside the noteheads.
fn build_octave_clef_case(clef: Clef) -> String {
    ScoreBuilder::new()
        .clef(clef)
        .key_signature(KeySignature::Flats(2))
        .time_signature(4, 4)
        .note(p("Bb", 4), Duration::QTR)
        .beam_group(vec![
            (p("C", 5), Duration::EIGHTH),
            (p("D", 5), Duration::EIGHTH),
        ])
        .note(p("F", 5), Duration::QTR)
        .note(p("G", 5), Duration::QTR)
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

/// Five-staff section with two nested sub-brackets (Violin I + Violin II
/// share one inner bracket; Viola + Cello + Bass share another), under a
/// single outer section bracket. Exercises the score-level
/// `MultiStaffScore::with_sub_brackets(...)` builder end-to-end and locks
/// the SVG output against unintended layout/render drift.
fn build_sub_brackets_score() -> String {
    fn line(clef: Clef, pitches: &[(&str, i8)]) -> ScoreBuilder {
        let mut b = ScoreBuilder::new().clef(clef).time_signature(4, 4);
        for (n, oct) in pitches {
            b = b.note(p(n, *oct), Duration::QTR);
        }
        b.end_barline()
    }

    let v1 = line(Clef::Treble, &[("E", 5), ("G", 5), ("A", 5), ("B", 5)]);
    let v2 = line(Clef::Treble, &[("C", 5), ("E", 5), ("F", 5), ("G", 5)]);
    let va = line(Clef::Treble, &[("G", 4), ("A", 4), ("B", 4), ("C", 5)]);
    let vc = line(Clef::Bass, &[("E", 3), ("G", 3), ("A", 3), ("B", 3)]);
    let cb = line(Clef::Bass, &[("E", 2), ("E", 2), ("E", 2), ("E", 2)]);

    MultiStaffScore::section(vec![v1, v2, va, vc, cb])
        .with_sub_brackets(vec![
            SubBracket {
                start_index: 0,
                staff_count: 2,
            },
            SubBracket {
                start_index: 2,
                staff_count: 3,
            },
        ])
        .render_svg()
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
        ("C", 4),
        ("D", 4),
        ("E", 4),
        ("F", 4),
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
        ("B", 5),
        ("C", 6),
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

/// Cross-system hairpins: a crescendo that spans a system break (rendered as
/// a solid trailing half-wedge in system 1 plus a *dashed* incoming half-wedge
/// in system 2), plus a within-system decrescendo on system 2 for contrast.
fn build_cross_system_hairpins() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // System 1, measure 1: cresc starts here
        .note(p("C", 4), Duration::QTR)
        .dynamic(Dynamic::Pp)
        .hairpin_start(HairpinType::Crescendo)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // System 1, measure 2: cresc continues — no end before the system break
        .note(p("G", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .barline()
        // System 2, measure 3: cresc ends; then a within-system decresc
        .note(p("B", 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Ff)
        .note(p("A", 4), Duration::QTR)
        .hairpin_start(HairpinType::Decrescendo)
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Piano)
        .barline()
        // System 2, measure 4: final plain phrase
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("C", 4), Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Same notes/structure as `build_cross_system_hairpins`, but with every
/// hairpin and dynamic removed. Used as a delta baseline so the cross-system
/// hairpin golden can assert *exactly* how many extra `<line>` elements the
/// hairpins contribute.
fn build_cross_system_hairpins_baseline() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        .note(p("G", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .barline()
        .note(p("B", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("C", 4), Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Dashed-text dynamic markings via `ScoreBuilder`: exercises all three
/// `CrescTextKind` variants (`Crescendo`, `Decrescendo`, `Diminuendo`) on
/// distinct note spans within a single system. Each kind produces one
/// italic label (`>cresc.</text>`, `>decresc.</text>`, `>dim.</text>`) plus
/// one dashed continuation line, so the rendered SVG must add exactly
/// 3 dashed lines and 3 italic-styled `<text>` elements over the matching
/// no-marking baseline (`build_cresc_text_baseline`).
///
/// Layout: 3 measures of 4 quarters each at 4 measures/system (default).
/// Each marking lives entirely within its own measure so the three labels
/// and their dashed continuations sit side-by-side along the staff
/// baseline. No system break is forced; the cross-system continuation
/// path is covered by `build_cross_system_cresc_text` instead.
fn build_cresc_text() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        // Measure 1: cresc. spans notes 1..4
        .note(p("C", 4), Duration::QTR)
        .cresc_text()
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .cresc_text_end()
        .barline()
        // Measure 2: decresc. spans notes 1..4
        .note(p("G", 4), Duration::QTR)
        .decresc_text()
        .note(p("F", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .cresc_text_end()
        .barline()
        // Measure 3: dim. spans notes 1..4
        .note(p("C", 4), Duration::QTR)
        .dim_text()
        .note(p("B", 3), Duration::QTR)
        .note(p("A", 3), Duration::QTR)
        .note(p("G", 3), Duration::QTR)
        .cresc_text_end()
        .end_barline()
        .render_svg()
}

/// Same notes/structure as `build_cresc_text` with every `cresc_text_*`
/// call removed. Used as a delta baseline so the within-system cresc-text
/// golden can assert *exactly* how many extra `<line>` / `<text>` /
/// `stroke-dasharray` elements the three markings contribute over the
/// underlying staff/clef/barline rendering.
fn build_cresc_text_baseline() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .barline()
        .note(p("C", 4), Duration::QTR)
        .note(p("B", 3), Duration::QTR)
        .note(p("A", 3), Duration::QTR)
        .note(p("G", 3), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Cross-system dashed-text crescendo: a `cresc.` marking whose start
/// lands on system 1 and whose end lands on system 2. The engraved
/// convention (mirrored by `layout_cresc_text_continuation`) is that
/// the italic label lives on the *source* (start) side only, with a
/// dashed continuation line spanning each half — so the page renderer
/// must emit exactly 1 label and 2 dashed lines for the cross-system
/// pair, not 2 labels.
///
/// Layout: 4 measures of 4 quarters each at 2 measures/system → 2
/// systems. The cresc. starts on measure 1 and ends on measure 3 (first
/// note of system 2), forcing the page renderer's
/// `draw_cross_system_cresc_texts` code path.
fn build_cross_system_cresc_text() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // System 1, measure 1: cresc. starts here
        .note(p("C", 4), Duration::QTR)
        .cresc_text_start(CrescTextKind::Crescendo)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // System 1, measure 2: cresc. continues across system break
        .note(p("G", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .barline()
        // System 2, measure 3: cresc. ends on the first note
        .note(p("B", 4), Duration::QTR)
        .cresc_text_end()
        .note(p("A", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // System 2, measure 4: final plain phrase
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("C", 4), Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Same notes/structure as `build_cross_system_cresc_text` but with the
/// `cresc_text_start` / `cresc_text_end` calls removed. Used as a delta
/// baseline so the cross-system cresc-text golden can pin the exact
/// label / dashed-line contribution of the cross-system path.
fn build_cross_system_cresc_text_baseline() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        .note(p("G", 4), Duration::HALF)
        .note(p("A", 4), Duration::HALF)
        .barline()
        .note(p("B", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("C", 4), Duration::HALF)
        .end_barline()
        .render_svg()
}

/// Combined hairpin styling: dashed wedge + niente "o" circle, exercising
/// both [`NientePlacement`] variants on a within-system score. Mirrors the
/// `build_cresc_text` pattern (multiple variants laid out per-measure so the
/// SVG element-count deltas pin each variant's contribution).
///
/// Layout: 2 measures of 4 quarters each at 4 measures/system (default), so
/// both hairpins fit within a single system — within-system wedge geometry
/// only (no cross-system continuation, which is exercised by
/// `build_cross_system_hairpins`).
///
/// Per measure: one full hairpin (start → end) plus a `hairpin_dashed()` and
/// a `hairpin_niente*()` call on the start note. The renderer must produce
/// (over the no-marking baseline):
///   - exactly 2 `<circle>` elements (one niente "o" per hairpin), both with
///     `fill="none"` and NO `stroke-dasharray` (engraved convention: the
///     circle stays solid even on a dashed wedge);
///   - exactly 4 `stroke-dasharray` attributes (each within-system dashed
///     wedge = 2 lines × 2 hairpins);
///   - exactly 4 extra `<line>` elements (each within-system wedge = 2
///     lines × 2 hairpins).
///
/// Variants per measure:
///   1. Crescendo + `hairpin_dashed` + `hairpin_niente()` (ClosedEnd
///      convenience): circle at the closed (pointy) tip = wedge start.
///   2. Decrescendo + `hairpin_dashed` + `hairpin_niente_start(OpenEnd)`:
///      circle at the open (wide) tip = wedge start (modern convention).
fn build_hairpin_niente_dashed() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        // Measure 1: crescendo + dashed + closed-end niente.
        .note(p("C", 4), Duration::QTR)
        .dynamic(Dynamic::Pp)
        .hairpin_start(HairpinType::Crescendo)
        .hairpin_dashed()
        .hairpin_niente()
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Forte)
        .barline()
        // Measure 2: decrescendo + dashed + open-end niente.
        .note(p("G", 4), Duration::QTR)
        .dynamic(Dynamic::Forte)
        .hairpin_start(HairpinType::Decrescendo)
        .hairpin_dashed()
        .hairpin_niente_start(NientePlacement::OpenEnd)
        .note(p("F", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Pp)
        .end_barline()
        .render_svg()
}

/// Same notes/structure as `build_hairpin_niente_dashed` with every
/// `hairpin_*` / `dynamic` call removed. Used as a delta baseline so the
/// golden can assert exactly how many extra `<line>`, `<circle>`, and
/// `stroke-dasharray` elements the combo markings contribute over the
/// underlying staff/clef/notes/barline rendering.
fn build_hairpin_niente_dashed_baseline() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
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

/// Chord symbols that exercise SMuFL accidental glyph composition: flat-root
/// (`Bb`, `Ebmaj7`), sharp-root (`F#m`, `C#7`), altered-extension flats
/// (`F#m7b5`, `C7b9`), altered-extension sharps (`D7#9`), and slash chords
/// (`D/Bb`). The plain symbols (`Cmaj7`, `Am`, `G7`, `F`) are included as
/// byte-stability controls — they must round-trip through the composite
/// renderer to single `<text>` elements with the same visual position as the
/// non-composite renderer.
fn build_chord_symbols_with_accidentals() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("Bb", 3), Duration::QTR)
        .chord_symbol("Bb")
        .note(p("Eb", 4), Duration::QTR)
        .chord_symbol("Ebmaj7")
        .note(p("F", 4), Duration::QTR)
        .chord_symbol("F")
        .note(p("Bb", 3), Duration::QTR)
        .chord_symbol("Bb7")
        .barline()
        .note(p("F#", 4), Duration::QTR)
        .chord_symbol("F#m")
        .note(p("C#", 4), Duration::QTR)
        .chord_symbol("C#7")
        .note(p("F#", 4), Duration::QTR)
        .chord_symbol("F#m7b5")
        .note(p("C", 4), Duration::QTR)
        .chord_symbol("C7b9")
        .barline()
        .note(p("D", 4), Duration::QTR)
        .chord_symbol("D7#9")
        .note(p("C", 4), Duration::QTR)
        .chord_symbol("Cmaj7")
        .note(p("D", 4), Duration::QTR)
        .chord_symbol("D/Bb")
        .note(p("G", 4), Duration::QTR)
        .chord_symbol("G7")
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
fn golden_dynamics_lookalikes() {
    let svg = build_dynamics_lookalikes();
    let plain = build_dynamics_lookalikes_plain();

    // Structural validity
    assert!(svg.starts_with("<svg"), "dynamics_lookalikes should be SVG");
    assert!(
        svg.contains("</svg>"),
        "dynamics_lookalikes should close SVG"
    );

    // Path-count guard: the 5 lookalike dynamics (Mezzo/Mp/Mf in M1,
    // Sfp/SforzatoPiano in M2) must add exactly 5 paths on top of the
    // dynamics-stripped baseline. Catches a silent regression where one
    // of these less-common glyphs maps to an empty font slot.
    let full_paths = svg.matches("<path").count();
    let plain_paths = plain.matches("<path").count();
    let added_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        added_paths, 5,
        "the 5 lookalike dynamics must add exactly 5 paths: full={full_paths}, \
         plain={plain_paths}, delta={added_paths}, expected=5"
    );

    // Distinct-d guard: the whole point of putting these glyphs
    // side-by-side is that each is a distinct shape in Bravura. If two
    // ever collapsed to the same glyph (e.g. a future font swap aliased
    // `Mezzo` to `Mp` or unified the two `s` letterforms in
    // `Sfp`/`SforzatoPiano`), the comparison example would mislead. This
    // assertion is the canary.
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
        5,
        "the 5 lookalike dynamics must contribute 5 unique path d-strings \
         (no glyph collapse in either cluster), got {}",
        added.len()
    );

    // Byte-inequality vs neighboring dynamics baselines. The `dynamics`
    // baseline shares some glyphs (Mp/Mf appear there too) but uses
    // different gestures (hairpins, more variants); `dynamics_variants`
    // shares Mezzo/SforzatoPiano but adds `Z` and uses a single-measure
    // C–F layout. Sanity check that the new golden isn't accidentally
    // identical to a prior one — if it were, this test would silently be
    // duplicate coverage rather than independent verification.
    assert_ne!(
        svg,
        build_dynamics(),
        "dynamics_lookalikes must differ from the existing `dynamics` baseline"
    );
    assert_ne!(
        svg,
        build_dynamics_variants(),
        "dynamics_lookalikes must differ from the `dynamics_variants` baseline"
    );

    assert_golden("dynamics_lookalikes", &svg);
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
fn golden_accent_extensions() {
    let svg = build_accent_extensions();
    let plain = build_accent_extensions_plain();

    // Structural validity
    assert!(svg.starts_with("<svg"), "accent_extensions should be SVG");
    assert!(svg.contains("</svg>"), "accent_extensions should close SVG");

    // Path-count guard: each variant must add exactly one path on top of
    // the same score with no articulations. Catches a silent regression
    // where any of the 4 variants maps to a missing glyph (zero paths)
    // or to a multi-path glyph (more than 1).
    let full_paths = svg.matches("<path").count();
    let plain_paths = plain.matches("<path").count();
    let added_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        added_paths,
        ACCENT_EXTENSION_VARIANTS.len(),
        "each accent-extension variant must add exactly one path: \
         full={full_paths}, plain={plain_paths}, delta={added_paths}, \
         expected={}",
        ACCENT_EXTENSION_VARIANTS.len()
    );

    // Distinct-d guard: each variant must contribute a unique SMuFL path
    // payload. Bravura ships distinct outlines for ArticSoftAccent,
    // ArticStress, ArticUnstress, and ArticLaissezVibrer — if any two
    // glyphs accidentally collapse to the same d-string (e.g. a typo'd
    // `glyph()` arm aliasing one onto another), this assertion fails.
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
        ACCENT_EXTENSION_VARIANTS.len(),
        "accent-extension variants must contribute exactly {} unique \
         path d-strings (no glyph collapses), got {}",
        ACCENT_EXTENSION_VARIANTS.len(),
        added.len()
    );

    // Placement-contract guard: the score alternates pitches below and
    // above the middle line (E4, C5, G4, A5). For HALF notes the
    // engraver picks the stem direction from staff position, which
    // sends each variant to alternating sides. The 2 "above" glyphs
    // and 2 "below" glyphs must each be distinct d-strings, so we
    // expect the added set to contain at least 4 unique d-strings —
    // already covered by the prior assertion. Additionally, sanity-
    // check that the *plain* score, which has none of these glyphs,
    // does not coincidentally contain any of the added d-strings.
    let plain_set = distinct_d(&plain);
    for d in &added {
        assert!(
            !plain_set.contains(d),
            "accent-extension d-string {d:?} should not appear in \
             the plain (no-articulation) baseline"
        );
    }

    // Sanity check: this score must differ from the existing
    // `articulations` baseline (different gesture set, different
    // glyphs, different layout). Catches an accidental copy that
    // collapses two goldens onto the same SVG.
    assert_ne!(
        svg,
        build_articulations(),
        "accent_extensions must differ from the existing articulations baseline"
    );
    // And from the fermata-variants baseline (same shape, completely
    // different glyph family).
    assert_ne!(
        svg,
        build_fermata_variants(),
        "accent_extensions must differ from the fermata_variants baseline"
    );

    assert_golden("accent_extensions", &svg);
}

#[test]
fn golden_bow_strokes() {
    let svg = build_bow_strokes();
    let plain = build_bow_strokes_plain();

    // Structural validity
    assert!(svg.starts_with("<svg"), "bow_strokes should be SVG");
    assert!(svg.contains("</svg>"), "bow_strokes should close SVG");

    // Path-count guard: each variant must add exactly one path on top of
    // the same score with no articulations. Catches a silent regression
    // where either of the 2 variants maps to a missing glyph (zero
    // paths) or to a multi-path glyph (more than 1).
    let full_paths = svg.matches("<path").count();
    let plain_paths = plain.matches("<path").count();
    let added_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        added_paths,
        BOW_STROKE_VARIANTS.len(),
        "each bow-stroke variant must add exactly one path: \
         full={full_paths}, plain={plain_paths}, delta={added_paths}, \
         expected={}",
        BOW_STROKE_VARIANTS.len()
    );

    // Distinct-d guard: each variant must contribute a unique SMuFL
    // path payload. Bravura ships distinct outlines for StringsUpBow and
    // StringsDownBow — if a `glyph()` arm typo collapsed them onto the
    // same glyph, this assertion fails.
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
        BOW_STROKE_VARIANTS.len(),
        "bow-stroke variants must contribute exactly {} unique path \
         d-strings (no glyph collapses), got {}",
        BOW_STROKE_VARIANTS.len(),
        added.len()
    );

    // Plain-disjoint check: every newly-added d-string must NOT appear
    // in the plain (no-articulation) baseline. Catches the (unlikely
    // but possible) regression where a bow-stroke glyph's path data
    // coincidentally matches a notehead, rest, or clef path in the
    // baseline.
    let plain_set = distinct_d(&plain);
    for d in &added {
        assert!(
            !plain_set.contains(d),
            "bow-stroke d-string {d:?} should not appear in the plain \
             (no-articulation) baseline"
        );
    }

    // Placement-contract guard: bow strokes always render *above* the
    // notehead regardless of stem direction. The two notes in this
    // score (E4, C5) have opposite auto-assigned stem directions
    // (up, down respectively). For each bow-stroke glyph, the y of its
    // `<path transform="translate(x,y)`-style anchor must be ABOVE
    // (numerically less than) the notehead-row y. We can't easily
    // recover per-path y from the rendered SVG without parsing
    // transforms, but we can guard the contract by asserting that
    // neither bow d-string aliases any d-string in `build_articulations`
    // (which contains a Staccato on E4 with stem-up — placed BELOW the
    // note via the normal bucket — distinct geometry). If a regression
    // routed bow strokes into the normal bucket, the E4 bow would land
    // below the note (matching the staccato y-region), and the byte-
    // exact baseline below would catch the placement shift. Here we
    // assert the cross-baseline distinctness explicitly.
    assert_ne!(
        svg,
        build_articulations(),
        "bow_strokes must differ from the existing articulations baseline"
    );
    // And from the fermata-variants baseline (different glyph family,
    // also always-above but in the fermata bucket — different y-offset).
    assert_ne!(
        svg,
        build_fermata_variants(),
        "bow_strokes must differ from the fermata_variants baseline"
    );
    // And from the accent-extensions baseline (same overall score
    // shape — 2 measures of HALF + HALF-rest — but different glyph
    // family and different bucket; a byte-equal match would mean the
    // bow variants collapsed onto the accent extensions, which would
    // be a glyph-routing regression).
    assert_ne!(
        svg,
        build_accent_extensions(),
        "bow_strokes must differ from the accent_extensions baseline"
    );

    assert_golden("bow_strokes", &svg);
}

#[test]
fn golden_combined_articulations() {
    let svg = build_combined_articulations();
    let plain = build_combined_articulations_plain();

    // Structural validity
    assert!(
        svg.starts_with("<svg"),
        "combined_articulations should be SVG"
    );
    assert!(
        svg.contains("</svg>"),
        "combined_articulations should close SVG"
    );

    // Path-count guard: each variant must add exactly one path on top of
    // the same score with no articulations. Catches a silent regression
    // where any of the 4 variants maps to a missing glyph (zero paths)
    // or to a multi-path glyph (more than 1) — e.g. if a `glyph()` arm
    // were accidentally changed to render a combined variant as a stack
    // of the two primitive glyphs (Accent + Staccato as two paths
    // instead of the single ArticAccentStaccatoAbove path), the delta
    // would climb to 2 per variant.
    let full_paths = svg.matches("<path").count();
    let plain_paths = plain.matches("<path").count();
    let added_paths = full_paths.saturating_sub(plain_paths);
    assert_eq!(
        added_paths,
        COMBINED_ARTICULATION_VARIANTS.len(),
        "each combined-articulation variant must add exactly one path: \
         full={full_paths}, plain={plain_paths}, delta={added_paths}, \
         expected={}",
        COMBINED_ARTICULATION_VARIANTS.len()
    );

    // Distinct-d guard: each variant must contribute a unique SMuFL path
    // payload. Bravura ships distinct outlines for each of
    // ArticAccentStaccato{Above,Below}, ArticMarcatoStaccato{Above,Below},
    // ArticTenutoStaccato{Above,Below}, ArticTenutoAccent{Above,Below} —
    // if any two variants collapse to the same d-string (a typo'd
    // `glyph()` arm aliasing one onto another), this assertion fails.
    // Note: the pitch alternation means each variant exercises *one* of
    // its arms in this score; the 4 added paths should still be 4
    // distinct d-strings because the 4 variants have 4 distinct glyph
    // outlines.
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
        COMBINED_ARTICULATION_VARIANTS.len(),
        "combined-articulation variants must contribute exactly {} \
         unique path d-strings (no glyph collapses), got {}",
        COMBINED_ARTICULATION_VARIANTS.len(),
        added.len()
    );

    // Plain-disjoint check: every newly-added d-string must NOT appear
    // in the plain (no-articulation) baseline. Catches the (unlikely
    // but possible) regression where a combined-articulation glyph's
    // path data coincidentally matches a notehead, rest, or clef path
    // in the baseline.
    let plain_set = distinct_d(&plain);
    for d in &added {
        assert!(
            !plain_set.contains(d),
            "combined-articulation d-string {d:?} should not appear in \
             the plain (no-articulation) baseline"
        );
    }

    // Cross-baseline distinctness vs `build_articulations()` — catches
    // a glyph-routing regression that collapses a combined variant onto
    // its primitive equivalent (e.g. AccentStaccato → Accent), or that
    // changes the bucket assignment.
    assert_ne!(
        svg,
        build_articulations(),
        "combined_articulations must differ from the existing articulations baseline"
    );
    // Cross-baseline distinctness vs `build_fermata_variants()` — catches
    // a regression that routes a combined variant into the fermata
    // bucket (always-above placement at fermata y-offset).
    assert_ne!(
        svg,
        build_fermata_variants(),
        "combined_articulations must differ from the fermata_variants baseline"
    );
    // Cross-baseline distinctness vs `build_bow_strokes()` — catches a
    // regression that routes a combined variant into the bow-stroke
    // bucket (always-above placement at bow-stroke y-offset).
    assert_ne!(
        svg,
        build_bow_strokes(),
        "combined_articulations must differ from the bow_strokes baseline"
    );
    // Cross-baseline distinctness vs `build_accent_extensions()` — same
    // score shape (4 measures of HALF + HALF-rest with
    // measures_per_system(4) and the same pitch sequence) and same
    // normal-bucket stem-opposite contract; the only thing that should
    // differ is the 4 SMuFL glyph payloads themselves. A byte-equal
    // match would mean the four combined variants collapsed onto the
    // four accent-extension variants (a glyph-routing regression).
    assert_ne!(
        svg,
        build_accent_extensions(),
        "combined_articulations must differ from the accent_extensions baseline"
    );

    assert_golden("combined_articulations", &svg);
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
fn golden_treble8ba_clef() {
    assert_golden("treble8ba_clef", &build_treble8ba_clef());
}

#[test]
fn golden_treble8va_clef() {
    assert_golden("treble8va_clef", &build_treble8va_clef());
}

/// Octave-transposing clefs must place every glyph exactly where plain treble
/// does — only the clef glyph itself differs. Asserted as an invariant rather
/// than left to the frozen baselines, so a future rebaseline cannot quietly
/// bless an octave shift the way the pre-§7 goldens did.
///
/// This compares every drawn element, so it covers beam polygons, stems, and
/// ledger lines — not just noteheads.
#[test]
fn transposing_clefs_place_notes_like_treble() {
    let treble = build_treble_for_octave_clef_comparison();

    for (name, svg) in [
        ("treble8ba", build_treble8ba_clef()),
        ("treble8va", build_treble8va_clef()),
    ] {
        // The clef glyph is the one legitimate difference — and the viewBox
        // that frames it, since page bounds include each clef's own ink (the
        // "8" above or below). Drop the <svg> header and the first <path> (the
        // clef) from each and require the rest to match exactly.
        let strip_clef = |s: &str| -> Vec<String> {
            let mut seen_clef = false;
            s.lines()
                .filter(|l| !l.starts_with("<svg "))
                .filter(|l| {
                    if !seen_clef && l.trim_start().starts_with("<path ") {
                        seen_clef = true;
                        return false;
                    }
                    true
                })
                .map(str::to_owned)
                .collect()
        };

        let base = strip_clef(&treble);
        let other = strip_clef(&svg);

        assert_eq!(
            base.len(),
            other.len(),
            "{name} produced a different element count than treble"
        );
        for (i, (b, o)) in base.iter().zip(other.iter()).enumerate() {
            assert_eq!(
                b, o,
                "{name} differs from treble at line {i} — a transposing clef must not \
                 move noteheads, stems, beams, or the viewBox:\n  treble: {b}\n  {name}: {o}"
            );
        }
    }
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
fn golden_sub_brackets_score() {
    let svg = build_sub_brackets_score();

    // Structural guards alongside the frozen baseline. These would have
    // caught the bugs that the score-level `with_sub_brackets(...)` builder
    // is specifically introduced to prevent.

    // The main bracket is shifted left by 0.76 sp = 190 fu to make room
    // for the sub-brackets, so the scroll glyphs anchor at x = -315
    // (= -125 - 190). A regression that loses the shift would put the
    // scroll back at x = -125.
    assert!(
        svg.contains("translate(-315,"),
        "main bracket scrolls should anchor at x=-315 after the leftward shift"
    );
    assert!(
        !svg.contains("translate(-125,"),
        "main bracket scrolls must NOT anchor at the pre-shift x=-125"
    );

    // Sub-brackets render as thin <line> elements with the SMuFL
    // `subBracketThickness` engraving default (0.16 sp = 40 fu). The
    // baseline must contain at least 2 such lines (one per sub-bracket).
    let thin_count = svg.matches("stroke-width=\"40\"").count();
    assert!(
        thin_count >= 2,
        "expected ≥2 sub-bracket lines at stroke-width=40, got {thin_count}"
    );

    // The main bracket's thick line is at stroke-width = 125 fu
    // (BRACKET_THICKNESS_SS=0.5 × 250). One main bracket → at least 1 hit.
    let main_count = svg.matches("stroke-width=\"125\"").count();
    assert!(
        main_count >= 1,
        "expected ≥1 main-bracket thick line at stroke-width=125, got {main_count}"
    );

    assert_golden("sub_brackets_score", &svg);
}

#[test]
fn golden_lyrics() {
    let svg = build_lyrics();

    // Structural guards alongside the frozen baseline.

    // Hyphen between syllables is a separate centered '-' text element, not
    // appended to the source syllable. The baseline must NEVER contain
    // ">Hap -<" or ">birth -<" (ASCII-concatenated-hyphen regression canary).
    assert!(
        !svg.contains(">Hap -<"),
        "source syllable text must not contain trailing ' -'; rendering regressed"
    );
    assert!(
        !svg.contains(">birth -<"),
        "source syllable text must not contain trailing ' -'; rendering regressed"
    );

    // The source syllables must appear standalone.
    assert!(
        svg.contains(">Hap<"),
        "should contain 'Hap' as its own text"
    );
    assert!(
        svg.contains(">birth<"),
        "should contain 'birth' as its own text"
    );

    // At least two standalone hyphens '-' should appear (one between
    // Hap/py, one between birth/day). The melisma "to/you!" pair uses an
    // extender (a <line>), not a hyphen.
    let hyphen_count = svg.matches(">-<").count();
    assert!(
        hyphen_count >= 2,
        "expected >=2 standalone hyphen text elements, got {hyphen_count}"
    );

    // The extender is still drawn as a <line> for the melisma "to/you!".
    assert!(
        svg.contains("<line"),
        "extender or staff/stem lines should exist"
    );

    assert_golden("lyrics", &svg);
}

#[test]
fn golden_chord_symbols() {
    assert_golden("chord_symbols", &build_chord_symbols());
}

#[test]
fn golden_chord_symbols_with_accidentals() {
    let svg = build_chord_symbols_with_accidentals();

    // Structural guards alongside the frozen baseline.

    // The baseline of `chord_symbols` (no accidentals) must NOT contain any
    // `<path ... d=` elements that come from chord-symbol accidental glyphs.
    // The accidental-bearing baseline must — at minimum one path per
    // accidental in the symbol set: Bb (1), Ebmaj7 (1), Bb7 (1), F#m (1),
    // C#7 (1), F#m7b5 (2), C7b9 (1), D7#9 (1), D/Bb (1) = 10 chord-symbol
    // accidentals. The total `<path>` count includes staff content, so we
    // assert a lower bound that strictly exceeds the no-accidental count of
    // the plain golden.
    let plain = build_chord_symbols();
    let plain_paths = plain.matches("<path").count();
    let accidental_paths = svg.matches("<path").count();
    assert!(
        accidental_paths > plain_paths,
        "with-accidentals SVG should have more <path> elements ({accidental_paths}) than the plain chord_symbols SVG ({plain_paths})"
    );

    // Byte-inequality vs `chord_symbols` — the two scores intentionally
    // differ on every accidental-bearing symbol. If a regression collapsed
    // the composite-renderer path back to plain text, the accidental-bearing
    // baseline would *visually* differ but might produce text content
    // equivalent to the plain-text path; the assertion below also relies on
    // the byte-inequality at the SVG level.
    assert_ne!(
        svg, plain,
        "accidental-bearing score must differ byte-wise from plain chord-symbol score"
    );

    // Composite-renderer-specific marker: text-anchor="start" must appear
    // in the rendered SVG (the composite path always uses start anchor;
    // the simple path uses anchor="middle"). Catches a regression that
    // routes through the simple renderer for accidental-bearing input.
    assert!(
        svg.contains(r#"text-anchor="start""#),
        "composite chord-symbol renderer must emit text-anchor=\"start\""
    );

    // The composite path produces multiple `<text>` elements for symbols
    // with accidentals. F#m7b5 alone produces 3 text runs (F, m7, 5).
    // Total text count across 12 symbols is hard to hand-compute exactly,
    // but it must STRICTLY exceed 12 (would equal 12 if every symbol were
    // a single text run — i.e. all-plain).
    let text_count = svg.matches("<text").count();
    assert!(
        text_count > 12,
        "with-accidentals SVG must have more than 12 text elements (got {text_count}) — \
         indicates segments are being split by the composite renderer"
    );

    // ASCII-text regression canary: the `>F#m7b5<` form must NOT appear
    // anywhere — that was the pre-feature rendering. Same for `>Bb<`.
    assert!(
        !svg.contains(">F#m7b5<"),
        "F#m7b5 should be split into glyph segments, not rendered as a single text run"
    );
    assert!(
        !svg.contains(">Bb<"),
        "Bb should be split into 'B' + flat glyph, not rendered as 'Bb' text"
    );

    assert_golden("chord_symbols_with_accidentals", &svg);
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
        ("C", 4),
        ("D", 4),
        ("E", 4),
        ("F", 4),
        ("G", 4),
        ("A", 4),
        ("B", 4),
        ("C", 5),
        ("D", 5),
        ("E", 5),
        ("F", 5),
        ("G", 5),
        ("A", 5),
        ("B", 5),
        ("C", 6),
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
    let added: HashSet<_> = distinct_d(&svg)
        .difference(&distinct_d(&plain_svg))
        .cloned()
        .collect();
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
fn golden_cross_system_hairpins() {
    let svg = build_cross_system_hairpins();

    // Engraved convention (Gould, *Behind Bars*): the trailing half-wedge on
    // the source system is solid, the incoming half-wedge on the target
    // system is dashed. Each half-wedge is rendered as 2 `<line>` elements
    // (upper + lower arm of the wedge), so the dashed incoming half should
    // contribute *exactly* 2 `stroke-dasharray` occurrences. The within-
    // system decrescendo is fully solid and must not add any dasharray.
    let dasharray_count = svg.matches("stroke-dasharray").count();
    assert_eq!(
        dasharray_count, 2,
        "exactly 2 dasharray attributes expected (the 2 lines of the dashed \
         incoming half-wedge); got {dasharray_count} — \
         a regression would either drop the dashed half (count=0) or dash \
         the wrong half (count=4)"
    );

    // Delta-baseline: rendering the same notes with no hairpins gives the
    // staff/stem/barline line count. Subtracting it isolates the hairpin
    // contribution.
    //
    //   cross-system crescendo: trailing half (2 lines, solid)
    //                         + incoming half (2 lines, dashed)
    //                         = 4 lines
    //   within-system decrescendo: 1 wedge (2 lines, solid)
    //                         = 2 lines
    //   total hairpin lines  = 6
    //
    // Pinning the exact delta catches regressions that would, e.g., drop
    // the incoming-half emission, double-emit the trailing half, or fall
    // back to a single-wedge same-system layout that ignores the system
    // break.
    let baseline = build_cross_system_hairpins_baseline();
    let with_lines = svg.matches("<line ").count();
    let no_lines = baseline.matches("<line ").count();
    assert_eq!(
        with_lines,
        no_lines + 6,
        "cross-system cresc (4 lines: solid trailing + dashed incoming) \
         plus within-system decresc (2 lines) should add exactly 6 lines; \
         got {with_lines} vs baseline {no_lines}"
    );

    // The baseline must itself contain no hairpin artifacts — protects the
    // delta assertion above against a baseline-side leak.
    assert!(
        !baseline.contains("stroke-dasharray"),
        "the no-hairpin baseline must not contain stroke-dasharray; \
         a leak here would invalidate the delta assertion"
    );

    // Counter-example: the same crescendo, but laid out so it fits within a
    // single system (no system break under the wedge). The within-system code
    // path uses solid lines exclusively — dashed continuation is a
    // cross-system artifact only. This pins the implication direction:
    // "dasharray appears" ⇒ "the wedge crossed a system break".
    let within_system_only = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .hairpin_start(HairpinType::Crescendo)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();
    assert!(
        !within_system_only.contains("stroke-dasharray"),
        "a within-system hairpin must render entirely solid; dasharray here \
         would mean the dashed-continuation code path leaked outside the \
         cross-system case"
    );

    assert_golden("cross_system_hairpins", &svg);
}

#[test]
fn golden_cresc_text() {
    let svg = build_cresc_text();
    let baseline = build_cresc_text_baseline();

    // Delta-baseline assertions: stripping the three `cresc_text_*` calls
    // gives the underlying staff/clef/notes/barlines rendering. The
    // marking contribution must be *exactly*:
    //
    //   3 dashed continuation lines (one per kind — `cresc.`, `decresc.`,
    //                                 `dim.`)
    //   3 italic `<text>` labels    (one per kind, label content distinct)
    //
    // Anything else means a regression — e.g. dropping the label
    // emission (no extra text), duplicating the dashed line, or letting
    // the wedgeless renderer leak `<line>` elements that should not exist
    // on a no-marking baseline.

    let svg_lines = svg.matches("<line ").count();
    let base_lines = baseline.matches("<line ").count();
    assert_eq!(
        svg_lines,
        base_lines + 3,
        "cresc-text adds exactly 3 dashed lines over baseline (got delta \
         {}, expected 3)",
        svg_lines as i64 - base_lines as i64
    );

    let svg_dasharray = svg.matches("stroke-dasharray").count();
    let base_dasharray = baseline.matches("stroke-dasharray").count();
    assert_eq!(
        base_dasharray, 0,
        "no-marking baseline must not contain stroke-dasharray; a leak \
         here would invalidate the delta assertion"
    );
    assert_eq!(
        svg_dasharray, 3,
        "cresc-text must emit exactly 3 stroke-dasharray attributes \
         (one per dashed continuation line), got {svg_dasharray}"
    );

    // Each kind's italic label appears exactly once in the SVG. The
    // `decresc.` label is a superstring of `cresc.`, so we use the
    // closing-tag-bound substring `>cresc.</text>` (matches the bare
    // `cresc.` label only, not the `decresc.` label) to disambiguate.
    let cresc_labels = svg.matches(">cresc.</text>").count();
    let decresc_labels = svg.matches(">decresc.</text>").count();
    let dim_labels = svg.matches(">dim.</text>").count();
    assert_eq!(
        cresc_labels, 1,
        "expected exactly 1 '>cresc.</text>' label, got {cresc_labels}"
    );
    assert_eq!(
        decresc_labels, 1,
        "expected exactly 1 '>decresc.</text>' label, got {decresc_labels}"
    );
    assert_eq!(
        dim_labels, 1,
        "expected exactly 1 '>dim.</text>' label, got {dim_labels}"
    );

    // Italic styling: each cresc-text label carries `font-style="italic"`.
    // The baseline has no italic text (clef/notes/barlines are all glyph
    // paths or plain text); count delta must be at least 3 (one per label).
    let svg_italic = svg.matches("font-style=\"italic\"").count();
    let base_italic = baseline.matches("font-style=\"italic\"").count();
    assert!(
        svg_italic >= base_italic + 3,
        "italic-styling delta must be at least 3 (one per kind label); \
         got {} vs baseline {}",
        svg_italic,
        base_italic
    );

    assert_golden("cresc_text", &svg);
}

#[test]
fn golden_cross_system_cresc_text() {
    let svg = build_cross_system_cresc_text();
    let baseline = build_cross_system_cresc_text_baseline();

    // The cross-system `cresc.` marking is rendered by
    // `draw_cross_system_cresc_texts` in `page_renderer/mod.rs`. The
    // engraved convention (mirror of cross-system ottava and trill
    // extensions) is:
    //
    //   source half (system 1): italic label + dashed continuation line
    //   target half (system 2): dashed continuation line ONLY (label
    //                            suppressed via
    //                            `layout_cresc_text_continuation`)
    //
    // So a cross-system pair contributes exactly:
    //   - 1 `>cresc.</text>` label   (lives on the source side only)
    //   - 2 dashed lines             (1 trailing + 1 incoming)
    //   - 2 `stroke-dasharray`       (one per dashed line)
    //
    // If the incoming half accidentally calls `layout_cresc_text` instead
    // of the continuation variant, we'd get 2 labels — caught here.
    // If the cross-system path is silently skipped, we'd get 0 labels —
    // also caught here.

    let svg_lines = svg.matches("<line ").count();
    let base_lines = baseline.matches("<line ").count();
    assert_eq!(
        svg_lines,
        base_lines + 2,
        "cross-system cresc-text adds exactly 2 dashed lines (1 trailing \
         + 1 incoming) over baseline; got delta {}",
        svg_lines as i64 - base_lines as i64
    );

    let svg_dasharray = svg.matches("stroke-dasharray").count();
    let base_dasharray = baseline.matches("stroke-dasharray").count();
    assert_eq!(
        base_dasharray, 0,
        "no-marking baseline must not contain stroke-dasharray"
    );
    assert_eq!(
        svg_dasharray, 2,
        "cross-system cresc-text must emit exactly 2 stroke-dasharray \
         attributes (one per dashed half), got {svg_dasharray}"
    );

    // Exactly one `cresc.` label across both systems — the continuation
    // half on system 2 must NOT repeat the label.
    let cresc_labels = svg.matches(">cresc.</text>").count();
    assert_eq!(
        cresc_labels, 1,
        "cross-system cresc-text must emit exactly 1 '>cresc.</text>' \
         label (label lives on the source system only, NOT duplicated \
         on the target/continuation half); got {cresc_labels}"
    );
    // The other two label variants must not appear — Crescendo kind is
    // requested explicitly, so `decresc.` / `dim.` are negative controls.
    assert!(
        !svg.contains(">decresc.</text>"),
        "'>decresc.</text>' must not appear in a `Crescendo`-kind cross-\
         system cresc-text render"
    );
    assert!(
        !svg.contains(">dim.</text>"),
        "'>dim.</text>' must not appear in a `Crescendo`-kind cross-\
         system cresc-text render"
    );

    assert_golden("cross_system_cresc_text", &svg);
}

#[test]
fn golden_hairpin_niente_dashed() {
    let svg = build_hairpin_niente_dashed();
    let baseline = build_hairpin_niente_dashed_baseline();

    // The combo (dashed wedge + niente circle) contributes a fixed,
    // structurally-pinned set of elements over the no-marking baseline.
    // Each of the 2 within-system hairpins emits:
    //   - 2 dashed `<line>` elements (upper + lower wedge arm), each with
    //     `stroke-dasharray`
    //   - 1 `<circle>` element with `fill="none"` and NO `stroke-dasharray`
    //     (engraved convention: niente "o" stays solid even on a dashed
    //     wedge)
    // The baseline has no hairpins → no wedge lines, no dasharray, no
    // circles, so the deltas pin the entire combo path.

    // ---- Wedge line delta: 2 hairpins × 2 lines = 4 extra lines. ----
    let svg_lines = svg.matches("<line ").count();
    let base_lines = baseline.matches("<line ").count();
    assert_eq!(
        svg_lines,
        base_lines + 4,
        "two within-system dashed hairpins add exactly 4 wedge lines (2 per \
         hairpin: upper + lower arm); got delta {} expected 4",
        svg_lines as i64 - base_lines as i64
    );

    // ---- Dashed-wedge dasharray count: 4 attributes total. ----
    let svg_dasharray = svg.matches("stroke-dasharray").count();
    let base_dasharray = baseline.matches("stroke-dasharray").count();
    assert_eq!(
        base_dasharray, 0,
        "no-marking baseline must not contain stroke-dasharray; a leak here \
         would invalidate the delta assertion"
    );
    assert_eq!(
        svg_dasharray, 4,
        "two dashed wedges × 2 lines each = exactly 4 stroke-dasharray \
         attributes; got {svg_dasharray}"
    );

    // ---- Niente circle delta: 2 hairpins → exactly 2 circles. ----
    // Engraver emits no `<circle>` elements for any other element, so a
    // count of 2 is a tight pin on the combo path.
    let svg_circles = svg.matches("<circle ").count();
    let base_circles = baseline.matches("<circle ").count();
    assert_eq!(
        base_circles, 0,
        "no-marking baseline must not contain <circle> elements; a leak \
         here would invalidate the delta assertion"
    );
    assert_eq!(
        svg_circles, 2,
        "two hairpins each with hairpin_niente* set must emit exactly 2 \
         <circle> elements (one per niente); got {svg_circles}"
    );

    // ---- Per-circle engraving invariants: open ring + solid stroke. ----
    // Each niente circle must carry `fill="none"` (open "o", not filled
    // disk) AND must NOT carry `stroke-dasharray` (the dashed-wedge style
    // does not propagate to the circle — engraved convention).
    let mut circle_count = 0;
    let mut fill_none_count = 0;
    let mut dashed_circle_count = 0;
    for line in svg.lines() {
        if line.contains("<circle ") {
            circle_count += 1;
            if line.contains(r#"fill="none""#) {
                fill_none_count += 1;
            }
            if line.contains("stroke-dasharray") {
                dashed_circle_count += 1;
            }
        }
    }
    assert_eq!(
        circle_count, 2,
        "line-scan circle count must match substring count; got {circle_count}"
    );
    assert_eq!(
        fill_none_count, 2,
        "both niente circles must carry fill=\"none\" (engraved open ring); \
         got {fill_none_count}"
    );
    assert_eq!(
        dashed_circle_count, 0,
        "niente circles must NOT carry stroke-dasharray even when the wedge \
         is dashed (engraved convention: circle stays solid); got {dashed_circle_count}"
    );

    // ---- Counter-example: removing the niente flag must drop both circles. ----
    // This pins the implication: `<circle>` count = 2 ⇒ both niente flags
    // are reaching the renderer. If the niente flag silently dropped, the
    // dashed wedges would still render (4 lines, 4 dasharray) but the
    // circle count would fall to 0.
    let no_niente = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .hairpin_start(HairpinType::Crescendo)
        .hairpin_dashed()
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .hairpin_end()
        .barline()
        .note(p("G", 4), Duration::QTR)
        .hairpin_start(HairpinType::Decrescendo)
        .hairpin_dashed()
        .note(p("F", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .hairpin_end()
        .end_barline()
        .render_svg();
    assert_eq!(
        no_niente.matches("<circle ").count(),
        0,
        "stripping the hairpin_niente* calls must drop both circles; any \
         residual <circle> means the niente render path is firing on the \
         wrong flag"
    );
    // Same structure must still emit 4 dashed-wedge lines and 4 dasharrays
    // — the dashed-wedge path is independent of the niente flag.
    assert_eq!(
        no_niente.matches("stroke-dasharray").count(),
        4,
        "no-niente variant must still carry the 4 dashed-wedge dasharray \
         attrs (dashed path is independent of niente)"
    );

    assert_golden("hairpin_niente_dashed", &svg);
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
        .fret(6, 0)
        .next()
        .fret(5, 2)
        .next()
        .fret(4, 2)
        .next()
        .fret(3, 0)
        .barline()
        // Measure 2: scale on string 1
        .fret(1, 0)
        .next()
        .fret(1, 3)
        .next()
        .fret(1, 5)
        .next()
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
        .fret(1, 12)
        .next()
        .fret(2, 12)
        .next()
        .fret(1, 15)
        .next()
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
        .fret(1, 5)
        .slide()
        .next()
        .fret(1, 7)
        .next()
        .fret(1, 3)
        .next()
        .fret(1, 0)
        .barline()
        // Measure 2: descending slide on string 2
        .fret(2, 12)
        .slide()
        .next()
        .fret(2, 9)
        .next()
        .fret(2, 7)
        .next()
        .fret(1, 5)
        .barline()
        // Measure 3: chord slide (power chord shift)
        .fret(6, 3)
        .fret(5, 5)
        .fret(4, 5)
        .slide()
        .next()
        .fret(6, 5)
        .fret(5, 7)
        .fret(4, 7)
        .next()
        .rest()
        .next()
        .fret(6, 0)
        .barline()
        // Measure 4: consecutive slides (chain)
        .fret(1, 5)
        .slide()
        .next()
        .fret(1, 7)
        .slide()
        .next()
        .fret(1, 9)
        .slide()
        .next()
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
        .fret(1, 5)
        .hammer()
        .next()
        .fret(1, 7)
        .next()
        .rest()
        .next()
        .rest()
        .barline()
        // Measure 2: pull-off 7→5
        .fret(1, 7)
        .pull()
        .next()
        .fret(1, 5)
        .next()
        .rest()
        .next()
        .rest()
        .barline()
        // Measure 3: chain 5→7→5
        .fret(2, 5)
        .hammer()
        .next()
        .fret(2, 7)
        .pull()
        .next()
        .fret(2, 5)
        .next()
        .rest()
        .barline()
        // Measure 4: multi-string chord hammer
        .fret(5, 5)
        .fret(4, 7)
        .fret(3, 7)
        .hammer()
        .next()
        .fret(5, 7)
        .fret(4, 9)
        .fret(3, 9)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_tab_hammer_pull() {
    assert_golden("tab_hammer_pull", &build_tab_hammer_pull());
}

fn build_semantic_bends() -> String {
    let mut score = GuitarScore::standard();
    score.set_time_signature(1, 4);
    let source = score.note(p("G", 4), Duration::QTR, 1, 3).unwrap();
    score.barline().unwrap();
    let arrival = score.note(p("G", 4), Duration::QTR, 1, 3).unwrap();
    score.barline().unwrap();
    let reattack = score.note(p("G", 4), Duration::QTR, 1, 3).unwrap();
    score.barline().unwrap();
    let release_end = score.note(p("G", 4), Duration::QTR, 1, 3).unwrap();
    score.end_barline().unwrap();
    score
        .bend(
            BendGesture::new(
                source,
                1,
                BendPitch::exact(p("A", 4)),
                GuitarMoment::onset(arrival),
            )
            .with_release(BendRelease::new(
                GuitarMoment::after(reattack, Duration::EIGHTH),
                GuitarMoment::onset(release_end),
                BendPitch::exact(p("G", 4)),
            ))
            .reattacked_at(reattack),
        )
        .unwrap();
    MultiStaffScore::guitar(score)
        .measures_per_system(2)
        .system_width_fu(10_000.0)
        .render_svg()
}

#[test]
fn golden_semantic_bends() {
    let svg = build_semantic_bends();
    assert!(svg.contains("data-bend-view=\"standard\""));
    assert!(svg.contains("data-bend-view=\"tab\""));
    assert!(svg.contains("data-bend-phase=\"hold\""));
    assert!(svg.contains("data-bend-phase=\"release\""));
    assert!(svg.contains("data-bend-phase=\"reattack\""));
    assert_golden("semantic_bends", &svg);
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
    let mut score = GuitarScore::standard();
    score
        .set_key_signature(KeySignature::Open)
        .set_time_signature(4, 4);
    score.note(p("E", 4), Duration::QTR, 1, 0).unwrap();
    score.note(p("G", 4), Duration::QTR, 1, 3).unwrap();
    score.note(p("B", 4), Duration::QTR, 1, 7).unwrap();
    score.note(p("E", 5), Duration::QTR, 1, 12).unwrap();
    score.barline().unwrap();
    score.note(p("E", 5), Duration::QTR, 1, 12).unwrap();
    score.note(p("B", 4), Duration::QTR, 1, 7).unwrap();
    score.note(p("G", 4), Duration::HALF, 1, 3).unwrap();
    score.end_barline().unwrap();

    MultiStaffScore::guitar(score)
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
    assert!(svg.contains(">12</text>"), "should show fret 12");

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
    assert!(
        svg.contains("stroke-dasharray"),
        "should contain dashed lines"
    );
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
    assert!(
        svg.contains("stroke-dasharray"),
        "should contain dashed lines"
    );
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

/// Score that exercises **all four** pedal variants (Down, Up, Half, Sost)
/// in a single page render. The previous `golden_pedal_marks` fixture
/// only covers the original Down/Up pair; this one freezes the visual
/// baseline of the newer `PedalMark::Half` and `PedalMark::Sost`
/// variants through the full page-renderer pipeline.
///
/// Layout: 8 quarter notes across 2 measures.
///   M1: C4-pedal_down, E4, G4, C5-pedal_up
///   M2: G4-pedal_half, E4, C4, G3-pedal_sost
fn build_all_pedal_marks() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        // Measure 1: Down + Up (the original pair)
        .note(p("C", 4), Duration::QTR)
        .pedal_down()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .pedal_up()
        .barline()
        // Measure 2: Half + Sost (the new variants)
        .note(p("G", 4), Duration::QTR)
        .pedal_half()
        .note(p("E", 4), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .note(p("G", 3), Duration::QTR)
        .pedal_sost()
        .end_barline()
        .render_svg()
}

#[test]
fn golden_all_pedal_marks() {
    let svg = build_all_pedal_marks();

    // ---- Path delta: 4 pedal marks add exactly 4 paths. ----
    // Each PedalMark variant renders as one <path> (glyph outline) with no
    // additional decoration (no extension lines, no bracket). This pins
    // the contract that every variant is structurally a single glyph.
    let base_svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .barline()
        .note(p("G", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .note(p("G", 3), Duration::QTR)
        .end_barline()
        .render_svg();
    let with_pedal = svg.matches("<path").count();
    let no_pedal = base_svg.matches("<path").count();
    assert_eq!(
        with_pedal,
        no_pedal + 4,
        "all-four-variants score must add exactly 4 paths (one per pedal \
         mark); got delta {}",
        with_pedal as i64 - no_pedal as i64
    );

    // ---- Variant distinctness: must differ from same-variant scores. ----
    // If a regression collapsed Half → Down (e.g. a copy-paste of the Down
    // branch in PedalMark::glyph()), the all-four-variants SVG would
    // become byte-identical to an all-Down score. Same logic for Sost.
    // These assertions pin variant-specific glyph routing through the
    // entire builder → layout → renderer pipeline.
    let all_down = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_down()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .pedal_down()
        .barline()
        .note(p("G", 4), Duration::QTR)
        .pedal_down()
        .note(p("E", 4), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .note(p("G", 3), Duration::QTR)
        .pedal_down()
        .end_barline()
        .render_svg();
    assert_ne!(
        svg, all_down,
        "all-four-variants SVG must differ from all-Down SVG; if equal, the \
         Up/Half/Sost branches are silently mapping to Down (regression in \
         PedalMark::glyph or builder)"
    );
    assert_eq!(
        all_down.matches("<path").count(),
        no_pedal + 4,
        "all-Down score must also add exactly 4 paths (sanity check on \
         the delta arithmetic above)"
    );

    let all_half = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .note(p("C", 4), Duration::QTR)
        .pedal_half()
        .note(p("E", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .pedal_half()
        .barline()
        .note(p("G", 4), Duration::QTR)
        .pedal_half()
        .note(p("E", 4), Duration::QTR)
        .note(p("C", 4), Duration::QTR)
        .note(p("G", 3), Duration::QTR)
        .pedal_half()
        .end_barline()
        .render_svg();
    assert_ne!(
        svg, all_half,
        "all-four-variants SVG must differ from all-Half SVG; if equal, the \
         Down/Up/Sost branches are silently mapping to Half"
    );

    // ---- Distinctness from the existing Down/Up-only fixture. ----
    // build_pedal_marks() uses only Down + Up across 1 measure; this
    // fixture spans 2 measures with all four variants. They must differ.
    assert_ne!(
        svg,
        build_pedal_marks(),
        "all-four-variants golden must differ from the original Down/Up-only \
         pedal_marks golden (different fixture, different mark set)"
    );

    assert_golden("all_pedal_marks", &svg);
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
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::HALF)
        .arpeggio(ArpeggioDirection::Up)
        .note(p("C", 5), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .barline()
        // Measure 2: downward arpeggio on D minor triad + single-note arpeggio
        .chord(vec![p("D", 4), p("F", 4), p("A", 4)], Duration::HALF)
        .arpeggio(ArpeggioDirection::Down)
        .note(p("E", 5), Duration::QTR)
        .arpeggio(ArpeggioDirection::Up)
        .rest(Duration::QTR)
        .barline()
        // Measure 3: wide voicing + downward arpeggio on two-note chord
        .chord(vec![p("C", 4), p("G", 4), p("E", 5)], Duration::HALF)
        .arpeggio(ArpeggioDirection::Up)
        .chord(vec![p("B", 4), p("D", 5)], Duration::HALF)
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
    assert!(
        scale_count >= 5,
        "should have ≥5 arpeggio scale transforms: {scale_count}"
    );

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
    assert_ne!(
        svg, hbar_version,
        "church-rest and H-bar must render differently"
    );

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
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Down, 1.2)
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .barline()
        // Start, Up direction, default-length 0.75 ss.
        .note(p("E", 5), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Start, HookDirection::Up, 0.75)
        .barline()
        .note(p("D", 5), Duration::WHOLE)
        .barline()
        // End, Up direction, short 0.5 ss hook.
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::End, HookDirection::Up, 0.5)
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .barline()
        // Chord trill: Both, Up direction, 1.0 ss hook.
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::WHOLE)
        .trill_with_extension_bracketed_custom(TrillBracketSide::Both, HookDirection::Up, 1.0)
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
        .trill_with_extension_bracketed_custom(TrillBracketSide::Start, HookDirection::Up, 0.9)
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
    use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
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
                TrillExtensionSpeedOptions::new(*speed).with_ornament(Ornament::TrillWithMordent),
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
    use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
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
    use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
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

/// Four whole-note trills across two systems with explicit-length
/// extensions of 2.0, 4.0, and 1.5 staff spaces (plus an unannotated
/// baseline). Exercises `ScoreBuilder::trill_with_extension_length_ss`.
fn build_trill_short_extension() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(2.0)
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_length_ss(4.0)
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension_length_ss(1.5)
        .end_barline()
        .render_svg()
}

/// The same musical content but every trill uses the default extension
/// (next-note / system-edge termination). Used as the regression baseline
/// for proving the explicit-length call shortened the wiggles.
fn build_trill_short_extension_defaults() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .render_svg()
}

/// Same musical content with only the "tr" prefix glyph (no wiggle).
/// Used to verify the explicit-length wiggles still produce *some*
/// segments rather than silently dropping every wiggle.
fn build_trill_short_extension_plain() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_short_extension() {
    let svg = build_trill_short_extension();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    let defaults = build_trill_short_extension_defaults();
    let plain = build_trill_short_extension_plain();

    let svg_paths = svg.matches("<path").count();
    let defaults_paths = defaults.matches("<path").count();
    let plain_paths = plain.matches("<path").count();

    // (1) Explicit-length wiggles still produce wiggle segments (path
    //     count strictly exceeds the no-wiggle baseline). Regression
    //     canary against a future change that drops every wiggle when
    //     the explicit length is set.
    assert!(
        svg_paths > plain_paths,
        "explicit-length wiggles must produce some wiggle paths: \
         explicit={svg_paths}, plain_trill={plain_paths}"
    );

    // (2) Explicit lengths shorter than natural produce strictly fewer
    //     paths than the default (next-note / system-edge) extensions.
    //     Three of the four measures specify short explicit lengths, so
    //     three measures each lose tiles vs. the all-defaults baseline.
    assert!(
        svg_paths < defaults_paths,
        "explicit lengths must shorten wiggles vs defaults: \
         explicit={svg_paths}, defaults={defaults_paths}"
    );

    // (3) Byte-inequality vs the all-defaults baseline. The explicit
    //     lengths must change SVG content; if a future refactor silently
    //     drops the length annotation, both versions would render the
    //     same bytes and this assertion would fire.
    assert_ne!(
        svg, defaults,
        "explicit-length score must not be byte-identical to all-defaults score"
    );

    // (4) The number of "tr" glyph paths must be the same in both
    //     versions — explicit length affects the wiggle, not the prefix.
    //     This grounds the path-count comparison: the path-count
    //     reduction in (2) comes solely from missing wiggle tiles, not
    //     from a missing "tr".
    let svg_tr_count = svg.matches("translate(").count();
    let defaults_tr_count = defaults.matches("translate(").count();
    assert!(
        svg_tr_count <= defaults_tr_count,
        "explicit-length must not ADD any glyphs vs defaults — only remove wiggle tiles: \
         explicit_translates={svg_tr_count}, defaults_translates={defaults_tr_count}"
    );

    assert_golden("trill_short_extension", &svg);
}

/// Four whole-note trills across two systems exercising the
/// `extension_length_ss` field on the two single-purpose option bundles
/// (`TrillBracketOptions`, `TrillExtensionSpeedOptions`). M1/M2 use
/// bracket+length; M3/M4 use speed+length. Until this field was added to
/// the single-purpose bundles, a caller wanting either combination had to
/// widen to `TrillExtensionFullOptions` first.
fn build_trill_options_with_length() -> String {
    use music_engraver::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};
    use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(2.0),
        )
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::End).with_extension_length_ss(4.0),
        )
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow).with_extension_length_ss(2.0),
        )
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster).with_extension_length_ss(3.0),
        )
        .end_barline()
        .render_svg()
}

/// Same musical content + same bracket sides + same speeds, but with
/// `extension_length_ss` LEFT UNSET on every bundle. Used as the
/// length-isolation baseline: any path-count difference between this and
/// `build_trill_options_with_length` comes from the explicit-length field.
fn build_trill_options_with_length_no_length_variant() -> String {
    use music_engraver::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};
    use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
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
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::End,
        ))
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Slow,
        ))
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Faster,
        ))
        .end_barline()
        .render_svg()
}

/// Same musical content as `build_trill_options_with_length`, but each
/// bundle is widened via `From` to `TrillExtensionFullOptions` (which
/// propagates `extension_length_ss` into the unified bundle's `length_ss`).
/// Used to verify the documented byte-equivalence between the
/// single-purpose-bundle path and the widened-full-options path.
fn build_trill_options_with_length_widened_variant() -> String {
    use music_engraver::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};
    use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
    use music_engraver::layout::trill_options::TrillExtensionFullOptions;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(2.0),
        ))
        .barline()
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillBracketOptions::new(TrillBracketSide::End).with_extension_length_ss(4.0),
        ))
        .barline()
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow).with_extension_length_ss(2.0),
        ))
        .barline()
        .note(p("C", 5), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster).with_extension_length_ss(3.0),
        ))
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_options_with_length() {
    let svg = build_trill_options_with_length();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    let no_length = build_trill_options_with_length_no_length_variant();
    let widened = build_trill_options_with_length_widened_variant();

    let svg_paths = svg.matches("<path").count();
    let no_length_paths = no_length.matches("<path").count();
    let svg_lines = svg.matches("<line ").count();
    let no_length_lines = no_length.matches("<line ").count();

    // Structural guard 1: explicit lengths must SHORTEN the wiggle vs the
    // no-length baseline. All four chosen lengths sit well below the natural
    // span of the corresponding whole notes. If the new field is silently
    // dropped, both versions render the same wiggle tile counts and this
    // fires.
    assert!(
        svg_paths < no_length_paths,
        "explicit-length wiggles must produce fewer paths than no-length variant: \
         with_length={svg_paths}, no_length={no_length_paths}"
    );

    // Structural guard 2: byte-inequality vs the no-length variant. The
    // explicit length must change SVG content; an SVG-level no-op refactor
    // would fire here.
    assert_ne!(
        svg, no_length,
        "explicit-length score must not be byte-identical to no-length variant"
    );

    // Structural guard 3: bracket hook count is INVARIANT under the
    // explicit length. M1 Both=2 + M2 End=1 + M3/M4 no bracket = 3 hook
    // lines either way. The explicit length affects the wiggle terminus,
    // NOT the bracket presence — catches a regression where setting the
    // length accidentally suppresses the bracket itself.
    assert_eq!(
        svg_lines, no_length_lines,
        "explicit length must NOT change bracket hook count: \
         with_length={svg_lines}, no_length={no_length_lines} \
         (the new field affects the wiggle terminus, not the bracket presence)"
    );

    // Structural guard 4: byte-equivalence with the widened-full-options
    // path. The documented `From<TrillBracketOptions>` and
    // `From<TrillExtensionSpeedOptions>` impls on
    // `TrillExtensionFullOptions` propagate `extension_length_ss` into
    // the unified bundle's `length_ss`. So
    //   `trill_with_extension_bracketed_with_options(opts)`  and
    //   `trill_with_extension_full_options(opts.into())`
    // must produce byte-identical SVG. Catches a regression where either
    // side of the From conversion drops the new field.
    assert_eq!(
        svg, widened,
        "single-purpose bundles with extension_length_ss must produce byte-identical SVG \
         to the widened TrillExtensionFullOptions path"
    );

    // Structural guard 5: byte-inequality vs the existing
    // `trill_short_extension` baseline. That golden uses
    // `trill_with_extension_length_ss(L)` (the standalone builder) on plain
    // trills with no bracket and no speed override, while this golden
    // overlays bracket and speed on top of explicit lengths. If a future
    // refactor accidentally collapsed the option-bundle paths to the
    // standalone path, the two outputs would converge. The existing
    // single-purpose bundles add bracket hooks (3 extra `<line>` elements)
    // and a non-Standard speed glyph mix on M3/M4, both of which must
    // visibly differ.
    let standalone_short = build_trill_short_extension();
    assert_ne!(
        svg, standalone_short,
        "trill_options_with_length must differ from trill_short_extension: \
         the option-bundle paths add bracket hooks and a speed override mix \
         that the standalone-length path cannot express"
    );

    assert_golden("trill_options_with_length", &svg);
}

/// Four whole/half-note trills across two systems exercising the full
/// four-way combination `bracket + speed + ornament + length` through
/// `TrillExtensionFullOptions` — the unique capability of the unified bundle
/// over the two single-purpose bundles (neither of which can express all
/// four knobs in a single call).
///
/// Mirrors `build_trill_full_options` exactly, but adds a
/// `.with_length_ss(...)` on every measure, so every SVG byte-difference is
/// attributable to the new length field.
fn build_trill_full_options_with_length() -> String {
    use music::notation::rhythm::duration::DurationKind;
    use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
    use music_engraver::layout::trill_extension::TrillWiggleSpeed;
    use music_engraver::layout::trill_options::TrillExtensionFullOptions;
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Both bracket + Slow speed + TrillWithMordent + 2.0ss length.
        .note(p("G", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(2.0),
        )
        .barline()
        // M2: End bracket + Up direction + Faster speed + plain Trill + 4.0ss length.
        // M2 is the last note of system 1; explicit length disables
        // cross-system propagation so the End hook anchors on system 1.
        .note(p("A", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up)
                .with_speed(TrillWiggleSpeed::Faster)
                .with_length_ss(4.0),
        )
        .barline()
        // M3: Start bracket + 1.0ss hook + Slowest speed + TrillWithMordent +
        // 3.0ss length. Slowest is the widest wiggle glyph; 3.0ss is the
        // smallest length that still tiles at least one segment.
        .note(p("B", 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Start)
                .with_bracket_length_ss(1.0)
                .with_speed(TrillWiggleSpeed::Slowest)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(3.0),
        )
        .barline()
        // M4: chord + Both bracket + Standard speed + TrillWithMordent + 2.5ss length.
        // Exercises the chord arm's length-field wire-up.
        .chord(
            vec![p("C", 4), p("E", 4), p("G", 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(2.5),
        )
        .note(p("D", 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

#[test]
fn golden_trill_full_options_with_length() {
    let svg = build_trill_full_options_with_length();

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    let full_no_length = build_trill_full_options();
    let svg_paths = svg.matches("<path").count();
    let svg_lines = svg.matches("<line ").count();
    let full_paths = full_no_length.matches("<path").count();
    let full_lines = full_no_length.matches("<line ").count();

    // Structural guard 1: byte-inequality vs the existing
    // `trill_full_options` baseline (same bracket+speed+ornament, no
    // length). Locks in that adding `.with_length_ss(...)` to every
    // measure produces a visibly different SVG. A regression that
    // silently dropped the length field at the builder layer would fire.
    assert_ne!(
        svg, full_no_length,
        "trill_full_options_with_length must differ from trill_full_options: \
         the only difference between the two builders is the .with_length_ss(...) call \
         on every measure"
    );

    // Structural guard 2: explicit lengths must SHORTEN the wiggle vs the
    // no-length baseline. All four chosen lengths sit well below the
    // natural spans. If `length_ss` stops propagating through the unified
    // bundle, both versions render the same wiggle tile counts.
    assert!(
        svg_paths < full_paths,
        "explicit-length wiggles must produce fewer paths than no-length variant: \
         with_length={svg_paths}, no_length={full_paths}"
    );

    // Structural guard 3: bracket-hook count INVARIANCE. Even though M2's
    // cross-system End hook moves between systems (system 2 in the
    // no-length variant via `draw_cross_system_trill_extensions`,
    // system 1 in the with-length variant via the within-system pass),
    // the **net total** is the same: 1 End hook for M2 either way.
    // Catches a regression where setting the length accidentally
    // suppresses a hook (e.g., the renderer drops the End hook on the
    // source system without also disabling cross-system propagation).
    //
    // The chosen lengths (2.0, 4.0, 3.0, 2.5 ss) are deliberately above
    // the wiggle-too-short fail-safe threshold for every speed used —
    // M3 with Slowest speed needs ≥3.0ss, the others need ≥1.5ss.
    assert_eq!(
        svg_lines, full_lines,
        "net bracket hook count must be invariant under explicit length: \
         with_length={svg_lines}, no_length={full_lines} \
         (cross-system continuation hooks move position but the total is preserved)"
    );

    // Structural guard 4: byte-inequality vs the standalone
    // `trill_short_extension` baseline. That baseline uses the
    // `trill_with_extension_length_ss(L)` shortcut on plain trills with
    // no bracket and no speed override; this baseline layers bracket +
    // speed + compound ornament on top of explicit lengths. If a future
    // refactor accidentally collapsed the unified-options path to the
    // standalone path (dropping the bracket+speed knobs), the two
    // baselines would converge.
    let standalone_short = build_trill_short_extension();
    assert_ne!(
        svg, standalone_short,
        "trill_full_options_with_length must differ from trill_short_extension: \
         the unified-options path adds bracket + speed + compound-ornament knobs \
         that the standalone-length shortcut cannot express"
    );

    // Structural guard 5: byte-inequality vs the existing
    // `trill_options_with_length` baseline. That baseline uses the two
    // single-purpose bundles with `extension_length_ss` (one per
    // measure). This baseline uses the unified bundle with FOUR knobs
    // set on every measure (`bracket + speed + ornament + length`),
    // which neither single-purpose bundle can express on its own. The
    // two outputs are therefore expected to differ — the cross-baseline
    // canary catches a future refactor that accidentally collapsed the
    // unified-bundle path's expressiveness down to the single-purpose
    // bundles'.
    let options_with_length = build_trill_options_with_length();
    assert_ne!(
        svg, options_with_length,
        "trill_full_options_with_length must differ from trill_options_with_length: \
         the unified bundle here sets bracket + speed + ornament + length on every \
         measure, exercising combinations only the unified bundle can express"
    );

    assert_golden("trill_full_options_with_length", &svg);
}

/// Advanced guitar vocabulary stays visually stable as one integrated score.
#[test]
fn golden_advanced_guitar_vocabulary() {
    let svg = advanced_guitar_fixture::render()
        .expect("the integrated advanced-guitar fixture should be valid");

    assert_eq!(
        svg.matches("Capo II · Tuning: DADGAD · 1=D4 2=A3 3=G3 4=D3 5=A2 6=D2")
            .count(),
        1
    );
    let setup_header = svg
        .find("data-guitar-setup-header=\"true\"")
        .expect("setup metadata has a reserved header group");
    let first_chord_symbol = svg
        .find(">E</text>")
        .expect("first-system chord symbol is rendered");
    assert!(setup_header < first_chord_symbol);
    assert_eq!(svg.matches(">Legend:</text>").count(), 1);
    assert!(svg.contains("data-guitar-barre-fragment=\"start\""));
    assert!(svg.contains("data-guitar-barre-fragment=\"end\""));
    assert!(svg.contains("data-guitar-barre-fragment=\"complete\""));
    assert!(!svg.contains(
        "data-guitar-barre-fragment=\"end\" data-guitar-barre-lane=\"0\" data-guitar-barre-length=\"0.000\""
    ));
    assert!(svg.contains(">harm.</text>"));
    assert!(svg.contains(">A.H.</text>"));
    assert!(svg.contains(">P.H.</text>"));
    assert!(svg.contains(">T.H.</text>"));
    assert!(svg.contains(">&lt;12&gt;</text>"));
    assert!(svg.contains(">&lt;10&gt;</text>"));
    assert!(svg.contains(">(5)</text>"));
    assert!(svg.contains(">x</text>"));
    assert!(svg.contains(">slap</text>"));
    assert!(svg.contains(">pop</text>"));
    assert!(svg.contains(">golpe  body percussion</text>"));
    assert!(svg.contains(">square  fretboard percussion</text>"));
    assert!(svg.contains(">x-perc.  string percussion</text>"));
    assert!(svg.contains(">down strum</text>"));
    assert!(svg.contains(">up strum</text>"));
    assert!(svg.contains(">↓  down-pick</text>"));
    assert!(!svg.contains(">D  down strum</text>"));
    assert_eq!(svg.matches("data-guitar-legend-scale=\"0.3\"").count(), 2);
    assert!(!svg.contains(">U  up strum</text>"));
    assert!(!svg.contains(">CII</text>"));
    assert!(svg.contains(">II</text>"));

    let font = music_engraver::font::bravura_font();
    let glyph_count = |glyph| {
        let outline = font
            .glyph_outline(glyph)
            .expect("advanced guitar fixture glyph must exist");
        svg.matches(&outline.path_data).count()
    };
    assert!(glyph_count(Glyph::GuitarStrumDown) > 0);
    assert!(glyph_count(Glyph::GuitarStrumUp) > 0);
    assert!(glyph_count(Glyph::GuitarBarreFull) > 0);
    assert!(glyph_count(Glyph::GuitarBarreHalf) > 0);
    assert!(glyph_count(Glyph::StringsHarmonic) > 0);
    assert!(glyph_count(NoteheadStyle::Diamond.glyph(2)) > 0);
    assert!(glyph_count(NoteheadStyle::CircleX.glyph(3)) > 0);
    assert!(glyph_count(NoteheadStyle::Square.glyph(3)) > 0);
    assert!(glyph_count(NoteheadStyle::Slash.glyph(3)) > 0);
    assert!(glyph_count(NoteheadStyle::X.glyph(3)) >= 8);

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
        "dynamics_lookalikes",
        "tuplet",
        "slurs",
        "articulations",
        "fermata_variants",
        "annotations",
        "bass_clef",
        "auto_breaks",
        "optimal_breaks",
        "grand_staff",
        "lyrics",
        "chord_symbols",
        "chord_symbols_with_accidentals",
        "ornaments",
        "ornaments_full",
        "hairpins",
        "cross_system_hairpins",
        "cross_system_ties",
        "expression_text",
        "multi_staff_cross_system",
        "tab_score",
        "tab_slides",
        "tab_hammer_pull",
        "semantic_bends",
        "volta_brackets",
        "cross_system_volta",
        "guitar_tab",
        "navigation_signs",
        "ottava_brackets",
        "cross_system_ottava",
        "pedal_marks",
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
        "trill_short_extension",
        "trill_options_with_length",
        "trill_full_options_with_length",
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
