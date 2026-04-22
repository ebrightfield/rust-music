/// Tab vibrato example: renders fret numbers with vibrato wavy lines.
///
/// Demonstrates normal and wide vibrato on single notes and chords,
/// combined with bends.
use music_engraver::layout::tab_bend::BendAmount;
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        // Measure 1: normal vibrato on single notes
        .quarter()
        .fret(1, 5)
        .vibrato()
        .next()
        .fret(1, 7)
        .vibrato()
        .next()
        .fret(2, 5)
        .vibrato()
        .next()
        .fret(1, 5)
        .barline()
        // Measure 2: wide vibrato on single notes + chord
        .fret(1, 12)
        .wide_vibrato()
        .next()
        .fret(3, 9)
        .wide_vibrato()
        .next()
        .fret(1, 7)
        .fret(2, 7)
        .vibrato()
        .next()
        .fret(1, 5)
        .barline()
        // Measure 3: vibrato combined with bend
        .fret(1, 7)
        .vibrato()
        .bend(BendAmount::Full)
        .next()
        .fret(2, 9)
        .wide_vibrato()
        .bend(BendAmount::Half)
        .next()
        .fret(1, 5)
        .next()
        .fret(1, 3)
        .vibrato()
        .barline()
        // Measure 4: no vibrato for contrast + final wide vibrato
        .fret(1, 0)
        .next()
        .fret(2, 0)
        .next()
        .fret(3, 0)
        .next()
        .fret(1, 12)
        .wide_vibrato()
        .end_barline()
        .render_svg();

    // Write SVG output
    let out_dir = std::path::Path::new("music-engraver/examples/output");
    std::fs::create_dir_all(out_dir).expect("create output dir");
    let path = out_dir.join("tab_vibrato.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {} ({} bytes)", path.display(), svg.len());

    // Structural assertions
    assert!(svg.starts_with("<svg "), "should be valid SVG");
    assert!(svg.contains(" Q"), "should contain vibrato wavy paths (Q commands)");
    assert!(
        svg.contains("fill=\"none\""),
        "vibrato waves should have no fill"
    );
    // Count vibrato wave paths (unfilled stroke paths with Q commands)
    let wave_count = svg
        .matches("fill=\"none\" stroke=\"black\"")
        .count();
    assert!(
        wave_count >= 8,
        "should have at least 8 vibrato waves (got {})",
        wave_count
    );
    println!(
        "Assertions passed: {} vibrato waves rendered",
        wave_count
    );
}
