use crate::layout::tab_palm_mute::{TabPalmMuteDashLayout, TabPalmMuteLayout};
use crate::render::{SvgWriter, TextStyle};

/// Draw a "P.M." palm mute text annotation above a tab fret event.
pub fn draw_tab_palm_mute(svg: &mut SvgWriter, layout: &TabPalmMuteLayout) {
    svg.add_text(
        layout.x,
        layout.y,
        "P.M.",
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

/// Draw a dashed continuation line for a palm mute passage.
///
/// The dashed line extends from after the "P.M." text to the end of the
/// muted passage, indicating sustained palm muting across multiple events.
pub fn draw_tab_palm_mute_dash(svg: &mut SvgWriter, layout: &TabPalmMuteDashLayout) {
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
    use crate::layout::tab_palm_mute::{layout_tab_palm_mute, layout_tab_palm_mute_dash};

    fn setup() -> TabStaffLayout {
        let font = bravura_font();
        let config = font.engraving_config();
        TabStaffLayout::guitar(0.0, 0.0, 5000.0, &config)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 400.0, -100.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn draw_palm_mute_adds_text_element() {
        let staff = setup();
        let layout = layout_tab_palm_mute(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_palm_mute(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("<text "),
            "should render a text element for P.M."
        );
    }

    #[test]
    fn draw_palm_mute_text_content_is_pm() {
        let staff = setup();
        let layout = layout_tab_palm_mute(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_palm_mute(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("P.M."),
            "text content should be 'P.M.'"
        );
    }

    #[test]
    fn draw_palm_mute_text_is_italic() {
        let staff = setup();
        let layout = layout_tab_palm_mute(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_palm_mute(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("font-style=\"italic\""),
            "P.M. text should be italic"
        );
    }

    #[test]
    fn draw_palm_mute_text_is_centered() {
        let staff = setup();
        let layout = layout_tab_palm_mute(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_palm_mute(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("text-anchor=\"middle\""),
            "P.M. text should be centered"
        );
    }

    #[test]
    fn draw_palm_mute_dash_adds_line() {
        let staff = setup();
        let font = bravura_font();
        let config = font.engraving_config();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_palm_mute_dash(&staff, 500.0, 1500.0, stroke).unwrap();
        let mut svg = make_svg();
        draw_tab_palm_mute_dash(&mut svg, &dash);
        let output = svg.to_svg();
        assert!(
            output.contains("<line "),
            "should render a line element for the dashed line"
        );
    }

    #[test]
    fn draw_palm_mute_dash_has_dash_array() {
        let staff = setup();
        let font = bravura_font();
        let config = font.engraving_config();
        let stroke = config.stem_thickness_fu();
        let dash = layout_tab_palm_mute_dash(&staff, 500.0, 1500.0, stroke).unwrap();
        let mut svg = make_svg();
        draw_tab_palm_mute_dash(&mut svg, &dash);
        let output = svg.to_svg();
        assert!(
            output.contains("stroke-dasharray"),
            "dashed line should have stroke-dasharray attribute"
        );
    }

    #[test]
    fn different_x_positions_produce_different_svg() {
        let staff = setup();
        let l1 = layout_tab_palm_mute(&staff, 100.0);
        let l2 = layout_tab_palm_mute(&staff, 200.0);
        let mut svg1 = make_svg();
        let mut svg2 = make_svg();
        draw_tab_palm_mute(&mut svg1, &l1);
        draw_tab_palm_mute(&mut svg2, &l2);
        assert_ne!(
            svg1.to_svg(),
            svg2.to_svg(),
            "different x positions should produce different SVG"
        );
    }

    #[test]
    fn draw_palm_mute_does_not_add_path() {
        let staff = setup();
        let layout = layout_tab_palm_mute(&staff, 1000.0);
        let mut svg = make_svg();
        draw_tab_palm_mute(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            !output.contains("<path "),
            "P.M. text should not produce a path element (text-only)"
        );
    }
}
