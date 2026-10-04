use crate::font::MusicFont;
use crate::layout::tuplet::{TupletBracketLayout, TupletPlacement};
use crate::render::svg_writer::SvgWriter;

/// Draw a tuplet bracket with number onto the SVG.
///
/// Renders:
/// 1. Left hook (vertical tick), unless [`TupletBracketLayout::left_hook`] is off
/// 2. Left bracket line (from left hook to number gap)
/// 3. Number glyph(s) (centered in the gap)
/// 4. Right bracket line (from number gap to right hook)
/// 5. Right hook (vertical tick), unless [`TupletBracketLayout::right_hook`] is off
///
/// The bracket is interrupted around the number to avoid collision. With
/// [`TupletBracketLayout::show_bracket`] off only the number is drawn.
pub fn draw_tuplet_bracket(
    svg: &mut SvgWriter,
    layout: &TupletBracketLayout,
    font: &MusicFont,
    x_offset: f64,
    y_offset: f64,
) {
    let x_l = layout.x_left + x_offset;
    let x_r = layout.x_right + x_offset;
    let y = layout.bracket_y + y_offset;
    let thick = layout.bracket_thickness;

    // Hook direction: hooks point toward the notes
    let hook_dir = match layout.placement {
        TupletPlacement::Above => 1.0,
        TupletPlacement::Below => -1.0,
    };
    let hook_end_y = y + hook_dir * layout.hook_height;

    let gap_l = layout.gap_left_x + x_offset;
    let gap_r = layout.gap_right_x + x_offset;

    if layout.show_bracket {
        if layout.left_hook {
            svg.add_line(x_l, y, x_l, hook_end_y, "black", thick);
        }

        // Left bracket line (from left hook to gap)
        if gap_l > x_l {
            svg.add_line(x_l, y, gap_l, y, "black", thick);
        }

        // Right bracket line (from gap to right hook)
        if gap_r < x_r {
            svg.add_line(gap_r, y, x_r, y, "black", thick);
        }

        if layout.right_hook {
            svg.add_line(x_r, y, x_r, hook_end_y, "black", thick);
        }
    }

    // Number glyph(s)
    let mut glyph_x = layout.number_x + x_offset;
    for glyph in &layout.number_glyphs {
        if let Ok(outline) = font.glyph_outline(*glyph) {
            let transform = format!("translate({}, {})", glyph_x, y);
            svg.add_path(&outline.path_data, "black", Some(&transform));
            glyph_x += font.glyph_advance(*glyph).unwrap_or(0) as f64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::tuplet::{layout_tuplet_bracket, tuplet_number_glyphs, TupletPlacement};

    fn make_layout(placement: TupletPlacement, number: u32) -> TupletBracketLayout {
        let font = bravura_font();
        let ss = font.engraving_config().staff_space;
        let thickness_ss = font.engraving_config().tuplet_bracket_thickness;

        let glyphs = tuplet_number_glyphs(number);
        let number_width: f64 = glyphs
            .iter()
            .map(|g| font.glyph_advance(*g).unwrap_or(0) as f64)
            .sum();

        layout_tuplet_bracket(
            100.0,
            900.0,
            &[2, 4, 6],
            placement,
            number,
            ss,
            thickness_ss,
            number_width,
        )
    }

    #[test]
    fn triplet_above_renders_lines_and_path() {
        let font = bravura_font();
        let layout = make_layout(TupletPlacement::Above, 3);
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg, &layout, &font, 0.0, 0.0);
        let output = svg.to_svg();

        let line_count = output.matches("<line ").count();
        let path_count = output.matches("<path ").count();
        // 2 hooks + up to 2 bracket segments = up to 4 lines
        assert!(
            line_count >= 3,
            "expected at least 3 lines (2 hooks + bracket), got {}",
            line_count
        );
        assert!(
            path_count >= 1,
            "expected at least 1 path (number glyph), got {}",
            path_count
        );
    }

    #[test]
    fn quintuplet_below_renders() {
        let font = bravura_font();
        let layout = make_layout(TupletPlacement::Below, 5);
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg, &layout, &font, 0.0, 0.0);
        let output = svg.to_svg();
        assert!(
            output.contains("<path "),
            "should contain number glyph path"
        );
    }

    #[test]
    fn above_and_below_differ() {
        let font = bravura_font();
        let above = make_layout(TupletPlacement::Above, 3);
        let below = make_layout(TupletPlacement::Below, 3);

        let mut svg_above = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg_above, &above, &font, 0.0, 0.0);
        let mut svg_below = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg_below, &below, &font, 0.0, 0.0);

        assert_ne!(svg_above.to_svg(), svg_below.to_svg());
    }

    #[test]
    fn different_numbers_produce_different_glyphs() {
        let font = bravura_font();
        let triplet = make_layout(TupletPlacement::Above, 3);
        let quintuplet = make_layout(TupletPlacement::Above, 5);

        let mut svg3 = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg3, &triplet, &font, 0.0, 0.0);
        let mut svg5 = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg5, &quintuplet, &font, 0.0, 0.0);

        assert_ne!(svg3.to_svg(), svg5.to_svg());
    }

    #[test]
    fn x_offset_shifts_output() {
        let font = bravura_font();
        let layout = make_layout(TupletPlacement::Above, 3);

        let mut svg0 = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg0, &layout, &font, 0.0, 0.0);
        let mut svg_off = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg_off, &layout, &font, 500.0, 0.0);

        assert_ne!(svg0.to_svg(), svg_off.to_svg());
    }

    #[test]
    fn y_offset_shifts_output() {
        let font = bravura_font();
        let layout = make_layout(TupletPlacement::Above, 3);

        let mut svg0 = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg0, &layout, &font, 0.0, 0.0);
        let mut svg_off = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg_off, &layout, &font, 0.0, 300.0);

        assert_ne!(svg0.to_svg(), svg_off.to_svg());
    }

    #[test]
    fn two_digit_number_renders_two_paths() {
        let font = bravura_font();
        let layout = make_layout(TupletPlacement::Above, 12);
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg, &layout, &font, 0.0, 0.0);
        let output = svg.to_svg();
        let path_count = output.matches("<path ").count();
        assert_eq!(
            path_count, 2,
            "two-digit number should render 2 paths, got {}",
            path_count
        );
    }

    #[test]
    fn bracket_has_four_lines_with_gap() {
        let font = bravura_font();
        let layout = make_layout(TupletPlacement::Above, 3);
        let mut svg = SvgWriter::new(200.0, 200.0, 0.0, 0.0, 2000.0, 2000.0);
        draw_tuplet_bracket(&mut svg, &layout, &font, 0.0, 0.0);
        let output = svg.to_svg();
        let line_count = output.matches("<line ").count();
        // 2 hooks + 2 bracket segments (left of gap + right of gap) = 4
        assert_eq!(line_count, 4, "expected 4 lines, got {}", line_count);
    }
}
