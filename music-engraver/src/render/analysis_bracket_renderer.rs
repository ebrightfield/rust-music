//! One bracket collector for ordinary event anchors, independent of grouping
//! and barline elements. System and page passes share the same segment drawing.

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::analysis_bracket::{
    layout_analysis_bracket, AnalysisBracketLayout, AnalysisBracketSpec, AnalysisBracketStyle,
};
use crate::layout::measure::MeasureElement;
use crate::layout::page::PageSystem;
use crate::layout::placement::Placement;
use crate::layout::staff::StaffLayout;
use crate::layout::system::SystemLayout;
use crate::render::svg_writer::TextStyle;
use crate::render::system_renderer::span_event_advance;
use crate::render::SvgWriter;

#[derive(Clone)]
struct Anchor {
    x: f64,
    right: f64,
    top: f64,
    bottom: f64,
    start: Option<AnalysisBracketSpec>,
    end: bool,
}

fn has_bracket_start(system: &SystemLayout) -> bool {
    system.measures.iter().any(|measure| {
        std::iter::once(&measure.layout)
            .chain(measure.additional_voice_layouts.iter())
            .flat_map(|layout| layout.elements.iter())
            .any(|element| match &element.element {
                MeasureElement::Note(n) => n.annotations.analysis_bracket_start.is_some(),
                MeasureElement::Chord(c) => c.annotations.analysis_bracket_start.is_some(),
                MeasureElement::Rest(r) => r.annotations.analysis_bracket_start.is_some(),
                _ => false,
            })
    })
}

/// Collect event positions separately per voice, preserving measure/event order.
/// Structural GroupMarks, meter changes, and visible or invisible barlines
/// consume no anchor, so spans pass over them unchanged.
fn anchors(
    system: &SystemLayout,
    font: &MusicFont,
    ss: f64,
) -> Result<Vec<Vec<Anchor>>, FontError> {
    let voice_count = system
        .measures
        .iter()
        .map(|m| m.additional_voice_layouts.len() + 1)
        .max()
        .unwrap_or(0);
    let mut voices = vec![Vec::new(); voice_count];
    for measure in &system.measures {
        for (voice, output) in voices.iter_mut().enumerate() {
            let layout = if voice == 0 {
                Some(&measure.layout)
            } else {
                measure.additional_voice_layouts.get(voice - 1)
            };
            let Some(layout) = layout else {
                continue;
            };
            for e in &layout.elements {
                let x = measure.x_offset + e.x;
                let (positions, stem, duration, styles, count, start, end, rest) = match &e.element
                {
                    MeasureElement::Note(n) => (
                        Some((n.staff_position, n.staff_position)),
                        (n.duration_log2 >= 1
                            && n.annotations.stem
                                == crate::layout::measure::StemVisibility::Visible)
                            .then(|| {
                                n.stem_direction.unwrap_or_else(|| {
                                    crate::layout::stem::auto_stem_direction(n.staff_position)
                                })
                            }),
                        n.duration_log2,
                        &n.annotations.notehead_styles[..],
                        1,
                        &n.annotations.analysis_bracket_start,
                        n.annotations.analysis_bracket_end,
                        false,
                    ),
                    MeasureElement::Chord(c) => (
                        Some((
                            *c.staff_positions.iter().max().unwrap_or(&4),
                            *c.staff_positions.iter().min().unwrap_or(&4),
                        )),
                        (c.duration_log2 >= 1
                            && c.annotations.stem
                                == crate::layout::measure::StemVisibility::Visible)
                            .then(|| {
                                c.stem_direction.unwrap_or_else(|| {
                                    crate::layout::stem::auto_stem_direction_chord(
                                        &c.staff_positions,
                                    )
                                })
                            }),
                        c.duration_log2,
                        &c.annotations.notehead_styles[..],
                        c.staff_positions.len(),
                        &c.annotations.analysis_bracket_start,
                        c.annotations.analysis_bracket_end,
                        false,
                    ),
                    MeasureElement::Rest(r) => (
                        None,
                        None,
                        r.duration_log2,
                        &[][..],
                        0,
                        &r.annotations.analysis_bracket_start,
                        r.annotations.analysis_bracket_end,
                        true,
                    ),
                    _ => continue,
                };
                let width = span_event_advance(font, duration, styles, count, rest)?;
                let (top, bottom) = if let Some((high, low)) = positions {
                    let mut top = -(high as f64) * ss * 0.5 - ss * 0.4;
                    let mut bottom = -(low as f64) * ss * 0.5 + ss * 0.4;
                    // A stem (including its attached beam) may extend 3.5 spaces.
                    match stem {
                        Some(crate::layout::stem::StemDirection::Up) => top -= ss * 3.5,
                        Some(crate::layout::stem::StemDirection::Down) => bottom += ss * 3.5,
                        None => {}
                    }
                    (top, bottom)
                } else {
                    (-ss * 2.4, -ss * 1.6)
                };
                output.push(Anchor {
                    x,
                    right: x + width,
                    top,
                    bottom,
                    start: start.clone(),
                    end,
                });
            }
        }
    }
    Ok(voices)
}

/// Draw the actual horizontal stroke, terminal hooks and optional first label.
pub fn draw_analysis_bracket(svg: &mut SvgWriter, layout: &AnalysisBracketLayout) {
    match layout.style {
        AnalysisBracketStyle::Solid => svg.add_line(
            layout.x_start,
            layout.y,
            layout.x_end,
            layout.y,
            "black",
            layout.stroke_width,
        ),
        AnalysisBracketStyle::Dashed => {
            let ss = layout.stroke_width / 0.08;
            let dash = format!("{},{}", ss * 0.35, ss * 0.23);
            svg.add_dashed_line(
                layout.x_start,
                layout.y,
                layout.x_end,
                layout.y,
                "black",
                layout.stroke_width,
                &dash,
            );
        }
    }
    let hook = match layout.placement {
        Placement::Above => layout.hook_length,
        Placement::Below => -layout.hook_length,
    };
    if layout.start_hook {
        svg.add_line(
            layout.x_start,
            layout.y,
            layout.x_start,
            layout.y + hook,
            "black",
            layout.stroke_width,
        );
    }
    if layout.end_hook {
        svg.add_line(
            layout.x_end,
            layout.y,
            layout.x_end,
            layout.y + hook,
            "black",
            layout.stroke_width,
        );
    }
    if let Some(label) = &layout.label {
        let ss = layout.stroke_width / 0.08;
        let text_y = layout.y
            + if layout.placement == Placement::Above {
                -ss * 0.25
            } else {
                ss * 0.9
            };
        let style = TextStyle {
            anchor: "start",
            ..TextStyle::normal(ss * 0.9)
        };
        svg.add_text(layout.x_start, text_y, label, &style);
    }
}

#[derive(Clone)]
struct Open {
    spec: AnalysisBracketSpec,
    system_index: usize,
    anchor_index: usize,
}

struct Segment {
    system_index: usize,
    voice: usize,
    start: usize,
    end: usize,
    open: Open,
    cross_system: bool,
    first: bool,
    last: bool,
}

/// Resolve all bracket pairs in score order, then paint only segments owned
/// by this pass. A bracket crossing N systems remains one logical open span.
fn draw_segments(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    systems: &[(&SystemLayout, f64, f64)],
    cross_system: bool,
) -> Result<(), FontError> {
    let anchors_by_system: Vec<_> = systems
        .iter()
        .map(|(s, _, _)| anchors(s, font, config.staff_space))
        .collect::<Result<_, _>>()?;
    let voice_count = anchors_by_system.iter().map(Vec::len).max().unwrap_or(0);
    let mut segments = Vec::new();
    for voice in 0..voice_count {
        let mut open: Vec<Open> = Vec::new();
        let mut pending: Vec<Segment> = Vec::new();
        for (sys, voices) in anchors_by_system.iter().enumerate() {
            let Some(events) = voices.get(voice) else {
                continue;
            };
            for (idx, event) in events.iter().enumerate() {
                if event.end {
                    if let Some(bracket) = open.pop() {
                        let crossed = bracket.system_index != sys;
                        if crossed {
                            for (s, piece) in anchors_by_system
                                .iter()
                                .enumerate()
                                .take(sys + 1)
                                .skip(bracket.system_index)
                            {
                                let len = piece.get(voice).map_or(0, Vec::len);
                                if len == 0 {
                                    continue;
                                }
                                segments.push(Segment {
                                    system_index: s,
                                    voice,
                                    start: if s == bracket.system_index {
                                        bracket.anchor_index
                                    } else {
                                        0
                                    },
                                    end: if s == sys { idx } else { len - 1 },
                                    open: bracket.clone(),
                                    cross_system: true,
                                    first: s == bracket.system_index,
                                    last: s == sys,
                                });
                            }
                        } else {
                            pending.push(Segment {
                                system_index: sys,
                                voice,
                                start: bracket.anchor_index,
                                end: idx,
                                open: bracket,
                                cross_system: false,
                                first: true,
                                last: true,
                            });
                        }
                    }
                }
                if let Some(spec) = &event.start {
                    open.push(Open {
                        spec: spec.clone(),
                        system_index: sys,
                        anchor_index: idx,
                    });
                }
            }
        }
        segments.extend(pending);
    }
    // Occupied horizontal lanes on each system, separately per above/below.
    let mut occupied: Vec<Vec<(f64, f64, Placement, usize)>> = vec![Vec::new(); systems.len()];
    for segment in segments {
        let sys = segment.system_index;
        let (_, x, y) = systems[sys];
        let events = &anchors_by_system[sys][segment.voice];
        let ss = config.staff_space;
        let left = if segment.first {
            x + events[segment.start].x
        } else {
            x + events[0].x - ss * 1.4
        };
        let right = if segment.last {
            x + events[segment.end].right
        } else {
            x + systems[sys].0.staff_width
        };
        let top = events[segment.start..=segment.end]
            .iter()
            .map(|e| e.top)
            .fold(0.0_f64, f64::min)
            + y
            + ss * 4.0;
        let bottom = events[segment.start..=segment.end]
            .iter()
            .map(|e| e.bottom)
            .fold(0.0_f64, f64::max)
            + y
            + ss * 4.0;
        let lane = (0..)
            .find(|lane| {
                !occupied[sys].iter().any(|(a, b, side, n)| {
                    *side == segment.open.spec.placement && *n == *lane && left < *b && right > *a
                })
            })
            .unwrap();
        let staff = StaffLayout::new(x, y, systems[sys].0.staff_width, ss);
        if let Some(layout) = layout_analysis_bracket(
            &segment.open.spec,
            left,
            right,
            staff.y_of(8),
            staff.y_of(0),
            top,
            bottom,
            ss,
            lane,
            segment.first,
            segment.last,
            segment.first,
        ) {
            if segment.cross_system == cross_system {
                draw_analysis_bracket(svg, &layout);
            }
            occupied[sys].push((left, right, segment.open.spec.placement, lane));
        }
    }
    Ok(())
}

pub(crate) fn draw_system_analysis_brackets(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    system: &SystemLayout,
    x: f64,
    y: f64,
) -> Result<(), FontError> {
    if !has_bracket_start(system) {
        return Ok(());
    }
    draw_segments(svg, font, config, &[(system, x, y)], false)
}

pub(crate) fn draw_cross_system_analysis_brackets(
    svg: &mut SvgWriter,
    font: &MusicFont,
    config: &EngravingConfig,
    systems: &[PageSystem],
) -> Result<(), FontError> {
    if !systems.iter().any(|s| has_bracket_start(&s.system)) {
        return Ok(());
    }
    let refs: Vec<_> = systems.iter().map(|s| (&s.system, s.x, s.y)).collect();
    draw_segments(svg, font, config, &refs, true)
}
