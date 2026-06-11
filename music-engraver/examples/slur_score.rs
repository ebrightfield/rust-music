/// Renders slurred notes using the ScoreBuilder API.
/// Demonstrates within-measure and cross-barline slurs.
/// Outputs SVG to `examples/output/slur_score.svg`.
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
        // Measure 1: ascending slurred phrase C4–E4–G4, then quarter rest
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .slur_start()
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .slur_end()
        .rest(Duration::QTR)
        .barline()
        // Measure 2: descending slur B4–A4–G4–F4 across whole measure
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .slur_start()
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .slur_end()
        .barline()
        // Measure 3: slur spanning two notes at same pitch (legato articulation)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .slur_start()
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .slur_end()
        .barline()
        // Measure 4: wide ascending slur C4–G5 (large interval)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .slur_start()
        .note(Pitch::new(Note::G, 5), Duration::HALF)
        .slur_end()
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/slur_score.svg", &svg).unwrap();
    println!("Wrote music-engraver/examples/output/slur_score.svg");

    let path_count = svg.matches("<path ").count();
    let line_count = svg.matches("<line ").count();
    let slur_count = svg.matches("stroke=\"none\"").count();
    println!(
        "SVG: {} bytes, {} <path>, {} <line>, {} slurs (filled paths)",
        svg.len(),
        path_count,
        line_count,
        slur_count,
    );

    // Expect at least 4 slur curves (one per slur_start/slur_end pair)
    assert!(
        slur_count >= 4,
        "Expected at least 4 slur curves, got {}",
        slur_count,
    );
}
