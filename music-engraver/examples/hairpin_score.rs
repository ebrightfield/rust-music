/// Renders notes with hairpin (crescendo/decrescendo) wedges using the ScoreBuilder API.
/// Demonstrates crescendo and decrescendo markings across notes and barlines.
/// Outputs SVG to `examples/output/hairpin_score.svg`.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(0)) // C major
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: ascending cresc from p to f
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .cresc()
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Forte)
        .barline()
        // Measure 2: held note with decresc
        .note(Pitch::new(Note::G, 4), Duration::HALF)
        .dynamic(Dynamic::Ff)
        .decresc()
        .note(Pitch::new(Note::E, 4), Duration::HALF)
        .hairpin_end()
        .dynamic(Dynamic::Piano)
        .barline()
        // Measure 3: cresc spanning barline into measure 4
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .dynamic(Dynamic::Pp)
        .cresc()
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::F, 4), Duration::HALF)
        .barline()
        // Measure 4: hairpin continues, then decresc to pp
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Fff)
        .decresc()
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .hairpin_end()
        .dynamic(Dynamic::Pp)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("hairpin_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!("SVG contains {} paths, {} lines", path_count, line_count);

    // Expect noteheads + clef + dynamics + hairpin lines
    // 12 notes + 1 clef + 8 dynamics = ~21 paths
    assert!(
        path_count > 15,
        "expected paths for notes, clef, dynamics: got {}",
        path_count
    );
    // 5 staff lines per system × 2 systems + stems + hairpin wedge lines
    // Each hairpin = 2 lines, 4 hairpins = 8 lines
    // Staff: 10, stems: ~12, hairpins: ~8 = ~30
    assert!(
        line_count > 20,
        "expected staff lines + stems + hairpin wedges: got {}",
        line_count
    );

    // Hairpins are drawn as <line> elements — count lines that are not staff lines or stems
    // (can't easily distinguish, but at least verify we have enough)
    println!("Hairpin score rendered successfully.");
}
