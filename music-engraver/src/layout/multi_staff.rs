//! Multi-staff grouping layout: braces, brackets, and barline connectors.
//!
//! A [`StaffGroup`] defines how multiple staves are visually connected in a
//! system — piano grand staff (brace + joined barlines), orchestral section
//! (bracket), or independent staves (none). Layout functions compute the
//! geometry for connectors placed at the left edge of the system.
//!
//! The connector types follow standard engraving practice:
//! - **Brace**: curly brace joining 2 staves of one instrument (piano, harp, organ).
//!   Uses the SMuFL `Brace` glyph, scaled to span the inter-staff distance.
//! - **Bracket**: thick L-shaped bracket grouping a section (strings, winds).
//!   Drawn as a thick vertical line with short horizontal serifs at top and bottom.
//! - **None**: staves are visually independent (no connector).

use smufl::Glyph;

use crate::layout::staff::StaffLayout;

/// How staves in a group are visually connected at the left edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectorKind {
    /// Curly brace (piano, harp, organ).
    Brace,
    /// Thick bracket with serifs (orchestral section).
    Bracket,
    /// No connector; staves are visually independent.
    None,
}

/// A group of staves that share a left-edge connector and joined barlines.
#[derive(Clone, Debug)]
pub struct StaffGroup {
    /// Number of staves in this group (e.g. 2 for grand staff).
    pub staff_count: usize,
    /// Visual connector at the left edge.
    pub connector: ConnectorKind,
    /// Whether barlines are drawn continuously through all staves in the group.
    pub joined_barlines: bool,
    /// Inner sub-bracket groupings, each spanning a contiguous range of staves
    /// within this group. Drawn as a thinner inner bracket sitting just to the
    /// right of the main bracket — the standard published-engraving convention
    /// (Gould, Behind Bars; Lilypond's `StaffGroup` ↔ `GrandStaff` nesting)
    /// for two-deep section grouping (e.g. Violin I + Violin II share an
    /// inner bracket within the larger string-section bracket). Only honoured
    /// when `connector == ConnectorKind::Bracket`; ignored otherwise.
    pub sub_brackets: Vec<SubBracket>,
}

/// A nested sub-bracket grouping a contiguous range of staves within a parent
/// [`StaffGroup`]. Indexes are 0-based and refer to staves in the parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubBracket {
    /// Index of the first staff (0-based, within the parent group).
    pub start_index: usize,
    /// Number of staves spanned. Must be >= 2 to produce visible output;
    /// values of 0 or 1 are silently dropped by [`layout_multi_staff`].
    pub staff_count: usize,
}

impl StaffGroup {
    /// Grand staff for keyboard instruments: 2 staves, brace, joined barlines.
    pub fn grand_staff() -> Self {
        Self {
            staff_count: 2,
            connector: ConnectorKind::Brace,
            joined_barlines: true,
            sub_brackets: Vec::new(),
        }
    }

    /// Orchestral section bracket: N staves, bracket, joined barlines.
    pub fn section(staff_count: usize) -> Self {
        Self {
            staff_count,
            connector: ConnectorKind::Bracket,
            joined_barlines: true,
            sub_brackets: Vec::new(),
        }
    }

    /// Independent staves with no connector or joined barlines.
    pub fn independent(staff_count: usize) -> Self {
        Self {
            staff_count,
            connector: ConnectorKind::None,
            joined_barlines: false,
            sub_brackets: Vec::new(),
        }
    }

    /// Replace this group's sub-bracket list. Convenience for the builder
    /// style: `StaffGroup::section(5).with_sub_brackets(vec![...])`.
    pub fn with_sub_brackets(mut self, sub_brackets: Vec<SubBracket>) -> Self {
        self.sub_brackets = sub_brackets;
        self
    }
}

/// Spacing constants for multi-staff layout, in staff spaces.
pub const INTER_STAFF_GAP_SS: f64 = 6.0;
/// Brace glyph placement offset from the staff edge, in staff spaces.
pub const BRACE_LEFT_OFFSET_SS: f64 = 0.5;
/// Bracket line thickness, in staff spaces. Matches Bravura's
/// `bracketThickness` engraving default.
pub const BRACKET_THICKNESS_SS: f64 = 0.5;
/// Sub-bracket line thickness, in staff spaces. Matches Bravura's
/// `subBracketThickness` engraving default; thinner than the main bracket so
/// nested grouping reads visually subordinate.
pub const SUB_BRACKET_THICKNESS_SS: f64 = 0.16;
/// Horizontal gap between the right edge of the main bracket's thick line
/// and the left edge of the inner sub-bracket, in staff spaces. Empirical
/// engraving convention — large enough that the two brackets do not visually
/// merge, small enough that the nested pair reads as one structure.
pub const SUB_BRACKET_GAP_SS: f64 = 0.3;

/// Computed geometry for a brace connector.
///
/// `BraceLayout` describes only the **geometric intent** (where the brace must
/// span vertically); the renderer queries the actual brace glyph's bounding
/// box from the font and derives the scale factor at draw time. This keeps
/// the layout font-agnostic — fonts whose brace glyph has a different
/// design-height than Bravura's render correctly without any layout change.
#[derive(Clone, Debug)]
pub struct BraceLayout {
    /// SMuFL glyph for the brace.
    pub glyph: Glyph,
    /// X position of the brace glyph (left of the staff system).
    pub x: f64,
    /// Y position of the top of the staff system (top of the first staff).
    /// The brace's top edge will align with this y.
    pub y_top: f64,
    /// Y position of the bottom of the staff system (bottom of the last
    /// staff). The brace's bottom edge will align with this y.
    pub y_bottom: f64,
}

impl BraceLayout {
    /// Total vertical span the brace must cover (>= 0).
    pub fn span_height(&self) -> f64 {
        self.y_bottom - self.y_top
    }

    /// Vertical midpoint between `y_top` and `y_bottom` — convenient for
    /// callers that want to center other ornamentation on the brace.
    pub fn y_center(&self) -> f64 {
        (self.y_top + self.y_bottom) / 2.0
    }
}

/// Computed geometry for a bracket connector.
///
/// The bracket is rendered as a thick vertical line with decorative SMuFL
/// scroll glyphs (`bracketTop`, `bracketBottom`) at each end — the published
/// engraving convention for orchestral grouping brackets.
///
/// `BracketLayout` records only the **geometric intent**: where the thick
/// line spans (`y_top..y_bottom` along `x`) and which scroll glyphs to
/// attach. The renderer queries each scroll glyph's actual bbox from the
/// font's SMuFL metadata and computes a translate that aligns the glyph's
/// inner edge (the edge meeting the line) with the line's endpoint — so the
/// seam stays clean across SMuFL fonts whose scroll-glyph origin may
/// diverge from the typical Bravura convention (bBoxSW for `bracketTop`,
/// bBoxNW for `bracketBottom`).
#[derive(Clone, Debug)]
pub struct BracketLayout {
    /// X position of the bracket's thick vertical line (left edge). The
    /// scroll glyphs are aligned so their left edges sit at this x.
    pub x: f64,
    /// Y of the top of the thick line. The top scroll glyph's bottom edge
    /// is aligned to this y.
    pub y_top: f64,
    /// Y of the bottom of the thick line. The bottom scroll glyph's top
    /// edge is aligned to this y.
    pub y_bottom: f64,
    /// Thickness of the vertical line, in font design units.
    pub thickness: f64,
    /// SMuFL glyph for the decorative top scroll (curls upward/outward).
    pub top_glyph: Glyph,
    /// SMuFL glyph for the decorative bottom scroll (curls downward/outward).
    pub bottom_glyph: Glyph,
}

/// Computed geometry for a nested sub-bracket.
///
/// Rendered as a thin vertical line (no scroll decoration — Lilypond's
/// convention for two-deep section grouping) sitting just to the right of
/// the parent bracket's thick line. The renderer draws a single stroked
/// line; no SMuFL glyph is required at the sub-bracket size (Bravura
/// provides scroll glyphs for the main bracket only, and conventional
/// engraving omits scrolls on the inner bracket so the nesting reads
/// hierarchically).
#[derive(Clone, Debug)]
pub struct SubBracketLayout {
    /// X position of the sub-bracket's thin vertical line (left edge of the
    /// stroked region).
    pub x: f64,
    /// Y of the top of the line (top of the first staff in the sub-group).
    pub y_top: f64,
    /// Y of the bottom of the line (bottom of the last staff in the sub-group).
    pub y_bottom: f64,
    /// Thickness of the line, in font design units. From SMuFL's
    /// `subBracketThickness` engraving default.
    pub thickness: f64,
}

/// Computed vertical positions for staves in a multi-staff system.
#[derive(Clone, Debug)]
pub struct MultiStaffLayout {
    /// Y-origin for each staff (top line of each staff), from top to bottom.
    pub staff_y_origins: Vec<f64>,
    /// The inter-staff gap in font design units.
    pub inter_staff_gap: f64,
    /// Staff space in font design units (shared by all staves).
    pub staff_space: f64,
    /// Optional brace geometry (if connector is `Brace`).
    pub brace: Option<BraceLayout>,
    /// Optional bracket geometry (if connector is `Bracket`).
    pub bracket: Option<BracketLayout>,
    /// Inner sub-bracket geometries, one per valid `SubBracket` in the parent
    /// group. Empty unless the group is bracketed AND supplied sub-brackets.
    /// A `SubBracket` with `staff_count < 2` or that overshoots the parent's
    /// range is silently dropped here rather than turned into a layout entry.
    pub sub_brackets: Vec<SubBracketLayout>,
}

impl MultiStaffLayout {
    /// Total height from top of first staff to bottom of last staff.
    pub fn total_height(&self) -> f64 {
        if self.staff_y_origins.is_empty() {
            return 0.0;
        }
        let first_top = self.staff_y_origins[0];
        let last_top = self.staff_y_origins[self.staff_y_origins.len() - 1];
        let staff_height = self.staff_space * 4.0; // 5 lines = 4 spaces
        last_top - first_top + staff_height
    }
}

/// Compute vertical positions for staves in a group.
///
/// `y_start` is the y-coordinate for the top line of the first staff.
/// `staff_space` is the inter-line distance (font design units).
/// `staff_width` is the horizontal width of each staff.
///
/// Returns a `MultiStaffLayout` with y-origins for each staff and optional
/// connector geometry.
pub fn layout_multi_staff(
    group: &StaffGroup,
    y_start: f64,
    staff_space: f64,
    _staff_width: f64,
) -> MultiStaffLayout {
    let staff_height = staff_space * 4.0; // 5 lines → 4 inter-line gaps
    let gap = INTER_STAFF_GAP_SS * staff_space;

    let mut staff_y_origins = Vec::with_capacity(group.staff_count);
    for i in 0..group.staff_count {
        let y = y_start + (i as f64) * (staff_height + gap);
        staff_y_origins.push(y);
    }

    let brace = if group.connector == ConnectorKind::Brace && group.staff_count >= 2 {
        let top = staff_y_origins[0];
        let bottom_staff_top = staff_y_origins[group.staff_count - 1];
        let bottom = bottom_staff_top + staff_height;
        // The vertical scale is computed by the renderer using the font's
        // actual brace glyph height (read from SMuFL metadata) — see
        // `draw_brace`. The layout records only the geometric intent.
        Some(BraceLayout {
            glyph: Glyph::Brace,
            x: -BRACE_LEFT_OFFSET_SS * staff_space,
            y_top: top,
            y_bottom: bottom,
        })
    } else {
        None
    };

    // Sub-brackets are honoured only inside a Bracket-connected group.
    // Resolve valid entries first; the resolved list both shapes the main
    // bracket's x-offset (so it shifts left to make room) and populates the
    // returned layout.
    let resolved_sub_brackets: Vec<&SubBracket> = if group.connector == ConnectorKind::Bracket {
        group
            .sub_brackets
            .iter()
            .filter(|sb| {
                sb.staff_count >= 2
                    && sb.start_index < group.staff_count
                    && sb.start_index + sb.staff_count <= group.staff_count
            })
            .collect()
    } else {
        Vec::new()
    };
    let has_sub_brackets = !resolved_sub_brackets.is_empty();

    // The main bracket's left edge sits at -bracket_thickness when no
    // sub-brackets are present (its right edge meets the staff at x=0). With
    // sub-brackets, the main bracket shifts left by enough to accommodate
    // (gap + sub_bracket_thickness + gap) — leaving the sub-bracket nested
    // between the main bracket's right edge and the staff origin.
    let main_bracket_x = if has_sub_brackets {
        -(BRACKET_THICKNESS_SS + SUB_BRACKET_GAP_SS + SUB_BRACKET_THICKNESS_SS + SUB_BRACKET_GAP_SS)
            * staff_space
    } else {
        -BRACKET_THICKNESS_SS * staff_space
    };

    let bracket = if group.connector == ConnectorKind::Bracket && group.staff_count >= 2 {
        let y_top = staff_y_origins[0];
        let y_bottom = staff_y_origins[group.staff_count - 1] + staff_height;

        Some(BracketLayout {
            x: main_bracket_x,
            y_top,
            y_bottom,
            thickness: BRACKET_THICKNESS_SS * staff_space,
            top_glyph: Glyph::BracketTop,
            bottom_glyph: Glyph::BracketBottom,
        })
    } else {
        None
    };

    // Sub-bracket geometry: each sub-bracket spans `staff_count` staves
    // starting at `start_index`. Its line sits one gap to the right of the
    // main bracket's right edge — `main_bracket_x + bracket_thickness + gap`
    // (the SUB_BRACKET_GAP_SS just inside the sub-bracket; another gap exists
    // between sub-bracket and staff origin, baked into `main_bracket_x`).
    let sub_bracket_layouts: Vec<SubBracketLayout> = if has_sub_brackets {
        let main_right_edge = main_bracket_x + BRACKET_THICKNESS_SS * staff_space;
        let sub_x = main_right_edge + SUB_BRACKET_GAP_SS * staff_space;
        let sub_thickness = SUB_BRACKET_THICKNESS_SS * staff_space;
        resolved_sub_brackets
            .iter()
            .map(|sb| {
                let first = sb.start_index;
                let last = sb.start_index + sb.staff_count - 1;
                let y_top = staff_y_origins[first];
                let y_bottom = staff_y_origins[last] + staff_height;
                SubBracketLayout {
                    x: sub_x,
                    y_top,
                    y_bottom,
                    thickness: sub_thickness,
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    MultiStaffLayout {
        staff_y_origins,
        inter_staff_gap: gap,
        staff_space,
        brace,
        bracket,
        sub_brackets: sub_bracket_layouts,
    }
}

/// Build `StaffLayout` instances for each staff in a multi-staff layout.
pub fn staff_layouts_from_multi(
    multi: &MultiStaffLayout,
    x: f64,
    width: f64,
) -> Vec<StaffLayout> {
    multi
        .staff_y_origins
        .iter()
        .map(|&y| StaffLayout::new(x, y, width, multi.staff_space))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SS: f64 = 250.0; // typical staff space in font design units

    #[test]
    fn grand_staff_has_two_staves_and_brace() {
        let g = StaffGroup::grand_staff();
        assert_eq!(g.staff_count, 2);
        assert_eq!(g.connector, ConnectorKind::Brace);
        assert!(g.joined_barlines);
    }

    #[test]
    fn section_has_bracket() {
        let g = StaffGroup::section(4);
        assert_eq!(g.staff_count, 4);
        assert_eq!(g.connector, ConnectorKind::Bracket);
        assert!(g.joined_barlines);
    }

    #[test]
    fn independent_has_no_connector() {
        let g = StaffGroup::independent(3);
        assert_eq!(g.staff_count, 3);
        assert_eq!(g.connector, ConnectorKind::None);
        assert!(!g.joined_barlines);
    }

    #[test]
    fn layout_two_staves_y_origins() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);

        assert_eq!(layout.staff_y_origins.len(), 2);
        assert_eq!(layout.staff_y_origins[0], 100.0);

        // Second staff starts after first staff height + gap
        let staff_height = SS * 4.0;
        let gap = INTER_STAFF_GAP_SS * SS;
        let expected_y2 = 100.0 + staff_height + gap;
        assert!(
            (layout.staff_y_origins[1] - expected_y2).abs() < 0.01,
            "second staff y: expected {expected_y2}, got {}",
            layout.staff_y_origins[1]
        );
    }

    #[test]
    fn layout_three_staves_y_origins_evenly_spaced() {
        let group = StaffGroup::section(3);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);

        assert_eq!(layout.staff_y_origins.len(), 3);
        let gap1 = layout.staff_y_origins[1] - layout.staff_y_origins[0];
        let gap2 = layout.staff_y_origins[2] - layout.staff_y_origins[1];
        assert!(
            (gap1 - gap2).abs() < 0.01,
            "staves should be evenly spaced: gap1={gap1}, gap2={gap2}"
        );
    }

    #[test]
    fn brace_layout_present_for_grand_staff() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 50.0, SS, 5000.0);

        let brace = layout.brace.as_ref().expect("grand staff should have brace");
        assert_eq!(brace.glyph, Glyph::Brace);
        // Span covers both staves and the inter-staff gap.
        // staff_height = 4 ss; gap = 6 ss; 2 staves → 14 ss total.
        let expected_span = 14.0 * SS;
        assert!(
            (brace.span_height() - expected_span).abs() < 0.01,
            "brace span_height: expected {expected_span}, got {}",
            brace.span_height()
        );
        assert_eq!(brace.y_top, 50.0, "brace top should be at staff system top");
        let expected_bottom = 50.0 + expected_span;
        assert!(
            (brace.y_bottom - expected_bottom).abs() < 0.01,
            "brace bottom: expected {expected_bottom}, got {}",
            brace.y_bottom
        );
    }

    #[test]
    fn brace_center_is_midpoint() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let brace = layout.brace.as_ref().unwrap();

        let top = layout.staff_y_origins[0];
        let bottom = layout.staff_y_origins[1] + SS * 4.0;
        let expected_center = (top + bottom) / 2.0;

        assert!(
            (brace.y_center() - expected_center).abs() < 0.01,
            "brace center: expected {expected_center}, got {}",
            brace.y_center()
        );
        // y_top and y_bottom should bracket the center.
        assert!(brace.y_top < brace.y_center());
        assert!(brace.y_bottom > brace.y_center());
    }

    #[test]
    fn brace_x_is_left_of_staff() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let brace = layout.brace.as_ref().unwrap();
        assert!(brace.x < 0.0, "brace should be left of x=0");
    }

    #[test]
    fn bracket_layout_present_for_section() {
        let group = StaffGroup::section(3);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);

        assert!(layout.brace.is_none(), "section should not have brace");
        let bracket = layout.bracket.as_ref().expect("section should have bracket");
        assert_eq!(bracket.y_top, 0.0);
        assert!(bracket.y_bottom > bracket.y_top);
        assert!(bracket.thickness > 0.0);
        assert_eq!(bracket.top_glyph, Glyph::BracketTop);
        assert_eq!(bracket.bottom_glyph, Glyph::BracketBottom);
    }

    #[test]
    fn bracket_spans_from_top_to_bottom() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 100.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();

        assert_eq!(bracket.y_top, 100.0);
        let expected_bottom = layout.staff_y_origins[1] + SS * 4.0;
        assert!(
            (bracket.y_bottom - expected_bottom).abs() < 0.01,
            "bracket bottom: expected {expected_bottom}, got {}",
            bracket.y_bottom
        );
    }

    #[test]
    fn no_connector_for_independent_group() {
        let group = StaffGroup::independent(3);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);

        assert!(layout.brace.is_none());
        assert!(layout.bracket.is_none());
    }

    #[test]
    fn single_staff_no_brace() {
        let group = StaffGroup {
            staff_count: 1,
            connector: ConnectorKind::Brace,
            joined_barlines: false,
            sub_brackets: Vec::new(),
        };
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);

        assert_eq!(layout.staff_y_origins.len(), 1);
        // Brace not generated for single staff even if requested
        assert!(layout.brace.is_none());
    }

    #[test]
    fn total_height_two_staves() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);

        let expected = layout.staff_y_origins[1] + SS * 4.0;
        assert!(
            (layout.total_height() - expected).abs() < 0.01,
            "total_height: expected {expected}, got {}",
            layout.total_height()
        );
    }

    #[test]
    fn total_height_empty() {
        let group = StaffGroup::independent(0);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        assert_eq!(layout.total_height(), 0.0);
    }

    #[test]
    fn staff_layouts_from_multi_count() {
        let group = StaffGroup::grand_staff();
        let multi = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let staves = staff_layouts_from_multi(&multi, 10.0, 5000.0);

        assert_eq!(staves.len(), 2);
        assert_eq!(staves[0].x, 10.0);
        assert_eq!(staves[0].width, 5000.0);
        assert_eq!(staves[0].staff_space, SS);
        assert_eq!(staves[0].y_origin, multi.staff_y_origins[0]);
        assert_eq!(staves[1].y_origin, multi.staff_y_origins[1]);
    }

    #[test]
    fn staff_layouts_y_origins_match() {
        let group = StaffGroup::section(3);
        let multi = layout_multi_staff(&group, 50.0, SS, 8000.0);
        let staves = staff_layouts_from_multi(&multi, 0.0, 8000.0);

        for (i, staff) in staves.iter().enumerate() {
            assert_eq!(
                staff.y_origin, multi.staff_y_origins[i],
                "staff {i} y_origin mismatch"
            );
        }
    }

    #[test]
    fn inter_staff_gap_uses_constant() {
        let group = StaffGroup::grand_staff();
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let expected_gap = INTER_STAFF_GAP_SS * SS;
        assert!(
            (layout.inter_staff_gap - expected_gap).abs() < 0.01,
            "gap: expected {expected_gap}, got {}",
            layout.inter_staff_gap
        );
    }

    #[test]
    fn bracket_uses_smufl_scroll_glyphs() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();
        // The decorative scrolls are SMuFL `bracketTop`/`bracketBottom` — the
        // published-engraving convention for orchestral grouping brackets.
        assert_eq!(bracket.top_glyph, Glyph::BracketTop);
        assert_eq!(bracket.bottom_glyph, Glyph::BracketBottom);
        // Top and bottom are distinct glyphs (scroll curves in opposite
        // vertical directions). Catches a regression where both fields
        // accidentally got the same value.
        assert_ne!(bracket.top_glyph, bracket.bottom_glyph);
    }

    #[test]
    fn bracket_thickness_matches_smufl_engraving_default() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();
        // Bravura's `bracketThickness` is 0.5 staff spaces; in design units
        // that's 0.5 * SS. Locks the constant choice — if anyone ever
        // bumps `BRACKET_THICKNESS_SS` to track an updated SMuFL convention,
        // this fires for confirmation.
        let expected = 0.5 * SS;
        assert!(
            (bracket.thickness - expected).abs() < 1e-6,
            "bracket.thickness: expected {expected}, got {}",
            bracket.thickness
        );
    }

    #[test]
    fn bracket_x_is_left_of_staff_by_thickness() {
        let group = StaffGroup::section(2);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let bracket = layout.bracket.as_ref().unwrap();
        // The bracket's left edge sits one bracket-thickness to the left of
        // the staff origin (x=0), so the thick line's right edge meets x=0.
        let expected_x = -BRACKET_THICKNESS_SS * SS;
        assert!(
            (bracket.x - expected_x).abs() < 1e-6,
            "bracket.x: expected {expected_x}, got {}",
            bracket.x
        );
    }

    #[test]
    fn no_sub_brackets_emitted_when_group_has_none() {
        let group = StaffGroup::section(4);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        assert!(layout.sub_brackets.is_empty());
    }

    #[test]
    fn sub_brackets_ignored_for_non_bracket_connector() {
        // Sub-brackets only render inside a Bracket-connected group. Setting
        // them on a Brace or None group is a no-op rather than an error.
        let group = StaffGroup::grand_staff()
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        assert!(
            layout.sub_brackets.is_empty(),
            "Brace-connected group must drop sub-brackets, got {:?}",
            layout.sub_brackets
        );

        let group_none = StaffGroup::independent(3)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout_none = layout_multi_staff(&group_none, 0.0, SS, 5000.0);
        assert!(layout_none.sub_brackets.is_empty());
    }

    #[test]
    fn sub_bracket_emitted_for_valid_range() {
        // A single sub-bracket spanning staves 0..2 within a 5-staff bracket
        // group (e.g. Violin I + Violin II within a string section).
        let group = StaffGroup::section(5)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);

        assert_eq!(layout.sub_brackets.len(), 1);
        let sb = &layout.sub_brackets[0];
        // y_top matches the first spanned staff's origin (staff 0 → y_start).
        assert_eq!(sb.y_top, 0.0);
        // y_bottom matches the last spanned staff's bottom (staff 1).
        let staff_height = SS * 4.0;
        let expected_bottom = layout.staff_y_origins[1] + staff_height;
        assert!(
            (sb.y_bottom - expected_bottom).abs() < 1e-6,
            "sub_bracket.y_bottom: expected {expected_bottom}, got {}",
            sb.y_bottom
        );
    }

    #[test]
    fn sub_bracket_thickness_matches_smufl_default() {
        let group = StaffGroup::section(3)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let sb = &layout.sub_brackets[0];
        // Bravura's `subBracketThickness` is 0.16 staff-spaces.
        let expected = SUB_BRACKET_THICKNESS_SS * SS;
        assert!(
            (sb.thickness - expected).abs() < 1e-6,
            "sub_bracket.thickness: expected {expected}, got {}",
            sb.thickness
        );
        // Sub-bracket is meaningfully thinner than the main bracket so the
        // nesting hierarchy is visually unambiguous.
        let main_thickness = BRACKET_THICKNESS_SS * SS;
        assert!(
            sb.thickness < main_thickness,
            "sub_bracket thickness {} must be < main bracket thickness {}",
            sb.thickness, main_thickness
        );
    }

    #[test]
    fn sub_bracket_sits_inside_main_bracket() {
        // When sub-brackets are present, the main bracket shifts left to make
        // room and the sub-bracket sits between the main bracket's right edge
        // and the staff origin (x=0). This test pins the exact x-positions
        // against the SS-scaled engraving-default constants so a regression
        // that drops the shift or the gap fires.
        let group = StaffGroup::section(3)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 2 }]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let main = layout.bracket.as_ref().unwrap();
        let sb = &layout.sub_brackets[0];

        // Main bracket x shifts left by (gap + sub_thickness + gap):
        // x = -(0.5 + 0.3 + 0.16 + 0.3) * SS = -1.26 * SS.
        let expected_main_x =
            -(BRACKET_THICKNESS_SS + SUB_BRACKET_GAP_SS + SUB_BRACKET_THICKNESS_SS
                + SUB_BRACKET_GAP_SS)
                * SS;
        assert!(
            (main.x - expected_main_x).abs() < 1e-6,
            "main bracket.x with sub-brackets: expected {expected_main_x}, got {}",
            main.x
        );
        // Sub-bracket x = main_right_edge + gap = -(sub_thickness + gap) * SS.
        let expected_sb_x = -(SUB_BRACKET_THICKNESS_SS + SUB_BRACKET_GAP_SS) * SS;
        assert!(
            (sb.x - expected_sb_x).abs() < 1e-6,
            "sub_bracket.x: expected {expected_sb_x}, got {}",
            sb.x
        );
        // Verify the geometric relationships: main is left of sub; sub is
        // left of the staff origin; there's a gap between them.
        assert!(main.x < sb.x, "main bracket must be left of sub-bracket");
        assert!(sb.x + sb.thickness < 0.0, "sub-bracket must end before staff origin");
        let gap_between = sb.x - (main.x + main.thickness);
        let expected_gap = SUB_BRACKET_GAP_SS * SS;
        assert!(
            (gap_between - expected_gap).abs() < 1e-6,
            "gap between main bracket right edge and sub-bracket left edge: \
             expected {expected_gap}, got {gap_between}"
        );
    }

    #[test]
    fn main_bracket_x_unchanged_when_no_sub_brackets() {
        // Anti-regression for the conditional shift: a section bracket with
        // no sub-brackets must keep its previous x position so existing
        // visual baselines (e.g. the `grand_staff` golden, the
        // `multi_staff_cross_system` golden) stay byte-identical.
        let group = StaffGroup::section(3);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        let main = layout.bracket.as_ref().unwrap();
        let expected = -BRACKET_THICKNESS_SS * SS;
        assert!(
            (main.x - expected).abs() < 1e-6,
            "main bracket.x without sub-brackets: expected {expected}, got {}",
            main.x
        );
    }

    #[test]
    fn multiple_sub_brackets_each_produce_one_layout() {
        // 6 staves: two nested sub-brackets covering staves 0..2 and 3..5.
        // Mirrors a string section with Violin I/II grouped + Viola/Cello
        // grouped inside a single Bracket.
        let group = StaffGroup::section(6).with_sub_brackets(vec![
            SubBracket { start_index: 0, staff_count: 2 },
            SubBracket { start_index: 3, staff_count: 3 },
        ]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        assert_eq!(layout.sub_brackets.len(), 2);

        let staff_height = SS * 4.0;
        // First sub-bracket spans staves 0..1.
        let sb0 = &layout.sub_brackets[0];
        assert_eq!(sb0.y_top, layout.staff_y_origins[0]);
        assert!(
            (sb0.y_bottom - (layout.staff_y_origins[1] + staff_height)).abs() < 1e-6
        );
        // Second sub-bracket spans staves 3..5.
        let sb1 = &layout.sub_brackets[1];
        assert_eq!(sb1.y_top, layout.staff_y_origins[3]);
        assert!(
            (sb1.y_bottom - (layout.staff_y_origins[5] + staff_height)).abs() < 1e-6
        );
        // Both sub-brackets share x (they sit in the same vertical column
        // just inside the main bracket).
        assert!((sb0.x - sb1.x).abs() < 1e-6);
        // And neither is identical: they cover different vertical ranges.
        assert_ne!(sb0.y_top, sb1.y_top);
    }

    #[test]
    fn invalid_sub_brackets_silently_dropped() {
        // 4-staff group with three invalid sub-bracket entries and one
        // valid one. Only the valid entry should produce a layout. The
        // invalid cases: (a) staff_count = 1 (single staff is not a group),
        // (b) start_index out of range, (c) range overshoots parent.
        let group = StaffGroup::section(4).with_sub_brackets(vec![
            SubBracket { start_index: 0, staff_count: 1 },  // too small
            SubBracket { start_index: 4, staff_count: 2 },  // start out of range
            SubBracket { start_index: 2, staff_count: 5 },  // overshoots end
            SubBracket { start_index: 0, staff_count: 2 },  // valid
        ]);
        let layout = layout_multi_staff(&group, 0.0, SS, 5000.0);
        assert_eq!(
            layout.sub_brackets.len(),
            1,
            "only the one valid sub-bracket should produce a layout entry"
        );
    }

    #[test]
    fn sub_bracket_full_span_equals_parent_bracket_span() {
        // A sub-bracket covering the entire parent group should span exactly
        // the same y range as the main bracket. Confirms y math agrees.
        let group = StaffGroup::section(3)
            .with_sub_brackets(vec![SubBracket { start_index: 0, staff_count: 3 }]);
        let layout = layout_multi_staff(&group, 50.0, SS, 5000.0);
        let main = layout.bracket.as_ref().unwrap();
        let sb = &layout.sub_brackets[0];
        assert!((sb.y_top - main.y_top).abs() < 1e-6);
        assert!((sb.y_bottom - main.y_bottom).abs() < 1e-6);
    }
}
