//! SVG rendering for coordinated semantic bend-gesture segments.

use crate::layout::bend_gesture::BendSegmentLayout;
use crate::render::{SvgWriter, TextStyle};

/// Draw one resolved bend phase fragment.
pub(crate) fn draw_bend_segment(svg: &mut SvgWriter, layout: &BendSegmentLayout) {
    let control_x = (layout.x_start + layout.x_end) / 2.0;
    let control_y = if layout.is_level() {
        layout.y_start
    } else {
        (layout.y_start + layout.y_end) / 2.0 - 0.18 * layout.staff_space
    };
    let path = format!(
        "M{},{} Q{},{} {},{}",
        layout.x_start, layout.y_start, control_x, control_y, layout.x_end, layout.y_end
    );

    svg.add_raw(&format!(
        "<g data-bend-gesture=\"{}\" data-bend-string=\"{}\" data-bend-view=\"{}\" data-bend-phase=\"{}\" data-bend-fragment=\"{}\">",
        layout.gesture_index,
        layout.string,
        layout.view.label(),
        layout.phase.label(),
        layout.fragment.label(),
    ));

    // A white under-stroke keeps a same-system bend legible where it passes a
    // measure barline without semantically splitting the phase there.
    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"white\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
        path,
        layout.stroke_width * 3.0,
    ));
    svg.add_raw(&format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"black\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
        path, layout.stroke_width,
    ));

    if layout.arrow_at_end {
        let direction = if layout.y_end <= layout.y_start {
            -1.0
        } else {
            1.0
        };
        let arrow_height = 0.28 * layout.staff_space;
        let arrow_half_width = 0.14 * layout.staff_space;
        let base_y = layout.y_end - direction * arrow_height;
        svg.add_raw(&format!(
            "<path d=\"M{},{} L{},{} L{},{} Z\" fill=\"black\" stroke=\"none\"/>",
            layout.x_end,
            layout.y_end,
            layout.x_end - arrow_half_width,
            base_y,
            layout.x_end + arrow_half_width,
            base_y,
        ));
    }

    if let Some(label) = &layout.amount_label {
        svg.add_text(
            (layout.x_start + layout.x_end) / 2.0,
            layout.y_start.min(layout.y_end) - 0.35 * layout.staff_space,
            label,
            &TextStyle {
                font_family: "sans-serif",
                font_size: 0.55 * layout.staff_space,
                fill: "black",
                anchor: "middle",
                font_weight: "bold",
                font_style: "normal",
                dominant_baseline: "auto",
            },
        );
    }

    if let Some(label) = &layout.target_label {
        svg.add_text(
            layout.x_end + 0.3 * layout.staff_space,
            layout.y_end - 0.3 * layout.staff_space,
            label,
            &TextStyle {
                font_family: "serif",
                font_size: 0.5 * layout.staff_space,
                fill: "black",
                anchor: "start",
                font_weight: "normal",
                font_style: "normal",
                dominant_baseline: "auto",
            },
        );
    }

    svg.add_raw("</g>");
}
