use ttf_parser::OutlineBuilder;

/// SVG path data extracted from a font glyph outline.
///
/// Coordinates are in font units (typically 1000 units per em for CFF fonts).
/// The y-axis is flipped relative to SVG convention: font coordinates have y
/// increasing upward, but the path data stored here has already been flipped
/// so it can be used directly in SVG (y increasing downward).
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphOutline {
    pub path_data: String,
    pub advance_width: u16,
}

/// Collects ttf-parser outline callbacks into an SVG path data string.
///
/// Y-coordinates are negated during collection because font coordinate
/// systems have y-up, while SVG has y-down.
pub(crate) struct SvgPathBuilder {
    path: String,
}

impl SvgPathBuilder {
    pub fn new() -> Self {
        Self {
            path: String::with_capacity(512),
        }
    }

    pub fn into_path_data(self) -> String {
        self.path
    }
}

impl OutlineBuilder for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.push('M');
        self.path.push_str(&format_coord(x));
        self.path.push(' ');
        self.path.push_str(&format_coord(-y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path.push('L');
        self.path.push_str(&format_coord(x));
        self.path.push(' ');
        self.path.push_str(&format_coord(-y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path.push('Q');
        self.path.push_str(&format_coord(x1));
        self.path.push(' ');
        self.path.push_str(&format_coord(-y1));
        self.path.push(' ');
        self.path.push_str(&format_coord(x));
        self.path.push(' ');
        self.path.push_str(&format_coord(-y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path.push('C');
        self.path.push_str(&format_coord(x1));
        self.path.push(' ');
        self.path.push_str(&format_coord(-y1));
        self.path.push(' ');
        self.path.push_str(&format_coord(x2));
        self.path.push(' ');
        self.path.push_str(&format_coord(-y2));
        self.path.push(' ');
        self.path.push_str(&format_coord(x));
        self.path.push(' ');
        self.path.push_str(&format_coord(-y));
    }

    fn close(&mut self) {
        self.path.push('Z');
    }
}

/// Format a coordinate, omitting unnecessary trailing zeros.
fn format_coord(v: f32) -> String {
    if v == v.round() && v.abs() < 1_000_000.0 {
        format!("{}", v as i32)
    } else {
        // Use up to 2 decimal places, trim trailing zeros
        let s = format!("{:.2}", v);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_coord_integers() {
        assert_eq!(format_coord(0.0), "0");
        assert_eq!(format_coord(100.0), "100");
        assert_eq!(format_coord(-42.0), "-42");
    }

    #[test]
    fn format_coord_decimals() {
        assert_eq!(format_coord(1.5), "1.5");
        assert_eq!(format_coord(1.25), "1.25");
        assert_eq!(format_coord(1.10), "1.1");
    }

    #[test]
    fn svg_path_builder_produces_valid_path() {
        let mut builder = SvgPathBuilder::new();
        builder.move_to(0.0, 100.0);
        builder.line_to(50.0, 200.0);
        builder.curve_to(60.0, 210.0, 70.0, 220.0, 80.0, 100.0);
        builder.close();
        let path = builder.into_path_data();
        // y is negated for SVG
        assert_eq!(path, "M0 -100L50 -200C60 -210 70 -220 80 -100Z");
    }
}
