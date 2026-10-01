use music::note::spelling::Accidental;

use crate::font::{FontError, MusicFont};
use crate::layout::accidental::{accidental_glyph, accidental_x};
use crate::layout::staff::StaffLayout;
use crate::layout::StaffPosition;
use crate::render::SvgWriter;

/// Draw an accidental to the left of a notehead at the given staff position.
///
/// Returns the x-position at which the accidental was drawn, or `None` if
/// no accidental was drawn (e.g., natural without `show_natural`).
/// The x-position can be used to compute the total horizontal extent
/// of the note+accidental group.
pub fn draw_accidental(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    notehead_x: f64,
    position: StaffPosition,
    accidental: Accidental,
    show_natural: bool,
) -> Result<Option<f64>, FontError> {
    let glyph = match accidental_glyph(accidental, show_natural) {
        Some(g) => g,
        None => return Ok(None),
    };

    let outline = font.glyph_outline(glyph)?;
    let acc_advance = outline.advance_width as f64;
    let x = accidental_x(notehead_x, acc_advance, staff.staff_space);
    let y = staff.y_of(position);

    let transform = format!("translate({x}, {y})");
    svg.add_path(&outline.path_data, "black", Some(&transform));

    Ok(Some(x))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{bravura_font, EngravingConfig};
    use crate::layout::accidental::ACCIDENTAL_NOTEHEAD_PADDING_SS;
    use smufl::Glyph;

    fn setup() -> (MusicFont<'static>, EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, config, staff)
    }

    #[test]
    fn sharp_accidental_produces_path() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        let result = draw_accidental(&mut svg, &staff, &font, 500.0, 4, Accidental::Sharp, false);
        let acc_x = result.unwrap().expect("sharp should be drawn");
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1);
        assert!(acc_x < 500.0, "accidental x should be left of notehead");
    }

    #[test]
    fn flat_accidental_at_bottom_line() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        let result = draw_accidental(&mut svg, &staff, &font, 500.0, 0, Accidental::Flat, false);
        let acc_x = result.unwrap().expect("flat should be drawn");
        let output = svg.to_svg();

        assert_eq!(output.matches("<path ").count(), 1);
        // Should be translated to y of position 0
        let expected_y = staff.y_of(0);
        let y_str = format!("{expected_y})");
        assert!(output.contains(&y_str), "should contain y={expected_y}");
        assert!(acc_x < 500.0);
    }

    #[test]
    fn natural_not_drawn_by_default() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        let result = draw_accidental(
            &mut svg,
            &staff,
            &font,
            500.0,
            4,
            Accidental::Natural,
            false,
        );
        assert!(result.unwrap().is_none());
        assert_eq!(svg.to_svg().matches("<path ").count(), 0);
    }

    #[test]
    fn natural_drawn_when_show_natural_true() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        let result = draw_accidental(&mut svg, &staff, &font, 500.0, 4, Accidental::Natural, true);
        assert!(result.unwrap().is_some());
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn double_sharp_produces_path() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        let result = draw_accidental(
            &mut svg,
            &staff,
            &font,
            500.0,
            4,
            Accidental::DoubleSharp,
            false,
        );
        assert!(result.unwrap().is_some());
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn double_flat_produces_path() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        let result = draw_accidental(
            &mut svg,
            &staff,
            &font,
            500.0,
            4,
            Accidental::DoubleFlat,
            false,
        );
        assert!(result.unwrap().is_some());
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn accidental_x_position_uses_correct_padding() {
        let (font, _, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        let notehead_x = 500.0;
        let acc_x = draw_accidental(
            &mut svg,
            &staff,
            &font,
            notehead_x,
            4,
            Accidental::Sharp,
            false,
        )
        .unwrap()
        .unwrap();

        let sharp_advance = font.glyph_advance(Glyph::AccidentalSharp).unwrap() as f64;
        let expected_padding = ACCIDENTAL_NOTEHEAD_PADDING_SS * staff.staff_space;
        let expected_x = notehead_x - sharp_advance - expected_padding;

        assert!(
            (acc_x - expected_x).abs() < 1e-6,
            "expected {expected_x}, got {acc_x}"
        );
    }

    #[test]
    fn different_accidentals_produce_different_paths() {
        let (font, _, staff) = setup();

        let mut svg_sharp = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        draw_accidental(
            &mut svg_sharp,
            &staff,
            &font,
            500.0,
            4,
            Accidental::Sharp,
            false,
        )
        .unwrap();

        let mut svg_flat = SvgWriter::new(800.0, 200.0, -500.0, -200.0, 6000.0, 1500.0);
        draw_accidental(
            &mut svg_flat,
            &staff,
            &font,
            500.0,
            4,
            Accidental::Flat,
            false,
        )
        .unwrap();

        assert_ne!(
            svg_sharp.to_svg(),
            svg_flat.to_svg(),
            "sharp and flat should produce different SVG"
        );
    }

    #[test]
    fn accidental_at_different_positions_has_different_y() {
        let (font, _, staff) = setup();

        let mut svg_low = SvgWriter::new(800.0, 200.0, -500.0, -500.0, 6000.0, 2000.0);
        draw_accidental(
            &mut svg_low,
            &staff,
            &font,
            500.0,
            0,
            Accidental::Sharp,
            false,
        )
        .unwrap();

        let mut svg_high = SvgWriter::new(800.0, 200.0, -500.0, -500.0, 6000.0, 2000.0);
        draw_accidental(
            &mut svg_high,
            &staff,
            &font,
            500.0,
            8,
            Accidental::Sharp,
            false,
        )
        .unwrap();

        assert_ne!(
            svg_low.to_svg(),
            svg_high.to_svg(),
            "different positions should produce different y-translations"
        );
    }
}
