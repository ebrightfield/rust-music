//! Minimal end-to-end example: extract noteheadBlack from Bravura and write an SVG.

use music_engraver::font::{bravura_font, GlyphOutline};
use music_engraver::render::SvgWriter;
use smufl::Glyph;
use std::fs;
use std::path::Path;

fn main() {
    let font = bravura_font();
    let upm = font.units_per_em() as f64;
    let staff_space = upm / 4.0; // 250 font units for Bravura

    // Extract the notehead glyph outline.
    let outline: GlyphOutline = font
        .glyph_outline(Glyph::NoteheadBlack)
        .expect("noteheadBlack should exist in Bravura");

    let margin = 100.0;
    let staff_width = 1200.0;
    let vb_x = -margin;
    let vb_y = -2.0 * staff_space - margin;
    let vb_w = staff_width + 2.0 * margin;
    let vb_h = 4.0 * staff_space + 2.0 * margin;

    let mut writer = SvgWriter::new(400.0, 200.0, vb_x, vb_y, vb_w, vb_h);

    // Draw 5 staff lines
    for i in -2i32..=2 {
        let y = i as f64 * staff_space;
        writer.add_line(0.0, y, staff_width, y, "#999", 10.0);
    }

    // Place the notehead centered on the middle staff line
    writer.add_path(
        &outline.path_data,
        "black",
        Some("translate(400, 0)"),
    );

    let svg = writer.to_svg();

    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    fs::create_dir_all(&out_dir).expect("create output dir");
    let out_path = out_dir.join("notehead_black.svg");
    fs::write(&out_path, &svg).expect("write SVG");
    println!("Wrote {}", out_path.display());
}
