/// SVG rendering for lyric text syllables.
///
/// Renders roman (upright) text below the staff. Hyphens between adjacent
/// syllables and melisma extender lines are drawn separately in a second
/// pass, by [`draw_lyric_hyphen`] and [`draw_lyric_extender`] respectively,
/// once positions of all consecutive notes are known.
use crate::layout::lyric::LyricLayout;
use crate::render::svg_writer::TextStyle;
use crate::render::SvgWriter;

/// Draw a lyric syllable onto the SVG.
///
/// Renders roman serif text centered on the note's x-position, below the staff.
/// Hyphenated continuation (`LyricContinuation::Hyphen`) does NOT append a
/// trailing hyphen to the syllable — proper engraving places the hyphen
/// centered between the syllable and the next one. Use [`draw_lyric_hyphen`]
/// in a second pass when the next note's x-position is known.
pub fn draw_lyric(svg: &mut SvgWriter, layout: &LyricLayout) {
    svg.add_text(
        layout.x_center,
        layout.y_baseline,
        &layout.text,
        &TextStyle::normal(layout.font_size),
    );
}

/// Estimated half-width of a lyric syllable, in ems, used to push the
/// hyphen away from the syllable text edges. Conservative — the bundled
/// font is not used for text, so this is a layout estimate. A typical
/// 3-letter syllable occupies ~1.5 em; half of that is ~0.75. We use
/// 0.5 em so that even short syllables (1–2 letters) have some clearance.
const HYPHEN_SYLLABLE_HALF_WIDTH_EM: f64 = 0.5;

/// Minimum gap between source and target syllables (in staff spaces) below
/// which no hyphen is drawn. Prevents drawing a hyphen on top of overlapping
/// or near-overlapping syllables.
const HYPHEN_MIN_GAP_SS: f64 = 0.6;

/// Draw a hyphen between a syllable with `Hyphen` continuation and the next
/// syllable's note position.
///
/// The hyphen is centered horizontally between the two note centers, on the
/// shared lyric baseline. Visual convention (Gould, Gardner Read): a single
/// hyphen character `-` of the same font size as the surrounding lyrics,
/// centered in the gap between syllables — not appended to the source
/// syllable text.
///
/// `from_x` is the center-x of the source syllable's note.
/// `to_x` is the center-x of the target syllable's note.
/// `y_baseline` is the lyric text baseline (shared across all lyrics in a system).
/// `font_size` is the lyric font size in font design units.
/// `staff_space` is the staff space size in font design units; used for the
/// minimum-gap check.
///
/// Returns `true` if a hyphen was drawn, `false` if the gap was too small.
pub fn draw_lyric_hyphen(
    svg: &mut SvgWriter,
    from_x: f64,
    to_x: f64,
    y_baseline: f64,
    font_size: f64,
    staff_space: f64,
) -> bool {
    let half_width = HYPHEN_SYLLABLE_HALF_WIDTH_EM * font_size;
    let gap = to_x - from_x - 2.0 * half_width;
    if gap < HYPHEN_MIN_GAP_SS * staff_space {
        return false;
    }

    let midpoint = 0.5 * (from_x + to_x);
    svg.add_text(midpoint, y_baseline, "-", &TextStyle::normal(font_size));
    true
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
    fn hyphen_continuation_renders_only_syllable_text() {
        // draw_lyric must NOT append " -" to a hyphenated syllable.
        // The hyphen between syllables is drawn separately by
        // draw_lyric_hyphen() in a second pass.
        let syl = LyricSyllable::with_hyphen("hap");
        let layout = layout_lyric(&syl, 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_lyric(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains(">hap<"),
            "hyphen syllable should render only 'hap', got: {output}"
        );
        assert!(
            !output.contains("hap -"),
            "draw_lyric must not append ' -' to hyphenated syllable text; got: {output}"
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
            !output.contains("love -"),
            "extender should not have a hyphen"
        );
    }

    #[test]
    fn draw_lyric_hyphen_emits_centered_text_element() {
        let mut svg = test_svg();
        let drew = draw_lyric_hyphen(&mut svg, 200.0, 800.0, 1500.0, 100.0, 250.0);
        assert!(drew, "hyphen should be drawn for a 600fu gap");
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<text").count(),
            1,
            "hyphen should produce exactly 1 text element"
        );
        // midpoint = (200+800)/2 = 500
        assert!(
            output.contains(r#"x="500""#),
            "hyphen should be centered at midpoint x=500, got: {output}"
        );
        assert!(
            output.contains(r#"y="1500""#),
            "hyphen y should match baseline 1500, got: {output}"
        );
        assert!(
            output.contains(">-<"),
            "hyphen text content should be '-', got: {output}"
        );
        // Must use the lyric font size, not the surrounding default.
        assert!(
            output.contains(r#"font-size="100""#),
            "hyphen should use the given 100 font size, got: {output}"
        );
    }

    #[test]
    fn draw_lyric_hyphen_uses_text_anchor_middle() {
        let mut svg = test_svg();
        draw_lyric_hyphen(&mut svg, 100.0, 700.0, 1500.0, 80.0, 250.0);
        let output = svg.to_svg();
        assert!(
            output.contains(r#"text-anchor="middle""#),
            "hyphen text should be center-anchored, got: {output}"
        );
    }

    #[test]
    fn draw_lyric_hyphen_returns_false_when_gap_too_small() {
        let mut svg = test_svg();
        // Centers 10fu apart with 100fu font: half-widths consume 2 * 50 = 100fu,
        // leaving a -90fu gap — well below the 0.6 * 250 = 150fu minimum.
        let drew = draw_lyric_hyphen(&mut svg, 200.0, 210.0, 1500.0, 100.0, 250.0);
        assert!(!drew, "hyphen should not draw when syllables overlap");
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<text").count(),
            0,
            "no text element should be emitted when hyphen is skipped"
        );
    }

    #[test]
    fn draw_lyric_hyphen_skips_when_gap_below_threshold() {
        let mut svg = test_svg();
        // half_width = 0.5 * 80 = 40fu, two of them = 80fu.
        // min gap = 0.6 * 250 = 150fu.
        // Centers 220fu apart: gap = 220 - 80 = 140fu (below 150fu) → skip.
        let drew = draw_lyric_hyphen(&mut svg, 100.0, 320.0, 1500.0, 80.0, 250.0);
        assert!(!drew, "gap of 140 below 150 threshold should not draw");
        assert_eq!(svg.to_svg().matches("<text").count(), 0);
    }

    #[test]
    fn draw_lyric_hyphen_draws_when_gap_above_threshold() {
        let mut svg = test_svg();
        // Centers 250fu apart with 80fu font:
        // gap = 250 - (2 * 0.5 * 80) = 250 - 80 = 170fu > 0.6 * 250 = 150fu → draw.
        let drew = draw_lyric_hyphen(&mut svg, 100.0, 350.0, 1500.0, 80.0, 250.0);
        assert!(drew, "gap of 170 above 150 threshold should draw");
        let output = svg.to_svg();
        assert_eq!(output.matches("<text").count(), 1);
        // midpoint = (100+350)/2 = 225
        assert!(
            output.contains(r#"x="225""#),
            "hyphen midpoint x should be 225, got: {output}"
        );
    }

    #[test]
    fn draw_lyric_hyphen_position_is_independent_of_text() {
        // Two calls with the same coordinates produce the same hyphen — text
        // content of source/target syllables is not part of the input.
        let mut a = test_svg();
        let mut b = test_svg();
        draw_lyric_hyphen(&mut a, 300.0, 900.0, 1500.0, 100.0, 250.0);
        draw_lyric_hyphen(&mut b, 300.0, 900.0, 1500.0, 100.0, 250.0);
        assert_eq!(a.to_svg(), b.to_svg());
    }

    #[test]
    fn draw_lyric_hyphen_midpoint_moves_with_endpoints() {
        // Sanity: shifting both endpoints by +100 shifts the midpoint by +100.
        let mut left = test_svg();
        let mut right = test_svg();
        draw_lyric_hyphen(&mut left, 100.0, 700.0, 1500.0, 100.0, 250.0);
        draw_lyric_hyphen(&mut right, 200.0, 800.0, 1500.0, 100.0, 250.0);
        assert!(left.to_svg().contains(r#"x="400""#));
        assert!(right.to_svg().contains(r#"x="500""#));
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
