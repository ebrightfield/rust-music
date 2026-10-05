//! SVG rendering for tempo markings: each composed line of a
//! [`TempoMarkLayout`] is drawn with [`draw_text_line`].

use crate::font::{FontError, MusicFont};
use crate::layout::tempo::TempoMarkLayout;
use crate::render::text_line_renderer::draw_text_line;
use crate::render::SvgWriter;

/// Draw a tempo marking onto the SVG.
pub fn draw_tempo_mark(
    svg: &mut SvgWriter,
    layout: &TempoMarkLayout,
    font: &MusicFont,
) -> Result<(), FontError> {
    for line in &layout.lines {
        draw_text_line(svg, font, &line.line, layout.x_left, line.baseline_y)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::tempo::*;

    fn render(mark: &TempoMark) -> String {
        let font = bravura_font();
        let layout = layout_tempo_mark(mark, 100.0, -700.0, &font, 250.0).unwrap();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -1200.0, 6000.0, 2000.0);
        draw_tempo_mark(&mut svg, &layout, &font).unwrap();
        svg.to_svg()
    }

    #[test]
    fn words_are_bold_text_without_glyphs() {
        let out = render(&TempoMark::text("Allegro"));
        assert!(out.contains(">Allegro</text>"));
        assert!(out.contains(r#"font-weight="bold""#));
        assert!(!out.contains("<path"));
    }

    #[test]
    fn parenthesized_metronome_draws_paren_glyph_and_value_on_one_baseline() {
        let out = render(&TempoMark::metronome(
            MetronomeMark::bpm(MetronomeNoteKind::Quarter, 60)
                .approx()
                .parenthesized(),
        ));
        // "(" is end-anchored on the glyph; the value follows it.
        assert!(out.contains(r#"text-anchor="end""#), "{out}");
        assert!(out.contains(">(</text>"));
        assert!(out.contains(">= c. 60)</text>"));
        assert_eq!(out.matches("<path").count(), 1);
        assert!(out.contains("scale(0.7,0.7)"), "{out}");
        assert_eq!(out.matches(r#"y="-700""#).count(), 2, "{out}");
    }
}
