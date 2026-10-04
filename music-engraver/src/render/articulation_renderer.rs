use smufl::Glyph;

use crate::font::FontError;
use crate::font::MusicFont;
use crate::layout::articulation::{ArticulationKind, ArticulationLayout, ArticulationPlacement};
use crate::render::SvgWriter;

/// Gap between a parenthesized mark's ink and each parenthesis, in staff
/// spaces.
const MARK_PARENTHESIS_GAP_SS: f64 = 0.1;

/// Proportions of the horizontal stroke and suspended block in Schönberg's
/// broad mark (mn-c12-r009), relative to Bravura's tenuto and a staff space.
const BROAD_MARK_BAR_SCALE: f64 = 2.5;
const BROAD_MARK_BLOCK_WIDTH_SS: f64 = 1.4;
const BROAD_MARK_BLOCK_HEIGHT_SS: f64 = 0.22;

/// Draw an articulation glyph at the position computed by `layout_articulation`,
/// enclosed in parentheses when the layout asks for it.
///
/// Returns `Ok(())` after rendering the glyph path.
pub fn draw_articulation(
    writer: &mut SvgWriter,
    font: &MusicFont,
    layout: &ArticulationLayout,
) -> Result<(), FontError> {
    let outline = font.glyph_outline(layout.glyph)?;
    if layout.kind == ArticulationKind::BroadMark {
        let ss = f64::from(font.units_per_em()) / 4.0;
        let width = outline.advance_width as f64 * BROAD_MARK_BAR_SCALE;
        writer.add_path(
            &outline.path_data,
            "black",
            Some(&format!(
                "translate({},{}) scale({BROAD_MARK_BAR_SCALE},1)",
                layout.x - width / 2.0,
                layout.y
            )),
        );
        let block_width = BROAD_MARK_BLOCK_WIDTH_SS * ss;
        let block_height = BROAD_MARK_BLOCK_HEIGHT_SS * ss;
        let block_y = match layout.placement {
            ArticulationPlacement::Above => layout.y,
            ArticulationPlacement::Below => layout.y - block_height,
        };
        writer.add_rect(layout.x - block_width / 2.0, block_y, block_width, block_height, "black");
        if layout.parenthesized {
            draw_mark_parentheses_around(
                writer,
                font,
                layout.x - width / 2.0,
                layout.x + width / 2.0,
                layout.y + block_height / 2.0,
            )?;
        }
    } else {
        let transform = format!("translate({},{})", layout.x, layout.y);
        writer.add_path(&outline.path_data, "black", Some(&transform));
        if layout.parenthesized {
            draw_mark_parentheses(writer, font, layout.glyph, layout.x, layout.y)?;
        }
    }
    Ok(())
}

/// Enclose the mark `glyph`, drawn with its origin at `(x, y)`, in SMuFL
/// parentheses (`accidentalParensLeft` / `accidentalParensRight`), each
/// [`MARK_PARENTHESIS_GAP_SS`] clear of the glyph's bounding box and
/// vertically centred on it.
pub(crate) fn draw_mark_parentheses(
    writer: &mut SvgWriter,
    font: &MusicFont,
    glyph: Glyph,
    x: f64,
    y: f64,
) -> Result<(), FontError> {
    let (left, right, center) = match font.glyph_bbox_design_units(glyph) {
        Some(bbox) => (
            x + bbox.x_left,
            x + bbox.x_right,
            y + (bbox.y_top + bbox.y_bottom) / 2.0,
        ),
        None => (x, x + font.glyph_advance(glyph)? as f64, y),
    };
    draw_mark_parentheses_around(writer, font, left, right, center)
}

fn draw_mark_parentheses_around(
    writer: &mut SvgWriter,
    font: &MusicFont,
    left: f64,
    right: f64,
    center: f64,
) -> Result<(), FontError> {
    let gap = MARK_PARENTHESIS_GAP_SS * f64::from(font.units_per_em()) / 4.0;
    let open = font.glyph_outline(Glyph::AccidentalParensLeft)?;
    let open_x = left - gap - open.advance_width as f64;
    writer.add_path(
        &open.path_data,
        "black",
        Some(&format!("translate({open_x},{center})")),
    );
    let close = font.glyph_outline(Glyph::AccidentalParensRight)?;
    let close_x = right + gap;
    writer.add_path(
        &close.path_data,
        "black",
        Some(&format!("translate({close_x},{center})")),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::articulation::{layout_articulation, Articulation, ArticulationPlacement};
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
        assert!(
            svg.contains("translate("),
            "should have a translate transform"
        );
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
        let accent = layout_articulation(Articulation::Accent, 100.0, 4, StemDirection::Up, &staff);

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

        let plain = layout_articulation(Articulation::Fermata, 100.0, 4, StemDirection::Up, &staff);
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
            assert!(!d.is_empty(), "{v:?} path d-data should be non-empty",);
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
                    assert_ne!(a, b, "fermata variants {i} and {j} share path d-data",);
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

        let above = layout_articulation(
            Articulation::Staccato,
            100.0,
            4,
            StemDirection::Down,
            &staff,
        );
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

    #[test]
    fn up_bow_and_down_bow_produce_distinct_path_data() {
        // The two bow glyphs must render as visually distinct shapes —
        // otherwise the up/down distinction collapses to one symbol.
        let font = test_font();
        let staff = test_staff();
        let up_layout =
            layout_articulation(Articulation::UpBow, 100.0, 4, StemDirection::Up, &staff);
        let down_layout =
            layout_articulation(Articulation::DownBow, 100.0, 4, StemDirection::Up, &staff);

        let mut w_up = test_writer();
        draw_articulation(&mut w_up, &font, &up_layout).unwrap();
        let up_svg = w_up.to_svg();
        let up_d = path_d_data(&up_svg);

        let mut w_down = test_writer();
        draw_articulation(&mut w_down, &font, &down_layout).unwrap();
        let down_svg = w_down.to_svg();
        let down_d = path_d_data(&down_svg);

        assert!(!up_d.is_empty(), "up-bow d-data should be non-empty");
        assert!(!down_d.is_empty(), "down-bow d-data should be non-empty");
        assert_ne!(up_d, down_d, "up-bow and down-bow paths must differ");
    }

    #[test]
    fn all_four_combined_variants_render_distinct_paths() {
        // Each combined-articulation (AccentStaccato, MarcatoStaccato,
        // TenutoStaccato, TenutoAccent) must render as a distinct shape.
        // Also: each must differ from its visual "constituents" (e.g.
        // AccentStaccato differs from both Accent and Staccato), otherwise
        // a wiring typo could collapse the combined glyph onto a simple one.
        let font = test_font();
        let staff = test_staff();
        let combined = [
            Articulation::AccentStaccato,
            Articulation::MarcatoStaccato,
            Articulation::TenutoStaccato,
            Articulation::TenutoAccent,
        ];
        let mut combined_paths: Vec<(Articulation, String)> = Vec::new();
        for c in combined {
            let layout = layout_articulation(c, 100.0, 4, StemDirection::Up, &staff);
            let mut writer = test_writer();
            draw_articulation(&mut writer, &font, &layout).unwrap();
            let d = path_d_data(&writer.to_svg());
            assert!(!d.is_empty(), "{c:?} should produce non-empty path data");
            combined_paths.push((c, d));
        }

        // Pairwise distinct among the 4 combined variants.
        for (i, (ai, di)) in combined_paths.iter().enumerate() {
            for (j, (aj, dj)) in combined_paths.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        di, dj,
                        "{ai:?} and {aj:?} share path data — combined glyphs must be visually distinct",
                    );
                }
            }
        }

        // Each combined must differ from every simple constituent glyph.
        // This catches a typo that would map e.g. TenutoAccent → ArticAccentBelow.
        for simple in [
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
        ] {
            let layout = layout_articulation(simple, 100.0, 4, StemDirection::Up, &staff);
            let mut writer = test_writer();
            draw_articulation(&mut writer, &font, &layout).unwrap();
            let simple_d = path_d_data(&writer.to_svg());
            for (c, cd) in &combined_paths {
                assert_ne!(
                    cd, &simple_d,
                    "{c:?} path must differ from simple {simple:?}",
                );
            }
        }
    }

    #[test]
    fn combined_variants_render_above_and_below_distinctly() {
        // Above-variant and Below-variant of each combined articulation must
        // render as different path data — Bravura supplies a real
        // above-below pair, not a vertical flip applied at draw time.
        let font = test_font();
        let staff = test_staff();
        for c in [
            Articulation::AccentStaccato,
            Articulation::MarcatoStaccato,
            Articulation::TenutoStaccato,
            Articulation::TenutoAccent,
        ] {
            // Stem-up → Below glyph; stem-down → Above glyph.
            let below = layout_articulation(c, 100.0, 4, StemDirection::Up, &staff);
            let above = layout_articulation(c, 100.0, 4, StemDirection::Down, &staff);
            assert_eq!(below.placement, ArticulationPlacement::Below);
            assert_eq!(above.placement, ArticulationPlacement::Above);

            let mut wb = test_writer();
            draw_articulation(&mut wb, &font, &below).unwrap();
            let db = path_d_data(&wb.to_svg());

            let mut wa = test_writer();
            draw_articulation(&mut wa, &font, &above).unwrap();
            let da = path_d_data(&wa.to_svg());

            assert!(!da.is_empty(), "{c:?} above path empty");
            assert!(!db.is_empty(), "{c:?} below path empty");
            assert_ne!(da, db, "{c:?} above/below path data should differ");
        }
    }

    #[test]
    fn accent_extensions_render_distinct_paths_from_plain_accent_and_each_other() {
        // The three SMuFL accent extensions (SoftAccent, Stress, Unstress)
        // must each render as a unique shape, distinct from one another and
        // from the plain Accent — otherwise a Glyph wiring regression would
        // collapse multiple variants to the same visual.
        let font = test_font();
        let staff = test_staff();
        let extensions = [
            Articulation::SoftAccent,
            Articulation::Stress,
            Articulation::Unstress,
        ];

        // Capture plain-Accent path for cross-comparison.
        let plain_layout =
            layout_articulation(Articulation::Accent, 100.0, 4, StemDirection::Up, &staff);
        let mut plain_writer = test_writer();
        draw_articulation(&mut plain_writer, &font, &plain_layout).unwrap();
        let plain_d = path_d_data(&plain_writer.to_svg());
        assert!(!plain_d.is_empty(), "plain Accent path d-data empty");

        // Capture each extension's path on the Below side (stem-up default).
        let mut ext_paths: Vec<(Articulation, String)> = Vec::new();
        for &a in &extensions {
            let layout = layout_articulation(a, 100.0, 4, StemDirection::Up, &staff);
            assert_eq!(
                layout.placement,
                ArticulationPlacement::Below,
                "{a:?} stem-up default must be Below",
            );
            let mut writer = test_writer();
            draw_articulation(&mut writer, &font, &layout).unwrap();
            let d = path_d_data(&writer.to_svg());
            assert!(!d.is_empty(), "{a:?} below path d-data should be non-empty");
            assert_ne!(
                d, plain_d,
                "{a:?} below path must differ from plain Accent below path",
            );
            ext_paths.push((a, d));
        }

        // Pairwise distinct among the 3 extensions.
        for (i, (ai, di)) in ext_paths.iter().enumerate() {
            for (j, (aj, dj)) in ext_paths.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        di, dj,
                        "{ai:?} and {aj:?} share path data — extensions must be visually distinct",
                    );
                }
            }
        }

        // Each extension must also produce a distinct Above-glyph (the
        // stem-down case) — and that Above path must differ from its own
        // Below path, since Bravura ships a real above/below pair (not a
        // flip applied at draw time).
        for &a in &extensions {
            let above = layout_articulation(a, 100.0, 4, StemDirection::Down, &staff);
            let below = layout_articulation(a, 100.0, 4, StemDirection::Up, &staff);
            assert_eq!(above.placement, ArticulationPlacement::Above);
            assert_eq!(below.placement, ArticulationPlacement::Below);

            let mut wa = test_writer();
            draw_articulation(&mut wa, &font, &above).unwrap();
            let da = path_d_data(&wa.to_svg());

            let mut wb = test_writer();
            draw_articulation(&mut wb, &font, &below).unwrap();
            let db = path_d_data(&wb.to_svg());

            assert!(!da.is_empty(), "{a:?} above path empty");
            assert!(!db.is_empty(), "{a:?} below path empty");
            assert_ne!(da, db, "{a:?} above/below path data should differ");
        }
    }

    #[test]
    fn laissez_vibrer_renders_distinct_paths_above_below_and_from_other_articulations() {
        // The LaissezVibrer ("l.v.") variant must render through the
        // Bravura outline extractor as:
        //   (a) non-empty path data for both Above and Below placements,
        //   (b) different path d-data between Above and Below — Bravura
        //       ships a real above/below pair, not a draw-time flip,
        //   (c) different path d-data from every other articulation in
        //       the enum (both placements) — a Glyph-wiring typo could
        //       otherwise silently alias l.v. onto, e.g., a tenuto line
        //       or an accent.
        let font = test_font();
        let staff = test_staff();

        // Capture l.v. Below (stem-up default) and Above (stem-down) paths.
        let below_layout = layout_articulation(
            Articulation::LaissezVibrer,
            100.0,
            4,
            StemDirection::Up,
            &staff,
        );
        let above_layout = layout_articulation(
            Articulation::LaissezVibrer,
            100.0,
            4,
            StemDirection::Down,
            &staff,
        );
        assert_eq!(below_layout.placement, ArticulationPlacement::Below);
        assert_eq!(above_layout.placement, ArticulationPlacement::Above);

        let mut wb = test_writer();
        draw_articulation(&mut wb, &font, &below_layout).unwrap();
        let lv_below_d = path_d_data(&wb.to_svg());

        let mut wa = test_writer();
        draw_articulation(&mut wa, &font, &above_layout).unwrap();
        let lv_above_d = path_d_data(&wa.to_svg());

        // (a) Both paths are non-empty.
        assert!(!lv_above_d.is_empty(), "l.v. above path d-data is empty");
        assert!(!lv_below_d.is_empty(), "l.v. below path d-data is empty");
        // (b) Above and below differ.
        assert_ne!(
            lv_above_d, lv_below_d,
            "l.v. above and below path d-data should differ",
        );

        // (c) Path data differs from every other articulation, both
        // placements. Sweeps the full enum (minus l.v. itself) and asserts
        // no aliasing. This is the broadest possible regression net for
        // the glyph-wiring change.
        let others: &[Articulation] = &[
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
            Articulation::Fermata,
            Articulation::FermataLong,
            Articulation::FermataShort,
            Articulation::FermataVeryLong,
            Articulation::FermataVeryShort,
            Articulation::FermataHenzeLong,
            Articulation::FermataHenzeShort,
            Articulation::UpBow,
            Articulation::DownBow,
            Articulation::AccentStaccato,
            Articulation::MarcatoStaccato,
            Articulation::TenutoStaccato,
            Articulation::TenutoAccent,
            Articulation::SoftAccent,
            Articulation::Stress,
            Articulation::Unstress,
        ];
        for &o in others {
            // Below (stem-up) comparison.
            let below_other = layout_articulation(o, 100.0, 4, StemDirection::Up, &staff);
            let mut wo_b = test_writer();
            draw_articulation(&mut wo_b, &font, &below_other).unwrap();
            let other_below_d = path_d_data(&wo_b.to_svg());
            assert_ne!(
                lv_below_d, other_below_d,
                "l.v.-below path must differ from {o:?} (stem-up render)",
            );
            // Above (stem-down) comparison. Bow strokes ignore the
            // placement arg in glyph(), but the layout still routes them
            // through their single SMuFL glyph, so the comparison remains
            // meaningful (and the bow-glyph d-data must still differ from
            // l.v.-above).
            let above_other = layout_articulation(o, 100.0, 4, StemDirection::Down, &staff);
            let mut wo_a = test_writer();
            draw_articulation(&mut wo_a, &font, &above_other).unwrap();
            let other_above_d = path_d_data(&wo_a.to_svg());
            assert_ne!(
                lv_above_d, other_above_d,
                "l.v.-above path must differ from {o:?} (stem-down render)",
            );
        }
    }

    #[test]
    fn bow_stroke_path_differs_from_articulation_glyphs() {
        // Bow strokes must not silently alias to any standard articulation
        // path — a Glyph wiring regression would otherwise be invisible.
        let font = test_font();
        let staff = test_staff();
        let bow_layout =
            layout_articulation(Articulation::UpBow, 100.0, 4, StemDirection::Up, &staff);
        let mut bow_writer = test_writer();
        draw_articulation(&mut bow_writer, &font, &bow_layout).unwrap();
        let bow_d = path_d_data(&bow_writer.to_svg());
        assert!(!bow_d.is_empty());

        for artic in [
            Articulation::Staccato,
            Articulation::Tenuto,
            Articulation::Accent,
            Articulation::Marcato,
            Articulation::Staccatissimo,
            Articulation::Fermata,
        ] {
            let layout = layout_articulation(artic, 100.0, 4, StemDirection::Up, &staff);
            let mut writer = test_writer();
            draw_articulation(&mut writer, &font, &layout).unwrap();
            let d = path_d_data(&writer.to_svg());
            assert_ne!(
                d, bow_d,
                "bow-stroke path should differ from {artic:?} path",
            );
        }
    }
}
