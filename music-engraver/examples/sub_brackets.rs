//! Renders a 6-staff section bracket with two nested sub-brackets — the
//! published-engraving convention for two-deep section grouping (e.g.
//! Violin I + Violin II share a sub-bracket inside the larger string-section
//! bracket).
//!
//! Produces `examples/output/sub_brackets.svg`.

use music_engraver::font::bravura_font;
use music_engraver::layout::multi_staff::{
    layout_multi_staff, staff_layouts_from_multi, StaffGroup, SubBracket,
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
    let staff_width = 5000.0;

    // 6 staves, two nested sub-brackets:
    //   * staves 0..2 — e.g. Violin I + Violin II
    //   * staves 3..6 — e.g. Viola + Cello + Bass
    // The main section bracket spans all 6 staves; each sub-bracket marks
    // an instrument family within it.
    let section = StaffGroup::section(6).with_sub_brackets(vec![
        SubBracket { start_index: 0, staff_count: 2 },
        SubBracket { start_index: 3, staff_count: 3 },
    ]);
    let layout = layout_multi_staff(&section, 100.0, ss, staff_width);
    let staves = staff_layouts_from_multi(&layout, 100.0, staff_width);

    // ViewBox accommodates the leftward-shifted main bracket plus a small
    // margin to the left of the bracket. Main bracket x is at
    // -(0.5 + 0.3 + 0.16 + 0.3) * SS = -1.26 * 250 = -315.
    let vb_x = -500.0;
    let vb_w = staff_width + 800.0;
    let total_height = layout.staff_y_origins.last().unwrap() + ss * 4.0 + 100.0;
    let px_per_unit = 7.0 / ss;
    let px_w = vb_w * px_per_unit;
    let px_h = (total_height + 100.0) * px_per_unit;

    let mut svg = SvgWriter::new(px_w, px_h, vb_x, 0.0, vb_w, total_height + 100.0);

    let config = font.engraving_config();
    for staff in &staves {
        draw_staff_lines(&mut svg, staff, &config);
    }
    draw_multi_staff_connectors(&mut svg, &font, &layout).expect("draw bracket + sub-brackets");

    // Joined barline at the start of the system — spans all staves so the
    // bracket grouping reads correctly.
    let st = staves[0].y_origin;
    let sb = staves.last().unwrap().y_origin + ss * 4.0;
    draw_joined_barline(&mut svg, 100.0, st, sb, config.thin_barline_thickness_fu());

    let output = svg.to_svg();
    let path = out_dir.join("sub_brackets.svg");
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

    // Verify expected content. With 6 staves + 1 main bracket + 2 sub-brackets:
    //   paths: 2 (main bracket scrolls — top + bottom)
    //   lines: 30 staff lines (5 per staff × 6) + 1 main bracket thick line
    //        + 2 sub-bracket thin lines + 1 joined barline = 34.
    assert!(output.contains("<svg"), "should be valid SVG");
    assert_eq!(
        path_count, 2,
        "main bracket scrolls = 2 paths, got {path_count}"
    );
    assert_eq!(
        line_count, 34,
        "30 staff lines + 1 main bracket + 2 sub-brackets + 1 joined barline = 34 lines, got {line_count}"
    );
}
