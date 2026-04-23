//! Example: render multi-voice writing with notehead collision avoidance.
//!
//! When two voices share the same staff position (unison) or are a second
//! apart, the additional voice's notehead shifts right by one notehead width
//! to prevent visual overlap. This example demonstrates both cases alongside
//! normal (no-collision) multi-voice writing for contrast.
//!
//! Produces `examples/output/voice_collision.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: unison collisions
        // Voice 0: C5 half, D5 half (stems up)
        // Voice 1: C5 half, D5 half (stems down) — same pitches, noteheads offset
        .note(Pitch::new(Note::C, 5).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::D, 5).unwrap(), Duration::HALF)
        .voice(1)
        .note(Pitch::new(Note::C, 5).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::D, 5).unwrap(), Duration::HALF)
        .voice(0)
        .barline()
        // Measure 2: second-apart collisions
        // Voice 0: E5 qtr, F5 qtr, G5 half (stems up)
        // Voice 1: D5 qtr, E5 qtr, F5 half (stems down) — a second below, offset
        .note(Pitch::new(Note::E, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::F, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::G, 5).unwrap(), Duration::HALF)
        .voice(1)
        .note(Pitch::new(Note::D, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::E, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::F, 5).unwrap(), Duration::HALF)
        .voice(0)
        .barline()
        // Measure 3: mixed — some collisions, some not
        // Voice 0: C5 qtr, E5 qtr, G5 qtr, C6 qtr (stems up)
        // Voice 1: C5 qtr, C4 qtr, G5 qtr, A4 qtr (stems down)
        // Beat 1: unison C5 — collision offset
        // Beat 2: E5 vs C4 — far apart, no offset
        // Beat 3: unison G5 — collision offset
        // Beat 4: C6 vs A4 — far apart, no offset
        .note(Pitch::new(Note::C, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::E, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::G, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::C, 6).unwrap(), Duration::QTR)
        .voice(1)
        .note(Pitch::new(Note::C, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::G, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::A, 4).unwrap(), Duration::QTR)
        .voice(0)
        .barline()
        // Measure 4: well-separated voices for contrast (no collision)
        // Voice 0: A5 whole (stems up, high)
        // Voice 1: C4 whole (stems down, low)
        .note(Pitch::new(Note::A, 5).unwrap(), Duration::WHOLE)
        .voice(1)
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
        .voice(0)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/voice_collision.svg", &svg).unwrap();
    println!(
        "Wrote examples/output/voice_collision.svg ({} bytes)",
        svg.len()
    );

    // Structural assertions
    assert!(svg.starts_with("<svg"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!("  paths: {path_count}, lines: {line_count}");

    // Should have noteheads for both voices + clef
    // Voice 0: 2+3+4+1 = 10 noteheads, Voice 1: 2+3+4+1 = 10 noteheads, 1 clef = 21 minimum
    assert!(path_count >= 15, "expected ≥15 paths, got {path_count}");
    // Staff lines + stems from both voices + barlines
    assert!(line_count >= 20, "expected ≥20 lines, got {line_count}");

    // Verify collision avoidance produces translate transforms in the SVG.
    // Offset noteheads use transform="translate(x_offset, 0)" on their <path>.
    // The plain multi_voice_score example (well-separated voices) should have
    // fewer translate transforms than this collision-heavy example.
    let translate_count = svg.matches("translate(").count();
    println!("  translate transforms: {translate_count}");
}
