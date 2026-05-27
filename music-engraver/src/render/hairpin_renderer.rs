use crate::layout::hairpin::{HairpinLayout, HairpinType};
use crate::render::svg_writer::SvgWriter;

/// Draw a hairpin (crescendo/decrescendo wedge) onto the SVG.
///
/// A crescendo is drawn as two lines diverging from a point on the left
/// to an opening on the right. A decrescendo is the mirror: open on the
/// left, converging to a point on the right.
///
/// When the layout carries a [`crate::layout::hairpin::NienteCircleLayout`]
/// (i.e. it was produced by [`crate::layout::hairpin::layout_hairpin_with_niente`]),
/// an additional open "o" circle is drawn at the closed end of the wedge to
/// mark to/from silence.
pub fn draw_hairpin(svg: &mut SvgWriter, layout: &HairpinLayout) {
    let HairpinLayout {
        kind,
        x_start,
        x_end,
        y_center,
        half_opening,
        stroke_width,
        niente,
    } = *layout;

    let (top_left_y, bot_left_y, top_right_y, bot_right_y) = match kind {
        HairpinType::Crescendo => {
            // Point on left, opening on right
            (y_center, y_center, y_center - half_opening, y_center + half_opening)
        }
        HairpinType::Decrescendo => {
            // Opening on left, point on right
            (y_center - half_opening, y_center + half_opening, y_center, y_center)
        }
    };

    // Top line of wedge
    svg.add_line(x_start, top_left_y, x_end, top_right_y, "black", stroke_width);
    // Bottom line of wedge
    svg.add_line(x_start, bot_left_y, x_end, bot_right_y, "black", stroke_width);

    // Niente "o" circle — open (fill="none") so it reads as a small ring.
    if let Some(n) = niente {
        svg.add_circle(n.cx, n.cy, n.radius, "black", n.stroke_width, "none");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::hairpin::layout_hairpin;

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 200.0, 0.0, 0.0, 2000.0, 500.0)
    }

    #[test]
    fn crescendo_produces_two_lines() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        let line_count = out.matches("<line ").count();
        assert_eq!(line_count, 2, "hairpin should produce 2 lines, got {line_count}");
    }

    #[test]
    fn decrescendo_produces_two_lines() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        let line_count = out.matches("<line ").count();
        assert_eq!(line_count, 2);
    }

    #[test]
    fn crescendo_starts_at_point() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // Both lines start at the same y (the center point)
        // The two lines should have x1="100" and y1 = y_center
        let y_center = layout.y_center;
        let y_str = format!("y1=\"{y_center}\"");
        let count = out.matches(&y_str).count();
        assert_eq!(count, 2, "both lines should start at y_center for crescendo");
    }

    #[test]
    fn decrescendo_ends_at_point() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // Both lines end at the same y (the center point)
        let y_center = layout.y_center;
        let y_str = format!("y2=\"{y_center}\"");
        let count = out.matches(&y_str).count();
        assert_eq!(count, 2, "both lines should end at y_center for decrescendo");
    }

    #[test]
    fn crescendo_and_decrescendo_produce_different_svg() {
        let mut svg_c = make_svg();
        let mut svg_d = make_svg();
        let c = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let d = layout_hairpin(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg_c, &c);
        draw_hairpin(&mut svg_d, &d);
        assert_ne!(svg_c.to_svg(), svg_d.to_svg());
    }

    #[test]
    fn stroke_width_appears_in_svg() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 15.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains("stroke-width=\"15\""), "stroke-width should be 15");
    }

    #[test]
    fn x_coordinates_in_svg() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Crescendo, 200.0, 800.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains("x1=\"200\""), "x_start should appear as x1");
        assert!(out.contains("x2=\"800\""), "x_end should appear as x2");
    }

    #[test]
    fn crescendo_right_end_has_opening() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // Right end should have two different y2 values (y_center ± half_opening)
        let y_top = layout.y_center - layout.half_opening;
        let y_bot = layout.y_center + layout.half_opening;
        let top_str = format!("y2=\"{y_top}\"");
        let bot_str = format!("y2=\"{y_bot}\"");
        assert!(out.contains(&top_str), "top line should end at y_center - half_opening");
        assert!(out.contains(&bot_str), "bottom line should end at y_center + half_opening");
    }

    #[test]
    fn decrescendo_left_end_has_opening() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        let y_top = layout.y_center - layout.half_opening;
        let y_bot = layout.y_center + layout.half_opening;
        let top_str = format!("y1=\"{y_top}\"");
        let bot_str = format!("y1=\"{y_bot}\"");
        assert!(out.contains(&top_str), "top line should start at y_center - half_opening");
        assert!(out.contains(&bot_str), "bottom line should start at y_center + half_opening");
    }

    // ---- niente circle ----

    use crate::layout::hairpin::layout_hairpin_with_niente;

    #[test]
    fn plain_hairpin_emits_no_circle() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("<circle ").count(),
            0,
            "plain hairpin must produce zero <circle> elements"
        );
    }

    #[test]
    fn niente_hairpin_emits_exactly_one_circle() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("<circle ").count(),
            1,
            "niente hairpin must produce exactly one <circle> element"
        );
    }

    #[test]
    fn niente_hairpin_still_emits_two_wedge_lines() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("<line ").count(),
            2,
            "niente decoration must not displace the wedge — still 2 lines"
        );
    }

    #[test]
    fn niente_crescendo_circle_anchored_at_x_start() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // cx is the niente center; for crescendo, that's x_start (100.0).
        assert!(out.contains(r#"cx="100""#), "crescendo niente cx should be x_start (100.0); SVG:\n{out}");
    }

    #[test]
    fn niente_decrescendo_circle_anchored_at_x_end() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // cx is the niente center; for decrescendo, that's x_end (600.0).
        assert!(out.contains(r#"cx="600""#), "decrescendo niente cx should be x_end (600.0); SVG:\n{out}");
    }

    #[test]
    fn niente_cy_matches_hairpin_y_center() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let cy = layout.y_center;
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(
            out.contains(&format!(r#"cy="{cy}""#)),
            "niente cy should equal y_center ({cy}); SVG:\n{out}"
        );
    }

    #[test]
    fn niente_circle_is_open_o_not_filled_dot() {
        // Engraved niente is an open "o" — fill="none", stroked. A filled
        // dot would be visually indistinguishable from a fermata dot or
        // staccato and is wrong notation.
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // Extract the circle line — it must carry fill="none" AND stroke="black".
        let circle_line = out
            .lines()
            .find(|l| l.contains("<circle "))
            .expect("circle element must be present");
        assert!(
            circle_line.contains(r#"fill="none""#),
            "niente must be an open circle (fill=\"none\"); got: {circle_line}"
        );
        assert!(
            circle_line.contains(r#"stroke="black""#),
            "niente must be stroked black; got: {circle_line}"
        );
    }

    #[test]
    fn niente_circle_radius_in_svg_matches_layout() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let r = layout.niente.unwrap().radius;
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(
            out.contains(&format!(r#"r="{r}""#)),
            "niente r should be {r}; SVG:\n{out}"
        );
    }

    #[test]
    fn niente_circle_stroke_width_matches_hairpin_stroke() {
        // Custom stroke width — niente ring must inherit it so the circle
        // reads as the same line weight as the wedge.
        let custom_sw = 13.0;
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, custom_sw);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        let circle_line = out
            .lines()
            .find(|l| l.contains("<circle "))
            .expect("circle present");
        assert!(
            circle_line.contains(&format!(r#"stroke-width="{custom_sw}""#)),
            "niente circle stroke-width should equal {custom_sw}; got: {circle_line}"
        );
        // And the wedge lines should also carry the same stroke-width.
        let line_count_with_sw = out
            .matches(&format!(r#"stroke-width="{custom_sw}""#))
            .count();
        assert_eq!(
            line_count_with_sw, 3,
            "stroke-width {custom_sw} should appear on 2 wedge lines + 1 circle = 3; got {line_count_with_sw}"
        );
    }

    #[test]
    fn niente_circle_lives_inside_viewbox_bounds_for_typical_layout() {
        // Sanity: a typical layout should not place the niente off-staff at
        // negative-or-zero radius. Smoke check that catches a future regression
        // where radius is computed from the wrong factor.
        let layout = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let n = layout.niente.unwrap();
        assert!(n.radius > 0.0, "radius must be strictly positive");
        // Stay reasonable — the engraved circle is small, well under one
        // staff space in diameter.
        assert!(n.radius < 250.0, "radius should be << staff_space (250.0); got {}", n.radius);
    }

    #[test]
    fn niente_crescendo_and_decrescendo_produce_different_svg() {
        let mut svg_c = make_svg();
        let mut svg_d = make_svg();
        let c = layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let d = layout_hairpin_with_niente(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg_c, &c);
        draw_hairpin(&mut svg_d, &d);
        // Different circle anchor (x_start vs x_end) → different SVG payload.
        assert_ne!(svg_c.to_svg(), svg_d.to_svg());
    }
}
