use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::hairpin::layout_hairpin;
use crate::layout::page::{PageLayout, PageSystem};
use crate::layout::slur::{layout_half_slur_left, layout_half_slur_right, slur_direction_from_stem};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::auto_stem_direction;
use crate::layout::tie::{
    layout_half_tie_left, layout_half_tie_right, tie_direction_from_stem, TieDirection,
};
use crate::render::note_renderer::NoteheadKind;
use crate::render::hairpin_renderer::draw_hairpin;
use crate::render::slur_renderer::draw_slur;
use crate::render::system_renderer::{collect_hairpin_note_info, collect_note_positions, collect_slur_note_info, draw_system};
use crate::render::tie_renderer::draw_tie;
use crate::render::{SvgWriter, TextStyle};

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

    // Draw measure numbers above the start of each system
    if page.show_measure_numbers {
        draw_measure_numbers(&mut svg, config, page);
    }

    // Draw cross-system ties between adjacent systems
    draw_cross_system_ties(&mut svg, font, config, page)?;

    // Draw cross-system slurs between adjacent systems
    draw_cross_system_slurs(&mut svg, font, config, page)?;

    // Draw cross-system hairpins between adjacent systems
    draw_cross_system_hairpins(&mut svg, font, config, page)?;

    Ok(svg)
}

/// Measure number offset above the top staff line, in staff spaces.
/// Placed slightly above rehearsal/tempo mark territory (which is at ~2.5–2.8ss).
pub(crate) const MEASURE_NUMBER_ABOVE_STAFF_SS: f64 = 1.8;

/// Font size for measure numbers, in staff spaces.
pub(crate) const MEASURE_NUMBER_FONT_SIZE_SS: f64 = 1.2;

/// Draw measure numbers above the first measure of each system.
///
/// Typically shows "1" above the first system, then the measure number of
/// the first bar in each subsequent system. The number is placed above the
/// top staff line, left-aligned with the start of the first measure's
/// note content (after the prefix: clef, key sig, time sig).
fn draw_measure_numbers(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    page: &PageLayout,
) {
    let ss = config.staff_space;
    let font_size = MEASURE_NUMBER_FONT_SIZE_SS * ss;
    let y_offset = MEASURE_NUMBER_ABOVE_STAFF_SS * ss;

    let style = TextStyle {
        font_family: "serif",
        font_size,
        fill: "black",
        anchor: "start",
        font_weight: "normal",
        font_style: "normal",
    };

    for page_system in &page.systems {
        let system = &page_system.system;
        if system.measures.is_empty() {
            continue;
        }

        // x: position at the start of the first measure's content area
        // (which is after the system prefix: clef + key sig + time sig).
        let first_measure = &system.measures[0];
        let x = page_system.x + first_measure.x_offset;

        // y: above the top staff line (top line is at page_system.y)
        let y = page_system.y - y_offset;

        let number = page_system.first_measure_number;
        svg.add_text(x, y, &number.to_string(), &style);
    }
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

/// A note at the end of a system with an unresolved `hairpin_start`.
struct UnresolvedHairpin {
    /// Absolute x of the note's right edge plus spacing.
    x_right: f64,
    /// Hairpin type (crescendo or decrescendo).
    kind: crate::layout::hairpin::HairpinType,
    /// Right edge of the system's staff lines (absolute x).
    staff_right: f64,
    /// Y of the bottom staff line (absolute).
    staff_bottom_y: f64,
}

/// A note at the start of the next system that has `hairpin_end = true`.
struct IncomingHairpinTarget {
    /// Absolute x of the note's left edge minus spacing.
    x_left: f64,
    /// Left edge of the system's note area (after prefix).
    staff_left: f64,
    /// Y of the bottom staff line (absolute).
    staff_bottom_y: f64,
}

/// Find notes with `hairpin_start` at the end of a system that have no
/// matching `hairpin_end` within the same system.
fn find_unresolved_hairpins(
    font: &MusicFont,
    config: &EngravingConfig,
    page_system: &PageSystem,
) -> Result<Vec<UnresolvedHairpin>, FontError> {
    let system = &page_system.system;
    let note_info = collect_hairpin_note_info(system);
    let staff = StaffLayout::new(
        page_system.x,
        page_system.y,
        system.staff_width,
        config.staff_space,
    );

    let mut unresolved = Vec::new();

    for (i, info) in note_info.iter().enumerate() {
        let Some(hairpin_type) = info.hairpin_start else {
            continue;
        };

        // Check if there's a matching hairpin_end within this system
        let has_end = note_info[i + 1..].iter().any(|n| n.hairpin_end);

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

        // Match the 0.3ss padding used in system_renderer for hairpin start
        let x_right = page_system.x + info.x + advance + 0.3 * config.staff_space;

        unresolved.push(UnresolvedHairpin {
            x_right,
            kind: hairpin_type,
            staff_right: page_system.x + system.staff_width,
            staff_bottom_y: staff.bottom_y(),
        });
    }

    Ok(unresolved)
}

/// Find the first note with `hairpin_end = true` in a system (candidate for
/// incoming cross-system hairpin).
fn find_incoming_hairpin_targets(
    config: &EngravingConfig,
    page_system: &PageSystem,
) -> Vec<IncomingHairpinTarget> {
    let system = &page_system.system;
    let note_info = collect_hairpin_note_info(system);
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
        if !info.hairpin_end {
            continue;
        }

        // Match the 0.3ss padding used in system_renderer for hairpin end
        let x_left = page_system.x + info.x - 0.3 * config.staff_space;

        targets.push(IncomingHairpinTarget {
            x_left,
            staff_left: first_measure_x,
            staff_bottom_y: staff.bottom_y(),
        });

        // Only need the first hairpin_end target
        break;
    }

    targets
}

/// Draw cross-system hairpins between adjacent systems on a page.
///
/// For each unresolved `hairpin_start` at the end of system N, finds the first
/// `hairpin_end` note at the start of system N+1 and draws two half-hairpins:
/// one trailing to the right edge of system N, one leading from the left
/// of system N+1.
fn draw_cross_system_hairpins(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    page: &PageLayout,
) -> Result<(), FontError> {
    let stroke_width = config.staff_line_thickness_fu();

    for i in 0..page.systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_hairpins(font, config, &page.systems[i])?;
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_hairpin_targets(config, &page.systems[i + 1]);

        for hp_src in &unresolved {
            // Draw trailing half-hairpin at the end of the source system
            let right_layout = layout_hairpin(
                hp_src.kind,
                hp_src.x_right,
                hp_src.staff_right,
                hp_src.staff_bottom_y,
                config.staff_space,
                stroke_width,
            );
            draw_hairpin(svg, &right_layout);

            // Draw incoming half-hairpin at the start of the target system
            if let Some(tgt) = targets.first() {
                let left_layout = layout_hairpin(
                    hp_src.kind,
                    tgt.staff_left,
                    tgt.x_left,
                    tgt.staff_bottom_y,
                    config.staff_space,
                    stroke_width,
                );
                draw_hairpin(svg, &left_layout);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
