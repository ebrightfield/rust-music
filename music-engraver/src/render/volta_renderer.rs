/// Volta bracket rendering — draws 1st/2nd ending brackets above the staff.
///
/// A volta bracket consists of:
/// - A horizontal line spanning the measure(s)
/// - Optional left and/or right vertical hooks
/// - Optional text label (e.g. "1.", "2.") at the left side
use crate::layout::volta::VoltaBracketLayout;
use crate::render::{SvgWriter, TextStyle};

/// Draw a volta bracket segment onto the SVG.
pub fn draw_volta_bracket(svg: &mut SvgWriter, layout: &VoltaBracketLayout) {
    let thickness = layout.line_thickness;

    // Horizontal top line
    svg.add_line(
        layout.x_left,
        layout.y_top,
        layout.x_right,
        layout.y_top,
        "black",
        thickness,
    );

    // Left vertical hook (downward from top line)
    if layout.left_hook {
        svg.add_line(
            layout.x_left,
            layout.y_top,
            layout.x_left,
            layout.y_hook_bottom,
            "black",
            thickness,
        );
    }

    // Right vertical hook (downward from top line)
    if layout.right_hook {
        svg.add_line(
            layout.x_right,
            layout.y_top,
            layout.x_right,
            layout.y_hook_bottom,
            "black",
            thickness,
        );
    }

    // Text label
    if let Some((tx, ty, ref text)) = layout.text {
        svg.add_text(tx, ty, text, &TextStyle {
            font_family: "serif",
            font_size: layout.font_size,
            fill: "black",
            anchor: "start",
            font_weight: "bold",
            font_style: "normal",
            dominant_baseline: "auto",
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::staff::StaffLayout;
    use crate::layout::volta::{layout_volta_bracket, VoltaAnnotation, VoltaHooks};

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 2000.0, 250.0)
    }

    fn make_layout(text: Option<&str>, hooks: VoltaHooks) -> VoltaBracketLayout {
        let ann = VoltaAnnotation {
            text: text.map(|s| s.to_string()),
            hooks,
        };
        layout_volta_bracket(&ann, 100.0, 600.0, &test_staff(), 250.0)
    }

    #[test]
    fn both_hooks_draws_three_lines() {
        let layout = make_layout(Some("1."), VoltaHooks::Both);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        // 3 lines: top horizontal + left hook + right hook
        let line_count = output.matches("<line ").count();
        assert_eq!(line_count, 3, "both hooks should produce 3 lines");
    }

    #[test]
    fn left_only_draws_two_lines() {
        let layout = make_layout(Some("1."), VoltaHooks::LeftOnly);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        let line_count = output.matches("<line ").count();
        assert_eq!(line_count, 2, "left only should produce 2 lines");
    }

    #[test]
    fn right_only_draws_two_lines() {
        let layout = make_layout(None, VoltaHooks::RightOnly);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        let line_count = output.matches("<line ").count();
        assert_eq!(line_count, 2, "right only should produce 2 lines");
    }

    #[test]
    fn neither_draws_one_line() {
        let layout = make_layout(None, VoltaHooks::Neither);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        let line_count = output.matches("<line ").count();
        assert_eq!(line_count, 1, "neither hook should produce 1 line (top only)");
    }

    #[test]
    fn text_label_renders_when_present() {
        let layout = make_layout(Some("2."), VoltaHooks::Both);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">2.</text>"), "should contain '2.' text");
        assert!(output.contains("font-weight=\"bold\""), "text should be bold");
    }

    #[test]
    fn no_text_when_none() {
        let layout = make_layout(None, VoltaHooks::RightOnly);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(!output.contains("<text "), "should contain no text element");
    }

    #[test]
    fn different_texts_produce_different_svg() {
        let layout1 = make_layout(Some("1."), VoltaHooks::Both);
        let layout2 = make_layout(Some("2."), VoltaHooks::Both);
        let mut svg1 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        let mut svg2 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg1, &layout1);
        draw_volta_bracket(&mut svg2, &layout2);
        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn hooks_vs_no_hooks_produce_different_svg() {
        let with_hooks = make_layout(Some("1."), VoltaHooks::Both);
        let no_hooks = make_layout(Some("1."), VoltaHooks::Neither);
        let mut svg1 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        let mut svg2 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_volta_bracket(&mut svg1, &with_hooks);
        draw_volta_bracket(&mut svg2, &no_hooks);
        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }
}
