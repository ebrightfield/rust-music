use crate::layout::hairpin::{HairpinLayout, HairpinType};
use crate::render::svg_writer::SvgWriter;

/// Draw a hairpin (crescendo/decrescendo wedge) onto the SVG.
///
/// A crescendo is drawn as two lines diverging from a point on the left
/// to an opening on the right. A decrescendo is the mirror: open on the
/// left, converging to a point on the right.
pub fn draw_hairpin(svg: &mut SvgWriter, layout: &HairpinLayout) {
    let HairpinLayout {
        kind,
        x_start,
        x_end,
        y_center,
        half_opening,
        stroke_width,
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
}
