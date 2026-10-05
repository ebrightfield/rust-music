use smufl::Glyph;

use crate::font::{FontError, MusicFont};
use crate::layout::dot::{
    dot_staff_position, first_dot_x, DOT_GLYPH, DOT_INTER_DOT_SPACING_SS, DOT_PARENTHESES_GAP_SS,
};
use crate::layout::staff::StaffLayout;
use crate::layout::StaffPosition;
use crate::render::note_renderer::glyph_transform;
use crate::render::SvgWriter;

/// Draw augmentation dots for a note drawn at `scale` (1.0 for normal size).
///
/// Renders `dot_count` augmentation dots to the right of a notehead at
/// the given staff position. Dots on staff lines are shifted up to the
/// space above per standard engraving convention. With `parenthesized`
/// (LilyPond `Dots.parenthesized`) the dots are enclosed in notehead
/// parentheses, the opening one taking the dots' usual gap after the
/// notehead.
///
/// Returns the x-position just past the last dot (or closing parenthesis),
/// or `None` if `dot_count` is 0.
#[allow(clippy::too_many_arguments)]
pub fn draw_dots(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    notehead_x: f64,
    notehead_advance: f64,
    position: StaffPosition,
    dot_count: u8,
    scale: f64,
    parenthesized: bool,
) -> Result<Option<f64>, FontError> {
    if dot_count == 0 {
        return Ok(None);
    }

    let unit = staff.staff_space * scale;
    let outline = font.glyph_outline(DOT_GLYPH)?;
    let dot_advance = outline.advance_width as f64 * scale;
    let y = staff.y_of(dot_staff_position(position));

    let first_x = if parenthesized {
        let open = font.glyph_outline(Glyph::NoteheadParenthesisLeft)?;
        let open_x = notehead_x + notehead_advance + DOT_PARENTHESES_GAP_SS * unit;
        svg.add_path(
            &open.path_data,
            "black",
            Some(&glyph_transform(open_x, y, scale)),
        );
        open_x + open.advance_width as f64 * scale + DOT_PARENTHESES_GAP_SS * unit
    } else {
        first_dot_x(notehead_x, notehead_advance, unit)
    };

    let inter_dot = DOT_INTER_DOT_SPACING_SS * unit;
    let mut end_x = first_x;
    for index in 0..dot_count {
        let x = first_x + f64::from(index) * inter_dot;
        svg.add_path(
            &outline.path_data,
            "black",
            Some(&glyph_transform(x, y, scale)),
        );
        end_x = x + dot_advance;
    }

    if parenthesized {
        let close = font.glyph_outline(Glyph::NoteheadParenthesisRight)?;
        let close_x = end_x + DOT_PARENTHESES_GAP_SS * unit;
        svg.add_path(
            &close.path_data,
            "black",
            Some(&glyph_transform(close_x, y, scale)),
        );
        end_x = close_x + close.advance_width as f64 * scale;
    }
    Ok(Some(end_x))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{bravura_font, EngravingConfig};
    use crate::layout::dot::DOT_NOTEHEAD_PADDING_SS;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    #[test]
    fn zero_dots_returns_none_and_no_elements() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        let result = draw_dots(&mut svg, &staff, &font, 500.0, 295.0, 4, 0, 1.0, false).unwrap();
        assert!(result.is_none());
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<path ").count(),
            0,
            "no dots should be drawn"
        );
    }

    #[test]
    fn single_dot_produces_one_path() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let result = draw_dots(&mut svg, &staff, &font, 500.0, 295.0, 5, 1, 1.0, false).unwrap();
        assert!(result.is_some());
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1, "exactly one dot path");
    }

    #[test]
    fn double_dot_produces_two_paths() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_dots(&mut svg, &staff, &font, 500.0, 295.0, 5, 2, 1.0, false).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 2, "two dot paths");
    }

    #[test]
    fn triple_dot_produces_three_paths() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_dots(&mut svg, &staff, &font, 500.0, 295.0, 5, 3, 1.0, false).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 3, "three dot paths");
    }

    #[test]
    fn dot_in_space_uses_note_position_y() {
        let (font, _, staff) = setup();
        // Position 5 is a space — dot should stay at position 5
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2000.0);
        draw_dots(&mut svg, &staff, &font, 500.0, 295.0, 5, 1, 1.0, false).unwrap();
        let output = svg.to_svg();

        let expected_y = staff.y_of(5);
        let translate = "translate(".to_string();
        assert!(output.contains(&translate));
        // Verify the y component of the translate
        let y_str = format!(", {expected_y})");
        assert!(
            output.contains(&y_str),
            "dot y should be {expected_y}, got: {output}"
        );
    }

    #[test]
    fn dot_on_line_shifts_up_to_space() {
        let (font, _, staff) = setup();
        // Position 4 is middle line — dot should be at position 5 (space above)
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2000.0);
        draw_dots(&mut svg, &staff, &font, 500.0, 295.0, 4, 1, 1.0, false).unwrap();
        let output = svg.to_svg();

        let shifted_y = staff.y_of(5); // space above middle line
        let y_str = format!(", {shifted_y})");
        assert!(
            output.contains(&y_str),
            "dot on line should shift up: expected y={shifted_y}, got: {output}"
        );
    }

    #[test]
    fn dot_on_bottom_line_shifts_to_first_space() {
        let (font, _, staff) = setup();
        // Position 0 → dot at position 1
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2000.0);
        draw_dots(&mut svg, &staff, &font, 500.0, 295.0, 0, 1, 1.0, false).unwrap();
        let output = svg.to_svg();

        let shifted_y = staff.y_of(1);
        let y_str = format!(", {shifted_y})");
        assert!(
            output.contains(&y_str),
            "dot at bottom line should shift to first space"
        );
    }

    #[test]
    fn dot_x_is_right_of_notehead() {
        let (font, _, staff) = setup();
        let notehead_x = 500.0;
        let notehead_advance = 295.0;
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2000.0);
        draw_dots(
            &mut svg,
            &staff,
            &font,
            notehead_x,
            notehead_advance,
            5,
            1,
            1.0,
            false,
        )
        .unwrap();
        let output = svg.to_svg();

        let ss = staff.staff_space;
        let expected_x = notehead_x + notehead_advance + DOT_NOTEHEAD_PADDING_SS * ss;
        let x_str = format!("translate({expected_x}");
        assert!(
            output.contains(&x_str),
            "dot should be at x={expected_x}, got: {output}"
        );
    }

    #[test]
    fn returns_x_past_last_dot() {
        let (font, _, staff) = setup();
        let result = draw_dots(
            &mut svg_for_test(),
            &staff,
            &font,
            500.0,
            295.0,
            5,
            1,
            1.0,
            false,
        )
        .unwrap();
        let end_x = result.unwrap();
        let ss = staff.staff_space;
        let first_x = 500.0 + 295.0 + DOT_NOTEHEAD_PADDING_SS * ss;
        let dot_advance = font.glyph_advance(DOT_GLYPH).unwrap() as f64;
        let expected_end = first_x + dot_advance;
        assert!(
            (end_x - expected_end).abs() < 1e-6,
            "end_x={end_x}, expected={expected_end}"
        );
    }

    fn svg_for_test() -> SvgWriter {
        SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2000.0)
    }
}
