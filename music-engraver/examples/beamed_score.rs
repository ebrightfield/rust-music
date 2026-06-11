//! Example: render beamed note groups using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/beamed_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::{Duration, DurationKind};
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(1)) // G major
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: four beamed eighth notes + quarter + quarter rest
        .beam_group(vec![
            (Pitch::new(Note::G, 4), Duration::EIGHTH),
            (Pitch::new(Note::A, 4), Duration::EIGHTH),
            (Pitch::new(Note::B, 4), Duration::EIGHTH),
            (Pitch::new(Note::C, 5), Duration::EIGHTH),
        ])
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .rest(Duration::QTR)
        .barline()
        // Measure 2: two beamed sixteenths + eighth rest + beamed descending eighths + half
        .beam_group(vec![
            (Pitch::new(Note::E, 5), Duration::SIXTEENTH),
            (Pitch::new(Note::D, 5), Duration::SIXTEENTH),
        ])
        .rest(Duration::EIGHTH)
        .beam_group(vec![
            (Pitch::new(Note::C, 5), Duration::EIGHTH),
            (Pitch::new(Note::B, 4), Duration::EIGHTH),
        ])
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .barline()
        // Measure 3: beamed sixteenth-note flourish + dotted half note
        .beam_group(vec![
            (Pitch::new(Note::G, 4), Duration::SIXTEENTH),
            (Pitch::new(Note::A, 4), Duration::SIXTEENTH),
            (Pitch::new(Note::B, 4), Duration::SIXTEENTH),
            (Pitch::new(Note::C, 5), Duration::SIXTEENTH),
        ])
        .note(
            Pitch::new(Note::D, 5),
            Duration::new(DurationKind::Half, 1),
        )
        .barline()
        // Measure 4: mixed beam group (eighth + two sixteenths) + quarter + quarter
        .beam_group(vec![
            (Pitch::new(Note::D, 5), Duration::EIGHTH),
            (Pitch::new(Note::C, 5), Duration::SIXTEENTH),
            (Pitch::new(Note::B, 4), Duration::SIXTEENTH),
        ])
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("beamed_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Validate structure
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let polygon_count = svg.matches("<polygon").count();
    println!(
        "SVG contains {} paths, {} lines, {} polygons",
        path_count, line_count, polygon_count
    );
    // Must have beam polygons
    assert!(polygon_count > 0, "expected beam polygons in output");
    // Must have noteheads (paths for glyph outlines)
    assert!(path_count > 5, "expected multiple notehead paths");
}
