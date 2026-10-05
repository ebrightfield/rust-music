//! Draw bar numbers placed by [`crate::layout::bar_number`].

use crate::layout::bar_number::{BarNumberLayout, BAR_NUMBER_FONT_SIZE_SS};
use crate::render::{SvgWriter, TextStyle};

/// Draw each placed bar number as left-aligned serif text.
pub fn draw_bar_numbers(svg: &mut SvgWriter, numbers: &[BarNumberLayout], staff_space: f64) {
    let style = TextStyle {
        font_family: "serif",
        font_size: BAR_NUMBER_FONT_SIZE_SS * staff_space,
        fill: "black",
        anchor: "start",
        font_weight: "normal",
        font_style: "normal",
        dominant_baseline: "auto",
    };
    for number in numbers {
        svg.add_text(number.x, number.y, &number.number.to_string(), &style);
    }
}
