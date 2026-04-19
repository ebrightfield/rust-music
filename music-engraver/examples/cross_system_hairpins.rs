/// Renders hairpins (crescendo/decrescendo wedges) that cross system boundaries
/// using the ScoreBuilder API.
/// Cross-system hairpins are rendered as two half-wedges: a trailing wedge to the
/// right edge of the source system, and an incoming wedge from the left edge
/// of the target system.
/// Outputs SVG to `examples/output/cross_system_hairpins.svg`.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn p(note: Note, oct: u8) -> Pitch {
    Pitch::new(note, oct).unwrap()
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(0)) // C major
        .time_signature(4, 4)
        .measures_per_system(2)
        // System 1:
        // Measure 1: ascending phrase with crescendo starting
        .note(p(Note::C, 4), Duration::QTR)
        .dynamic(Dynamic::Pp)
        .cresc()
        .note(p(Note::D, 4), Duration::QTR)
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::F, 4), Duration::QTR)
        .barline()
        // Measure 2: crescendo continues, crossing system break
        .note(p(Note::G, 4), Duration::HALF)
        .note(p(Note::A, 4), Duration::HALF)
        .barline()
        // System 2:
        // Measure 3: crescendo ends, then decrescendo starts
        .note(p(Note::B, 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Ff)
        .note(p(Note::A, 4), Duration::QTR)
        .decresc()
        .note(p(Note::G, 4), Duration::QTR)
        .note(p(Note::F, 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Piano)
        .barline()
        // Measure 4: final phrase
        .note(p(Note::E, 4), Duration::QTR)
        .note(p(Note::D, 4), Duration::QTR)
        .note(p(Note::C, 4), Duration::HALF)
        .dynamic(Dynamic::Pp)
        .end_barline()
        .render_svg();

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("cross_system_hairpins.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "SVG: {} paths, {} lines",
        path_count, line_count
    );

    // 12 notes + 1 clef × 2 systems + 5 dynamics = ~19 paths
    assert!(path_count > 15, "expected paths for notes, clefs, dynamics: got {}", path_count);
    // 5 staff lines × 2 systems + stems + hairpin wedge lines
    // Cross-system cresc: 2 half-hairpins × 2 lines each = 4 lines
    // Within-system decresc: 1 hairpin × 2 lines = 2 lines
    // Staff: 10, stems: ~10, hairpins: ~6 = ~26
    assert!(line_count > 20, "expected staff lines + stems + hairpin wedges: got {}", line_count);

    println!("Cross-system hairpins rendered successfully.");
}
