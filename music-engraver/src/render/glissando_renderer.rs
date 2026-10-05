use std::fmt::Write;

use crate::layout::glissando::{GlissandoLayout, GlissandoStyle};
use crate::render::svg_writer::TextStyle;
use crate::render::SvgWriter;

/// Draw a glissando between clipped notehead edges (including system fragments).
pub fn draw_glissando(svg: &mut SvgWriter, layout: &GlissandoLayout) {
    match layout.style {
        GlissandoStyle::Line | GlissandoStyle::LineWithText => svg.add_line(
            layout.x_start,
            layout.y_start,
            layout.x_end,
            layout.y_end,
            "black",
            layout.stroke_width,
        ),
        GlissandoStyle::Dashed => {
            let ss = layout.stroke_width / 0.08;
            let dash = format!("{},{}", ss * 0.36, ss * 0.23);
            svg.add_dashed_line(
                layout.x_start,
                layout.y_start,
                layout.x_end,
                layout.y_end,
                "black",
                layout.stroke_width,
                &dash,
            );
        }
        GlissandoStyle::Wavy => {
            let dx = layout.x_end - layout.x_start;
            let dy = layout.y_end - layout.y_start;
            let length = dx.hypot(dy);
            let ss = layout.stroke_width / 0.08;
            let count = ((length / (ss * 0.36)).ceil() as usize).max(2);
            let amplitude = ss * 0.16;
            let mut path = format!("<path d=\"M {} {}", layout.x_start, layout.y_start);
            for i in 1..count {
                let t = i as f64 / count as f64;
                let side = if i % 2 == 0 { -amplitude } else { amplitude };
                let x = layout.x_start + t * dx - side * dy / length;
                let y = layout.y_start + t * dy + side * dx / length;
                let _ = write!(path, " L {x} {y}");
            }
            let _ = write!(
                path,
                " L {} {}\" fill=\"none\" stroke=\"black\" stroke-width=\"{}\"/>",
                layout.x_end, layout.y_end, layout.stroke_width
            );
            svg.add_raw(&path);
        }
    }

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
        let layout =
            layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<line"), "should contain a line element");
    }

    #[test]
    fn line_only_no_text() {
        let staff = test_staff();
        let layout =
            layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            !output.contains("gliss."),
            "Line style should not show text"
        );
    }

    #[test]
    fn line_with_text_shows_gliss_label() {
        let staff = test_staff();
        let layout = layout_glissando(
            100.0,
            0,
            1200.0,
            8,
            &staff,
            GlissandoStyle::LineWithText,
            None,
        )
        .unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("gliss."),
            "LineWithText should show 'gliss.' label"
        );
    }

    #[test]
    fn text_is_italic() {
        let staff = test_staff();
        let layout = layout_glissando(
            100.0,
            0,
            1200.0,
            8,
            &staff,
            GlissandoStyle::LineWithText,
            None,
        )
        .unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("italic"), "gliss. label should be italic");
    }

    #[test]
    fn ascending_line_coordinates() {
        let staff = test_staff();
        let layout =
            layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg, &layout);
        let output = svg.to_svg();
        // Should contain x1 attribute with the layout's x_start value
        let x1_str = format!("x1=\"{}\"", layout.x_start);
        assert!(
            output.contains(&x1_str),
            "should embed x_start: {} in {}",
            x1_str,
            output
        );
    }

    #[test]
    fn ascending_vs_descending_produce_different_svgs() {
        let staff = test_staff();
        let asc =
            layout_glissando(100.0, 0, 1200.0, 8, &staff, GlissandoStyle::Line, None).unwrap();
        let desc =
            layout_glissando(100.0, 8, 1200.0, 0, &staff, GlissandoStyle::Line, None).unwrap();
        let mut svg1 = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        let mut svg2 = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_glissando(&mut svg1, &asc);
        draw_glissando(&mut svg2, &desc);
        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }
}
