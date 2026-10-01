/// Example: ottava brackets (8va/8vb/15ma/15mb) via ScoreBuilder API.
///
/// Renders 4 measures across 2 systems demonstrating:
/// - 8va bracket over high notes in measure 1
/// - 8vb bracket under low notes in measure 2
/// - 15ma bracket in measure 3
/// - 15mb bracket in measure 4
///
/// Run: cargo run --example ottava_score
use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::ottava::OttavaKind;
use music_engraver::score::ScoreBuilder;

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Open)
        .time_signature(4, 4)
        .measures_per_system(2)
        // Measure 1: 8va bracket over high passage
        .note(Pitch::new(Note::C, 6), Duration::QTR)
        .ottava_start(OttavaKind::Ottava8va)
        .note(Pitch::new(Note::D, 6), Duration::QTR)
        .note(Pitch::new(Note::E, 6), Duration::QTR)
        .note(Pitch::new(Note::F, 6), Duration::QTR)
        .ottava_end()
        .barline()
        // Measure 2: 8vb bracket under low passage
        .note(Pitch::new(Note::C, 3), Duration::QTR)
        .ottava_start(OttavaKind::Ottava8vb)
        .note(Pitch::new(Note::B, 2), Duration::QTR)
        .note(Pitch::new(Note::A, 2), Duration::QTR)
        .ottava_end()
        .note(Pitch::new(Note::G, 3), Duration::QTR)
        .barline()
        // Measure 3: 15ma bracket (two octaves up)
        .note(Pitch::new(Note::G, 5), Duration::HALF)
        .ottava_start(OttavaKind::Ottava15ma)
        .note(Pitch::new(Note::A, 5), Duration::HALF)
        .ottava_end()
        .barline()
        // Measure 4: 15mb bracket (two octaves down)
        .note(Pitch::new(Note::E, 3), Duration::HALF)
        .ottava_start(OttavaKind::Ottava15mb)
        .note(Pitch::new(Note::D, 3), Duration::HALF)
        .ottava_end()
        .end_barline()
        .render_svg();

    let dir = std::path::Path::new("music-engraver/examples/output");
    std::fs::create_dir_all(dir).expect("create output dir");
    let path = dir.join("ottava_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {}", path.display());

    // Structural assertions
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains(">8va</text>"), "should contain 8va label");
    assert!(svg.contains(">8vb</text>"), "should contain 8vb label");
    assert!(svg.contains(">15ma</text>"), "should contain 15ma label");
    assert!(svg.contains(">15mb</text>"), "should contain 15mb label");
    assert!(
        svg.contains("stroke-dasharray"),
        "should contain dashed lines"
    );

    let paths = svg.matches("<path ").count();
    let lines = svg.matches("<line ").count();
    let texts = svg.matches("<text ").count();
    println!(
        "Stats: {} paths, {} lines, {} texts, {} bytes",
        paths,
        lines,
        texts,
        svg.len()
    );
}
