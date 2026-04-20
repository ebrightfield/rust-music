//! SVG rendering for chord symbol markings.
//!
//! Renders bold text above the staff using the geometry from
//! [`crate::layout::chord_symbol`].

use crate::layout::chord_symbol::ChordSymbolLayout;
use crate::render::svg_writer::TextStyle;
use crate::render::SvgWriter;

/// Draw a chord symbol onto the SVG.
///
/// Renders bold serif text centered on the note's x-position, above the staff.
/// Bold weight distinguishes chord symbols from other above-staff text.
pub fn draw_chord_symbol(svg: &mut SvgWriter, layout: &ChordSymbolLayout) {
    let style = TextStyle {
        anchor: "middle",
        ..TextStyle::bold(layout.font_size)
    };
    svg.add_text(layout.x_center, layout.y_baseline, &layout.text, &style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::chord_symbol::layout_chord_symbol;
    use crate::layout::staff::StaffLayout;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, 250.0)
    }

    fn test_svg() -> SvgWriter {
        SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0)
    }

    #[test]
    fn chord_symbol_renders_text_element() {
        let layout = layout_chord_symbol("Cmaj7", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_chord_symbol(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<text"), "should contain a text element");
        assert!(output.contains(">Cmaj7<"), "should contain 'Cmaj7'");
    }

    #[test]
    fn chord_symbol_is_bold() {
        let layout = layout_chord_symbol("Am", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_chord_symbol(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("bold"), "chord symbol should be bold");
    }

    #[test]
    fn chord_symbol_is_centered() {
        let layout = layout_chord_symbol("G7", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_chord_symbol(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            output.contains("middle"),
            "chord symbol should be centered (text-anchor: middle)"
        );
    }

    #[test]
    fn different_symbols_produce_different_output() {
        let layout_a = layout_chord_symbol("C", 500.0, &test_staff(), 250.0);
        let layout_b = layout_chord_symbol("Am7", 500.0, &test_staff(), 250.0);

        let mut svg_a = test_svg();
        draw_chord_symbol(&mut svg_a, &layout_a);

        let mut svg_b = test_svg();
        draw_chord_symbol(&mut svg_b, &layout_b);

        assert_ne!(svg_a.to_svg(), svg_b.to_svg());
    }

    #[test]
    fn different_positions_produce_different_output() {
        let layout_a = layout_chord_symbol("C", 200.0, &test_staff(), 250.0);
        let layout_b = layout_chord_symbol("C", 800.0, &test_staff(), 250.0);

        let mut svg_a = test_svg();
        draw_chord_symbol(&mut svg_a, &layout_a);

        let mut svg_b = test_svg();
        draw_chord_symbol(&mut svg_b, &layout_b);

        assert_ne!(svg_a.to_svg(), svg_b.to_svg());
    }

    #[test]
    fn chord_symbol_has_no_path() {
        let layout = layout_chord_symbol("Dm", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_chord_symbol(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<path").count(),
            0,
            "chord symbol should have no paths"
        );
    }

    #[test]
    fn text_count_is_one() {
        let layout = layout_chord_symbol("F#m7b5", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_chord_symbol(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<text").count(),
            1,
            "should produce exactly 1 text element"
        );
    }

    #[test]
    fn not_italic() {
        let layout = layout_chord_symbol("C", 500.0, &test_staff(), 250.0);
        let mut svg = test_svg();
        draw_chord_symbol(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(
            !output.contains("italic"),
            "chord symbol should not be italic (that's expression text)"
        );
    }
}
