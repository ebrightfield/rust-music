/// Tab slide example: diagonal slide lines between fret positions on guitar tablature.
///
/// Renders 4 measures across 2 systems showing:
/// 1. Single-string ascending slide (fret 5 → 7)
/// 2. Single-string descending slide (fret 12 → 9)
/// 3. Multi-string chord slide (power chord shift)
/// 4. Consecutive slides on one string (5 → 7 → 9 → 12)
use music_engraver::score::tab::TabScoreBuilder;
use std::fs;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .measures_per_system(2)
        .system_width_fu(12000.0)
        // Measure 1: single-string ascending slide on string 1
        .fret(1, 5).slide().next()
        .fret(1, 7).next()
        .fret(1, 3).next()
        .fret(1, 0)
        .barline()
        // Measure 2: descending slide on string 2, plus a non-slide note
        .fret(2, 12).slide().next()
        .fret(2, 9).next()
        .fret(2, 7).next()
        .fret(1, 5)
        .barline()
        // Measure 3: power chord slide (strings 6+5+4 shift up)
        .fret(6, 3).fret(5, 5).fret(4, 5).slide().next()
        .fret(6, 5).fret(5, 7).fret(4, 7).next()
        .rest().next()
        .fret(6, 0)
        .barline()
        // Measure 4: consecutive slides on string 1 (chain)
        .fret(1, 5).slide().next()
        .fret(1, 7).slide().next()
        .fret(1, 9).slide().next()
        .fret(1, 12)
        .end_barline()
        .render_svg();

    fs::create_dir_all("music-engraver/examples/output").ok();
    fs::write("music-engraver/examples/output/tab_slides.svg", &svg).unwrap();
    println!("Wrote music-engraver/examples/output/tab_slides.svg");
    println!("  {} bytes", svg.len());
    println!("  {} lines", svg.matches("<line ").count());
    println!("  {} paths", svg.matches("<path ").count());
    println!("  {} texts", svg.matches("<text ").count());
    println!("  {} rects", svg.matches("<rect ").count());

    // Verify structural expectations
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Fret numbers present
    assert!(svg.contains(">5</text>"), "should have fret 5");
    assert!(svg.contains(">7</text>"), "should have fret 7");
    assert!(svg.contains(">12</text>"), "should have fret 12");
    assert!(svg.contains(">9</text>"), "should have fret 9");
    assert!(svg.contains(">0</text>"), "should have open string");

    // Slide lines: measure 1 (1 slide), measure 2 (1 slide), measure 3 (3 slides
    // for 3-string chord), measure 4 (3 consecutive slides) = 8 total slide lines
    // Staff lines: 2 systems × 6 = 12, plus barlines
    let line_count = svg.matches("<line ").count();
    // At least 12 staff lines + some barlines + some slide lines
    assert!(
        line_count >= 20,
        "should have at least 20 lines (staff + barlines + slides), got {line_count}"
    );

    // Text: m1(4) + m2(4) + m3(7 frets in 2 chords + 1 single) + m4(4) = ~19 fret texts
    let text_count = svg.matches("<text ").count();
    assert!(
        text_count >= 15,
        "should have at least 15 fret number texts, got {text_count}"
    );
}
