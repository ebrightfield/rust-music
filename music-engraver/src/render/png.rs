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

/// Pixel-content verification helpers shared across PNG-rendering test modules.
///
/// Lives in `png.rs` so multi-staff and tab score tests can reuse the same
/// decode/inspect primitives that the `png` module's own tests use. The
/// alternative (duplicating each helper inline in three test modules) drifts
/// quickly — a tolerance bumped in one place but not another would mask the
/// regression class these helpers exist to catch.
#[cfg(test)]
pub(crate) mod test_helpers {
    use super::tiny_skia;

    /// A pixel is "ink" if its premultiplied alpha is at least this threshold.
    /// 32 is large enough to ignore the long anti-aliasing tail and small
    /// enough to count meaningful coverage — robust to AA changes between
    /// rasterizer versions.
    pub const INK_ALPHA_THRESHOLD: u8 = 32;

    /// Extract pixel width and height from the IHDR chunk (PNG bytes 16–23).
    pub fn png_dimensions(data: &[u8]) -> (u32, u32) {
        assert!(data.len() >= 24, "PNG too short for IHDR");
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        (w, h)
    }

    /// Decode PNG bytes back into a `tiny_skia::Pixmap` for pixel inspection.
    pub fn decode_pixmap(png_bytes: &[u8]) -> tiny_skia::Pixmap {
        tiny_skia::Pixmap::decode_png(png_bytes).expect("decode PNG bytes")
    }

    /// Count pixels whose premultiplied alpha is at or above `alpha_threshold`.
    pub fn count_inked_pixels(pixmap: &tiny_skia::Pixmap, alpha_threshold: u8) -> usize {
        pixmap
            .pixels()
            .iter()
            .filter(|p| p.alpha() >= alpha_threshold)
            .count()
    }

    /// Returns `(x_min, y_min, x_max, y_max)` of inked pixels, inclusive.
    /// `None` when no pixel meets the alpha threshold.
    pub fn inked_bbox(
        pixmap: &tiny_skia::Pixmap,
        alpha_threshold: u8,
    ) -> Option<(u32, u32, u32, u32)> {
        let (w, h) = (pixmap.width(), pixmap.height());
        let mut x_min = u32::MAX;
        let mut y_min = u32::MAX;
        let mut x_max = 0u32;
        let mut y_max = 0u32;
        let mut any = false;
        for y in 0..h {
            for x in 0..w {
                let p = pixmap.pixel(x, y).expect("in-bounds pixel");
                if p.alpha() >= alpha_threshold {
                    any = true;
                    if x < x_min {
                        x_min = x;
                    }
                    if y < y_min {
                        y_min = y;
                    }
                    if x > x_max {
                        x_max = x;
                    }
                    if y > y_max {
                        y_max = y;
                    }
                }
            }
        }
        if any {
            Some((x_min, y_min, x_max, y_max))
        } else {
            None
        }
    }

    /// Returns the number of pixels in row `y` whose alpha meets the threshold.
    pub fn inked_pixels_in_row(
        pixmap: &tiny_skia::Pixmap,
        y: u32,
        alpha_threshold: u8,
    ) -> usize {
        (0..pixmap.width())
            .filter(|&x| {
                pixmap
                    .pixel(x, y)
                    .map(|p| p.alpha() >= alpha_threshold)
                    .unwrap_or(false)
            })
            .count()
    }

    /// Count rows whose inked-pixel density is at least `density_fraction` of
    /// the image width. Useful for counting "staff-line-like" horizontal bands.
    pub fn count_dense_rows(
        pixmap: &tiny_skia::Pixmap,
        alpha_threshold: u8,
        density_fraction: f64,
    ) -> usize {
        let w = pixmap.width() as f64;
        let min_count = (w * density_fraction) as usize;
        (0..pixmap.height())
            .filter(|&y| inked_pixels_in_row(pixmap, y, alpha_threshold) >= min_count)
            .count()
    }
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

    use super::test_helpers::png_dimensions;

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

    // -- Pixel-content verification --
    //
    // The tests above check that PNG bytes are well-formed and that headers
    // report sensible dimensions, but a PNG with all-transparent pixels would
    // still pass them. The tests below decode the PNG back into a pixmap and
    // assert on actual pixel content, catching regressions such as: a blank
    // canvas, a font-loading failure that renders glyphs as invisible, or a
    // rasterizer transform bug that paints into the wrong region.
    //
    // Helpers (`decode_pixmap`, `count_inked_pixels`, `inked_bbox`,
    // `inked_pixels_in_row`, and `INK_ALPHA_THRESHOLD`) live in the sibling
    // `test_helpers` module so multi-staff and tab score tests can reuse them.

    use super::test_helpers::{
        count_inked_pixels, decode_pixmap, inked_bbox, inked_pixels_in_row, INK_ALPHA_THRESHOLD,
    };

    fn score_sparse_whole_rest() -> String {
        // Minimal-ink score: just clef + time sig + a single whole rest (a
        // small filled rectangle — much less ink than note glyphs with stems).
        use crate::score::ScoreBuilder;
        use music::notation::clef::Clef;
        use music::notation::rhythm::duration::Duration;

        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .rest(Duration::WHOLE)
            .end_barline()
            .render_svg()
    }

    fn score_dense_four_notes() -> String {
        // Dense-ink score: 4 quarter notes at distinct pitches → 4 notehead
        // glyphs + 4 stems + (since one note is below the staff and one above)
        // ledger lines, plus the same staff/clef/timesig as the sparse score.
        use crate::score::ScoreBuilder;
        use music::notation::clef::Clef;
        use music::notation::rhythm::duration::Duration;
        use music::note::note::Note;
        use music::note::pitch::Pitch;

        ScoreBuilder::new()
            .clef(Clef::Treble)
            .time_signature(4, 4)
            .note(Pitch::new(Note::C, 4).expect("valid pitch"), Duration::QTR)
            .note(Pitch::new(Note::E, 4).expect("valid pitch"), Duration::QTR)
            .note(Pitch::new(Note::G, 4).expect("valid pitch"), Duration::QTR)
            .note(Pitch::new(Note::C, 5).expect("valid pitch"), Duration::QTR)
            .end_barline()
            .render_svg()
    }

    #[test]
    fn pixmap_dimensions_match_png_header_dimensions() {
        let svg = score_sparse_whole_rest();
        let png = svg_to_png(&svg, 1.0).unwrap();

        let (hdr_w, hdr_h) = png_dimensions(&png);
        let pixmap = decode_pixmap(&png);
        assert_eq!(
            pixmap.width(),
            hdr_w,
            "decoded width must match IHDR width"
        );
        assert_eq!(
            pixmap.height(),
            hdr_h,
            "decoded height must match IHDR height"
        );
    }

    #[test]
    fn score_png_contains_non_zero_inked_pixels() {
        // A blank PNG passes the existing size and dimension tests but fails this one.
        // Catches: font loading failure making all glyphs invisible; rasterizer
        // painting into the wrong region; usvg parse silently emitting an empty tree.
        let svg = score_dense_four_notes();
        let png = svg_to_png(&svg, 1.0).unwrap();
        let pixmap = decode_pixmap(&png);

        let ink_count = count_inked_pixels(&pixmap, INK_ALPHA_THRESHOLD);
        let total = (pixmap.width() * pixmap.height()) as usize;
        assert!(
            ink_count >= 200,
            "score PNG has only {ink_count} inked pixels out of {total}; \
             expected a clef, time signature, four notes, and staff lines"
        );
    }

    #[test]
    fn score_png_is_mostly_transparent_background() {
        // The rendered staff occupies a horizontal band in the middle of a much
        // taller page; if the renderer ever started painting an opaque
        // background, this would fire.
        let svg = score_dense_four_notes();
        let png = svg_to_png(&svg, 1.0).unwrap();
        let pixmap = decode_pixmap(&png);

        let total = (pixmap.width() * pixmap.height()) as usize;
        let transparent = pixmap
            .pixels()
            .iter()
            .filter(|p| p.alpha() == 0)
            .count();
        let transparent_fraction = transparent as f64 / total as f64;
        assert!(
            transparent_fraction > 0.5,
            "expected background to dominate, but only {transparent}/{total} \
             ({:.1}%) pixels are fully transparent",
            100.0 * transparent_fraction
        );
    }

    #[test]
    fn empty_svg_renders_fully_transparent_png() {
        // No drawn elements → every pixel must be alpha == 0. Catches a
        // regression where the rasterizer ever started clearing to opaque white.
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 40 40">
</svg>"#;
        let png = svg_to_png(svg_str, 1.0).unwrap();
        let pixmap = decode_pixmap(&png);

        let opaque_any = pixmap.pixels().iter().any(|p| p.alpha() != 0);
        assert!(
            !opaque_any,
            "empty SVG must yield fully transparent PNG, but some pixel had non-zero alpha"
        );
    }

    #[test]
    fn opaque_filled_rect_renders_all_pixels_opaque() {
        // Sanity check on the encode → decode round-trip: a rect filling the
        // whole viewBox must produce a pixmap where every pixel has alpha 255.
        let svg_str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20">
  <rect x="0" y="0" width="20" height="20" fill="black"/>
</svg>"#;
        let png = svg_to_png(svg_str, 1.0).unwrap();
        let pixmap = decode_pixmap(&png);

        let all_opaque = pixmap.pixels().iter().all(|p| p.alpha() == 255);
        assert!(
            all_opaque,
            "full-canvas opaque rect must yield all alpha=255 pixels"
        );
        // And RGB must be near-black (premultiplied, but black * 1.0 = black).
        let avg_r: u32 = pixmap.pixels().iter().map(|p| p.red() as u32).sum::<u32>()
            / pixmap.pixels().len() as u32;
        assert!(
            avg_r < 16,
            "expected near-black average red channel, got {avg_r}"
        );
    }

    #[test]
    fn red_filled_rect_decodes_as_red_dominant() {
        // Color-channel sanity check: a red rect should decode with R >> G, R >> B.
        // Catches: channel-order bugs (BGRA-as-RGBA swap), premultiplication bugs.
        let svg_str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
  <rect x="0" y="0" width="10" height="10" fill="#ff0000"/>
</svg>"##;
        let png = svg_to_png(svg_str, 1.0).unwrap();
        let pixmap = decode_pixmap(&png);

        let center = pixmap.pixel(5, 5).expect("center pixel");
        assert_eq!(center.alpha(), 255, "red rect center must be opaque");
        assert!(
            center.red() > center.green() + 64,
            "red channel ({}) should dominate green ({})",
            center.red(),
            center.green()
        );
        assert!(
            center.red() > center.blue() + 64,
            "red channel ({}) should dominate blue ({})",
            center.red(),
            center.blue()
        );
    }

    #[test]
    fn inked_pixel_count_scales_roughly_with_area() {
        // 2× scale → ~4× ink pixels (area scaling). Tolerance accounts for
        // anti-aliasing differences at edges. This proves the scale parameter
        // is genuinely changing rasterization, not just metadata.
        let svg = score_dense_four_notes();
        let png_1x = svg_to_png(&svg, 1.0).unwrap();
        let png_2x = svg_to_png(&svg, 2.0).unwrap();

        let ink_1x = count_inked_pixels(&decode_pixmap(&png_1x), INK_ALPHA_THRESHOLD);
        let ink_2x = count_inked_pixels(&decode_pixmap(&png_2x), INK_ALPHA_THRESHOLD);

        assert!(ink_1x > 0 && ink_2x > 0);
        let ratio = ink_2x as f64 / ink_1x as f64;
        // Theoretical area ratio is 4.0× for filled shapes. For thin-stroke
        // content (staff lines, note stems), AA fringe inflates the 1× count
        // proportionally more than the 2× count, pulling the observed ratio
        // toward ~2×. Empirically this score lands at ~2.5×. We bound the
        // ratio in [2.0, 5.5] — the lower bound guards against "scale does
        // nothing" (ratio ≈ 1.0); the upper guards against pathological
        // over-scaling (ratio ≈ 8× would indicate an extra factor of 2 leaked
        // into stroke widths or layout).
        assert!(
            (2.0..=5.5).contains(&ratio),
            "2×/1× ink-pixel ratio is {ratio:.2} (got {ink_2x} vs {ink_1x}); \
             expected somewhere in [2×, 4×] for line-heavy musical content"
        );
    }

    #[test]
    fn score_png_ink_bounding_box_spans_most_of_width() {
        // The clef sits at the left edge of the staff, the end barline at the
        // right edge: inked content must span most of the image. Catches a bug
        // where (e.g.) all content rendered into a single column.
        let svg = score_dense_four_notes();
        let png = svg_to_png(&svg, 1.0).unwrap();
        let pixmap = decode_pixmap(&png);

        let bbox = inked_bbox(&pixmap, INK_ALPHA_THRESHOLD).expect("score PNG should have ink");
        let bbox_width = bbox.2 - bbox.0;
        let image_width = pixmap.width();
        let span_fraction = bbox_width as f64 / image_width as f64;
        assert!(
            span_fraction > 0.5,
            "ink bbox width {bbox_width} is only {:.1}% of image width {image_width}; \
             expected staff to span >50%",
            100.0 * span_fraction
        );
    }

    #[test]
    fn different_scores_produce_different_inked_pixel_counts() {
        // Regression canary: a refactor that accidentally short-circuited
        // rendering (e.g., cached output keyed on something content-independent)
        // would produce the same PNG for any score. Pick two scores with
        // markedly different note density.
        let svg_a = score_sparse_whole_rest();
        let svg_b = score_dense_four_notes();
        let ink_a = count_inked_pixels(&decode_pixmap(&svg_to_png(&svg_a, 1.0).unwrap()), INK_ALPHA_THRESHOLD);
        let ink_b = count_inked_pixels(&decode_pixmap(&svg_to_png(&svg_b, 1.0).unwrap()), INK_ALPHA_THRESHOLD);

        assert_ne!(
            ink_a, ink_b,
            "different scores produced identical inked-pixel counts ({ink_a}); \
             rendering may be short-circuited"
        );
        // Score B has 4 notes + 4 stems + flag-less heads vs. score A's 1 note +
        // 3 rests, so B should have more ink.
        assert!(
            ink_b > ink_a,
            "score with 4 notes ({ink_b} ink) should have more ink than \
             score with 1 note + 3 rests ({ink_a} ink)"
        );
    }

    #[test]
    fn score_png_has_dense_horizontal_band_consistent_with_staff() {
        // Staff lines span the full measure width. If we sweep rows from top to
        // bottom, at least one row should have a substantial fraction of its
        // pixels inked — that's a staff line. Catches a regression where staff
        // lines are dropped or replaced with dashed/intermittent strokes.
        let svg = score_dense_four_notes();
        let png = svg_to_png(&svg, 1.0).unwrap();
        let pixmap = decode_pixmap(&png);

        // We want the densest single row: that's the most line-like.
        let densest = (0..pixmap.height())
            .map(|y| inked_pixels_in_row(&pixmap, y, INK_ALPHA_THRESHOLD))
            .max()
            .unwrap_or(0);
        let width = pixmap.width() as usize;
        let density = densest as f64 / width as f64;
        assert!(
            density > 0.3,
            "densest row has only {densest}/{width} ({:.1}%) inked pixels; \
             expected a staff line to give at least 30% density",
            100.0 * density
        );
    }
}
