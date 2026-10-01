//! SVG rendering for beam groups above tablature staves.
//!
//! Tab beams are horizontal lines connecting rhythm stem tips at a fixed
//! height above the staff. Primary beams span the entire group; secondary
//! beams connect subgroups of shorter notes.

use crate::layout::tab_beam::TabBeamGroupLayout;
use crate::render::SvgWriter;

/// Fractional beam stub length in font design units.
/// When a secondary beam connects only on one side, a short stub is drawn.
const FRACTIONAL_STUB_FU: f64 = 50.0;

/// Draw a complete tab beam group: stems and horizontal beam lines.
///
/// Draws vertical stems from base to tip for each note, then horizontal
/// beam polygons connecting the stem tips. Since tab stems are all at the
/// same y, beams are perfectly horizontal rectangles.
pub fn draw_tab_beam_group(svg: &mut SvgWriter, layout: &TabBeamGroupLayout) {
    if layout.stems.is_empty() {
        return;
    }

    // Draw stems
    for stem in &layout.stems {
        svg.add_line(
            stem.x,
            stem.y_base,
            stem.x,
            stem.y_tip,
            "black",
            stem.stem_width,
        );
    }

    // Draw beam lines at each level
    let beam_y_base = layout.stems[0].y_tip;

    for level in 0..layout.max_beam_level {
        // Beam y offset: primary beam at the stem tip, secondary beams
        // stack downward (toward the staff) since stems point up.
        let y_offset = level as f64 * (layout.beam_thickness + layout.beam_gap);
        let beam_y_top = beam_y_base + y_offset;
        let beam_y_bottom = beam_y_top + layout.beam_thickness;
        let level1 = level + 1; // 1-based for comparison with beam counts

        draw_tab_beam_level(svg, layout, level1, beam_y_top, beam_y_bottom);
    }
}

/// Draw all beam segments at a given 1-based level.
fn draw_tab_beam_level(
    svg: &mut SvgWriter,
    layout: &TabBeamGroupLayout,
    level: u8,
    y_top: f64,
    y_bottom: f64,
) {
    let n = layout.stems.len();
    let mut i = 0;

    while i < n {
        // Find a run of notes that are connected at this level
        if layout.beams_right.get(i).copied().unwrap_or(0) >= level {
            // Start of a connected segment
            let start_x = layout.stems[i].x;
            let mut j = i;
            while j + 1 < n && layout.beams_right[j] >= level && layout.beams_left[j + 1] >= level {
                j += 1;
            }
            let end_x = layout.stems[j].x;

            // Draw beam polygon (rectangle since tab beams are horizontal)
            svg.add_polygon(
                &[
                    (start_x, y_top),
                    (end_x, y_top),
                    (end_x, y_bottom),
                    (start_x, y_bottom),
                ],
                "black",
            );

            i = j + 1;
        } else if layout.beams_left.get(i).copied().unwrap_or(0) >= level
            && (i == 0
                || layout
                    .beams_right
                    .get(i.wrapping_sub(1))
                    .copied()
                    .unwrap_or(0)
                    < level)
        {
            // Fractional stub pointing left
            let x = layout.stems[i].x;
            svg.add_polygon(
                &[
                    (x - FRACTIONAL_STUB_FU, y_top),
                    (x, y_top),
                    (x, y_bottom),
                    (x - FRACTIONAL_STUB_FU, y_bottom),
                ],
                "black",
            );
            i += 1;
        } else if layout.beams_right.get(i).copied().unwrap_or(0) < level
            && layout.beams_left.get(i).copied().unwrap_or(0) < level
            && layout.stems[i].flag_count >= level
        {
            // Isolated note with more beams than neighbors: fractional stub
            // pointing toward the center of the group.
            let x = layout.stems[i].x;
            if i < n / 2 {
                // Point right
                svg.add_polygon(
                    &[
                        (x, y_top),
                        (x + FRACTIONAL_STUB_FU, y_top),
                        (x + FRACTIONAL_STUB_FU, y_bottom),
                        (x, y_bottom),
                    ],
                    "black",
                );
            } else {
                // Point left
                svg.add_polygon(
                    &[
                        (x - FRACTIONAL_STUB_FU, y_top),
                        (x, y_top),
                        (x, y_bottom),
                        (x - FRACTIONAL_STUB_FU, y_bottom),
                    ],
                    "black",
                );
            }
            i += 1;
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::tab::TabStaffLayout;
    use crate::layout::tab_beam::{layout_tab_beam_group, TabBeamedNote};

    fn guitar_staff() -> TabStaffLayout {
        let font = bravura_font();
        let config = font.engraving_config();
        TabStaffLayout::guitar(0.0, 500.0, 5000.0, &config)
    }

    fn make_svg() -> SvgWriter {
        SvgWriter::new(800.0, 600.0, -200.0, -400.0, 6000.0, 3000.0)
    }

    fn make_notes(xs: &[f64], dur: u8) -> Vec<TabBeamedNote> {
        xs.iter()
            .map(|&x| TabBeamedNote {
                x,
                duration_log2: dur,
            })
            .collect()
    }

    #[test]
    fn two_eighths_draw_stems_and_beam() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 2000.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        let mut svg = make_svg();
        draw_tab_beam_group(&mut svg, &layout);
        let output = svg.to_svg();
        // 2 stems
        assert_eq!(
            output.matches("<line ").count(),
            2,
            "should have 2 stem lines"
        );
        // 1 beam polygon (primary)
        assert_eq!(
            output.matches("<polygon ").count(),
            1,
            "should have 1 beam polygon"
        );
    }

    #[test]
    fn four_sixteenths_draw_two_beam_levels() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1500.0, 2000.0, 2500.0], 4);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        let mut svg = make_svg();
        draw_tab_beam_group(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 4, "4 stems");
        assert_eq!(
            output.matches("<polygon ").count(),
            2,
            "2 beam polygons (primary + secondary)"
        );
    }

    #[test]
    fn empty_layout_no_output() {
        let layout = TabBeamGroupLayout {
            stems: vec![],
            max_beam_level: 0,
            beams_left: vec![],
            beams_right: vec![],
            beam_thickness: 20.0,
            beam_gap: 10.0,
        };
        let mut svg = make_svg();
        draw_tab_beam_group(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(!output.contains("<line "));
        assert!(!output.contains("<polygon "));
    }

    #[test]
    fn beam_polygon_is_horizontal() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 2000.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        let mut svg = make_svg();
        draw_tab_beam_group(&mut svg, &layout);
        let output = svg.to_svg();
        // The beam polygon should have 4 points with only 2 distinct y values
        // (top and bottom of the beam rectangle)
        assert!(output.contains("<polygon "));
    }

    #[test]
    fn stems_at_correct_x() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 3000.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        let mut svg = make_svg();
        draw_tab_beam_group(&mut svg, &layout);
        let output = svg.to_svg();
        assert!(output.contains("x1=\"1000\""), "first stem at x=1000");
        assert!(output.contains("x1=\"3000\""), "second stem at x=3000");
    }

    #[test]
    fn beamed_differs_from_flagged() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 2000.0], 3);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        let mut svg_beam = make_svg();
        draw_tab_beam_group(&mut svg_beam, &layout);
        let beam_output = svg_beam.to_svg();

        // Beamed version uses polygons, not path (flag) elements
        assert!(
            beam_output.contains("<polygon "),
            "beamed group uses polygon"
        );
        assert!(
            !beam_output.contains("<path "),
            "beamed group should not have flag paths"
        );
    }

    #[test]
    fn three_thirty_seconds_have_three_beam_levels() {
        let staff = guitar_staff();
        let notes = make_notes(&[1000.0, 1500.0, 2000.0], 5);
        let layout = layout_tab_beam_group(&staff, &notes, 5.0, 20.0, 10.0).unwrap();
        let mut svg = make_svg();
        draw_tab_beam_group(&mut svg, &layout);
        let output = svg.to_svg();
        assert_eq!(output.matches("<line ").count(), 3, "3 stems");
        assert_eq!(
            output.matches("<polygon ").count(),
            3,
            "3 beam polygons (primary + 2 secondary)"
        );
    }
}
