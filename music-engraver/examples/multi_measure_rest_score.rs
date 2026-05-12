//! Example: multi-measure rests inside a player part.
//!
//! Renders a four-measure score that mixes a notated opening, a compressed
//! 8-measure rest (typical of an orchestra part where the player is tacet
//! for several bars), another notated measure, and a closing 16-measure rest.
//! The H-bar plus count number is engraving convention for empty passages.
//!
//! Produces `examples/output/multi_measure_rest_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(4)
        // Measure 1: notated entry
        .note(Pitch::new(Note::C, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::D, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::E, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::F, 5).unwrap(), Duration::QTR)
        .barline()
        // Measures 2-9 compressed into one frame: tacet 8 bars
        .multi_measure_rest(8)
        .barline()
        // Measure 10: re-entry
        .note(Pitch::new(Note::G, 5).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::E, 5).unwrap(), Duration::HALF)
        .barline()
        // Measures 11-26 compressed into one frame: tacet 16 bars
        .multi_measure_rest(16)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/multi_measure_rest_score.svg", &svg).unwrap();
    println!(
        "Wrote examples/output/multi_measure_rest_score.svg ({} bytes)",
        svg.len()
    );

    // Structural sanity checks
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains(">8</text>"), "should contain count '8'");
    assert!(svg.contains(">16</text>"), "should contain count '16'");

    let rect_count = svg.matches("<rect").count();
    // Each multi-measure rest contributes 3 rects (2 serifs + crossbar) = 6 minimum.
    // Other elements (e.g. rehearsal marks) would add boxed rects, but this score has none.
    assert!(
        rect_count >= 6,
        "expected ≥6 rects for two H-bars (2×3), got {rect_count}",
    );

    let path_count = svg.matches("<path").count();
    println!("  paths: {path_count}, rects: {rect_count}");
}
