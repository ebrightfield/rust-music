//! Example: render notes with ornament markings using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/ornaments_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ornament::Ornament;
use music_engraver::score::ScoreBuilder;

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: trill on E4, mordent on F4, turn on G4, inverted turn on A4
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .ornament(Ornament::Trill)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .ornament(Ornament::Mordent)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .ornament(Ornament::Turn)
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .ornament(Ornament::InvertedTurn)
        .barline()
        // Measure 2: inverted mordent, turn with slash, short trill, plain quarter
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .ornament(Ornament::InvertedMordent)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .ornament(Ornament::TurnSlash)
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .ornament(Ornament::ShortTrill)
        .note(Pitch::new(Note::E, 5), Duration::QTR)
        .barline()
        // Measure 3: trill on high note (above staff), trill on chord
        .note(Pitch::new(Note::A, 5), Duration::HALF)
        .ornament(Ornament::Trill)
        .chord(
            vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ],
            Duration::HALF,
        )
        .ornament(Ornament::Turn)
        .barline()
        // Measure 4: whole note with mordent
        .note(Pitch::new(Note::G, 4), Duration::WHOLE)
        .ornament(Ornament::Mordent)
        .end_barline()
        .render_svg();

    let path = out_dir.join("ornaments_score.svg");
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
    // We should have at least 12 noteheads + 1 clef + ornament paths
    assert!(
        path_count >= 15,
        "expected at least 15 paths, got {path_count}"
    );
}
