/// Renders flagged notes (8th, 16th, 32nd) on a treble-clef staff,
/// combining noteheads, stems, flags, accidentals, and dots.
/// Outputs SVG to `examples/output/`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::render::stem_renderer::{stem_endpoints, stem_x};
use music_engraver::render::{
    draw_clef, draw_flag, draw_note, draw_staff_lines, draw_stem, NoteheadKind, SvgWriter,
};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 9000.0, &config);

    // (pitch, flag_count): various flagged notes across the staff
    let notes: Vec<(Pitch, u8)> = vec![
        (Pitch::new(Note::C, 4), 1), // middle C, 8th, stem up, ledger below
        (Pitch::new(Note::E, 4), 2), // bottom line, 16th, stem up
        (Pitch::new(Note::G, 4), 1), // second line, 8th, stem up
        (Pitch::new(Note::B, 4), 3), // middle line, 32nd, stem down
        (Pitch::new(Note::D, 5), 1), // fourth space, 8th, stem down
        (Pitch::new(Note::F, 5), 2), // top line, 16th, stem down
        (Pitch::new(Note::A, 5), 1), // ledger above, 8th, stem down
    ];

    let mut svg = SvgWriter::new(1100.0, 300.0, -100.0, -1500.0, 10000.0, 4000.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, staff.x + staff.staff_space, &clef, &font).unwrap();

    let start_x = 1200.0;
    let spacing = 1000.0;

    for (i, (pitch, flag_count)) in notes.iter().enumerate() {
        let x = start_x + i as f64 * spacing;
        let pos = pitch_to_staff_position(pitch, &Clef::Treble);
        let direction = auto_stem_direction(pos);

        let advance = draw_note(
            &mut svg,
            &staff,
            &font,
            &config,
            x,
            pos,
            NoteheadKind::Filled,
        )
        .unwrap();

        let thickness = config.stem_thickness_fu();
        draw_stem(&mut svg, &staff, &config, x, advance, pos, direction, 1.0);

        // Compute stem tip for flag placement
        let sx = stem_x(x, advance, direction, thickness);
        let (y_top, y_bottom) = stem_endpoints(&staff, pos, direction, 1.0);
        let tip_y = match direction {
            music_engraver::layout::stem::StemDirection::Up => y_top,
            music_engraver::layout::stem::StemDirection::Down => y_bottom,
        };

        draw_flag(&mut svg, &font, sx, tip_y, *flag_count, direction, 1.0).unwrap();
    }

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/flagged_notes.svg", &output).unwrap();
    println!("Wrote music-engraver/examples/output/flagged_notes.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG length: {} bytes, {} <path> elements, {} <line> elements",
        output.len(),
        path_count,
        line_count,
    );

    // Expected: 1 clef + 7 noteheads + 7 flags = 15 paths
    // Expected: 5 staff lines + 2 ledger lines + 7 stems = 14 lines
    println!("Expected: 15 paths (1 clef + 7 noteheads + 7 flags)");
    println!("Expected: 14 lines (5 staff + 2 ledger + 7 stems)");
}
