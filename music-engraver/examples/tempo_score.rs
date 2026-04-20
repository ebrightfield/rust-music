//! Example: render notes with tempo markings using the high-level ScoreBuilder API.
//!
//! Produces `examples/output/tempo_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::tempo::{MetronomeNoteKind, TempoMark};
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
        _ => panic!("unknown note: {name}"),
    };
    Pitch::new(note, octave).unwrap()
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1 — "Allegro" text tempo at start + metronome ♩=120
        .note(p("C", 4), Duration::QTR)
        .tempo(TempoMark::TextWithMetronome {
            text: "Allegro".into(),
            note_kind: MetronomeNoteKind::Quarter,
            dotted: false,
            bpm: 132,
        })
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // Measure 2 — no tempo marking
        .note(p("G", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .barline()
        // Measure 3 — tempo change: "Andante" text only
        .note(p("B", 4), Duration::QTR)
        .tempo(TempoMark::Text("Andante".into()))
        .note(p("A", 4), Duration::QTR)
        .note(p("G", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // Measure 4 — metronome only: dotted quarter = 72
        .note(p("E", 4), Duration::QTR)
        .tempo(TempoMark::Metronome {
            note_kind: MetronomeNoteKind::Quarter,
            dotted: true,
            bpm: 72,
        })
        .note(p("D", 4), Duration::QTR)
        .rest(Duration::HALF)
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("tempo_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Validate
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let text_count = svg.matches("<text").count();
    println!(
        "SVG contains {} paths, {} lines, {} texts",
        path_count, line_count, text_count
    );

    // Verify tempo mark content
    assert!(svg.contains(">Allegro<"), "should contain 'Allegro'");
    assert!(svg.contains("= 132"), "should contain '= 132'");
    assert!(svg.contains(">Andante<"), "should contain 'Andante'");
    assert!(svg.contains("= 72"), "should contain '= 72'");
    // At least 3 tempo text elements (Allegro, = 132, Andante, = 72)
    assert!(text_count >= 4, "expected at least 4 text elements, got {text_count}");
}
