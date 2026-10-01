//! Tier 2 — config-matrix tests.
//!
//! Exercises that the same input pcset, rendered under different
//! `NamingConfig` and `ChordNameDisplayConfig` combinations, produces the
//! documented distinct output. Covers:
//!   - `MajNotation` (Delta / Maj / MajCap / LowerMaj)
//!   - `utf8_accidentals` (ASCII vs unicode)
//!   - `prefer_add_notation` (Tier 2.2)
//!   - `show_omissions` (Tier 2.3)
//!   - `distinguish_sixth_from_thirteenth` (Tier 2.6)
//!   - `report_ambiguities` (Tier 2.5)

use std::collections::HashSet;

use music::note::pitch_class::Pc;
use music::note::pitch_class::Pc::*;
use music::note_collections::chord_name::naming_heuristics::{
    infer_chord_quality_detailed, infer_chord_quality_with,
};
use music::note_collections::chord_name::quality::chord::QualityAmbiguity;
use music::note_collections::chord_name::{ChordNameDisplayConfig, MajNotation, NamingConfig};

/// Run inference with the supplied `NamingConfig` and render under the
/// supplied `ChordNameDisplayConfig`.
fn render(pcs: &[Pc], naming: &NamingConfig, display: &ChordNameDisplayConfig) -> String {
    let pcs: HashSet<Pc> = pcs.iter().copied().collect();
    let (_, q) = infer_chord_quality_with(&pcs, naming).expect("inference must succeed");
    q.expect("quality must be produced").to_string(display)
}

// ---------------------------------------------------------------------------
// MajNotation toggle (Tier 2.1)
// ---------------------------------------------------------------------------

#[test]
fn maj_notation_delta_is_default() {
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(
        render(&[Pc0, Pc4, Pc7, Pc11], &NamingConfig::default(), &dsp),
        "Δ7"
    );
}

#[test]
fn maj_notation_maj_capital() {
    let dsp = ChordNameDisplayConfig {
        maj_notation: MajNotation::Maj,
        ..Default::default()
    };
    assert_eq!(
        render(&[Pc0, Pc4, Pc7, Pc11], &NamingConfig::default(), &dsp),
        "Maj7"
    );
}

#[test]
fn maj_notation_compact_m() {
    let dsp = ChordNameDisplayConfig {
        maj_notation: MajNotation::MajCap,
        ..Default::default()
    };
    assert_eq!(
        render(&[Pc0, Pc4, Pc7, Pc11], &NamingConfig::default(), &dsp),
        "M7"
    );
}

#[test]
fn maj_notation_lowercase_maj() {
    let dsp = ChordNameDisplayConfig {
        maj_notation: MajNotation::LowerMaj,
        ..Default::default()
    };
    assert_eq!(
        render(&[Pc0, Pc4, Pc7, Pc11], &NamingConfig::default(), &dsp),
        "maj7"
    );
}

// ---------------------------------------------------------------------------
// utf8_accidentals toggle (Tier 2.1)
// ---------------------------------------------------------------------------

#[test]
fn utf8_accidentals_renders_unicode_flat() {
    let dsp = ChordNameDisplayConfig {
        utf8_accidentals: true,
        ..Default::default()
    };
    // 7(b5) with unicode: 7(♭5)
    assert_eq!(
        render(&[Pc0, Pc4, Pc6, Pc10], &NamingConfig::default(), &dsp),
        "7 (♭5)"
    );
}

#[test]
fn utf8_accidentals_renders_unicode_sharp() {
    let dsp = ChordNameDisplayConfig {
        utf8_accidentals: true,
        ..Default::default()
    };
    // 7(#11) with P5 present and unicode: 7(♯11)
    assert_eq!(
        render(&[Pc0, Pc4, Pc6, Pc7, Pc10], &NamingConfig::default(), &dsp),
        "7 (♯11)"
    );
}

#[test]
fn utf8_accidentals_default_is_ascii() {
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(
        render(&[Pc0, Pc4, Pc6, Pc10], &NamingConfig::default(), &dsp),
        "7 (b5)"
    );
}

// ---------------------------------------------------------------------------
// prefer_add_notation (Tier 2.2)
// ---------------------------------------------------------------------------

#[test]
fn add9_with_prefer_add_notation() {
    let naming = NamingConfig::default().prefer_add_notation(true);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc2, Pc4, Pc7], &naming, &dsp), "add9");
}

#[test]
fn default_renders_ninth_as_alt() {
    // Without the add flag, a triad + 9th renders as "Maj (9)".
    let naming = NamingConfig::default().prefer_add_notation(false);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc2, Pc4, Pc7], &naming, &dsp), "Maj (9)");
}

#[test]
fn minor_add9_with_prefer_add_notation() {
    let naming = NamingConfig::default().prefer_add_notation(true);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc2, Pc3, Pc7], &naming, &dsp), "madd9");
}

#[test]
fn add11_no_seventh_no_ninth() {
    let naming = NamingConfig::default().prefer_add_notation(true);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc4, Pc5, Pc7], &naming, &dsp), "add11");
}

#[test]
fn add_six_nine() {
    // {root, 9, 3, 5, 6} with add flag → "6/9"
    let naming = NamingConfig::default().prefer_add_notation(true);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc2, Pc4, Pc7, Pc9], &naming, &dsp), "6/9");
}

// ---------------------------------------------------------------------------
// show_omissions (Tier 2.3)
// ---------------------------------------------------------------------------

#[test]
fn show_omissions_emits_no5() {
    // {root, M3, b7} — dominant shell, missing P5 (Pc7).
    let naming = NamingConfig::default().show_omissions(true);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc4, Pc10], &naming, &dsp), "7 (no5)");
}

#[test]
fn show_omissions_off_hides_no5() {
    // Same chord with show_omissions=false: plain "7".
    let naming = NamingConfig::default().show_omissions(false);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc4, Pc10], &naming, &dsp), "7");
}

#[test]
fn show_omissions_complete_chord_no_emission() {
    // Complete Maj7: nothing omitted, no "no5" marker.
    let naming = NamingConfig::default().show_omissions(true);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc4, Pc7, Pc10], &naming, &dsp), "7");
}

// ---------------------------------------------------------------------------
// distinguish_sixth_from_thirteenth (Tier 2.6)
// ---------------------------------------------------------------------------

#[test]
fn distinguish_sixth_on_emits_six_chord() {
    let naming = NamingConfig::default().distinguish_sixth_from_thirteenth(true);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc4, Pc7, Pc9], &naming, &dsp), "Maj6");
}

#[test]
fn distinguish_sixth_off_emits_thirteen_alt() {
    // Legacy pre-Tier-1.2 behavior: render as "Maj (13)".
    let naming = NamingConfig::default().distinguish_sixth_from_thirteenth(false);
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc4, Pc7, Pc9], &naming, &dsp), "Maj (13)");
}

// ---------------------------------------------------------------------------
// report_ambiguities (Tier 2.5)
// ---------------------------------------------------------------------------

#[test]
fn ambiguity_duplicate_third() {
    let naming = NamingConfig::default().report_ambiguities(true);
    let pcs: HashSet<Pc> = [Pc0, Pc3, Pc4, Pc7].iter().copied().collect();
    let (_q, ambiguities) = infer_chord_quality_detailed(&pcs, &naming);
    assert!(
        ambiguities
            .iter()
            .any(|a| matches!(a, QualityAmbiguity::DuplicateScaleDegree { degree: 3, .. })),
        "expected duplicate-3rd ambiguity, got {:?}",
        ambiguities,
    );
}

#[test]
fn ambiguity_sixth_vs_thirteenth() {
    let naming = NamingConfig::default().report_ambiguities(true);
    let pcs: HashSet<Pc> = [Pc0, Pc4, Pc7, Pc9].iter().copied().collect();
    let (_q, ambiguities) = infer_chord_quality_detailed(&pcs, &naming);
    assert!(
        ambiguities.iter().any(|a| matches!(
            a,
            QualityAmbiguity::SixthVsThirteenth { has_seventh: false }
        )),
        "expected 6-vs-13 ambiguity, got {:?}",
        ambiguities,
    );
}

#[test]
fn ambiguity_off_returns_empty() {
    let naming = NamingConfig::default().report_ambiguities(false);
    let pcs: HashSet<Pc> = [Pc0, Pc3, Pc4, Pc7].iter().copied().collect();
    let (_q, ambiguities) = infer_chord_quality_detailed(&pcs, &naming);
    assert!(
        ambiguities.is_empty(),
        "expected no ambiguities reported when flag is off"
    );
}

// ---------------------------------------------------------------------------
// explicit_sus4
// ---------------------------------------------------------------------------

#[test]
fn suspension_style_can_be_explicit_or_compact() {
    let compact = ChordNameDisplayConfig::default();
    let explicit = ChordNameDisplayConfig {
        explicit_sus4: true,
        ..Default::default()
    };
    assert_eq!(
        render(&[Pc0, Pc5, Pc7], &NamingConfig::default(), &compact),
        "sus4"
    );
    assert_eq!(
        render(&[Pc0, Pc5, Pc7, Pc10], &NamingConfig::default(), &compact),
        "7sus"
    );
    assert_eq!(
        render(&[Pc0, Pc5, Pc7, Pc10], &NamingConfig::default(), &explicit),
        "7sus4"
    );
}

// ---------------------------------------------------------------------------
// Preset sanity
// ---------------------------------------------------------------------------

#[test]
fn preset_pop_prefers_add() {
    let naming = NamingConfig::pop();
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc2, Pc4, Pc7], &naming, &dsp), "add9");
}

#[test]
fn preset_strict_prefers_add() {
    let naming = NamingConfig::strict();
    let dsp = ChordNameDisplayConfig::default();
    assert_eq!(render(&[Pc0, Pc2, Pc4, Pc7], &naming, &dsp), "add9");
}

#[test]
fn preset_jazz_uses_alt_form() {
    let naming = NamingConfig::jazz();
    let dsp = ChordNameDisplayConfig::default();
    // Jazz keeps add-preference off, so the 9 renders as an alt.
    assert_eq!(render(&[Pc0, Pc2, Pc4, Pc7], &naming, &dsp), "Maj (9)");
}
