//! Example: render notes with trill ornaments + wavy-line extensions using
//! the ScoreBuilder `.trill_with_extension()` method.
//!
//! Produces `examples/output/trill_extension_score.svg`. Exercises the
//! per-note wavy-line continuation that should follow a sustained "tr".

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
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
        // M1: half-note trill with extension to a quarter, plain quarter, plain quarter
        .note(p(Note::G, 4), Duration::HALF)
        .trill_with_extension()
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .barline()
        // M2: dotted-half-note trill with extension over a long span, then a quarter
        .note(p(Note::E, 5), Duration::new(DurationKind::Half, 1))
        .trill_with_extension()
        .note(p(Note::D, 5), Duration::QTR)
        .barline()
        // M3: whole-note trill with extension across the whole measure
        // (next note is in M4, so the wiggle runs to the right edge of M3)
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        // M4: plain quarter receives the trill's wiggle endpoint, then chord trill
        .note(p(Note::C, 5), Duration::QTR)
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension()
        .end_barline()
        // M5: terminating note (final note in the system gets no wiggle)
        .note(p(Note::F, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let path = out_dir.join("trill_extension_score.svg");
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

    // Verify basic SVG structure
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");

    // We expect: 5 noteheads + chord (3) + clef (1) + 4 "tr" glyphs +
    // many wiggle segments. Use a conservative lower bound.
    assert!(
        path_count >= 18,
        "expected at least 18 paths (noteheads + clef + trills + wiggles), got {path_count}"
    );

    // Compare against the same score without extensions: extensions must add
    // tangible path content. We rebuild the score with `.ornament(Trill)` in
    // place of `.trill_with_extension()` to isolate the wiggle contribution.
    let no_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::HALF)
        .ornament(Ornament::Trill)
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .barline()
        .note(p(Note::E, 5), Duration::new(DurationKind::Half, 1))
        .ornament(Ornament::Trill)
        .note(p(Note::D, 5), Duration::QTR)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .ornament(Ornament::Trill)
        .barline()
        .note(p(Note::C, 5), Duration::QTR)
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .ornament(Ornament::Trill)
        .end_barline()
        .note(p(Note::F, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let no_ext_paths = no_ext.matches("<path").count();
    assert!(
        path_count > no_ext_paths,
        "extension wiggle must add paths beyond plain trill rendering: \
         with_ext={path_count}, without_ext={no_ext_paths}"
    );
}
