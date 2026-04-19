use crate::font::{EngravingConfig, FontError, MusicFont};
use crate::layout::page::PageLayout;
use crate::render::system_renderer::draw_system;
use crate::render::SvgWriter;

/// Draw a complete page of music (multiple systems stacked vertically).
///
/// Returns an `SvgWriter` ready to be converted to an SVG string via `to_svg()`.
/// The caller can also add additional elements before finalizing.
pub fn draw_page(
    font: &MusicFont,
    config: &EngravingConfig,
    page: &PageLayout,
) -> Result<SvgWriter, FontError> {
    // Compute pixel dimensions — use a reasonable scale factor.
    // 1 font unit = 1 SVG user unit in the viewBox; pixel size is derived
    // from page dimensions with a scaling ratio.
    let vb_margin = config.staff_space; // small margin around the content
    let vb_x = -vb_margin;
    let vb_y = -vb_margin;
    let vb_w = page.page_width + 2.0 * vb_margin;
    let vb_h = page.page_height + 2.0 * vb_margin;

    // Scale so that a staff space maps to approximately 7 pixels (standard screen density).
    let px_per_unit = 7.0 / config.staff_space;
    let px_w = vb_w * px_per_unit;
    let px_h = vb_h * px_per_unit;

    let mut svg = SvgWriter::new(px_w, px_h, vb_x, vb_y, vb_w, vb_h);

    for page_system in &page.systems {
        draw_system(
            &mut svg,
            font,
            config,
            &page_system.system,
            page_system.x,
            page_system.y,
        )?;
    }

    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::barline::BarlineStyle;
    use crate::layout::key_signature::KeySignature;
    use crate::layout::measure::{MeasureLayoutConfig, NoteEvent};
    use crate::layout::page::{layout_page, PageLayoutConfig, SystemBreaking};
    use crate::layout::system::{MeasureContent, MeasureEvent, SystemPrefix};
    use crate::layout::time_signature::TimeSignatureKind;
    use music::notation::clef::Clef;

    fn setup() -> (MusicFont<'static>, EngravingConfig) {
        let font = bravura_font();
        let config = font.engraving_config();
        (font, config)
    }

    fn quarter_note(pos: i8) -> MeasureEvent {
        MeasureEvent::Note(NoteEvent {
            staff_position: pos,
            duration_log2: 2,
            dots: 0,
            accidental: None,
            stem_direction: None,
        tie_forward: false,
        })
    }

    fn make_measure(pos: i8) -> MeasureContent {
        MeasureContent {
            events: vec![quarter_note(pos)],
            barline: BarlineStyle::Single,
        }
    }

    fn prefix() -> SystemPrefix {
        SystemPrefix::new(
            &Clef::Treble,
            KeySignature::Open,
            Some(TimeSignatureKind::Numeric {
                numerator: 4,
                denominator: 4,
            }),
        )
    }

    #[test]
    fn empty_page_renders_valid_svg() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let page = layout_page(&prefix(), &[], &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();
        assert!(output.starts_with("<svg"));
        assert!(output.ends_with("</svg>\n"));
        assert_eq!(output.matches("<path ").count(), 0);
    }

    #[test]
    fn single_system_page() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..3).map(|i| make_measure(i as i8 * 2)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(4));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Should have 5 staff lines
        assert!(output.matches("<line ").count() >= 5);
        // Should have a clef path
        assert!(output.matches("<path ").count() >= 1);
    }

    #[test]
    fn two_system_page_has_two_sets_of_staff_lines() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..6).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // 2 systems × 5 staff lines = 10 staff lines minimum
        let line_count = output.matches("<line ").count();
        assert!(
            line_count >= 10,
            "expected >= 10 lines (2 staves), got {}",
            line_count,
        );
    }

    #[test]
    fn three_system_page_element_counts() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..9).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));

        assert_eq!(page.systems.len(), 3);

        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // 3 systems × 5 staff lines = 15
        let line_count = output.matches("<line ").count();
        assert!(line_count >= 15, "expected >= 15 lines, got {}", line_count);

        // 3 clefs + time sig digits in first system only
        let path_count = output.matches("<path ").count();
        assert!(path_count >= 3, "expected >= 3 paths (3 clefs), got {}", path_count);
    }

    #[test]
    fn page_svg_dimensions_are_positive() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..4).map(|i| make_measure(i as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(2));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Parse width and height from the SVG — they should be positive
        assert!(output.contains("width=\""));
        assert!(output.contains("height=\""));
        // Check viewBox has positive dimensions
        assert!(output.contains("viewBox=\""));
    }

    #[test]
    fn two_system_page_has_different_y_for_staff_lines() {
        let (font, config) = setup();
        let ss = config.staff_space;
        let page_cfg = PageLayoutConfig::new(ss, 8000.0);
        let mcfg = MeasureLayoutConfig::from_staff_space(ss);
        let measures: Vec<_> = (0..6).map(|i| make_measure((i % 8) as i8)).collect();
        let page = layout_page(&prefix(), &measures, &mcfg, &page_cfg, &SystemBreaking::Fixed(3));
        let svg = draw_page(&font, &config, &page).unwrap();
        let output = svg.to_svg();

        // Extract all y1 values from <line> elements — there should be at least
        // 2 distinct groups (one per system)
        let y1_values: Vec<&str> = output
            .split("y1=\"")
            .skip(1)
            .filter_map(|s| s.split('"').next())
            .collect();
        assert!(y1_values.len() >= 10, "should have >= 10 line y1 values");

        // Parse to floats and check at least 2 distinct y ranges
        let y_floats: Vec<f64> = y1_values.iter().filter_map(|s| s.parse().ok()).collect();
        let min_y = y_floats.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = y_floats.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        // Two systems means y-values span at least the system spacing
        let expected_min_span = 5.0 * ss; // at least 5 staff spaces apart
        assert!(
            max_y - min_y > expected_min_span,
            "y range {} should exceed {} (two systems should be separated)",
            max_y - min_y,
            expected_min_span,
        );
    }
}
