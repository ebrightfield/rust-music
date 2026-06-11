//! Example: render a short melody using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/score_builder.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: D4 E4 F#4 G4 (quarter notes)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::Fis, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .barline()
        // Measure 2: A4 dotted half + eighth rest
        .note(
            Pitch::new(Note::A, 4),
            Duration::new(DurationKind::Half, 1),
        )
        .rest(Duration::QTR)
        .barline()
        // Measure 3: B4 A4 G4 F#4 (eighth notes)
        .note(Pitch::new(Note::B, 4), Duration::EIGHTH)
        .note(Pitch::new(Note::A, 4), Duration::EIGHTH)
        .note(Pitch::new(Note::G, 4), Duration::EIGHTH)
        .note(Pitch::new(Note::Fis, 4), Duration::EIGHTH)
        .note(Pitch::new(Note::E, 4), Duration::EIGHTH)
        .note(Pitch::new(Note::D, 4), Duration::EIGHTH)
        .rest(Duration::QTR)
        .barline()
        // Measure 4: D5 whole note with final barline
        .note(Pitch::new(Note::D, 5), Duration::WHOLE)
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("score_builder.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Basic validation
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!("SVG contains {} paths, {} lines", path_count, line_count);
}
