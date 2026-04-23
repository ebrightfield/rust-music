//! Example: render multi-voice writing using the high-level ScoreBuilder API.
//!
//! Demonstrates two independent voices on a single staff:
//! voice 0 (melody, stems up) and voice 1 (bass, stems down).
//!
//! Produces `examples/output/multi_voice_score.svg`.

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
        // Measure 1: soprano melody (voice 0) + alto accompaniment (voice 1)
        // Voice 0: melody — E5 half, D5 half (stems up)
        .note(Pitch::new(Note::E, 5).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::D, 5).unwrap(), Duration::HALF)
        // Voice 1: bass — C4 whole (stems down)
        .voice(1)
        .note(Pitch::new(Note::C, 4).unwrap(), Duration::WHOLE)
        .voice(0)
        .barline()
        // Measure 2: interleaved rhythm — voice 0 quarters, voice 1 half notes
        // Voice 0: G5 qtr, F5 qtr, E5 qtr, D5 qtr
        .note(Pitch::new(Note::G, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::F, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::E, 5).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::D, 5).unwrap(), Duration::QTR)
        // Voice 1: E4 half, D4 half
        .voice(1)
        .note(Pitch::new(Note::E, 4).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::D, 4).unwrap(), Duration::HALF)
        .voice(0)
        .barline()
        // Measure 3: voice 0 half + 2 quarters, voice 1 quarter rest + half + quarter
        .note(Pitch::new(Note::C, 5).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::B, 4).unwrap(), Duration::QTR)
        .note(Pitch::new(Note::A, 4).unwrap(), Duration::QTR)
        .voice(1)
        .rest(Duration::QTR)
        .note(Pitch::new(Note::G, 3).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::A, 3).unwrap(), Duration::QTR)
        .voice(0)
        .barline()
        // Measure 4: converging voices — both end on unison C5/C4
        .note(Pitch::new(Note::A, 4).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::G, 4).unwrap(), Duration::HALF)
        .voice(1)
        .note(Pitch::new(Note::E, 3).unwrap(), Duration::HALF)
        .note(Pitch::new(Note::F, 3).unwrap(), Duration::HALF)
        .voice(0)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/multi_voice_score.svg", &svg).unwrap();
    println!(
        "Wrote examples/output/multi_voice_score.svg ({} bytes)",
        svg.len()
    );

    // Structural assertions
    assert!(svg.starts_with("<svg"));
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!("  paths: {path_count}, lines: {line_count}");

    // Multi-voice should produce more elements than a single-voice score
    // At minimum: clef + noteheads for voice 0 + noteheads for voice 1 + dots
    assert!(path_count >= 15, "expected ≥15 paths, got {path_count}");
    // Staff lines + stems for both voices
    assert!(line_count >= 20, "expected ≥20 lines, got {line_count}");
}
