use crate::font::{FontError, MusicFont};
use crate::layout::dynamics::{layout_dynamic, Dynamic, DynamicLayout};
use crate::layout::staff::StaffLayout;
use crate::render::SvgWriter;

/// Draw a dynamic marking below the staff, centered on `note_center_x`.
///
/// Returns the `DynamicLayout` describing where the glyph was placed,
/// or a `FontError` if the glyph cannot be resolved.
pub fn draw_dynamic(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    font: &MusicFont,
    dynamic: Dynamic,
    note_center_x: f64,
) -> Result<DynamicLayout, FontError> {
    let outline = font.glyph_outline(dynamic.glyph())?;
    let advance_width = outline.advance_width as f64;

    let layout = layout_dynamic(
        dynamic,
        note_center_x,
        advance_width,
        staff.bottom_y(),
        staff.staff_space,
    );

    let transform = format!("translate({}, {})", layout.x, layout.y);
    svg.add_path(&outline.path_data, "black", Some(&transform));

    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::dynamics::DYNAMICS_BELOW_STAFF_SS;
    use smufl::Glyph;

    fn setup() -> (MusicFont<'static>, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (font, staff)
    }

    #[test]
    fn forte_produces_single_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let result = draw_dynamic(&mut svg, &staff, &font, Dynamic::Forte, 500.0);
        assert!(result.is_ok());
        let output = svg.to_svg();
        assert_eq!(output.matches("<path ").count(), 1);
    }

    #[test]
    fn piano_produces_single_path() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_dynamic(&mut svg, &staff, &font, Dynamic::Piano, 500.0).unwrap();
        assert_eq!(svg.to_svg().matches("<path ").count(), 1);
    }

    #[test]
    fn different_dynamics_produce_different_paths() {
        let (font, staff) = setup();

        let mut svg_f = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_dynamic(&mut svg_f, &staff, &font, Dynamic::Forte, 500.0).unwrap();

        let mut svg_p = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_dynamic(&mut svg_p, &staff, &font, Dynamic::Piano, 500.0).unwrap();

        assert_ne!(svg_f.to_svg(), svg_p.to_svg());
    }

    #[test]
    fn layout_y_is_below_staff_bottom() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_dynamic(&mut svg, &staff, &font, Dynamic::Mf, 500.0).unwrap();
        let expected_y = staff.bottom_y() + DYNAMICS_BELOW_STAFF_SS * staff.staff_space;
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
        let layout = draw_dynamic(&mut svg, &staff, &font, Dynamic::Ff, note_x).unwrap();

        let advance = font.glyph_advance(Glyph::DynamicFf).unwrap() as f64;
        let expected_x = note_x - advance / 2.0;
        assert!(
            (layout.x - expected_x).abs() < 1e-6,
            "expected x={expected_x}, got {}",
            layout.x
        );
    }

    #[test]
    fn all_dynamics_render_without_error() {
        let (font, staff) = setup();
        for dyn_mark in &Dynamic::ALL {
            let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
            let result = draw_dynamic(&mut svg, &staff, &font, *dyn_mark, 500.0);
            assert!(result.is_ok(), "{:?} failed to render", dyn_mark);
            assert_eq!(
                svg.to_svg().matches("<path ").count(),
                1,
                "{:?} should produce exactly 1 path",
                dyn_mark
            );
        }
    }

    #[test]
    fn all_dynamics_produce_distinct_svg_output() {
        // Each variant must produce a visually distinct SVG (different
        // glyph path data). If two variants accidentally collapsed to the
        // same glyph this catches it at the render layer too.
        let (font, staff) = setup();
        let mut seen: Vec<String> = Vec::new();
        for dyn_mark in &Dynamic::ALL {
            let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
            draw_dynamic(&mut svg, &staff, &font, *dyn_mark, 500.0).unwrap();
            let output = svg.to_svg();
            assert!(
                !seen.contains(&output),
                "{:?} produced SVG identical to a prior variant",
                dyn_mark
            );
            seen.push(output);
        }
        assert_eq!(seen.len(), Dynamic::ALL.len());
    }

    #[test]
    fn extreme_dynamics_render_via_smufl_advance() {
        // Bravura includes glyphs for all six p/f levels. If any of these
        // lookups fail, the font is missing a glyph we claim to support.
        let (font, _) = setup();
        for g in [
            Glyph::DynamicPppppp,
            Glyph::DynamicPpppp,
            Glyph::DynamicPppp,
            Glyph::DynamicFfff,
            Glyph::DynamicFffff,
            Glyph::DynamicFfffff,
        ] {
            let adv = font
                .glyph_advance(g)
                .unwrap_or_else(|_| panic!("Bravura missing glyph {:?}", g));
            assert!(adv > 0, "{:?} should have nonzero advance width", g);
        }
    }

    #[test]
    fn different_x_positions_produce_different_svg() {
        let (font, staff) = setup();

        let mut svg1 = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_dynamic(&mut svg1, &staff, &font, Dynamic::Forte, 300.0).unwrap();

        let mut svg2 = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        draw_dynamic(&mut svg2, &staff, &font, Dynamic::Forte, 700.0).unwrap();

        assert_ne!(svg1.to_svg(), svg2.to_svg());
    }

    #[test]
    fn transform_contains_layout_coordinates() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_dynamic(&mut svg, &staff, &font, Dynamic::Piano, 600.0).unwrap();
        let output = svg.to_svg();
        // The transform should contain the x and y from the layout
        assert!(output.contains("translate("));
        assert!(output.contains(&format!("{}", layout.x)));
    }

    #[test]
    fn returned_glyph_matches_dynamic() {
        let (font, staff) = setup();
        let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
        let layout = draw_dynamic(&mut svg, &staff, &font, Dynamic::Pp, 500.0).unwrap();
        assert_eq!(layout.glyph, Glyph::DynamicPp);
    }

    /// Extract the first `d="..."` payload from an SVG string, panicking
    /// if absent. Used by the per-variant distinct-path assertions.
    fn first_path_d(svg: &str) -> String {
        let start = svg.find("d=\"").expect("svg has no path d attribute") + 3;
        let rest = &svg[start..];
        let end = rest.find('"').expect("unterminated d attribute");
        rest[..end].to_string()
    }

    #[test]
    fn new_variants_render_distinct_path_data_in_bravura() {
        // Render each of the three new variants and assert their path
        // d-strings are pairwise distinct AND distinct from a representative
        // neighbour glyph. Catches a hypothetical regression where one of
        // the new variants silently picks up the same Bravura outline as
        // an existing glyph (e.g. if a glyph mapping typo collapses
        // SforzatoPiano onto SforzandoPiano).
        let (font, staff) = setup();
        let render = |dyn_mark: Dynamic| -> String {
            let mut svg = SvgWriter::new(800.0, 300.0, -500.0, -200.0, 6000.0, 2000.0);
            draw_dynamic(&mut svg, &staff, &font, dyn_mark, 500.0).unwrap();
            first_path_d(&svg.to_svg())
        };
        let mezzo = render(Dynamic::Mezzo);
        let z = render(Dynamic::Z);
        let sforzato_piano = render(Dynamic::SforzatoPiano);
        // Pairwise distinct among the new trio.
        assert_ne!(mezzo, z);
        assert_ne!(mezzo, sforzato_piano);
        assert_ne!(z, sforzato_piano);
        // Distinct from each new variant's closest existing neighbour.
        assert_ne!(mezzo, render(Dynamic::Mp));
        assert_ne!(mezzo, render(Dynamic::Mf));
        assert_ne!(z, render(Dynamic::Sfz));
        assert_ne!(z, render(Dynamic::Fz));
        assert_ne!(sforzato_piano, render(Dynamic::Sfp));
        assert_ne!(sforzato_piano, render(Dynamic::Sf));
    }

    #[test]
    fn new_variants_have_nonzero_advance_in_bravura() {
        // Guards against silent "glyph not in font" failures when a future
        // SMuFL font swap omits one of these less-common glyphs.
        let (font, _) = setup();
        for g in [
            Glyph::DynamicMezzo,
            Glyph::DynamicZ,
            Glyph::DynamicSforzatoPiano,
        ] {
            let adv = font
                .glyph_advance(g)
                .unwrap_or_else(|_| panic!("Bravura missing glyph {:?}", g));
            assert!(adv > 0, "{:?} should have nonzero advance width", g);
        }
    }
}
