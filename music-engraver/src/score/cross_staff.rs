//! Route a single musical timeline through independent notation staves.
//! Each destination receives exactly one sounding event or an invisible spacer
//! at each onset; the existing shared rhythm grid owns horizontal alignment.

use std::collections::HashMap;

use crate::font::{FontError, MusicFont};
use crate::layout::glissando::{
    layout_glissando_between_staves, layout_half_glissando_left, layout_half_glissando_right,
    GlissandoStyle,
};
use crate::layout::measure::{MeasureElement, MeasureLayout};
use crate::layout::page::PageSystem;
use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;
use crate::layout::system::SystemLayout;
use crate::render::glissando_renderer::draw_glissando;
use crate::render::system_renderer::widest_notehead_advance;
use crate::render::SvgWriter;

use super::{CrossStaffError, ScoreBuilder};
use crate::score::{event::ScoreEvent, CompletedMeasure};

#[derive(Clone, Copy, Debug)]
pub(super) struct CrossStaffGlissando {
    pub from: usize,
    pub to: usize,
    pub style: GlissandoStyle,
}

/// Move each event once from the voice to its assigned stave. The other
/// staves receive only duration-matched invisible spacers, including in
/// measures with an independently entered voice on those staves.
pub(super) fn distribute_voice(
    mut voice: ScoreBuilder,
    staves: &mut [ScoreBuilder],
) -> Result<Vec<CrossStaffGlissando>, CrossStaffError> {
    let mut spans = Vec::new();
    let mut last_note = None;
    let mut next_id = 0;
    let staff_count = staves.len();
    for stave in staves.iter_mut() {
        if stave.time_signature.is_none() {
            stave.time_signature.clone_from(&voice.time_signature);
        }
    }
    for (measure_index, mut measure) in voice.measures.drain(..).enumerate() {
        let mut assigned = Vec::with_capacity(staff_count);
        for stave in staves.iter_mut() {
            if stave.measures.len() <= measure_index {
                stave.measures.push(CompletedMeasure {
                    events: Vec::new(),
                    barline: measure.barline,
                    volta: measure.volta.clone(),
                    timing: measure.timing.clone(),
                });
            }
            let existing = &mut stave.measures[measure_index];
            let voice_index = existing
                .events
                .iter()
                .map(|(voice, _)| *voice)
                .max()
                .map_or(Some(0), |n| n.checked_add(1))
                .ok_or(CrossStaffError::UnsupportedEvent {
                    event: next_id,
                    kind: "no free voice index on destination stave",
                })?;
            existing.barline = measure.barline;
            existing.timing.cadenza |= measure.timing.cadenza;
            assigned.push((voice_index, Vec::new()));
            if voice.accidental_policy == crate::layout::accidental::AccidentalPolicy::Forget {
                stave.accidental_policy = voice.accidental_policy;
            }
        }
        for (source_voice, mut event) in measure.events.drain(..) {
            if source_voice != 0 {
                return Err(CrossStaffError::InvalidVoice {
                    event: next_id,
                    voice: source_voice,
                });
            }
            let destination = match &mut event {
                ScoreEvent::Note { annotations, .. }
                | ScoreEvent::Chord { annotations, .. }
                | ScoreEvent::Rest { annotations, .. }
                | ScoreEvent::Spacer { annotations, .. } => annotations.on_staff.unwrap_or(0),
                ScoreEvent::Barline(_) | ScoreEvent::TimeSignatureChange(_) => 0,
                ScoreEvent::ClefChange(_) => {
                    return Err(CrossStaffError::UnsupportedEvent {
                        event: next_id,
                        kind: "clef change (set clef on destination stave)",
                    })
                }
                ScoreEvent::GroupMark(_) => {
                    return Err(CrossStaffError::UnsupportedEvent {
                        event: next_id,
                        kind: "beam/tuplet spanning stave changes",
                    })
                }
                ScoreEvent::MultiMeasureRest { .. } => {
                    return Err(CrossStaffError::UnsupportedEvent {
                        event: next_id,
                        kind: "multi-measure rest",
                    })
                }
            };
            if destination >= staff_count {
                return Err(CrossStaffError::InvalidStaff {
                    event: next_id,
                    staff: destination,
                    staff_count,
                });
            }
            if let ScoreEvent::Note { annotations, .. }
            | ScoreEvent::Chord { annotations, .. }
            | ScoreEvent::Rest { annotations, .. }
            | ScoreEvent::Spacer { annotations, .. } = &mut event
            {
                if !voice.leading_text_marks.is_empty() {
                    let mut marks = std::mem::take(&mut voice.leading_text_marks);
                    marks.append(&mut annotations.text_scripts);
                    annotations.text_scripts = marks;
                }
                if voice.stemless {
                    annotations.stem = crate::layout::measure::StemVisibility::Hidden;
                }
            }
            let pitched = match &mut event {
                ScoreEvent::Note { annotations, .. } | ScoreEvent::Chord { annotations, .. } => {
                    annotations.cross_staff_id = Some(next_id);
                    true
                }
                _ => false,
            };
            if pitched {
                if let Some((from, style)) = pending_glissando(&mut last_note) {
                    spans.push(CrossStaffGlissando {
                        from,
                        to: next_id,
                        style,
                    });
                }
                let style = match &mut event {
                    ScoreEvent::Note { annotations, .. }
                    | ScoreEvent::Chord { annotations, .. } => annotations.glissando_start.take(),
                    _ => None,
                };
                last_note = Some((next_id, style));
            }
            let duration = match &event {
                ScoreEvent::Note { duration, .. }
                | ScoreEvent::Chord { duration, .. }
                | ScoreEvent::Rest { duration, .. } => Some(*duration),
                ScoreEvent::Spacer { duration, .. } => *duration,
                _ => None,
            };
            let replicate = matches!(
                event,
                ScoreEvent::Barline(_) | ScoreEvent::TimeSignatureChange(_)
            );
            for (staff, (voice_index, events)) in assigned.iter_mut().enumerate() {
                if staff == destination {
                    // Fill other staves first; move the real event into its one destination below.
                    continue;
                }
                if let Some(duration) = duration {
                    events.push((
                        *voice_index,
                        ScoreEvent::Spacer {
                            duration: Some(duration),
                            annotations: Default::default(),
                        },
                    ));
                } else if replicate {
                    events.push((*voice_index, event.clone()));
                }
            }
            let (voice_index, events) = &mut assigned[destination];
            events.push((*voice_index, event));
            next_id += 1;
        }
        for (stave, (_, events)) in staves.iter_mut().zip(assigned) {
            stave.measures[measure_index].events.extend(events);
        }
    }
    Ok(spans)
}

fn pending_glissando(
    previous: &mut Option<(usize, Option<GlissandoStyle>)>,
) -> Option<(usize, GlissandoStyle)> {
    previous
        .as_mut()
        .and_then(|(id, style)| style.take().map(|style| (*id, style)))
}

#[derive(Clone, Copy)]
pub(super) struct CrossStaffAnchor {
    system: usize,
    stave: usize,
    x: f64,
    y: f64,
    staff_position: i8,
    head_width: f64,
}

pub(super) fn collect_anchors(
    font: &MusicFont,
    ss: f64,
    system_index: usize,
    systems: &[SystemLayout],
    y_origins: &[f64],
    x: f64,
    anchors: &mut HashMap<usize, CrossStaffAnchor>,
) -> Result<(), FontError> {
    for (stave, system) in systems.iter().enumerate() {
        for measure in &system.measures {
            let staff = StaffLayout::new(
                x + measure.x_offset,
                y_origins[stave],
                system.staff_width,
                ss,
            );
            for voice_layout in
                std::iter::once(&measure.layout).chain(&measure.additional_voice_layouts)
            {
                collect_measure_anchors(font, system_index, stave, &staff, voice_layout, anchors)?;
            }
        }
    }
    Ok(())
}

fn collect_measure_anchors(
    font: &MusicFont,
    system: usize,
    stave: usize,
    staff: &StaffLayout,
    layout: &MeasureLayout,
    anchors: &mut HashMap<usize, CrossStaffAnchor>,
) -> Result<(), FontError> {
    for element in &layout.elements {
        let (id, pos, log2, styles, count) = match &element.element {
            MeasureElement::Note(note) => (
                note.annotations.cross_staff_id,
                note.staff_position,
                note.duration_log2,
                &note.annotations.notehead_styles,
                1,
            ),
            MeasureElement::Chord(chord) => {
                let top = chord.staff_positions.iter().copied().max().unwrap_or(0);
                let bottom = chord.staff_positions.iter().copied().min().unwrap_or(0);
                let pos = if chord.stem_direction == Some(StemDirection::Down) {
                    top
                } else {
                    bottom
                };
                (
                    chord.annotations.cross_staff_id,
                    pos,
                    chord.duration_log2,
                    &chord.annotations.notehead_styles,
                    chord.staff_positions.len(),
                )
            }
            _ => continue,
        };
        if let Some(id) = id {
            anchors.insert(
                id,
                CrossStaffAnchor {
                    system,
                    stave,
                    x: staff.x + element.x,
                    y: staff.y_origin,
                    staff_position: pos,
                    head_width: widest_notehead_advance(font, log2, styles, count)?
                        .max(staff.staff_space * 1.18),
                },
            );
        }
    }
    Ok(())
}

/// Conservative outer-stave ink reach for a routed voice. Whole notes with
/// flats can rise well above the treble clef; low ledger notes can extend past
/// the bass staff. This clearance is used only for page bounds, not spacing.
pub(super) fn outer_note_extent_ss(system: &SystemLayout) -> (f64, f64) {
    let mut above = 0.0_f64;
    let mut below = 0.0_f64;
    for measure in &system.measures {
        for layout in std::iter::once(&measure.layout).chain(&measure.additional_voice_layouts) {
            for element in &layout.elements {
                let (positions, has_accidental, stem_up, stem_down) = match &element.element {
                    MeasureElement::Note(note) => (
                        std::slice::from_ref(&note.staff_position),
                        note.accidental.is_some(),
                        note.duration_log2 > 0
                            && note.stem_direction.unwrap_or_else(|| {
                                crate::layout::stem::auto_stem_direction(note.staff_position)
                            }) == StemDirection::Up,
                        note.duration_log2 > 0
                            && note.stem_direction.unwrap_or_else(|| {
                                crate::layout::stem::auto_stem_direction(note.staff_position)
                            }) == StemDirection::Down,
                    ),
                    MeasureElement::Chord(chord) => (
                        chord.staff_positions.as_slice(),
                        chord.accidentals.iter().any(Option::is_some),
                        chord.duration_log2 > 0
                            && chord.stem_direction.unwrap_or_else(|| {
                                crate::layout::stem::auto_stem_direction_chord(
                                    &chord.staff_positions,
                                )
                            }) == StemDirection::Up,
                        chord.duration_log2 > 0
                            && chord.stem_direction.unwrap_or_else(|| {
                                crate::layout::stem::auto_stem_direction_chord(
                                    &chord.staff_positions,
                                )
                            }) == StemDirection::Down,
                    ),
                    _ => continue,
                };
                for &position in positions {
                    let above_head = (f64::from(position) - 8.0) / 2.0;
                    let below_head = -f64::from(position) / 2.0;
                    above = above.max(above_head + if has_accidental { 2.0 } else { 0.8 });
                    below = below.max(below_head + 0.8);
                    if stem_up {
                        above = above.max(above_head + 3.7);
                    }
                    if stem_down {
                        below = below.max(below_head + 3.7);
                    }
                }
            }
        }
    }
    (above, below)
}

pub(super) fn draw_cross_staff_glissandos(
    svg: &mut SvgWriter,
    ss: f64,
    glissandos: &[CrossStaffGlissando],
    anchors: &HashMap<usize, CrossStaffAnchor>,
    systems: &[Vec<PageSystem>],
) {
    for span in glissandos {
        let (Some(from), Some(to)) = (anchors.get(&span.from), anchors.get(&span.to)) else {
            continue;
        };
        let source_staff = StaffLayout::new(
            systems[from.stave][from.system].x,
            from.y,
            systems[from.stave][from.system].system.staff_width,
            ss,
        );
        let target_staff = StaffLayout::new(
            systems[to.stave][to.system].x,
            to.y,
            systems[to.stave][to.system].system.staff_width,
            ss,
        );
        if from.system == to.system {
            if let Some(line) = layout_glissando_between_staves(
                from.x + from.head_width - ss * 1.18,
                from.staff_position,
                to.x,
                to.staff_position,
                &source_staff,
                &target_staff,
                span.style,
            ) {
                draw_glissando(svg, &line);
            }
        } else {
            let right = source_staff.x + source_staff.width;
            if let Some(mut line) = layout_half_glissando_right(
                from.x + from.head_width - ss * 1.18,
                from.staff_position,
                right,
                &source_staff,
                span.style,
            ) {
                line.y_end = line.y_start
                    + (target_staff.y_of(to.staff_position)
                        - source_staff.y_of(from.staff_position))
                    .clamp(-ss * 2.0, ss * 2.0);
                draw_glissando(svg, &line);
            }
            // Never run an incoming fragment through the receiving system's
            // clef, key signature, meter, or left-hand connector/label.
            let prefix_right = systems[to.stave][to.system]
                .system
                .measures
                .first()
                .map(|m| {
                    m.layout
                        .elements
                        .iter()
                        .take_while(|e| {
                            matches!(
                                e.element,
                                MeasureElement::Clef(_)
                                    | MeasureElement::KeySignature(_)
                                    | MeasureElement::TimeSignature(_)
                            )
                        })
                        .map(|e| target_staff.x + m.x_offset + e.x + e.width)
                        .fold(target_staff.x, f64::max)
                })
                .unwrap_or(target_staff.x);
            let left = (to.x - ss * 2.2).max(prefix_right + ss * 0.2);
            if let Some(mut line) =
                layout_half_glissando_left(left, to.x, to.staff_position, &target_staff, span.style)
            {
                line.y_start = line.y_end
                    - (target_staff.y_of(to.staff_position)
                        - source_staff.y_of(from.staff_position))
                    .clamp(-ss * 2.0, ss * 2.0);
                draw_glissando(svg, &line);
            }
        }
    }
}
