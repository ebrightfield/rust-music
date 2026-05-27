/// Hairpin type: crescendo (opening wedge) or decrescendo (closing wedge).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HairpinType {
    /// Opening wedge: starts narrow/closed, ends wide/open. Indicates increasing volume.
    Crescendo,
    /// Closing wedge: starts wide/open, ends narrow/closed. Indicates decreasing volume.
    Decrescendo,
}

/// Which tip of the wedge carries the niente "o" circle.
///
/// Drives the choice between [`layout_hairpin_with_niente`] (closed-end —
/// the common "al niente" / "dal niente" convention) and
/// [`layout_hairpin_with_niente_at_open_end`] (rare modern-notation variant
/// used by Lachenmann, Sciarrino, etc).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NientePlacement {
    /// Niente "o" sits at the closed (pointy) tip of the wedge. Standard
    /// engraving convention for "from silence" (crescendo) or "to silence"
    /// (decrescendo).
    ClosedEnd,
    /// Niente "o" sits at the open (wide) tip of the wedge. Used in
    /// contemporary scores where the silence is at the wide side — e.g. a
    /// crescendo opening out into silence.
    OpenEnd,
}

/// Open circle drawn at the closed (pointy) end of a hairpin, indicating
/// "to/from silence" (niente / al niente / dal niente).
///
/// Coordinates in font design units. The renderer draws this as a stroked
/// circle with no fill (open "o" convention). For [`HairpinType::Crescendo`]
/// the niente sits at the left tip (`HairpinLayout::x_start`); for
/// [`HairpinType::Decrescendo`] it sits at the right tip (`HairpinLayout::x_end`).
#[derive(Clone, Copy, Debug)]
pub struct NienteCircleLayout {
    /// X-coordinate of the circle center (matches the hairpin tip x).
    pub cx: f64,
    /// Y-coordinate of the circle center (matches `HairpinLayout::y_center`).
    pub cy: f64,
    /// Circle radius in font design units.
    pub radius: f64,
    /// Stroke width — matches the parent hairpin's `stroke_width` so the
    /// circle reads as the same line weight as the wedge.
    pub stroke_width: f64,
}

/// Dash/gap pattern for a dashed hairpin wedge.
///
/// Both lengths are in font design units. The renderer emits these as an
/// SVG `stroke-dasharray` attribute (`"<dash_length>,<gap_length>"`) on
/// both wedge lines. A niente "o" circle, when present, remains solid —
/// engraved convention treats the niente as a definite symbol independent
/// of the wedge's dashed/solid styling.
#[derive(Clone, Copy, Debug)]
pub struct HairpinDashStyle {
    /// Length of each dash, in font design units.
    pub dash_length: f64,
    /// Length of each gap between dashes, in font design units.
    pub gap_length: f64,
}

/// Layout result for a hairpin (crescendo/decrescendo wedge).
///
/// All coordinates in font design units. The hairpin is drawn as two
/// converging or diverging lines from (x_start, y) to (x_end, y).
#[derive(Clone, Debug)]
pub struct HairpinLayout {
    /// Hairpin type.
    pub kind: HairpinType,
    /// X-coordinate of the left end.
    pub x_start: f64,
    /// X-coordinate of the right end.
    pub x_end: f64,
    /// Y-coordinate of the center line (midpoint between top and bottom lines).
    pub y_center: f64,
    /// Half the opening width at the wide end, in font design units.
    /// The wedge spans from (y_center - half_opening) to (y_center + half_opening).
    pub half_opening: f64,
    /// Stroke width for the hairpin lines.
    pub stroke_width: f64,
    /// Optional niente (open-circle) marker at the closed end. `None` for a
    /// plain hairpin; `Some` only when constructed via
    /// [`layout_hairpin_with_niente`].
    pub niente: Option<NienteCircleLayout>,
    /// Optional dashed-wedge style. `None` for a plain solid hairpin;
    /// `Some` only when constructed via [`layout_hairpin_dashed`]. The
    /// renderer applies the dash pattern to both wedge lines.
    pub dashed: Option<HairpinDashStyle>,
}

/// Default vertical distance from bottom staff line to hairpin center, in staff spaces.
/// Placed slightly below dynamics text (which sits at 2.5ss below staff).
pub const HAIRPIN_BELOW_STAFF_SS: f64 = 3.5;

/// Default half-opening of the hairpin at its widest, in staff spaces.
/// A full opening of ~1 staff space is standard for engraved hairpins.
pub const HAIRPIN_HALF_OPENING_SS: f64 = 0.5;

/// Default radius of the niente "o" circle, in staff spaces.
///
/// Standard engraved practice puts a small open circle ~0.4 staff spaces in
/// diameter at the closed end of a hairpin to indicate to/from silence.
/// Radius 0.2ss → diameter 0.4ss.
pub const HAIRPIN_NIENTE_RADIUS_SS: f64 = 0.2;

/// Default dash length for a dashed hairpin, in staff spaces.
///
/// 0.4ss matches typical engraved practice for hairpin continuation marks
/// and modern dashed-wedge notation. Combined with [`HAIRPIN_GAP_LENGTH_SS`]
/// this gives a roughly 2:1 dash-to-gap ratio.
pub const HAIRPIN_DASH_LENGTH_SS: f64 = 0.4;

/// Default gap length between dashes for a dashed hairpin, in staff spaces.
///
/// 0.2ss matches typical engraved practice. The 2:1 dash:gap ratio reads
/// clearly without being so fine that downstream rasterizers fuse the
/// dashes into a solid line at small zoom levels.
pub const HAIRPIN_GAP_LENGTH_SS: f64 = 0.2;

/// Compute the layout for a hairpin (crescendo/decrescendo wedge).
///
/// `x_start` and `x_end` are the horizontal extents of the hairpin.
/// `staff_bottom_y` is the y-coordinate of the bottom staff line.
/// `staff_space` is one staff space in font design units.
/// `stroke_width` is the line thickness (typically from `EngravingConfig::hairpin_thickness`
/// or a fallback like `staff_line_thickness`).
pub fn layout_hairpin(
    kind: HairpinType,
    x_start: f64,
    x_end: f64,
    staff_bottom_y: f64,
    staff_space: f64,
    stroke_width: f64,
) -> HairpinLayout {
    let y_center = staff_bottom_y + HAIRPIN_BELOW_STAFF_SS * staff_space;
    let half_opening = HAIRPIN_HALF_OPENING_SS * staff_space;

    HairpinLayout {
        kind,
        x_start,
        x_end,
        y_center,
        half_opening,
        stroke_width,
        niente: None,
        dashed: None,
    }
}

/// Compute the layout for a hairpin with a niente "o" circle at its closed
/// (pointy) end.
///
/// Identical to [`layout_hairpin`] except that the returned layout carries a
/// [`NienteCircleLayout`] in its [`HairpinLayout::niente`] field. The circle
/// sits at:
/// - `(x_start, y_center)` for [`HairpinType::Crescendo`] — "from silence"
/// - `(x_end,   y_center)` for [`HairpinType::Decrescendo`] — "to silence"
///
/// The circle's radius is [`HAIRPIN_NIENTE_RADIUS_SS`] times the staff space
/// and its `stroke_width` matches the parent wedge so the open "o" reads as
/// the same line weight as the hairpin lines. Wedge geometry (x_start,
/// x_end, y_center, half_opening, stroke_width) is byte-identical to the
/// plain [`layout_hairpin`] for the same arguments — niente is purely
/// additive.
pub fn layout_hairpin_with_niente(
    kind: HairpinType,
    x_start: f64,
    x_end: f64,
    staff_bottom_y: f64,
    staff_space: f64,
    stroke_width: f64,
) -> HairpinLayout {
    let mut layout = layout_hairpin(kind, x_start, x_end, staff_bottom_y, staff_space, stroke_width);
    let cx = match kind {
        HairpinType::Crescendo => x_start,
        HairpinType::Decrescendo => x_end,
    };
    layout.niente = Some(NienteCircleLayout {
        cx,
        cy: layout.y_center,
        radius: HAIRPIN_NIENTE_RADIUS_SS * staff_space,
        stroke_width,
    });
    layout
}

/// Compute the layout for a hairpin with a niente "o" circle at its open
/// (wide) end — the rarer mirror of [`layout_hairpin_with_niente`].
///
/// Most niente hairpins place the circle at the closed (pointy) end. The
/// open-end variant is used for special-effect markings where the silence
/// is at the wide side of the wedge — e.g. a crescendo that starts at full
/// dynamic and *opens out* into silence (an open-end "to silence" reading),
/// or a decrescendo that begins from silence at its widest. Standard in
/// contemporary scores by composers like Lachenmann and Sciarrino.
///
/// Anchor placement is the mirror of [`layout_hairpin_with_niente`]:
/// - `(x_end,   y_center)` for [`HairpinType::Crescendo`] — circle at the
///   wide right end (open end of a crescendo).
/// - `(x_start, y_center)` for [`HairpinType::Decrescendo`] — circle at the
///   wide left end (open end of a decrescendo).
///
/// Radius, `cy`, and `stroke_width` are computed identically to the
/// closed-end constructor; only `cx` differs. Wedge geometry is
/// byte-identical to [`layout_hairpin`] for the same arguments — the
/// niente is purely additive.
///
/// The circle remains a small open "o" at the wedge midline. A reader sees
/// it at the wedge's mouth rather than at its tip; the visual conflict
/// with the diverging wedge lines at small radii is the same trade-off
/// engravers accept in the closed-end case at the tip.
pub fn layout_hairpin_with_niente_at_open_end(
    kind: HairpinType,
    x_start: f64,
    x_end: f64,
    staff_bottom_y: f64,
    staff_space: f64,
    stroke_width: f64,
) -> HairpinLayout {
    let mut layout = layout_hairpin(kind, x_start, x_end, staff_bottom_y, staff_space, stroke_width);
    let cx = match kind {
        HairpinType::Crescendo => x_end,
        HairpinType::Decrescendo => x_start,
    };
    layout.niente = Some(NienteCircleLayout {
        cx,
        cy: layout.y_center,
        radius: HAIRPIN_NIENTE_RADIUS_SS * staff_space,
        stroke_width,
    });
    layout
}

/// Compute the layout for a dashed hairpin wedge.
///
/// Identical to [`layout_hairpin`] except that the returned layout carries
/// a [`HairpinDashStyle`] in its [`HairpinLayout::dashed`] field. Dash and
/// gap lengths are derived from [`HAIRPIN_DASH_LENGTH_SS`] and
/// [`HAIRPIN_GAP_LENGTH_SS`] scaled by `staff_space`, so the dash pattern
/// scales proportionally with staff size.
///
/// Dashed hairpins are used for:
/// - continuation of a hairpin across a system break (the second segment
///   is conventionally dashed),
/// - "soft" or implied crescendi in modern notation,
/// - text-equivalent dashed continuations (independent of the "cresc. - - -"
///   text variant, which uses dashed text rather than a wedge).
///
/// Wedge geometry (x_start, x_end, y_center, half_opening, stroke_width)
/// is byte-identical to the plain [`layout_hairpin`] for the same
/// arguments — dashed is purely additive. The [`HairpinLayout::niente`]
/// field remains `None`; callers wanting both a dashed wedge and a niente
/// circle can mutate the returned layout's `niente` field after the call.
pub fn layout_hairpin_dashed(
    kind: HairpinType,
    x_start: f64,
    x_end: f64,
    staff_bottom_y: f64,
    staff_space: f64,
    stroke_width: f64,
) -> HairpinLayout {
    let mut layout = layout_hairpin(kind, x_start, x_end, staff_bottom_y, staff_space, stroke_width);
    layout.dashed = Some(HairpinDashStyle {
        dash_length: HAIRPIN_DASH_LENGTH_SS * staff_space,
        gap_length: HAIRPIN_GAP_LENGTH_SS * staff_space,
    });
    layout
}

/// Compute the layout for a hairpin in any of the four supported styles.
///
/// Single entry point for the score-event chain: callers pass the optional
/// niente placement and the dashed flag and get back a fully-styled
/// [`HairpinLayout`]. Equivalent to choosing among
/// [`layout_hairpin`], [`layout_hairpin_dashed`],
/// [`layout_hairpin_with_niente`], and [`layout_hairpin_with_niente_at_open_end`]
/// based on the flags — but written once, so the system renderer and the
/// cross-system splitter on the page renderer don't have to duplicate the
/// combinatorics.
///
/// Wedge geometry (x_start, x_end, y_center, half_opening, stroke_width) is
/// byte-identical to [`layout_hairpin`] for the same arguments regardless of
/// the niente / dashed flags. The niente "o" circle, when present, sits at
/// the tip selected by [`NientePlacement`] and remains solid even when
/// `dashed = true` — engraved convention treats the niente as a definite
/// symbol independent of the wedge's dashed/solid style.
pub fn layout_hairpin_styled(
    kind: HairpinType,
    x_start: f64,
    x_end: f64,
    staff_bottom_y: f64,
    staff_space: f64,
    stroke_width: f64,
    niente: Option<NientePlacement>,
    dashed: bool,
) -> HairpinLayout {
    let mut layout = match niente {
        Some(NientePlacement::ClosedEnd) => layout_hairpin_with_niente(
            kind,
            x_start,
            x_end,
            staff_bottom_y,
            staff_space,
            stroke_width,
        ),
        Some(NientePlacement::OpenEnd) => layout_hairpin_with_niente_at_open_end(
            kind,
            x_start,
            x_end,
            staff_bottom_y,
            staff_space,
            stroke_width,
        ),
        None => layout_hairpin(kind, x_start, x_end, staff_bottom_y, staff_space, stroke_width),
    };
    if dashed {
        layout.dashed = Some(HairpinDashStyle {
            dash_length: HAIRPIN_DASH_LENGTH_SS * staff_space,
            gap_length: HAIRPIN_GAP_LENGTH_SS * staff_space,
        });
    }
    layout
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: f64 = 250.0;
    const BOTTOM_Y: f64 = 1000.0;
    const SW: f64 = 10.0;

    fn cresc() -> HairpinLayout {
        layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    fn decresc() -> HairpinLayout {
        layout_hairpin(HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    #[test]
    fn crescendo_kind() {
        assert_eq!(cresc().kind, HairpinType::Crescendo);
    }

    #[test]
    fn decrescendo_kind() {
        assert_eq!(decresc().kind, HairpinType::Decrescendo);
    }

    #[test]
    fn x_positions_preserved() {
        let h = cresc();
        assert!((h.x_start - 100.0).abs() < 1e-6);
        assert!((h.x_end - 600.0).abs() < 1e-6);
    }

    #[test]
    fn y_center_below_staff() {
        let h = cresc();
        let expected = BOTTOM_Y + HAIRPIN_BELOW_STAFF_SS * SS;
        assert!((h.y_center - expected).abs() < 1e-6);
        assert!(h.y_center > BOTTOM_Y);
    }

    #[test]
    fn half_opening_scales_with_staff_space() {
        let small = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 0.0, 200.0, SW);
        let large = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 0.0, 400.0, SW);
        assert!(large.half_opening > small.half_opening);
        assert!((small.half_opening - HAIRPIN_HALF_OPENING_SS * 200.0).abs() < 1e-6);
        assert!((large.half_opening - HAIRPIN_HALF_OPENING_SS * 400.0).abs() < 1e-6);
    }

    #[test]
    fn stroke_width_preserved() {
        let h = cresc();
        assert!((h.stroke_width - SW).abs() < 1e-6);
    }

    #[test]
    fn different_staff_bottom_shifts_y() {
        let h1 = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 500.0, SS, SW);
        let h2 = layout_hairpin(HairpinType::Crescendo, 0.0, 100.0, 1500.0, SS, SW);
        assert!(h2.y_center > h1.y_center);
        let diff = h2.y_center - h1.y_center;
        assert!((diff - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn zero_width_hairpin() {
        let h = layout_hairpin(HairpinType::Crescendo, 300.0, 300.0, BOTTOM_Y, SS, SW);
        assert!((h.x_start - h.x_end).abs() < 1e-6);
    }

    #[test]
    fn crescendo_and_decrescendo_same_geometry() {
        let c = cresc();
        let d = decresc();
        // Same positions, only kind differs
        assert!((c.y_center - d.y_center).abs() < 1e-6);
        assert!((c.half_opening - d.half_opening).abs() < 1e-6);
        assert!((c.x_start - d.x_start).abs() < 1e-6);
        assert!((c.x_end - d.x_end).abs() < 1e-6);
    }

    #[test]
    fn opening_is_positive() {
        let h = cresc();
        assert!(h.half_opening > 0.0);
    }

    // ---- niente circle ----

    #[test]
    fn plain_hairpin_has_no_niente() {
        assert!(cresc().niente.is_none());
        assert!(decresc().niente.is_none());
    }

    fn cresc_n() -> HairpinLayout {
        layout_hairpin_with_niente(HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    fn decresc_n() -> HairpinLayout {
        layout_hairpin_with_niente(HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    #[test]
    fn with_niente_crescendo_places_circle_at_start() {
        let h = cresc_n();
        let n = h.niente.expect("crescendo with niente must carry circle");
        assert!((n.cx - 100.0).abs() < 1e-9, "cx should equal x_start (100.0), got {}", n.cx);
        assert!((n.cy - h.y_center).abs() < 1e-9, "cy should equal y_center");
    }

    #[test]
    fn with_niente_decrescendo_places_circle_at_end() {
        let h = decresc_n();
        let n = h.niente.expect("decrescendo with niente must carry circle");
        assert!((n.cx - 600.0).abs() < 1e-9, "cx should equal x_end (600.0), got {}", n.cx);
        assert!((n.cy - h.y_center).abs() < 1e-9, "cy should equal y_center");
    }

    #[test]
    fn niente_radius_matches_const_times_staff_space() {
        let h = cresc_n();
        let n = h.niente.unwrap();
        let expected = HAIRPIN_NIENTE_RADIUS_SS * SS;
        assert!(
            (n.radius - expected).abs() < 1e-9,
            "radius should be HAIRPIN_NIENTE_RADIUS_SS * staff_space ({expected}), got {}",
            n.radius
        );
    }

    #[test]
    fn niente_radius_scales_with_staff_space() {
        let small = layout_hairpin_with_niente(HairpinType::Crescendo, 0.0, 100.0, 0.0, 200.0, SW);
        let large = layout_hairpin_with_niente(HairpinType::Crescendo, 0.0, 100.0, 0.0, 400.0, SW);
        let r_small = small.niente.unwrap().radius;
        let r_large = large.niente.unwrap().radius;
        assert!(r_large > r_small, "larger staff space → larger niente");
        assert!((r_small - HAIRPIN_NIENTE_RADIUS_SS * 200.0).abs() < 1e-9);
        assert!((r_large - HAIRPIN_NIENTE_RADIUS_SS * 400.0).abs() < 1e-9);
        // Ratio of radii equals ratio of staff spaces (linear scaling).
        assert!(((r_large / r_small) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn niente_stroke_width_matches_hairpin() {
        let custom_sw = 17.5;
        let h = layout_hairpin_with_niente(
            HairpinType::Decrescendo, 0.0, 500.0, BOTTOM_Y, SS, custom_sw,
        );
        let n = h.niente.unwrap();
        assert!((n.stroke_width - custom_sw).abs() < 1e-9);
        assert!((h.stroke_width - n.stroke_width).abs() < 1e-9);
    }

    #[test]
    fn niente_does_not_alter_wedge_geometry() {
        let plain = layout_hairpin(HairpinType::Crescendo, 123.0, 789.0, BOTTOM_Y, SS, SW);
        let with_n = layout_hairpin_with_niente(HairpinType::Crescendo, 123.0, 789.0, BOTTOM_Y, SS, SW);
        assert_eq!(plain.kind, with_n.kind);
        assert!((plain.x_start - with_n.x_start).abs() < 1e-12);
        assert!((plain.x_end - with_n.x_end).abs() < 1e-12);
        assert!((plain.y_center - with_n.y_center).abs() < 1e-12);
        assert!((plain.half_opening - with_n.half_opening).abs() < 1e-12);
        assert!((plain.stroke_width - with_n.stroke_width).abs() < 1e-12);
        assert!(plain.niente.is_none());
        assert!(with_n.niente.is_some());
    }

    #[test]
    fn niente_y_lives_on_hairpin_midline() {
        let h = cresc_n();
        let n = h.niente.unwrap();
        // The circle must straddle the central axis exactly — engraved
        // niente sits on the midline regardless of wedge aperture so the
        // reader sees the "o" merging into the wedge tip.
        assert!((n.cy - h.y_center).abs() < 1e-12);
    }

    #[test]
    fn crescendo_and_decrescendo_nientes_target_opposite_ends() {
        // Same x_start/x_end on both → crescendo niente.cx != decrescendo niente.cx
        let c = cresc_n();
        let d = decresc_n();
        let cn = c.niente.unwrap();
        let dn = d.niente.unwrap();
        assert!((cn.cx - 100.0).abs() < 1e-9);
        assert!((dn.cx - 600.0).abs() < 1e-9);
        assert!((cn.cx - dn.cx).abs() > 1.0, "endpoints must differ");
        // Heights match on the same staff_bottom_y / staff_space.
        assert!((cn.cy - dn.cy).abs() < 1e-12);
        assert!((cn.radius - dn.radius).abs() < 1e-12);
    }

    #[test]
    fn niente_const_layout_is_copy() {
        let h = cresc_n();
        let n = h.niente.unwrap();
        let copy = n; // Copy semantics — would not compile without Copy.
        assert!((n.cx - copy.cx).abs() < 1e-12);
        assert!((n.cy - copy.cy).abs() < 1e-12);
        assert!((n.radius - copy.radius).abs() < 1e-12);
    }

    // ---- open-end niente (rare mirror of closed-end niente) ----

    fn cresc_n_open() -> HairpinLayout {
        layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        )
    }

    fn decresc_n_open() -> HairpinLayout {
        layout_hairpin_with_niente_at_open_end(
            HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        )
    }

    #[test]
    fn open_end_niente_crescendo_places_circle_at_x_end() {
        // For a crescendo, the open end is the wide right side → x_end.
        let h = cresc_n_open();
        let n = h.niente.expect("open-end crescendo must carry niente");
        assert!(
            (n.cx - 600.0).abs() < 1e-9,
            "open-end crescendo niente.cx should equal x_end (600.0); got {}",
            n.cx
        );
        assert!((n.cy - h.y_center).abs() < 1e-9, "cy should equal y_center");
    }

    #[test]
    fn open_end_niente_decrescendo_places_circle_at_x_start() {
        // For a decrescendo, the open end is the wide left side → x_start.
        let h = decresc_n_open();
        let n = h.niente.expect("open-end decrescendo must carry niente");
        assert!(
            (n.cx - 100.0).abs() < 1e-9,
            "open-end decrescendo niente.cx should equal x_start (100.0); got {}",
            n.cx
        );
        assert!((n.cy - h.y_center).abs() < 1e-9, "cy should equal y_center");
    }

    #[test]
    fn open_end_niente_mirrors_closed_end_niente_anchor() {
        // Open-end and closed-end niente target opposite tips for the same kind.
        // For Crescendo: closed-end is x_start, open-end is x_end.
        let closed = cresc_n();
        let open = cresc_n_open();
        let cn = closed.niente.unwrap();
        let on = open.niente.unwrap();
        assert!(
            (cn.cx - 100.0).abs() < 1e-9,
            "closed-end crescendo anchored at x_start"
        );
        assert!(
            (on.cx - 600.0).abs() < 1e-9,
            "open-end crescendo anchored at x_end"
        );
        // Anchor diff equals the full wedge span (500.0).
        assert!(
            ((on.cx - cn.cx) - 500.0).abs() < 1e-9,
            "anchor diff should equal full x span; got {}",
            on.cx - cn.cx
        );
        // Geometry (cy, radius, stroke_width) is identical — only cx differs.
        assert!((cn.cy - on.cy).abs() < 1e-12);
        assert!((cn.radius - on.radius).abs() < 1e-12);
        assert!((cn.stroke_width - on.stroke_width).abs() < 1e-12);
    }

    #[test]
    fn open_end_niente_decrescendo_mirrors_crescendo_closed_anchor_choice() {
        // For Decrescendo: closed-end is x_end, open-end is x_start.
        // Cross-check the symmetric flip relative to crescendo.
        let closed = decresc_n();
        let open = decresc_n_open();
        let cn = closed.niente.unwrap();
        let on = open.niente.unwrap();
        assert!((cn.cx - 600.0).abs() < 1e-9, "closed-end decresc at x_end");
        assert!((on.cx - 100.0).abs() < 1e-9, "open-end decresc at x_start");
        // Anchor differs by the full span (in the opposite direction from cresc).
        assert!(
            ((cn.cx - on.cx) - 500.0).abs() < 1e-9,
            "closed - open = full span for decrescendo"
        );
    }

    #[test]
    fn open_end_niente_radius_matches_const_times_staff_space() {
        let h = cresc_n_open();
        let n = h.niente.unwrap();
        let expected = HAIRPIN_NIENTE_RADIUS_SS * SS;
        assert!(
            (n.radius - expected).abs() < 1e-9,
            "open-end niente radius should be HAIRPIN_NIENTE_RADIUS_SS * staff_space ({expected}), got {}",
            n.radius
        );
    }

    #[test]
    fn open_end_niente_radius_scales_with_staff_space() {
        let small = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo, 0.0, 100.0, 0.0, 200.0, SW,
        );
        let large = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo, 0.0, 100.0, 0.0, 400.0, SW,
        );
        let rs = small.niente.unwrap().radius;
        let rl = large.niente.unwrap().radius;
        assert!(rl > rs, "larger staff space → larger niente");
        assert!((rs - HAIRPIN_NIENTE_RADIUS_SS * 200.0).abs() < 1e-9);
        assert!((rl - HAIRPIN_NIENTE_RADIUS_SS * 400.0).abs() < 1e-9);
        assert!(((rl / rs) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn open_end_niente_stroke_width_matches_hairpin() {
        let custom_sw = 17.5;
        let h = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo, 0.0, 500.0, BOTTOM_Y, SS, custom_sw,
        );
        let n = h.niente.unwrap();
        assert!((n.stroke_width - custom_sw).abs() < 1e-9);
        assert!((h.stroke_width - n.stroke_width).abs() < 1e-9);
    }

    #[test]
    fn open_end_niente_does_not_alter_wedge_geometry() {
        // Open-end niente is purely additive — wedge geometry must be byte-identical
        // to the plain layout_hairpin output.
        let plain = layout_hairpin(HairpinType::Crescendo, 123.0, 789.0, BOTTOM_Y, SS, SW);
        let with_open = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo, 123.0, 789.0, BOTTOM_Y, SS, SW,
        );
        assert_eq!(plain.kind, with_open.kind);
        assert!((plain.x_start - with_open.x_start).abs() < 1e-12);
        assert!((plain.x_end - with_open.x_end).abs() < 1e-12);
        assert!((plain.y_center - with_open.y_center).abs() < 1e-12);
        assert!((plain.half_opening - with_open.half_opening).abs() < 1e-12);
        assert!((plain.stroke_width - with_open.stroke_width).abs() < 1e-12);
        assert!(plain.niente.is_none());
        assert!(with_open.niente.is_some());
    }

    #[test]
    fn open_end_niente_y_lives_on_hairpin_midline() {
        // Open-end niente sits at the wide end of the wedge but still on the
        // midline (not on the top/bottom wedge line) — same convention as
        // the closed-end variant.
        let h = cresc_n_open();
        let n = h.niente.unwrap();
        assert!((n.cy - h.y_center).abs() < 1e-12);
    }

    #[test]
    fn open_end_niente_does_not_set_dashed() {
        // Independence from the dashed flag — the open-end niente constructor
        // must NOT silently populate `dashed`.
        assert!(cresc_n_open().dashed.is_none());
        assert!(decresc_n_open().dashed.is_none());
    }

    #[test]
    fn open_end_niente_constructor_distinct_from_closed_end_constructor() {
        // Locks the API contract: the two constructors place the circle at
        // different anchors for the same arguments. A regression that aliased
        // one to the other would surface immediately.
        let closed = layout_hairpin_with_niente(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        );
        let open = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        );
        let cn = closed.niente.unwrap();
        let on = open.niente.unwrap();
        assert!(
            (cn.cx - on.cx).abs() > 1.0,
            "closed-end and open-end constructors must place the circle at different x; \
             got cn.cx={} on.cx={}",
            cn.cx, on.cx
        );
    }

    #[test]
    fn open_end_niente_combo_with_dashed_supported_via_field_mutation() {
        // The dashed+niente combinator path also works for the open-end
        // variant — the additive fields are independent. Pins the contract
        // so a future refactor doesn't silently break composition.
        let mut h = layout_hairpin_with_niente_at_open_end(
            HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        );
        assert!(h.niente.is_some());
        assert!(h.dashed.is_none());
        h.dashed = Some(HairpinDashStyle {
            dash_length: 80.0,
            gap_length: 40.0,
        });
        assert!(h.niente.is_some(), "field mutation must not clear niente");
        assert!(h.dashed.is_some(), "field mutation must set dashed");
        // Open-end decrescendo anchor still pinned at x_start after mutation.
        assert!((h.niente.unwrap().cx - 100.0).abs() < 1e-9);
    }

    // ---- dashed hairpin ----

    fn cresc_d() -> HairpinLayout {
        layout_hairpin_dashed(HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    fn decresc_d() -> HairpinLayout {
        layout_hairpin_dashed(HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW)
    }

    #[test]
    fn plain_hairpin_has_no_dashed() {
        // Both constructors that lack a "_dashed" suffix must yield a solid wedge.
        assert!(cresc().dashed.is_none());
        assert!(decresc().dashed.is_none());
        assert!(cresc_n().dashed.is_none());
        assert!(decresc_n().dashed.is_none());
    }

    #[test]
    fn dashed_hairpin_crescendo_carries_style() {
        let h = cresc_d();
        let d = h.dashed.expect("dashed crescendo must carry a dash style");
        assert!(d.dash_length > 0.0, "dash_length must be strictly positive");
        assert!(d.gap_length > 0.0, "gap_length must be strictly positive");
    }

    #[test]
    fn dashed_hairpin_decrescendo_carries_style() {
        let h = decresc_d();
        let d = h.dashed.expect("dashed decrescendo must carry a dash style");
        assert!(d.dash_length > 0.0);
        assert!(d.gap_length > 0.0);
    }

    #[test]
    fn dashed_lengths_match_const_times_staff_space() {
        let h = cresc_d();
        let d = h.dashed.unwrap();
        let expected_dash = HAIRPIN_DASH_LENGTH_SS * SS;
        let expected_gap = HAIRPIN_GAP_LENGTH_SS * SS;
        assert!(
            (d.dash_length - expected_dash).abs() < 1e-9,
            "dash_length should be HAIRPIN_DASH_LENGTH_SS * staff_space ({expected_dash}), got {}",
            d.dash_length
        );
        assert!(
            (d.gap_length - expected_gap).abs() < 1e-9,
            "gap_length should be HAIRPIN_GAP_LENGTH_SS * staff_space ({expected_gap}), got {}",
            d.gap_length
        );
    }

    #[test]
    fn dashed_lengths_scale_linearly_with_staff_space() {
        let small = layout_hairpin_dashed(HairpinType::Crescendo, 0.0, 100.0, 0.0, 200.0, SW);
        let large = layout_hairpin_dashed(HairpinType::Crescendo, 0.0, 100.0, 0.0, 400.0, SW);
        let ds = small.dashed.unwrap();
        let dl = large.dashed.unwrap();
        assert!(dl.dash_length > ds.dash_length, "larger staff space → longer dash");
        assert!(dl.gap_length > ds.gap_length, "larger staff space → longer gap");
        // Linear scaling: 200 → 400 doubles both.
        assert!(((dl.dash_length / ds.dash_length) - 2.0).abs() < 1e-9);
        assert!(((dl.gap_length / ds.gap_length) - 2.0).abs() < 1e-9);
        // Absolute values lock to the constants.
        assert!((ds.dash_length - HAIRPIN_DASH_LENGTH_SS * 200.0).abs() < 1e-9);
        assert!((ds.gap_length - HAIRPIN_GAP_LENGTH_SS * 200.0).abs() < 1e-9);
        assert!((dl.dash_length - HAIRPIN_DASH_LENGTH_SS * 400.0).abs() < 1e-9);
        assert!((dl.gap_length - HAIRPIN_GAP_LENGTH_SS * 400.0).abs() < 1e-9);
    }

    #[test]
    fn dashed_default_dash_exceeds_gap() {
        // The default 2:1 dash:gap ratio reads clearly without fusing at small zoom.
        // Lock the relationship so a future const tweak that flips it surfaces here.
        let h = cresc_d();
        let d = h.dashed.unwrap();
        assert!(
            d.dash_length > d.gap_length,
            "default dash ({}) should exceed default gap ({}) for a 2:1 read",
            d.dash_length, d.gap_length
        );
        // Ratio is roughly 2:1 by const choice.
        let ratio = d.dash_length / d.gap_length;
        assert!((ratio - 2.0).abs() < 1e-9, "default ratio should be 2:1, got {ratio}");
    }

    #[test]
    fn dashed_does_not_alter_wedge_geometry() {
        let plain = layout_hairpin(HairpinType::Crescendo, 123.0, 789.0, BOTTOM_Y, SS, SW);
        let dashed = layout_hairpin_dashed(HairpinType::Crescendo, 123.0, 789.0, BOTTOM_Y, SS, SW);
        assert_eq!(plain.kind, dashed.kind);
        assert!((plain.x_start - dashed.x_start).abs() < 1e-12);
        assert!((plain.x_end - dashed.x_end).abs() < 1e-12);
        assert!((plain.y_center - dashed.y_center).abs() < 1e-12);
        assert!((plain.half_opening - dashed.half_opening).abs() < 1e-12);
        assert!((plain.stroke_width - dashed.stroke_width).abs() < 1e-12);
        assert!(plain.dashed.is_none());
        assert!(dashed.dashed.is_some());
    }

    #[test]
    fn dashed_does_not_set_niente() {
        // dashed and niente are independent additive fields. Constructing
        // a dashed hairpin must NOT silently populate niente.
        assert!(cresc_d().niente.is_none());
        assert!(decresc_d().niente.is_none());
    }

    #[test]
    fn dashed_style_is_copy() {
        let h = cresc_d();
        let d = h.dashed.unwrap();
        let copy = d; // Copy semantics — would not compile if Copy were removed.
        assert!((d.dash_length - copy.dash_length).abs() < 1e-12);
        assert!((d.gap_length - copy.gap_length).abs() < 1e-12);
    }

    #[test]
    fn dashed_zero_width_hairpin_still_carries_style() {
        // Edge case: collapsed wedge (x_start == x_end). The dashed flag
        // should still be set — the renderer's degenerate-line behavior
        // (zero-length dashes) is its own concern, not the layout's.
        let h = layout_hairpin_dashed(HairpinType::Crescendo, 300.0, 300.0, BOTTOM_Y, SS, SW);
        assert!((h.x_start - h.x_end).abs() < 1e-12);
        assert!(h.dashed.is_some(), "zero-width hairpin should still carry dashed style");
    }

    #[test]
    fn dashed_crescendo_and_decrescendo_same_geometry() {
        // Same arguments → same dashed wedge geometry. Only `kind` differs.
        let c = cresc_d();
        let d = decresc_d();
        assert!((c.y_center - d.y_center).abs() < 1e-12);
        assert!((c.half_opening - d.half_opening).abs() < 1e-12);
        assert!((c.x_start - d.x_start).abs() < 1e-12);
        assert!((c.x_end - d.x_end).abs() < 1e-12);
        // Dashed style is also identical between cresc and decresc on the same args.
        let cd = c.dashed.unwrap();
        let dd = d.dashed.unwrap();
        assert!((cd.dash_length - dd.dash_length).abs() < 1e-12);
        assert!((cd.gap_length - dd.gap_length).abs() < 1e-12);
    }

    #[test]
    fn dashed_combo_with_niente_supported_via_field_mutation() {
        // Although there is no single constructor for dashed+niente, the
        // additive fields are public — callers can compose them. This test
        // pins that contract so a future move of the fields behind getters
        // doesn't silently break the combinator path.
        let mut h = layout_hairpin_with_niente(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        );
        assert!(h.niente.is_some());
        assert!(h.dashed.is_none());
        h.dashed = Some(HairpinDashStyle {
            dash_length: 80.0,
            gap_length: 40.0,
        });
        assert!(h.niente.is_some(), "field mutation must not clear niente");
        assert!(h.dashed.is_some(), "field mutation must set dashed");
        let d = h.dashed.unwrap();
        assert!((d.dash_length - 80.0).abs() < 1e-12);
        assert!((d.gap_length - 40.0).abs() < 1e-12);
    }

    // ---- layout_hairpin_styled (unified entry point) ----
    //
    // These tests pin the contract that the unified helper is exactly
    // equivalent to the individual constructors for every (niente × dashed)
    // combination — same geometry, same niente anchor, same dash pattern.
    // The score-event chain (system_renderer + page_renderer cross-system
    // splitter) routes through this single helper; if the equivalence
    // breaks, a downstream golden delta will catch the visual change but
    // these unit tests pin the layer at its source.

    #[test]
    fn styled_no_niente_no_dash_equals_plain_hairpin() {
        let s = layout_hairpin_styled(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW, None, false,
        );
        let p = layout_hairpin(HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW);
        assert!(s.niente.is_none(), "no niente requested → none in layout");
        assert!(s.dashed.is_none(), "no dashed requested → none in layout");
        assert!((s.x_start - p.x_start).abs() < 1e-12);
        assert!((s.x_end - p.x_end).abs() < 1e-12);
        assert!((s.y_center - p.y_center).abs() < 1e-12);
        assert!((s.half_opening - p.half_opening).abs() < 1e-12);
        assert_eq!(s.kind, p.kind);
    }

    #[test]
    fn styled_dashed_no_niente_equals_layout_hairpin_dashed() {
        let s = layout_hairpin_styled(
            HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW, None, true,
        );
        let p = layout_hairpin_dashed(HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW);
        let ds = s.dashed.expect("styled dashed=true → Some");
        let dp = p.dashed.expect("layout_hairpin_dashed → Some");
        assert!((ds.dash_length - dp.dash_length).abs() < 1e-12);
        assert!((ds.gap_length - dp.gap_length).abs() < 1e-12);
        assert!((ds.dash_length - HAIRPIN_DASH_LENGTH_SS * SS).abs() < 1e-12);
        assert!((ds.gap_length - HAIRPIN_GAP_LENGTH_SS * SS).abs() < 1e-12);
        assert!(s.niente.is_none());
    }

    #[test]
    fn styled_closed_niente_no_dash_anchors_at_closed_tip() {
        let s = layout_hairpin_styled(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
            Some(NientePlacement::ClosedEnd), false,
        );
        let p = layout_hairpin_with_niente(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        );
        assert!(s.dashed.is_none());
        let ns = s.niente.expect("closed niente requested → Some");
        let np = p.niente.expect("layout_hairpin_with_niente → Some");
        // Crescendo closed tip is at x_start = 100.
        assert!((ns.cx - 100.0).abs() < 1e-12);
        assert!((ns.cx - np.cx).abs() < 1e-12);
        assert!((ns.cy - np.cy).abs() < 1e-12);
        assert!((ns.radius - np.radius).abs() < 1e-12);
        assert!((ns.stroke_width - np.stroke_width).abs() < 1e-12);
    }

    #[test]
    fn styled_open_niente_no_dash_anchors_at_open_tip() {
        let s = layout_hairpin_styled(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
            Some(NientePlacement::OpenEnd), false,
        );
        let p = layout_hairpin_with_niente_at_open_end(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
        );
        assert!(s.dashed.is_none());
        let ns = s.niente.expect("open niente requested → Some");
        let np = p.niente.expect("layout_hairpin_with_niente_at_open_end → Some");
        // Crescendo open tip is at x_end = 600.
        assert!((ns.cx - 600.0).abs() < 1e-12);
        assert!((ns.cx - np.cx).abs() < 1e-12);
    }

    #[test]
    fn styled_decrescendo_closed_niente_anchors_at_x_end() {
        // For a decrescendo, the closed (pointy) end is the right tip.
        let s = layout_hairpin_styled(
            HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
            Some(NientePlacement::ClosedEnd), false,
        );
        let n = s.niente.unwrap();
        assert!(
            (n.cx - 600.0).abs() < 1e-12,
            "decrescendo closed-end niente should anchor at x_end (600.0); got {}",
            n.cx
        );
    }

    #[test]
    fn styled_decrescendo_open_niente_anchors_at_x_start() {
        // Mirror of the previous test: decrescendo open tip is the left tip.
        let s = layout_hairpin_styled(
            HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
            Some(NientePlacement::OpenEnd), false,
        );
        let n = s.niente.unwrap();
        assert!(
            (n.cx - 100.0).abs() < 1e-12,
            "decrescendo open-end niente should anchor at x_start (100.0); got {}",
            n.cx
        );
    }

    #[test]
    fn styled_dashed_with_closed_niente_carries_both() {
        // The combinator path: both flags set → layout has both `dashed`
        // and `niente` populated. This is the contract that lets the
        // ScoreBuilder request a dashed wedge with a niente "o" via a
        // single helper call.
        let s = layout_hairpin_styled(
            HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
            Some(NientePlacement::ClosedEnd), true,
        );
        let n = s.niente.expect("dashed + closed niente → niente populated");
        assert!((n.cx - 100.0).abs() < 1e-12);
        let d = s.dashed.expect("dashed + closed niente → dashed populated");
        assert!((d.dash_length - HAIRPIN_DASH_LENGTH_SS * SS).abs() < 1e-12);
        assert!((d.gap_length - HAIRPIN_GAP_LENGTH_SS * SS).abs() < 1e-12);
    }

    #[test]
    fn styled_dashed_with_open_niente_carries_both() {
        let s = layout_hairpin_styled(
            HairpinType::Decrescendo, 100.0, 600.0, BOTTOM_Y, SS, SW,
            Some(NientePlacement::OpenEnd), true,
        );
        let n = s.niente.expect("dashed + open niente → niente populated");
        // Decrescendo open tip is at x_start.
        assert!((n.cx - 100.0).abs() < 1e-12);
        assert!(s.dashed.is_some(), "dashed + open niente → dashed populated");
    }

    #[test]
    fn styled_geometry_independent_of_niente_and_dashed() {
        // All four combinations on the same arguments share identical
        // wedge geometry — niente and dashed are purely additive overlays.
        let args = (HairpinType::Crescendo, 100.0, 600.0, BOTTOM_Y, SS, SW);
        let cases = [
            layout_hairpin_styled(args.0, args.1, args.2, args.3, args.4, args.5, None, false),
            layout_hairpin_styled(args.0, args.1, args.2, args.3, args.4, args.5, None, true),
            layout_hairpin_styled(
                args.0, args.1, args.2, args.3, args.4, args.5,
                Some(NientePlacement::ClosedEnd), false,
            ),
            layout_hairpin_styled(
                args.0, args.1, args.2, args.3, args.4, args.5,
                Some(NientePlacement::OpenEnd), true,
            ),
        ];
        let r = &cases[0];
        for c in &cases[1..] {
            assert!((c.x_start - r.x_start).abs() < 1e-12);
            assert!((c.x_end - r.x_end).abs() < 1e-12);
            assert!((c.y_center - r.y_center).abs() < 1e-12);
            assert!((c.half_opening - r.half_opening).abs() < 1e-12);
            assert_eq!(c.kind, r.kind);
        }
    }
}
