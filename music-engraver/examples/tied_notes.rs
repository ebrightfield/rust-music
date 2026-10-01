/// Renders tied notes on a treble-clef staff.
/// Outputs SVG to `examples/output/tied_notes.svg`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::{auto_stem_direction, StemDirection};
use music_engraver::layout::tie::{layout_tie, tie_direction_from_stem, TieDirection};
use music_engraver::render::{
    draw_clef, draw_staff_lines, draw_stemmed_note, draw_tie, NoteheadKind, SvgWriter,
};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 16000.0, &config);

    let notehead_advance = font.glyph_advance(smufl::Glyph::NoteheadBlack).unwrap() as f64;

    let mut svg = SvgWriter::new(1600.0, 350.0, -200.0, -1500.0, 17000.0, 5000.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, &clef, &font).unwrap();

    // --- Pair 1: E4 quarter tied to E4 quarter (stems up → tie under) ---
    let p1 = Pitch::new(Note::E, 4);
    let sp1 = pitch_to_staff_position(&p1, &Clef::Treble);
    let x1a = 1800.0;
    let x1b = 3000.0;
    let dir1 = auto_stem_direction(sp1);

    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x1a,
        sp1,
        NoteheadKind::Filled,
        Some(dir1),
    )
    .unwrap();
    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x1b,
        sp1,
        NoteheadKind::Filled,
        Some(dir1),
    )
    .unwrap();

    let tie_y1 = staff.y_of(sp1);
    let tie_dir1 = tie_direction_from_stem(dir1);
    let tie_layout1 = layout_tie(x1a + notehead_advance, x1b, tie_y1, tie_dir1, &config);
    draw_tie(&mut svg, &tie_layout1);

    // --- Pair 2: B4 half tied to B4 quarter (stems down → tie over) ---
    let p2 = Pitch::new(Note::B, 4);
    let sp2 = pitch_to_staff_position(&p2, &Clef::Treble);
    let x2a = 4500.0;
    let x2b = 6500.0;
    let dir2 = auto_stem_direction(sp2);

    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x2a,
        sp2,
        NoteheadKind::Half,
        Some(dir2),
    )
    .unwrap();
    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x2b,
        sp2,
        NoteheadKind::Filled,
        Some(dir2),
    )
    .unwrap();

    let tie_y2 = staff.y_of(sp2);
    let tie_dir2 = tie_direction_from_stem(dir2);
    let tie_layout2 = layout_tie(x2a + notehead_advance, x2b, tie_y2, tie_dir2, &config);
    draw_tie(&mut svg, &tie_layout2);

    // --- Pair 3: A5 quarter tied to A5 quarter (above staff, stems down → tie over) ---
    let p3 = Pitch::new(Note::A, 5);
    let sp3 = pitch_to_staff_position(&p3, &Clef::Treble);
    let x3a = 8000.0;
    let x3b = 9500.0;
    let dir3 = auto_stem_direction(sp3);

    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x3a,
        sp3,
        NoteheadKind::Filled,
        Some(dir3),
    )
    .unwrap();
    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x3b,
        sp3,
        NoteheadKind::Filled,
        Some(dir3),
    )
    .unwrap();

    let tie_y3 = staff.y_of(sp3);
    let tie_dir3 = tie_direction_from_stem(dir3);
    let tie_layout3 = layout_tie(x3a + notehead_advance, x3b, tie_y3, tie_dir3, &config);
    draw_tie(&mut svg, &tie_layout3);

    // --- Pair 4: C4 (ledger line) tied to C4 — short tie, stems up → tie under ---
    let p4 = Pitch::new(Note::C, 4);
    let sp4 = pitch_to_staff_position(&p4, &Clef::Treble);
    let x4a = 11000.0;
    let x4b = 12200.0;
    let dir4 = auto_stem_direction(sp4);

    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x4a,
        sp4,
        NoteheadKind::Filled,
        Some(dir4),
    )
    .unwrap();
    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x4b,
        sp4,
        NoteheadKind::Filled,
        Some(dir4),
    )
    .unwrap();

    let tie_y4 = staff.y_of(sp4);
    let tie_dir4 = tie_direction_from_stem(dir4);
    let tie_layout4 = layout_tie(x4a + notehead_advance, x4b, tie_y4, tie_dir4, &config);
    draw_tie(&mut svg, &tie_layout4);

    // --- Pair 5: forced direction — G4 with explicit Over tie ---
    let p5 = Pitch::new(Note::G, 4);
    let sp5 = pitch_to_staff_position(&p5, &Clef::Treble);
    let x5a = 13500.0;
    let x5b = 14800.0;

    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x5a,
        sp5,
        NoteheadKind::Filled,
        Some(StemDirection::Up),
    )
    .unwrap();
    draw_stemmed_note(
        &mut svg,
        &staff,
        &font,
        &config,
        x5b,
        sp5,
        NoteheadKind::Filled,
        Some(StemDirection::Up),
    )
    .unwrap();

    let tie_y5 = staff.y_of(sp5);
    let tie_layout5 = layout_tie(
        x5a + notehead_advance,
        x5b,
        tie_y5,
        TieDirection::Over,
        &config,
    );
    draw_tie(&mut svg, &tie_layout5);

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/tied_notes.svg", &output).unwrap();
    println!("Wrote music-engraver/examples/output/tied_notes.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG: {} bytes, {} <path>, {} <line>",
        output.len(),
        path_count,
        line_count,
    );
}
