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

/// Speed/density variant for the trill wavy-line extension.
///
/// SMuFL defines a family of `wiggleTrill*` glyphs at progressively shorter
/// (faster, denser) and longer (slower, sparser) repeat offsets. Selecting a
/// speed visually communicates how rapidly the trill should be played without
/// changing the glyph's overall meaning. `Standard` is the conventional
/// neutral wiggle that most published engravings use.
///
/// The numeric ordering of variants matches SMuFL's progression from fastest
/// (densest tiles) to slowest (sparsest tiles).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrillWiggleSpeed {
    /// `wiggleTrillFastest` — the densest wiggle.
    Fastest,
    /// `wiggleTrillFasterStill`.
    FasterStill,
    /// `wiggleTrillFaster`.
    Faster,
    /// `wiggleTrillFast`.
    Fast,
    /// `wiggleTrill` — the neutral, default wiggle.
    Standard,
    /// `wiggleTrillSlow`.
    Slow,
    /// `wiggleTrillSlower`.
    Slower,
    /// `wiggleTrillSlowerStill`.
    SlowerStill,
    /// `wiggleTrillSlowest` — the sparsest wiggle.
    Slowest,
}

impl TrillWiggleSpeed {
    /// Map this speed to its SMuFL `Glyph`.
    pub fn to_glyph(self) -> Glyph {
        match self {
            Self::Fastest => Glyph::WiggleTrillFastest,
            Self::FasterStill => Glyph::WiggleTrillFasterStill,
            Self::Faster => Glyph::WiggleTrillFaster,
            Self::Fast => Glyph::WiggleTrillFast,
            Self::Standard => Glyph::WiggleTrill,
            Self::Slow => Glyph::WiggleTrillSlow,
            Self::Slower => Glyph::WiggleTrillSlower,
            Self::SlowerStill => Glyph::WiggleTrillSlowerStill,
            Self::Slowest => Glyph::WiggleTrillSlowest,
        }
    }

    /// Canonical ordering from fastest (densest) to slowest (sparsest).
    pub const ALL: [TrillWiggleSpeed; 9] = [
        Self::Fastest,
        Self::FasterStill,
        Self::Faster,
        Self::Fast,
        Self::Standard,
        Self::Slow,
        Self::Slower,
        Self::SlowerStill,
        Self::Slowest,
    ];
}

// The default cannot be derived: the desired default is `Standard`, which is
// not the first variant. (Derived `Default` would pick `Fastest`.)
#[allow(clippy::derivable_impls)]
impl Default for TrillWiggleSpeed {
    fn default() -> Self {
        Self::Standard
    }
}

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
    layout_trill_extension_with_glyph(start_x, end_x, y, Glyph::WiggleTrill, segment_advance_fu)
}

/// Lay out a trill wavy-line extension using a caller-chosen wiggle glyph.
///
/// Identical to [`layout_trill_extension`] except the tile glyph is specified
/// explicitly. Use this when rendering a speed-variant wiggle
/// (`Glyph::WiggleTrillFast`, `WiggleTrillSlow`, etc.) — the caller is
/// responsible for querying that glyph's advance width from the active font
/// and passing it as `segment_advance_fu`. The two values must agree: passing
/// `WiggleTrillFast` with the standard wiggle's advance would tile gaps or
/// overlaps between segments.
///
/// Returns `None` under the same conditions as [`layout_trill_extension`]:
/// non-positive advance, zero span, negative span, or span smaller than one
/// segment.
pub fn layout_trill_extension_with_glyph(
    start_x: f64,
    end_x: f64,
    y: f64,
    glyph: Glyph,
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
        glyph,
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

    #[test]
    fn wiggle_speed_to_glyph_distinct() {
        // Every speed must map to a unique SMuFL glyph — without this,
        // selecting a speed wouldn't actually produce a different wiggle.
        let glyphs: Vec<Glyph> = TrillWiggleSpeed::ALL.iter().map(|s| s.to_glyph()).collect();
        let mut sorted = glyphs.clone();
        sorted.sort_by_key(|g| format!("{g:?}"));
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            TrillWiggleSpeed::ALL.len(),
            "speeds must map to distinct glyphs"
        );
    }

    #[test]
    fn wiggle_speed_standard_is_wiggle_trill() {
        // Standard must be the default neutral glyph so existing callers
        // (which select `Standard` via `Default`) keep their current output.
        assert_eq!(TrillWiggleSpeed::Standard.to_glyph(), Glyph::WiggleTrill);
    }

    #[test]
    fn wiggle_speed_default_is_standard() {
        assert_eq!(TrillWiggleSpeed::default(), TrillWiggleSpeed::Standard);
    }

    #[test]
    fn wiggle_speed_all_contains_nine_variants() {
        assert_eq!(TrillWiggleSpeed::ALL.len(), 9);
    }

    #[test]
    fn wiggle_speed_all_includes_every_variant() {
        // Pattern-match every variant so a new addition to the enum must
        // also be added to ALL — preventing accidental drift.
        for speed in TrillWiggleSpeed::ALL {
            let _ok = match speed {
                TrillWiggleSpeed::Fastest
                | TrillWiggleSpeed::FasterStill
                | TrillWiggleSpeed::Faster
                | TrillWiggleSpeed::Fast
                | TrillWiggleSpeed::Standard
                | TrillWiggleSpeed::Slow
                | TrillWiggleSpeed::Slower
                | TrillWiggleSpeed::SlowerStill
                | TrillWiggleSpeed::Slowest => true,
            };
        }
    }

    #[test]
    fn layout_with_glyph_uses_provided_glyph() {
        let layout =
            layout_trill_extension_with_glyph(0.0, 200.0, 50.0, Glyph::WiggleTrillFast, 50.0)
                .expect("non-empty span fits");
        assert_eq!(layout.glyph, Glyph::WiggleTrillFast);
        assert_eq!(layout.segment_xs.len(), 4);
    }

    #[test]
    fn layout_with_glyph_handles_slow_variant() {
        let layout =
            layout_trill_extension_with_glyph(0.0, 600.0, 0.0, Glyph::WiggleTrillSlowest, 200.0)
                .expect("non-empty span fits");
        assert_eq!(layout.glyph, Glyph::WiggleTrillSlowest);
        assert_eq!(layout.segment_xs.len(), 3);
        assert_eq!(layout.segment_advance, 200.0);
    }

    #[test]
    fn layout_with_glyph_returns_none_for_zero_advance() {
        // Same fail-safe as the non-glyph version.
        assert!(
            layout_trill_extension_with_glyph(0.0, 100.0, 0.0, Glyph::WiggleTrillFast, 0.0)
                .is_none()
        );
    }

    #[test]
    fn layout_trill_extension_delegates_to_with_glyph_using_standard() {
        // The convenience function MUST use Standard (`WiggleTrill`) so old
        // call sites unchanged by this chunk keep their pixel-perfect output.
        let a = layout_trill_extension(0.0, 300.0, 25.0, 100.0).unwrap();
        let b =
            layout_trill_extension_with_glyph(0.0, 300.0, 25.0, Glyph::WiggleTrill, 100.0).unwrap();
        assert_eq!(a, b);
    }
}
