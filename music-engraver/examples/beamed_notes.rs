/// Renders beamed note groups on a treble-clef staff.
/// Outputs SVG to `examples/output/beamed_notes.svg`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::beam::{beam_group_stem_direction, layout_beam_group, BeamedNote};
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::render::{
    draw_beam_group, draw_clef, draw_note, draw_staff_lines, NoteheadKind, SvgWriter,
};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 14000.0, &config);

    let notehead_advance = font
        .glyph_advance(smufl::Glyph::NoteheadBlack)
        .unwrap() as f64;

    let mut svg = SvgWriter::new(1400.0, 350.0, -200.0, -1500.0, 15000.0, 5000.0);

    draw_staff_lines(&mut svg, &staff, &config);

    let clef = ClefLayout::from_clef(Clef::Treble);
    draw_clef(&mut svg, &staff, &clef, &font).unwrap();

    // --- Group 1: four ascending eighth notes (stems up) ---
    let group1_pitches = [
        Pitch::new(Note::C, 4), // middle C (ledger line)
        Pitch::new(Note::D, 4),
        Pitch::new(Note::E, 4),
        Pitch::new(Note::F, 4),
    ];
    let group1_start_x = 1500.0;
    let group1_spacing = 400.0;

    let group1_notes: Vec<BeamedNote> = group1_pitches
        .iter()
        .enumerate()
        .map(|(i, p)| BeamedNote {
            x: group1_start_x + i as f64 * group1_spacing,
            staff_position: pitch_to_staff_position(p, &Clef::Treble),
            duration_log2: 3, // eighth notes
        })
        .collect();

    // Draw noteheads + ledger lines
    for note in &group1_notes {
        draw_note(
            &mut svg, &staff, &font, &config, note.x, note.staff_position,
            NoteheadKind::Filled,
        )
        .unwrap();
    }

    let dir1 = beam_group_stem_direction(&group1_notes);
    let layout1 = layout_beam_group(&group1_notes, dir1, staff.staff_space);
    draw_beam_group(&mut svg, &staff, &config, &group1_notes, &layout1, notehead_advance);

    // --- Group 2: two sixteenth notes descending (stems down) ---
    let group2_pitches = [
        Pitch::new(Note::B, 4),
        Pitch::new(Note::A, 4),
    ];
    let group2_start_x = 4000.0;

    let group2_notes: Vec<BeamedNote> = group2_pitches
        .iter()
        .enumerate()
        .map(|(i, p)| BeamedNote {
            x: group2_start_x + i as f64 * 400.0,
            staff_position: pitch_to_staff_position(p, &Clef::Treble),
            duration_log2: 4, // sixteenth notes
        })
        .collect();

    for note in &group2_notes {
        draw_note(
            &mut svg, &staff, &font, &config, note.x, note.staff_position,
            NoteheadKind::Filled,
        )
        .unwrap();
    }

    let dir2 = beam_group_stem_direction(&group2_notes);
    let layout2 = layout_beam_group(&group2_notes, dir2, staff.staff_space);
    draw_beam_group(&mut svg, &staff, &config, &group2_notes, &layout2, notehead_advance);

    // --- Group 3: three notes with mixed durations (eighth + two sixteenths, stems up) ---
    let group3_pitches = [
        Pitch::new(Note::E, 4),
        Pitch::new(Note::G, 4),
        Pitch::new(Note::A, 4),
    ];
    let group3_start_x = 5500.0;
    let group3_durs: [u8; 3] = [3, 4, 4]; // eighth, 16th, 16th

    let group3_notes: Vec<BeamedNote> = group3_pitches
        .iter()
        .zip(group3_durs.iter())
        .enumerate()
        .map(|(i, (p, &dur))| BeamedNote {
            x: group3_start_x + i as f64 * 400.0,
            staff_position: pitch_to_staff_position(p, &Clef::Treble),
            duration_log2: dur,
        })
        .collect();

    for note in &group3_notes {
        draw_note(
            &mut svg, &staff, &font, &config, note.x, note.staff_position,
            NoteheadKind::Filled,
        )
        .unwrap();
    }

    let dir3 = beam_group_stem_direction(&group3_notes);
    let layout3 = layout_beam_group(&group3_notes, dir3, staff.staff_space);
    draw_beam_group(&mut svg, &staff, &config, &group3_notes, &layout3, notehead_advance);

    // --- Group 4: high notes stems down (F5, E5, D5), 32nd notes ---
    let group4_pitches = [
        Pitch::new(Note::F, 5),
        Pitch::new(Note::E, 5),
        Pitch::new(Note::D, 5),
    ];
    let group4_start_x = 7500.0;

    let group4_notes: Vec<BeamedNote> = group4_pitches
        .iter()
        .enumerate()
        .map(|(i, p)| BeamedNote {
            x: group4_start_x + i as f64 * 350.0,
            staff_position: pitch_to_staff_position(p, &Clef::Treble),
            duration_log2: 5, // 32nd notes
        })
        .collect();

    for note in &group4_notes {
        draw_note(
            &mut svg, &staff, &font, &config, note.x, note.staff_position,
            NoteheadKind::Filled,
        )
        .unwrap();
    }

    let dir4 = beam_group_stem_direction(&group4_notes);
    let layout4 = layout_beam_group(&group4_notes, dir4, staff.staff_space);
    draw_beam_group(&mut svg, &staff, &config, &group4_notes, &layout4, notehead_advance);

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write(
        "music-engraver/examples/output/beamed_notes.svg",
        &output,
    )
    .unwrap();
    println!("Wrote music-engraver/examples/output/beamed_notes.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    let polygon_count = output.matches("<polygon ").count();
    println!(
        "SVG: {} bytes, {} <path>, {} <line>, {} <polygon>",
        output.len(), path_count, line_count, polygon_count,
    );
}
