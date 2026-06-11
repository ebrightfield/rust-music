/// Render grace notes via ScoreBuilder API.
///
/// Demonstrates acciaccatura (slashed) and appoggiatura grace notes
/// attached to various notes and chords across 2 systems (4 measures).
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::grace::GraceNoteKind;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: acciaccatura grace notes (slashed)
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .grace_note(
            Pitch::new(Note::D, 4),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .grace_note(
            Pitch::new(Note::Fis, 4),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .barline()
        // Measure 2: appoggiatura grace notes (no slash)
        .note(Pitch::new(Note::C, 5), Duration::HALF)
        .grace_note(
            Pitch::new(Note::B, 4),
            GraceNoteKind::Appoggiatura,
        )
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .grace_note(
            Pitch::new(Note::Cis, 5),
            GraceNoteKind::Appoggiatura,
        )
        .barline()
        // Measure 3: grace notes on low and high notes
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .grace_note(
            Pitch::new(Note::B, 3),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::A, 5), Duration::QTR)
        .grace_note(
            Pitch::new(Note::G, 5),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::F, 4), Duration::HALF)
        .barline()
        // Measure 4: grace note on a chord + rest (rest should be no-op)
        .chord(
            vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ],
            Duration::HALF,
        )
        .grace_note(
            Pitch::new(Note::B, 3),
            GraceNoteKind::Acciaccatura,
        )
        .rest(Duration::QTR)
        // Grace note on rest — should be no-op
        .grace_note(
            Pitch::new(Note::A, 4),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/grace_notes_score.svg",
        &svg,
    )
    .expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "output should be SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    println!(
        "grace_notes_score.svg: {} bytes, {} paths, {} lines",
        svg.len(),
        path_count,
        line_count
    );

    // Grace notes add paths with scale transforms
    assert!(
        svg.contains("scale(0.6"),
        "grace notes should have 0.6 scale transform"
    );

    // At least some paths for grace notes
    assert!(
        path_count >= 20,
        "expected at least 20 paths (notes + clefs + grace notes), got {path_count}"
    );
}
