/// Renders slurs that cross system boundaries using the ScoreBuilder API.
/// Cross-system slurs are rendered as two half-slurs: a trailing arc to the
/// right edge of the source system, and an incoming arc from the left edge
/// of the target system.
/// Outputs SVG to `examples/output/cross_system_slurs.svg`.
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
        .key_signature(KeySignature::Sharps(0)) // C major
        .time_signature(4, 4)
        .measures_per_system(2)
        // System 1:
        // Measure 1: quarter C4, half E4, quarter G4 (ascending phrase)
        .note(p(Note::C, 4), Duration::QTR)
        .note(p(Note::E, 4), Duration::HALF)
        .note(p(Note::G, 4), Duration::QTR)
        .barline()
        // Measure 2: half A4, quarter B4, quarter C5 — slur starts on C5
        // and crosses the system break to D5 in measure 3
        .note(p(Note::A, 4), Duration::HALF)
        .note(p(Note::B, 4), Duration::QTR)
        .note(p(Note::C, 5), Duration::QTR)
        .slur_start() // slur crosses system 1 → system 2
        .barline()
        // System 2:
        // Measure 3: quarter D5 (slur destination), quarter E5, half F5
        // plus a within-system slur on F5→E5→D5 across barline
        .note(p(Note::D, 5), Duration::QTR)
        .slur_end()
        .note(p(Note::E, 5), Duration::QTR)
        .note(p(Note::F, 5), Duration::QTR)
        .slur_start() // within-system cross-barline slur
        .note(p(Note::E, 5), Duration::QTR)
        .barline()
        // Measure 4: quarter D5 (slur end), quarter C5, half G4
        .note(p(Note::D, 5), Duration::QTR)
        .slur_end()
        .note(p(Note::C, 5), Duration::QTR)
        .note(p(Note::G, 4), Duration::HALF)
        .end_barline()
        .render_svg();

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("cross_system_slurs.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let slur_count = svg.matches("stroke=\"none\"").count();
    println!(
        "SVG: {} paths, {} lines, {} slurs (filled paths)",
        path_count, line_count, slur_count
    );

    // Cross-system slur on C5→D5 (system 1→2): 2 half-slurs (trailing + incoming)
    // Within-system cross-barline slur F5→D5 (measure 3→4): 1 full slur
    // Total: at least 3 slur curves
    assert!(
        slur_count >= 3,
        "expected at least 3 slurs (2 half-slurs + 1 full), got {}",
        slur_count
    );
}
