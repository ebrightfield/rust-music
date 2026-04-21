/// Renders a guitar tablature score via TabScoreBuilder to SVG.
///
/// Demonstrates: fret numbers on various strings, chords (simultaneous fret
/// numbers), multi-measure layout with barlines, and multi-system breaking.
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .measures_per_system(2)
        // Measure 1: E minor chord arpeggio (open position)
        .fret(6, 0).next()  // open low E
        .fret(5, 2).next()  // A string fret 2 (B)
        .fret(4, 2).next()  // D string fret 2 (E)
        .fret(3, 0)         // open G
        .barline()
        // Measure 2: ascending scale on string 1
        .fret(1, 0).next()   // open high E
        .fret(1, 1).next()   // F
        .fret(1, 3).next()   // G
        .fret(1, 5)          // A
        .barline()
        // Measure 3: power chord (E5) — simultaneous
        .fret(6, 0)
        .fret(5, 2)
        .fret(4, 2)
        .next()
        .rest()
        .next()
        .fret(6, 3)
        .fret(5, 5)
        .fret(4, 5)
        .barline()
        // Measure 4: high position with 2-digit frets
        .fret(1, 12).next()
        .fret(1, 15).next()
        .fret(2, 12).next()
        .fret(1, 17)
        .end_barline();

    let output = svg.render_svg();

    // Write to file
    let dir = std::path::Path::new("music-engraver/examples/output");
    std::fs::create_dir_all(dir).expect("create output dir");
    let path = dir.join("tab_score.svg");
    std::fs::write(&path, &output).expect("write SVG");

    println!("Wrote {} bytes to {}", output.len(), path.display());

    // Validate structure
    assert!(output.starts_with("<svg"));
    assert!(output.contains("</svg>"));

    // Should have fret number texts
    assert!(output.contains(">0</text>"), "should have open string");
    assert!(output.contains(">2</text>"), "should have fret 2");
    assert!(output.contains(">12</text>"), "should have fret 12");
    assert!(output.contains(">15</text>"), "should have fret 15");
    assert!(output.contains(">17</text>"), "should have fret 17");

    // Should have 2 systems × TAB clef paths
    let path_count = output.matches("<path ").count();
    assert!(
        path_count >= 2,
        "should have at least 2 TAB clef paths, got {path_count}"
    );

    // Count text elements (fret numbers)
    let text_count = output.matches("<text ").count();
    println!(
        "SVG: {} paths, {} lines, {} texts, {} rects",
        output.matches("<path ").count(),
        output.matches("<line ").count(),
        text_count,
        output.matches("<rect ").count(),
    );
}
