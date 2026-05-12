use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::glissando::{layout_glissando, GlissandoStyle};
use crate::layout::hairpin::{layout_hairpin, HairpinType};
use crate::layout::ornament::{layout_ornament, Ornament};
use crate::layout::ottava::{layout_ottava_bracket, OttavaKind};
use crate::layout::lyric::{LyricContinuation, LyricSyllable, LYRIC_BELOW_STAFF_SS};
use crate::layout::measure::{MeasureElement, PositionedElement};
use crate::layout::slur::{layout_slur, slur_direction_from_stem};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{auto_stem_direction, StemDirection};
use crate::layout::system::SystemLayout;
use crate::layout::tie::{layout_tie, tie_direction_from_stem};
use crate::layout::trill_extension::layout_trill_extension;
use crate::layout::volta::layout_volta_bracket;
use crate::render::measure_renderer::{draw_additional_voices, draw_measure};
use crate::render::note_renderer::NoteheadKind;
use crate::render::glissando_renderer::draw_glissando;
use crate::render::hairpin_renderer::draw_hairpin;
use crate::render::lyric_renderer::draw_lyric_extender;
use crate::render::slur_renderer::draw_slur;
use crate::render::staff_renderer::draw_staff_lines;
use crate::render::tie_renderer::draw_tie;
use crate::render::ottava_renderer::draw_ottava_bracket;
use crate::render::trill_extension_renderer::draw_trill_extension;
use crate::render::volta_renderer::draw_volta_bracket;
use crate::render::SvgWriter;
use smufl::Glyph;

/// Iterate over all positioned elements in a measure, including both the
/// primary voice and any additional voices. Each element is yielded with
/// its absolute x within the system (`measure.x_offset + elem.x`).
///
/// Additional voices share the same x-coordinate space as the primary
/// voice (they were scaled to match width in `layout_system()`), so spans
/// (ties, slurs, hairpins, etc.) resolve correctly across all voices.
fn all_measure_elements(measure: &crate::layout::system::SystemMeasure) -> impl Iterator<Item = (f64, &PositionedElement)> {
    let base_x = measure.x_offset;
    let primary = measure.layout.elements.iter().map(move |e| (base_x + e.x, e));
    let additional = measure.additional_voice_layouts.iter().flat_map(move |voice_layout| {
        voice_layout.elements.iter().map(move |e| (base_x + e.x, e))
    });
    primary.chain(additional)
}

/// Collect notes from the system's positioned elements in order, yielding
/// (x_in_system, staff_position, duration_log2, tie_forward, stem_direction_override)
/// for each note event. Chord notes are expanded into individual entries so
/// each chord note can be tied independently. Skips clefs, rests, barlines, etc.
///
/// Scans both the primary voice and any additional voices so that ties
/// within secondary voices are resolved.
pub(crate) fn collect_note_positions(system: &SystemLayout) -> Vec<(f64, i8, u8, bool, Option<StemDirection>)> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push((elem_x, n.staff_position, n.duration_log2, n.annotations.tie_forward, n.stem_direction));
                }
                MeasureElement::Chord(c) => {
                    for &pos in &c.staff_positions {
                        notes.push((elem_x, pos, c.duration_log2, c.annotations.tie_forward, c.stem_direction));
                    }
                }
                _ => {}
            }
        }
    }
    notes
}

/// Draw a complete system (staff lines + all measures) onto an SVG writer.
///
/// Staff lines span the full `staff_width` of the system layout.
/// Each measure is drawn at its computed x-offset. After all measures are
/// rendered, tie curves are drawn between notes marked with `tie_forward`.
pub fn draw_system(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    x: f64,
    y: f64,
) -> Result<(), FontError> {
    if system.measures.is_empty() {
        return Ok(());
    }

    // Create staff spanning the full system width
    let staff = StaffLayout::new(x, y, system.staff_width, config.staff_space);

    // Draw continuous staff lines
    draw_staff_lines(svg, &staff, config);

    // Draw each measure at its offset (primary voice + additional voices)
    let clef = system.clef_kind.to_clef();
    for sys_measure in &system.measures {
        let measure_x = x + sys_measure.x_offset;
        draw_measure(
            svg,
            &staff,
            font,
            config,
            &sys_measure.layout,
            measure_x,
            &clef,
        )?;
        if !sys_measure.additional_voice_layouts.is_empty() {
            draw_additional_voices(
                svg,
                &staff,
                font,
                config,
                &sys_measure.layout,
                &sys_measure.additional_voice_layouts,
                measure_x,
            )?;
        }
    }

    // Draw ties between notes with tie_forward = true and their target notes
    draw_system_ties(svg, font, config, system, &staff, x)?;

    // Draw slurs between notes marked with slur_start and slur_end
    draw_system_slurs(svg, font, config, system, &staff, x)?;

    // Draw hairpins between notes marked with hairpin_start and hairpin_end
    draw_system_hairpins(svg, font, config, system, &staff, x)?;

    // Draw lyric extender lines (melisma) between syllables with Extender
    // continuation and the next note that has a lyric
    draw_system_lyric_extenders(svg, config, system, &staff, x);

    // Draw volta brackets above measures that have volta annotations
    draw_system_volta_brackets(svg, config, system, &staff, x);

    // Draw ottava brackets (8va/8vb dashed lines) between marked notes
    draw_system_ottava_brackets(svg, font, config, system, &staff, x)?;

    // Draw glissando lines between notes marked with glissando_start
    draw_system_glissandos(svg, config, system, &staff, x);

    // Draw trill wavy-line extensions to the next note for notes marked with
    // trill_extension. A "tr" glyph (the actual ornament) is already drawn by
    // the measure renderer via draw_ornament; this pass only adds the trailing
    // wiggle.
    draw_system_trill_extensions(svg, font, system, &staff, x)?;

    Ok(())
}

/// After all measures in a system are drawn, scan for tied notes and draw
/// tie curves between them.
///
/// For each note with `tie_forward = true`, finds the next note at the same
/// staff position and draws a tie from the right edge of the first notehead
/// to the left edge of the second.
fn draw_system_ties(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let note_positions = collect_note_positions(system);

    for (i, &(nx, pos, dur_log2, tie_forward, stem_dir)) in note_positions.iter().enumerate() {
        if !tie_forward {
            continue;
        }

        // Find the next note at the same staff position
        let target = note_positions[i + 1..]
            .iter()
            .find(|&&(_, target_pos, _, _, _)| target_pos == pos);

        let Some(&(target_x, _, _, _, _)) = target else {
            continue;
        };

        // Compute notehead advance width for tie endpoint positioning
        let notehead_kind = match dur_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        // Tie starts at right edge of first notehead, ends at left edge of second
        let tie_x_start = system_x + nx + advance;
        let tie_x_end = system_x + target_x;

        // Determine tie direction from stem direction
        let direction = stem_dir.unwrap_or_else(|| auto_stem_direction(pos));
        let tie_dir = tie_direction_from_stem(direction);

        // Y-coordinate of the notehead center
        let note_y = staff.y_of(pos);

        let tie_layout = layout_tie(tie_x_start, tie_x_end, note_y, tie_dir, config);
        draw_tie(svg, &tie_layout);
    }

    Ok(())
}

/// Positional info for a note relevant to slur drawing.
pub(crate) struct SlurNoteInfo {
    pub(crate) x: f64,
    pub(crate) staff_position: i8,
    pub(crate) duration_log2: u8,
    pub(crate) stem_direction: Option<StemDirection>,
    pub(crate) slur_start: bool,
    pub(crate) slur_end: bool,
}

/// Collect slur-relevant note info from the system.
pub(crate) fn collect_slur_note_info(system: &SystemLayout) -> Vec<SlurNoteInfo> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push(SlurNoteInfo {
                        x: elem_x,
                        staff_position: n.staff_position,
                        duration_log2: n.duration_log2,
                        stem_direction: n.stem_direction,
                        slur_start: n.annotations.slur_start,
                        slur_end: n.annotations.slur_end,
                    });
                }
                MeasureElement::Chord(c) => {
                    let top_pos = c.staff_positions.iter().copied().max().unwrap_or(0);
                    let bot_pos = c.staff_positions.iter().copied().min().unwrap_or(0);
                    let dir = c.stem_direction.unwrap_or_else(|| auto_stem_direction(top_pos));
                    let attach_pos = match dir {
                        StemDirection::Up => bot_pos,
                        StemDirection::Down => top_pos,
                    };
                    notes.push(SlurNoteInfo {
                        x: elem_x,
                        staff_position: attach_pos,
                        duration_log2: c.duration_log2,
                        stem_direction: c.stem_direction,
                        slur_start: c.annotations.slur_start,
                        slur_end: c.annotations.slur_end,
                    });
                }
                _ => {}
            }
        }
    }
    notes
}

/// Draw slur curves between notes marked with `slur_start` and `slur_end`.
///
/// For each note with `slur_start = true`, finds the next note with
/// `slur_end = true` and draws a slur curve between them. Slur direction
/// is determined from the start note's stem direction.
fn draw_system_slurs(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let note_info = collect_slur_note_info(system);

    for (i, info) in note_info.iter().enumerate() {
        if !info.slur_start {
            continue;
        }

        // Find the next note with slur_end = true
        let target = note_info[i + 1..]
            .iter()
            .find(|n| n.slur_end);

        let Some(target) = target else {
            continue;
        };

        // Compute notehead advance width for slur endpoint positioning
        let notehead_kind = match info.duration_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        // Slur starts at right edge of first notehead, ends at left edge of last
        let slur_x_start = system_x + info.x + advance;
        let slur_x_end = system_x + target.x;

        // Determine slur direction from start note's stem
        let stem_dir = info.stem_direction
            .unwrap_or_else(|| auto_stem_direction(info.staff_position));
        let direction = slur_direction_from_stem(stem_dir);

        // Y-coordinates at attachment points
        let start_y = staff.y_of(info.staff_position);
        let end_y = staff.y_of(target.staff_position);

        let slur_layout = layout_slur(
            slur_x_start,
            slur_x_end,
            start_y,
            end_y,
            direction,
            config,
        );
        draw_slur(svg, &slur_layout);
    }

    Ok(())
}

/// Positional info for a note relevant to hairpin drawing.
pub(crate) struct HairpinNoteInfo {
    pub(crate) x: f64,
    pub(crate) duration_log2: u8,
    pub(crate) hairpin_start: Option<HairpinType>,
    pub(crate) hairpin_end: bool,
}

pub(crate) fn collect_hairpin_note_info(system: &SystemLayout) -> Vec<HairpinNoteInfo> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push(HairpinNoteInfo {
                        x: elem_x,
                        duration_log2: n.duration_log2,
                        hairpin_start: n.annotations.hairpin_start,
                        hairpin_end: n.annotations.hairpin_end,
                    });
                }
                MeasureElement::Chord(c) => {
                    notes.push(HairpinNoteInfo {
                        x: elem_x,
                        duration_log2: c.duration_log2,
                        hairpin_start: c.annotations.hairpin_start,
                        hairpin_end: c.annotations.hairpin_end,
                    });
                }
                _ => {}
            }
        }
    }
    notes
}

fn draw_system_hairpins(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let note_info = collect_hairpin_note_info(system);

    for (i, info) in note_info.iter().enumerate() {
        let Some(hairpin_type) = info.hairpin_start else {
            continue;
        };

        // Find the next note with hairpin_end = true
        let target = note_info[i + 1..]
            .iter()
            .find(|n| n.hairpin_end);

        let Some(target) = target else {
            continue;
        };

        // Compute notehead advance width for hairpin start positioning
        let notehead_kind = match info.duration_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        // Hairpin starts right of the first notehead, ends at left of the target
        let hp_x_start = system_x + info.x + advance + 0.3 * config.staff_space;
        let hp_x_end = system_x + target.x - 0.3 * config.staff_space;

        // Use staff line thickness as hairpin stroke width
        let stroke_width = config.staff_line_thickness_fu();

        let hp_layout = layout_hairpin(
            hairpin_type,
            hp_x_start,
            hp_x_end,
            staff.bottom_y(),
            config.staff_space,
            stroke_width,
        );
        draw_hairpin(svg, &hp_layout);
    }

    Ok(())
}

/// Positional info for a note/chord relevant to lyric extender drawing.
pub(crate) struct LyricNoteInfo {
    /// X-offset of the note within the system (before system_x is added).
    pub(crate) x: f64,
    /// The lyric syllable, if any.
    pub(crate) lyric: Option<LyricSyllable>,
}

/// Collect lyric-relevant note info from the system in sequential order.
pub(crate) fn collect_lyric_note_info(system: &SystemLayout) -> Vec<LyricNoteInfo> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push(LyricNoteInfo {
                        x: elem_x,
                        lyric: n.annotations.lyric.clone(),
                    });
                }
                MeasureElement::Chord(c) => {
                    notes.push(LyricNoteInfo {
                        x: elem_x,
                        lyric: c.annotations.lyric.clone(),
                    });
                }
                _ => {}
            }
        }
    }
    notes
}

/// Draw lyric extender lines (melisma underscores) between syllables that
/// have `Extender` continuation and the next note position.
///
/// The extender runs from just past the source syllable text to just before
/// the target note's position. The target is the next note/chord in the
/// system regardless of whether it has a lyric — the held syllable sustains
/// until the next rhythmic event.
fn draw_system_lyric_extenders(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) {
    let note_info = collect_lyric_note_info(system);

    let y_baseline = staff.y_of(0) + LYRIC_BELOW_STAFF_SS * config.staff_space;
    let stroke_width = config.staff_line_thickness_fu();

    for (i, info) in note_info.iter().enumerate() {
        let Some(ref lyric) = info.lyric else {
            continue;
        };
        if lyric.continuation != LyricContinuation::Extender {
            continue;
        }

        // Find the next note/chord (any — the held syllable extends until
        // the next rhythmic event regardless of whether it carries a lyric).
        let Some(target) = note_info.get(i + 1) else {
            continue;
        };

        draw_lyric_extender(
            svg,
            system_x + info.x,
            system_x + target.x,
            y_baseline,
            config.staff_space,
            stroke_width,
        );
    }
}

/// Draw volta brackets above measures that have volta annotations.
fn draw_system_volta_brackets(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) {
    for sys_measure in &system.measures {
        let Some(ref annotation) = sys_measure.volta else {
            continue;
        };

        let x_left = system_x + sys_measure.x_offset;
        let x_right = x_left + sys_measure.layout.total_width;

        let bracket_layout = layout_volta_bracket(
            annotation,
            x_left,
            x_right,
            staff,
            config.staff_space,
        );
        draw_volta_bracket(svg, &bracket_layout);
    }
}

/// Positional info for a note relevant to ottava bracket drawing.
pub(crate) struct OttavaNoteInfo {
    pub(crate) x: f64,
    pub(crate) duration_log2: u8,
    pub(crate) ottava_start: Option<OttavaKind>,
    pub(crate) ottava_end: bool,
}

pub(crate) fn collect_ottava_note_info(system: &SystemLayout) -> Vec<OttavaNoteInfo> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push(OttavaNoteInfo {
                        x: elem_x,
                        duration_log2: n.duration_log2,
                        ottava_start: n.annotations.ottava_start,
                        ottava_end: n.annotations.ottava_end,
                    });
                }
                MeasureElement::Chord(c) => {
                    notes.push(OttavaNoteInfo {
                        x: elem_x,
                        duration_log2: c.duration_log2,
                        ottava_start: c.annotations.ottava_start,
                        ottava_end: c.annotations.ottava_end,
                    });
                }
                _ => {}
            }
        }
    }
    notes
}

/// Draw ottava brackets (8va/8vb/15ma/15mb) between notes marked with
/// `ottava_start` and `ottava_end`.
fn draw_system_ottava_brackets(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let note_info = collect_ottava_note_info(system);

    for (i, info) in note_info.iter().enumerate() {
        let Some(kind) = info.ottava_start else {
            continue;
        };

        // Find the next note with ottava_end = true
        let target = note_info[i + 1..]
            .iter()
            .find(|n| n.ottava_end);

        let Some(target) = target else {
            continue;
        };

        // Compute notehead advance width for endpoint positioning
        let notehead_kind = match info.duration_log2 {
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let outline = font.glyph_outline(notehead_kind.glyph())?;
        let advance = outline.advance_width as f64;

        let ott_x_start = system_x + info.x;
        let ott_x_end = system_x + target.x + advance;

        let bracket_layout = layout_ottava_bracket(
            kind,
            ott_x_start,
            ott_x_end,
            staff,
            config.staff_space,
            true,
        );
        draw_ottava_bracket(svg, &bracket_layout);
    }

    Ok(())
}

/// Info about a note's glissando state for second-pass rendering.
pub(crate) struct GlissandoNoteInfo {
    pub x: f64,
    pub staff_position: i8,
    pub stem_direction: Option<StemDirection>,
    pub glissando_start: Option<GlissandoStyle>,
}

/// Collect note info relevant to glissando rendering from a system's elements.
pub(crate) fn collect_glissando_note_info(system: &SystemLayout) -> Vec<GlissandoNoteInfo> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push(GlissandoNoteInfo {
                        x: elem_x,
                        staff_position: n.staff_position,
                        stem_direction: n.stem_direction,
                        glissando_start: n.annotations.glissando_start,
                    });
                }
                MeasureElement::Chord(c) => {
                    let top_pos = c.staff_positions.iter().copied().max().unwrap_or(0);
                    let bot_pos = c.staff_positions.iter().copied().min().unwrap_or(0);
                    let dir = c.stem_direction.unwrap_or_else(|| auto_stem_direction(top_pos));
                    let attach_pos = match dir {
                        StemDirection::Up => bot_pos,
                        StemDirection::Down => top_pos,
                    };
                    notes.push(GlissandoNoteInfo {
                        x: elem_x,
                        staff_position: attach_pos,
                        stem_direction: c.stem_direction,
                        glissando_start: c.annotations.glissando_start,
                    });
                }
                _ => {}
            }
        }
    }
    notes
}

/// Draw glissando lines between notes marked with `glissando_start`.
fn draw_system_glissandos(
    svg: &mut SvgWriter,
    _config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) {
    let notes = collect_glissando_note_info(system);

    for (i, note) in notes.iter().enumerate() {
        let Some(style) = note.glissando_start else {
            continue;
        };

        // Find the next note (any position — glissando connects to the
        // immediately following note, unlike ties which match by position)
        let Some(target) = notes.get(i + 1) else {
            continue;
        };

        if let Some(layout) = layout_glissando(
            system_x + note.x,
            note.staff_position,
            system_x + target.x,
            target.staff_position,
            staff,
            style,
            note.stem_direction,
        ) {
            draw_glissando(svg, &layout);
        }
    }
}

/// Info about a note's trill-extension state for second-pass rendering.
pub(crate) struct TrillExtensionNoteInfo {
    /// X-coordinate of the note's notehead within the system layout space
    /// (i.e. without `system_x` added — same convention as the other
    /// `collect_*_note_info` functions).
    pub x: f64,
    /// Staff position; used by the system renderer to call `layout_ornament`
    /// in exactly the same way the measure renderer did, so the wiggle
    /// shares a baseline with the "tr" glyph.
    pub staff_position: i8,
    /// Whether the note has a trill ornament whose extension is enabled.
    /// Already filtered to "trill + extension" — the collector skips other
    /// ornaments and skips notes whose `trill_extension` flag is false.
    pub has_trill_extension: bool,
}

/// Collect notes relevant to trill-extension rendering. Includes a `None`-like
/// entry (via `has_trill_extension = false`) for every note so the draw pass
/// can find the immediately following note regardless of whether it also has
/// a trill — exactly the same shape as `collect_glissando_note_info`.
pub(crate) fn collect_trill_extension_note_info(
    system: &SystemLayout,
) -> Vec<TrillExtensionNoteInfo> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    let has_ext = n.annotations.trill_extension
                        && matches!(n.annotations.ornament, Some(Ornament::Trill));
                    notes.push(TrillExtensionNoteInfo {
                        x: elem_x,
                        staff_position: n.staff_position,
                        has_trill_extension: has_ext,
                    });
                }
                MeasureElement::Chord(c) => {
                    // Anchor the extension to the highest note of the chord,
                    // matching how the trill glyph is positioned by the
                    // measure renderer.
                    let top_pos = c.staff_positions.iter().copied().max().unwrap_or(0);
                    let has_ext = c.annotations.trill_extension
                        && matches!(c.annotations.ornament, Some(Ornament::Trill));
                    notes.push(TrillExtensionNoteInfo {
                        x: elem_x,
                        staff_position: top_pos,
                        has_trill_extension: has_ext,
                    });
                }
                _ => {}
            }
        }
    }
    notes
}

/// Distance (in staff spaces) past the "tr" glyph where the wiggle starts.
const TRILL_EXTENSION_GLYPH_GAP_SS: f64 = 0.15;
/// Distance (in staff spaces) before the next notehead where the wiggle ends.
/// Leaves visual breathing room so the wiggle doesn't crash into the notehead.
const TRILL_EXTENSION_NOTE_GAP_SS: f64 = 0.30;

/// Draw trill wavy-line extensions for notes marked with `trill_extension`.
///
/// The wiggle starts just past the "tr" glyph (so the two read as one
/// continuous mark) and ends just short of the next note. Notes whose
/// extension would have no room (single-segment minimum span) silently
/// render no wiggle — the trill is still indicated by the "tr" glyph
/// alone, which is conventional for short trills.
fn draw_system_trill_extensions(
    svg: &mut SvgWriter,
    font: &MusicFont,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let notes = collect_trill_extension_note_info(system);

    let trill_advance = font.glyph_advance(Glyph::OrnamentTrill)? as f64;
    let wiggle_advance = font.glyph_advance(Glyph::WiggleTrill)? as f64;

    for (i, note) in notes.iter().enumerate() {
        if !note.has_trill_extension {
            continue;
        }

        // The wiggle continues until the next note in the system; if there
        // is none (last note in system), skip. Cross-system trills are a
        // deferred enhancement — see open issues.
        let Some(target) = notes.get(i + 1) else {
            continue;
        };

        // The "tr" glyph is positioned by layout_ornament; mirror its math
        // so the wiggle's y matches the trill glyph's baseline.
        let ornament_layout = layout_ornament(
            Ornament::Trill,
            system_x + note.x,
            note.staff_position,
            staff,
        );

        let staff_space = staff.staff_space;
        let trill_x = ornament_layout.x;
        let start_x = trill_x + trill_advance + TRILL_EXTENSION_GLYPH_GAP_SS * staff_space;
        let end_x = system_x + target.x - TRILL_EXTENSION_NOTE_GAP_SS * staff_space;

        if let Some(layout) =
            layout_trill_extension(start_x, end_x, ornament_layout.y, wiggle_advance)
        {
            draw_trill_extension(svg, font, &layout)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
