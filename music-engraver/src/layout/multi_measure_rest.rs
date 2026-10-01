use crate::layout::staff::StaffLayout;
use smufl::Glyph;

/// Distance from the middle staff line to the top/bottom edge of the H-bar,
/// in staff spaces. The bar spans from line 2 (space below middle) to
/// line 6 (space above middle), i.e. 1 staff space above and below the
/// middle line.
pub const HBAR_HALF_HEIGHT_SS: f64 = 1.0;

/// Thickness of the vertical serif strokes at each end of the H-bar,
/// in staff spaces.
pub const HBAR_SERIF_THICKNESS_SS: f64 = 0.16;

/// Horizontal padding from barline edges to H-bar ends, in staff spaces.
/// Prevents the bar from touching the barlines.
pub const HBAR_HORIZONTAL_PADDING_SS: f64 = 1.5;

/// Thickness of the horizontal bar (the crossbar of the H), in staff spaces.
pub const HBAR_BAR_THICKNESS_SS: f64 = 0.5;

/// Distance from the top of the H-bar to the baseline of the count number,
/// in staff spaces.
pub const COUNT_ABOVE_HBAR_SS: f64 = 0.8;

/// Font size for the measure count number, in staff spaces.
pub const COUNT_FONT_SIZE_SS: f64 = 1.8;

/// Horizontal spacing between adjacent rest glyphs in a church-rest cluster,
/// in staff spaces. Wide enough that breve + whole rest read as two separate
/// rests rather than a single longer glyph.
pub const CHURCH_REST_GLYPH_SPACING_SS: f64 = 0.7;

/// Distance from the top staff line (position 8) to the baseline of the count
/// number above a church-rest cluster, in staff spaces. Church-rest glyphs
/// sit on the middle line, so a smaller offset than the H-bar variant
/// suffices to clear them.
pub const CHURCH_REST_COUNT_ABOVE_STAFF_SS: f64 = 1.4;

/// Maximum count for which a church-rest cluster is drawn. Counts above this
/// always render as an H-bar — the rest-glyph cluster would otherwise span
/// too far to read as a single duration.
pub const CHURCH_REST_MAX_COUNT: u32 = 4;

/// Style used to render a multi-measure rest.
///
/// `HBar` is the modern default (a thick horizontal bar with vertical serifs
/// and a count number above). `Church` is the older "church rest" convention
/// used for small counts (1–4): combinations of whole and breve rest glyphs
/// placed on the staff, optionally with a count number above. For counts
/// above [`CHURCH_REST_MAX_COUNT`], church style falls back to the H-bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MultiMeasureRestStyle {
    /// Modern thick-bar style with vertical serifs and a count number.
    #[default]
    HBar,
    /// Church-rest style: stacked SMuFL rest glyphs on the staff for small
    /// counts. Falls back to [`HBar`](Self::HBar) for counts greater than
    /// [`CHURCH_REST_MAX_COUNT`].
    Church,
}

/// Computed geometry for a multi-measure rest H-bar with count number.
#[derive(Clone, Debug)]
pub struct MultiMeasureRestLayout {
    /// X coordinate of the left edge of the H-bar (left serif).
    pub x_left: f64,
    /// X coordinate of the right edge of the H-bar (right serif).
    pub x_right: f64,
    /// Y coordinate of the top of the vertical serifs.
    pub y_top: f64,
    /// Y coordinate of the bottom of the vertical serifs.
    pub y_bottom: f64,
    /// Y coordinate of the top edge of the horizontal bar.
    pub bar_y_top: f64,
    /// Y coordinate of the bottom edge of the horizontal bar.
    pub bar_y_bottom: f64,
    /// Thickness of the vertical serif strokes in font design units.
    pub serif_thickness: f64,
    /// X coordinate for the centered count number text.
    pub count_x: f64,
    /// Y coordinate for the count number text baseline.
    pub count_y: f64,
    /// Font size for the count number in font design units.
    pub count_font_size: f64,
    /// The number of measures of rest.
    pub count: u32,
}

/// Compute the layout for a multi-measure rest spanning a given width.
///
/// The H-bar (thick horizontal line with vertical serifs at each end) is
/// centered vertically on the middle staff line, spanning most of the
/// available width with padding from the edges. A count number is placed
/// above the bar, centered horizontally.
///
/// `x_left_edge` and `x_right_edge` define the available horizontal space
/// (typically the barline-to-barline span of the rest measure).
pub fn layout_multi_measure_rest(
    x_left_edge: f64,
    x_right_edge: f64,
    count: u32,
    staff: &StaffLayout,
) -> MultiMeasureRestLayout {
    let ss = staff.staff_space;

    let h_pad = HBAR_HORIZONTAL_PADDING_SS * ss;
    let x_left = x_left_edge + h_pad;
    let x_right = x_right_edge - h_pad;

    // Middle line is staff position 4
    let mid_y = staff.y_of(4);
    let half_height = HBAR_HALF_HEIGHT_SS * ss;
    let y_top = mid_y - half_height;
    let y_bottom = mid_y + half_height;

    let bar_half = HBAR_BAR_THICKNESS_SS * ss / 2.0;
    let bar_y_top = mid_y - bar_half;
    let bar_y_bottom = mid_y + bar_half;

    let serif_thickness = HBAR_SERIF_THICKNESS_SS * ss;

    let count_x = (x_left + x_right) / 2.0;
    let count_y = y_top - COUNT_ABOVE_HBAR_SS * ss;
    let count_font_size = COUNT_FONT_SIZE_SS * ss;

    MultiMeasureRestLayout {
        x_left,
        x_right,
        y_top,
        y_bottom,
        bar_y_top,
        bar_y_bottom,
        serif_thickness,
        count_x,
        count_y,
        count_font_size,
        count,
    }
}

/// A single rest glyph placement within a church-rest cluster.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChurchRestGlyph {
    /// SMuFL glyph to draw (typically [`Glyph::RestWhole`] or
    /// [`Glyph::RestDoubleWhole`]).
    pub glyph: Glyph,
    /// X coordinate of the glyph origin (translation target). The glyph
    /// outline is drawn with its left edge near this x — the SMuFL convention
    /// is that the glyph origin sits at the natural notational anchor for
    /// that glyph.
    pub x: f64,
    /// Y coordinate of the glyph origin (the staff line the rest hangs from
    /// or sits on).
    pub y: f64,
}

/// Computed geometry for a church-rest cluster of SMuFL rest glyphs.
///
/// The cluster is centered horizontally within `[x_left_edge, x_right_edge]`.
/// Glyph widths are not known at layout time (they depend on the font); the
/// renderer translates each glyph to its `(x, y)` anchor and the natural
/// glyph metrics handle the rest. The placement uses fixed nominal advance
/// widths derived from the staff space — Bravura's whole and breve rests are
/// both roughly 1.4 staff spaces wide, so this gives visually balanced
/// spacing without a font query.
#[derive(Clone, Debug)]
pub struct ChurchRestLayout {
    /// Rest glyphs to draw, left to right.
    pub glyphs: Vec<ChurchRestGlyph>,
    /// X coordinate for the centered count number text.
    pub count_x: f64,
    /// Y coordinate for the count number text baseline.
    pub count_y: f64,
    /// Font size for the count number in font design units.
    pub count_font_size: f64,
    /// The number of measures of rest this cluster represents. Matches the
    /// caller-supplied count even for `count == 0` or `count > CHURCH_REST_MAX_COUNT`;
    /// in those cases callers should fall back to the H-bar form.
    pub count: u32,
}

/// Decide which SMuFL rest glyphs depict a church-rest cluster of `count`
/// measures.
///
/// Conventions (from common practice for small "tacet" counts):
/// - **1**: a single whole rest, hanging from the 4th line (the standard
///   single-bar tacet glyph).
/// - **2**: a single breve (double-whole) rest, sitting on the 3rd line.
/// - **3**: breve + whole — the breve sits on the 3rd line, the whole hangs
///   from the 4th line; together they read "2 + 1".
/// - **4**: two breve rests side by side, reading "2 + 2".
///
/// Returns an empty vector for any other count — the H-bar form is the only
/// reasonable rendering above 4 (the cluster would span too wide and become
/// ambiguous).
fn church_rest_glyph_sequence(count: u32) -> Vec<Glyph> {
    match count {
        1 => vec![Glyph::RestWhole],
        2 => vec![Glyph::RestDoubleWhole],
        3 => vec![Glyph::RestDoubleWhole, Glyph::RestWhole],
        4 => vec![Glyph::RestDoubleWhole, Glyph::RestDoubleWhole],
        _ => Vec::new(),
    }
}

/// Staff position the rest glyph anchors against.
///
/// In Bravura the whole-rest origin sits on the 4th line (the rest hangs
/// below it) and the breve-rest origin sits on the 3rd (middle) line (the
/// rest extends upward into the space above). Position numbering in this
/// codebase: 0 = bottom line, 4 = middle line, 6 = 4th line from bottom,
/// 8 = top line.
fn church_rest_glyph_anchor_position(glyph: Glyph) -> i8 {
    match glyph {
        Glyph::RestWhole => 6,
        Glyph::RestDoubleWhole => 4,
        // Longa & maxima not used by the current sequences; default to the
        // middle line if a future variant adds them.
        _ => 4,
    }
}

/// Compute the layout for a church-rest cluster.
///
/// For `count` in `1..=CHURCH_REST_MAX_COUNT`, picks an appropriate sequence
/// of SMuFL rest glyphs and places them centered within
/// `[x_left_edge, x_right_edge]`. For other counts, returns a layout with
/// an empty `glyphs` field — callers should fall back to
/// [`layout_multi_measure_rest`] (H-bar).
pub fn layout_church_rest(
    x_left_edge: f64,
    x_right_edge: f64,
    count: u32,
    staff: &StaffLayout,
) -> ChurchRestLayout {
    let ss = staff.staff_space;
    let glyph_seq = church_rest_glyph_sequence(count);

    // Center the cluster within the available span. We use a nominal advance
    // width of ~1.4 staff spaces per glyph (a reasonable estimate for the
    // Bravura whole and breve rests) so x positions don't require a font
    // query. The renderer translates each glyph to its computed origin.
    let nominal_glyph_w_ss = 1.4;
    let spacing_ss = CHURCH_REST_GLYPH_SPACING_SS;
    let n = glyph_seq.len() as f64;
    let cluster_width_ss = if n == 0.0 {
        0.0
    } else {
        n * nominal_glyph_w_ss + (n - 1.0).max(0.0) * spacing_ss
    };

    let span_center = (x_left_edge + x_right_edge) / 2.0;
    let cluster_left = span_center - (cluster_width_ss * ss) / 2.0;

    let mut glyphs = Vec::with_capacity(glyph_seq.len());
    for (i, glyph) in glyph_seq.iter().enumerate() {
        let i_f = i as f64;
        // x targets the *left edge* of the glyph (glyph origins in Bravura
        // have x=0 at the leftmost outline coordinate).
        let x = cluster_left + i_f * (nominal_glyph_w_ss + spacing_ss) * ss;
        let pos = church_rest_glyph_anchor_position(*glyph);
        let y = staff.y_of(pos);
        glyphs.push(ChurchRestGlyph {
            glyph: *glyph,
            x,
            y,
        });
    }

    // Count number sits above the staff (clearly above any rest glyph).
    let count_y = staff.y_of(8) - CHURCH_REST_COUNT_ABOVE_STAFF_SS * ss;
    let count_x = span_center;
    let count_font_size = COUNT_FONT_SIZE_SS * ss;

    ChurchRestLayout {
        glyphs,
        count_x,
        count_y,
        count_font_size,
        count,
    }
}

/// True when `count` is in the range where church-rest depiction is
/// supported (i.e. [`layout_church_rest`] will return a non-empty glyph
/// sequence).
pub fn church_rest_supported(count: u32) -> bool {
    (1..=CHURCH_REST_MAX_COUNT).contains(&count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staff() -> StaffLayout {
        StaffLayout::new(0.0, 0.0, 5000.0, 250.0)
    }

    #[test]
    fn hbar_centered_on_middle_line() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let mid_y = staff.y_of(4);
        let center = (layout.y_top + layout.y_bottom) / 2.0;
        assert!(
            (center - mid_y).abs() < 0.01,
            "H-bar should be centered on middle line: center={center}, mid_y={mid_y}"
        );
    }

    #[test]
    fn hbar_spans_one_space_each_side() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let height = layout.y_bottom - layout.y_top;
        let expected = 2.0 * HBAR_HALF_HEIGHT_SS * staff.staff_space;
        assert!(
            (height - expected).abs() < 0.01,
            "H-bar height should be {expected}, got {height}"
        );
    }

    #[test]
    fn hbar_padded_from_edges() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(100.0, 4900.0, 4, &staff);
        let pad = HBAR_HORIZONTAL_PADDING_SS * staff.staff_space;
        assert!(
            (layout.x_left - (100.0 + pad)).abs() < 0.01,
            "x_left should be padded from left edge"
        );
        assert!(
            (layout.x_right - (4900.0 - pad)).abs() < 0.01,
            "x_right should be padded from right edge"
        );
    }

    #[test]
    fn bar_thinner_than_serifs() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let bar_height = layout.bar_y_bottom - layout.bar_y_top;
        let serif_height = layout.y_bottom - layout.y_top;
        assert!(
            bar_height < serif_height,
            "horizontal bar ({bar_height}) should be thinner than serif span ({serif_height})"
        );
    }

    #[test]
    fn bar_centered_on_middle_line() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let mid_y = staff.y_of(4);
        let bar_center = (layout.bar_y_top + layout.bar_y_bottom) / 2.0;
        assert!(
            (bar_center - mid_y).abs() < 0.01,
            "bar center ({bar_center}) should match middle line ({mid_y})"
        );
    }

    #[test]
    fn count_above_hbar() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        assert!(
            layout.count_y < layout.y_top,
            "count y ({}) should be above (less than) H-bar top ({})",
            layout.count_y,
            layout.y_top
        );
    }

    #[test]
    fn count_centered_horizontally() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(100.0, 4900.0, 4, &staff);
        let expected_center = (layout.x_left + layout.x_right) / 2.0;
        assert!(
            (layout.count_x - expected_center).abs() < 0.01,
            "count x ({}) should be centered ({})",
            layout.count_x,
            expected_center
        );
    }

    #[test]
    fn count_preserved() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 12, &staff);
        assert_eq!(layout.count, 12);
    }

    #[test]
    fn serif_thickness_positive() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        assert!(layout.serif_thickness > 0.0);
    }

    #[test]
    fn font_size_positive() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        assert!(layout.count_font_size > 0.0);
    }

    #[test]
    fn different_counts_same_geometry() {
        let staff = test_staff();
        let l4 = layout_multi_measure_rest(0.0, 5000.0, 4, &staff);
        let l16 = layout_multi_measure_rest(0.0, 5000.0, 16, &staff);
        // Geometry identical, only count differs
        assert!((l4.x_left - l16.x_left).abs() < 0.01);
        assert!((l4.y_top - l16.y_top).abs() < 0.01);
        assert_ne!(l4.count, l16.count);
    }

    #[test]
    fn narrow_span_still_produces_layout() {
        let staff = test_staff();
        let layout = layout_multi_measure_rest(0.0, 1000.0, 2, &staff);
        // Even if x_right < x_left (very narrow), we still get a layout
        // (renderer can decide whether to draw)
        assert_eq!(layout.count, 2);
    }

    #[test]
    fn nonzero_origin() {
        let staff = StaffLayout::new(500.0, 200.0, 5000.0, 250.0);
        let layout = layout_multi_measure_rest(500.0, 5500.0, 3, &staff);
        let mid_y = staff.y_of(4);
        let center = (layout.y_top + layout.y_bottom) / 2.0;
        assert!(
            (center - mid_y).abs() < 0.01,
            "should work with nonzero origin"
        );
    }

    // ── Church-rest tests ─────────────────────────────────────────────────

    #[test]
    fn multi_measure_rest_style_default_is_hbar() {
        let s: MultiMeasureRestStyle = Default::default();
        assert_eq!(s, MultiMeasureRestStyle::HBar);
    }

    #[test]
    fn church_rest_count_1_uses_whole_rest() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 1, &staff);
        assert_eq!(layout.glyphs.len(), 1);
        assert_eq!(layout.glyphs[0].glyph, Glyph::RestWhole);
    }

    #[test]
    fn church_rest_count_2_uses_breve_rest() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 2, &staff);
        assert_eq!(layout.glyphs.len(), 1);
        assert_eq!(layout.glyphs[0].glyph, Glyph::RestDoubleWhole);
    }

    #[test]
    fn church_rest_count_3_is_breve_plus_whole() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 3, &staff);
        assert_eq!(layout.glyphs.len(), 2);
        assert_eq!(layout.glyphs[0].glyph, Glyph::RestDoubleWhole);
        assert_eq!(layout.glyphs[1].glyph, Glyph::RestWhole);
    }

    #[test]
    fn church_rest_count_4_is_two_breves() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 4, &staff);
        assert_eq!(layout.glyphs.len(), 2);
        assert_eq!(layout.glyphs[0].glyph, Glyph::RestDoubleWhole);
        assert_eq!(layout.glyphs[1].glyph, Glyph::RestDoubleWhole);
    }

    #[test]
    fn church_rest_unsupported_counts_have_no_glyphs() {
        let staff = test_staff();
        for n in &[0u32, 5, 7, 20, 100] {
            let layout = layout_church_rest(0.0, 5000.0, *n, &staff);
            assert!(
                layout.glyphs.is_empty(),
                "count {n} should produce no glyphs (caller falls back to H-bar)"
            );
            // Even with no glyphs the count itself is preserved so callers
            // can inspect it.
            assert_eq!(layout.count, *n);
        }
    }

    #[test]
    fn church_rest_whole_anchors_on_fourth_line() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 1, &staff);
        // Whole rest hangs from the 4th line (position 6 in this codebase).
        let expected_y = staff.y_of(6);
        assert!(
            (layout.glyphs[0].y - expected_y).abs() < 0.01,
            "whole rest y={} should equal 4th-line y={}",
            layout.glyphs[0].y,
            expected_y
        );
    }

    #[test]
    fn church_rest_breve_anchors_on_middle_line() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 2, &staff);
        // Breve sits on the middle (3rd) line, position 4.
        let expected_y = staff.y_of(4);
        assert!(
            (layout.glyphs[0].y - expected_y).abs() < 0.01,
            "breve rest y={} should equal middle-line y={}",
            layout.glyphs[0].y,
            expected_y
        );
    }

    #[test]
    fn church_rest_glyphs_horizontally_ordered() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 4, &staff);
        assert!(
            layout.glyphs[1].x > layout.glyphs[0].x,
            "second glyph (x={}) should be right of first (x={})",
            layout.glyphs[1].x,
            layout.glyphs[0].x
        );
    }

    #[test]
    fn church_rest_glyph_spacing_is_about_one_staff_space() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 3, &staff);
        let gap = layout.glyphs[1].x - layout.glyphs[0].x;
        // Gap is one nominal glyph width (1.4ss) + spacing (0.7ss) = 2.1ss
        let expected_gap = (1.4 + CHURCH_REST_GLYPH_SPACING_SS) * staff.staff_space;
        assert!(
            (gap - expected_gap).abs() < 0.01,
            "glyph gap {gap} should equal {expected_gap}"
        );
    }

    #[test]
    fn church_rest_cluster_centered_within_span() {
        let staff = test_staff();
        let layout = layout_church_rest(1000.0, 4000.0, 4, &staff);
        // Cluster center should equal span center
        let span_center = (1000.0 + 4000.0) / 2.0;
        let cluster_center =
            (layout.glyphs[0].x + layout.glyphs[1].x) / 2.0 + 0.7 * staff.staff_space; // half a glyph width offset (rough)
                                                                                       // Looser tolerance since the "center" depends on the nominal glyph
                                                                                       // width assumption — assert the count_x is precisely centered instead.
        assert!((layout.count_x - span_center).abs() < 0.01);
        // And the cluster as a whole should be near the center, not the edge.
        assert!(layout.glyphs[0].x > 1000.0 + 100.0);
        assert!(cluster_center > span_center - 500.0 && cluster_center < span_center + 500.0);
    }

    #[test]
    fn church_rest_count_x_matches_span_center() {
        let staff = test_staff();
        let layout = layout_church_rest(500.0, 4500.0, 3, &staff);
        let expected = 2500.0;
        assert!(
            (layout.count_x - expected).abs() < 0.01,
            "count_x {} should equal span center {}",
            layout.count_x,
            expected
        );
    }

    #[test]
    fn church_rest_count_y_is_above_top_staff_line() {
        let staff = test_staff();
        let layout = layout_church_rest(0.0, 5000.0, 2, &staff);
        let top_line_y = staff.y_of(8);
        assert!(
            layout.count_y < top_line_y,
            "count y={} should be above (smaller than) top line y={}",
            layout.count_y,
            top_line_y
        );
    }

    #[test]
    fn church_rest_supported_predicate() {
        for n in 1..=CHURCH_REST_MAX_COUNT {
            assert!(church_rest_supported(n), "count {n} should be supported");
        }
        assert!(!church_rest_supported(0));
        assert!(!church_rest_supported(CHURCH_REST_MAX_COUNT + 1));
        assert!(!church_rest_supported(100));
    }

    #[test]
    fn church_rest_count_preserved_for_all_counts() {
        let staff = test_staff();
        for n in 0..=10u32 {
            let layout = layout_church_rest(0.0, 5000.0, n, &staff);
            assert_eq!(layout.count, n, "count must round-trip for n={n}");
        }
    }

    #[test]
    fn church_rest_anchor_helper_defaults_for_unknown_glyph() {
        // The helper is internal but routed through the public layout; cover
        // the fallback branch directly so future additions to the glyph set
        // can't silently default an anchor.
        assert_eq!(church_rest_glyph_anchor_position(Glyph::RestWhole), 6);
        assert_eq!(church_rest_glyph_anchor_position(Glyph::RestDoubleWhole), 4);
        assert_eq!(church_rest_glyph_anchor_position(Glyph::RestLonga), 4);
        assert_eq!(church_rest_glyph_anchor_position(Glyph::RestQuarter), 4);
    }
}
