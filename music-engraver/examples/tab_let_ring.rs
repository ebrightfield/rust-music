/// Renders a tablature score demonstrating "let ring" notation.
///
/// "let ring" is shown as italic text above the staff, with dashed
/// continuation lines for sustained let-ring passages.
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        // Measure 1: Arpeggio with let ring on each note (dashed continuation)
        .quarter()
        .fret(6, 0)
        .let_ring()
        .next()
        .quarter()
        .fret(5, 2)
        .let_ring()
        .next()
        .quarter()
        .fret(4, 2)
        .let_ring()
        .next()
        .quarter()
        .fret(3, 1)
        .let_ring()
        .barline()
        // Measure 2: Single let ring on open chord, then normal notes
        .half()
        .fret(6, 0)
        .fret(5, 2)
        .fret(4, 2)
        .fret(3, 1)
        .fret(2, 0)
        .fret(1, 0)
        .let_ring()
        .next()
        .quarter()
        .fret(1, 3)
        .next()
        .quarter()
        .fret(1, 5)
        .barline()
        // Measure 3: Alternating let ring and normal
        .quarter()
        .fret(4, 0)
        .let_ring()
        .next()
        .quarter()
        .fret(3, 2)
        .next()
        .quarter()
        .fret(2, 3)
        .let_ring()
        .next()
        .quarter()
        .fret(1, 0)
        .barline()
        // Measure 4: Full measure let ring passage with high frets
        .quarter()
        .fret(1, 12)
        .let_ring()
        .next()
        .quarter()
        .fret(2, 12)
        .let_ring()
        .next()
        .quarter()
        .fret(3, 12)
        .let_ring()
        .next()
        .quarter()
        .fret(1, 15)
        .let_ring()
        .end_barline()
        .measures_per_system(2)
        .render_svg();

    // Write to file
    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/tab_let_ring.svg", &svg).expect("write SVG");

    // Verify structural elements
    let text_count = svg.matches("<text ").count();
    let line_count = svg.matches("<line ").count();
    let lr_count = svg.matches("let ring").count();

    println!(
        "tab_let_ring.svg: {} texts, {} lines, {} let ring occurrences, {} bytes",
        text_count,
        line_count,
        lr_count,
        svg.len()
    );

    assert!(svg.starts_with("<svg"));
    assert!(
        lr_count >= 8,
        "should have at least 8 'let ring' text elements, got {lr_count}"
    );
    assert!(
        svg.contains("stroke-dasharray"),
        "consecutive let ring events should produce dashed continuation lines"
    );
    assert!(svg.contains("italic"), "let ring text should be italic");
}
