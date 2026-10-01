//! Grand staff (piano) rendering via MultiStaffScore.
//!
//! Renders a 2-measure piano score with treble and bass clef staves
//! connected by a brace and joined barlines.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::multi_staff::MultiStaffScore;
use music_engraver::score::ScoreBuilder;

fn main() {
    let treble = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        // Measure 1: ascending quarter notes
        .note(Pitch::new(Note::D, 5), Duration::QTR)
        .note(Pitch::new(Note::E, 5), Duration::QTR)
        .note(Pitch::new(Note::Fis, 5), Duration::QTR)
        .note(Pitch::new(Note::G, 5), Duration::QTR)
        .barline()
        // Measure 2: half notes
        .note(Pitch::new(Note::A, 5), Duration::HALF)
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .end_barline();

    let bass = ScoreBuilder::new()
        .clef(Clef::Bass)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        // Measure 1: whole note
        .note(Pitch::new(Note::D, 3), Duration::WHOLE)
        .barline()
        // Measure 2: half notes
        .note(Pitch::new(Note::A, 2), Duration::HALF)
        .note(Pitch::new(Note::D, 3), Duration::HALF)
        .end_barline();

    let svg = MultiStaffScore::grand_staff(treble, bass).render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/grand_staff_score.svg", &svg).unwrap();

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "Grand staff score: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count,
    );

    // Verify structural elements
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(
        svg.contains("scale(1,"),
        "should contain brace glyph with scale"
    );
    // 2 staves × 5 lines = 10 staff lines minimum
    assert!(
        line_count >= 10,
        "need at least 10 staff lines, got {line_count}"
    );
    // At least: 2 clefs + 2 key sigs (2 sharps each = 4 paths) + 2 time sigs (num+denom each = 4 paths)
    //   + noteheads + brace = many paths
    assert!(path_count >= 15, "need at least 15 paths, got {path_count}");
}
