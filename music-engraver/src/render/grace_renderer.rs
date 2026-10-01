use crate::font::{FontError, MusicFont};
use crate::layout::grace::GraceNoteLayout;
use crate::render::SvgWriter;

/// Draw a grace note glyph at the computed position with the appropriate scale.
///
/// The SMuFL composite grace note glyph includes notehead, stem, flag, and
/// (for acciaccatura) the slash — all rendered as a single scaled path.
pub fn draw_grace_note(
    writer: &mut SvgWriter,
    font: &MusicFont,
    layout: &GraceNoteLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(layout.glyph)?;

    // Apply both translation and scale. The grace note glyph is rendered at
    // GRACE_NOTE_SCALE size, centered on the glyph origin.
    let transform = format!(
        "translate({},{}) scale({},{})",
        layout.x, layout.y, layout.scale, layout.scale
    );
    writer.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::grace::{layout_grace_note, GraceNoteKind};
    use crate::layout::staff::StaffLayout;
    use crate::layout::stem::StemDirection;

    fn test_font() -> MusicFont<'static> {
        bravura_font()
    }

    fn test_staff() -> StaffLayout {
        let font = test_font();
        let config = font.engraving_config();
        StaffLayout::from_config(0.0, 0.0, 5000.0, &config)
    }

    fn test_writer() -> SvgWriter {
        SvgWriter::new(200.0, 100.0, 0.0, 0.0, 1000.0, 500.0)
    }

    #[test]
    fn draw_acciaccatura_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_grace_note(
            500.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let mut writer = test_writer();
        draw_grace_note(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
        assert!(
            svg.contains("translate("),
            "should have translate transform"
        );
        assert!(svg.contains("scale("), "should have scale transform");
    }

    #[test]
    fn draw_appoggiatura_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_grace_note(
            500.0,
            4,
            GraceNoteKind::Appoggiatura,
            StemDirection::Down,
            &staff,
        );
        let mut writer = test_writer();
        draw_grace_note(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
    }

    #[test]
    fn acciaccatura_and_appoggiatura_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let acc_layout = layout_grace_note(
            500.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let app_layout = layout_grace_note(
            500.0,
            4,
            GraceNoteKind::Appoggiatura,
            StemDirection::Up,
            &staff,
        );

        let mut w1 = test_writer();
        draw_grace_note(&mut w1, &font, &acc_layout).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_grace_note(&mut w2, &font, &app_layout).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(svg1, svg2, "acciaccatura and appoggiatura should differ");
    }

    #[test]
    fn stem_up_and_down_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let up_layout = layout_grace_note(
            500.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let down_layout = layout_grace_note(
            500.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Down,
            &staff,
        );

        let mut w1 = test_writer();
        draw_grace_note(&mut w1, &font, &up_layout).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_grace_note(&mut w2, &font, &down_layout).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(
            svg1, svg2,
            "stem up and down should produce different glyphs"
        );
    }

    #[test]
    fn scale_factor_embedded_in_transform() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_grace_note(
            500.0,
            4,
            GraceNoteKind::Acciaccatura,
            StemDirection::Up,
            &staff,
        );
        let mut writer = test_writer();
        draw_grace_note(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains("scale(0.6,0.6)"),
            "should contain scale(0.6,0.6) in transform"
        );
    }

    #[test]
    fn x_coordinate_embedded_in_transform() {
        let font = test_font();
        let staff = test_staff();
        let layout = layout_grace_note(
            1234.0,
            4,
            GraceNoteKind::Appoggiatura,
            StemDirection::Up,
            &staff,
        );
        let mut writer = test_writer();
        draw_grace_note(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        // The x coordinate of the grace note should appear in the SVG.
        // The x is to the left of 1234, so it will be a positive number
        // starting with digits in the hundreds range.
        let x_prefix = format!("{:.0}", layout.x);
        assert!(
            svg.contains(&x_prefix),
            "SVG should contain grace note x-coordinate {x_prefix}"
        );
    }

    #[test]
    fn all_four_variants_render_without_error() {
        let font = test_font();
        let staff = test_staff();
        let kinds = [GraceNoteKind::Acciaccatura, GraceNoteKind::Appoggiatura];
        let dirs = [StemDirection::Up, StemDirection::Down];

        for kind in &kinds {
            for dir in &dirs {
                let layout = layout_grace_note(500.0, 4, *kind, *dir, &staff);
                let mut writer = test_writer();
                let result = draw_grace_note(&mut writer, &font, &layout);
                assert!(
                    result.is_ok(),
                    "{kind:?} {dir:?} should render without error"
                );
                let svg = writer.to_svg();
                assert!(
                    svg.contains("<path"),
                    "{kind:?} {dir:?} should produce a path"
                );
            }
        }
    }
}
