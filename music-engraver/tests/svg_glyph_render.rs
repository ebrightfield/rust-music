//! Integration test: font → glyph outline → SVG rendering pipeline.

use music_engraver::font::bravura_font;
use music_engraver::render::SvgWriter;
use smufl::Glyph;

/// Render a single notehead on staff lines and validate the SVG output.
#[test]
fn notehead_black_renders_to_valid_svg() {
    let font = bravura_font();
    let outline = font
        .glyph_outline(Glyph::NoteheadBlack)
        .expect("noteheadBlack should exist");

    let staff_space = font.units_per_em() as f64 / 4.0; // 250 units for Bravura
    let margin = 100.0;
    let staff_width = 1200.0;
    let vb_x = -margin;
    let vb_y = -2.0 * staff_space - margin;
    let vb_w = staff_width + 2.0 * margin;
    let vb_h = 4.0 * staff_space + 2.0 * margin;

    let mut writer = SvgWriter::new(400.0, 200.0, vb_x, vb_y, vb_w, vb_h);

    // Draw 5 staff lines
    for i in -2i32..=2 {
        let y = i as f64 * staff_space;
        writer.add_line(0.0, y, staff_width, y, "#000", 10.0);
    }

    // Place the notehead at the middle line (y=0)
    writer.add_path(&outline.path_data, "black", Some("translate(200, 0)"));

    let svg = writer.to_svg();

    // Structural assertions
    assert!(svg.starts_with("<svg"), "should be a valid SVG document");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("</svg>"), "should be closed");

    // Must contain exactly 5 staff lines
    let line_count = svg.matches("<line ").count();
    assert_eq!(line_count, 5, "should have 5 staff lines");

    // Must contain exactly 1 path (the notehead)
    let path_count = svg.matches("<path ").count();
    assert_eq!(path_count, 1, "should have 1 glyph path");

    // The path should contain the actual outline data (cubic curves)
    assert!(svg.contains("M"), "path should contain move commands");
    assert!(svg.contains("C"), "path should contain cubic curves");
    assert!(svg.contains("Z"), "path should be closed");

    // The notehead should be translated
    assert!(
        svg.contains("translate(200, 0)"),
        "notehead should be positioned via transform"
    );

    // viewBox should be present with correct dimensions
    assert!(svg.contains("viewBox=\""));
}

/// Multiple different glyphs should produce distinct SVG output.
#[test]
fn different_glyphs_produce_different_svgs() {
    let font = bravura_font();

    let glyphs = [
        Glyph::NoteheadBlack,
        Glyph::NoteheadWhole,
        Glyph::GClef,
        Glyph::FClef,
    ];

    let outlines: Vec<_> = glyphs
        .iter()
        .map(|g| font.glyph_outline(*g).expect("glyph should exist"))
        .collect();

    // All pairs should be different
    for i in 0..outlines.len() {
        for j in (i + 1)..outlines.len() {
            assert_ne!(
                outlines[i].path_data, outlines[j].path_data,
                "{:?} and {:?} should have different outlines",
                glyphs[i], glyphs[j]
            );
        }
    }
}

/// Glyph advance widths should be reasonable for known glyphs.
#[test]
fn glyph_advance_widths_are_reasonable() {
    let font = bravura_font();
    let upm = font.units_per_em() as u16;

    // noteheadBlack: ~1.18 staff spaces wide
    let bk = font.glyph_advance(Glyph::NoteheadBlack).unwrap();
    assert!(bk > 200 && bk < 400, "noteheadBlack advance = {bk}");

    // noteheadWhole: wider than black notehead
    let wh = font.glyph_advance(Glyph::NoteheadWhole).unwrap();
    assert!(wh > bk, "whole notehead should be wider than black");

    // gClef: substantial width
    let gc = font.glyph_advance(Glyph::GClef).unwrap();
    assert!(gc > 100 && gc < upm * 2, "gClef advance = {gc}");
}
