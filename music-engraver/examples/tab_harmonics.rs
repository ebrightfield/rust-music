/// Example: tablature natural harmonics via TabScoreBuilder API.
///
/// Renders 4 measures across 2 systems showing natural harmonics
/// on common fret positions (5, 7, 12) with the harmonic indicator (○).
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .measures_per_system(2)
        // Measure 1: natural harmonics at fret 12 (octave harmonics)
        .fret(1, 12)
        .harmonic()
        .next()
        .fret(2, 12)
        .harmonic()
        .next()
        .fret(3, 12)
        .harmonic()
        .next()
        .fret(4, 12)
        .harmonic()
        .barline()
        // Measure 2: harmonics at fret 7 on multiple strings (chord harmonic)
        .fret(1, 7)
        .fret(2, 7)
        .fret(3, 7)
        .harmonic()
        .next()
        .fret(1, 5)
        .harmonic()
        .next()
        .fret(6, 12)
        .harmonic()
        .barline()
        // Measure 3: mix of harmonics and normal notes
        .fret(1, 12)
        .harmonic()
        .next()
        .fret(1, 0) // normal open string, no harmonic
        .next()
        .fret(1, 7)
        .harmonic()
        .next()
        .fret(1, 5) // normal fret 5, no harmonic
        .barline()
        // Measure 4: harmonics with vibrato
        .fret(1, 12)
        .harmonic()
        .vibrato()
        .next()
        .fret(2, 7)
        .harmonic()
        .next()
        .fret(1, 5)
        .fret(2, 5)
        .harmonic()
        .end_barline()
        .render_svg();

    // Write to examples/output/
    let dir = std::path::Path::new("examples/output");
    std::fs::create_dir_all(dir).expect("create output dir");
    let path = dir.join("tab_harmonics.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} bytes to {}", svg.len(), path.display());

    // Structural assertions
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should close SVG");
    assert!(
        svg.contains("scale(0.6)"),
        "should contain harmonic glyph scale transforms"
    );

    let path_count = svg.matches("<path ").count();
    let text_count = svg.matches("<text ").count();
    let line_count = svg.matches("<line ").count();
    println!(
        "Elements: {} paths, {} texts, {} lines",
        path_count, text_count, line_count
    );

    // 2 TAB clef paths + 12 harmonic glyphs = 14 paths minimum
    assert!(
        path_count >= 14,
        "should have at least 14 paths (2 TAB clefs + 12 harmonic indicators), got {path_count}"
    );
}
