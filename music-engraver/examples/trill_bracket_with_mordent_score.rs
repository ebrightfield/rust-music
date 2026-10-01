//! Example: render bracketed precomposed compound trills via
//! `ScoreBuilder::trill_with_extension_bracketed_with_options(opts)` with
//! `.with_ornament(Ornament::TrillWithMordent)`.
//!
//! The bracketed compound is the "duration-explicit, mordent-suffixed" trill —
//! `OrnamentPrecompTrillWithMordent` as the prefix glyph, a wavy line under
//! the trilled span, and short vertical hooks at one or both ends of the
//! wavy line marking the trill's precise extent. Common in Baroque keyboard
//! repertoire where the duration of a trill must be unambiguous and the
//! mordent suffix is part of the ornament's character.
//!
//! Produces `examples/output/trill_bracket_with_mordent_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketOptions, TrillBracketSide};
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = build_bracketed_compound();

    let path = out_dir.join("trill_bracket_with_mordent_score.svg");
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

    // ---- Structural assertions ----

    // 1. Basic SVG structure.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    // 2. The bracketed compound score must add hook lines vs the plain
    //    (no-bracket) compound-extension score over the same content. Six
    //    bracket sides total across the example (see `build_bracketed_compound`
    //    for the per-measure breakdown):
    //      M1 Both     → 2 hooks
    //      M2 Start    → 1 hook
    //      M3 End      → 1 hook
    //      M4 chord Both → 2 hooks
    //    Total: 6 hook <line>s added vs the no-bracket compound baseline.
    let no_bracket = build_no_bracket_compound();
    let bracket_lines = svg.matches("<line ").count();
    let no_bracket_lines = no_bracket.matches("<line ").count();
    let delta = bracket_lines.saturating_sub(no_bracket_lines);
    assert_eq!(
        delta, 6,
        "Expected 6 hook lines added by bracket geometry (M1 Both+M2 Start+M3 End+M4 chord Both); \
         got delta={delta} (bracketed={bracket_lines}, no_bracket={no_bracket_lines})"
    );

    // 3. The bracketed compound must produce a *different* SVG than the
    //    same-shape score with plain `Ornament::Trill` brackets. This is the
    //    key regression canary that the ornament override actually took
    //    effect through the renderer (different prefix glyph + wider wiggle
    //    start).
    let plain_trill_bracketed = build_bracketed_plain_trill();
    assert_ne!(
        svg, plain_trill_bracketed,
        "Bracketed compound must differ from bracketed plain trill: different prefix glyph, \
         wider wiggle start"
    );

    // 4. The hook *count* must match between the bracketed compound and the
    //    bracketed plain trill: bracket geometry is glyph-independent.
    assert_eq!(
        bracket_lines,
        plain_trill_bracketed.matches("<line ").count(),
        "Bracket hook count must be glyph-independent — plain-trill and compound brackets \
         emit the same number of <line>s"
    );
}

/// Build the canonical bracketed-compound-trill score.
///
/// 2 measures per system, 4 measures total = 2 systems.
///
/// - M1: whole-note compound-trill with `Both` brackets (default direction
///   Down, default length ~0.75ss).
/// - M2: whole-note compound-trill with `Start`-only bracket, `Up`
///   direction, custom 1.0ss length. Tests partial overrides through the
///   options builder.
/// - M3: whole-note compound-trill with `End`-only bracket, default
///   direction, custom 0.5ss length.
/// - M4: chord compound-trill with `Both` brackets, default direction,
///   default length. Tests chord targeting.
pub fn build_bracketed_compound() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Both brackets, all defaults, compound ornament override.
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        .barline()
        // M2: Start only, Up direction, 1.0ss length.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Start)
                .with_ornament(Ornament::TrillWithMordent)
                .with_direction(HookDirection::Up)
                .with_length_ss(1.0),
        )
        .barline()
        // M3: End only, default direction, 0.5ss length.
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::End)
                .with_ornament(Ornament::TrillWithMordent)
                .with_length_ss(0.5),
        )
        .barline()
        // M4: chord compound-trill, Both, defaults.
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Both)
                .with_ornament(Ornament::TrillWithMordent),
        )
        // M4 padding quarter so the bracket has a target note.
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same score as `build_bracketed_compound` but with plain (no-bracket)
/// `.trill_with_mordent_with_extension()` calls. Used as the baseline for
/// counting the hook <line> delta.
fn build_no_bracket_compound() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_mordent_with_extension()
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}

/// Same score as `build_bracketed_compound` but with plain `Trill` ornament
/// brackets (no `.with_ornament`). Used to prove the compound override
/// produces a *different* SVG output. Bracket geometry (hook count) is
/// glyph-independent, so the <line> count must match.
fn build_bracketed_plain_trill() -> String {
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
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::Start)
                .with_direction(HookDirection::Up)
                .with_length_ss(1.0),
        )
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_bracketed_with_options(
            TrillBracketOptions::new(TrillBracketSide::End).with_length_ss(0.5),
        )
        .barline()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension_bracketed_with_options(TrillBracketOptions::new(
            TrillBracketSide::Both,
        ))
        .note(p(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg()
}
