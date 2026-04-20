/// SVG rendering for expression text markings.
///
/// Renders italic text below the staff using the geometry from
/// [`crate::layout::expression`].
use crate::layout::expression::ExpressionLayout;
use crate::render::SvgWriter;

/// Draw an expression text marking onto the SVG.
///
/// Renders italic serif text centered on the note's x-position, below the staff.
pub fn draw_expression(svg: &mut SvgWriter, layout: &ExpressionLayout) {
    svg.add_styled_text(
        layout.x_center,
        layout.y_baseline,
        &layout.text,
        "serif",
        layout.font_size,
        "black",
        "middle",
        "normal",
        "italic",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::expression::layout_expression;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    fn test_svg() -> SvgWriter {
        SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn expression_renders_text_element() {
        let layout = layout_expression("dolce", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_expression(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<text"), "should contain a text element");
        assert!(output.contains(">dolce<"), "should contain 'dolce'");
    }

    #[test]
    fn expression_is_italic() {
        let layout = layout_expression("legato", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_expression(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("italic"), "expression text should be italic");
    }

    #[test]
    fn expression_is_centered() {
        let layout = layout_expression("cantabile", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_expression(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("middle"),
            "expression text should be centered (text-anchor: middle)"
        );
    }

    #[test]
    fn different_expressions_produce_different_output() {
        let layout_a = layout_expression("dolce", 500.0, &test_staff(), 250.0);
        let layout_b = layout_expression("legato", 500.0, &test_staff(), 250.0);

        let mut svg_a = test_svg();
        draw_expression(&mut svg_a, &layout_a);

        let mut svg_b = test_svg();
        draw_expression(&mut svg_b, &layout_b);

        assert_ne!(svg_a.to_svg(), svg_b.to_svg());
    }

    #[test]
    fn different_positions_produce_different_output() {
        let layout_a = layout_expression("dolce", 200.0, &test_staff(), 250.0);
        let layout_b = layout_expression("dolce", 800.0, &test_staff(), 250.0);

        let mut svg_a = test_svg();
        draw_expression(&mut svg_a, &layout_a);

        let mut svg_b = test_svg();
        draw_expression(&mut svg_b, &layout_b);

        assert_ne!(svg_a.to_svg(), svg_b.to_svg());
    }

    #[test]
    fn expression_has_no_path() {
        let layout = layout_expression("espressivo", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_expression(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<path").count(),
            0,
            "expression text should have no paths"
        );
    }

    #[test]
    fn text_count_is_one() {
        let layout = layout_expression("molto", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_expression(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<text").count(),
            1,
            "should produce exactly 1 text element"
        );
    }
}
