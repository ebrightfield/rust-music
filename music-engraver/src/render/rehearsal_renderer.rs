/// Renders rehearsal marks (boxed or plain text above the staff).
use crate::layout::rehearsal::RehearsalMarkLayout;
use crate::render::svg_writer::{SvgWriter, TextStyle};

/// Draw a rehearsal mark using the given layout.
///
/// For `Boxed` style, draws a stroked rectangle around the text.
/// Text is rendered as an SVG `<text>` element centered on `x_center`.
pub fn draw_rehearsal_mark(svg: &mut SvgWriter, layout: &RehearsalMarkLayout) {
    // Draw box first (behind text)
    if let Some((bx, by, bw, bh)) = layout.box_rect {
        svg.add_stroked_rect(
            bx,
            by,
            bw,
            bh,
            "white",
            "black",
            layout.box_stroke_width,
        );
    }

    let style = TextStyle {
        anchor: "middle",
        ..TextStyle::bold(layout.font_size)
    };
    svg.add_text(
        layout.x_center,
        layout.y_baseline,
        &layout.text,
        &style,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::rehearsal::{layout_rehearsal_mark, RehearsalStyle};
    use crate::layout::staff::StaffLayout;
    use crate::render::svg_writer::SvgWriter;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 400.0, 0.0, -500.0, 4000.0, 2000.0)
    }

    #[test]
    fn boxed_rehearsal_produces_rect_and_text() {
        let mut svg = make_svg();
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        draw_rehearsal_mark(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains("<rect "), "should have a box rect");
        assert!(out.contains("<text "), "should have text");
        assert!(out.contains(">A</text>"), "text content should be 'A'");
    }

    #[test]
    fn plain_rehearsal_has_text_but_no_rect() {
        let mut svg = make_svg();
        let layout =
            layout_rehearsal_mark("B", 500.0, &test_staff(), 250.0, RehearsalStyle::Plain);
        draw_rehearsal_mark(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(!out.contains("<rect "), "plain style should have no rect");
        assert!(out.contains("<text "), "should have text");
        assert!(out.contains(">B</text>"));
    }

    #[test]
    fn boxed_rect_has_stroke() {
        let mut svg = make_svg();
        let layout =
            layout_rehearsal_mark("C", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        draw_rehearsal_mark(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains(r#"stroke="black""#));
        assert!(out.contains(r#"fill="white""#));
    }

    #[test]
    fn text_is_bold_for_both_styles() {
        for style in [RehearsalStyle::Boxed, RehearsalStyle::Plain] {
            let mut svg = make_svg();
            let layout = layout_rehearsal_mark("D", 500.0, &test_staff(), 250.0, style);
            draw_rehearsal_mark(&mut svg, &layout);
            let out = svg.to_svg();
            assert!(
                out.contains(r#"font-weight="bold""#),
                "rehearsal mark should be bold for {:?}",
                style
            );
        }
    }

    #[test]
    fn text_centered_with_middle_anchor() {
        let mut svg = make_svg();
        let layout =
            layout_rehearsal_mark("E", 750.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        draw_rehearsal_mark(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains(r#"text-anchor="middle""#));
        assert!(out.contains(r#"x="750""#));
    }

    #[test]
    fn different_letters_produce_different_text() {
        let mut svg1 = make_svg();
        let mut svg2 = make_svg();
        let l1 = layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        let l2 = layout_rehearsal_mark("B", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        draw_rehearsal_mark(&mut svg1, &l1);
        draw_rehearsal_mark(&mut svg2, &l2);
        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn numeric_rehearsal_mark() {
        let mut svg = make_svg();
        let layout =
            layout_rehearsal_mark("12", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        draw_rehearsal_mark(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains(">12</text>"));
    }

    #[test]
    fn font_size_embedded_in_text_element() {
        let mut svg = make_svg();
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        draw_rehearsal_mark(&mut svg, &layout);
        let out = svg.to_svg();
        // font_size = 1.8 * 250 = 450
        assert!(out.contains(r#"font-size="450""#));
    }

    #[test]
    fn serif_font_family_used() {
        let mut svg = make_svg();
        let layout =
            layout_rehearsal_mark("A", 500.0, &test_staff(), 250.0, RehearsalStyle::Boxed);
        draw_rehearsal_mark(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains(r#"font-family="serif""#));
    }
}
