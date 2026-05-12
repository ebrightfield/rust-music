//! Trill wavy-line extension — the wiggle that follows a "tr" glyph for a
//! sustained trill.
//!
//! A trill on a note of any duration longer than ~a quarter is conventionally
//! drawn with a wavy line trailing the "tr" glyph for the full duration of
//! the trill. The wavy line is constructed by tiling the SMuFL `wiggleTrill`
//! segment glyph horizontally between the trill's start and end x-coordinates.
//!
//! This module computes the geometry only; the renderer (`trill_extension_renderer`)
//! emits one path per segment by translating the wiggle glyph outline.
//!
//! Layout is font-agnostic: callers query the wiggle segment's advance width
//! from the active font and pass it in as `segment_advance_fu`. Renderers
//! consume both the layout and the font to emit per-segment paths.

use smufl::Glyph;

/// Computed positions for a tiled trill wavy-line extension.
#[derive(Clone, Debug, PartialEq)]
pub struct TrillExtensionLayout {
    /// X-coordinate of each tiled segment's left edge.
    ///
    /// Empty when the available span is shorter than one segment.
    pub segment_xs: Vec<f64>,
    /// Y-coordinate shared by every segment (the wiggle's anchor baseline).
    ///
    /// Conventionally aligned with the trill "tr" glyph's y so the wavy line
    /// reads as a horizontal continuation of the trill marking.
    pub y: f64,
    /// SMuFL glyph used for each tile.
    pub glyph: Glyph,
    /// Per-segment advance in font units (echoed from the input so renderers
    /// don't need to re-query the font for centering or hit-testing).
    pub segment_advance: f64,
}

/// Lay out a trill wavy-line extension by tiling the wiggle segment glyph.
///
/// Tiles whole copies of `Glyph::WiggleTrill` from `start_x` rightward toward
/// `end_x`. Returns `None` when the span is shorter than a single segment or
/// when `segment_advance_fu` is non-positive (which would imply zero-width
/// tiles and an infinite loop).
///
/// The number of segments is `floor((end_x - start_x) / segment_advance_fu)`.
/// Any leftover gap on the right is left empty rather than stretching the
/// tiles — non-uniform horizontal scaling of the wiggle glyph distorts its
/// shape, and the visual gap of less than one segment width is small enough
/// not to read as a missing wiggle.
///
/// `y` should be the y-coordinate of the trill "tr" glyph's anchor so the
/// wiggle reads as a continuation.
pub fn layout_trill_extension(
    start_x: f64,
    end_x: f64,
    y: f64,
    segment_advance_fu: f64,
) -> Option<TrillExtensionLayout> {
    if segment_advance_fu <= 0.0 {
        return None;
    }
    let span = end_x - start_x;
    if span < segment_advance_fu {
        return None;
    }

    let count = (span / segment_advance_fu).floor() as usize;
    if count == 0 {
        return None;
    }

    let segment_xs = (0..count)
        .map(|i| start_x + (i as f64) * segment_advance_fu)
        .collect();

    Some(TrillExtensionLayout {
        segment_xs,
        y,
        glyph: Glyph::WiggleTrill,
        segment_advance: segment_advance_fu,
    })
}

/// Compute the x-coordinate of the right edge of a trill extension layout.
///
/// Useful for hit-testing, anchor placement, or when chaining decorations
/// after the wiggle line ends.
pub fn trill_extension_right_edge(layout: &TrillExtensionLayout) -> f64 {
    match layout.segment_xs.last() {
        Some(last) => last + layout.segment_advance,
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_span_returns_none() {
        assert!(layout_trill_extension(100.0, 100.0, 50.0, 80.0).is_none());
    }

    #[test]
    fn span_smaller_than_segment_returns_none() {
        // 70 < 80, can't even fit one segment
        assert!(layout_trill_extension(100.0, 170.0, 50.0, 80.0).is_none());
    }

    #[test]
    fn negative_span_returns_none() {
        assert!(layout_trill_extension(200.0, 100.0, 50.0, 80.0).is_none());
    }

    #[test]
    fn zero_segment_advance_returns_none() {
        assert!(layout_trill_extension(100.0, 200.0, 50.0, 0.0).is_none());
    }

    #[test]
    fn negative_segment_advance_returns_none() {
        // Guards against floor() producing a negative count.
        assert!(layout_trill_extension(100.0, 200.0, 50.0, -5.0).is_none());
    }

    #[test]
    fn exact_fit_yields_one_segment() {
        let layout = layout_trill_extension(100.0, 180.0, 50.0, 80.0).unwrap();
        assert_eq!(layout.segment_xs, vec![100.0]);
        assert_eq!(layout.y, 50.0);
        assert_eq!(layout.glyph, Glyph::WiggleTrill);
        assert_eq!(layout.segment_advance, 80.0);
    }

    #[test]
    fn two_segments_when_span_double() {
        let layout = layout_trill_extension(100.0, 260.0, 50.0, 80.0).unwrap();
        assert_eq!(layout.segment_xs, vec![100.0, 180.0]);
    }

    #[test]
    fn span_between_n_and_n_plus_one_tiles_floor() {
        // 100..295 = 195 wide, 195/80 = 2.4375 -> 2 segments, no fractional tile
        let layout = layout_trill_extension(100.0, 295.0, 50.0, 80.0).unwrap();
        assert_eq!(layout.segment_xs.len(), 2);
        assert_eq!(layout.segment_xs[0], 100.0);
        assert_eq!(layout.segment_xs[1], 180.0);
    }

    #[test]
    fn many_segments_uniform_spacing() {
        let layout = layout_trill_extension(0.0, 1000.0, 0.0, 100.0).unwrap();
        assert_eq!(layout.segment_xs.len(), 10);
        for (i, x) in layout.segment_xs.iter().enumerate() {
            assert!((x - (i as f64 * 100.0)).abs() < 1e-9, "segment {i} at {x}");
        }
    }

    #[test]
    fn y_coordinate_is_preserved() {
        let layout = layout_trill_extension(0.0, 100.0, 123.456, 50.0).unwrap();
        assert_eq!(layout.y, 123.456);
    }

    #[test]
    fn glyph_is_wiggle_trill() {
        let layout = layout_trill_extension(0.0, 100.0, 0.0, 50.0).unwrap();
        assert_eq!(layout.glyph, Glyph::WiggleTrill);
    }

    #[test]
    fn right_edge_after_last_segment() {
        let layout = layout_trill_extension(100.0, 260.0, 0.0, 80.0).unwrap();
        // Last tile starts at 180, segment width 80 -> right edge = 260
        assert_eq!(trill_extension_right_edge(&layout), 260.0);
    }

    #[test]
    fn right_edge_for_single_segment() {
        let layout = layout_trill_extension(50.0, 130.0, 0.0, 80.0).unwrap();
        assert_eq!(trill_extension_right_edge(&layout), 130.0);
    }

    #[test]
    fn segment_xs_strictly_increasing() {
        let layout = layout_trill_extension(0.0, 500.0, 0.0, 60.0).unwrap();
        for pair in layout.segment_xs.windows(2) {
            assert!(pair[1] > pair[0], "segments not increasing: {pair:?}");
        }
    }

    #[test]
    fn extension_does_not_overflow_end_x() {
        // The right edge of the final segment must not exceed end_x.
        let end_x = 295.0;
        let layout = layout_trill_extension(100.0, end_x, 0.0, 80.0).unwrap();
        let right = trill_extension_right_edge(&layout);
        assert!(
            right <= end_x,
            "right edge {right} must not exceed end_x {end_x}"
        );
    }

    #[test]
    fn fractional_advance_works() {
        // Non-integer segment widths shouldn't break the tiling.
        let layout = layout_trill_extension(0.0, 100.0, 0.0, 33.333).unwrap();
        // floor(100 / 33.333) = 3 segments
        assert_eq!(layout.segment_xs.len(), 3);
        assert!((layout.segment_xs[1] - 33.333).abs() < 1e-9);
    }
}
