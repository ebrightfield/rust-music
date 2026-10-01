//! Example: render trills exercising the explicit `extension_length_ss`
//! field on the two single-purpose option bundles —
//! [`TrillBracketOptions`](music_engraver::layout::trill_bracket::TrillBracketOptions)
//! and [`TrillExtensionSpeedOptions`](music_engraver::layout::trill_extension::TrillExtensionSpeedOptions).
//!
//! Produces `examples/output/trill_options_with_length_score.svg`.
//!
//! Until the most recent chunk added `extension_length_ss` to the two
//! single-purpose bundles, a caller wanting "bracket + explicit length" or
//! "speed + explicit length" had to widen to
//! [`TrillExtensionFullOptions`](music_engraver::layout::trill_options::TrillExtensionFullOptions)
//! via `.into()` then layer in `.with_length_ss(...)`. Now both knobs are
//! reachable through the relevant single-purpose bundle directly.
//!
//! Four measures across two systems exercise both bundles and combinations
//! that ONLY the new field unlocks (without widening):
//! - M1: `TrillBracketOptions::new(Both).with_extension_length_ss(2.0)`
//!   — Both-side bracket capping a SHORT explicit-length wiggle (2.0ss).
//!   The End hook anchors at the shortened terminus.
//! - M2: `TrillBracketOptions::new(End).with_extension_length_ss(4.0)`
//!   — End-side bracket capping a MEDIUM explicit-length wiggle (4.0ss).
//!   Demonstrates the End hook moves with the explicit length.
//! - M3: `TrillExtensionSpeedOptions::new(Slow).with_extension_length_ss(2.0)`
//!   — Slow speed wiggle clamped to a SHORT explicit length (2.0ss).
//! - M4: `TrillExtensionSpeedOptions::new(Faster).with_extension_length_ss(3.0)`
//!   — Faster speed wiggle clamped to a MEDIUM explicit length (3.0ss).
//!   Last note of the system: explicit length disables cross-system
//!   propagation.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};
use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
use music_engraver::layout::trill_options::TrillExtensionFullOptions;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

/// The featured score: bracket+length on M1/M2, speed+length on M3/M4.
fn build_score() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: bracket Both + short explicit length (2.0ss).
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(2.0),
        )
        .barline()
        // M2: bracket End + medium explicit length (4.0ss).
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::End).with_extension_length_ss(4.0),
        )
        .barline()
        // M3: Slow speed + short explicit length (2.0ss).
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow).with_extension_length_ss(2.0),
        )
        .barline()
        // M4: Faster speed + medium explicit length (3.0ss). Last note of
        // system 2 — explicit length disables cross-system propagation.
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension_speed_with_options(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster).with_extension_length_ss(3.0),
        )
        .end_barline()
        .render_svg()
}

/// Same musical content but with NO explicit length on either bundle —
/// every wiggle takes the natural span (next-note left edge within-system,
/// system right edge cross-system). Used to prove the explicit length
/// actually shortened things.
fn build_score_no_length() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::End,
        ))
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Slow,
        ))
        .barline()
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(
            TrillWiggleSpeed::Faster,
        ))
        .end_barline()
        .render_svg()
}

/// Same musical content, but each option bundle is widened to the unified
/// `TrillExtensionFullOptions` via `.into()` (which carries the explicit
/// length through the documented `From` conversion). Used to verify the
/// single-purpose bundles produce a byte-identical SVG to the widened path.
fn build_score_widened() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(2.0),
        ))
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillBracketOptions::new(TrillBracketSide::End).with_extension_length_ss(4.0),
        ))
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow).with_extension_length_ss(2.0),
        ))
        .barline()
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension_full_options(TrillExtensionFullOptions::from(
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster).with_extension_length_ss(3.0),
        ))
        .end_barline()
        .render_svg()
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = build_score();
    let path = out_dir.join("trill_options_with_length_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "Wrote {} ({} bytes, {} paths, {} lines)",
        path.display(),
        svg.len(),
        path_count,
        line_count
    );

    // --- Structural assertions, each a regression canary for a specific
    // propagation path of the new `extension_length_ss` field: ---

    // (1) Basic SVG validity.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    let no_length = build_score_no_length();
    let no_length_paths = no_length.matches("<path").count();
    let no_length_lines = no_length.matches("<line ").count();

    // (2) The explicit length must SHORTEN the wiggles vs the no-length
    // baseline. All four chosen lengths (2.0, 4.0, 2.0, 3.0 ss) sit well
    // below the natural spans of the corresponding whole notes (the
    // wiggle in a whole-note measure naturally spans many staff spaces).
    // If the new field is silently dropped, both versions render the
    // same wiggle tile counts and this fires.
    assert!(
        path_count < no_length_paths,
        "explicit-length wiggles must produce fewer paths than no-length baseline: \
         with_length={path_count}, no_length={no_length_paths}"
    );

    // (3) Byte-inequality vs no-length baseline. The explicit length
    // must change SVG content; an SVG-level no-op refactor would fail
    // here.
    assert_ne!(
        svg, no_length,
        "explicit-length score must not be byte-identical to no-length score"
    );

    // (4) Bracket hook count: M1 Both = 2, M2 End = 1, M3/M4 no bracket.
    // Total bracket hook contribution = 3 <line> elements compared to a
    // hypothetical no-bracket variant. Critically: this must be the
    // SAME count as the no-length variant (the explicit length affects
    // the wiggle's terminus, NOT the number of bracket hooks). This
    // canary catches a regression where setting the explicit length
    // accidentally suppresses the bracket itself.
    assert_eq!(
        line_count, no_length_lines,
        "explicit length must NOT change bracket hook count: \
         with_length={line_count}, no_length={no_length_lines} \
         (the new field affects the wiggle terminus, not the bracket presence)"
    );

    // (5) Byte-equivalence with the widened-to-full-options path. The
    // documented `From<TrillBracketOptions>` and
    // `From<TrillExtensionSpeedOptions>` conversions on
    // `TrillExtensionFullOptions` propagate `extension_length_ss` into
    // the unified bundle's `length_ss`. So
    //   `trill_with_extension_bracketed_with_options(opts)`  and
    //   `trill_with_extension_full_options(opts.into())`
    // must produce byte-identical SVG. If a future refactor drops the
    // field from either side of the conversion, this fires.
    let widened = build_score_widened();
    assert_eq!(
        svg, widened,
        "single-purpose bundles with extension_length_ss must produce byte-identical SVG \
         to the widened TrillExtensionFullOptions path (proves the From conversion \
         propagates the new field)"
    );

    // (6) Sanity: at least one wiggle tile must still render. If all four
    // explicit lengths somehow get clamped to zero (regression in the
    // non-positive fail-safe), the output would degenerate to four
    // bracket-only "tr" glyphs with no wiggle at all. We pick a positive
    // length on every measure, so the explicit-length variant must still
    // have strictly more paths than a hypothetical bracket-only / "tr"
    // baseline. We approximate that lower bound with `no_length_paths -
    // many`: explicit-length must be > 0 paths and < no_length_paths,
    // which we already covered, so this just sanity-checks path_count.
    assert!(path_count > 0, "must render at least one path");
}
