/// Render tremolo notation via ScoreBuilder API.
///
/// Demonstrates all 3 tremolo counts (single, double, triple) on notes
/// with varying durations and stem directions across 2 systems (4 measures)
/// in C major 4/4.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::tremolo::TremoloCount;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: single tremolo on quarter notes (ascending)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .tremolo(TremoloCount::Single)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .tremolo(TremoloCount::Single)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .tremolo(TremoloCount::Double)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .tremolo(TremoloCount::Triple)
        .barline()
        // Measure 2: tremolo on half notes + varied registers
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .tremolo(TremoloCount::Single)
        .note(Pitch::new(Note::A, 3), Duration::HALF)
        .tremolo(TremoloCount::Double)
        .barline()
        // Measure 3: mix of tremolo and non-tremolo
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .tremolo(TremoloCount::Triple)
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .tremolo(TremoloCount::Single)
        .barline()
        // Measure 4: high register tremolo (stems down)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .tremolo(TremoloCount::Double)
        .note(Pitch::new(Note::E, 5), Duration::QTR)
        .tremolo(TremoloCount::Single)
        .note(Pitch::new(Note::G, 5), Duration::HALF)
        .tremolo(TremoloCount::Triple)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/tremolo_score.svg", &svg).expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "output should be SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "tremolo_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // 13 notes + 2 clefs + 10 tremolo glyphs = at least 22 paths
    // (note without tremolo doesn't add a tremolo path)
    assert!(
        path_count >= 22,
        "expected at least 22 paths (notes + clefs + tremolo glyphs), got {path_count}"
    );

    // Staff lines (5 per system × 2) + stems (13 notes) = at least 20 lines
    assert!(
        line_count >= 20,
        "expected at least 20 lines (staff + stems), got {line_count}"
    );
}
