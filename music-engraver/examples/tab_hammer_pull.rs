/// Example: tablature hammer-on and pull-off notation.
///
/// Renders 4 measures across 2 systems showing:
/// 1. Single-string hammer-on (5→7 on string 1)
/// 2. Single-string pull-off (7→5 on string 1)
/// 3. Hammer-pull chain (5→7→5 on string 1)
/// 4. Multi-string chord hammer-on (power chord shift)
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .measures_per_system(2)
        // Measure 1: hammer-on 5→7 on string 1
        .fret(1, 5)
        .hammer()
        .next()
        .fret(1, 7)
        .next()
        .rest()
        .next()
        .rest()
        .barline()
        // Measure 2: pull-off 7→5 on string 1
        .fret(1, 7)
        .pull()
        .next()
        .fret(1, 5)
        .next()
        .rest()
        .next()
        .rest()
        .barline()
        // Measure 3: chain 5→7→5 on string 2
        .fret(2, 5)
        .hammer()
        .next()
        .fret(2, 7)
        .pull()
        .next()
        .fret(2, 5)
        .next()
        .rest()
        .barline()
        // Measure 4: multi-string chord hammer (power chord shift)
        .fret(5, 5)
        .fret(4, 7)
        .fret(3, 7)
        .hammer()
        .next()
        .fret(5, 7)
        .fret(4, 9)
        .fret(3, 9)
        .end_barline()
        .render_svg();

    let output_dir = std::path::Path::new("music-engraver/examples/output");
    std::fs::create_dir_all(output_dir).expect("create output dir");
    let path = output_dir.join("tab_hammer_pull.svg");
    std::fs::write(&path, &svg).expect("write SVG");
    println!("Wrote {}", path.display());

    // Structural assertions
    assert!(svg.starts_with("<svg"), "output should be SVG");
    assert!(svg.contains("</svg>"), "output should be well-formed");
    assert!(svg.contains(">H</text>"), "should contain H label");
    assert!(svg.contains(">P</text>"), "should contain P label");

    let path_count = svg.matches("<path ").count();
    let text_count = svg.matches("<text ").count();
    let line_count = svg.matches("<line ").count();
    println!(
        "{path_count} paths, {text_count} texts, {line_count} lines, {} bytes",
        svg.len()
    );
}
