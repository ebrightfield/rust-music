//! Example: render a score with rehearsal marks using the ScoreBuilder API.
//!
//! Produces `examples/output/rehearsal_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::rehearsal::RehearsalStyle;
use music_engraver::score::ScoreBuilder;

fn p(name: &str, octave: u8) -> Pitch {
    let note = match name {
        "C" => Note::C,
        "D" => Note::D,
        "E" => Note::E,
        "F" => Note::F,
        "G" => Note::G,
        "A" => Note::A,
        "B" => Note::B,
        "Bb" => Note::Bes,
        _ => panic!("unknown note: {name}"),
    };
    Pitch::new(note, octave).unwrap()
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1 — Rehearsal A (boxed) at the start
        .note(p("C", 4), Duration::QTR)
        .rehearsal_mark("A", RehearsalStyle::Boxed)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // Measure 2 — no rehearsal mark
        .note(p("G", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .barline()
        // Measure 3 — Rehearsal B (boxed) on system 2
        .note(p("D", 5), Duration::QTR)
        .rehearsal_mark("B", RehearsalStyle::Boxed)
        .note(p("C", 5), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .barline()
        // Measure 4 — Rehearsal "1" (plain, no box) on the chord
        .chord(vec![p("C", 4), p("E", 4), p("G", 4)], Duration::HALF)
        .rehearsal_mark("1", RehearsalStyle::Plain)
        .note(p("G", 4), Duration::QTR)
        .rehearsal_mark("Fine", RehearsalStyle::Boxed)
        .rest(Duration::QTR)
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("rehearsal_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Basic validation
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let text_count = svg.matches("<text").count();
    let rect_count = svg.matches("<rect").count();
    println!(
        "SVG contains {} paths, {} lines, {} texts, {} rects",
        path_count, line_count, text_count, rect_count
    );

    // Should have at least 3 rehearsal mark texts (A, B, 1, Fine)
    assert!(text_count >= 4, "expected at least 4 text elements for rehearsal marks, got {text_count}");
    // A, B, Fine are boxed → at least 3 rects
    assert!(rect_count >= 3, "expected at least 3 rects for boxed rehearsal marks, got {rect_count}");
    // Verify specific rehearsal mark content
    assert!(svg.contains(">A<"), "rehearsal mark 'A' should appear");
    assert!(svg.contains(">B<"), "rehearsal mark 'B' should appear");
    assert!(svg.contains(">1<"), "rehearsal mark '1' should appear");
    assert!(svg.contains(">Fine<"), "rehearsal mark 'Fine' should appear");
}
