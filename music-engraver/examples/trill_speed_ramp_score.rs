//! Example: exercise [`ScoreBuilder::trill_with_extension_speed_ramp`] across
//! the two ramp shapes (Constant / Linear) on a real staff.
//!
//! The convenience builder is the multi-speed counterpart of
//! [`ScoreBuilder::trill_with_extension_speed`] — same hard-coded
//! `Ornament::Trill`, same `trill_extension = true`, but instead of a single
//! `TrillWiggleSpeed` it accepts a [`TrillSpeedRamp`] plus a region count.
//! At draw time the system renderer evenly partitions the wavy line's span
//! into N regions and tiles each with the speed chosen by the ramp.
//!
//! Measure layout:
//! - M1: Constant(Standard) with 3 regions. Visually identical to the
//!   single-speed default `trill_with_extension()` — locks in the
//!   degenerate-equivalence at the score level.
//! - M2: Linear(Slow → Fast) with 3 regions. Accelerating wiggle: density
//!   increases left-to-right. Reads as one continuous wavy line of
//!   gradually-thickening density.
//! - M3: Linear(Fast → Slow) with 3 regions. Decelerating wiggle: density
//!   decreases left-to-right. The reverse of M2.
//! - M4: chord variant. Same accelerating ramp as M2, applied to a
//!   C-E-G half chord. Exercises the chord branch of the builder
//!   end-to-end (the multi-speed dispatch is identical for notes and
//!   chords — both populate `annotations.trill_speed_ramp`, which the
//!   system renderer's collector reads regardless of element kind).
//!
//! Produces `examples/output/trill_speed_ramp_score.svg`.

use music::note::note::Note;
use music::note::pitch::Pitch;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::trill_extension::{TrillSpeedRamp, TrillWiggleSpeed};
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = build_score();

    let path = out_dir.join("trill_speed_ramp_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line ").count();
    println!(
        "Wrote {} ({} bytes, {} paths, {} lines)",
        path.display(),
        svg.len(),
        path_count,
        line_count,
    );

    // ---- Structural assertions ----

    // 1. Valid SVG structure.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    // 2. The accel/decel direction must be observable: build the same
    //    score with the *direction* of M2 and M3's ramps swapped (Fast→Slow
    //    in M2 and Slow→Fast in M3) and assert byte-distinct SVG. If the
    //    ramp's direction silently failed to propagate (e.g. the builder
    //    stored ramps in a HashMap and emitted them in unsorted order),
    //    swapping the directions would produce byte-identical SVG.
    let swapped = build_swapped_direction_variant();
    assert_ne!(
        svg, swapped,
        "swapping accel⇄decel direction must produce distinct SVG"
    );

    // 3. The multi-speed wiggle path must produce strictly more <path>
    //    elements than a "no trill" baseline (every measure plain, no
    //    ornament). Tiles are <path> elements, so the multi-speed score
    //    must add at least a handful of them.
    let no_trills = build_no_trills_variant();
    let no_trills_paths = no_trills.matches("<path").count();
    assert!(
        path_count > no_trills_paths,
        "multi-speed score must add wiggle paths beyond no-trill baseline: \
         full={path_count}, no_trills={no_trills_paths}"
    );

    // 4. The Linear ramp in M2 (Slow→Fast) must produce different SVG
    //    than a Constant(Standard) ramp with the same region count over
    //    the same span. This isolates the "ramp variant actually drives
    //    glyph selection" canary at the score level.
    let all_constant = build_all_constant_standard_variant();
    assert_ne!(
        svg, all_constant,
        "Linear ramps must produce distinct SVG from all-Constant(Standard) ramps"
    );
}

/// Canonical multi-speed score: every measure exercises a different ramp
/// shape. M1 Constant(Standard), M2 Linear(Slow→Fast), M3 Linear(Fast→Slow),
/// M4 chord with Linear(Slow→Fast).
pub fn build_score() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Constant(Standard), 3 regions. Visually a uniform wiggle.
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3)
        .barline()
        // M2: Linear(Slow→Fast), 3 regions. Accelerating wiggle.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .barline()
        // M3: Linear(Fast→Slow), 3 regions. Decelerating wiggle.
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Fast, TrillWiggleSpeed::Slow),
            3,
        )
        .barline()
        // M4: chord variant. Same accelerating ramp as M2 on C-E-G half.
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same notes as the canonical score, but M2 and M3 have their ramp
/// directions swapped. Used to assert the direction is observable in
/// the rendered SVG.
fn build_swapped_direction_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3)
        .barline()
        // M2: swapped to Linear(Fast→Slow), decelerating.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Fast, TrillWiggleSpeed::Slow),
            3,
        )
        .barline()
        // M3: swapped to Linear(Slow→Fast), accelerating.
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_speed_ramp(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same notes, but no trill ornament on any measure. Baseline for "the
/// multi-speed score actually emits wiggle paths."
fn build_no_trills_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same notes, every measure using Constant(Standard) with 3 regions.
/// Used to isolate "ramp variant choice" from "region count" — same N=3,
/// same span, only difference is Linear vs Constant. The canonical score
/// must render differently because three of its measures use Linear.
fn build_all_constant_standard_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3)
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3)
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_speed_ramp(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3)
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}
