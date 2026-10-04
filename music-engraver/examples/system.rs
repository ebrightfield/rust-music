use music::notation::clef::Clef;
use music_engraver::font::bravura_font;
use music_engraver::layout::barline::BarlineStyle;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::measure::{MeasureLayoutConfig, NoteAnnotations, NoteEvent, RestEvent};
use music_engraver::layout::system::{layout_system, MeasureContent, MeasureEvent, SystemPrefix};
use music_engraver::layout::time_signature::TimeSignatureKind;
use music_engraver::render::system_renderer::draw_system;
use music_engraver::render::SvgWriter;

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let measure_cfg = MeasureLayoutConfig::from_staff_space(config.staff_space);

    let prefix = SystemPrefix::new(
        &Clef::Treble,
        KeySignature::Sharps(2),
        Some(TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        }),
    );

    let measures = vec![
        // Measure 1: D4 quarter, F#4 quarter, A4 quarter, D5 quarter
        MeasureContent {
            events: vec![
                MeasureEvent::Note(NoteEvent {
                    staff_position: -1, // D4 in treble
                    duration_log2: 2,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Note(NoteEvent {
                    staff_position: 1, // F#4
                    duration_log2: 2,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Note(NoteEvent {
                    staff_position: 3, // A4
                    duration_log2: 2,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Note(NoteEvent {
                    staff_position: 6, // D5
                    duration_log2: 2,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
            ],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 2: dotted half G4, quarter rest
        MeasureContent {
            events: vec![
                MeasureEvent::Note(NoteEvent {
                    staff_position: 2, // G4
                    duration_log2: 1,  // half note
                    dots: 1,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Rest(RestEvent {
                    duration_log2: 2,
                    dots: 0,
                    annotations: Default::default(),
                }),
            ],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 3: E4 eighth, F#4 eighth, G4 eighth, A4 eighth, B4 half
        MeasureContent {
            events: vec![
                MeasureEvent::Note(NoteEvent {
                    staff_position: 0, // E4
                    duration_log2: 3,  // eighth
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Note(NoteEvent {
                    staff_position: 1, // F#4
                    duration_log2: 3,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Note(NoteEvent {
                    staff_position: 2, // G4
                    duration_log2: 3,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Note(NoteEvent {
                    staff_position: 3, // A4
                    duration_log2: 3,
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
                MeasureEvent::Note(NoteEvent {
                    staff_position: 4, // B4
                    duration_log2: 1,  // half
                    dots: 0,
                    accidental: None,
                    stem_direction: None,
                    annotations: NoteAnnotations::default(),
                }),
            ],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    // Layout with a target width to demonstrate justification
    let target_width = 10000.0;
    let system = layout_system(&prefix, &measures, &measure_cfg, Some(target_width));

    let mut svg = SvgWriter::new(1200.0, 200.0, -200.0, -200.0, 11000.0, 2000.0);
    draw_system(&mut svg, &font, &config, &system, 0.0, 0.0).unwrap();

    let output = svg.to_svg();

    // Write output
    std::fs::create_dir_all("music-engraver/examples/output").ok();
    std::fs::write("music-engraver/examples/output/system.svg", &output).unwrap();

    // Verify output
    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!("System SVG: {} paths, {} lines", path_count, line_count);
    println!(
        "System has {} measures, total width: {:.0}",
        system.measures.len(),
        system.total_width
    );
    for (i, m) in system.measures.iter().enumerate() {
        println!(
            "  Measure {}: x_offset={:.0}, width={:.0}, {} elements",
            i + 1,
            m.x_offset,
            m.layout.total_width,
            m.layout.elements.len()
        );
    }

    assert!(output.starts_with("<svg"));
    assert!(output.contains("</svg>"));
    assert!(path_count > 0, "should have paths");
    assert!(line_count >= 5, "should have at least 5 staff lines");
}
