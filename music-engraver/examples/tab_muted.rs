/// Renders a tablature score demonstrating muted/dead string notation.
///
/// Muted strings are shown as "x" on the string line, using the same
/// background-rect + centered-text approach as fret numbers. Supports
/// mixed fret+mute chords (e.g. power chords with muted high strings).
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let svg = TabScoreBuilder::guitar()
        // Measure 1: All-mute strum (percussive ghost strum)
        .quarter()
        .mute(1)
        .mute(2)
        .mute(3)
        .mute(4)
        .mute(5)
        .mute(6)
        .next()
        // Power chord with muted high strings: E5 power chord (6th/5th fretted, 4-1 muted)
        .quarter()
        .fret(6, 0)
        .fret(5, 2)
        .fret(4, 2)
        .mute(3)
        .mute(2)
        .mute(1)
        .next()
        // Single muted string between fretted notes
        .quarter()
        .fret(6, 3)
        .mute(5)
        .fret(4, 5)
        .next()
        // Normal frets for contrast
        .quarter()
        .fret(1, 0)
        .fret(2, 1)
        .barline()
        // Measure 2: Muted bass strings with high melody
        .quarter()
        .mute(6)
        .mute(5)
        .fret(1, 5)
        .next()
        .quarter()
        .mute(6)
        .mute(5)
        .fret(1, 7)
        .next()
        // All mutes with rhythm stem
        .eighth()
        .mute(6)
        .mute(5)
        .mute(4)
        .next()
        .eighth()
        .mute(6)
        .mute(5)
        .mute(4)
        .barline()
        // Measure 3: Individual muted strings in sequence
        .quarter()
        .mute(6)
        .next()
        .quarter()
        .mute(5)
        .next()
        .quarter()
        .mute(4)
        .next()
        .quarter()
        .mute(3)
        .barline()
        // Measure 4: Mixed techniques — mutes with slides and bends
        .quarter()
        .fret(6, 3)
        .mute(1)
        .mute(2)
        .next()
        .quarter()
        .fret(6, 5)
        .mute(1)
        .mute(2)
        .next()
        .half()
        .fret(1, 12)
        .mute(6)
        .mute(5)
        .mute(4)
        .end_barline()
        .measures_per_system(2)
        .render_svg();

    // Write to file
    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/tab_muted.svg", &svg).expect("write SVG");

    // Verify structural elements
    let x_count = svg.matches(">x</text>").count();
    let text_count = svg.matches("<text ").count();
    let line_count = svg.matches("<line ").count();
    let rect_count = svg.matches("<rect ").count();

    println!(
        "tab_muted.svg: {} 'x' markers, {} texts total, {} lines, {} rects, {} bytes",
        x_count,
        text_count,
        line_count,
        rect_count,
        svg.len()
    );

    assert!(svg.starts_with("<svg"));
    // 6 all-mute + 3+3 power chord mutes + 1 skip mute + 2+2 bass mutes
    // + 3+3 triple mutes + 4 individual + 2+2+3 mixed = ~30 x markers
    assert!(
        x_count >= 20,
        "should have at least 20 'x' markers, got {x_count}"
    );
    // Each muted string gets a background rect (same as fret numbers)
    assert!(
        rect_count >= x_count,
        "each 'x' should have a background rect, got {rect_count} rects for {x_count} x markers"
    );
    // Verify fret numbers coexist with mutes
    assert!(
        svg.contains(">0</text>"),
        "should render fret number 0 alongside muted strings"
    );
    assert!(
        svg.contains(">12</text>"),
        "should render fret number 12 alongside muted strings"
    );
}
