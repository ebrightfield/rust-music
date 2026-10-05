//! One exact performed-onset grid for all voices and staves of a system.
//!
//! A column has the widest structural prefix, leading accidental/grace rod,
//! rhythmic right rod, and spring needed by *any* participant. Zero-duration
//! events keep voice order: structural glyphs precede sounding events at that
//! onset; beam/tuplet marks attach to the sounding column. Equal-onset glyphs
//! on different staves share the column, not each other's cumulative widths.

use std::collections::BTreeMap;

use crate::layout::group::GroupMark;
use crate::layout::line_break::written_length;
use crate::layout::measure::{
    trailing_padding, MeasureElement, MeasureLayout, MeasureLayoutConfig,
};
use crate::layout::measure_meta::MeasureLength;
use crate::layout::system::{MeasureContent, SystemLayout, SystemPrefix};

#[derive(Default)]
struct Column {
    prefix: f64,
    left: f64,
    right: f64,
    spring: f64,
    barline_offset: f64,
    x: f64,
}

struct Span {
    from: MeasureLength,
    to: MeasureLength,
    duration: MeasureLength,
    spring: f64,
}

#[derive(Clone, Copy)]
struct VoiceTiming {
    start: MeasureLength,
    first_onset: MeasureLength,
    end: MeasureLength,
}

struct GridPlacement<'a> {
    columns: &'a BTreeMap<MeasureLength, Column>,
    scale: f64,
    rod: f64,
    spring: f64,
}

fn length(element: &MeasureElement) -> MeasureLength {
    match element {
        MeasureElement::Note(n) => written_length(n.duration_log2, n.dots),
        MeasureElement::Rest(r) => written_length(r.duration_log2, r.dots),
        MeasureElement::Chord(c) => written_length(c.duration_log2, c.dots),
        MeasureElement::Spacer(s) => written_length(s.duration_log2, s.dots),
        _ => MeasureLength::ZERO,
    }
}

fn performed_length(written: MeasureLength, tuplets: &[(u32, u32)]) -> MeasureLength {
    tuplets
        .iter()
        .fold(written, |length, &(number, in_time_of)| {
            if number == 0 || in_time_of == 0 {
                length
            } else {
                MeasureLength::new(
                    length.numerator() * u64::from(in_time_of),
                    length.denominator() * u64::from(number),
                )
            }
        })
}

fn advance_tuplets(element: &MeasureElement, tuplets: &mut Vec<(u32, u32)>) {
    match element {
        MeasureElement::GroupMark(GroupMark::TupletStart { spec, .. }) => {
            tuplets.push((spec.number, spec.in_time_of));
        }
        MeasureElement::GroupMark(GroupMark::TupletEnd { .. }) => {
            tuplets.pop();
        }
        _ => {}
    }
}

/// Visit each event at its exact performed onset, reusing one tuplet stack per
/// voice. A closing barline belongs to the common measure end, not to the
/// length of its own (possibly empty or shorter) voice.
fn walk_voice(
    layout: &MeasureLayout,
    start: MeasureLength,
    first_onset: MeasureLength,
    end: MeasureLength,
    mut visit: impl FnMut(usize, MeasureLength, MeasureLength),
) {
    let mut onset = first_onset;
    let mut tuplets = Vec::new();
    let closing = layout
        .elements
        .iter()
        .rposition(|element| matches!(element.element, MeasureElement::Barline(_)));
    let mut leading_prefix = true;
    for (index, positioned) in layout.elements.iter().enumerate() {
        let element = &positioned.element;
        let resumes = matches!(
            element,
            MeasureElement::GroupMark(
                GroupMark::BeamStart {
                    continued: true,
                    ..
                } | GroupMark::TupletStart {
                    continued: true,
                    ..
                }
            )
        );
        let prefix = leading_prefix
            && matches!(
                element,
                MeasureElement::Clef(_)
                    | MeasureElement::KeySignature(_)
                    | MeasureElement::TimeSignature(_)
            );
        let at = if Some(index) == closing {
            end
        } else if resumes || prefix {
            start
        } else {
            onset
        };
        if is_rhythmic(element) {
            leading_prefix = false;
        }
        let duration = performed_length(length(element), &tuplets);
        visit(index, at, duration);
        advance_tuplets(element, &mut tuplets);
        onset = if Some(index) == closing {
            end
        } else {
            onset + duration
        };
    }
}

fn voice_end(layout: &MeasureLayout, first_onset: MeasureLength) -> MeasureLength {
    let mut end = first_onset;
    walk_voice(
        layout,
        first_onset,
        first_onset,
        first_onset,
        |_, _, duration| end = end + duration,
    );
    end
}

fn is_rhythmic(element: &MeasureElement) -> bool {
    matches!(
        element,
        MeasureElement::Note(_)
            | MeasureElement::Chord(_)
            | MeasureElement::Rest(_)
            | MeasureElement::Spacer(_)
            | MeasureElement::MultiMeasureRest { .. }
    )
}

fn structural_before(
    layout: &MeasureLayout,
    config: &MeasureLayoutConfig,
    timing: VoiceTiming,
    columns: &mut BTreeMap<MeasureLength, Column>,
    spans: &mut Vec<Span>,
    spring_factor: f64,
) {
    let VoiceTiming { start, first_onset, end } = timing;
    let mut current_tick = start;
    let mut prefix = 0.0;
    let mut previous_end = 0.0;
    let mut previous_trailing = 0.0;
    walk_voice(layout, start, first_onset, end, |index, tick, duration| {
        let element = &layout.elements[index];
        if tick != current_tick {
            current_tick = tick;
            prefix = 0.0;
        }
        let leading = (element.x - previous_end - previous_trailing).max(0.0);
        previous_end = element.x + element.width;
        previous_trailing = trailing_padding(&element.element, config);
        let col = columns.entry(tick).or_default();
        if is_rhythmic(&element.element) {
            col.prefix = col.prefix.max(prefix);
            col.left = col.left.max(prefix + leading);
            col.right = col.right.max(element.rod);
            if duration != MeasureLength::ZERO && tick < end {
                spans.push(Span {
                    from: tick,
                    to: (tick + duration).min(end),
                    duration,
                    spring: element.spring * spring_factor,
                });
            }
        } else if matches!(element.element, MeasureElement::GroupMark(_)) {
            // A span mark has no rod; its anchor will be the shared note x.
        } else {
            prefix += leading;
            if matches!(element.element, MeasureElement::Barline(_)) {
                col.barline_offset = col.barline_offset.max(prefix);
            }
            prefix += element.rod + previous_trailing;
            col.prefix = col.prefix.max(prefix);
        }
    });
}

fn position_voice(
    layout: &mut MeasureLayout,
    config: &MeasureLayoutConfig,
    timing: VoiceTiming,
    placement: &GridPlacement<'_>,
) {
    let VoiceTiming { start, first_onset, end } = timing;
    let columns = placement.columns;
    let scale = placement.scale;
    let total_rod = placement.rod;
    let total_spring = placement.spring;
    let mut tick = start;
    let mut onset = first_onset;
    let mut tuplets = Vec::new();
    let closing = layout
        .elements
        .iter()
        .rposition(|element| matches!(element.element, MeasureElement::Barline(_)));
    let mut prefix = 0.0;
    let mut previous_end = 0.0;
    let mut previous_trailing = 0.0;
    let mut leading_prefix = true;
    for (index, positioned) in layout.elements.iter_mut().enumerate() {
        let resumes = matches!(
            positioned.element,
            MeasureElement::GroupMark(
                GroupMark::BeamStart {
                    continued: true,
                    ..
                } | GroupMark::TupletStart {
                    continued: true,
                    ..
                }
            )
        );
        let leading_structural = leading_prefix
            && matches!(
                positioned.element,
                MeasureElement::Clef(_)
                    | MeasureElement::KeySignature(_)
                    | MeasureElement::TimeSignature(_)
            );
        let at = if Some(index) == closing {
            end
        } else if resumes || leading_structural {
            start
        } else {
            onset
        };
        if is_rhythmic(&positioned.element) {
            leading_prefix = false;
        }
        let duration = performed_length(length(&positioned.element), &tuplets);
        advance_tuplets(&positioned.element, &mut tuplets);
        onset = if Some(index) == closing {
            end
        } else {
            onset + duration
        };
        if at != tick {
            tick = at;
            prefix = 0.0;
        }
        let leading = (positioned.x - previous_end - previous_trailing).max(0.0);
        previous_end = positioned.x + positioned.width;
        previous_trailing = trailing_padding(&positioned.element, config);
        let col = &columns[&at];
        if is_rhythmic(&positioned.element) {
            positioned.x = col.x + col.prefix.max(col.left);
            positioned.spring = col.spring * scale;
            positioned.rod = col.right;
            positioned.width = positioned.rod + positioned.spring;
        } else if matches!(positioned.element, MeasureElement::GroupMark(_)) {
            positioned.x = col.x + col.prefix.max(col.left);
        } else if matches!(positioned.element, MeasureElement::Barline(_)) {
            prefix += leading;
            positioned.x = col.x + col.barline_offset;
            prefix += positioned.rod + previous_trailing;
        } else {
            prefix += leading;
            positioned.x = col.x + prefix;
            prefix += positioned.rod + previous_trailing;
        }
    }
    layout.total_rod = total_rod;
    layout.total_spring = total_spring;
    layout.total_width = total_rod + total_spring;
}

/// Build every visual measure's grid *once* across all participating stave
/// layouts and their voices, then justify common springs once for the system.
/// Staves lacking a measure or voice still inherit its exact shared boundary.
/// `staves` and `systems` have identical order and measure slices.
pub(crate) fn align_shared_grid(
    staves: &[(&SystemPrefix, &[MeasureContent], Option<&MeasureContent>)],
    systems: &mut [SystemLayout],
    config: &MeasureLayoutConfig,
    target_width: Option<f64>,
) {
    let count = systems.iter().map(|s| s.measures.len()).max().unwrap_or(0);
    let mut grids = Vec::with_capacity(count);
    let mut system_rod = 0.0;
    let mut system_spring = 0.0;
    for i in 0..count {
        let mut end = MeasureLength::ZERO;
        let mut start = MeasureLength::ZERO;
        for (staff, (_, contents, next)) in systems.iter().zip(staves) {
            if let (Some(measure), Some(content)) = (staff.measures.get(i), contents.get(i)) {
                start = content.meta.visual_start;
                let candidate = contents.get(i + 1).or(*next);
                if let Some(next_piece) = candidate.filter(|next_piece| {
                    next_piece.meta.continuation && next_piece.meta.number == content.meta.number
                }) {
                    end = end.max(next_piece.meta.visual_start);
                } else {
                    let first = content
                        .meta
                        .visual_voice_onsets
                        .first()
                        .copied()
                        .unwrap_or(start);
                    end = end.max(voice_end(&measure.layout, first));
                    for (index, voice) in measure.additional_voice_layouts.iter().enumerate() {
                        let first = content
                            .meta
                            .visual_voice_onsets
                            .get(index + 1)
                            .copied()
                            .unwrap_or(start);
                        end = end.max(voice_end(voice, first));
                    }
                    if measure.layout.elements.iter().any(|element| {
                        matches!(element.element, MeasureElement::MultiMeasureRest { .. })
                    }) {
                        // A multi-measure rest has no ordinary rhythmic member,
                        // but its H-bar needs a column before the closing barline.
                        let visual_length = content
                            .meta
                            .nominal_length
                            .or((content.meta.actual_length != MeasureLength::ZERO)
                                .then_some(content.meta.actual_length))
                            .unwrap_or(MeasureLength::new(1, 1));
                        end = end.max(start + visual_length);
                    }
                }
            }
        }
        let shortest = systems
            .iter()
            .filter_map(|s| s.measures.get(i))
            .flat_map(|m| std::iter::once(&m.layout).chain(m.additional_voice_layouts.iter()))
            .flat_map(|layout| layout.elements.iter())
            .filter_map(|el| match &el.element {
                MeasureElement::Note(n) => Some(n.duration_log2),
                MeasureElement::Chord(c) => Some(c.duration_log2),
                MeasureElement::Rest(r) => Some(r.duration_log2),
                MeasureElement::Spacer(s) => Some(s.duration_log2),
                _ => None,
            })
            .max()
            .unwrap_or(2);
        let mut columns = BTreeMap::new();
        columns.insert(start, Column::default());
        columns.insert(end, Column::default());
        let mut spans = Vec::new();
        for (system, (_, contents, _)) in systems.iter().zip(staves) {
            let Some((measure, content)) = system.measures.get(i).zip(contents.get(i)) else {
                continue;
            };
            for (index, layout) in std::iter::once(&measure.layout)
                .chain(measure.additional_voice_layouts.iter())
                .enumerate()
            {
                let local_shortest = layout
                    .elements
                    .iter()
                    .filter_map(|el| match &el.element {
                        MeasureElement::Note(n) => Some(n.duration_log2),
                        MeasureElement::Chord(c) => Some(c.duration_log2),
                        MeasureElement::Rest(r) => Some(r.duration_log2),
                        MeasureElement::Spacer(s) => Some(s.duration_log2),
                        _ => None,
                    })
                    .max()
                    .unwrap_or(2);
                let factor =
                    (2.0_f64).powf(f64::from(shortest - local_shortest) * config.spacing_exponent);
                let first = content
                    .meta
                    .visual_voice_onsets
                    .get(index)
                    .copied()
                    .unwrap_or(start);
                structural_before(
                    layout,
                    config,
                    VoiceTiming { start, first_onset: first, end },
                    &mut columns,
                    &mut spans,
                    factor,
                );
            }
        }
        let has_multi_rest = systems
            .iter()
            .filter_map(|system| system.measures.get(i))
            .any(|measure| {
                measure.layout.elements.iter().any(|element| {
                    matches!(element.element, MeasureElement::MultiMeasureRest { .. })
                })
            });
        let ticks: Vec<_> = columns.keys().copied().collect();
        for pair in ticks.windows(2) {
            let (at, next) = (pair[0], pair[1]);
            let mut spring = 0.0_f64;
            for span in &spans {
                if span.from <= at && span.to > at {
                    let stop = next.min(span.to);
                    let interval = stop.as_f64() - at.as_f64();
                    spring = spring.max(span.spring * interval / span.duration.as_f64());
                }
            }
            if spring == 0.0 && !has_multi_rest {
                // Every voice may be sustaining an attack from before a
                // mid-measure split. Keep the continuation interval as a
                // shared spring instead of collapsing elapsed time.
                let relative = (next.as_f64() - at.as_f64()) * 2.0_f64.powi(i32::from(shortest));
                spring = config.spring_constant * relative.powf(config.spacing_exponent);
            }
            columns.get_mut(&at).expect("tick exists").spring = spring;
        }
        let rod: f64 = columns
            .values()
            .map(|c| c.prefix.max(c.left) + c.right)
            .sum();
        let spring: f64 = columns.values().map(|c| c.spring).sum();
        system_rod += rod;
        system_spring += spring;
        grids.push((columns, start, end, rod, spring));
    }
    let scale = match target_width {
        Some(width) if system_spring > 0.0 => ((width - system_rod) / system_spring).max(0.0),
        _ => 1.0,
    };
    let mut offset = 0.0;
    for (i, (mut columns, start, end, rod, spring)) in grids.into_iter().enumerate() {
        let mut x = 0.0;
        for col in columns.values_mut() {
            col.x = x;
            x += col.prefix.max(col.left) + col.right + col.spring * scale;
        }
        let placement = GridPlacement {
            columns: &columns,
            scale,
            rod,
            spring: spring * scale,
        };
        for (system, (_, contents, _)) in systems.iter_mut().zip(staves) {
            if let Some((measure, content)) = system.measures.get_mut(i).zip(contents.get(i)) {
                measure.shared_closing_barline_x = columns[&end].x + columns[&end].barline_offset;
                measure.x_offset = offset;
                let first = content
                    .meta
                    .visual_voice_onsets
                    .first()
                    .copied()
                    .unwrap_or(start);
                position_voice(
                    &mut measure.layout,
                    config,
                    VoiceTiming { start, first_onset: first, end },
                    &placement,
                );
                for (index, layout) in measure.additional_voice_layouts.iter_mut().enumerate() {
                    let first = content
                        .meta
                        .visual_voice_onsets
                        .get(index + 1)
                        .copied()
                        .unwrap_or(start);
                    position_voice(
                        layout,
                        config,
                        VoiceTiming { start, first_onset: first, end },
                        &placement,
                    );
                }
            }
        }
        offset += x;
    }
    for system in systems {
        system.total_width = offset;
        system.staff_width = target_width.unwrap_or(offset).max(offset);
    }
}

#[cfg(test)]
#[path = "rhythm_grid_tests.rs"]
mod tests;
