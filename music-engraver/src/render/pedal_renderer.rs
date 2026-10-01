use crate::font::{FontError, MusicFont};
use crate::layout::pedal::{layout_pedal, PedalLayout, PedalMark};
use crate::layout::staff::StaffLayout;
use crate::render::SvgWriter;

/// Draw a pedal marking (Ped. or *) below the staff, centered on `note_center_x`.
///
/// Returns the `PedalLayout` describing where the glyph was placed,
/// or a `FontError` if the glyph cannot be resolved.
pub fn draw_pedal(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    mark: PedalMark,
    note_center_x: f64,
) -> Result<PedalLayout, FontError> {
    let outline = font.glyph_outline(mark.glyph())?;
    let advance_width = outline.advance_width as f64;

    let layout = layout_pedal(mark, note_center_x, advance_width, staff);

    let transform = format!("translate({}, {})", layout.x, layout.y);
    svg.add_path(&outline.path_data, "black", Some(&transform));

    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::pedal::PEDAL_BELOW_STAFF_SS;
    use smufl::Glyph;

    fn setup() -> (MusicFont<'static>, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, staff)
    }

    #[test]
    fn pedal_down_produces_single_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let result = draw_pedal(&mut svg, &staff, &font, PedalMark::Down, 500.0);
        assert!(result.is_ok());
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn pedal_up_produces_single_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg, &staff, &font, PedalMark::Up, 500.0).unwrap();
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn down_and_up_produce_different_paths() {
        let (font, staff) = setup();

        let mut svg_down = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg_down, &staff, &font, PedalMark::Down, 500.0).unwrap();

        let mut svg_up = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg_up, &staff, &font, PedalMark::Up, 500.0).unwrap();

        assert_ne!(svg_down.to_svg(), svg_up.to_svg());
    }

    #[test]
    fn layout_y_is_below_staff_at_correct_distance() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_pedal(&mut svg, &staff, &font, PedalMark::Down, 500.0).unwrap();
        let expected_y = staff.bottom_y() + PEDAL_BELOW_STAFF_SS * staff.staff_space;
        assert!(
            (layout.y - expected_y).abs() < 1e-6,
            "expected y={expected_y}, got {}",
            layout.y
        );
    }

    #[test]
    fn layout_is_centered_on_note() {
        let (font, staff) = setup();
        let note_x = 750.0;
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_pedal(&mut svg, &staff, &font, PedalMark::Down, note_x).unwrap();

        let advance = font.glyph_advance(Glyph::KeyboardPedalPed).unwrap() as f64;
        let expected_x = note_x - advance / 2.0;
        assert!(
            (layout.x - expected_x).abs() < 1e-6,
            "expected x={expected_x}, got {}",
            layout.x
        );
    }

    #[test]
    fn different_x_positions_produce_different_svg() {
        let (font, staff) = setup();

        let mut svg1 = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg1, &staff, &font, PedalMark::Down, 300.0).unwrap();

        let mut svg2 = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg2, &staff, &font, PedalMark::Down, 700.0).unwrap();

        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn transform_contains_translate() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg, &staff, &font, PedalMark::Down, 600.0).unwrap();
        let output = svg.to_svg();
        assert!(output.contains("translate("));
    }

    #[test]
    fn returned_glyph_matches_mark() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_pedal(&mut svg, &staff, &font, PedalMark::Up, 500.0).unwrap();
        assert_eq!(layout.glyph, Glyph::KeyboardPedalUp);
    }

    #[test]
    fn pedal_half_produces_single_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg, &staff, &font, PedalMark::Half, 500.0).unwrap();
        // Exactly one <path> — the Half glyph itself, no decoration.
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn pedal_sost_produces_single_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_pedal(&mut svg, &staff, &font, PedalMark::Sost, 500.0).unwrap();
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn pedal_half_returned_glyph_matches_mark() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_pedal(&mut svg, &staff, &font, PedalMark::Half, 500.0).unwrap();
        assert_eq!(layout.glyph, Glyph::KeyboardPedalHalf);
    }

    #[test]
    fn pedal_sost_returned_glyph_matches_mark() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_pedal(&mut svg, &staff, &font, PedalMark::Sost, 500.0).unwrap();
        assert_eq!(layout.glyph, Glyph::KeyboardPedalSost);
    }

    #[test]
    fn all_four_pedal_variants_produce_distinct_svg() {
        // Each pedal mark must render to a visually distinct SVG so a reader
        // can tell Down ("Ped."), Up ("*"), Half, and Sost ("Sost.") apart at
        // the same x-position. The glyph paths differ in d-attribute content.
        let (font, staff) = setup();
        let mut outputs = Vec::new();
        for mark in [
            PedalMark::Down,
            PedalMark::Up,
            PedalMark::Half,
            PedalMark::Sost,
        ] {
            let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
            draw_pedal(&mut svg, &staff, &font, mark, 500.0).unwrap();
            outputs.push((mark, svg.to_svg()));
        }
        for i in 0..outputs.len() {
            for j in (i + 1)..outputs.len() {
                assert_ne!(
                    outputs[i].1, outputs[j].1,
                    "PedalMark::{:?} and PedalMark::{:?} produced identical SVG",
                    outputs[i].0, outputs[j].0,
                );
            }
        }
    }

    #[test]
    fn pedal_half_centered_on_note() {
        // Half is a narrower glyph than Ped./Sost., so x = note - advance/2
        // must use the Half glyph's own advance, not the Down glyph's. This
        // catches a regression where the layout reused a hardcoded width.
        let (font, staff) = setup();
        let note_x = 750.0;
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_pedal(&mut svg, &staff, &font, PedalMark::Half, note_x).unwrap();
        let advance = font.glyph_advance(Glyph::KeyboardPedalHalf).unwrap() as f64;
        let expected_x = note_x - advance / 2.0;
        assert!(
            (layout.x - expected_x).abs() < 1e-6,
            "Half glyph must center using its own advance width; expected x={expected_x}, got {}",
            layout.x
        );
    }

    #[test]
    fn pedal_sost_centered_on_note() {
        let (font, staff) = setup();
        let note_x = 750.0;
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_pedal(&mut svg, &staff, &font, PedalMark::Sost, note_x).unwrap();
        let advance = font.glyph_advance(Glyph::KeyboardPedalSost).unwrap() as f64;
        let expected_x = note_x - advance / 2.0;
        assert!(
            (layout.x - expected_x).abs() < 1e-6,
            "Sost glyph must center using its own advance width; expected x={expected_x}, got {}",
            layout.x
        );
    }
}
