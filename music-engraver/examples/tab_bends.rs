/// Tab bends example: demonstrates bend arrows with different amounts on a guitar tab.
///
/// Renders 4 measures across 2 systems showing full, half, quarter, and chord bends.
use music_engraver::layout::tab_bend::BendAmount;
use music_engraver::score::tab::TabScoreBuilder;
use std::fs;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .measures_per_system(2)
        .system_width_fu(12000.0)
        // Measure 1: full bend on string 2 fret 8, then half bend on string 1 fret 7
        .fret(2, 8)
        .bend(BendAmount::Full)
        .next()
        .fret(1, 7)
        .bend(BendAmount::Half)
        .next()
        .fret(3, 9)
        .next()
        .fret(1, 5)
        .barline()
        // Measure 2: quarter bend, then chord bend (2 strings), then normal note
        .fret(3, 7)
        .bend(BendAmount::Quarter)
        .next()
        .fret(1, 10)
        .fret(2, 10)
        .bend(BendAmount::Full)
        .next()
        .fret(1, 12)
        .bend(BendAmount::OneAndHalf)
        .next()
        .fret(1, 7)
        .barline()
        // Measure 3: mix of bends and slides
        .fret(1, 5)
        .bend(BendAmount::Full)
        .next()
        .fret(1, 7)
        .slide()
        .next()
        .fret(1, 9)
        .bend(BendAmount::Half)
        .next()
        .fret(1, 12)
        .barline()
        // Measure 4: hammer with bend
        .fret(2, 5)
        .hammer()
        .next()
        .fret(2, 7)
        .bend(BendAmount::Full)
        .next()
        .fret(3, 9)
        .bend(BendAmount::Full)
        .next()
        .rest()
        .end_barline()
        .render_svg();

    fs::create_dir_all("examples/output").expect("create output dir");
    fs::write("examples/output/tab_bends.svg", &svg).expect("write SVG");

    // Verify structure
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should close SVG");
    assert!(svg.contains(">full</text>"), "should have 'full' bend labels");
    assert!(svg.contains(">1/2</text>"), "should have '1/2' bend labels");
    assert!(svg.contains(">1/4</text>"), "should have '1/4' bend label");
    assert!(svg.contains(">1 1/2</text>"), "should have '1 1/2' bend label");

    // Count bend arrow paths (filled arrowheads have fill="black")
    let filled_paths = svg.matches("fill=\"black\"").count();
    assert!(
        filled_paths >= 8,
        "should have at least 8 filled arrowheads (one per bend), got {filled_paths}"
    );

    let path_count = svg.matches("<path ").count();
    let line_count = svg.matches("<line ").count();
    let text_count = svg.matches("<text ").count();
    println!(
        "tab_bends.svg: {} paths, {} lines, {} texts ({} bytes)",
        path_count,
        line_count,
        text_count,
        svg.len()
    );
}
