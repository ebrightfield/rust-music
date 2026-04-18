use std::fmt::Write;

use crate::layout::tie::TieLayout;
use crate::render::SvgWriter;

/// Build SVG path data for a tie (crescent shape from two cubic Bézier curves).
///
/// The outer curve goes from start → end (forward), the inner curve returns
/// from end → start, forming a closed filled shape.
fn tie_path_data(layout: &TieLayout) -> String {
    let mut d = String::with_capacity(256);

    // Move to start point
    let _ = write!(d, "M{:.2},{:.2}", layout.x_start, layout.y_endpoint);

    // Outer curve: start → end
    let _ = write!(
        d,
        " C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
        layout.outer_cp1.0,
        layout.outer_cp1.1,
        layout.outer_cp2.0,
        layout.outer_cp2.1,
        layout.x_end,
        layout.y_endpoint,
    );

    // Inner curve: end → start (reversed control points, closer to note)
    let _ = write!(
        d,
        " C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
        layout.inner_cp1.0,
        layout.inner_cp1.1,
        layout.inner_cp2.0,
        layout.inner_cp2.1,
        layout.x_start,
        layout.y_endpoint,
    );

    // Close path
    d.push('Z');
    d
}

/// Draw a tie as a filled crescent shape.
///
/// Returns the path data string for testing purposes.
pub fn draw_tie(svg: &mut SvgWriter, layout: &TieLayout) -> String {
    let path_data = tie_path_data(layout);
    svg.add_filled_path(&path_data, "black");
    path_data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::EngravingConfig;
    use crate::layout::tie::{layout_tie, TieDirection};

    fn test_config() -> EngravingConfig {
        let metadata: smufl::Metadata =
            serde_json::from_slice(crate::font::BRAVURA_METADATA).unwrap();
        EngravingConfig::from_smufl(&metadata.engraving_defaults, 1000)
    }

    fn make_tie(direction: TieDirection) -> TieLayout {
        let cfg = test_config();
        layout_tie(100.0, 500.0, 400.0, direction, &cfg)
    }

    #[test]
    fn path_starts_with_move_to() {
        let layout = make_tie(TieDirection::Over);
        let path = tie_path_data(&layout);
        assert!(path.starts_with("M100.00,"));
    }

    #[test]
    fn path_has_two_cubic_beziers_and_close() {
        let layout = make_tie(TieDirection::Over);
        let path = tie_path_data(&layout);
        // Should have exactly 2 'C' commands and end with 'Z'
        let c_count = path.matches(" C").count();
        assert_eq!(c_count, 2, "expected 2 cubic Bézier commands, got {c_count}");
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn path_returns_to_start() {
        let layout = make_tie(TieDirection::Under);
        let path = tie_path_data(&layout);
        // The inner curve should end at the start x
        assert!(path.contains("100.00,"));
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn draw_tie_adds_path_element() {
        let layout = make_tie(TieDirection::Over);
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 1000.0, 500.0);
        draw_tie(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("<path "));
        assert!(output.contains(r#"fill="black""#));
        assert!(output.contains(r#"stroke="none""#));
    }

    #[test]
    fn draw_tie_returns_path_data() {
        let layout = make_tie(TieDirection::Over);
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 1000.0, 500.0);
        let path = draw_tie(&mut svg, &layout);
        assert!(path.starts_with('M'));
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn over_and_under_produce_different_paths() {
        let over_layout = make_tie(TieDirection::Over);
        let under_layout = make_tie(TieDirection::Under);
        let over_path = tie_path_data(&over_layout);
        let under_path = tie_path_data(&under_layout);
        assert_ne!(over_path, under_path);
    }

    #[test]
    fn path_contains_expected_endpoints() {
        let layout = make_tie(TieDirection::Over);
        let path = tie_path_data(&layout);
        // Start x=100, end x=500 should both appear
        assert!(path.contains("100.00"));
        assert!(path.contains("500.00"));
    }

    #[test]
    fn short_tie_path_is_valid() {
        let cfg = test_config();
        let layout = layout_tie(200.0, 260.0, 300.0, TieDirection::Under, &cfg);
        let mut svg = SvgWriter::new(400.0, 200.0, 0.0, 0.0, 600.0, 400.0);
        let path = draw_tie(&mut svg, &layout);
        // Should still be well-formed
        assert!(path.starts_with('M'));
        assert!(path.ends_with('Z'));
        assert_eq!(path.matches(" C").count(), 2);
    }

    #[test]
    fn long_tie_path_is_valid() {
        let cfg = test_config();
        let layout = layout_tie(50.0, 2000.0, 500.0, TieDirection::Over, &cfg);
        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 2500.0, 800.0);
        let path = draw_tie(&mut svg, &layout);
        assert!(path.starts_with('M'));
        assert!(path.ends_with('Z'));
        assert_eq!(path.matches(" C").count(), 2);
    }
}
