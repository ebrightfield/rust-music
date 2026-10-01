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

    /// Stricter counterpart to
    /// [`with_bracket_length_ss`](Self::with_bracket_length_ss): rejects at
    /// the options-bundle construction site the hook lengths that either
    /// produce a degenerate (invisible) hook or that the renderer silently
    /// folds via [`f64::abs`], returning `Option<Self>`.
    ///
    /// **This validator's accept band is intentionally narrower than the
    /// renderer's**, unlike
    /// [`with_extension_length_ss_validated`](Self::with_extension_length_ss_validated)
    /// (this same bundle) which mirrors its renderer's accept band exactly.
    /// The [`layout_trill_bracket_hook`](crate::layout::trill_bracket::layout_trill_bracket_hook)
    /// renderer applies `let length = length.abs();` — negative lengths
    /// silently flip to positive while the explicit
    /// [`HookDirection`](crate::layout::HookDirection) wins. That fold makes
    /// `.with_bracket_length_ss(-1.0)` indistinguishable from
    /// `.with_bracket_length_ss(1.0)` at draw time, so the "explicit-flip
    /// intent" of the caller (probably meaning "flip to Up") is silently
    /// overridden. This validator surfaces that misuse at construction time
    /// by rejecting negatives outright; callers wanting a flipped hook
    /// should pass [`HookDirection::Up`](crate::layout::HookDirection::Up)
    /// to [`with_bracket_direction`](Self::with_bracket_direction).
    ///
    /// Zero is also rejected: a 0.0 length produces a degenerate hook with
    /// `y_top == y_bottom` (no visible line). NaN is rejected because
    /// `NaN > 0.0` is false and the renderer would otherwise emit NaN hook
    /// coordinates.
    ///
    /// The accept-band predicate is `length_ss > 0.0`, numerically
    /// identical to this bundle's
    /// [`with_extension_length_ss_validated`](Self::with_extension_length_ss_validated)
    /// despite the different rationale (extension-length: renderer collapses
    /// to no-wiggle; bracket-length: renderer silently folds and degenerates
    /// at zero).
    ///
    /// Rejection rules (`None` returned):
    /// - `length_ss == 0.0` (positive or negative zero — both fail
    ///   `> 0.0` and both produce a degenerate hook).
    /// - `length_ss < 0.0` (any negative finite, including `-∞`). The
    ///   renderer would silently fold via `.abs()`, overriding the caller's
    ///   apparent flip intent.
    /// - `length_ss.is_nan()` (any NaN payload — NaN comparisons return
    ///   false, so `NaN > 0.0` is false; the renderer would otherwise emit
    ///   NaN hook coordinates).
    ///
    /// Acceptance (`Some(self)` returned with `bracket_length_ss` populated):
    /// - `length_ss > 0.0` (any finite positive value, plus `+∞` —
    ///   matching the permissive setter's storage behaviour byte-for-byte).
    /// - All other fields on `self` (including `bracket`, `bracket_direction`,
    ///   `speed`, `speed_ramp`, `ornament`, `length_ss`) are preserved
    ///   unchanged (additive contract, matching every other
    ///   validator-pairing on this bundle).
    ///
    /// Writes to the [`bracket_length_ss`](Self::bracket_length_ss) field
    /// (the bracket hook length). Distinct from
    /// [`length_ss`](Self::length_ss) (the wiggle's *extension* termination
    /// length, controlled by
    /// [`with_extension_length_ss_validated`](Self::with_extension_length_ss_validated)).
    /// The naming-disambiguation invariant from the permissive setter is
    /// preserved.
    ///
    /// `const`-callable, matching every other setter on this bundle. On the
    /// `None` branch the builder chain is broken at the call site and the
    /// partially-built bundle is dropped — there is no fallback that
    /// silently leaves `bracket_length_ss` unset, because that would demote
    /// a rejection into a no-op.
    ///
    /// Mirrors the validator-pairing pattern at
    /// [`crate::layout::trill_bracket::TrillBracketOptions::with_hook_length_ss_validated`]
    /// (which targets the corresponding `length_ss` field on the
    /// single-purpose bracket-options bundle).
    /// [`crate::layout::trill_extension::TrillExtensionSpeedOptions`] has no
    /// hook-length field and therefore no corresponding validator.
    pub const fn with_bracket_length_ss_validated(mut self, length_ss: f64) -> Option<Self> {
        // Tighter than the renderer's accept band by deliberate choice — the
        // renderer's `abs()` fold makes negatives indistinguishable from
        // their positive counterparts at draw time, silently overriding the
        // caller's apparent flip intent. Rejecting at construction time
        // surfaces the misuse and steers callers toward
        // `HookDirection::{Up, Down}` for explicit direction control.
        if length_ss > 0.0 {
            self.bracket_length_ss = Some(length_ss);
            Some(self)
        } else {
            None
        }
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
    /// - Every other field on `self` (bracket-related fields, speed,
    ///   length_ss, speed_ramp) is preserved unchanged (additive contract,
    ///   matching the permissive setter and every other validator-pairing
    ///   on this bundle, e.g.
    ///   [`with_speed_ramp_validated_ramp_count`](Self::with_speed_ramp_validated_ramp_count)).
    ///
    /// `const`-callable, matching every other setter on this bundle. On the
    /// `None` branch the builder chain is broken at the call site and the
    /// partially-built bundle is dropped — there is no fallback that
    /// silently leaves `ornament` unset, because that would demote a
    /// rejection into a no-op.
    ///
    /// Mirrors the validator-pairing pattern at
    /// [`crate::layout::trill_bracket::TrillBracketOptions::with_ornament_validated`]
    /// and [`crate::layout::trill_extension::TrillExtensionSpeedOptions::with_ornament_validated`]
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
    /// Like its permissive counterpart, this validator writes to the
    /// [`length_ss`](Self::length_ss) field, *not* to
    /// [`bracket_length_ss`](Self::bracket_length_ss) (which controls the
    /// bracket hook length). The naming-disambiguation contract from the
    /// permissive setter is preserved.
    ///
    /// Rejection rules (`None` returned):
    /// - `length_ss == 0.0` (positive or negative zero — both fail
    ///   `length_ss > 0.0`).
    /// - `length_ss < 0.0` (any negative finite, including `-∞`).
    /// - `length_ss.is_nan()` (any NaN payload — NaN comparisons return
    ///   false).
    ///
    /// Acceptance (`Some(self)` returned with `length_ss` populated):
    /// - `length_ss > 0.0` (any finite positive value, plus `+∞`).
    /// - All other fields on `self` (including `bracket`, `bracket_direction`,
    ///   `bracket_length_ss`, `speed`, `speed_ramp`, `ornament`) are
    ///   preserved unchanged (additive contract, matching the permissive
    ///   setter byte-for-byte).
    ///
    /// `const`-callable, matching every other setter on this bundle. On the
    /// `None` branch the builder chain is broken at the call site and the
    /// partially-built bundle is dropped — there is no fallback that
    /// silently leaves `length_ss` unset, because that would demote a
    /// rejection into a no-op.
    ///
    /// Mirrors the validator-pairing pattern at
    /// [`crate::layout::trill_bracket::TrillBracketOptions::with_extension_length_ss_validated`]
    /// and
    /// [`crate::layout::trill_extension::TrillExtensionSpeedOptions::with_extension_length_ss_validated`]
    /// — all three options bundles expose the same
    /// `with_extension_length_ss` / `with_extension_length_ss_validated`
    /// pair.
    pub const fn with_extension_length_ss_validated(mut self, length_ss: f64) -> Option<Self> {
        // Mirror the renderer's accept band literally — any future change to
        // the renderer's "would draw" check (e.g. tightening to a minimum
        // tile-width) needs to update this clause in lockstep.
        if length_ss > 0.0 {
            self.length_ss = Some(length_ss);
            Some(self)
        } else {
            None
        }
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
    ///
    /// This setter stores any `(ramp, region_count)` pair unchanged so the
    /// bundle can travel through annotation pipelines whose validity is only
    /// checked at draw time (mirroring [`TrillSpeedRampSpec::new`] and
    /// [`TrillSpeedRamp::linear`]). Callers wanting construction-time
    /// rejection of the degenerate inputs documented at
    /// [`TrillSpeedRampSpec::new_validated`] should use
    /// [`with_speed_ramp_validated_ramp_count`](Self::with_speed_ramp_validated_ramp_count),
    /// which returns `Option<Self>` and rejects the same inputs that
    /// [`TrillSpeedRamp::synthesize_regions`] would reject at draw time.
    pub const fn with_speed_ramp_ramp_count(
        mut self,
        ramp: TrillSpeedRamp,
        region_count: usize,
    ) -> Self {
        self.speed_ramp = Some(TrillSpeedRampSpec::new(ramp, region_count));
        self
    }

    /// Stricter counterpart to
    /// [`with_speed_ramp_ramp_count`](Self::with_speed_ramp_ramp_count):
    /// rejects the degenerate `(ramp, region_count)` pairs at the
    /// options-bundle construction site, returning `Option<Self>`.
    ///
    /// Byte-equivalent on the `Some` branch to
    /// `with_speed_ramp(TrillSpeedRampSpec::new_validated(ramp, region_count)?)` —
    /// the validator is a thin pass-through to
    /// [`TrillSpeedRampSpec::new_validated`], not an independent check, so
    /// the rejection rules are exactly that method's:
    ///
    /// - `None` when `region_count == 0` (no regions to emit for *any* ramp).
    /// - `None` when `ramp` is [`TrillSpeedRamp::Linear`] and
    ///   `region_count < 2` (a single-region linear progression is
    ///   ill-defined; [`TrillSpeedRamp::synthesize_regions`] would itself
    ///   return `None`).
    /// - `Some(self)` (with `speed_ramp` populated) otherwise.
    ///
    /// All other fields on `self` are preserved unchanged on the `Some`
    /// branch — the validator is additive, mirroring the existing
    /// `with_speed_ramp_ramp_count` contract. On the `None` branch the
    /// builder chain is broken at the call site; the partially-populated
    /// bundle is dropped (consistent with the `Option<Self>` shape — there
    /// is no "leave `speed_ramp` unset and return `Some(self)`" fallback,
    /// because that would silently demote the multi-speed setter into a
    /// no-op).
    ///
    /// As with the permissive setter, the single-speed [`speed`](Self::speed)
    /// field is *not* cleared on accept — the dispatch contract permits
    /// `speed` and `speed_ramp` to coexist (the renderer dispatches on
    /// `speed_ramp.is_some()`).
    ///
    /// `const`-callable, matching the `const` shape of every other setter
    /// on this bundle and of [`TrillSpeedRampSpec::new_validated`] itself.
    /// Mirrors the
    /// [`TrillSpeedRamp::linear`] / [`TrillSpeedRamp::linear_validated`] and
    /// [`TrillSpeedRampSpec::new`] / [`TrillSpeedRampSpec::new_validated`]
    /// pairing one layer higher up the stack.
    pub const fn with_speed_ramp_validated_ramp_count(
        mut self,
        ramp: TrillSpeedRamp,
        region_count: usize,
    ) -> Option<Self> {
        // Delegate to the underlying spec validator: any rejection it
        // performs is mirrored here, and any acceptance produces a fully-
        // formed spec that we drop into `speed_ramp` without further
        // massaging. Locked together by this single call site — narrowing
        // either validator narrows both.
        match TrillSpeedRampSpec::new_validated(ramp, region_count) {
            Some(spec) => {
                self.speed_ramp = Some(spec);
                Some(self)
            }
            None => None,
        }
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

    // --- TrillExtensionFullOptions ornament validator ---

    #[test]
    fn with_ornament_validated_accepts_trill() {
        // Canonical accept case: plain Trill on a fresh bundle. Every
        // unset field stays unset on accept.
        let opts = TrillExtensionFullOptions::new().with_ornament_validated(Ornament::Trill);
        let opts = opts.expect("Trill must be accepted");
        assert_eq!(opts.ornament, Some(Ornament::Trill));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.length_ss, None);
        assert_eq!(opts.speed_ramp, None);
    }

    #[test]
    fn with_ornament_validated_accepts_trill_with_mordent() {
        // The other accept variant.
        let opts =
            TrillExtensionFullOptions::new().with_ornament_validated(Ornament::TrillWithMordent);
        let opts = opts.expect("TrillWithMordent must be accepted");
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
    }

    #[test]
    fn with_ornament_validated_rejects_short_trill() {
        // Headline rejection case for the full-options surface.
        let opts = TrillExtensionFullOptions::new().with_ornament_validated(Ornament::ShortTrill);
        assert!(opts.is_none(), "ShortTrill must be rejected");
    }

    #[test]
    fn with_ornament_validated_rejects_every_non_supporting_variant() {
        // Walk all 15 ornaments and cross-validate `is_none()` iff
        // `!supports_trill_extension()`. Locks the validator's accept band
        // to the predicate's accept band.
        for &ornament in Ornament::ALL.iter() {
            let result = TrillExtensionFullOptions::new().with_ornament_validated(ornament);
            assert_eq!(
                result.is_some(),
                ornament.supports_trill_extension(),
                "{ornament:?}: validator accept must match supports_trill_extension"
            );
        }
    }

    #[test]
    fn with_ornament_validated_some_branch_byte_equals_unvalidated() {
        // For every accepted ornament, layered on top of a fully-populated
        // bundle, the validated and permissive setters must produce
        // field-by-field equal bundles. Catches a future refactor that
        // started normalizing accepted inputs.
        for &ornament in Ornament::ALL
            .iter()
            .filter(|o| o.supports_trill_extension())
        {
            let permissive = TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_bracket_direction(HookDirection::Up)
                .with_bracket_length_ss(0.85)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_length_ss(4.0)
                .with_ornament(ornament);
            let validated = TrillExtensionFullOptions::new()
                .with_bracket(TrillBracketSide::Both)
                .with_bracket_direction(HookDirection::Up)
                .with_bracket_length_ss(0.85)
                .with_speed(TrillWiggleSpeed::Slow)
                .with_length_ss(4.0)
                .with_ornament_validated(ornament)
                .expect("supported ornament must accept");
            assert_eq!(
                permissive, validated,
                "{ornament:?}: validated vs permissive byte-equal"
            );
        }
    }

    #[test]
    fn with_ornament_validated_preserves_other_setters_on_some() {
        // Layered on top of a fully-populated bundle (including speed_ramp,
        // which exercises the bundle's largest field), every prior field
        // survives unchanged on accept. Locks the additive contract across
        // every knob on the bundle.
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::End)
            .with_bracket_direction(HookDirection::Down)
            .with_bracket_length_ss(0.75)
            .with_speed(TrillWiggleSpeed::Fast)
            .with_length_ss(3.0)
            .with_speed_ramp_ramp_count(
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                4,
            )
            .with_ornament_validated(Ornament::TrillWithMordent)
            .expect("supported ornament must accept");
        assert_eq!(opts.bracket, Some(TrillBracketSide::End));
        assert_eq!(opts.bracket_direction, Some(HookDirection::Down));
        assert_eq!(opts.bracket_length_ss, Some(0.75));
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Fast));
        assert_eq!(opts.length_ss, Some(3.0));
        assert_eq!(
            opts.speed_ramp,
            Some(TrillSpeedRampSpec::new(
                TrillSpeedRamp::Linear {
                    start: TrillWiggleSpeed::Slow,
                    end: TrillWiggleSpeed::Fast,
                },
                4,
            ))
        );
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
    }

    #[test]
    fn with_ornament_validated_is_const_callable() {
        // `const` items hold one Some and two Nones across distinct
        // rejected ornaments. Drops `const fn` would break this.
        const ACCEPTED: Option<TrillExtensionFullOptions> =
            TrillExtensionFullOptions::new().with_ornament_validated(Ornament::TrillWithMordent);
        const REJECTED_SHORT_TRILL: Option<TrillExtensionFullOptions> =
            TrillExtensionFullOptions::new().with_ornament_validated(Ornament::ShortTrill);
        const REJECTED_MORDENT: Option<TrillExtensionFullOptions> =
            TrillExtensionFullOptions::new().with_ornament_validated(Ornament::Mordent);
        assert!(ACCEPTED.is_some());
        assert!(REJECTED_SHORT_TRILL.is_none());
        assert!(REJECTED_MORDENT.is_none());
    }

    #[test]
    fn with_ornament_validated_overwrites_prior_value_on_some() {
        // Two consecutive validated calls on the accept band: last write
        // wins (matches the permissive setter's last-write-wins).
        let opts = TrillExtensionFullOptions::new()
            .with_ornament_validated(Ornament::Trill)
            .expect("Trill accepts")
            .with_ornament_validated(Ornament::TrillWithMordent)
            .expect("TrillWithMordent accepts");
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
    }

    #[test]
    fn with_ornament_validated_none_branch_does_not_partially_populate() {
        // On a rejected ornament, the entire bundle is dropped via the
        // `Option<Self>` shape — there is no fallback that silently leaves
        // `ornament` unset and returns Some(self). Pins down the
        // documented "rejection breaks the chain" contract.
        //
        // We verify by composition: rejecting an ornament must not produce
        // a `Some` bundle whose ornament is None on the bracket-populated
        // input. The shape `Option<Self>` makes the contract structural,
        // but this test also rules out a hypothetical future refactor that
        // tried to return `Some(self)` with `ornament` unset on rejection.
        let result = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Start)
            .with_ornament_validated(Ornament::Turn);
        assert!(
            result.is_none(),
            "Turn must be rejected; bundle must not be partially populated"
        );
    }

    #[test]
    fn with_ornament_validated_some_branch_isolation() {
        // From a fresh `new()`, only `ornament` is populated on accept.
        // Locks the validator's "single-field write" contract.
        let opts = TrillExtensionFullOptions::new()
            .with_ornament_validated(Ornament::Trill)
            .expect("Trill accepts");
        assert_eq!(opts.ornament, Some(Ornament::Trill));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.length_ss, None);
        assert_eq!(opts.speed_ramp, None);
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
        let bracket = TrillBracketOptions::new(TrillBracketSide::End).with_extension_length_ss(4.0);
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
        const _SPEC: TrillSpeedRampSpec =
            TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 3);
        const _SPEC_LINEAR: TrillSpeedRampSpec = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            7,
        );
    }

    #[test]
    fn ramp_spec_partial_eq_sensitive_to_ramp() {
        // Same region_count, different ramp → distinct specs.
        let a = TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Fast), 4);
        let b = TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Slow), 4);
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
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp_ramp_count(TrillSpeedRamp::constant(TrillWiggleSpeed::Faster), 4);
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
                let sugar_form =
                    TrillExtensionFullOptions::new().with_speed_ramp_ramp_count(ramp, region_count);
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
        let first = TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Fast), 3);
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
        let spec_form =
            TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Fastest), 2);
        let opts_a = TrillExtensionFullOptions::new()
            .with_speed_ramp_ramp_count(
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
                5,
            )
            .with_speed_ramp(spec_form);
        assert_eq!(opts_a.speed_ramp, Some(spec_form));

        let opts_b = TrillExtensionFullOptions::new()
            .with_speed_ramp(spec_form)
            .with_speed_ramp_ramp_count(TrillSpeedRamp::constant(TrillWiggleSpeed::Slowest), 4);
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
        let spec = TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Slow), 4);
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
        const _OPTS: TrillExtensionFullOptions =
            TrillExtensionFullOptions::new().with_speed_ramp(TrillSpeedRampSpec::new(
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slowest, TrillWiggleSpeed::Fastest),
                4,
            ));
        const _OPTS_SUGAR: TrillExtensionFullOptions = TrillExtensionFullOptions::new()
            .with_speed_ramp_ramp_count(TrillSpeedRamp::constant(TrillWiggleSpeed::Faster), 3);
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
        let spec = TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Fastest), 2);
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
        let opts_dir = TrillExtensionFullOptions::new().with_bracket_direction(HookDirection::Up);
        let opts_blen = TrillExtensionFullOptions::new().with_bracket_length_ss(0.9);
        let opts_speed = TrillExtensionFullOptions::new().with_speed(TrillWiggleSpeed::Fast);
        let opts_orn = TrillExtensionFullOptions::new().with_ornament(Ornament::Trill);
        let opts_len = TrillExtensionFullOptions::new().with_length_ss(2.0);
        let opts_ext_len = TrillExtensionFullOptions::new().with_extension_length_ss(2.0);
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
        let degenerate_zero =
            TrillSpeedRampSpec::new(TrillSpeedRamp::constant(TrillWiggleSpeed::Standard), 0);
        let opts_zero = TrillExtensionFullOptions::new().with_speed_ramp(degenerate_zero);
        assert_eq!(opts_zero.speed_ramp, Some(degenerate_zero));
        assert_eq!(opts_zero.speed_ramp.unwrap().region_count, 0);

        let degenerate_one_linear = TrillSpeedRampSpec::new(
            TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
            1,
        );
        let opts_one = TrillExtensionFullOptions::new().with_speed_ramp(degenerate_one_linear);
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

    // --- with_speed_ramp_validated_ramp_count — strict counterpart to
    // with_speed_ramp_ramp_count. Pins down the rejection rules and the
    // Some-branch byte-equivalence with the permissive setter. The
    // rejection rules are inherited verbatim from
    // `TrillSpeedRampSpec::new_validated`, so these tests also serve as
    // cross-validation that the delegation hasn't drifted.

    #[test]
    fn with_speed_ramp_validated_ramp_count_rejects_zero_region_count_for_constant() {
        // Per the underlying spec validator: zero region count is a
        // rejection for any ramp variant, including `Constant`.
        let ramp = TrillSpeedRamp::constant(TrillWiggleSpeed::Standard);
        let result = TrillExtensionFullOptions::new().with_speed_ramp_validated_ramp_count(ramp, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_rejects_zero_region_count_for_linear() {
        // Symmetric to the constant case — zero is rejected for `Linear`
        // as well. Walks the boundary where the two rejection rules
        // overlap.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let result = TrillExtensionFullOptions::new().with_speed_ramp_validated_ramp_count(ramp, 0);
        assert_eq!(result, None);
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_rejects_one_region_for_linear() {
        // The Linear-specific rejection: a single-region linear progression
        // is ill-defined (only one endpoint can land on the region's
        // speed). The permissive setter would have accepted this and
        // produced a spec that fails at `synthesize_regions` time; the
        // validated setter rejects it up front.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let result = TrillExtensionFullOptions::new().with_speed_ramp_validated_ramp_count(ramp, 1);
        assert_eq!(result, None);
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_accepts_one_region_for_constant() {
        // The Constant carve-out: `region_count == 1` is the documented
        // minimum for a constant ramp (a single region trivially renders
        // the chosen speed across the entire span). Pin down that the
        // validator does *not* accidentally reject this — that would
        // narrow the accept band relative to `new_validated`.
        let ramp = TrillSpeedRamp::constant(TrillWiggleSpeed::Faster);
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(ramp, 1)
            .expect("Constant + 1 region is valid");
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(ramp, 1)));
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_accepts_two_regions_for_linear() {
        // The minimum-valid `Linear` case: `region_count == 2` lets both
        // endpoints land on their respective region's speed. The first
        // `Some` branch into the Linear arm of the validator.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slowest, TrillWiggleSpeed::Fastest);
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(ramp, 2)
            .expect("Linear + 2 regions is valid");
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(ramp, 2)));
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_some_branch_byte_equals_unvalidated() {
        // On any accepted `(ramp, region_count)` pair, the validated
        // setter must produce a bundle field-by-field equal to the
        // permissive setter. The PartialEq derive covers every field,
        // including the `speed_ramp` we just populated. Catches a
        // hypothetical drift where the validator started normalizing
        // accepted inputs (e.g. collapsing `Linear { s, s }` into
        // `Constant(s)`), which would silently change the stored spec.
        //
        // Walks accepted pairs only — the rejected pairs are covered by
        // the per-rule tests above and by
        // `with_speed_ramp_validated_ramp_count_rejection_matches_spec_new_validated`.
        for region_count in [1usize, 2, 3, 7, 15] {
            for ramp in [
                TrillSpeedRamp::constant(TrillWiggleSpeed::Slowest),
                TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
                TrillSpeedRamp::constant(TrillWiggleSpeed::Fastest),
            ] {
                let permissive =
                    TrillExtensionFullOptions::new().with_speed_ramp_ramp_count(ramp, region_count);
                let validated = TrillExtensionFullOptions::new()
                    .with_speed_ramp_validated_ramp_count(ramp, region_count)
                    .unwrap_or_else(|| {
                        panic!(
                            "validator rejected Constant ramp + region_count={region_count} \
                             that permissive setter accepted"
                        )
                    });
                assert_eq!(
                    permissive, validated,
                    "ramp={ramp:?} region_count={region_count}"
                );
            }
            // Linear ramps need region_count >= 2 to land on the accept
            // band; skip the (Linear, 1) case here — it's the dedicated
            // rejection test above.
            if region_count < 2 {
                continue;
            }
            for ramp in [
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slowest, TrillWiggleSpeed::Fastest),
                TrillSpeedRamp::linear(TrillWiggleSpeed::Fast, TrillWiggleSpeed::Slow),
                TrillSpeedRamp::linear(TrillWiggleSpeed::Standard, TrillWiggleSpeed::Standard),
            ] {
                let permissive =
                    TrillExtensionFullOptions::new().with_speed_ramp_ramp_count(ramp, region_count);
                let validated = TrillExtensionFullOptions::new()
                    .with_speed_ramp_validated_ramp_count(ramp, region_count)
                    .unwrap_or_else(|| {
                        panic!(
                            "validator rejected Linear ramp + region_count={region_count} \
                             that permissive setter accepted (ramp={ramp:?})"
                        )
                    });
                assert_eq!(
                    permissive, validated,
                    "ramp={ramp:?} region_count={region_count}"
                );
            }
        }
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_some_branch_isolation() {
        // On the `Some` branch from a fresh `new()`, only `speed_ramp` is
        // populated — every other field remains `None`. Mirrors
        // `with_speed_ramp_ramp_count_sets_only_speed_ramp` for the
        // validated counterpart.
        let ramp = TrillSpeedRamp::constant(TrillWiggleSpeed::Fast);
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(ramp, 3)
            .expect("Constant + 3 regions is valid");
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(ramp, 3)));
        assert_eq!(opts.bracket, None);
        assert_eq!(opts.bracket_direction, None);
        assert_eq!(opts.bracket_length_ss, None);
        assert_eq!(opts.speed, None);
        assert_eq!(opts.ornament, None);
        assert_eq!(opts.length_ss, None);
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_preserves_other_setters_on_some() {
        // The validator must be additive: chaining it on top of a bundle
        // with other fields already set leaves those fields intact on the
        // `Some` branch. Catches a regression where the validator
        // accidentally cleared a sibling field (e.g. zeroed `speed` on
        // accept, breaking the documented "speed + speed_ramp may
        // coexist" contract).
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_bracket_direction(HookDirection::Down)
            .with_bracket_length_ss(0.85)
            .with_speed(TrillWiggleSpeed::Standard)
            .with_ornament(Ornament::TrillWithMordent)
            .with_length_ss(3.25)
            .with_speed_ramp_validated_ramp_count(ramp, 4)
            .expect("Linear + 4 regions is valid");
        assert_eq!(opts.bracket, Some(TrillBracketSide::Both));
        assert_eq!(opts.bracket_direction, Some(HookDirection::Down));
        assert_eq!(opts.bracket_length_ss, Some(0.85));
        // The `speed` field must survive — the dispatch contract permits
        // speed + speed_ramp to coexist (speed_ramp.is_some() wins).
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Standard));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.length_ss, Some(3.25));
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(ramp, 4)));
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_is_const_callable() {
        // `const fn` symmetry: the validator must be callable in a `const`
        // context, matching every other setter on this bundle and
        // `TrillSpeedRampSpec::new_validated` itself. The compile-time
        // assertion form mirrors `linear_validated_is_const_callable` and
        // `spec_new_validated_is_const_callable` in `trill_extension.rs`.
        //
        // Hold one `Some` and two distinct `None` cases so a future
        // change that removes `const` from any branch trips this canary.
        const SOME_OPTS: Option<TrillExtensionFullOptions> = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(
                TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
                2,
            );
        const NONE_ZERO: Option<TrillExtensionFullOptions> = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(
                TrillSpeedRamp::constant(TrillWiggleSpeed::Standard),
                0,
            );
        const NONE_LINEAR_ONE: Option<TrillExtensionFullOptions> = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
                1,
            );
        assert!(SOME_OPTS.is_some());
        assert!(NONE_ZERO.is_none());
        assert!(NONE_LINEAR_ONE.is_none());
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_rejection_matches_spec_new_validated() {
        // Cross-validation: for every `(ramp, region_count)` pair in the
        // walk, the setter rejects iff `TrillSpeedRampSpec::new_validated`
        // rejects. Locks the delegation: any future divergence (e.g. the
        // setter starts validating an extra rule the spec doesn't) trips
        // this canary.
        //
        // Walks the boundary band — region_counts 0..=3 against Constant
        // and Linear variants. That covers all four documented rejection
        // / accept-edge cases.
        let constant = TrillSpeedRamp::constant(TrillWiggleSpeed::Standard);
        let linear = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        for region_count in 0..=3 {
            for ramp in [constant, linear] {
                let setter_result = TrillExtensionFullOptions::new()
                    .with_speed_ramp_validated_ramp_count(ramp, region_count);
                let spec_result = TrillSpeedRampSpec::new_validated(ramp, region_count);
                // Same rejection iff: setter is None iff spec is None.
                assert_eq!(
                    setter_result.is_none(),
                    spec_result.is_none(),
                    "rejection drift at ramp={ramp:?} region_count={region_count}: \
                     setter={setter_result:?} spec={spec_result:?}"
                );
                // And on the `Some` branch, the setter's stored
                // `speed_ramp` must equal the spec the validator
                // produced — no normalization on accept.
                if let (Some(opts), Some(spec)) = (setter_result, spec_result) {
                    assert_eq!(
                        opts.speed_ramp,
                        Some(spec),
                        "Some-branch spec mismatch at ramp={ramp:?} region_count={region_count}"
                    );
                }
            }
        }
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_none_branch_does_not_partially_populate() {
        // On `None`, the partially-built bundle is dropped — the caller
        // doesn't observe a bundle with `speed_ramp` mysteriously set to
        // some "fallback" value. The `Option<Self>` shape guarantees this
        // by construction (no `self` is returned on the `None` branch),
        // but this test pins the contract down at the call-site level so
        // a future refactor that swapped the return type for `Self` (with
        // silent fallback) would fail an existing test rather than
        // silently changing behavior. The follow-on observation is that
        // there's no `is_none()`-with-mutation escape hatch: the only way
        // to populate `speed_ramp` via this setter is to pass an accepted
        // pair, period.
        let result = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Start)
            .with_speed_ramp_validated_ramp_count(
                TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast),
                1,
            );
        assert_eq!(result, None);
        // A fresh bundle with only the prior setters applied should
        // *not* be smuggled out — confirming that, on the `None` path,
        // the only observable outcome at the call site is `None`. (No
        // direct field assertion is possible here because the bundle is
        // dropped; this test documents the contract via its mere
        // existence and the `None` assertion above.)
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_accepts_linear_equal_endpoint_speeds() {
        // Mirrors `spec_new_validated_accepts_linear_with_equal_endpoint_speeds`
        // up one layer: the validator does *not* additionally reject a
        // `Linear { s, s }` (rejecting equal endpoints is
        // `linear_validated`'s job). Locks in the orthogonal-layering
        // carve-out at the options-bundle layer.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Standard, TrillWiggleSpeed::Standard);
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(ramp, 3)
            .expect(
                "Linear { s, s } + region_count >= 2 is accepted (rejection of \
                     equal endpoints lives on `TrillSpeedRamp::linear_validated`)",
            );
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(ramp, 3)));
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_overwrites_prior_value_on_some() {
        // Same last-write-wins semantic as the permissive setter when both
        // pairs are on the accept band. Catches a regression where the
        // validator switched to "first write wins" or accumulated into a
        // Vec.
        let first = TrillSpeedRamp::constant(TrillWiggleSpeed::Slow);
        let second = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(first, 3)
            .expect("first pair valid")
            .with_speed_ramp_validated_ramp_count(second, 4)
            .expect("second pair valid");
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(second, 4)));
        // Sanity: confirm the two pairs would have produced distinct
        // specs — otherwise the overwrite assertion above is vacuous.
        assert_ne!(
            TrillSpeedRampSpec::new(first, 3),
            TrillSpeedRampSpec::new(second, 4)
        );
    }

    #[test]
    fn with_speed_ramp_validated_ramp_count_some_branch_feeds_synthesize_regions() {
        // End-to-end smoke: an accepted bundle's `speed_ramp` must feed
        // cleanly into `synthesize_regions` and produce `Some(regions)`
        // with `region_count` entries at the spec's region count.
        // Mirrors `spec_new_validated_some_branch_feeds_synthesize_regions`
        // one layer up. If the bundle layer ever introduced its own
        // post-construction massaging of the stored spec (e.g. clamping
        // region_count downward), this assertion would catch it.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let opts = TrillExtensionFullOptions::new()
            .with_speed_ramp_validated_ramp_count(ramp, 4)
            .expect("Linear + 4 regions is valid");
        let unpacked = opts.speed_ramp.expect("speed_ramp populated on accept");
        assert_eq!(unpacked.region_count, 4);
        let regions = unpacked
            .ramp
            .synthesize_regions(0.0, 400.0, unpacked.region_count, |_speed| 50.0)
            .expect("validated spec must feed synthesize_regions cleanly");
        assert_eq!(regions.len(), 4);
        // Region starts at evenly-spaced 0, 100, 200, 300 across [0, 400].
        assert_eq!(regions[0].start_x, 0.0);
        assert_eq!(regions[1].start_x, 100.0);
        assert_eq!(regions[2].start_x, 200.0);
        assert_eq!(regions[3].start_x, 300.0);
    }

    // --- TrillExtensionFullOptions: with_extension_length_ss_validated ---
    //
    // The validator's accept band mirrors the renderer's "would draw a
    // wiggle" check (`length_ss > 0.0`) in
    // `system_renderer::draw_trill_extensions_for_system`. The reject band
    // is therefore precisely the `Some(_)` non-positive arm that collapses
    // `end_x` to `start_x` (no wiggle). On accept the validator writes to
    // the underlying `length_ss` field (same field as the permissive setter)
    // — *not* to `bracket_length_ss`. These tests pin the rejection rules,
    // the field-targeting invariant, and the Some-branch byte-equivalence
    // with the permissive setter.

    #[test]
    fn with_extension_length_ss_validated_rejects_zero() {
        let result = TrillExtensionFullOptions::new().with_extension_length_ss_validated(0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn with_extension_length_ss_validated_rejects_negative_zero() {
        // `-0.0 > 0.0` is false. Catches a refactor that used
        // `length_ss.is_sign_negative()` or `length_ss != 0.0` instead.
        let result = TrillExtensionFullOptions::new().with_extension_length_ss_validated(-0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn with_extension_length_ss_validated_rejects_negative_finite() {
        for len in [-0.001, -0.5, -1.0, -10.0, -1.0e6] {
            let result = TrillExtensionFullOptions::new().with_extension_length_ss_validated(len);
            assert_eq!(result, None, "expected rejection for len={len}");
        }
    }

    #[test]
    fn with_extension_length_ss_validated_rejects_negative_infinity() {
        let result =
            TrillExtensionFullOptions::new().with_extension_length_ss_validated(f64::NEG_INFINITY);
        assert_eq!(result, None);
    }

    #[test]
    fn with_extension_length_ss_validated_rejects_nan() {
        let result = TrillExtensionFullOptions::new().with_extension_length_ss_validated(f64::NAN);
        assert_eq!(result, None);
    }

    #[test]
    fn with_extension_length_ss_validated_accepts_positive_finite() {
        for len in [0.001, 0.5, 1.0, 2.75, 1.0e6] {
            let opts = TrillExtensionFullOptions::new()
                .with_extension_length_ss_validated(len)
                .unwrap_or_else(|| panic!("expected accept for len={len}"));
            assert_eq!(opts.length_ss, Some(len), "len={len}");
            // The accessor mirrors the field — also pin that down.
            assert_eq!(
                opts.extension_length_ss(),
                Some(len),
                "accessor mismatch len={len}"
            );
        }
    }

    #[test]
    fn with_extension_length_ss_validated_accepts_positive_infinity() {
        // `+∞ > 0.0` is true — mirrors the renderer's accept band exactly.
        let opts = TrillExtensionFullOptions::new()
            .with_extension_length_ss_validated(f64::INFINITY)
            .expect("+infinity must be accepted to mirror the renderer");
        assert_eq!(opts.length_ss, Some(f64::INFINITY));
    }

    #[test]
    fn with_extension_length_ss_validated_some_byte_equals_permissive() {
        // On any accepted length, the validated and permissive setters
        // produce structurally identical options. PartialEq covers every
        // field.
        for len in [0.001, 0.5, 1.0, 2.75, 1.0e6, f64::INFINITY] {
            let permissive = TrillExtensionFullOptions::new().with_extension_length_ss(len);
            let validated = TrillExtensionFullOptions::new()
                .with_extension_length_ss_validated(len)
                .unwrap_or_else(|| panic!("expected accept for len={len}"));
            assert_eq!(permissive, validated, "len={len}");
        }
    }

    #[test]
    fn with_extension_length_ss_validated_writes_length_ss_not_bracket_length_ss() {
        // The validator must target the wiggle's length field, not the
        // bracket hook length. This is the same naming-disambiguation
        // invariant as the permissive setter — locks the field-targeting
        // contract down at the validated layer.
        let opts = TrillExtensionFullOptions::new()
            .with_extension_length_ss_validated(2.5)
            .expect("2.5 is valid");
        assert_eq!(opts.length_ss, Some(2.5));
        assert_eq!(opts.bracket_length_ss, None);
        // And the accessor agrees with the field.
        assert_eq!(opts.extension_length_ss(), Some(2.5));
    }

    #[test]
    fn with_extension_length_ss_validated_preserves_other_setters_on_some() {
        // The validator must be additive: chaining it on top of a bundle
        // with other fields already set leaves those fields intact on the
        // `Some` branch. Walks the largest combinable surface (bracket +
        // direction + bracket length + speed + ornament + speed ramp) to
        // catch a regression that cleared any sibling field.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::End)
            .with_bracket_direction(HookDirection::Up)
            .with_bracket_length_ss(0.7)
            .with_speed(TrillWiggleSpeed::Slow)
            .with_ornament(Ornament::TrillWithMordent)
            .with_speed_ramp(TrillSpeedRampSpec::new(ramp, 3))
            .with_extension_length_ss_validated(3.25)
            .expect("3.25 is a valid extension length");
        assert_eq!(opts.bracket, Some(TrillBracketSide::End));
        assert_eq!(opts.bracket_direction, Some(HookDirection::Up));
        assert_eq!(opts.bracket_length_ss, Some(0.7));
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Slow));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(ramp, 3)));
        assert_eq!(opts.length_ss, Some(3.25));
    }

    #[test]
    fn with_extension_length_ss_validated_overwrites_prior_value_on_some() {
        // Same last-write-wins semantic as the permissive setter, including
        // when chained against the legacy `with_length_ss` permissive name.
        let opts = TrillExtensionFullOptions::new()
            .with_extension_length_ss_validated(1.0)
            .expect("1.0 is valid")
            .with_extension_length_ss_validated(2.5)
            .expect("2.5 is valid");
        assert_eq!(opts.length_ss, Some(2.5));

        // Cross-name last-write-wins: validated wins over a prior
        // `with_length_ss(0.5)`.
        let opts2 = TrillExtensionFullOptions::new()
            .with_length_ss(0.5)
            .with_extension_length_ss_validated(2.5)
            .expect("2.5 is valid");
        assert_eq!(opts2.length_ss, Some(2.5));
    }

    #[test]
    fn with_extension_length_ss_validated_is_const_callable() {
        // `const fn` symmetry: matches every other setter on this bundle
        // and the existing `with_speed_ramp_validated_ramp_count` validator.
        const SOME_OPTS: Option<TrillExtensionFullOptions> =
            TrillExtensionFullOptions::new().with_extension_length_ss_validated(1.5);
        const NONE_OPTS: Option<TrillExtensionFullOptions> =
            TrillExtensionFullOptions::new().with_extension_length_ss_validated(0.0);
        assert!(SOME_OPTS.is_some());
        assert!(NONE_OPTS.is_none());
    }

    #[test]
    fn with_extension_length_ss_validated_none_branch_does_not_partially_populate() {
        // On `None`, the partially-built bundle is dropped — the caller
        // doesn't observe a bundle with `length_ss` mysteriously set to a
        // sentinel. The `Option<Self>` shape guarantees this by
        // construction. Pin it down by combining a non-trivial chain with
        // a rejecting validator call.
        let result = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Start)
            .with_extension_length_ss_validated(0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn with_extension_length_ss_validated_rejection_matches_renderer_accept_band() {
        // Cross-validation: for every probe value, the validator's
        // `is_none()` must equal the renderer's "would suppress" predicate
        // (`!(length_ss > 0.0)`). Locks the validator and renderer
        // together at this third layer too — any future drift trips this.
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
            let validator_is_none = TrillExtensionFullOptions::new()
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

    #[test]
    fn with_extension_length_ss_validated_three_bundles_agree_on_accept_band() {
        // Triangulate across all three options bundles — for every probe
        // value, all three validators must agree on accept/reject. The
        // permissive/validated split applied independently to three
        // bundles is the riskiest part of this pattern; this test catches
        // any drift between the three implementations.
        use crate::layout::trill_bracket::TrillBracketOptions;
        use crate::layout::trill_extension::TrillExtensionSpeedOptions;
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
            let bracket_is_none = TrillBracketOptions::new(TrillBracketSide::Both)
                .with_extension_length_ss_validated(len)
                .is_none();
            let speed_is_none = TrillExtensionSpeedOptions::new(TrillWiggleSpeed::Standard)
                .with_extension_length_ss_validated(len)
                .is_none();
            let full_is_none = TrillExtensionFullOptions::new()
                .with_extension_length_ss_validated(len)
                .is_none();
            assert_eq!(
                bracket_is_none, speed_is_none,
                "bracket/speed disagree at len={len}: bracket={bracket_is_none} speed={speed_is_none}"
            );
            assert_eq!(
                speed_is_none, full_is_none,
                "speed/full disagree at len={len}: speed={speed_is_none} full={full_is_none}"
            );
        }
    }

    // --- with_bracket_length_ss_validated — strict counterpart to
    // with_bracket_length_ss. Like the parallel
    // `with_hook_length_ss_validated` on `TrillBracketOptions`, this accepts
    // only `length_ss > 0.0`, deliberately tightening past the renderer's
    // `.abs()`-tolerant accept band. The rationale is identical: surface
    // explicit-flip misuse (callers should set `HookDirection::Up` via
    // `with_bracket_direction`, not negate the length) and reject the
    // degenerate-hook zero case. These tests pin the rejection rules, lock
    // down field-targeting (writes `bracket_length_ss`, NOT `length_ss`),
    // and verify byte-equivalence with the permissive setter on every
    // accepted value.

    #[test]
    fn with_bracket_length_ss_validated_rejects_zero() {
        // 0.0 → degenerate hook (y_top == y_bottom). Construction-time
        // rejection surfaces the misuse before draw time.
        let result = TrillExtensionFullOptions::new().with_bracket_length_ss_validated(0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn with_bracket_length_ss_validated_rejects_negative_zero() {
        // -0.0 fails `> 0.0` just like 0.0 — catches a refactor that
        // switched to `is_sign_negative()` or `!= 0.0`.
        let result = TrillExtensionFullOptions::new().with_bracket_length_ss_validated(-0.0);
        assert_eq!(result, None);
    }

    #[test]
    fn with_bracket_length_ss_validated_rejects_negative_finite() {
        // The headline rejection — the renderer's `.abs()` fold would
        // silently swallow these, overriding explicit-flip intent. Walks
        // representative magnitudes to pin down `> 0.0` rather than a
        // per-magnitude band.
        for len in [-0.001, -0.5, -1.0, -10.0, -1.0e6] {
            let result = TrillExtensionFullOptions::new().with_bracket_length_ss_validated(len);
            assert_eq!(result, None, "expected rejection for len={len}");
        }
    }

    #[test]
    fn with_bracket_length_ss_validated_rejects_negative_infinity() {
        // -∞ > 0.0 is false, so the validator rejects. The renderer would
        // otherwise apply `.abs()` and silently emit +∞.
        let result =
            TrillExtensionFullOptions::new().with_bracket_length_ss_validated(f64::NEG_INFINITY);
        assert_eq!(result, None);
    }

    #[test]
    fn with_bracket_length_ss_validated_rejects_nan() {
        // NaN > 0.0 is false. The renderer would otherwise emit NaN hook
        // coordinates.
        let result = TrillExtensionFullOptions::new().with_bracket_length_ss_validated(f64::NAN);
        assert_eq!(result, None);
    }

    #[test]
    fn with_bracket_length_ss_validated_accepts_positive_finite() {
        // Headline accept band: typical hook lengths. Walks small (< 1),
        // unit, and large magnitudes to pin down that the predicate is
        // `> 0.0` and not band-restricted.
        for len in [0.001, 0.5, 0.75, 1.0, 2.75, 1.0e6] {
            let opts = TrillExtensionFullOptions::new()
                .with_bracket_length_ss_validated(len)
                .unwrap_or_else(|| panic!("expected accept for len={len}"));
            assert_eq!(opts.bracket_length_ss, Some(len), "len={len}");
        }
    }

    #[test]
    fn with_bracket_length_ss_validated_accepts_positive_infinity() {
        // +∞ > 0.0 is true — accepted for byte-equivalence with the
        // permissive setter.
        let opts = TrillExtensionFullOptions::new()
            .with_bracket_length_ss_validated(f64::INFINITY)
            .expect("+infinity must be accepted to match the permissive setter");
        assert_eq!(opts.bracket_length_ss, Some(f64::INFINITY));
    }

    #[test]
    fn with_bracket_length_ss_validated_writes_bracket_length_ss_not_length_ss() {
        // The validator must target the bracket hook length field, NOT the
        // wiggle's extension termination length. Mirror of
        // `with_extension_length_ss_validated_writes_length_ss_not_bracket_length_ss`
        // — both validators on this bundle pin down the
        // naming-disambiguation invariant from their permissive setters.
        let opts = TrillExtensionFullOptions::new()
            .with_bracket_length_ss_validated(0.85)
            .expect("0.85 is valid");
        assert_eq!(opts.bracket_length_ss, Some(0.85));
        assert_eq!(opts.length_ss, None);
        // And the extension_length_ss accessor agrees with the field.
        assert_eq!(opts.extension_length_ss(), None);
    }

    #[test]
    fn with_bracket_length_ss_validated_some_byte_equals_permissive() {
        // On any accepted length the validator must produce a bundle
        // field-by-field equal to the permissive setter. PartialEq covers
        // every field. Catches a hypothetical normalization on accept.
        for len in [0.001, 0.5, 0.85, 1.0, 2.75, 1.0e6, f64::INFINITY] {
            let permissive = TrillExtensionFullOptions::new().with_bracket_length_ss(len);
            let validated = TrillExtensionFullOptions::new()
                .with_bracket_length_ss_validated(len)
                .unwrap_or_else(|| panic!("expected accept for len={len}"));
            assert_eq!(permissive, validated, "len={len}");
        }
    }

    #[test]
    fn with_bracket_length_ss_validated_preserves_other_setters_on_some() {
        // Additive contract over the largest combinable surface — bracket,
        // direction, speed, speed-ramp, ornament, extension length all
        // survive byte-for-byte on the `Some` branch.
        let ramp = TrillSpeedRamp::linear(TrillWiggleSpeed::Slow, TrillWiggleSpeed::Fast);
        let opts = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::End)
            .with_bracket_direction(HookDirection::Up)
            .with_speed(TrillWiggleSpeed::Slow)
            .with_ornament(Ornament::TrillWithMordent)
            .with_extension_length_ss(4.0)
            .with_speed_ramp(TrillSpeedRampSpec::new(ramp, 3))
            .with_bracket_length_ss_validated(0.85)
            .expect("0.85 is a valid bracket hook length");
        assert_eq!(opts.bracket, Some(TrillBracketSide::End));
        assert_eq!(opts.bracket_direction, Some(HookDirection::Up));
        assert_eq!(opts.speed, Some(TrillWiggleSpeed::Slow));
        assert_eq!(opts.ornament, Some(Ornament::TrillWithMordent));
        assert_eq!(opts.length_ss, Some(4.0));
        assert_eq!(opts.speed_ramp, Some(TrillSpeedRampSpec::new(ramp, 3)));
        assert_eq!(opts.bracket_length_ss, Some(0.85));
    }

    #[test]
    fn with_bracket_length_ss_validated_overwrites_prior_value_on_some() {
        // Same last-write-wins semantic as the permissive setter when both
        // values are on the accept band.
        let opts = TrillExtensionFullOptions::new()
            .with_bracket_length_ss_validated(0.5)
            .expect("0.5 is valid")
            .with_bracket_length_ss_validated(1.25)
            .expect("1.25 is valid");
        assert_eq!(opts.bracket_length_ss, Some(1.25));
    }

    #[test]
    fn with_bracket_length_ss_validated_is_const_callable() {
        // `const fn` symmetry: matches every other setter on this bundle.
        const SOME_OPTS: Option<TrillExtensionFullOptions> =
            TrillExtensionFullOptions::new().with_bracket_length_ss_validated(0.85);
        const NONE_OPTS: Option<TrillExtensionFullOptions> =
            TrillExtensionFullOptions::new().with_bracket_length_ss_validated(-0.5);
        assert!(SOME_OPTS.is_some());
        assert!(NONE_OPTS.is_none());
    }

    #[test]
    fn with_bracket_length_ss_validated_none_branch_does_not_partially_populate() {
        // On `None` the partially-built bundle is dropped via the
        // `Option<Self>` shape — there's no fallback that demotes a
        // rejection into a no-op. Combine with a non-trivial chain to
        // assert the contract holds even when prior fields were set.
        let result = TrillExtensionFullOptions::new()
            .with_bracket(TrillBracketSide::Both)
            .with_speed(TrillWiggleSpeed::Slow)
            .with_bracket_length_ss_validated(-1.0);
        assert!(result.is_none());
    }

    #[test]
    fn with_bracket_length_ss_validated_rejection_matches_visible_unfolded_hook_predicate() {
        // Cross-validation: for every probe value, the validator's
        // `is_none()` must equal the complement of the "would draw a
        // visible non-folded hook" predicate (`!(length_ss > 0.0)`).
        // Like the parallel test on `TrillBracketOptions`, this is NOT a
        // literal mirror of the renderer's accept band — the renderer
        // tolerates negatives via `.abs()` and emits a degenerate line at
        // zero. The validator deliberately tightens past those cases to
        // surface explicit-flip misuse and reject degenerate hooks.
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
            0.85,
            1.0,
            1.0e6,
            f64::INFINITY,
        ] {
            let validator_is_none = TrillExtensionFullOptions::new()
                .with_bracket_length_ss_validated(len)
                .is_none();
            let visible_unfolded_hook = len > 0.0;
            assert_eq!(
                validator_is_none, !visible_unfolded_hook,
                "drift at len={len}: validator_is_none={validator_is_none} \
                 visible_unfolded_hook={visible_unfolded_hook}"
            );
        }
    }

    #[test]
    fn with_bracket_length_ss_validated_two_bundles_agree_on_accept_band() {
        // Fourth-layer cross-validation: walks the probe set across
        // `TrillBracketOptions::with_hook_length_ss_validated` (which
        // targets `length_ss`, the hook length on that bundle) and this
        // bundle's `with_bracket_length_ss_validated` (which targets
        // `bracket_length_ss`). Both validators apply the same accept-band
        // predicate (`> 0.0`) — any drift between them would surface here.
        // This is the bracket-hook analogue of the
        // `three_bundles_agree_on_accept_band` test for the extension
        // length surface; only two bundles have hook-length fields, hence
        // the narrower name.
        use crate::layout::trill_bracket::TrillBracketOptions;
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
            0.85,
            1.0,
            1.0e6,
            f64::INFINITY,
        ] {
            let bracket_is_none = TrillBracketOptions::new(TrillBracketSide::Both)
                .with_hook_length_ss_validated(len)
                .is_none();
            let full_is_none = TrillExtensionFullOptions::new()
                .with_bracket_length_ss_validated(len)
                .is_none();
            assert_eq!(
                bracket_is_none, full_is_none,
                "two bundles disagree at len={len}: bracket={bracket_is_none} full={full_is_none}"
            );
        }
    }
}
