//! PNG rasterization of SVG output.
//!
//! Available only when the `png` feature is enabled. Converts an SVG string
//! (as produced by [`SvgWriter::to_svg`](super::SvgWriter::to_svg) or
//! [`ScoreBuilder::render_svg`](crate::score::ScoreBuilder::render_svg))
//! into PNG image bytes using `resvg` + `tiny-skia`.
//!
//! The bundled Bravura OTF is registered with the font database so that any
//! `<text>` elements referencing Bravura render correctly. System fonts are
//! not loaded by default — call [`PngRenderer::load_system_fonts`] if your
//! SVG uses non-bundled font families (e.g. serif text in rehearsal marks
//! or expression markings).

use std::sync::Arc;

use fontdb::Database;
use resvg::tiny_skia;
use resvg::usvg;

use crate::font::BRAVURA_OTF;

/// Errors that can occur during PNG rendering.
#[derive(Debug, thiserror::Error)]
pub enum PngError {
    /// The SVG string could not be parsed by usvg.
    #[error("SVG parse error: {0}")]
    SvgParse(String),

    /// The rasterized image has zero dimensions.
    #[error("rendered image has zero dimensions ({width}x{height})")]
    ZeroDimensions { width: u32, height: u32 },

    /// PNG encoding failed.
    #[error("PNG encoding error: {0}")]
    Encode(String),
}

/// Rasterizes SVG strings to PNG bytes.
///
/// Manages a font database and rendering options. The bundled Bravura font
/// is always available; call [`load_system_fonts`](Self::load_system_fonts)
/// to add the host system's fonts (needed for serif/sans-serif text in
/// rehearsal marks, tempo markings, and expression text).
pub struct PngRenderer {
    fontdb: Database,
    /// Scale factor applied to the SVG dimensions (1.0 = native viewBox size).
    scale: f32,
}

impl PngRenderer {
    /// Create a new renderer with the bundled Bravura font loaded.
    ///
    /// Scale factor `1.0` renders at the SVG's intrinsic pixel dimensions.
    /// Use `2.0` for retina/HiDPI output.
    pub fn new(scale: f32) -> Self {
        let mut fontdb = Database::new();
        fontdb.load_font_data(BRAVURA_OTF.to_vec());
        Self { fontdb, scale }
    }

    /// Load system fonts into the font database.
    ///
    /// Required if the SVG contains `<text>` elements that reference
    /// font families other than Bravura (e.g. "serif", "sans-serif",
    /// "Times New Roman"). Without this, such text may render as
    /// invisible or use a fallback glyph.
    pub fn load_system_fonts(&mut self) {
        self.fontdb.load_system_fonts();
    }

    /// Load a custom font from raw bytes.
    pub fn load_font_data(&mut self, data: Vec<u8>) {
        self.fontdb.load_font_data(data);
    }

    /// Render an SVG string to PNG bytes.
    ///
    /// Returns the raw PNG file contents suitable for writing to disk
    /// or embedding in other formats.
    pub fn render_png(&self, svg: &str) -> Result<Vec<u8>, PngError> {
        let options = usvg::Options {
            fontdb: Arc::new(self.fontdb.clone()),
            ..usvg::Options::default()
        };

        let tree = usvg::Tree::from_str(svg, &options)
            .map_err(|e| PngError::SvgParse(e.to_string()))?;

        let size = tree.size();
        let width = (size.width() * self.scale) as u32;
        let height = (size.height() * self.scale) as u32;

        if width == 0 || height == 0 {
            return Err(PngError::ZeroDimensions { width, height });
        }

        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or(PngError::ZeroDimensions { width, height })?;

        let transform = tiny_skia::Transform::from_scale(self.scale, self.scale);
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        pixmap
            .encode_png()
            .map_err(|e| PngError::Encode(e.to_string()))
    }
}

impl Default for PngRenderer {
    /// Creates a renderer at 1x scale with the bundled Bravura font.
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// Convenience function: render SVG to PNG at the given scale factor.
///
/// Loads only the bundled Bravura font (no system fonts). For more control,
/// use [`PngRenderer`] directly.
pub fn svg_to_png(svg: &str, scale: f32) -> Result<Vec<u8>, PngError> {
    let renderer = PngRenderer::new(scale);
    renderer.render_png(svg)
}

/// Check whether the `png` feature is available (always `true` when this
/// module is compiled). Useful for runtime feature detection in downstream
/// crates.
///
/// This function exists so callers don't need `#[cfg(feature = "png")]`
/// guards — they can call this at runtime instead.
pub const fn is_available() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::SvgWriter;

    fn minimal_svg() -> String {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        w.add_rect(10.0, 10.0, 80.0, 80.0, "black");
        w.to_svg()
    }

    #[test]
    fn png_renderer_default_has_scale_one() {
        let r = PngRenderer::default();
        assert!((r.scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn render_minimal_svg_to_png() {
        let svg = minimal_svg();
        let png_bytes = svg_to_png(&svg, 1.0).unwrap();

        // PNG magic bytes: 0x89 P N G \r \n 0x1A \n
        assert!(png_bytes.len() > 8, "PNG output too small");
        assert_eq!(&png_bytes[0..4], &[0x89, b'P', b'N', b'G']);
        assert_eq!(&png_bytes[4..8], &[0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn render_at_2x_scale_produces_larger_image() {
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="30" viewBox="0 0 50 30">
  <rect x="0" y="0" width="50" height="30" fill="red"/>
</svg>"#;

        let png_1x = svg_to_png(svg_str, 1.0).unwrap();
        let png_2x = svg_to_png(svg_str, 2.0).unwrap();

        // 2x should produce more bytes (larger pixel buffer)
        assert!(
            png_2x.len() > png_1x.len(),
            "2x PNG ({}) should be larger than 1x ({})",
            png_2x.len(),
            png_1x.len()
        );

        // Both should be valid PNGs
        assert_eq!(&png_1x[0..4], &[0x89, b'P', b'N', b'G']);
        assert_eq!(&png_2x[0..4], &[0x89, b'P', b'N', b'G']);
    }

    #[test]
    fn invalid_svg_returns_parse_error() {
        let result = svg_to_png("not valid svg at all", 1.0);
        assert!(result.is_err());
        match result.unwrap_err() {
            PngError::SvgParse(msg) => assert!(!msg.is_empty()),
            other => panic!("expected SvgParse, got: {other:?}"),
        }
    }

    #[test]
    fn render_svg_with_path_elements() {
        let mut w = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 1000.0, 1000.0);
        w.add_path("M100 100 L200 100 L200 200 Z", "black", None);
        w.add_line(0.0, 500.0, 1000.0, 500.0, "black", 10.0);
        let svg = w.to_svg();

        let png_bytes = svg_to_png(&svg, 1.0).unwrap();
        assert_eq!(&png_bytes[0..4], &[0x89, b'P', b'N', b'G']);
        assert!(png_bytes.len() > 100, "PNG should contain meaningful content");
    }

    #[test]
    fn convenience_function_matches_renderer() {
        let svg = minimal_svg();
        let via_fn = svg_to_png(&svg, 1.0).unwrap();
        let via_struct = PngRenderer::new(1.0).render_png(&svg).unwrap();

        // Same input + same scale should produce identical output
        assert_eq!(via_fn, via_struct);
    }

    /// Extract pixel width and height from the IHDR chunk (bytes 16–23).
    fn png_dimensions(data: &[u8]) -> (u32, u32) {
        assert!(data.len() >= 24, "PNG too short for IHDR");
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        (w, h)
    }

    #[test]
    fn is_available_returns_true() {
        assert!(is_available());
    }

    #[test]
    fn dimensions_match_svg_viewbox_at_1x() {
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80">
  <rect x="0" y="0" width="120" height="80" fill="blue"/>
</svg>"#;
        let png = svg_to_png(svg_str, 1.0).unwrap();
        let (w, h) = png_dimensions(&png);
        assert_eq!(w, 120, "width at 1× should match SVG width");
        assert_eq!(h, 80, "height at 1× should match SVG height");
    }

    #[test]
    fn dimensions_double_at_2x_scale() {
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="40" viewBox="0 0 60 40">
  <rect x="0" y="0" width="60" height="40" fill="green"/>
</svg>"#;
        let png_2x = svg_to_png(svg_str, 2.0).unwrap();
        let (w, h) = png_dimensions(&png_2x);
        assert_eq!(w, 120, "width at 2× should be 2 * 60");
        assert_eq!(h, 80, "height at 2× should be 2 * 40");
    }

    #[test]
    fn dimensions_at_3x_scale() {
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="20" viewBox="0 0 30 20">
  <rect x="0" y="0" width="30" height="20" fill="red"/>
</svg>"#;
        let png_3x = svg_to_png(svg_str, 3.0).unwrap();
        let (w, h) = png_dimensions(&png_3x);
        assert_eq!(w, 90, "width at 3× should be 3 * 30");
        assert_eq!(h, 60, "height at 3× should be 3 * 20");
    }

    #[test]
    fn score_png_has_nonzero_dimensions() {
        use crate::score::ScoreBuilder;
        use music::notation::clef::Clef;
        use music::notation::rhythm::duration::Duration;
        use music::note::note::Note;
        use music::note::pitch::Pitch;

        let png = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4).expect("valid pitch"), Duration::QTR)
            .rest(Duration::QTR)
            .end_barline()
            .render_png(1.0);

        let (w, h) = png_dimensions(&png);
        assert!(w > 50, "score PNG width should be at least 50px, got {w}");
        assert!(h > 20, "score PNG height should be at least 20px, got {h}");
    }

    #[test]
    fn score_svg_renders_to_png() {
        use crate::score::ScoreBuilder;
        use music::notation::clef::Clef;
        use music::notation::rhythm::duration::Duration;
        use music::note::note::Note;
        use music::note::pitch::Pitch;

        let svg = ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4).expect("valid pitch"), Duration::QTR)
            .note(Pitch::new(Note::E, 4).expect("valid pitch"), Duration::QTR)
            .note(Pitch::new(Note::G, 4).expect("valid pitch"), Duration::QTR)
            .rest(Duration::QTR)
            .end_barline()
            .render_svg();

        let mut renderer = PngRenderer::new(2.0);
        renderer.load_system_fonts();
        let png_bytes = renderer.render_png(&svg).unwrap();

        assert_eq!(&png_bytes[0..4], &[0x89, b'P', b'N', b'G']);
        // A real score PNG with notes/staff should be non-trivial in size
        assert!(
            png_bytes.len() > 500,
            "score PNG too small: {} bytes",
            png_bytes.len()
        );
    }
}
