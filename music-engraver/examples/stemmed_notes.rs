/// Renders several notes with stems on a treble-clef staff, demonstrating
/// automatic stem direction and stem rendering. Outputs SVG to `examples/output/`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::render::note_renderer::NoteheadKind;
use music_engraver::render::stem_renderer::draw_stem;
use music_engraver::render::{draw_clef, draw_note, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 7000.0, &config);

    // Notes spanning from below the staff to above, to show stem direction switching
    let notes: Vec<(Pitch, NoteheadKind)> = vec![
        (Pitch::new(Note::C, 4), NoteheadKind::Filled), // middle C, ledger below, stem up
        (Pitch::new(Note::E, 4), NoteheadKind::Filled), // bottom line, stem up
        (Pitch::new(Note::G, 4), NoteheadKind::Filled), // second line, stem up
        (Pitch::new(Note::B, 4), NoteheadKind::Half),   // middle line, stem down
        (Pitch::new(Note::D, 5), NoteheadKind::Filled), // fourth line space, stem down
        (Pitch::new(Note::F, 5), NoteheadKind::Filled), // top line, stem down
        (Pitch::new(Note::A, 5), NoteheadKind::Half),   // ledger above, stem down
    ];

    let mut svg = SvgWriter::new(900.0, 250.0, -100.0, -1200.0, 8000.0, 3500.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, &clef, &font).unwrap();

    let start_x = 1200.0;
    let spacing = 800.0;

    for (i, (pitch, kind)) in notes.iter().enumerate() {
        let x = start_x + i as f64 * spacing;
        let pos = pitch_to_staff_position(pitch, &Clef::Treble);
        let advance = draw_note(&mut svg, &staff, &font, &config, x, pos, *kind).unwrap();

        // Whole notes don't get stems
        if *kind != NoteheadKind::Whole {
            let direction = auto_stem_direction(pos);
            draw_stem(&mut svg, &staff, &config, x, advance, pos, direction);
        }
    }

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/stemmed_notes.svg", &output).unwrap();
    println!("Wrote music-engraver/examples/output/stemmed_notes.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG length: {} bytes, {} <path> elements, {} <line> elements",
        output.len(),
        path_count,
        line_count,
    );

    // Expected: 8 paths (1 clef + 7 noteheads), 5 staff lines + 2 ledger lines + 7 stems = 14 lines
    println!("Expected: 8 paths (1 clef + 7 noteheads)");
    println!("Expected: ~14 lines (5 staff + 2 ledger + 7 stems)");
}
