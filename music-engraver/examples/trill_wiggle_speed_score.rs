//! Example: render trills with the various SMuFL `wiggleTrill*` speed
//! variants via the ScoreBuilder `.trill_with_extension_speed(speed)`
//! method. Each measure picks a different speed so the visual difference
//! between fast (dense) and slow (sparse) wiggles is easy to compare in
//! one rendering.
//!
//! Produces `examples/output/trill_wiggle_speed_score.svg`.

use music::note::note::Note;
use music::note::pitch::Pitch;
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::trill_extension::TrillWiggleSpeed;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: u8) -> Pitch {
    Pitch::new(note, octave).expect("valid pitch")
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: Fastest wiggle — densest tiles, visually communicating a rapid trill.
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Fastest)
        .barline()
        // M2: Fast wiggle — one step less dense.
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Fast)
        .barline()
        // M3: Standard wiggle — the neutral default (`wiggleTrill`).
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Standard)
        .barline()
        // M4: Slow wiggle — sparser tiles, a slower trill.
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slow)
        .barline()
        // M5: Slowest wiggle — sparsest tiles, a deliberately slow trill.
        // The wiggle terminates at the right edge of the measure (cross-
        // system continuation, since this is the last note of system 3).
        .note(p(Note::D, 5), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
        .end_barline()
        // M6: terminating quarter — receives the incoming wiggle from M5.
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::F, 5), Duration::QTR)
        .note(p(Note::G, 5), Duration::QTR)
        .note(p(Note::A, 5), Duration::QTR)
        .end_barline()
        .render_svg();

    let path = out_dir.join("trill_wiggle_speed_score.svg");
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

    // Basic SVG sanity.
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    // Conservative lower bound: 5 noteheads + 4 quarter noteheads + clef +
    // 5 "tr" glyphs + many wiggle segments → easily ≥18 paths.
    assert!(
        path_count >= 18,
        "expected at least 18 paths, got {path_count}"
    );

    // Compare against the same score with all wiggles at Slowest density:
    // the mixed-speed version must produce strictly more tile paths because
    // four of the five measures use a denser glyph than Slowest.
    let all_slowest = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
        .barline()
        .note(p(Note::A, 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
        .barline()
        .note(p(Note::C, 5), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
        .barline()
        .note(p(Note::D, 5), Duration::WHOLE)
        .trill_with_extension_speed(TrillWiggleSpeed::Slowest)
        .end_barline()
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::F, 5), Duration::QTR)
        .note(p(Note::G, 5), Duration::QTR)
        .note(p(Note::A, 5), Duration::QTR)
        .end_barline()
        .render_svg();

    let slowest_paths = all_slowest.matches("<path").count();
    assert!(
        path_count > slowest_paths,
        "mixed-speed wiggle must tile more total segments than all-Slowest: \
         mixed={path_count}, all-slowest={slowest_paths}"
    );
}
