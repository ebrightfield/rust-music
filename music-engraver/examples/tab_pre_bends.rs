/// Tab pre-bends and release bends example.
///
/// Renders 4 measures across 2 systems demonstrating pre-bend arrows
/// (straight vertical) and release bend arrows (downward curve).
use music_engraver::layout::tab_bend::BendAmount;
use music_engraver::score::tab::TabScoreBuilder;
use std::fs;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .measures_per_system(2)
        .system_width_fu(12000.0)
        // Measure 1: full pre-bend on string 2, then half pre-bend on string 1
        .fret(2, 8)
        .pre_bend(BendAmount::Full)
        .next()
        .fret(1, 7)
        .pre_bend(BendAmount::Half)
        .next()
        .fret(3, 9)
        .next()
        .fret(1, 5)
        .barline()
        // Measure 2: pre-bend then release sequence, chord pre-bend
        .fret(1, 7)
        .pre_bend(BendAmount::Full)
        .next()
        .fret(1, 5)
        .release()
        .next()
        .fret(1, 10)
        .fret(2, 10)
        .pre_bend(BendAmount::Half)
        .next()
        .fret(1, 12)
        .barline()
        // Measure 3: regular bend followed by release on next note
        .fret(2, 7)
        .bend(BendAmount::Full)
        .next()
        .fret(2, 5)
        .release()
        .next()
        .fret(3, 9)
        .pre_bend(BendAmount::Quarter)
        .next()
        .fret(1, 7)
        .barline()
        // Measure 4: mixed — pre-bend, normal, release, pre-bend with slide
        .fret(1, 5)
        .pre_bend(BendAmount::OneAndHalf)
        .next()
        .fret(1, 7)
        .next()
        .fret(2, 8)
        .release()
        .next()
        .rest()
        .end_barline()
        .render_svg();

    fs::create_dir_all("examples/output").expect("create output dir");
    fs::write("examples/output/tab_pre_bends.svg", &svg).expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should close SVG");

    // Pre-bends have text labels (full, 1/2, 1/4, 1 1/2)
    assert!(svg.contains(">full</text>"), "should have 'full' pre-bend labels");
    assert!(svg.contains(">1/2</text>"), "should have '1/2' pre-bend labels");
    assert!(svg.contains(">1/4</text>"), "should have '1/4' pre-bend label");
    assert!(svg.contains(">1 1/2</text>"), "should have '1 1/2' pre-bend label");

    // Pre-bends use filled arrowheads
    let filled_paths = svg.matches("fill=\"black\"").count();
    assert!(
        filled_paths >= 6,
        "should have at least 6 filled arrowheads, got {filled_paths}"
    );

    // Release bends have downward arrows (no text label)
    // They produce paths (curve + arrowhead)
    let path_count = svg.matches("<path ").count();
    let line_count = svg.matches("<line ").count();
    let text_count = svg.matches("<text ").count();
    println!(
        "tab_pre_bends.svg: {} paths, {} lines, {} texts ({} bytes)",
        path_count, line_count, text_count, svg.len()
    );
}
