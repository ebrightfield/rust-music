use crate::layout::multi_measure_rest::MultiMeasureRestLayout;
use crate::render::svg_writer::{RectStyle, TextStyle};
use crate::render::SvgWriter;

/// Draw a multi-measure rest (H-bar with count number) onto an SVG writer.
///
/// Renders:
/// - Two vertical serif strokes (left and right edges of the H-bar)
/// - One filled horizontal rectangle (the thick crossbar)
/// - A bold centered number above the bar showing the measure count
pub fn draw_multi_measure_rest(svg: &mut SvgWriter, layout: &MultiMeasureRestLayout) {
    let serif_w = layout.serif_thickness;

    // Left vertical serif
    svg.add_styled_rect(
        layout.x_left,
        layout.y_top,
        serif_w,
        layout.y_bottom - layout.y_top,
        &RectStyle {
            fill: "black",
            stroke: "none",
            stroke_width: 0.0,
        },
    );

    // Right vertical serif
    svg.add_styled_rect(
        layout.x_right - serif_w,
        layout.y_top,
        serif_w,
        layout.y_bottom - layout.y_top,
        &RectStyle {
            fill: "black",
            stroke: "none",
            stroke_width: 0.0,
        },
    );

    // Horizontal bar (crossbar connecting the two serifs)
    svg.add_styled_rect(
        layout.x_left,
        layout.bar_y_top,
        layout.x_right - layout.x_left,
        layout.bar_y_bottom - layout.bar_y_top,
        &RectStyle {
            fill: "black",
            stroke: "none",
            stroke_width: 0.0,
        },
    );

    // Count number above the bar
    let style = TextStyle {
        anchor: "middle",
        ..TextStyle::bold(layout.count_font_size)
    };
    svg.add_text(
        layout.count_x,
        layout.count_y,
        &layout.count.to_string(),
        &style,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::multi_measure_rest::layout_multi_measure_rest;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 5000.0, 250.0)
    }

    fn render(count: u32) -> String {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, count, &staff);
        let mut svg = SvgWriter::new(600.0, 300.0, 0.0, 0.0, 600.0, 300.0);
        draw_multi_measure_rest(&mut svg, &layout);
        svg.to_svg()
    }

    #[test]
    fn draws_three_rects() {
        let output = render(4);
        let rect_count = output.matches("<rect").count();
        assert_eq!(
            rect_count, 3,
            "should have 3 rects (2 serifs + 1 bar), got {rect_count}"
        );
    }

    #[test]
    fn draws_count_text() {
        let output = render(4);
        assert!(
            output.contains(">4</text>"),
            "should contain count text '4'"
        );
    }

    #[test]
    fn different_counts_produce_different_text() {
        let out4 = render(4);
        let out12 = render(12);
        assert!(out4.contains(">4</text>"));
        assert!(out12.contains(">12</text>"));
        assert_ne!(out4, out12);
    }

    #[test]
    fn count_text_is_bold() {
        let output = render(4);
        assert!(
            output.contains("font-weight=\"bold\""),
            "count should be bold"
        );
    }

    #[test]
    fn count_text_is_centered() {
        let output = render(4);
        assert!(
            output.contains("text-anchor=\"middle\""),
            "count text should be centered"
        );
    }

    #[test]
    fn rects_are_filled_black() {
        let output = render(4);
        let black_fill_count = output.matches("fill=\"black\"").count();
        assert!(
            black_fill_count >= 3,
            "all 3 rects should have fill=\"black\", got {black_fill_count}"
        );
    }

    #[test]
    fn single_measure_rest_renders() {
        let output = render(1);
        assert!(output.contains(">1</text>"));
        assert_eq!(output.matches("<rect").count(), 3);
    }

    #[test]
    fn large_count_renders() {
        let output = render(128);
        assert!(output.contains(">128</text>"));
    }
}
