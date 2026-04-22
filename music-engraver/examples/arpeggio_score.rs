/// Render arpeggios (rolled chords) via ScoreBuilder API.
///
/// Demonstrates upward and downward arpeggios on chords and single notes
/// across 2 systems (4 measures) in C major 4/4. Shows wavy line
/// placement on triads, wide voicings, and single-note arpeggios.
///
/// Produces `examples/output/arpeggio_score.svg`.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::arpeggio::ArpeggioDirection;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: upward arpeggio on C major triad, then plain quarter for contrast
        .chord(
            vec![
                Pitch::new(Note::C, 4).expect("valid pitch"),
                Pitch::new(Note::E, 4).expect("valid pitch"),
                Pitch::new(Note::G, 4).expect("valid pitch"),
            ],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Up)
        .note(Pitch::new(Note::C, 5).expect("valid pitch"), Duration::QTR)
        .note(Pitch::new(Note::G, 4).expect("valid pitch"), Duration::QTR)
        .barline()
        // Measure 2: downward arpeggio on D minor triad + arpeggio on single note
        .chord(
            vec![
                Pitch::new(Note::D, 4).expect("valid pitch"),
                Pitch::new(Note::F, 4).expect("valid pitch"),
                Pitch::new(Note::A, 4).expect("valid pitch"),
            ],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Down)
        .note(Pitch::new(Note::E, 5).expect("valid pitch"), Duration::QTR)
        .arpeggio(ArpeggioDirection::Up)
        .rest(Duration::QTR)
        .barline()
        // Measure 3: wide voicing with arpeggio (C4-G4-E5, spans ledger line territory)
        .chord(
            vec![
                Pitch::new(Note::C, 4).expect("valid pitch"),
                Pitch::new(Note::G, 4).expect("valid pitch"),
                Pitch::new(Note::E, 5).expect("valid pitch"),
            ],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Up)
        .chord(
            vec![
                Pitch::new(Note::B, 4).expect("valid pitch"),
                Pitch::new(Note::D, 5).expect("valid pitch"),
            ],
            Duration::HALF,
        )
        .arpeggio(ArpeggioDirection::Down)
        .barline()
        // Measure 4: whole-note arpeggio on 4-note chord (C-E-G-B)
        .chord(
            vec![
                Pitch::new(Note::C, 4).expect("valid pitch"),
                Pitch::new(Note::E, 4).expect("valid pitch"),
                Pitch::new(Note::G, 4).expect("valid pitch"),
                Pitch::new(Note::B, 4).expect("valid pitch"),
            ],
            Duration::WHOLE,
        )
        .arpeggio(ArpeggioDirection::Up)
        .end_barline()
        .render_svg();

    // Write output
    let output_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("output");
    std::fs::create_dir_all(&output_dir).expect("create output dir");
    let output_path = output_dir.join("arpeggio_score.svg");
    std::fs::write(&output_path, &svg).expect("write SVG");
    println!("Wrote {}", output_path.display());
    println!("Size: {} bytes", svg.len());

    // Structural assertions
    let paths = svg.matches("<path ").count();
    let lines = svg.matches("<line ").count();
    println!("Paths: {paths}, Lines: {lines}");

    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should be closed SVG");
    // 5 chords with arpeggios = 5 arpeggio paths, plus noteheads + clef
    assert!(paths >= 10, "should have ≥10 paths (arpeggios + noteheads + clef): {paths}");
    // Arpeggio transforms contain "scale(" for vertical scaling
    let scale_count = svg.matches("scale(1,").count();
    assert!(scale_count >= 5, "should have ≥5 scaled arpeggio glyphs: {scale_count}");
}
