use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::measure::NoteheadStyle;
use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;
use crate::layout::StaffPosition;
use crate::render::stem_renderer::draw_stem;
use crate::render::SvgWriter;
use smufl::Glyph;

/// Which notehead glyph to use, determined by duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteheadKind {
    /// Breve (double whole) notehead.
    DoubleWhole,
    Whole,
    Half,
    Filled,
}

impl NoteheadKind {
    /// Map to the corresponding SMuFL glyph.
    pub fn glyph(self) -> Glyph {
        match self {
            NoteheadKind::DoubleWhole => Glyph::NoteheadDoubleWhole,
            NoteheadKind::Whole => Glyph::NoteheadWhole,
            NoteheadKind::Half => Glyph::NoteheadHalf,
            NoteheadKind::Filled => Glyph::NoteheadBlack,
        }
    }

    /// Representative log2 duration used by duration-aware notehead styles.
    pub fn duration_log2(self) -> i8 {
        match self {
            Self::DoubleWhole => -1,
            Self::Whole => 0,
            Self::Half => 1,
            Self::Filled => 2,
        }
    }
}

/// Draw ledger lines for a note at the given staff position.
///
/// Ledger lines extend symmetrically past the notehead by `leger_line_extension`
/// (scaled by `scale`, the note's size) on each side.
pub fn draw_ledger_lines(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    config: &EngravingConfig,
    note_x: f64,
    notehead_width: f64,
    position: StaffPosition,
    scale: f64,
) {
    let ys = staff.ledger_line_ys(position);
    if ys.is_empty() {
        return;
    }

    let extension = config.leger_line_extension_fu() * scale;
    let thickness = config.leger_line_thickness_fu();
    let x1 = note_x - extension;
    let x2 = note_x + notehead_width + extension;

    for y in ys {
        svg.add_line(x1, y, x2, y, "black", thickness);
    }
}

/// SVG transform placing a glyph's origin at `(x, y)` drawn at `scale` (1.0
/// for normal size).
pub(crate) fn glyph_transform(x: f64, y: f64, scale: f64) -> String {
    if (scale - 1.0).abs() < f64::EPSILON {
        format!("translate({x}, {y})")
    } else {
        format!("translate({x}, {y}) scale({scale})")
    }
}

/// Draw a notehead glyph at a given x-position and staff position.
///
/// Returns the advance width of the notehead in font design units, useful
/// for positioning subsequent elements (stems, accidentals, dots).
pub fn draw_notehead(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    position: StaffPosition,
    kind: NoteheadKind,
) -> Result<f64, FontError> {
    draw_styled_notehead(svg, staff, font, x, position, kind, NoteheadStyle::Normal, 1.0)
}

/// Return the actual font advance for a duration-aware semantic notehead.
pub fn notehead_advance(
    font: &MusicFont,
    duration_log2: i8,
    style: NoteheadStyle,
) -> Result<f64, FontError> {
    Ok(font
        .glyph_outline(style.glyph(duration_log2))?
        .advance_width as f64)
}

/// Draw a semantic notehead at `scale` (1.0 for normal size).
///
/// Returns the drawn (scaled) advance of the notehead.
#[allow(clippy::too_many_arguments)]
pub fn draw_styled_notehead(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    position: StaffPosition,
    kind: NoteheadKind,
    style: NoteheadStyle,
    scale: f64,
) -> Result<f64, FontError> {
    let outline = font.glyph_outline(style.glyph(kind.duration_log2()))?;
    let transform = glyph_transform(x, staff.y_of(position), scale);
    svg.add_path(&outline.path_data, "black", Some(&transform));
    Ok(outline.advance_width as f64 * scale)
}

/// Draw real SMuFL notehead parentheses around the span `left_x..right_x` at
/// `position`, drawn at `scale`.
///
/// The opening parenthesis ends at `left_x` (the notehead's left edge, or its
/// accidental's when the accidental is enclosed too); the closing one starts
/// at `right_x` (the notehead's right edge). Stems stay attached to the
/// notehead, never to its enclosure. Returns the opening parenthesis's left
/// edge, the enclosure's leftmost ink.
#[allow(clippy::too_many_arguments)]
pub fn draw_notehead_parentheses(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    left_x: f64,
    right_x: f64,
    position: StaffPosition,
    scale: f64,
) -> Result<f64, FontError> {
    let y = staff.y_of(position);
    let left = font.glyph_outline(Glyph::NoteheadParenthesisLeft)?;
    let open_x = left_x - left.advance_width as f64 * scale;
    svg.add_path(&left.path_data, "black", Some(&glyph_transform(open_x, y, scale)));
    let right = font.glyph_outline(Glyph::NoteheadParenthesisRight)?;
    let right_transform = glyph_transform(right_x, y, scale);
    svg.add_path(&right.path_data, "black", Some(&right_transform));
    Ok(open_x)
}

/// Draw a complete note: notehead + ledger lines (if needed).
///
/// Returns the advance width of the notehead.
pub fn draw_note(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    position: StaffPosition,
    kind: NoteheadKind,
) -> Result<f64, FontError> {
    let advance = draw_notehead(svg, staff, font, x, position, kind)?;
    draw_ledger_lines(svg, staff, config, x, advance, position, 1.0);
    Ok(advance)
}

/// Draw a complete note with stem: notehead + ledger lines + stem.
///
/// Breves and whole notes have no stem; pass `None` for direction to skip the stem,
/// or this function will draw one regardless of kind when direction is `Some`.
///
/// Returns the advance width of the notehead.
#[allow(clippy::too_many_arguments)]
pub fn draw_stemmed_note(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    config: &EngravingConfig,
    x: f64,
    position: StaffPosition,
    kind: NoteheadKind,
    direction: Option<StemDirection>,
) -> Result<f64, FontError> {
    let advance = draw_note(svg, staff, font, config, x, position, kind)?;
    if let Some(dir) = direction {
        draw_stem(svg, staff, config, x, advance, position, dir, 1.0);
    }
    Ok(advance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    // --- NoteheadKind ---

    #[test]
    fn notehead_kind_whole_maps_to_correct_glyph() {
        assert_eq!(NoteheadKind::Whole.glyph(), Glyph::NoteheadWhole);
    }

    #[test]
    fn notehead_kind_half_maps_to_correct_glyph() {
        assert_eq!(NoteheadKind::Half.glyph(), Glyph::NoteheadHalf);
    }

    #[test]
    fn notehead_kind_double_whole_maps_to_breve_glyph() {
        assert_eq!(
            NoteheadKind::DoubleWhole.glyph(),
            Glyph::NoteheadDoubleWhole
        );
        assert_eq!(
            NoteheadStyle::Normal.glyph(NoteheadKind::DoubleWhole.duration_log2()),
            Glyph::NoteheadDoubleWhole
        );
    }

    #[test]
    fn notehead_kind_filled_maps_to_correct_glyph() {
        assert_eq!(NoteheadKind::Filled.glyph(), Glyph::NoteheadBlack);
    }

    // --- draw_notehead ---

    #[test]
    fn draw_notehead_produces_path_element() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_notehead(&mut svg, &staff, &font, 500.0, 0, NoteheadKind::Filled).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("<path "), "should produce a <path> element");
        assert!(output.contains("fill=\"black\""));
    }

    #[test]
    fn draw_notehead_at_bottom_line_has_correct_y() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_notehead(&mut svg, &staff, &font, 500.0, 0, NoteheadKind::Filled).unwrap();
        let output = svg.to_svg();

        // Bottom line (pos 0): y = (8 - 0) * 125 = 1000
        let expected_y = staff.y_of(0);
        assert!((expected_y - 1000.0).abs() < f64::EPSILON);
        assert!(
            output.contains("translate(500, 1000)"),
            "notehead should be translated to (500, 1000), got: {output}"
        );
    }

    #[test]
    fn draw_notehead_at_top_line_has_correct_y() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_notehead(&mut svg, &staff, &font, 500.0, 8, NoteheadKind::Filled).unwrap();
        let output = svg.to_svg();

        // Top line (pos 8): y = 0
        assert!(output.contains("translate(500, 0)"));
    }

    #[test]
    fn draw_notehead_returns_positive_advance_width() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance =
            draw_notehead(&mut svg, &staff, &font, 500.0, 4, NoteheadKind::Filled).unwrap();
        // Bravura noteheadBlack advance is ~295 font units
        assert!(advance > 200.0, "advance should be > 200, got {advance}");
        assert!(advance < 500.0, "advance should be < 500, got {advance}");
    }

    #[test]
    fn different_notehead_kinds_produce_different_paths() {
        let (font, _, staff) = setup();
        let mut svg1 = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_notehead(&mut svg1, &staff, &font, 0.0, 4, NoteheadKind::Filled).unwrap();
        let mut svg2 = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_notehead(&mut svg2, &staff, &font, 0.0, 4, NoteheadKind::Whole).unwrap();
        assert_ne!(
            svg1.to_svg(),
            svg2.to_svg(),
            "filled and whole noteheads should differ"
        );
    }

    // --- draw_ledger_lines ---

    #[test]
    fn no_ledger_lines_for_note_on_staff() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_ledger_lines(&mut svg, &staff, &config, 500.0, 295.0, 4, 1.0);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            0,
            "position 4 (middle line) needs no ledger lines"
        );
    }

    #[test]
    fn one_ledger_line_below_for_middle_c_in_treble() {
        let (_, config, staff) = setup();
        // Middle C in treble = position -2
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_ledger_lines(&mut svg, &staff, &config, 500.0, 295.0, -2, 1.0);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            1,
            "position -2 needs exactly 1 ledger line"
        );
    }

    #[test]
    fn two_ledger_lines_below() {
        let (_, config, staff) = setup();
        // Position -4: two ledger lines at -2 and -4
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_ledger_lines(&mut svg, &staff, &config, 500.0, 295.0, -4, 1.0);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            2,
            "position -4 needs exactly 2 ledger lines"
        );
    }

    #[test]
    fn ledger_lines_extend_past_notehead() {
        let (_, config, staff) = setup();
        let note_x = 500.0;
        let notehead_width = 295.0;
        let extension = config.leger_line_extension_fu();

        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_ledger_lines(&mut svg, &staff, &config, note_x, notehead_width, -2, 1.0);
        let output = svg.to_svg();

        let expected_x1 = note_x - extension;
        let expected_x2 = note_x + notehead_width + extension;
        let x1_str = format!("x1=\"{expected_x1}\"");
        let x2_str = format!("x2=\"{expected_x2}\"");
        assert!(
            output.contains(&x1_str),
            "ledger line should start at {expected_x1}"
        );
        assert!(
            output.contains(&x2_str),
            "ledger line should end at {expected_x2}"
        );
    }

    #[test]
    fn ledger_lines_use_correct_thickness() {
        let (_, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_ledger_lines(&mut svg, &staff, &config, 500.0, 295.0, -2, 1.0);
        let output = svg.to_svg();

        let expected_sw = format!("stroke-width=\"{}\"", config.leger_line_thickness_fu());
        assert!(
            output.contains(&expected_sw),
            "ledger lines should use leger_line_thickness from config"
        );
    }

    #[test]
    fn one_ledger_line_above() {
        let (_, config, staff) = setup();
        // Position 10: one ledger line above
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -500.0, 6000.0, 2000.0);
        draw_ledger_lines(&mut svg, &staff, &config, 500.0, 295.0, 10, 1.0);
        let output = svg.to_svg();
        assert_eq!(
            output.matches("<line ").count(),
            1,
            "position 10 needs exactly 1 ledger line"
        );

        // y of position 10 = (8 - 10) * 125 = -250
        let expected_y = staff.y_of(10);
        assert!((expected_y - -250.0).abs() < f64::EPSILON);
        let y_str = format!("y1=\"{expected_y}\"");
        assert!(output.contains(&y_str));
    }

    #[test]
    fn no_ledger_lines_in_first_space_outside_staff() {
        let (_, config, staff) = setup();
        // Position -1 and 9: just outside but no ledger line needed
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_ledger_lines(&mut svg, &staff, &config, 500.0, 295.0, -1, 1.0);
        assert_eq!(svg.to_svg().matches("<line ").count(), 0);

        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_ledger_lines(&mut svg, &staff, &config, 500.0, 295.0, 9, 1.0);
        assert_eq!(svg.to_svg().matches("<line ").count(), 0);
    }

    // --- draw_note (composite) ---

    #[test]
    fn draw_note_on_staff_has_path_no_ledger_lines() {
        let (font, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        draw_note(
            &mut svg,
            &staff,
            &font,
            &config,
            500.0,
            4,
            NoteheadKind::Filled,
        )
        .unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "one notehead path");
        assert_eq!(output.matches("<line ").count(), 0, "no ledger lines");
    }

    #[test]
    fn draw_note_with_ledger_lines_has_path_and_lines() {
        let (font, config, staff) = setup();
        // Middle C in treble: position -2, needs one ledger line
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 2000.0);
        draw_note(
            &mut svg,
            &staff,
            &font,
            &config,
            500.0,
            -2,
            NoteheadKind::Filled,
        )
        .unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "one notehead path");
        assert_eq!(output.matches("<line ").count(), 1, "one ledger line");
    }

    #[test]
    fn draw_whole_note_has_wider_advance_than_filled() {
        let (font, config, staff) = setup();
        let mut svg1 = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        let advance_filled = draw_note(
            &mut svg1,
            &staff,
            &font,
            &config,
            0.0,
            4,
            NoteheadKind::Filled,
        )
        .unwrap();
        let mut svg2 = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        let advance_whole = draw_note(
            &mut svg2,
            &staff,
            &font,
            &config,
            0.0,
            4,
            NoteheadKind::Whole,
        )
        .unwrap();
        // Whole notes are wider than filled noteheads in Bravura
        assert!(
            advance_whole > advance_filled,
            "whole note advance ({advance_whole}) should be > filled ({advance_filled})"
        );
    }

    // --- draw_stemmed_note ---

    #[test]
    fn stemmed_note_with_stem_up_has_path_and_stem_line() {
        let (font, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2500.0);
        draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            500.0,
            0,
            NoteheadKind::Filled,
            Some(StemDirection::Up),
        )
        .unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "one notehead path");
        // 1 stem line, no ledger lines (position 0 = bottom line)
        assert_eq!(
            output.matches("<line ").count(),
            1,
            "one stem line for on-staff note"
        );
    }

    #[test]
    fn stemmed_note_with_ledger_lines_has_stem_plus_ledger() {
        let (font, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 6000.0, 2500.0);
        // Position -2 (middle C in treble): 1 ledger line + 1 stem = 2 lines
        draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            500.0,
            -2,
            NoteheadKind::Filled,
            Some(StemDirection::Up),
        )
        .unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "one notehead path");
        assert_eq!(
            output.matches("<line ").count(),
            2,
            "one ledger line + one stem"
        );
    }

    #[test]
    fn stemmed_note_none_direction_skips_stem() {
        let (font, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            500.0,
            4,
            NoteheadKind::Whole,
            None,
        )
        .unwrap();
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1, "one notehead path");
        assert_eq!(
            output.matches("<line ").count(),
            0,
            "no stem for whole note"
        );
    }

    #[test]
    fn stemmed_note_returns_same_advance_as_draw_note() {
        let (font, config, staff) = setup();
        let mut svg1 = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        let adv1 = draw_note(
            &mut svg1,
            &staff,
            &font,
            &config,
            0.0,
            4,
            NoteheadKind::Filled,
        )
        .unwrap();
        let mut svg2 = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 1500.0);
        let adv2 = draw_stemmed_note(
            &mut svg2,
            &staff,
            &font,
            &config,
            0.0,
            4,
            NoteheadKind::Filled,
            Some(StemDirection::Down),
        )
        .unwrap();
        assert!(
            (adv1 - adv2).abs() < f64::EPSILON,
            "advance width should be identical"
        );
    }

    #[test]
    fn stemmed_note_stem_down_has_correct_y_range() {
        let (font, config, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -200.0, -500.0, 6000.0, 3000.0);
        // Position 6 (top space), stem down — tip should extend below
        draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            500.0,
            6,
            NoteheadKind::Filled,
            Some(StemDirection::Down),
        )
        .unwrap();
        let output = svg.to_svg();

        // The stem's y1 should be the notehead y, y2 should be below
        let notehead_y = staff.y_of(6);
        let y1_str = format!("y1=\"{notehead_y}\"");
        assert!(
            output.contains(&y1_str),
            "stem y1 should be at notehead y={notehead_y}"
        );
    }

    #[test]
    fn styled_parenthesized_notehead_uses_assigned_smufl_paths() {
        let (font, _, staff) = setup();
        let selected = font
            .glyph_outline(Glyph::NoteheadDiamondBlack)
            .unwrap()
            .path_data;
        let ordinary = font.glyph_outline(Glyph::NoteheadBlack).unwrap().path_data;
        let left = font
            .glyph_outline(Glyph::NoteheadParenthesisLeft)
            .unwrap()
            .path_data;
        let right = font
            .glyph_outline(Glyph::NoteheadParenthesisRight)
            .unwrap()
            .path_data;
        let left_advance = font
            .glyph_advance(Glyph::NoteheadParenthesisLeft)
            .unwrap() as f64;
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance = draw_styled_notehead(
            &mut svg,
            &staff,
            &font,
            500.0,
            4,
            NoteheadKind::Filled,
            NoteheadStyle::Diamond,
            1.0,
        )
        .unwrap();
        draw_notehead_parentheses(&mut svg, &staff, &font, 500.0, 500.0 + advance, 4, 1.0)
            .unwrap();
        let output = svg.to_svg();
        assert!(output.contains(&selected));
        assert!(output.contains(&left));
        assert!(output.contains(&right));
        assert!(!output.contains(&ordinary));
        let y = staff.y_of(4);
        assert!(output.contains(&format!("translate({}, {y})", 500.0 - left_advance)));
        assert!(output.contains(&format!("translate({}, {y})", 500.0 + advance)));
    }

    #[test]
    fn scaled_notehead_scales_glyph_and_advance() {
        let (font, _, staff) = setup();
        let full = font.glyph_advance(Glyph::NoteheadBlack).unwrap() as f64;
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let advance = draw_styled_notehead(
            &mut svg,
            &staff,
            &font,
            500.0,
            4,
            NoteheadKind::Filled,
            NoteheadStyle::Normal,
            0.5,
        )
        .unwrap();
        assert!((advance - full * 0.5).abs() < 1e-9);
        let y = staff.y_of(4);
        assert!(svg
            .to_svg()
            .contains(&format!("translate(500, {y}) scale(0.5)")));
    }
}
