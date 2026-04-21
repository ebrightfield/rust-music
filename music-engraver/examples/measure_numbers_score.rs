//! Example: render a score with measure numbers above each system.
//!
//! Produces `examples/output/measure_numbers_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
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
        .show_measure_numbers()
        // Measure 1
        .note(p("C", 4), Duration::QTR)
        .note(p("D", 4), Duration::QTR)
        .note(p("E", 4), Duration::QTR)
        .note(p("F", 4), Duration::QTR)
        .barline()
        // Measure 2
        .note(p("G", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .barline()
        // Measure 3
        .note(p("D", 5), Duration::QTR)
        .note(p("C", 5), Duration::QTR)
        .note(p("B", 4), Duration::QTR)
        .note(p("A", 4), Duration::QTR)
        .barline()
        // Measure 4
        .note(p("G", 4), Duration::HALF)
        .note(p("E", 4), Duration::HALF)
        .barline()
        // Measure 5
        .note(p("C", 4), Duration::HALF)
        .rest(Duration::HALF)
        .barline()
        // Measure 6
        .note(p("C", 5), Duration::WHOLE)
        .end_barline()
        .render_svg();

    // Write to output file
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("measure_numbers_score.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Basic validation
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));

    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let text_count = svg.matches("<text").count();
    println!(
        "SVG contains {} paths, {} lines, {} texts",
        path_count, line_count, text_count
    );

    // 3 systems with 2 measures each → 3 measure number texts ("1", "3", "5")
    assert!(text_count >= 3, "expected at least 3 text elements for measure numbers, got {text_count}");
    assert!(svg.contains(">1<"), "measure number '1' should appear");
    assert!(svg.contains(">3<"), "measure number '3' should appear");
    assert!(svg.contains(">5<"), "measure number '5' should appear");

    // Measure numbers should NOT show "2", "4", "6" (those are mid-system)
    // (Note: "2" could appear in path data, so we check for the specific text element pattern)
    // Just verify the 3 expected numbers are present
    let number_texts: Vec<&str> = svg.split("<text")
        .filter_map(|chunk| {
            if let Some(start) = chunk.find('>') {
                if let Some(end) = chunk[start..].find("</text>") {
                    let text = &chunk[start + 1..start + end];
                    if text.len() <= 2 && text.chars().all(|c| c.is_ascii_digit()) {
                        return Some(text);
                    }
                }
            }
            None
        })
        .collect();
    println!("Measure number texts found: {:?}", number_texts);
    assert!(
        number_texts.contains(&"1") && number_texts.contains(&"3") && number_texts.contains(&"5"),
        "expected measure numbers 1, 3, 5 but got {:?}",
        number_texts
    );
}
