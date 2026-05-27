/// SVG rendering for dashed-text crescendo / diminuendo markings
/// ("cresc. - - -", "decresc. - - -", "dim. - - -").
///
/// Consumes [`CrescTextLayout`] from [`crate::layout::cresc_text`] and
/// emits:
///   1. an italic serif text element for the label, anchored at the left
///      edge of the marking (text-anchor: start), and
///   2. a dashed continuation line from `x_line_start` to `x_end` at the
///      label baseline.
///
/// The dashed line is suppressed entirely if `x_end <= x_line_start`
/// (the caller has placed the marking so close together that there is
/// no room for a continuation line — the label alone communicates the
/// directive).
use crate::layout::cresc_text::CrescTextLayout;
use crate::render::{SvgWriter, TextStyle};

/// Draw a dashed-text crescendo marking onto the SVG.
///
/// Emits the italic text label *only* when `layout.has_label == true`
/// — cross-system continuation segments suppress the label so the
/// dashed line reads continuously across the system boundary.
pub fn draw_cresc_text(svg: &mut SvgWriter, layout: &CrescTextLayout) {
    if layout.has_label {
        // Italic, non-bold label, left-anchored — Gould convention. The
        // ottava renderer uses bold-italic ("8va") but cresc./dim. is
        // traditionally italic-only, matching dynamics and expression
        // markings in the same band.
        svg.add_text(
            layout.label_x,
            layout.label_y,
            &layout.label,
            &TextStyle {
                font_family: "serif",
                font_size: layout.font_size,
                fill: "black",
                anchor: "start",
                font_weight: "normal",
                font_style: "italic",
                dominant_baseline: "auto",
            },
        );
    }

    // Dashed continuation line. Omit if there's no room.
    if layout.x_line_start < layout.x_end {
        let dash_spec = format!("{},{}", layout.dash_length, layout.dash_gap);
        svg.add_dashed_line(
            layout.x_line_start,
            layout.y_baseline,
            layout.x_end,
            layout.y_baseline,
            "black",
            layout.line_thickness,
            &dash_spec,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::cresc_text::{
        layout_cresc_text, layout_cresc_text_continuation, CrescTextKind, CRESC_TEXT_DASH_GAP_SS,
        CRESC_TEXT_DASH_LENGTH_SS,
    };
    use crate::layout::staff::StaffLayout;

    const SS: f64 = 250.0;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 4000.0, SS)
    }

    fn test_svg() -> SvgWriter {
        SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0)
    }

    fn render(kind: CrescTextKind, x_start: f64, x_end: f64) -> String {
        let layout = layout_cresc_text(kind, x_start, x_end, &test_staff(), SS);
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        svg.to_svg()
    }

    // ---- text element / label content ------------------------------------

    #[test]
    fn renders_one_text_element() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1000.0);
        assert_eq!(
            out.matches("<text").count(),
            1,
            "should emit exactly one <text> element"
        );
    }

    #[test]
    fn cresc_label_text_content() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1000.0);
        assert!(
            out.contains(">cresc.</text>"),
            "should contain literal 'cresc.' as text content; got:\n{out}"
        );
    }

    #[test]
    fn decresc_label_text_content() {
        let out = render(CrescTextKind::Decrescendo, 100.0, 1000.0);
        assert!(
            out.contains(">decresc.</text>"),
            "should contain literal 'decresc.' as text content"
        );
    }

    #[test]
    fn dim_label_text_content() {
        let out = render(CrescTextKind::Diminuendo, 100.0, 1000.0);
        assert!(
            out.contains(">dim.</text>"),
            "should contain literal 'dim.' as text content"
        );
    }

    // ---- text style ------------------------------------------------------

    #[test]
    fn label_is_italic() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1000.0);
        assert!(
            out.contains("font-style=\"italic\""),
            "label must be italic"
        );
    }

    #[test]
    fn label_is_not_bold() {
        // Distinguishes this from ottava (which IS bold-italic). cresc.
        // text is italic only.
        let out = render(CrescTextKind::Crescendo, 100.0, 1000.0);
        assert!(
            out.contains("font-weight=\"normal\""),
            "label must have explicit font-weight=normal (italic-only, not bold); got:\n{out}"
        );
        assert!(
            !out.contains("font-weight=\"bold\""),
            "label must NOT be bold"
        );
    }

    #[test]
    fn label_is_left_anchored() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1000.0);
        assert!(
            out.contains("text-anchor=\"start\""),
            "label must be left-anchored so x_start aligns with the first note; got:\n{out}"
        );
    }

    // ---- dashed line -----------------------------------------------------

    #[test]
    fn emits_dashed_continuation_line() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert!(
            out.contains("stroke-dasharray"),
            "should emit a dashed continuation line"
        );
    }

    #[test]
    fn dashed_line_count_is_exactly_one() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1500.0);
        let count = out.matches("stroke-dasharray").count();
        assert_eq!(count, 1, "should emit exactly 1 dashed line (got {count})");
    }

    #[test]
    fn dasharray_value_matches_layout_constants() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1500.0);
        let expected = format!(
            "stroke-dasharray=\"{},{}\"",
            CRESC_TEXT_DASH_LENGTH_SS * SS,
            CRESC_TEXT_DASH_GAP_SS * SS
        );
        assert!(
            out.contains(&expected),
            "expected to find {expected:?} in SVG; got:\n{out}"
        );
    }

    #[test]
    fn no_dashed_line_when_x_end_equals_x_start() {
        // Degenerate: no room for a line — label only.
        let out = render(CrescTextKind::Crescendo, 500.0, 500.0);
        assert!(
            !out.contains("stroke-dasharray"),
            "should NOT emit a dashed line when there's no room"
        );
        // But the label IS still emitted.
        assert_eq!(
            out.matches("<text").count(),
            1,
            "label must still render even when the line is suppressed"
        );
    }

    #[test]
    fn no_dashed_line_when_x_end_inside_label_region() {
        // x_end is past x_start but still before x_line_start (label
        // region). The line should be suppressed.
        let layout = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 110.0, &test_staff(), SS);
        // sanity: precondition for this test is that 110 < x_line_start
        assert!(
            layout.x_end < layout.x_line_start,
            "test precondition: x_end should fall inside the label region"
        );
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        let out = svg.to_svg();
        assert!(
            !out.contains("stroke-dasharray"),
            "should NOT emit a dashed line when x_end < x_line_start"
        );
    }

    // ---- coordinate routing ---------------------------------------------

    #[test]
    fn dashed_line_starts_at_x_line_start() {
        let layout = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 2000.0, &test_staff(), SS);
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        let out = svg.to_svg();
        let needle = format!("x1=\"{}\"", layout.x_line_start);
        assert!(
            out.contains(&needle),
            "expected x1={} in dashed line; got:\n{out}",
            layout.x_line_start
        );
    }

    #[test]
    fn dashed_line_ends_at_x_end() {
        let layout = layout_cresc_text(CrescTextKind::Crescendo, 100.0, 2000.0, &test_staff(), SS);
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        let out = svg.to_svg();
        let needle = format!("x2=\"{}\"", layout.x_end);
        assert!(
            out.contains(&needle),
            "expected x2={} in dashed line; got:\n{out}",
            layout.x_end
        );
    }

    #[test]
    fn label_text_x_matches_layout_label_x() {
        let layout = layout_cresc_text(CrescTextKind::Crescendo, 175.5, 2000.0, &test_staff(), SS);
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        let out = svg.to_svg();
        let needle = format!("x=\"{}\"", layout.label_x);
        assert!(
            out.contains(&needle),
            "expected label x={} in text element; got:\n{out}",
            layout.label_x
        );
    }

    // ---- visual distinctness --------------------------------------------

    #[test]
    fn different_kinds_produce_different_svg() {
        let a = render(CrescTextKind::Crescendo, 100.0, 1500.0);
        let b = render(CrescTextKind::Decrescendo, 100.0, 1500.0);
        let c = render(CrescTextKind::Diminuendo, 100.0, 1500.0);
        assert_ne!(a, b, "cresc. and decresc. SVG must differ");
        assert_ne!(b, c, "decresc. and dim. SVG must differ");
        assert_ne!(a, c, "cresc. and dim. SVG must differ");
    }

    #[test]
    fn different_endpoints_produce_different_svg() {
        let a = render(CrescTextKind::Crescendo, 100.0, 1000.0);
        let b = render(CrescTextKind::Crescendo, 100.0, 2000.0);
        assert_ne!(
            a, b,
            "different x_end values must produce visually distinct output"
        );
    }

    // ---- element count guards -------------------------------------------

    #[test]
    fn emits_no_path_elements() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert_eq!(
            out.matches("<path").count(),
            0,
            "cresc.-text renderer should emit no <path> elements (only text + line)"
        );
    }

    #[test]
    fn emits_exactly_one_line_when_room_for_continuation() {
        let out = render(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert_eq!(
            out.matches("<line ").count(),
            1,
            "should emit exactly one line (the dashed continuation)"
        );
    }

    #[test]
    fn emits_zero_lines_when_no_room_for_continuation() {
        let out = render(CrescTextKind::Crescendo, 500.0, 500.0);
        assert_eq!(
            out.matches("<line ").count(),
            0,
            "no continuation, no line"
        );
    }

    // ---- cross-system continuation segments -----------------------------

    fn render_continuation(kind: CrescTextKind, x_start: f64, x_end: f64) -> String {
        let layout =
            layout_cresc_text_continuation(kind, x_start, x_end, &test_staff(), SS);
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        svg.to_svg()
    }

    #[test]
    fn continuation_emits_zero_text_elements() {
        // The whole point of the continuation variant: no label.
        let out = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert_eq!(
            out.matches("<text").count(),
            0,
            "continuation must NOT emit a <text> element; got:\n{out}"
        );
    }

    #[test]
    fn continuation_emits_zero_text_for_all_kinds() {
        // Regression guard: a regression that special-cased one kind
        // would surface here.
        for kind in [
            CrescTextKind::Crescendo,
            CrescTextKind::Decrescendo,
            CrescTextKind::Diminuendo,
        ] {
            let out = render_continuation(kind, 100.0, 1500.0);
            assert_eq!(
                out.matches("<text").count(),
                0,
                "continuation must emit zero <text> elements for kind {:?}; got:\n{out}",
                kind
            );
        }
    }

    #[test]
    fn continuation_emits_no_label_string_anywhere() {
        // Even more defensive: the label content ("cresc.", "decresc.",
        // "dim.") must not appear anywhere in the output — not just
        // inside <text>...</text>. A regression that hand-rolled the
        // text using a different element would still be caught.
        let out_c = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        let out_d = render_continuation(CrescTextKind::Decrescendo, 100.0, 1500.0);
        let out_m = render_continuation(CrescTextKind::Diminuendo, 100.0, 1500.0);
        assert!(
            !out_c.contains("cresc."),
            "continuation SVG must not contain literal 'cresc.'; got:\n{out_c}"
        );
        assert!(
            !out_d.contains("decresc."),
            "continuation SVG must not contain literal 'decresc.'; got:\n{out_d}"
        );
        assert!(
            !out_m.contains("dim."),
            "continuation SVG must not contain literal 'dim.'; got:\n{out_m}"
        );
    }

    #[test]
    fn continuation_emits_exactly_one_line_when_room() {
        let out = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert_eq!(
            out.matches("<line ").count(),
            1,
            "continuation should emit exactly one dashed line; got:\n{out}"
        );
    }

    #[test]
    fn continuation_emits_dashed_stroke() {
        let out = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert!(
            out.contains("stroke-dasharray"),
            "continuation line must be dashed; got:\n{out}"
        );
    }

    #[test]
    fn continuation_dasharray_matches_layout_constants() {
        // Same dash-spec as the in-system marking — so the visual
        // register is continuous across the system boundary.
        let out = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        let expected = format!(
            "stroke-dasharray=\"{},{}\"",
            CRESC_TEXT_DASH_LENGTH_SS * SS,
            CRESC_TEXT_DASH_GAP_SS * SS
        );
        assert!(
            out.contains(&expected),
            "expected {expected:?} in continuation SVG; got:\n{out}"
        );
    }

    #[test]
    fn continuation_dashed_line_starts_at_x_start() {
        // No label region — line begins at x_start.
        let x_start = 175.5;
        let layout = layout_cresc_text_continuation(
            CrescTextKind::Crescendo,
            x_start,
            2000.0,
            &test_staff(),
            SS,
        );
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        let out = svg.to_svg();
        let needle = format!("x1=\"{}\"", x_start);
        assert!(
            out.contains(&needle),
            "continuation dashed line must start at x_start ({}); got:\n{out}",
            x_start
        );
    }

    #[test]
    fn continuation_dashed_line_ends_at_x_end() {
        let x_end = 1875.25;
        let layout = layout_cresc_text_continuation(
            CrescTextKind::Crescendo,
            100.0,
            x_end,
            &test_staff(),
            SS,
        );
        let mut svg = test_svg();
        draw_cresc_text(&mut svg, &layout);
        let out = svg.to_svg();
        let needle = format!("x2=\"{}\"", x_end);
        assert!(
            out.contains(&needle),
            "continuation dashed line must end at x_end ({}); got:\n{out}",
            x_end
        );
    }

    #[test]
    fn continuation_no_line_when_x_end_equals_x_start() {
        // Degenerate range: zero-length continuation. Should emit
        // nothing — no text and no line.
        let out = render_continuation(CrescTextKind::Crescendo, 500.0, 500.0);
        assert_eq!(
            out.matches("<line ").count(),
            0,
            "zero-length continuation should emit no <line>"
        );
        assert_eq!(
            out.matches("<text").count(),
            0,
            "zero-length continuation should still emit no <text>"
        );
    }

    #[test]
    fn continuation_differs_visually_from_plain() {
        // Same coords, same kind — different SVG (continuation has
        // no label, line starts further left).
        let plain = render(CrescTextKind::Crescendo, 100.0, 1500.0);
        let cont = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert_ne!(
            plain, cont,
            "plain and continuation must render to visually distinct SVG"
        );
    }

    #[test]
    fn continuation_emits_no_path_elements() {
        let out = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert_eq!(
            out.matches("<path").count(),
            0,
            "continuation should emit no <path> elements"
        );
    }

    #[test]
    fn continuation_emits_no_italic_style() {
        // No <text> => no font-style attribute either. Belt-and-
        // suspenders alongside continuation_emits_zero_text_elements.
        let out = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        assert!(
            !out.contains("font-style=\"italic\""),
            "continuation must not emit italic styling (no label rendered); got:\n{out}"
        );
    }

    #[test]
    fn continuation_kinds_render_identically_when_no_label() {
        // Since the label is suppressed and the dashed-line geometry
        // is identical across kinds (same dash_length/gap/thickness,
        // same baseline), all three kinds must produce *identical*
        // SVG for the same x_start/x_end. This locks the "label is
        // the only thing that varies between kinds" invariant.
        let c = render_continuation(CrescTextKind::Crescendo, 100.0, 1500.0);
        let d = render_continuation(CrescTextKind::Decrescendo, 100.0, 1500.0);
        let m = render_continuation(CrescTextKind::Diminuendo, 100.0, 1500.0);
        assert_eq!(
            c, d,
            "Crescendo/Decrescendo continuation SVG must be identical (no label, identical line geometry)"
        );
        assert_eq!(
            d, m,
            "Decrescendo/Diminuendo continuation SVG must be identical"
        );
    }
}
