/// Renders notes with volta brackets (1st/2nd ending) using the ScoreBuilder API.
/// Demonstrates single-measure voltas, multi-measure voltas, and combined with
/// repeat barlines.
/// Outputs SVG to `examples/output/volta_brackets_score.svg`.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::barline::BarlineStyle;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(0)) // C major
        .time_signature(4, 4)
        .measures_per_system(4)
        // Measure 1: plain opening
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
        .barline_style(BarlineStyle::StartRepeat)
        // Measure 2: single-measure 1st ending (both hooks)
        .volta_start("1.")
        .note(Pitch::new(Note::G, 4).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::E, 4).unwrap(), Duration::HALF)
        .volta_end()
        .barline_style(BarlineStyle::EndRepeat)
        // Measure 3: single-measure 2nd ending (both hooks)
        .volta_start("2.")
        .note(Pitch::new(Note::A, 4).unwrap(), Duration::WHOLE)
        .volta_end()
        .barline()
        // Measure 4: start of multi-measure 3rd ending
        .volta_start("3.")
        .note(Pitch::new(Note::B, 4).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::A, 4).unwrap(), Duration::HALF)
        .barline()
        // Measure 5 (system 2): continuation of multi-measure volta
        .note(Pitch::new(Note::G, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::F, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::E, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::D, 4).unwrap(), Duration::QTR)
        .volta_end()
        .barline()
        // Measure 6: plain closing
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
        .end_barline()
        .render_svg();

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("volta_brackets_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Structural assertions
    let line_count = svg.matches("<line").count();
    let path_count = svg.matches("<path").count();
    let text_count = svg.matches("<text").count();
    println!(
        "Element counts: {} paths, {} lines, {} texts",
        path_count, line_count, text_count
    );

    // Volta bracket text labels should appear
    assert!(svg.contains(">1.<"), "should contain '1.' volta label");
    assert!(svg.contains(">2.<"), "should contain '2.' volta label");
    assert!(svg.contains(">3.<"), "should contain '3.' volta label");

    // Should have at least some lines from volta brackets (hooks + top lines)
    assert!(line_count >= 15, "expected at least 15 lines (staff + stems + volta brackets)");
    assert!(text_count >= 3, "expected at least 3 texts (volta labels)");
}
