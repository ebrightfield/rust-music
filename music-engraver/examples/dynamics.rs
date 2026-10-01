/// Renders dynamics markings (pp, p, mp, mf, f, ff, fff) below a treble-clef
/// staff with notes, demonstrating placement below the staff.
/// Outputs SVG to `examples/output/dynamics.svg`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::render::dynamics_renderer::draw_dynamic;
use music_engraver::render::note_renderer::{draw_stemmed_note, NoteheadKind};
use music_engraver::render::{draw_clef, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 9000.0, &config);

    let clef = Clef::Treble;
    let clef_layout = ClefLayout::from_clef(Clef::Treble);

    // Notes with dynamics underneath
    let entries: Vec<(Pitch, Dynamic)> = vec![
        (Pitch::new(Note::E, 4), Dynamic::Pp),
        (Pitch::new(Note::G, 4), Dynamic::Piano),
        (Pitch::new(Note::A, 4), Dynamic::Mp),
        (Pitch::new(Note::B, 4), Dynamic::Mf),
        (Pitch::new(Note::D, 5), Dynamic::Forte),
        (Pitch::new(Note::E, 5), Dynamic::Ff),
        (Pitch::new(Note::G, 5), Dynamic::Fff),
    ];

    // Larger viewBox to accommodate dynamics below staff
    let mut svg = SvgWriter::new(1100.0, 350.0, -100.0, -1200.0, 10000.0, 4500.0);

    draw_staff_lines(&mut svg, &staff, &config);
    draw_clef(&mut svg, &staff, &clef_layout, &font).unwrap();

    let start_x = 1200.0;
    let spacing = 1000.0;

    for (i, (pitch, dynamic)) in entries.iter().enumerate() {
        let x = start_x + i as f64 * spacing;
        let pos = pitch_to_staff_position(pitch, &clef);
        let dir = auto_stem_direction(pos);
        let advance = draw_stemmed_note(
            &mut svg,
            &staff,
            &font,
            &config,
            x,
            pos,
            NoteheadKind::Filled,
            Some(dir),
        )
        .unwrap();

        // Center dynamic on the notehead
        let note_center = x + advance / 2.0;
        draw_dynamic(&mut svg, &staff, &font, *dynamic, note_center).unwrap();
    }

    let output = svg.to_svg();

    // Verify structure
    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    assert!(
        path_count >= 14, // 7 noteheads + 1 clef + 7 dynamics (at minimum, some may have more)
        "expected at least 14 paths, got {path_count}"
    );
    assert_eq!(line_count, 5 + 7, "5 staff lines + 7 stems = 12 lines");

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/dynamics.svg", &output).unwrap();

    println!(
        "Wrote dynamics.svg ({} bytes, {} paths, {} lines)",
        output.len(),
        path_count,
        line_count,
    );
}
