//! Renders a grand staff (piano) with brace connector and a bracket group.
//!
//! Produces `examples/output/grand_staff.svg`.

use music_engraver::font::bravura_font;
use music_engraver::layout::multi_staff::{
    layout_multi_staff, staff_layouts_from_multi, StaffGroup,
};
use music_engraver::render::multi_staff_renderer::{
    draw_joined_barline, draw_multi_staff_connectors,
};
use music_engraver::render::staff_renderer::draw_staff_lines;
use music_engraver::render::SvgWriter;

fn main() {
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");

    let font = bravura_font();
    let ss = font.engraving_config().staff_space;
    let staff_width = 6000.0;

    // --- Grand staff (piano) ---
    let grand = StaffGroup::grand_staff();
    let grand_layout = layout_multi_staff(&grand, 100.0, ss, staff_width);
    let grand_staves = staff_layouts_from_multi(&grand_layout, 100.0, staff_width);

    // --- Bracket group (e.g. string section) ---
    let section = StaffGroup::section(3);
    let section_y_start = grand_layout.total_height() + 100.0 + 8.0 * ss;
    let section_layout = layout_multi_staff(&section, section_y_start, ss, staff_width);
    let section_staves = staff_layouts_from_multi(&section_layout, 100.0, staff_width);

    // Compute viewBox
    let total_height = section_layout.staff_y_origins.last().unwrap() + ss * 4.0 + 100.0;
    let vb_x = -300.0;
    let vb_w = staff_width + 500.0;
    let px_w = vb_w / 10.0;
    let px_h = (total_height + 100.0) / 10.0;

    let mut svg = SvgWriter::new(px_w, px_h, vb_x, 0.0, vb_w, total_height + 100.0);

    // Draw grand staff
    let config = font.engraving_config();
    for staff in &grand_staves {
        draw_staff_lines(&mut svg, staff, &config);
    }
    draw_multi_staff_connectors(&mut svg, &font, &grand_layout).expect("draw brace");

    // Joined barline at start
    let gt = grand_staves[0].y_origin;
    let gb = grand_staves[1].y_origin + ss * 4.0;
    draw_joined_barline(&mut svg, 100.0, gt, gb, config.thin_barline_thickness_fu());

    // Draw section bracket group
    for staff in &section_staves {
        draw_staff_lines(&mut svg, staff, &config);
    }
    draw_multi_staff_connectors(&mut svg, &font, &section_layout).expect("draw bracket");

    // Joined barline for section
    let st = section_staves[0].y_origin;
    let sb = section_staves[2].y_origin + ss * 4.0;
    draw_joined_barline(&mut svg, 100.0, st, sb, config.thin_barline_thickness_fu());

    let output = svg.to_svg();
    let path = out_dir.join("grand_staff.svg");
    std::fs::write(&path, &output).expect("write SVG");

    let path_count = output.matches("<path").count();
    let line_count = output.matches("<line").count();
    println!(
        "Wrote {} ({} bytes, {} paths, {} lines)",
        path.display(),
        output.len(),
        path_count,
        line_count,
    );

    // Verify expected content. The section bracket now uses SMuFL
    // `bracketTop` + `bracketBottom` scroll glyphs (2 paths) plus 1 thick
    // vertical line — so the path/line breakdown is:
    //   paths: 1 brace + 2 bracket scrolls = 3
    //   lines: 25 staff (5 × 5) + 1 bracket vertical + 2 joined barlines = 28
    assert!(output.contains("<svg"), "should be valid SVG");
    assert!(
        path_count >= 3,
        "should have at least 3 paths (brace + 2 bracket scrolls), got {path_count}"
    );
    assert!(
        line_count >= 25,
        "should have at least 25 staff lines, got {line_count}"
    );
}
