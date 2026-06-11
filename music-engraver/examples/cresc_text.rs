/// Renders dashed-text dynamic markings ("cresc. - - -", "decresc. - - -",
/// "dim. - - -") below treble-clef staves with notes. Demonstrates:
///   1. plain `layout_cresc_text` with the italic label + dashed continuation,
///      one staff per kind so the three labels are visually side-by-side,
///   2. `layout_cresc_text_continuation` (label suppressed) on a fourth staff,
///      mirroring the cross-system continuation convention used by hairpins
///      and trill extensions.
///
/// Outputs SVG to `examples/output/cresc_text.svg`.
///
/// This example exercises the layout/renderer pair directly because the
/// dashed-text marking has not yet been plumbed through `ScoreBuilder`. A
/// follow-up will wire `cresc.` / `decresc.` / `dim.` builder methods so
/// callers can request a dashed-text marking instead of a hairpin.
use music::notation::clef::Clef;
use music::note::note::Note;
use music::note::pitch::Pitch;

use music_engraver::font::bravura_font;
use music_engraver::layout::clef::ClefLayout;
use music_engraver::layout::cresc_text::{
    layout_cresc_text, layout_cresc_text_continuation, CrescTextKind,
};
use music_engraver::layout::note_placement::pitch_to_staff_position;
use music_engraver::layout::staff::StaffLayout;
use music_engraver::layout::stem::auto_stem_direction;
use music_engraver::render::cresc_text_renderer::draw_cresc_text;
use music_engraver::render::note_renderer::{draw_stemmed_note, NoteheadKind};
use music_engraver::render::{draw_clef, draw_staff_lines, SvgWriter};

/// Vertical spacing between successive staves in font design units.
/// Four staves of music need to clear staff height + below-staff dashed-text
/// band + headroom for the next staff's clef. 4500 FU ≈ 18 staff spaces
/// gives roughly 2 SS of breathing room above each label band.
const ROW_SPACING_FU: f64 = 4500.0;

fn p(note: Note, oct: i8) -> Pitch {
    Pitch::new(note, oct)
}

fn main() {
    let font = bravura_font();
    let config = font.engraving_config();

    // Each row's y_origin slides down by ROW_SPACING_FU. The page viewBox
    // height is set so the bottom row's dashed text fits comfortably.
    let row_ys: [f64; 4] = [0.0, ROW_SPACING_FU, 2.0 * ROW_SPACING_FU, 3.0 * ROW_SPACING_FU];

    let clef = Clef::Treble;
    let clef_layout = ClefLayout::from_clef(Clef::Treble);

    // The same 8-note phrase per row — visually identical staves so the only
    // thing varying between rows is the dashed-text marking.
    let pitches: [Pitch; 8] = [
        p(Note::E, 4),
        p(Note::G, 4),
        p(Note::A, 4),
        p(Note::B, 4),
        p(Note::D, 5),
        p(Note::E, 5),
        p(Note::D, 5),
        p(Note::B, 4),
    ];

    let start_x = 1200.0;
    let spacing = 1000.0;

    // SVG viewBox: width covers clef + 8 notes + right padding;
    // height covers four rows of staff + dashed-text band.
    let viewbox_h = 4.0 * ROW_SPACING_FU + 2000.0;
    let mut svg = SvgWriter::new(1200.0, 600.0, -100.0, -500.0, 11000.0, viewbox_h);

    // Row layouts: one CrescTextKind each on rows 0–2, a continuation on row 3.
    let kinds = [
        Some(CrescTextKind::Crescendo),
        Some(CrescTextKind::Decrescendo),
        Some(CrescTextKind::Diminuendo),
        None, // row 3 uses the continuation (label-suppressed) variant
    ];

    let mut total_cresc_lines = 0usize;
    let mut total_label_text_elements = 0usize;

    for (row_idx, &kind_opt) in kinds.iter().enumerate() {
        let y_origin = row_ys[row_idx];
        let staff = StaffLayout::from_config(0.0, y_origin, 10000.0, &config);

        draw_staff_lines(&mut svg, &staff, &config);
        draw_clef(&mut svg, &staff, &clef_layout, &font).unwrap();

        let mut note_xs: Vec<f64> = Vec::with_capacity(8);
        let mut note_advances: Vec<f64> = Vec::with_capacity(8);
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

        // Span the marking from note 1 to just past note 7 (the last note),
        // so the dashed line covers most of the staff width on every row.
        let x_start = note_xs[1];
        let x_end = note_xs[7] + note_advances[7];

        match kind_opt {
            Some(kind) => {
                let layout = layout_cresc_text(kind, x_start, x_end, &staff, staff.staff_space);
                // The plain layout sets has_label = true → renderer emits the
                // italic <text> label AND the dashed continuation line.
                assert!(layout.has_label, "plain layout must emit the label");
                draw_cresc_text(&mut svg, &layout);
                total_label_text_elements += 1;
                total_cresc_lines += 1;
            }
            None => {
                let layout = layout_cresc_text_continuation(
                    CrescTextKind::Crescendo,
                    x_start,
                    x_end,
                    &staff,
                    staff.staff_space,
                );
                // Continuation suppresses the label → renderer emits ONLY the
                // dashed line, mirroring the cross-system convention.
                assert!(!layout.has_label, "continuation must suppress the label");
                draw_cresc_text(&mut svg, &layout);
                total_cresc_lines += 1;
            }
        }
    }

    let output = svg.to_svg();

    // ---- Structural assertions (so a regression that broke this example's
    //      rendered output would surface as a `cargo run --example` failure). ----

    // 4 rows × (1 clef glyph + 8 noteheads) = 36 <path>.
    let path_count = output.matches("<path ").count();
    assert!(
        path_count >= 36,
        "expected at least 36 <path> elements (4 clefs + 32 noteheads), got {path_count}"
    );

    // <line> elements: 4 rows × (5 staff lines + 8 stems) = 52 minimum,
    // plus one dashed continuation line per row = 4 → total ≥ 56.
    let line_count = output.matches("<line ").count();
    assert!(
        line_count >= 56,
        "expected at least 56 <line> elements (staff lines + stems + 4 dashed-text lines), got {line_count}"
    );

    // Exactly four dashed lines from the cresc_text renderer (one per row).
    // Any other line emitter in this example uses solid strokes only.
    let dasharray_count = output.matches("stroke-dasharray").count();
    assert_eq!(
        dasharray_count, 4,
        "expected exactly 4 dashed-text continuation lines (one per row), got {dasharray_count}"
    );
    assert_eq!(
        total_cresc_lines, 4,
        "expected exactly 4 cresc-text invocations (3 plain + 1 continuation)"
    );
    assert_eq!(
        total_label_text_elements, 3,
        "expected 3 label-emitting layouts (one per plain kind)"
    );

    // Each plain kind's italic label must appear as text content.
    assert!(
        output.contains(">cresc.</text>"),
        "expected '>cresc.</text>' in output"
    );
    assert!(
        output.contains(">decresc.</text>"),
        "expected '>decresc.</text>' in output"
    );
    assert!(
        output.contains(">dim.</text>"),
        "expected '>dim.</text>' in output"
    );

    // Italic styling on the labels.
    assert!(
        output.contains("font-style=\"italic\""),
        "expected italic font-style on labels"
    );

    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/output");
    std::fs::create_dir_all(&out_dir).unwrap();
    let path = out_dir.join("cresc_text.svg");
    std::fs::write(&path, &output).unwrap();

    println!(
        "Wrote {} ({} bytes, {} paths, {} lines, {} dashed)",
        path.display(),
        output.len(),
        path_count,
        line_count,
        dasharray_count
    );
}
