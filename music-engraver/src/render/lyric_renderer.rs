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
/// (Extender lines between syllables require knowledge of the next note's
/// x-position and are not rendered by this function — they need a separate
/// pass after all syllables are positioned.)
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
}
