use music::notation::clef::Clef;

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::accidental::ResolvedAccidental;
use crate::layout::beam::{layout_beam_group, BeamGroupLayout, BeamedNote};
use crate::layout::chord::{
    chord_left_notehead_offset, layout_chord_noteheads, notehead_x_offset, ChordNote,
};
use crate::layout::dot::dot_staff_position;
use crate::layout::grace::layout_grace_note;
use crate::layout::mark_extent::element_annotations;
use crate::layout::measure::{
    BeamGroupEvent, ChordEvent, MeasureElement, MeasureLayout, NoteAnnotations, NoteEvent,
    NoteheadStyle, TupletGroupEvent,
};
use crate::layout::multi_measure_rest::{
    church_rest_supported, layout_church_rest, layout_multi_measure_rest, MultiMeasureRestStyle,
};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{
    auto_stem_direction, auto_stem_direction_chord, stem_length_staff_spaces, StemDirection,
};
use crate::layout::tuplet::{
    layout_tuplet_bracket, tuplet_number_glyphs, tuplet_placement_from_stem, TupletBracketLayout,
    TupletPlacement,
};
use crate::render::accidental_renderer::{chord_accidental_column_offsets, draw_accidental};
use crate::render::barline_renderer::draw_barline;
use crate::render::beam_renderer::draw_beam_group_with_advances;
use crate::render::church_rest_renderer::draw_church_rest;
use crate::render::dot_renderer::draw_dots;
use crate::render::flag_renderer::draw_flag;
use crate::render::grace_renderer::draw_grace_note;
use crate::render::key_sig_renderer::draw_key_signature;
use crate::render::multi_measure_rest_renderer::draw_multi_measure_rest;
use crate::render::note_renderer::{
    draw_ledger_lines, draw_styled_notehead, notehead_advance, NoteheadKind,
};
use crate::render::staff_renderer::draw_clef;
use crate::render::stem_renderer::{draw_stem, stem_endpoints, stem_x};
use crate::render::time_sig_renderer::draw_time_signature;
use crate::render::tremolo_renderer::draw_tremolo;
use crate::render::tuplet_renderer::draw_tuplet_bracket;
use crate::render::SvgWriter;

mod event_marks;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_rest_marks;

use event_marks::{draw_event_marks, draw_rest_event, draw_text_marks, EventAnchor};

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
    // Text marks of the most recent event wait for the barline that
    // follows it: (event x, event rod, marks).
    let mut pending_marks: Option<(f64, f64, &[crate::layout::text_script::TextScript])> = None;
    for positioned in &layout.elements {
        let elem_x = x_offset + positioned.x;
        if let Some(annotations) = element_annotations(&positioned.element) {
            pending_marks = Some((elem_x, positioned.rod, &annotations.text_marks));
        }
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
                draw_beam_group_event(svg, staff, font, config, elem_x, positioned.width, bg)?;
            }
            MeasureElement::TupletGroup(tg) => {
                draw_tuplet_group_event(svg, staff, font, config, elem_x, positioned.width, tg)?;
            }
            MeasureElement::Rest(rest) => {
                draw_rest_event(svg, staff, font, config, elem_x, rest, 0.0)?;
            }
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
                let width = draw_barline(svg, staff, font, elem_x, *style)?;
                if let Some((_, _, marks)) = pending_marks.take() {
                    draw_text_marks(svg, staff, font, config, marks, elem_x + width / 2.0)?;
                }
            }
        }
    }
    if let Some((event_x, rod, marks)) = pending_marks {
        draw_text_marks(svg, staff, font, config, marks, event_x + rod)?;
    }

    Ok(())
}


/// Draw additional voices for a measure at the same x-positions as the primary voice.
///
/// Each voice layout contains only rhythmic elements (notes/rests/chords/beams/tuplets)
/// and barlines. Barlines are skipped (already drawn by the primary voice). Rests are
/// displaced vertically to avoid collision with the primary voice: voice 1 rests move
/// down (below staff center), voice 2 rests move up. The displacement is 2 staff spaces.
///
/// Noteheads that collide with the primary voice (unison or second apart) are offset
/// horizontally by one notehead width to avoid overlap.
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

        for (elem_idx, positioned) in voice_layout.elements.iter().enumerate() {
            // Element-level offset (inner_note_index = None): applies to
            // standalone Note/Chord — shift the whole event by one notehead
            // width via `elem_x`. Per-note offsets (inner_note_index = Some(i))
            // live on BeamGroup/TupletGroup and bypass `elem_x` entirely;
            // they're handed to the beam/tuplet renderer as a per-note shift
            // slice so only the colliding noteheads move and the stems and
            // beam line stay anchored at the original beat positions.
            let element_collision_shift = collision_offsets
                .iter()
                .find(|o| o.element_index == elem_idx && o.inner_note_index.is_none())
                .map(|o| o.x_offset_noteheads * notehead_width)
                .unwrap_or(0.0);
            let elem_x = x_offset + positioned.x + element_collision_shift;

            match &positioned.element {
                // Skip non-rhythmic elements — the primary voice already drew them.
                // Multi-measure rests are a whole-measure property and only render
                // from the primary voice; an additional voice sitting on top of one
                // would just produce a duplicate H-bar.
                MeasureElement::Clef(_)
                | MeasureElement::KeySignature(_)
                | MeasureElement::TimeSignature(_)
                | MeasureElement::MultiMeasureRest { .. }
                | MeasureElement::Barline(_) => {}

                MeasureElement::Note(note) => {
                    draw_note_event(svg, staff, font, config, elem_x, note)?;
                }
                MeasureElement::Chord(chord) => {
                    draw_chord_event(svg, staff, font, config, elem_x, chord)?;
                }
                MeasureElement::BeamGroup(bg) => {
                    let per_note_shifts = per_note_shifts_for_group(
                        &collision_offsets,
                        elem_idx,
                        bg.notes.len(),
                        notehead_width,
                    );
                    draw_beam_group_event_with_offsets(
                        svg,
                        staff,
                        font,
                        config,
                        elem_x,
                        positioned.width,
                        bg,
                        &per_note_shifts,
                    )?;
                }
                MeasureElement::TupletGroup(tg) => {
                    let per_note_shifts = per_note_shifts_for_group(
                        &collision_offsets,
                        elem_idx,
                        tg.beam_group.notes.len(),
                        notehead_width,
                    );
                    draw_tuplet_group_event_with_offsets(
                        svg,
                        staff,
                        font,
                        config,
                        elem_x,
                        positioned.width,
                        tg,
                        &per_note_shifts,
                    )?;
                }
                MeasureElement::Rest(rest) => {
                    // Rests don't get collision offset — use original x
                    let rest_x = x_offset + positioned.x;
                    draw_rest_event(svg, staff, font, config, rest_x, rest, rest_displacement)?;
                }
            }
        }
    }
    Ok(())
}

/// Build a per-note x-shift slice for a beam/tuplet group from the flat
/// list of [`VoiceCollisionOffset`]s. Returns an empty `Vec` if no per-note
/// offset targets this element, allowing the renderer to skip per-note
/// dispatch entirely (and so preserve the byte-identical render when there
/// are no collisions inside the group).
fn per_note_shifts_for_group(
    collision_offsets: &[crate::layout::voice_collision::VoiceCollisionOffset],
    elem_idx: usize,
    note_count: usize,
    notehead_width: f64,
) -> Vec<f64> {
    let group_offsets: Vec<&crate::layout::voice_collision::VoiceCollisionOffset> =
        collision_offsets
            .iter()
            .filter(|o| o.element_index == elem_idx && o.inner_note_index.is_some())
            .collect();
    if group_offsets.is_empty() {
        return Vec::new();
    }
    let mut shifts = vec![0.0; note_count];
    for o in &group_offsets {
        // Safety: `inner_note_index` is guaranteed `Some` by the filter above.
        let i = o.inner_note_index.unwrap();
        if i < note_count {
            shifts[i] = o.x_offset_noteheads * notehead_width;
        }
    }
    shifts
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
fn draw_note_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    note: &NoteEvent,
) -> Result<(), FontError> {
    let kind = notehead_kind_from_log2(note.duration_log2);
    let style = notehead_style(&note.annotations, 0);
    let parenthesized = parenthesized_notehead(&note.annotations, 0);
    let position = note.staff_position;

    // Draw grace note before principal (if present)
    if let Some((grace_pos, grace_kind)) = note.annotations.grace_note {
        let stem_dir = note
            .stem_direction
            .unwrap_or_else(|| auto_stem_direction(position));
        let grace_layout = layout_grace_note(x, grace_pos, grace_kind, stem_dir, staff);
        draw_grace_note(svg, font, &grace_layout)?;

        // Connecting slur from grace to principal (canonical for acciaccatura)
        if note.annotations.grace_note_slur {
            if let Some(slur) = crate::layout::grace::layout_grace_note_slur(
                &grace_layout,
                x,
                position,
                stem_dir,
                staff,
                config,
            ) {
                crate::render::slur_renderer::draw_slur(svg, &slur);
            }
        }
    }

    // Draw accidental (pre-resolved) to the left of notehead
    if let Some(accidental) = note.accidental {
        draw_accidental(svg, staff, font, x, 0.0, position, accidental)?;
    }

    // Draw notehead
    let advance = draw_styled_notehead(svg, staff, font, x, position, kind, style, parenthesized)?;

    // Draw ledger lines
    draw_ledger_lines(svg, staff, config, x, advance, position);

    // Draw arpeggio wavy line to the left of the note if present
    if let Some(arp_dir) = note.annotations.arpeggio {
        if let Some(arp_layout) =
            crate::layout::arpeggio::layout_arpeggio(arp_dir, &[position], x, staff)
        {
            crate::render::arpeggio_renderer::draw_arpeggio(svg, font, &arp_layout)?;
        }
    }

    // Determine stem direction
    let needs_stem = note.duration_log2 >= 1; // whole notes have no stem
    let direction = if needs_stem {
        Some(
            note.stem_direction
                .unwrap_or_else(|| auto_stem_direction(position)),
        )
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

    // Draw tremolo slashes on the stem if present
    if let Some(tremolo_count) = note.annotations.tremolo {
        if let Some(dir) = direction {
            let sx = stem_x(x, advance, dir, config.stem_thickness_fu());
            let notehead_y = staff.y_of(position);
            let (y_top, y_bottom) = stem_endpoints(staff, position, dir);
            let tip_y = match dir {
                StemDirection::Up => y_top,
                StemDirection::Down => y_bottom,
            };
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
        draw_dots(svg, staff, font, x, advance, dot_pos, note.dots)?;
    }

    let notehead_y = staff.y_of(position);
    let half_head = config.staff_space / 2.0;
    let anchor = EventAnchor {
        left_x: x,
        width: advance,
        top_y: notehead_y - half_head,
        bottom_y: notehead_y + half_head,
        articulation_position: position,
        stem: direction.unwrap_or(StemDirection::Up),
        ornament_position: position,
    };
    draw_event_marks(svg, staff, font, config, &anchor, &note.annotations)?;

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
    let direction = chord
        .stem_direction
        .unwrap_or_else(|| auto_stem_direction_chord(&chord.staff_positions));

    // Draw grace note before chord (if present)
    if let Some((grace_pos, grace_kind)) = chord.annotations.grace_note {
        let grace_layout = layout_grace_note(x, grace_pos, grace_kind, direction, staff);
        draw_grace_note(svg, font, &grace_layout)?;

        // Connecting slur from grace to nearest chord note (canonical for
        // acciaccatura — attach to the chord member closest in pitch).
        if chord.annotations.grace_note_slur {
            let nearest = chord
                .staff_positions
                .iter()
                .copied()
                .min_by_key(|&p| (p as i32 - grace_pos as i32).abs())
                .unwrap_or(grace_pos);
            if let Some(slur) = crate::layout::grace::layout_grace_note_slur(
                &grace_layout,
                x,
                nearest,
                direction,
                staff,
                config,
            ) {
                crate::render::slur_renderer::draw_slur(svg, &slur);
            }
        }
    }

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
            .map(|width| widest.max(width))
    })?;

    // Accidentals stack in columns left of the chord's leftmost notehead.
    let accidental_anchor = x + chord_left_notehead_offset(&layouts, direction) * advance;
    let accidental_columns = chord_accidental_column_offsets(font, staff, &layouts)?;

    // Draw each notehead (with offset for seconds)
    for (note_layout, &column_offset) in layouts.iter().zip(&accidental_columns) {
        let x_off = notehead_x_offset(note_layout.offset, direction) * advance;
        let note_x = x + x_off;

        // Draw accidental
        if let Some(accidental) = note_layout.accidental {
            draw_accidental(
                svg,
                staff,
                font,
                accidental_anchor,
                column_offset,
                note_layout.staff_position,
                accidental,
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
            note_layout.parenthesized,
        )?;

        // Draw ledger lines for this note
        draw_ledger_lines(
            svg,
            staff,
            config,
            note_x,
            note_advance,
            note_layout.staff_position,
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

        let attach_note = layouts
            .iter()
            .find(|note| note.staff_position == attach_pos)
            .expect("the selected chord attachment position must exist");
        let attach_advance =
            notehead_advance(font, chord.duration_log2, attach_note.notehead_style)?;
        let attach_x = x + notehead_x_offset(attach_note.offset, direction) * advance;
        let thickness = config.stem_thickness_fu();
        let sx = stem_x(attach_x, attach_advance, direction, thickness);
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

        // Draw tremolo slashes on the chord stem if present
        if let Some(tremolo_count) = chord.annotations.tremolo {
            let tip_y = match direction {
                StemDirection::Up => y_top,
                StemDirection::Down => y_bottom,
            };
            // Use the far note (tip side) as the notehead reference
            let far_pos = match direction {
                StemDirection::Up => layouts.iter().map(|n| n.staff_position).max().unwrap(),
                StemDirection::Down => layouts.iter().map(|n| n.staff_position).min().unwrap(),
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
            draw_dots(svg, staff, font, dot_base_x, advance, dot_pos, chord.dots)?;
        }
    }

    // Articulations attach to the note on the opposite side from the stem:
    // stem-up → below → lowest note; stem-down → above → highest note.
    let top_pos = chord.staff_positions.iter().copied().max().unwrap_or(4);
    let bottom_pos = chord.staff_positions.iter().copied().min().unwrap_or(4);
    let half_head = config.staff_space / 2.0;
    let anchor = EventAnchor {
        left_x: x,
        width: advance,
        top_y: staff.y_of(top_pos) - half_head,
        bottom_y: staff.y_of(bottom_pos) + half_head,
        articulation_position: match direction {
            StemDirection::Up => bottom_pos,
            StemDirection::Down => top_pos,
        },
        stem: direction,
        ornament_position: top_pos,
    };
    draw_event_marks(svg, staff, font, config, &anchor, &chord.annotations)?;

    Ok(())
}

/// Draw a beam group event: noteheads + ledger lines + accidentals + dots,
/// then beams and stems via `draw_beam_group`.
fn grouped_member_positions(note: &NoteEvent) -> &[i8] {
    note.annotations.grouped_chord.as_ref().map_or_else(
        || std::slice::from_ref(&note.staff_position),
        |chord| chord.staff_positions.as_slice(),
    )
}

fn grouped_member_accidentals(note: &NoteEvent) -> &[Option<ResolvedAccidental>] {
    note.annotations.grouped_chord.as_ref().map_or_else(
        || std::slice::from_ref(&note.accidental),
        |chord| chord.accidentals.as_slice(),
    )
}

struct RenderedBeamGroup {
    layout: BeamGroupLayout,
    positions: Vec<i8>,
}

fn draw_beam_group_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    group_x: f64,
    total_width: f64,
    bg: &BeamGroupEvent,
) -> Result<(), FontError> {
    draw_beam_group_event_with_offsets(svg, staff, font, config, group_x, total_width, bg, &[])?;
    Ok(())
}

/// Beam-group renderer with per-member horizontal offsets for cross-voice
/// collision avoidance. Collision shifts move a member's noteheads and their
/// attached glyphs, while chord stems and the beam skeleton remain beat-anchored.
#[allow(clippy::too_many_arguments)]
fn draw_beam_group_event_with_offsets(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    group_x: f64,
    total_width: f64,
    bg: &BeamGroupEvent,
    per_note_x_shift: &[f64],
) -> Result<Option<RenderedBeamGroup>, FontError> {
    let n = bg.notes.len();
    if n == 0 {
        return Ok(None);
    }
    assert!(
        per_note_x_shift.is_empty() || per_note_x_shift.len() == n,
        "per_note_x_shift length must match beam group member count"
    );

    let durations: Vec<i8> = bg.notes.iter().map(|note| note.duration_log2).collect();
    let local_offsets = crate::layout::beam::beam_group_note_x_offsets(&durations, total_width);
    let positions: Vec<i8> = bg
        .notes
        .iter()
        .flat_map(grouped_member_positions)
        .copied()
        .collect();
    let direction = bg
        .stem_direction
        .unwrap_or_else(|| auto_stem_direction_chord(&positions));
    let shift_for = |index: usize| per_note_x_shift.get(index).copied().unwrap_or_default();

    let mut notehead_advances = Vec::with_capacity(n);
    let mut beamed_notes = Vec::with_capacity(n);
    for (index, note) in bg.notes.iter().enumerate() {
        let member_positions = grouped_member_positions(note);
        let member_accidentals = grouped_member_accidentals(note);
        debug_assert_eq!(member_positions.len(), member_accidentals.len());
        let base_x = group_x + local_offsets[index];
        let drawn_x = base_x + shift_for(index);
        let chord_notes: Vec<ChordNote> = member_positions
            .iter()
            .enumerate()
            .map(|(tone, &staff_position)| ChordNote {
                staff_position,
                accidental: member_accidentals[tone],
                notehead_style: notehead_style(&note.annotations, tone),
                parenthesized: parenthesized_notehead(&note.annotations, tone),
            })
            .collect();
        let layouts = layout_chord_noteheads(&chord_notes, direction);
        let widest_advance = layouts.iter().try_fold(0.0_f64, |widest, layout| {
            notehead_advance(font, note.duration_log2, layout.notehead_style)
                .map(|advance| widest.max(advance))
        })?;

        let accidental_anchor =
            drawn_x + chord_left_notehead_offset(&layouts, direction) * widest_advance;
        let accidental_columns = chord_accidental_column_offsets(font, staff, &layouts)?;

        for (layout, &accidental_column) in layouts.iter().zip(&accidental_columns) {
            let column_offset = notehead_x_offset(layout.offset, direction) * widest_advance;
            let note_x = drawn_x + column_offset;
            if let Some(accidental) = layout.accidental {
                draw_accidental(
                    svg,
                    staff,
                    font,
                    accidental_anchor,
                    accidental_column,
                    layout.staff_position,
                    accidental,
                )?;
            }
            let advance = draw_styled_notehead(
                svg,
                staff,
                font,
                note_x,
                layout.staff_position,
                notehead_kind_from_log2(note.duration_log2),
                layout.notehead_style,
                layout.parenthesized,
            )?;
            draw_ledger_lines(svg, staff, config, note_x, advance, layout.staff_position);
        }

        if note.dots > 0 {
            let dot_x = if layouts.iter().any(|layout| layout.offset) {
                drawn_x + widest_advance
            } else {
                drawn_x
            };
            for layout in &layouts {
                draw_dots(
                    svg,
                    staff,
                    font,
                    dot_x,
                    widest_advance,
                    dot_staff_position(layout.staff_position),
                    note.dots,
                )?;
            }
        }

        if let Some(symbol) = &note.annotations.chord_symbol {
            let layout = crate::layout::chord_symbol::layout_chord_symbol_composite(
                symbol,
                base_x + widest_advance / 2.0,
                staff,
                config.staff_space,
                font.units_per_em(),
                |glyph| font.glyph_advance(glyph).unwrap_or(0),
            );
            crate::render::chord_symbol_renderer::draw_chord_symbol_composite(svg, font, &layout)?;
        }

        let min_position = layouts
            .iter()
            .map(|layout| layout.staff_position)
            .min()
            .expect("a grouped member always contains at least one notehead");
        let max_position = layouts
            .iter()
            .map(|layout| layout.staff_position)
            .max()
            .expect("a grouped member always contains at least one notehead");
        let (attachment_position, beam_side_position) = match direction {
            StemDirection::Up => (min_position, max_position),
            StemDirection::Down => (max_position, min_position),
        };
        let attachment = layouts
            .iter()
            .find(|layout| layout.staff_position == attachment_position)
            .expect("the grouped chord attachment position must exist");
        let attachment_advance =
            notehead_advance(font, note.duration_log2, attachment.notehead_style)?;
        let attachment_x =
            base_x + notehead_x_offset(attachment.offset, direction) * widest_advance;
        if attachment_position != beam_side_position {
            let stem_x = stem_x(
                attachment_x,
                attachment_advance,
                direction,
                config.stem_thickness_fu(),
            );
            svg.add_line(
                stem_x,
                staff.y_of(attachment_position),
                stem_x,
                staff.y_of(beam_side_position),
                "black",
                config.stem_thickness_fu(),
            );
        }
        notehead_advances.push(attachment_advance);
        beamed_notes.push(BeamedNote {
            x: attachment_x,
            staff_position: beam_side_position,
            duration_log2: note.duration_log2,
        });
    }

    let beam_layout = layout_beam_group(&beamed_notes, direction, staff.staff_space);
    draw_beam_group_with_advances(
        svg,
        staff,
        config,
        &beamed_notes,
        &beam_layout,
        &notehead_advances,
    );
    Ok(Some(RenderedBeamGroup {
        layout: beam_layout,
        positions,
    }))
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
    draw_tuplet_group_event_with_offsets(svg, staff, font, config, group_x, total_width, tg, &[])
}

/// Tuplet-group renderer with per-note horizontal offsets for cross-voice
/// collision avoidance. See [`draw_beam_group_event_with_offsets`] for the
/// per-note shift semantics; the tuplet bracket itself does *not* shift —
/// it frames the original beam-group rhythmic positions, not the
/// collision-displaced noteheads.
#[allow(clippy::too_many_arguments)]
fn draw_tuplet_group_event_with_offsets(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    group_x: f64,
    total_width: f64,
    tg: &TupletGroupEvent,
    per_note_x_shift: &[f64],
) -> Result<(), FontError> {
    // Draw the underlying beam group and retain its exact chord-aware geometry
    // for bracket placement and beam clearance.
    let Some(rendered) = draw_beam_group_event_with_offsets(
        svg,
        staff,
        font,
        config,
        group_x,
        total_width,
        &tg.beam_group,
        per_note_x_shift,
    )?
    else {
        return Ok(());
    };

    let direction = rendered.layout.direction;
    let placement = tuplet_placement_from_stem(direction);

    // Compute advance width of tuplet number glyph(s) for centering.
    let number_glyphs = tuplet_number_glyphs(tg.tuplet_number);
    let number_width: f64 = number_glyphs
        .iter()
        .map(|g| font.glyph_advance(*g).unwrap_or(0) as f64)
        .sum();

    let bracket_thickness_ss = config.tuplet_bracket_thickness;

    let mut bracket_layout = layout_tuplet_bracket(
        group_x,
        group_x + total_width,
        &rendered.positions,
        placement,
        tg.tuplet_number,
        staff.staff_space,
        bracket_thickness_ss,
        number_width,
    );
    clear_tuplet_bracket_from_beam(&mut bracket_layout, &rendered.layout, staff.staff_space);

    draw_tuplet_bracket(svg, &bracket_layout, font, 0.0, 0.0);
    Ok(())
}

fn clear_tuplet_bracket_from_beam(
    bracket: &mut TupletBracketLayout,
    beam: &BeamGroupLayout,
    staff_space: f64,
) {
    let Some((&first_tip, rest)) = beam.stem_tip_ys.split_first() else {
        return;
    };
    let clearance = staff_space;
    let collision_free_y = match bracket.placement {
        TupletPlacement::Above => {
            rest.iter().fold(first_tip, |outer, &tip| outer.min(tip)) - clearance
        }
        TupletPlacement::Below => {
            rest.iter().fold(first_tip, |outer, &tip| outer.max(tip)) + clearance
        }
    };
    let adjusted_y = match bracket.placement {
        TupletPlacement::Above => bracket.bracket_y.min(collision_free_y),
        TupletPlacement::Below => bracket.bracket_y.max(collision_free_y),
    };
    bracket.bracket_y = adjusted_y;
    bracket.number_y = adjusted_y;
}
