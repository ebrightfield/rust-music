/// Renders tuplet brackets (triplet and quintuplet) above and below staff notes.
use music_engraver::font::bravura_font;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::StemDirection;
use music_engraver::layout::tuplet::{
    layout_tuplet_bracket, tuplet_number_glyphs, TupletPlacement,
};
use music_engraver::render::note_renderer::{draw_stemmed_note, NoteheadKind};
use music_engraver::render::staff_renderer::draw_staff_lines;
use music_engraver::render::tuplet_renderer::draw_tuplet_bracket;
use music_engraver::render::SvgWriter;

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let ss = config.staff_space;

    let staff_width = 3500.0;
    let staff = StaffLayout::new(100.0, 200.0, staff_width, ss);

    let mut svg = SvgWriter::new(800.0, 500.0, 0.0, 0.0, 3800.0, 2000.0);
    draw_staff_lines(&mut svg, &staff, &config);

    // --- Triplet above staff (3 eighth notes, stems down → bracket above) ---
    let positions: &[i8] = &[2, 4, 6];
    let note_xs = [300.0, 600.0, 900.0];

    for (i, &pos) in positions.iter().enumerate() {
        draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            note_xs[i],
            pos,
            NoteheadKind::Filled,
            Some(StemDirection::Down),
        )
        .unwrap();
    }

    let number_width_3: f64 = tuplet_number_glyphs(3)
        .iter()
        .map(|g| font.glyph_advance(*g).unwrap_or(0) as f64)
        .sum();
    let triplet_layout = layout_tuplet_bracket(
        note_xs[0],
        note_xs[2],
        positions,
        TupletPlacement::Above,
        3,
        ss,
        config.tuplet_bracket_thickness,
        number_width_3,
    );
    draw_tuplet_bracket(&mut svg, &triplet_layout, &font, 0.0, staff.y_origin);

    // --- Quintuplet below staff (5 quarter notes, stems up → bracket below) ---
    let positions5: &[i8] = &[0, 2, 1, 3, 0];
    let note_xs5 = [1400.0, 1650.0, 1900.0, 2150.0, 2400.0];

    for (i, &pos) in positions5.iter().enumerate() {
        draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            note_xs5[i],
            pos,
            NoteheadKind::Filled,
            Some(StemDirection::Up),
        )
        .unwrap();
    }

    let number_width_5: f64 = tuplet_number_glyphs(5)
        .iter()
        .map(|g| font.glyph_advance(*g).unwrap_or(0) as f64)
        .sum();
    let quint_layout = layout_tuplet_bracket(
        note_xs5[0],
        note_xs5[4],
        positions5,
        TupletPlacement::Below,
        5,
        ss,
        config.tuplet_bracket_thickness,
        number_width_5,
    );
    draw_tuplet_bracket(&mut svg, &quint_layout, &font, 0.0, staff.y_origin);

    // --- Triplet above staff with ledger lines (high notes, stems down) ---
    let positions_high: &[i8] = &[10, 12, 10];
    let note_xs_high = [2800.0, 3050.0, 3300.0];

    for (i, &pos) in positions_high.iter().enumerate() {
        draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            note_xs_high[i],
            pos,
            NoteheadKind::Filled,
            Some(StemDirection::Down),
        )
        .unwrap();
    }

    let triplet_high_layout = layout_tuplet_bracket(
        note_xs_high[0],
        note_xs_high[2],
        positions_high,
        TupletPlacement::Above,
        3,
        ss,
        config.tuplet_bracket_thickness,
        number_width_3,
    );
    draw_tuplet_bracket(&mut svg, &triplet_high_layout, &font, 0.0, staff.y_origin);

    let output = svg.to_svg();

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    eprintln!(
        "Tuplet brackets example: {} paths, {} lines, {} bytes",
        path_count,
        line_count,
        output.len()
    );

    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write(
        "music-engraver/examples/output/tuplet_brackets.svg",
        &output,
    )
    .unwrap();
    eprintln!("Wrote music-engraver/examples/output/tuplet_brackets.svg");
}
