use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::glissando::{layout_half_glissando_left, layout_half_glissando_right, GlissandoStyle};
use crate::layout::hairpin::layout_hairpin;
use crate::layout::lyric::{LyricContinuation, LYRIC_BELOW_STAFF_SS, LYRIC_FONT_SIZE_SS};
use crate::layout::ornament::{layout_ornament, Ornament};
use crate::layout::ottava::{layout_ottava_bracket, OttavaKind};
use crate::layout::page::{PageLayout, PageSystem};
use crate::layout::slur::{layout_half_slur_left, layout_half_slur_right, slur_direction_from_stem};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::auto_stem_direction;
use crate::layout::tie::{
    layout_half_tie_left, layout_half_tie_right, tie_direction_from_stem, TieDirection,
};
use crate::layout::trill_bracket::{
    layout_trill_bracket_hooks_multi_speed, HookDirection, TrillBracketSide,
};
use crate::layout::trill_extension::{
    layout_trill_extension_multi_speed, layout_trill_extension_with_glyph,
    trill_extension_right_edge, TrillSpeedRampSpec, TrillWiggleSpeed,
};
use crate::render::trill_bracket_renderer::{draw_trill_bracket_hook, draw_trill_bracket_hooks};
use crate::render::note_renderer::NoteheadKind;
use crate::render::glissando_renderer::draw_glissando;
use crate::render::hairpin_renderer::draw_hairpin;
use crate::render::lyric_renderer::{draw_lyric_extender, draw_lyric_hyphen};
use crate::render::ottava_renderer::draw_ottava_bracket;
use crate::render::slur_renderer::draw_slur;
use crate::render::system_renderer::{
    collect_glissando_note_info, collect_hairpin_note_info, collect_lyric_note_info,
    collect_note_positions, collect_ottava_note_info, collect_slur_note_info,
    collect_trill_extension_note_info, draw_system, layout_trill_end_hook,
    TRILL_BRACKET_HOOK_LENGTH_SS, TRILL_EXTENSION_NOTE_GAP_SS,
};
use crate::render::tie_renderer::draw_tie;
use crate::render::trill_extension_renderer::{
    draw_trill_extension, draw_trill_extension_multi_speed,
};
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
    draw_cross_system_ties(&mut svg, font, config, &page.systems)?;

    // Draw cross-system slurs between adjacent systems
    draw_cross_system_slurs(&mut svg, font, config, &page.systems)?;

    // Draw cross-system hairpins between adjacent systems
    draw_cross_system_hairpins(&mut svg, font, config, &page.systems)?;

    // Draw cross-system lyric extender lines between adjacent systems
    draw_cross_system_lyric_extenders(&mut svg, config, &page.systems);

    // Draw cross-system lyric hyphens between adjacent systems
    draw_cross_system_lyric_hyphens(&mut svg, config, &page.systems);

    // Draw cross-system ottava brackets between adjacent systems
    draw_cross_system_ottava_brackets(&mut svg, font, config, &page.systems)?;

    // Draw cross-system glissando lines between adjacent systems
    draw_cross_system_glissandos(&mut svg, font, config, &page.systems)?;

    // Draw cross-system trill wavy-line extensions between adjacent systems
    draw_cross_system_trill_extensions(&mut svg, font, config, &page.systems)?;

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
        dominant_baseline: "auto",
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

/// Draw cross-system ties between adjacent systems.
///
/// For each unresolved tie at the end of system N, finds the matching note
/// at the start of system N+1 and draws two half-ties: one trailing to the
/// right edge of system N, one leading from the left of system N+1.
pub(crate) fn draw_cross_system_ties(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    systems: &[PageSystem],
) -> Result<(), FontError> {
    for i in 0..systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_ties(font, config, &systems[i])?;
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_tie_targets(config, &systems[i + 1]);

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

/// Draw cross-system slurs between adjacent systems.
///
/// For each unresolved `slur_start` at the end of system N, finds the first
/// `slur_end` note at the start of system N+1 and draws two half-slurs:
/// one trailing to the right edge of system N, one leading from the left
/// of system N+1.
pub(crate) fn draw_cross_system_slurs(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    systems: &[PageSystem],
) -> Result<(), FontError> {
    for i in 0..systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_slurs(font, config, &systems[i])?;
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_slur_targets(config, &systems[i + 1]);

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

/// Draw cross-system hairpins between adjacent systems.
///
/// For each unresolved `hairpin_start` at the end of system N, finds the first
/// `hairpin_end` note at the start of system N+1 and draws two half-hairpins:
/// one trailing to the right edge of system N, one leading from the left
/// of system N+1.
pub(crate) fn draw_cross_system_hairpins(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    systems: &[PageSystem],
) -> Result<(), FontError> {
    let stroke_width = config.staff_line_thickness_fu();

    for i in 0..systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_hairpins(font, config, &systems[i])?;
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_hairpin_targets(config, &systems[i + 1]);

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

/// Draw cross-system lyric extender lines between adjacent systems on a page.
///
/// When a lyric syllable with `Extender` continuation appears on the last note
/// of a system and the held syllable continues into the next system, two
/// half-extender lines are drawn: one trailing from the source syllable to the
/// right edge of the system, and one leading from the left of the next system
/// to the first note's position (the rhythmic event where the sustained
/// syllable ends).
pub(crate) fn draw_cross_system_lyric_extenders(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    systems: &[PageSystem],
) {
    let stroke_width = config.staff_line_thickness_fu();

    for i in 0..systems.len().saturating_sub(1) {
        let src_system = &systems[i];
        let note_info = collect_lyric_note_info(&src_system.system);

        // Find the last note with an extender that has no target within the
        // same system (i.e., it's the last note or the remaining notes have
        // no next target to resolve it).
        let Some(last_extender) = find_last_unresolved_extender(&note_info) else {
            continue;
        };

        let src_staff = StaffLayout::new(
            src_system.x,
            src_system.y,
            src_system.system.staff_width,
            config.staff_space,
        );
        let src_y_baseline =
            src_staff.y_of(0) + LYRIC_BELOW_STAFF_SS * config.staff_space;

        // Draw trailing half-extender from the source syllable to the right
        // edge of the system.
        let x_from = src_system.x + last_extender.x;
        let x_to = src_system.x + src_system.system.staff_width;
        draw_lyric_extender(svg, x_from, x_to, src_y_baseline, config.staff_space, stroke_width);

        // Draw incoming half-extender at the start of the target system.
        let tgt_system = &systems[i + 1];
        let tgt_note_info = collect_lyric_note_info(&tgt_system.system);

        if let Some(first_note) = tgt_note_info.first() {
            let tgt_staff = StaffLayout::new(
                tgt_system.x,
                tgt_system.y,
                tgt_system.system.staff_width,
                config.staff_space,
            );
            let tgt_y_baseline =
                tgt_staff.y_of(0) + LYRIC_BELOW_STAFF_SS * config.staff_space;

            // Start from the left edge of the first measure's content area
            let first_measure_x = tgt_system
                .system
                .measures
                .first()
                .map(|m| tgt_system.x + m.x_offset)
                .unwrap_or(tgt_system.x);

            let x_to_tgt = tgt_system.x + first_note.x;
            draw_lyric_extender(
                svg,
                first_measure_x,
                x_to_tgt,
                tgt_y_baseline,
                config.staff_space,
                stroke_width,
            );
        }
    }
}

/// Find the last note in a system that has an extender lyric with no target
/// note following it within the same system.
///
/// An extender is "unresolved" if there is no subsequent note/chord in the
/// note_info list to draw the extender line to — meaning the held syllable
/// continues past the system boundary.
fn find_last_unresolved_extender(note_info: &[crate::render::system_renderer::LyricNoteInfo]) -> Option<&crate::render::system_renderer::LyricNoteInfo> {
    // Walk backwards: the last note with an extender that has no next note
    // to resolve it is the one whose index == note_info.len() - 1, since the
    // within-system renderer would have drawn the line if a target existed.
    // More generally, the last extender with no subsequent note is unresolved.
    for (i, info) in note_info.iter().enumerate().rev() {
        let Some(ref lyric) = info.lyric else {
            continue;
        };
        if lyric.continuation == LyricContinuation::Extender {
            // Check if there's a next note to resolve it within the system
            if i + 1 < note_info.len() {
                // Resolved within the system — skip
                return None;
            }
            return Some(info);
        }
        // If we hit a note without an extender, no unresolved extender exists
        // (any earlier extender would have had a target note after it).
        return None;
    }
    None
}

/// Draw cross-system lyric hyphens between adjacent systems on a page.
///
/// When the last syllable on a system has `Hyphen` continuation, the hyphen
/// is drawn between the source syllable (on the source system) and the first
/// syllable-bearing note on the next system. The single hyphen is centered
/// over the system break — the closer half of the gap on the source system,
/// the closer half on the target system. We approximate this with one hyphen
/// per side: a trailing hyphen near the right edge of the source system and
/// a leading hyphen near the left edge of the target system. Some engraving
/// conventions draw only one (closest to the syllable text); we draw both
/// for visual symmetry, which Gould describes as acceptable.
pub(crate) fn draw_cross_system_lyric_hyphens(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    systems: &[PageSystem],
) {
    let font_size = LYRIC_FONT_SIZE_SS * config.staff_space;

    for i in 0..systems.len().saturating_sub(1) {
        let src_system = &systems[i];
        let note_info = collect_lyric_note_info(&src_system.system);

        // Find the last note with a Hyphen continuation that has no
        // syllable-bearing target within the same system (otherwise the
        // within-system pass already drew it).
        let Some(last_hyphen) = find_last_unresolved_hyphen(&note_info) else {
            continue;
        };

        let src_staff = StaffLayout::new(
            src_system.x,
            src_system.y,
            src_system.system.staff_width,
            config.staff_space,
        );
        let src_y_baseline = src_staff.y_of(0) + LYRIC_BELOW_STAFF_SS * config.staff_space;

        // Trailing hyphen on the source system, between the source syllable
        // and the right edge of the system's staff.
        let from_x_src = src_system.x + last_hyphen.x;
        let to_x_src = src_system.x + src_system.system.staff_width;
        draw_lyric_hyphen(
            svg,
            from_x_src,
            to_x_src,
            src_y_baseline,
            font_size,
            config.staff_space,
        );

        // Leading hyphen on the target system, between the left edge of the
        // target system's content area and the first syllable-bearing note.
        let tgt_system = &systems[i + 1];
        let tgt_note_info = collect_lyric_note_info(&tgt_system.system);
        let Some(first_lyric_note) = tgt_note_info.iter().find(|n| n.lyric.is_some()) else {
            continue;
        };

        let tgt_staff = StaffLayout::new(
            tgt_system.x,
            tgt_system.y,
            tgt_system.system.staff_width,
            config.staff_space,
        );
        let tgt_y_baseline = tgt_staff.y_of(0) + LYRIC_BELOW_STAFF_SS * config.staff_space;

        let first_measure_x = tgt_system
            .system
            .measures
            .first()
            .map(|m| tgt_system.x + m.x_offset)
            .unwrap_or(tgt_system.x);
        let to_x_tgt = tgt_system.x + first_lyric_note.x;
        draw_lyric_hyphen(
            svg,
            first_measure_x,
            to_x_tgt,
            tgt_y_baseline,
            font_size,
            config.staff_space,
        );
    }
}

/// Find the last note in a system with `Hyphen` continuation that has no
/// subsequent syllable-bearing note in the same system.
///
/// Walks backwards: the search ends at the first syllable-bearing note. If
/// that syllable has hyphen continuation but no successor with a lyric, it is
/// unresolved and the hyphen crosses the system boundary. If it has any other
/// continuation (or its successor has a lyric within the system), no
/// cross-system hyphen is needed.
fn find_last_unresolved_hyphen(
    note_info: &[crate::render::system_renderer::LyricNoteInfo],
) -> Option<&crate::render::system_renderer::LyricNoteInfo> {
    for (i, info) in note_info.iter().enumerate().rev() {
        let Some(ref lyric) = info.lyric else {
            continue;
        };
        if lyric.continuation != LyricContinuation::Hyphen {
            // Last syllable carries no hyphen — nothing crosses the boundary.
            return None;
        }
        // Does any subsequent note in the system carry a lyric?
        let has_target = note_info.iter().skip(i + 1).any(|n| n.lyric.is_some());
        if has_target {
            // Within-system hyphen already drawn — not unresolved.
            return None;
        }
        return Some(info);
    }
    None
}

/// An ottava bracket at the end of a system with no matching `ottava_end`.
struct UnresolvedOttava {
    /// Absolute x of the note (system_x + note_x).
    x_start: f64,
    /// Ottava kind (8va, 8vb, 15ma, 15mb).
    kind: OttavaKind,
    /// Right edge of the system's staff lines (absolute x).
    staff_right: f64,
}

/// A note at the start of the next system that has `ottava_end = true`.
struct IncomingOttavaTarget {
    /// Absolute x of the note's right edge.
    x_right: f64,
}

/// Find notes with `ottava_start` at the end of a system that have no
/// matching `ottava_end` within the same system.
fn find_unresolved_ottavas(
    font: &MusicFont,
    page_system: &PageSystem,
) -> Result<Vec<UnresolvedOttava>, FontError> {
    let system = &page_system.system;
    let note_info = collect_ottava_note_info(system);

    let mut unresolved = Vec::new();

    for (i, info) in note_info.iter().enumerate() {
        let Some(kind) = info.ottava_start else {
            continue;
        };

        // Check if there's a matching ottava_end within this system
        let has_end = note_info[i + 1..].iter().any(|n| n.ottava_end);

        if has_end {
            continue; // Resolved within the system
        }

        let notehead_kind = match info.duration_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let _outline = font.glyph_outline(notehead_kind.glyph())?;

        unresolved.push(UnresolvedOttava {
            x_start: page_system.x + info.x,
            kind,
            staff_right: page_system.x + system.staff_width,
        });
    }

    Ok(unresolved)
}

/// Find the first note with `ottava_end = true` in a system.
fn find_incoming_ottava_targets(
    font: &MusicFont,
    page_system: &PageSystem,
) -> Result<Vec<IncomingOttavaTarget>, FontError> {
    let system = &page_system.system;
    let note_info = collect_ottava_note_info(system);

    let mut targets = Vec::new();

    for info in &note_info {
        if !info.ottava_end {
            continue;
        }

        let notehead_kind = match info.duration_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        targets.push(IncomingOttavaTarget {
            x_right: page_system.x + info.x + advance,
        });

        break; // Only need the first
    }

    Ok(targets)
}

/// Draw cross-system ottava brackets between adjacent systems.
///
/// For each unresolved `ottava_start` at the end of system N, draws a
/// trailing half-bracket (no end hook) to the right edge of system N.
/// If system N+1 has a matching `ottava_end`, draws an incoming
/// half-bracket (with end hook, no label) from the left edge of system N+1.
///
/// An unresolved `ottava_start` on the **last** system of the page is also
/// drawn as a trailing half-bracket (no end hook) to the system's right
/// edge — Gould's convention reads this as "ottava continues beyond what's
/// notated here," matching how mid-page cross-system trailing brackets
/// terminate. Without this handling, an ottava that starts but never ends
/// before the page break would silently disappear.
pub(crate) fn draw_cross_system_ottava_brackets(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    systems: &[PageSystem],
) -> Result<(), FontError> {
    for i in 0..systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_ottavas(font, &systems[i])?;
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_ottava_targets(font, &systems[i + 1])?;

        for ott_src in &unresolved {
            let src_staff = StaffLayout::new(
                systems[i].x,
                systems[i].y,
                systems[i].system.staff_width,
                config.staff_space,
            );

            // Trailing half-bracket: from start note to right edge of system,
            // no end hook (bracket continues into the next system).
            let right_layout = layout_ottava_bracket(
                ott_src.kind,
                ott_src.x_start,
                ott_src.staff_right,
                &src_staff,
                config.staff_space,
                false, // no end hook
            );
            draw_ottava_bracket(svg, &right_layout);

            // Incoming half-bracket at start of target system
            if let Some(tgt) = targets.first() {
                let tgt_system = &systems[i + 1];
                let tgt_staff = StaffLayout::new(
                    tgt_system.x,
                    tgt_system.y,
                    tgt_system.system.staff_width,
                    config.staff_space,
                );

                // Start from the first measure's content area
                let first_measure_x = tgt_system
                    .system
                    .measures
                    .first()
                    .map(|m| tgt_system.x + m.x_offset)
                    .unwrap_or(tgt_system.x);

                // Incoming bracket: from left edge to the end note,
                // with end hook (bracket terminates here).
                // Use a minimal label width so the dashed line starts promptly.
                let incoming_layout = layout_ottava_bracket(
                    ott_src.kind,
                    first_measure_x,
                    tgt.x_right,
                    &tgt_staff,
                    config.staff_space,
                    true, // end hook
                );
                draw_ottava_bracket(svg, &incoming_layout);
            }
        }
    }

    // Final-system case: an ottava_start with no matching ottava_end anywhere
    // on the page (in particular, on the last system itself with no later
    // system to spill into) still needs its trailing half-bracket drawn so
    // the reader sees "ottava began here." This mirrors the cross-system
    // trailing case above — no end hook, dashed line to the system's right
    // edge — and matches the trill-extension cross-system convention.
    if let Some(last) = systems.last() {
        let unresolved = find_unresolved_ottavas(font, last)?;
        let last_staff = StaffLayout::new(
            last.x,
            last.y,
            last.system.staff_width,
            config.staff_space,
        );
        for ott_src in &unresolved {
            let trailing_layout = layout_ottava_bracket(
                ott_src.kind,
                ott_src.x_start,
                ott_src.staff_right,
                &last_staff,
                config.staff_space,
                false, // no end hook — bracket "continues beyond the page"
            );
            draw_ottava_bracket(svg, &trailing_layout);
        }
    }

    Ok(())
}

/// A note at the end of a system with an unresolved `glissando_start`.
struct UnresolvedGlissando {
    /// Absolute x of the note.
    x: f64,
    /// Staff position of the source note.
    staff_position: i8,
    /// Glissando style (Line or LineWithText).
    style: GlissandoStyle,
    /// Right edge of the system's staff lines (absolute x).
    staff_right: f64,
}

/// The first note in the next system (target for incoming cross-system glissando).
struct IncomingGlissandoTarget {
    /// Absolute x of the target note.
    x: f64,
    /// Staff position of the target note.
    staff_position: i8,
    /// Left edge of the system's note area (after prefix).
    staff_left: f64,
}

/// Find the last note with `glissando_start` that has no subsequent note
/// within the same system to resolve against.
fn find_unresolved_glissandos(
    page_system: &PageSystem,
) -> Vec<UnresolvedGlissando> {
    let system = &page_system.system;
    let notes = collect_glissando_note_info(system);

    let mut unresolved = Vec::new();

    for (i, note) in notes.iter().enumerate() {
        let Some(style) = note.glissando_start else {
            continue;
        };

        // If there's a next note in this system, the glissando resolves within-system
        if i + 1 < notes.len() {
            continue;
        }

        unresolved.push(UnresolvedGlissando {
            x: page_system.x + note.x,
            staff_position: note.staff_position,
            style,
            staff_right: page_system.x + system.staff_width,
        });
    }

    unresolved
}

/// Find the first note in a system (candidate target for incoming cross-system glissando).
fn find_incoming_glissando_targets(
    page_system: &PageSystem,
) -> Vec<IncomingGlissandoTarget> {
    let system = &page_system.system;
    let notes = collect_glissando_note_info(system);

    let first_measure_x = system
        .measures
        .first()
        .map(|m| page_system.x + m.x_offset)
        .unwrap_or(page_system.x);

    let mut targets = Vec::new();

    if let Some(first_note) = notes.first() {
        targets.push(IncomingGlissandoTarget {
            x: page_system.x + first_note.x,
            staff_position: first_note.staff_position,
            staff_left: first_measure_x,
        });
    }

    targets
}

/// Draw cross-system glissando lines between adjacent systems.
///
/// For each unresolved `glissando_start` at the end of system N (the last note
/// with a glissando that has no subsequent note within the system), draws a
/// trailing half-glissando to the right edge of system N. If system N+1 has
/// a note, draws an incoming half-glissando from the left edge of system N+1
/// to that note.
pub(crate) fn draw_cross_system_glissandos(
    svg: &mut SvgWriter,
    _font: &MusicFont,
    config: &EngravingConfig,
    systems: &[PageSystem],
) -> Result<(), FontError> {
    for i in 0..systems.len().saturating_sub(1) {
        let unresolved = find_unresolved_glissandos(&systems[i]);
        if unresolved.is_empty() {
            continue;
        }

        let targets = find_incoming_glissando_targets(&systems[i + 1]);

        for gliss_src in &unresolved {
            let src_staff = StaffLayout::new(
                systems[i].x,
                systems[i].y,
                systems[i].system.staff_width,
                config.staff_space,
            );

            // Trailing half-glissando from source note to right edge
            if let Some(right_layout) = layout_half_glissando_right(
                gliss_src.x,
                gliss_src.staff_position,
                gliss_src.staff_right,
                &src_staff,
                gliss_src.style,
            ) {
                draw_glissando(svg, &right_layout);
            }

            // Incoming half-glissando at start of target system
            if let Some(tgt) = targets.first() {
                let tgt_staff = StaffLayout::new(
                    systems[i + 1].x,
                    systems[i + 1].y,
                    systems[i + 1].system.staff_width,
                    config.staff_space,
                );

                if let Some(left_layout) = layout_half_glissando_left(
                    tgt.staff_left,
                    tgt.x,
                    tgt.staff_position,
                    &tgt_staff,
                ) {
                    draw_glissando(svg, &left_layout);
                }
            }
        }
    }

    Ok(())
}

/// A trill-with-extension on the **last note** of a system whose wavy line
/// needs to resume on the following system.
///
/// `system_renderer::draw_system_trill_extensions` already terminates the
/// source-system wiggle at the system's right edge (independent of the next
/// system); this struct only carries the geometry the page renderer needs
/// to draw the *incoming* wiggle on system N+1.
struct UnresolvedTrillExtension {
    /// Y-coordinate of the trill "tr" glyph on the source system, expressed
    /// as an offset from the **top staff line** (`y_of(8)`) of that system.
    ///
    /// Stored as an offset (not an absolute y) so the incoming wiggle on
    /// system N+1 — which sits at a different page y — can re-anchor to its
    /// own staff and land at the same height-above-the-staff as the source
    /// trill. Without this, two stacked systems on the same page would
    /// produce wiggles drifting visually relative to their staves.
    y_above_top_line: f64,
    /// Optional bracket side requested by the user on the source trill note.
    /// The page renderer uses this to decide whether to cap the incoming
    /// wiggle on system N+1 with an End hook: `Some(End | Both)` → yes;
    /// `Some(Start)` or `None` → no. The Start hook (if requested) was
    /// already drawn on the source system by `draw_system_trill_extensions`.
    bracket: Option<TrillBracketSide>,
    /// Optional override for the bracket hook direction. Only meaningful when
    /// `bracket` is `Some(End | Both)`. `None` selects `HookDirection::Down`.
    bracket_direction: Option<HookDirection>,
    /// Optional override for the bracket hook length, in staff spaces. Only
    /// meaningful when `bracket` is `Some(End | Both)`. `None` selects the
    /// default `TRILL_BRACKET_HOOK_LENGTH_SS`.
    bracket_length_ss: Option<f64>,
    /// Speed variant for the wavy-line glyph. The incoming wiggle on system
    /// N+1 must tile the same glyph as the trailing wiggle on system N so a
    /// reader sees one continuous wavy line of consistent density across the
    /// line break. `None` selects the standard wiggle.
    ///
    /// Ignored for glyph selection when `speed_ramp.is_some()`: the
    /// multi-speed renderer reads per-region speeds from the ramp and the
    /// incoming wiggle on system N+1 re-synthesizes those regions against
    /// the target system's span.
    wiggle_speed: Option<TrillWiggleSpeed>,
    /// Optional multi-speed ramp spec, mirroring the source-system
    /// dispatch. `None` selects the single-speed incoming path (using
    /// `wiggle_speed`); `Some(spec)` engages the multi-speed incoming
    /// path: regions are re-synthesized across the target system's
    /// `[staff_left, first_note.x - gap]` span and tiled by
    /// `draw_trill_extension_multi_speed`.
    ///
    /// Re-synthesis (rather than carrying the source-system regions
    /// across) is deliberate: the multi-speed convention is
    /// "evenly-distributed regions across the wiggle span." The source-
    /// system span and the target-system span are different lengths in
    /// general, so reusing the source's region geometry would compress or
    /// stretch the speed progression asymmetrically across the line
    /// break. Re-synthesizing per system keeps each system's wiggle
    /// region-uniform on its own terms and matches the within-system
    /// convention exactly.
    speed_ramp: Option<TrillSpeedRampSpec>,
}

/// The first note on the target system (system N+1) that an incoming
/// cross-system wiggle terminates at.
struct IncomingTrillExtensionTarget {
    /// Absolute x of the target note's notehead left edge.
    x: f64,
    /// Absolute x of the start of the target system's note content area
    /// (after the system prefix: clef, key sig, time sig). The incoming
    /// wiggle starts here.
    staff_left: f64,
}

/// Find a trill-with-extension on the last note of a system whose wavy line
/// should continue into the next system.
///
/// Returns at most one entry per system because the wiggle model is "one
/// trill extends until the next note." A trill whose immediately following
/// note exists within the same system has already been resolved by
/// `draw_system_trill_extensions` and contributes nothing here.
fn find_unresolved_trill_extension(
    page_system: &PageSystem,
    staff_space: f64,
) -> Option<UnresolvedTrillExtension> {
    let system = &page_system.system;
    let notes = collect_trill_extension_note_info(system);
    let last = notes.last()?;
    if !last.has_trill_extension {
        return None;
    }
    // An explicit length terminates the wiggle within the source system
    // unconditionally (matching the within-system clamp in
    // `draw_system_trill_extensions`), so the trill is "resolved" — no
    // incoming wiggle on system N+1, no cross-system End hook either.
    if matches!(last.explicit_length_ss, Some(len) if len > 0.0) {
        return None;
    }
    // Recompute the source ornament y the same way system_renderer does,
    // then strip the staff's absolute y so the offset is portable across
    // systems on the same page.
    let src_staff = StaffLayout::new(
        page_system.x,
        page_system.y,
        system.staff_width,
        staff_space,
    );
    let ornament = layout_ornament(
        Ornament::Trill,
        page_system.x + last.x,
        last.staff_position,
        &src_staff,
    );
    Some(UnresolvedTrillExtension {
        y_above_top_line: ornament.y - src_staff.y_of(8),
        bracket: last.bracket,
        bracket_direction: last.bracket_direction,
        bracket_length_ss: last.bracket_length_ss,
        wiggle_speed: last.wiggle_speed,
        speed_ramp: last.speed_ramp,
    })
}

/// Find the first note on a system that an incoming cross-system trill
/// extension should terminate at.
fn find_incoming_trill_extension_target(
    page_system: &PageSystem,
) -> Option<IncomingTrillExtensionTarget> {
    let system = &page_system.system;
    let notes = collect_trill_extension_note_info(system);
    let first = notes.first()?;

    let staff_left = system
        .measures
        .first()
        .map(|m| page_system.x + m.x_offset)
        .unwrap_or(page_system.x);

    Some(IncomingTrillExtensionTarget {
        x: page_system.x + first.x,
        staff_left,
    })
}

/// Draw the incoming half of a cross-system trill wavy line on system N+1.
///
/// When a trill-with-extension lands on the last note of system N, the
/// system renderer already extends the wiggle to system N's right edge.
/// This pass adds the matching incoming wiggle on system N+1: starting at
/// the staff's note-content left edge (after clef/key/time prefix) and
/// terminating just short of the system's first notehead.
///
/// The incoming wiggle is anchored to the target staff's top line at the
/// same height-above-the-staff as the source trill glyph, so the wavy line
/// reads as a continuation regardless of the inter-system gap.
///
/// If system N+1 has no notes (an empty system), no incoming wiggle is
/// drawn — there's nothing for it to lead up to. If the available span
/// (staff_left → first note) is smaller than a single wiggle segment,
/// `layout_trill_extension` returns `None` and the renderer silently
/// skips, matching the within-system fail-safe.
pub(crate) fn draw_cross_system_trill_extensions(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    systems: &[PageSystem],
) -> Result<(), FontError> {
    if systems.len() < 2 {
        return Ok(());
    }

    let staff_space = config.staff_space;

    for i in 0..systems.len() - 1 {
        let Some(src) = find_unresolved_trill_extension(&systems[i], staff_space) else {
            continue;
        };
        let Some(tgt) = find_incoming_trill_extension_target(&systems[i + 1]) else {
            continue;
        };

        let tgt_system = &systems[i + 1];
        let tgt_staff = StaffLayout::new(
            tgt_system.x,
            tgt_system.y,
            tgt_system.system.staff_width,
            staff_space,
        );

        let y = tgt_staff.y_of(8) + src.y_above_top_line;
        let start_x = tgt.staff_left;
        let end_x = tgt.x - TRILL_EXTENSION_NOTE_GAP_SS * staff_space;

        // Dispatch on the source-system trill's ramp: multi-speed (ramp
        // present) supersedes the single-speed path. Region geometry is
        // re-synthesized against the *target* system's span so the
        // incoming wiggle's regions are evenly distributed across the new
        // span (matching the within-system convention). The source-system
        // wiggle's per-region tiling is not carried across the line break.
        if let Some(spec) = src.speed_ramp {
            // Synthesizer is font-agnostic; thread per-glyph advance
            // lookup through. Returns `None` for degenerate specs
            // (region_count==0, Linear with region_count==1) or
            // non-positive spans (e.g. when start_x >= end_x because the
            // target system's first note sits at the staff_left). The
            // None fall-through matches the within-system fail-safe.
            let regions = match spec.ramp.synthesize_regions(
                start_x,
                end_x,
                spec.region_count,
                |speed| font.glyph_advance(speed.to_glyph()).unwrap_or(0) as f64,
            ) {
                Some(r) => r,
                None => continue,
            };
            if let Some(layout) =
                layout_trill_extension_multi_speed(end_x, y, &regions)
            {
                draw_trill_extension_multi_speed(svg, font, &layout)?;

                // End hook on the incoming wiggle. Anchored at the
                // right edge of the multi-speed layout's last tile
                // (via `layout_trill_bracket_hooks_multi_speed`).
                // Source-system already drew the Start hook (if any).
                if matches!(src.bracket, Some(TrillBracketSide::End | TrillBracketSide::Both)) {
                    let hook_stroke = config.thin_barline_thickness_fu();
                    let hook_length = src
                        .bracket_length_ss
                        .map(|ss| ss * staff_space)
                        .unwrap_or(TRILL_BRACKET_HOOK_LENGTH_SS * staff_space);
                    let direction = src.bracket_direction.unwrap_or(HookDirection::Down);
                    // The within-system multi-speed path strips Start
                    // for cross-system trills via
                    // `bracket_side_for_system_pass` — we mirror that
                    // here by drawing End only (Start was suppressed at
                    // source). Passing `TrillBracketSide::End` directly
                    // would suffice, but going through the multi-speed
                    // helper keeps the anchor logic in one place.
                    let hooks = layout_trill_bracket_hooks_multi_speed(
                        &layout,
                        TrillBracketSide::End,
                        hook_length,
                        direction,
                        hook_stroke,
                    );
                    draw_trill_bracket_hooks(svg, &hooks);
                }
            }
            continue;
        }

        // Single-speed path. Tile the incoming wiggle with the same speed
        // glyph as the source wiggle so a sustained trill reads as one
        // continuous wavy line of consistent density across the line
        // break.
        let wiggle_glyph = src.wiggle_speed.unwrap_or_default().to_glyph();
        let wiggle_advance = font.glyph_advance(wiggle_glyph)? as f64;

        if let Some(layout) =
            layout_trill_extension_with_glyph(start_x, end_x, y, wiggle_glyph, wiggle_advance)
        {
            draw_trill_extension(svg, font, &layout)?;

            // End hook (cross-system continuation): if the user asked for an
            // End or Both bracket on the source trill, cap the incoming
            // wiggle on system N+1 with a vertical hook at its right edge.
            // The Start hook (if any) was already drawn on system N. We
            // place the hook flush with the wiggle's *visible* terminus
            // (the right edge of the final whole-segment tile) — the same
            // anchor used by `layout_trill_bracket_hooks`, so within-system
            // and cross-system End hooks land identically.
            if matches!(src.bracket, Some(TrillBracketSide::End | TrillBracketSide::Both)) {
                let hook_stroke = config.thin_barline_thickness_fu();
                let hook_length = src
                    .bracket_length_ss
                    .map(|ss| ss * staff_space)
                    .unwrap_or(TRILL_BRACKET_HOOK_LENGTH_SS * staff_space);
                let direction = src.bracket_direction.unwrap_or(HookDirection::Down);
                let hook_x = trill_extension_right_edge(&layout);
                let hook = layout_trill_end_hook(hook_x, y, hook_length, direction, hook_stroke);
                draw_trill_bracket_hook(svg, &hook);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
