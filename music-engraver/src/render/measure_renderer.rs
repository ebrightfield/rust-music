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
use crate::render::flag_renderer::draw_flag;
use crate::render::key_sig_renderer::draw_key_signature;
use crate::render::note_renderer::{draw_ledger_lines, draw_notehead, NoteheadKind};
use crate::render::rest_renderer::draw_rest;
use crate::render::staff_renderer::draw_clef;
use crate::render::stem_renderer::{draw_stem, stem_endpoints, stem_x};
use crate::render::time_sig_renderer::draw_time_signature;
use crate::render::SvgWriter;

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

    // Draw dynamic marking below the staff if present
    if let Some(dyn_mark) = note.dynamic {
        let note_center_x = x + advance / 2.0;
        draw_dynamic(svg, staff, font, dyn_mark, note_center_x)?;
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

    // Draw dynamic marking below the staff if present
    if let Some(dyn_mark) = chord.dynamic {
        let chord_center_x = x + advance / 2.0;
        draw_dynamic(svg, staff, font, dyn_mark, chord_center_x)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::barline::BarlineStyle;
    use crate::layout::clef::ClefLayout;
    use crate::layout::key_signature::KeySignature;
    use crate::layout::measure::{layout_measure, BeamGroupEvent, MeasureLayoutConfig, RestEvent};
    use crate::layout::time_signature::TimeSignatureKind;
    use crate::render::staff_renderer::draw_staff_lines;
    use smufl::Glyph;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 200.0, -500.0, -500.0, 8000.0, 2000.0)
    }

    #[test]
    fn empty_measure_produces_no_elements() {
        let (font, config, staff) = setup();
        let layout = MeasureLayout {
            elements: vec![],
            total_width: 0.0,
        };
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();
        // Only the SVG wrapper, no path or line elements
        assert_eq!(output.matches("<path ").count(), 0);
        assert_eq!(output.matches("<line ").count(), 0);
    }

    #[test]
    fn measure_with_single_quarter_note() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // Should have 1 notehead path + 1 stem line
        assert_eq!(output.matches("<path ").count(), 1, "one notehead");
        assert_eq!(output.matches("<line ").count(), 1, "one stem line");
    }

    #[test]
    fn measure_with_whole_note_has_no_stem() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 0, // whole note
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "one notehead");
        assert_eq!(output.matches("<line ").count(), 0, "no stem for whole note");
    }

    #[test]
    fn measure_with_eighth_note_has_flag() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 3, // eighth note
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 notehead + 1 flag = 2 paths, 1 stem line
        assert_eq!(output.matches("<path ").count(), 2, "notehead + flag");
        assert_eq!(output.matches("<line ").count(), 1, "one stem");
    }

    #[test]
    fn measure_with_dotted_quarter() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 5, // in a space (dot position unchanged)
            duration_log2: 2,
            dots: 1,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 notehead + 1 dot = 2 paths, 1 stem
        assert_eq!(output.matches("<path ").count(), 2, "notehead + dot");
        assert_eq!(output.matches("<line ").count(), 1, "one stem");
    }

    #[test]
    fn measure_with_accidental_note() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: Some(Glyph::AccidentalSharp),
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 accidental + 1 notehead = 2 paths, 1 stem
        assert_eq!(output.matches("<path ").count(), 2, "accidental + notehead");
        assert_eq!(output.matches("<line ").count(), 1, "one stem");
    }

    #[test]
    fn measure_with_rest() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Rest(RestEvent {
            duration_log2: 2,
            dots: 0,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "one rest glyph");
        assert_eq!(output.matches("<line ").count(), 0, "no lines for rest");
    }

    #[test]
    fn measure_with_barline() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 notehead path, 1 stem + 1 barline = 2 lines
        assert_eq!(output.matches("<path ").count(), 1, "one notehead");
        assert_eq!(output.matches("<line ").count(), 2, "stem + barline");
    }

    #[test]
    fn measure_with_clef_and_key_signature() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::KeySignature(KeySignature::Sharps(2)),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 clef + 2 key sig accidentals + 1 notehead = 4 paths
        assert_eq!(output.matches("<path ").count(), 4, "clef + 2 sharps + notehead");
        // 1 stem line
        assert!(output.matches("<line ").count() >= 1, "at least 1 stem");
    }

    #[test]
    fn measure_with_time_signature() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![
            MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            }),
        ];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 digit paths (4, 4) + 1 notehead = 3 paths
        assert_eq!(output.matches("<path ").count(), 3, "2 digits + notehead");
    }

    #[test]
    fn x_offset_shifts_elements() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);

        let mut svg_0 = make_svg();
        draw_measure(&mut svg_0, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();

        let mut svg_offset = make_svg();
        draw_measure(
            &mut svg_offset,
            &staff,
            &font,
            &config,
            &layout,
            1000.0,
            &Clef::Treble,
        )
        .unwrap();

        // The SVGs should differ because of the offset
        assert_ne!(
            svg_0.to_svg(),
            svg_offset.to_svg(),
            "offset should produce different SVG"
        );
        // The offset version should contain translate with larger x
        assert!(svg_offset.to_svg().contains("translate(1000"));
    }

    #[test]
    fn notehead_kind_from_log2_mapping() {
        assert_eq!(notehead_kind_from_log2(0), NoteheadKind::Whole);
        assert_eq!(notehead_kind_from_log2(1), NoteheadKind::Half);
        assert_eq!(notehead_kind_from_log2(2), NoteheadKind::Filled);
        assert_eq!(notehead_kind_from_log2(3), NoteheadKind::Filled);
        assert_eq!(notehead_kind_from_log2(7), NoteheadKind::Filled);
    }

    #[test]
    fn flag_count_from_log2_mapping() {
        assert_eq!(flag_count_from_log2(0), 0); // whole
        assert_eq!(flag_count_from_log2(1), 0); // half
        assert_eq!(flag_count_from_log2(2), 0); // quarter
        assert_eq!(flag_count_from_log2(3), 1); // eighth
        assert_eq!(flag_count_from_log2(4), 2); // sixteenth
        assert_eq!(flag_count_from_log2(5), 3); // 32nd
        assert_eq!(flag_count_from_log2(7), 5); // 128th
    }

    #[test]
    fn full_measure_with_all_element_types() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![
            MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
            MeasureElement::KeySignature(KeySignature::Flats(1)),
            MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
                numerator: 3,
                denominator: 4,
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 6,
                duration_log2: 3,
                dots: 0,
                accidental: Some(Glyph::AccidentalNatural),
                stem_direction: Some(StemDirection::Down),
            tie_forward: false,
            dynamic: None,
            }),
            MeasureElement::Rest(RestEvent {
                duration_log2: 2,
                dots: 0,
            }),
            MeasureElement::Barline(BarlineStyle::Single),
        ];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_staff_lines(&mut svg, &staff, &config);
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // Count expected paths:
        // clef(1) + flat(1) + time(2 digits) + notehead(1) + notehead(1) + accidental(1) + flag(1) + rest(1) = 9
        assert_eq!(output.matches("<path ").count(), 9, "expected 9 paths total");

        // Lines: 5 staff lines + 2 stems + 1 barline = 8
        assert_eq!(output.matches("<line ").count(), 8, "expected 8 lines total");
    }

    #[test]
    fn note_with_ledger_lines_below_staff() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: -2, // middle C in treble
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 notehead, 1 stem + 1 ledger line = 2 lines
        assert_eq!(output.matches("<path ").count(), 1);
        assert_eq!(output.matches("<line ").count(), 2, "stem + 1 ledger line");
    }

    // --- chord rendering ---

    #[test]
    fn chord_two_notes_third_apart() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4], // E4 and B4 in treble — a fifth
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 noteheads, 1 stem
        assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
        assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
    }

    #[test]
    fn chord_with_second_has_two_noteheads() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        // Adjacent positions 4 and 5 — a second, one notehead should be offset
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![4, 5],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 noteheads (at different x-offsets due to second), 1 stem
        assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
        assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
        // Verify both noteheads have different translate positions
        let translates: Vec<&str> = output.matches("translate(").collect();
        assert_eq!(translates.len(), 2, "two translate transforms for two noteheads");
    }

    #[test]
    fn chord_with_accidentals() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![Some(Glyph::AccidentalSharp), None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 accidental + 2 noteheads = 3 paths, 1 stem
        assert_eq!(output.matches("<path ").count(), 3, "accidental + 2 noteheads");
        assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
    }

    #[test]
    fn chord_whole_note_no_stem() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4, 8],
            duration_log2: 0, // whole note chord
            dots: 0,
            accidentals: vec![None, None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 3 noteheads, no stem
        assert_eq!(output.matches("<path ").count(), 3, "three noteheads");
        assert_eq!(output.matches("<line ").count(), 0, "no stem for whole note chord");
    }

    #[test]
    fn chord_eighth_note_has_flag() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4],
            duration_log2: 3, // eighth note
            dots: 0,
            accidentals: vec![None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 noteheads + 1 flag = 3 paths, 1 stem
        assert_eq!(output.matches("<path ").count(), 3, "2 noteheads + flag");
        assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
    }

    #[test]
    fn chord_with_ledger_lines() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        // Chord spanning from below staff to on staff
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![-2, 4], // C4 (ledger line) and B4
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 noteheads, 1 stem + 1 ledger line = 2 lines
        assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
        assert_eq!(output.matches("<line ").count(), 2, "stem + ledger line");
    }

    #[test]
    fn chord_empty_produces_nothing() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 0, "empty chord = no paths");
        assert_eq!(output.matches("<line ").count(), 0, "empty chord = no lines");
    }

    #[test]
    fn chord_dotted_quarter() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![3, 5], // in spaces, so dots don't need line avoidance
            duration_log2: 2,
            dots: 1,
            accidentals: vec![None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 noteheads + 2 dots = 4 paths, 1 stem
        assert_eq!(output.matches("<path ").count(), 4, "2 noteheads + 2 dots");
        assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
    }

    #[test]
    fn chord_differs_from_single_note() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

        let single = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        })];
        let chord = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];

        let layout_s = layout_measure(&single, &cfg);
        let layout_c = layout_measure(&chord, &cfg);

        let mut svg_s = make_svg();
        draw_measure(&mut svg_s, &staff, &font, &config, &layout_s, 0.0, &Clef::Treble).unwrap();
        let mut svg_c = make_svg();
        draw_measure(&mut svg_c, &staff, &font, &config, &layout_c, 0.0, &Clef::Treble).unwrap();

        // Chord should have more paths (2 noteheads vs 1)
        assert!(
            svg_c.to_svg().matches("<path ").count() > svg_s.to_svg().matches("<path ").count(),
            "chord should have more noteheads than single note"
        );
    }

    #[test]
    fn stem_direction_override_is_respected() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

        // Position 0 (bottom line): auto would be stem up
        let elements_up = vec![MeasureElement::Note(NoteEvent {
            staff_position: 0,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: Some(StemDirection::Up),
        tie_forward: false,
        dynamic: None,
        })];
        let elements_down = vec![MeasureElement::Note(NoteEvent {
            staff_position: 0,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: Some(StemDirection::Down),
        tie_forward: false,
        dynamic: None,
        })];

        let layout_up = layout_measure(&elements_up, &cfg);
        let layout_down = layout_measure(&elements_down, &cfg);

        let mut svg_up = make_svg();
        draw_measure(&mut svg_up, &staff, &font, &config, &layout_up, 0.0, &Clef::Treble).unwrap();

        let mut svg_down = make_svg();
        draw_measure(
            &mut svg_down,
            &staff,
            &font,
            &config,
            &layout_down,
            0.0,
            &Clef::Treble,
        )
        .unwrap();

        // Different stem directions should produce different SVG
        assert_ne!(
            svg_up.to_svg(),
            svg_down.to_svg(),
            "up vs down stem should differ"
        );
    }

    // --- beam group rendering ---

    #[test]
    fn beam_group_two_eighths() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: vec![
                NoteEvent {
                    staff_position: 0,
                    duration_log2: 3,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                tie_forward: false,
                dynamic: None,
                },
                NoteEvent {
                    staff_position: 2,
                    duration_log2: 3,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                tie_forward: false,
                dynamic: None,
                },
            ],
            stem_direction: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 noteheads as paths
        assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
        // 2 stems as lines
        assert_eq!(output.matches("<line ").count(), 2, "two stems");
        // 1 primary beam as polygon
        assert_eq!(output.matches("<polygon ").count(), 1, "one beam polygon");
    }

    #[test]
    fn beam_group_four_sixteenths() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: vec![
                NoteEvent { staff_position: 0, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                NoteEvent { staff_position: 2, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                NoteEvent { staff_position: 4, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                NoteEvent { staff_position: 6, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            ],
            stem_direction: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 4, "four noteheads");
        assert_eq!(output.matches("<line ").count(), 4, "four stems");
        // Primary + secondary beam = 2 polygons
        assert_eq!(output.matches("<polygon ").count(), 2, "two beam polygons (primary + secondary)");
    }

    #[test]
    fn beam_group_with_accidental() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: vec![
                NoteEvent {
                    staff_position: 2,
                    duration_log2: 3,
                    dots: 0,
                    accidental: Some(Glyph::AccidentalSharp),
                    stem_direction: None,
                tie_forward: false,
                dynamic: None,
                },
                NoteEvent {
                    staff_position: 4,
                    duration_log2: 3,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                tie_forward: false,
                dynamic: None,
                },
            ],
            stem_direction: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 accidental + 2 noteheads = 3 paths
        assert_eq!(output.matches("<path ").count(), 3, "accidental + 2 noteheads");
        assert_eq!(output.matches("<line ").count(), 2, "two stems");
        assert_eq!(output.matches("<polygon ").count(), 1, "one beam");
    }

    #[test]
    fn beam_group_with_ledger_lines() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        // Notes below the staff requiring ledger lines
        let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: vec![
                NoteEvent { staff_position: -2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                NoteEvent { staff_position: -4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            ],
            stem_direction: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 2, "two noteheads");
        // 2 stems + ledger lines (1 for pos -2, 2 for pos -4)
        let line_count = output.matches("<line ").count();
        assert!(line_count >= 5, "stems + ledger lines: got {line_count}");
    }

    #[test]
    fn beam_group_empty() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: vec![],
            stem_direction: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 0);
        assert_eq!(output.matches("<line ").count(), 0);
        assert_eq!(output.matches("<polygon ").count(), 0);
    }

    #[test]
    fn beam_group_differs_from_flagged_notes() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

        // Two separate eighth notes (flagged)
        let flagged = vec![
            MeasureElement::Note(NoteEvent {
                staff_position: 0,
                duration_log2: 3,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            }),
            MeasureElement::Note(NoteEvent {
                staff_position: 2,
                duration_log2: 3,
                dots: 0,
                accidental: None,
                stem_direction: None,
            tie_forward: false,
            dynamic: None,
            }),
        ];
        // Same notes but beamed
        let beamed = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: vec![
                NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            ],
            stem_direction: None,
        })];

        let layout_f = layout_measure(&flagged, &cfg);
        let layout_b = layout_measure(&beamed, &cfg);

        let mut svg_f = make_svg();
        draw_measure(&mut svg_f, &staff, &font, &config, &layout_f, 0.0, &Clef::Treble).unwrap();
        let mut svg_b = make_svg();
        draw_measure(&mut svg_b, &staff, &font, &config, &layout_b, 0.0, &Clef::Treble).unwrap();

        let out_f = svg_f.to_svg();
        let out_b = svg_b.to_svg();

        // Flagged: 2 noteheads + 2 flags = 4 paths, 0 polygons
        assert_eq!(out_f.matches("<path ").count(), 4, "flagged: 2 noteheads + 2 flags");
        assert_eq!(out_f.matches("<polygon ").count(), 0, "flagged: no polygons");

        // Beamed: 2 noteheads = 2 paths, 1 polygon
        assert_eq!(out_b.matches("<path ").count(), 2, "beamed: 2 noteheads only");
        assert_eq!(out_b.matches("<polygon ").count(), 1, "beamed: 1 beam polygon");
    }

    #[test]
    fn beam_group_mixed_durations() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        // Eighth + two sixteenths
        let elements = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: vec![
                NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                NoteEvent { staff_position: 4, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                NoteEvent { staff_position: 6, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            ],
            stem_direction: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 3, "three noteheads");
        assert_eq!(output.matches("<line ").count(), 3, "three stems");
        // Primary beam spanning all + secondary beam for 16th notes
        assert!(
            output.matches("<polygon ").count() >= 2,
            "at least 2 beam polygons"
        );
    }

    // --- dynamics rendering in measure ---

    use crate::layout::dynamics::Dynamic;

    #[test]
    fn note_with_dynamic_adds_extra_path() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: false,
            dynamic: Some(Dynamic::Forte),
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 notehead + 1 dynamic glyph = 2 paths, 1 stem
        assert_eq!(output.matches("<path ").count(), 2, "notehead + dynamic");
        assert_eq!(output.matches("<line ").count(), 1, "one stem");
    }

    #[test]
    fn note_without_dynamic_no_extra_path() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: false,
            dynamic: None,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 1 notehead, no dynamic
        assert_eq!(output.matches("<path ").count(), 1, "notehead only");
    }

    #[test]
    fn chord_with_dynamic_adds_extra_path() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Chord(ChordEvent {
            staff_positions: vec![0, 4],
            duration_log2: 2,
            dots: 0,
            accidentals: vec![None, None],
            stem_direction: None,
            tie_forward: false,
            dynamic: Some(Dynamic::Pp),
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 2 noteheads + 1 dynamic = 3 paths, 1 stem
        assert_eq!(output.matches("<path ").count(), 3, "2 noteheads + dynamic");
        assert_eq!(output.matches("<line ").count(), 1, "one shared stem");
    }

    #[test]
    fn dynamic_glyph_positioned_below_staff() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::Note(NoteEvent {
            staff_position: 4,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
            tie_forward: false,
            dynamic: Some(Dynamic::Mf),
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // Dynamic should be rendered with a translate below the staff.
        // The staff bottom_y is at position 0. The dynamic is 2.5 staff spaces below.
        let bottom_y = staff.bottom_y();
        let dynamic_y = bottom_y + 2.5 * staff.staff_space;
        // The SVG should contain a translate with the dynamic y coordinate
        let y_str = format!("{dynamic_y}");
        assert!(
            output.contains(&y_str),
            "dynamic translate should contain y={dynamic_y}"
        );
    }

    #[test]
    fn different_dynamics_on_notes_produce_different_svgs() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

        let make = |dyn_mark: Dynamic| {
            let elements = vec![MeasureElement::Note(NoteEvent {
                staff_position: 4,
                duration_log2: 2,
                dots: 0,
                accidental: None,
                stem_direction: None,
                tie_forward: false,
                dynamic: Some(dyn_mark),
            })];
            let layout = layout_measure(&elements, &cfg);
            let mut svg = make_svg();
            draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
            svg.to_svg()
        };

        let svg_p = make(Dynamic::Piano);
        let svg_f = make(Dynamic::Forte);
        assert_ne!(svg_p, svg_f, "different dynamics should produce different SVGs");
    }

    // --- tuplet group rendering ---

    use crate::layout::measure::TupletGroupEvent;

    #[test]
    fn tuplet_triplet_renders_beams_plus_bracket() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::TupletGroup(TupletGroupEvent {
            beam_group: BeamGroupEvent {
                notes: vec![
                    NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                    NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                    NoteEvent { staff_position: 4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                ],
                stem_direction: None,
            },
            tuplet_number: 3,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 3 noteheads + 1 tuplet number glyph = 4 paths
        assert_eq!(output.matches("<path ").count(), 4, "3 noteheads + tuplet number");
        // 3 stems + 2 hooks + 2 bracket segments = 7 lines
        assert_eq!(output.matches("<line ").count(), 7, "3 stems + 4 bracket/hook lines");
        // 1 primary beam polygon
        assert_eq!(output.matches("<polygon ").count(), 1, "one beam polygon");
    }

    #[test]
    fn tuplet_differs_from_plain_beam_group() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let notes = vec![
            NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            NoteEvent { staff_position: 4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
        ];

        let plain = vec![MeasureElement::BeamGroup(BeamGroupEvent {
            notes: notes.clone(),
            stem_direction: None,
        })];
        let tuplet = vec![MeasureElement::TupletGroup(TupletGroupEvent {
            beam_group: BeamGroupEvent {
                notes: notes.clone(),
                stem_direction: None,
            },
            tuplet_number: 3,
        })];

        let layout_p = layout_measure(&plain, &cfg);
        let layout_t = layout_measure(&tuplet, &cfg);

        let mut svg_p = make_svg();
        draw_measure(&mut svg_p, &staff, &font, &config, &layout_p, 0.0, &Clef::Treble).unwrap();
        let mut svg_t = make_svg();
        draw_measure(&mut svg_t, &staff, &font, &config, &layout_t, 0.0, &Clef::Treble).unwrap();

        let out_p = svg_p.to_svg();
        let out_t = svg_t.to_svg();

        // Tuplet has extra bracket lines and number glyph
        let paths_p = out_p.matches("<path ").count();
        let paths_t = out_t.matches("<path ").count();
        assert_eq!(paths_t, paths_p + 1, "tuplet adds 1 extra path (number glyph)");

        let lines_p = out_p.matches("<line ").count();
        let lines_t = out_t.matches("<line ").count();
        assert!(lines_t > lines_p, "tuplet has more lines (bracket + hooks)");
    }

    #[test]
    fn tuplet_quintuplet_renders() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::TupletGroup(TupletGroupEvent {
            beam_group: BeamGroupEvent {
                notes: vec![
                    NoteEvent { staff_position: 2, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                    NoteEvent { staff_position: 3, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                    NoteEvent { staff_position: 4, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                    NoteEvent { staff_position: 5, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                    NoteEvent { staff_position: 6, duration_log2: 4, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
                ],
                stem_direction: None,
            },
            tuplet_number: 5,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        // 5 noteheads + 1 tuplet "5" glyph = 6 paths
        assert_eq!(output.matches("<path ").count(), 6, "5 noteheads + tuplet number");
    }

    #[test]
    fn tuplet_empty_produces_nothing() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let elements = vec![MeasureElement::TupletGroup(TupletGroupEvent {
            beam_group: BeamGroupEvent {
                notes: vec![],
                stem_direction: None,
            },
            tuplet_number: 3,
        })];
        let layout = layout_measure(&elements, &cfg);
        let mut svg = make_svg();
        draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 0);
        assert_eq!(output.matches("<line ").count(), 0);
    }

    #[test]
    fn tuplet_different_numbers_produce_different_glyphs() {
        let (font, config, staff) = setup();
        let cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);
        let notes = vec![
            NoteEvent { staff_position: 0, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            NoteEvent { staff_position: 2, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
            NoteEvent { staff_position: 4, duration_log2: 3, dots: 0, accidental: None, stem_direction: None, tie_forward: false, dynamic: None },
        ];

        let make = |number: u32| {
            let elems = vec![MeasureElement::TupletGroup(TupletGroupEvent {
                beam_group: BeamGroupEvent {
                    notes: notes.clone(),
                    stem_direction: None,
                },
                tuplet_number: number,
            })];
            let layout = layout_measure(&elems, &cfg);
            let mut svg = make_svg();
            draw_measure(&mut svg, &staff, &font, &config, &layout, 0.0, &Clef::Treble).unwrap();
            svg.to_svg()
        };

        let svg_3 = make(3);
        let svg_5 = make(5);
        assert_ne!(svg_3, svg_5, "triplet and quintuplet should differ");
    }
}