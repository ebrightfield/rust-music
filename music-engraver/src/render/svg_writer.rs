use std::fmt::Write;

/// Minimal SVG document builder.
///
/// Accumulates SVG elements and produces a complete SVG document string.
/// All coordinates are in abstract units; the caller sets the viewBox
/// to map those units to the desired pixel size.
pub struct SvgWriter {
    width: f64,
    height: f64,
    view_box: (f64, f64, f64, f64),
    elements: String,
}

impl SvgWriter {
    /// Create a new SVG writer with the given pixel dimensions and viewBox.
    pub fn new(width: f64, height: f64, vb_x: f64, vb_y: f64, vb_w: f64, vb_h: f64) -> Self {
        Self {
            width,
            height,
            view_box: (vb_x, vb_y, vb_w, vb_h),
            elements: String::with_capacity(4096),
        }
    }

    /// Add a raw SVG element string.
    pub fn add_raw(&mut self, svg_fragment: &str) {
        self.elements.push_str(svg_fragment);
        self.elements.push('\n');
    }

    /// Add a `<path>` element with the given path data, fill color, and optional transform.
    pub fn add_path(&mut self, path_data: &str, fill: &str, transform: Option<&str>) {
        let _ = write!(self.elements, r#"  <path d="{path_data}" fill="{fill}""#);
        if let Some(t) = transform {
            let _ = write!(self.elements, r#" transform="{t}""#);
        }
        self.elements.push_str("/>\n");
    }

    /// Add a horizontal line (useful for staff lines).
    pub fn add_line(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: &str,
        stroke_width: f64,
    ) {
        let _ = write!(
            self.elements,
            r#"  <line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{stroke_width}"/>"#
        );
        self.elements.push('\n');
    }

    /// Produce the complete SVG document as a string.
    pub fn to_svg(&self) -> String {
        let (vx, vy, vw, vh) = self.view_box;
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="{vx} {vy} {vw} {vh}">
{elements}</svg>
"#,
            w = self.width,
            h = self.height,
            vx = vx,
            vy = vy,
            vw = vw,
            vh = vh,
            elements = self.elements,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_writer_produces_valid_svg_document() {
        let mut w = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 1000.0, 1000.0);
        w.add_path("M0 0L100 100Z", "black", None);
        let svg = w.to_svg();

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("width=\"200\""));
        assert!(svg.contains("height=\"200\""));
        assert!(svg.contains("viewBox=\"0 0 1000 1000\""));
        assert!(svg.contains(r#"<path d="M0 0L100 100Z" fill="black"/>"#));
        assert!(svg.ends_with("</svg>\n"));
    }

    #[test]
    fn svg_writer_path_with_transform() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        w.add_path("M0 0L10 10Z", "red", Some("translate(50, 50)"));
        let svg = w.to_svg();
        assert!(svg.contains(r#"transform="translate(50, 50)""#));
    }

    #[test]
    fn svg_writer_line_element() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        w.add_line(0.0, 50.0, 100.0, 50.0, "#000", 2.0);
        let svg = w.to_svg();
        assert!(svg.contains(r#"x1="0""#));
        assert!(svg.contains(r#"y1="50""#));
        assert!(svg.contains(r#"x2="100""#));
        assert!(svg.contains(r##"stroke="#000""##));
        assert!(svg.contains(r#"stroke-width="2""#));
    }
}
