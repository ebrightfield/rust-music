/// Renders key signatures (sharps and flats) on treble and bass clef staves.
/// Outputs SVG to `examples/output/key_signatures.svg`.
use music::notation::clef::Clef;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::render::{draw_clef, draw_key_signature, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff_width = 20000.0;

    // Two staves: treble (top) and bass (bottom)
    let treble_y = 0.0;
    let bass_y = 2500.0;
    let treble_staff = StaffLayout::from_config(0.0, treble_y, staff_width, &config);
    let bass_staff = StaffLayout::from_config(0.0, bass_y, staff_width, &config);

    let mut svg = SvgWriter::new(2200.0, 700.0, -200.0, -600.0, 21000.0, 6000.0);

    // Draw staff lines
    draw_staff_lines(&mut svg, &treble_staff, &config);
    draw_staff_lines(&mut svg, &bass_staff, &config);

    // Draw clefs
    let treble_clef = ClefLayout::from_clef(Clef::Treble);
    let bass_clef = ClefLayout::from_clef(Clef::Bass);
    draw_clef(&mut svg, &treble_staff, &treble_clef, &font).unwrap();
    draw_clef(&mut svg, &bass_staff, &bass_clef, &font).unwrap();

    // Key signatures to render: 1–7 sharps, then 1–7 flats
    let clef_advance = 1200.0;
    let group_spacing = 2400.0;

    // Render sharps (1 through 4) on treble, then flats (1 through 4) on treble
    // Render sharps (5 through 7) on bass, then flats (5 through 7) on bass
    let treble_keys: Vec<KeySignature> = (1..=4)
        .map(KeySignature::Sharps)
        .chain((1..=4).map(KeySignature::Flats))
        .collect();

    let bass_keys: Vec<KeySignature> = (5..=7)
        .map(KeySignature::Sharps)
        .chain((5..=7).map(KeySignature::Flats))
        .collect();

    let mut x = clef_advance;
    let mut total_treble_paths = 0u32;
    for key in &treble_keys {
        let width = draw_key_signature(&mut svg, &treble_staff, &font, x, key, &Clef::Treble).unwrap();
        assert!(width > 0.0, "key sig should have positive width");
        let count = match key {
            KeySignature::Sharps(n) | KeySignature::Flats(n) => *n as u32,
            KeySignature::Open => 0,
        };
        total_treble_paths += count;
        x += width + group_spacing;
    }

    x = clef_advance;
    let mut total_bass_paths = 0u32;
    for key in &bass_keys {
        let width = draw_key_signature(&mut svg, &bass_staff, &font, x, key, &Clef::Bass).unwrap();
        assert!(width > 0.0, "key sig should have positive width");
        let count = match key {
            KeySignature::Sharps(n) | KeySignature::Flats(n) => *n as u32,
            KeySignature::Open => 0,
        };
        total_bass_paths += count;
        x += width + group_spacing;
    }

    let output = svg.to_svg();
    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write(
        "music-engraver/examples/output/key_signatures.svg",
        &output,
    )
    .unwrap();
    println!("Wrote music-engraver/examples/output/key_signatures.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG: {} bytes, {} <path>, {} <line>",
        output.len(),
        path_count,
        line_count,
    );

    // Expected paths: 2 clefs + treble key sig accidentals + bass key sig accidentals
    // Treble: 1+2+3+4 sharps + 1+2+3+4 flats = 10 + 10 = 20
    // Bass: 5+6+7 sharps + 5+6+7 flats = 18 + 18 = 36
    let expected_paths = 2 + total_treble_paths + total_bass_paths;
    assert_eq!(
        path_count,
        expected_paths as usize,
        "Expected {expected_paths} paths (2 clefs + {total_treble_paths} treble + {total_bass_paths} bass key sig accidentals)"
    );
    // Expected lines: 10 staff lines (5 per staff)
    assert_eq!(line_count, 10, "Expected 10 lines (5 per staff)");
}
