//! SVG rendering for tablature vibrato notation.

use crate::layout::tab_vibrato::TabVibratoLayout;
use crate::render::SvgWriter;

/// Draw a vibrato wavy line above a tab fret position.
///
/// Renders the pre-computed quadratic Bézier path as an unfilled stroke.
pub fn draw_tab_vibrato(svg: &mut SvgWriter, layout: &TabVibratoLayout) {
    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"{:.2}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
        layout.path_data, layout.stroke_width
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_vibrato::{layout_tab_vibrato, VibratoKind};

    fn test_staff() -> TabStaffLayout {
        TabStaffLayout::new(0.0, 0.0, 1000.0, 250.0, 6)
    }

    #[test]
    fn draw_produces_path_element() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_vibrato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<path "), "should contain a path element");
    }

    #[test]
    fn path_has_no_fill() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_vibrato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("fill=\"none\""));
    }

    #[test]
    fn path_has_stroke() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_vibrato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("stroke=\"black\""));
        assert!(output.contains("stroke-width="));
    }

    #[test]
    fn path_contains_bezier_data() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 3, 500.0, VibratoKind::Normal, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_vibrato(&mut svg, &layout);
        let output = svg.to_svg();
        // Should contain Q commands from the quadratic Bézier wave
        assert!(output.contains(" Q"), "path should contain Q commands");
    }

    #[test]
    fn normal_and_wide_produce_different_svg() {
        let staff = test_staff();
        let normal_layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        let wide_layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Wide, 5.0);

        let mut svg1 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_vibrato(&mut svg1, &normal_layout);

        let mut svg2 = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_vibrato(&mut svg2, &wide_layout);

        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn rounded_line_caps() {
        let staff = test_staff();
        let layout = layout_tab_vibrato(&staff, 1, 500.0, VibratoKind::Normal, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_vibrato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("stroke-linecap=\"round\""));
    }
}
