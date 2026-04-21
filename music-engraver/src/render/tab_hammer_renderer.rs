//! SVG rendering for tablature hammer-on and pull-off arcs.

use crate::layout::tab_hammer::TabLegatoLayout;
use crate::render::{SvgWriter, TextStyle};

/// Draw a hammer-on or pull-off arc between two fret positions on the tab staff.
///
/// Renders a quadratic Bézier arc curving above the string line, with the
/// technique label ("H" or "P") centered at the apex. The arc is unfilled
/// (stroke only) to distinguish it from tied note curves.
pub fn draw_tab_legato(svg: &mut SvgWriter, layout: &TabLegatoLayout) {
    // Quadratic Bézier: M x_start,y_string Q x_mid,y_apex x_end,y_string
    let path_data = format!(
        "M{},{} Q{},{} {},{}",
        layout.x_start, layout.y_string,
        layout.x_text, layout.y_apex,
        layout.x_end, layout.y_string,
    );

    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"{}\"/>",
        path_data, layout.stroke_width,
    ));

    // Label text centered at the apex
    svg.add_text(
        layout.x_text,
        layout.y_text,
        layout.kind.label(),
        &TextStyle {
            font_family: "sans-serif",
            font_size: layout.font_size,
            fill: "black",
            anchor: "middle",
            font_weight: "bold",
            font_style: "normal",
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_hammer::{layout_tab_legato, LegatoKind};

    fn test_staff() -> TabStaffLayout {
        TabStaffLayout::new(0.0, 0.0, 5000.0, 250.0, 6)
    }

    #[test]
    fn draw_legato_adds_path_and_text() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg, &layout);
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "should have 1 arc path");
        assert_eq!(output.matches("<text ").count(), 1, "should have 1 text label");
    }

    #[test]
    fn hammer_on_shows_h_label() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">H</text>"), "should show 'H' label");
    }

    #[test]
    fn pull_off_shows_p_label() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::PullOff, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">P</text>"), "should show 'P' label");
    }

    #[test]
    fn arc_is_unfilled_stroke() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("fill=\"none\""), "arc should be unfilled");
        assert!(output.contains("stroke=\"black\""), "arc should have black stroke");
    }

    #[test]
    fn hammer_and_pull_produce_different_svg() {
        let staff = test_staff();
        let h = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let p = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::PullOff, 5.0).unwrap();

        let mut svg_h = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg_h, &h);
        let mut svg_p = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg_p, &p);

        assert_ne!(svg_h.to_svg(), svg_p.to_svg(), "H and P should produce different SVG");
    }

    #[test]
    fn arc_contains_quadratic_bezier() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg, &layout);
        let output = svg.to_svg();
        // Quadratic Bézier uses Q command in path data
        assert!(output.contains(" Q"), "path should use quadratic Bézier (Q command)");
    }

    #[test]
    fn text_is_bold_sans_serif() {
        let staff = test_staff();
        let layout = layout_tab_legato(&staff, 1, 500.0, 1000.0, LegatoKind::HammerOn, 5.0).unwrap();
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_legato(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("font-weight=\"bold\""), "label should be bold");
        assert!(output.contains("font-family=\"sans-serif\""), "label should be sans-serif");
    }
}
