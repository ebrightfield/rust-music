//! SVG rendering for tablature slide lines.

use crate::layout::tab_slide::TabSlideLayout;
use crate::render::SvgWriter;

/// Draw a slide line between two fret positions on the tab staff.
///
/// Renders a diagonal (or horizontal) line from the source fret to the
/// target fret. The line is black with the stroke width specified in the layout.
pub fn draw_tab_slide(svg: &mut SvgWriter, layout: &TabSlideLayout) {
    svg.add_line(
        layout.x_start,
        layout.y_start,
        layout.x_end,
        layout.y_end,
        "black",
        layout.stroke_width,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_slide::layout_tab_slide;

    fn test_staff() -> TabStaffLayout {
        TabStaffLayout::new(0.0, 0.0, 5000.0, 250.0, 6)
    }

    #[test]
    fn draw_slide_adds_one_line() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 500.0, 1000.0, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_slide(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            1,
            "slide should add exactly 1 line element"
        );
    }

    #[test]
    fn slide_line_has_correct_coordinates() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 3, 500.0, 1000.0, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_slide(&mut svg, &layout);
        let output = svg.to_svg();

        // Verify the line contains the expected x1/x2 values
        assert!(output.contains(&format!("x1=\"{}", layout.x_start)));
        assert!(output.contains(&format!("x2=\"{}", layout.x_end)));
    }

    #[test]
    fn slide_line_is_black() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 500.0, 1000.0, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_slide(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("stroke=\"black\""));
    }

    #[test]
    fn slide_line_has_correct_stroke_width() {
        let staff = test_staff();
        let layout = layout_tab_slide(&staff, 1, 500.0, 1000.0, 7.5).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_slide(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("stroke-width=\"7.5\""));
    }

    #[test]
    fn different_strings_produce_different_slide_y() {
        let staff = test_staff();
        let layout1 = layout_tab_slide(&staff, 1, 500.0, 1000.0, 5.0).unwrap();
        let layout4 = layout_tab_slide(&staff, 4, 500.0, 1000.0, 5.0).unwrap();

        let mut svg1 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_slide(&mut svg1, &layout1);
        let mut svg4 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_slide(&mut svg4, &layout4);

        assert_ne!(
            svg1.to_svg(),
            svg4.to_svg(),
            "slides on different strings should produce different SVG"
        );
    }
}
