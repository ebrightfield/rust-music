/// SVG rendering for tempo markings.
///
/// Renders tempo text (bold) and metronome note symbols (SMuFL glyphs)
/// above the staff using the geometry from [`crate::layout::tempo`].
use crate::font::MusicFont;
use crate::layout::tempo::TempoMarkLayout;
use crate::render::svg_writer::TextStyle;
use crate::render::SvgWriter;

/// Draw a tempo marking onto the SVG.
///
/// Renders bold text for the tempo name, and a SMuFL note glyph + "= BPM"
/// text for metronome marks.
pub fn draw_tempo_mark(svg: &mut SvgWriter, layout: &TempoMarkLayout, font: &MusicFont) {
    let bold = TextStyle::bold(layout.font_size);

    // Draw text portion (if any)
    if !layout.text.is_empty() {
        svg.add_text(layout.x_left, layout.y_baseline, &layout.text, &bold);
    }

    // Draw metronome portion (if any)
    if let Some(ref metro) = layout.metronome {
        // Draw the note symbol glyph
        if let Ok(outline) = font.glyph_outline(metro.notehead_glyph) {
            let transform = format!("translate({}, {})", metro.note_x, layout.y_baseline);
            svg.add_path(&outline.path_data, "black", Some(&transform));
        }

        // Draw "= BPM" text
        svg.add_text(metro.eq_text_x, layout.y_baseline, &metro.eq_text, &bold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::staff::StaffLayout;
    use crate::layout::tempo::*;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    fn test_svg() -> SvgWriter {
        SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn text_only_renders_text_element() {
        let font = bravura_font();
        let mut svg = test_svg();
        let layout = layout_tempo_mark(
            &TempoMark::Text("Allegro".into()),
            100.0,
            &test_staff(),
            250.0,
        );
        draw_tempo_mark(&mut svg, &layout, &font);
        let output = svg.to_svg();
        assert!(output.contains("<text"), "should contain a text element");
        assert!(output.contains(">Allegro<"), "should contain 'Allegro'");
        assert!(output.contains("bold"), "should be bold");
        assert!(!output.contains("<path"), "text-only should have no path");
    }

    #[test]
    fn metronome_only_renders_glyph_and_eq_text() {
        let font = bravura_font();
        let mut svg = test_svg();
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            100.0,
            &test_staff(),
            250.0,
        );
        draw_tempo_mark(&mut svg, &layout, &font);
        let output = svg.to_svg();
        // Should have a path for the note glyph
        assert!(
            output.contains("<path"),
            "should contain a path for note glyph"
        );
        // Should have text for "= 120"
        assert!(output.contains("= 120"), "should contain '= 120'");
    }

    #[test]
    fn combined_renders_both_text_and_glyph() {
        let font = bravura_font();
        let mut svg = test_svg();
        let layout = layout_tempo_mark(
            &TempoMark::TextWithMetronome {
                text: "Allegro".into(),
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 132,
            },
            100.0,
            &test_staff(),
            250.0,
        );
        draw_tempo_mark(&mut svg, &layout, &font);
        let output = svg.to_svg();
        assert!(output.contains(">Allegro<"), "should contain text");
        assert!(output.contains("<path"), "should contain note glyph path");
        assert!(output.contains("= 132"), "should contain BPM");
    }

    #[test]
    fn different_bpm_produces_different_output() {
        let font = bravura_font();
        let staff = test_staff();

        let mut svg1 = test_svg();
        let l1 = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 60,
            },
            0.0,
            &staff,
            250.0,
        );
        draw_tempo_mark(&mut svg1, &l1, &font);

        let mut svg2 = test_svg();
        let l2 = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            0.0,
            &staff,
            250.0,
        );
        draw_tempo_mark(&mut svg2, &l2, &font);

        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn different_note_kinds_produce_different_glyphs() {
        let font = bravura_font();
        let staff = test_staff();

        let mut svg_q = test_svg();
        let lq = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            0.0,
            &staff,
            250.0,
        );
        draw_tempo_mark(&mut svg_q, &lq, &font);

        let mut svg_h = test_svg();
        let lh = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Half,
                dotted: false,
                bpm: 120,
            },
            0.0,
            &staff,
            250.0,
        );
        draw_tempo_mark(&mut svg_h, &lh, &font);

        assert_ne!(svg_q.to_svg(), svg_h.to_svg());
    }

    #[test]
    fn text_only_tempo_has_no_path() {
        let font = bravura_font();
        let mut svg = test_svg();
        let layout =
            layout_tempo_mark(&TempoMark::Text("Adagio".into()), 0.0, &test_staff(), 250.0);
        draw_tempo_mark(&mut svg, &layout, &font);
        let output = svg.to_svg();
        let path_count = output.matches("<path").count();
        assert_eq!(path_count, 0, "text-only tempo should have 0 paths");
    }

    #[test]
    fn metronome_text_count() {
        let font = bravura_font();
        let mut svg = test_svg();
        let layout = layout_tempo_mark(
            &TempoMark::Metronome {
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 120,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        draw_tempo_mark(&mut svg, &layout, &font);
        let output = svg.to_svg();
        // Metronome-only: 1 text element (for "= 120"), no text for empty string
        let text_count = output.matches("<text").count();
        assert_eq!(text_count, 1, "metronome-only should have 1 text element");
    }

    #[test]
    fn combined_text_count() {
        let font = bravura_font();
        let mut svg = test_svg();
        let layout = layout_tempo_mark(
            &TempoMark::TextWithMetronome {
                text: "Vivace".into(),
                note_kind: MetronomeNoteKind::Quarter,
                dotted: false,
                bpm: 160,
            },
            0.0,
            &test_staff(),
            250.0,
        );
        draw_tempo_mark(&mut svg, &layout, &font);
        let output = svg.to_svg();
        // Combined: 2 text elements (tempo name + "= BPM")
        let text_count = output.matches("<text").count();
        assert_eq!(text_count, 2, "combined should have 2 text elements");
    }
}
