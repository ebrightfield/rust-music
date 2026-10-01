/// Render glissando lines via ScoreBuilder API.
///
/// Demonstrates both glissando styles (plain line and line-with-text) on notes
/// across 2 systems (4 measures) in C major 4/4. Glissando lines connect
/// consecutive notes with a diagonal line indicating a continuous pitch slide.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::glissando::GlissandoStyle;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: ascending glissando (plain line) C4→G4, then descending E5→B4
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 5), Duration::QTR)
        .glissando(GlissandoStyle::Line)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .barline()
        // Measure 2: ascending glissando with "gliss." text C4→A5 (wide interval)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .glissando(GlissandoStyle::LineWithText)
        .note(Pitch::new(Note::A, 5), Duration::HALF)
        .barline()
        // Measure 3: same-pitch glissando (horizontal line, rare but valid)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .glissando(GlissandoStyle::Line)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .barline()
        // Measure 4: descending with text + plain quarter for contrast
        .note(Pitch::new(Note::G, 5), Duration::QTR)
        .glissando(GlissandoStyle::LineWithText)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::HALF)
        .end_barline()
        .render_svg();

    // Write to output
    std::fs::create_dir_all("examples/output").expect("create output dir");
    std::fs::write("examples/output/glissando_score.svg", &svg).expect("write SVG");
    println!(
        "Wrote examples/output/glissando_score.svg ({} bytes)",
        svg.len()
    );

    // Structural assertions
    assert!(svg.starts_with("<svg"));
    let path_count = svg.matches("<path ").count();
    let line_count = svg.matches("<line ").count();
    println!("{path_count} paths, {line_count} lines");
    assert!(path_count >= 10, "should have noteheads+clef: {path_count}");
    assert!(
        line_count >= 15,
        "should have staff+stems+glissando lines: {line_count}"
    );

    // Glissando lines should be present (diagonal lines beyond staff lines + stems)
    // The "gliss." text should appear for LineWithText style
    assert!(
        svg.contains("gliss."),
        "LineWithText style should produce 'gliss.' label"
    );
}
