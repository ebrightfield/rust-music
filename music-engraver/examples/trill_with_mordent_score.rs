//! Example: render notes with the precomposed trill-with-mordent compound
//! ornament plus a wavy-line extension, via
//! `ScoreBuilder::trill_with_mordent_with_extension()`.
//!
//! The precomposed glyph (`OrnamentPrecompTrillWithMordent`) reads as
//! "trill, then a mordent at the end" — a single compact mark for an
//! ornament that historically would have been written as two stacked
//! glyphs. With an extension, the wiggle starts past the *full* compound
//! glyph (not just the "tr" prefix), so the mordent suffix remains
//! visually intact.
//!
//! Common in Baroque keyboard music (Couperin, J. S. Bach's keyboard
//! works). Produces `examples/output/trill_with_mordent_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // M1: half-note compound-trill with extension, then quarter notes
        .note(p(Note::G, 4), Duration::HALF)
        .trill_with_mordent_with_extension()
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .barline()
        // M2: dotted-half compound-trill with extension over a long span
        .note(p(Note::E, 5), Duration::new(DurationKind::Half, 1))
        .trill_with_mordent_with_extension()
        .note(p(Note::D, 5), Duration::QTR)
        .barline()
        // M3: whole-note compound-trill with extension across the whole
        // measure. The next note is in M4 (within the same system) so the
        // wiggle runs to that note's left edge.
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_mordent_with_extension()
        .barline()
        // M4: chord compound-trill at end of system
        .note(p(Note::C, 5), Duration::QTR)
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_mordent_with_extension()
        .end_barline()
        // M5: terminating note. The chord compound-trill at the end of M4
        // extends its wiggle to the right edge of its system per the
        // cross-system convention (an incoming wiggle then resumes on the
        // next system leading up to this F4).
        .note(p(Note::F, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let path = out_dir.join("trill_with_mordent_score.svg");
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

    // Validate basic SVG structure
    assert!(svg.starts_with("<svg"), "output should be valid SVG");
    assert!(svg.contains("</svg>"), "output should have closing SVG tag");
    assert!(
        path_count >= 18,
        "expected at least 18 paths (noteheads + clef + compound-trills + wiggles), got {path_count}"
    );

    // Compare against the same score with plain `Ornament::TrillWithMordent`
    // (no extension flag). The wiggle must add paths.
    let no_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::HALF)
        .ornament(Ornament::TrillWithMordent)
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .barline()
        .note(p(Note::E, 5), Duration::new(DurationKind::Half, 1))
        .ornament(Ornament::TrillWithMordent)
        .note(p(Note::D, 5), Duration::QTR)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .ornament(Ornament::TrillWithMordent)
        .barline()
        .note(p(Note::C, 5), Duration::QTR)
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .ornament(Ornament::TrillWithMordent)
        .end_barline()
        .note(p(Note::F, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let no_ext_paths = no_ext.matches("<path").count();
    assert!(
        path_count > no_ext_paths,
        "extension wiggle must add paths beyond plain compound-trill rendering: \
         with_ext={path_count}, without_ext={no_ext_paths}"
    );

    // The compound ornament glyph is wider than the bare "tr", so the
    // wiggle on each note starts further right. Compared to a plain-trill
    // extension over the same musical content, this score should differ
    // byte-for-byte — different prefix glyph, different wiggle start.
    let trill_ext = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        .note(p(Note::G, 4), Duration::HALF)
        .trill_with_extension()
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .barline()
        .note(p(Note::E, 5), Duration::new(DurationKind::Half, 1))
        .trill_with_extension()
        .note(p(Note::D, 5), Duration::QTR)
        .barline()
        .note(p(Note::B, 4), Duration::WHOLE)
        .trill_with_extension()
        .barline()
        .note(p(Note::C, 5), Duration::QTR)
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::new(DurationKind::Half, 1),
        )
        .trill_with_extension()
        .end_barline()
        .note(p(Note::F, 4), Duration::WHOLE)
        .end_barline()
        .render_svg();

    assert_ne!(
        svg, trill_ext,
        "compound-trill+ext and plain-trill+ext must differ — different prefix glyph, \
         different wiggle start positions"
    );
}
