/// Renders rehearsal marks (A, B, C, numbered) above a treble-clef staff
/// with notes, demonstrating boxed and plain styles.
/// Outputs SVG to `examples/output/rehearsal_marks.svg`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::rehearsal::{layout_rehearsal_mark, RehearsalStyle};
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::render::note_renderer::{draw_stemmed_note, NoteheadKind};
use music_engraver::render::rehearsal_renderer::draw_rehearsal_mark;
use music_engraver::render::{draw_clef, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 11000.0, &config);

    let clef = Clef::Treble;
    let clef_layout = ClefLayout::from_clef(Clef::Treble);

    // Notes with rehearsal marks above
    let entries: Vec<(Pitch, &str, RehearsalStyle)> = vec![
        (Pitch::new(Note::E, 4), "A", RehearsalStyle::Boxed),
        (Pitch::new(Note::G, 4), "B", RehearsalStyle::Boxed),
        (Pitch::new(Note::B, 4), "C", RehearsalStyle::Plain),
        (Pitch::new(Note::D, 5), "1", RehearsalStyle::Boxed),
        (Pitch::new(Note::F, 5), "12", RehearsalStyle::Boxed),
    ];

    // Extended viewBox to accommodate rehearsal marks above staff
    let staff_space = config.staff_space;
    let mut svg = SvgWriter::new(1100.0, 400.0, -100.0, -2500.0, 12000.0, 5000.0);

    draw_staff_lines(&mut svg, &staff, &config);
    draw_clef(
        &mut svg,
        &staff,
        staff.x + staff.staff_space,
        &clef_layout,
        &font,
    )
    .unwrap();

    let start_x = 1500.0;
    let spacing = 1800.0;

    for (i, (pitch, mark, style)) in entries.iter().enumerate() {
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

        // Center rehearsal mark on the notehead
        let note_center = x + advance / 2.0;
        let layout = layout_rehearsal_mark(mark, note_center, &staff, staff_space, *style);
        draw_rehearsal_mark(&mut svg, &layout);
    }

    let output = svg.to_svg();

    // Verify structure
    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    let text_count = output.matches("<text ").count();
    let rect_count = output.matches("<rect ").count();

    assert_eq!(text_count, 5, "5 rehearsal mark texts");
    // 4 boxed + 0 plain = 4 rects for boxes
    assert_eq!(rect_count, 4, "4 boxed rehearsal marks produce 4 rects");
    assert!(
        path_count >= 6, // 5 noteheads + 1 clef
        "expected at least 6 paths, got {path_count}"
    );
    assert_eq!(line_count, 5 + 5, "5 staff lines + 5 stems = 10 lines");

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write(
        "music-engraver/examples/output/rehearsal_marks.svg",
        &output,
    )
    .unwrap();

    println!(
        "Wrote rehearsal_marks.svg ({} bytes, {} paths, {} lines, {} texts, {} rects)",
        output.len(),
        path_count,
        line_count,
        text_count,
        rect_count,
    );
}
