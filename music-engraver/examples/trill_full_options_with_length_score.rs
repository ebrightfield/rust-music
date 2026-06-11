//! Example: exercise [`TrillExtensionFullOptions::with_length_ss`] alongside
//! the bracket / speed / ornament knobs in a single call — the truly unique
//! capability of the unified options bundle.
//!
//! Until this chunk added a dedicated example, the
//! [`TrillExtensionFullOptions::with_length_ss`] path was visually proofed only
//! transitively: the existing `trill_options_with_length` golden asserts
//! byte-equivalence between the single-purpose bundles (with explicit length)
//! and the *widened* `From<...> + with_length_ss(...)` path, which means the
//! unified bundle's length knob has SVG-level coverage when its other knobs
//! match what the single-purpose bundles can express. But the four-way
//! combination `bracket + speed + ornament + length`, where ALL four are
//! actively set, cannot be reached through either single-purpose bundle in a
//! single call — exercising it directly here closes that gap.
//!
//! Produces `examples/output/trill_full_options_with_length_score.svg`.
//!
//! Four measures across two systems mirror the existing `trill_full_options`
//! example exactly (same bracket / speed / ornament choices). Each measure
//! adds a `.with_length_ss(...)` call so every byte of difference vs
//! `trill_full_options.svg` is attributable to the new length field:
//!
//! - M1: `Both` bracket + `Slow` speed + `TrillWithMordent` + 2.0ss length.
//! - M2: `End` bracket + `Up`-direction hook + `Faster` speed + plain `Trill`
//!   + 4.0ss length.
//! - M3: `Start` bracket + 1.0ss hook length + `Slowest` speed +
//!   `TrillWithMordent` + 3.0ss extension length. (`Slowest` is the widest
//!   wiggle glyph — must give it enough span to render a full segment, or the
//!   layout fail-safe drops the wiggle AND its Start hook.)
//! - M4: chord (C-E-G half) + `Both` bracket + `Standard` speed +
//!   `TrillWithMordent` + 2.5ss extension length, followed by a plain
//!   quarter `D4`. The chord arm of the builder carries the length too.

use music::note::note::Note;
use music::note::pitch::Pitch;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
use music_engraver::layout::trill_extension::TrillWiggleSpeed;
use music_engraver::layout::trill_options::TrillExtensionFullOptions;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

/// The featured score: every measure exercises the full four-way combination
/// `bracket + speed + ornament + length` through a single
/// [`TrillExtensionFullOptions`] call.
pub fn build_full_options_with_length_score() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Both bracket + Slow speed + TrillWithMordent + 2.0ss length.
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(2.0),
        )
        .barline()
        // M2: End bracket + Up direction + Faster speed + plain Trill + 4.0ss length.
        // M2 is the last note of system 1: an explicit length disables
        // cross-system propagation, so the End hook anchors here on system 1
        // rather than continuing to system 2.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up)
                .with_speed(TrillWiggleSpeed::Faster)
                .with_length_ss(4.0),
        )
        .barline()
        // M3: Start bracket + 1.0ss hook + Slowest speed + TrillWithMordent +
        // 3.0ss length. `Slowest` is the widest wiggle glyph (~2.4ss advance
        // in Bravura); a too-short length triggers the renderer's
        // single-segment fail-safe and drops both the wiggle AND its Start
        // hook. 3.0ss leaves room for one segment + a Start hook.
        .note(p(Note::B, 4), Duration::WHOLE)
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
        // The chord arm of the builder must carry the length field through to
        // the renderer the same way the Note arm does.
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(2.5),
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same content but with `with_length_ss(...)` stripped from every measure —
/// every wiggle takes its natural span. Used as the length-isolation baseline:
/// any SVG difference is attributable solely to the new length field. Note
/// that M2 (last note of system 1) is cross-system in this variant, so its
/// wiggle and End hook propagate to system 2; the with-length variant pulls
/// the End hook back to system 1 instead. **Net** bracket-hook count is
/// invariant (one End hook drawn either way), but its position changes.
fn build_full_options_no_length_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up)
                .with_speed(TrillWiggleSpeed::Faster),
        )
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Start)
                .with_bracket_length_ss(1.0)
                .with_speed(TrillWiggleSpeed::Slowest)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same content with every measure's length set to a *single* value (2.0ss)
/// instead of the four heterogeneous lengths in the featured score. Used to
/// prove the per-measure lengths are independently propagated — if a future
/// refactor accidentally collapsed `length_ss` to a global value, the two
/// variants would converge.
fn build_full_options_uniform_length_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(2.0),
        )
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up)
                .with_speed(TrillWiggleSpeed::Faster)
                .with_length_ss(2.0),
        )
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Start)
                .with_bracket_length_ss(1.0)
                .with_speed(TrillWiggleSpeed::Slowest)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(3.0),
        )
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(2.0),
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = build_full_options_with_length_score();
    let path = out_dir.join("trill_full_options_with_length_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line ").count();
    println!(
        "Wrote {} ({} bytes, {} paths, {} lines)",
        path.display(),
        svg.len(),
        path_count,
        line_count
    );

    // ---- Structural assertions, each a regression canary for a specific
    // propagation path of the new four-way combination: ----

    // (1) Basic SVG validity.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    let no_length = build_full_options_no_length_variant();
    let no_length_paths = no_length.matches("<path").count();
    let no_length_lines = no_length.matches("<line ").count();

    // (2) The four explicit lengths (2.0, 4.0, 3.0, 2.5 ss) all sit well
    // below the natural spans of their corresponding whole/half notes.
    // The new field MUST shorten the wiggle path count.
    //
    // This is the new-capability canary at the example layer: if the
    // `length_ss` field on the unified bundle stops propagating into
    // `NoteAnnotations::trill_extension_length_ss`, this fires.
    assert!(
        path_count < no_length_paths,
        "explicit-length wiggles must produce fewer paths than no-length baseline: \
         with_length={path_count}, no_length={no_length_paths}"
    );

    // (3) Byte-inequality vs no-length baseline. A no-op refactor that
    // accidentally stripped the length field at the score-builder layer
    // would fall through this.
    assert_ne!(
        svg, no_length,
        "explicit-length score must not be byte-identical to no-length variant"
    );

    // (4) Bracket hook count IS invariant — even though M2's cross-system
    // End hook moves between systems (system 2 in the no-length variant
    // via `draw_cross_system_trill_extensions`, system 1 in the
    // with-length variant via the within-system pass), the **net total**
    // is the same: 1 End hook for M2 is drawn either way.
    //
    // This is a stronger statement than `lines <= no_length_lines`:
    // catches a regression where setting the length suppresses a hook
    // entirely (e.g., a future bug where the end_x clamp accidentally
    // drops the End hook on the source system without also dropping the
    // cross-system propagation flag — net hooks would decrease).
    //
    // The lengths in this score are chosen so that every wiggle is wide
    // enough to render at least one full segment. (M3 with Slowest
    // speed needs ~3.0ss minimum to render any wiggle at all.)
    assert_eq!(
        line_count, no_length_lines,
        "explicit length must NOT change net bracket hook count: \
         with_length={line_count}, no_length={no_length_lines} \
         (cross-system continuation hooks move between systems but \
          totals stay invariant)"
    );

    // (5) Per-measure independence: a variant where every measure uses
    // 2.0ss (with M3 bumped to 3.0ss for the wiggle-too-short fail-safe)
    // must produce a byte-different SVG. If a future refactor
    // accidentally collapsed `length_ss` to a global field at the
    // builder layer, the two variants would converge.
    let uniform = build_full_options_uniform_length_variant();
    assert_ne!(
        svg, uniform,
        "per-measure lengths must propagate independently — heterogeneous \
         (2.0/4.0/3.0/2.5) must differ from uniform (2.0/2.0/3.0/2.0)"
    );

    // (6) Sanity: at least one wiggle tile renders.
    assert!(path_count > 0, "must render at least one path");

    // (7) The shortened score is strictly smaller than the no-length
    // baseline by a meaningful amount — explicit lengths remove wiggle
    // segments, never add. A ~30% byte reduction is conservative
    // (whole-note natural wiggles tile many segments).
    assert!(
        svg.len() < no_length.len() * 9 / 10,
        "shortened score must be substantially smaller (>10% byte reduction): \
         with_length={}, no_length={}",
        svg.len(),
        no_length.len()
    );
}
