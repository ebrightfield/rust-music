//! Vertical hook markers ("brackets") at the start and/or end of a trill
//! wavy-line extension.
//!
//! The bracket form of a trill range is a wavy line whose start and/or end
//! is capped with a short vertical line. This makes the exact start and end
//! of the trill unambiguous — useful for sustained trills under tied notes,
//! cross-system trills, or notation in which the trill's range must be made
//! explicit (Behind Bars: "where the duration of a trill must be precisely
//! defined, the wavy line is bracketed at one or both ends").
//!
//! The hook is a single thin vertical line at the wiggle's terminating x,
//! drawn either downward from the wiggle baseline (the conventional default
//! when the wiggle sits above the staff) or upward (when the wiggle sits
//! below the staff or below the notes it decorates). Length is ~0.75 staff
//! spaces by tradition; the caller passes it explicitly so this stays
//! font-/style-agnostic.
//!
//! Layout produces pure geometry. The renderer
//! (`crate::render::trill_bracket_renderer`) emits one `<line>` per hook.

use crate::layout::ornament::Ornament;
use crate::layout::trill_extension::{
    multi_speed_trill_extension_right_edge, trill_extension_right_edge,
    MultiSpeedTrillExtensionLayout, TrillExtensionLayout,
};

/// Which end(s) of a trill wavy-line extension should be capped with a hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrillBracketSide {
    /// Hook only at the wiggle's left edge.
    Start,
    /// Hook only at the wiggle's right edge.
    End,
    /// Hook at both ends — produces a square-bracket-style trill range.
    Both,
}

/// Ergonomic options bundle for the bracket-form trill API.
///
/// The all-or-nothing custom variant of the bracket builder requires the
/// caller to supply both a `HookDirection` and a length even when they only
/// want to override one of them. This struct lets callers express "bracket
/// on this side; for any unspecified knob, fall back to the conventional
/// default" by leaving the optional fields as `None`.
///
/// Construction is fluent — start with [`TrillBracketOptions::new`] (or
/// `TrillBracketSide::into()`) and chain the with-methods for the knobs you
/// care about:
///
/// ```no_run
/// use music_engraver::layout::trill_bracket::{
///     HookDirection, TrillBracketOptions, TrillBracketSide,
/// };
///
/// // Default direction (Down) and default length (~0.75ss), End only.
/// let _ = TrillBracketOptions::new(TrillBracketSide::End);
///
/// // Override length while keeping conventional Down direction.
/// let _ = TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(1.0);
///
/// // Same call, clearer name (non-breaking alias).
/// let _ = TrillBracketOptions::new(TrillBracketSide::Both).with_hook_length_ss(1.0);
///
/// // Override direction while keeping default length.
/// let _ = TrillBracketOptions::new(TrillBracketSide::Start).with_direction(HookDirection::Up);
/// ```
///
/// `None` for either optional field means "the layout/renderer picks the
/// conventional default at draw time," and the resulting SVG is byte-
/// identical to the existing non-custom `trill_with_extension_bracketed`
/// API. Callers who want to lock in the defaults explicitly should pass
/// `Some(HookDirection::Down)` and `Some(0.75)` instead.
///
/// To bracket a compound trill (precomposed "trill + mordent"), set
/// [`Self::ornament`] to `Some(Ornament::TrillWithMordent)`. `None` (the
/// default) selects `Ornament::Trill` at the score-builder layer. The
/// ornament must satisfy [`Ornament::supports_trill_extension`] — passing
/// an unsupported ornament (e.g. [`Ornament::ShortTrill`] or a turn) makes
/// the renderer's collector skip the extension entirely (no wiggle, no
/// bracket), and only the ornament glyph itself is drawn.
///
/// To explicitly terminate the wavy-line extension after a fixed staff-
/// space distance — independent of where the next note falls — set
/// [`Self::extension_length_ss`] via
/// [`with_extension_length_ss`](Self::with_extension_length_ss). The
/// bracket's `End` hook (when present) anchors at the shortened wiggle
/// terminus, so a `Both`-bracketed trill with an explicit length renders as
/// a square-bracket-style range capped at exactly the requested point.
/// `None` (the default) lets the wiggle extend per the usual convention
/// (next-note left edge within-system; system right edge cross-system).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrillBracketOptions {
    /// Which end(s) of the wiggle to bracket — always required.
    pub side: TrillBracketSide,
    /// Hook direction override. `None` defers to the renderer's conventional
    /// default (`HookDirection::Down`).
    pub direction: Option<HookDirection>,
    /// Hook length in staff spaces. `None` defers to the renderer's
    /// conventional default (~0.75ss). Naming note: this is the **hook**
    /// length (the short vertical line capping the wiggle), not the wiggle's
    /// horizontal extension length — see [`Self::extension_length_ss`] for
    /// the latter.
    ///
    /// At read sites where the bare name reads ambiguously next to
    /// `extension_length_ss`, prefer the
    /// [`hook_length_ss()`](Self::hook_length_ss) accessor which returns the
    /// same value under a clearer name. The field itself keeps its original
    /// name to preserve struct-literal construction compatibility.
    pub length_ss: Option<f64>,
    /// Ornament glyph override. `None` selects [`Ornament::Trill`] (the
    /// conventional "tr" mark). `Some(Ornament::TrillWithMordent)` selects
    /// the precomposed compound. The ornament must satisfy
    /// [`Ornament::supports_trill_extension`]; otherwise the renderer will
    /// silently skip the extension and the bracket.
    pub ornament: Option<Ornament>,
    /// Optional explicit termination length for the wavy line, in staff
    /// spaces. `None` (the default) lets the wiggle extend to the next note
    /// (within-system) or to the system's right edge (cross-system) per the
    /// usual convention. `Some(length_ss)` clamps the wiggle to terminate no
    /// later than `length_ss` staff spaces past its natural start, mirroring
    /// the contract of
    /// [`crate::score::ScoreBuilder::trill_with_extension_length_ss`].
    /// Clamping is one-sided (overruns clamp to the natural span;
    /// non-positive values produce no wiggle). A positive explicit length
    /// disables cross-system propagation — the wiggle terminates within its
    /// source system. The `End` bracket hook anchors at the shortened
    /// terminus.
    ///
    /// Renamed away from a bare `length_ss` to avoid collision with the
    /// existing [`Self::length_ss`] field (which is the *hook* length, not
    /// the extension's termination length).
    pub extension_length_ss: Option<f64>,
}

impl TrillBracketOptions {
    /// Construct options bracketing the given side with every knob at
    /// renderer defaults (Down direction, ~0.75ss hook length, plain `Trill`
    /// ornament, natural-span extension).
    pub const fn new(side: TrillBracketSide) -> Self {
        Self {
            side,
            direction: None,
            length_ss: None,
            ornament: None,
            extension_length_ss: None,
        }
    }

    /// Override the hook direction. Pass `HookDirection::Up` for trills
    /// rendered below the staff so the hook still points back toward the
    /// affected notes.
    pub const fn with_direction(mut self, direction: HookDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Override the **hook** length in staff spaces. Reasonable values are
    /// roughly 0.5..=1.0; the layout does not clamp, and a 0.0 length
    /// produces a degenerate hook (no visible line).
    ///
    /// To shorten the *wiggle* (extension) instead, see
    /// [`with_extension_length_ss`](Self::with_extension_length_ss).
    ///
    /// Prefer [`with_hook_length_ss`](Self::with_hook_length_ss) at call
    /// sites where the bare name `length_ss` reads ambiguously next to
    /// `extension_length_ss` — both names write the same field and are
    /// byte-equivalent. The shorter name is retained for backwards
    /// compatibility (renaming the public method would be breaking).
    pub const fn with_length_ss(mut self, length_ss: f64) -> Self {
        self.length_ss = Some(length_ss);
        self
    }

    /// Non-breaking alias for [`with_length_ss`](Self::with_length_ss) with a
    /// clearer name. Writes to the same [`length_ss`](Self::length_ss) field
    /// (the bracket *hook* length, not the wiggle's horizontal extension
    /// length — see [`with_extension_length_ss`](Self::with_extension_length_ss)).
    ///
    /// The two setters are byte-equivalent; pick whichever reads more
    /// clearly at the call site. Locked into the same field by construction,
    /// so a future refactor that splits them would have to update both
    /// methods together.
    pub const fn with_hook_length_ss(mut self, hook_length_ss: f64) -> Self {
        self.length_ss = Some(hook_length_ss);
        self
    }

    /// Read the bracket *hook* length, equivalent to accessing the
    /// [`length_ss`](Self::length_ss) field directly but named to match
    /// [`with_hook_length_ss`](Self::with_hook_length_ss) — clearer at the
    /// call site when the surrounding code also reads
    /// [`extension_length_ss`](Self::extension_length_ss).
    pub const fn hook_length_ss(&self) -> Option<f64> {
        self.length_ss
    }

    /// Override the ornament glyph. Pass `Ornament::TrillWithMordent` for
    /// a bracketed precomposed compound trill; any other ornament that
    /// satisfies [`Ornament::supports_trill_extension`] (currently only
    /// `Trill` itself or `TrillWithMordent`) is also valid. Passing an
    /// unsupported ornament makes the renderer drop the entire trill
    /// extension (no wiggle, no bracket — only the ornament glyph).
    pub const fn with_ornament(mut self, ornament: Ornament) -> Self {
        self.ornament = Some(ornament);
        self
    }

    /// Set an explicit termination length for the wavy line, in staff
    /// spaces, measured from the wiggle's natural start (past the ornament
    /// glyph and its trailing padding). Overruns are clamped to the natural
    /// span; a non-positive length suppresses the wiggle entirely. A
    /// positive length disables cross-system propagation.
    ///
    /// Mirrors the contract of
    /// [`crate::score::ScoreBuilder::trill_with_extension_length_ss`] and is
    /// independent of every other knob on this bundle — combining
    /// `.with_extension_length_ss(L)` with a bracket anchors the end hook
    /// at the shortened terminus.
    pub const fn with_extension_length_ss(mut self, length_ss: f64) -> Self {
        self.extension_length_ss = Some(length_ss);
        self
    }
}

impl From<TrillBracketSide> for TrillBracketOptions {
    fn from(side: TrillBracketSide) -> Self {
        Self::new(side)
    }
}

/// Whether the hook extends downward from (or upward to) the wiggle baseline.
///
/// The wiggle's `y` is conventionally the baseline of the trill "tr" glyph,
/// which itself sits *above* the staff for trills on notes in or below the
/// staff. So the default cap direction is `Down` — the hook points back
/// toward the staff and reads as "this is where the trill ends." Callers
/// rendering a trill that already sits below the staff (rare) can flip to
/// `Up` so the hook still points back toward the affected notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookDirection {
    /// Hook extends from baseline downward (toward larger SVG y).
    Down,
    /// Hook extends from baseline upward (toward smaller SVG y).
    Up,
}

/// Geometry of a single bracket hook: a thin vertical line.
///
/// `y_top` is always the smaller (numerically lower in SVG y, i.e. higher
/// on the page) of the two y-coordinates and `y_bottom` the larger; this
/// invariant lets renderers and tests reason about extent without re-deriving
/// the direction.
#[derive(Clone, Debug, PartialEq)]
pub struct TrillBracketHookLayout {
    /// X-coordinate of the hook's vertical line.
    pub x: f64,
    /// Smaller (upper, on-page) y endpoint.
    pub y_top: f64,
    /// Larger (lower, on-page) y endpoint.
    pub y_bottom: f64,
    /// Stroke thickness for the line.
    pub stroke_width: f64,
}

/// Build a single hook anchored at `(x, baseline_y)` extending by `length`
/// in the given direction. `length` is the absolute hook length in the same
/// units as `baseline_y` (callers convert from staff spaces to font units
/// before invoking).
///
/// Returns a hook whose `y_top`/`y_bottom` are always ordered with
/// `y_top <= y_bottom`. Length is treated as an absolute magnitude:
/// negative `length` is folded to its absolute value rather than flipping
/// direction silently, so the explicit `HookDirection` always wins.
pub fn layout_trill_bracket_hook(
    x: f64,
    baseline_y: f64,
    length: f64,
    direction: HookDirection,
    stroke_width: f64,
) -> TrillBracketHookLayout {
    let length = length.abs();
    let (y_top, y_bottom) = match direction {
        HookDirection::Down => (baseline_y, baseline_y + length),
        HookDirection::Up => (baseline_y - length, baseline_y),
    };
    TrillBracketHookLayout {
        x,
        y_top,
        y_bottom,
        stroke_width,
    }
}

/// Build the bracket hooks for an existing trill extension.
///
/// Hook x-positions:
/// - `Start` hooks sit at the wiggle's leftmost segment's left edge
///   (`extension.segment_xs[0]`).
/// - `End` hooks sit at the wiggle's right edge as computed by
///   [`trill_extension_right_edge`] (so the hook is flush with the wiggle's
///   visible terminus, not the *start* of the last tile).
///
/// Returns an empty vector when the extension has no segments — a wiggle
/// that could not be laid out cannot meaningfully be bracketed. Length and
/// stroke width are passed through to each generated hook.
pub fn layout_trill_bracket_hooks(
    extension: &TrillExtensionLayout,
    side: TrillBracketSide,
    length: f64,
    direction: HookDirection,
    stroke_width: f64,
) -> Vec<TrillBracketHookLayout> {
    let Some(&first_x) = extension.segment_xs.first() else {
        return Vec::new();
    };
    let right_edge = trill_extension_right_edge(extension);
    let baseline_y = extension.y;

    let mut hooks = Vec::with_capacity(2);
    if matches!(side, TrillBracketSide::Start | TrillBracketSide::Both) {
        hooks.push(layout_trill_bracket_hook(
            first_x,
            baseline_y,
            length,
            direction,
            stroke_width,
        ));
    }
    if matches!(side, TrillBracketSide::End | TrillBracketSide::Both) {
        hooks.push(layout_trill_bracket_hook(
            right_edge,
            baseline_y,
            length,
            direction,
            stroke_width,
        ));
    }
    hooks
}

/// Multi-speed counterpart of [`layout_trill_bracket_hooks`].
///
/// Hook x-positions are anchored to the multi-speed layout's first tile's
/// left edge (Start) and the right edge of its last tile as computed by
/// [`multi_speed_trill_extension_right_edge`] (End). The geometry matches
/// the single-speed function exactly — the only difference is the data
/// shape we read from, since [`MultiSpeedTrillExtensionLayout`] has
/// `tiles: Vec<TrillExtensionTile>` rather than `segment_xs: Vec<f64>`.
///
/// Returns an empty vector when the multi-speed layout has no tiles — a
/// wiggle that could not be laid out cannot meaningfully be bracketed,
/// matching the single-speed function's fail-safe.
pub fn layout_trill_bracket_hooks_multi_speed(
    extension: &MultiSpeedTrillExtensionLayout,
    side: TrillBracketSide,
    length: f64,
    direction: HookDirection,
    stroke_width: f64,
) -> Vec<TrillBracketHookLayout> {
    let Some(first_tile) = extension.tiles.first() else {
        return Vec::new();
    };
    let first_x = first_tile.x;
    let right_edge = multi_speed_trill_extension_right_edge(extension);
    let baseline_y = extension.y;

    let mut hooks = Vec::with_capacity(2);
    if matches!(side, TrillBracketSide::Start | TrillBracketSide::Both) {
        hooks.push(layout_trill_bracket_hook(
            first_x,
            baseline_y,
            length,
            direction,
            stroke_width,
        ));
    }
    if matches!(side, TrillBracketSide::End | TrillBracketSide::Both) {
        hooks.push(layout_trill_bracket_hook(
            right_edge,
            baseline_y,
            length,
            direction,
            stroke_width,
        ));
    }
    hooks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::trill_extension::layout_trill_extension;
    use smufl::Glyph;

    #[test]
    fn single_hook_down_has_baseline_at_top() {
        let h = layout_trill_bracket_hook(100.0, 50.0, 30.0, HookDirection::Down, 4.0);
        assert_eq!(h.x, 100.0);
        assert_eq!(h.y_top, 50.0);
        assert_eq!(h.y_bottom, 80.0);
        assert_eq!(h.stroke_width, 4.0);
    }

    #[test]
    fn single_hook_up_has_baseline_at_bottom() {
        let h = layout_trill_bracket_hook(100.0, 50.0, 30.0, HookDirection::Up, 4.0);
        assert_eq!(h.x, 100.0);
        assert_eq!(h.y_top, 20.0);
        assert_eq!(h.y_bottom, 50.0);
    }

    #[test]
    fn hook_invariant_y_top_le_y_bottom_for_both_directions() {
        let down = layout_trill_bracket_hook(0.0, 0.0, 30.0, HookDirection::Down, 1.0);
        let up = layout_trill_bracket_hook(0.0, 0.0, 30.0, HookDirection::Up, 1.0);
        assert!(down.y_top <= down.y_bottom);
        assert!(up.y_top <= up.y_bottom);
    }

    #[test]
    fn negative_length_treated_as_absolute_value() {
        // Negative length should not silently flip direction; the explicit
        // HookDirection always wins. This guards against caller bugs where
        // a "length" expression accidentally goes negative.
        let h = layout_trill_bracket_hook(0.0, 50.0, -30.0, HookDirection::Down, 1.0);
        assert_eq!(h.y_top, 50.0);
        assert_eq!(h.y_bottom, 80.0);
    }

    #[test]
    fn zero_length_produces_degenerate_hook() {
        // A zero-length hook is still produced (caller's responsibility to
        // filter); both endpoints equal the baseline.
        let h = layout_trill_bracket_hook(50.0, 100.0, 0.0, HookDirection::Down, 1.0);
        assert_eq!(h.y_top, 100.0);
        assert_eq!(h.y_bottom, 100.0);
    }

    #[test]
    fn stroke_width_preserved() {
        let h = layout_trill_bracket_hook(0.0, 0.0, 30.0, HookDirection::Down, 7.5);
        assert!((h.stroke_width - 7.5).abs() < 1e-9);
    }

    fn make_extension() -> TrillExtensionLayout {
        // 3 segments of width 80 starting at x=100, y=50.
        layout_trill_extension(100.0, 100.0 + 3.0 * 80.0, 50.0, 80.0).unwrap()
    }

    #[test]
    fn hooks_start_only_returns_one_hook_at_first_x() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Start, 30.0, HookDirection::Down, 4.0);
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].x, 100.0);
    }

    #[test]
    fn hooks_end_only_returns_one_hook_at_right_edge() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::End, 30.0, HookDirection::Down, 4.0);
        assert_eq!(hooks.len(), 1);
        // 3 segments × 80, starting at 100 → right edge = 340.
        assert_eq!(hooks[0].x, 340.0);
    }

    #[test]
    fn hooks_both_returns_two_hooks() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Both, 30.0, HookDirection::Down, 4.0);
        assert_eq!(hooks.len(), 2);
        // First hook at start, second at right edge.
        assert_eq!(hooks[0].x, 100.0);
        assert_eq!(hooks[1].x, 340.0);
    }

    #[test]
    fn hooks_share_extension_baseline_y() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Both, 30.0, HookDirection::Down, 4.0);
        for h in &hooks {
            // Down hooks: y_top equals the baseline.
            assert_eq!(h.y_top, ext.y);
        }
    }

    #[test]
    fn hooks_up_direction_anchors_baseline_at_bottom() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Both, 30.0, HookDirection::Up, 4.0);
        for h in &hooks {
            assert_eq!(h.y_bottom, ext.y);
            assert_eq!(h.y_top, ext.y - 30.0);
        }
    }

    #[test]
    fn hooks_empty_extension_returns_empty_vec() {
        let ext = TrillExtensionLayout {
            segment_xs: vec![],
            y: 50.0,
            glyph: Glyph::WiggleTrill,
            segment_advance: 80.0,
        };
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Both, 30.0, HookDirection::Down, 4.0);
        assert!(hooks.is_empty());
    }

    #[test]
    fn hooks_single_segment_start_x_equals_extension_start() {
        let ext = layout_trill_extension(50.0, 130.0, 0.0, 80.0).unwrap();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Start, 20.0, HookDirection::Down, 2.0);
        assert_eq!(hooks[0].x, 50.0);
    }

    #[test]
    fn hooks_single_segment_end_x_equals_right_edge() {
        let ext = layout_trill_extension(50.0, 130.0, 0.0, 80.0).unwrap();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::End, 20.0, HookDirection::Down, 2.0);
        // Single segment of width 80 starting at 50 → right edge = 130.
        assert_eq!(hooks[0].x, 130.0);
    }

    #[test]
    fn hooks_length_propagated_into_each_hook() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Both, 42.0, HookDirection::Down, 1.0);
        for h in &hooks {
            assert!((h.y_bottom - h.y_top - 42.0).abs() < 1e-9);
        }
    }

    #[test]
    fn hooks_stroke_width_propagated_into_each_hook() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Both, 30.0, HookDirection::Down, 5.5);
        for h in &hooks {
            assert!((h.stroke_width - 5.5).abs() < 1e-9);
        }
    }

    #[test]
    fn hooks_start_x_strictly_less_than_end_x_for_both() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::Both, 30.0, HookDirection::Down, 4.0);
        assert!(hooks[0].x < hooks[1].x);
    }

    #[test]
    fn hooks_end_x_matches_trill_extension_right_edge_helper() {
        let ext = make_extension();
        let hooks =
            layout_trill_bracket_hooks(&ext, TrillBracketSide::End, 30.0, HookDirection::Down, 4.0);
        // Cross-check against the public right-edge helper to ensure the two
        // layout APIs stay in sync.
        assert_eq!(
            hooks[0].x,
            crate::layout::trill_extension::trill_extension_right_edge(&ext)
        );
    }

    // --- layout_trill_bracket_hooks_multi_speed ---

    fn make_multi_speed_extension() -> MultiSpeedTrillExtensionLayout {
        // Two regions, each with 2 tiles. Region 0: glyph=WiggleTrillSlow,
        // advance=100, span starts at x=100 → tiles at 100, 200.
        // Region 1: glyph=WiggleTrillFast, advance=80, span starts at
        // x=300 → tiles at 300, 380. Right edge = 380 + 80 = 460.
        // y=50.
        use crate::layout::trill_extension::{
            layout_trill_extension_multi_speed, TrillSpeedRegion,
        };
        layout_trill_extension_multi_speed(
            460.0,
            50.0,
            &[
                TrillSpeedRegion {
                    start_x: 100.0,
                    glyph: Glyph::WiggleTrillSlow,
                    segment_advance: 100.0,
                },
                TrillSpeedRegion {
                    start_x: 300.0,
                    glyph: Glyph::WiggleTrillFast,
                    segment_advance: 80.0,
                },
            ],
        )
        .expect("multi-speed layout must succeed")
    }

    #[test]
    fn multi_speed_hooks_start_only_at_first_tile_x() {
        let ext = make_multi_speed_extension();
        let hooks = layout_trill_bracket_hooks_multi_speed(
            &ext,
            TrillBracketSide::Start,
            30.0,
            HookDirection::Down,
            4.0,
        );
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].x, 100.0);
    }

    #[test]
    fn multi_speed_hooks_end_only_at_right_edge() {
        let ext = make_multi_speed_extension();
        let hooks = layout_trill_bracket_hooks_multi_speed(
            &ext,
            TrillBracketSide::End,
            30.0,
            HookDirection::Down,
            4.0,
        );
        assert_eq!(hooks.len(), 1);
        // Region 1's last tile at x=380, advance 80 → right edge = 460.
        assert_eq!(hooks[0].x, 460.0);
    }

    #[test]
    fn multi_speed_hooks_both_returns_two_at_endpoints() {
        let ext = make_multi_speed_extension();
        let hooks = layout_trill_bracket_hooks_multi_speed(
            &ext,
            TrillBracketSide::Both,
            30.0,
            HookDirection::Down,
            4.0,
        );
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].x, 100.0);
        assert_eq!(hooks[1].x, 460.0);
    }

    #[test]
    fn multi_speed_hooks_empty_layout_returns_empty_vec() {
        let ext = MultiSpeedTrillExtensionLayout {
            tiles: vec![],
            y: 50.0,
        };
        let hooks = layout_trill_bracket_hooks_multi_speed(
            &ext,
            TrillBracketSide::Both,
            30.0,
            HookDirection::Down,
            4.0,
        );
        assert!(hooks.is_empty());
    }

    #[test]
    fn multi_speed_hooks_share_layout_y_baseline_down() {
        let ext = make_multi_speed_extension();
        let hooks = layout_trill_bracket_hooks_multi_speed(
            &ext,
            TrillBracketSide::Both,
            30.0,
            HookDirection::Down,
            4.0,
        );
        for h in &hooks {
            assert_eq!(h.y_top, ext.y);
            assert_eq!(h.y_bottom, ext.y + 30.0);
        }
    }

    #[test]
    fn multi_speed_hooks_match_single_speed_when_only_one_region() {
        // Equivalence canary: a multi-speed layout with one region whose
        // tiles match a single-speed layout's segment_xs must produce the
        // same bracket hook x-coordinates as the single-speed bracket
        // helper. Guards against the two helpers drifting in their
        // start/right-edge anchor conventions.
        use crate::layout::trill_extension::{
            layout_trill_extension, layout_trill_extension_multi_speed, TrillSpeedRegion,
        };
        let single = layout_trill_extension(50.0, 50.0 + 3.0 * 80.0, 0.0, 80.0).unwrap();
        let multi = layout_trill_extension_multi_speed(
            50.0 + 3.0 * 80.0,
            0.0,
            &[TrillSpeedRegion {
                start_x: 50.0,
                glyph: Glyph::WiggleTrill,
                segment_advance: 80.0,
            }],
        )
        .unwrap();
        let single_hooks = layout_trill_bracket_hooks(
            &single,
            TrillBracketSide::Both,
            30.0,
            HookDirection::Down,
            4.0,
        );
        let multi_hooks = layout_trill_bracket_hooks_multi_speed(
            &multi,
            TrillBracketSide::Both,
            30.0,
            HookDirection::Down,
            4.0,
        );
        assert_eq!(single_hooks.len(), multi_hooks.len());
        for (s, m) in single_hooks.iter().zip(multi_hooks.iter()) {
            assert_eq!(s.x, m.x);
            assert_eq!(s.y_top, m.y_top);
            assert_eq!(s.y_bottom, m.y_bottom);
        }
    }

    // --- TrillBracketOptions (builder) ---

    #[test]
    fn options_new_has_required_side_and_unset_overrides() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Start);
        assert_eq!(opts.side, TrillBracketSide::Start);
        assert_eq!(opts.direction, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn options_with_direction_sets_only_direction() {
        let opts = TrillBracketOptions::new(TrillBracketSide::End).with_direction(HookDirection::Up);
        assert_eq!(opts.direction, Some(HookDirection::Up));
        assert_eq!(opts.length_ss, None, "length must remain unset");
        assert_eq!(opts.side, TrillBracketSide::End, "side must remain End");
    }

    #[test]
    fn options_with_length_ss_sets_only_length() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(1.25);
        assert_eq!(opts.length_ss, Some(1.25));
        assert_eq!(opts.direction, None, "direction must remain unset");
        assert_eq!(opts.side, TrillBracketSide::Both);
    }

    #[test]
    fn options_chained_with_methods_both_apply() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_direction(HookDirection::Up)
            .with_length_ss(0.5);
        assert_eq!(opts.direction, Some(HookDirection::Up));
        assert_eq!(opts.length_ss, Some(0.5));
    }

    #[test]
    fn options_chain_order_independent() {
        let a = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_direction(HookDirection::Down)
            .with_length_ss(0.9);
        let b = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_length_ss(0.9)
            .with_direction(HookDirection::Down);
        assert_eq!(a, b, "chain order must not affect the resulting options");
    }

    #[test]
    fn options_with_method_overwrites_prior_value() {
        // Calling with_direction twice keeps the last value — covers the
        // case where a caller composes options conditionally.
        let opts = TrillBracketOptions::new(TrillBracketSide::Start)
            .with_direction(HookDirection::Down)
            .with_direction(HookDirection::Up);
        assert_eq!(opts.direction, Some(HookDirection::Up));
    }

    #[test]
    fn options_from_side_matches_new() {
        let from_side: TrillBracketOptions = TrillBracketSide::End.into();
        assert_eq!(from_side, TrillBracketOptions::new(TrillBracketSide::End));
    }

    #[test]
    fn options_const_constructible() {
        // The constructor and with-methods must be `const`-callable so the
        // common defaults can live in `const` items at module scope. If
        // someone removes `const fn`, this test stops compiling.
        const _OPTS: TrillBracketOptions = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_direction(HookDirection::Down)
            .with_length_ss(0.75);
    }

    #[test]
    fn options_copy_does_not_consume_original() {
        // Options is `Copy`; passing it by value to a function that returns
        // it must leave the original usable. Catches accidental removal of
        // the `Copy` derive.
        fn take(opts: TrillBracketOptions) -> TrillBracketOptions {
            opts
        }
        let orig = TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(1.0);
        let _ = take(orig);
        assert_eq!(orig.length_ss, Some(1.0));
    }

    // --- TrillBracketOptions: non-breaking `hook_length_ss` alias for `length_ss` ---
    //
    // The `length_ss` field is the hook length (the short vertical line
    // capping the wiggle). The same struct also carries `extension_length_ss`
    // (the wiggle's horizontal termination length). The bare name
    // `length_ss` is ambiguous next to `extension_length_ss`; the alias
    // `hook_length_ss` (setter `with_hook_length_ss`, getter `hook_length_ss`)
    // names the hook semantics explicitly without breaking the existing API.
    // The two setters write the same field; these tests lock that in.

    #[test]
    fn with_hook_length_ss_writes_to_length_ss_field() {
        // The new setter must populate the existing `length_ss` field — not
        // a separate parallel field. Catches a refactor that accidentally
        // introduces a phantom `hook_length_ss` field that diverges from
        // `length_ss` at the storage layer.
        let opts = TrillBracketOptions::new(TrillBracketSide::Both).with_hook_length_ss(0.85);
        assert_eq!(opts.length_ss, Some(0.85));
        // And no other knob is touched.
        assert_eq!(opts.direction, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.extension_length_ss, None);
        assert_eq!(opts.side, TrillBracketSide::Both);
    }

    #[test]
    fn with_hook_length_ss_is_byte_equivalent_to_with_length_ss() {
        // Both setters must produce structurally identical options. If a
        // future refactor splits them into different field assignments, this
        // catches it directly. PartialEq derive covers every field — the
        // assertion fires if any field diverges.
        for &len in &[0.0, 0.5, 0.75, 1.0, 1.5, 3.5, -2.0] {
            let via_legacy = TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(len);
            let via_new =
                TrillBracketOptions::new(TrillBracketSide::Both).with_hook_length_ss(len);
            assert_eq!(
                via_legacy, via_new,
                "with_length_ss({len}) and with_hook_length_ss({len}) must produce equal options"
            );
        }
    }

    #[test]
    fn with_hook_length_ss_overwrites_with_length_ss_when_chained() {
        // Last-write-wins canary: a caller chaining both setters lands on
        // whichever was called last. The two are aliases, so this is the
        // expected sequential-mutation semantic.
        let opts_last_new = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_length_ss(0.5)
            .with_hook_length_ss(1.2);
        assert_eq!(opts_last_new.length_ss, Some(1.2));

        let opts_last_legacy = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_hook_length_ss(1.2)
            .with_length_ss(0.5);
        assert_eq!(opts_last_legacy.length_ss, Some(0.5));
    }

    #[test]
    fn hook_length_ss_getter_returns_length_ss_field() {
        // Getter must return the same Option<f64> the field holds, including
        // the `None` default.
        let unset = TrillBracketOptions::new(TrillBracketSide::Both);
        assert_eq!(unset.hook_length_ss(), None);
        assert_eq!(unset.hook_length_ss(), unset.length_ss);

        let via_legacy = TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(0.6);
        assert_eq!(via_legacy.hook_length_ss(), Some(0.6));
        assert_eq!(via_legacy.hook_length_ss(), via_legacy.length_ss);

        let via_new = TrillBracketOptions::new(TrillBracketSide::Both).with_hook_length_ss(0.6);
        assert_eq!(via_new.hook_length_ss(), Some(0.6));
        assert_eq!(via_new.hook_length_ss(), via_new.length_ss);
    }

    #[test]
    fn with_hook_length_ss_is_distinct_from_with_extension_length_ss() {
        // Critical naming-disambiguation canary, mirroring
        // `options_with_extension_length_ss_is_distinct_from_with_length_ss`
        // for the new alias. The two methods MUST write to different fields
        // — hook length and extension length are semantically distinct knobs
        // even though both are lengths in staff spaces.
        let hook_only =
            TrillBracketOptions::new(TrillBracketSide::Both).with_hook_length_ss(1.0);
        let ext_only =
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(1.0);

        assert_eq!(hook_only.length_ss, Some(1.0));
        assert_eq!(hook_only.extension_length_ss, None);
        assert_eq!(ext_only.length_ss, None);
        assert_eq!(ext_only.extension_length_ss, Some(1.0));
        assert_ne!(
            hook_only, ext_only,
            "hook length and extension length must be independent fields"
        );
    }

    #[test]
    fn with_hook_length_ss_chains_with_other_setters() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Start)
            .with_direction(HookDirection::Up)
            .with_hook_length_ss(0.65)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(3.0);
        assert_eq!(opts.side, TrillBracketSide::Start);
        assert_eq!(opts.direction, Some(HookDirection::Up));
        assert_eq!(opts.length_ss, Some(0.65));
        assert_eq!(opts.hook_length_ss(), Some(0.65));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.extension_length_ss, Some(3.0));
    }

    #[test]
    fn with_hook_length_ss_chain_order_independent_from_other_setters() {
        // Mirrors `options_chain_order_independent` for the new setter:
        // calling order must not affect the resulting struct.
        let a = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_direction(HookDirection::Down)
            .with_hook_length_ss(0.9)
            .with_ornament(Ornament::TrillWithMordent);
        let b = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_ornament(Ornament::TrillWithMordent)
            .with_hook_length_ss(0.9)
            .with_direction(HookDirection::Down);
        let c = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_hook_length_ss(0.9)
            .with_direction(HookDirection::Down)
            .with_ornament(Ornament::TrillWithMordent);
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn with_hook_length_ss_is_const_constructible() {
        // Mirror of `options_const_constructible` for the new setter. Locks
        // in `const fn` — a future change that drops `const` would silently
        // disqualify the new method from `const` items at module scope; this
        // test stops compiling in that case.
        const _OPTS: TrillBracketOptions = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_direction(HookDirection::Down)
            .with_hook_length_ss(0.75);
    }

    #[test]
    fn hook_length_ss_getter_is_const_callable() {
        // The getter must be `const fn` for the same reason — symmetry with
        // the setter and to let callers read defaults at const-eval time.
        const _LEN: Option<f64> =
            TrillBracketOptions::new(TrillBracketSide::Both).hook_length_ss();
        // The accessor on the `None` default must yield `None` — basic value
        // check beyond the bare "compiles in const context" guarantee.
        assert_eq!(_LEN, None);
    }

    // --- TrillBracketOptions ornament override ---

    #[test]
    fn options_new_has_unset_ornament() {
        // `None` here is meaningful: the score-builder layer collapses it to
        // `Ornament::Trill`. The layout struct itself stays glyph-agnostic.
        let opts = TrillBracketOptions::new(TrillBracketSide::Both);
        assert_eq!(opts.ornament, None);
    }

    #[test]
    fn options_with_ornament_sets_only_ornament() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_ornament(Ornament::TrillWithMordent);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.direction, None, "direction must remain unset");
        assert_eq!(opts.length_ss, None, "length must remain unset");
        assert_eq!(opts.side, TrillBracketSide::Both);
    }

    #[test]
    fn options_with_ornament_chains_with_other_setters() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Start)
            .with_ornament(Ornament::TrillWithMordent)
            .with_direction(HookDirection::Up)
            .with_length_ss(1.1);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.direction, Some(HookDirection::Up));
        assert_eq!(opts.length_ss, Some(1.1));
    }

    #[test]
    fn options_with_ornament_chain_order_independent() {
        let a = TrillBracketOptions::new(TrillBracketSide::End)
            .with_ornament(Ornament::TrillWithMordent)
            .with_length_ss(0.9);
        let b = TrillBracketOptions::new(TrillBracketSide::End)
            .with_length_ss(0.9)
            .with_ornament(Ornament::TrillWithMordent);
        assert_eq!(a, b, "with_ornament must commute with other setters");
    }

    #[test]
    fn options_with_ornament_overwrites_prior_value() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_ornament(Ornament::Trill)
            .with_ornament(Ornament::TrillWithMordent);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
    }

    #[test]
    fn options_with_ornament_const_constructible() {
        // `with_ornament` must be `const` so the canonical bundles can live
        // in `const` items. Removal of `const fn` makes this stop compiling.
        const _OPTS: TrillBracketOptions = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_ornament(Ornament::TrillWithMordent);
    }

    #[test]
    fn options_with_ornament_accepts_unsupported_ornament_at_layout_layer() {
        // The layout struct does not validate the contract — it is the
        // renderer's collector (filtering by `supports_trill_extension()`)
        // that drops the extension for unsupported ornaments. Documenting
        // this at the layout layer: an unsupported ornament must still
        // round-trip through the field unchanged so the renderer can apply
        // the filter consistently.
        let opts = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_ornament(Ornament::ShortTrill);
        assert_eq!(opts.ornament, Some(Ornament::ShortTrill));
        assert!(!Ornament::ShortTrill.supports_trill_extension());
    }

    // --- TrillBracketOptions extension length override ---

    #[test]
    fn options_new_has_unset_extension_length_ss() {
        // The newly-added field must start `None` so existing callers (who
        // never touch it) keep their byte-equivalent SVG output.
        let opts = TrillBracketOptions::new(TrillBracketSide::Both);
        assert_eq!(opts.extension_length_ss, None);
    }

    #[test]
    fn options_with_extension_length_ss_sets_only_extension_length() {
        // Setting the new field must NOT disturb the hook length, direction,
        // ornament, or side — those are independent knobs.
        let opts =
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(2.75);
        assert_eq!(opts.extension_length_ss, Some(2.75));
        assert_eq!(opts.length_ss, None, "hook length must remain unset");
        assert_eq!(opts.direction, None, "direction must remain unset");
        assert_eq!(opts.ornament, None, "ornament must remain unset");
        assert_eq!(opts.side, TrillBracketSide::Both, "side must survive");
    }

    #[test]
    fn options_with_extension_length_ss_is_distinct_from_with_length_ss() {
        // Critical regression canary: `with_length_ss` and
        // `with_extension_length_ss` write *different* fields. If a future
        // refactor accidentally aliased them — e.g. by collapsing both
        // setters onto the same field — this assertion fires. Catches a
        // very plausible naming-confusion bug.
        let hook_only = TrillBracketOptions::new(TrillBracketSide::Both).with_length_ss(1.0);
        let ext_only =
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(1.0);
        assert_eq!(hook_only.length_ss, Some(1.0));
        assert_eq!(hook_only.extension_length_ss, None);
        assert_eq!(ext_only.length_ss, None);
        assert_eq!(ext_only.extension_length_ss, Some(1.0));
        assert_ne!(hook_only, ext_only);
    }

    #[test]
    fn options_with_extension_length_ss_chains_with_other_setters() {
        let opts = TrillBracketOptions::new(TrillBracketSide::End)
            .with_direction(HookDirection::Up)
            .with_length_ss(0.6)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(3.25);
        assert_eq!(opts.side, TrillBracketSide::End);
        assert_eq!(opts.direction, Some(HookDirection::Up));
        assert_eq!(opts.length_ss, Some(0.6));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.extension_length_ss, Some(3.25));
    }

    #[test]
    fn options_with_extension_length_ss_chain_order_independent() {
        let a = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_extension_length_ss(2.0)
            .with_length_ss(0.8);
        let b = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_length_ss(0.8)
            .with_extension_length_ss(2.0);
        assert_eq!(
            a, b,
            "with_extension_length_ss must commute with with_length_ss"
        );
    }

    #[test]
    fn options_with_extension_length_ss_overwrites_prior_value() {
        let opts = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_extension_length_ss(1.0)
            .with_extension_length_ss(5.5);
        assert_eq!(opts.extension_length_ss, Some(5.5));
    }

    #[test]
    fn options_with_extension_length_ss_accepts_non_positive_at_layout_layer() {
        // Layout layer is validation-free (mirrors the policy for
        // unsupported ornaments) — zero and negative values round-trip
        // unchanged. The renderer's existing non-positive fail-safe gives
        // them the "no wiggle" semantic at draw time.
        let zero =
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(0.0);
        assert_eq!(zero.extension_length_ss, Some(0.0));
        let neg =
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(-1.5);
        assert_eq!(neg.extension_length_ss, Some(-1.5));
    }

    #[test]
    fn options_with_extension_length_ss_const_constructible() {
        // The new setter must remain `const`-callable so canonical bundles
        // can live in module-level `const` items. Removing `const fn`
        // breaks this compile-time canary.
        const _OPTS: TrillBracketOptions = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_length_ss(0.75)
            .with_extension_length_ss(2.5);
    }

    #[test]
    fn options_extension_length_some_zero_distinct_from_none() {
        // `Some(0.0)` is semantically distinct from `None` (it suppresses
        // the wiggle at the renderer's non-positive fail-safe; `None` lets
        // the natural span flow). PartialEq must keep them distinct.
        let none = TrillBracketOptions::new(TrillBracketSide::Both);
        let zero =
            TrillBracketOptions::new(TrillBracketSide::Both).with_extension_length_ss(0.0);
        assert_ne!(none, zero);
    }
}
