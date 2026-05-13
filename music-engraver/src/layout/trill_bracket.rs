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

use crate::layout::trill_extension::{trill_extension_right_edge, TrillExtensionLayout};

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
/// // Override direction while keeping default length.
/// let _ = TrillBracketOptions::new(TrillBracketSide::Start).with_direction(HookDirection::Up);
/// ```
///
/// `None` for either optional field means "the layout/renderer picks the
/// conventional default at draw time," and the resulting SVG is byte-
/// identical to the existing non-custom `trill_with_extension_bracketed`
/// API. Callers who want to lock in the defaults explicitly should pass
/// `Some(HookDirection::Down)` and `Some(0.75)` instead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrillBracketOptions {
    /// Which end(s) of the wiggle to bracket — always required.
    pub side: TrillBracketSide,
    /// Hook direction override. `None` defers to the renderer's conventional
    /// default (`HookDirection::Down`).
    pub direction: Option<HookDirection>,
    /// Hook length in staff spaces. `None` defers to the renderer's
    /// conventional default (~0.75ss).
    pub length_ss: Option<f64>,
}

impl TrillBracketOptions {
    /// Construct options bracketing the given side with both knobs at
    /// renderer defaults (Down direction, ~0.75ss length).
    pub const fn new(side: TrillBracketSide) -> Self {
        Self {
            side,
            direction: None,
            length_ss: None,
        }
    }

    /// Override the hook direction. Pass `HookDirection::Up` for trills
    /// rendered below the staff so the hook still points back toward the
    /// affected notes.
    pub const fn with_direction(mut self, direction: HookDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Override the hook length in staff spaces. Reasonable values are
    /// roughly 0.5..=1.0; the layout does not clamp, and a 0.0 length
    /// produces a degenerate hook (no visible line).
    pub const fn with_length_ss(mut self, length_ss: f64) -> Self {
        self.length_ss = Some(length_ss);
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
}
