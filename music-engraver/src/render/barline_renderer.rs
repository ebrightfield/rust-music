use crate::font::{FontError, MusicFont};
use crate::layout::barline::{barline_layout, BarlineLayout, BarlineStyle};
use crate::layout::staff::StaffLayout;
use crate::render::SvgWriter;
use smufl::Glyph;

/// Draw a barline of the given style at position `x`.
///
/// Returns the total advance width of the barline group.
pub fn draw_barline(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    style: BarlineStyle,
) -> Result<f64, FontError> {
    let config = font.engraving_config();
    let layout = barline_layout(style, x, staff, &config);
    render_barline_layout(svg, font, &layout)
}

/// Render a pre-computed barline layout to SVG.
fn render_barline_layout(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &BarlineLayout,
) -> Result<f64, FontError> {
    for stroke in &layout.strokes {
        svg.add_line(
            stroke.x,
            stroke.y_top,
            stroke.x,
            stroke.y_bottom,
            "black",
            stroke.thickness,
        );
    }

    if let Some(ref dots) = layout.dots {
        let outline = font.glyph_outline(Glyph::RepeatDot)?;
        let advance = outline.advance_width as f64;
        // Centre the dot glyph on the dot x-position
        let glyph_x = dots.x - advance / 2.0;

        let upper_transform = format!("translate({glyph_x}, {})", dots.y_upper);
        svg.add_path(&outline.path_data, "black", Some(&upper_transform));

        let lower_transform = format!("translate({glyph_x}, {})", dots.y_lower);
        svg.add_path(&outline.path_data, "black", Some(&lower_transform));
    }

    Ok(layout.width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn setup() -> (MusicFont<'static>, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, staff)
    }

    #[test]
    fn single_barline_produces_one_line() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let width = draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::Single).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 1);
        assert_eq!(output.matches("<path ").count(), 0);
        assert!(width > 0.0);
    }

    #[test]
    fn double_barline_produces_two_lines() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::Double).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 2);
    }

    #[test]
    fn final_barline_produces_two_lines() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::Final).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 2);
    }

    #[test]
    fn final_barline_second_line_is_thicker() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::Final).unwrap();
        let output = svg.to_svg();
        // Extract stroke-width values from <line> elements
        let widths: Vec<f64> = output
            .match_indices("stroke-width=\"")
            .map(|(i, _)| {
                let start = i + "stroke-width=\"".len();
                let end = output[start..].find('"').unwrap() + start;
                output[start..end].parse::<f64>().unwrap()
            })
            .collect();
        assert_eq!(widths.len(), 2);
        assert!(
            widths[1] > widths[0],
            "thick stroke should be wider than thin"
        );
    }

    #[test]
    fn end_repeat_has_two_lines_and_two_dot_paths() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::EndRepeat).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 2);
        assert_eq!(output.matches("<path ").count(), 2, "two repeat dots");
    }

    #[test]
    fn start_repeat_has_two_lines_and_two_dot_paths() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::StartRepeat).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 2);
        assert_eq!(output.matches("<path ").count(), 2);
    }

    #[test]
    fn barline_width_is_positive_for_all_styles() {
        let (font, staff) = setup();
        for style in [
            BarlineStyle::Single,
            BarlineStyle::Double,
            BarlineStyle::Final,
            BarlineStyle::StartRepeat,
            BarlineStyle::EndRepeat,
        ] {
            let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
            let width = draw_barline(&mut svg, &staff, &font, 0.0, style).unwrap();
            assert!(width > 0.0, "{style:?} barline should have positive width");
        }
    }

    #[test]
    fn single_barline_line_spans_full_staff_height() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::Single).unwrap();
        let output = svg.to_svg();
        let y_top = staff.y_of(8);
        let y_bottom = staff.y_of(0);
        assert!(output.contains(&format!("y1=\"{y_top}\"")));
        assert!(output.contains(&format!("y2=\"{y_bottom}\"")));
    }

    #[test]
    fn repeat_dots_reference_repeat_dot_glyph() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_barline(&mut svg, &staff, &font, 500.0, BarlineStyle::EndRepeat).unwrap();
        let output = svg.to_svg();
        // Both dot paths should have the same path data (same glyph)
        let paths: Vec<&str> = output
            .match_indices(" d=\"")
            .map(|(i, _)| {
                let start = i + 4;
                let end = output[start..].find('"').unwrap() + start;
                &output[start..end]
            })
            .collect();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], paths[1], "both repeat dots use the same glyph");
    }
}
