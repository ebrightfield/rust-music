use smufl::Glyph;

use crate::font::{FontError, MusicFont};
use crate::layout::accidental::{
    accidental_x, layout_accidental_columns, ResolvedAccidental, ACCIDENTAL_COLUMN_GAP_SS,
};
use crate::layout::chord::ChordNoteLayout;
use crate::layout::staff::StaffLayout;
use crate::layout::StaffPosition;
use crate::render::note_renderer::glyph_transform;
use crate::render::SvgWriter;

/// Draw a resolved accidental to the left of a notehead column, at `scale`
/// (1.0 for normal size).
///
/// `notehead_x` is the left edge of the notehead column the accidental belongs
/// to and `column_offset` how much further left its stacked accidental column
/// sits (`0.0` for the column nearest the noteheads). The accidental ends
/// [`crate::layout::accidental::ACCIDENTAL_NOTEHEAD_PADDING_SS`] (scaled) left
/// of `notehead_x - column_offset`. A parenthesized (cautionary) accidental is
/// drawn as `AccidentalParensLeft`, the accidental glyph, then
/// `AccidentalParensRight`, with the closing parenthesis ending where a plain
/// accidental would.
///
/// Returns the x-position of the leftmost drawn glyph, i.e. the accidental's
/// full left extent.
#[allow(clippy::too_many_arguments)]
pub fn draw_accidental(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    notehead_x: f64,
    column_offset: f64,
    position: StaffPosition,
    accidental: ResolvedAccidental,
    scale: f64,
) -> Result<f64, FontError> {
    let column_x = notehead_x - column_offset;
    let y = staff.y_of(position);
    let unit = staff.staff_space * scale;
    let outline = font.glyph_outline(accidental.glyph)?;
    let glyph_advance = outline.advance_width as f64 * scale;
    if !accidental.parenthesized {
        let x = accidental_x(column_x, glyph_advance, unit);
        draw_glyph(svg, &outline.path_data, x, y, scale);
        return Ok(x);
    }

    let close = font.glyph_outline(Glyph::AccidentalParensRight)?;
    let open = font.glyph_outline(Glyph::AccidentalParensLeft)?;
    let close_x = accidental_x(column_x, close.advance_width as f64 * scale, unit);
    let glyph_x = close_x - glyph_advance;
    let open_x = glyph_x - open.advance_width as f64 * scale;
    draw_glyph(svg, &open.path_data, open_x, y, scale);
    draw_glyph(svg, &outline.path_data, glyph_x, y, scale);
    draw_glyph(svg, &close.path_data, close_x, y, scale);
    Ok(open_x)
}

fn draw_glyph(svg: &mut SvgWriter, path_data: &str, x: f64, y: f64, scale: f64) {
    svg.add_path(path_data, "black", Some(&glyph_transform(x, y, scale)));
}

/// Horizontal advance of a resolved accidental, including its parentheses.
pub fn resolved_accidental_advance(
    font: &MusicFont,
    accidental: ResolvedAccidental,
) -> Result<f64, FontError> {
    let glyph = font.glyph_advance(accidental.glyph)? as f64;
    if !accidental.parenthesized {
        return Ok(glyph);
    }
    Ok(font.glyph_advance(Glyph::AccidentalParensLeft)? as f64
        + glyph
        + font.glyph_advance(Glyph::AccidentalParensRight)? as f64)
}

/// Stacked-column offsets for the accidentals of one chord drawn at `scale`,
/// parallel to `notes` (`0.0` for notes without an accidental).
///
/// Accidentals within a sixth of each other take separate columns (see
/// [`layout_accidental_columns`]); a column is as wide as its widest member,
/// parentheses included, so a cautionary accidental pushes outer columns
/// clear of itself.
pub(crate) fn chord_accidental_column_offsets(
    font: &MusicFont,
    staff: &StaffLayout,
    notes: &[ChordNoteLayout],
    scale: f64,
) -> Result<Vec<f64>, FontError> {
    let mut offsets = vec![0.0; notes.len()];
    if notes
        .iter()
        .filter(|note| note.accidental.is_some())
        .count()
        < 2
    {
        return Ok(offsets);
    }
    let mut stacked = Vec::with_capacity(notes.len());
    let mut owners = Vec::with_capacity(notes.len());
    for (index, note) in notes.iter().enumerate() {
        if let Some(accidental) = note.accidental {
            stacked.push((
                note.staff_position,
                resolved_accidental_advance(font, accidental)? * scale,
            ));
            owners.push(index);
        }
    }
    let columns = layout_accidental_columns(
        &stacked,
        ACCIDENTAL_COLUMN_GAP_SS * staff.staff_space * scale,
    );
    for (owner, offset) in owners.into_iter().zip(columns.column_offsets) {
        offsets[owner] = offset;
    }
    Ok(offsets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{bravura_font, EngravingConfig};
    use crate::layout::accidental::ACCIDENTAL_NOTEHEAD_PADDING_SS;
    use crate::layout::measure::NoteheadStyle;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    fn writer() -> SvgWriter {
        SvgWriter::new(800.0, 200.0, -500.0, -500.0, 6000.0, 2000.0)
    }

    /// `(glyph, x, y)` for every drawn path, in document order, identified by
    /// exact path data.
    fn drawn_glyphs(svg: &str, font: &MusicFont, candidates: &[Glyph]) -> Vec<(Glyph, f64, f64)> {
        svg.split("<path d=\"")
            .skip(1)
            .map(|path| {
                let (data, rest) = path.split_once('"').unwrap();
                let glyph = *candidates
                    .iter()
                    .find(|glyph| font.glyph_outline(**glyph).unwrap().path_data == data)
                    .expect("every drawn path is a candidate glyph");
                let translate = rest.split_once("translate(").unwrap().1;
                let (x, y) = translate
                    .split_once(')')
                    .unwrap()
                    .0
                    .split_once(", ")
                    .unwrap();
                (glyph, x.parse().unwrap(), y.parse().unwrap())
            })
            .collect()
    }

    const ACCIDENTAL_GLYPHS: [Glyph; 7] = [
        Glyph::AccidentalSharp,
        Glyph::AccidentalFlat,
        Glyph::AccidentalNatural,
        Glyph::AccidentalDoubleSharp,
        Glyph::AccidentalDoubleFlat,
        Glyph::AccidentalParensLeft,
        Glyph::AccidentalParensRight,
    ];

    #[test]
    fn plain_accidental_is_one_glyph_padded_left_of_the_notehead() {
        let (font, _, staff) = setup();
        let mut svg = writer();
        let left = draw_accidental(
            &mut svg,
            &staff,
            &font,
            500.0,
            0.0,
            0,
            ResolvedAccidental::plain(Glyph::AccidentalFlat),
            1.0,
        )
        .unwrap();

        let flat = font.glyph_advance(Glyph::AccidentalFlat).unwrap() as f64;
        let expected_x = 500.0 - flat - ACCIDENTAL_NOTEHEAD_PADDING_SS * staff.staff_space;
        assert_eq!(left, expected_x);
        assert_eq!(
            drawn_glyphs(&svg.to_svg(), &font, &ACCIDENTAL_GLYPHS),
            vec![(Glyph::AccidentalFlat, expected_x, staff.y_of(0))]
        );
    }

    #[test]
    fn cautionary_accidental_draws_parens_around_the_glyph() {
        let (font, _, staff) = setup();
        let mut svg = writer();
        let left = draw_accidental(
            &mut svg,
            &staff,
            &font,
            500.0,
            0.0,
            4,
            ResolvedAccidental::cautionary(Glyph::AccidentalNatural),
            1.0,
        )
        .unwrap();

        let advance = |glyph| font.glyph_advance(glyph).unwrap() as f64;
        let close_x = 500.0
            - advance(Glyph::AccidentalParensRight)
            - ACCIDENTAL_NOTEHEAD_PADDING_SS * staff.staff_space;
        let natural_x = close_x - advance(Glyph::AccidentalNatural);
        let open_x = natural_x - advance(Glyph::AccidentalParensLeft);
        let y = staff.y_of(4);
        assert_eq!(
            drawn_glyphs(&svg.to_svg(), &font, &ACCIDENTAL_GLYPHS),
            vec![
                (Glyph::AccidentalParensLeft, open_x, y),
                (Glyph::AccidentalNatural, natural_x, y),
                (Glyph::AccidentalParensRight, close_x, y),
            ]
        );
        assert_eq!(left, open_x);
        assert_eq!(
            500.0 - ACCIDENTAL_NOTEHEAD_PADDING_SS * staff.staff_space - left,
            resolved_accidental_advance(
                &font,
                ResolvedAccidental::cautionary(Glyph::AccidentalNatural)
            )
            .unwrap()
        );
    }

    #[test]
    fn column_offset_moves_the_accidental_left_by_exactly_the_offset() {
        let (font, _, staff) = setup();
        let sharp = ResolvedAccidental::plain(Glyph::AccidentalSharp);
        let near =
            draw_accidental(&mut writer(), &staff, &font, 500.0, 0.0, 4, sharp, 1.0).unwrap();
        let far =
            draw_accidental(&mut writer(), &staff, &font, 500.0, 274.0, 4, sharp, 1.0).unwrap();
        assert_eq!(near - far, 274.0);
    }

    fn chord_note(
        position: StaffPosition,
        accidental: Option<ResolvedAccidental>,
    ) -> ChordNoteLayout {
        ChordNoteLayout {
            staff_position: position,
            offset: false,
            accidental,
            notehead_style: NoteheadStyle::Normal,
            parenthesized: false,
        }
    }

    #[test]
    fn chord_accidentals_a_third_apart_stack_clear_of_a_cautionary_member() {
        let (font, _, staff) = setup();
        let gap = ACCIDENTAL_COLUMN_GAP_SS * staff.staff_space;
        let sharp = ResolvedAccidental::plain(Glyph::AccidentalSharp);
        let cautionary = ResolvedAccidental::cautionary(Glyph::AccidentalSharp);

        let plain_offsets = chord_accidental_column_offsets(
            &font,
            &staff,
            &[chord_note(2, Some(sharp)), chord_note(4, Some(sharp))],
            1.0,
        )
        .unwrap();
        let sharp_advance = font.glyph_advance(Glyph::AccidentalSharp).unwrap() as f64;
        // The higher accidental takes the nearest column.
        assert_eq!(plain_offsets, vec![sharp_advance + gap, 0.0]);

        let cautionary_offsets = chord_accidental_column_offsets(
            &font,
            &staff,
            &[chord_note(2, Some(sharp)), chord_note(4, Some(cautionary))],
            1.0,
        )
        .unwrap();
        let cautionary_advance = resolved_accidental_advance(&font, cautionary).unwrap();
        assert_eq!(cautionary_offsets, vec![cautionary_advance + gap, 0.0]);

        // Drawn, the outer sharp's right edge clears the cautionary group's
        // left parenthesis by exactly the column gap: no horizontal overlap.
        let mut svg = writer();
        let cautionary_left =
            draw_accidental(&mut svg, &staff, &font, 500.0, 0.0, 4, cautionary, 1.0).unwrap();
        let sharp_left = draw_accidental(
            &mut svg,
            &staff,
            &font,
            500.0,
            cautionary_offsets[0],
            2,
            sharp,
            1.0,
        )
        .unwrap();
        assert!((cautionary_left - (sharp_left + sharp_advance) - gap).abs() < 1e-9);
    }

    #[test]
    fn a_single_chord_accidental_needs_no_stacking() {
        let (font, _, staff) = setup();
        let offsets = chord_accidental_column_offsets(
            &font,
            &staff,
            &[
                chord_note(
                    2,
                    Some(ResolvedAccidental::cautionary(Glyph::AccidentalFlat)),
                ),
                chord_note(4, None),
            ],
            1.0,
        )
        .unwrap();
        assert_eq!(offsets, vec![0.0, 0.0]);
    }
}
