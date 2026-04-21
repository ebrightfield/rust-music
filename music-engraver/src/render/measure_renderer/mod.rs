use music::notation::clef::Clef;

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::beam::{layout_beam_group, BeamedNote};
use crate::layout::chord::{layout_chord_noteheads, notehead_x_offset, ChordNote};
use crate::layout::dot::dot_staff_position;
use crate::layout::measure::{
    BeamGroupEvent, ChordEvent, MeasureElement, MeasureLayout, NoteEvent, TupletGroupEvent,
};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{
    auto_stem_direction, auto_stem_direction_chord, stem_length_staff_spaces, StemDirection,
};
use crate::layout::tuplet::{layout_tuplet_bracket, tuplet_number_glyphs, tuplet_placement_from_stem};
use crate::render::beam_renderer::draw_beam_group;
use crate::render::barline_renderer::draw_barline;
use crate::render::tuplet_renderer::draw_tuplet_bracket;
use crate::render::dot_renderer::draw_dots;
use crate::render::dynamics_renderer::draw_dynamic;
use crate::render::expression_renderer::draw_expression;
use crate::render::flag_renderer::draw_flag;
use crate::render::lyric_renderer::draw_lyric;
use crate::layout::lyric::layout_lyric;
use crate::render::rehearsal_renderer::draw_rehearsal_mark;
use crate::layout::expression::layout_expression;
use crate::layout::rehearsal::layout_rehearsal_mark;
use crate::layout::articulation::layout_articulation;
use crate::layout::tempo::layout_tempo_mark;
use crate::layout::grace::layout_grace_note;
use crate::render::articulation_renderer::draw_articulation;
use crate::render::grace_renderer::draw_grace_note;
use crate::render::key_sig_renderer::draw_key_signature;
use crate::render::tempo_renderer::draw_tempo_mark;
use crate::render::note_renderer::{draw_ledger_lines, draw_notehead, NoteheadKind};
use crate::render::rest_renderer::draw_rest;
use crate::render::staff_renderer::draw_clef;
use crate::render::stem_renderer::{draw_stem, stem_endpoints, stem_x};
use crate::render::time_sig_renderer::draw_time_signature;
use crate::render::SvgWriter;

#[cfg(test)]
mod tests;

/// Draw a complete laid-out measure onto an SVG writer.
///
/// `x_offset` shifts the entire measure horizontally (for multi-measure rendering).
/// `clef_for_key_sig` determines accidental placement for key signatures.
///
/// Draws staff lines first, then iterates through positioned elements.
pub fn draw_measure(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    layout: &MeasureLayout,
    x_offset: f64,
    clef_for_key_sig: &Clef,
) -> Result<(), FontError> {
    for positioned in &layout.elements {
        let elem_x = x_offset + positioned.x;
        match &positioned.element {
            MeasureElement::Clef(clef_layout) => {
                draw_clef(svg, staff, clef_layout, font)?;
            }
            MeasureElement::KeySignature(key) => {
                draw_key_signature(svg, staff, font, elem_x, key, clef_for_key_sig)?;
            }
            MeasureElement::TimeSignature(kind) => {
                draw_time_signature(svg, staff, font, elem_x, kind)?;
            }
            MeasureElement::Note(note) => {
                draw_note_event(svg, staff, font, config, elem_x, note)?;
            }
            MeasureElement::Chord(chord) => {
                draw_chord_event(svg, staff, font, config, elem_x, chord)?;
            }
            MeasureElement::BeamGroup(bg) => {
                draw_beam_group_event(
                    svg,
                    staff,
                    font,
                    config,
                    elem_x,
                    positioned.width,
                    bg,
                )?;
            }
            MeasureElement::TupletGroup(tg) => {
                draw_tuplet_group_event(
                    svg,
                    staff,
                    font,
                    config,
                    elem_x,
                    positioned.width,
                    tg,
                )?;
            }
            MeasureElement::Rest(rest) => {
                draw_rest(svg, staff, font, elem_x, rest.duration_log2)?;
            }
            MeasureElement::Barline(style) => {
                draw_barline(svg, staff, font, elem_x, *style)?;
            }
        }
    }

    Ok(())
}

/// Notehead kind from log2 duration: 0=whole, 1=half, 2+=filled.
fn notehead_kind_from_log2(duration_log2: u8) -> NoteheadKind {
    match duration_log2 {
        0 => NoteheadKind::Whole,
        1 => NoteheadKind::Half,
        _ => NoteheadKind::Filled,
    }
}

/// Number of flags from log2 duration: 0–2 have no flags, 3=one flag, 4=two, etc.
fn flag_count_from_log2(duration_log2: u8) -> u8 {
    duration_log2.saturating_sub(2)
}

/// Draw a complete note event: accidental + notehead + ledger lines + stem + flag + dots.
fn draw_note_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    note: &NoteEvent,
) -> Result<(), FontError> {
    let kind = notehead_kind_from_log2(note.duration_log2);
    let position = note.staff_position;

    // Draw grace note before principal (if present)
    if let Some((grace_pos, grace_kind)) = note.annotations.grace_note {
        let stem_dir = note
            .stem_direction
            .unwrap_or_else(|| auto_stem_direction(position));
        let grace_layout = layout_grace_note(x, grace_pos, grace_kind, stem_dir, staff);
        draw_grace_note(svg, font, &grace_layout)?;
    }

    // Draw accidental (pre-resolved glyph) to the left of notehead
    if let Some(acc_glyph) = note.accidental {
        let outline = font.glyph_outline(acc_glyph)?;
        let acc_advance = outline.advance_width as f64;
        let padding = 0.12 * staff.staff_space;
        let acc_x = x - acc_advance - padding;
        let acc_y = staff.y_of(position);
        let transform = format!("translate({acc_x}, {acc_y})");
        svg.add_path(&outline.path_data, "black", Some(&transform));
    }

    // Draw notehead
    let advance = draw_notehead(svg, staff, font, x, position, kind)?;

    // Draw ledger lines
    draw_ledger_lines(svg, staff, config, x, advance, position);

    // Determine stem direction
    let needs_stem = note.duration_log2 >= 1; // whole notes have no stem
    let direction = if needs_stem {
        Some(note.stem_direction.unwrap_or_else(|| auto_stem_direction(position)))
    } else {
        None
    };

    // Draw stem
    if let Some(dir) = direction {
        draw_stem(svg, staff, config, x, advance, position, dir);

        // Draw flag
        let flags = flag_count_from_log2(note.duration_log2);
        if flags > 0 {
            let sx = stem_x(x, advance, dir, config.stem_thickness_fu());
            let (y_top, y_bottom) = stem_endpoints(staff, position, dir);
            let tip_y = match dir {
                StemDirection::Up => y_top,
                StemDirection::Down => y_bottom,
            };
            draw_flag(svg, font, sx, tip_y, flags, dir)?;
        }
    }

    // Draw augmentation dots
    if note.dots > 0 {
        let dot_pos = dot_staff_position(position);
        draw_dots(svg, staff, font, x, advance, dot_pos, note.dots)?;
    }

    // Draw articulation near the notehead if present
    if let Some(artic) = note.annotations.articulation {
        let stem_dir = direction.unwrap_or(StemDirection::Up);
        let note_center_x = x + advance / 2.0;
        let artic_layout =
            layout_articulation(artic, note_center_x, position, stem_dir, staff);
        draw_articulation(svg, font, &artic_layout)?;
    }

    // Draw dynamic marking below the staff if present
    if let Some(dyn_mark) = note.annotations.dynamic {
        let note_center_x = x + advance / 2.0;
        draw_dynamic(svg, staff, font, dyn_mark, note_center_x)?;
    }

    // Draw rehearsal mark above the staff if present
    if let Some((ref text, style)) = note.annotations.rehearsal_mark {
        let note_center_x = x + advance / 2.0;
        let layout = layout_rehearsal_mark(text, note_center_x, staff, config.staff_space, style);
        draw_rehearsal_mark(svg, &layout);
    }

    // Draw tempo mark above the staff if present
    if let Some(ref mark) = note.annotations.tempo_mark {
        let layout = layout_tempo_mark(mark, x, staff, config.staff_space);
        draw_tempo_mark(svg, &layout, font);
    }

    // Draw expression text below the staff if present
    if let Some(ref text) = note.annotations.expression {
        let note_center_x = x + advance / 2.0;
        let layout = layout_expression(text, note_center_x, staff, config.staff_space);
        draw_expression(svg, &layout);
    }

    // Draw lyric syllable below the staff if present
    if let Some(ref syllable) = note.annotations.lyric {
        let note_center_x = x + advance / 2.0;
        let layout = layout_lyric(syllable, note_center_x, staff, config.staff_space);
        draw_lyric(svg, &layout);
    }

    // Draw chord symbol above the staff if present
    if let Some(ref symbol) = note.annotations.chord_symbol {
        let note_center_x = x + advance / 2.0;
        let layout =
            crate::layout::chord_symbol::layout_chord_symbol(symbol, note_center_x, staff, config.staff_space);
        crate::render::chord_symbol_renderer::draw_chord_symbol(svg, &layout);
    }

    // Draw ornament above the staff if present
    if let Some(orn) = note.annotations.ornament {
        let note_center_x = x + advance / 2.0;
        let orn_layout =
            crate::layout::ornament::layout_ornament(orn, note_center_x, position, staff);
        crate::render::ornament_renderer::draw_ornament(svg, font, &orn_layout)?;
    }

    // Draw navigation sign (segno, coda) above the staff if present
    if let Some(sign) = note.annotations.navigation_sign {
        let note_center_x = x + advance / 2.0;
        let nav_layout =
            crate::layout::navigation::layout_navigation_sign(sign, note_center_x, staff);
        crate::render::navigation_renderer::draw_navigation_sign(svg, font, &nav_layout)?;
    }

    Ok(())
}

/// Draw a chord event: multiple noteheads + shared stem + ledger lines + accidentals + flag + dots.
///
/// Uses `layout_chord_noteheads` to compute notehead offsets for seconds, then draws
/// each notehead at the correct x-offset, a single shared stem spanning the full chord,
/// and optional flags/dots.
fn draw_chord_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    chord: &ChordEvent,
) -> Result<(), FontError> {
    if chord.staff_positions.is_empty() {
        return Ok(());
    }

    let kind = notehead_kind_from_log2(chord.duration_log2);
    let direction = chord.stem_direction.unwrap_or_else(|| {
        auto_stem_direction_chord(&chord.staff_positions)
    });

    // Draw grace note before chord (if present)
    if let Some((grace_pos, grace_kind)) = chord.annotations.grace_note {
        let grace_layout = layout_grace_note(x, grace_pos, grace_kind, direction, staff);
        draw_grace_note(svg, font, &grace_layout)?;
    }

    // Build ChordNote list for layout
    let chord_notes: Vec<ChordNote> = chord
        .staff_positions
        .iter()
        .zip(chord.accidentals.iter().chain(std::iter::repeat(&None)))
        .map(|(&pos, acc)| ChordNote {
            staff_position: pos,
            accidental: *acc,
        })
        .collect();

    let layouts = layout_chord_noteheads(&chord_notes, direction);

    // We need the notehead advance width for positioning
    let notehead_glyph = kind.glyph();
    let outline = font.glyph_outline(notehead_glyph)?;
    let advance = outline.advance_width as f64;

    // Draw each notehead (with offset for seconds)
    for note_layout in &layouts {
        let x_off = notehead_x_offset(note_layout.offset, direction) * advance;
        let note_x = x + x_off;

        // Draw accidental
        if let Some(acc_glyph) = note_layout.accidental {
            let acc_outline = font.glyph_outline(acc_glyph)?;
            let acc_advance = acc_outline.advance_width as f64;
            let padding = 0.12 * staff.staff_space;
            let acc_x = note_x - acc_advance - padding;
            let acc_y = staff.y_of(note_layout.staff_position);
            let transform = format!("translate({acc_x}, {acc_y})");
            svg.add_path(&acc_outline.path_data, "black", Some(&transform));
        }

        // Draw notehead
        draw_notehead(svg, staff, font, note_x, note_layout.staff_position, kind)?;

        // Draw ledger lines for this note
        draw_ledger_lines(svg, staff, config, note_x, advance, note_layout.staff_position);
    }

    // Draw shared stem spanning from the closest to farthest note
    let needs_stem = chord.duration_log2 >= 1;
    if needs_stem {
        let min_pos = layouts.iter().map(|n| n.staff_position).min().unwrap();
        let max_pos = layouts.iter().map(|n| n.staff_position).max().unwrap();

        // Stem attaches at the note closest to the tip direction:
        // stem up → bottom note is the attachment, tip extends above top note
        // stem down → top note is the attachment, tip extends below bottom note
        let (attach_pos, far_pos) = match direction {
            StemDirection::Up => (min_pos, max_pos),
            StemDirection::Down => (max_pos, min_pos),
        };

        // Compute stem tip: start from the far note and extend by standard stem length
        let stem_len_ss = stem_length_staff_spaces(far_pos, direction);
        let stem_len_fu = stem_len_ss * staff.staff_space;

        let attach_y = staff.y_of(attach_pos);
        let far_y = staff.y_of(far_pos);

        let (y_top, y_bottom) = match direction {
            StemDirection::Up => {
                let tip_y = far_y - stem_len_fu;
                (tip_y, attach_y)
            }
            StemDirection::Down => {
                let tip_y = far_y + stem_len_fu;
                (attach_y, tip_y)
            }
        };

        let thickness = config.stem_thickness_fu();
        let sx = stem_x(x, advance, direction, thickness);
        svg.add_line(sx, y_top, sx, y_bottom, "black", thickness);

        // Draw flag
        let flags = flag_count_from_log2(chord.duration_log2);
        if flags > 0 {
            let tip_y = match direction {
                StemDirection::Up => y_top,
                StemDirection::Down => y_bottom,
            };
            draw_flag(svg, font, sx, tip_y, flags, direction)?;
        }
    }

    // Draw augmentation dots (placed relative to the topmost note for stem-down,
    // bottommost for stem-up — convention: dots placed to avoid staff lines)
    if chord.dots > 0 {
        // Place dots after the rightmost notehead column
        let has_offset = layouts.iter().any(|n| n.offset);
        let dot_base_x = if has_offset { x + advance } else { x };

        // Draw dots for each note in the chord
        for note_layout in &layouts {
            let dot_pos = dot_staff_position(note_layout.staff_position);
            draw_dots(svg, staff, font, dot_base_x, advance, dot_pos, chord.dots)?;
        }
    }

    // Draw articulation near the chord if present (uses outer note position)
    if let Some(artic) = chord.annotations.articulation {
        let chord_center_x = x + advance / 2.0;
        // Articulation attaches to the note on the opposite side from the stem:
        // stem-up → articulation below → use lowest note; stem-down → above → use highest
        let attach_pos = match direction {
            StemDirection::Up => chord.staff_positions.iter().copied().min().unwrap_or(4),
            StemDirection::Down => chord.staff_positions.iter().copied().max().unwrap_or(4),
        };
        let artic_layout =
            layout_articulation(artic, chord_center_x, attach_pos, direction, staff);
        draw_articulation(svg, font, &artic_layout)?;
    }

    // Draw dynamic marking below the staff if present
    if let Some(dyn_mark) = chord.annotations.dynamic {
        let chord_center_x = x + advance / 2.0;
        draw_dynamic(svg, staff, font, dyn_mark, chord_center_x)?;
    }

    // Draw rehearsal mark above the staff if present
    if let Some((ref text, style)) = chord.annotations.rehearsal_mark {
        let chord_center_x = x + advance / 2.0;
        let layout = layout_rehearsal_mark(text, chord_center_x, staff, config.staff_space, style);
        draw_rehearsal_mark(svg, &layout);
    }

    // Draw tempo mark above the staff if present
    if let Some(ref mark) = chord.annotations.tempo_mark {
        let layout = layout_tempo_mark(mark, x, staff, config.staff_space);
        draw_tempo_mark(svg, &layout, font);
    }

    // Draw expression text below the staff if present
    if let Some(ref text) = chord.annotations.expression {
        let chord_center_x = x + advance / 2.0;
        let layout = layout_expression(text, chord_center_x, staff, config.staff_space);
        draw_expression(svg, &layout);
    }

    // Draw lyric syllable below the staff if present
    if let Some(ref syllable) = chord.annotations.lyric {
        let chord_center_x = x + advance / 2.0;
        let layout = layout_lyric(syllable, chord_center_x, staff, config.staff_space);
        draw_lyric(svg, &layout);
    }

    // Draw chord symbol above the staff if present
    if let Some(ref symbol) = chord.annotations.chord_symbol {
        let chord_center_x = x + advance / 2.0;
        let layout =
            crate::layout::chord_symbol::layout_chord_symbol(symbol, chord_center_x, staff, config.staff_space);
        crate::render::chord_symbol_renderer::draw_chord_symbol(svg, &layout);
    }

    // Draw ornament above the staff if present
    if let Some(orn) = chord.annotations.ornament {
        let chord_center_x = x + advance / 2.0;
        // Use topmost note for ornament positioning (ornaments always above)
        let top_pos = chord.staff_positions.iter().copied().max().unwrap_or(4);
        let orn_layout =
            crate::layout::ornament::layout_ornament(orn, chord_center_x, top_pos, staff);
        crate::render::ornament_renderer::draw_ornament(svg, font, &orn_layout)?;
    }

    // Draw navigation sign (segno, coda) above the staff if present
    if let Some(sign) = chord.annotations.navigation_sign {
        let chord_center_x = x + advance / 2.0;
        let nav_layout =
            crate::layout::navigation::layout_navigation_sign(sign, chord_center_x, staff);
        crate::render::navigation_renderer::draw_navigation_sign(svg, font, &nav_layout)?;
    }

    Ok(())
}

/// Draw a beam group event: noteheads + ledger lines + accidentals + dots,
/// then beams and stems via `draw_beam_group`.
///
/// Sub-notes are distributed across `total_width` using proportional spacing,
/// then beam geometry is computed from the resulting x-coordinates.
fn draw_beam_group_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    group_x: f64,
    total_width: f64,
    bg: &BeamGroupEvent,
) -> Result<(), FontError> {
    let n = bg.notes.len();
    if n == 0 {
        return Ok(());
    }

    // Compute x-positions for each note within the group using proportional spacing.
    // The shortest note gets factor 1.0; each doubling gets `ratio` more.
    let shortest_log2 = bg.notes.iter().map(|n| n.duration_log2).max().unwrap_or(3);
    let spacing_ratio = 1.6_f64;

    let factors: Vec<f64> = bg
        .notes
        .iter()
        .map(|note| {
            let steps = shortest_log2 as f64 - note.duration_log2 as f64;
            spacing_ratio.powf(steps)
        })
        .collect();
    let total_factor: f64 = factors.iter().sum();

    // Distribute total_width across notes proportionally
    let mut note_xs = Vec::with_capacity(n);
    let mut x = 0.0_f64;
    for factor in &factors {
        note_xs.push(group_x + x);
        x += total_width * factor / total_factor;
    }

    // Get notehead advance width
    let notehead_glyph = NoteheadKind::Filled.glyph();
    let outline = font.glyph_outline(notehead_glyph)?;
    let advance = outline.advance_width as f64;

    // Draw noteheads, accidentals, ledger lines, and dots for each note
    for (i, note) in bg.notes.iter().enumerate() {
        let nx = note_xs[i];

        // Accidental
        if let Some(acc_glyph) = note.accidental {
            let acc_outline = font.glyph_outline(acc_glyph)?;
            let acc_advance = acc_outline.advance_width as f64;
            let padding = 0.12 * staff.staff_space;
            let acc_x = nx - acc_advance - padding;
            let acc_y = staff.y_of(note.staff_position);
            let transform = format!("translate({acc_x}, {acc_y})");
            svg.add_path(&acc_outline.path_data, "black", Some(&transform));
        }

        // Notehead (beamed notes are always filled)
        draw_notehead(svg, staff, font, nx, note.staff_position, NoteheadKind::Filled)?;

        // Ledger lines
        draw_ledger_lines(svg, staff, config, nx, advance, note.staff_position);

        // Augmentation dots
        if note.dots > 0 {
            let dot_pos = dot_staff_position(note.staff_position);
            draw_dots(svg, staff, font, nx, advance, dot_pos, note.dots)?;
        }
    }

    // Build BeamedNote list for beam layout computation
    let beamed_notes: Vec<BeamedNote> = bg
        .notes
        .iter()
        .enumerate()
        .map(|(i, note)| BeamedNote {
            x: note_xs[i],
            staff_position: note.staff_position,
            duration_log2: note.duration_log2,
        })
        .collect();

    // Determine stem direction
    let positions: Vec<i8> = bg.notes.iter().map(|n| n.staff_position).collect();
    let direction = bg
        .stem_direction
        .unwrap_or_else(|| auto_stem_direction_chord(&positions));

    // Compute beam layout and render
    let beam_layout = layout_beam_group(&beamed_notes, direction, staff.staff_space);
    draw_beam_group(svg, staff, config, &beamed_notes, &beam_layout, advance);

    Ok(())
}

/// Draw a tuplet group: the underlying beam group plus a tuplet bracket with number.
///
/// Delegates to `draw_beam_group_event` for note/beam rendering, then overlays
/// the tuplet bracket positioned relative to the beam group's extreme notes.
#[allow(clippy::too_many_arguments)]
fn draw_tuplet_group_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    group_x: f64,
    total_width: f64,
    tg: &TupletGroupEvent,
) -> Result<(), FontError> {
    // Draw the underlying beam group (noteheads, stems, beams)
    draw_beam_group_event(svg, staff, font, config, group_x, total_width, &tg.beam_group)?;

    if tg.beam_group.notes.is_empty() {
        return Ok(());
    }

    // Determine stem direction (same logic as beam group)
    let positions: Vec<i8> = tg.beam_group.notes.iter().map(|n| n.staff_position).collect();
    let direction = tg
        .beam_group
        .stem_direction
        .unwrap_or_else(|| auto_stem_direction_chord(&positions));

    let placement = tuplet_placement_from_stem(direction);

    // Compute advance width of tuplet number glyph(s) for centering
    let number_glyphs = tuplet_number_glyphs(tg.tuplet_number);
    let number_width: f64 = number_glyphs
        .iter()
        .map(|g| font.glyph_advance(*g).unwrap_or(0) as f64)
        .sum();

    let bracket_thickness_ss = config.tuplet_bracket_thickness;

    let bracket_layout = layout_tuplet_bracket(
        group_x,
        group_x + total_width,
        &positions,
        placement,
        tg.tuplet_number,
        staff.staff_space,
        bracket_thickness_ss,
        number_width,
    );

    draw_tuplet_bracket(svg, &bracket_layout, font, 0.0, 0.0);

    Ok(())
}
