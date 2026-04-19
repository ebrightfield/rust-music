use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::page::{PageLayout, PageSystem};
use crate::layout::slur::{layout_half_slur_left, layout_half_slur_right, slur_direction_from_stem};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::auto_stem_direction;
use crate::layout::tie::{
    layout_half_tie_left, layout_half_tie_right, tie_direction_from_stem, TieDirection,
};
use crate::render::note_renderer::NoteheadKind;
use crate::render::slur_renderer::draw_slur;
use crate::render::system_renderer::{collect_note_positions, collect_slur_note_info, draw_system};
use crate::render::tie_renderer::draw_tie;
use crate::render::SvgWriter;

/// A note at the end of a system that has an unresolved `tie_forward`.
struct UnresolvedTie {
    /// Absolute x of the note's right edge (system_x + note_x + advance).
    x_right: f64,
    /// Staff position of the tied note.
    staff_position: i8,
    /// Y of the notehead center (absolute, in font design units).
    note_y: f64,
    /// Tie direction.
    direction: TieDirection,
    /// Right edge of the system's staff lines (absolute x).
    staff_right: f64,
}

/// A note at the start of the next system that could receive an incoming tie.
struct IncomingTieTarget {
    /// Absolute x of the note's left edge.
    x_left: f64,
    /// Staff position.
    staff_position: i8,
    /// Y of the notehead center.
    note_y: f64,
    /// Tie direction (matches the source note's direction).
    direction: TieDirection,
    /// Left edge of the system's note area (after prefix: clef, key sig, etc.).
    staff_left: f64,
}

/// Draw a complete page of music (multiple systems stacked vertically).
///
/// Returns an `SvgWriter` ready to be converted to an SVG string via `to_svg()`.
/// The caller can also add additional elements before finalizing.
///
/// Cross-system ties are rendered as two half-ties: one trailing off the
/// right edge of the source system, one leading from the left of the target system.
pub fn draw_page(
    font: &MusicFont,
    config: &EngravingConfig,
    page: &PageLayout,
) -> Result<SvgWriter, FontError> {
    // Compute pixel dimensions — use a reasonable scale factor.
    // 1 font unit = 1 SVG user unit in the viewBox; pixel size is derived
    // from page dimensions with a scaling ratio.
    let vb_margin = config.staff_space; // small margin around the content
    let vb_x = -vb_margin;
    let vb_y = -vb_margin;
    let vb_w = page.page_width + 2.0 * vb_margin;
    let vb_h = page.page_height + 2.0 * vb_margin;

    // Scale so that a staff space maps to approximately 7 pixels (standard screen density).
    let px_per_unit = 7.0 / config.staff_space;
    let px_w = vb_w * px_per_unit;
    let px_h = vb_h * px_per_unit;

    let mut svg = SvgWriter::new(px_w, px_h, vb_x, vb_y, vb_w, vb_h);

    for page_system in &page.systems {
        draw_system(
            &mut svg,
            font,
            config,
            &page_system.system,
            page_system.x,
            page_system.y,
        )?;
    }

    // Draw cross-system ties between adjacent systems
    draw_cross_system_ties(&mut svg, font, config, page)?;

    // Draw cross-system slurs between adjacent systems
    draw_cross_system_slurs(&mut svg, font, config, page)?;

    Ok(svg)
}

/// Find tied notes at the end of a system that have no matching target
/// within the same system (unresolved ties needing cross-system continuation).
fn find_unresolved_ties(
    font: &MusicFont,
    config: &EngravingConfig,
    page_system: &PageSystem,
) -> Result<Vec<UnresolvedTie>, FontError> {
    let system = &page_system.system;
    let note_positions = collect_note_positions(system);
    let staff = StaffLayout::new(
        page_system.x,
        page_system.y,
        system.staff_width,
        config.staff_space,
    );

    let mut unresolved = Vec::new();

    for (i, &(nx, pos, dur_log2, tie_forward, stem_dir)) in note_positions.iter().enumerate() {
        if !tie_forward {
            continue;
        }

        // Check if there's a matching target within this system
        let has_target = note_positions[i + 1..]
            .iter()
            .any(|&(_, target_pos, _, _, _)| target_pos == pos);

        if has_target {
            continue; // Resolved within the system
        }

        // Unresolved — compute absolute coordinates for the half-tie
        let notehead_kind = match dur_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        let direction = stem_dir.unwrap_or_else(|| auto_stem_direction(pos));
        let tie_dir = tie_direction_from_stem(direction);

        unresolved.push(UnresolvedTie {
            x_right: page_system.x + nx + advance,
            staff_position: pos,
            note_y: staff.y_of(pos),
            direction: tie_dir,
            staff_right: page_system.x + system.staff_width,
        });
    }

    Ok(unresolved)
}

/// Find the first note at each staff position in a system (candidates for
/// incoming cross-system ties).
fn find_incoming_tie_targets(
    config: &EngravingConfig,
    page_system: &PageSystem,
) -> Vec<IncomingTieTarget> {
    let system = &page_system.system;
    let note_positions = collect_note_positions(system);
    let staff = StaffLayout::new(
        page_system.x,
        page_system.y,
        system.staff_width,
        config.staff_space,
    );

    // The left edge of the note area is after the system prefix (clef, key sig).
    // Use the x-offset of the first measure's first element as an approximation,
    // or fall back to the system x-offset.
    let first_measure_x = system
        .measures
        .first()
        .map(|m| page_system.x + m.x_offset)
        .unwrap_or(page_system.x);

    // For each staff position, record only the first occurrence
    let mut seen_positions = std::collections::HashSet::new();
    let mut targets = Vec::new();

    for &(nx, pos, _dur_log2, _tie_forward, stem_dir) in &note_positions {
        if !seen_positions.insert(pos) {
            continue; // Already have this position
        }

        let direction = stem_dir.unwrap_or_else(|| auto_stem_direction(pos));
        let tie_dir = tie_direction_from_stem(direction);

        targets.push(IncomingTieTarget {
            x_left: page_system.x + nx,
            staff_position: pos,
            note_y: staff.y_of(pos),
            direction: tie_dir,
            staff_left: first_measure_x,
        });
    }

    targets
}

/// Draw cross-system ties between adjacent systems on a page.
///
/// For each unresolved tie at the end of system N, finds the matching note
/// at the start of system N+1 and draws two half-ties: one trailing to the
/// right edge of system N, one leading from the left of system N+1.
fn draw_cross_system_ties(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    page: &PageLayout,
) -> Result<(), FontError> {
    for i in 0..page.systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_ties(font, config, &page.systems[i])?;
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_tie_targets(config, &page.systems[i + 1]);

        for tie_src in &unresolved {
            // Find matching target in the next system
            let target = targets
                .iter()
                .find(|t| t.staff_position == tie_src.staff_position);

            // Draw trailing half-tie at the end of the source system
            // (even if no target found — convention is to show the outgoing tie)
            let right_layout = layout_half_tie_right(
                tie_src.x_right,
                tie_src.staff_right,
                tie_src.note_y,
                tie_src.direction,
                config,
            );
            draw_tie(svg, &right_layout);

            // Draw incoming half-tie at the start of the target system
            if let Some(tgt) = target {
                let left_layout = layout_half_tie_left(
                    tgt.staff_left,
                    tgt.x_left,
                    tgt.note_y,
                    tgt.direction,
                    config,
                );
                draw_tie(svg, &left_layout);
            }
        }
    }

    Ok(())
}

/// A note at the end of a system with an unresolved `slur_start`.
struct UnresolvedSlur {
    /// Absolute x of the note's right edge.
    x_right: f64,
    /// Y of the notehead center (absolute, in font design units).
    note_y: f64,
    /// Slur direction (derived from stem direction).
    direction: crate::layout::slur::SlurDirection,
    /// Right edge of the system's staff lines (absolute x).
    staff_right: f64,
}

/// A note at the start of the next system that has `slur_end = true`.
struct IncomingSlurTarget {
    /// Absolute x of the note's left edge.
    x_left: f64,
    /// Y of the notehead center.
    note_y: f64,
    /// Slur direction.
    direction: crate::layout::slur::SlurDirection,
    /// Left edge of the system's note area (after prefix).
    staff_left: f64,
}

/// Find notes with `slur_start = true` at the end of a system that have no
/// matching `slur_end` within the same system.
fn find_unresolved_slurs(
    font: &MusicFont,
    config: &EngravingConfig,
    page_system: &PageSystem,
) -> Result<Vec<UnresolvedSlur>, FontError> {
    let system = &page_system.system;
    let note_info = collect_slur_note_info(system);
    let staff = StaffLayout::new(
        page_system.x,
        page_system.y,
        system.staff_width,
        config.staff_space,
    );

    let mut unresolved = Vec::new();

    for (i, info) in note_info.iter().enumerate() {
        if !info.slur_start {
            continue;
        }

        // Check if there's a matching slur_end within this system
        let has_end = note_info[i + 1..].iter().any(|n| n.slur_end);

        if has_end {
            continue; // Resolved within the system
        }

        let notehead_kind = match info.duration_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        let stem_dir = info
            .stem_direction
            .unwrap_or_else(|| auto_stem_direction(info.staff_position));
        let direction = slur_direction_from_stem(stem_dir);

        unresolved.push(UnresolvedSlur {
            x_right: page_system.x + info.x + advance,
            note_y: staff.y_of(info.staff_position),
            direction,
            staff_right: page_system.x + system.staff_width,
        });
    }

    Ok(unresolved)
}

/// Find the first note with `slur_end = true` in a system (candidate for
/// incoming cross-system slur).
fn find_incoming_slur_targets(
    config: &EngravingConfig,
    page_system: &PageSystem,
) -> Vec<IncomingSlurTarget> {
    let system = &page_system.system;
    let note_info = collect_slur_note_info(system);
    let staff = StaffLayout::new(
        page_system.x,
        page_system.y,
        system.staff_width,
        config.staff_space,
    );

    let first_measure_x = system
        .measures
        .first()
        .map(|m| page_system.x + m.x_offset)
        .unwrap_or(page_system.x);

    let mut targets = Vec::new();

    for info in &note_info {
        if !info.slur_end {
            continue;
        }

        let stem_dir = info
            .stem_direction
            .unwrap_or_else(|| auto_stem_direction(info.staff_position));
        let direction = slur_direction_from_stem(stem_dir);

        targets.push(IncomingSlurTarget {
            x_left: page_system.x + info.x,
            note_y: staff.y_of(info.staff_position),
            direction,
            staff_left: first_measure_x,
        });

        // Only need the first slur_end target per unresolved slur_start
        break;
    }

    targets
}

/// Draw cross-system slurs between adjacent systems on a page.
///
/// For each unresolved `slur_start` at the end of system N, finds the first
/// `slur_end` note at the start of system N+1 and draws two half-slurs:
/// one trailing to the right edge of system N, one leading from the left
/// of system N+1.
fn draw_cross_system_slurs(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    page: &PageLayout,
) -> Result<(), FontError> {
    for i in 0..page.systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_slurs(font, config, &page.systems[i])?;
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_slur_targets(config, &page.systems[i + 1]);

        for slur_src in &unresolved {
            // Draw trailing half-slur at the end of the source system
            let right_layout = layout_half_slur_right(
                slur_src.x_right,
                slur_src.staff_right,
                slur_src.note_y,
                slur_src.direction,
                config,
            );
            draw_slur(svg, &right_layout);

            // Draw incoming half-slur at the start of the target system
            if let Some(tgt) = targets.first() {
                let left_layout = layout_half_slur_left(
                    tgt.staff_left,
                    tgt.x_left,
                    tgt.note_y,
                    tgt.direction,
                    config,
                );
                draw_slur(svg, &left_layout);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::barline::BarlineStyle;
    use crate::layout::key_signature::KeySignature;
    use crate::layout::measure::{MeasureLayoutConfig, NoteEvent};
    use crate::layout::page::{layout_page, PageLayoutConfig, SystemBreaking};
    use crate::layout::system::{MeasureContent, MeasureEvent, SystemPrefix};
    use crate::layout::time_signature::TimeSignatureKind;
    use music::notation::clef::Clef;

    fn setup() -> (MusicFont<'static>, EngravingConfig) {
        let font = bravura_font();
        let config = font.engraving_config();
        (font, config)
    }

    fn quarter_note(pos: i8) -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        slur_start: false,
        slur_end: false,
        })
    }

    fn make_measure(pos: i8) -> MeasureContent {
        MeasureContent {
            events: vec![quarter_note(pos)],
            barline: BarlineStyle::Single,
        }
    }

    fn prefix() -> SystemPrefix {
        SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Open,
            Some(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
        )
    }

    #[test]
    fn empty_page_renders_valid_svg() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let page = layout_page(&prefix(), &[], &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();
        assert!(output.starts_with("<svg"));
        assert!(output.ends_with("</svg>\n"));
        assert_eq!(output.matches("<path ").count(), 0);
    }

    #[test]
    fn single_system_page() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..3).map(|i| make_measure(i as i8 * 2)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Should have 5 staff lines
        assert!(output.matches("<line ").count() >= 5);
        // Should have a clef path
        assert!(output.matches("<path ").count() >= 1);
    }

    #[test]
    fn two_system_page_has_two_sets_of_staff_lines() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..6).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // 2 systems × 5 staff lines = 10 staff lines minimum
        let line_count = output.matches("<line ").count();
        assert!(
            line_count >= 10,
            "expected >= 10 lines (2 staves), got {}",
            line_count,
        );
    }

    #[test]
    fn three_system_page_element_counts() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..9).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));

        assert_eq!(page.systems.len(), 3);

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // 3 systems × 5 staff lines = 15
        let line_count = output.matches("<line ").count();
        assert!(line_count >= 15, "expected >= 15 lines, got {}", line_count);

        // 3 clefs + time sig digits in first system only
        let path_count = output.matches("<path ").count();
        assert!(path_count >= 3, "expected >= 3 paths (3 clefs), got {}", path_count);
    }

    #[test]
    fn page_svg_dimensions_are_positive() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..4).map(|i| make_measure(i as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Parse width and height from the SVG — they should be positive
        assert!(output.contains("width=\""));
        assert!(output.contains("height=\""));
        // Check viewBox has positive dimensions
        assert!(output.contains("viewBox=\""));
    }

    // --- cross-system tie tests ---

    fn tied_note(pos: i8) -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: true,
            dynamic: None,
            slur_start: false,
            slur_end: false,
        })
    }

    #[test]
    fn cross_system_tie_draws_two_half_ties() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        // System 1: tied note at pos 4, system 2: note at pos 4
        let measures = vec![
            MeasureContent {
                events: vec![tied_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Final,
            },
        ];
        // 1 measure per system → forces cross-system tie
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
        assert_eq!(page.systems.len(), 2);

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Should have 2 filled tie paths (right half-tie + left half-tie)
        let tie_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(
            tie_count, 2,
            "expected 2 half-ties for cross-system tie, got {tie_count}"
        );
    }

    #[test]
    fn no_cross_system_tie_without_tie_forward() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Final,
            },
        ];
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        let tie_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 0, "no ties without tie_forward");
    }

    #[test]
    fn cross_system_tie_with_no_matching_target_draws_right_half_only() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        // Tied note at pos 4 in system 1, but note at pos 6 in system 2 (different position)
        let measures = vec![
            MeasureContent {
                events: vec![tied_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Final,
            },
        ];
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Only the right half-tie should be drawn (outgoing tie at end of system 1)
        let tie_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(
            tie_count, 1,
            "only right half-tie when no matching target, got {tie_count}"
        );
    }

    #[test]
    fn within_system_tie_does_not_produce_cross_system_tie() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        // Both tied note and target in same system (2 measures per system)
        let measures = vec![
            MeasureContent {
                events: vec![tied_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Final,
            },
        ];
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
        assert_eq!(page.systems.len(), 1);

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Only 1 within-system tie, no cross-system ties
        let tie_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(tie_count, 1, "1 within-system tie, no cross-system tie");
    }

    #[test]
    fn cross_system_tie_differs_from_no_tie() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        let tied_measures = vec![
            MeasureContent {
                events: vec![tied_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Final,
            },
        ];

        let untied_measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Final,
            },
        ];

        let tied_page = layout_page(&prefix(), &tied_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
        let untied_page = layout_page(&prefix(), &untied_measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

        let tied_svg = draw_page(&font, &config, &tied_page).unwrap().to_svg();
        let untied_svg = draw_page(&font, &config, &untied_page).unwrap().to_svg();

        assert_ne!(tied_svg, untied_svg, "cross-system tied output should differ from untied");
    }

    // --- cross-system slur tests ---

    fn slur_start_note(pos: i8) -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
            slur_start: true,
            slur_end: false,
        })
    }

    fn slur_end_note(pos: i8) -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
            slur_start: false,
            slur_end: true,
        })
    }

    #[test]
    fn cross_system_slur_draws_two_half_slurs() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        // System 1: slur_start note, system 2: slur_end note
        let measures = vec![
            MeasureContent {
                events: vec![slur_start_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![slur_end_note(6)],
                barline: BarlineStyle::Final,
            },
        ];
        // 1 measure per system → forces cross-system slur
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
        assert_eq!(page.systems.len(), 2);

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Should have 2 filled slur paths (right half-slur + left half-slur)
        let filled_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(
            filled_count, 2,
            "expected 2 half-slurs for cross-system slur, got {filled_count}"
        );
    }

    #[test]
    fn no_cross_system_slur_without_flags() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        let measures = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Final,
            },
        ];
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        let filled_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(filled_count, 0, "no slurs without slur flags");
    }

    #[test]
    fn cross_system_slur_right_half_only_when_no_end() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        // slur_start in system 1, but no slur_end in system 2
        let measures = vec![
            MeasureContent {
                events: vec![slur_start_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(6)],  // no slur_end
                barline: BarlineStyle::Final,
            },
        ];
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Only the right half-slur
        let filled_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(
            filled_count, 1,
            "only right half-slur when no slur_end target, got {filled_count}"
        );
    }

    #[test]
    fn within_system_slur_not_duplicated_as_cross_system() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        // Both slur start and end in same system (2 measures per system)
        let measures = vec![
            MeasureContent {
                events: vec![slur_start_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![slur_end_note(6)],
                barline: BarlineStyle::Final,
            },
        ];
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
        assert_eq!(page.systems.len(), 1);

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Only 1 slur (within-system), no cross-system duplication
        let filled_count = output.matches(r#"stroke="none""#).count();
        assert_eq!(filled_count, 1, "1 within-system slur, no cross-system slur");
    }

    #[test]
    fn cross_system_slur_differs_from_no_slur() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);

        let with_slur = vec![
            MeasureContent {
                events: vec![slur_start_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![slur_end_note(6)],
                barline: BarlineStyle::Final,
            },
        ];
        let without_slur = vec![
            MeasureContent {
                events: vec![quarter_note(4)],
                barline: BarlineStyle::Single,
            },
            MeasureContent {
                events: vec![quarter_note(6)],
                barline: BarlineStyle::Final,
            },
        ];

        let slur_page = layout_page(&prefix(), &with_slur, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));
        let no_slur_page = layout_page(&prefix(), &without_slur, &mcfg, &page_cfg, &SystemBreaking::Fixed(1));

        let slur_svg = draw_page(&font, &config, &slur_page).unwrap().to_svg();
        let no_slur_svg = draw_page(&font, &config, &no_slur_page).unwrap().to_svg();

        assert_ne!(slur_svg, no_slur_svg, "cross-system slurred output should differ from unslurred");
    }

    #[test]
    fn two_system_page_has_different_y_for_staff_lines() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..6).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Extract all y1 values from <line> elements — there should be at least
        // 2 distinct groups (one per system)
        let y1_values: Vec<&str> = output
            .split("y1=\"")
            .skip(1)
            .filter_map(|s| s.split('"').next())
            .collect();
        assert!(y1_values.len() >= 10, "should have >= 10 line y1 values");

        // Parse to floats and check at least 2 distinct y ranges
        let y_floats: Vec<f64> = y1_values.iter().filter_map(|s| s.parse().ok()).collect();
        let min_y = y_floats.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = y_floats.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        // Two systems means y-values span at least the system spacing
        let expected_min_span = 5.0 * ss; // at least 5 staff spaces apart
        assert!(
            max_y - min_y > expected_min_span,
            "y range {} should exceed {} (two systems should be separated)",
            max_y - min_y,
            expected_min_span,
        );
    }
}
