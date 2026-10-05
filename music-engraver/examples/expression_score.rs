//! Example: render notes with expression text using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/expression_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::text_script::TextScript;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Flats(0)) // C major
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: lyrical opening with "dolce" + piano dynamic
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .dynamic(Dynamic::Piano)
        .text_script(TextScript::below("dolce").italic().centered())
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .barline()
        // Measure 2: expressive continuation with "espressivo"
        .note(Pitch::new(Note::C, 5), Duration::HALF)
        .text_script(TextScript::below("espressivo").italic().centered())
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .barline()
        // Measure 3: legato passage with "legato"
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .dynamic(Dynamic::Mp)
        .text_script(TextScript::below("legato").italic().centered())
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .barline()
        // Measure 4: gentle ending with "cantabile" + "morendo"
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .text_script(TextScript::below("cantabile").italic().centered())
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .text_script(TextScript::below("morendo").italic().centered())
        .rest(Duration::QTR) // expression on rest is a no-op
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("expression_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Validate structure
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let text_count = svg.matches("<text").count();
    println!(
        "SVG contains {} paths, {} lines, {} texts",
        path_count, line_count, text_count
    );
    // Expression text should appear as <text> elements
    assert!(
        text_count >= 4,
        "expected at least 4 expression text elements"
    );
    assert!(svg.contains(">dolce<"), "should contain 'dolce'");
    assert!(svg.contains(">espressivo<"), "should contain 'espressivo'");
    assert!(svg.contains(">legato<"), "should contain 'legato'");
    assert!(svg.contains(">cantabile<"), "should contain 'cantabile'");
    assert!(svg.contains(">morendo<"), "should contain 'morendo'");
    assert!(svg.contains("italic"), "expression text should be italic");
    // Notes + clef paths
    assert!(path_count > 10, "expected notehead and clef paths");
    // Staff lines + stems
    assert!(line_count > 10, "expected staff lines and stems");
}
