use crate::font::EngravingConfig;
use crate::layout::beam::{BeamGroupLayout, BeamedNote};
use crate::layout::staff::StaffLayout;
use crate::layout::stem::StemDirection;
use crate::render::stem_renderer::stem_x;
use crate::render::SvgWriter;

/// Fractional beam stub length in staff spaces.
/// When a secondary beam can only connect on one side, a short stub is drawn.
const FRACTIONAL_BEAM_LENGTH_SS: f64 = 0.75;

/// Draw a complete beam group: stems and beam lines.
///
/// For each note, draws a vertical stem from the notehead to the beam line.
/// Then draws beam lines (primary and secondary) connecting the stems.
///
/// `notes` and `layout` must have the same length. `notehead_advance` is
/// the advance width of the filled notehead glyph (needed for stem x-placement).
pub fn draw_beam_group(
    svg: &mut SvgWriter,
    staff: &StaffLayout,
    config: &EngravingConfig,
    notes: &[BeamedNote],
    layout: &BeamGroupLayout,
    notehead_advance: f64,
) {
    assert_eq!(notes.len(), layout.stem_tip_ys.len());
    if notes.is_empty() {
        return;
    }

    let stem_thick = config.stem_thickness_fu();
    let beam_thick = config.beam_thickness_fu();
    let beam_gap = config.beam_spacing_fu();

    // Draw stems: vertical line from notehead to beam attachment point
    for (i, note) in notes.iter().enumerate() {
        let notehead_y = staff.y_of(note.staff_position);
        let tip_y = layout.stem_tip_ys[i];
        let sx = stem_x(note.x, notehead_advance, layout.direction, stem_thick);

        let (y1, y2) = if tip_y < notehead_y {
            (tip_y, notehead_y)
        } else {
            (notehead_y, tip_y)
        };
        svg.add_line(sx, y1, sx, y2, "black", stem_thick);
    }

    // Draw beam lines at each level
    for level in 0..layout.max_beam_level {
        draw_beam_level(
            svg,
            notes,
            layout,
            notehead_advance,
            stem_thick,
            beam_thick,
            beam_gap,
            level,
            staff.staff_space,
        );
    }
}

/// Draw all beam segments at a given level (0 = primary, 1 = secondary, etc.).
fn draw_beam_level(
    svg: &mut SvgWriter,
    notes: &[BeamedNote],
    layout: &BeamGroupLayout,
    notehead_advance: f64,
    stem_thick: f64,
    beam_thick: f64,
    beam_gap: f64,
    level: u8,
    staff_space: f64,
) {
    let n = notes.len();
    let level1 = level + 1; // 1-based level for comparison with beam counts

    // Vertical offset from primary beam for this level.
    // Beams stack away from the noteheads (toward the stem tip direction).
    let beam_offset = level as f64 * (beam_thick + beam_gap);
    let dir_sign = match layout.direction {
        StemDirection::Up => 1.0,   // beams at top of stem, stack downward (positive y)
        StemDirection::Down => -1.0, // beams at bottom of stem, stack upward (negative y)
    };

    // Walk through notes and find connected segments at this level.
    // A segment connects consecutive notes where both have enough beams.
    let mut i = 0;
    while i < n {
        // Does this note have beams at this level on its right side?
        let has_right = layout.beams_right[i] >= level1;
        // Does this note have beams at this level on its left side?
        let has_left = layout.beams_left[i] >= level1;

        if has_right {
            // Find how far this beam segment extends to the right
            let start = i;
            let mut end = i;
            while end + 1 < n && layout.beams_left[end + 1] >= level1 {
                end += 1;
                if layout.beams_right[end] < level1 {
                    break;
                }
            }

            // Draw beam segment from start to end
            draw_beam_segment(
                svg,
                notes,
                layout,
                notehead_advance,
                stem_thick,
                beam_thick,
                beam_offset,
                dir_sign,
                start,
                end,
            );
            i = end + 1;
        } else if has_left && !has_right && (i == 0 || layout.beams_right[i - 1] < level1) {
            // Isolated left-only fractional beam (stub)
            draw_fractional_beam(
                svg,
                notes,
                layout,
                notehead_advance,
                stem_thick,
                beam_thick,
                beam_offset,
                dir_sign,
                staff_space,
                i,
                true, // points left
            );
            i += 1;
        } else {
            i += 1;
        }
    }
}

/// Draw a beam segment connecting notes[start] to notes[end].
fn draw_beam_segment(
    svg: &mut SvgWriter,
    notes: &[BeamedNote],
    layout: &BeamGroupLayout,
    notehead_advance: f64,
    stem_thick: f64,
    beam_thick: f64,
    beam_offset: f64,
    dir_sign: f64,
    start: usize,
    end: usize,
) {
    let x_left = stem_x(notes[start].x, notehead_advance, layout.direction, stem_thick);
    let x_right = stem_x(notes[end].x, notehead_advance, layout.direction, stem_thick);
    let y_left = layout.stem_tip_ys[start] + beam_offset * dir_sign;
    let y_right = layout.stem_tip_ys[end] + beam_offset * dir_sign;

    // Beam is a filled parallelogram with thickness beam_thick
    let y_left_bottom = y_left + beam_thick * dir_sign;
    let y_right_bottom = y_right + beam_thick * dir_sign;

    // Extend beam slightly past stems on the outside edges (half stem width)
    let half_stem = stem_thick / 2.0;
    let xl = x_left - half_stem;
    let xr = x_right + half_stem;

    svg.add_polygon(
        &[
            (xl, y_left),
            (xr, y_right),
            (xr, y_right_bottom),
            (xl, y_left_bottom),
        ],
        "black",
    );
}

/// Draw a fractional (stub) beam for an isolated secondary beam.
fn draw_fractional_beam(
    svg: &mut SvgWriter,
    notes: &[BeamedNote],
    layout: &BeamGroupLayout,
    notehead_advance: f64,
    stem_thick: f64,
    beam_thick: f64,
    beam_offset: f64,
    dir_sign: f64,
    staff_space: f64,
    note_idx: usize,
    _points_left: bool,
) {
    let x_note = stem_x(
        notes[note_idx].x,
        notehead_advance,
        layout.direction,
        stem_thick,
    );
    let y_note = layout.stem_tip_ys[note_idx] + beam_offset * dir_sign;

    let stub_len = FRACTIONAL_BEAM_LENGTH_SS * staff_space;

    // Fractional beams point toward the beat (left by convention)
    let x_stub = x_note - stub_len;

    // Interpolate y at stub end (beams are sloped, so we need the slope)
    // For fractional beams, use flat (same y) since they're short stubs
    let y_stub = y_note;

    let y_bottom = y_note + beam_thick * dir_sign;
    let y_stub_bottom = y_stub + beam_thick * dir_sign;

    let half_stem = stem_thick / 2.0;

    svg.add_polygon(
        &[
            (x_stub, y_stub),
            (x_note + half_stem, y_note),
            (x_note + half_stem, y_bottom),
            (x_stub, y_stub_bottom),
        ],
        "black",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::bravura_font;
    use crate::layout::beam::{layout_beam_group, BeamedNote};
    use crate::layout::staff::StaffLayout;
    use crate::layout::stem::StemDirection;

    const SS: f64 = 250.0;

    fn setup() -> (EngravingConfig, StaffLayout) {
        let font = bravura_font();
        let config = font.engraving_config();
        let staff = StaffLayout::from_config(0.0, 0.0, 5000.0, &config);
        (config, staff)
    }

    fn make_notes(positions: &[(f64, i8, u8)]) -> Vec<BeamedNote> {
        positions
            .iter()
            .map(|&(x, pos, dur)| BeamedNote {
                x,
                staff_position: pos,
                duration_log2: dur,
            })
            .collect()
    }

    fn count_element(svg: &str, tag: &str) -> usize {
        svg.matches(&format!("<{tag} ")).count()
    }

    #[test]
    fn two_eighths_stems_up_produces_lines_and_polygon() {
        let (config, staff) = setup();
        let notes = make_notes(&[(500.0, 0, 3), (1000.0, 2, 3)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        // 2 stems (lines) + 1 primary beam (polygon)
        assert_eq!(count_element(&output, "line"), 2, "should have 2 stems");
        assert_eq!(
            count_element(&output, "polygon"),
            1,
            "should have 1 beam"
        );
    }

    #[test]
    fn two_sixteenths_produces_two_beam_polygons() {
        let (config, staff) = setup();
        let notes = make_notes(&[(500.0, 2, 4), (1000.0, 4, 4)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        // 2 stems + 2 beam levels (primary + secondary)
        assert_eq!(count_element(&output, "line"), 2);
        assert_eq!(count_element(&output, "polygon"), 2);
    }

    #[test]
    fn four_eighths_produces_one_beam_polygon() {
        let (config, staff) = setup();
        let notes = make_notes(&[
            (500.0, 0, 3),
            (800.0, 2, 3),
            (1100.0, 4, 3),
            (1400.0, 2, 3),
        ]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        assert_eq!(count_element(&output, "line"), 4, "4 stems");
        assert_eq!(count_element(&output, "polygon"), 1, "1 primary beam");
    }

    #[test]
    fn stems_down_beam_below_noteheads() {
        let (config, staff) = setup();
        let notes = make_notes(&[(500.0, 6, 3), (1000.0, 8, 3)]);
        let layout = layout_beam_group(&notes, StemDirection::Down, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        assert_eq!(count_element(&output, "line"), 2);
        assert_eq!(count_element(&output, "polygon"), 1);
        // Beam polygon y-coordinates should be below noteheads (larger y)
        // The stem tips are below noteheads for down stems
        let notehead_y_top = staff.y_of(8); // 0.0
        for tip in &layout.stem_tip_ys {
            assert!(*tip > notehead_y_top, "tip {tip} should be below top-line notehead");
        }
    }

    #[test]
    fn mixed_eighth_sixteenth_produces_correct_beams() {
        let (config, staff) = setup();
        // Eighth, sixteenth, sixteenth — primary beam spans all 3,
        // secondary beam between notes 2 and 3
        let notes = make_notes(&[(500.0, 2, 3), (800.0, 4, 4), (1100.0, 6, 4)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        assert_eq!(count_element(&output, "line"), 3, "3 stems");
        // Primary beam (spanning all) + secondary beam (connecting 16th notes)
        assert!(
            count_element(&output, "polygon") >= 2,
            "at least 2 beam polygons (primary + secondary)"
        );
    }

    #[test]
    fn empty_group_produces_nothing() {
        let (config, staff) = setup();
        let layout = BeamGroupLayout {
            direction: StemDirection::Up,
            stem_tip_ys: vec![],
            max_beam_level: 1,
            beams_left: vec![],
            beams_right: vec![],
        };

        let mut svg = SvgWriter::new(800.0, 200.0, 0.0, 0.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &[], &layout, 295.0);
        let output = svg.to_svg();

        assert_eq!(count_element(&output, "line"), 0);
        assert_eq!(count_element(&output, "polygon"), 0);
    }

    #[test]
    fn beam_polygon_has_four_points() {
        let (config, staff) = setup();
        let notes = make_notes(&[(500.0, 4, 3), (1000.0, 4, 3)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        // Each polygon should have 4 points (3 commas = 4 coordinates)
        for line in output.lines() {
            if line.contains("<polygon") {
                let points_start = line.find("points=\"").unwrap() + 8;
                let points_end = line[points_start..].find('"').unwrap() + points_start;
                let points_str = &line[points_start..points_end];
                let point_count = points_str.split(' ').count();
                assert_eq!(point_count, 4, "beam polygon should have 4 points");
            }
        }
    }

    #[test]
    fn beam_thickness_matches_config() {
        let (config, staff) = setup();
        let notes = make_notes(&[(500.0, 4, 3), (1500.0, 4, 3)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        // For a flat beam (same position), the polygon should have two y-values
        // separated by beam_thickness
        for line in output.lines() {
            if line.contains("<polygon") {
                let points_start = line.find("points=\"").unwrap() + 8;
                let points_end = line[points_start..].find('"').unwrap() + points_start;
                let points_str = &line[points_start..points_end];
                let coords: Vec<(f64, f64)> = points_str
                    .split(' ')
                    .map(|p| {
                        let mut parts = p.split(',');
                        let x: f64 = parts.next().unwrap().parse().unwrap();
                        let y: f64 = parts.next().unwrap().parse().unwrap();
                        (x, y)
                    })
                    .collect();
                // Points 0,1 are top edge; points 2,3 are bottom edge
                // For flat beam: y0 == y1, y2 == y3, |y2-y0| == beam_thickness
                let y_diff = (coords[3].1 - coords[0].1).abs();
                assert!(
                    (y_diff - config.beam_thickness_fu()).abs() < 1e-6,
                    "beam thickness {y_diff} should match config {}",
                    config.beam_thickness_fu()
                );
            }
        }
    }

    #[test]
    fn thirty_second_notes_produce_three_beams() {
        let (config, staff) = setup();
        let notes = make_notes(&[(500.0, 2, 5), (1000.0, 4, 5)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        assert_eq!(count_element(&output, "line"), 2, "2 stems");
        assert_eq!(
            count_element(&output, "polygon"),
            3,
            "3 beam levels for 32nd notes"
        );
    }

    #[test]
    fn single_beamed_note_gets_fractional_beam() {
        let (config, staff) = setup();
        let notes = make_notes(&[(500.0, 4, 3)]);
        let layout = layout_beam_group(&notes, StemDirection::Up, SS);

        let mut svg = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg, &staff, &config, &notes, &layout, 295.0);
        let output = svg.to_svg();

        assert_eq!(count_element(&output, "line"), 1, "1 stem");
        // Should have a fractional beam (left-pointing stub)
        assert_eq!(
            count_element(&output, "polygon"),
            1,
            "1 fractional beam"
        );
    }

    #[test]
    fn stem_x_positions_respect_direction() {
        let (config, staff) = setup();
        let advance = 295.0;

        // Stems up: x on right side of notehead
        let notes_up = make_notes(&[(500.0, 2, 3), (1000.0, 4, 3)]);
        let layout_up = layout_beam_group(&notes_up, StemDirection::Up, SS);
        let mut svg_up = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg_up, &staff, &config, &notes_up, &layout_up, advance);

        // Stems down: x on left side of notehead
        let notes_dn = make_notes(&[(500.0, 6, 3), (1000.0, 8, 3)]);
        let layout_dn = layout_beam_group(&notes_dn, StemDirection::Down, SS);
        let mut svg_dn = SvgWriter::new(800.0, 200.0, -200.0, -500.0, 6000.0, 2500.0);
        draw_beam_group(&mut svg_dn, &staff, &config, &notes_dn, &layout_dn, advance);

        let out_up = svg_up.to_svg();
        let out_dn = svg_dn.to_svg();

        // The stem x1 values should differ between up and down
        let extract_x1 = |s: &str| -> Vec<String> {
            s.lines()
                .filter(|l| l.contains("<line "))
                .map(|l| {
                    let start = l.find("x1=\"").unwrap() + 4;
                    let end = l[start..].find('"').unwrap() + start;
                    l[start..end].to_string()
                })
                .collect()
        };

        let x1_up = extract_x1(&out_up);
        let x1_dn = extract_x1(&out_dn);

        // Up stems should be at right edge (500+295-15=780), down at left (500+15=515)
        assert_ne!(x1_up[0], x1_dn[0], "stem-up and stem-down x should differ");
    }
}
