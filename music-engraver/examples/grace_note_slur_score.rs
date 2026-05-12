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

use music_engraver::layout::grace::GraceNoteKind;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: acciaccatura with slur (canonical "crushed note")
        .note(Pitch::new(Note::E, 4).expect("valid pitch"), Duration::QTR)
        .grace_note_slur(
            Pitch::new(Note::D, 4).expect("valid pitch"),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::G, 4).expect("valid pitch"), Duration::QTR)
        .grace_note_slur(
            Pitch::new(Note::Fis, 4).expect("valid pitch"),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::A, 4).expect("valid pitch"), Duration::HALF)
        .barline()
        // Measure 2: appoggiatura with slur (stem-up & stem-down principal)
        .note(Pitch::new(Note::C, 5).expect("valid pitch"), Duration::HALF)
        .grace_note_slur(
            Pitch::new(Note::B, 4).expect("valid pitch"),
            GraceNoteKind::Appoggiatura,
        )
        .note(Pitch::new(Note::D, 5).expect("valid pitch"), Duration::HALF)
        .grace_note_slur(
            Pitch::new(Note::Cis, 5).expect("valid pitch"),
            GraceNoteKind::Appoggiatura,
        )
        .barline()
        // Measure 3: wide intervals (grace below, principal above; and vice versa)
        .note(Pitch::new(Note::A, 5).expect("valid pitch"), Duration::QTR)
        .grace_note_slur(
            Pitch::new(Note::E, 5).expect("valid pitch"),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::C, 4).expect("valid pitch"), Duration::QTR)
        .grace_note_slur(
            Pitch::new(Note::F, 4).expect("valid pitch"),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::G, 4).expect("valid pitch"), Duration::HALF)
        .barline()
        // Measure 4: slurred grace on a chord, plus a rest (slur should be no-op on rest)
        .chord(
            vec![
                Pitch::new(Note::C, 4).expect("valid pitch"),
                Pitch::new(Note::E, 4).expect("valid pitch"),
                Pitch::new(Note::G, 4).expect("valid pitch"),
            ],
            Duration::HALF,
        )
        .grace_note_slur(
            Pitch::new(Note::B, 3).expect("valid pitch"),
            GraceNoteKind::Acciaccatura,
        )
        .rest(Duration::QTR)
        .grace_note_slur(
            Pitch::new(Note::A, 4).expect("valid pitch"),
            GraceNoteKind::Acciaccatura,
        )
        .note(Pitch::new(Note::D, 4).expect("valid pitch"), Duration::QTR)
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

    // Mechanical assertions.
    assert!(svg.starts_with("<svg"), "output should be SVG");
    assert!(
        svg.contains("scale(0.6"),
        "grace glyphs should appear at scale 0.6"
    );
    // 6 grace_note_slur calls land on pitched events (measures 1–3: 2+2+2 = 6),
    // measure 4 chord = 1 more. The grace_note_slur on the rest is a no-op.
    // Total expected slur paths: 7.
    assert_eq!(
        slur_path_count, 7,
        "expected exactly 7 slur paths (one per pitched grace_note_slur call), got {slur_path_count}"
    );
}
