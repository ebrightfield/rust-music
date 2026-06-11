/// Render breath marks via ScoreBuilder API.
///
/// Demonstrates all 3 breath mark types (comma, tick, caesura) on notes
/// across 2 systems (4 measures) in C major 4/4. Breath marks indicate
/// a brief pause and are placed above the staff, to the right of the note.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::breath::BreathMark;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: comma breaths between ascending notes
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .breath_mark(BreathMark::Comma)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .breath_mark(BreathMark::Comma)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .barline()
        // Measure 2: tick breath + caesura on descending notes
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .breath_mark(BreathMark::Tick)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .breath_mark(BreathMark::Caesura)
        .barline()
        // Measure 3: mixed breath marks on low notes (stems up)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .breath_mark(BreathMark::Comma)
        .note(Pitch::new(Note::D, 4), Duration::HALF)
        .breath_mark(BreathMark::Tick)
        .barline()
        // Measure 4: caesura for a dramatic pause, then plain notes
        .note(Pitch::new(Note::E, 5), Duration::QTR)
        .breath_mark(BreathMark::Caesura)
        .rest(Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/breath_marks_score.svg",
        &svg,
    )
    .expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "output should be SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "breath_marks_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // 12 notes + 2 clefs + 7 breath marks = at least 21 paths
    assert!(
        path_count >= 21,
        "expected at least 21 paths (notes + clefs + breath marks), got {path_count}"
    );

    // Verify breath mark translate transforms are present
    let translate_count = svg.matches("translate(").count();
    assert!(
        translate_count >= 7,
        "expected at least 7 translate transforms for breath marks, got {translate_count}"
    );
}
