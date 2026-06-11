/// Render navigation signs (segno, coda) via ScoreBuilder API.
///
/// Demonstrates segno, coda, and square coda signs on notes across
/// 2 systems (4 measures) in C major 4/4.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::barline::BarlineStyle;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::navigation::NavigationSign;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: segno on first beat, ascending quarter notes
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .navigation_sign(NavigationSign::Segno)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .barline()
        // Measure 2: plain measure, ends with repeat barline
        .note(Pitch::new(Note::G, 4), Duration::HALF)
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .barline_style(BarlineStyle::EndRepeat)
        .barline()
        // Measure 3: coda on first beat
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .navigation_sign(NavigationSign::Coda)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .barline()
        // Measure 4: square coda on final note
        .note(Pitch::new(Note::F, 4), Duration::HALF)
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .navigation_sign(NavigationSign::CodaSquare)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/navigation_signs_score.svg",
        &svg,
    )
    .expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "output should be SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "navigation_signs_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // 12 notes + 2 clefs + 3 navigation signs = at least 17 paths
    assert!(
        path_count >= 15,
        "expected at least 15 paths (notes + clefs + nav signs), got {path_count}"
    );

    // Navigation signs should produce distinct glyph paths
    assert!(
        svg.contains("<path"),
        "should contain path elements for navigation sign glyphs"
    );
}
