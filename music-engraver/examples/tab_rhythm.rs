//! Tablature with rhythm stems above the staff.
//!
//! Renders four measures of guitar TAB with rhythm notation: quarter notes,
//! eighth notes, half notes, and a whole note, with custom stem geometry.

use music_engraver::layout::TabRhythmStyle;
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        .system_width_fu(10000.0)
        .rhythm_style(TabRhythmStyle::new(2.0, 4.0).expect("positive TAB stem length"))
        // Measure 1: E minor arpeggio with quarter note rhythm
        .quarter()
        .fret(6, 0)
        .next() // open low E — quarter
        .quarter()
        .fret(5, 2)
        .next() // B on A string — quarter
        .eighth()
        .fret(4, 2)
        .next() // E on D string — eighth
        .eighth()
        .fret(3, 0) // open G — eighth
        .barline()
        // Measure 2: power chord + half note + whole note
        .half()
        .fret(6, 3) // G power chord (simultaneous)
        .fret(5, 5)
        .fret(4, 5)
        .next()
        .half()
        .fret(1, 3) // G on high E — half
        .barline()
        // Measure 3: ascending scale with sixteenth notes
        .duration(4)
        .fret(1, 0)
        .next() // sixteenth
        .duration(4)
        .fret(1, 1)
        .next() // sixteenth
        .duration(4)
        .fret(1, 2)
        .next() // sixteenth
        .duration(4)
        .fret(1, 3)
        .next() // sixteenth
        .whole()
        .fret(1, 5) // whole note rest of bar
        .barline()
        // Measure 4: mixed — no duration on some events
        .quarter()
        .fret(3, 0)
        .next()
        .fret(3, 2)
        .next() // no duration — no stem
        .eighth()
        .fret(3, 4)
        .next()
        .quarter()
        .rest() // rest with quarter duration stem
        .end_barline()
        .render_svg();

    let output_dir = std::path::Path::new("music-engraver/examples/output");
    std::fs::create_dir_all(output_dir).expect("create output dir");
    let path = output_dir.join("tab_rhythm.svg");
    std::fs::write(&path, &svg).expect("write SVG");

    println!("Wrote {} bytes to {}", svg.len(), path.display());
    println!("  <line> count:  {}", svg.matches("<line ").count());
    println!("  <path> count:  {}", svg.matches("<path ").count());
    println!("  <text> count:  {}", svg.matches("<text ").count());
    println!("  <rect> count:  {}", svg.matches("<rect ").count());

    // Verify structural expectations
    assert!(svg.starts_with("<svg"), "should be valid SVG");
    assert!(svg.contains("</svg>"), "should have closing tag");

    // Should have rhythm stems (extra lines beyond staff + barlines)
    let line_count = svg.matches("<line ").count();
    // 6 staff lines, barlines, and rhythm stems
    assert!(
        line_count > 12,
        "should have staff lines + barlines + rhythm stems, got {line_count}"
    );

    // Should have flag paths for eighth and sixteenth notes
    let path_count = svg.matches("<path ").count();
    // TAB clef path plus flag paths
    assert!(
        path_count > 2,
        "should have TAB clef paths + flag paths, got {path_count}"
    );
}
