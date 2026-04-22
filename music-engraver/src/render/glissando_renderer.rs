use crate::layout::glissando::GlissandoLayout;
use crate::render::svg_writer::TextStyle;
use crate::render::SvgWriter;

/// Draw a glissando line (and optional "gliss." text label) onto an SVG writer.
///
/// Renders a straight diagonal line from the source note to the target note.
/// When `show_text` is true, adds italic "gliss." text centered along the line.
pub fn draw_glissando(svg: &mut SvgWriter, layout: &GlissandoLayout) {
    svg.add_line(
        layout.x_start,
        layout.y_start,
        layout.x_end,
        layout.y_end,
        "black",
        layout.stroke_width,
    );

    if layout.show_text {
        let style = TextStyle {
            font_style: "italic",
            ..TextStyle::normal(layout.text_font_size)
        };
        svg.add_text(layout.text_x, layout.text_y, "gliss.", &style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::glissando::{layout_glissando, GlissandoStyle};
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 5000.0, 250.0)
    }

    #[test]
    fn draws_line_element() {
        let staff = test_staff();
        let layout = layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<line"), "should contain a line element");
    }

    #[test]
    fn line_only_no_text() {
        let staff = test_staff();
        let layout = layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(!output.contains("gliss."), "Line style should not show text");
    }

    #[test]
    fn line_with_text_shows_gliss_label() {
        let staff = test_staff();
        let layout = layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::LineWithText, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("gliss."), "LineWithText should show 'gliss.' label");
    }

    #[test]
    fn text_is_italic() {
        let staff = test_staff();
        let layout = layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::LineWithText, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("italic"), "gliss. label should be italic");
    }

    #[test]
    fn ascending_line_coordinates() {
        let staff = test_staff();
        let layout = layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        // Should contain x1 attribute with the layout's x_start value
        let x1_str = format!("x1=\"{}\"", layout.x_start);
        assert!(output.contains(&x1_str), "should embed x_start: {} in {}", x1_str, output);
    }

    #[test]
    fn ascending_vs_descending_produce_different_svgs() {
        let staff = test_staff();
        let asc = layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let desc = layout_glissando(100.0, 8, 1200.0, 0, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg1 = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        let mut svg2 = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg1, &asc);
        draw_glissando(&mut svg2, &desc);
        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }
}
