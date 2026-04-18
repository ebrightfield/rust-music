use crate::font::{FontError, MusicFont};
use crate::layout::staff::StaffLayout;
use crate::layout::time_signature::{TimeSignatureKind, TimeSignatureLayout, time_signature_layout};
use crate::render::SvgWriter;

/// Draw a time signature at position `x` on the staff.
///
/// Returns the total advance width of the time signature group.
pub fn draw_time_signature(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    kind: &TimeSignatureKind,
) -> Result<f64, FontError> {
    let layout = time_signature_layout(kind, |g| {
        font.glyph_outline(g)
            .map(|o| o.advance_width as f64)
            .unwrap_or(0.0)
    });
    render_time_sig_layout(svg, staff, font, x, &layout)
}

fn render_time_sig_layout(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    x: f64,
    layout: &TimeSignatureLayout,
) -> Result<f64, FontError> {
    for &(glyph, staff_pos, x_offset) in &layout.glyphs {
        let outline = font.glyph_outline(glyph)?;
        let y = staff.y_of(staff_pos);
        let gx = x + x_offset;
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
    fn common_time_produces_one_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let width =
            draw_time_signature(&mut svg, &staff, &font, 500.0, &TimeSignatureKind::Common)
                .unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
        assert!(width > 0.0);
    }

    #[test]
    fn cut_common_produces_one_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let width =
            draw_time_signature(&mut svg, &staff, &font, 500.0, &TimeSignatureKind::CutCommon)
                .unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
        assert!(width > 0.0);
    }

    #[test]
    fn four_four_produces_two_paths() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let kind = TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        };
        draw_time_signature(&mut svg, &staff, &font, 500.0, &kind).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 2);
    }

    #[test]
    fn twelve_eight_produces_three_paths() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let kind = TimeSignatureKind::Numeric {
            numerator: 12,
            denominator: 8,
        };
        draw_time_signature(&mut svg, &staff, &font, 500.0, &kind).unwrap();
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 3);
    }

    #[test]
    fn common_time_glyph_at_middle_line() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_time_signature(&mut svg, &staff, &font, 500.0, &TimeSignatureKind::Common).unwrap();
        let output = svg.to_svg();
        let y_middle = staff.y_of(4);
        let expected = format!("translate(500, {y_middle})");
        assert!(
            output.contains(&expected),
            "expected transform at middle line"
        );
    }

    #[test]
    fn numeric_numerator_at_position_6_denominator_at_2() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        let kind = TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        };
        draw_time_signature(&mut svg, &staff, &font, 500.0, &kind).unwrap();
        let output = svg.to_svg();
        let y6 = staff.y_of(6);
        let y2 = staff.y_of(2);
        assert!(
            output.contains(&format!(", {y6})")),
            "numerator should be at staff position 6"
        );
        assert!(
            output.contains(&format!(", {y2})")),
            "denominator should be at staff position 2"
        );
    }

    #[test]
    fn common_and_cut_produce_different_paths() {
        let (font, staff) = setup();
        let mut svg1 = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_time_signature(&mut svg1, &staff, &font, 0.0, &TimeSignatureKind::Common).unwrap();
        let mut svg2 = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
        draw_time_signature(&mut svg2, &staff, &font, 0.0, &TimeSignatureKind::CutCommon).unwrap();
        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn width_is_positive_for_all_kinds() {
        let (font, staff) = setup();
        let kinds = [
            TimeSignatureKind::Common,
            TimeSignatureKind::CutCommon,
            TimeSignatureKind::Numeric {
                numerator: 3,
                denominator: 4,
            },
            TimeSignatureKind::Numeric {
                numerator: 6,
                denominator: 8,
            },
            TimeSignatureKind::Numeric {
                numerator: 12,
                denominator: 8,
            },
        ];
        for kind in &kinds {
            let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -200.0, 6000.0, 1500.0);
            let width = draw_time_signature(&mut svg, &staff, &font, 0.0, kind).unwrap();
            assert!(width > 0.0, "{:?} should have positive width", kind);
        }
    }
}
