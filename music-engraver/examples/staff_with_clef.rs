//! Renders a staff with treble, bass, alto, and tenor clefs to SVG files.

use music::notation::clef::Clef;
use music_engraver::font::bravura_font;
use music_engraver::layout::{ClefLayout, StaffLayout};
use music_engraver::render::{draw_clef, draw_staff_lines, SvgWriter};
use std::fs;
use std::path::Path;

fn render_staff_with_clef(clef: Clef, filename: &str) {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff_space = config.staff_space;

    // Leave room above and below for clef extenders (G clef extends well above staff)
    let margin = 3.0 * staff_space;
    let staff_width = 20.0 * staff_space;
    let staff = StaffLayout::from_config(0.0, margin, staff_width, &config);

    let vb_w = staff_width + 2.0 * margin;
    let vb_h = staff.height() + 2.0 * margin;

    let mut svg = SvgWriter::new(800.0, 200.0, -margin, 0.0, vb_w, vb_h);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef_layout = ClefLayout::from_clef(clef);
    draw_clef(&mut svg, &staff, &clef_layout, &font).expect("draw clef");

    let output = svg.to_svg();
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    fs::create_dir_all(&out_dir).expect("create output dir");
    let out_path = out_dir.join(filename);
    fs::write(&out_path, &output).expect("write SVG");
    println!("Wrote {}", out_path.display());
}

fn main() {
    render_staff_with_clef(Clef::Treble, "staff_treble_clef.svg");
    render_staff_with_clef(Clef::Bass, "staff_bass_clef.svg");
    render_staff_with_clef(Clef::Alto, "staff_alto_clef.svg");
    render_staff_with_clef(Clef::Tenor, "staff_tenor_clef.svg");
}
