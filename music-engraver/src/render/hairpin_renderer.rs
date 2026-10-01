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
///
/// When the layout carries a [`crate::layout::hairpin::HairpinDashStyle`]
/// (i.e. it was produced by [`crate::layout::hairpin::layout_hairpin_dashed`]),
/// the wedge lines are emitted with a `stroke-dasharray` attribute. The
/// niente "o" circle, if present, remains solid — engraved convention
/// treats the niente as a definite symbol independent of the wedge's
/// dashed/solid style.
pub fn draw_hairpin(svg: &mut SvgWriter, layout: &HairpinLayout) {
    let HairpinLayout {
        kind,
        x_start,
        x_end,
        y_center,
        half_opening,
        stroke_width,
        niente,
        dashed,
    } = *layout;

    let (top_left_y, bot_left_y, top_right_y, bot_right_y) = match kind {
        HairpinType::Crescendo => {
            // Point on left, opening on right
            (
                y_center,
                y_center,
                y_center - half_opening,
                y_center + half_opening,
            )
        }
        HairpinType::Decrescendo => {
            // Opening on left, point on right
            (
                y_center - half_opening,
                y_center + half_opening,
                y_center,
                y_center,
            )
        }
    };

    if let Some(d) = dashed {
        let dash_array = format!("{},{}", d.dash_length, d.gap_length);
        // Top line of wedge (dashed)
        svg.add_dashed_line(
            x_start,
            top_left_y,
            x_end,
            top_right_y,
            "black",
            stroke_width,
            &dash_array,
        );
        // Bottom line of wedge (dashed)
        svg.add_dashed_line(
            x_start,
            bot_left_y,
            x_end,
            bot_right_y,
            "black",
            stroke_width,
            &dash_array,
        );
    } else {
        // Top line of wedge
        svg.add_line(
            x_start,
            top_left_y,
            x_end,
            top_right_y,
            "black",
            stroke_width,
        );
        // Bottom line of wedge
        svg.add_line(
            x_start,
            bot_left_y,
            x_end,
            bot_right_y,
            "black",
            stroke_width,
        );
    }

    // Niente "o" circle — open (fill="none") so it reads as a small ring.
    // Always solid (no dashing) even on a dashed-wedge hairpin.
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
        assert_eq!(
            line_count, 2,
            "hairpin should produce 2 lines, got {line_count}"
        );
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
        assert_eq!(
            count, 2,
            "both lines should start at y_center for crescendo"
        );
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
        assert_eq!(
            count, 2,
            "both lines should end at y_center for decrescendo"
        );
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
        assert!(
            out.contains("stroke-width=\"15\""),
            "stroke-width should be 15"
        );
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
        assert!(
            out.contains(&top_str),
            "top line should end at y_center - half_opening"
        );
        assert!(
            out.contains(&bot_str),
            "bottom line should end at y_center + half_opening"
        );
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
        assert!(
            out.contains(&top_str),
            "top line should start at y_center - half_opening"
        );
        assert!(
            out.contains(&bot_str),
            "bottom line should start at y_center + half_opening"
        );
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
        let layout =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
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
        let layout =
            layout_hairpin_with_niente(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
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
        let layout =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // cx is the niente center; for crescendo, that's x_start (100.0).
        assert!(
            out.contains(r#"cx="100""#),
            "crescendo niente cx should be x_start (100.0); SVG:\n{out}"
        );
    }

    #[test]
    fn niente_decrescendo_circle_anchored_at_x_end() {
        let mut svg = make_svg();
        let layout =
            layout_hairpin_with_niente(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // cx is the niente center; for decrescendo, that's x_end (600.0).
        assert!(
            out.contains(r#"cx="600""#),
            "decrescendo niente cx should be x_end (600.0); SVG:\n{out}"
        );
    }

    #[test]
    fn niente_cy_matches_hairpin_y_center() {
        let mut svg = make_svg();
        let layout =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
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
        let layout =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
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
        let layout =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
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
        let layout = layout_hairpin_with_niente(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            custom_sw,
        );
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
        let layout =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let n = layout.niente.unwrap();
        assert!(n.radius > 0.0, "radius must be strictly positive");
        // Stay reasonable — the engraved circle is small, well under one
        // staff space in diameter.
        assert!(
            n.radius < 250.0,
            "radius should be << staff_space (250.0); got {}",
            n.radius
        );
    }

    #[test]
    fn niente_crescendo_and_decrescendo_produce_different_svg() {
        let mut svg_c = make_svg();
        let mut svg_d = make_svg();
        let c =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let d =
            layout_hairpin_with_niente(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg_c, &c);
        draw_hairpin(&mut svg_d, &d);
        // Different circle anchor (x_start vs x_end) → different SVG payload.
        assert_ne!(svg_c.to_svg(), svg_d.to_svg());
    }

    // ---- dashed hairpin ----

    use crate::layout::hairpin::layout_hairpin_dashed;

    #[test]
    fn plain_hairpin_emits_no_dasharray() {
        let mut svg = make_svg();
        let layout = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("stroke-dasharray").count(),
            0,
            "plain hairpin must not emit a stroke-dasharray attribute on any line"
        );
    }

    #[test]
    fn dashed_hairpin_emits_dasharray_on_both_lines() {
        let mut svg = make_svg();
        let layout =
            layout_hairpin_dashed(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("stroke-dasharray").count(),
            2,
            "dashed hairpin must emit stroke-dasharray on both wedge lines (got {} occurrences); SVG:\n{out}",
            out.matches("stroke-dasharray").count()
        );
    }

    #[test]
    fn dashed_hairpin_still_emits_two_lines() {
        let mut svg = make_svg();
        let layout =
            layout_hairpin_dashed(HairpinType::Decrescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("<line ").count(),
            2,
            "dashed hairpin must still produce exactly 2 wedge lines"
        );
    }

    #[test]
    fn dashed_hairpin_dasharray_value_matches_layout() {
        // dash_length = 0.4 * 250 = 100, gap_length = 0.2 * 250 = 50.
        // The dasharray string emitted by the renderer must contain those numbers.
        let mut svg = make_svg();
        let layout =
            layout_hairpin_dashed(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let d = layout.dashed.unwrap();
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        let expected = format!(r#"stroke-dasharray="{},{}""#, d.dash_length, d.gap_length);
        assert!(
            out.contains(&expected),
            "expected dasharray {expected:?} in SVG; got:\n{out}"
        );
    }

    #[test]
    fn dashed_hairpin_dasharray_uses_design_units_not_staff_spaces() {
        // Regression guard: dasharray must be in font design units (matches
        // x/y coordinates) not in staff-space units. A future refactor that
        // forgets to multiply by staff_space would emit "0.4,0.2" — much
        // too fine — and this test catches it.
        let mut svg = make_svg();
        let layout =
            layout_hairpin_dashed(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // For SS=250 the correct emitted values are 100,50. The wrong values
        // (raw staff-space constants) would be 0.4,0.2.
        assert!(out.contains(r#"stroke-dasharray="100,50""#), "SVG:\n{out}");
        assert!(
            !out.contains(r#"stroke-dasharray="0.4,0.2""#),
            "must NOT emit raw staff-space constants; SVG:\n{out}"
        );
    }

    #[test]
    fn dashed_hairpin_stroke_width_preserved() {
        let custom_sw = 14.0;
        let mut svg = make_svg();
        let layout = layout_hairpin_dashed(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            custom_sw,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // Both wedge lines must carry the custom stroke width.
        let count = out
            .matches(&format!(r#"stroke-width="{custom_sw}""#))
            .count();
        assert_eq!(
            count, 2,
            "stroke-width {custom_sw} should appear on both dashed wedge lines; got {count}; SVG:\n{out}"
        );
    }

    #[test]
    fn dashed_and_plain_hairpin_produce_different_svg() {
        let mut svg_p = make_svg();
        let mut svg_d = make_svg();
        let p = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let d = layout_hairpin_dashed(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg_p, &p);
        draw_hairpin(&mut svg_d, &d);
        assert_ne!(
            svg_p.to_svg(),
            svg_d.to_svg(),
            "dashed and plain hairpins must produce visually distinct SVG"
        );
    }

    #[test]
    fn dashed_lines_share_same_dasharray_value() {
        // Both wedge lines must carry the SAME dasharray (a single dash
        // pattern across the whole wedge — not a per-line override).
        let mut svg = make_svg();
        let layout =
            layout_hairpin_dashed(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // Extract all stroke-dasharray="..." values and assert they're all equal.
        let mut values = vec![];
        for line in out.lines() {
            if let Some(start) = line.find(r#"stroke-dasharray=""#) {
                let after = &line[start + r#"stroke-dasharray=""#.len()..];
                if let Some(end) = after.find('"') {
                    values.push(after[..end].to_string());
                }
            }
        }
        assert_eq!(
            values.len(),
            2,
            "expected 2 dasharray values; got {values:?}"
        );
        assert_eq!(
            values[0], values[1],
            "both wedge lines must share the same dasharray"
        );
    }

    #[test]
    fn dashed_decrescendo_x_coordinates_in_svg() {
        let mut svg = make_svg();
        let layout =
            layout_hairpin_dashed(HairpinType::Decrescendo, 200.0, 800.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(out.contains(r#"x1="200""#), "x_start should appear as x1");
        assert!(out.contains(r#"x2="800""#), "x_end should appear as x2");
    }

    #[test]
    fn dashed_hairpin_with_niente_keeps_circle_solid() {
        // Combo via field mutation: dashed wedge + niente circle. The wedge
        // lines must be dashed; the niente circle must remain solid (no
        // stroke-dasharray attribute on the <circle> element). Engraved
        // convention: the niente "o" is a definite symbol, never dashed.
        let mut layout =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        layout.dashed = Some(crate::layout::hairpin::HairpinDashStyle {
            dash_length: 100.0,
            gap_length: 50.0,
        });
        let mut svg = make_svg();
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        // Wedge lines: 2 dashed.
        assert_eq!(out.matches("<line ").count(), 2);
        // The two wedge lines carry dasharray.
        assert_eq!(out.matches("stroke-dasharray").count(), 2);
        // Niente circle is present.
        assert_eq!(out.matches("<circle ").count(), 1);
        // The circle element must NOT carry stroke-dasharray.
        let circle_line = out
            .lines()
            .find(|l| l.contains("<circle "))
            .expect("circle element must be present in combined output");
        assert!(
            !circle_line.contains("stroke-dasharray"),
            "niente circle must remain solid even on a dashed hairpin; got: {circle_line}"
        );
        // And the circle is still an open "o".
        assert!(circle_line.contains(r#"fill="none""#));
    }

    #[test]
    fn dashed_hairpin_alone_emits_no_circle() {
        // The dashed constructor must NOT also set niente — symmetry with
        // the layout-level "dashed_does_not_set_niente" test.
        let mut svg = make_svg();
        let layout =
            layout_hairpin_dashed(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("<circle ").count(),
            0,
            "dashed-only hairpin must emit zero <circle> elements"
        );
    }

    // ---- open-end niente (rare mirror of closed-end niente) ----

    use crate::layout::hairpin::layout_hairpin_with_niente_at_open_end;

    #[test]
    fn open_end_niente_hairpin_emits_exactly_one_circle() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("<circle ").count(),
            1,
            "open-end niente hairpin must produce exactly one <circle> element"
        );
    }

    #[test]
    fn open_end_niente_hairpin_still_emits_two_wedge_lines() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Decrescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("<line ").count(),
            2,
            "open-end niente decoration must not displace the wedge — still 2 lines"
        );
    }

    #[test]
    fn open_end_niente_crescendo_circle_anchored_at_x_end() {
        // For crescendo, the open end is the wide right side → cx == x_end.
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(
            out.contains(r#"cx="600""#),
            "open-end crescendo niente cx should be x_end (600.0); SVG:\n{out}"
        );
        assert!(
            !out.contains(r#"cx="100""#),
            "open-end crescendo niente must NOT be at x_start (100.0); SVG:\n{out}"
        );
    }

    #[test]
    fn open_end_niente_decrescendo_circle_anchored_at_x_start() {
        // For decrescendo, the open end is the wide left side → cx == x_start.
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Decrescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(
            out.contains(r#"cx="100""#),
            "open-end decrescendo niente cx should be x_start (100.0); SVG:\n{out}"
        );
        assert!(
            !out.contains(r#"cx="600""#),
            "open-end decrescendo niente must NOT be at x_end (600.0); SVG:\n{out}"
        );
    }

    #[test]
    fn open_end_niente_cy_matches_hairpin_y_center() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        let cy = layout.y_center;
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(
            out.contains(&format!(r#"cy="{cy}""#)),
            "open-end niente cy should equal y_center ({cy}); SVG:\n{out}"
        );
    }

    #[test]
    fn open_end_niente_circle_is_open_o_not_filled_dot() {
        // Same convention as closed-end: open "o", fill="none", stroked black.
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        let circle_line = out
            .lines()
            .find(|l| l.contains("<circle "))
            .expect("circle element must be present");
        assert!(
            circle_line.contains(r#"fill="none""#),
            "open-end niente must be an open circle (fill=\"none\"); got: {circle_line}"
        );
        assert!(
            circle_line.contains(r#"stroke="black""#),
            "open-end niente must be stroked black; got: {circle_line}"
        );
    }

    #[test]
    fn open_end_niente_circle_radius_in_svg_matches_layout() {
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        let r = layout.niente.unwrap().radius;
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(
            out.contains(&format!(r#"r="{r}""#)),
            "open-end niente r should be {r}; SVG:\n{out}"
        );
    }

    #[test]
    fn open_end_niente_circle_stroke_width_matches_hairpin_stroke() {
        let custom_sw = 13.0;
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            custom_sw,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        let circle_line = out
            .lines()
            .find(|l| l.contains("<circle "))
            .expect("circle present");
        assert!(
            circle_line.contains(&format!(r#"stroke-width="{custom_sw}""#)),
            "open-end niente circle stroke-width should equal {custom_sw}; got: {circle_line}"
        );
        // Wedge lines + circle = 3 elements with the custom stroke width.
        let line_count_with_sw = out
            .matches(&format!(r#"stroke-width="{custom_sw}""#))
            .count();
        assert_eq!(
            line_count_with_sw, 3,
            "stroke-width {custom_sw} should appear on 2 wedge lines + 1 circle = 3; got {line_count_with_sw}"
        );
    }

    #[test]
    fn open_end_and_closed_end_niente_produce_different_svg() {
        // The two constructors must produce visually distinct SVG for the same
        // hairpin kind and args — different anchor → different cx in output.
        let c =
            layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, 1000.0, 250.0, 10.0);
        let o = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        let mut svg_c = make_svg();
        let mut svg_o = make_svg();
        draw_hairpin(&mut svg_c, &c);
        draw_hairpin(&mut svg_o, &o);
        assert_ne!(
            svg_c.to_svg(),
            svg_o.to_svg(),
            "closed-end and open-end niente hairpins must produce visually distinct SVG"
        );
    }

    #[test]
    fn open_end_niente_emits_no_dasharray_by_default() {
        // open-end niente constructor must not silently dashes the wedge —
        // mirror of niente/dashed independence on the closed-end variant.
        let mut svg = make_svg();
        let layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(
            out.matches("stroke-dasharray").count(),
            0,
            "open-end-niente-only hairpin must not emit stroke-dasharray on any element"
        );
    }

    #[test]
    fn open_end_dashed_combo_keeps_circle_solid() {
        // Combo via field mutation: dashed wedge + open-end niente. The wedge
        // lines must be dashed; the niente circle must remain solid. Mirrors
        // the closed-end combo invariant.
        let mut layout = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo,
            100.0,
            600.0,
            1000.0,
            250.0,
            10.0,
        );
        layout.dashed = Some(crate::layout::hairpin::HairpinDashStyle {
            dash_length: 100.0,
            gap_length: 50.0,
        });
        let mut svg = make_svg();
        draw_hairpin(&mut svg, &layout);
        let out = svg.to_svg();
        assert_eq!(out.matches("<line ").count(), 2);
        assert_eq!(out.matches("stroke-dasharray").count(), 2);
        assert_eq!(out.matches("<circle ").count(), 1);
        let circle_line = out
            .lines()
            .find(|l| l.contains("<circle "))
            .expect("circle element must be present in combined output");
        assert!(
            !circle_line.contains("stroke-dasharray"),
            "open-end niente circle must remain solid even on a dashed hairpin; got: {circle_line}"
        );
        assert!(circle_line.contains(r#"fill="none""#));
        // Open-end crescendo niente is at x_end (600.0).
        assert!(circle_line.contains(r#"cx="600""#));
    }
}
