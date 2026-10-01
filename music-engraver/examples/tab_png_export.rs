//! Renders a guitar tablature score to PNG via the `png` feature, exercising
//! the `save_png` convenience method and the DPI-based scale ergonomics.
//!
//! Run with:
//! ```sh
//! cargo run --example tab_png_export --features png
//! ```
//!
//! Produces `examples/output/tab_score.png` (and an SVG alongside it for
//! comparison) rendered at 300 DPI print resolution.

use music_engraver::render::png::{dpi_to_scale, BASELINE_DPI};
use music_engraver::score::tab::TabScoreBuilder;

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    // A short E-minor-pentatonic lick on a six-string guitar.
    let build = || {
        TabScoreBuilder::guitar()
            .quarter()
            .fret(1, 0)
            .next()
            .quarter()
            .fret(1, 3)
            .next()
            .quarter()
            .fret(2, 0)
            .next()
            .quarter()
            .fret(2, 3)
            .barline()
            .quarter()
            .fret(3, 0)
            .next()
            .quarter()
            .fret(3, 2)
            .next()
            .half()
            .fret(2, 0)
            .end_barline()
    };

    // SVG alongside the PNG, for visual comparison.
    let svg_path = out_dir.join("tab_score.svg");
    std::fs::write(&svg_path, build().render_svg()).expect("write SVG");
    println!("Wrote SVG: {}", svg_path.display());

    // Render at 300 DPI (typical print resolution) via the DPI helper, then
    // write straight to disk with the new `save_png` convenience method.
    let print_dpi = 300.0_f32;
    let png_path = out_dir.join("tab_score.png");
    build()
        .save_png(&png_path, dpi_to_scale(print_dpi))
        .expect("save_png should succeed");

    let bytes = std::fs::read(&png_path).expect("read written PNG");
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    println!(
        "Wrote PNG: {} ({} bytes, {width}×{height} px at {print_dpi} DPI; \
         baseline is {BASELINE_DPI} DPI)",
        png_path.display(),
        bytes.len(),
    );
}
