/// Renders all 5 barline styles on a treble-clef staff.
/// Outputs SVG to `examples/output/`.
use music::notation::clef::Clef;

use music_engraver::font::bravura_font;
use music_engraver::layout::barline::BarlineStyle;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::render::{draw_barline, draw_clef, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff =
        music_engraver::layout::staff::StaffLayout::from_config(0.0, 0.0, 9000.0, &config);

    let mut svg = SvgWriter::new(1100.0, 300.0, -100.0, -1500.0, 10000.0, 4000.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, &clef, &font).unwrap();

    let styles = [
        (BarlineStyle::Single, "single"),
        (BarlineStyle::Double, "double"),
        (BarlineStyle::Final, "final"),
        (BarlineStyle::StartRepeat, "start repeat"),
        (BarlineStyle::EndRepeat, "end repeat"),
    ];

    let start_x = 1500.0;
    let spacing = 1500.0;

    for (i, (style, label)) in styles.iter().enumerate() {
        let x = start_x + i as f64 * spacing;
        let width = draw_barline(&mut svg, &staff, &font, x, *style).unwrap();
        assert!(width > 0.0, "{} barline should have positive width", label);
    }

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/barlines.svg", &output).unwrap();
    println!("Wrote music-engraver/examples/output/barlines.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG length: {} bytes, {} <path> elements, {} <line> elements",
        output.len(),
        path_count,
        line_count,
    );

    // Expected paths: 1 clef + 2 dots (start repeat) + 2 dots (end repeat) = 5
    // Expected lines: 5 staff + 1 single + 2 double + 2 final + 2 start repeat + 2 end repeat = 14
    assert_eq!(path_count, 5, "Expected 5 paths (1 clef + 4 repeat dots)");
    assert_eq!(line_count, 14, "Expected 14 lines (5 staff + 9 barline strokes)");
    println!("Expected: 5 paths, 14 lines");
}
