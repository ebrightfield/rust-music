use std::fmt::Write;

use crate::layout::slur::SlurLayout;
use crate::render::SvgWriter;

/// Build SVG path data for a slur (crescent shape from two cubic Bézier curves).
///
/// Unlike ties, slurs may have different start/end y-positions since they
/// connect notes at different pitches.
fn slur_path_data(layout: &SlurLayout) -> String {
    let mut d = String::with_capacity(256);

    // Move to start point
    let _ = write!(d, "M{:.2},{:.2}", layout.x_start, layout.y_start);

    // Outer curve: start → end
    let _ = write!(
        d,
        " C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
        layout.outer_cp1.0,
        layout.outer_cp1.1,
        layout.outer_cp2.0,
        layout.outer_cp2.1,
        layout.x_end,
        layout.y_end,
    );

    // Inner curve: end → start (reversed control points)
    let _ = write!(
        d,
        " C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
        layout.inner_cp1.0,
        layout.inner_cp1.1,
        layout.inner_cp2.0,
        layout.inner_cp2.1,
        layout.x_start,
        layout.y_start,
    );

    d.push('Z');
    d
}

/// Draw a slur as a filled crescent shape.
///
/// Returns the path data string for testing purposes.
pub fn draw_slur(svg: &mut SvgWriter, layout: &SlurLayout) -> String {
    let path_data = slur_path_data(layout);
    svg.add_filled_path(&path_data, "black");
    path_data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::EngravingConfig;
    use crate::layout::slur::{layout_slur, SlurDirection};

    fn test_config() -> EngravingConfig {
        let metadata: smufl::Metadata =
            serde_json::from_slice(crate::font::BRAVURA_METADATA).unwrap();
        EngravingConfig::from_smufl(&metadata.engraving_defaults, 1000)
    }

    fn make_slur(direction: SlurDirection) -> SlurLayout {
        let cfg = test_config();
        layout_slur(100.0, 500.0, 400.0, 400.0, direction, &cfg)
    }

    fn make_asymmetric_slur() -> SlurLayout {
        let cfg = test_config();
        layout_slur(100.0, 600.0, 300.0, 500.0, SlurDirection::Over, &cfg)
    }

    #[test]
    fn path_starts_with_move_to() {
        let layout = make_slur(SlurDirection::Over);
        let path = slur_path_data(&layout);
        assert!(path.starts_with("M100.00,"));
    }

    #[test]
    fn path_has_two_cubic_beziers_and_close() {
        let layout = make_slur(SlurDirection::Under);
        let path = slur_path_data(&layout);
        let c_count = path.matches(" C").count();
        assert_eq!(c_count, 2, "expected 2 cubic Bézier commands, got {c_count}");
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn draw_slur_adds_filled_path_element() {
        let layout = make_slur(SlurDirection::Over);
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 1000.0, 500.0);
        draw_slur(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<path "));
        assert!(output.contains(r#"fill="black""#));
        assert!(output.contains(r#"stroke="none""#));
    }

    #[test]
    fn draw_slur_returns_path_data() {
        let layout = make_slur(SlurDirection::Over);
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 1000.0, 500.0);
        let path = draw_slur(&mut svg, &layout);
        assert!(path.starts_with('M'));
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn over_and_under_produce_different_paths() {
        let over_path = slur_path_data(&make_slur(SlurDirection::Over));
        let under_path = slur_path_data(&make_slur(SlurDirection::Under));
        assert_ne!(over_path, under_path);
    }

    #[test]
    fn asymmetric_slur_has_different_start_end_y() {
        let layout = make_asymmetric_slur();
        let path = slur_path_data(&layout);
        // The path should start and end at different y values
        assert!(
            (layout.y_start - layout.y_end).abs() > 1.0,
            "Asymmetric slur should have different start/end y"
        );
        // Verify both x endpoints appear in path
        assert!(path.contains("100.00"));
        assert!(path.contains("600.00"));
    }

    #[test]
    fn path_contains_endpoint_coordinates() {
        let layout = make_slur(SlurDirection::Over);
        let path = slur_path_data(&layout);
        assert!(path.contains("100.00"));
        assert!(path.contains("500.00"));
    }

    #[test]
    fn short_slur_path_is_valid() {
        let cfg = test_config();
        let layout = layout_slur(200.0, 260.0, 300.0, 320.0, SlurDirection::Under, &cfg);
        let path = slur_path_data(&layout);
        assert!(path.starts_with('M'));
        assert!(path.ends_with('Z'));
        assert_eq!(path.matches(" C").count(), 2);
    }

    #[test]
    fn long_slur_path_is_valid() {
        let cfg = test_config();
        let layout = layout_slur(50.0, 3000.0, 400.0, 600.0, SlurDirection::Over, &cfg);
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 3500.0, 800.0);
        let path = draw_slur(&mut svg, &layout);
        assert!(path.starts_with('M'));
        assert!(path.ends_with('Z'));
        assert_eq!(path.matches(" C").count(), 2);
    }
}
