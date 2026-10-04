/// Render grace notes connected to their principal notes by a small slur.
///
/// Demonstrates the canonical engraving for acciaccatura — a slashed grace
/// note with a slur from the grace to the principal — and the same pattern
/// applied to appoggiatura and to chords. Compare against
/// `grace_notes_score.rs`, which renders the same gestures without slurs.
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::layout::grace::{GraceNoteKind, GraceNotes};
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn slurred_grace(pitch: Pitch, kind: GraceNoteKind) -> GraceNotes {
    GraceNotes::new(kind).note(pitch, Duration::EIGHTH).slur()
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: acciaccatura with slur (canonical "crushed note")
        .note(Pitch::new(Note::E, 4), Duration::QTR)
        .grace_notes(slurred_grace(Pitch::new(Note::D, 4), GraceNoteKind::Acciaccatura))
        .note(Pitch::new(Note::G, 4), Duration::QTR)
        .grace_notes(slurred_grace(Pitch::new(Note::Fis, 4), GraceNoteKind::Acciaccatura))
        .note(Pitch::new(Note::A, 4), Duration::HALF)
        .barline()
        // Measure 2: appoggiatura with slur (stem-up & stem-down principal)
        .note(Pitch::new(Note::C, 5), Duration::HALF)
        .grace_notes(slurred_grace(Pitch::new(Note::B, 4), GraceNoteKind::Appoggiatura))
        .note(Pitch::new(Note::D, 5), Duration::HALF)
        .grace_notes(slurred_grace(Pitch::new(Note::Cis, 5), GraceNoteKind::Appoggiatura))
        .barline()
        // Measure 3: wide intervals (grace below, principal above; and vice versa)
        .note(Pitch::new(Note::A, 5), Duration::QTR)
        .grace_notes(slurred_grace(Pitch::new(Note::E, 5), GraceNoteKind::Acciaccatura))
        .note(Pitch::new(Note::C, 4), Duration::QTR)
        .grace_notes(slurred_grace(Pitch::new(Note::F, 4), GraceNoteKind::Acciaccatura))
        .note(Pitch::new(Note::G, 4), Duration::HALF)
        .barline()
        // Measure 4: slurred grace on a chord, plus a rest (slur should be no-op on rest)
        .chord(
            vec![
                Pitch::new(Note::C, 4),
                Pitch::new(Note::E, 4),
                Pitch::new(Note::G, 4),
            ],
            Duration::HALF,
        )
        .grace_notes(slurred_grace(Pitch::new(Note::B, 3), GraceNoteKind::Acciaccatura))
        .rest(Duration::QTR)
        .grace_notes(slurred_grace(Pitch::new(Note::A, 4), GraceNoteKind::Acciaccatura))
        .note(Pitch::new(Note::D, 4), Duration::QTR)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/grace_note_slur_score.svg",
        &svg,
    )
    .expect("write SVG");

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let slur_path_count = svg.matches(r#"stroke="none""#).count();
    println!(
        "grace_note_slur_score.svg: {} bytes, {} paths, {} lines, {} slur paths",
        svg.len(),
        path_count,
        line_count,
        slur_path_count
    );

}
