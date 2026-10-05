use music::notation::clef::Clef;

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::articulation::{layout_articulation_stack, layout_chord_articulation_stack};
use crate::layout::chord::{
    chord_left_notehead_offset, layout_chord_noteheads, notehead_x_offset, ChordNote,
};
use crate::layout::dot::dot_staff_position;
use crate::layout::expression::layout_expression;
use crate::layout::grace::{grace_stem_direction, layout_grace_group, layout_grace_slur};
use crate::layout::group::scan_groups;
use crate::layout::lyric::layout_lyric;
use crate::layout::measure::{
    ChordEvent, MeasureElement, MeasureLayout, NoteAnnotations, NoteEvent, NoteheadStyle,
    PositionedElement, StemVisibility,
};
use crate::layout::staff::StaffPosition;
use crate::layout::multi_measure_rest::{
    church_rest_supported, layout_church_rest, layout_multi_measure_rest, MultiMeasureRestStyle,
};
use crate::layout::rehearsal::layout_rehearsal_mark;
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{
    auto_stem_direction, auto_stem_direction_chord, stem_length_staff_spaces, StemDirection,
};
use crate::layout::tempo::layout_tempo_mark;
use crate::render::accidental_renderer::{chord_accidental_column_offsets, draw_accidental};
use crate::render::articulation_renderer::draw_articulation;
use crate::render::barline_renderer::draw_barline;
use crate::render::church_rest_renderer::draw_church_rest;
use crate::render::dot_renderer::draw_dots;
use crate::render::dynamics_renderer::draw_dynamic;
use crate::render::expression_renderer::draw_expression;
use crate::render::flag_renderer::draw_flag;
use crate::render::group_renderer::draw_groups;
use crate::render::grace_renderer::draw_grace_group;
use crate::render::key_sig_renderer::draw_key_signature;
use crate::render::lyric_renderer::draw_lyric;
use crate::render::multi_measure_rest_renderer::draw_multi_measure_rest;
use crate::render::note_renderer::{
    draw_ledger_lines, draw_notehead_parentheses, draw_styled_notehead, notehead_advance,
    NoteheadKind,
};
use crate::render::rehearsal_renderer::draw_rehearsal_mark;
use crate::render::rest_renderer::{draw_rest, draw_rest_displaced};
use crate::render::staff_renderer::draw_clef;
use crate::render::stem_renderer::{draw_stem, stem_endpoints, stem_x};
use crate::render::tempo_renderer::draw_tempo_mark;
use crate::render::time_sig_renderer::draw_time_signature;
use crate::render::tremolo_renderer::draw_tremolo;
use crate::render::SvgWriter;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_groups;

/// Draw a complete laid-out measure onto an SVG writer: every element, then
/// the stems, beams, and tuplet brackets of the measure's beam and tuplet
/// spans (a span continuing across a barline is drawn as a broken piece).
///
/// `x_offset` shifts the entire measure horizontally (for multi-measure rendering).
/// `clef_for_key_sig` determines accidental placement for key signatures.
pub fn draw_measure(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    layout: &MeasureLayout,
    x_offset: f64,
    clef_for_key_sig: &Clef,
) -> Result<(), FontError> {
    draw_measure_elements(svg, staff, font, config, layout, x_offset, clef_for_key_sig)?;
    let items: Vec<_> = layout
        .elements
        .iter()
        .map(|positioned| (x_offset + positioned.x, &positioned.element))
        .collect();
    draw_groups(svg, staff, font, config, &items)
}

/// Per element: whether it is a member of a beam span. Beamed notes and
/// chords leave their stem and flag to the beam.
fn beamed_elements(elements: &[PositionedElement]) -> Vec<bool> {
    scan_groups(elements.iter().map(|positioned| &positioned.element))
        .beam_of
        .iter()
        .map(Option::is_some)
        .collect()
}

/// Draw a laid-out measure's elements without its beam and tuplet spans,
/// which the system renderer draws per voice across the whole system.
pub(crate) fn draw_measure_elements(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    layout: &MeasureLayout,
    x_offset: f64,
    clef_for_key_sig: &Clef,
) -> Result<(), FontError> {
    let beamed = beamed_elements(&layout.elements);
    for (index, positioned) in layout.elements.iter().enumerate() {
        let elem_x = x_offset + positioned.x;
        match &positioned.element {
            MeasureElement::Clef(clef_layout) => {
                draw_clef(svg, staff, elem_x, clef_layout, font)?;
            }
            MeasureElement::KeySignature(key) => {
                draw_key_signature(svg, staff, font, elem_x, key, clef_for_key_sig)?;
            }
            MeasureElement::TimeSignature(kind) => {
                draw_time_signature(svg, staff, font, elem_x, kind)?;
            }
            MeasureElement::Note(note) => {
                draw_note_event(svg, staff, font, config, elem_x, note, beamed[index])?;
            }
            MeasureElement::Chord(chord) => {
                draw_chord_event(svg, staff, font, config, elem_x, chord, beamed[index])?;
            }
            // Span marks take no space; spans are drawn by `draw_groups`.
            MeasureElement::GroupMark(_) => {}
            MeasureElement::Rest(rest) => {
                draw_rest(svg, staff, font, elem_x, rest.duration_log2)?;
            }
            // A spacer takes time and space but draws nothing.
            MeasureElement::Spacer(_) => {}
            MeasureElement::MultiMeasureRest { count, style } => {
                // The rest fills the rhythmic width allocated by layout — the
                // measure barlines sit just outside this span, so the cluster
                // (or H-bar) takes the same region a whole note would occupy.
                // Church-rest is only meaningful for small counts; for larger
                // counts the renderer transparently falls back to the H-bar
                // form rather than producing a sparse pair of rest glyphs in
                // a wide measure.
                match style {
                    MultiMeasureRestStyle::Church if church_rest_supported(*count) => {
                        let layout =
                            layout_church_rest(elem_x, elem_x + positioned.width, *count, staff);
                        draw_church_rest(svg, font, &layout)?;
                    }
                    _ => {
                        let layout = layout_multi_measure_rest(
                            elem_x,
                            elem_x + positioned.width,
                            *count,
                            staff,
                        );
                        draw_multi_measure_rest(svg, &layout);
                    }
                }
            }
            MeasureElement::Barline(style) => {
                draw_barline(svg, staff, font, elem_x, *style)?;
            }
        }
    }

    Ok(())
}

/// Draw additional voices for a measure at the same x-positions as the primary voice.
///
/// Each voice layout contains only rhythmic elements (notes/rests/chords and
/// span marks) and barlines. Barlines are skipped (already drawn by the
/// primary voice). Rests are displaced vertically to avoid collision with the
/// primary voice: voice 1 rests move down (below staff center), voice 2 rests
/// move up. The displacement is 2 staff spaces. Beam and tuplet spans are not
/// drawn here: the system renderer draws them per voice across the system.
///
/// Noteheads that collide with the primary voice (unison or second apart) are
/// offset horizontally by one notehead width to avoid overlap. A standalone
/// note or chord moves with its stem; a beamed member's noteheads move while
/// its stem stays on the beam at the unshifted beat position.
pub fn draw_additional_voices(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    primary_layout: &MeasureLayout,
    voice_layouts: &[MeasureLayout],
    x_offset: f64,
) -> Result<(), FontError> {
    // Notehead width for computing collision offsets (filled notehead is the common case)
    let notehead_width = font
        .glyph_outline(smufl::Glyph::NoteheadBlack)
        .map(|o| o.advance_width as f64)
        .unwrap_or(250.0);

    for (voice_idx, voice_layout) in voice_layouts.iter().enumerate() {
        // Voice index 0 = additional voice 1 (odd → stems down, rests displaced down)
        // Voice index 1 = additional voice 2 (even → stems up, rests displaced up)
        // Displacement: odd voices move rests down, even voices move rests up
        let rest_displacement = if voice_idx % 2 == 0 {
            // Odd-numbered voice (voice 1, 3, …): displace rests down (positive y)
            staff.staff_space * 2.0
        } else {
            // Even-numbered additional voice (voice 2, 4, …): displace rests up (negative y)
            -(staff.staff_space * 2.0)
        };

        // Compute collision offsets between primary and this additional voice
        let collision_offsets = crate::layout::voice_collision::compute_voice_collision_offsets(
            primary_layout,
            voice_layout,
        );
        let beamed = beamed_elements(&voice_layout.elements);

        for (elem_idx, positioned) in voice_layout.elements.iter().enumerate() {
            let collision_shift = collision_offsets
                .iter()
                .find(|o| o.element_index == elem_idx)
                .map(|o| o.x_offset_noteheads * notehead_width)
                .unwrap_or(0.0);
            let elem_x = x_offset + positioned.x + collision_shift;

            match &positioned.element {
                // Skip non-rhythmic elements — the primary voice already drew them.
                // Multi-measure rests are a whole-measure property and only render
                // from the primary voice; an additional voice sitting on top of one
                // would just produce a duplicate H-bar.
                MeasureElement::Clef(_)
                | MeasureElement::KeySignature(_)
                | MeasureElement::TimeSignature(_)
                | MeasureElement::MultiMeasureRest { .. }
                | MeasureElement::GroupMark(_)
                | MeasureElement::Spacer(_)
                | MeasureElement::Barline(_) => {}

                MeasureElement::Note(note) => {
                    draw_note_event(svg, staff, font, config, elem_x, note, beamed[elem_idx])?;
                }
                MeasureElement::Chord(chord) => {
                    draw_chord_event(svg, staff, font, config, elem_x, chord, beamed[elem_idx])?;
                }
                MeasureElement::Rest(rest) => {
                    // Rests don't get collision offset — use original x
                    let rest_x = x_offset + positioned.x;
                    draw_rest_displaced(
                        svg,
                        staff,
                        font,
                        rest_x,
                        rest.duration_log2,
                        rest_displacement,
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Notehead kind from log2 duration: -1=breve, 0=whole, 1=half, 2+=filled.
fn notehead_kind_from_log2(duration_log2: i8) -> NoteheadKind {
    match duration_log2 {
        ..=-1 => NoteheadKind::DoubleWhole,
        0 => NoteheadKind::Whole,
        1 => NoteheadKind::Half,
        _ => NoteheadKind::Filled,
    }
}

fn notehead_style(annotations: &NoteAnnotations, index: usize) -> NoteheadStyle {
    annotations
        .notehead_styles
        .get(index)
        .copied()
        .unwrap_or_default()
}

fn parenthesized_notehead(annotations: &NoteAnnotations, index: usize) -> bool {
    annotations
        .parenthesized_noteheads
        .get(index)
        .copied()
        .unwrap_or(false)
}

/// Number of flags from log2 duration: breve through quarter (≤ 2) have no
/// flags, 3=one flag, 4=two, etc.
fn flag_count_from_log2(duration_log2: i8) -> u8 {
    u8::try_from(duration_log2.saturating_sub(2)).unwrap_or(0)
}

/// Draw a complete note event: accidental + notehead + ledger lines + stem + flag + dots.
///
/// A `beamed` note (a beam span member) draws everything but its stem and
/// flag, which the beam renderer supplies.
/// Draw the grace group carried by `annotations` so that it ends before
/// `principal_left_x` (the principal's leftmost ink), with its slur to the
/// principal notehead at `slur_target` when the group asks for one.
///
/// Grace stems follow `forced_stem` (the principal's forced direction, if
/// any); the slur curves away from `principal_stem`.
#[allow(clippy::too_many_arguments)]
fn draw_principal_grace_group(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    annotations: &NoteAnnotations,
    forced_stem: Option<StemDirection>,
    principal_left_x: f64,
    slur_target: (f64, StaffPosition),
    principal_stem: StemDirection,
) -> Result<(), FontError> {
    let Some(group) = &annotations.grace_group else {
        return Ok(());
    };
    if group.notes.is_empty() {
        return Ok(());
    }
    let layout = layout_grace_group(
        group,
        principal_left_x,
        grace_stem_direction(forced_stem),
        staff,
        config,
    );
    draw_grace_group(svg, staff, font, config, &layout)?;
    if group.slur {
        let (center_x, position) = slur_target;
        if let Some(slur) =
            layout_grace_slur(&layout, center_x, position, principal_stem, staff, config)
        {
            crate::render::slur_renderer::draw_slur(svg, &slur);
        }
    }
    Ok(())
}

/// Draw a complete note event: grace group + accidental + notehead (with
/// parentheses) + ledger lines + stem + flag + dots, at the note's size.
fn draw_note_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    note: &NoteEvent,
    beamed: bool,
) -> Result<(), FontError> {
    let kind = notehead_kind_from_log2(note.duration_log2);
    let style = notehead_style(&note.annotations, 0);
    let parenthesized = parenthesized_notehead(&note.annotations, 0);
    let scale = note.annotations.size.scale();
    let position = note.staff_position;

    // Draw accidental (pre-resolved) to the left of notehead
    let mut left_x = match note.accidental {
        Some(accidental) => {
            draw_accidental(svg, staff, font, x, 0.0, position, accidental, scale)?
        }
        None => x,
    };

    // Draw notehead, enclosing it and its accidental when parenthesized
    let advance = draw_styled_notehead(svg, staff, font, x, position, kind, style, scale)?;
    if parenthesized {
        left_x =
            draw_notehead_parentheses(svg, staff, font, left_x, x + advance, position, scale)?;
    }

    // Draw ledger lines
    draw_ledger_lines(svg, staff, config, x, advance, position, scale);

    // Draw arpeggio wavy line to the left of the note if present
    if let Some(arp_dir) = note.annotations.arpeggio {
        if let Some(arp_layout) =
            crate::layout::arpeggio::layout_arpeggio(arp_dir, &[position], x, staff)
        {
            crate::render::arpeggio_renderer::draw_arpeggio(svg, font, &arp_layout)?;
        }
    }

    // Determine stem direction (it also places articulations, slurs, and
    // graces when the stem itself is hidden)
    let needs_stem = note.duration_log2 >= 1; // whole notes have no stem
    let resolved_direction = note
        .stem_direction
        .unwrap_or_else(|| auto_stem_direction(position));
    let direction = needs_stem.then_some(resolved_direction);

    draw_principal_grace_group(
        svg, staff, font, config, &note.annotations, note.stem_direction,
        left_x, (x + advance / 2.0, position), resolved_direction,
    )?;

    // A beamed member's stem is drawn with its beam. Stemless members have
    // no flag either, but retain their noteheads and annotations.
    if let Some(dir) = direction.filter(|_| !beamed && note.annotations.stem == StemVisibility::Visible) {
        draw_stem(svg, staff, config, x, advance, position, dir, scale);
        let sx = stem_x(x, advance, dir, config.stem_thickness_fu() * scale);
        let (top, bottom) = stem_endpoints(staff, position, dir, scale);
        let tip_y = match dir { StemDirection::Up => top, StemDirection::Down => bottom };
        // Draw flag
        let flags = flag_count_from_log2(note.duration_log2);
        if flags > 0 {
            draw_flag(svg, font, sx, tip_y, flags, dir, scale)?;
        }

        // Draw tremolo slashes on the stem if present
        if let Some(tremolo_count) = note.annotations.tremolo {
            let notehead_y = staff.y_of(position);
            let trem_layout = crate::layout::tremolo::layout_tremolo(
                tremolo_count,
                sx,
                notehead_y,
                tip_y,
                staff,
                dir,
            );
            draw_tremolo(svg, font, &trem_layout)?;
        }
    }

    // Draw augmentation dots
    if note.dots > 0 {
        let dot_pos = dot_staff_position(position);
        draw_dots(
            svg,
            staff,
            font,
            x,
            advance,
            dot_pos,
            note.dots,
            scale,
            note.annotations.parenthesized_dots,
        )?;
    }

    // Draw articulation(s) near the notehead if present
    if !note.annotations.articulations.is_empty() {
        let stem_dir = direction.unwrap_or(StemDirection::Up);
        let note_center_x = x + advance / 2.0;
        let artic_layouts = layout_articulation_stack(
            &note.annotations.articulations,
            note_center_x,
            position,
            stem_dir,
            staff,
        );
        for artic_layout in &artic_layouts {
            draw_articulation(svg, font, artic_layout)?;
        }
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
        let layout = crate::layout::chord_symbol::layout_chord_symbol_composite(
            symbol,
            note_center_x,
            staff,
            config.staff_space,
            font.units_per_em(),
            |g| font.glyph_advance(g).unwrap_or(0),
        );
        crate::render::chord_symbol_renderer::draw_chord_symbol_composite(svg, font, &layout)?;
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

    // Draw pedal marking (Ped. or *) below the staff if present
    if let Some(pedal_mark) = note.annotations.pedal {
        let note_center_x = x + advance / 2.0;
        crate::render::pedal_renderer::draw_pedal(svg, staff, font, pedal_mark, note_center_x)?;
    }

    // Draw breath mark above the staff, to the right of the note
    if let Some(breath) = note.annotations.breath_mark {
        let note_right_x = x + advance;
        let breath_layout = crate::layout::breath::layout_breath_mark(
            breath,
            note_right_x,
            staff,
            note.annotations.breath_mark_parenthesized,
        );
        crate::render::breath_renderer::draw_breath_mark(svg, font, &breath_layout)?;
    }

    Ok(())
}

/// Draw a chord event: multiple noteheads + shared stem + ledger lines + accidentals + flag + dots.
///
/// Uses `layout_chord_noteheads` to compute notehead offsets for seconds, then draws
/// each notehead at the correct x-offset, a single shared stem spanning the full chord,
/// and optional flags/dots. A `beamed` chord (a beam span member) leaves its
/// stem and flag to the beam renderer.
/// and optional flags/dots, all at the chord's size.
fn draw_chord_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    chord: &ChordEvent,
    beamed: bool,
) -> Result<(), FontError> {
    if chord.staff_positions.is_empty() {
        return Ok(());
    }

    let kind = notehead_kind_from_log2(chord.duration_log2);
    let scale = chord.annotations.size.scale();
    let direction = chord
        .stem_direction
        .unwrap_or_else(|| auto_stem_direction_chord(&chord.staff_positions));

    let chord_notes: Vec<ChordNote> = chord
        .staff_positions
        .iter()
        .enumerate()
        .map(|(index, &pos)| ChordNote {
            staff_position: pos,
            accidental: chord.accidentals.get(index).copied().flatten(),
            notehead_style: notehead_style(&chord.annotations, index),
            parenthesized: parenthesized_notehead(&chord.annotations, index),
        })
        .collect();

    let layouts = layout_chord_noteheads(&chord_notes, direction);

    // Use the widest selected head for collision columns, stem attachment, and dots.
    let advance = layouts.iter().try_fold(0.0_f64, |widest, note| {
        notehead_advance(font, chord.duration_log2, note.notehead_style)
            .map(|width| widest.max(width * scale))
    })?;

    // Accidentals stack in columns left of the chord's leftmost notehead.
    let accidental_anchor = x + chord_left_notehead_offset(&layouts, direction) * advance;
    let accidental_columns = chord_accidental_column_offsets(font, staff, &layouts, scale)?;

    // Draw each notehead (with offset for seconds)
    let mut left_x = accidental_anchor;
    for (note_layout, &column_offset) in layouts.iter().zip(&accidental_columns) {
        let x_off = notehead_x_offset(note_layout.offset, direction) * advance;
        let note_x = x + x_off;

        // Draw accidental
        let mut note_left = note_x;
        if let Some(accidental) = note_layout.accidental {
            note_left = draw_accidental(
                svg,
                staff,
                font,
                accidental_anchor,
                column_offset,
                note_layout.staff_position,
                accidental,
                scale,
            )?;
        }

        let note_advance = draw_styled_notehead(
            svg,
            staff,
            font,
            note_x,
            note_layout.staff_position,
            kind,
            note_layout.notehead_style,
            scale,
        )?;
        if note_layout.parenthesized {
            note_left = draw_notehead_parentheses(
                svg,
                staff,
                font,
                note_left,
                note_x + note_advance,
                note_layout.staff_position,
                scale,
            )?;
        }
        left_x = left_x.min(note_left);

        // Draw ledger lines for this note
        draw_ledger_lines(
            svg,
            staff,
            config,
            note_x,
            note_advance,
            note_layout.staff_position,
            scale,
        );
    }

    // Draw arpeggio wavy line to the left of the chord if present
    if let Some(arp_dir) = chord.annotations.arpeggio {
        if let Some(arp_layout) =
            crate::layout::arpeggio::layout_arpeggio(arp_dir, &chord.staff_positions, x, staff)
        {
            crate::render::arpeggio_renderer::draw_arpeggio(svg, font, &arp_layout)?;
        }
    }

    let min_pos = layouts.iter().map(|n| n.staff_position).min().unwrap();
    let max_pos = layouts.iter().map(|n| n.staff_position).max().unwrap();

    // Draw the grace group before the chord's leftmost ink, slurred to the
    // chord member closest in pitch to the first grace note.
    if let Some(first_grace) = chord
        .annotations
        .grace_group
        .as_ref()
        .and_then(|group| group.notes.first())
    {
        let nearest = chord
            .staff_positions
            .iter()
            .copied()
            .min_by_key(|&p| (i32::from(p) - i32::from(first_grace.staff_position)).abs())
            .unwrap_or(first_grace.staff_position);
        draw_principal_grace_group(
            svg,
            staff,
            font,
            config,
            &chord.annotations,
            chord.stem_direction,
            left_x,
            (x + advance / 2.0, nearest),
            direction,
        )?;
    }

    // Draw shared stem spanning from the closest to farthest note
    let needs_stem =
        chord.duration_log2 >= 1 && chord.annotations.stem == StemVisibility::Visible;
    if needs_stem {
        // Stem attaches at the note closest to the tip direction:
        // stem up → bottom note is the attachment, tip extends above top note
        // stem down → top note is the attachment, tip extends below bottom note
        let (attach_pos, far_pos) = match direction {
            StemDirection::Up => (min_pos, max_pos),
            StemDirection::Down => (max_pos, min_pos),
        };

        // Compute stem tip: start from the far note and extend by standard stem length
        let stem_len_ss = stem_length_staff_spaces(far_pos, direction, scale);
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

        let attach_note = layouts
            .iter()
            .find(|note| note.staff_position == attach_pos)
            .expect("the selected chord attachment position must exist");
        let attach_advance =
            notehead_advance(font, chord.duration_log2, attach_note.notehead_style)? * scale;
        let attach_x = x + notehead_x_offset(attach_note.offset, direction) * advance;
        let thickness = config.stem_thickness_fu() * scale;
        let sx = stem_x(attach_x, attach_advance, direction, thickness);
        if !beamed {
            svg.add_line(sx, y_top, sx, y_bottom, "black", thickness);

            // Draw flag
            let flags = flag_count_from_log2(chord.duration_log2);
            if flags > 0 {
                let tip_y = match direction {
                    StemDirection::Up => y_top,
                    StemDirection::Down => y_bottom,
                };
                draw_flag(svg, font, sx, tip_y, flags, direction, scale)?;
            }
        }

        // Draw tremolo slashes on the chord stem if present
        if let Some(tremolo_count) = chord.annotations.tremolo {
            let tip_y = match direction {
                StemDirection::Up => y_top,
                StemDirection::Down => y_bottom,
            };
            // Use the far note (tip side) as the notehead reference
            let far_pos = match direction {
                StemDirection::Up => max_pos,
                StemDirection::Down => min_pos,
            };
            let notehead_y = staff.y_of(far_pos);
            let trem_layout = crate::layout::tremolo::layout_tremolo(
                tremolo_count,
                sx,
                notehead_y,
                tip_y,
                staff,
                direction,
            );
            draw_tremolo(svg, font, &trem_layout)?;
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
            draw_dots(
                svg,
                staff,
                font,
                dot_base_x,
                advance,
                dot_pos,
                chord.dots,
                scale,
                chord.annotations.parenthesized_dots,
            )?;
        }
    }

    // Draw articulation(s) near the chord if present: marks above stack from
    // the highest note, marks below from the lowest.
    if !chord.annotations.articulations.is_empty() {
        let chord_center_x = x + advance / 2.0;
        let artic_layouts = layout_chord_articulation_stack(
            &chord.annotations.articulations,
            chord_center_x,
            (min_pos, max_pos),
            direction,
            staff,
        );
        for artic_layout in &artic_layouts {
            draw_articulation(svg, font, artic_layout)?;
        }
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
        let layout = crate::layout::chord_symbol::layout_chord_symbol_composite(
            symbol,
            chord_center_x,
            staff,
            config.staff_space,
            font.units_per_em(),
            |g| font.glyph_advance(g).unwrap_or(0),
        );
        crate::render::chord_symbol_renderer::draw_chord_symbol_composite(svg, font, &layout)?;
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

    // Draw pedal marking (Ped. or *) below the staff if present
    if let Some(pedal_mark) = chord.annotations.pedal {
        let chord_center_x = x + advance / 2.0;
        crate::render::pedal_renderer::draw_pedal(svg, staff, font, pedal_mark, chord_center_x)?;
    }

    // Draw breath mark above the staff, to the right of the chord
    if let Some(breath) = chord.annotations.breath_mark {
        let chord_right_x = x + advance;
        let breath_layout = crate::layout::breath::layout_breath_mark(
            breath,
            chord_right_x,
            staff,
            chord.annotations.breath_mark_parenthesized,
        );
        crate::render::breath_renderer::draw_breath_mark(svg, font, &breath_layout)?;
    }

    Ok(())
}

