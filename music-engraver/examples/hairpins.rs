/// Renders hairpins (crescendo/decrescendo wedges) below a treble-clef staff
/// with notes, demonstrating placement below the staff alongside dynamics.
/// Outputs SVG to `examples/output/hairpins.svg`.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::dynamics::Dynamic;
use music_engraver::layout::hairpin::{layout_hairpin, HairpinType};
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::render::dynamics_renderer::draw_dynamic;
use music_engraver::render::hairpin_renderer::draw_hairpin;
use music_engraver::render::note_renderer::{draw_stemmed_note, NoteheadKind};
use music_engraver::render::{draw_clef, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 10000.0, &config);

    let clef = Clef::Treble;
    let clef_layout = ClefLayout::from_clef(Clef::Treble);

    // Notes: p — cresc — — — f — decresc — — pp
    let pitches: Vec<Pitch> = vec![
        Pitch::new(Note::E, 4).unwrap(),
        Pitch::new(Note::G, 4).unwrap(),
        Pitch::new(Note::A, 4).unwrap(),
        Pitch::new(Note::B, 4).unwrap(),
        Pitch::new(Note::D, 5).unwrap(),
        Pitch::new(Note::E, 5).unwrap(),
        Pitch::new(Note::D, 5).unwrap(),
        Pitch::new(Note::B, 4).unwrap(),
    ];

    let mut svg = SvgWriter::new(1200.0, 400.0, -100.0, -1200.0, 11000.0, 5000.0);

    draw_staff_lines(&mut svg, &staff, &config);
    draw_clef(&mut svg, &staff, &clef_layout, &font).unwrap();

    let start_x = 1200.0;
    let spacing = 1000.0;
    let mut note_xs: Vec<f64> = Vec::new();
    let mut note_advances: Vec<f64> = Vec::new();

    for (i, pitch) in pitches.iter().enumerate() {
        let x = start_x + i as f64 * spacing;
        let pos = pitch_to_staff_position(pitch, &clef);
        let dir = auto_stem_direction(pos);
        let advance = draw_stemmed_note(
            &mut svg, &staff, &font, &config, x, pos, NoteheadKind::Filled, Some(dir),
        )
        .unwrap();
        note_xs.push(x);
        note_advances.push(advance);
    }

    // Dynamic: p under first note
    let center0 = note_xs[0] + note_advances[0] / 2.0;
    draw_dynamic(&mut svg, &staff, &font, Dynamic::Piano, center0).unwrap();

    // Crescendo hairpin from note 1 to note 3
    let cresc_layout = layout_hairpin(
        HairpinType::Crescendo,
        note_xs[1],
        note_xs[3] + note_advances[3],
        staff.bottom_y(),
        staff.staff_space,
        config.staff_line_thickness_fu(),
    );
    draw_hairpin(&mut svg, &cresc_layout);

    // Dynamic: f under note 4
    let center4 = note_xs[4] + note_advances[4] / 2.0;
    draw_dynamic(&mut svg, &staff, &font, Dynamic::Forte, center4).unwrap();

    // Decrescendo hairpin from note 5 to note 7
    let decresc_layout = layout_hairpin(
        HairpinType::Decrescendo,
        note_xs[5],
        note_xs[7] + note_advances[7],
        staff.bottom_y(),
        staff.staff_space,
        config.staff_line_thickness_fu(),
    );
    draw_hairpin(&mut svg, &decresc_layout);

    // Dynamic: pp under last note
    let center7 = note_xs[7] + note_advances[7] / 2.0;
    draw_dynamic(&mut svg, &staff, &font, Dynamic::Pp, center7).unwrap();

    let output = svg.to_svg();

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    // 8 noteheads + 1 clef + 3 dynamics = 12 paths
    // 5 staff lines + 8 stems + 4 hairpin lines = 17 lines
    assert!(
        path_count >= 12,
        "expected at least 12 paths, got {path_count}"
    );
    assert!(
        line_count >= 17,
        "expected at least 17 lines, got {line_count}"
    );

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/hairpins.svg", &output).unwrap();

    println!(
        "Wrote hairpins.svg ({} bytes, {} paths, {} lines)",
        output.len(),
        path_count,
        line_count,
    );
}
