//! Example: render tempo markings above a staff.
//!
//! Produces `examples/output/tempo_marks.svg`.

use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::layout::tempo::*;
use music_engraver::render::note_renderer::{draw_stemmed_note, NoteheadKind};
use music_engraver::render::tempo_renderer::draw_tempo_mark;
use music_engraver::render::{draw_clef, draw_staff_lines, SvgWriter};

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let staff = StaffLayout::from_config(0.0, 0.0, 10000.0, &config);
    let staff_space = config.staff_space;

    let clef = Clef::Treble;
    let clef_layout = ClefLayout::from_clef(Clef::Treble);

    let mut svg = SvgWriter::new(800.0, 300.0, -200.0, -1200.0, 10500.0, 2500.0);
    draw_staff_lines(&mut svg, &staff, &config);
    draw_clef(
        &mut svg,
        &staff,
        staff.x + staff.staff_space,
        &clef_layout,
        &font,
    )
    .unwrap();

    // Place 4 notes with different tempo marks above them
    let notes = [
        (Note::C, 5i8, 500.0f64),
        (Note::E, 5, 3000.0),
        (Note::G, 4, 5500.0),
        (Note::D, 5, 8000.0),
    ];

    for &(note, octave, x) in &notes {
        let pitch = Pitch::new(note, octave);
        let pos = pitch_to_staff_position(&pitch, &clef);
        let dir = auto_stem_direction(pos);
        draw_stemmed_note(
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
    }

    // 1. Text-only: "Allegro" above first note
    let l1 = layout_tempo_mark(
        &TempoMark::Text("Allegro".into()),
        notes[0].2,
        &staff,
        staff_space,
    );
    draw_tempo_mark(&mut svg, &l1, &font);

    // 2. Metronome only: quarter = 120 above second note
    let l2 = layout_tempo_mark(
        &TempoMark::Metronome {
            note_kind: MetronomeNoteKind::Quarter,
            dotted: false,
            bpm: 120,
        },
        notes[1].2,
        &staff,
        staff_space,
    );
    draw_tempo_mark(&mut svg, &l2, &font);

    // 3. Combined: "Andante" dotted quarter = 72 above third note
    let l3 = layout_tempo_mark(
        &TempoMark::TextWithMetronome {
            text: "Andante".into(),
            note_kind: MetronomeNoteKind::Quarter,
            dotted: true,
            bpm: 72,
        },
        notes[2].2,
        &staff,
        staff_space,
    );
    draw_tempo_mark(&mut svg, &l3, &font);

    // 4. Eighth = 160 above fourth note
    let l4 = layout_tempo_mark(
        &TempoMark::Metronome {
            note_kind: MetronomeNoteKind::Eighth,
            dotted: false,
            bpm: 160,
        },
        notes[3].2,
        &staff,
        staff_space,
    );
    draw_tempo_mark(&mut svg, &l4, &font);

    let output = svg.to_svg();

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).expect("create output dir");
    let path = out_dir.join("tempo_marks.svg");
    std::fs::write(&path, &output).expect("write SVG");
    println!("Wrote {} bytes to {}", output.len(), path.display());

    // Validate
    assert!(output.starts_with("<svg"));
    assert!(output.contains("</svg>"));

    let path_count = output.matches("<path").count();
    let line_count = output.matches("<line").count();
    let text_count = output.matches("<text").count();
    println!(
        "SVG contains {} paths, {} lines, {} texts",
        path_count, line_count, text_count
    );

    // Verify content
    assert!(output.contains(">Allegro<"), "should contain 'Allegro'");
    assert!(output.contains("= 120"), "should contain '= 120'");
    assert!(output.contains(">Andante<"), "should contain 'Andante'");
    assert!(output.contains("= 72"), "should contain '= 72'");
    assert!(output.contains("= 160"), "should contain '= 160'");
    // Note glyphs for metronome marks (3 marks with note symbols)
    assert!(
        path_count >= 3,
        "expected at least 3 glyph paths, got {path_count}"
    );
    assert!(
        text_count >= 5,
        "expected at least 5 text elements, got {text_count}"
    );
}
