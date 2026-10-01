use music_engraver::font::{bravura_font, EngravingConfig, MusicFont};
use music_engraver::layout::tab::TabStaffLayout;
use music_engraver::render::tab_renderer::{
    draw_fret_number_at, draw_tab_clef, draw_tab_staff_lines,
};
use music_engraver::render::SvgWriter;

fn main() {
    let font: MusicFont = bravura_font();
    let config: EngravingConfig = font.engraving_config();
    let ss = config.staff_space;

    // --- 6-string guitar tab staff ---
    let guitar = TabStaffLayout::guitar(0.0, ss * 3.0, ss * 30.0, &config);
    let mut svg = SvgWriter::new(
        800.0,
        300.0,
        -ss,
        0.0,
        ss * 34.0,
        guitar.height() + ss * 8.0,
    );

    draw_tab_staff_lines(&mut svg, &guitar, &config);
    draw_tab_clef(&mut svg, &guitar, &font).unwrap();

    // Open E chord: E-B-E-G#-B-E  → frets 0-2-2-1-0-0
    let chord_x = ss * 6.0;
    draw_fret_number_at(&mut svg, &guitar, 6, 0, chord_x); // low E open
    draw_fret_number_at(&mut svg, &guitar, 5, 2, chord_x); // A → fret 2
    draw_fret_number_at(&mut svg, &guitar, 4, 2, chord_x); // D → fret 2
    draw_fret_number_at(&mut svg, &guitar, 3, 1, chord_x); // G → fret 1
    draw_fret_number_at(&mut svg, &guitar, 2, 0, chord_x); // B open
    draw_fret_number_at(&mut svg, &guitar, 1, 0, chord_x); // high E open

    // Ascending scale on string 1: frets 0, 1, 3, 5, 7, 9, 12
    let frets = [0, 1, 3, 5, 7, 9, 12];
    for (i, &fret) in frets.iter().enumerate() {
        let x = ss * (10.0 + i as f64 * 2.5);
        draw_fret_number_at(&mut svg, &guitar, 1, fret, x);
    }

    // Power chord on frets 5-7
    let power_x = ss * 28.0;
    draw_fret_number_at(&mut svg, &guitar, 6, 5, power_x);
    draw_fret_number_at(&mut svg, &guitar, 5, 7, power_x);
    draw_fret_number_at(&mut svg, &guitar, 4, 7, power_x);

    let output = svg.to_svg();

    // Verify structure
    let line_count = output.matches("<line ").count();
    let path_count = output.matches("<path ").count();
    let text_count = output.matches("<text ").count();
    let rect_count = output.matches("<rect ").count();
    println!("Guitar TAB: {line_count} lines, {path_count} paths, {text_count} texts, {rect_count} rects");

    assert_eq!(line_count, 6, "6 staff lines");
    assert_eq!(path_count, 1, "1 TAB clef path");
    assert_eq!(
        text_count, 16,
        "6 chord + 7 scale + 3 power chord = 16 fret numbers"
    );
    assert_eq!(rect_count, 16, "each fret number has a white bg rect");

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/tab_staff.svg", &output).unwrap();
    println!(
        "Wrote examples/output/tab_staff.svg ({} bytes)",
        output.len()
    );
}
