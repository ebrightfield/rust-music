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
}

impl StaffGroup {
    /// Grand staff for keyboard instruments: 2 staves, brace, joined barlines.
    pub fn grand_staff() -> Self {
        Self {
            staff_count: 2,
            connector: ConnectorKind::Brace,
            joined_barlines: true,
        }
    }

    /// Orchestral section bracket: N staves, bracket, joined barlines.
    pub fn section(staff_count: usize) -> Self {
        Self {
            staff_count,
            connector: ConnectorKind::Bracket,
            joined_barlines: true,
        }
    }

    /// Independent staves with no connector or joined barlines.
    pub fn independent(staff_count: usize) -> Self {
        Self {
            staff_count,
            connector: ConnectorKind::None,
            joined_barlines: false,
        }
    }
}

/// Spacing constants for multi-staff layout, in staff spaces.
pub const INTER_STAFF_GAP_SS: f64 = 6.0;
/// Brace glyph placement offset from the staff edge, in staff spaces.
pub const BRACE_LEFT_OFFSET_SS: f64 = 0.5;
/// Bracket line thickness, in staff spaces.
pub const BRACKET_THICKNESS_SS: f64 = 0.5;

/// Computed geometry for a brace connector.
#[derive(Clone, Debug)]
pub struct BraceLayout {
    /// SMuFL glyph for the brace.
    pub glyph: Glyph,
    /// X position of the brace glyph (left of the staff system).
    pub x: f64,
    /// Y position of the brace glyph center (midpoint between top of first
    /// staff and bottom of last staff).
    pub y_center: f64,
    /// Total height the brace must span (top of first staff to bottom of last).
    pub span_height: f64,
    /// Scale factor to apply to the brace glyph to match `span_height`.
    pub scale_y: f64,
}

/// Computed geometry for a bracket connector.
///
/// The bracket is rendered as a thick vertical line with decorative SMuFL
/// scroll glyphs (`bracketTop`, `bracketBottom`) at each end — the published
/// engraving convention for orchestral grouping brackets. Each glyph's origin
/// (its inner edge where the scroll meets the line) sits at the corresponding
/// endpoint of the thick line, so the line segment and glyph join cleanly
/// without an explicit serif stroke.
#[derive(Clone, Debug)]
pub struct BracketLayout {
    /// X position of the bracket's thick vertical line (left edge).
    pub x: f64,
    /// Y of the top of the thick line (where `top_glyph`'s origin anchors).
    pub y_top: f64,
    /// Y of the bottom of the thick line (where `bottom_glyph`'s origin anchors).
    pub y_bottom: f64,
    /// Thickness of the vertical line, in font design units.
    pub thickness: f64,
    /// SMuFL glyph for the decorative top scroll. The glyph's origin sits at
    /// the bottom-left of its bounding box (font convention: bBoxSW); its
    /// outline extends upward and rightward, so drawing it translated to
    /// `(x, y_top)` makes the scroll appear above `y_top` while joining the
    /// thick line at `y_top` exactly.
    pub top_glyph: Glyph,
    /// SMuFL glyph for the decorative bottom scroll. The glyph's origin sits
    /// at the top-left of its bounding box (font convention: bBoxNW for the
    /// bottom variant); its outline extends downward and rightward, so
    /// drawing it translated to `(x, y_bottom)` makes the scroll appear
    /// below `y_bottom` while joining the thick line at `y_bottom` exactly.
    pub bottom_glyph: Glyph,
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
        let span_height = bottom_staff_top + staff_height - top;
        let y_center = top + span_height / 2.0;

        // The SMuFL brace glyph is designed to span 1 staff space in height
        // at em-square scale. We scale it to match the total span.
        let design_height = staff_space;
        let scale_y = span_height / design_height;

        Some(BraceLayout {
            glyph: Glyph::Brace,
            x: -BRACE_LEFT_OFFSET_SS * staff_space,
            y_center,
            span_height,
            scale_y,
        })
    } else {
        None
    };

    let bracket = if group.connector == ConnectorKind::Bracket && group.staff_count >= 2 {
        let y_top = staff_y_origins[0];
        let y_bottom = staff_y_origins[group.staff_count - 1] + staff_height;

        Some(BracketLayout {
            x: -BRACKET_THICKNESS_SS * staff_space,
            y_top,
            y_bottom,
            thickness: BRACKET_THICKNESS_SS * staff_space,
            top_glyph: Glyph::BracketTop,
            bottom_glyph: Glyph::BracketBottom,
        })
    } else {
        None
    };

    MultiStaffLayout {
        staff_y_origins,
        inter_staff_gap: gap,
        staff_space,
        brace,
        bracket,
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
        assert!(brace.span_height > 0.0);
        assert!(brace.scale_y > 1.0, "brace should be scaled up from 1 staff space");
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
            (brace.y_center - expected_center).abs() < 0.01,
            "brace center: expected {expected_center}, got {}",
            brace.y_center
        );
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
}
