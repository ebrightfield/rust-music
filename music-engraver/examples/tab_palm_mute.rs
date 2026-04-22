/// Renders a tablature score demonstrating palm muting notation.
///
/// Palm muting is shown as "P.M." text above the staff, with dashed
/// continuation lines for sustained palm mute passages.
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        // Measure 1: Single palm-muted power chord, then normal notes
        .quarter()
        .fret(6, 0)
        .fret(5, 2)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .fret(5, 2)
        .palm_mute()
        .next()
        .quarter()
        .fret(1, 3)
        .next()
        .quarter()
        .fret(1, 5)
        .barline()
        // Measure 2: Three consecutive palm-muted eighths (dashed line) + quarter
        .eighth()
        .fret(6, 0)
        .palm_mute()
        .next()
        .eighth()
        .fret(6, 0)
        .palm_mute()
        .next()
        .eighth()
        .fret(6, 0)
        .palm_mute()
        .next()
        .quarter()
        .fret(5, 2)
        .barline()
        // Measure 3: Alternating palm mute and normal
        .quarter()
        .fret(6, 3)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 5)
        .next()
        .quarter()
        .fret(6, 3)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 5)
        .barline()
        // Measure 4: Full measure palm mute passage (long dashed line)
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .next()
        .quarter()
        .fret(6, 0)
        .palm_mute()
        .end_barline()
        .measures_per_system(2)
        .render_svg();

    // Write to file
    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/tab_palm_mute.svg", &svg)
        .expect("write SVG");

    // Verify structural elements
    let text_count = svg.matches("<text ").count();
    let line_count = svg.matches("<line ").count();
    let pm_count = svg.matches("P.M.").count();

    println!(
        "tab_palm_mute.svg: {} texts, {} lines, {} P.M. occurrences, {} bytes",
        text_count,
        line_count,
        pm_count,
        svg.len()
    );

    assert!(svg.starts_with("<svg"));
    assert!(pm_count >= 8, "should have at least 8 P.M. text elements, got {pm_count}");
    assert!(
        svg.contains("stroke-dasharray"),
        "consecutive palm mutes should produce dashed continuation lines"
    );
}
