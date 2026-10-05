use smufl::Glyph;

use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::flag::flag_glyph;
use crate::layout::grace::GraceGroupLayout;
use crate::layout::measure::NoteheadStyle;
use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;
use crate::render::accidental_renderer::draw_accidental;
use crate::render::dot_renderer::draw_dots;
use crate::render::flag_renderer::draw_flag;
use crate::render::note_renderer::{draw_ledger_lines, draw_styled_notehead, NoteheadKind};
use crate::render::SvgWriter;

/// Bravura's `flag8thUp` slash anchors (`graceNoteSlashSW`, `graceNoteSlashNE`),
/// in staff spaces relative to the stem tip with y up; used when the font's
/// metadata lacks them.
const SLASH_UP_FALLBACK: ((f64, f64), (f64, f64)) = ((-0.644, -2.456), (1.284, -0.796));

/// Bravura's `flag8thDown` slash anchors (`graceNoteSlashNW`, `graceNoteSlashSE`).
const SLASH_DOWN_FALLBACK: ((f64, f64), (f64, f64)) = ((-0.596, 2.168), (1.328, 0.628));

/// Draw a grace group laid out by [`crate::layout::grace::layout_grace_group`]:
/// every note's accidental, scaled notehead, ledger lines, dots, stem and
/// flag, then the group's beams and the acciaccatura slash through the first
/// stem.
pub fn draw_grace_group(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    layout: &GraceGroupLayout,
) -> Result<(), FontError> {
    let scale = layout.scale;
    let thickness = config.stem_thickness_fu() * scale;
    for note in &layout.notes {
        if let Some(accidental) = note.accidental {
            draw_accidental(
                svg,
                staff,
                font,
                note.x,
                0.0,
                note.staff_position,
                accidental,
                scale,
            )?;
        }
        let kind = match note.duration_log2 {
            ..=-1 => NoteheadKind::DoubleWhole,
            0 => NoteheadKind::Whole,
            1 => NoteheadKind::Half,
            _ => NoteheadKind::Filled,
        };
        let advance = draw_styled_notehead(
            svg,
            staff,
            font,
            note.x,
            note.staff_position,
            kind,
            NoteheadStyle::Normal,
            scale,
        )?;
        draw_ledger_lines(
            svg,
            staff,
            config,
            note.x,
            advance,
            note.staff_position,
            scale,
        );
        draw_dots(
            svg,
            staff,
            font,
            note.x,
            advance,
            note.staff_position,
            note.dots,
            scale,
            false,
        )?;
        if let Some(stem) = note.stem {
            svg.add_line(
                stem.x,
                stem.y_notehead,
                stem.x,
                stem.y_tip,
                "black",
                thickness,
            );
            draw_flag(
                svg,
                font,
                stem.x,
                stem.y_tip,
                note.flags,
                layout.stem_direction,
                scale,
            )?;
        }
    }

    for beam in &layout.beams {
        let inward = match layout.stem_direction {
            StemDirection::Up => layout.beam_thickness,
            StemDirection::Down => -layout.beam_thickness,
        };
        svg.add_polygon(
            &[
                (beam.x1, beam.y1),
                (beam.x2, beam.y2),
                (beam.x2, beam.y2 + inward),
                (beam.x1, beam.y1 + inward),
            ],
            "black",
        );
    }

    if layout.slashed {
        if let Some(stem) = layout.notes.first().and_then(|note| note.stem) {
            let ((x1, y1), (x2, y2)) = slash_anchors(font, layout.stem_direction);
            let unit = staff.staff_space * scale;
            svg.add_line(
                stem.x + x1 * unit,
                stem.y_tip - y1 * unit,
                stem.x + x2 * unit,
                stem.y_tip - y2 * unit,
                "black",
                thickness,
            );
        }
    }
    Ok(())
}

/// The two ends of the acciaccatura slash in staff spaces relative to the
/// stem tip (y up), from the eighth-note flag's SMuFL `graceNoteSlash*`
/// anchors.
fn slash_anchors(font: &MusicFont, direction: StemDirection) -> ((f64, f64), (f64, f64)) {
    let flag = flag_glyph(1, direction).unwrap_or(Glyph::Flag8thUp);
    let anchors = font.metadata().anchors.get(flag);
    let point = |coord: Option<smufl::Coord>| coord.map(|c| (c.x().0, c.y().0));
    match direction {
        StemDirection::Up => anchors
            .and_then(|a| Some((point(a.grace_note_slash_sw)?, point(a.grace_note_slash_ne)?)))
            .unwrap_or(SLASH_UP_FALLBACK),
        StemDirection::Down => anchors
            .and_then(|a| Some((point(a.grace_note_slash_nw)?, point(a.grace_note_slash_se)?)))
            .unwrap_or(SLASH_DOWN_FALLBACK),
    }
}

#[cfg(test)]
mod tests;
