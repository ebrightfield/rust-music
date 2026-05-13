use crate::font::FontError;
use crate::font::MusicFont;
use crate::layout::articulation::ArticulationLayout;
use crate::render::SvgWriter;

/// Draw an articulation glyph at the position computed by `layout_articulation`.
///
/// Returns `Ok(())` after rendering the glyph path.
pub fn draw_articulation(
    writer: &mut SvgWriter,
    font: &MusicFont,
    layout: &ArticulationLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(layout.glyph)?;
    let transform = format!("translate({},{})", layout.x, layout.y);
    writer.add_path(&outline.path_data, "black", Some(&transform));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::articulation::{layout_articulation, Articulation};
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
    fn draw_staccato_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout =
            layout_articulation(Articulation::Staccato, 100.0, 4, StemDirection::Up, &staff);
        let mut writer = test_writer();
        draw_articulation(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
        assert!(svg.contains("translate("), "should have a translate transform");
    }

    #[test]
    fn draw_fermata_produces_path() {
        let font = test_font();
        let staff = test_staff();
        let layout =
            layout_articulation(Articulation::Fermata, 200.0, 8, StemDirection::Up, &staff);
        let mut writer = test_writer();
        draw_articulation(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(svg.contains("<path"), "should contain a path element");
    }

    #[test]
    fn different_articulations_produce_different_paths() {
        let font = test_font();
        let staff = test_staff();

        let staccato =
            layout_articulation(Articulation::Staccato, 100.0, 4, StemDirection::Up, &staff);
        let accent =
            layout_articulation(Articulation::Accent, 100.0, 4, StemDirection::Up, &staff);

        let mut w1 = test_writer();
        draw_articulation(&mut w1, &font, &staccato).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_articulation(&mut w2, &font, &accent).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(svg1, svg2, "staccato and accent should differ");
    }

    #[test]
    fn all_six_articulations_render_without_error() {
        let font = test_font();
        let staff = test_staff();
        let articulations = [
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
            Articulation::Fermata,
        ];
        for artic in &articulations {
            let layout = layout_articulation(*artic, 100.0, 4, StemDirection::Up, &staff);
            let mut writer = test_writer();
            let result = draw_articulation(&mut writer, &font, &layout);
            assert!(result.is_ok(), "{artic:?} should render without error");
            let svg = writer.to_svg();
            assert!(svg.contains("<path"), "{artic:?} should produce a path");
        }
    }

    #[test]
    fn all_fermata_duration_variants_render_distinct_paths() {
        // Every fermata duration variant must produce a path that visually
        // differs from the plain Fermata — otherwise the long/short/etc.
        // distinction would collapse to a single glyph.
        let font = test_font();
        let staff = test_staff();
        let variants = [
            Articulation::FermataLong,
            Articulation::FermataShort,
            Articulation::FermataVeryLong,
            Articulation::FermataVeryShort,
            Articulation::FermataHenzeLong,
            Articulation::FermataHenzeShort,
        ];

        let plain = layout_articulation(
            Articulation::Fermata,
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        let mut plain_writer = test_writer();
        draw_articulation(&mut plain_writer, &font, &plain).unwrap();
        let plain_svg = plain_writer.to_svg();
        let plain_d = path_d_data(&plain_svg);

        let mut all_paths: Vec<String> = vec![plain_d.clone()];
        for v in &variants {
            let layout = layout_articulation(*v, 100.0, 4, StemDirection::Up, &staff);
            let mut writer = test_writer();
            draw_articulation(&mut writer, &font, &layout).unwrap();
            let svg = writer.to_svg();
            assert!(svg.contains("<path"), "{v:?} should produce a path");
            let d = path_d_data(&svg);
            assert!(
                !d.is_empty(),
                "{v:?} path d-data should be non-empty",
            );
            assert_ne!(
                d, plain_d,
                "{v:?} path d-data should differ from plain Fermata",
            );
            all_paths.push(d);
        }

        // All 7 variants (plain + 6 duration) must produce 7 distinct path
        // d-strings — a regression here would mean two variants are mapping
        // to the same SMuFL glyph.
        for (i, a) in all_paths.iter().enumerate() {
            for (j, b) in all_paths.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        a, b,
                        "fermata variants {i} and {j} share path d-data",
                    );
                }
            }
        }
    }

    /// Extract the first `d="..."` payload from an SVG string. Returns ""
    /// if there is no path. Used to compare glyph outlines independently of
    /// the transform.
    fn path_d_data(svg: &str) -> String {
        let key = "d=\"";
        let Some(start) = svg.find(key) else {
            return String::new();
        };
        let after = &svg[start + key.len()..];
        let Some(end) = after.find('"') else {
            return String::new();
        };
        after[..end].to_string()
    }

    #[test]
    fn staccato_above_vs_below_produce_different_positions() {
        let font = test_font();
        let staff = test_staff();

        let above =
            layout_articulation(Articulation::Staccato, 100.0, 4, StemDirection::Down, &staff);
        let below =
            layout_articulation(Articulation::Staccato, 100.0, 4, StemDirection::Up, &staff);

        let mut w1 = test_writer();
        draw_articulation(&mut w1, &font, &above).unwrap();
        let svg1 = w1.to_svg();

        let mut w2 = test_writer();
        draw_articulation(&mut w2, &font, &below).unwrap();
        let svg2 = w2.to_svg();

        assert_ne!(
            svg1, svg2,
            "above and below placements should differ in transform"
        );
    }

    #[test]
    fn draw_articulation_embeds_x_coordinate() {
        let font = test_font();
        let staff = test_staff();
        let layout =
            layout_articulation(Articulation::Accent, 567.0, 4, StemDirection::Down, &staff);
        let mut writer = test_writer();
        draw_articulation(&mut writer, &font, &layout).unwrap();
        let svg = writer.to_svg();
        assert!(
            svg.contains("567"),
            "SVG should contain the x-coordinate 567"
        );
    }
}
