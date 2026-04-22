use crate::layout::tab_let_ring::{TabLetRingDashLayout, TabLetRingLayout};
use crate::render::{SvgWriter, TextStyle};

/// Draw a "let ring" text annotation above a tab fret event.
pub fn draw_tab_let_ring(svg: &mut SvgWriter, layout: &TabLetRingLayout) {
    svg.add_text(
        layout.x,
        layout.y,
        "let ring",
        &TextStyle {
            font_family: "serif",
            font_size: layout.font_size,
            fill: "black",
            anchor: "middle",
            font_weight: "normal",
            font_style: "italic",
            dominant_baseline: "auto",
        },
    );
}

/// Draw a dashed continuation line for a "let ring" passage.
///
/// The dashed line extends from after the "let ring" text to the end of the
/// ringing passage, indicating sustained open strings across multiple events.
pub fn draw_tab_let_ring_dash(svg: &mut SvgWriter, layout: &TabLetRingDashLayout) {
    svg.add_dashed_line(
        layout.x_start,
        layout.y,
        layout.x_end,
        layout.y,
        "black",
        layout.stroke_width,
        "4,3",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_let_ring::{layout_tab_let_ring, layout_tab_let_ring_dash};

    fn setup() -> TabStaffLayout {
        let font = bravura_font();
        let config = font.engraving_config();
        TabStaffLayout::guitar(0.0, 0.0, 5000.0, &config)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 400.0, -100.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn draw_let_ring_adds_text_element() {
        let staff = setup();
        let layout = layout_tab_let_ring(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_let_ring(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("<text "),
            "should render a text element for let ring"
        );
    }

    #[test]
    fn draw_let_ring_text_content() {
        let staff = setup();
        let layout = layout_tab_let_ring(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_let_ring(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("let ring"),
            "text content should be 'let ring'"
        );
    }

    #[test]
    fn draw_let_ring_text_is_italic() {
        let staff = setup();
        let layout = layout_tab_let_ring(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_let_ring(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("font-style=\"italic\""),
            "let ring text should be italic"
        );
    }

    #[test]
    fn draw_let_ring_text_is_centered() {
        let staff = setup();
        let layout = layout_tab_let_ring(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_let_ring(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("text-anchor=\"middle\""),
            "let ring text should be centered"
        );
    }

    #[test]
    fn draw_let_ring_dash_adds_line() {
        let staff = setup();
        let font = bravura_font();
        let config = font.engraving_config();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_let_ring_dash(&staff, 500.0, 2000.0, stroke).unwrap();
        let mut svg = make_svg();
        draw_tab_let_ring_dash(&mut svg, &dash);
        let output = svg.to_svg();
        assert!(
            output.contains("<line "),
            "should render a line element for the dashed line"
        );
    }

    #[test]
    fn draw_let_ring_dash_has_dash_array() {
        let staff = setup();
        let font = bravura_font();
        let config = font.engraving_config();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_let_ring_dash(&staff, 500.0, 2000.0, stroke).unwrap();
        let mut svg = make_svg();
        draw_tab_let_ring_dash(&mut svg, &dash);
        let output = svg.to_svg();
        assert!(
            output.contains("stroke-dasharray"),
            "dashed line should have stroke-dasharray attribute"
        );
    }

    #[test]
    fn different_x_positions_produce_different_svg() {
        let staff = setup();
        let l1 = layout_tab_let_ring(&staff, 100.0);
        let l2 = layout_tab_let_ring(&staff, 200.0);
        let mut svg1 = make_svg();
        let mut svg2 = make_svg();
        draw_tab_let_ring(&mut svg1, &l1);
        draw_tab_let_ring(&mut svg2, &l2);
        assert_ne!(
            svg1.to_svg(),
            svg2.to_svg(),
            "different x positions should produce different SVG"
        );
    }

    #[test]
    fn draw_let_ring_does_not_add_path() {
        let staff = setup();
        let layout = layout_tab_let_ring(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_let_ring(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            !output.contains("<path "),
            "let ring text should not produce a path element (text-only)"
        );
    }
}
