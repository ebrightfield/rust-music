/// Ottava bracket rendering — draws 8va/8vb dashed lines with text label and hook.
///
/// An ottava bracket consists of:
/// - An italic text label ("8va", "8vb", "15ma", "15mb") at the start
/// - A dashed horizontal line extending to the right
/// - An optional vertical hook at the end (downward for above, upward for below)
use crate::layout::ottava::OttavaBracketLayout;
use crate::render::{SvgWriter, TextStyle};

/// Draw an ottava bracket onto the SVG.
pub fn draw_ottava_bracket(svg: &mut SvgWriter, layout: &OttavaBracketLayout) {
    // Text label (italic, as per engraving convention)
    svg.add_text(layout.label_x, layout.label_y, &layout.label, &TextStyle {
        font_family: "serif",
        font_size: layout.font_size,
        fill: "black",
        anchor: "start",
        font_weight: "bold",
        font_style: "italic",
        dominant_baseline: "auto",
    });

    // Dashed horizontal line from after the label to the end
    if layout.x_line_start < layout.x_end {
        let dash_spec = format!("{},{}", layout.dash_length, layout.dash_gap);
        svg.add_dashed_line(
            layout.x_line_start,
            layout.y_line,
            layout.x_end,
            layout.y_line,
            "black",
            layout.line_thickness,
            &dash_spec,
        );
    }

    // End hook (vertical line at the right edge)
    if layout.has_end_hook {
        let hook_end_y = if layout.kind.is_above() {
            // Hook goes downward toward the staff
            layout.y_line + layout.hook_height
        } else {
            // Hook goes upward toward the staff
            layout.y_line - layout.hook_height
        };
        svg.add_line(
            layout.x_end,
            layout.y_line,
            layout.x_end,
            hook_end_y,
            "black",
            layout.line_thickness,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::ottava::{layout_ottava_bracket, OttavaKind};
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 2000.0, 250.0)
    }

    fn make_layout(kind: OttavaKind, has_hook: bool) -> OttavaBracketLayout {
        layout_ottava_bracket(kind, 100.0, 800.0, &test_staff(), 250.0, has_hook)
    }

    #[test]
    fn renders_text_label() {
        let layout = make_layout(OttavaKind::Ottava8va, true);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">8va</text>"), "should contain '8va' text");
    }

    #[test]
    fn label_is_italic() {
        let layout = make_layout(OttavaKind::Ottava8va, true);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("font-style=\"italic\""), "label should be italic");
    }

    #[test]
    fn label_is_bold() {
        let layout = make_layout(OttavaKind::Ottava8va, true);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("font-weight=\"bold\""), "label should be bold");
    }

    #[test]
    fn renders_dashed_line() {
        let layout = make_layout(OttavaKind::Ottava8va, true);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("stroke-dasharray"), "should have a dashed line");
    }

    #[test]
    fn with_hook_draws_solid_line() {
        let layout = make_layout(OttavaKind::Ottava8va, true);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        // Count lines that DON'T have dasharray — these are the hook
        let total_lines = output.matches("<line ").count();
        assert!(total_lines >= 1, "should have at least 1 line element for hook");
    }

    #[test]
    fn without_hook_has_fewer_lines() {
        let with_hook = make_layout(OttavaKind::Ottava8va, true);
        let no_hook = make_layout(OttavaKind::Ottava8va, false);
        let mut svg1 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        let mut svg2 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg1, &with_hook);
        draw_ottava_bracket(&mut svg2, &no_hook);
        let lines_with = svg1.to_svg().matches("<line ").count();
        let lines_without = svg2.to_svg().matches("<line ").count();
        assert_eq!(lines_with, lines_without + 1,
            "hook adds exactly 1 extra line element");
    }

    #[test]
    fn different_kinds_have_different_labels() {
        let l8va = make_layout(OttavaKind::Ottava8va, true);
        let l8vb = make_layout(OttavaKind::Ottava8vb, true);
        let mut svg1 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        let mut svg2 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg1, &l8va);
        draw_ottava_bracket(&mut svg2, &l8vb);
        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn fifteen_ma_label_correct() {
        let layout = make_layout(OttavaKind::Ottava15ma, true);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">15ma</text>"), "should contain '15ma' text");
    }

    #[test]
    fn fifteen_mb_label_correct() {
        let layout = make_layout(OttavaKind::Ottava15mb, true);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_ottava_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">15mb</text>"), "should contain '15mb' text");
    }
}
