//! Example: render tuplet groups using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/tuplet_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(0)) // C major
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: eighth-note triplet (3 in the time of 2) + quarter + quarter rest
        .tuplet(
            3,
            vec![
                (Pitch::new(Note::C, 4), Duration::EIGHTH),
                (Pitch::new(Note::E, 4), Duration::EIGHTH),
                (Pitch::new(Note::G, 4), Duration::EIGHTH),
            ],
        )
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .rest(Duration::QTR)
        .barline()
        // Measure 2: quarter-note triplet filling 2 beats + half note
        .tuplet(
            3,
            vec![
                (Pitch::new(Note::D, 5), Duration::EIGHTH),
                (Pitch::new(Note::C, 5), Duration::EIGHTH),
                (Pitch::new(Note::B, 4), Duration::EIGHTH),
            ],
        )
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .barline()
        // Measure 3: sixteenth-note quintuplet + dotted quarter + eighth
        .tuplet(
            5,
            vec![
                (Pitch::new(Note::G, 4), Duration::SIXTEENTH),
                (Pitch::new(Note::A, 4), Duration::SIXTEENTH),
                (Pitch::new(Note::B, 4), Duration::SIXTEENTH),
                (Pitch::new(Note::C, 5), Duration::SIXTEENTH),
                (Pitch::new(Note::D, 5), Duration::SIXTEENTH),
            ],
        )
        .note(
            Pitch::new(Note::E, 5),
            Duration::new(music::notation::rhythm::duration::DurationKind::Qtr, 1),
        )
        .note(Pitch::new(Note::D, 5), Duration::EIGHTH)
        .barline()
        // Measure 4: two separate triplets filling the bar
        .tuplet(
            3,
            vec![
                (Pitch::new(Note::C, 5), Duration::EIGHTH),
                (Pitch::new(Note::B, 4), Duration::EIGHTH),
                (Pitch::new(Note::A, 4), Duration::EIGHTH),
            ],
        )
        .tuplet(
            3,
            vec![
                (Pitch::new(Note::G, 4), Duration::EIGHTH),
                (Pitch::new(Note::F, 4), Duration::EIGHTH),
                (Pitch::new(Note::E, 4), Duration::EIGHTH),
            ],
        )
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("tuplet_score.svg");
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
    // Must have beam polygons (tuplets use beamed notes)
    assert!(polygon_count > 0, "expected beam polygons in output");
    // Must have tuplet bracket lines
    assert!(line_count > 10, "expected staff + stem + bracket lines");
    // Must have noteheads + tuplet digit glyphs
    assert!(path_count > 10, "expected notehead and tuplet number paths");
}
