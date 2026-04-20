//! Renders a short score to both SVG and PNG via the `png` feature.
//!
//! Run with:
//! ```sh
//! cargo run --example png_export --features png
//! ```
//!
//! Produces `examples/output/score.png` alongside the SVG equivalent.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::score::ScoreBuilder;

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    // Build a 2-measure score in D major.
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .key_signature(KeySignature::Sharps(2))
        .time_signature(4, 4)
        .note(
            Pitch::new(Note::D, 4).expect("valid pitch"),
            Duration::QTR,
        )
        .note(
            Pitch::new(Note::Fis, 4).expect("valid pitch"),
            Duration::QTR,
        )
        .note(
            Pitch::new(Note::A, 4).expect("valid pitch"),
            Duration::HALF,
        )
        .barline()
        .note(
            Pitch::new(Note::B, 4).expect("valid pitch"),
            Duration::QTR,
        )
        .note(
            Pitch::new(Note::A, 4).expect("valid pitch"),
            Duration::QTR,
        )
        .note(
            Pitch::new(Note::Fis, 4).expect("valid pitch"),
            Duration::QTR,
        )
        .note(
            Pitch::new(Note::D, 4).expect("valid pitch"),
            Duration::QTR,
        )
        .end_barline()
        .render_svg();

    // Write SVG for comparison.
    let svg_path = out_dir.join("score.svg");
    std::fs::write(&svg_path, &svg).expect("write SVG");
    println!("Wrote SVG: {} ({} bytes)", svg_path.display(), svg.len());

    // Render PNG at 2× scale (retina).
    let png_bytes = {
        use music_engraver::render::png::PngRenderer;
        let mut renderer = PngRenderer::new(2.0);
        renderer.load_system_fonts();
        renderer
            .render_png(&svg)
            .expect("PNG rendering should succeed")
    };

    let png_path = out_dir.join("score.png");
    std::fs::write(&png_path, &png_bytes).expect("write PNG");
    println!(
        "Wrote PNG: {} ({} bytes)",
        png_path.display(),
        png_bytes.len()
    );

    // Parse IHDR to report pixel dimensions.
    let width = u32::from_be_bytes([png_bytes[16], png_bytes[17], png_bytes[18], png_bytes[19]]);
    let height = u32::from_be_bytes([png_bytes[20], png_bytes[21], png_bytes[22], png_bytes[23]]);
    println!("PNG dimensions: {width}×{height} px");
}
