/// Tab beam group example: beamed eighth and sixteenth notes on guitar tablature.
///
/// Renders 4 measures across 2 systems showing:
/// 1. Four beamed eighth notes (ascending on string 1)
/// 2. Two beamed sixteenths + individual quarter
/// 3. Beamed eighth chord (E minor arpeggio on strings 1-3)
/// 4. Mixed: quarter + beamed sixteenth run
use music_engraver::score::tab::TabScoreBuilder;
use std::fs;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .measures_per_system(2)
        .system_width_fu(12000.0)
        // Measure 1: four beamed eighths ascending on string 1
        .beam_start()
        .eighth().fret(1, 0).next()
        .eighth().fret(1, 2).next()
        .eighth().fret(1, 3).next()
        .eighth().fret(1, 5)
        .beam_end()
        .barline()
        // Measure 2: two beamed sixteenths + a quarter
        .beam_start()
        .duration(4).fret(1, 7).next()
        .duration(4).fret(1, 8)
        .beam_end()
        .next()
        .quarter().fret(1, 5)
        .barline()
        // Measure 3: beamed eighths with chords (strings 1-3)
        .beam_start()
        .eighth().fret(1, 0).fret(2, 0).fret(3, 0).next()
        .eighth().fret(1, 2).fret(2, 2).fret(3, 1).next()
        .eighth().fret(1, 3).fret(2, 3).fret(3, 2)
        .beam_end()
        .barline()
        // Measure 4: quarter + 4 beamed sixteenths
        .quarter().fret(1, 0).next()
        .beam_start()
        .duration(4).fret(1, 5).next()
        .duration(4).fret(1, 7).next()
        .duration(4).fret(1, 8).next()
        .duration(4).fret(1, 10)
        .beam_end()
        .end_barline()
        .render_svg();

    fs::create_dir_all("music-engraver/examples/output").ok();
    fs::write("music-engraver/examples/output/tab_beams.svg", &svg).unwrap();
    println!("Wrote music-engraver/examples/output/tab_beams.svg");
    println!("  {} bytes", svg.len());
    println!("  {} lines", svg.matches("<line ").count());
    println!("  {} paths", svg.matches("<path ").count());
    println!("  {} texts", svg.matches("<text ").count());
    println!("  {} polygons", svg.matches("<polygon ").count());

    // Verify structural expectations
    let line_count = svg.matches("<line ").count();
    let polygon_count = svg.matches("<polygon ").count();
    let text_count = svg.matches("<text ").count();

    // 2 systems × 6 staff lines = 12, plus barlines and stems
    assert!(line_count >= 12, "should have at least 12 staff lines, got {line_count}");
    // Beam polygons: measure 1 (1 primary), measure 2 (2: primary+secondary),
    // measure 3 (1 primary), measure 4 (2: primary+secondary)
    assert!(polygon_count >= 4, "should have at least 4 beam polygons, got {polygon_count}");
    // Fret numbers: m1(4) + m2(2+1) + m3(9 frets across 3 chords) + m4(1+4) = 21
    assert!(text_count >= 15, "should have at least 15 fret numbers, got {text_count}");
}
