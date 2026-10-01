//! Example: exercise [`ScoreBuilder::trill_with_extension_full_options`] across
//! several **bracket + speed + ornament** combinations on the same staff.
//!
//! The unique capability of [`TrillExtensionFullOptions`] over the two
//! single-purpose options bundles ([`TrillBracketOptions`] /
//! [`TrillExtensionSpeedOptions`]) is that bracket and wiggle speed can be set
//! **together in one call** — `TrillBracketOptions` can only set bracket fields
//! (no speed override), and `TrillExtensionSpeedOptions` can only set the speed
//! field (no bracket). Every measure here picks a combination that *neither*
//! single-purpose bundle can express on its own:
//!
//! - M1: `Both` bracket + `Slow` speed + `TrillWithMordent` — the
//!   "every-knob-set" trill: compound prefix glyph, sparse wiggle, square
//!   brackets at both ends.
//! - M2: `End` bracket + `Faster` speed + plain `Trill` + `Up` hook direction —
//!   bracket-with-direction-override paired with a denser-than-standard wiggle.
//! - M3: `Start` bracket + `Slowest` speed + `TrillWithMordent` + 1.0ss hook
//!   length — start-only bracket with custom length on the sparsest wiggle.
//! - M4: chord (C-E-G half) with `Both` bracket + `Standard` speed +
//!   `TrillWithMordent` — chord variant. (`Standard` is the renderer's default
//!   wiggle glyph, but it's still explicitly set through the options bundle so
//!   the chord arm of the builder is exercised end-to-end.)
//!
//! Closes the "visual proofing — a small example exercising bracket+speed
//! combinations on compound trills through `trill_with_extension_full_options`"
//! follow-up from the 2026-05-13 `TrillExtensionFullOptions` introduction.
//!
//! Produces `examples/output/trill_full_options_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
use music_engraver::layout::trill_extension::TrillWiggleSpeed;
use music_engraver::layout::trill_options::TrillExtensionFullOptions;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = build_full_options_score();

    let path = out_dir.join("trill_full_options_score.svg");
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

    // 2. Bracket-hook delta. M1 Both=2 + M2 End=1 + M3 Start=1 + M4 Both=2 = 6
    //    hook <line> elements. Compare against an identical score with no
    //    bracket on any measure (but same speeds + same ornaments + same
    //    extension flag), which isolates the brackets' contribution to the
    //    <line> count and proves every measure's bracket actually rendered.
    let no_bracket = build_full_options_no_bracket_variant();
    let no_bracket_lines = no_bracket.matches("<line ").count();
    let hook_delta = line_count.saturating_sub(no_bracket_lines);
    assert_eq!(
        hook_delta, 6,
        "expected exactly 6 bracket hook lines (M1 Both=2 + M2 End=1 + M3 Start=1 + M4 Both=2); \
         got delta={hook_delta} (full={line_count}, no_bracket={no_bracket_lines})"
    );

    // 3. Speed propagation canary. The same score with every measure's
    //    `.speed = None` (renderer falls back to Standard wiggle) must
    //    produce a byte-different SVG. Three measures use non-Standard
    //    speeds (Slow/Faster/Slowest); only M4 uses Standard. If the speed
    //    field silently failed to propagate from the full-options bundle,
    //    this assertion would fail.
    let no_speed = build_full_options_no_speed_variant();
    assert_ne!(
        svg, no_speed,
        "full-options score must differ from same options with speed=None: \
         three measures pick non-Standard wiggle glyphs"
    );

    // 4. Bracket propagation canary. The same score with every measure's
    //    `.bracket = None` (no hooks at all) must produce a byte-different
    //    SVG AND fewer <line> elements. If the bracket field silently
    //    failed to propagate, the line counts would match.
    let no_bracket_lines_count = no_bracket.matches("<line ").count();
    assert!(
        line_count > no_bracket_lines_count,
        "full-options score must have strictly more <line> elements than no-bracket variant: \
         full={line_count}, no_bracket={no_bracket_lines_count}"
    );

    // 5. Ornament propagation canary. The same score with every
    //    `.ornament(TrillWithMordent)` swapped for plain `Trill` (via
    //    `.ornament = None` which collapses to `Trill` at the builder)
    //    must produce a byte-different SVG. The compound glyph is ~470
    //    font-units wider than bare "tr" in Bravura, so the wiggle start
    //    positions differ. If `.with_ornament(TrillWithMordent)` silently
    //    failed to propagate, this would fail.
    let plain_trill = build_full_options_plain_trill_variant();
    assert_ne!(
        svg, plain_trill,
        "full-options compound score must differ from same options with ornament=None: \
         different prefix glyph, different wiggle start positions"
    );

    // 6. Bracket hook geometry is glyph-independent: plain-Trill and
    //    TrillWithMordent must emit the same <line> count for the same
    //    bracket settings. (The prefix glyph affects path data, not the
    //    hook count.) This sandwiches the previous assertion: ornament
    //    changed something, but it did NOT change the bracket hooks.
    assert_eq!(
        line_count,
        plain_trill.matches("<line ").count(),
        "bracket hook count must be glyph-independent — plain and compound \
         brackets emit the same <line> count"
    );

    // 7. The full-options score must have strictly more paths than the
    //    same score with no extension flag at all (every measure just
    //    `.ornament(TrillWithMordent)` with no wiggle, no brackets). Confirms
    //    the wiggle is actually rendering on at least some measures.
    let no_extension = build_full_options_no_extension_variant();
    let no_ext_paths = no_extension.matches("<path").count();
    assert!(
        path_count > no_ext_paths,
        "full-options score must add wiggle paths beyond plain compound (no extension): \
         full={path_count}, no_extension={no_ext_paths}"
    );
}

/// Canonical full-options score: every measure exercises a different
/// combination of bracket + speed + ornament knobs that only
/// [`TrillExtensionFullOptions`] can express in a single call.
pub fn build_full_options_score() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Both bracket + Slow speed + TrillWithMordent (every knob set).
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        // M2: End bracket + Faster speed + plain Trill + Up direction.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up)
                .with_speed(TrillWiggleSpeed::Faster),
        )
        .barline()
        // M3: Start bracket + Slowest speed + TrillWithMordent + 1.0ss length.
        .note(p(Note::B, 4), Duration::WHOLE)
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

/// Same musical content + same ornaments + same speeds + same extension flag,
/// but every measure's `.bracket = None` (no hooks). Used for the bracket
/// propagation canary: the line-count delta isolates the brackets' contribution.
fn build_full_options_no_bracket_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_speed(TrillWiggleSpeed::Slow)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new().with_speed(TrillWiggleSpeed::Faster),
        )
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
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
                .with_speed(TrillWiggleSpeed::Standard)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same musical content + same brackets + same ornaments + same extension flag,
/// but every measure's `.speed = None` (renderer uses Standard wiggle). Used
/// for the speed propagation canary.
fn build_full_options_no_speed_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::End)
                .with_bracket_direction(HookDirection::Up),
        )
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Start)
                .with_bracket_length_ss(1.0)
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
                .with_ornament(Ornament::TrillWithMordent),
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same brackets + same speeds + same extension flag, but every measure's
/// `.ornament = None` (collapses to plain `Trill` at the builder layer). Used
/// for the ornament propagation canary.
fn build_full_options_plain_trill_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Slow),
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
                .with_speed(TrillWiggleSpeed::Slowest),
        )
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_full_options(
            TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_speed(TrillWiggleSpeed::Standard),
        )
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same musical content, but every measure uses `.ornament(TrillWithMordent)`
/// with *no* extension and *no* bracket — just the compound ornament glyph
/// alone. Used as the no-wiggle / no-bracket baseline for the "wiggle is
/// actually rendering" sanity check.
fn build_full_options_no_extension_variant() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .ornament(Ornament::TrillWithMordent)
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .ornament(Ornament::TrillWithMordent)
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .ornament(Ornament::TrillWithMordent)
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}
