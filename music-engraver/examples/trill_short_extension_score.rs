//! Example: render trills with explicit-length wavy-line extensions using
//! the ScoreBuilder `.trill_with_extension_length_ss(length_ss)` method.
//!
//! Produces `examples/output/trill_short_extension_score.svg`. The default
//! `.trill_with_extension()` runs the wiggle until the next note (or the
//! system's right edge for the last note of a system). The explicit-length
//! variant lets the wiggle "run out" earlier — useful when a trill should
//! release partway through a held note's natural duration.
//!
//! Four measures across two systems explore the feature:
//! - M1: a whole-note trill with default extension (baseline for the eye)
//! - M2: a whole-note trill with explicit 2.0-ss length — the wiggle
//!   terminates clearly before the next note
//! - M3: a whole-note trill with explicit 4.0-ss length — longer than M2
//!   but still short of the natural span (the same trill at different
//!   release points reads clearly against M1)
//! - M4: a trill on the LAST note of the second system with an explicit
//!   1.5-ss length — proves the explicit length disables cross-system
//!   propagation (without it, the wiggle would extend to the system's
//!   right edge)

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: u8) -> Pitch {
    Pitch::new(note, octave).expect("valid pitch")
}

fn build_score() -> String {
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: baseline — whole-note trill with default-length extension.
        // The wiggle runs to the next note (M2's first event).
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        // M2: whole-note trill with explicit 2.0-ss length — short wiggle.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_length_ss(2.0)
        .barline()
        // M3: whole-note trill with explicit 4.0-ss length — medium wiggle.
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_length_ss(4.0)
        .barline()
        // M4: trill on the LAST note of the system. Without an explicit
        // length the wiggle would extend to the right edge of the system.
        // With explicit 1.5-ss, it stops well before the barline.
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension_length_ss(1.5)
        .end_barline()
        .render_svg()
}

fn build_score_default_lengths() -> String {
    // Same musical content but with default (next-note / system-edge)
    // extensions on every trill — used as a regression baseline to prove
    // the explicit-length call actually changed something.
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension()
        .end_barline()
        .render_svg()
}

fn build_score_plain_trill() -> String {
    // Same musical content with only the "tr" prefix glyph — no wiggle
    // anywhere. Used to verify the new method's wiggle contribution
    // exceeds zero (i.e. the explicit-length wiggle isn't silently dropped
    // for all four measures).
    ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p(Note::C, 5), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .end_barline()
        .render_svg()
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = build_score();
    let path = out_dir.join("trill_short_extension_score.svg");
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

    // --- Structural assertions, every one of which would fail under a
    // specific regression of the explicit-length feature: ---

    // (1) Basic SVG validity.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    // (2) Explicit-length wiggles still draw at least *some* segments —
    // the explicit-length path must not silently drop the entire wiggle
    // on every measure. Compared against the plain-trill baseline (no
    // wiggle at all), the explicit-length version must have strictly
    // more paths.
    let plain = build_score_plain_trill();
    let plain_paths = plain.matches("<path").count();
    assert!(
        path_count > plain_paths,
        "explicit-length wiggles must still draw segments: \
         with_explicit={path_count}, plain_trill_only={plain_paths}"
    );

    // (3) Explicit-length wiggles are SHORTER than default-length wiggles
    // for the same musical content. Across four measures with short
    // explicit lengths (2.0, 4.0, 1.5 ss vs. natural next-note / edge
    // distances), the explicit version must have strictly fewer paths
    // (the wiggle on M2/M3/M4 each loses tiles).
    let defaults = build_score_default_lengths();
    let default_paths = defaults.matches("<path").count();
    assert!(
        path_count < default_paths,
        "explicit-length wiggles must be shorter than default-length: \
         with_explicit={path_count}, with_defaults={default_paths}"
    );

    // (4) Byte-inequality vs the all-defaults baseline — different lengths,
    // different wiggle terminus positions, different SVG bytes.
    assert_ne!(
        svg, defaults,
        "explicit-length score must not be byte-identical to all-defaults score"
    );

    // (5) The last note (in M4) has an explicit length. Without it, the
    // wiggle would extend to the system right edge (cross-system case
    // when M4 is the last note of the system). With explicit length, it
    // terminates earlier — so the explicit-length total path count must
    // be strictly less than the all-defaults total path count, and
    // specifically the last-measure's contribution must be smaller.
    //
    // Test this via a tighter comparison: build a score where M4 has the
    // default extension but M1-M3 have the same explicit lengths as the
    // primary score. The difference is exactly M4's wiggle terminus.
    let hybrid = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_length_ss(2.0)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_length_ss(4.0)
        .barline()
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension() // M4: default this time
        .end_barline()
        .render_svg();
    let hybrid_paths = hybrid.matches("<path").count();
    assert!(
        path_count < hybrid_paths,
        "explicit length on the last system's last note must yield fewer paths than \
         default cross-system extension: with_explicit_M4={path_count}, default_M4={hybrid_paths}"
    );
}
