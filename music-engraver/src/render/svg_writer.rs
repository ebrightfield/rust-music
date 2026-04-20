use std::fmt::Write;

/// Font and styling properties for an SVG `<text>` element.
///
/// Groups the typographic attributes that tend to travel together,
/// keeping `SvgWriter` method signatures small.
#[derive(Clone, Debug)]
pub struct TextStyle<'a> {
    pub font_family: &'a str,
    pub font_size: f64,
    pub fill: &'a str,
    pub anchor: &'a str,
    pub font_weight: &'a str,
    pub font_style: &'a str,
}

impl<'a> TextStyle<'a> {
    /// Normal-weight, upright text in a serif font, black, centered.
    pub fn normal(font_size: f64) -> Self {
        Self {
            font_family: "serif",
            font_size,
            fill: "black",
            anchor: "middle",
            font_weight: "normal",
            font_style: "normal",
        }
    }

    /// Bold, upright text in a serif font, black, left-aligned.
    pub fn bold(font_size: f64) -> Self {
        Self {
            font_family: "serif",
            font_size,
            fill: "black",
            anchor: "start",
            font_weight: "bold",
            font_style: "normal",
        }
    }

    /// Normal-weight, italic text in a serif font, black, centered.
    pub fn italic(font_size: f64) -> Self {
        Self {
            font_family: "serif",
            font_size,
            fill: "black",
            anchor: "middle",
            font_weight: "normal",
            font_style: "italic",
        }
    }
}

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

    /// Add a filled rectangle (axis-aligned).
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64, fill: &str) {
        let _ = write!(
            self.elements,
            r#"  <rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{fill}"/>"#
        );
        self.elements.push('\n');
    }

    /// Add a filled polygon defined by an array of (x, y) points.
    pub fn add_polygon(&mut self, points: &[(f64, f64)], fill: &str) {
        let _ = write!(self.elements, r#"  <polygon points=""#);
        for (i, &(x, y)) in points.iter().enumerate() {
            if i > 0 {
                self.elements.push(' ');
            }
            let _ = write!(self.elements, "{x},{y}");
        }
        let _ = write!(self.elements, r#"" fill="{fill}"/>"#);
        self.elements.push('\n');
    }

    /// Add a `<path>` element with pre-built path data, a fill color, and no stroke.
    /// Unlike `add_path`, this accepts a `String` directly to avoid double-allocation
    /// when the caller has already assembled the path data.
    pub fn add_filled_path(&mut self, path_data: &str, fill: &str) {
        let _ = write!(
            self.elements,
            r#"  <path d="{path_data}" fill="{fill}" stroke="none"/>"#,
        );
        self.elements.push('\n');
    }

    /// Add a `<text>` element at the given position with the given style.
    ///
    /// Font size is in the same coordinate system as the viewBox.
    pub fn add_text(&mut self, x: f64, y: f64, text: &str, style: &TextStyle<'_>) {
        let _ = write!(
            self.elements,
            r#"  <text x="{x}" y="{y}" font-family="{ff}" font-size="{fs}" fill="{fill}" text-anchor="{anchor}" font-weight="{fw}" font-style="{fst}">{text}</text>"#,
            ff = style.font_family,
            fs = style.font_size,
            fill = style.fill,
            anchor = style.anchor,
            fw = style.font_weight,
            fst = style.font_style,
        );
        self.elements.push('\n');
    }

    /// Add a stroked rectangle (for boxed rehearsal marks, etc.).
    #[allow(clippy::too_many_arguments)]
    pub fn add_stroked_rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: &str,
        stroke: &str,
        stroke_width: f64,
    ) {
        let _ = write!(
            self.elements,
            r#"  <rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}"/>"#,
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

    #[test]
    fn svg_writer_rect_element() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        w.add_rect(10.0, 20.0, 50.0, 30.0, "blue");
        let svg = w.to_svg();
        assert!(svg.contains(r#"<rect "#));
        assert!(svg.contains(r#"x="10""#));
        assert!(svg.contains(r#"y="20""#));
        assert!(svg.contains(r#"width="50""#));
        assert!(svg.contains(r#"height="30""#));
        assert!(svg.contains(r#"fill="blue""#));
    }

    #[test]
    fn svg_writer_polygon_element() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        w.add_polygon(&[(0.0, 0.0), (50.0, 0.0), (50.0, 30.0), (0.0, 30.0)], "red");
        let svg = w.to_svg();
        assert!(svg.contains(r#"<polygon "#));
        assert!(svg.contains(r#"points="0,0 50,0 50,30 0,30""#));
        assert!(svg.contains(r#"fill="red""#));
    }

    #[test]
    fn svg_writer_polygon_empty_points() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        w.add_polygon(&[], "black");
        let svg = w.to_svg();
        assert!(svg.contains(r#"points="""#));
    }

    #[test]
    fn svg_writer_text_element() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 1000.0, 1000.0);
        w.add_text(250.0, 100.0, "A", &TextStyle::normal(200.0));
        let svg = w.to_svg();
        assert!(svg.contains("<text "));
        assert!(svg.contains(r#"x="250""#));
        assert!(svg.contains(r#"y="100""#));
        assert!(svg.contains(r#"font-family="serif""#));
        assert!(svg.contains(r#"font-size="200""#));
        assert!(svg.contains(r#"text-anchor="middle""#));
        assert!(svg.contains(">A</text>"));
    }

    #[test]
    fn svg_writer_styled_text_element() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 1000.0, 1000.0);
        let style = TextStyle {
            font_family: "Times",
            font_size: 150.0,
            fill: "black",
            anchor: "start",
            font_weight: "normal",
            font_style: "italic",
        };
        w.add_text(50.0, 60.0, "cresc.", &style);
        let svg = w.to_svg();
        assert!(svg.contains(r#"font-weight="normal""#));
        assert!(svg.contains(r#"font-style="italic""#));
        assert!(svg.contains(">cresc.</text>"));
    }

    #[test]
    fn svg_writer_stroked_rect_element() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 1000.0, 1000.0);
        w.add_stroked_rect(10.0, 20.0, 300.0, 200.0, "none", "black", 5.0);
        let svg = w.to_svg();
        assert!(svg.contains(r#"<rect "#));
        assert!(svg.contains(r#"fill="none""#));
        assert!(svg.contains(r#"stroke="black""#));
        assert!(svg.contains(r#"stroke-width="5""#));
        assert!(svg.contains(r#"width="300""#));
    }

    #[test]
    fn svg_writer_text_with_special_chars() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        let style = TextStyle {
            font_family: "sans-serif",
            ..TextStyle::normal(50.0)
        };
        w.add_text(0.0, 0.0, "12", &style);
        let svg = w.to_svg();
        assert!(svg.contains(">12</text>"));
    }

    #[test]
    fn text_style_normal_defaults() {
        let s = TextStyle::normal(100.0);
        assert_eq!(s.font_family, "serif");
        assert_eq!(s.font_size, 100.0);
        assert_eq!(s.fill, "black");
        assert_eq!(s.anchor, "middle");
        assert_eq!(s.font_weight, "normal");
        assert_eq!(s.font_style, "normal");
    }

    #[test]
    fn text_style_bold_defaults() {
        let s = TextStyle::bold(200.0);
        assert_eq!(s.font_weight, "bold");
        assert_eq!(s.anchor, "start");
        assert_eq!(s.font_size, 200.0);
    }

    #[test]
    fn text_style_italic_defaults() {
        let s = TextStyle::italic(150.0);
        assert_eq!(s.font_style, "italic");
        assert_eq!(s.font_weight, "normal");
        assert_eq!(s.anchor, "middle");
    }

    #[test]
    fn text_style_override_via_struct_update() {
        let s = TextStyle {
            fill: "red",
            anchor: "end",
            ..TextStyle::bold(120.0)
        };
        assert_eq!(s.fill, "red");
        assert_eq!(s.anchor, "end");
        assert_eq!(s.font_weight, "bold");
        assert_eq!(s.font_size, 120.0);
    }

    #[test]
    fn add_text_emits_all_style_attributes() {
        let mut w = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 1000.0, 1000.0);
        let style = TextStyle {
            font_family: "Helvetica",
            font_size: 300.0,
            fill: "blue",
            anchor: "end",
            font_weight: "bold",
            font_style: "italic",
        };
        w.add_text(10.0, 20.0, "Test", &style);
        let svg = w.to_svg();
        assert!(svg.contains(r#"font-family="Helvetica""#));
        assert!(svg.contains(r#"font-size="300""#));
        assert!(svg.contains(r#"fill="blue""#));
        assert!(svg.contains(r#"text-anchor="end""#));
        assert!(svg.contains(r#"font-weight="bold""#));
        assert!(svg.contains(r#"font-style="italic""#));
        assert!(svg.contains(">Test</text>"));
    }
}
