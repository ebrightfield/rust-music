//! Example: render precomposed compound trills (`Ornament::TrillWithMordent`)
//! at various wiggle speeds via
//! `ScoreBuilder::trill_with_extension_speed_with_options(opts)` with
//! `.with_ornament(Ornament::TrillWithMordent)`.
//!
//! Each measure picks a different `TrillWiggleSpeed` so the visual
//! difference between fast (dense tiles) and slow (sparse tiles) wiggles
//! is easy to compare alongside the wider compound prefix glyph. The
//! compound glyph (`OrnamentPrecompTrillWithMordent`) is ~470 font-units
//! wider than the bare "tr" in Bravura — the wiggle must start past the
//! *full* compound glyph rather than just the "tr" prefix.
//!
//! Closes the visual-proofing gap left open by the 2026-05-13
//! `TrillExtensionSpeedOptions` introduction chunk: the speed-options API
//! had byte-equivalence and renderer-propagation unit tests, but no
//! example you could open in a browser to confirm "compound at slow speed
//! looks like what I expect."
//!
//! Produces `examples/output/trill_speed_with_mordent_score.svg`.

use music::note::note::Note;
use music::note::pitch::Pitch;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = build_mixed_speed_compound();

    let path = out_dir.join("trill_speed_with_mordent_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
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

    // 2. The compound glyph + extension must add wiggle paths beyond the
    //    same score with `.ornament(TrillWithMordent)` (no extension).
    //    Sanity: the wiggle is actually rendering.
    let plain_compound = build_plain_compound_no_extension();
    let plain_paths = plain_compound.matches("<path").count();
    assert!(
        path_count > plain_paths,
        "compound-with-extension must add wiggle paths beyond plain compound: \
         compound_ext={path_count}, plain_compound={plain_paths}"
    );

    // 3. Regression canary: the compound-speed score must produce a
    //    byte-different SVG than the same speeds with plain `Trill`
    //    ornament. This is the key propagation test — proves
    //    `.with_ornament(Ornament::TrillWithMordent)` actually reached the
    //    renderer's glyph-advance lookup (bare "tr" = 521 fu vs compound =
    //    ~990 fu in Bravura, so wiggle start positions differ).
    let plain_trill_same_speeds = build_plain_trill_same_speeds();
    assert_ne!(
        svg, plain_trill_same_speeds,
        "compound-trill at speeds must differ byte-for-byte from plain-trill at \
         the same speeds: different prefix glyph, different wiggle start positions"
    );

    // 4. Regression canary: the compound-speed score must produce a
    //    byte-different SVG than the same content via
    //    `TrillExtensionSpeedOptions::new(speed)` (default ornament =
    //    plain Trill). Proves `.with_ornament` on the options bundle
    //    actually overrode the ornament, not just left it at Trill.
    let compound_via_options_default = build_default_ornament_options_same_speeds();
    assert_ne!(
        svg, compound_via_options_default,
        "compound override via .with_ornament must differ from options-default \
         (which collapses to Trill at the builder layer)"
    );

    // 5. The compound-speed score must produce strictly more tile paths
    //    than the same score with every wiggle pinned to `Slowest`.
    //    Sandwich-checks that the mixed speeds aren't being silently
    //    collapsed to a single glyph (the bug a regression here would
    //    show as identical path counts).
    let all_slowest_compound = build_all_slowest_compound();
    let slowest_paths = all_slowest_compound.matches("<path").count();
    assert!(
        path_count > slowest_paths,
        "mixed-speed compound must tile more total segments than all-Slowest \
         compound: mixed={path_count}, all-slowest={slowest_paths}"
    );

    // 6. The compound-speed score must produce strictly fewer tile paths
    //    than the same score with every wiggle pinned to `Fastest`.
    //    Other side of the sandwich.
    let all_fastest_compound = build_all_fastest_compound();
    let fastest_paths = all_fastest_compound.matches("<path").count();
    assert!(
        fastest_paths > path_count,
        "all-Fastest compound must tile more total segments than mixed: \
         all-fastest={fastest_paths}, mixed={path_count}"
    );
}

/// Canonical mixed-speed compound-trill score. Walks `TrillWiggleSpeed`
/// from fastest to slowest, one measure each, with the compound
/// `TrillWithMordent` ornament on every measure. Terminating quarters
/// anchor the wiggle of the last measure on each system.
pub fn build_mixed_speed_compound() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
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

    // Terminating quarter so the last measure's trill has a within-system
    // anchor for its incoming wiggle on the resumed system.
    b.note(p(Note::B, 5), Duration::QTR).end_barline().render_svg()
}

/// Same musical content + speeds as `build_mixed_speed_compound`, but
/// using the plain `trill_with_extension_speed(speed)` builder
/// (`Ornament::Trill`, no compound). Used for the regression canary that
/// the `.with_ornament(...)` propagated through the renderer.
fn build_plain_trill_same_speeds() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
    ];

    for (i, speed) in TrillWiggleSpeed::ALL.iter().enumerate() {
        let (n, oct) = pitches[i];
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed(*speed)
            .barline();
    }

    b.note(p(Note::B, 5), Duration::QTR).end_barline().render_svg()
}

/// Same musical content + speeds as `build_mixed_speed_compound`, but
/// using `TrillExtensionSpeedOptions::new(speed)` without
/// `.with_ornament(...)` (i.e. defaulting to `Ornament::Trill`). Used to
/// prove the `.with_ornament` half of the bundle propagated.
fn build_default_ornament_options_same_speeds() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
    ];

    for (i, speed) in TrillWiggleSpeed::ALL.iter().enumerate() {
        let (n, oct) = pitches[i];
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .trill_with_extension_speed_with_options(TrillExtensionSpeedOptions::new(*speed))
            .barline();
    }

    b.note(p(Note::B, 5), Duration::QTR).end_barline().render_svg()
}

/// Same musical content as `build_mixed_speed_compound`, but every
/// measure uses `.ornament(Ornament::TrillWithMordent)` with *no*
/// extension flag. Used as the no-wiggle baseline for the "wiggle is
/// actually rendering" sanity check.
fn build_plain_compound_no_extension() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
    ];

    for (n, oct) in pitches {
        b = b
            .note(p(n, oct), Duration::WHOLE)
            .ornament(Ornament::TrillWithMordent)
            .barline();
    }

    b.note(p(Note::B, 5), Duration::QTR).end_barline().render_svg()
}

/// All-Slowest compound-trill version of the canonical score. Lower
/// bookend for the sandwich check that mixed speeds aren't being
/// collapsed.
fn build_all_slowest_compound() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
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

    b.note(p(Note::B, 5), Duration::QTR).end_barline().render_svg()
}

/// All-Fastest compound-trill version of the canonical score. Upper
/// bookend for the sandwich check.
fn build_all_fastest_compound() -> String {
    let mut b = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(3);

    let pitches = [
        (Note::G, 4),
        (Note::A, 4),
        (Note::B, 4),
        (Note::C, 5),
        (Note::D, 5),
        (Note::E, 5),
        (Note::F, 5),
        (Note::G, 5),
        (Note::A, 5),
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

    b.note(p(Note::B, 5), Duration::QTR).end_barline().render_svg()
}
