//! SVG primitives and math helpers.

use std::f64::consts::PI;

/// Low-level SVG string builder.
///
/// Builds SVG documents by accumulating SVG elements as strings.
pub struct SvgBuilder {
    parts: Vec<String>,
    width: u32,
    height: u32,
}

impl SvgBuilder {
    /// Create a new SVG builder with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let mut parts = Vec::new();
        parts.push(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            width, height, width, height
        ));
        parts.push(
            r#"  <style>
    .note-text { font-family: Arial, sans-serif; font-size: 11px; font-weight: bold; text-anchor: middle; dominant-baseline: middle; }
    .title-text { font-family: Arial, sans-serif; font-size: 14px; font-weight: bold; text-anchor: middle; }
    .fret-text { font-family: Arial, sans-serif; font-size: 10px; text-anchor: middle; }
    .string-text { font-family: Arial, sans-serif; font-size: 11px; font-weight: bold; text-anchor: middle; }
  </style>"#
                .to_string(),
        );

        Self {
            parts,
            width,
            height,
        }
    }

    /// Get the width of the SVG.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the SVG.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Add a circle element.
    pub fn circle(
        &mut self,
        cx: f64,
        cy: f64,
        r: u32,
        fill: &str,
        stroke: &str,
        stroke_width: f64,
    ) -> &mut Self {
        self.parts.push(format!(
            r#"  <circle cx="{:.1}" cy="{:.1}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
            cx, cy, r, fill, stroke, stroke_width
        ));
        self
    }

    /// Add a line element.
    pub fn line(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: &str,
        stroke_width: f64,
        opacity: f64,
    ) -> &mut Self {
        self.parts.push(format!(
            r#"  <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{}" stroke-opacity="{}"/>"#,
            x1, y1, x2, y2, stroke, stroke_width, opacity
        ));
        self
    }

    /// Add a text element.
    pub fn text(&mut self, x: f64, y: f64, content: &str, class: &str) -> &mut Self {
        self.parts.push(format!(
            r#"  <text x="{:.1}" y="{:.1}" class="{}">{}</text>"#,
            x, y, class, content
        ));
        self
    }

    /// Add a text element with a specific fill color.
    pub fn text_colored(
        &mut self,
        x: f64,
        y: f64,
        content: &str,
        class: &str,
        fill: &str,
    ) -> &mut Self {
        self.parts.push(format!(
            r#"  <text x="{:.1}" y="{:.1}" class="{}" fill="{}">{}</text>"#,
            x, y, class, fill, content
        ));
        self
    }

    /// Start a group element.
    pub fn group_start(&mut self, class: &str) -> &mut Self {
        self.parts.push(format!(r#"  <g class="{}">"#, class));
        self
    }

    /// End a group element.
    pub fn group_end(&mut self) -> &mut Self {
        self.parts.push("  </g>".to_string());
        self
    }

    /// Add a rectangle element.
    pub fn rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: &str,
        stroke: &str,
        stroke_width: f64,
    ) -> &mut Self {
        self.parts.push(format!(
            r#"  <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
            x, y, width, height, fill, stroke, stroke_width
        ));
        self
    }

    /// Add a path element.
    pub fn path(&mut self, d: &str, fill: &str, stroke: &str, stroke_width: f64) -> &mut Self {
        self.parts.push(format!(
            r#"  <path d="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
            d, fill, stroke, stroke_width
        ));
        self
    }

    /// Build the final SVG string.
    pub fn build(mut self) -> String {
        self.parts.push("</svg>".to_string());
        self.parts.join("\n")
    }
}

/// Convert a pitch class to coordinates on a circle.
///
/// Places pitch class 0 at the top (12 o'clock position) and proceeds clockwise.
pub fn pc_to_coords(pc_val: u8, cx: f64, cy: f64, radius: f64) -> (f64, f64) {
    // Start at top (12 o'clock = -90 degrees) and go clockwise
    let angle = ((pc_val as f64) * 30.0 - 90.0) * PI / 180.0;
    let x = cx + radius * angle.cos();
    let y = cy + radius * angle.sin();
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_builder_basic() {
        let svg = SvgBuilder::new(100, 100).build();
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("xmlns"));
        assert!(svg.contains("viewBox"));
    }

    #[test]
    fn test_svg_builder_circle() {
        let mut builder = SvgBuilder::new(100, 100);
        builder.circle(50.0, 50.0, 10, "#ff0000", "#000000", 1.0);
        let svg = builder.build();
        assert!(svg.contains("<circle"));
        assert!(svg.contains("fill=\"#ff0000\""));
    }

    #[test]
    fn test_svg_builder_line() {
        let mut builder = SvgBuilder::new(100, 100);
        builder.line(10.0, 10.0, 90.0, 90.0, "#000000", 2.0, 1.0);
        let svg = builder.build();
        assert!(svg.contains("<line"));
    }

    #[test]
    fn test_svg_builder_text() {
        let mut builder = SvgBuilder::new(100, 100);
        builder.text(50.0, 50.0, "Hello", "note-text");
        let svg = builder.build();
        assert!(svg.contains("<text"));
        assert!(svg.contains("Hello"));
    }

    #[test]
    fn test_pc_to_coords_top() {
        // Pc 0 should be at the top (12 o'clock)
        let (x, y) = pc_to_coords(0, 100.0, 100.0, 80.0);
        assert!((x - 100.0).abs() < 0.001); // Should be centered horizontally
        assert!((y - 20.0).abs() < 0.001); // Should be at top (cy - radius)
    }

    #[test]
    fn test_pc_to_coords_right() {
        // Pc 3 should be at 3 o'clock (right)
        let (x, y) = pc_to_coords(3, 100.0, 100.0, 80.0);
        assert!((x - 180.0).abs() < 0.001); // Should be at right (cx + radius)
        assert!((y - 100.0).abs() < 0.001); // Should be centered vertically
    }
}
