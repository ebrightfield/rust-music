/// Renders ties that cross system boundaries using the ScoreBuilder API.
/// Cross-system ties are rendered as two half-ties: a trailing arc to the
/// right edge of the source system, and an incoming arc from the left edge
/// of the target system.
/// Outputs SVG to `examples/output/cross_system_ties.svg`.
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
        .key_signature(KeySignature::Sharps(1)) // G major
        .time_signature(4, 4)
        .measures_per_system(2)
        // System 1:
        // Measure 1: quarter G4, half B4, quarter D5
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::HALF)
        .note(p(Note::D, 5), Duration::QTR)
        .barline()
        // Measure 2: half D5, quarter E5, quarter G4 tied across system break
        .note(p(Note::D, 5), Duration::HALF)
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::G, 4), Duration::QTR)
        .tie() // This tie crosses from system 1 → system 2
        .barline()
        // System 2:
        // Measure 3: quarter G4 (tie destination), quarter A4, half B4 tied across barline
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::B, 4), Duration::HALF)
        .tie() // This tie crosses barline within system 2
        .barline()
        // Measure 4: quarter B4 (tie destination), quarter A4, half G4 (final)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::A, 4), Duration::QTR)
        .note(p(Note::G, 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("cross_system_ties.svg");
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

    // Cross-system tie on G4 (system 1→2): 2 half-ties (trailing + incoming)
    // Within-system tie on B4 (measure 3→4): 1 full tie
    // Total: 3 tie curves minimum
    assert!(
        tie_count >= 3,
        "expected at least 3 ties (2 half-ties + 1 full), got {}",
        tie_count
    );
}
