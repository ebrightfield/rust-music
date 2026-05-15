//! SVG rendering for chord symbol markings.
//!
//! Two render paths are provided:
//!
//! - [`draw_chord_symbol`] — single bold `<text>` element. The original v1
//!   path; renders `#`/`b` as plain ASCII characters. Stable, font-agnostic.
//! - [`draw_chord_symbol_composite`] — text + SMuFL accidental glyph
//!   composition. Walks a [`ChordSymbolCompositeLayout`] emitting `<text>` for
//!   text runs and `<path>` for accidental glyphs. The renderer used by the
//!   measure renderer.

use smufl::Glyph;

use crate::font::{FontError, MusicFont};
use crate::layout::chord_symbol::{
    ChordSymbolCompositeLayout, ChordSymbolLayout, ChordSymbolSegment,
};
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

/// Draw a chord symbol with SMuFL accidental glyph composition.
///
/// Walks the composite layout's segment boxes:
///
/// - [`ChordSymbolSegment::Text`] runs are emitted as a left-anchored bold
///   `<text>` element at the segment's `x_left` baseline.
/// - [`ChordSymbolSegment::Sharp`] / [`ChordSymbolSegment::Flat`] /
///   [`ChordSymbolSegment::Natural`] resolve to SMuFL `accidentalSharp` /
///   `accidentalFlat` / `accidentalNatural` glyph paths, transformed to the
///   segment's position and scaled to its (reduced) font size.
///
/// Returns `Ok(())` after rendering. Returns `Err` only if a required SMuFL
/// glyph is missing from the font (which would be a font-bundling regression).
pub fn draw_chord_symbol_composite(
    svg: &mut SvgWriter,
    font: &MusicFont,
    layout: &ChordSymbolCompositeLayout,
) -> Result<(), FontError> {
    let upe = font.units_per_em().max(1) as f64;
    for box_ in &layout.boxes {
        match &box_.segment {
            ChordSymbolSegment::Text(s) => {
                // Left-anchored: `x_left` is the literal left edge of the run.
                let style = TextStyle {
                    anchor: "start",
                    ..TextStyle::bold(box_.font_size)
                };
                svg.add_text(box_.x_left, box_.y_baseline, s, &style);
            }
            ChordSymbolSegment::Sharp => {
                draw_accidental_glyph(svg, font, Glyph::AccidentalSharp, box_, upe)?
            }
            ChordSymbolSegment::Flat => {
                draw_accidental_glyph(svg, font, Glyph::AccidentalFlat, box_, upe)?
            }
            ChordSymbolSegment::Natural => {
                draw_accidental_glyph(svg, font, Glyph::AccidentalNatural, box_, upe)?
            }
        }
    }
    Ok(())
}

/// Emit one accidental glyph at the segment's box.
///
/// The glyph outline is in the music font's design-unit space (units_per_em
/// units tall). To render it at the segment's reduced font size, we apply a
/// uniform scale of `font_size / units_per_em` after translating to the
/// segment's baseline-left position.
fn draw_accidental_glyph(
    svg: &mut SvgWriter,
    font: &MusicFont,
    glyph: Glyph,
    box_: &crate::layout::chord_symbol::ChordSymbolSegmentBox,
    upe: f64,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(glyph)?;
    let scale = box_.font_size / upe;
    let transform = format!(
        "translate({},{}) scale({})",
        box_.x_left, box_.y_baseline, scale
    );
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
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

    // ---------------- draw_chord_symbol_composite ----------------

    use crate::font::bravura_font;
    use crate::layout::chord_symbol::layout_chord_symbol_composite;

    fn test_font() -> crate::font::MusicFont<'static> {
        bravura_font()
    }

    fn composite(text: &str) -> ChordSymbolCompositeLayout {
        let font = test_font();
        let upe = font.units_per_em();
        layout_chord_symbol_composite(
            text,
            500.0,
            &test_staff(),
            250.0,
            upe,
            |g| font.glyph_advance(g).unwrap_or(0),
        )
    }

    #[test]
    fn composite_plain_text_emits_one_text_element_no_paths() {
        // Byte-equivalence guard: a chord symbol with no accidentals must
        // still produce exactly one `<text>` element and zero `<path>`
        // elements, matching the non-composite renderer's element counts.
        let layout = composite("Cmaj7");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<text").count(), 1, "expected 1 text element");
        assert_eq!(output.matches("<path").count(), 0, "expected 0 path elements");
        assert!(output.contains(">Cmaj7<"), "text content should be 'Cmaj7'");
    }

    #[test]
    fn composite_f_sharp_emits_text_plus_path() {
        let layout = composite("F#");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<text").count(),
            1,
            "F# should produce exactly 1 text element (the 'F')"
        );
        assert_eq!(
            output.matches("<path").count(),
            1,
            "F# should produce exactly 1 path element (the sharp glyph)"
        );
        assert!(output.contains(">F<"), "expected the 'F' text fragment");
        // The '#' must NOT appear as raw text content — it's been promoted to a glyph.
        assert!(
            !output.contains(">#<") && !output.contains(">F#<"),
            "'#' should not appear as text content; output was {output}"
        );
    }

    #[test]
    fn composite_b_flat_emits_text_plus_path() {
        let layout = composite("Bb");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<text").count(), 1, "expected 1 text run ('B')");
        assert_eq!(output.matches("<path").count(), 1, "expected 1 flat glyph path");
        assert!(output.contains(">B<"));
        assert!(
            !output.contains(">Bb<") && !output.contains(">b<"),
            "'b' should be a glyph, not text"
        );
    }

    #[test]
    fn composite_complex_symbol_emits_expected_segment_counts() {
        // F#m7b5 — 5 segments: F, #, m7, b, 5 → 3 text elements + 2 paths.
        let layout = composite("F#m7b5");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<text").count(),
            3,
            "F#m7b5 should produce 3 text elements (F, m7, 5)"
        );
        assert_eq!(
            output.matches("<path").count(),
            2,
            "F#m7b5 should produce 2 accidental paths (# and b)"
        );
        assert!(output.contains(">F<"));
        assert!(output.contains(">m7<"));
        assert!(output.contains(">5<"));
    }

    #[test]
    fn composite_empty_string_emits_nothing() {
        let layout = composite("");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<text").count(), 0);
        assert_eq!(output.matches("<path").count(), 0);
    }

    #[test]
    fn composite_segments_use_anchor_start_not_middle() {
        // Text runs in the composite layout are positioned by left edge, so
        // they must use `text-anchor="start"`. Sanity guard against a
        // regression that copies the simple renderer's "middle" anchor.
        let layout = composite("F#");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(
            output.contains(r#"text-anchor="start""#),
            "composite text should use text-anchor=\"start\", got: {output}"
        );
    }

    #[test]
    fn composite_paths_have_translate_and_scale_transforms() {
        // The accidental glyph path must be positioned (translate) AND scaled.
        let layout = composite("F#");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("translate("), "expected translate transform");
        assert!(output.contains(" scale("), "expected scale transform");
    }

    #[test]
    fn composite_different_accidentals_produce_different_paths() {
        // Defensive: ensure the renderer is actually picking up the glyph
        // from the segment kind, not always emitting the same glyph.
        let font = test_font();
        let layout_sharp = composite("F#");
        let layout_flat = composite("Bb");
        let layout_natural = composite("F\u{266E}");
        let mut svg_sharp = test_svg();
        let mut svg_flat = test_svg();
        let mut svg_natural = test_svg();
        draw_chord_symbol_composite(&mut svg_sharp, &font, &layout_sharp).unwrap();
        draw_chord_symbol_composite(&mut svg_flat, &font, &layout_flat).unwrap();
        draw_chord_symbol_composite(&mut svg_natural, &font, &layout_natural).unwrap();
        // Extract just the path data (everything after the first `d="`).
        fn path_d(svg: &SvgWriter) -> String {
            let s = svg.to_svg();
            let start = s.find(r#"d=""#).expect("should contain a path");
            let after = &s[start + 3..];
            let end = after.find('"').expect("path data should close");
            after[..end].to_string()
        }
        let d_sharp = path_d(&svg_sharp);
        let d_flat = path_d(&svg_flat);
        let d_natural = path_d(&svg_natural);
        assert_ne!(d_sharp, d_flat, "sharp and flat glyphs should have different path data");
        assert_ne!(d_sharp, d_natural, "sharp and natural glyphs should differ");
        assert_ne!(d_flat, d_natural, "flat and natural glyphs should differ");
    }

    #[test]
    fn composite_emits_bold_text_for_text_runs() {
        let layout = composite("F#m7b5");
        let mut svg = test_svg();
        let font = test_font();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let output = svg.to_svg();
        // Each `<text>` must carry font-weight="bold" — chord symbols are bold,
        // and the composite path must preserve that on every text run.
        let bold_count = output.matches(r#"font-weight="bold""#).count();
        assert_eq!(bold_count, 3, "all 3 text runs in F#m7b5 should be bold");
    }
}
