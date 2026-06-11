//! Example: render various chord types on a treble staff using ScoreBuilder.
//!
//! Produces `examples/output/chords.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, oct: i8) -> Pitch {
    Pitch::new(note, oct)
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: open fifth C4-G4 quarter, C major triad quarter, D minor triad quarter, quarter rest
        .chord(vec![p(Note::C, 4), p(Note::G, 4)], Duration::QTR)
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::QTR,
        )
        .chord(
            vec![p(Note::D, 4), p(Note::F, 4), p(Note::A, 4)],
            Duration::QTR,
        )
        .rest(Duration::QTR)
        .barline()
        // Measure 2: second interval E4-F4 half (tests notehead offset), G major triad half
        .chord(vec![p(Note::E, 4), p(Note::F, 4)], Duration::HALF)
        .chord(
            vec![p(Note::G, 3), p(Note::B, 3), p(Note::D, 4)],
            Duration::HALF,
        )
        .barline()
        // Measure 3: cluster C4-D4-E4 quarter (adjacent seconds), Bb major with accidental
        .chord(
            vec![p(Note::C, 4), p(Note::D, 4), p(Note::E, 4)],
            Duration::QTR,
        )
        .chord(
            vec![p(Note::Bes, 3), p(Note::D, 4), p(Note::F, 4)],
            Duration::QTR,
        )
        // Wide voicing: C4-G4-E5
        .chord(
            vec![p(Note::C, 4), p(Note::G, 4), p(Note::E, 5)],
            Duration::QTR,
        )
        .rest(Duration::QTR)
        .barline()
        // Measure 4: whole-note chord (no stem), then final barline
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4), p(Note::C, 5)],
            Duration::WHOLE,
        )
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("chords.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Basic validation
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!("SVG contains {} paths, {} lines", path_count, line_count);

    // Should have multiple noteheads per chord — more paths than a single-note melody
    assert!(path_count > 20, "expected many paths for chords, got {}", path_count);
}
