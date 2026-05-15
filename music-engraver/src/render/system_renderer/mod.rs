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
use crate::layout::trill_bracket::{
    layout_trill_bracket_hook, layout_trill_bracket_hooks, HookDirection, TrillBracketSide,
};
use crate::layout::trill_extension::{layout_trill_extension_with_glyph, TrillWiggleSpeed};
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
use crate::render::trill_bracket_renderer::draw_trill_bracket_hooks;
use crate::render::trill_extension_renderer::draw_trill_extension;
use crate::render::volta_renderer::draw_volta_bracket;
use crate::render::SvgWriter;

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
    draw_system_trill_extensions(svg, font, config, system, &staff, x)?;

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
///
/// Only handles brackets that resolve within a single system. The
/// cross-system case (unresolved `ottava_start` in this system + matching
/// `ottava_end` in a later system, or no matching `ottava_end` at all on
/// the page) is owned by `page_renderer::draw_cross_system_ottava_brackets`,
/// which has the page-wide system list needed to draw the trailing
/// half-bracket on system N and the incoming half-bracket on system N+1.
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
    /// Whether the note has an extension-supporting trill ornament whose
    /// extension is enabled. Already filtered by
    /// [`Ornament::supports_trill_extension`] — the collector skips notes
    /// whose ornament is not Trill or TrillWithMordent, and skips notes
    /// whose `trill_extension` flag is false.
    pub has_trill_extension: bool,
    /// The actual ornament variant on the note. Carried alongside
    /// `has_trill_extension` so the draw pass can look up the *correct*
    /// glyph advance to position the wiggle's start: `OrnamentTrill` and
    /// `OrnamentPrecompTrillWithMordent` have different advances (the
    /// precomposed compound includes the mordent suffix), and the wiggle
    /// must start after the full glyph, not just the "tr" prefix. `None`
    /// when `has_trill_extension == false`.
    pub ornament: Option<Ornament>,
    /// Optional bracket form for this trill extension. Filtered the same way
    /// as `has_trill_extension`: only carries through when the underlying
    /// note actually has both `Trill + extension`. A bracket request on a
    /// non-trilled note is silently inert.
    pub bracket: Option<TrillBracketSide>,
    /// Optional override for the bracket hook direction. Only meaningful when
    /// `bracket` is also `Some`. `None` selects `HookDirection::Down`.
    pub bracket_direction: Option<HookDirection>,
    /// Optional override for the bracket hook length, in staff spaces. Only
    /// meaningful when `bracket` is also `Some`. `None` selects the default
    /// `TRILL_BRACKET_HOOK_LENGTH_SS`.
    pub bracket_length_ss: Option<f64>,
    /// Speed variant for the wavy-line glyph. Filtered the same way as
    /// `bracket`: only meaningful when `has_trill_extension == true`.
    /// `None` selects the standard wiggle.
    pub wiggle_speed: Option<TrillWiggleSpeed>,
    /// Optional explicit termination length for the wiggle, in staff spaces.
    /// Filtered the same way as `bracket`: only carries through when
    /// `has_trill_extension == true`. `None` selects the conventional
    /// "extend to next note or system edge" behavior; `Some(length_ss)`
    /// clamps the wiggle so it terminates no later than `length_ss` past
    /// its natural start. When set to a positive value the trill is also
    /// treated as definitively terminated within its source system
    /// (no cross-system propagation, since the explicit length specifies
    /// a definite endpoint).
    pub explicit_length_ss: Option<f64>,
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
                        && n.annotations
                            .ornament
                            .map(|o| o.supports_trill_extension())
                            .unwrap_or(false);
                    let ornament = if has_ext { n.annotations.ornament } else { None };
                    let bracket = if has_ext { n.annotations.trill_bracket } else { None };
                    let bracket_direction = if bracket.is_some() {
                        n.annotations.trill_bracket_direction
                    } else {
                        None
                    };
                    let bracket_length_ss = if bracket.is_some() {
                        n.annotations.trill_bracket_length_ss
                    } else {
                        None
                    };
                    let wiggle_speed = if has_ext {
                        n.annotations.trill_wiggle_speed
                    } else {
                        None
                    };
                    let explicit_length_ss = if has_ext {
                        n.annotations.trill_extension_length_ss
                    } else {
                        None
                    };
                    notes.push(TrillExtensionNoteInfo {
                        x: elem_x,
                        staff_position: n.staff_position,
                        has_trill_extension: has_ext,
                        ornament,
                        bracket,
                        bracket_direction,
                        bracket_length_ss,
                        wiggle_speed,
                        explicit_length_ss,
                    });
                }
                MeasureElement::Chord(c) => {
                    // Anchor the extension to the highest note of the chord,
                    // matching how the trill glyph is positioned by the
                    // measure renderer.
                    let top_pos = c.staff_positions.iter().copied().max().unwrap_or(0);
                    let has_ext = c.annotations.trill_extension
                        && c.annotations
                            .ornament
                            .map(|o| o.supports_trill_extension())
                            .unwrap_or(false);
                    let ornament = if has_ext { c.annotations.ornament } else { None };
                    let bracket = if has_ext { c.annotations.trill_bracket } else { None };
                    let bracket_direction = if bracket.is_some() {
                        c.annotations.trill_bracket_direction
                    } else {
                        None
                    };
                    let bracket_length_ss = if bracket.is_some() {
                        c.annotations.trill_bracket_length_ss
                    } else {
                        None
                    };
                    let wiggle_speed = if has_ext {
                        c.annotations.trill_wiggle_speed
                    } else {
                        None
                    };
                    let explicit_length_ss = if has_ext {
                        c.annotations.trill_extension_length_ss
                    } else {
                        None
                    };
                    notes.push(TrillExtensionNoteInfo {
                        x: elem_x,
                        staff_position: top_pos,
                        has_trill_extension: has_ext,
                        ornament,
                        bracket,
                        bracket_direction,
                        bracket_length_ss,
                        wiggle_speed,
                        explicit_length_ss,
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
/// Shared with `page_renderer::draw_cross_system_trill_extensions` so the
/// incoming wiggle on system N+1 terminates with the same visual gap.
pub(crate) const TRILL_EXTENSION_NOTE_GAP_SS: f64 = 0.30;
/// Distance (in staff spaces) before the system's right edge where a
/// cross-system wiggle terminates. Conventionally the wavy line stops just
/// short of the final barline so the two marks remain visually distinct.
/// Tuned slightly larger than the inter-note gap because the barline carries
/// more visual weight than a notehead.
const TRILL_EXTENSION_SYSTEM_EDGE_GAP_SS: f64 = 0.5;
/// Length of a trill bracket hook in staff spaces. Behind Bars: "the wavy line
/// is bracketed at one or both ends" — the hook is short, conventionally
/// around three-quarters of a staff space, just long enough to read as a
/// vertical terminator rather than a barline.
pub(crate) const TRILL_BRACKET_HOOK_LENGTH_SS: f64 = 0.75;

/// Draw trill wavy-line extensions for notes marked with `trill_extension`.
///
/// The wiggle starts just past the "tr" glyph (so the two read as one
/// continuous mark) and ends just short of the next note. When the trilled
/// note is the last note in its system, the wiggle extends instead to the
/// right edge of the system (with a small gap before the final barline) —
/// the convention is that a sustained trill at the end of a system continues
/// visually to the system break. Notes whose extension would have no room
/// (single-segment minimum span) silently render no wiggle — the trill is
/// still indicated by the "tr" glyph alone.
fn draw_system_trill_extensions(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let notes = collect_trill_extension_note_info(system);

    let hook_stroke = config.thin_barline_thickness_fu();
    let default_hook_length = TRILL_BRACKET_HOOK_LENGTH_SS * staff.staff_space;

    for (i, note) in notes.iter().enumerate() {
        if !note.has_trill_extension {
            continue;
        }

        // The ornament-glyph y is glyph-independent (layout_ornament only
        // reads staff geometry), so any extension-supporting ornament gives
        // the same baseline — passing `Trill` here is sufficient. The
        // glyph *advance*, however, differs: a precomposed
        // TrillWithMordent glyph extends further to the right than a bare
        // "tr", and the wiggle must start past the full glyph extent so
        // the mordent suffix isn't overdrawn.
        let ornament_kind = note.ornament.unwrap_or(Ornament::Trill);
        let ornament_layout = layout_ornament(
            Ornament::Trill,
            system_x + note.x,
            note.staff_position,
            staff,
        );

        let staff_space = staff.staff_space;
        let trill_x = ornament_layout.x;
        let trill_advance = font.glyph_advance(ornament_kind.glyph())? as f64;
        let start_x = trill_x + trill_advance + TRILL_EXTENSION_GLYPH_GAP_SS * staff_space;

        // End at the next note's left edge — or, if this is the last note
        // in the system, at the system's right edge (just inside the final
        // barline). This is the cross-system convention: a trilled note at
        // the end of a system extends its wiggle to the system break.
        //
        // When the user supplied an explicit length, clamp the natural
        // end_x with `start_x + length_ss * staff_space`. The clamp is
        // one-sided (we only ever shorten, never extend past the natural
        // endpoint), so an explicit length larger than the natural span is
        // a no-op rather than an overrun. A positive explicit length also
        // disables cross-system propagation: the trill terminates within
        // this system at the requested point, regardless of position.
        let natural_end_x = match notes.get(i + 1) {
            Some(target) => system_x + target.x - TRILL_EXTENSION_NOTE_GAP_SS * staff_space,
            None => system_x + system.staff_width
                - TRILL_EXTENSION_SYSTEM_EDGE_GAP_SS * staff_space,
        };
        let (end_x, cross_system) = match note.explicit_length_ss {
            Some(len_ss) if len_ss > 0.0 => {
                let requested = start_x + len_ss * staff_space;
                (requested.min(natural_end_x), false)
            }
            // Negative or zero explicit lengths produce no wiggle: setting
            // end_x equal to start_x guarantees `layout_trill_extension`
            // returns None (span < segment_advance), matching the
            // documented fail-safe.
            Some(_) => (start_x, false),
            None => (natural_end_x, notes.get(i + 1).is_none()),
        };

        // Look up the wiggle glyph + advance for this note's chosen speed.
        // Each wiggleTrill* variant has its own advance, so the lookup must
        // travel with the glyph choice.
        let wiggle_glyph = note.wiggle_speed.unwrap_or_default().to_glyph();
        let wiggle_advance = font.glyph_advance(wiggle_glyph)? as f64;

        if let Some(layout) = layout_trill_extension_with_glyph(
            start_x,
            end_x,
            ornament_layout.y,
            wiggle_glyph,
            wiggle_advance,
        ) {
            draw_trill_extension(svg, font, &layout)?;

            // Bracket hooks: cap the wiggle's start and/or end with a short
            // vertical line. For cross-system trills, the End hook is
            // suppressed here and drawn instead on system N+1 at the
            // terminus of the incoming wiggle (so the bracket frames the
            // trill's semantic range, not the per-system wiggle fragment).
            if let Some(side) = note.bracket {
                let render_side = bracket_side_for_system_pass(side, cross_system);
                if let Some(side) = render_side {
                    let hook_length = note
                        .bracket_length_ss
                        .map(|ss| ss * staff.staff_space)
                        .unwrap_or(default_hook_length);
                    let direction = note.bracket_direction.unwrap_or(HookDirection::Down);
                    let hooks = layout_trill_bracket_hooks(
                        &layout,
                        side,
                        hook_length,
                        direction,
                        hook_stroke,
                    );
                    draw_trill_bracket_hooks(svg, &hooks);
                }
            }
        }
    }

    Ok(())
}

/// Filter a user-requested bracket side down to what should be rendered on
/// the current system pass. When the wiggle continues into the next system
/// (`cross_system = true`), the End hook is dropped here so it can be drawn
/// by the page renderer on system N+1 at the incoming wiggle's terminus.
///
/// - Within-system: render exactly what the user asked for.
/// - Cross-system Start: render the Start hook only.
/// - Cross-system End: render nothing on this system (page renderer handles
///   the End hook).
/// - Cross-system Both: render the Start hook only (page renderer handles
///   the End hook).
pub(crate) fn bracket_side_for_system_pass(
    requested: TrillBracketSide,
    cross_system: bool,
) -> Option<TrillBracketSide> {
    if !cross_system {
        return Some(requested);
    }
    match requested {
        TrillBracketSide::Start => Some(TrillBracketSide::Start),
        TrillBracketSide::End => None,
        TrillBracketSide::Both => Some(TrillBracketSide::Start),
    }
}

/// Layout a single bracket end-hook at an arbitrary x. Used by the page
/// renderer to draw the End hook of a cross-system trill on system N+1.
/// `direction` defaults to `HookDirection::Down` for the conventional case
/// of a trill rendered above the staff; callers pass `HookDirection::Up` to
/// flip the hook for trills rendered below the staff.
pub(crate) fn layout_trill_end_hook(
    x: f64,
    baseline_y: f64,
    hook_length: f64,
    direction: HookDirection,
    stroke_width: f64,
) -> crate::layout::trill_bracket::TrillBracketHookLayout {
    layout_trill_bracket_hook(x, baseline_y, hook_length, direction, stroke_width)
}

#[cfg(test)]
mod tests;
