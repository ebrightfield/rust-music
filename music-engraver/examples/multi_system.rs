use music::notation::clef::Clef;
use music_engraver::font::bravura_font;
use music_engraver::layout::barline::BarlineStyle;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::measure::{MeasureLayoutConfig, NoteAnnotations, NoteEvent, RestEvent};
use music_engraver::layout::page::{layout_page, PageLayoutConfig, SystemBreaking};
use music_engraver::layout::system::{MeasureContent, MeasureEvent, SystemPrefix};
use music_engraver::layout::time_signature::TimeSignatureKind;
use music_engraver::render::page_renderer::draw_page;

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let ss = config.staff_space;

    let prefix = SystemPrefix::new(
        &Clef::Treble,
        KeySignature::Sharps(2),
        Some(TimeSignatureKind::Numeric {
            numerator: 4,
            denominator: 4,
        }),
    );

    let mcfg = MeasureLayoutConfig::from_staff_space(ss);

    // 8 measures of simple melody
    let measures = vec![
        // Measure 1: D4 E4 F#4 G4 (ascending quarter notes)
        MeasureContent {
            events: vec![
                quarter(1), // D4 in treble
                quarter(2), // E4
                quarter(3), // F#4
                quarter(4), // G4
            ],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 2: A4 B4 half
        MeasureContent {
            events: vec![
                quarter(5), // A4
                quarter(6), // B4
                half(7),    // C5
            ],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 3: D5 whole
        MeasureContent {
            events: vec![whole(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 4: quarter rest, three quarters descending
        MeasureContent {
            events: vec![qrest(), quarter(6), quarter(4), quarter(2)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 5: two halves
        MeasureContent {
            events: vec![half(0), half(4)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 6: dotted half + quarter
        MeasureContent {
            events: vec![dotted_half(5), quarter(3)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 7: four quarters
        MeasureContent {
            events: vec![quarter(4), quarter(2), quarter(6), quarter(8)],
            barline: BarlineStyle::Single,
            volta: None,
            additional_voices: vec![],
        },
        // Measure 8: whole note (final)
        MeasureContent {
            events: vec![whole(4)],
            barline: BarlineStyle::Final,
            volta: None,
            additional_voices: vec![],
        },
    ];

    let page_cfg = PageLayoutConfig::new(ss, 10000.0);

    // Break into 2 systems of 4 measures each
    let page = layout_page(
        &prefix,
        &measures,
        &mcfg,
        &page_cfg,
        &SystemBreaking::Fixed(4),
    );

    let svg = draw_page(&font, &config, &page).expect("render failed");
    let output = svg.to_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/multi_system.svg", &output).unwrap();

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    let systems = page.systems.len();
    println!("Wrote examples/output/multi_system.svg");
    println!(
        "  {systems} systems, {path_count} paths, {line_count} lines, {} bytes",
        output.len()
    );

    assert!(output.starts_with("<svg"));
    assert!(output.contains("</svg>"));
    assert_eq!(systems, 2);
    // 2 systems × 5 staff lines = 10 minimum
    assert!(line_count >= 10, "expected >= 10 lines, got {line_count}");
    // Clef paths: 2 (one per system), plus noteheads, time sig digits, key sig accidentals
    assert!(path_count >= 4, "expected >= 4 paths, got {path_count}");
}

fn quarter(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 2,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })
}

fn half(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 1,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })
}

fn whole(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 0,
        dots: 0,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })
}

fn dotted_half(pos: i8) -> MeasureEvent {
    MeasureEvent::Note(NoteEvent {
        staff_position: pos,
        duration_log2: 1,
        dots: 1,
        accidental: None,
        stem_direction: None,
        annotations: NoteAnnotations::default(),
    })
}

fn qrest() -> MeasureEvent {
    MeasureEvent::Rest(RestEvent {
        duration_log2: 2,
        dots: 0,
    })
}
