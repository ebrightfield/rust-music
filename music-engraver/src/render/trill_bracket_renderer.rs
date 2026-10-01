//! Render the vertical hook markers ("brackets") at the start and/or end
//! of a trill wavy-line extension.
//!
//! Each hook is emitted as a single SVG `<line>`. The renderer is
//! intentionally font-free: the layout step has already resolved all
//! geometry to pixel/font-unit coordinates.

use crate::layout::trill_bracket::TrillBracketHookLayout;
use crate::render::SvgWriter;

/// Render a single bracket hook as an SVG `<line>`.
///
/// Uses the hook's own `stroke_width`; the bracket convention is for the
/// hook to match the wiggle's visual weight, which the caller controls by
/// passing a thin barline / hairpin thickness at layout time.
pub fn draw_trill_bracket_hook(svg: &mut SvgWriter, hook: &TrillBracketHookLayout) {
    svg.add_line(
        hook.x,
        hook.y_top,
        hook.x,
        hook.y_bottom,
        "black",
        hook.stroke_width,
    );
}

/// Render zero or more bracket hooks. Empty slices are a no-op.
pub fn draw_trill_bracket_hooks(svg: &mut SvgWriter, hooks: &[TrillBracketHookLayout]) {
    for hook in hooks {
        draw_trill_bracket_hook(svg, hook);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::trill_bracket::{
        layout_trill_bracket_hook, layout_trill_bracket_hooks, HookDirection, TrillBracketSide,
    };
    use crate::layout::trill_extension::layout_trill_extension;

    fn test_writer() -> SvgWriter {
        SvgWriter::new(400.0, 100.0, 0.0, 0.0, 2000.0, 500.0)
    }

    #[test]
    fn draw_single_hook_emits_one_line_element() {
        let hook = layout_trill_bracket_hook(100.0, 50.0, 30.0, HookDirection::Down, 4.0);
        let mut w = test_writer();
        draw_trill_bracket_hook(&mut w, &hook);
        let svg = w.to_svg();
        assert_eq!(svg.matches("<line ").count(), 1);
    }

    #[test]
    fn draw_single_hook_embeds_x_y_endpoints() {
        let hook = layout_trill_bracket_hook(123.0, 50.0, 30.0, HookDirection::Down, 4.0);
        let mut w = test_writer();
        draw_trill_bracket_hook(&mut w, &hook);
        let svg = w.to_svg();
        assert!(svg.contains(r#"x1="123""#), "missing x1: {svg}");
        assert!(svg.contains(r#"x2="123""#), "missing x2: {svg}");
        assert!(svg.contains(r#"y1="50""#), "missing y1: {svg}");
        assert!(svg.contains(r#"y2="80""#), "missing y2: {svg}");
    }

    #[test]
    fn draw_single_hook_embeds_stroke_width() {
        let hook = layout_trill_bracket_hook(0.0, 0.0, 30.0, HookDirection::Down, 7.5);
        let mut w = test_writer();
        draw_trill_bracket_hook(&mut w, &hook);
        let svg = w.to_svg();
        assert!(
            svg.contains(r#"stroke-width="7.5""#),
            "missing stroke-width: {svg}"
        );
    }

    #[test]
    fn draw_single_hook_stroke_is_black() {
        let hook = layout_trill_bracket_hook(0.0, 0.0, 30.0, HookDirection::Down, 4.0);
        let mut w = test_writer();
        draw_trill_bracket_hook(&mut w, &hook);
        let svg = w.to_svg();
        assert!(svg.contains(r#"stroke="black""#), "missing stroke: {svg}");
    }

    #[test]
    fn draw_up_hook_uses_correct_y_endpoints() {
        let hook = layout_trill_bracket_hook(50.0, 100.0, 30.0, HookDirection::Up, 4.0);
        // Up hook: y_top = 70, y_bottom = 100
        let mut w = test_writer();
        draw_trill_bracket_hook(&mut w, &hook);
        let svg = w.to_svg();
        assert!(svg.contains(r#"y1="70""#), "y1 should be 70: {svg}");
        assert!(svg.contains(r#"y2="100""#), "y2 should be 100: {svg}");
    }

    #[test]
    fn draw_hooks_empty_slice_is_noop() {
        let mut w = test_writer();
        draw_trill_bracket_hooks(&mut w, &[]);
        let svg = w.to_svg();
        assert_eq!(svg.matches("<line ").count(), 0);
    }

    #[test]
    fn draw_hooks_emits_one_line_per_hook() {
        let ext = layout_trill_extension(100.0, 100.0 + 3.0 * 80.0, 50.0, 80.0).unwrap();
        let hooks = layout_trill_bracket_hooks(
            &ext,
            TrillBracketSide::Both,
            30.0,
            HookDirection::Down,
            4.0,
        );
        assert_eq!(hooks.len(), 2);
        let mut w = test_writer();
        draw_trill_bracket_hooks(&mut w, &hooks);
        let svg = w.to_svg();
        assert_eq!(svg.matches("<line ").count(), 2);
    }

    #[test]
    fn draw_hooks_both_sides_embeds_distinct_x_values() {
        let ext = layout_trill_extension(100.0, 100.0 + 3.0 * 80.0, 50.0, 80.0).unwrap();
        let hooks = layout_trill_bracket_hooks(
            &ext,
            TrillBracketSide::Both,
            30.0,
            HookDirection::Down,
            4.0,
        );
        let mut w = test_writer();
        draw_trill_bracket_hooks(&mut w, &hooks);
        let svg = w.to_svg();
        // Start hook at x=100, end hook at x=340
        assert!(svg.contains(r#"x1="100""#), "missing start x: {svg}");
        assert!(svg.contains(r#"x1="340""#), "missing end x: {svg}");
    }

    #[test]
    fn draw_hooks_uses_per_hook_stroke_width() {
        // If two hooks have different stroke widths, both should appear in the output.
        let h1 = layout_trill_bracket_hook(10.0, 0.0, 30.0, HookDirection::Down, 2.0);
        let h2 = layout_trill_bracket_hook(20.0, 0.0, 30.0, HookDirection::Down, 9.0);
        let mut w = test_writer();
        draw_trill_bracket_hooks(&mut w, &[h1, h2]);
        let svg = w.to_svg();
        assert!(svg.contains(r#"stroke-width="2""#), "missing sw=2: {svg}");
        assert!(svg.contains(r#"stroke-width="9""#), "missing sw=9: {svg}");
    }

    #[test]
    fn draw_zero_length_hook_emits_degenerate_line() {
        // Caller responsibility to filter zero-length hooks; the renderer
        // emits exactly what the layout describes, including a degenerate
        // y1==y2 line.
        let hook = layout_trill_bracket_hook(50.0, 100.0, 0.0, HookDirection::Down, 4.0);
        let mut w = test_writer();
        draw_trill_bracket_hook(&mut w, &hook);
        let svg = w.to_svg();
        assert_eq!(svg.matches("<line ").count(), 1);
        assert!(svg.contains(r#"y1="100""#));
        assert!(svg.contains(r#"y2="100""#));
    }
}
