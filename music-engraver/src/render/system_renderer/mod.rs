use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::dynamics::{layout_dynamic_mark, DynamicMark};
use crate::layout::glissando::{layout_glissando, GlissandoStyle, GLISSANDO_NOTEHEAD_WIDTH_SS};
use crate::layout::hairpin::{
    hairpin_reference_y, layout_hairpin_styled, HairpinType, NientePlacement,
};
use crate::layout::lyric::{verse_baseline, LyricContinuation, LyricSyllable, VerseLyric};
use crate::layout::measure::{MeasureElement, NoteAnnotations, NoteheadStyle, PositionedElement};
use crate::layout::ornament::{layout_ornament, Ornament};
use crate::layout::ottava::{layout_ottava_bracket, OttavaKind};
use crate::layout::placement::Placement;
use crate::layout::rest::rest_glyph;
use crate::layout::slur::{layout_slur, slur_direction_from_stem};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{auto_stem_direction, StemDirection};
use crate::layout::system::SystemLayout;
use crate::layout::text_spanner::{layout_text_spanner, TextSpanner};
use crate::layout::tie::{layout_tie, tie_direction_from_stem};
use crate::layout::trill_bracket::{
    layout_trill_bracket_hook, layout_trill_bracket_hooks, layout_trill_bracket_hooks_multi_speed,
    HookDirection, TrillBracketSide,
};
use crate::layout::trill_extension::{
    layout_trill_extension_multi_speed, layout_trill_extension_with_glyph, TrillSpeedRampSpec,
    TrillWiggleSpeed,
};
use crate::layout::volta::layout_volta_bracket;
use crate::render::analysis_bracket_renderer::draw_system_analysis_brackets;
use crate::render::glissando_renderer::draw_glissando;
use crate::render::group_renderer::{draw_groups, GroupItem};
use crate::render::hairpin_renderer::draw_hairpin;
use crate::render::lyric_renderer::{
    draw_lyric_extender, draw_lyric_hyphen_between, extender_source_x, extender_target_x,
};
use crate::render::measure_renderer::{
    collision_shifts, draw_additional_voices, draw_measure_elements,
};
use crate::render::note_renderer::notehead_advance;
use crate::render::ottava_renderer::draw_ottava_bracket;
use crate::render::slur_renderer::draw_slur;
use crate::render::staff_renderer::draw_staff_lines;
use crate::render::text_spanner_renderer::draw_text_spanner;
use crate::render::tie_renderer::draw_tie;
use crate::render::trill_bracket_renderer::draw_trill_bracket_hooks;
use crate::render::trill_extension_renderer::{
    draw_trill_extension, draw_trill_extension_multi_speed,
};
use crate::render::volta_renderer::draw_volta_bracket;
use crate::render::SvgWriter;

/// Iterate over all positioned elements in a measure, including both the
/// primary voice and any additional voices. Each element is yielded with
/// its absolute x within the system (`measure.x_offset + elem.x`).
///
/// Additional voices share the same x-coordinate space as the primary
/// voice (they were scaled to match width in `layout_system()`), so spans
/// (ties, slurs, hairpins, etc.) resolve correctly across all voices.
fn all_measure_elements(
    measure: &crate::layout::system::SystemMeasure,
) -> impl Iterator<Item = (f64, &PositionedElement)> {
    let base_x = measure.x_offset;
    let primary = measure
        .layout
        .elements
        .iter()
        .map(move |e| (base_x + e.x, e));
    let additional = measure
        .additional_voice_layouts
        .iter()
        .flat_map(move |voice_layout| voice_layout.elements.iter().map(move |e| (base_x + e.x, e)));
    primary.chain(additional)
}

fn annotation_notehead_style(annotations: &NoteAnnotations, index: usize) -> NoteheadStyle {
    annotations
        .notehead_styles
        .get(index)
        .copied()
        .unwrap_or_default()
}

fn chord_notehead_style(
    chord: &crate::layout::measure::ChordEvent,
    staff_position: i8,
) -> NoteheadStyle {
    chord
        .staff_positions
        .iter()
        .position(|&position| position == staff_position)
        .map(|index| annotation_notehead_style(&chord.annotations, index))
        .unwrap_or_default()
}

pub(crate) fn widest_notehead_advance(
    font: &MusicFont,
    duration_log2: i8,
    styles: &[NoteheadStyle],
    notehead_count: usize,
) -> Result<f64, FontError> {
    (0..notehead_count.max(1)).try_fold(0.0_f64, |widest, index| {
        notehead_advance(
            font,
            duration_log2,
            styles.get(index).copied().unwrap_or_default(),
        )
        .map(|advance| widest.max(advance))
    })
}

/// Advance of a span endpoint event: its widest notehead, or the rest glyph.
pub(crate) fn span_event_advance(
    font: &MusicFont,
    duration_log2: i8,
    styles: &[NoteheadStyle],
    notehead_count: usize,
    is_rest: bool,
) -> Result<f64, FontError> {
    if is_rest {
        return rest_glyph(duration_log2)
            .map_or(Ok(0.0), |glyph| font.glyph_advance(glyph).map(f64::from));
    }
    widest_notehead_advance(font, duration_log2, styles, notehead_count)
}

/// Collect notes from the system's positioned elements in order, yielding
/// `(x, staff position, duration, style, tie, stem direction)` for each note.
/// Chord notes are expanded into individual entries so each chord tone retains
/// its own custom notehead and can be tied independently.
pub(crate) fn collect_note_positions(
    system: &SystemLayout,
) -> Vec<(f64, i8, i8, NoteheadStyle, bool, Option<StemDirection>)> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push((
                        elem_x,
                        n.staff_position,
                        n.duration_log2,
                        annotation_notehead_style(&n.annotations, 0),
                        n.annotations.tie_forward,
                        n.stem_direction,
                    ));
                }
                MeasureElement::Chord(c) => {
                    for &pos in &c.staff_positions {
                        notes.push((
                            elem_x,
                            pos,
                            c.duration_log2,
                            chord_notehead_style(c, pos),
                            c.annotations.tie_forward,
                            c.stem_direction,
                        ));
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
        draw_measure_elements(
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

    // Draw beam and tuplet spans per voice across the whole system, so a span
    // crossing a barline is one beam or bracket.
    draw_system_groups(svg, font, config, system, &staff, x)?;

    // Draw ties between notes with tie_forward = true and their target notes
    draw_system_ties(svg, font, config, system, &staff, x)?;

    // Draw slurs between notes marked with slur_start and slur_end
    draw_system_slurs(svg, font, config, system, &staff, x)?;

    // Draw hairpins between notes marked with hairpin_start and hairpin_end
    draw_system_hairpins(svg, font, config, system, &staff, x)?;

    // Draw text spanners ("rit. - - -", "cresc. - - -") between events
    // marked with text_spanner_start and text_spanner_end.
    draw_system_text_spanners(svg, font, config, system, &staff, x)?;

    // Draw lyric extender lines (melisma) between syllables with Extender
    // continuation and the next note that has a lyric
    draw_system_lyric_extenders(svg, config, system, &staff, x);

    // Draw lyric hyphens (centered between syllables) for Hyphen continuation.
    // Engraving convention: the hyphen is a separate centered glyph between
    // two syllables, not appended to the source syllable text.
    draw_system_lyric_hyphens(svg, config, system, &staff, x);

    // Draw volta brackets above measures that have volta annotations
    draw_system_volta_brackets(svg, config, system, &staff, x);

    // Draw ottava brackets (8va/8vb dashed lines) between marked notes
    draw_system_ottava_brackets(svg, font, config, system, &staff, x)?;
    // Resolve standard-notation analysis brackets from laid-out event anchors.
    draw_system_analysis_brackets(svg, font, config, system, x, y)?;

    // Draw glissando lines between notes marked with glissando_start
    draw_system_glissandos(svg, font, system, &staff, x)?;

    // Draw trill wavy-line extensions to the next note for notes marked with
    // trill_extension. A "tr" glyph (the actual ornament) is already drawn by
    // the measure renderer via draw_ornament; this pass only adds the trailing
    // wiggle.
    draw_system_trill_extensions(svg, font, config, system, &staff, x)?;

    Ok(())
}

/// Draw every voice's beam and tuplet spans, scanning the voice's elements
/// across all of the system's measures. Spans crossing a barline join; spans
/// continuing from or onto another system are drawn as broken pieces.
fn draw_system_groups(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let voices = system
        .measures
        .iter()
        .map(|measure| measure.additional_voice_layouts.len() + 1)
        .max()
        .unwrap_or(0);
    for voice in 0..voices {
        let items: Vec<_> = system
            .measures
            .iter()
            .flat_map(|measure| {
                let layout = match voice {
                    0 => Some(&measure.layout),
                    _ => measure.additional_voice_layouts.get(voice - 1),
                };
                let measure_x = system_x + measure.x_offset;
                layout.into_iter().flat_map(move |layout| {
                    let shifts = if voice == 0 {
                        Vec::new()
                    } else {
                        collision_shifts(font, &measure.layout, layout)
                    };
                    layout
                        .elements
                        .iter()
                        .enumerate()
                        .map(move |(index, positioned)| {
                            let x = measure_x + positioned.x;
                            GroupItem {
                                x,
                                ink_x: x + shifts.get(index).copied().unwrap_or(0.0),
                                element: &positioned.element,
                            }
                        })
                })
            })
            .collect();
        draw_groups(svg, staff, font, config, &items)?;
    }
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

    for (i, &(nx, pos, dur_log2, style, tie_forward, stem_dir)) in note_positions.iter().enumerate()
    {
        if !tie_forward {
            continue;
        }

        // Find the next note at the same staff position
        let target = note_positions[i + 1..]
            .iter()
            .find(|&&(_, target_pos, _, _, _, _)| target_pos == pos);

        let Some(&(target_x, _, _, _, _, _)) = target else {
            continue;
        };

        let advance = notehead_advance(font, dur_log2, style)?;

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
    pub(crate) duration_log2: i8,
    pub(crate) notehead_style: NoteheadStyle,
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
                        notehead_style: annotation_notehead_style(&n.annotations, 0),
                        stem_direction: n.stem_direction,
                        slur_start: n.annotations.slur_start,
                        slur_end: n.annotations.slur_end,
                    });
                }
                MeasureElement::Chord(c) => {
                    let top_pos = c.staff_positions.iter().copied().max().unwrap_or(0);
                    let bot_pos = c.staff_positions.iter().copied().min().unwrap_or(0);
                    let dir = c
                        .stem_direction
                        .unwrap_or_else(|| auto_stem_direction(top_pos));
                    let attach_pos = match dir {
                        StemDirection::Up => bot_pos,
                        StemDirection::Down => top_pos,
                    };
                    notes.push(SlurNoteInfo {
                        x: elem_x,
                        staff_position: attach_pos,
                        duration_log2: c.duration_log2,
                        notehead_style: chord_notehead_style(c, attach_pos),
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
        let target = note_info[i + 1..].iter().find(|n| n.slur_end);

        let Some(target) = target else {
            continue;
        };

        let advance = notehead_advance(font, info.duration_log2, info.notehead_style)?;

        // Slur starts at right edge of first notehead, ends at left edge of last
        let slur_x_start = system_x + info.x + advance;
        let slur_x_end = system_x + target.x;

        // Determine slur direction from start note's stem
        let stem_dir = info
            .stem_direction
            .unwrap_or_else(|| auto_stem_direction(info.staff_position));
        let direction = slur_direction_from_stem(stem_dir);

        // Y-coordinates at attachment points
        let start_y = staff.y_of(info.staff_position);
        let end_y = staff.y_of(target.staff_position);

        let slur_layout = layout_slur(slur_x_start, slur_x_end, start_y, end_y, direction, config);
        draw_slur(svg, &slur_layout);
    }

    Ok(())
}

/// Positional info for a note relevant to hairpin drawing.
pub(crate) struct HairpinNoteInfo<'a> {
    pub(crate) x: f64,
    pub(crate) duration_log2: i8,
    pub(crate) notehead_styles: &'a [NoteheadStyle],
    pub(crate) notehead_count: usize,
    pub(crate) hairpin_start: Option<HairpinType>,
    pub(crate) hairpin_end: bool,
    /// Mirror of `NoteAnnotations::hairpin_dashed`. Set on the start side of
    /// a hairpin to request a dashed wedge; ignored on notes that aren't a
    /// hairpin start. `collect_hairpin_note_info` propagates this from both
    /// `MeasureElement::Note` and `MeasureElement::Chord` annotations so that
    /// `draw_system_hairpins` and the cross-system splitter both see it.
    pub(crate) hairpin_dashed: bool,
    /// Mirror of `NoteAnnotations::hairpin_niente`. When `Some`, the wedge
    /// starting at this note carries an open "o" circle at the tip selected
    /// by the [`NientePlacement`]. `None` on notes that aren't a hairpin
    /// start, or on plain hairpins.
    pub(crate) hairpin_niente: Option<NientePlacement>,
    /// Side of the staff of a hairpin starting here (mirror of
    /// `NoteAnnotations::dynamics_placement`).
    pub(crate) placement: Placement,
    /// Whether this endpoint is a rest (hairpins may start or end on rests).
    pub(crate) is_rest: bool,
    /// The event's dynamic, which a hairpin starting or ending here clears.
    pub(crate) dynamic: Option<&'a DynamicMark>,
}

impl HairpinNoteInfo<'_> {
    /// Horizontal extent (system-relative) of this event's dynamic when it
    /// sits on `side`.
    fn dynamic_extent(
        &self,
        font: &MusicFont,
        staff_space: f64,
        side: Placement,
    ) -> Result<Option<(f64, f64)>, FontError> {
        let Some(mark) = self.dynamic.filter(|_| self.placement == side) else {
            return Ok(None);
        };
        let advance = span_event_advance(
            font,
            self.duration_log2,
            self.notehead_styles,
            self.notehead_count,
            self.is_rest,
        )?;
        let layout = layout_dynamic_mark(mark, font, staff_space)?;
        let left = layout.left_for(self.x + advance / 2.0);
        Ok(Some((left, left + layout.line.width)))
    }

    /// System-relative x where a hairpin starting here begins: 0.3 ss right
    /// of the notehead or rest, and of a dynamic on the hairpin's side.
    pub(crate) fn hairpin_start_x(
        &self,
        font: &MusicFont,
        staff_space: f64,
    ) -> Result<f64, FontError> {
        let advance = span_event_advance(
            font,
            self.duration_log2,
            self.notehead_styles,
            self.notehead_count,
            self.is_rest,
        )?;
        let mut x = self.x + advance + 0.3 * staff_space;
        if let Some((_, right)) = self.dynamic_extent(font, staff_space, self.placement)? {
            x = x.max(right + 0.3 * staff_space);
        }
        Ok(x)
    }

    /// System-relative x where a hairpin on `side` ending here stops: 0.3 ss
    /// left of the event, and of a dynamic on that side.
    pub(crate) fn hairpin_end_x(
        &self,
        font: &MusicFont,
        staff_space: f64,
        side: Placement,
    ) -> Result<f64, FontError> {
        let mut x = self.x - 0.3 * staff_space;
        if let Some((left, _)) = self.dynamic_extent(font, staff_space, side)? {
            x = x.min(left - 0.3 * staff_space);
        }
        Ok(x)
    }
}

pub(crate) fn collect_hairpin_note_info(system: &SystemLayout) -> Vec<HairpinNoteInfo<'_>> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push(HairpinNoteInfo {
                        x: elem_x,
                        duration_log2: n.duration_log2,
                        notehead_styles: &n.annotations.notehead_styles,
                        notehead_count: 1,
                        hairpin_start: n.annotations.hairpin_start,
                        hairpin_end: n.annotations.hairpin_end,
                        hairpin_dashed: n.annotations.hairpin_dashed,
                        hairpin_niente: n.annotations.hairpin_niente,
                        placement: n.annotations.dynamics_placement,
                        is_rest: false,
                        dynamic: n.annotations.dynamic.as_ref(),
                    });
                }
                MeasureElement::Chord(c) => {
                    notes.push(HairpinNoteInfo {
                        x: elem_x,
                        duration_log2: c.duration_log2,
                        notehead_styles: &c.annotations.notehead_styles,
                        notehead_count: c.staff_positions.len(),
                        hairpin_start: c.annotations.hairpin_start,
                        hairpin_end: c.annotations.hairpin_end,
                        hairpin_dashed: c.annotations.hairpin_dashed,
                        hairpin_niente: c.annotations.hairpin_niente,
                        placement: c.annotations.dynamics_placement,
                        is_rest: false,
                        dynamic: c.annotations.dynamic.as_ref(),
                    });
                }
                MeasureElement::Rest(r) => {
                    notes.push(HairpinNoteInfo {
                        x: elem_x,
                        duration_log2: r.duration_log2,
                        notehead_styles: &[],
                        notehead_count: 1,
                        hairpin_start: r.annotations.hairpin_start,
                        hairpin_end: r.annotations.hairpin_end,
                        hairpin_dashed: r.annotations.hairpin_dashed,
                        hairpin_niente: r.annotations.hairpin_niente,
                        placement: r.annotations.dynamics_placement,
                        is_rest: true,
                        dynamic: r.annotations.dynamic.as_ref(),
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
        let target = note_info[i + 1..].iter().find(|n| n.hairpin_end);

        let Some(target) = target else {
            continue;
        };

        // The hairpin runs from right of the start event (and its dynamic)
        // to left of the target (and its dynamic).
        let hp_x_start = system_x + info.hairpin_start_x(font, config.staff_space)?;
        let hp_x_end = system_x + target.hairpin_end_x(font, config.staff_space, info.placement)?;

        // Use staff line thickness as hairpin stroke width
        let stroke_width = config.staff_line_thickness_fu();

        let hp_layout = layout_hairpin_styled(
            hairpin_type,
            hp_x_start,
            hp_x_end,
            hairpin_reference_y(info.placement, staff, config.staff_space),
            config.staff_space,
            stroke_width,
            info.hairpin_niente,
            info.hairpin_dashed,
        );
        draw_hairpin(svg, &hp_layout);
    }

    Ok(())
}

/// Positional info for an event relevant to text-spanner drawing.
///
/// Conceptually parallel to [`HairpinNoteInfo`]: spanners start and end on
/// notes, chords or rests.
pub(crate) struct TextSpannerNoteInfo<'a> {
    pub(crate) x: f64,
    pub(crate) duration_log2: i8,
    pub(crate) notehead_styles: &'a [NoteheadStyle],
    pub(crate) notehead_count: usize,
    pub(crate) is_rest: bool,
    pub(crate) start: Option<&'a TextSpanner>,
    pub(crate) end: bool,
}

pub(crate) fn collect_text_spanner_note_info(
    system: &SystemLayout,
) -> Vec<TextSpannerNoteInfo<'_>> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            let (duration_log2, notehead_styles, notehead_count, is_rest, annotations) =
                match &elem.element {
                    MeasureElement::Note(n) => (
                        n.duration_log2,
                        n.annotations.notehead_styles.as_slice(),
                        1,
                        false,
                        &n.annotations,
                    ),
                    MeasureElement::Chord(c) => (
                        c.duration_log2,
                        c.annotations.notehead_styles.as_slice(),
                        c.staff_positions.len(),
                        false,
                        &c.annotations,
                    ),
                    MeasureElement::Rest(r) => (r.duration_log2, &[][..], 1, true, &r.annotations),
                    _ => continue,
                };
            notes.push(TextSpannerNoteInfo {
                x: elem_x,
                duration_log2,
                notehead_styles,
                notehead_count,
                is_rest,
                start: annotations.text_spanner_start.as_ref(),
                end: annotations.text_spanner_end,
            });
        }
    }
    notes
}

/// Draw text spanners that start and end within this system.
///
/// The label starts just right of the start event and the line ends just
/// left of the end event, with the hairpin padding. Spanners whose end lies
/// on a later system are drawn by the page renderer's cross-system pass.
fn draw_system_text_spanners(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
    let note_info = collect_text_spanner_note_info(system);

    for (i, info) in note_info.iter().enumerate() {
        let Some(spanner) = info.start else {
            continue;
        };
        let Some(target) = note_info[i + 1..].iter().find(|n| n.end) else {
            continue;
        };
        let advance = span_event_advance(
            font,
            info.duration_log2,
            info.notehead_styles,
            info.notehead_count,
            info.is_rest,
        )?;
        let x_start = system_x + info.x + advance + 0.3 * config.staff_space;
        let x_end = system_x + target.x - 0.3 * config.staff_space;
        let layout = layout_text_spanner(spanner, x_start, x_end, staff, config.staff_space);
        draw_text_spanner(svg, &layout);
    }

    Ok(())
}

/// A pitched event and its lyrics, in system-local coordinates. Voice identity
/// is retained: interleaved secondary notes never steal another voice's spans.
pub(crate) struct LyricNoteInfo<'a> {
    pub(crate) x: f64,
    pub(crate) voice: u8,
    pub(crate) lyrics: &'a [VerseLyric],
}

impl LyricNoteInfo<'_> {
    pub(crate) fn verse(&self, verse: u16) -> Option<&VerseLyric> {
        self.lyrics
            .iter()
            .find(|lyric| lyric.verse == verse && !lyric.syllable.skip)
    }

    pub(crate) fn syllable(&self, verse: u16) -> Option<&LyricSyllable> {
        self.verse(verse).map(|lyric| &lyric.syllable)
    }
}

/// Collect notes/chords, including every beamed and tuplet member, from each
/// voice in temporal order. Group marks and rests do not become lyric anchors.
pub(crate) fn collect_lyric_note_info(
    system: &SystemLayout,
    staff_space: f64,
) -> Vec<LyricNoteInfo<'_>> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (voice, layout) in std::iter::once(&measure.layout)
            .chain(measure.additional_voice_layouts.iter())
            .enumerate()
        {
            for element in &layout.elements {
                let (lyrics, scale) = match &element.element {
                    MeasureElement::Note(n) => {
                        (&n.annotations.lyrics[..], n.annotations.size.scale())
                    }
                    MeasureElement::Chord(c) => {
                        (&c.annotations.lyrics[..], c.annotations.size.scale())
                    }
                    _ => continue,
                };
                notes.push(LyricNoteInfo {
                    x: measure.x_offset + element.x + 0.59 * staff_space * scale,
                    voice: voice as u8,
                    lyrics,
                });
            }
        }
    }
    notes
}

/// Draw each verse's melisma to its next syllable, or to the final pitched
/// member of its voice when it continues past this system. Explicit skips do
/// not stop a melisma.
fn draw_system_lyric_extenders(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) {
    let notes = collect_lyric_note_info(system, config.staff_space);
    for (i, info) in notes.iter().enumerate() {
        for lyric in info.lyrics {
            if lyric.syllable.skip || lyric.syllable.continuation != LyricContinuation::Extender {
                continue;
            }
            let target = notes[i + 1..]
                .iter()
                .filter(|n| n.voice == info.voice)
                .find(|n| n.syllable(lyric.verse).is_some())
                .or_else(|| notes[i + 1..].iter().rev().find(|n| n.voice == info.voice));
            if let Some(target) = target {
                draw_lyric_extender(
                    svg,
                    system_x + extender_source_x(info.x, lyric, config.staff_space),
                    system_x
                        + target.verse(lyric.verse).map_or(target.x, |next| {
                            extender_target_x(target.x, next, config.staff_space)
                        }),
                    verse_baseline(staff, config.staff_space, lyric.verse),
                    config.staff_space,
                    config.staff_line_thickness_fu(),
                );
            }
        }
    }
}

/// A visible hyphen is drawn only between real syllables of the same verse
/// and associated voice. Hidden hyphens retain their continuation semantics
/// without a visible glyph.
fn draw_system_lyric_hyphens(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) {
    let notes = collect_lyric_note_info(system, config.staff_space);
    for (i, info) in notes.iter().enumerate() {
        for lyric in info.lyrics {
            if lyric.syllable.skip || lyric.syllable.continuation != LyricContinuation::Hyphen {
                continue;
            }
            if let Some(target) = notes[i + 1..]
                .iter()
                .find(|n| n.voice == info.voice && n.syllable(lyric.verse).is_some())
            {
                let next = target.verse(lyric.verse).expect("target has a syllable");
                draw_lyric_hyphen_between(
                    svg,
                    system_x + info.x,
                    lyric,
                    system_x + target.x,
                    next,
                    verse_baseline(staff, config.staff_space, lyric.verse),
                    config.staff_space,
                );
            }
        }
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

        let bracket_layout =
            layout_volta_bracket(annotation, x_left, x_right, staff, config.staff_space);
        draw_volta_bracket(svg, &bracket_layout);
    }
}

/// Positional info for a note relevant to ottava bracket drawing.
pub(crate) struct OttavaNoteInfo<'a> {
    pub(crate) x: f64,
    pub(crate) duration_log2: i8,
    pub(crate) notehead_styles: &'a [NoteheadStyle],
    pub(crate) notehead_count: usize,
    pub(crate) ottava_start: Option<OttavaKind>,
    pub(crate) ottava_end: bool,
}

pub(crate) fn collect_ottava_note_info(system: &SystemLayout) -> Vec<OttavaNoteInfo<'_>> {
    let mut notes = Vec::new();
    for measure in &system.measures {
        for (elem_x, elem) in all_measure_elements(measure) {
            match &elem.element {
                MeasureElement::Note(n) => {
                    notes.push(OttavaNoteInfo {
                        x: elem_x,
                        duration_log2: n.duration_log2,
                        notehead_styles: &n.annotations.notehead_styles,
                        notehead_count: 1,
                        ottava_start: n.annotations.ottava_start,
                        ottava_end: n.annotations.ottava_end,
                    });
                }
                MeasureElement::Chord(c) => {
                    notes.push(OttavaNoteInfo {
                        x: elem_x,
                        duration_log2: c.duration_log2,
                        notehead_styles: &c.annotations.notehead_styles,
                        notehead_count: c.staff_positions.len(),
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
        let target = note_info[i + 1..].iter().find(|n| n.ottava_end);

        let Some(target) = target else {
            continue;
        };

        let target_advance = widest_notehead_advance(
            font,
            target.duration_log2,
            target.notehead_styles,
            target.notehead_count,
        )?;

        let ott_x_start = system_x + info.x;
        let ott_x_end = system_x + target.x + target_advance;

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
pub(crate) struct GlissandoNoteInfo<'a> {
    pub x: f64,
    pub staff_position: i8,
    pub stem_direction: Option<StemDirection>,
    pub glissando_start: Option<GlissandoStyle>,
    pub duration_log2: i8,
    pub notehead_styles: &'a [NoteheadStyle],
    pub notehead_count: usize,
}

impl GlissandoNoteInfo<'_> {
    /// Correct the historical 1.18ss head estimate only for glyphs wider
    /// than it (notably whole notes). Preserve existing short-head spacing.
    pub(crate) fn source_width_correction(
        &self,
        font: &MusicFont,
        ss: f64,
    ) -> Result<f64, FontError> {
        let advance = widest_notehead_advance(
            font,
            self.duration_log2,
            self.notehead_styles,
            self.notehead_count,
        )?;
        Ok((advance - ss * GLISSANDO_NOTEHEAD_WIDTH_SS).max(0.0))
    }
}

/// Collect note info relevant to glissando rendering from a system's elements.
pub(crate) fn collect_glissando_note_info(system: &SystemLayout) -> Vec<GlissandoNoteInfo<'_>> {
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
                        duration_log2: n.duration_log2,
                        notehead_styles: &n.annotations.notehead_styles,
                        notehead_count: 1,
                    });
                }
                MeasureElement::Chord(c) => {
                    let top_pos = c.staff_positions.iter().copied().max().unwrap_or(0);
                    let bot_pos = c.staff_positions.iter().copied().min().unwrap_or(0);
                    let dir = c
                        .stem_direction
                        .unwrap_or_else(|| auto_stem_direction(top_pos));
                    let attach_pos = match dir {
                        StemDirection::Up => bot_pos,
                        StemDirection::Down => top_pos,
                    };
                    notes.push(GlissandoNoteInfo {
                        x: elem_x,
                        staff_position: attach_pos,
                        stem_direction: c.stem_direction,
                        glissando_start: c.annotations.glissando_start,
                        duration_log2: c.duration_log2,
                        notehead_styles: &c.annotations.notehead_styles,
                        notehead_count: c.staff_positions.len(),
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
    font: &MusicFont,
    system: &SystemLayout,
    staff: &StaffLayout,
    system_x: f64,
) -> Result<(), FontError> {
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
            system_x + note.x + note.source_width_correction(font, staff.staff_space)?,
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
    Ok(())
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
    /// Optional explicit end-anchor expressed as an offset into the
    /// system's flat note sequence — the wiggle terminates at the note
    /// `n` positions past the trilled note (1 = next, 2 = two-after, ...).
    /// Filtered the same way as `bracket`: only carries through when
    /// `has_trill_extension == true`. `None` selects the default
    /// "extend to immediately following note" behavior. An offset that
    /// walks past the end of the system collapses to the system-edge
    /// fallback (same as a trilled last note). When both
    /// [`explicit_length_ss`](Self::explicit_length_ss) and this field
    /// are set, the explicit length wins at draw time — see the
    /// annotation field's docstring for the rationale.
    pub to_note_offset: Option<usize>,
    /// Optional multi-speed ramp spec for this trill. Filtered the same
    /// way as `bracket`: only carries through when
    /// `has_trill_extension == true`. `None` selects the single-speed
    /// renderer path (using `wiggle_speed`); `Some(spec)` engages the
    /// multi-speed renderer path — `wiggle_speed` is ignored for glyph
    /// selection in that case (the per-region glyphs come from the ramp).
    pub speed_ramp: Option<TrillSpeedRampSpec>,
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
                    let ornament = if has_ext {
                        n.annotations.ornament
                    } else {
                        None
                    };
                    let bracket = if has_ext {
                        n.annotations.trill_bracket
                    } else {
                        None
                    };
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
                    let to_note_offset = if has_ext {
                        n.annotations.trill_extension_to_note_offset
                    } else {
                        None
                    };
                    let speed_ramp = if has_ext {
                        n.annotations.trill_speed_ramp
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
                        to_note_offset,
                        speed_ramp,
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
                    let ornament = if has_ext {
                        c.annotations.ornament
                    } else {
                        None
                    };
                    let bracket = if has_ext {
                        c.annotations.trill_bracket
                    } else {
                        None
                    };
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
                    let to_note_offset = if has_ext {
                        c.annotations.trill_extension_to_note_offset
                    } else {
                        None
                    };
                    let speed_ramp = if has_ext {
                        c.annotations.trill_speed_ramp
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
                        to_note_offset,
                        speed_ramp,
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

        // End at the target note's left edge — by default the
        // immediately following note (offset = 1), or at the note
        // `to_note_offset` positions past the trilled note when the user
        // requested a specific anchor. If the trilled note is the last
        // note in its system (or the requested offset walks past the end
        // of the system), the wiggle extends instead to the right edge of
        // the system (just inside the final barline). The cross-system
        // convention only applies to the *natural* last-note case
        // (i.e. `i + 1` is past the end with no explicit offset); an
        // explicit offset that walks past the end terminates at the
        // system edge but does NOT propagate across the system break —
        // an offset is a definite anchor request, not a "let it flow"
        // signal.
        //
        // When the user supplied an explicit length, clamp the natural
        // end_x with `start_x + length_ss * staff_space`. The clamp is
        // one-sided (we only ever shorten, never extend past the natural
        // endpoint), so an explicit length larger than the natural span is
        // a no-op rather than an overrun. A positive explicit length also
        // disables cross-system propagation: the trill terminates within
        // this system at the requested point, regardless of position. The
        // explicit length wins over the to-note offset when both are set
        // (per the annotation field's documented precedence).
        let target_offset = note.to_note_offset.unwrap_or(1);
        let (natural_end_x, natural_cross_system) = if target_offset == 0 {
            // Zero offset is degenerate: target is the trilled note
            // itself. Collapse end_x to start_x so layout_trill_extension
            // suppresses the wiggle.
            (start_x, false)
        } else {
            match notes.get(i + target_offset) {
                Some(target) => (
                    system_x + target.x - TRILL_EXTENSION_NOTE_GAP_SS * staff_space,
                    false,
                ),
                None => {
                    let edge = system_x + system.staff_width
                        - TRILL_EXTENSION_SYSTEM_EDGE_GAP_SS * staff_space;
                    // Cross-system propagation is reserved for the
                    // *natural* last-note case (no explicit offset). An
                    // explicit offset is treated as a definite anchor.
                    let cross = note.to_note_offset.is_none() && notes.get(i + 1).is_none();
                    (edge, cross)
                }
            }
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
            None => (natural_end_x, natural_cross_system),
        };

        // Dispatch: multi-speed (ramp present) supersedes single-speed.
        // The single-speed branch uses `wiggle_speed`; the multi-speed
        // branch synthesizes per-region speeds from `speed_ramp` and
        // ignores `wiggle_speed` for glyph selection. Bracket-hook
        // rendering and cross-system propagation behave identically in
        // both branches.
        if let Some(spec) = note.speed_ramp {
            // Multi-speed path. The synthesizer is font-agnostic; we
            // thread the active font's per-glyph advance lookup through
            // the closure. `synthesize_regions` returns `None` for
            // degenerate specs (region_count==0, or Linear with
            // region_count==1) and for non-positive spans (start_x >=
            // end_x — happens when an explicit non-positive length
            // collapsed end_x to start_x). The fall-through to no wiggle
            // matches the single-speed renderer's fail-safe.
            let regions =
                match spec
                    .ramp
                    .synthesize_regions(start_x, end_x, spec.region_count, |speed| {
                        font.glyph_advance(speed.to_glyph()).unwrap_or(0) as f64
                    }) {
                    Some(r) => r,
                    None => continue,
                };
            if let Some(layout) =
                layout_trill_extension_multi_speed(end_x, ornament_layout.y, &regions)
            {
                draw_trill_extension_multi_speed(svg, font, &layout)?;

                // Brackets in multi-speed mode anchor to the leftmost
                // tile's left edge (Start) and the rightmost tile's right
                // edge (End). The multi-speed bracket helper reads from
                // `MultiSpeedTrillExtensionLayout` directly so we don't
                // need to fabricate a single-speed shim.
                if let Some(side) = note.bracket {
                    let render_side = bracket_side_for_system_pass(side, cross_system);
                    if let Some(side) = render_side {
                        let hook_length = note
                            .bracket_length_ss
                            .map(|ss| ss * staff.staff_space)
                            .unwrap_or(default_hook_length);
                        let direction = note.bracket_direction.unwrap_or(HookDirection::Down);
                        let hooks = layout_trill_bracket_hooks_multi_speed(
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
            continue;
        }

        // Single-speed path. Look up the wiggle glyph + advance for this
        // note's chosen speed. Each wiggleTrill* variant has its own
        // advance, so the lookup must travel with the glyph choice.
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
