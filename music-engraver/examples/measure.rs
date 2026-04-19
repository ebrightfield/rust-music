/// Renders a complete measure with clef, key signature, time signature,
/// notes (with accidentals, dots, flags), a rest, and a final barline.
/// Outputs SVG to `examples/output/measure.svg`.
use music::notation::clef::Clef;

use music_engraver::font::bravura_font;
use music_engraver::layout::barline::BarlineStyle;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::key_signature::KeySignature;
use music_engraver::layout::measure::{
    layout_measure, MeasureElement, MeasureLayoutConfig, NoteEvent, RestEvent,
};
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::time_signature::TimeSignatureKind;
use music_engraver::render::{draw_measure, draw_staff_lines, SvgWriter};
use smufl::Glyph;

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();
    let ss = config.staff_space;

    let measure_config = MeasureLayoutConfig::from_staff_space(ss);

    // Build a measure: Treble clef, 2 sharps (D major), 3/4 time,
    // dotted quarter F#4, eighth A4, quarter rest — final barline.
    let elements = vec![
        MeasureElement::Clef(ClefLayout::from_clef(Clef::Treble)),
        MeasureElement::KeySignature(KeySignature::Sharps(2)),
        MeasureElement::TimeSignature(TimeSignatureKind::Numeric {
            numerator: 3,
            denominator: 4,
        }),
        // Dotted quarter note on staff position 1 (F4 in treble clef)
        MeasureElement::Note(NoteEvent {
            staff_position: 1,
            duration_log2: 2,
            dots: 1,
            accidental: None, // F# implied by key signature
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        }),
        // Eighth note on staff position 5 (A4 in treble clef) with a natural
        MeasureElement::Note(NoteEvent {
            staff_position: 5,
            duration_log2: 3,
            dots: 0,
            accidental: Some(Glyph::AccidentalNatural),
            stem_direction: None,
        tie_forward: false,
        dynamic: None,
        }),
        // Quarter rest
        MeasureElement::Rest(RestEvent {
            duration_log2: 2,
            dots: 0,
        }),
        MeasureElement::Barline(BarlineStyle::Final),
    ];

    let layout = layout_measure(&elements, &measure_config);

    // Staff spans the full measure width plus a margin
    let staff_width = layout.total_width + 2.0 * ss;
    let staff = StaffLayout::from_config(0.0, 0.0, staff_width, &config);

    let mut svg = SvgWriter::new(
        900.0,
        180.0,
        -100.0,
        -200.0 - 4.0 * ss,
        staff_width + 200.0,
        8.0 * ss + 400.0,
    );

    draw_staff_lines(&mut svg, &staff, &config);
    draw_measure(
        &mut svg,
        &staff,
        &font,
        &config,
        &layout,
        ss, // small left margin offset
        &Clef::Treble,
    )
    .unwrap();

    let output = svg.to_svg();

    std::fs::create_dir_all("music-engraver/examples/output").unwrap();
    std::fs::write("music-engraver/examples/output/measure.svg", &output).unwrap();
    println!("Wrote music-engraver/examples/output/measure.svg");

    let path_count = output.matches("<path ").count();
    let line_count = output.matches("<line ").count();
    println!(
        "SVG: {} bytes, {} <path>, {} <line>",
        output.len(),
        path_count,
        line_count,
    );

    // Expected paths:
    //   clef(1) + key sig sharps(2) + time sig digits(2)
    //   + notehead(1) + dot(1) + notehead(1) + accidental(1) + flag(1) + rest(1)
    //   = 11 paths
    // Expected lines:
    //   5 staff lines + 2 stems + 2 barline strokes (final = thin + thick)
    //   = 9 lines
    println!("Expected: 11 paths, 9 lines");
}
