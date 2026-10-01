//! Example: old-style "church rest" multi-measure rests for small counts.
//!
//! Renders a 4-measure score showing each supported church-rest count (1–4)
//! alongside notated material. For each measure the underlying rest count is
//! shown as a bold number above the staff, while the body of the measure
//! uses combinations of whole + breve SMuFL rest glyphs:
//!
//! - count 1: a single whole rest hanging from the 4th line.
//! - count 2: a single breve (double-whole) rest sitting on the 3rd line.
//! - count 3: a breve + a whole rest, side by side.
//! - count 4: two breve rests side by side.
//!
//! Produces `examples/output/church_rest_score.svg`.

use music::notation::clef::Clef;
use music::notation::rhythm::duration::Duration;
use music::note::note::Note;
use music::note::pitch::Pitch;
use music_engraver::score::ScoreBuilder;

fn n(note: Note, octave: i8) -> Pitch {
    Pitch::new(note, octave)
}

fn main() {
    let svg = ScoreBuilder::new()
        .clef(Clef::Treble)
        .time_signature(4, 4)
        .measures_per_system(2)
        // ── m1: a single whole-note pickup so the part doesn't start on a rest.
        .note(n(Note::C, 5), Duration::WHOLE)
        .barline()
        // ── m2: church rest, count 1 (single whole rest).
        .multi_measure_rest_church(1)
        .barline()
        // ── m3: church rest, count 2 (single breve rest).
        .multi_measure_rest_church(2)
        .barline()
        // ── m4: a quarter-note line, then a church rest of 3 in the next bar.
        .note(n(Note::D, 5), Duration::QTR)
        .note(n(Note::E, 5), Duration::QTR)
        .note(n(Note::F, 5), Duration::QTR)
        .note(n(Note::G, 5), Duration::QTR)
        .barline()
        // ── m5: church rest, count 3 (breve + whole).
        .multi_measure_rest_church(3)
        .barline()
        // ── m6: church rest, count 4 (two breves).
        .multi_measure_rest_church(4)
        .barline()
        // ── m7: church rest, count 7 — falls back to H-bar form.
        .multi_measure_rest_church(7)
        .barline()
        // ── m8: a closing whole note so the part has a final pitch.
        .note(n(Note::C, 5), Duration::WHOLE)
        .end_barline()
        .render_svg();

    std::fs::create_dir_all("examples/output").unwrap();
    std::fs::write("examples/output/church_rest_score.svg", &svg).unwrap();
    println!(
        "Wrote examples/output/church_rest_score.svg ({} bytes)",
        svg.len()
    );

    assert!(svg.starts_with("<svg"));

    // We expect the small-count church rests to each contribute at least one
    // <path> element (the SMuFL rest glyph) — but no <rect> elements (those
    // are only used by the H-bar form). Count the bold count-number texts.
    let bold_texts = svg.matches("font-weight=\"bold\"").count();
    println!("bold-weighted text elements (count numbers): {bold_texts}");

    // The church-rest measures (count 1, 2, 3, 4) plus the H-bar fallback
    // for count 7 each produce one bold count number, so we expect at least
    // 5 bold-weighted text elements in the rendered SVG.
    assert!(
        bold_texts >= 5,
        "expected at least 5 bold count texts (4 church + 1 H-bar fallback), got {bold_texts}"
    );

    // Each of "1", "2", "3", "4", "7" must appear in a <text> node.
    for n in &["1", "2", "3", "4", "7"] {
        let needle = format!(">{n}</text>");
        assert!(
            svg.contains(&needle),
            "expected count text {needle:?} in SVG",
        );
    }

    // The count-7 measure uses the H-bar fallback, which emits 3 <rect>
    // elements. The 4 church-rest measures emit no rects. So we expect at
    // least 3 rects (plus possibly background/system rects, if any).
    let rects = svg.matches("<rect").count();
    println!("rect count (≥3 expected from count-7 H-bar fallback): {rects}");
    assert!(
        rects >= 3,
        "expected ≥3 rects from H-bar fallback, got {rects}"
    );
}
