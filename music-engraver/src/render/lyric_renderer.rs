/// SVG rendering for lyric text syllables.
///
/// Renders roman (upright) text below the staff, with optional trailing
/// hyphen or extender line, using the geometry from [`crate::layout::lyric`].
use crate::layout::lyric::{LyricContinuation, LyricLayout};
use crate::render::svg_writer::TextStyle;
use crate::render::SvgWriter;

/// Draw a lyric syllable onto the SVG.
///
/// Renders roman serif text centered on the note's x-position, below the staff.
/// Appends a trailing hyphen character if continuation is `Hyphen`.
/// Extender lines are drawn separately by [`draw_lyric_extender`] in a
/// second pass once all note positions are known.
pub fn draw_lyric(svg: &mut SvgWriter, layout: &LyricLayout) {
    let display_text = match &layout.continuation {
        LyricContinuation::Hyphen => format!("{} -", layout.text),
        _ => layout.text.clone(),
    };

    svg.add_text(
        layout.x_center,
        layout.y_baseline,
        &display_text,
        &TextStyle::normal(layout.font_size),
    );
}

/// Horizontal padding before the extender line starts (past the syllable text),
/// in staff spaces.
const EXTENDER_LEFT_PAD_SS: f64 = 0.4;

/// Horizontal padding before the target note where the extender line ends,
/// in staff spaces.
const EXTENDER_RIGHT_PAD_SS: f64 = 0.2;

/// Draw a melisma extender line between a syllable with `Extender` continuation
/// and the next syllable's note position.
///
/// `from_x` is the center-x of the source syllable's note.
/// `to_x` is the center-x of the target note (where the next lyric starts).
/// `y_baseline` is the lyric text baseline (shared across all lyrics in a system).
/// `staff_space` is the staff space size in font design units.
/// `stroke_width` is the line thickness.
pub fn draw_lyric_extender(
    svg: &mut SvgWriter,
    from_x: f64,
    to_x: f64,
    y_baseline: f64,
    staff_space: f64,
    stroke_width: f64,
) {
    let x_start = from_x + EXTENDER_LEFT_PAD_SS * staff_space;
    let x_end = to_x - EXTENDER_RIGHT_PAD_SS * staff_space;

    // Only draw if there is meaningful horizontal space
    if x_end <= x_start + staff_space * 0.2 {
        return;
    }

    svg.add_line(x_start, y_baseline, x_end, y_baseline, "black", stroke_width);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::lyric::{layout_lyric, LyricSyllable};
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    fn test_svg() -> SvgWriter {
        SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn lyric_renders_text_element() {
        let syl = LyricSyllable::word("day");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<text"), "should contain a text element");
        assert!(output.contains(">day<"), "should contain 'day'");
    }

    #[test]
    fn lyric_is_not_italic() {
        let syl = LyricSyllable::word("la");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            !output.contains("italic"),
            "lyrics should be roman (upright), not italic"
        );
    }

    #[test]
    fn lyric_is_centered() {
        let syl = LyricSyllable::word("sing");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("middle"),
            "lyric text should be centered (text-anchor: middle)"
        );
    }

    #[test]
    fn hyphen_continuation_appends_hyphen() {
        let syl = LyricSyllable::with_hyphen("hap");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("hap -"),
            "hyphen syllable should show 'hap -' but got: {}",
            output
        );
    }

    #[test]
    fn no_continuation_has_no_hyphen() {
        let syl = LyricSyllable::word("day");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            !output.contains("day -"),
            "word-end syllable should not have a trailing hyphen"
        );
    }

    #[test]
    fn extender_renders_text_only() {
        let syl = LyricSyllable::with_extender("love");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains(">love<"), "should contain 'love'");
        // Extender line is drawn separately (needs next note x position)
        assert!(
            !output.contains("hap -"),
            "extender should not have a hyphen"
        );
    }

    #[test]
    fn different_lyrics_produce_different_output() {
        let syl_a = LyricSyllable::word("day");
        let syl_b = LyricSyllable::word("night");
        let layout_a = layout_lyric(&syl_a, 500.0, &test_staff(), 250.0);
        let layout_b = layout_lyric(&syl_b, 500.0, &test_staff(), 250.0);

        let mut svg_a = test_svg();
        draw_lyric(&mut svg_a, &layout_a);

        let mut svg_b = test_svg();
        draw_lyric(&mut svg_b, &layout_b);

        assert_ne!(svg_a.to_svg(), svg_b.to_svg());
    }

    #[test]
    fn text_count_is_one() {
        let syl = LyricSyllable::word("la");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<text").count(),
            1,
            "should produce exactly 1 text element"
        );
    }

    #[test]
    fn lyric_has_no_path() {
        let syl = LyricSyllable::word("test");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<path").count(),
            0,
            "lyric text should have no paths"
        );
    }

    #[test]
    fn extender_line_draws_horizontal_line() {
        let mut svg = test_svg();
        draw_lyric_extender(&mut svg, 200.0, 800.0, 1500.0, 250.0, 5.0);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line").count(),
            1,
            "extender should produce exactly 1 line element"
        );
        // y1 == y2 (horizontal line at baseline)
        assert!(output.contains("y1=\"1500\""), "y1 should be at baseline");
        assert!(output.contains("y2=\"1500\""), "y2 should be at baseline");
    }

    #[test]
    fn extender_line_has_correct_x_range() {
        let mut svg = test_svg();
        let staff_space = 250.0;
        draw_lyric_extender(&mut svg, 200.0, 800.0, 1500.0, staff_space, 5.0);
        let output = svg.to_svg();
        // x_start = 200 + 0.4*250 = 300
        let expected_x1 = 200.0 + super::EXTENDER_LEFT_PAD_SS * staff_space;
        assert!(
            output.contains(&format!("x1=\"{expected_x1}\"")),
            "x1 should be from_x + left padding, got: {output}"
        );
        // x_end = 800 - 0.2*250 = 750
        let expected_x2 = 800.0 - super::EXTENDER_RIGHT_PAD_SS * staff_space;
        assert!(
            output.contains(&format!("x2=\"{expected_x2}\"")),
            "x2 should be to_x - right padding, got: {output}"
        );
    }

    #[test]
    fn extender_line_not_drawn_when_too_short() {
        let mut svg = test_svg();
        // from_x=200, to_x=250 — after padding the span is negative
        draw_lyric_extender(&mut svg, 200.0, 250.0, 1500.0, 250.0, 5.0);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line").count(),
            0,
            "extender should not be drawn when span is too short"
        );
    }

    #[test]
    fn extender_line_uses_given_stroke_width() {
        let mut svg = test_svg();
        draw_lyric_extender(&mut svg, 100.0, 1000.0, 1500.0, 250.0, 7.5);
        let output = svg.to_svg();
        assert!(
            output.contains("stroke-width=\"7.5\""),
            "stroke-width should match the given value"
        );
    }

    #[test]
    fn extender_line_stroke_is_black() {
        let mut svg = test_svg();
        draw_lyric_extender(&mut svg, 100.0, 1000.0, 1500.0, 250.0, 5.0);
        let output = svg.to_svg();
        assert!(
            output.contains("stroke=\"black\""),
            "extender line should be black"
        );
    }
}
