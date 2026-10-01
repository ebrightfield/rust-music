use crate::font::{FontError, MusicFont};
use crate::layout::key_signature::{key_signature_layout, KeySignature, KeySignatureLayout};
use crate::layout::staff::StaffLayout;
use crate::render::SvgWriter;
use music::notation::clef::Clef;

/// Draw a key signature at position `x` on the staff.
///
/// Returns the total advance width of the key signature group (0 for open key).
pub fn draw_key_signature(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    key: &KeySignature,
    clef: &Clef,
) -> Result<f64, FontError> {
    let config = font.engraving_config();
    let layout = key_signature_layout(
        key,
        clef,
        |g| {
            font.glyph_outline(g)
                .map(|o| o.advance_width as f64)
                .unwrap_or(0.0)
        },
        config.staff_space,
    );
    render_key_sig_layout(svg, staff, font, x, &layout)
}

fn render_key_sig_layout(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    layout: &KeySignatureLayout,
) -> Result<f64, FontError> {
    for acc in &layout.accidentals {
        let outline = font.glyph_outline(acc.glyph)?;
        let y = staff.y_of(acc.staff_position);
        let gx = x + acc.x_offset;
        let transform = format!("translate({gx}, {y})");
        svg.add_path(&outline.path_data, "black", Some(&transform));
    }
    Ok(layout.width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;

    fn setup() -> (MusicFont<'static>, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, staff)
    }

    #[test]
    fn open_key_produces_no_paths() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let width = draw_key_signature(
            &mut svg,
            &staff,
            &font,
            500.0,
            &KeySignature::Open,
            &Clef::Treble,
        )
        .unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 0);
        assert!((width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn one_sharp_produces_one_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let width = draw_key_signature(
            &mut svg,
            &staff,
            &font,
            500.0,
            &KeySignature::Sharps(1),
            &Clef::Treble,
        )
        .unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
        assert!(width > 0.0);
    }

    #[test]
    fn four_sharps_produces_four_paths() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_key_signature(
            &mut svg,
            &staff,
            &font,
            500.0,
            &KeySignature::Sharps(4),
            &Clef::Treble,
        )
        .unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 4);
    }

    #[test]
    fn seven_flats_produces_seven_paths() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_key_signature(
            &mut svg,
            &staff,
            &font,
            500.0,
            &KeySignature::Flats(7),
            &Clef::Treble,
        )
        .unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 7);
    }

    #[test]
    fn sharp_placed_at_correct_y_for_treble_f_sharp() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_key_signature(
            &mut svg,
            &staff,
            &font,
            500.0,
            &KeySignature::Sharps(1),
            &Clef::Treble,
        )
        .unwrap();
        let output = svg.to_svg();
        // F# in treble clef is at staff position 8 (top line)
        let y_top = staff.y_of(8);
        let expected_fragment = format!(", {y_top})");
        assert!(
            output.contains(&expected_fragment),
            "sharp should be at top line y={y_top}"
        );
    }

    #[test]
    fn flat_placed_at_correct_y_for_treble_b_flat() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_key_signature(
            &mut svg,
            &staff,
            &font,
            500.0,
            &KeySignature::Flats(1),
            &Clef::Treble,
        )
        .unwrap();
        let output = svg.to_svg();
        // Bb in treble clef at staff position 4 (middle line)
        let y_mid = staff.y_of(4);
        let expected_fragment = format!(", {y_mid})");
        assert!(
            output.contains(&expected_fragment),
            "flat should be at middle line y={y_mid}"
        );
    }

    #[test]
    fn width_increases_with_more_accidentals() {
        let (font, staff) = setup();
        let mut svg1 = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let w1 = draw_key_signature(
            &mut svg1,
            &staff,
            &font,
            0.0,
            &KeySignature::Sharps(2),
            &Clef::Treble,
        )
        .unwrap();
        let mut svg2 = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let w2 = draw_key_signature(
            &mut svg2,
            &staff,
            &font,
            0.0,
            &KeySignature::Sharps(5),
            &Clef::Treble,
        )
        .unwrap();
        assert!(w2 > w1, "5 sharps should be wider than 2 sharps");
    }

    #[test]
    fn bass_clef_sharp_positions_differ_from_treble() {
        let (font, staff) = setup();
        let mut svg_treble = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_key_signature(
            &mut svg_treble,
            &staff,
            &font,
            0.0,
            &KeySignature::Sharps(3),
            &Clef::Treble,
        )
        .unwrap();
        let mut svg_bass = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_key_signature(
            &mut svg_bass,
            &staff,
            &font,
            0.0,
            &KeySignature::Sharps(3),
            &Clef::Bass,
        )
        .unwrap();
        // Same glyph shapes but different y-positions
        assert_ne!(svg_treble.to_svg(), svg_bass.to_svg());
    }
}
