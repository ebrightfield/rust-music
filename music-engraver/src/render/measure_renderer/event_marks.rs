//! Marks attached to one note, chord or rest: articulations, dynamics,
//! tempo and rehearsal marks, text scripts, lyrics, chord symbols,
//! ornaments, navigation signs, pedal and breath marks.
//!
//! One code path serves every event kind so rests get exactly the marks
//! notes get. Marks on one side of the staff stack outward from the event's
//! noteheads or rest glyph and its articulations in LilyPond's
//! outside-staff order — dynamics, then text scripts, then the tempo mark —
//! each keeping its conventional default position unless the marks inside
//! it push it further out.

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::articulation::{
    layout_articulation_stack, layout_chord_articulation_stack, ArticulationPlacement,
};
use crate::layout::dynamics::{layout_dynamic_mark, DYNAMICS_ABOVE_STAFF_SS, DYNAMICS_BELOW_STAFF_SS};
use crate::layout::lyric::layout_lyric;
use crate::layout::measure::{NoteAnnotations, RestEvent};
use crate::layout::rest::{rest_glyph, rest_staff_position, rest_y};
use crate::layout::placement::Placement;
use crate::layout::rehearsal::layout_rehearsal_mark;
use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;
use crate::layout::tempo::{layout_tempo_mark, tempo_baseline};
use crate::layout::text_script::{
    PlacedLineItem, TextAlign, TextLineLayout, TextScript, TEXT_SCRIPT_BELOW_STAFF_SS,
    TEXT_SCRIPT_PADDING_SS, TEXT_SCRIPT_STACK_GAP_SS,
};
use crate::render::articulation_renderer::draw_articulation;
use crate::render::dot_renderer::draw_dots;
use crate::render::lyric_renderer::draw_lyric;
use crate::render::rehearsal_renderer::draw_rehearsal_mark;
use crate::render::rest_renderer::draw_rest_displaced;
use crate::render::tempo_renderer::draw_tempo_mark;
use crate::render::text_line_renderer::draw_text_line;
use crate::render::{SvgWriter, TextStyle};

/// The footprint of an event that marks attach to.
pub(super) struct EventAnchor {
    /// Left edge of the notehead column or rest glyph.
    pub(super) left_x: f64,
    /// Width of the notehead column or rest glyph.
    pub(super) width: f64,
    /// Top of the event's noteheads or rest glyph (SVG y-down: the smallest
    /// y). Stems are not included: dynamics and text keep their bands
    /// beside stems, as in the established layout, and only move out for
    /// ledger-line noteheads and articulations.
    pub(super) top_y: f64,
    /// Bottom of the event's noteheads or rest glyph (the largest y).
    pub(super) bottom_y: f64,
    /// Staff position articulations attach to.
    pub(super) articulation_position: i8,
    /// Lowest and highest staff positions for a chord; other events use one
    /// articulation anchor on either side.
    pub(super) chord_positions: Option<(i8, i8)>,
    /// Stem direction steering articulation placement (stem-opposite side).
    pub(super) stem: StemDirection,
    /// Staff position ornaments are placed above.
    pub(super) ornament_position: i8,
}

impl EventAnchor {
    fn center_x(&self) -> f64 {
        self.left_x + self.width / 2.0
    }
}

/// Vertical cursors of the two stacks: the y the next mark's near edge may
/// reach on each side of the staff.
struct Stacks {
    above: f64,
    below: f64,
    gap: f64,
}

impl Stacks {
    /// Baseline for a line of `ascent`/`descent` on `placement`'s side whose
    /// default baseline is `default`, advancing that side's cursor past it.
    fn place(
        &mut self,
        placement: Placement,
        default: f64,
        ascent: f64,
        descent: f64,
    ) -> f64 {
        match placement {
            Placement::Above => {
                let baseline = default.min(self.above - descent);
                self.above = baseline - ascent - self.gap;
                baseline
            }
            Placement::Below => {
                let baseline = default.max(self.below + ascent);
                self.below = baseline + descent + self.gap;
                baseline
            }
        }
    }
}

/// Draw a text line aligned on an anchor spanning `left..left + width`.
///
/// A line that is a single text run is emitted with the matching SVG
/// `text-anchor`, so centered and right-aligned words are exact rather than
/// placed from an estimated width.
pub(crate) fn draw_aligned_text_line(
    svg: &mut SvgWriter,
    font: &MusicFont,
    line: &TextLineLayout,
    align: TextAlign,
    left: f64,
    width: f64,
    baseline: f64,
) -> Result<(), FontError> {
    if let [PlacedLineItem::Text {
        text,
        font: text_font,
        font_size,
        ..
    }] = line.items.as_slice()
    {
        let (x, anchor) = match align {
            TextAlign::Left => (left, "start"),
            TextAlign::Center => (left + width / 2.0, "middle"),
            TextAlign::Right => (left + width, "end"),
        };
        let style = TextStyle {
            font_family: "serif",
            font_size: *font_size,
            fill: "black",
            anchor,
            font_weight: text_font.svg_weight(),
            font_style: text_font.svg_style(),
            dominant_baseline: "auto",
        };
        svg.add_text(x, baseline, text, &style);
        return Ok(());
    }
    draw_text_line(svg, font, line, line.left_for(align, left, width), baseline)
}

/// Default baseline of a text script on `placement`'s side of `staff`.
fn text_script_default_baseline(
    script: &TextScript,
    line: &TextLineLayout,
    staff: &StaffLayout,
    staff_space: f64,
) -> f64 {
    match script.placement {
        Placement::Above => {
            staff.y_of(8) - TEXT_SCRIPT_PADDING_SS * staff_space - line.descent
        }
        Placement::Below => staff.y_of(0) + TEXT_SCRIPT_BELOW_STAFF_SS * staff_space,
    }
}

/// Draw every mark in `annotations` around the event at `anchor`.
pub(super) fn draw_event_marks(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    anchor: &EventAnchor,
    annotations: &NoteAnnotations,
) -> Result<(), FontError> {
    let ss = config.staff_space;
    let center_x = anchor.center_x();
    let padding = TEXT_SCRIPT_PADDING_SS * ss;
    let mut stacks = Stacks {
        above: anchor.top_y.min(staff.y_of(8)) - padding,
        below: anchor.bottom_y.max(staff.y_of(0)) + padding,
        gap: TEXT_SCRIPT_STACK_GAP_SS * ss,
    };

    // Articulations sit closest to the event.
    let articulations = if let Some(positions) = anchor.chord_positions {
        layout_chord_articulation_stack(
            &annotations.articulations,
            center_x,
            positions,
            anchor.stem,
            staff,
        )
    } else {
        layout_articulation_stack(
            &annotations.articulations,
            center_x,
            anchor.articulation_position,
            anchor.stem,
            staff,
        )
    };
    for layout in &articulations {
        if let Some(bbox) = font.glyph_bbox_design_units(layout.glyph) {
            match layout.placement {
                ArticulationPlacement::Above => {
                    stacks.above = stacks.above.min(layout.y + bbox.y_top - stacks.gap);
                }
                ArticulationPlacement::Below => {
                    stacks.below = stacks.below.max(layout.y + bbox.y_bottom + stacks.gap);
                }
            }
        }
        draw_articulation(svg, font, layout)?;
    }

    let ornament = annotations.ornament.map(|orn| {
        crate::layout::ornament::layout_ornament(orn, center_x, anchor.ornament_position, staff)
    });
    if let Some(layout) = &ornament {
        if let Some(bbox) = font.glyph_bbox_design_units(layout.glyph) {
            stacks.above = stacks.above.min(layout.y + bbox.y_top - stacks.gap);
        }
    }

    // Place dynamics, then text scripts, then the tempo mark (LilyPond's
    // outside-staff priority order); draw in the established element order.
    let dynamic = match &annotations.dynamic {
        Some(mark) => {
            let layout = layout_dynamic_mark(mark, font, ss)?;
            let default = match annotations.dynamics_placement {
                Placement::Above => staff.y_of(8) - DYNAMICS_ABOVE_STAFF_SS * ss,
                Placement::Below => staff.bottom_y() + DYNAMICS_BELOW_STAFF_SS * ss,
            };
            let baseline = stacks.place(
                annotations.dynamics_placement,
                default,
                layout.line.ascent,
                layout.line.descent,
            );
            Some((layout, baseline))
        }
        None => None,
    };

    let mut scripts = Vec::with_capacity(annotations.text_scripts.len());
    for script in &annotations.text_scripts {
        let line = script.layout(font, ss)?;
        let default = text_script_default_baseline(script, &line, staff, ss);
        let baseline = stacks.place(script.placement, default, line.ascent, line.descent);
        scripts.push((script.align, line, baseline));
    }

    let tempo = match &annotations.tempo_mark {
        Some(mark) => {
            let probe = layout_tempo_mark(mark, anchor.left_x, 0.0, font, ss)?;
            let descent = probe
                .lines
                .last()
                .map_or(0.0, |line| line.line.descent);
            let baseline = tempo_baseline(staff, ss).min(stacks.above - descent);
            Some(layout_tempo_mark(mark, anchor.left_x, baseline, font, ss)?)
        }
        None => None,
    };

    if let Some((layout, baseline)) = &dynamic {
        draw_text_line(svg, font, &layout.line, layout.left_for(center_x), *baseline)?;
    }

    if let Some((text, style)) = &annotations.rehearsal_mark {
        let layout = layout_rehearsal_mark(text, center_x, staff, ss, *style);
        draw_rehearsal_mark(svg, &layout);
    }

    if let Some(layout) = &tempo {
        draw_tempo_mark(svg, layout, font)?;
    }

    for (align, line, baseline) in &scripts {
        draw_aligned_text_line(
            svg,
            font,
            line,
            *align,
            anchor.left_x,
            anchor.width,
            *baseline,
        )?;
    }

    if let Some(syllable) = &annotations.lyric {
        let layout = layout_lyric(syllable, center_x, staff, ss);
        draw_lyric(svg, &layout);
    }

    if let Some(symbol) = &annotations.chord_symbol {
        let layout = crate::layout::chord_symbol::layout_chord_symbol_composite(
            symbol,
            center_x,
            staff,
            ss,
            font.units_per_em(),
            |g| font.glyph_advance(g).unwrap_or(0),
        );
        crate::render::chord_symbol_renderer::draw_chord_symbol_composite(svg, font, &layout)?;
    }

    if let Some(layout) = &ornament {
        crate::render::ornament_renderer::draw_ornament(svg, font, layout)?;
    }

    if let Some(sign) = annotations.navigation_sign {
        let layout = crate::layout::navigation::layout_navigation_sign(sign, center_x, staff);
        crate::render::navigation_renderer::draw_navigation_sign(svg, font, &layout)?;
    }

    if let Some(pedal_mark) = annotations.pedal {
        crate::render::pedal_renderer::draw_pedal(svg, staff, font, pedal_mark, center_x)?;
    }

    if let Some(breath) = annotations.breath_mark {
        let layout = crate::layout::breath::layout_breath_mark(
            breath,
            anchor.left_x + anchor.width,
            staff,
            annotations.breath_mark_parenthesized,
        );
        crate::render::breath_renderer::draw_breath_mark(svg, font, &layout)?;
    }

    Ok(())
}

/// Draw a rest (displaced vertically by `y_displacement`, as in additional
/// voices), its augmentation dots, and every mark attached to it.
pub(super) fn draw_rest_event(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    rest: &RestEvent,
    y_displacement: f64,
) -> Result<(), FontError> {
    let advance = draw_rest_displaced(svg, staff, font, x, rest.duration_log2, y_displacement)?;
    let Some(glyph) = rest_glyph(rest.duration_log2) else {
        return Ok(());
    };
    let half_space = config.staff_space / 2.0;
    let position = rest_staff_position(rest.duration_log2)
        - (y_displacement / half_space).round() as i8;
    if rest.dots > 0 {
        draw_dots(svg, staff, font, x, advance, position, rest.dots, 1.0, false)?;
    }
    let origin_y = rest_y(staff, rest.duration_log2) + y_displacement;
    let (top_y, bottom_y) = font
        .glyph_bbox_design_units(glyph)
        .map_or((origin_y, origin_y), |bbox| {
            (origin_y + bbox.y_top, origin_y + bbox.y_bottom)
        });
    let anchor = EventAnchor {
        left_x: x,
        width: advance,
        top_y,
        bottom_y,
        articulation_position: position,
        chord_positions: None,
        // Rest marks go above, as with a stem-down note.
        stem: StemDirection::Down,
        ornament_position: position,
    };
    draw_event_marks(svg, staff, font, config, &anchor, &rest.annotations)
}

/// Draw `marks` (an event's `text_marks`) above the staff, aligned on a
/// barline (or event edge) at `x`.
pub(super) fn draw_text_marks(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    marks: &[TextScript],
    x: f64,
) -> Result<(), FontError> {
    let ss = config.staff_space;
    let mut stacks = Stacks {
        above: staff.y_of(8) - TEXT_SCRIPT_PADDING_SS * ss,
        below: staff.y_of(0) + TEXT_SCRIPT_PADDING_SS * ss,
        gap: TEXT_SCRIPT_STACK_GAP_SS * ss,
    };
    for mark in marks {
        let line = mark.layout(font, ss)?;
        let default = text_script_default_baseline(mark, &line, staff, ss);
        let baseline = stacks.place(mark.placement, default, line.ascent, line.descent);
        draw_aligned_text_line(svg, font, &line, mark.align, x, 0.0, baseline)?;
    }
    Ok(())
}

