/// Renders time signatures (4/4, 6/8, 12/8, common, cut common) on a treble-clef staff.
/// Outputs SVG to `examples/output/`.
use music::notation::clef::Clef;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::time_signature::TimeSignatureKind;
use music_engraver::render::{draw_clef, draw_staff_lines, draw_time_signature, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = music_engraver::layout::staff::StaffLayout::from_config(0.0, 0.0, 12000.0, &config);

    let mut svg = SvgWriter::new(1400.0, 300.0, -100.0, -1500.0, 13000.0, 4000.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, &clef, &font).unwrap();

    let signatures: Vec<(TimeSignatureKind, &str)> = vec![
        (
            TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            },
            "4/4",
        ),
        (
            TimeSignatureKind::Numeric {
                numerator: 6,
                denominator: 8,
            },
            "6/8",
        ),
        (
            TimeSignatureKind::Numeric {
                numerator: 12,
                denominator: 8,
            },
            "12/8",
        ),
        (TimeSignatureKind::Common, "common"),
        (TimeSignatureKind::CutCommon, "cut common"),
    ];

    let start_x = 1500.0;
    let spacing = 2000.0;

    for (i, (kind, label)) in signatures.iter().enumerate() {
        let x = start_x + i as f64 * spacing;
        let width = draw_time_signature(&mut svg, &staff, &font, x, kind).unwrap();
        assert!(
            width > 0.0,
            "{} time signature should have positive width",
            label
        );
    }

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write(
        "music-engraver/examples/output/time_signatures.svg",
        &output,
    )
    .unwrap();
    println!("Wrote music-engraver/examples/output/time_signatures.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG length: {} bytes, {} <path> elements, {} <line> elements",
        output.len(),
        path_count,
        line_count,
    );

    // Expected paths: 1 clef + 2 (4/4) + 2 (6/8) + 3 (12/8) + 1 (common) + 1 (cut common) = 10
    // Expected lines: 5 staff lines
    assert_eq!(
        path_count, 10,
        "Expected 10 paths (1 clef + 9 time sig glyphs)"
    );
    assert_eq!(line_count, 5, "Expected 5 lines (staff only)");
}
