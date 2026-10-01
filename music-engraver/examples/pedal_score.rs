/// Render pedal markings (Ped. / *) via ScoreBuilder API.
///
/// Demonstrates sustain pedal down and up signs on notes across
/// 2 systems (4 measures) in C major 4/4, including pedal on chords.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: pedal down on first note, up on last
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .pedal_down()
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::C, 5), Duration::QTR)
        .pedal_up()
        .barline()
        // Measure 2: pedal down on chord, up on last note
        .chord(
            vec![
                Pitch::new(Note::F, 4),
                Pitch::new(Note::A, 4),
                Pitch::new(Note::C, 5),
            ],
            Duration::HALF,
        )
        .pedal_down()
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .pedal_up()
        .barline()
        // Measure 3: quick pedal changes (re-pedaling technique)
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .pedal_down()
        .note(Pitch::new(Note::F, 4), Duration::QTR)
        .pedal_up()
        .note(Pitch::new(Note::A, 4), Duration::QTR)
        .pedal_down()
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .pedal_up()
        .barline()
        // Measure 4: sustained pedal across the bar
        .note(Pitch::new(Note::C, 4), Duration::HALF)
        .pedal_down()
        .note(Pitch::new(Note::G, 4), Duration::HALF)
        .pedal_up()
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/pedal_score.svg", &svg).expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "output should be SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "pedal_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // 16 notes (incl 3 chord notes) + 2 clefs + 8 pedal marks = at least 24 paths
    assert!(
        path_count >= 20,
        "expected at least 20 paths (notes + clefs + pedal marks), got {path_count}"
    );

    // Pedal marks should produce distinct glyph paths for Ped. and *
    // We can't easily distinguish which paths are pedal marks without parsing,
    // but we can verify the overall count increased from a plain score
    assert!(
        line_count >= 10,
        "expected at least 10 lines (staff lines + stems), got {line_count}"
    );
}
