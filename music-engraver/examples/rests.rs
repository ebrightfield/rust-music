/// Renders all 8 rest durations (whole through 128th) on a treble-clef staff.
/// Outputs SVG to `examples/output/`.
use music::notation::clef::Clef;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::render::{draw_clef, draw_rest, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff =
        music_engraver::layout::staff::StaffLayout::from_config(0.0, 0.0, 11000.0, &config);

    let mut svg = SvgWriter::new(1300.0, 300.0, -100.0, -1500.0, 12000.0, 4000.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, &clef, &font).unwrap();

    let start_x = 1500.0;
    let spacing = 1100.0;

    // Draw all 8 rest durations: whole(0), half(1), quarter(2), 8th(3),
    // 16th(4), 32nd(5), 64th(6), 128th(7)
    let labels = [
        "whole", "half", "quarter", "8th", "16th", "32nd", "64th", "128th",
    ];
    for log2 in 0..=7u8 {
        let x = start_x + log2 as f64 * spacing;
        let advance = draw_rest(&mut svg, &staff, &font, x, log2).unwrap();
        assert!(advance > 0.0, "{} rest advance should be positive", labels[log2 as usize]);
    }

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/rests.svg", &output).unwrap();
    println!("Wrote music-engraver/examples/output/rests.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG length: {} bytes, {} <path> elements, {} <line> elements",
        output.len(),
        path_count,
        line_count,
    );

    // Expected: 1 clef + 8 rests = 9 paths
    // Expected: 5 staff lines
    assert_eq!(path_count, 9, "Expected 9 paths (1 clef + 8 rests)");
    assert_eq!(line_count, 5, "Expected 5 lines (staff lines only)");
    println!("Expected: 9 paths (1 clef + 8 rests), 5 lines (staff)");
}
