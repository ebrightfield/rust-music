/// Renders several notes on a treble-clef staff, demonstrating notehead
/// placement and ledger lines. Outputs SVG to `examples/output/`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::render::note_renderer::NoteheadKind;
use music_engraver::render::{draw_clef, draw_note, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 6000.0, &config);

    // Notes to render: middle C (ledger below), E4 (bottom line), B4 (middle), F5 (top), A5 (ledger above)
    let notes: Vec<(Pitch, NoteheadKind)> = vec![
        (Pitch::new(Note::C, 4), NoteheadKind::Filled),
        (Pitch::new(Note::E, 4), NoteheadKind::Filled),
        (Pitch::new(Note::B, 4), NoteheadKind::Half),
        (Pitch::new(Note::F, 5), NoteheadKind::Filled),
        (Pitch::new(Note::A, 5), NoteheadKind::Whole),
    ];

    let mut svg = SvgWriter::new(800.0, 200.0, -100.0, -500.0, 7000.0, 2500.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, staff.x + staff.staff_space, &clef, &font).unwrap();

    // Place notes with even horizontal spacing
    let start_x = 1200.0;
    let spacing = 900.0;

    for (i, (pitch, kind)) in notes.iter().enumerate() {
        let x = start_x + i as f64 * spacing;
        let pos = pitch_to_staff_position(pitch, &Clef::Treble);
        draw_note(&mut svg, &staff, &font, &config, x, pos, *kind).unwrap();
    }

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/single_notes.svg", &output).unwrap();
    println!("Wrote music-engraver/examples/output/single_notes.svg");
    println!(
        "SVG length: {} bytes, {} path elements, {} line elements",
        output.len(),
        output.matches("<path ").count(),
        output.matches("<line ").count(),
    );
}
