/// Renders slur curves between notes on a treble-clef staff.
/// Demonstrates slurs over ascending, descending, and same-pitch note pairs,
/// in both Over and Under directions.
/// Outputs SVG to `examples/output/slurs.svg`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::slur::{layout_slur, SlurDirection};
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::render::note_renderer::{draw_stemmed_note, NoteheadKind};
use music_engraver::render::slur_renderer::draw_slur;
use music_engraver::render::{draw_clef, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 11000.0, &config);

    let clef = Clef::Treble;
    let clef_layout = ClefLayout::from_clef(Clef::Treble);

    let mut svg = SvgWriter::new(1100.0, 200.0, 0.0, -200.0, 11000.0, 2000.0);

    draw_staff_lines(&mut svg, &staff, &config);
    draw_clef(&mut svg, &staff, &clef_layout, &font).unwrap();
    let base_x = 900.0;

    let notehead_width = font
        .glyph_advance(smufl::Glyph::NoteheadBlack)
        .unwrap_or(0) as f64;

    // Slur pairs: (start_pitch, end_pitch, direction, x_offset)
    let pairs: Vec<(Pitch, Pitch, SlurDirection, f64)> = vec![
        // Ascending slur under (stems up)
        (
            Pitch::new(Note::C, 4),
            Pitch::new(Note::G, 4),
            SlurDirection::Under,
            0.0,
        ),
        // Descending slur over (stems down)
        (
            Pitch::new(Note::B, 4),
            Pitch::new(Note::E, 4),
            SlurDirection::Over,
            2200.0,
        ),
        // Same-pitch slur under
        (
            Pitch::new(Note::A, 4),
            Pitch::new(Note::A, 4),
            SlurDirection::Under,
            4400.0,
        ),
        // Wide ascending slur over (ledger line territory)
        (
            Pitch::new(Note::C, 4),
            Pitch::new(Note::A, 5),
            SlurDirection::Over,
            6600.0,
        ),
    ];

    for (start_pitch, end_pitch, direction, x_off) in &pairs {
        let x1 = base_x + x_off;
        let x2 = x1 + 1500.0;

        let pos1 = pitch_to_staff_position(start_pitch, &clef);
        let pos2 = pitch_to_staff_position(end_pitch, &clef);

        let dir1 = auto_stem_direction(pos1);
        let dir2 = auto_stem_direction(pos2);

        draw_stemmed_note(
            &mut svg, &staff, &font, &config, x1, pos1,
            NoteheadKind::Filled, Some(dir1),
        ).unwrap();
        draw_stemmed_note(
            &mut svg, &staff, &font, &config, x2, pos2,
            NoteheadKind::Filled, Some(dir2),
        ).unwrap();

        let y1 = staff.y_of(pos1);
        let y2 = staff.y_of(pos2);

        let slur = layout_slur(
            x1 + notehead_width,
            x2,
            y1,
            y2,
            *direction,
            &config,
        );
        draw_slur(&mut svg, &slur);
    }

    let output = svg.to_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/slurs.svg", &output).unwrap();

    // Verify SVG structure
    assert!(output.starts_with("<svg"));
    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!("slurs.svg: {path_count} paths, {line_count} lines, {} bytes", output.len());
    // 1 clef + 8 noteheads + 4 slurs = 13 paths; 5 staff lines + 8 stems + ledger lines
    assert!(path_count >= 13, "expected at least 13 paths, got {path_count}");
    assert!(line_count >= 13, "expected at least 13 lines, got {line_count}");
}
