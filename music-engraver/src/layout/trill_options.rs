//! Unified options bundle for trill-with-extension annotations.
//!
//! [`TrillBracketOptions`] (in `trill_bracket`) covers bracket-side + hook
//! direction + hook length + ornament. [`TrillExtensionSpeedOptions`] (in
//! `trill_extension`) covers wiggle speed + ornament. Both close their own
//! corner of the trill-extension feature surface, but neither lets a caller
//! request a *combination* of bracket and speed in a single call — a
//! `TrillWithMordent` rendered at `Slowest` wiggle speed with `End` bracket
//! and a custom hook length is reachable only by hand-constructing the
//! annotation fields directly.
//!
//! [`TrillExtensionFullOptions`] subsumes both: every knob is optional, so
//! all of the existing single-purpose options bundles can be expressed as
//! `From<TrillBracketOptions>` / `From<TrillExtensionSpeedOptions>`
//! conversions into this one. The bare
//! [`crate::score::ScoreBuilder::trill_with_extension`] case corresponds to
//! `TrillExtensionFullOptions::new()` (all fields `None`).
//!
//! The bundle stays glyph-agnostic: the `ornament` field is the source of
//! truth for the SMuFL glyph picked at draw time, and the renderer's
//! existing collector — which filters by
//! [`crate::layout::ornament::Ornament::supports_trill_extension`] — applies
//! uniformly regardless of which API path the annotation came in through.

use crate::layout::ornament::Ornament;
use crate::layout::trill_bracket::{HookDirection, TrillBracketOptions, TrillBracketSide};
use crate::layout::trill_extension::{
    TrillExtensionSpeedOptions, TrillSpeedRamp, TrillSpeedRampSpec, TrillWiggleSpeed,
};

/// Unified options for a trill-with-extension annotation.
///
/// Every knob is optional; `None` means "use the existing default at draw
/// time" rather than "set the field to a sentinel." Concretely:
///
/// - `bracket = None` — no bracket hooks at all (the trill is a plain
///   trailing wiggle).
/// - `bracket_direction = None` / `bracket_length_ss = None` — fall back to
///   the renderer's conventional defaults (`HookDirection::Down`, ~0.75
///   staff spaces). Setting these without a bracket is a no-op.
/// - `speed = None` — use the standard wiggle glyph (`WiggleTrill`).
/// - `ornament = None` — use the plain `Trill` glyph (`OrnamentTrill`).
///
/// Construction is fluent:
///
/// ```no_run
/// use music_engraver::layout::ornament::Ornament;
/// use music_engraver::layout::trill_bracket::{HookDirection, TrillBracketSide};
/// use music_engraver::layout::trill_extension::TrillWiggleSpeed;
/// use music_engraver::layout::trill_options::TrillExtensionFullOptions;
///
/// // Bracket + speed + compound ornament + explicit termination length in one call.
/// let _ = TrillExtensionFullOptions::new()
///     .with_bracket(TrillBracketSide::End)
///     .with_bracket_direction(HookDirection::Down)
///     .with_bracket_length_ss(0.9)
///     .with_speed(TrillWiggleSpeed::Slow)
///     .with_ornament(Ornament::TrillWithMordent)
///     .with_length_ss(3.5);
/// ```
///
/// `From<TrillBracketOptions>` and `From<TrillExtensionSpeedOptions>` are
/// provided so existing single-purpose options bundles can be widened to a
/// full bundle without restating their state:
///
/// ```no_run
/// use music_engraver::layout::trill_bracket::{TrillBracketOptions, TrillBracketSide};
/// use music_engraver::layout::trill_options::TrillExtensionFullOptions;
///
/// let bracket_only = TrillBracketOptions::new(TrillBracketSide::Both);
/// let widened: TrillExtensionFullOptions = bracket_only.into();
/// ```
///
/// The ornament must satisfy [`Ornament::supports_trill_extension`];
/// passing an unsupported ornament makes the renderer's collector silently
/// drop the wiggle and any bracket hooks, leaving only the ornament glyph.
/// This contract matches the two single-purpose options bundles.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TrillExtensionFullOptions {
    /// Which end(s) of the wiggle to bracket. `None` means no bracket.
    pub bracket: Option<TrillBracketSide>,
    /// Hook direction override. `None` defers to the renderer's default
    /// ([`HookDirection::Down`]). Ignored when `bracket == None`.
    pub bracket_direction: Option<HookDirection>,
    /// Hook length in staff spaces. `None` defers to the renderer's default
    /// (~0.75ss). Ignored when `bracket == None`.
    pub bracket_length_ss: Option<f64>,
    /// Wiggle speed/density override. `None` uses the neutral
    /// [`TrillWiggleSpeed::Standard`] glyph (`WiggleTrill`).
    pub speed: Option<TrillWiggleSpeed>,
    /// Ornament glyph override. `None` selects [`Ornament::Trill`].
    pub ornament: Option<Ornament>,
    /// Optional explicit termination length for the wavy line, in staff
    /// spaces. `None` lets the wiggle extend to the next note (within-system)
    /// or to the system's right edge (cross-system) per the usual convention.
    /// `Some(length_ss)` clamps the wiggle to terminate no later than
    /// `length_ss` staff spaces past its natural start, mirroring the contract
    /// of [`crate::score::ScoreBuilder::trill_with_extension_length_ss`].
    /// Clamping is one-sided (overruns clamp to the natural span; non-positive
    /// values produce no wiggle). A positive explicit length disables
    /// cross-system propagation — the wiggle terminates within its source
    /// system. Independent of bracket: combining `Some(length_ss)` with a
    /// bracket anchors the end-hook at the explicitly-shortened terminus.
    ///
    /// Note: this is the **wiggle extension** length, *not* the bracket hook
    /// length — see [`bracket_length_ss`](Self::bracket_length_ss) for the
    /// latter. The clearer-named
    /// [`extension_length_ss()`](Self::extension_length_ss) accessor returns
    /// the same `Option<f64>`.
    pub length_ss: Option<f64>,
    /// Optional multi-speed ramp specification. `None` (the default) means
    /// "single-speed wiggle" — the renderer uses [`speed`](Self::speed) (or
    /// the [`TrillWiggleSpeed::Standard`] default when that is also `None`).
    /// `Some(spec)` requests a multi-speed wiggle synthesized from the
    /// ramp + region count at draw time via
    /// [`TrillSpeedRamp::synthesize_regions`] fed into
    /// [`crate::layout::trill_extension::layout_trill_extension_multi_speed`].
    ///
    /// When `speed_ramp` is set, the single-speed [`speed`](Self::speed)
    /// field is ignored — the ramp's per-region speeds supersede it. Both
    /// fields are allowed to be set simultaneously so the options bundle
    /// composes additively (e.g. a caller widening from
    /// [`TrillExtensionSpeedOptions`] then layering in a ramp doesn't have
    /// to first clear the speed field). The dispatch is monotone:
    /// `speed_ramp.is_some()` → multi-speed path; otherwise single-speed.
    ///
    /// Carrying a `region_count` of `0` or a `Linear` ramp with
    /// `region_count == 1` is the same "degenerate input → fall back to
    /// single-speed" contract as a hand-built
    /// [`TrillSpeedRamp::synthesize_regions`] call: the synthesizer returns
    /// `None` and the renderer drops back to the single-speed path. The
    /// layout layer does not pre-reject those at options-construction time
    /// (mirroring the unsupported-ornament / non-positive-length policy).
    pub speed_ramp: Option<TrillSpeedRampSpec>,
}

impl TrillExtensionFullOptions {
    /// Construct an options bundle with every knob unset (all defaults).
    ///
    /// Applying this to a note via the ScoreBuilder is byte-equivalent to
    /// calling [`crate::score::ScoreBuilder::trill_with_extension`].
    pub const fn new() -> Self {
        Self {
            bracket: None,
            bracket_direction: None,
            bracket_length_ss: None,
            speed: None,
            ornament: None,
            length_ss: None,
            speed_ramp: None,
        }
    }

    /// Bracket the specified side(s) of the wiggle.
    pub const fn with_bracket(mut self, side: TrillBracketSide) -> Self {
        self.bracket = Some(side);
        self
    }

    /// Override the hook direction. Ignored when no bracket is set.
    pub const fn with_bracket_direction(mut self, direction: HookDirection) -> Self {
        self.bracket_direction = Some(direction);
        self
    }

    /// Override the hook length in staff spaces. Ignored when no bracket is
    /// set. The renderer does not clamp; a 0.0 length produces a degenerate
    /// hook (no visible line).
    pub const fn with_bracket_length_ss(mut self, length_ss: f64) -> Self {
        self.bracket_length_ss = Some(length_ss);
        self
    }

    /// Override the wiggle speed/density variant.
    pub const fn with_speed(mut self, speed: TrillWiggleSpeed) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Override the ornament glyph. Pass [`Ornament::TrillWithMordent`] for
    /// a precomposed compound trill. The ornament must satisfy
    /// [`Ornament::supports_trill_extension`]; an unsupported ornament
    /// causes the renderer to drop the entire trill extension.
    pub const fn with_ornament(mut self, ornament: Ornament) -> Self {
        self.ornament = Some(ornament);
        self
    }

    /// Set an explicit termination length for the wavy line, in staff spaces,
    /// measured from the wiggle's natural start (past the ornament glyph and
    /// its trailing padding). Overruns are clamped to the natural span; a
    /// non-positive length suppresses the wiggle entirely. Setting a positive
    /// length disables cross-system propagation for this trill.
    ///
    /// Mirrors the contract of
    /// [`crate::score::ScoreBuilder::trill_with_extension_length_ss`] and is
    /// independent of the bracket — a bracketed trill with an explicit length
    /// anchors its end hook at the shortened terminus.
    ///
    /// Prefer
    /// [`with_extension_length_ss`](Self::with_extension_length_ss) at call
    /// sites where the bare name `length_ss` reads ambiguously next to
    /// `bracket_length_ss` — both setters write the same field and are
    /// byte-equivalent. The shorter name is retained for backwards
    /// compatibility (renaming the public method would be breaking).
    pub const fn with_length_ss(mut self, length_ss: f64) -> Self {
        self.length_ss = Some(length_ss);
        self
    }

    /// Non-breaking alias for [`with_length_ss`](Self::with_length_ss) with a
    /// clearer name. Writes to the same [`length_ss`](Self::length_ss) field
    /// (the wiggle's horizontal *extension* termination length, not the
    /// bracket hook length — see
    /// [`with_bracket_length_ss`](Self::with_bracket_length_ss)).
    ///
    /// The two setters are byte-equivalent; pick whichever reads more clearly
    /// at the call site. Locked into the same field by construction, so a
    /// future refactor that splits them would have to update both methods
    /// together. This alias symmetrically mirrors
    /// [`crate::layout::trill_bracket::TrillBracketOptions::with_extension_length_ss`]
    /// — both single-purpose and full-options bundles now expose the same
    /// `with_extension_length_ss` setter name for the wiggle's termination
    /// length.
    pub const fn with_extension_length_ss(mut self, length_ss: f64) -> Self {
        self.length_ss = Some(length_ss);
        self
    }

    /// Read the wiggle's explicit *extension* termination length, equivalent
    /// to accessing the [`length_ss`](Self::length_ss) field directly but
    /// named to match
    /// [`with_extension_length_ss`](Self::with_extension_length_ss) — clearer
    /// at the call site when the surrounding code also reads
    /// [`bracket_length_ss`](Self::bracket_length_ss). Returns `None` when no
    /// explicit length is set.
    pub const fn extension_length_ss(&self) -> Option<f64> {
        self.length_ss
    }

    /// Attach a multi-speed ramp spec to the options bundle. The ramp's
    /// per-region speeds supersede [`speed`](Self::speed) when both are set
    /// — the renderer dispatches on `speed_ramp.is_some()`.
    ///
    /// This is the spec-typed setter; for callers building inline, the
    /// two-arg [`with_speed_ramp_ramp_count`](Self::with_speed_ramp_ramp_count)
    /// is more ergonomic.
    pub const fn with_speed_ramp(mut self, spec: TrillSpeedRampSpec) -> Self {
        self.speed_ramp = Some(spec);
        self
    }

    /// Attach a multi-speed ramp spec inline from a ramp + region count.
    /// Byte-equivalent to
    /// `with_speed_ramp(TrillSpeedRampSpec::new(ramp, region_count))`.
    ///
    /// Most call sites prefer this form because it elides the explicit
    /// `TrillSpeedRampSpec::new(...)` wrapping.
    pub const fn with_speed_ramp_ramp_count(
        mut self,
        ramp: TrillSpeedRamp,
        region_count: usize,
    ) -> Self {
        self.speed_ramp = Some(TrillSpeedRampSpec::new(ramp, region_count));
        self
    }
}

impl From<TrillBracketOptions> for TrillExtensionFullOptions {
    fn from(opts: TrillBracketOptions) -> Self {
        // `length_ss` propagates from the single-purpose bundle's
        // `extension_length_ss` field — both `From<TrillBracketOptions>`
        // here and `trill_with_extension_bracketed_with_options(opts)`
        // write the same value, so the documented byte-equivalence with
        // the full-options widening path survives a non-`None`
        // extension length.
        Self {
            bracket: Some(opts.side),
            bracket_direction: opts.direction,
            bracket_length_ss: opts.length_ss,
            speed: None,
            ornament: opts.ornament,
            length_ss: opts.extension_length_ss,
            // Bracket-only options carry no multi-speed ramp by
            // construction — `TrillBracketOptions` has no ramp field. Any
            // ramp the caller wants must be layered in on the widened
            // bundle via `with_speed_ramp(_ramp_count)?`.
            speed_ramp: None,
        }
    }
}

impl From<TrillExtensionSpeedOptions> for TrillExtensionFullOptions {
    fn from(opts: TrillExtensionSpeedOptions) -> Self {
        // `length_ss` propagates from the speed bundle's
        // `extension_length_ss` to mirror the byte-equivalence guarantee
        // with `trill_with_extension_speed_with_options(opts)` when the
        // speed bundle carries an explicit length.
        Self {
            bracket: None,
            bracket_direction: None,
            bracket_length_ss: None,
            speed: Some(opts.speed),
            ornament: opts.ornament,
            length_ss: opts.extension_length_ss,
            // Single-speed options carry no multi-speed ramp. A caller
            // that wants to widen a `TrillExtensionSpeedOptions` and add a
            // ramp must layer the ramp on the widened bundle — the speed
            // field stays populated and is superseded by the ramp at
            // draw time per the `speed_ramp` field's dispatch doc.
            speed_ramp: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Constructors and defaults ---

    #[test]
    fn new_has_every_field_unset() {
        let opts = TrillExtensionFullOptions::new();
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
        assert_eq!(opts.speed_ramp, None);
    }

    #[test]
    fn default_matches_new() {
        // The `#[derive(Default)]` must agree with `new()` because the
        // builder method's no-op contract relies on a guaranteed all-None
        // starting state. If a future `new()` ever changed its defaults
        // without updating the derive, this would catch it.
        assert_eq!(
            TrillExtensionFullOptions::default(),
            TrillExtensionFullOptions::new()
        );
    }

    // --- With-methods set only the targeted field ---

    #[test]
    fn with_bracket_sets_only_bracket() {
        let opts = TrillExtensionFullOptions::new().with_bracket(TrillBracketSide::End);
        assert_eq!(opts.bracket, Some(TrillBracketSide::End));
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_bracket_direction_sets_only_direction() {
        let opts = TrillExtensionFullOptions::new().with_bracket_direction(HookDirection::Up);
        assert_eq!(opts.bracket_direction, Some(HookDirection::Up));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_bracket_length_ss_sets_only_length() {
        let opts = TrillExtensionFullOptions::new().with_bracket_length_ss(1.25);
        assert_eq!(opts.bracket_length_ss, Some(1.25));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_speed_sets_only_speed() {
        let opts = TrillExtensionFullOptions::new().with_speed(TrillWiggleSpeed::Slowest);
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Slowest));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_ornament_sets_only_ornament() {
        let opts = TrillExtensionFullOptions::new().with_ornament(Ornament::TrillWithMordent);
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_length_ss_sets_only_length_ss() {
        // The new method must isolate the same way the other five with_*
        // setters do: flip only `length_ss` and leave every other field at
        // its `None` default. Catches a regression where the setter
        // accidentally resets a sibling field.
        let opts = TrillExtensionFullOptions::new().with_length_ss(3.5);
        assert_eq!(opts.length_ss, Some(3.5));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
    }

    // --- Chaining ---

    #[test]
    fn chain_sets_all_six_knobs() {
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_bracket_direction(HookDirection::Down)
            .with_bracket_length_ss(0.9)
            .with_speed(TrillWiggleSpeed::Fast)
            .with_ornament(Ornament::TrillWithMordent)
            .with_length_ss(4.25);
        assert_eq!(opts.bracket, Some(TrillBracketSide::Both));
        assert_eq!(opts.bracket_direction, Some(HookDirection::Down));
        assert_eq!(opts.bracket_length_ss, Some(0.9));
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Fast));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.length_ss, Some(4.25));
    }

    #[test]
    fn chain_order_independent() {
        // The six with_* methods all commute. Two equivalent orderings must
        // produce equal options bundles. Catches a regression where a
        // with-method accidentally reset another field.
        let a = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::End)
            .with_speed(TrillWiggleSpeed::Slow)
            .with_ornament(Ornament::TrillWithMordent)
            .with_bracket_length_ss(0.7)
            .with_bracket_direction(HookDirection::Up)
            .with_length_ss(2.5);
        let b = TrillExtensionFullOptions::new()
            .with_length_ss(2.5)
            .with_ornament(Ornament::TrillWithMordent)
            .with_bracket_direction(HookDirection::Up)
            .with_bracket_length_ss(0.7)
            .with_speed(TrillWiggleSpeed::Slow)
            .with_bracket(TrillBracketSide::End);
        assert_eq!(a, b);
    }

    #[test]
    fn with_method_overwrites_prior_value() {
        let opts = TrillExtensionFullOptions::new()
            .with_speed(TrillWiggleSpeed::Standard)
            .with_speed(TrillWiggleSpeed::Slowest);
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Slowest));
    }

    #[test]
    fn with_length_ss_overwrites_prior_value() {
        // Same overwrite semantic as the other with_* methods — last write
        // wins. If a future change accidentally introduced "first write
        // wins" or accumulated values, this canary fires.
        let opts = TrillExtensionFullOptions::new()
            .with_length_ss(1.0)
            .with_length_ss(7.5);
        assert_eq!(opts.length_ss, Some(7.5));
    }

    #[test]
    fn with_length_ss_accepts_non_positive_at_layout_layer() {
        // Layout layer is validation-free (mirrors the policy for
        // unsupported ornaments) — zero and negative values round-trip
        // unchanged. The renderer's existing non-positive fail-safe is
        // what gives them the "no wiggle" semantic at draw time.
        let zero = TrillExtensionFullOptions::new().with_length_ss(0.0);
        assert_eq!(zero.length_ss, Some(0.0));
        let neg = TrillExtensionFullOptions::new().with_length_ss(-2.0);
        assert_eq!(neg.length_ss, Some(-2.0));
    }

    // --- Const constructibility ---

    #[test]
    fn const_constructible() {
        // Every builder method must be `const`-callable so canonical bundles
        // can live in module-level `const` items. Removing `const fn`
        // breaks this compile-time test.
        const _OPTS: TrillExtensionFullOptions = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_bracket_direction(HookDirection::Down)
            .with_bracket_length_ss(0.75)
            .with_speed(TrillWiggleSpeed::Standard)
            .with_ornament(Ornament::TrillWithMordent)
            .with_length_ss(3.0);
    }

    // --- Copy semantics ---

    #[test]
    fn copy_does_not_consume_original() {
        fn take(opts: TrillExtensionFullOptions) -> TrillExtensionFullOptions {
            opts
        }
        let orig = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_bracket_length_ss(1.0)
            .with_speed(TrillWiggleSpeed::Slow);
        let _ = take(orig);
        // Field values must survive the by-value call.
        assert_eq!(orig.bracket, Some(TrillBracketSide::Both));
        assert_eq!(orig.bracket_length_ss, Some(1.0));
        assert_eq!(orig.speed, Some(TrillWiggleSpeed::Slow));
    }

    // --- From conversions ---

    #[test]
    fn from_trill_bracket_options_preserves_bracket_fields() {
        let bracket_only = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_direction(HookDirection::Up)
            .with_length_ss(1.1)
            .with_ornament(Ornament::TrillWithMordent);
        let full: TrillExtensionFullOptions = bracket_only.into();
        assert_eq!(full.bracket, Some(TrillBracketSide::Both));
        assert_eq!(full.bracket_direction, Some(HookDirection::Up));
        assert_eq!(full.bracket_length_ss, Some(1.1));
        assert_eq!(full.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(
            full.speed, None,
            "speed must remain unset — bracket-only options carry no speed"
        );
        assert_eq!(
            full.length_ss, None,
            "length_ss must remain unset — bracket-only options carry no termination length"
        );
    }

    #[test]
    fn from_trill_bracket_options_with_unset_overrides_widens_cleanly() {
        // A bare TrillBracketOptions::new(side) widens to a full bundle
        // with only `bracket` populated — proves the From conversion does
        // not synthesize defaults for `direction`/`length_ss`/`length_ss`.
        let bracket_only = TrillBracketOptions::new(TrillBracketSide::Start);
        let full: TrillExtensionFullOptions = bracket_only.into();
        assert_eq!(full.bracket, Some(TrillBracketSide::Start));
        assert_eq!(full.bracket_direction, None);
        assert_eq!(full.bracket_length_ss, None);
        assert_eq!(full.speed, None);
        assert_eq!(full.ornament, None);
        assert_eq!(full.length_ss, None);
    }

    #[test]
    fn from_speed_options_preserves_speed_and_ornament() {
        let speed_only = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
            .with_ornament(Ornament::TrillWithMordent);
        let full: TrillExtensionFullOptions = speed_only.into();
        assert_eq!(full.speed, Some(TrillWiggleSpeed::Faster));
        assert_eq!(full.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(
            full.bracket, None,
            "bracket must remain unset — speed-only options carry no bracket"
        );
        assert_eq!(full.bracket_direction, None);
        assert_eq!(full.bracket_length_ss, None);
        assert_eq!(
            full.length_ss, None,
            "length_ss must remain unset — speed-only options carry no termination length"
        );
    }

    #[test]
    fn from_speed_options_with_default_ornament_widens_cleanly() {
        // Bare TrillExtensionSpeedOptions::new(speed) widens to a full
        // bundle with only `speed` populated.
        let speed_only = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Slowest);
        let full: TrillExtensionFullOptions = speed_only.into();
        assert_eq!(full.speed, Some(TrillWiggleSpeed::Slowest));
        assert_eq!(full.ornament, None);
        assert_eq!(full.bracket, None);
        assert_eq!(full.bracket_direction, None);
        assert_eq!(full.bracket_length_ss, None);
        assert_eq!(full.length_ss, None);
    }

    #[test]
    fn widen_then_add_length_ss_round_trips_other_fields() {
        // The intended ergonomic path: widen a single-purpose bundle then
        // layer in the missing knob via `.with_length_ss(...)`. The
        // bracket-side fields must survive the chain unchanged.
        let bracket_only = TrillBracketOptions::new(TrillBracketSide::Both)
            .with_direction(HookDirection::Down)
            .with_length_ss(0.8);
        let widened: TrillExtensionFullOptions = bracket_only.into();
        let augmented = widened.with_length_ss(2.5);
        assert_eq!(augmented.bracket, Some(TrillBracketSide::Both));
        assert_eq!(augmented.bracket_direction, Some(HookDirection::Down));
        assert_eq!(
            augmented.bracket_length_ss,
            Some(0.8),
            "bracket hook length must survive the widening + augmentation"
        );
        assert_eq!(augmented.length_ss, Some(2.5));
        assert_eq!(augmented.speed, None);
        assert_eq!(augmented.ornament, None);
    }

    // --- Layout-layer contract: unsupported ornaments round-trip unchanged ---

    #[test]
    fn accepts_unsupported_ornament_at_layout_layer() {
        // Matches the behaviour of the two single-purpose options bundles:
        // the layout struct does not enforce the
        // supports_trill_extension() predicate. The renderer's collector
        // (filtering by that predicate) drops the extension downstream.
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_ornament(Ornament::ShortTrill);
        assert_eq!(opts.ornament, Some(Ornament::ShortTrill));
        assert!(!Ornament::ShortTrill.supports_trill_extension());
    }

    // --- PartialEq sensitivity ---

    #[test]
    fn distinct_brackets_compare_distinct() {
        let a = TrillExtensionFullOptions::new().with_bracket(TrillBracketSide::Start);
        let b = TrillExtensionFullOptions::new().with_bracket(TrillBracketSide::End);
        assert_ne!(a, b);
    }

    #[test]
    fn distinct_speeds_compare_distinct() {
        let a = TrillExtensionFullOptions::new().with_speed(TrillWiggleSpeed::Fast);
        let b = TrillExtensionFullOptions::new().with_speed(TrillWiggleSpeed::Slow);
        assert_ne!(a, b);
    }

    #[test]
    fn distinct_ornaments_compare_distinct() {
        let a = TrillExtensionFullOptions::new().with_ornament(Ornament::Trill);
        let b = TrillExtensionFullOptions::new().with_ornament(Ornament::TrillWithMordent);
        assert_ne!(a, b);
    }

    #[test]
    fn distinct_length_ss_values_compare_distinct() {
        // PartialEq must be sensitive to length_ss — catches a regression
        // where the derive ever forgot the new field.
        let a = TrillExtensionFullOptions::new().with_length_ss(1.0);
        let b = TrillExtensionFullOptions::new().with_length_ss(2.0);
        assert_ne!(a, b);
    }

    #[test]
    fn length_ss_some_zero_distinct_from_none() {
        // `Some(0.0)` is semantically distinct from `None` (it suppresses
        // the wiggle at the renderer's non-positive fail-safe, while `None`
        // means "use the natural span"). The PartialEq derive must reflect
        // that — they must not collapse.
        let none = TrillExtensionFullOptions::new();
        let zero = TrillExtensionFullOptions::new().with_length_ss(0.0);
        assert_ne!(none, zero);
    }

    // --- TrillExtensionFullOptions: non-breaking `extension_length_ss` alias for `length_ss` ---
    //
    // On this struct, `length_ss` is the wiggle's extension termination
    // length, while `bracket_length_ss` is the bracket hook length — the
    // opposite naming convention from `TrillBracketOptions`. The bare name
    // `length_ss` is ambiguous next to `bracket_length_ss`; the alias
    // `extension_length_ss` (setter `with_extension_length_ss`, getter
    // `extension_length_ss`) names the wiggle-extension semantics explicitly
    // without breaking the existing API. The two setters write the same
    // field; these tests lock that in. Mirrors the
    // `TrillBracketOptions::with_hook_length_ss` alias pattern from the prior
    // chunk.

    #[test]
    fn with_extension_length_ss_writes_to_length_ss_field() {
        // The new setter must populate the existing `length_ss` field — not a
        // separate parallel field. Catches a refactor that accidentally
        // introduces a phantom `extension_length_ss` field that diverges from
        // `length_ss` at the storage layer.
        let opts = TrillExtensionFullOptions::new().with_extension_length_ss(2.75);
        assert_eq!(opts.length_ss, Some(2.75));
        // And no other knob is touched.
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
    }

    #[test]
    fn with_extension_length_ss_is_byte_equivalent_to_with_length_ss() {
        // Both setters must produce structurally identical options. If a
        // future refactor splits them into different field assignments, this
        // catches it directly. PartialEq derive covers every field — the
        // assertion fires if any field diverges.
        for &len in &[0.0, 0.5, 0.75, 1.0, 1.5, 3.5, -2.0] {
            let via_legacy = TrillExtensionFullOptions::new().with_length_ss(len);
            let via_new = TrillExtensionFullOptions::new().with_extension_length_ss(len);
            assert_eq!(
                via_legacy, via_new,
                "with_length_ss({len}) and with_extension_length_ss({len}) must produce equal options"
            );
        }
    }

    #[test]
    fn with_extension_length_ss_overwrites_with_length_ss_when_chained() {
        // Last-write-wins canary: a caller chaining both setters lands on
        // whichever was called last. The two are aliases, so this is the
        // expected sequential-mutation semantic.
        let opts_last_new = TrillExtensionFullOptions::new()
            .with_length_ss(0.5)
            .with_extension_length_ss(1.2);
        assert_eq!(opts_last_new.length_ss, Some(1.2));

        let opts_last_legacy = TrillExtensionFullOptions::new()
            .with_extension_length_ss(1.2)
            .with_length_ss(0.5);
        assert_eq!(opts_last_legacy.length_ss, Some(0.5));
    }

    #[test]
    fn extension_length_ss_getter_returns_length_ss_field() {
        // Getter must return the same Option<f64> the field holds, including
        // the `None` default.
        let unset = TrillExtensionFullOptions::new();
        assert_eq!(unset.extension_length_ss(), None);
        assert_eq!(unset.extension_length_ss(), unset.length_ss);

        let via_legacy = TrillExtensionFullOptions::new().with_length_ss(0.6);
        assert_eq!(via_legacy.extension_length_ss(), Some(0.6));
        assert_eq!(via_legacy.extension_length_ss(), via_legacy.length_ss);

        let via_new = TrillExtensionFullOptions::new().with_extension_length_ss(0.6);
        assert_eq!(via_new.extension_length_ss(), Some(0.6));
        assert_eq!(via_new.extension_length_ss(), via_new.length_ss);
    }

    #[test]
    fn with_extension_length_ss_is_distinct_from_with_bracket_length_ss() {
        // Critical naming-disambiguation canary, mirroring the analogous
        // `with_hook_length_ss_is_distinct_from_with_extension_length_ss`
        // test on `TrillBracketOptions`. The two methods MUST write to
        // different fields — the wiggle's extension length and the bracket
        // hook length are semantically distinct knobs even though both are
        // lengths in staff spaces. This is the inverse-direction
        // disambiguation that motivates having this alias on
        // `TrillExtensionFullOptions` at all.
        let ext_only = TrillExtensionFullOptions::new().with_extension_length_ss(1.0);
        let hook_only = TrillExtensionFullOptions::new().with_bracket_length_ss(1.0);

        assert_eq!(ext_only.length_ss, Some(1.0));
        assert_eq!(ext_only.bracket_length_ss, None);
        assert_eq!(hook_only.length_ss, None);
        assert_eq!(hook_only.bracket_length_ss, Some(1.0));
        assert_ne!(
            ext_only, hook_only,
            "wiggle extension length and bracket hook length must be independent fields"
        );
    }

    #[test]
    fn with_extension_length_ss_chains_with_other_setters() {
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::End)
            .with_bracket_direction(HookDirection::Up)
            .with_bracket_length_ss(0.7)
            .with_speed(TrillWiggleSpeed::Slow)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(3.25);
        assert_eq!(opts.bracket, Some(TrillBracketSide::End));
        assert_eq!(opts.bracket_direction, Some(HookDirection::Up));
        assert_eq!(opts.bracket_length_ss, Some(0.7));
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Slow));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.length_ss, Some(3.25));
        assert_eq!(opts.extension_length_ss(), Some(3.25));
    }

    #[test]
    fn with_extension_length_ss_chain_order_independent_from_other_setters() {
        // Mirrors `chain_order_independent` for the new setter: calling order
        // must not affect the resulting struct.
        let a = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_extension_length_ss(2.5)
            .with_speed(TrillWiggleSpeed::Faster)
            .with_ornament(Ornament::TrillWithMordent);
        let b = TrillExtensionFullOptions::new()
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(2.5)
            .with_speed(TrillWiggleSpeed::Faster)
            .with_bracket(TrillBracketSide::Both);
        let c = TrillExtensionFullOptions::new()
            .with_speed(TrillWiggleSpeed::Faster)
            .with_bracket(TrillBracketSide::Both)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(2.5);
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn with_extension_length_ss_is_const_constructible() {
        // Mirror of `const_constructible` for the new setter. Locks in
        // `const fn` — a future change that drops `const` would silently
        // disqualify the new method from `const` items at module scope; this
        // test stops compiling in that case.
        const _OPTS: TrillExtensionFullOptions = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_extension_length_ss(3.0);
    }

    #[test]
    fn extension_length_ss_getter_is_const_callable() {
        // The getter must be `const fn` for the same reason — symmetry with
        // the setter and to let callers read defaults at const-eval time.
        const _LEN: Option<f64> = TrillExtensionFullOptions::new().extension_length_ss();
        // The accessor on the `None` default must yield `None` — basic value
        // check beyond the bare "compiles in const context" guarantee.
        assert_eq!(_LEN, None);
    }

    #[test]
    fn extension_length_ss_getter_after_widening_from_bracket_options() {
        // Widening a `TrillBracketOptions` that carries an
        // `extension_length_ss` must populate `TrillExtensionFullOptions::length_ss`
        // — and therefore the new `extension_length_ss()` accessor must
        // surface it correctly. Locks in the source-of-truth chain:
        //   TrillBracketOptions::extension_length_ss
        //     → TrillExtensionFullOptions::length_ss
        //     → TrillExtensionFullOptions::extension_length_ss()
        let bracket = TrillBracketOptions::new(TrillBracketSide::End)
            .with_extension_length_ss(4.0);
        let widened: TrillExtensionFullOptions = bracket.into();
        assert_eq!(widened.extension_length_ss(), Some(4.0));
        assert_eq!(widened.extension_length_ss(), widened.length_ss);
    }

    // --- TrillSpeedRampSpec integration: multi-speed ramp options ---
    //
    // Adds a `speed_ramp: Option<TrillSpeedRampSpec>` field plus
    // `with_speed_ramp` / `with_speed_ramp_ramp_count` setters. The
    // dispatch contract is documented on the field: when
    // `speed_ramp.is_some()` the renderer uses the multi-speed path and
    // the single-speed `speed` field is ignored; both fields are allowed
    // to coexist so widening + layering composes additively.

    #[test]
    fn ramp_spec_new_round_trips_fields() {
        // Basic constructor canary: the public field values come out the
        // same shape they went in. Catches a refactor that reordered the
        // struct fields without updating `new`.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let spec = TrillSpeedRampSpec::new(ramp, 5);
        assert_eq!(spec.ramp, ramp);
        assert_eq!(spec.region_count, 5);
    }

    #[test]
    fn ramp_spec_is_const_constructible() {
        // `TrillSpeedRampSpec::new` must be `const fn` so canonical specs
        // can live in module-level `const` items alongside the ramp.
        const _SPEC: TrillSpeedRampSpec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
            3,
        );
        const _SPEC_LINEAR: TrillSpeedRampSpec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            7,
        );
    }

    #[test]
    fn ramp_spec_partial_eq_sensitive_to_ramp() {
        // Same region_count, different ramp → distinct specs.
        let a = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Fast),
            4,
        );
        let b = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Slow),
            4,
        );
        assert_ne!(a, b);
    }

    #[test]
    fn ramp_spec_partial_eq_sensitive_to_region_count() {
        // Same ramp, different region_count → distinct specs. Catches a
        // PartialEq derive that ever dropped a field.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        assert_ne!(
            TrillSpeedRampSpec::new(ramp, 3),
            TrillSpeedRampSpec::new(ramp, 5)
        );
    }

    #[test]
    fn with_speed_ramp_sets_only_speed_ramp() {
        // Setter isolation: setting `speed_ramp` does not touch any other
        // field. Mirrors the analogous isolation tests for the other
        // setters.
        let spec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        );
        let opts = TrillExtensionFullOptions::new().with_speed_ramp(spec);
        assert_eq!(opts.speed_ramp, Some(spec));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_speed_ramp_ramp_count_sets_only_speed_ramp() {
        // The two-arg variant must isolate the same way the spec-typed
        // variant does — particularly that it does not write into the
        // single-speed `speed` field. Catches a regression where the
        // sugar accidentally fanned out into two fields.
        let opts = TrillExtensionFullOptions::new().with_speed_ramp_ramp_count(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Faster),
            4,
        );
        assert_eq!(
            opts.speed_ramp,
            Some(TrillSpeedRampSpec::new(
                TrillSpeedRamp::constant(TrillWiggleSpeed::Faster),
                4
            ))
        );
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_speed_ramp_ramp_count_byte_equivalent_to_with_speed_ramp_new() {
        // The two-arg sugar must produce a struct field-by-field equal to
        // the spec-typed form. The PartialEq derive covers every field;
        // assertion fires if they diverge. Locks in the byte-equivalence
        // claim in the docstring.
        for region_count in [1usize, 2, 3, 7] {
            for ramp in [
                TrillSpeedRamp::constant(TrillWiggleSpeed::Fastest),
                TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slowest, TrillWiggleSpeed::Fastest),
                TrillSpeedRamp::linear(TrillWiggleSpeed::Fast, TrillWiggleSpeed::Slow),
            ] {
                let spec_form = TrillExtensionFullOptions::new()
                    .with_speed_ramp(TrillSpeedRampSpec::new(ramp, region_count));
                let sugar_form = TrillExtensionFullOptions::new()
                    .with_speed_ramp_ramp_count(ramp, region_count);
                assert_eq!(
                    spec_form, sugar_form,
                    "ramp={ramp:?} region_count={region_count}"
                );
            }
        }
    }

    #[test]
    fn with_speed_ramp_overwrites_prior_value() {
        // Same last-write-wins semantic as the other setters. Catches a
        // regression where the setter accumulated into a `Vec` or kept the
        // first write.
        let first = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Fast),
            3,
        );
        let second = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            7,
        );
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp(first)
            .with_speed_ramp(second);
        assert_eq!(opts.speed_ramp, Some(second));
        // Sanity: confirm the two specs aren't accidentally equal — the
        // overwrite test only carries weight when they differ.
        assert_ne!(first, second);
    }

    #[test]
    fn with_speed_ramp_overwrites_with_speed_ramp_ramp_count() {
        // Cross-setter last-write-wins: chaining the two variants in
        // either order ends on the last call's value. Both setters write
        // the same field, so this must hold.
        let spec_form = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Fastest),
            2,
        );
        let opts_a = TrillExtensionFullOptions::new()
            .with_speed_ramp_ramp_count(
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
                5,
            )
            .with_speed_ramp(spec_form);
        assert_eq!(opts_a.speed_ramp, Some(spec_form));

        let opts_b = TrillExtensionFullOptions::new()
            .with_speed_ramp(spec_form)
            .with_speed_ramp_ramp_count(
                TrillSpeedRamp::constant(TrillWiggleSpeed::Slowest),
                4,
            );
        assert_eq!(
            opts_b.speed_ramp,
            Some(TrillSpeedRampSpec::new(
                TrillSpeedRamp::constant(TrillWiggleSpeed::Slowest),
                4
            ))
        );
    }

    #[test]
    fn with_speed_ramp_chains_with_other_setters() {
        // Layering the new setter on top of an otherwise-populated bundle
        // must leave the other six fields intact and populate
        // `speed_ramp`. Locks in the contract that this setter is
        // additive.
        let spec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slowest, TrillWiggleSpeed::Fastest),
            5,
        );
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_bracket_direction(HookDirection::Down)
            .with_bracket_length_ss(0.8)
            .with_speed(TrillWiggleSpeed::Standard)
            .with_ornament(Ornament::TrillWithMordent)
            .with_length_ss(3.0)
            .with_speed_ramp(spec);
        assert_eq!(opts.bracket, Some(TrillBracketSide::Both));
        assert_eq!(opts.bracket_direction, Some(HookDirection::Down));
        assert_eq!(opts.bracket_length_ss, Some(0.8));
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Standard));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.length_ss, Some(3.0));
        assert_eq!(opts.speed_ramp, Some(spec));
    }

    #[test]
    fn with_speed_ramp_chain_order_independent_from_other_setters() {
        // The seven setters all commute. Three different orderings of the
        // same setter calls must yield equal bundles. Catches a
        // regression where any setter accidentally cleared a sibling.
        let spec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Slow),
            4,
        );
        let a = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::End)
            .with_speed_ramp(spec)
            .with_speed(TrillWiggleSpeed::Fast)
            .with_ornament(Ornament::TrillWithMordent);
        let b = TrillExtensionFullOptions::new()
            .with_ornament(Ornament::TrillWithMordent)
            .with_speed(TrillWiggleSpeed::Fast)
            .with_speed_ramp(spec)
            .with_bracket(TrillBracketSide::End);
        let c = TrillExtensionFullOptions::new()
            .with_speed(TrillWiggleSpeed::Fast)
            .with_bracket(TrillBracketSide::End)
            .with_ornament(Ornament::TrillWithMordent)
            .with_speed_ramp(spec);
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn with_speed_ramp_is_const_constructible() {
        // Locks in `const fn` on the new setter. A future change that
        // dropped `const` would break this compile-time canary.
        const _OPTS: TrillExtensionFullOptions = TrillExtensionFullOptions::new()
            .with_speed_ramp(TrillSpeedRampSpec::new(
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slowest, TrillWiggleSpeed::Fastest),
                4,
            ));
        const _OPTS_SUGAR: TrillExtensionFullOptions =
            TrillExtensionFullOptions::new().with_speed_ramp_ramp_count(
                TrillSpeedRamp::constant(TrillWiggleSpeed::Faster),
                3,
            );
    }

    #[test]
    fn speed_ramp_and_speed_can_coexist_on_options() {
        // Per the field's documented dispatch: `speed_ramp.is_some()`
        // wins, but the single-speed `speed` is allowed to remain
        // populated (so widening from `TrillExtensionSpeedOptions` then
        // adding a ramp doesn't have to clear `speed` first). This test
        // pins down the "no field reset" property at the options layer —
        // the renderer's dispatch is tested separately when wired up.
        let spec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        );
        let opts = TrillExtensionFullOptions::new()
            .with_speed(TrillWiggleSpeed::Slowest)
            .with_speed_ramp(spec);
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Slowest));
        assert_eq!(opts.speed_ramp, Some(spec));
    }

    #[test]
    fn with_speed_does_not_clear_speed_ramp() {
        // Symmetric guard: layering `.with_speed(...)` on top of an
        // already-populated `speed_ramp` must not silently drop the ramp.
        // Catches a regression that "promoted" the single-speed setter
        // into a multi-speed reset.
        let spec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Fastest),
            2,
        );
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp(spec)
            .with_speed(TrillWiggleSpeed::Standard);
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Standard));
        assert_eq!(opts.speed_ramp, Some(spec));
    }

    #[test]
    fn from_bracket_options_leaves_speed_ramp_none() {
        // `TrillBracketOptions` has no ramp field, so the widening
        // conversion must leave `speed_ramp` as `None`. Pins down the
        // From-impl explicit None propagation. Catches a regression that
        // synthesized a "default ramp" — which would silently engage the
        // multi-speed renderer path for callers who only asked for a
        // bracket.
        let bracket_only = TrillBracketOptions::new(TrillBracketSide::Start)
            .with_direction(HookDirection::Up)
            .with_length_ss(1.5)
            .with_ornament(Ornament::TrillWithMordent);
        let full: TrillExtensionFullOptions = bracket_only.into();
        assert_eq!(full.speed_ramp, None);
    }

    #[test]
    fn from_speed_options_leaves_speed_ramp_none() {
        // `TrillExtensionSpeedOptions` is a single-speed bundle by
        // construction; widening must leave `speed_ramp` as `None`. The
        // `speed` field is set, the ramp is not — the renderer dispatches
        // on `speed_ramp.is_some()`, so a widening that synthesized a
        // ramp would silently move the trill to the multi-speed path.
        let speed_only = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Faster)
            .with_ornament(Ornament::TrillWithMordent);
        let full: TrillExtensionFullOptions = speed_only.into();
        assert_eq!(full.speed_ramp, None);
    }

    #[test]
    fn widen_from_speed_options_then_add_ramp() {
        // The intended ergonomic path: widen a `TrillExtensionSpeedOptions`
        // to a full bundle, then layer in a ramp. The speed field must
        // survive (the dispatch contract permits speed + ramp to coexist).
        let speed_only = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard);
        let widened: TrillExtensionFullOptions = speed_only.into();
        let spec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            5,
        );
        let augmented = widened.with_speed_ramp(spec);
        assert_eq!(augmented.speed, Some(TrillWiggleSpeed::Standard));
        assert_eq!(augmented.speed_ramp, Some(spec));
    }

    #[test]
    fn distinct_speed_ramp_specs_compare_distinct() {
        // PartialEq must be sensitive to the new field — catches a
        // derive that forgot to pick it up if the field were renamed and
        // the derive ever fell out of sync.
        let a = TrillExtensionFullOptions::new().with_speed_ramp(TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Fast),
            3,
        ));
        let b = TrillExtensionFullOptions::new().with_speed_ramp(TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Slow),
            3,
        ));
        assert_ne!(a, b);
    }

    #[test]
    fn speed_ramp_some_distinct_from_none() {
        // `Some(spec)` is semantically distinct from `None` (the
        // single-speed path vs. the multi-speed path at draw time). The
        // PartialEq derive must reflect that — same shape canary as
        // `length_ss_some_zero_distinct_from_none` for the length field.
        let none = TrillExtensionFullOptions::new();
        let some = TrillExtensionFullOptions::new().with_speed_ramp(TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
            1,
        ));
        assert_ne!(none, some);
    }

    #[test]
    fn other_setters_do_not_write_to_speed_ramp() {
        // Cross-setter isolation canary: every other `with_*` setter
        // must leave `speed_ramp` unset. The existing per-setter tests
        // each assert other fields are unset but predate this field;
        // this test closes that gap in one place.
        let opts_bracket = TrillExtensionFullOptions::new().with_bracket(TrillBracketSide::End);
        let opts_dir =
            TrillExtensionFullOptions::new().with_bracket_direction(HookDirection::Up);
        let opts_blen = TrillExtensionFullOptions::new().with_bracket_length_ss(0.9);
        let opts_speed = TrillExtensionFullOptions::new().with_speed(TrillWiggleSpeed::Fast);
        let opts_orn = TrillExtensionFullOptions::new().with_ornament(Ornament::Trill);
        let opts_len = TrillExtensionFullOptions::new().with_length_ss(2.0);
        let opts_ext_len =
            TrillExtensionFullOptions::new().with_extension_length_ss(2.0);
        for opts in [
            opts_bracket,
            opts_dir,
            opts_blen,
            opts_speed,
            opts_orn,
            opts_len,
            opts_ext_len,
        ] {
            assert_eq!(opts.speed_ramp, None);
        }
    }

    #[test]
    fn default_speed_ramp_is_none() {
        // The derived `Default` must agree with `new()` on the new
        // field. The existing `default_matches_new` covers full-struct
        // equality; this is a targeted canary that fails with a clearer
        // message if the derive's behavior for the new field ever
        // diverges from `new()`.
        assert_eq!(TrillExtensionFullOptions::default().speed_ramp, None);
    }

    #[test]
    fn ramp_spec_carries_degenerate_inputs_unchanged() {
        // The spec stores raw inputs; rejection happens at
        // `synthesize_regions` time. `region_count = 0` and a `Linear`
        // ramp with `region_count = 1` both round-trip through the
        // options bundle unchanged. Locks in the
        // "no construction-time validation" policy that mirrors
        // unsupported-ornament and non-positive-length handling.
        let degenerate_zero = TrillSpeedRampSpec::new(
            TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
            0,
        );
        let opts_zero = TrillExtensionFullOptions::new().with_speed_ramp(degenerate_zero);
        assert_eq!(opts_zero.speed_ramp, Some(degenerate_zero));
        assert_eq!(opts_zero.speed_ramp.unwrap().region_count, 0);

        let degenerate_one_linear = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            1,
        );
        let opts_one =
            TrillExtensionFullOptions::new().with_speed_ramp(degenerate_one_linear);
        assert_eq!(opts_one.speed_ramp, Some(degenerate_one_linear));
        assert_eq!(opts_one.speed_ramp.unwrap().region_count, 1);
    }

    #[test]
    fn spec_in_options_synthesizes_regions_at_draw_time() {
        // End-to-end contract canary: a `TrillSpeedRampSpec` carried in
        // options must feed cleanly into `synthesize_regions` at draw
        // time without massaging. Mirrors
        // `ramp_synthesized_regions_feed_into_multi_speed_layout` in the
        // trill_extension test module but routed through the options
        // bundle as the source of truth.
        let spec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            3,
        );
        let opts = TrillExtensionFullOptions::new().with_speed_ramp(spec);
        // Retrieve at "draw time": unpack from options and call the
        // synthesizer with a dummy font advance (each speed advances 50
        // font units in the dummy lookup; real callers thread their
        // active font's glyph-advance closure here).
        let unpacked = opts.speed_ramp.expect("speed_ramp populated by builder");
        let regions = unpacked
            .ramp
            .synthesize_regions(0.0, 300.0, unpacked.region_count, |_speed| 50.0)
            .expect("synthesize_regions accepts non-degenerate spec");
        assert_eq!(regions.len(), 3);
        // Synthesized region starts evenly spaced across [0, 300] for
        // region_count = 3.
        assert_eq!(regions[0].start_x, 0.0);
        assert_eq!(regions[1].start_x, 100.0);
        assert_eq!(regions[2].start_x, 200.0);
        // Linear Slow → Fast over 3 regions → glyphs progress
        // from `WiggleTrillSlow` (index 5) through `WiggleTrill`
        // (index 4) to `WiggleTrillFast` (index 3).
        assert_eq!(regions[0].glyph, TrillWiggleSpeed::Slow.to_glyph());
        assert_eq!(regions[1].glyph, TrillWiggleSpeed::Standard.to_glyph());
        assert_eq!(regions[2].glyph, TrillWiggleSpeed::Fast.to_glyph());
    }
}
