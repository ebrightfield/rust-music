//! SVG rendering for tablature bend arrows (bend, pre-bend, release).

use crate::layout::tab_bend::{
    TabBendLayout, TabBendReleaseLayout, TabPreBendLayout, TabReleaseLayout,
};
use crate::render::{SvgWriter, TextStyle};

/// Draw a bend arrow at a fret position on the tab staff.
///
/// Renders a curved line from the fret position upward, with a filled
/// arrowhead at the tip and text indicating the bend amount centered
/// above the arrow.
pub fn draw_tab_bend(svg: &mut SvgWriter, layout: &TabBendLayout) {
    // Start the curve partway between string line and arrowhead base
    let y_start = layout.arrow_y_base + (layout.y_string - layout.arrow_y_base) * 0.5;

    let path_data = format!(
        "M{},{} Q{},{} {},{}",
        layout.x, y_start,
        layout.x_control, layout.y_control,
        layout.x, layout.arrow_y_base,
    );

    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"{}\"/>",
        path_data, layout.stroke_width,
    ));

    // Arrowhead: filled triangle pointing up
    let arrow_path = format!(
        "M{},{} L{},{} L{},{} Z",
        layout.x, layout.y_tip,
        layout.arrow_x_left, layout.arrow_y_base,
        layout.arrow_x_right, layout.arrow_y_base,
    );

    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"black\" stroke=\"none\"/>",
        arrow_path,
    ));

    // Bend amount text above the arrow
    svg.add_text(
        layout.x_text,
        layout.y_text,
        layout.amount.label(),
        &TextStyle {
            font_family: "sans-serif",
            font_size: layout.font_size,
            fill: "black",
            anchor: "middle",
            font_weight: "bold",
            font_style: "normal",
            dominant_baseline: "auto",
        },
    );
}

/// Draw a pre-bend arrow at a fret position on the tab staff.
///
/// Renders a straight vertical line from the fret position upward, with a
/// filled arrowhead at the tip and text indicating the bend amount. The
/// straight shaft (vs. curved for a regular bend) signals that the string
/// is already bent when picked.
pub fn draw_tab_pre_bend(svg: &mut SvgWriter, layout: &TabPreBendLayout) {
    // Straight vertical line from base to arrowhead base
    svg.add_line(
        layout.x,
        layout.y_base,
        layout.x,
        layout.arrow_y_base,
        "black",
        layout.stroke_width,
    );

    // Arrowhead: filled triangle pointing up (same as regular bend)
    let arrow_path = format!(
        "M{},{} L{},{} L{},{} Z",
        layout.x, layout.y_tip,
        layout.arrow_x_left, layout.arrow_y_base,
        layout.arrow_x_right, layout.arrow_y_base,
    );

    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"black\" stroke=\"none\"/>",
        arrow_path,
    ));

    // Bend amount text above the arrow
    svg.add_text(
        layout.x_text,
        layout.y_text,
        layout.amount.label(),
        &TextStyle {
            font_family: "sans-serif",
            font_size: layout.font_size,
            fill: "black",
            anchor: "middle",
            font_weight: "bold",
            font_style: "normal",
            dominant_baseline: "auto",
        },
    );
}

/// Draw a release bend arrow at a fret position on the tab staff.
///
/// Renders a curved line from the top of a prior bend downward toward the
/// string, with a filled arrowhead at the bottom pointing down.
pub fn draw_tab_release(svg: &mut SvgWriter, layout: &TabReleaseLayout) {
    // Curve from top to arrowhead base (downward)
    let path_data = format!(
        "M{},{} Q{},{} {},{}",
        layout.x, layout.y_top,
        layout.x_control, layout.y_control,
        layout.x, layout.arrow_y_base,
    );

    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"{}\"/>",
        path_data, layout.stroke_width,
    ));

    // Arrowhead: filled triangle pointing down
    let arrow_path = format!(
        "M{},{} L{},{} L{},{} Z",
        layout.x, layout.y_bottom,
        layout.arrow_x_left, layout.arrow_y_base,
        layout.arrow_x_right, layout.arrow_y_base,
    );

    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"black\" stroke=\"none\"/>",
        arrow_path,
    ));
}

/// Draw a combined bend-and-release arc at a fret position on the tab staff.
///
/// Renders the gesture as an inverted V: a curve rising from the fret position
/// to an apex (with an upward arrowhead and the amount label), then a curve
/// descending back to the string (with a downward arrowhead). This is the
/// single-event form of "bend up, then release back down".
pub fn draw_tab_bend_release(svg: &mut SvgWriter, layout: &TabBendReleaseLayout) {
    // Up-curve: from the arc base near the fret number to the apex arrowhead.
    let up_start = layout.up_arrow_y_base + (layout.y_base - layout.up_arrow_y_base) * 0.5;
    let up_path = format!(
        "M{},{} Q{},{} {},{}",
        layout.x, up_start,
        layout.x_control_up, layout.y_control_up,
        layout.x_apex, layout.up_arrow_y_base,
    );
    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"{}\"/>",
        up_path, layout.stroke_width,
    ));

    // Down-curve: from the apex back down to the release arrowhead near the string.
    let down_path = format!(
        "M{},{} Q{},{} {},{}",
        layout.x_apex, layout.y_apex,
        layout.x_control_down, layout.y_control_down,
        layout.x, layout.down_arrow_y_base,
    );
    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"{}\"/>",
        down_path, layout.stroke_width,
    ));

    // Up-arrowhead: filled triangle at the apex, pointing up.
    let up_arrow = format!(
        "M{},{} L{},{} L{},{} Z",
        layout.x_apex, layout.y_apex,
        layout.up_arrow_x_left, layout.up_arrow_y_base,
        layout.up_arrow_x_right, layout.up_arrow_y_base,
    );
    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"black\" stroke=\"none\"/>",
        up_arrow,
    ));

    // Down-arrowhead: filled triangle at the release landing, pointing down.
    let down_arrow = format!(
        "M{},{} L{},{} L{},{} Z",
        layout.x, layout.y_base,
        layout.down_arrow_x_left, layout.down_arrow_y_base,
        layout.down_arrow_x_right, layout.down_arrow_y_base,
    );
    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"black\" stroke=\"none\"/>",
        down_arrow,
    ));

    // Bend amount text above the apex.
    svg.add_text(
        layout.x_text,
        layout.y_text,
        layout.amount.label(),
        &TextStyle {
            font_family: "sans-serif",
            font_size: layout.font_size,
            fill: "black",
            anchor: "middle",
            font_weight: "bold",
            font_style: "normal",
            dominant_baseline: "auto",
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_bend::{
        layout_tab_bend, layout_tab_bend_release, layout_tab_pre_bend, layout_tab_release,
        BendAmount,
    };

    fn test_staff() -> TabStaffLayout {
        TabStaffLayout::new(0.0, 0.0, 5000.0, 250.0, 6)
    }

    #[test]
    fn draw_bend_adds_two_paths_and_text() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();

        // 1 curve path + 1 arrowhead path = 2 paths
        assert_eq!(output.matches("<path ").count(), 2, "should have curve + arrowhead paths");
        assert_eq!(output.matches("<text ").count(), 1, "should have 1 text label");
    }

    #[test]
    fn full_bend_shows_full_label() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">full</text>"), "should show 'full' label");
    }

    #[test]
    fn half_bend_shows_half_label() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Half, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">1/2</text>"), "should show '1/2' label");
    }

    #[test]
    fn quarter_bend_shows_quarter_label() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Quarter, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">1/4</text>"), "should show '1/4' label");
    }

    #[test]
    fn arrowhead_is_filled() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("fill=\"black\""), "arrowhead should be filled black");
    }

    #[test]
    fn curve_is_unfilled_stroke() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("fill=\"none\""), "curve should be unfilled");
        assert!(output.contains("stroke=\"black\""), "curve should have black stroke");
    }

    #[test]
    fn different_amounts_produce_different_svg() {
        let staff = test_staff();
        let full = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let half = layout_tab_bend(&staff, 1, 500.0, BendAmount::Half, 5.0);

        let mut svg_full = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg_full, &full);
        let mut svg_half = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg_half, &half);

        assert_ne!(svg_full.to_svg(), svg_half.to_svg(), "full and half bends should differ");
    }

    #[test]
    fn text_is_bold_sans_serif() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("font-weight=\"bold\""), "label should be bold");
        assert!(output.contains("font-family=\"sans-serif\""), "label should be sans-serif");
    }

    #[test]
    fn curve_contains_quadratic_bezier() {
        let staff = test_staff();
        let layout = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(" Q"), "curve path should use quadratic Bézier (Q command)");
    }

    // --- Pre-bend renderer tests ---

    #[test]
    fn pre_bend_adds_line_path_and_text() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_pre_bend(&mut svg, &layout);
        let output = svg.to_svg();
        // 1 vertical line + 1 arrowhead path
        assert_eq!(output.matches("<line ").count(), 1, "should have 1 vertical line");
        assert_eq!(output.matches("<path ").count(), 1, "should have 1 arrowhead path");
        assert_eq!(output.matches("<text ").count(), 1, "should have 1 text label");
    }

    #[test]
    fn pre_bend_shows_amount_label() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Half, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_pre_bend(&mut svg, &layout);
        assert!(svg.to_svg().contains(">1/2</text>"), "should show '1/2' label");
    }

    #[test]
    fn pre_bend_arrowhead_is_filled() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_pre_bend(&mut svg, &layout);
        assert!(svg.to_svg().contains("fill=\"black\""), "arrowhead should be filled");
    }

    #[test]
    fn pre_bend_has_no_curve() {
        let staff = test_staff();
        let layout = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_pre_bend(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(!output.contains(" Q"), "pre-bend should use straight line, not curve");
    }

    #[test]
    fn pre_bend_differs_from_regular_bend() {
        let staff = test_staff();
        let pre = layout_tab_pre_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let regular = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg_pre = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_pre_bend(&mut svg_pre, &pre);
        let mut svg_reg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg_reg, &regular);
        assert_ne!(svg_pre.to_svg(), svg_reg.to_svg(), "pre-bend and bend should differ");
    }

    // --- Release renderer tests ---

    #[test]
    fn release_adds_two_paths_no_text() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_release(&mut svg, &layout);
        let output = svg.to_svg();
        // 1 curve path + 1 arrowhead path = 2 paths, no text
        assert_eq!(output.matches("<path ").count(), 2, "should have curve + arrowhead");
        assert_eq!(output.matches("<text ").count(), 0, "release has no text label");
    }

    #[test]
    fn release_arrowhead_is_filled() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_release(&mut svg, &layout);
        assert!(svg.to_svg().contains("fill=\"black\""), "arrowhead should be filled");
    }

    #[test]
    fn release_curve_is_unfilled_stroke() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_release(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("fill=\"none\""), "curve should be unfilled");
        assert!(output.contains("stroke=\"black\""), "curve should have black stroke");
    }

    #[test]
    fn release_contains_quadratic_bezier() {
        let staff = test_staff();
        let layout = layout_tab_release(&staff, 1, 500.0, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_release(&mut svg, &layout);
        assert!(svg.to_svg().contains(" Q"), "release should use quadratic Bézier");
    }

    #[test]
    fn release_differs_from_regular_bend() {
        let staff = test_staff();
        let release = layout_tab_release(&staff, 1, 500.0, 5.0);
        let bend = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg_rel = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_release(&mut svg_rel, &release);
        let mut svg_bend = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg_bend, &bend);
        assert_ne!(svg_rel.to_svg(), svg_bend.to_svg(), "release and bend should differ");
    }

    // --- Bend-release renderer tests ---

    #[test]
    fn bend_release_adds_four_paths_and_text() {
        let staff = test_staff();
        let layout = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend_release(&mut svg, &layout);
        let output = svg.to_svg();
        // up-curve + down-curve + up-arrowhead + down-arrowhead = 4 paths
        assert_eq!(
            output.matches("<path ").count(),
            4,
            "should have up/down curves + up/down arrowheads = 4 paths"
        );
        assert_eq!(output.matches("<text ").count(), 1, "should have 1 apex label");
    }

    #[test]
    fn bend_release_shows_amount_label() {
        let staff = test_staff();
        let layout = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Half, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend_release(&mut svg, &layout);
        assert!(svg.to_svg().contains(">1/2</text>"), "should show '1/2' label");
    }

    #[test]
    fn bend_release_uses_two_quadratic_beziers() {
        let staff = test_staff();
        let layout = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend_release(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches(" Q").count(),
            2,
            "bend-release should draw two quadratic Bézier curves (up + down)"
        );
    }

    #[test]
    fn bend_release_arrowheads_filled() {
        let staff = test_staff();
        let layout = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend_release(&mut svg, &layout);
        assert!(svg.to_svg().contains("fill=\"black\""), "arrowheads should be filled");
    }

    #[test]
    fn bend_release_differs_from_bend() {
        let staff = test_staff();
        let br = layout_tab_bend_release(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let b = layout_tab_bend(&staff, 1, 500.0, BendAmount::Full, 5.0);
        let mut svg_br = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend_release(&mut svg_br, &br);
        let mut svg_b = SvgWriter::new(100.0, 100.0, 0.0, 0.0, 100.0, 100.0);
        draw_tab_bend(&mut svg_b, &b);
        assert_ne!(svg_br.to_svg(), svg_b.to_svg(), "bend-release and bend should differ");
    }
}
