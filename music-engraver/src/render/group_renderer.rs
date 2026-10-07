//! Beam and tuplet span rendering.
//!
//! Span members are ordinary note, chord, and rest elements drawn by the
//! measure renderer (beamed members without their own stem and flag). This
//! module draws what spans add on top: the stems and beams of every beam span
//! and the bracket and number of every tuplet span, for any sequence of
//! elements — one measure, or one voice across a whole system, where spans
//! crossing a barline join into one beam or bracket. A span still open at the
//! start or end of the sequence (it continues from or onto another system) is
//! drawn as a broken piece: beams run past the outer stem and the bracket
//! loses its hook on that side.

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::beam::{
    beam_level, layout_beam_group_scaled, orient_fractional_beams, subdivide_beam_counts,
    BeamedNote,
};
use crate::layout::chord::{layout_chord_noteheads, notehead_x_offset, ChordNote};
use crate::layout::group::{
    scan_groups, BeamSpec, GroupScan, GroupSegment, TupletBracketVisibility, TupletNumberDisplay,
    TupletSpec,
};
use crate::layout::measure::{MeasureElement, NoteheadStyle, StemVisibility};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::{auto_stem_direction, auto_stem_direction_chord, StemDirection};
use crate::layout::tuplet::{
    layout_tuplet_bracket_at, tuplet_number_glyphs, tuplet_placement_from_stem,
    tuplet_ratio_glyphs, TupletPlacement,
};
use crate::render::beam_renderer::draw_beam_group_with_styles;
use crate::render::note_renderer::notehead_advance;
use crate::render::stem_renderer::{stem_endpoints, stem_x};
use crate::render::tuplet_renderer::draw_tuplet_bracket;
use crate::render::SvgWriter;

/// How far a broken beam or tuplet bracket runs past its outermost member
/// toward the system break, in staff spaces.
const BROKEN_SPAN_EXTENSION_SS: f64 = 1.5;

/// Clearance between a tuplet bracket (or lone number) and the noteheads,
/// stems, and beams it frames, in staff spaces.
const TUPLET_CLEARANCE_SS: f64 = 1.0;

/// Distance between nested tuplet brackets on the same side, in staff spaces.
const NESTED_TUPLET_GAP_SS: f64 = 1.5;

/// One positioned element: `x` is the rhythmic column and `ink_x` is where
/// its noteheads and stems are engraved after cross-voice collision avoidance.
/// Tuplets and rhythmic subdivisions must continue to use `x`.
pub struct GroupItem<'a> {
    pub x: f64,
    pub ink_x: f64,
    pub element: &'a MeasureElement,
}

/// Stem geometry of one beamed note or chord.
struct MemberStem {
    /// Index into the drawn items.
    item: usize,
    /// X of the notehead the stem attaches to.
    attach_x: f64,
    /// Advance of that notehead.
    attach_advance: f64,
    /// Staff position of the notehead farthest from the beam.
    attach_position: i8,
    /// Staff position of the notehead nearest the beam.
    beam_side_position: i8,
    duration_log2: i8,
    scale: f64,
    visible: bool,
}

/// Geometry of a drawn beam that tuplet brackets must clear.
struct RenderedBeam {
    direction: StemDirection,
    /// Absolute stem-tip y per stemmed member, keyed by item index.
    tips: Vec<(usize, f64)>,
}

/// Draw the stems and beams of every beam span and the bracket and number of
/// every tuplet span among `items`, in left-to-right order.
pub fn draw_groups(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    items: &[GroupItem<'_>],
) -> Result<(), FontError> {
    let scan = scan_groups(items.iter().map(|item| item.element));
    if scan.beams.is_empty() && scan.tuplets.is_empty() {
        return Ok(());
    }
    let mut beams = Vec::with_capacity(scan.beams.len());
    for segment in &scan.beams {
        beams.push(draw_beam(svg, staff, font, config, items, &scan, segment)?);
    }
    // Inner tuplets first, so enclosing brackets can clear them.
    let mut order: Vec<usize> = (0..scan.tuplets.len()).collect();
    order.sort_by_key(|&index| std::cmp::Reverse(scan.tuplets[index].depth));
    let mut drawn_brackets = Vec::new();
    for index in order {
        draw_tuplet(
            svg,
            staff,
            font,
            config,
            items,
            &scan,
            &scan.tuplets[index],
            &beams,
            &mut drawn_brackets,
        )?;
    }
    Ok(())
}

fn style_at(styles: &[NoteheadStyle], index: usize) -> NoteheadStyle {
    styles.get(index).copied().unwrap_or_default()
}

/// The stem direction the measure renderer gives a note or chord element.
fn element_stem_direction(element: &MeasureElement) -> Option<StemDirection> {
    match element {
        MeasureElement::Note(note) => Some(
            note.stem_direction
                .unwrap_or_else(|| auto_stem_direction(note.staff_position)),
        ),
        MeasureElement::Chord(chord) if !chord.staff_positions.is_empty() => Some(
            chord
                .stem_direction
                .unwrap_or_else(|| auto_stem_direction_chord(&chord.staff_positions)),
        ),
        _ => None,
    }
}

fn member_stem(
    font: &MusicFont,
    item: usize,
    x: f64,
    element: &MeasureElement,
    direction: StemDirection,
) -> Result<Option<MemberStem>, FontError> {
    match element {
        MeasureElement::Note(note) => Ok(Some(MemberStem {
            item,
            attach_x: x,
            attach_advance: notehead_advance(
                font,
                note.duration_log2,
                style_at(&note.annotations.notehead_styles, 0),
            )? * note.annotations.size.scale(),
            attach_position: note.staff_position,
            beam_side_position: note.staff_position,
            duration_log2: note.duration_log2,
            scale: note.annotations.size.scale(),
            visible: note.annotations.stem == StemVisibility::Visible,
        })),
        MeasureElement::Chord(chord) if !chord.staff_positions.is_empty() => {
            // Mirror the chord renderer's notehead columns so the beam stem
            // attaches where the chord's own stem would.
            let chord_notes: Vec<ChordNote> = chord
                .staff_positions
                .iter()
                .enumerate()
                .map(|(index, &staff_position)| ChordNote {
                    staff_position,
                    accidental: chord.accidentals.get(index).copied().flatten(),
                    notehead_style: style_at(&chord.annotations.notehead_styles, index),
                    parenthesized: chord
                        .annotations
                        .parenthesized_noteheads
                        .get(index)
                        .copied()
                        .unwrap_or(false),
                })
                .collect();
            let layouts = layout_chord_noteheads(&chord_notes, direction);
            let scale = chord.annotations.size.scale();
            let widest = layouts.iter().try_fold(0.0_f64, |widest, layout| {
                notehead_advance(font, chord.duration_log2, layout.notehead_style)
                    .map(|advance| widest.max(advance * scale))
            })?;
            let lowest = layouts.iter().map(|layout| layout.staff_position).min();
            let highest = layouts.iter().map(|layout| layout.staff_position).max();
            let (Some(lowest), Some(highest)) = (lowest, highest) else {
                return Ok(None);
            };
            let (attach_position, beam_side_position) = match direction {
                StemDirection::Up => (lowest, highest),
                StemDirection::Down => (highest, lowest),
            };
            let attach = layouts
                .iter()
                .find(|layout| layout.staff_position == attach_position)
                .expect("the attachment position comes from the chord's own noteheads");
            Ok(Some(MemberStem {
                item,
                attach_x: x + notehead_x_offset(attach.offset, direction) * widest,
                attach_advance: notehead_advance(font, chord.duration_log2, attach.notehead_style)?
                    * scale,
                attach_position,
                beam_side_position,
                duration_log2: chord.duration_log2,
                scale,
                visible: chord.annotations.stem == StemVisibility::Visible,
            }))
        }
        _ => Ok(None),
    }
}

fn draw_beam(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    items: &[GroupItem<'_>],
    scan: &GroupScan,
    segment: &GroupSegment<BeamSpec>,
) -> Result<Option<RenderedBeam>, FontError> {
    // Members carry their beam's resolved direction (score conversion); the
    // farthest-from-the-middle rule covers hand-built layouts without one.
    let explicit = segment
        .members
        .iter()
        .find_map(|&item| match items[item].element {
            MeasureElement::Note(note) => note.stem_direction,
            MeasureElement::Chord(chord) => chord.stem_direction,
            _ => None,
        });
    let direction = explicit.unwrap_or_else(|| {
        let positions: Vec<i8> = segment
            .members
            .iter()
            .flat_map(|&item| match items[item].element {
                MeasureElement::Note(note) => vec![note.staff_position],
                MeasureElement::Chord(chord) => chord.staff_positions.clone(),
                _ => Vec::new(),
            })
            .collect();
        auto_stem_direction_chord(&positions)
    });

    let mut stems = Vec::with_capacity(segment.members.len());
    for &item in &segment.members {
        let GroupItem {
            ink_x: x, element, ..
        } = items[item];
        if let Some(stem) = member_stem(font, item, x, element, direction)? {
            stems.push(stem);
        }
    }
    // No beam can attach to a span of rests or wholly stemless members.
    if !stems.iter().any(|stem| stem.visible) {
        return Ok(None);
    }

    let thickness = config.stem_thickness_fu();
    for stem in &stems {
        // Chord stems run from the far notehead to the beam-side notehead;
        // the beam renderer continues them to the beam.
        if stem.visible && stem.attach_position != stem.beam_side_position {
            let thickness = config.stem_thickness_fu() * stem.scale;
            let sx = stem_x(stem.attach_x, stem.attach_advance, direction, thickness);
            svg.add_line(
                sx,
                staff.y_of(stem.attach_position),
                sx,
                staff.y_of(stem.beam_side_position),
                "black",
                thickness,
            );
        }
    }

    let notes: Vec<BeamedNote> = stems
        .iter()
        .map(|stem| BeamedNote {
            x: stem.attach_x,
            staff_position: stem.beam_side_position,
            duration_log2: stem.duration_log2,
        })
        .collect();
    let advances: Vec<f64> = stems.iter().map(|stem| stem.attach_advance).collect();
    let mut layout =
        layout_beam_group_scaled(&notes, direction, staff.staff_space, |i| stems[i].scale);
    if let Some(interval_log2) = segment.spec.subdivide_log2 {
        let onsets: Vec<f64> = stems.iter().map(|stem| scan.onsets[stem.item]).collect();
        subdivide_beam_counts(
            &notes,
            &onsets,
            interval_log2,
            &mut layout.beams_left,
            &mut layout.beams_right,
        );
    }
    orient_fractional_beams(
        &notes,
        |i| scan.onsets[stems[i].item],
        &mut layout.beams_left,
        &mut layout.beams_right,
    );
    // A broken side keeps only the beams that connect inward; the
    // continuation is drawn as an extension below instead of a stub.
    let last = notes.len() - 1;
    if segment.open_start {
        layout.beams_left[0] = 0;
        if last > 0 {
            layout.beams_right[0] = layout.beams_right[0].min(layout.beams_left[1].max(1));
        }
    }
    if segment.open_end {
        layout.beams_right[last] = 0;
        if last > 0 {
            layout.beams_left[last] =
                layout.beams_left[last].min(layout.beams_right[last - 1].max(1));
        }
    }
    draw_beam_group_with_styles(
        svg,
        staff,
        config,
        &notes,
        &layout,
        &advances,
        |i| stems[i].scale,
        |i| stems[i].visible,
    );

    let tips: Vec<f64> = layout
        .stem_tip_ys
        .iter()
        .map(|tip| tip + staff.y_origin)
        .collect();
    let stem_xs: Vec<f64> = notes
        .iter()
        .zip(&advances)
        .zip(&stems)
        .map(|((note, &advance), stem)| stem_x(note.x, advance, direction, thickness * stem.scale))
        .collect();
    let slope = if last > 0 && (stem_xs[last] - stem_xs[0]).abs() > f64::EPSILON {
        (tips[last] - tips[0]) / (stem_xs[last] - stem_xs[0])
    } else {
        0.0
    };
    let extension = BROKEN_SPAN_EXTENSION_SS * staff.staff_space;
    for (open, index, sign) in [(segment.open_start, 0, -1.0), (segment.open_end, last, 1.0)] {
        if open {
            draw_beam_extension(
                svg,
                config,
                direction,
                stem_xs[index],
                tips[index],
                slope,
                sign * extension,
                beam_level(notes[index].duration_log2).max(1),
            );
        }
    }

    Ok(Some(RenderedBeam {
        direction,
        tips: stems.iter().map(|stem| stem.item).zip(tips).collect(),
    }))
}

/// Beams of a broken beam continuing `length` (signed: negative runs left)
/// past the outer stem at `stem_x`, following the beam's slope.
#[allow(clippy::too_many_arguments)]
fn draw_beam_extension(
    svg: &mut SvgWriter,
    config: &EngravingConfig,
    direction: StemDirection,
    stem_x: f64,
    tip_y: f64,
    slope: f64,
    length: f64,
    levels: u8,
) {
    let thick = config.beam_thickness_fu();
    let step = thick + config.beam_spacing_fu();
    // Beams stack from the tip toward the noteheads.
    let sign = match direction {
        StemDirection::Up => 1.0,
        StemDirection::Down => -1.0,
    };
    let far_x = stem_x + length;
    for level in 0..levels {
        let near_y = tip_y + f64::from(level) * step * sign;
        let far_y = near_y + slope * length;
        svg.add_polygon(
            &[
                (stem_x, near_y),
                (far_x, far_y),
                (far_x, far_y + thick * sign),
                (stem_x, near_y + thick * sign),
            ],
            "black",
        );
    }
}

/// Right edge of an element's notehead (or rest) column, from its x.
fn element_right_x(font: &MusicFont, x: f64, element: &MeasureElement) -> Result<f64, FontError> {
    let advance = match element {
        MeasureElement::Note(note) => {
            notehead_advance(
                font,
                note.duration_log2,
                style_at(&note.annotations.notehead_styles, 0),
            )? * note.annotations.size.scale()
        }
        MeasureElement::Chord(chord) => {
            (0..chord.staff_positions.len().max(1)).try_fold(0.0_f64, |widest, index| {
                notehead_advance(
                    font,
                    chord.duration_log2,
                    style_at(&chord.annotations.notehead_styles, index),
                )
                .map(|advance| widest.max(advance * chord.annotations.size.scale()))
            })?
        }
        _ => notehead_advance(font, 2, NoteheadStyle::Normal)?,
    };
    Ok(x + advance)
}

/// Outermost y a member occupies on the bracket's side (smaller is higher).
fn member_extent(
    staff: &StaffLayout,
    element: &MeasureElement,
    beam_tip: Option<f64>,
    placement: TupletPlacement,
) -> f64 {
    let half_space = staff.staff_space / 2.0;
    let (lowest, highest, duration_log2, visible): (i8, i8, i8, bool) = match element {
        MeasureElement::Note(note) => (
            note.staff_position,
            note.staff_position,
            note.duration_log2,
            note.annotations.stem == StemVisibility::Visible,
        ),
        MeasureElement::Chord(chord) => {
            let (Some(&lowest), Some(&highest)) = (
                chord.staff_positions.iter().min(),
                chord.staff_positions.iter().max(),
            ) else {
                return staff.y_of(4);
            };
            (
                lowest,
                highest,
                chord.duration_log2,
                chord.annotations.stem == StemVisibility::Visible,
            )
        }
        // A rest's glyph stays within the staff's middle; bound it there.
        _ => {
            return match placement {
                TupletPlacement::Above => staff.y_of(7),
                TupletPlacement::Below => staff.y_of(1),
            };
        }
    };
    let direction = element_stem_direction(element);
    let mut extent = match placement {
        TupletPlacement::Above => staff.y_of(highest) - half_space,
        TupletPlacement::Below => staff.y_of(lowest) + half_space,
    };
    let stem_on_bracket_side = matches!(
        (direction, placement),
        (Some(StemDirection::Up), TupletPlacement::Above)
            | (Some(StemDirection::Down), TupletPlacement::Below)
    );
    if visible && stem_on_bracket_side && duration_log2 >= 1 {
        let tip = beam_tip.unwrap_or_else(|| {
            let direction = direction.expect("stem side implies a direction");
            let far = match direction {
                StemDirection::Up => highest,
                StemDirection::Down => lowest,
            };
            let (top, bottom) = stem_endpoints(
                staff,
                far,
                direction,
                match element {
                    MeasureElement::Note(note) => note.annotations.size.scale(),
                    MeasureElement::Chord(chord) => chord.annotations.size.scale(),
                    _ => 1.0,
                },
            );
            match direction {
                StemDirection::Up => top,
                StemDirection::Down => bottom,
            }
        });
        extent = match placement {
            TupletPlacement::Above => extent.min(tip),
            TupletPlacement::Below => extent.max(tip),
        };
    }
    extent
}

/// A drawn tuplet bracket: x span, line y, and side.
type DrawnBracket = (f64, f64, f64, TupletPlacement);

#[allow(clippy::too_many_arguments)]
fn draw_tuplet(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    items: &[GroupItem<'_>],
    scan: &GroupScan,
    segment: &GroupSegment<TupletSpec>,
    beams: &[Option<RenderedBeam>],
    drawn: &mut Vec<DrawnBracket>,
) -> Result<(), FontError> {
    let (Some(&first), Some(&last)) = (segment.members.first(), segment.members.last()) else {
        return Ok(());
    };
    let spec = segment.spec;
    let stemmed: Vec<usize> = segment
        .members
        .iter()
        .copied()
        .filter(|&item| element_stem_direction(items[item].element).is_some())
        .collect();

    // LilyPond's `if-no-beam`: one beam joining the first and last stemmed
    // members makes the bracket redundant.
    let covering_beam = match (stemmed.first(), stemmed.last()) {
        (Some(&first_stem), Some(&last_stem)) => scan.beam_of[first_stem]
            .filter(|&beam| scan.beam_of[last_stem] == Some(beam))
            .and_then(|beam| beams[beam].as_ref()),
        _ => None,
    };
    let show_bracket = match spec.bracket {
        TupletBracketVisibility::Always => true,
        TupletBracketVisibility::Never => false,
        TupletBracketVisibility::Auto => covering_beam.is_none(),
    };
    let glyphs = match spec.number_display {
        TupletNumberDisplay::Number => tuplet_number_glyphs(spec.number),
        TupletNumberDisplay::Ratio => tuplet_ratio_glyphs(spec.number, spec.in_time_of),
        TupletNumberDisplay::Hidden => Vec::new(),
    };
    if !show_bracket && glyphs.is_empty() {
        return Ok(());
    }

    // Stem side of the members: the covering beam's, else the majority.
    let placement = spec.placement.unwrap_or_else(|| match covering_beam {
        Some(beam) => tuplet_placement_from_stem(beam.direction),
        None => {
            let up = stemmed
                .iter()
                .filter(|&&item| {
                    element_stem_direction(items[item].element) == Some(StemDirection::Up)
                })
                .count();
            if up * 2 >= stemmed.len() {
                TupletPlacement::Above
            } else {
                TupletPlacement::Below
            }
        }
    });

    let extension = BROKEN_SPAN_EXTENSION_SS * staff.staff_space;
    let mut x_left = items[first].x;
    let mut x_right = element_right_x(font, items[last].x, items[last].element)?;
    if segment.open_start {
        x_left -= extension;
    }
    if segment.open_end {
        x_right += extension;
    }

    let beam_tip = |item: usize| {
        scan.beam_of[item]
            .and_then(|beam| beams[beam].as_ref())
            .and_then(|beam| {
                beam.tips
                    .iter()
                    .find(|&&(tip_item, _)| tip_item == item)
                    .map(|&(_, tip)| tip)
            })
    };
    let extents = segment
        .members
        .iter()
        .map(|&item| member_extent(staff, items[item].element, beam_tip(item), placement));
    let clearance = TUPLET_CLEARANCE_SS * staff.staff_space;
    let mut bracket_y = match placement {
        TupletPlacement::Above => extents.fold(f64::INFINITY, f64::min) - clearance,
        TupletPlacement::Below => extents.fold(f64::NEG_INFINITY, f64::max) + clearance,
    };
    let nested_gap = NESTED_TUPLET_GAP_SS * staff.staff_space;
    for &(left, right, y, side) in drawn.iter() {
        if side == placement && left < x_right && right > x_left {
            bracket_y = match placement {
                TupletPlacement::Above => bracket_y.min(y - nested_gap),
                TupletPlacement::Below => bracket_y.max(y + nested_gap),
            };
        }
    }

    let number_width: f64 = glyphs
        .iter()
        .map(|&glyph| f64::from(font.glyph_advance(glyph).unwrap_or(0)))
        .sum();
    let mut layout = layout_tuplet_bracket_at(
        x_left,
        x_right,
        bracket_y,
        placement,
        glyphs,
        number_width,
        staff.staff_space,
        config.tuplet_bracket_thickness,
    );
    layout.show_bracket = show_bracket;
    layout.left_hook = !segment.open_start;
    layout.right_hook = !segment.open_end;
    draw_tuplet_bracket(svg, &layout, font, 0.0, 0.0);
    drawn.push((x_left, x_right, bracket_y, placement));
    Ok(())
}
