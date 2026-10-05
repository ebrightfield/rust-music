//! SVG rendering of text spanners ("rit. - - -", "cresc. - - -").
//!
//! Consumes a [`TextSpannerLayout`] and emits the label (a left-anchored
//! `<text>`, only on segments with `has_label`) and the continuation line
//! along the label baseline: dashed, solid, or nothing. The line is omitted
//! when there is no room for it (`x_end <= x_line_start`).

use crate::layout::text_spanner::{SpannerLine, TextSpannerLayout};
use crate::render::{SvgWriter, TextStyle};

/// Draw one segment of a text spanner.
pub fn draw_text_spanner(svg: &mut SvgWriter, layout: &TextSpannerLayout) {
    if layout.has_label {
        svg.add_text(
            layout.x_start,
            layout.y_baseline,
            &layout.label,
            &TextStyle {
                font_family: "serif",
                font_size: layout.font_size,
                fill: "black",
                anchor: "start",
                font_weight: layout.font.svg_weight(),
                font_style: layout.font.svg_style(),
                dominant_baseline: "auto",
            },
        );
    }

    if layout.x_line_start >= layout.x_end {
        return;
    }
    match layout.line {
        SpannerLine::Dashed => {
            let dash_spec = format!("{},{}", layout.dash_length, layout.dash_gap);
            svg.add_dashed_line(
                layout.x_line_start,
                layout.y_baseline,
                layout.x_end,
                layout.y_baseline,
                "black",
                layout.line_thickness,
                &dash_spec,
            );
        }
        SpannerLine::Solid => svg.add_line(
            layout.x_line_start,
            layout.y_baseline,
            layout.x_end,
            layout.y_baseline,
            "black",
            layout.line_thickness,
        ),
        SpannerLine::None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::placement::Placement;
    use crate::layout::staff::StaffLayout;
    use crate::layout::text_spanner::{
        layout_text_spanner, layout_text_spanner_continuation, TextSpanner,
    };

    const SS: f64 = 250.0;

    fn render(spanner: &TextSpanner, continuation: bool) -> String {
        let staff = StaffLayout::new(0.0, 0.0, 4000.0, SS);
        let layout = if continuation {
            layout_text_spanner_continuation(spanner, 100.0, 2000.0, &staff, SS)
        } else {
            layout_text_spanner(spanner, 100.0, 2000.0, &staff, SS)
        };
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -500.0, 6000.0, 2000.0);
        draw_text_spanner(&mut svg, &layout);
        svg.to_svg()
    }

    #[test]
    fn dashed_spanner_emits_italic_label_and_dashed_line_on_the_baseline() {
        let out = render(&TextSpanner::rit(), false);
        assert!(out.contains(">rit.</text>"), "{out}");
        assert!(out.contains(r#"font-style="italic""#));
        assert!(out.contains(r#"font-weight="normal""#));
        let y = -1.6 * SS;
        assert!(out.contains(&format!(r#"y="{y}""#)), "{out}");
        assert!(
            out.contains(&format!(r#"y1="{y}" x2="2000" y2="{y}""#)),
            "{out}"
        );
        assert!(out.contains("stroke-dasharray=\"200,100\""), "{out}");
    }

    #[test]
    fn solid_and_lineless_spanners() {
        let solid = TextSpanner::new(
            "dim",
            crate::layout::text_spanner::SpannerLine::Solid,
            Placement::Above,
        );
        let out = render(&solid, false);
        assert!(out.contains("<line"));
        assert!(!out.contains("stroke-dasharray"));
        let bare = TextSpanner::new(
            "secco",
            crate::layout::text_spanner::SpannerLine::None,
            Placement::Below,
        );
        let out = render(&bare, false);
        assert!(out.contains(">secco</text>"));
        assert!(!out.contains("<line"));
    }

    #[test]
    fn continuation_segment_has_no_label() {
        let out = render(&TextSpanner::cresc(), true);
        assert!(!out.contains("<text"), "{out}");
        assert!(out.contains(r#"x1="100""#), "{out}");
    }
}
