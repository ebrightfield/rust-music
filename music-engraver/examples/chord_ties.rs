/// Renders tied chords using the ScoreBuilder API.
/// Demonstrates ties on multi-note chords (each note gets its own tie curve).
/// Outputs SVG to `examples/output/chord_ties.svg`.
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
        // Measure 1: C major triad half, tied to same chord as half
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::HALF,
        )
        .tie()
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4)],
            Duration::HALF,
        )
        .barline()
        // Measure 2: open fifth D4-A4 quarter tied across barline, then half + quarter
        .chord(vec![p(Note::D, 4), p(Note::A, 4)], Duration::QTR)
        .chord(vec![p(Note::F, 4), p(Note::A, 4)], Duration::QTR)
        .chord(vec![p(Note::G, 4), p(Note::B, 4)], Duration::QTR)
        .tie()
        // This G-B chord ties across the barline to measure 3
        .chord(vec![p(Note::G, 4), p(Note::B, 4)], Duration::QTR)
        .barline()
        // Measure 3: G-B quarter (tie destination from measure 2), then single note ties
        .chord(vec![p(Note::G, 4), p(Note::B, 4)], Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .tie()
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .rest(Duration::QTR)
        .barline()
        // Measure 4: whole-note chord (no ties) — final measure for contrast
        .chord(
            vec![p(Note::C, 4), p(Note::E, 4), p(Note::G, 4), p(Note::C, 5)],
            Duration::WHOLE,
        )
        .end_barline()
        .render_svg();

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("chord_ties.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let tie_count = svg.matches("stroke=\"none\"").count();
    println!(
        "SVG: {} paths, {} lines, {} ties",
        path_count, line_count, tie_count
    );

    // The C major triad tie in measure 1 should produce 3 tie curves
    // The single-note E4 tie in measure 3 should produce 1 tie curve
    // Cross-barline G-B tie (measures 2→3 within same system) produces 2 ties if on same system
    // Expect at least 4 ties total (3 from triad + 1 from single note)
    assert!(
        tie_count >= 4,
        "expected at least 4 ties, got {}",
        tie_count
    );
}
