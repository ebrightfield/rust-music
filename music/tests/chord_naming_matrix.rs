//! Tier 1.6 — canonical positive matrix for chord naming.
//!
//! Each row is `(pcs, expected_rendered_quality)`. The row set covers the
//! cases enumerated in `docs/chord-naming-correction-plan.md` §1. Rows are
//! validated against `infer_chord_quality` and the `ChordQuality::to_string`
//! output under `ChordNameDisplayConfig::default()`. The **quality** string is
//! compared; the root note is not included (inference is rootless).
//!
//! After Phase 2 (Tier 2.1), the default rendering uses
//! `MajNotation::Delta` (`Δ7`, `mΔ7`, `+Δ7`) and ASCII accidentals (`b9`,
//! `#11`). The alternate `Maj7` / `min7` style is tested in the config-matrix
//! suite via `MajNotation::Maj`.

use std::collections::HashSet;

use music::note::pitch_class::Pc;
use music::note::pitch_class::Pc::*;
use music::note_collections::chord_name::naming_heuristics::infer_chord_quality;
use music::note_collections::chord_name::ChordNameDisplayConfig;

/// Infer a chord quality from a raw pc-set literal and render it with default
/// display configuration. Returns the rendered string, or `None` if inference
/// produced no quality at all.
fn infer_and_render(pcs: &[Pc]) -> Option<String> {
    let pcs: HashSet<Pc> = pcs.iter().copied().collect();
    let (_h, q) = infer_chord_quality(&pcs)?;
    let cfg = ChordNameDisplayConfig::default();
    q.map(|q| q.to_string(&cfg))
}

/// Assert that inferring `pcs` produces `expected` under default rendering.
#[track_caller]
fn assert_infers(pcs: &[Pc], expected: &str) {
    let got = infer_and_render(pcs);
    assert_eq!(
        got.as_deref(),
        Some(expected),
        "inference mismatch for {:?}: expected {:?}, got {:?}",
        pcs, expected, got,
    );
}

// ---------------------------------------------------------------------------
// Triads
// ---------------------------------------------------------------------------

#[test]
fn triad_major() {
    assert_infers(&[Pc0, Pc4, Pc7], "Maj");
}

#[test]
fn triad_minor() {
    assert_infers(&[Pc0, Pc3, Pc7], "m");
}

#[test]
fn triad_dim() {
    assert_infers(&[Pc0, Pc3, Pc6], "dim");
}

#[test]
fn triad_aug() {
    assert_infers(&[Pc0, Pc4, Pc8], "Aug");
}

#[test]
fn triad_sus2() {
    assert_infers(&[Pc0, Pc2, Pc7], "sus2");
}

#[test]
fn triad_sus4() {
    assert_infers(&[Pc0, Pc5, Pc7], "sus4");
}

// Tier 1.3 fast-path: bare P5 must render as an interval, not a major triad.
#[test]
fn power_chord_renders_as_p5() {
    assert_infers(&[Pc0, Pc7], "P5");
}

// ---------------------------------------------------------------------------
// Intervals / singletons (Tier 1.3 fast-path)
// ---------------------------------------------------------------------------

#[test]
fn single_note_from_singleton() {
    assert_infers(&[Pc0], "note");
}

#[test]
fn single_note_from_empty() {
    // Pc0 normalization (Tier 0.2) folds the empty set into {Pc0}.
    assert_infers(&[], "note");
}

#[test]
fn interval_major_third() {
    assert_infers(&[Pc0, Pc4], "M3");
}

#[test]
fn interval_minor_third() {
    assert_infers(&[Pc0, Pc3], "m3");
}

#[test]
fn interval_tritone() {
    assert_infers(&[Pc0, Pc6], "TT");
}

// Implicit-root coverage (Tier 0.2): Pc7-only matches Pc0+Pc7.
#[test]
fn implicit_root_pc7_alone_is_p5() {
    assert_infers(&[Pc7], "P5");
}

// ---------------------------------------------------------------------------
// Sevenths
// ---------------------------------------------------------------------------

#[test]
fn maj7() {
    assert_infers(&[Pc0, Pc4, Pc7, Pc11], "Δ7");
}

#[test]
fn dom7() {
    assert_infers(&[Pc0, Pc4, Pc7, Pc10], "7");
}

#[test]
fn min7() {
    assert_infers(&[Pc0, Pc3, Pc7, Pc10], "m7");
}

#[test]
fn min_maj7() {
    assert_infers(&[Pc0, Pc3, Pc7, Pc11], "mΔ7");
}

#[test]
fn dim7() {
    assert_infers(&[Pc0, Pc3, Pc6, Pc9], "dim7");
}

#[test]
fn half_dim7() {
    assert_infers(&[Pc0, Pc3, Pc6, Pc10], "m7b5");
}

#[test]
fn aug7() {
    assert_infers(&[Pc0, Pc4, Pc8, Pc10], "+7");
}

#[test]
fn aug_maj7() {
    assert_infers(&[Pc0, Pc4, Pc8, Pc11], "+Δ7");
}

// ---------------------------------------------------------------------------
// Sixths (Tier 1.2)
// ---------------------------------------------------------------------------

#[test]
fn sixth_major() {
    // Before 1.2 this renders as "Maj (13)"; after, as "Maj6".
    assert_infers(&[Pc0, Pc4, Pc7, Pc9], "Maj6");
}

#[test]
fn sixth_minor() {
    assert_infers(&[Pc0, Pc3, Pc7, Pc9], "m6");
}

// ---------------------------------------------------------------------------
// ♭5 vs ♯11 disambiguation (Tier 1.1)
// ---------------------------------------------------------------------------

#[test]
fn seven_flat_five_no_p5() {
    // No P5 present → ♭5, not ♯11.
    assert_infers(&[Pc0, Pc4, Pc6, Pc10], "7 (b5)");
}

#[test]
fn seven_sharp_eleven_with_p5() {
    // P5 present → ♯11 remains.
    assert_infers(&[Pc0, Pc4, Pc6, Pc7, Pc10], "7 (#11)");
}

#[test]
fn maj7_flat_five_no_p5() {
    assert_infers(&[Pc0, Pc4, Pc6, Pc11], "Δ7 (b5)");
}

#[test]
fn maj7_sharp_eleven_with_p5() {
    assert_infers(&[Pc0, Pc4, Pc6, Pc7, Pc11], "Δ7 (#11)");
}

// ---------------------------------------------------------------------------
// Altered dominant (Tier 1.4)
// ---------------------------------------------------------------------------

#[test]
fn altered_dominant_full() {
    // Full altered collection: 3rd, ♭7, and alterations on both 9- and 5- axes.
    assert_infers(&[Pc0, Pc1, Pc3, Pc4, Pc6, Pc8, Pc10], "7alt");
}

#[test]
fn dom7_flat9_stays_specific() {
    // Only one alteration → specific label, not `7alt`.
    assert_infers(&[Pc0, Pc1, Pc4, Pc7, Pc10], "7 (b9)");
}
