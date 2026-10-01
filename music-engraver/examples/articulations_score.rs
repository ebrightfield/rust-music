/// Render articulations via ScoreBuilder API.
///
/// Demonstrates all 6 articulation types on notes with varying stem
/// directions across 2 systems (4 measures) in C major 4/4. Also
/// shows stacked articulations (multiple on a single note).
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::articulation::Articulation;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: staccato, tenuto, accent, marcato on ascending line
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .articulation(Articulation::Staccato)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .articulation(Articulation::Tenuto)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .articulation(Articulation::Accent)
        .note(Pitch::new(Note::B, 4), Duration::QTR)
        .articulation(Articulation::Marcato)
        .barline()
        // Measure 2: staccatissimo and fermata, plus mixed stem directions
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .articulation(Articulation::Staccatissimo)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .articulation(Articulation::Fermata)
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .articulation(Articulation::Staccato)
        .barline()
        // Measure 3: staccato on low notes (stems up, articulation below)
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .articulation(Articulation::Accent)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .articulation(Articulation::Tenuto)
        .note(Pitch::new(Note::E, 4), Duration::HALF)
        .articulation(Articulation::Marcato)
        .barline()
        // Measure 4: stacked articulations — multiple per note
        .note(Pitch::new(Note::G, 5), Duration::HALF)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::Fermata)
        .note(Pitch::new(Note::F, 5), Duration::HALF)
        .articulation(Articulation::Staccato)
        .articulation(Articulation::Accent)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/articulations_score.svg",
        &svg,
    )
    .expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "output should be SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "articulations_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // Should have articulation glyphs (each articulation adds a path)
    // 14 notes + 2 clefs + 12 articulations (including 2 stacked pairs) = at least 22 paths
    assert!(
        path_count >= 22,
        "expected at least 22 paths (notes + clefs + articulations), got {path_count}"
    );
}
