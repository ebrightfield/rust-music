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
    ChordSymbolCompositeLayout, ChordSymbolLayout, ChordSymbolSegment, ChordSymbolSegmentBox,
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
            // Single source of truth for chord-symbol → SMuFL mapping lives on
            // `ChordSymbolSegment::glyph()` (layout/chord_symbol.rs). The
            // or-pattern keeps match exhaustiveness — adding a new accidental
            // variant requires updating this arm — while folding the three
            // formerly-duplicated arms (Sharp/Flat/Natural → matching SMuFL
            // glyph) into a single dispatch through `seg.glyph()`. The
            // `expect` is a layout-layer invariant: any segment that
            // `is_accidental()` must yield `Some(glyph)`. A regression that
            // violated it would surface immediately as a panic in tests
            // covering each accidental variant, not silently produce wrong
            // SVG output.
            seg @ (ChordSymbolSegment::Sharp
            | ChordSymbolSegment::Flat
            | ChordSymbolSegment::Natural) => {
                let glyph = seg
                    .glyph()
                    .expect("accidental segment must resolve to a SMuFL glyph");
                draw_accidental_glyph(svg, font, glyph, box_, upe)?;
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
    box_: &ChordSymbolSegmentBox,
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
        layout_chord_symbol_composite(text, 500.0, &test_staff(), 250.0, upe, |g| {
            font.glyph_advance(g).unwrap_or(0)
        })
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
        assert_eq!(
            output.matches("<text").count(),
            1,
            "expected 1 text element"
        );
        assert_eq!(
            output.matches("<path").count(),
            0,
            "expected 0 path elements"
        );
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
        assert_eq!(
            output.matches("<text").count(),
            1,
            "expected 1 text run ('B')"
        );
        assert_eq!(
            output.matches("<path").count(),
            1,
            "expected 1 flat glyph path"
        );
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
        assert!(
            output.contains("translate("),
            "expected translate transform"
        );
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
        // `first_path_d` is the single test-module helper defined below — the
        // earlier copy of this helper that lived as a nested fn here has been
        // consolidated into the module-level definition so that any future
        // change to "what counts as the first path's d attribute" lives in one
        // place. If the helper drifts, all four tests (this one plus the
        // three `composite_dispatches_through_segment_glyph_for_*`) catch it.
        let d_sharp = first_path_d(&svg_sharp);
        let d_flat = first_path_d(&svg_flat);
        let d_natural = first_path_d(&svg_natural);
        assert_ne!(
            d_sharp, d_flat,
            "sharp and flat glyphs should have different path data"
        );
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

    // ---------------- glyph()-as-source-of-truth dispatch canaries ----------------
    //
    // These four tests lock in the contract that `draw_chord_symbol_composite`
    // dispatches accidental segments through `ChordSymbolSegment::glyph()` —
    // not via a private per-variant map duplicated inside the renderer. If a
    // future refactor reintroduces a duplicate map and accidentally writes a
    // different glyph for one variant (e.g. routing `Sharp` to
    // `AccidentalDoubleSharp`), exactly one of these tests fires while the
    // others continue to pass — pinpointing the regression to a single
    // variant.

    /// Extract the path `d` attribute from the *first* path in the SVG.
    ///
    /// Shared by the dispatch canaries below AND by
    /// `composite_different_accidentals_produce_different_paths` above — the
    /// nested-fn copy that used to live in that test was consolidated here so
    /// the "find the first `d=""` then read to the next `"`" logic exists in
    /// exactly one place. Two helpers with the same body but different names
    /// is the classic DRY smell: a fix or generalization to one would not
    /// propagate to the other.
    ///
    /// Behavior: scans `svg.to_svg()` for the first occurrence of `d="`,
    /// reads forward until the closing `"`, and returns the inner string.
    /// Panics (`expect`) if the SVG has no path attribute — which is what
    /// callers want, since they only invoke this on SVGs that contain at
    /// least one rendered glyph path.
    fn first_path_d(svg: &SvgWriter) -> String {
        let s = svg.to_svg();
        let start = s.find(r#"d=""#).expect("should contain a path");
        let after = &s[start + 3..];
        let end = after.find('"').expect("path data should close");
        after[..end].to_string()
    }

    // ---------------- first_path_d helper direct tests ----------------
    //
    // The helper itself was previously only exercised indirectly via the
    // dispatch canaries (which compare it against `font.glyph_outline(...)`).
    // Direct tests below pin down the helper's contract independent of the
    // renderer — so if a future refactor changes the helper's parsing rule
    // (e.g. handling multi-line `d="..."` attributes, or escaping inside the
    // attribute value), these fire first and pinpoint the helper as the
    // source rather than a renderer change.

    #[test]
    fn first_path_d_returns_path_data_for_single_path() {
        // Sanity: an SVG with one rendered glyph path must yield the same
        // string as `font.glyph_outline(glyph).path_data`. The first path's
        // `d` attribute is precisely that outline.
        let font = test_font();
        let layout = composite("F#");
        let mut svg = test_svg();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();

        let extracted = first_path_d(&svg);
        let expected = font
            .glyph_outline(
                ChordSymbolSegment::Sharp
                    .glyph()
                    .expect("Sharp has a glyph"),
            )
            .unwrap()
            .path_data;
        assert_eq!(
            extracted, expected,
            "first_path_d must return the rendered glyph's outline path data"
        );
    }

    #[test]
    fn first_path_d_returns_first_path_when_multiple_paths_present() {
        // For `F#m7b5` the renderer emits two glyph paths (sharp first,
        // then flat). The helper must return the *first* — the sharp's
        // outline, not the flat's. If a future refactor accidentally
        // matched the last `d="` (e.g. `rfind` instead of `find`), the
        // returned data would equal the flat's outline; the assert_ne
        // pins down which is which.
        let font = test_font();
        let layout = composite("F#m7b5");
        let mut svg = test_svg();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();

        let extracted = first_path_d(&svg);
        let sharp_outline = font
            .glyph_outline(ChordSymbolSegment::Sharp.glyph().unwrap())
            .unwrap()
            .path_data;
        let flat_outline = font
            .glyph_outline(ChordSymbolSegment::Flat.glyph().unwrap())
            .unwrap()
            .path_data;
        assert_eq!(
            extracted, sharp_outline,
            "first_path_d must return the sharp (first) glyph's path data"
        );
        assert_ne!(
            extracted, flat_outline,
            "first_path_d must NOT return the flat (later) glyph's path data"
        );
    }

    #[test]
    fn first_path_d_is_deterministic_across_repeated_calls() {
        // Two independent calls with byte-identical SVG input must return
        // byte-identical strings — locks in that the helper holds no
        // hidden state and walks the string deterministically.
        let font = test_font();
        let layout = composite("Bb");
        let mut svg1 = test_svg();
        let mut svg2 = test_svg();
        draw_chord_symbol_composite(&mut svg1, &font, &layout).unwrap();
        draw_chord_symbol_composite(&mut svg2, &font, &layout).unwrap();

        assert_eq!(
            first_path_d(&svg1),
            first_path_d(&svg2),
            "first_path_d must be deterministic"
        );
    }

    #[test]
    fn first_path_d_returned_string_is_non_empty() {
        // Defensive: a rendered glyph's path data is always a non-empty
        // sequence of SVG path commands. If a regression ever produced an
        // empty path (`d=""`), the helper would still parse it and return
        // `""` — this test fires first to pinpoint the regression to the
        // renderer rather than burying it in a downstream byte-comparison
        // that just says "two empty strings are equal."
        let font = test_font();
        let layout = composite("F\u{266E}");
        let mut svg = test_svg();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();

        let extracted = first_path_d(&svg);
        assert!(
            !extracted.is_empty(),
            "first_path_d returned an empty string for a rendered natural glyph"
        );
        // A real SVG path-data string starts with a command letter. The
        // SMuFL natural outline begins with `M` (move-to), the standard
        // starting command for any glyph outline.
        assert!(
            extracted.starts_with('M'),
            "first_path_d returned data that doesn't start with a move command: {extracted:?}"
        );
    }

    #[test]
    fn composite_dispatches_through_segment_glyph_for_sharp() {
        // `C#` → Text("C") + Sharp. The single emitted path must match the
        // font's outline for the glyph that `ChordSymbolSegment::Sharp.glyph()`
        // returns — currently `AccidentalSharp`. If `glyph()` is changed to
        // return a different SMuFL glyph (e.g. a small/raised variant for
        // chord-symbol contexts), the renderer must follow without code
        // changes here.
        let font = test_font();
        let layout = composite("C#");
        let mut svg = test_svg();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let rendered_d = first_path_d(&svg);

        let expected_glyph = ChordSymbolSegment::Sharp
            .glyph()
            .expect("Sharp must have a glyph");
        let expected_outline = font.glyph_outline(expected_glyph).unwrap();
        assert_eq!(
            rendered_d, expected_outline.path_data,
            "Sharp segment's rendered path must equal font outline for ChordSymbolSegment::Sharp.glyph()",
        );
    }

    #[test]
    fn composite_dispatches_through_segment_glyph_for_flat() {
        // `Cb` → Text("C") + Flat (`b` after uppercase root letter `C` is
        // a flat by chord-symbol rules). Rendered path must match the outline
        // for `ChordSymbolSegment::Flat.glyph()`.
        let font = test_font();
        let layout = composite("Cb");
        let mut svg = test_svg();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let rendered_d = first_path_d(&svg);

        let expected_glyph = ChordSymbolSegment::Flat
            .glyph()
            .expect("Flat must have a glyph");
        let expected_outline = font.glyph_outline(expected_glyph).unwrap();
        assert_eq!(
            rendered_d, expected_outline.path_data,
            "Flat segment's rendered path must equal font outline for ChordSymbolSegment::Flat.glyph()",
        );
    }

    #[test]
    fn composite_dispatches_through_segment_glyph_for_natural() {
        // `C♮` (U+266E) → Text("C") + Natural. Rendered path must match the
        // outline for `ChordSymbolSegment::Natural.glyph()`.
        let font = test_font();
        let layout = composite("C\u{266E}");
        let mut svg = test_svg();
        draw_chord_symbol_composite(&mut svg, &font, &layout).unwrap();
        let rendered_d = first_path_d(&svg);

        let expected_glyph = ChordSymbolSegment::Natural
            .glyph()
            .expect("Natural must have a glyph");
        let expected_outline = font.glyph_outline(expected_glyph).unwrap();
        assert_eq!(
            rendered_d, expected_outline.path_data,
            "Natural segment's rendered path must equal font outline for ChordSymbolSegment::Natural.glyph()",
        );
    }

    #[test]
    fn composite_dispatch_is_deterministic_across_repeated_calls() {
        // Determinism canary for the new `glyph()`-driven dispatch path.
        // Two independent calls with identical inputs must produce
        // byte-identical SVG output. If the dispatch ever pulled in
        // any non-deterministic state (env, time, RNG, hash iteration
        // order), this fires.
        let font = test_font();

        let layout1 = composite("F#m7b5");
        let layout2 = composite("F#m7b5");

        let mut svg1 = test_svg();
        let mut svg2 = test_svg();
        draw_chord_symbol_composite(&mut svg1, &font, &layout1).unwrap();
        draw_chord_symbol_composite(&mut svg2, &font, &layout2).unwrap();

        assert_eq!(
            svg1.to_svg(),
            svg2.to_svg(),
            "two identical-input calls must produce byte-identical SVG"
        );
    }
}
