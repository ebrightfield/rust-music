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

use crate::layout::ornament::Ornament;
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

    /// Index in [`TrillWiggleSpeed::ALL`] — `0` is [`Self::Fastest`],
    /// `8` is [`Self::Slowest`]. The ordering matches SMuFL's progression
    /// from densest tiles to sparsest tiles, so `a.index() < b.index()`
    /// iff `a` reads as faster than `b`.
    ///
    /// Total order is meaningful: subtraction of indices gives a signed
    /// "speed delta" used by [`TrillSpeedRamp::synthesize_regions`] to
    /// interpolate between two speed variants.
    pub const fn index(self) -> usize {
        match self {
            Self::Fastest => 0,
            Self::FasterStill => 1,
            Self::Faster => 2,
            Self::Fast => 3,
            Self::Standard => 4,
            Self::Slow => 5,
            Self::Slower => 6,
            Self::SlowerStill => 7,
            Self::Slowest => 8,
        }
    }

    /// Reverse of [`Self::index`]. Indices `0..=8` map to the canonical
    /// variants; any index `>= 9` saturates to [`Self::Slowest`] (and any
    /// negative value would have been clamped to `0` by the caller's
    /// `usize` cast).
    ///
    /// Saturating rather than `Option`-returning because the only sane
    /// internal caller is [`TrillSpeedRamp::synthesize_regions`], whose
    /// interpolation `round()` produces a non-negative integer in the
    /// closed interval `[min(start.index(), end.index()),
    /// max(start.index(), end.index())]` and therefore can never overflow
    /// the valid range. Out-of-range external callers (e.g. fuzz tests)
    /// get the closest variant rather than a panic or error.
    pub const fn from_index_saturating(i: usize) -> Self {
        match i {
            0 => Self::Fastest,
            1 => Self::FasterStill,
            2 => Self::Faster,
            3 => Self::Fast,
            4 => Self::Standard,
            5 => Self::Slow,
            6 => Self::Slower,
            7 => Self::SlowerStill,
            _ => Self::Slowest,
        }
    }
}

// The default cannot be derived: the desired default is `Standard`, which is
// not the first variant. (Derived `Default` would pick `Fastest`.)
#[allow(clippy::derivable_impls)]
impl Default for TrillWiggleSpeed {
    fn default() -> Self {
        Self::Standard
    }
}

/// Ergonomic options bundle for the speed-variant trill API.
///
/// The all-or-nothing `trill_with_extension_speed(speed)` ScoreBuilder method
/// hardcodes the ornament glyph to [`Ornament::Trill`] — so a caller wanting a
/// compound *trill-with-mordent* at a non-standard wiggle speed has no API to
/// reach the combination without hand-constructing annotations. This struct
/// lets callers express both knobs together: required speed + optional
/// ornament override. `None` ornament selects [`Ornament::Trill`] at the
/// score-builder layer.
///
/// Construction is fluent — start with [`TrillExtensionSpeedOptions::new`] (or
/// `TrillWiggleSpeed::into()`) and chain the with-method for the ornament if
/// you need it:
///
/// ```no_run
/// use music_engraver::layout::ornament::Ornament;
/// use music_engraver::layout::trill_extension::{TrillExtensionSpeedOptions, TrillWiggleSpeed};
///
/// // Default ornament (plain "tr"), Fast wiggle.
/// let _ = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast);
///
/// // Precomposed trill-with-mordent, Slow wiggle.
/// let _ = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
///     .with_ornament(Ornament::TrillWithMordent);
///
/// // Ergonomic conversion from a bare speed.
/// let _: TrillExtensionSpeedOptions = TrillWiggleSpeed::Standard.into();
/// ```
///
/// `None` for the ornament field means "the score-builder picks the
/// conventional default ([`Ornament::Trill`]) at attachment time," and the
/// resulting SVG is byte-identical to the existing non-options
/// `trill_with_extension_speed(speed)` API. Callers who want to lock the
/// default ornament in explicitly should pass `Some(Ornament::Trill)`.
///
/// The ornament must satisfy [`Ornament::supports_trill_extension`] — passing
/// an unsupported ornament (e.g. [`Ornament::ShortTrill`], a turn, a mordent)
/// makes the renderer's collector skip the extension entirely (no wiggle), and
/// only the ornament glyph itself is drawn. This matches
/// [`crate::layout::trill_bracket::TrillBracketOptions::ornament`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrillExtensionSpeedOptions {
    /// Wiggle speed/density variant — always required. There is no
    /// "default speed" hidden in the options because the existing
    /// `trill_with_extension()` method already covers the standard-speed
    /// case without any options at all.
    pub speed: TrillWiggleSpeed,
    /// Ornament glyph override. `None` selects [`Ornament::Trill`] (the
    /// conventional "tr" mark). `Some(Ornament::TrillWithMordent)` selects
    /// the precomposed compound. The ornament must satisfy
    /// [`Ornament::supports_trill_extension`]; otherwise the renderer will
    /// silently skip the extension.
    pub ornament: Option<Ornament>,
    /// Optional explicit termination length for the wavy line, in staff
    /// spaces. `None` (the default) lets the wiggle extend per the usual
    /// convention (next-note left edge within-system; system right edge
    /// cross-system). `Some(length_ss)` clamps the wiggle to terminate no
    /// later than `length_ss` staff spaces past its natural start, mirroring
    /// the contract of
    /// [`crate::score::ScoreBuilder::trill_with_extension_length_ss`].
    /// Clamping is one-sided (overruns clamp to the natural span;
    /// non-positive values produce no wiggle). A positive explicit length
    /// disables cross-system propagation — the wiggle terminates within its
    /// source system. Named symmetrically with
    /// [`crate::layout::trill_bracket::TrillBracketOptions::extension_length_ss`]
    /// so widening to
    /// [`crate::layout::trill_options::TrillExtensionFullOptions`] is a
    /// mechanical field-by-field copy.
    pub extension_length_ss: Option<f64>,
}

impl TrillExtensionSpeedOptions {
    /// Construct options at the given wiggle speed with every other knob at
    /// score-builder defaults (plain `Trill` ornament, natural-span
    /// extension).
    pub const fn new(speed: TrillWiggleSpeed) -> Self {
        Self {
            speed,
            ornament: None,
            extension_length_ss: None,
        }
    }

    /// Override the ornament glyph. Pass [`Ornament::TrillWithMordent`] for a
    /// compound precomposed trill at the chosen speed; any ornament that
    /// satisfies [`Ornament::supports_trill_extension`] is valid. Passing an
    /// unsupported ornament makes the renderer drop the entire trill
    /// extension (no wiggle — only the ornament glyph).
    ///
    /// This setter is permissive — any [`Ornament`] is stored as-is so the
    /// options bundle can travel through annotation pipelines whose validity
    /// is only checked at the renderer's collector. Callers wanting
    /// construction-time rejection of ornaments the renderer would silently
    /// drop should use
    /// [`with_ornament_validated`](Self::with_ornament_validated), which
    /// returns `Option<Self>` and rejects the exact set
    /// `!Ornament::supports_trill_extension()`.
    pub const fn with_ornament(mut self, ornament: Ornament) -> Self {
        self.ornament = Some(ornament);
        self
    }

    /// Stricter counterpart to [`with_ornament`](Self::with_ornament):
    /// rejects ornaments that do not satisfy
    /// [`Ornament::supports_trill_extension`] at the options-bundle
    /// construction site, returning `Option<Self>`.
    ///
    /// Currently `Ornament::Trill` and `Ornament::TrillWithMordent` are the
    /// only two variants that satisfy the predicate; every other variant
    /// (mordents, turns, the historical/precomposed family, `ShortTrill`)
    /// returns `None`. The accept set is locked to the predicate, not
    /// hardcoded, so a future expansion of `supports_trill_extension`
    /// automatically widens this method's accept band.
    ///
    /// Rejection rules (`None` returned):
    /// - `ornament` is any variant for which
    ///   [`Ornament::supports_trill_extension`] returns `false`.
    ///
    /// Acceptance (`Some(self)` returned with `ornament` populated):
    /// - `ornament` is any variant for which
    ///   [`Ornament::supports_trill_extension`] returns `true`.
    /// - All other fields on `self` are preserved unchanged (additive
    ///   contract, matching the permissive setter).
    ///
    /// `const`-callable, matching every other setter on this bundle. On the
    /// `None` branch the builder chain is broken at the call site and the
    /// partially-built bundle is dropped — there is no fallback that
    /// silently leaves `ornament` unset, because that would demote a
    /// rejection into a no-op.
    ///
    /// Mirrors the validator-pairing pattern at
    /// [`crate::layout::trill_bracket::TrillBracketOptions::with_ornament_validated`]
    /// and [`crate::layout::trill_options::TrillExtensionFullOptions::with_ornament_validated`]
    /// — all three options bundles expose the same `with_ornament` /
    /// `with_ornament_validated` pair.
    pub const fn with_ornament_validated(mut self, ornament: Ornament) -> Option<Self> {
        // The predicate is the single source of truth for the accept band —
        // any future broadening of `supports_trill_extension` automatically
        // widens this method's accept band without code changes here.
        if ornament.supports_trill_extension() {
            self.ornament = Some(ornament);
            Some(self)
        } else {
            None
        }
    }

    /// Set an explicit termination length for the wavy line, in staff
    /// spaces, measured from the wiggle's natural start (past the ornament
    /// glyph and its trailing padding). Overruns are clamped to the natural
    /// span; a non-positive length suppresses the wiggle entirely. A
    /// positive length disables cross-system propagation.
    ///
    /// Mirrors the contract of
    /// [`crate::score::ScoreBuilder::trill_with_extension_length_ss`] and is
    /// independent of the speed knob — combining
    /// `.with_extension_length_ss(L)` with a non-standard speed clamps a
    /// fast/slow wiggle to the requested length.
    ///
    /// This setter is permissive — any `f64` is stored as-is so the options
    /// bundle can travel through annotation pipelines whose validity is only
    /// checked at draw time (the renderer's fail-safe collapses end_x to
    /// start_x for non-positive lengths, silently producing no wiggle).
    /// Callers wanting construction-time rejection of values the renderer
    /// would silently suppress should use
    /// [`with_extension_length_ss_validated`](Self::with_extension_length_ss_validated),
    /// which returns `Option<Self>` and rejects exactly the band
    /// `!(length_ss > 0.0)`.
    pub const fn with_extension_length_ss(mut self, length_ss: f64) -> Self {
        self.extension_length_ss = Some(length_ss);
        self
    }

    /// Stricter counterpart to
    /// [`with_extension_length_ss`](Self::with_extension_length_ss): rejects
    /// at the options-bundle construction site exactly the lengths that the
    /// renderer's fail-safe would silently suppress (no wiggle drawn),
    /// returning `Option<Self>`.
    ///
    /// The accept band mirrors the renderer's "would draw a wiggle" check
    /// in
    /// [`crate::render::system_renderer`](crate::render::system_renderer)
    /// — `length_ss > 0.0`. The reject band is therefore precisely
    /// `length_ss <= 0.0 || length_ss.is_nan()`. `+∞` lands on the accept
    /// band because the renderer accepts it (and clamps it to the natural
    /// span); this preserves byte-equivalence with the permissive setter on
    /// the boundary case and matches the renderer literally.
    ///
    /// Rejection rules (`None` returned):
    /// - `length_ss == 0.0` (positive or negative zero — both fail
    ///   `length_ss > 0.0`).
    /// - `length_ss < 0.0` (any negative finite, including `-∞`).
    /// - `length_ss.is_nan()` (any NaN payload — NaN comparisons return
    ///   false).
    ///
    /// Acceptance (`Some(self)` returned with `extension_length_ss`
    /// populated):
    /// - `length_ss > 0.0` (any finite positive value, plus `+∞`).
    /// - All other fields on `self` (`speed`, `ornament`) are preserved
    ///   unchanged (additive contract, matching the permissive setter
    ///   byte-for-byte).
    ///
    /// `const`-callable, matching every other setter on this bundle. On the
    /// `None` branch the builder chain is broken at the call site and the
    /// partially-built bundle is dropped — there is no fallback that
    /// silently leaves `extension_length_ss` unset, because that would
    /// demote a rejection into a no-op.
    ///
    /// Mirrors the validator-pairing pattern at
    /// [`crate::layout::trill_bracket::TrillBracketOptions::with_extension_length_ss_validated`]
    /// and
    /// [`crate::layout::trill_options::TrillExtensionFullOptions::with_extension_length_ss_validated`]
    /// — all three options bundles expose the same
    /// `with_extension_length_ss` / `with_extension_length_ss_validated`
    /// pair.
    pub const fn with_extension_length_ss_validated(mut self, length_ss: f64) -> Option<Self> {
        // Mirror the renderer's accept band literally — any future change to
        // the renderer's "would draw" check (e.g. tightening to a minimum
        // tile-width) needs to update this clause in lockstep.
        if length_ss > 0.0 {
            self.extension_length_ss = Some(length_ss);
            Some(self)
        } else {
            None
        }
    }
}

impl From<TrillWiggleSpeed> for TrillExtensionSpeedOptions {
    fn from(speed: TrillWiggleSpeed) -> Self {
        Self::new(speed)
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

// ---------------------------------------------------------------------------
// Multi-speed trill extension
// ---------------------------------------------------------------------------

/// A single region of a multi-speed trill wiggle extension.
///
/// Real engraving uses progressively-denser or progressively-sparser wiggle
/// glyphs *within a single sustained trill* to indicate
/// acceleration/deceleration of the trill — a discrete mid-trill speed
/// change. Each region carries its own wiggle [`Glyph`] (typically a
/// `WiggleTrill*` speed variant; nothing about the layout enforces this) and
/// its own per-tile `segment_advance` (different speed variants have
/// different intrinsic widths, so callers must query each variant's advance
/// from the active font and supply it). Regions tile rightward from
/// `start_x` until the next region's `start_x` (or until the overall
/// `end_x` for the final region).
#[derive(Clone, Debug, PartialEq)]
pub struct TrillSpeedRegion {
    /// X-coordinate where this region begins.
    ///
    /// Regions must be sorted strictly non-decreasing by `start_x`. A
    /// zero-width region (`regions[i+1].start_x == regions[i].start_x`)
    /// is legal — it contributes zero tiles and exists only to mark a
    /// transition point — though it is rarely useful in practice.
    pub start_x: f64,
    /// SMuFL wiggle glyph to tile across this region.
    pub glyph: Glyph,
    /// Per-tile advance width in font units for `glyph`. Must be positive
    /// or the whole multi-speed layout is rejected.
    pub segment_advance: f64,
}

/// One placed tile in a multi-speed trill wiggle extension.
#[derive(Clone, Debug, PartialEq)]
pub struct TrillExtensionTile {
    /// X-coordinate of this tile's left edge.
    pub x: f64,
    /// Wiggle glyph for this tile (carried per-tile so consecutive tiles
    /// with different speeds can be rendered without the renderer needing
    /// to re-derive the speed from position).
    pub glyph: Glyph,
    /// This tile's advance width — echoed from its parent region so
    /// callers can compute the right edge without keeping the originating
    /// regions slice alive.
    pub advance: f64,
}

/// Computed tile positions for a multi-speed trill wavy-line extension.
///
/// Tiles within the vec are in left-to-right order. Adjacent tiles may
/// share a glyph (the layout function does not coalesce regions); the
/// renderer caches outlines per unique glyph so the redundancy is cheap.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiSpeedTrillExtensionLayout {
    /// All tiles in left-to-right order.
    pub tiles: Vec<TrillExtensionTile>,
    /// Shared baseline y for every tile — conventionally aligned with the
    /// preceding "tr" glyph's anchor so the wiggle reads as a horizontal
    /// continuation.
    pub y: f64,
}

/// Lay out a multi-speed trill wavy-line extension.
///
/// For each [`TrillSpeedRegion`] in `regions`, tiles whole copies of its
/// glyph from `region.start_x` rightward until either the next region's
/// `start_x` or `end_x` (whichever comes first). Any leftover gap at the
/// end of a region (less than one tile) is left empty — the same
/// fixed-tile convention as [`layout_trill_extension`]. Stretching a
/// wiggle glyph horizontally distorts its shape and visually reads as
/// wrong, so we never do it.
///
/// Returns `None` when any of the following hold:
/// - `regions` is empty.
/// - Any region's `segment_advance` is non-positive (would imply
///   zero-width tiles).
/// - `regions` is not sorted non-decreasing by `start_x` (catches caller
///   bugs early; a strict ordering would also forbid zero-width
///   transition regions which we permit).
/// - The first region's `start_x` exceeds `end_x`.
/// - The total tile count across all regions is zero (no region had room
///   for a single tile).
///
/// The function does not validate that the chosen glyphs are actually
/// `WiggleTrill*` variants — the layout layer is glyph-agnostic. A caller
/// supplying, say, `Glyph::NoteheadBlack` would produce a wiggle line of
/// noteheads; the renderer would happily emit them. That is a renderer
/// integration test's responsibility, not the layout function's.
pub fn layout_trill_extension_multi_speed(
    end_x: f64,
    y: f64,
    regions: &[TrillSpeedRegion],
) -> Option<MultiSpeedTrillExtensionLayout> {
    if regions.is_empty() {
        return None;
    }
    if regions[0].start_x > end_x {
        return None;
    }

    let mut tiles = Vec::new();
    for i in 0..regions.len() {
        let r = &regions[i];
        if r.segment_advance <= 0.0 {
            return None;
        }
        let region_end = if i + 1 < regions.len() {
            regions[i + 1].start_x
        } else {
            end_x
        };
        // Sort violation: the next region begins before this one's
        // start_x. (`region_end == r.start_x` is permitted — zero-width
        // region, contributes no tiles.)
        if region_end < r.start_x {
            return None;
        }
        let span = region_end - r.start_x;
        let count = (span / r.segment_advance).floor() as usize;
        for j in 0..count {
            tiles.push(TrillExtensionTile {
                x: r.start_x + (j as f64) * r.segment_advance,
                glyph: r.glyph,
                advance: r.segment_advance,
            });
        }
    }
    if tiles.is_empty() {
        return None;
    }
    Some(MultiSpeedTrillExtensionLayout { tiles, y })
}

/// X-coordinate of the rightmost edge of the last tile in a multi-speed
/// trill extension layout. Returns `0.0` for an empty layout — same
/// convention as [`trill_extension_right_edge`].
pub fn multi_speed_trill_extension_right_edge(
    layout: &MultiSpeedTrillExtensionLayout,
) -> f64 {
    match layout.tiles.last() {
        Some(last) => last.x + last.advance,
        None => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Speed-ramp synthesizer
// ---------------------------------------------------------------------------

/// Synthesizer for [`TrillSpeedRegion`] sequences expressing common
/// engraving patterns — currently constant-speed and linear-progression
/// (accelerating or decelerating) ramps.
///
/// Hand-constructing a `&[TrillSpeedRegion]` for a typical accelerating
/// trill (Slow → Standard → Fast, evenly distributed across a known span)
/// is mechanical and error-prone: the caller has to compute three start_x
/// values, pick the right intermediate `WiggleTrill*` glyph, and query
/// each glyph's advance from the active font. This enum + its
/// [`synthesize_regions`](Self::synthesize_regions) method does that
/// mechanical work, leaving the caller to express the *musical* intent
/// (start speed, end speed, number of regions) plus the font lookup.
///
/// The synthesizer is deliberately limited to two patterns. Non-linear
/// ramps (exponential, log, step) are rare in real engraving and easily
/// expressed by hand-constructing the regions slice. Adding a
/// `NonLinear(Box<dyn Fn(f64) -> TrillWiggleSpeed>)` variant later would
/// be additive and non-breaking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrillSpeedRamp {
    /// Single speed across the entire span. Emits `region_count` regions
    /// all carrying the same speed — visually identical to a single
    /// region because [`layout_trill_extension_multi_speed`] tiles each
    /// region independently and adjacent same-speed regions form a
    /// continuous run. The N-region form keeps the API uniform with
    /// [`Self::Linear`] so callers can swap variants without recomputing
    /// `region_count`.
    Constant(TrillWiggleSpeed),
    /// Linear progression from `start` (at the leftmost region) to `end`
    /// (at the rightmost region). Per-region speeds are computed by
    /// rounding `start.index() + t * (end.index() - start.index())` to
    /// the nearest integer for `t = i / (region_count - 1)`, then mapped
    /// back to the variant via [`TrillWiggleSpeed::from_index_saturating`].
    ///
    /// `start.index() > end.index()` produces an accelerating ramp
    /// (slower → faster, since lower indices are faster). The reverse
    /// produces a decelerating ramp. `start == end` is degenerate but
    /// permitted; it produces the same output as `Constant(start)` with
    /// the same `region_count`.
    ///
    /// Requires `region_count >= 2` because a single-region "linear
    /// progression" is ill-defined (only one endpoint can be the
    /// region's speed; both endpoints can't be). Callers that want a
    /// degenerate single-region trill should use [`Self::Constant`].
    Linear {
        start: TrillWiggleSpeed,
        end: TrillWiggleSpeed,
    },
}

impl TrillSpeedRamp {
    /// Construct a constant-speed ramp. Equivalent to
    /// `TrillSpeedRamp::Constant(speed)` but `const`-callable.
    pub const fn constant(speed: TrillWiggleSpeed) -> Self {
        Self::Constant(speed)
    }

    /// Construct a linear-progression ramp from `start` to `end`.
    /// Equivalent to `TrillSpeedRamp::Linear { start, end }` but
    /// `const`-callable. See [`Self::Linear`] for the direction
    /// convention.
    pub const fn linear(start: TrillWiggleSpeed, end: TrillWiggleSpeed) -> Self {
        Self::Linear { start, end }
    }

    /// Stricter counterpart to [`Self::linear`]: rejects the degenerate
    /// `start == end` case at construction time, returning `None`.
    ///
    /// The bare [`Self::linear`] constructor (and the public
    /// [`Self::Linear`] variant) accepts `start == end` for backwards
    /// compatibility — it produces a [`Self::Constant`]-equivalent
    /// output from [`Self::synthesize_regions`]. That permissive contract
    /// makes a `Linear { start: Standard, end: Standard }` syntactically
    /// valid even though it's musically meaningless (a linear progression
    /// with zero delta isn't a progression). Callers wanting compile-time
    /// or run-time confidence that a `Linear` ramp will actually
    /// interpolate between two distinct speeds should construct via this
    /// method and propagate the `None` upward — the call site sees
    /// "degenerate input" at construction rather than discovering it via
    /// surprising output.
    ///
    /// Returns:
    /// - `Some(Linear { start, end })` when `start != end` (the variants
    ///   carry distinct speed indices). Direction (accel vs decel) falls
    ///   out of the ordering exactly as for [`Self::linear`].
    /// - `None` when `start == end`. Callers in this branch should switch
    ///   to [`Self::constant`] or [`Self::Constant`] to express the
    ///   musical intent explicitly.
    ///
    /// Comparison is done via [`TrillWiggleSpeed::index`] (a `const fn`)
    /// so this constructor is itself `const`-callable and can live in
    /// module-level `const` items via `match`-on-`Option` patterns.
    pub const fn linear_validated(
        start: TrillWiggleSpeed,
        end: TrillWiggleSpeed,
    ) -> Option<Self> {
        // `TrillWiggleSpeed` does not implement `const PartialEq` (no
        // such trait exists on stable as of the current MSRV), so we
        // compare through the `index()` accessor — both `index()` calls
        // are `const fn` and `usize == usize` is const-callable.
        if start.index() == end.index() {
            None
        } else {
            Some(Self::Linear { start, end })
        }
    }

    /// Synthesize a sorted slice of [`TrillSpeedRegion`]s evenly
    /// distributed across `[start_x, end_x]`.
    ///
    /// Each region occupies `(end_x - start_x) / region_count` units of
    /// horizontal span. Region `i` (0-indexed) starts at
    /// `start_x + i * region_span`. The final region's right edge is
    /// `end_x` by construction, so the returned slice is ready to feed
    /// directly to [`layout_trill_extension_multi_speed`] with the same
    /// `end_x`.
    ///
    /// `advance_for_speed` is queried once per region with that region's
    /// chosen [`TrillWiggleSpeed`] — callers thread their active
    /// [`crate::font::MusicFont`]'s glyph-advance lookup through the
    /// closure. The synthesizer is font-agnostic.
    ///
    /// Returns `None` when:
    /// - `region_count == 0` (no regions to emit).
    /// - `end_x <= start_x` (zero-width or inverted span).
    /// - `self` is [`Self::Linear`] and `region_count < 2` (linear
    ///   progression is ill-defined for one region).
    pub fn synthesize_regions(
        &self,
        start_x: f64,
        end_x: f64,
        region_count: usize,
        advance_for_speed: impl Fn(TrillWiggleSpeed) -> f64,
    ) -> Option<Vec<TrillSpeedRegion>> {
        if region_count == 0 {
            return None;
        }
        if end_x <= start_x {
            return None;
        }
        if matches!(self, Self::Linear { .. }) && region_count < 2 {
            return None;
        }
        let region_span = (end_x - start_x) / (region_count as f64);
        let mut regions = Vec::with_capacity(region_count);
        for i in 0..region_count {
            let speed = match self {
                Self::Constant(s) => *s,
                Self::Linear { start, end } => {
                    let t = (i as f64) / ((region_count - 1) as f64);
                    let f_idx =
                        (start.index() as f64) + t * (end.index() as f64 - start.index() as f64);
                    // `f_idx` is in [min(start.index(), end.index()),
                    // max(start.index(), end.index())] ⊆ [0, 8], so
                    // round + cast cannot underflow.
                    TrillWiggleSpeed::from_index_saturating(f_idx.round() as usize)
                }
            };
            regions.push(TrillSpeedRegion {
                start_x: start_x + (i as f64) * region_span,
                glyph: speed.to_glyph(),
                segment_advance: advance_for_speed(speed),
            });
        }
        Some(regions)
    }
}

// ---------------------------------------------------------------------------
// Multi-speed trill: ramp + region count "intent spec"
// ---------------------------------------------------------------------------

/// Compact intent-spec pairing a [`TrillSpeedRamp`] with its `region_count`.
///
/// `TrillSpeedRamp::synthesize_regions` requires four inputs: the ramp, a
/// `(start_x, end_x)` span, the `region_count`, and a font-advance lookup.
/// The span and font are pipeline-level data — known only at draw time, when
/// the trill's anchoring note positions and active [`crate::font::MusicFont`]
/// are resolved. The ramp and region count are *caller intent* — known at
/// score-construction time. This struct bundles those two pieces so they can
/// travel together through annotation/options bundles (notably
/// [`crate::layout::trill_options::TrillExtensionFullOptions`]) without the
/// caller having to keep them in lockstep across separate fields.
///
/// Both fields are public for direct destructuring at the draw-time call
/// site; the [`new`](Self::new) constructor is provided for
/// `const`-callable bundle construction.
///
/// The bare [`new`](Self::new) constructor performs no validation —
/// `region_count == 0` and `region_count == 1` for a `Linear` ramp are
/// both *defined* failures in [`TrillSpeedRamp::synthesize_regions`]
/// (returning `None`). `new` stores the raw values; the consumer that
/// calls `synthesize_regions` observes the same `None` it would have for
/// a hand-built call. Callers that want construction-time rejection of
/// those degenerate inputs should use [`new_validated`](Self::new_validated),
/// which returns `None` for the same inputs the synthesizer would reject,
/// mirroring the [`TrillSpeedRamp::linear`] / [`TrillSpeedRamp::linear_validated`]
/// pairing on the ramp itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrillSpeedRampSpec {
    /// The ramp pattern (constant or linear-progression) to synthesize.
    pub ramp: TrillSpeedRamp,
    /// Number of evenly-spaced regions to emit across the trill's span.
    /// Must be `>= 1` for `Constant` and `>= 2` for `Linear` to produce a
    /// non-`None` result from
    /// [`TrillSpeedRamp::synthesize_regions`].
    pub region_count: usize,
}

impl TrillSpeedRampSpec {
    /// Construct a spec from a ramp and a region count. `const`-callable so
    /// canonical specs can live in module-level `const` items, mirroring
    /// [`TrillSpeedRamp::constant`] / [`TrillSpeedRamp::linear`].
    pub const fn new(ramp: TrillSpeedRamp, region_count: usize) -> Self {
        Self {
            ramp,
            region_count,
        }
    }

    /// Stricter counterpart to [`Self::new`]: rejects the degenerate
    /// `(ramp, region_count)` pairs at construction time, returning `None`.
    ///
    /// The bare [`Self::new`] constructor stores any pair unchanged so the
    /// spec can travel through annotation pipelines whose validity is only
    /// checked at draw time — mirroring the permissive policy of
    /// [`TrillSpeedRamp::linear`]. That contract makes a degenerate spec
    /// like `TrillSpeedRampSpec::new(Linear { … }, 1)` syntactically valid
    /// even though its [`TrillSpeedRamp::synthesize_regions`] call will
    /// return `None`. Callers wanting compile-time or run-time confidence
    /// that the spec will actually produce regions should construct via
    /// this method and propagate the `None` upward — the call site sees
    /// "degenerate input" at construction rather than discovering it via
    /// `synthesize_regions` returning `None` later.
    ///
    /// Returns:
    /// - `None` when `region_count == 0` (no regions to emit for *any*
    ///   ramp).
    /// - `None` when `ramp` is [`TrillSpeedRamp::Linear`] and
    ///   `region_count < 2` (a single-region linear progression is
    ///   ill-defined — only one endpoint can be the region's speed, both
    ///   endpoints can't be — and `TrillSpeedRamp::synthesize_regions`
    ///   would itself return `None`).
    /// - `Some(Self { ramp, region_count })` otherwise.
    ///
    /// [`TrillSpeedRamp::Constant`] explicitly accepts any
    /// `region_count >= 1`; a single-region `Constant` spec trivially
    /// renders the chosen speed across the entire span (the synthesizer
    /// behaves identically). Span-related degeneracies
    /// (`end_x <= start_x`, `region_count` exceeding what fits) are
    /// *draw-time* properties — they depend on the trill's anchoring note
    /// positions and are not knowable at spec construction — so they
    /// remain the synthesizer's responsibility.
    ///
    /// This method does NOT additionally reject a `Linear { start, end }`
    /// with `start == end`: that degenerate input is documented as
    /// permitted at [`TrillSpeedRamp::Linear`] (`linear` accepts it; only
    /// [`TrillSpeedRamp::linear_validated`] rejects it). Callers that want
    /// both layers of rejection should chain:
    /// `TrillSpeedRamp::linear_validated(s, e).and_then(|r| TrillSpeedRampSpec::new_validated(r, n))`.
    ///
    /// `const`-callable so canonical validated specs can live in
    /// module-level `const` items via `match`-on-`Option` patterns,
    /// mirroring [`TrillSpeedRamp::linear_validated`].
    pub const fn new_validated(ramp: TrillSpeedRamp, region_count: usize) -> Option<Self> {
        if region_count == 0 {
            return None;
        }
        // `matches!` is const-callable on stable; explicit Linear check
        // mirrors the gate in `TrillSpeedRamp::synthesize_regions`.
        if matches!(ramp, TrillSpeedRamp::Linear { .. }) && region_count < 2 {
            return None;
        }
        Some(Self {
            ramp,
            region_count,
        })
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

    // --- TrillExtensionSpeedOptions ---

    #[test]
    fn speed_options_new_has_required_speed_and_unset_ornament() {
        // `None` on the ornament is meaningful: the score-builder collapses it
        // to `Ornament::Trill` at attachment time, keeping the layout struct
        // glyph-agnostic. Speed is always required.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast);
        assert_eq!(opts.speed, TrillWiggleSpeed::Fast);
        assert_eq!(opts.ornament, None);
    }

    #[test]
    fn speed_options_with_ornament_sets_only_ornament() {
        // Speed must remain unchanged after with_ornament; only the ornament
        // field flips. Catches a regression where with_ornament accidentally
        // reset speed to Default.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_ornament(Ornament::TrillWithMordent);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(
            opts.speed,
            TrillWiggleSpeed::Slow,
            "speed must remain Slow after with_ornament"
        );
    }

    #[test]
    fn speed_options_with_ornament_overwrites_prior_value() {
        // Calling with_ornament twice keeps the last value — covers the case
        // where a caller composes options conditionally.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_ornament(Ornament::Trill)
            .with_ornament(Ornament::TrillWithMordent);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
    }

    #[test]
    fn speed_options_from_speed_matches_new() {
        let from_speed: TrillExtensionSpeedOptions = TrillWiggleSpeed::Faster.into();
        assert_eq!(
            from_speed,
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
        );
    }

    #[test]
    fn speed_options_const_constructible() {
        // The constructor and with-method must be `const`-callable so the
        // common defaults can live in `const` items at module scope. If
        // someone removes `const fn`, this test stops compiling.
        const _OPTS: TrillExtensionSpeedOptions =
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slowest)
                .with_ornament(Ornament::TrillWithMordent);
    }

    #[test]
    fn speed_options_copy_does_not_consume_original() {
        // Options is `Copy`; passing it by value to a function that returns it
        // must leave the original usable. Catches accidental removal of the
        // `Copy` derive.
        fn take(opts: TrillExtensionSpeedOptions) -> TrillExtensionSpeedOptions {
            opts
        }
        let orig = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast)
            .with_ornament(Ornament::TrillWithMordent);
        let _ = take(orig);
        assert_eq!(orig.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(orig.speed, TrillWiggleSpeed::Fast);
    }

    #[test]
    fn speed_options_accepts_unsupported_ornament_at_layout_layer() {
        // The layout struct does not validate the contract — it's the
        // renderer's collector (filtering by `supports_trill_extension()`)
        // that drops the extension for unsupported ornaments. Documenting
        // this at the layout layer: an unsupported ornament must still
        // round-trip through the field unchanged so the renderer can apply
        // the filter consistently.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_ornament(Ornament::ShortTrill);
        assert_eq!(opts.ornament, Some(Ornament::ShortTrill));
        assert!(
            !Ornament::ShortTrill.supports_trill_extension(),
            "ShortTrill must not support the extension"
        );
    }

    #[test]
    fn speed_options_different_speeds_compare_distinct() {
        // The struct derives PartialEq; differing speeds must compare unequal.
        let a = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast);
        let b = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow);
        assert_ne!(a, b);
    }

    #[test]
    fn speed_options_same_speed_different_ornament_compare_distinct() {
        // Two options bundles that differ only in ornament must compare
        // unequal — important for tests that rely on PartialEq to detect
        // accidental field collapse.
        let a = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard);
        let b = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_ornament(Ornament::TrillWithMordent);
        assert_ne!(a, b);
    }

    // --- TrillExtensionSpeedOptions extension length override ---

    #[test]
    fn speed_options_new_has_unset_extension_length_ss() {
        // The newly-added field must start `None` so existing callers (who
        // never touch it) keep their byte-equivalent SVG output and so the
        // `speed_options_default_matches_plain_speed_byte_for_byte` canary
        // continues to hold at the score-integration layer.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast);
        assert_eq!(opts.extension_length_ss, None);
    }

    #[test]
    fn speed_options_with_extension_length_ss_sets_only_extension_length() {
        // Setting the new field must NOT disturb the speed or the ornament.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_extension_length_ss(2.75);
        assert_eq!(opts.extension_length_ss, Some(2.75));
        assert_eq!(opts.speed, TrillWiggleSpeed::Slow);
        assert_eq!(opts.ornament, None);
    }

    #[test]
    fn speed_options_with_extension_length_ss_chains_with_ornament() {
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(1.5);
        assert_eq!(opts.speed, TrillWiggleSpeed::Faster);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.extension_length_ss, Some(1.5));
    }

    #[test]
    fn speed_options_with_extension_length_ss_chain_order_independent() {
        let a = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_extension_length_ss(2.0)
            .with_ornament(Ornament::TrillWithMordent);
        let b = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(2.0);
        assert_eq!(
            a, b,
            "with_extension_length_ss must commute with with_ornament"
        );
    }

    #[test]
    fn speed_options_with_extension_length_ss_overwrites_prior_value() {
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_extension_length_ss(1.0)
            .with_extension_length_ss(7.5);
        assert_eq!(opts.extension_length_ss, Some(7.5));
    }

    #[test]
    fn speed_options_with_extension_length_ss_accepts_non_positive_at_layout_layer() {
        // Mirrors the bracket-options policy: zero/negative round-trip
        // unchanged; the renderer's existing non-positive fail-safe handles
        // the "no wiggle" semantic at draw time.
        let zero = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_extension_length_ss(0.0);
        assert_eq!(zero.extension_length_ss, Some(0.0));
        let neg = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_extension_length_ss(-2.0);
        assert_eq!(neg.extension_length_ss, Some(-2.0));
    }

    #[test]
    fn speed_options_with_extension_length_ss_const_constructible() {
        // The new setter must remain `const`-callable so canonical bundles
        // can live in module-level `const` items. Compile-time canary if
        // someone ever drops `const fn`.
        const _OPTS: TrillExtensionSpeedOptions =
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slowest)
                .with_ornament(Ornament::TrillWithMordent)
                .with_extension_length_ss(3.0);
    }

    #[test]
    fn speed_options_extension_length_distinct_values_compare_distinct() {
        // PartialEq must be sensitive to the new field — catches a future
        // derive forgetting to include it.
        let a = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_extension_length_ss(1.0);
        let b = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_extension_length_ss(2.0);
        assert_ne!(a, b);
    }

    #[test]
    fn speed_options_extension_length_some_zero_distinct_from_none() {
        // `Some(0.0)` vs `None` are semantically different at the renderer
        // layer (zero suppresses wiggle; None lets the natural span flow).
        // PartialEq must keep them distinct.
        let none = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow);
        let zero = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_extension_length_ss(0.0);
        assert_ne!(none, zero);
    }

    // --- TrillExtensionSpeedOptions ornament validator ---

    #[test]
    fn speed_options_with_ornament_validated_accepts_trill() {
        // The canonical accept case: plain Trill is the default-mapped
        // ornament and the most-used variant on this surface.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Fast)
            .with_ornament_validated(Ornament::Trill);
        let opts = opts.expect("Trill must be accepted by the validator");
        assert_eq!(opts.ornament, Some(Ornament::Trill));
        assert_eq!(opts.speed, TrillWiggleSpeed::Fast, "speed must survive");
        assert_eq!(opts.extension_length_ss, None, "extension length unset");
    }

    #[test]
    fn speed_options_with_ornament_validated_accepts_trill_with_mordent() {
        // The other accept variant: the precomposed compound. Pins down the
        // accept band to the full predicate-defined set, not just `Trill`.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_ornament_validated(Ornament::TrillWithMordent);
        let opts = opts.expect("TrillWithMordent must be accepted");
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.speed, TrillWiggleSpeed::Slow);
    }

    #[test]
    fn speed_options_with_ornament_validated_rejects_short_trill() {
        // ShortTrill is the headline rejection case: the wave-less form of
        // the trill mark, and the documented "do not use" for an extension.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_ornament_validated(Ornament::ShortTrill);
        assert!(opts.is_none(), "ShortTrill must be rejected");
    }

    #[test]
    fn speed_options_with_ornament_validated_rejects_every_non_supporting_variant() {
        // Walk all 15 variants and cross-validate `is_none()` iff
        // `!supports_trill_extension()`. Locks the validator's accept band
        // to the predicate's accept band — any future drift trips this.
        for &ornament in Ornament::ALL.iter() {
            let result = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_ornament_validated(ornament);
            assert_eq!(
                result.is_some(),
                ornament.supports_trill_extension(),
                "{ornament:?}: validator accept must match supports_trill_extension"
            );
        }
    }

    #[test]
    fn speed_options_with_ornament_validated_some_branch_byte_equals_unvalidated() {
        // For every accepted ornament, the validated and permissive setters
        // must produce field-by-field equal bundles. Catches a future
        // refactor that started normalizing accepted inputs (e.g. clearing
        // extension_length_ss when adopting a supported ornament).
        for &ornament in Ornament::ALL.iter().filter(|o| o.supports_trill_extension()) {
            let permissive = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
                .with_extension_length_ss(2.5)
                .with_ornament(ornament);
            let validated = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
                .with_extension_length_ss(2.5)
                .with_ornament_validated(ornament)
                .expect("supported ornament must accept");
            assert_eq!(permissive, validated, "{ornament:?}: validated vs permissive byte-equal");
        }
    }

    #[test]
    fn speed_options_with_ornament_validated_preserves_other_setters_on_some() {
        // Chain on top of a populated bundle: every prior field must
        // survive. Locks the additive contract — a future refactor that
        // touched any other field on the accept path would fail here.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slowest)
            .with_extension_length_ss(4.25)
            .with_ornament_validated(Ornament::TrillWithMordent)
            .expect("supported ornament must accept");
        assert_eq!(opts.speed, TrillWiggleSpeed::Slowest, "speed survives");
        assert_eq!(
            opts.extension_length_ss,
            Some(4.25),
            "extension length survives"
        );
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
    }

    #[test]
    fn speed_options_with_ornament_validated_is_const_callable() {
        // The setter must be `const`-callable so accepted bundles can live
        // in `const` items and rejection branches can be evaluated at
        // compile time. `const` items hold one Some and two Nones across
        // distinct rejected ornaments to walk both control-flow branches.
        const ACCEPTED: Option<TrillExtensionSpeedOptions> =
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_ornament_validated(Ornament::Trill);
        const REJECTED_SHORT_TRILL: Option<TrillExtensionSpeedOptions> =
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_ornament_validated(Ornament::ShortTrill);
        const REJECTED_TURN: Option<TrillExtensionSpeedOptions> =
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_ornament_validated(Ornament::Turn);
        assert!(ACCEPTED.is_some());
        assert!(REJECTED_SHORT_TRILL.is_none());
        assert!(REJECTED_TURN.is_none());
    }

    #[test]
    fn speed_options_with_ornament_validated_overwrites_prior_value_on_some() {
        // Two consecutive validated calls on the accept band: the last
        // value wins (matches the permissive setter's last-write-wins).
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_ornament_validated(Ornament::Trill)
            .expect("Trill accepts")
            .with_ornament_validated(Ornament::TrillWithMordent)
            .expect("TrillWithMordent accepts");
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
    }

    #[test]
    fn speed_options_with_ornament_validated_some_branch_isolation() {
        // From a fresh `new()`, only `ornament` is populated on the accept
        // branch — extension_length_ss stays None. Locks the validator's
        // "single-field write" contract.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_ornament_validated(Ornament::Trill)
            .expect("Trill accepts");
        assert_eq!(opts.ornament, Some(Ornament::Trill));
        assert_eq!(opts.extension_length_ss, None);
    }

    // --- layout_trill_extension_multi_speed ---

    fn fast() -> TrillSpeedRegion {
        TrillSpeedRegion {
            start_x: 0.0,
            glyph: Glyph::WiggleTrillFast,
            segment_advance: 60.0,
        }
    }

    #[test]
    fn multi_speed_empty_regions_returns_none() {
        assert!(layout_trill_extension_multi_speed(1000.0, 0.0, &[]).is_none());
    }

    #[test]
    fn multi_speed_first_region_start_after_end_x_returns_none() {
        let regions = [TrillSpeedRegion {
            start_x: 500.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 100.0,
        }];
        assert!(layout_trill_extension_multi_speed(100.0, 0.0, &regions).is_none());
    }

    #[test]
    fn multi_speed_zero_advance_returns_none() {
        let regions = [TrillSpeedRegion {
            start_x: 0.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 0.0,
        }];
        assert!(layout_trill_extension_multi_speed(500.0, 0.0, &regions).is_none());
    }

    #[test]
    fn multi_speed_negative_advance_returns_none() {
        let regions = [TrillSpeedRegion {
            start_x: 0.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: -10.0,
        }];
        assert!(layout_trill_extension_multi_speed(500.0, 0.0, &regions).is_none());
    }

    #[test]
    fn multi_speed_zero_advance_in_second_region_returns_none() {
        // The validation must apply to every region, not just the first —
        // a bad advance late in the slice was the easy regression to miss
        // if the early-out only checked regions[0].
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrill,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 200.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 0.0, // bad
            },
        ];
        assert!(layout_trill_extension_multi_speed(500.0, 0.0, &regions).is_none());
    }

    #[test]
    fn multi_speed_out_of_order_regions_returns_none() {
        // regions[1].start_x < regions[0].start_x: invalid ordering.
        let regions = [
            TrillSpeedRegion {
                start_x: 200.0,
                glyph: Glyph::WiggleTrill,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 100.0, // before previous
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
        ];
        assert!(layout_trill_extension_multi_speed(500.0, 0.0, &regions).is_none());
    }

    #[test]
    fn multi_speed_zero_total_tiles_returns_none() {
        // Span too short for even one tile in the only region.
        let regions = [TrillSpeedRegion {
            start_x: 0.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 200.0,
        }];
        assert!(layout_trill_extension_multi_speed(50.0, 0.0, &regions).is_none());
    }

    #[test]
    fn multi_speed_single_region_matches_single_speed_layout() {
        // For one region, the multi-speed layout's tile positions must
        // equal the single-speed layout's segment_xs — keeping the two
        // entry points byte-equivalent on a degenerate input is the
        // critical contract that lets callers migrate to the multi-speed
        // path without changing rendered output.
        let single = layout_trill_extension_with_glyph(
            10.0,
            10.0 + 5.0 * 60.0,
            25.0,
            Glyph::WiggleTrillFast,
            60.0,
        )
        .unwrap();
        let multi = layout_trill_extension_multi_speed(
            10.0 + 5.0 * 60.0,
            25.0,
            &[TrillSpeedRegion {
                start_x: 10.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            }],
        )
        .unwrap();
        assert_eq!(multi.y, single.y);
        assert_eq!(multi.tiles.len(), single.segment_xs.len());
        for (i, tile) in multi.tiles.iter().enumerate() {
            assert!((tile.x - single.segment_xs[i]).abs() < 1e-9);
            assert_eq!(tile.glyph, single.glyph);
            assert_eq!(tile.advance, single.segment_advance);
        }
    }

    #[test]
    fn multi_speed_two_regions_have_correct_tile_glyphs() {
        // Three Fast tiles (0..180) followed by three Slow tiles (180..540).
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 180.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 120.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(540.0, 0.0, &regions).unwrap();
        assert_eq!(layout.tiles.len(), 6, "3 Fast + 3 Slow");
        for tile in &layout.tiles[..3] {
            assert_eq!(tile.glyph, Glyph::WiggleTrillFast);
            assert_eq!(tile.advance, 60.0);
        }
        for tile in &layout.tiles[3..] {
            assert_eq!(tile.glyph, Glyph::WiggleTrillSlow);
            assert_eq!(tile.advance, 120.0);
        }
    }

    #[test]
    fn multi_speed_two_regions_have_correct_tile_positions() {
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 180.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 120.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(540.0, 0.0, &regions).unwrap();
        let xs: Vec<f64> = layout.tiles.iter().map(|t| t.x).collect();
        assert_eq!(xs, vec![0.0, 60.0, 120.0, 180.0, 300.0, 420.0]);
    }

    #[test]
    fn multi_speed_three_regions_accel_pattern() {
        // Slow → Standard → Fast progression: a real engraving use case
        // (gradually-accelerating trill). Each region contributes one tile.
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 120.0,
            },
            TrillSpeedRegion {
                start_x: 120.0,
                glyph: Glyph::WiggleTrill,
                segment_advance: 100.0,
            },
            TrillSpeedRegion {
                start_x: 220.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(280.0, 50.0, &regions).unwrap();
        assert_eq!(layout.tiles.len(), 3);
        assert_eq!(layout.tiles[0].glyph, Glyph::WiggleTrillSlow);
        assert_eq!(layout.tiles[1].glyph, Glyph::WiggleTrill);
        assert_eq!(layout.tiles[2].glyph, Glyph::WiggleTrillFast);
        assert_eq!(layout.y, 50.0);
    }

    #[test]
    fn multi_speed_region_too_short_for_a_tile_contributes_nothing() {
        // Middle region's span (10) is less than its advance (100): that
        // region contributes zero tiles but does NOT abort the layout —
        // the other regions still tile normally.
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 60.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 100.0, // span = 10, < 100
            },
            TrillSpeedRegion {
                start_x: 70.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(190.0, 0.0, &regions).unwrap();
        // 1 Fast tile in region 0, 0 in region 1, 2 in region 2 = 3 total
        assert_eq!(layout.tiles.len(), 3);
        assert_eq!(layout.tiles[0].x, 0.0);
        assert_eq!(layout.tiles[0].glyph, Glyph::WiggleTrillFast);
        assert_eq!(layout.tiles[1].x, 70.0);
        assert_eq!(layout.tiles[1].glyph, Glyph::WiggleTrillFast);
        assert_eq!(layout.tiles[2].x, 130.0);
    }

    #[test]
    fn multi_speed_zero_width_region_is_legal() {
        // regions[i+1].start_x == regions[i].start_x: zero-width region.
        // Contributes no tiles. Must NOT be rejected as a sort violation.
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 100.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(300.0, 0.0, &regions).unwrap();
        // All tiles come from region[1] (Slow) since region[0] has zero span.
        assert_eq!(layout.tiles.len(), 3);
        for tile in &layout.tiles {
            assert_eq!(tile.glyph, Glyph::WiggleTrillSlow);
        }
    }

    #[test]
    fn multi_speed_no_distortion_of_glyph_widths() {
        // Critical correctness canary: tile positions must increment by
        // exactly `region.segment_advance` within a region (NOT by a
        // global average) — stretching a wiggle glyph distorts it
        // visually, which we explicitly forbid in the doc comment.
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 47.5, // intentionally non-round
            },
            TrillSpeedRegion {
                start_x: 200.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 113.7, // also non-round
            },
        ];
        let layout = layout_trill_extension_multi_speed(700.0, 0.0, &regions).unwrap();
        // Region 0: floor(200 / 47.5) = 4 tiles at 0, 47.5, 95.0, 142.5
        // Region 1: floor(500 / 113.7) = 4 tiles at 200, 313.7, 427.4, 541.1
        for window in layout.tiles[..4].windows(2) {
            let dx = window[1].x - window[0].x;
            assert!(
                (dx - 47.5).abs() < 1e-9,
                "Fast region must tile at exact 47.5 stride, saw {dx}"
            );
        }
        for window in layout.tiles[4..].windows(2) {
            let dx = window[1].x - window[0].x;
            assert!(
                (dx - 113.7).abs() < 1e-9,
                "Slow region must tile at exact 113.7 stride, saw {dx}"
            );
        }
    }

    #[test]
    fn multi_speed_right_edge_for_two_region_layout() {
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 180.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 120.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(540.0, 0.0, &regions).unwrap();
        // Last tile at 420, advance 120 -> right edge 540
        assert_eq!(multi_speed_trill_extension_right_edge(&layout), 540.0);
    }

    #[test]
    fn multi_speed_right_edge_uses_last_tiles_advance_not_first() {
        // The right edge formula `last.x + last.advance` MUST use the
        // last tile's advance, not the first region's — a regression
        // that hardcoded `regions[0].segment_advance` would fire here.
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 60.0,
            },
            TrillSpeedRegion {
                start_x: 60.0,
                glyph: Glyph::WiggleTrillSlowest,
                segment_advance: 500.0, // very wide
            },
        ];
        let layout = layout_trill_extension_multi_speed(700.0, 0.0, &regions).unwrap();
        // tiles: Fast at 0 (advance 60), Slowest at 60 (advance 500)
        assert_eq!(layout.tiles.len(), 2);
        assert_eq!(layout.tiles.last().unwrap().advance, 500.0);
        // right edge = 60 + 500 = 560 (NOT 60 + 60 = 120)
        assert_eq!(multi_speed_trill_extension_right_edge(&layout), 560.0);
    }

    #[test]
    fn multi_speed_right_edge_empty_layout_is_zero() {
        let layout = MultiSpeedTrillExtensionLayout {
            tiles: vec![],
            y: 0.0,
        };
        assert_eq!(multi_speed_trill_extension_right_edge(&layout), 0.0);
    }

    #[test]
    fn multi_speed_y_is_preserved_across_regions() {
        let regions = [fast(), {
            let mut r = fast();
            r.start_x = 300.0;
            r.glyph = Glyph::WiggleTrillSlow;
            r.segment_advance = 100.0;
            r
        }];
        let layout = layout_trill_extension_multi_speed(700.0, 271.5, &regions).unwrap();
        assert_eq!(layout.y, 271.5);
        // No per-tile y is stored; tiles share the layout's y.
    }

    #[test]
    fn multi_speed_tiles_strictly_increasing_x() {
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFastest,
                segment_advance: 30.0,
            },
            TrillSpeedRegion {
                start_x: 100.0,
                glyph: Glyph::WiggleTrill,
                segment_advance: 80.0,
            },
            TrillSpeedRegion {
                start_x: 500.0,
                glyph: Glyph::WiggleTrillSlowest,
                segment_advance: 200.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(1200.0, 0.0, &regions).unwrap();
        for window in layout.tiles.windows(2) {
            assert!(
                window[1].x > window[0].x,
                "tiles must be strictly increasing in x: {:?} -> {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn multi_speed_adjacent_regions_with_same_glyph_still_tile() {
        // No coalescing: two adjacent regions with the same glyph and
        // same advance still produce contiguous tiles, just like a single
        // wider region would. Locks in the no-coalescing decision.
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrill,
                segment_advance: 100.0,
            },
            TrillSpeedRegion {
                start_x: 300.0,
                glyph: Glyph::WiggleTrill,
                segment_advance: 100.0,
            },
        ];
        let layout = layout_trill_extension_multi_speed(600.0, 0.0, &regions).unwrap();
        assert_eq!(layout.tiles.len(), 6);
        for tile in &layout.tiles {
            assert_eq!(tile.glyph, Glyph::WiggleTrill);
            assert_eq!(tile.advance, 100.0);
        }
        let xs: Vec<f64> = layout.tiles.iter().map(|t| t.x).collect();
        assert_eq!(xs, vec![0.0, 100.0, 200.0, 300.0, 400.0, 500.0]);
    }

    #[test]
    fn multi_speed_does_not_overflow_end_x() {
        // The right edge of the entire layout must never exceed end_x —
        // even with multiple regions, each independently fixed-tiled.
        let regions = [
            TrillSpeedRegion {
                start_x: 0.0,
                glyph: Glyph::WiggleTrillFast,
                segment_advance: 47.5,
            },
            TrillSpeedRegion {
                start_x: 200.0,
                glyph: Glyph::WiggleTrillSlow,
                segment_advance: 113.7,
            },
        ];
        let end_x = 700.0;
        let layout = layout_trill_extension_multi_speed(end_x, 0.0, &regions).unwrap();
        let right = multi_speed_trill_extension_right_edge(&layout);
        assert!(
            right <= end_x,
            "right edge {right} must not exceed end_x {end_x}"
        );
    }

    // -----------------------------------------------------------------
    // TrillWiggleSpeed::index / from_index_saturating
    // -----------------------------------------------------------------

    #[test]
    fn index_canonical_order() {
        // Lock in the Fastest=0..Slowest=8 mapping. Any reorder of the
        // enum variants without a corresponding `index()` update would
        // fire here.
        assert_eq!(TrillWiggleSpeed::Fastest.index(), 0);
        assert_eq!(TrillWiggleSpeed::FasterStill.index(), 1);
        assert_eq!(TrillWiggleSpeed::Faster.index(), 2);
        assert_eq!(TrillWiggleSpeed::Fast.index(), 3);
        assert_eq!(TrillWiggleSpeed::Standard.index(), 4);
        assert_eq!(TrillWiggleSpeed::Slow.index(), 5);
        assert_eq!(TrillWiggleSpeed::Slower.index(), 6);
        assert_eq!(TrillWiggleSpeed::SlowerStill.index(), 7);
        assert_eq!(TrillWiggleSpeed::Slowest.index(), 8);
    }

    #[test]
    fn index_matches_position_in_all_array() {
        // The total-order contract: index() must agree with ALL's order.
        // A future regression that reordered ALL but forgot to update
        // index() (or vice-versa) would fire here.
        for (pos, v) in TrillWiggleSpeed::ALL.iter().enumerate() {
            assert_eq!(v.index(), pos, "ALL[{pos}] = {v:?} but index() = {}", v.index());
        }
    }

    #[test]
    fn index_round_trip_through_from_index_saturating() {
        // For every variant, from_index_saturating(v.index()) == v.
        // Locks in the bijection across the valid range.
        for v in TrillWiggleSpeed::ALL {
            assert_eq!(TrillWiggleSpeed::from_index_saturating(v.index()), v);
        }
    }

    #[test]
    fn from_index_saturating_clamps_high_values_to_slowest() {
        // Out-of-range indices saturate to Slowest, not panic. The
        // saturation point is exactly 8: anything ≥9 is Slowest.
        assert_eq!(TrillWiggleSpeed::from_index_saturating(9), TrillWiggleSpeed::Slowest);
        assert_eq!(TrillWiggleSpeed::from_index_saturating(100), TrillWiggleSpeed::Slowest);
        assert_eq!(
            TrillWiggleSpeed::from_index_saturating(usize::MAX),
            TrillWiggleSpeed::Slowest,
        );
    }

    #[test]
    fn from_index_saturating_zero_is_fastest() {
        // Pin down the lowest index → Fastest mapping at the boundary.
        assert_eq!(TrillWiggleSpeed::from_index_saturating(0), TrillWiggleSpeed::Fastest);
    }

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn index_and_from_index_saturating_are_const_callable() {
        // Compile-fail canary: if a future refactor removed `const fn`
        // from either method, these const items would fail to compile.
        const FAST_INDEX: usize = TrillWiggleSpeed::Fast.index();
        const STANDARD_FROM_IDX: TrillWiggleSpeed =
            TrillWiggleSpeed::from_index_saturating(4);
        assert_eq!(FAST_INDEX, 3);
        assert!(matches!(STANDARD_FROM_IDX, TrillWiggleSpeed::Standard));
    }

    // -----------------------------------------------------------------
    // TrillSpeedRamp constructors
    // -----------------------------------------------------------------

    #[test]
    fn ramp_constant_constructor_round_trips() {
        let r = TrillSpeedRamp::constant(TrillWiggleSpeed::Fast);
        assert_eq!(r, TrillSpeedRamp::Constant(TrillWiggleSpeed::Fast));
    }

    #[test]
    fn ramp_linear_constructor_round_trips() {
        let r = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        assert_eq!(
            r,
            TrillSpeedRamp::Linear {
                start: TrillWiggleSpeed::Slow,
                end: TrillWiggleSpeed::Fast,
            },
        );
    }

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn ramp_constructors_are_const_callable() {
        // Compile-fail canary on `const fn` for both constructors.
        const C: TrillSpeedRamp = TrillSpeedRamp::constant(TrillWiggleSpeed::Standard);
        const L: TrillSpeedRamp =
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        assert!(matches!(C, TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard)));
        assert!(matches!(
            L,
            TrillSpeedRamp::Linear {
                start: TrillWiggleSpeed::Slow,
                end: TrillWiggleSpeed::Fast,
            }
        ));
    }

    // -----------------------------------------------------------------
    // TrillSpeedRamp::linear_validated — strict constructor
    // -----------------------------------------------------------------

    #[test]
    fn linear_validated_accepts_distinct_speeds_accel_direction() {
        // Slow (idx 5) -> Fast (idx 3): distinct, accelerating. The
        // returned variant must be Linear with the exact speed values
        // round-tripped — same byte-content as `linear(...)`.
        let r = TrillSpeedRamp::linear_validated(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        assert_eq!(
            r,
            Some(TrillSpeedRamp::Linear {
                start: TrillWiggleSpeed::Slow,
                end: TrillWiggleSpeed::Fast,
            })
        );
    }

    #[test]
    fn linear_validated_accepts_distinct_speeds_decel_direction() {
        // Direction symmetry: Fast -> Slow (decel) is also accepted.
        // A regression that only accepted accel inputs (e.g. by checking
        // `start.index() < end.index()` instead of `!=`) would fail here.
        let r = TrillSpeedRamp::linear_validated(TrillWiggleSpeed::Fast, TrillWiggleSpeed::Slow);
        assert_eq!(
            r,
            Some(TrillSpeedRamp::Linear {
                start: TrillWiggleSpeed::Fast,
                end: TrillWiggleSpeed::Slow,
            })
        );
    }

    #[test]
    fn linear_validated_rejects_equal_speeds_for_every_variant() {
        // Walks all 9 canonical speed variants; `linear_validated(v, v)`
        // must return None for each one. A regression that hardcoded the
        // check against a specific variant (e.g. `Standard`) would pass
        // 1/9 tests and fail 8/9 — this test catches all 9 in one shot,
        // pinpointing the regression as "validation does not apply to
        // every variant" rather than "validation works for variant X".
        for speed in TrillWiggleSpeed::ALL {
            let r = TrillSpeedRamp::linear_validated(speed, speed);
            assert_eq!(
                r, None,
                "linear_validated must reject Linear {{ start: {:?}, end: {:?} }}",
                speed, speed
            );
        }
    }

    #[test]
    fn linear_validated_some_branch_byte_equals_linear_constructor() {
        // For every (start, end) pair with start != end, the Some-branch
        // result must be byte-identical to `linear(start, end)`. Locks in
        // the contract that the validated constructor only filters — it
        // never massages the field values. A regression that, say,
        // sorted the variants into accel order would change the field
        // ordering and fire this test.
        let pairs = [
            (TrillWiggleSpeed::Fastest, TrillWiggleSpeed::Slowest),
            (TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            (TrillWiggleSpeed::Standard, TrillWiggleSpeed::Slower),
            (TrillWiggleSpeed::Faster, TrillWiggleSpeed::SlowerStill),
        ];
        for (s, e) in pairs {
            let validated = TrillSpeedRamp::linear_validated(s, e)
                .expect("distinct speeds must validate");
            let permissive = TrillSpeedRamp::linear(s, e);
            assert_eq!(validated, permissive);
        }
    }

    #[test]
    fn linear_validated_endpoint_speeds_extreme_pair_accepted() {
        // Fastest (idx 0) -> Slowest (idx 8): the maximum-delta pair.
        // Both endpoints sit at the boundary of the valid index range.
        // A regression that ever did `start.index() > 0 && end.index() <
        // 8` (a too-narrow validity check) would reject this.
        let r =
            TrillSpeedRamp::linear_validated(TrillWiggleSpeed::Fastest, TrillWiggleSpeed::Slowest);
        assert_eq!(
            r,
            Some(TrillSpeedRamp::Linear {
                start: TrillWiggleSpeed::Fastest,
                end: TrillWiggleSpeed::Slowest,
            })
        );
    }

    #[test]
    fn linear_validated_minimal_distinct_pair_accepted() {
        // Adjacent variants (idx 4 vs idx 5) — the smallest possible
        // non-zero delta. Catches a regression where the validation
        // check accidentally required a minimum delta (e.g.
        // `(a.index() as i32 - b.index() as i32).abs() >= 2`).
        let r =
            TrillSpeedRamp::linear_validated(TrillWiggleSpeed::Standard, TrillWiggleSpeed::Slow);
        assert_eq!(
            r,
            Some(TrillSpeedRamp::Linear {
                start: TrillWiggleSpeed::Standard,
                end: TrillWiggleSpeed::Slow,
            })
        );
    }

    #[test]
    fn linear_validated_is_const_callable() {
        // Compile-fail canary on `const fn`. Two const items: one for
        // each branch (Some / None). If a future refactor accidentally
        // dropped the `const` qualifier, both lines would fail to
        // compile.
        const SOME_RAMP: Option<TrillSpeedRamp> =
            TrillSpeedRamp::linear_validated(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        const NONE_RAMP: Option<TrillSpeedRamp> = TrillSpeedRamp::linear_validated(
            TrillWiggleSpeed::Standard,
            TrillWiggleSpeed::Standard,
        );
        assert!(matches!(
            SOME_RAMP,
            Some(TrillSpeedRamp::Linear {
                start: TrillWiggleSpeed::Slow,
                end: TrillWiggleSpeed::Fast,
            })
        ));
        assert!(NONE_RAMP.is_none());
    }

    #[test]
    fn linear_validated_some_branch_feeds_synthesize_regions() {
        // End-to-end contract: a validated ramp passes through
        // `synthesize_regions` to produce a non-empty region slice.
        // Catches a regression where the validated branch produces a
        // structurally-valid `Self::Linear` but with field values that
        // somehow trip a downstream check in the synthesizer.
        let ramp =
            TrillSpeedRamp::linear_validated(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast)
                .expect("distinct speeds must validate");
        let regions = ramp
            .synthesize_regions(0.0, 240.0, 3, |_| 60.0)
            .expect("validated linear ramp with N>=2 must synthesize");
        assert_eq!(regions.len(), 3);
        // The first region carries the start speed's glyph and the last
        // region carries the end speed's glyph — locks in that the
        // validated ramp threads start/end through unchanged.
        assert_eq!(regions[0].glyph, TrillWiggleSpeed::Slow.to_glyph());
        assert_eq!(regions[2].glyph, TrillWiggleSpeed::Fast.to_glyph());
        // The middle region's glyph differs from both endpoints —
        // proves real interpolation happened. Standard sits at idx 4,
        // halfway between Slow (5) and Fast (3).
        assert_eq!(regions[1].glyph, TrillWiggleSpeed::Standard.to_glyph());
    }

    #[test]
    fn linear_validated_uses_index_not_pointer_equality() {
        // The validation compares speed *values* (via index()), not
        // memory addresses or any other identity. Two independently
        // constructed `TrillWiggleSpeed::Standard` values — one stored
        // in a local binding, one passed as a literal — must both
        // trigger the None branch. Catches a hypothetical regression
        // that introduced reference-based comparison (impossible here
        // because `TrillWiggleSpeed` is `Copy`, but the test pins the
        // semantic contract regardless).
        let s = TrillWiggleSpeed::Standard;
        let r = TrillSpeedRamp::linear_validated(s, TrillWiggleSpeed::Standard);
        assert_eq!(r, None);
    }

    // -----------------------------------------------------------------
    // TrillSpeedRamp::synthesize_regions — error cases
    // -----------------------------------------------------------------

    #[test]
    fn ramp_constant_zero_region_count_returns_none() {
        let r = TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard);
        assert!(r.synthesize_regions(0.0, 100.0, 0, |_| 80.0).is_none());
    }

    #[test]
    fn ramp_linear_zero_region_count_returns_none() {
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        assert!(r.synthesize_regions(0.0, 100.0, 0, |_| 80.0).is_none());
    }

    #[test]
    fn ramp_constant_inverted_x_returns_none() {
        // end_x < start_x.
        let r = TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard);
        assert!(r.synthesize_regions(200.0, 100.0, 3, |_| 80.0).is_none());
    }

    #[test]
    fn ramp_linear_inverted_x_returns_none() {
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        assert!(r.synthesize_regions(200.0, 100.0, 3, |_| 80.0).is_none());
    }

    #[test]
    fn ramp_zero_width_span_returns_none() {
        // end_x == start_x — a zero-width span produces zero-width
        // regions, which is rejected just like any other degenerate
        // input.
        let constant = TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard);
        let linear = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        assert!(constant.synthesize_regions(100.0, 100.0, 3, |_| 80.0).is_none());
        assert!(linear.synthesize_regions(100.0, 100.0, 3, |_| 80.0).is_none());
    }

    #[test]
    fn ramp_linear_single_region_returns_none() {
        // A single-region linear progression is ill-defined.
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        assert!(r.synthesize_regions(0.0, 100.0, 1, |_| 80.0).is_none());
    }

    #[test]
    fn ramp_constant_single_region_emits_one_region() {
        // Constant explicitly permits region_count == 1.
        let r = TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard);
        let regions = r
            .synthesize_regions(50.0, 250.0, 1, |_| 80.0)
            .expect("constant single region is valid");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].start_x, 50.0);
        assert_eq!(regions[0].glyph, Glyph::WiggleTrill);
        assert_eq!(regions[0].segment_advance, 80.0);
    }

    // -----------------------------------------------------------------
    // TrillSpeedRamp::synthesize_regions — Constant variant
    // -----------------------------------------------------------------

    #[test]
    fn ramp_constant_emits_n_regions_with_same_glyph() {
        // 5 regions, all same glyph & advance.
        let r = TrillSpeedRamp::Constant(TrillWiggleSpeed::Fast);
        let regions = r
            .synthesize_regions(0.0, 100.0, 5, |s| {
                // Sanity: the callback should be invoked with the same
                // speed for every region in the Constant case.
                assert_eq!(s, TrillWiggleSpeed::Fast);
                40.0
            })
            .unwrap();
        assert_eq!(regions.len(), 5);
        for region in &regions {
            assert_eq!(region.glyph, Glyph::WiggleTrillFast);
            assert_eq!(region.segment_advance, 40.0);
        }
    }

    #[test]
    fn ramp_constant_emits_evenly_spaced_start_xs() {
        // Span [0, 100] divided into 5 regions → start_x = 0,20,40,60,80.
        let r = TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard);
        let regions = r.synthesize_regions(0.0, 100.0, 5, |_| 30.0).unwrap();
        let xs: Vec<f64> = regions.iter().map(|r| r.start_x).collect();
        assert_eq!(xs, vec![0.0, 20.0, 40.0, 60.0, 80.0]);
    }

    #[test]
    fn ramp_constant_advance_callback_value_is_propagated() {
        // The callback's return is stored verbatim in the region's
        // segment_advance. A regression that hardcoded a wrong value
        // (e.g. always 100.0) would fire.
        let r = TrillSpeedRamp::Constant(TrillWiggleSpeed::Slow);
        let regions = r.synthesize_regions(0.0, 200.0, 3, |_| 137.42).unwrap();
        for region in &regions {
            assert_eq!(region.segment_advance, 137.42);
        }
    }

    #[test]
    fn ramp_constant_with_nonzero_start_x_offsets_regions() {
        // The first region's start_x is the span start (not 0.0).
        let r = TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard);
        let regions = r.synthesize_regions(500.0, 600.0, 4, |_| 25.0).unwrap();
        let xs: Vec<f64> = regions.iter().map(|r| r.start_x).collect();
        assert_eq!(xs, vec![500.0, 525.0, 550.0, 575.0]);
    }

    // -----------------------------------------------------------------
    // TrillSpeedRamp::synthesize_regions — Linear variant
    // -----------------------------------------------------------------

    #[test]
    fn ramp_linear_accel_endpoint_glyphs_match_input() {
        // Slow (index 5) → Fast (index 3), 3 regions: t = 0, 0.5, 1.
        // Indices: 5, 4, 3 — Slow, Standard, Fast.
        // The endpoints must hit *exactly* their input speeds (no
        // rounding error at t=0 or t=1).
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        let regions = r.synthesize_regions(0.0, 90.0, 3, |s| s.index() as f64 + 1.0).unwrap();
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].glyph, Glyph::WiggleTrillSlow);
        assert_eq!(regions[2].glyph, Glyph::WiggleTrillFast);
    }

    #[test]
    fn ramp_linear_accel_middle_region_is_intermediate_speed() {
        // The intermediate region must be a real intermediate — not
        // start, not end. For Slow(5) → Fast(3) at t=0.5, expected
        // speed index is 4 (Standard).
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        let regions = r.synthesize_regions(0.0, 90.0, 3, |_| 30.0).unwrap();
        assert_eq!(regions[1].glyph, Glyph::WiggleTrill); // Standard
    }

    #[test]
    fn ramp_linear_decel_progresses_from_fast_to_slow() {
        // Fast (3) → Slow (5), 3 regions. Should be Fast, Standard, Slow.
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Fast,
            end: TrillWiggleSpeed::Slow,
        };
        let regions = r.synthesize_regions(0.0, 90.0, 3, |_| 30.0).unwrap();
        let glyphs: Vec<Glyph> = regions.iter().map(|r| r.glyph).collect();
        assert_eq!(
            glyphs,
            vec![
                Glyph::WiggleTrillFast,
                Glyph::WiggleTrill,
                Glyph::WiggleTrillSlow,
            ]
        );
    }

    #[test]
    fn ramp_linear_advance_callback_invoked_with_per_region_speed() {
        // Critical correctness canary: the callback must be queried
        // with the *region's* speed, not (say) always the start speed.
        // A regression that mistakenly cached `start` for all regions
        // would fail here because the per-region speed differs.
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        // Returns a distinct advance per speed so a wrong-speed query
        // would store a wrong advance in the region.
        let regions = r
            .synthesize_regions(0.0, 90.0, 3, |s| match s {
                TrillWiggleSpeed::Slow => 100.0,
                TrillWiggleSpeed::Standard => 200.0,
                TrillWiggleSpeed::Fast => 300.0,
                _ => panic!("synthesizer queried unexpected speed {s:?}"),
            })
            .unwrap();
        assert_eq!(regions[0].segment_advance, 100.0);
        assert_eq!(regions[1].segment_advance, 200.0);
        assert_eq!(regions[2].segment_advance, 300.0);
    }

    #[test]
    fn ramp_linear_round_to_nearest_integer_index() {
        // 4 regions for Slow(5) → Faster(2): t = 0, 1/3, 2/3, 1.
        // f_idx = 5, 4.0, 3.0, 2.
        //   (5 + 1/3 * (2 - 5)) = 5 - 1 = 4.0  → round → 4 → Standard
        //   (5 + 2/3 * (2 - 5)) = 5 - 2 = 3.0  → round → 3 → Fast
        // Endpoints hit exactly: Slow at i=0, Faster at i=3.
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Faster,
        };
        let regions = r.synthesize_regions(0.0, 120.0, 4, |_| 30.0).unwrap();
        let indices: Vec<usize> = regions
            .iter()
            .map(|reg| {
                // Reverse the glyph→speed mapping via ALL search.
                TrillWiggleSpeed::ALL
                    .iter()
                    .find(|s| s.to_glyph() == reg.glyph)
                    .unwrap()
                    .index()
            })
            .collect();
        assert_eq!(indices, vec![5, 4, 3, 2]);
    }

    #[test]
    fn ramp_linear_evenly_spaced_start_xs() {
        // Span [0, 120] divided into 4 regions → start_x = 0,30,60,90.
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        let regions = r.synthesize_regions(0.0, 120.0, 4, |_| 30.0).unwrap();
        let xs: Vec<f64> = regions.iter().map(|r| r.start_x).collect();
        assert_eq!(xs, vec![0.0, 30.0, 60.0, 90.0]);
    }

    #[test]
    fn ramp_linear_degenerate_start_equals_end_emits_constant() {
        // Linear { start: Standard, end: Standard } with N=3 produces
        // 3 regions all with Standard speed — same as Constant(Standard).
        let linear = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Standard,
            end: TrillWiggleSpeed::Standard,
        };
        let constant = TrillSpeedRamp::Constant(TrillWiggleSpeed::Standard);
        let r_linear = linear.synthesize_regions(0.0, 90.0, 3, |_| 40.0).unwrap();
        let r_constant = constant.synthesize_regions(0.0, 90.0, 3, |_| 40.0).unwrap();
        assert_eq!(r_linear, r_constant);
    }

    #[test]
    fn ramp_linear_two_regions_emit_exact_endpoints() {
        // The minimum-valid Linear region_count. t = 0 and t = 1, so
        // the two regions must be exactly start and end (no
        // intermediates, no rounding).
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slowest,
            end: TrillWiggleSpeed::Fastest,
        };
        let regions = r.synthesize_regions(0.0, 100.0, 2, |_| 50.0).unwrap();
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].glyph, Glyph::WiggleTrillSlowest);
        assert_eq!(regions[1].glyph, Glyph::WiggleTrillFastest);
    }

    #[test]
    fn ramp_synthesized_regions_feed_into_multi_speed_layout() {
        // End-to-end contract: the synthesizer's output must be
        // accepted by `layout_trill_extension_multi_speed` without
        // additional massaging. A future change to that function's
        // validation rules that broke this contract would fire here.
        let r = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slow,
            end: TrillWiggleSpeed::Fast,
        };
        let advances = |s: TrillWiggleSpeed| match s {
            TrillWiggleSpeed::Slow => 60.0,
            TrillWiggleSpeed::Standard => 40.0,
            TrillWiggleSpeed::Fast => 25.0,
            _ => 30.0,
        };
        let regions = r.synthesize_regions(0.0, 300.0, 3, advances).unwrap();
        let layout = layout_trill_extension_multi_speed(300.0, 0.0, &regions)
            .expect("synthesized regions must produce a valid multi-speed layout");
        // At least one tile per region (each region is 100 wide and
        // every chosen advance ≤ 60 < 100).
        assert!(layout.tiles.len() >= 3);
        // Right edge must not overflow the synthesizer's end_x.
        assert!(multi_speed_trill_extension_right_edge(&layout) <= 300.0);
    }

    #[test]
    fn ramp_linear_sorted_start_xs_satisfies_layout_sort_invariant() {
        // The layout function rejects unsorted regions. Verify the
        // synthesizer produces sorted output for both directions.
        let accel = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Slowest,
            end: TrillWiggleSpeed::Fastest,
        };
        let decel = TrillSpeedRamp::Linear {
            start: TrillWiggleSpeed::Fastest,
            end: TrillWiggleSpeed::Slowest,
        };
        for r in [accel, decel] {
            let regions = r.synthesize_regions(100.0, 700.0, 6, |_| 50.0).unwrap();
            for window in regions.windows(2) {
                assert!(
                    window[1].start_x > window[0].start_x,
                    "regions must be sorted strictly increasing in start_x"
                );
            }
        }
    }

    // -----------------------------------------------------------------
    // TrillSpeedRampSpec::new_validated — strict constructor
    // -----------------------------------------------------------------

    #[test]
    fn spec_new_validated_rejects_zero_region_count_for_constant() {
        // region_count == 0 is the *defined* failure case in
        // `synthesize_regions` for any ramp; the validated constructor
        // surfaces that rejection at spec construction.
        let ramp = TrillSpeedRamp::constant(TrillWiggleSpeed::Standard);
        assert_eq!(TrillSpeedRampSpec::new_validated(ramp, 0), None);
    }

    #[test]
    fn spec_new_validated_rejects_zero_region_count_for_linear() {
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        assert_eq!(TrillSpeedRampSpec::new_validated(ramp, 0), None);
    }

    #[test]
    fn spec_new_validated_rejects_one_region_for_linear() {
        // A single-region linear progression is ill-defined: only one
        // endpoint can land on the region's speed, both endpoints
        // can't. The synthesizer rejects this at draw time; the
        // validated constructor rejects it at spec construction.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        assert_eq!(TrillSpeedRampSpec::new_validated(ramp, 1), None);
    }

    #[test]
    fn spec_new_validated_accepts_one_region_for_constant() {
        // The asymmetry that's the whole point of the method: `Constant`
        // explicitly permits region_count == 1 (a single tile-row run of
        // that speed across the span). This must be accepted.
        let ramp = TrillSpeedRamp::constant(TrillWiggleSpeed::Standard);
        let spec = TrillSpeedRampSpec::new_validated(ramp, 1).expect("Constant + 1 region is valid");
        assert_eq!(spec.ramp, ramp);
        assert_eq!(spec.region_count, 1);
    }

    #[test]
    fn spec_new_validated_accepts_two_regions_for_linear() {
        // The minimum-valid `region_count` for `Linear`.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slowest, TrillWiggleSpeed::Fastest);
        let spec = TrillSpeedRampSpec::new_validated(ramp, 2).expect("Linear + 2 regions is valid");
        assert_eq!(spec.ramp, ramp);
        assert_eq!(spec.region_count, 2);
    }

    #[test]
    fn spec_new_validated_accepts_typical_inputs_for_both_variants() {
        // Walks all 9 canonical speeds + a representative region_count.
        // Every speed × region_count >= 1 must succeed for `Constant`;
        // every distinct speed pair × region_count >= 2 must succeed for
        // `Linear`. Catches a regression that accidentally introduced a
        // narrower acceptance band (e.g. rejecting region_count == 1 for
        // any ramp, which would break the documented Constant carve-out).
        for &speed in &TrillWiggleSpeed::ALL {
            for &n in &[1usize, 2, 3, 7, 32] {
                let spec = TrillSpeedRampSpec::new_validated(
                    TrillSpeedRamp::constant(speed),
                    n,
                );
                assert!(
                    spec.is_some(),
                    "Constant({speed:?}) + region_count={n} must be accepted"
                );
            }
        }
        for &start in &TrillWiggleSpeed::ALL {
            for &end in &TrillWiggleSpeed::ALL {
                if start.index() == end.index() {
                    // Linear { s, s } is documented as permitted at
                    // construction by both `linear` and the spec's
                    // `new_validated` (rejection of degenerate equal
                    // endpoints belongs to `linear_validated`).
                }
                let ramp = TrillSpeedRamp::linear(start, end);
                for &n in &[2usize, 3, 5, 9, 32] {
                    let spec = TrillSpeedRampSpec::new_validated(ramp, n);
                    assert!(
                        spec.is_some(),
                        "Linear {{ start: {start:?}, end: {end:?} }} + region_count={n} must be accepted"
                    );
                }
            }
        }
    }

    #[test]
    fn spec_new_validated_some_branch_byte_equals_new() {
        // On the accept branch the validated constructor must produce a
        // spec field-by-field equal to the bare `new` constructor — the
        // validation is rejection-only, not normalization. PartialEq
        // covers both fields; this assertion catches a regression that
        // sneaks a normalization step into the validated path (e.g.,
        // clamping region_count or rewriting the ramp).
        let ramps = [
            TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
            TrillSpeedRamp::constant(TrillWiggleSpeed::Fastest),
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            TrillSpeedRamp::linear(TrillWiggleSpeed::Fastest, TrillWiggleSpeed::Slowest),
        ];
        // Use minimum-valid region_count per ramp so every pair is
        // accepted by both constructors.
        for &ramp in &ramps {
            let min_n = if matches!(ramp, TrillSpeedRamp::Linear { .. }) {
                2
            } else {
                1
            };
            let bare = TrillSpeedRampSpec::new(ramp, min_n);
            let validated = TrillSpeedRampSpec::new_validated(ramp, min_n)
                .expect("validated must accept minimum-valid pair");
            assert_eq!(bare, validated, "bare and validated must agree on accept");
        }
    }

    #[test]
    fn spec_new_validated_is_const_callable() {
        // Locks in `const fn` on the new constructor. A future change
        // that dropped `const` would break this compile-time canary —
        // matching the contract of `TrillSpeedRamp::linear_validated`.
        const SOME_SPEC: Option<TrillSpeedRampSpec> = TrillSpeedRampSpec::new_validated(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        );
        const NONE_SPEC_ZERO: Option<TrillSpeedRampSpec> = TrillSpeedRampSpec::new_validated(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
            0,
        );
        const NONE_SPEC_LINEAR_ONE: Option<TrillSpeedRampSpec> = TrillSpeedRampSpec::new_validated(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            1,
        );
        assert!(SOME_SPEC.is_some());
        assert_eq!(NONE_SPEC_ZERO, None);
        assert_eq!(NONE_SPEC_LINEAR_ONE, None);
    }

    #[test]
    fn spec_new_validated_some_branch_feeds_synthesize_regions() {
        // End-to-end contract: a spec accepted by `new_validated` must
        // unconditionally produce `Some` from `synthesize_regions` for
        // a non-degenerate span. Locks in the source-of-truth chain:
        //   new_validated rejects iff synthesize_regions would reject
        //   (modulo span/font-related inputs, which the spec does not
        //   own at construction time).
        let spec = TrillSpeedRampSpec::new_validated(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        )
        .unwrap();
        let regions = spec
            .ramp
            .synthesize_regions(0.0, 300.0, spec.region_count, |_| 50.0)
            .expect("validated spec must produce Some at draw time");
        assert_eq!(regions.len(), 3);
        // Spot-check that the synthesizer used the spec's region_count
        // verbatim (no off-by-one).
        assert_eq!(regions[0].start_x, 0.0);
        assert_eq!(regions[1].start_x, 100.0);
        assert_eq!(regions[2].start_x, 200.0);
    }

    #[test]
    fn spec_new_validated_accepts_linear_with_equal_endpoint_speeds() {
        // Documented carve-out: this method does NOT reject
        // `Linear { start, end }` where `start == end` — that rejection
        // belongs to `TrillSpeedRamp::linear_validated`. The two
        // validators compose orthogonally so callers wanting both layers
        // chain them.
        let degenerate_linear = TrillSpeedRamp::linear(
            TrillWiggleSpeed::Standard,
            TrillWiggleSpeed::Standard,
        );
        // region_count >= 2 satisfies the spec's own rule for Linear, so
        // it MUST be accepted here — surfacing the equal-endpoint
        // degeneracy is `linear_validated`'s job.
        let spec = TrillSpeedRampSpec::new_validated(degenerate_linear, 3)
            .expect("Linear { Std, Std } + 3 regions must pass spec validation");
        assert_eq!(spec.ramp, degenerate_linear);
        assert_eq!(spec.region_count, 3);
        // Composition with `linear_validated`: now this should reject.
        assert_eq!(
            TrillSpeedRamp::linear_validated(
                TrillWiggleSpeed::Standard,
                TrillWiggleSpeed::Standard,
            )
            .and_then(|r| TrillSpeedRampSpec::new_validated(r, 3)),
            None,
            "chaining linear_validated + new_validated must reject equal endpoints"
        );
    }

    #[test]
    fn spec_new_validated_rejection_table_matches_synthesize_regions_zero_region() {
        // Cross-check: for every rejection mode the spec validator owns
        // (`region_count == 0`, Linear with `region_count == 1`), the
        // synthesizer would have returned `None` too on a well-formed
        // span. Locks in the "validator is a strict subset of the
        // synthesizer's rejection set" invariant.
        let constant = TrillSpeedRamp::constant(TrillWiggleSpeed::Standard);
        let linear = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);

        // region_count == 0 rejected by both for any ramp.
        assert_eq!(TrillSpeedRampSpec::new_validated(constant, 0), None);
        assert!(constant.synthesize_regions(0.0, 100.0, 0, |_| 50.0).is_none());
        assert_eq!(TrillSpeedRampSpec::new_validated(linear, 0), None);
        assert!(linear.synthesize_regions(0.0, 100.0, 0, |_| 50.0).is_none());

        // region_count == 1 rejected only for Linear by both layers.
        assert!(TrillSpeedRampSpec::new_validated(constant, 1).is_some());
        assert!(constant.synthesize_regions(0.0, 100.0, 1, |_| 50.0).is_some());
        assert_eq!(TrillSpeedRampSpec::new_validated(linear, 1), None);
        assert!(linear.synthesize_regions(0.0, 100.0, 1, |_| 50.0).is_none());
    }

    #[test]
    fn spec_new_validated_does_not_mutate_inputs_on_accept() {
        // A trivial canary against a refactor that promoted `Linear`
        // with `region_count == 1` into `Constant` (a "helpful"
        // normalization). The accept branch must preserve the ramp
        // variant byte-for-byte.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Fast, TrillWiggleSpeed::Slow);
        let spec = TrillSpeedRampSpec::new_validated(ramp, 4).unwrap();
        assert!(matches!(spec.ramp, TrillSpeedRamp::Linear { .. }));
        // Same Linear endpoints survive.
        match spec.ramp {
            TrillSpeedRamp::Linear { start, end } => {
                assert_eq!(start, TrillWiggleSpeed::Fast);
                assert_eq!(end, TrillWiggleSpeed::Slow);
            }
            _ => panic!("ramp variant must survive validation unchanged"),
        }
    }

    // --- TrillExtensionSpeedOptions: with_extension_length_ss_validated ---
    //
    // The validator's accept band mirrors the renderer's "would draw a
    // wiggle" check (`length_ss > 0.0`) in
    // `system_renderer::draw_trill_extensions_for_system`. The reject band
    // is therefore precisely the `Some(_)` non-positive arm that collapses
    // `end_x` to `start_x` (no wiggle). These tests pin the rejection rules
    // and the Some-branch byte-equivalence with the permissive setter.

    #[test]
    fn speed_options_with_extension_length_ss_validated_rejects_zero() {
        // `0.0` is the headline rejection — the renderer's non-positive arm
        // suppresses the wiggle entirely. Construction-time rejection
        // surfaces the misuse rather than silently producing an empty SVG.
        let result = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_extension_length_ss_validated(0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_rejects_negative_zero() {
        // `-0.0 > 0.0` is false. Catches a refactor that used
        // `length_ss.is_sign_negative()` or `length_ss != 0.0` instead.
        let result = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_extension_length_ss_validated(-0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_rejects_negative_finite() {
        // Walks a representative set of negative magnitudes to lock in
        // that the predicate is `> 0.0` rather than a per-magnitude band.
        for len in [-0.001, -0.5, -1.0, -10.0, -1.0e6] {
            let result = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss_validated(len);
            assert_eq!(result, None, "expected rejection for len={len}");
        }
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_rejects_negative_infinity() {
        // `-∞ > 0.0` is false, so `-∞` rejects like any other negative.
        let result = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_extension_length_ss_validated(f64::NEG_INFINITY);
        assert_eq!(result, None);
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_rejects_nan() {
        // NaN > 0.0 is false (NaN comparisons always return false). The
        // renderer suppresses NaN — the validator must match.
        let result = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_extension_length_ss_validated(f64::NAN);
        assert_eq!(result, None);
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_accepts_positive_finite() {
        // The headline accept band: typical positive lengths. Walks small,
        // unit, and large magnitudes.
        for len in [0.001, 0.5, 1.0, 2.75, 1.0e6] {
            let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss_validated(len)
                .unwrap_or_else(|| panic!("expected accept for len={len}"));
            assert_eq!(opts.extension_length_ss, Some(len), "len={len}");
        }
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_accepts_positive_infinity() {
        // `+∞ > 0.0` is true — the renderer accepts `+∞` and clamps the
        // requested end_x to the natural span. The validator mirrors this
        // exactly to preserve byte-equivalence with the permissive setter
        // on the boundary case.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
            .with_extension_length_ss_validated(f64::INFINITY)
            .expect("+infinity must be accepted to mirror the renderer");
        assert_eq!(opts.extension_length_ss, Some(f64::INFINITY));
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_some_byte_equals_permissive() {
        // On any accepted length, the validator must produce a bundle
        // field-by-field equal to the permissive setter. PartialEq covers
        // every field.
        for len in [0.001, 0.5, 1.0, 2.75, 1.0e6, f64::INFINITY] {
            let permissive = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss(len);
            let validated = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss_validated(len)
                .unwrap_or_else(|| panic!("expected accept for len={len}"));
            assert_eq!(permissive, validated, "len={len}");
        }
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_preserves_other_setters_on_some() {
        // The validator must be additive: chaining it on top of a bundle
        // with other fields already set leaves those fields intact on the
        // `Some` branch.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss_validated(3.25)
            .expect("3.25 is a valid extension length");
        assert_eq!(opts.speed, TrillWiggleSpeed::Faster);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.extension_length_ss, Some(3.25));
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_overwrites_prior_value_on_some() {
        // Same last-write-wins semantic as the permissive setter when both
        // values are on the accept band.
        let opts = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slow)
            .with_extension_length_ss_validated(1.0)
            .expect("1.0 is valid")
            .with_extension_length_ss_validated(2.5)
            .expect("2.5 is valid");
        assert_eq!(opts.extension_length_ss, Some(2.5));
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_is_const_callable() {
        // `const fn` symmetry: matches every other setter on this bundle.
        const SOME_OPTS: Option<TrillExtensionSpeedOptions> =
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss_validated(1.5);
        const NONE_OPTS: Option<TrillExtensionSpeedOptions> =
            TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss_validated(0.0);
        assert!(SOME_OPTS.is_some());
        assert!(NONE_OPTS.is_none());
    }

    #[test]
    fn speed_options_with_extension_length_ss_validated_rejection_matches_renderer_accept_band() {
        // Cross-validation: for every probe value, the validator's
        // `is_none()` must equal the renderer's "would suppress" predicate
        // (`!(length_ss > 0.0)`). Locks the validator and renderer
        // together — any future drift trips this canary.
        for len in [
            -1.0e6,
            -2.0,
            -0.001,
            -0.0,
            0.0,
            f64::NEG_INFINITY,
            f64::NAN,
            0.001,
            0.5,
            1.0,
            1.0e6,
            f64::INFINITY,
        ] {
            let validator_is_none = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss_validated(len)
                .is_none();
            // Mirrors the renderer's condition verbatim, NaN behavior
            // included: `!(len > 0.0)` is true for NaN, whereas
            // `len <= 0.0` would be false. The NaN case is under test.
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            let renderer_would_suppress = !(len > 0.0);
            assert_eq!(
                validator_is_none, renderer_would_suppress,
                "drift at len={len}: validator_is_none={validator_is_none} \
                 renderer_would_suppress={renderer_would_suppress}"
            );
        }
    }
}
