/// Renders tied notes using the ScoreBuilder API.
/// Demonstrates within-measure and cross-barline ties.
/// Outputs SVG to `examples/output/tied_score.svg`.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
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
        // Measure 1: quarter G4, half D5 tied to...
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .tie()
        // ...quarter D5 (same pitch, completes the tie within the measure)
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .barline()
        // Measure 2: quarter notes, last G4 tied across barline
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .tie()
        .barline()
        // Measure 3: ...quarter G4 (cross-barline tie destination), then more notes
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .tie()
        .barline()
        // Measure 4: C5 tied from previous, then finish
        .note(Pitch::new(Note::C, 5), Duration::HALF)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/tied_score.svg", &svg).unwrap();
    println!("Wrote music-engraver/examples/output/tied_score.svg");

    let path_count = svg.matches("<path ").count();
    let line_count = svg.matches("<line ").count();
    let tie_count = svg.matches("stroke=\"none\"").count();
    println!(
        "SVG: {} bytes, {} <path>, {} <line>, {} ties",
        svg.len(),
        path_count,
        line_count,
        tie_count,
    );
}
